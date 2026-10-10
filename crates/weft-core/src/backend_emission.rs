//! Validate backend metadata against the resolved logical outputs.
use crate::{
    backend::{Column, Emission, Plan, Representation, ScalarCarrier, ScalarDecoder, Selection},
    error::{Diagnostic, Result},
    ir::{Family, Identity, LogicalType, Node},
};
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-EMIT", "emit", message)
}
enum Expected<'a> {
    Scalar(&'a LogicalType),
    Arithmetic(&'a crate::arithmetic_resolve::Domain),
    Field(&'a Identity),
    Related(&'a crate::application_model::RelationshipRead, u16),
}
fn scalar(column: &Column, expected: &LogicalType) -> bool {
    match &column.representation {
        Representation::Scalar {
            logical_type,
            carrier,
            decoder,
        } => {
            logical_type == expected
                && column.nullable == expected.nullable
                && matches!(
                    (&expected.family, carrier, decoder),
                    (Family::String, ScalarCarrier::Text, ScalarDecoder::Text)
                        | (
                            Family::Boolean,
                            ScalarCarrier::Boolean,
                            ScalarDecoder::Boolean
                        )
                        | (Family::Boolean, ScalarCarrier::Text, ScalarDecoder::Boolean)
                        | (
                            Family::Integer,
                            ScalarCarrier::Text,
                            ScalarDecoder::ExactInteger
                        )
                        | (
                            Family::Decimal,
                            ScalarCarrier::Text,
                            ScalarDecoder::ExactDecimal
                        )
                )
        }
        _ => false,
    }
}
pub(crate) fn validate(
    plan: Plan<'_>,
    emission: &Emission,
    selection: &Selection,
    assessments: &[crate::backend::Assessment],
) -> Result<()> {
    if emission.sql.trim().is_empty()
        || emission.sql.len() > 1024 * 1024
        || emission.sql.contains('\0')
        || emission.parameters.len() > 1024
    {
        return Err(fail("Emission SQL or parameter bounds are invalid"));
    }
    for (index, p) in emission.parameters.iter().enumerate() {
        if p.position != index + 1 || !p.origin.is_object() || p.logical_type.nullable {
            return Err(fail(
                "Parameter slots require contiguous positions, typed origins and non-null values",
            ));
        }
        let unbounded=p.logical_type.family==Family::Integer && p.logical_type.facets==serde_json::json!({});
        let admitted=assessments.iter().any(|a| (a.id=="type.integer.unbounded" || (matches!(plan,Plan::V03(_)) && (a.id=="arithmetic.exact.integer" || a.id=="aggregate.havingCountDistinctGreater"))) && a.status!=crate::backend::Status::Unsupported);
        if unbounded && !admitted { return Err(fail("Unbounded integer parameter requires explicit capability admission")); }
        if p.logical_type.family == Family::Decimal && p.logical_type.facets.get("precision").is_none() {
            let admitted = matches!(plan, Plan::V03(_)) && assessments.iter().any(|a| a.id == "arithmetic.exact.decimal" && a.status != crate::backend::Status::Unsupported);
            let scale = p.logical_type.facets["scale"].as_u64().ok_or_else(|| fail("Arithmetic decimal parameter requires its exact lexical scale"))?;
            if !admitted || p.logical_type.facets != serde_json::json!({"scale":scale}) {
                return Err(fail("Arithmetic decimal parameter requires explicit 0.3 capability admission"));
            }
            let domain = crate::arithmetic_resolve::token_domain(&p.value, Some(&Family::Decimal), &crate::ir::Span {start:0,end:0})
                .map_err(|_| fail("Arithmetic decimal parameter is not exact base-ten text"))?;
            if domain != (crate::arithmetic_resolve::Domain::Decimal {scale}) {
                return Err(fail("Arithmetic decimal parameter changes its original lexical scale"));
            }
        } else {
            validate_parameter_domain(&p.value, &p.logical_type, admitted)?;
        }
    }
    let expected: Vec<(&str, Expected<'_>)> = match plan {
        Plan::V01(p) => {
            let Node::Project { outputs, .. } = &p.root else {
                return Err(fail("Resolved 0.1 plan needs a projection root"));
            };
            outputs
                .iter()
                .map(|o| {
                    (
                        o.name.as_str(),
                        Expected::Scalar(o.expression.logical_type()),
                    )
                })
                .collect()
        }
        Plan::V03(p) => p.outputs.iter().map(|o|(o.name.as_str(),match &o.expression {
            crate::arithmetic_plan::Expression::Field{identity,..}=>Expected::Field(identity),
            crate::arithmetic_plan::Expression::CountDistinct{logical_type,..}|crate::arithmetic_plan::Expression::Count{logical_type}|crate::arithmetic_plan::Expression::Sum{logical_type,..}=>Expected::Scalar(logical_type),
            crate::arithmetic_plan::Expression::RelatedKeys{relationship,bound,..}=>Expected::Related(relationship,*bound),
            crate::arithmetic_plan::Expression::Arithmetic{expression}=>Expected::Arithmetic(&expression.domain),
        })).collect(),
        Plan::V02(p) => p
            .outputs
            .iter()
            .map(|o| {
                (
                    o.name.as_str(),
                    match &o.expression {
                        crate::application_ir::Expression::Field { identity, .. } => {
                            Expected::Field(identity)
                        }
                        crate::application_ir::Expression::Count { logical_type }
                        | crate::application_ir::Expression::Sum { logical_type, .. } => {
                            Expected::Scalar(logical_type)
                        }
                        crate::application_ir::Expression::RelatedKeys {
                            relationship,
                            bound,
                            ..
                        } => Expected::Related(relationship, *bound),
                    },
                )
            })
            .collect(),
    };
    if emission.columns.len() != expected.len() {
        return Err(fail(
            "Emission must describe every logical output exactly once",
        ));
    }
    let positioned = matches!(plan, Plan::V03(p) if p.required_capabilities.iter().any(|id|id=="project.positionedOutputs"));
    let repeated = expected.iter().enumerate().any(|(i,(name,_))|expected[..i].iter().any(|(other,_)|other==name));
    if matches!(plan, Plan::V03(_)) && positioned != repeated {return Err(fail("Positional output admission must match repeated logical labels"));}
    if positioned && !assessments.iter().any(|a|a.id=="project.positionedOutputs"&&a.status!=crate::backend::Status::Unsupported) {return Err(fail("Positional result labels require explicit capability admission"));}
    if positioned {
        let map=serde_json::json!(emission.columns.iter().map(|c|serde_json::json!({"position":c.position,"outputName":c.output_name,"carrierName":c.carrier_name,"sourceIdentities":c.source_identities})).collect::<Vec<_>>());
        if !emission.obligations.iter().any(|o|o.id=="weft.output.positioned" && o.owner==crate::backend::ObligationOwner::Host && o.failure_code=="WFT-OBLIGATION" && o.parameters["profile"]=="weft-positioned-output/0.3.0" && o.parameters["columns"]==map) {return Err(fail("Positional output requires its exact ordered host custody obligation"));}
    }
    let mut carriers=std::collections::BTreeSet::new();
    for (index, (column, (name, expected))) in emission.columns.iter().zip(expected).enumerate() {
        if positioned {
            let carrier=column.carrier_name.as_deref().ok_or_else(||fail("Positional outputs require every physical result name"))?;
            if carrier.is_empty()||carrier.len()>128||carrier.contains('\0')||!carriers.insert(carrier) {return Err(fail("Physical result names must be bounded, nonempty and unique"));}
            if let Expected::Field(identity)=&expected {if column.source_identities.as_slice()!=std::slice::from_ref(*identity) {return Err(fail("Positional Field output changes original ordinal lineage"));}}
        } else if column.carrier_name.is_some() {return Err(fail("Physical result names require explicit 0.3 positional admission"));}
        if column.position != index + 1
            || column.output_name != name
            || column.source_identities.is_empty()
            || column.source_identities.iter().any(|id| {
                !selection.fields.contains(id)
                    && !selection.records.contains(id)
                    && !selection.types.contains(id)
            })
        {
            return Err(fail(
                "Result columns must retain output order, names and selected source identities",
            ));
        }
        let valid = match expected {
            Expected::Scalar(t) => scalar(column, t),
            Expected::Arithmetic(domain) => {
                let (family,facets)=match domain {crate::arithmetic_resolve::Domain::Integer=>(Family::Integer,serde_json::json!({})),crate::arithmetic_resolve::Domain::Decimal{scale}=>(Family::Decimal,serde_json::json!({"scale":scale}))};
                scalar(column,&LogicalType{family,facets,nullable:false})
            },
            Expected::Field(identity) => {
                let graph=match plan {Plan::V02(p)=>&p.type_graph,Plan::V03(p)=>&p.type_graph,Plan::V01(_)=>unreachable!()};
                let descriptor = graph
                    .iter()
                    .find(|d| &d.identity == identity)
                    .ok_or_else(|| fail("Projected Field lacks its type descriptor"))?;
                match (&descriptor.shape, &column.representation) {
                    (
                        crate::application_model::Shape::Scalar { logical_type },
                        Representation::Scalar { .. },
                    ) if descriptor.availability.as_deref() == Some("required") => {
                        scalar(column, logical_type)
                    }
                    (
                        _,
                        Representation::Value {
                            descriptor,
                            native_null,
                        },
                    ) => {
                        descriptor == identity
                            && !column.nullable
                            && (!native_null
                                || assessments.iter().any(|a| {
                                    a.id == "value.nativeNull"
                                        && a.status != crate::backend::Status::Unsupported
                                }))
                    }
                    _ => false,
                }
            }
            Expected::Related(r, bound) => match &column.representation {
                Representation::RelatedKeys {
                    relationship,
                    key,
                    bound: b,
                } => {
                    relationship == &r.identity
                        && *b == bound
                        && key.id == r.target_key.id
                        && key.fields == r.target_key.fields
                        && key.types == r.target_key.types
                        && !column.nullable
                }
                _ => false,
            },
        };
        if !valid {
            return Err(fail("Result representation changes logical type, presence, relationship key or exact numeric decoding"));
        }
    }
    Ok(())
}
#[cfg(test)]
fn validate_parameter(value: &str,t:&LogicalType)->Result<()> {validate_parameter_domain(value,t,false)}
fn validate_parameter_domain(value: &str, t: &LogicalType, unbounded_admitted:bool) -> Result<()> {
    if t.family==Family::Integer && t.facets==serde_json::json!({}) && !unbounded_admitted {return Err(fail("Integer parameter needs an explicit bounded width"));}
    let kind = match t.family {
        Family::String => {
            if value.contains('\0') {
                return Err(fail("String parameter contains NUL"));
            }
            crate::syntax::LiteralKind::String
        }
        Family::Boolean => {
            if value != "true" && value != "false" {
                return Err(fail("Boolean parameter is not canonical exact text"));
            }
            crate::syntax::LiteralKind::Boolean
        }
        Family::Integer | Family::Decimal => {
            let unsigned = value.strip_prefix('-').unwrap_or(value);
            let mut parts = unsigned.split('.');
            let whole = parts.next().unwrap_or("");
            let frac = parts.next();
            if whole.is_empty()
                || !whole.bytes().all(|b| b.is_ascii_digit())
                || frac.is_some_and(|f| f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit()))
                || parts.next().is_some()
            {
                return Err(fail("Numeric parameter is not exact base-ten text"));
            }
            if t.family == Family::Integer && t.facets != serde_json::json!({}) {
                let w = &t.facets["integerWidth"];
                if !w["bits"].as_u64().is_some_and(|b| (1..=64).contains(&b))
                    || !w["signed"].is_boolean()
                {
                    return Err(fail("Integer parameter needs an explicit bounded width"));
                }
            } else if t.family == Family::Decimal {
                let p = t.facets["precision"].as_u64();
                let s = t.facets["scale"].as_u64();
                if !p.is_some_and(|p| (1..=28).contains(&p)) || !s.is_some_and(|s| s <= p.unwrap())
                {
                    return Err(fail("Decimal parameter needs an explicit exact domain"));
                }
            }
            crate::syntax::LiteralKind::Number
        }
    };
    crate::exact::literal(
        &crate::syntax::Literal {
            value: value.into(),
            kind,
            span: crate::ir::Span { start: 0, end: 0 },
        },
        t,
    )
    .map_err(|_| fail("Backend parameter value exceeds its declared logical domain"))?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn arithmetic_output_retains_exact_derived_domains() {
        let cases: serde_json::Value = serde_json::from_str(include_str!("../../../tests/application/fixtures/cases.json")).unwrap();
        let req = &cases.as_array().unwrap().iter().find(|c| c["id"] == "join-count").unwrap()["request"];
        let catalog = crate::model::Catalog::prepare(serde_json::from_value(req["modules"].clone()).unwrap()).unwrap();
        let plan = crate::arithmetic_application_resolve::resolve(&catalog, crate::arithmetic_query::parse("SELECT c.id+1 AS next,o.total*12.5000 AS scaled FROM Customer c JOIN Orders o ON o.customer_id=c.id").unwrap(), Default::default(), None).unwrap();
        let selection = Selection { records: vec![plan.source.record.clone()], ..Default::default() };
        let columns = plan.outputs.iter().enumerate().map(|(i, output)| {
            let crate::arithmetic_plan::Expression::Arithmetic { expression } = &output.expression else { panic!("expected arithmetic"); };
            let (family, facets, decoder) = match expression.domain {
                crate::arithmetic_resolve::Domain::Integer => (Family::Integer, json!({}), ScalarDecoder::ExactInteger),
                crate::arithmetic_resolve::Domain::Decimal { scale } => (Family::Decimal, json!({"scale":scale}), ScalarDecoder::ExactDecimal),
            };
            Column { position: i+1, carrier_name: None, output_name: output.name.clone(), representation: Representation::Scalar { logical_type: LogicalType { family, facets, nullable:false }, carrier:ScalarCarrier::Text, decoder }, source_identities:vec![plan.source.record.clone()], nullable:false }
        }).collect();
        let baseline = Emission { sql:"SELECT fixture".into(), parameters:vec![], obligations:vec![], columns };
        validate(Plan::V03(&plan), &baseline, &selection, &[]).unwrap();
        let mut parameterized = baseline.clone();
        parameterized.parameters.push(crate::backend::ParameterSlot {
            position:1, logical_type:LogicalType {family:Family::Decimal,facets:json!({"scale":4}),nullable:false},
            value:"12.5000".into(),origin:json!({"kind":"literal"}),
        });
        let assessment = crate::backend::Assessment {id:"arithmetic.exact.decimal".into(),status:crate::backend::Status::Candidate,evidence:vec![],obligations:vec![]};
        assert!(validate(Plan::V03(&plan),&parameterized,&selection,&[]).is_err());
        validate(Plan::V03(&plan),&parameterized,&selection,&[assessment.clone()]).unwrap();
        for (value,facets) in [("12.5000",json!({"scale":3})),("12.5e1",json!({"scale":4})),("12.5000",json!({"scale":4,"nativeWidth":38}))] {
            let mut bad=parameterized.clone();bad.parameters[0].value=value.into();bad.parameters[0].logical_type.facets=facets;
            assert!(validate(Plan::V03(&plan),&bad,&selection,&[assessment.clone()]).is_err());
        }
        for mutation in 0..4 {
            let mut emission = baseline.clone();
            if mutation == 0 { emission.columns[1].nullable = true; }
            else {
                let Representation::Scalar { logical_type, carrier, decoder } = &mut emission.columns[1].representation else { unreachable!() };
                match mutation {
                    1 => { logical_type.facets["precision"] = json!(38); },
                    2 => { *carrier = ScalarCarrier::Boolean; },
                    3 => { *decoder = ScalarDecoder::ExactInteger; },
                    _ => unreachable!(),
                }
            }
            assert_eq!(validate(Plan::V03(&plan), &emission, &selection, &[]).unwrap_err().code, "WFT-EMIT");
        }
    }
    #[test]
    fn exact_numeric_slots_refuse_invalid_domains_and_lexemes() {
        let integer = LogicalType {
            family: Family::Integer,
            facets: json!({"integerWidth":{"bits":64,"signed":false}}),
            nullable: false,
        };
        for v in ["0", "18446744073709551615", "0001"] {
            assert!(validate_parameter(v, &integer).is_ok());
        }
        for v in [
            "-1",
            "18446744073709551616",
            "1e2",
            "+1",
            "1.0",
            "1; DROP TABLE x",
        ] {
            assert!(validate_parameter(v, &integer).is_err());
        }
        let decimal = LogicalType {
            family: Family::Decimal,
            facets: json!({"precision":28,"scale":2}),
            nullable: false,
        };
        assert!(validate_parameter("9007199254740993.12", &decimal).is_ok());
        assert!(validate_parameter("1.2000", &decimal).is_ok());
        assert!(validate_parameter("1.201", &decimal).is_err());
        let unbounded = LogicalType {
            family: Family::Integer,
            facets: json!({}),
            nullable: false,
        };
        assert!(validate_parameter("1", &unbounded).is_err());
    }
    #[test]
    fn parameter_domain_guards_have_exact_diagnostics() {
        let ty = |family, facets| LogicalType {family, facets, nullable:false};
        let integer=ty(Family::Integer,json!({"integerWidth":{"bits":8,"signed":true}}));
        let decimal=ty(Family::Decimal,json!({"precision":3,"scale":1}));
        let string=ty(Family::String,json!({}));
        let boolean=ty(Family::Boolean,json!({}));
        for (value,t,message) in [
            ("a\0b",string,"String parameter contains NUL"),
            ("TRUE",boolean,"Boolean parameter is not canonical exact text"),
            ("",integer.clone(),"Numeric parameter is not exact base-ten text"),
            (".1",decimal.clone(),"Numeric parameter is not exact base-ten text"),
            ("1.",decimal.clone(),"Numeric parameter is not exact base-ten text"),
            ("1.2.3",decimal.clone(),"Numeric parameter is not exact base-ten text"),
            ("1e2",decimal.clone(),"Numeric parameter is not exact base-ten text"),
            ("128",integer.clone(),"Backend parameter value exceeds its declared logical domain"),
            ("-129",integer,"Backend parameter value exceeds its declared logical domain"),
            ("100.0",decimal.clone(),"Backend parameter value exceeds its declared logical domain"),
            ("1.01",decimal,"Backend parameter value exceeds its declared logical domain"),
        ] {
            let e=validate_parameter(value,&t).unwrap_err();assert_eq!(e.code,"WFT-EMIT");assert_eq!(e.message,message);
        }
        for facets in [json!({}),json!({"integerWidth":{"bits":0,"signed":true}}),json!({"integerWidth":{"bits":65,"signed":true}}),json!({"integerWidth":{"bits":8,"signed":"true"}})] {
            assert_eq!(validate_parameter("1",&ty(Family::Integer,facets)).unwrap_err().message,"Integer parameter needs an explicit bounded width");
        }
        for facets in [json!({}),json!({"precision":0,"scale":0}),json!({"precision":29,"scale":0}),json!({"precision":3}),json!({"precision":3,"scale":4})] {
            assert_eq!(validate_parameter("1",&ty(Family::Decimal,facets)).unwrap_err().message,"Decimal parameter needs an explicit exact domain");
        }
        for (value,t) in [("-128",ty(Family::Integer,json!({"integerWidth":{"bits":8,"signed":true}}))),("127",ty(Family::Integer,json!({"integerWidth":{"bits":8,"signed":true}}))),("99.9",ty(Family::Decimal,json!({"precision":3,"scale":1}))),("true",ty(Family::Boolean,json!({}))),("false",ty(Family::Boolean,json!({}))),("雪",ty(Family::String,json!({})))] {
            validate_parameter(value,&t).unwrap();
        }
    }
    #[test]
    fn related_result_metadata_preserves_authored_key_and_bound() {
        let cases:serde_json::Value=serde_json::from_str(include_str!("../../../tests/application/fixtures/cases.json")).unwrap();
        let req=&cases.as_array().unwrap().iter().find(|c|c["id"]=="related-page").unwrap()["request"];
        let modules=serde_json::from_value(req["modules"].clone()).unwrap();
        let (_,mut p)=crate::prepare_and_resolve_application(req["sql"].as_str().unwrap(),modules,Default::default(),None).unwrap();
        p.outputs.retain(|o|matches!(o.expression,crate::application_ir::Expression::RelatedKeys{..}));
        assert_eq!(p.outputs.len(),1);
        let crate::application_ir::Expression::RelatedKeys{relationship:r,bound,..}=&p.outputs[0].expression else {unreachable!()};
        let selection=Selection{records:vec![r.from.clone(),r.to.clone()],..Default::default()};
        let baseline=Emission{sql:"SELECT fixture".into(),parameters:vec![],obligations:vec![],columns:vec![Column{position:1,carrier_name: None, output_name:p.outputs[0].name.clone(),representation:Representation::RelatedKeys{relationship:r.identity.clone(),key:r.target_key.clone(),bound:*bound},source_identities:vec![r.from.clone()],nullable:false}]};
        validate(Plan::V02(&p),&baseline,&selection,&[]).unwrap();
        for mode in 0..7 {
            let mut e=baseline.clone();
            if mode==5 {e.columns[0].nullable=true;}
            else if mode==6 {e.columns[0].representation=Representation::Value{descriptor:r.from.clone(),native_null:false};}
            else {
                let Representation::RelatedKeys{relationship,key,bound}=&mut e.columns[0].representation else {unreachable!()};
                match mode {0=>relationship.revision="wrong".into(),1=>*bound+=1,2=>key.id="wrong".into(),3=>key.fields.clear(),4=>key.types.clear(),_=>unreachable!()}
            }
            let error=validate(Plan::V02(&p),&e,&selection,&[]).unwrap_err();
            assert_eq!(error.code,"WFT-EMIT");
            assert_eq!(error.message,"Result representation changes logical type, presence, relationship key or exact numeric decoding","mode {mode}");
        }
    }
    #[test]
    fn initial_plan_requires_projection_root_before_output_validation() {
        let modules=crate::model::ModuleInput{
            document_json:include_str!("../../../docs/helix/03-test/fixtures/sales.umf.json").into(),
            pin:crate::ir::ModelPin{document_id:"sales-fixture".into(),revision:"fixture".into(),umf_version:"0.7.0".into(),sha256:crate::json::sha256(include_str!("../../../docs/helix/03-test/fixtures/sales.umf.json").as_bytes())},
            selected_module_ids:vec!["sales".into()],
        };
        let (_,mut plan)=crate::prepare_and_resolve("SELECT c.name FROM Customer c",vec![modules]).unwrap();
        let Node::Project{input,..}=plan.root else {panic!("resolved query must have projection")};
        plan.root=*input;
        let emission=Emission{sql:"SELECT plausible".into(),parameters:vec![],columns:vec![],obligations:vec![]};
        let error=validate(Plan::V01(&plan),&emission,&Selection::default(),&[]).unwrap_err();
        assert_eq!(error.code,"WFT-EMIT");
        assert_eq!(error.message,"Resolved 0.1 plan needs a projection root");
    }
    #[test]
    fn scalar_carriers_preserve_family_nullability_and_facets() {
        for (family,facets,carriers,decoder) in [
            (Family::String,json!({}),vec![ScalarCarrier::Text],ScalarDecoder::Text),
            (Family::Boolean,json!({}),vec![ScalarCarrier::Text,ScalarCarrier::Boolean],ScalarDecoder::Boolean),
            (Family::Integer,json!({"integerWidth":{"bits":64,"signed":false}}),vec![ScalarCarrier::Text],ScalarDecoder::ExactInteger),
            (Family::Decimal,json!({"precision":28,"scale":2}),vec![ScalarCarrier::Text],ScalarDecoder::ExactDecimal),
        ] {
            for nullable in [false,true] {
                let t=LogicalType{family:family.clone(),facets:facets.clone(),nullable};
                for carrier in &carriers {
                    let column=Column{position:1,carrier_name: None, output_name:"value".into(),representation:Representation::Scalar{logical_type:t.clone(),carrier:carrier.clone(),decoder:decoder.clone()},source_identities:vec![],nullable};
                    assert!(scalar(&column,&t));
                    let mut wrong=column.clone();wrong.nullable=!nullable;assert!(!scalar(&wrong,&t));
                    let mut wrong=column.clone();
                    if let Representation::Scalar{logical_type,..}=&mut wrong.representation {logical_type.facets=json!({"changed":true});}
                    assert!(!scalar(&wrong,&t));
                    let mut wrong=column.clone();
                    if let Representation::Scalar{decoder,..}=&mut wrong.representation {*decoder=ScalarDecoder::ExactDecimal;}
                    if family!=Family::Decimal {assert!(!scalar(&wrong,&t));}
                }
            }
        }
    }
    #[test]
    fn numeric_results_require_exact_text_decoders() {
        let t = LogicalType {
            family: Family::Integer,
            facets: json!({}),
            nullable: false,
        };
        let mut c = Column {
            position: 1,
            carrier_name: None, output_name: "count".into(),
            representation: Representation::Scalar {
                logical_type: t.clone(),
                carrier: ScalarCarrier::Text,
                decoder: ScalarDecoder::ExactInteger,
            },
            source_identities: vec![],
            nullable: false,
        };
        assert!(scalar(&c, &t));
        c.representation = Representation::Scalar {
            logical_type: t.clone(),
            carrier: ScalarCarrier::Boolean,
            decoder: ScalarDecoder::ExactInteger,
        };
        assert!(!scalar(&c, &t));
        c.representation = Representation::Scalar {
            logical_type: t.clone(),
            carrier: ScalarCarrier::Text,
            decoder: ScalarDecoder::Text,
        };
        assert!(!scalar(&c, &t));
    }
    #[test]
    fn presence_and_native_null_require_qualified_descriptors() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/application/fixtures/cases.json"
        ))
        .unwrap();
        let req = &cases[0]["request"];
        let modules = serde_json::from_value(req["modules"].clone()).unwrap();
        let (_, p) = crate::prepare_and_resolve_application(
            req["sql"].as_str().unwrap(),
            modules,
            Default::default(),
            None,
        )
        .unwrap();
        let selection = Selection {
            fields: p
                .outputs
                .iter()
                .map(|o| {
                    if let crate::application_ir::Expression::Field { identity, .. } = &o.expression
                    {
                        identity.clone()
                    } else {
                        panic!("fixture")
                    }
                })
                .collect(),
            types: p.type_graph.iter().map(|d| d.identity.clone()).collect(),
            ..Default::default()
        };
        let mut e = Emission {
            sql: "SELECT fixture".into(),
            parameters: vec![],
            obligations: vec![],
            columns: p
                .outputs
                .iter()
                .enumerate()
                .map(|(index, o)| {
                    let crate::application_ir::Expression::Field { identity, .. } = &o.expression
                    else {
                        panic!("fixture")
                    };
                    Column {
                        position: index + 1,
                        carrier_name: None, output_name: o.name.clone(),
                        representation: Representation::Value {
                            descriptor: identity.clone(),
                            native_null: false,
                        },
                        source_identities: vec![identity.clone()],
                        nullable: false,
                    }
                })
                .collect(),
        };
        assert!(validate(Plan::V02(&p), &e, &selection, &[]).is_ok());
        // The complete baseline permits isolating descriptor/presence guard failures.
        let mut missing=p.clone();missing.type_graph.clear();
        let error=validate(Plan::V02(&missing),&e,&selection,&[]).unwrap_err();
        assert_eq!(error.code,"WFT-EMIT");
        assert_eq!(error.message,"Projected Field lacks its type descriptor");
        let mut nullable=e.clone();nullable.columns[0].nullable=true;
        let error=validate(Plan::V02(&p),&nullable,&selection,&[]).unwrap_err();
        assert_eq!(error.message,"Result representation changes logical type, presence, relationship key or exact numeric decoding");
        let mut wrong_kind=e.clone();
        wrong_kind.columns[0].representation=Representation::Scalar{
            logical_type:LogicalType{family:Family::String,facets:json!({}),nullable:false},carrier:ScalarCarrier::Text,decoder:ScalarDecoder::Text,
        };
        let error=validate(Plan::V02(&p),&wrong_kind,&selection,&[]).unwrap_err();
        assert_eq!(error.message,"Result representation changes logical type, presence, relationship key or exact numeric decoding");
        if let Representation::Value { native_null, .. } = &mut e.columns[3].representation {
            *native_null = true;
        }
        assert!(validate(Plan::V02(&p), &e, &selection, &[]).is_err());
        let capability = crate::backend::Assessment {
            id: "value.nativeNull".into(),
            status: crate::backend::Status::Supported,
            evidence: vec!["fixture".into()],
            obligations: vec![],
        };
        // Candidate is admitted here only after the pipeline's explicit opt-in;
        // unsupported or unrelated assessments cannot authorize native null.
        for (id, status, accepted) in [
            ("value.nativeNull", crate::backend::Status::Supported, true),
            ("value.nativeNull", crate::backend::Status::Candidate, true),
            ("value.nativeNull", crate::backend::Status::Unsupported, false),
            ("value.other", crate::backend::Status::Supported, false),
        ] {
            let mut assessment = capability.clone();
            assessment.id = id.into();
            assessment.status = status.clone();
            let result = validate(Plan::V02(&p), &e, &selection, &[assessment]);
            if accepted {
                assert!(result.is_ok(), "{id}: {status:?}");
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.code, "WFT-EMIT");
                assert_eq!(error.message, "Result representation changes logical type, presence, relationship key or exact numeric decoding");
            }
        }
        if let Representation::Value { descriptor, .. } = &mut e.columns[0].representation {
            descriptor.element = "wrong".into();
        }
        assert!(validate(Plan::V02(&p), &e, &selection, &[]).is_err());
    }
    #[test]
    fn positioned_metadata_refuses_incomplete_colliding_or_reordered_cells() {
        let cases:serde_json::Value=serde_json::from_str(include_str!("../../../tests/application/fixtures/cases.json")).unwrap();let req=&cases.as_array().unwrap().iter().find(|c|c["id"]=="join-count").unwrap()["request"];
        let catalog=crate::model::Catalog::prepare(serde_json::from_value(req["modules"].clone()).unwrap()).unwrap();let p=crate::arithmetic_application_resolve::resolve(&catalog,crate::arithmetic_query::parse("SELECT c.id,d.id,c.name FROM Customer c JOIN Customer d ON c.id=d.id").unwrap(),Default::default(),None).unwrap();
        let identities=p.outputs.iter().map(|o|{let crate::arithmetic_plan::Expression::Field{identity,..}=&o.expression else{panic!()};identity.clone()}).collect::<Vec<_>>();
        let selection=Selection{fields:identities.clone(),types:p.type_graph.iter().map(|d|d.identity.clone()).collect(),..Default::default()};
        let columns=p.outputs.iter().enumerate().map(|(i,o)|{let d=p.type_graph.iter().find(|d|d.identity==identities[i]).unwrap();let crate::application_model::Shape::Scalar{logical_type}=&d.shape else{panic!()};Column{position:i+1,output_name:o.name.clone(),carrier_name:Some(format!("_weft_output_{}",i+1)),source_identities:vec![identities[i].clone()],nullable:false,representation:Representation::Scalar{logical_type:logical_type.clone(),carrier:ScalarCarrier::Text,decoder:if logical_type.family==Family::Integer{ScalarDecoder::ExactInteger}else{ScalarDecoder::Text}}}}).collect();
        let mut baseline=Emission{sql:"SELECT fixture".into(),parameters:vec![],columns,obligations:vec![]};baseline.obligations.push(crate::backend::Obligation{id:"weft.output.positioned".into(),parameters:json!({"profile":"weft-positioned-output/0.3.0","columns":baseline.columns.iter().map(|c|json!({"position":c.position,"outputName":c.output_name,"carrierName":c.carrier_name,"sourceIdentities":c.source_identities})).collect::<Vec<_>>()}),owner:crate::backend::ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into()});let a=crate::backend::Assessment{id:"project.positionedOutputs".into(),status:crate::backend::Status::Candidate,evidence:vec![],obligations:vec![]};
        validate(Plan::V03(&p),&baseline,&selection,&[a.clone()]).unwrap();assert!(validate(Plan::V03(&p),&baseline,&selection,&[]).is_err());
        let mut missing=baseline.clone();missing.obligations.clear();assert!(validate(Plan::V03(&p),&missing,&selection,&[a.clone()]).is_err());
        let mut mismatch=baseline.clone();mismatch.obligations[0].parameters["columns"][0]["carrierName"]=json!("other");assert!(validate(Plan::V03(&p),&mismatch,&selection,&[a.clone()]).is_err());
        for mutation in 0..7 {let mut bad=baseline.clone();match mutation {0=>bad.columns[1].carrier_name=None,1=>bad.columns[1].carrier_name=bad.columns[0].carrier_name.clone(),2=>bad.columns[1].position=1,3=>bad.columns[1].output_name="changed".into(),4=>{bad.columns.pop();},5=>bad.columns[1].source_identities=vec![identities[2].clone()],_=>bad.columns[1].carrier_name=Some("".into())};bad.obligations[0].parameters["columns"]=json!(bad.columns.iter().map(|c|json!({"position":c.position,"outputName":c.output_name,"carrierName":c.carrier_name,"sourceIdentities":c.source_identities})).collect::<Vec<_>>());assert!(validate(Plan::V03(&p),&bad,&selection,&[a.clone()]).is_err(),"{mutation}");}
        let mut unique=p.clone();unique.outputs[1].name="other".into();unique.required_capabilities.retain(|c|c!="project.positionedOutputs");let mut bad=baseline.clone();bad.columns[1].output_name="other".into();assert!(validate(Plan::V03(&unique),&bad,&selection,&[]).is_err());
        // Position-based storage retains both repeated logical labels; a name-keyed map would collapse them.
        let original_row=vec!["first","second","name"];let cells=baseline.columns.iter().zip(&original_row).map(|(c,v)|(c.position,c.output_name.clone(),*v)).collect::<Vec<_>>();assert_eq!(cells.len(),3);assert_ne!(cells[0].2,cells[1].2);let dictionary=baseline.columns.iter().zip(original_row).map(|(c,v)|(c.output_name.clone(),v)).collect::<std::collections::BTreeMap<_,_>>();assert_eq!(dictionary.len(),2);
    }

}
