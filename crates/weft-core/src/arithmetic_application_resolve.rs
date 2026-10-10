//! Full internal 0.3 query resolution, reusing the original model semantic scope.
use crate::{
    application_resolve::{Parameter, Parameters},
    application_scope::Scope,
    application_syntax as old_ast, arithmetic_plan as ir, arithmetic_query as ast,
    arithmetic_resolve,
    error::{Diagnostic, Result},
    ir::{Family, LogicalType},
    model::Catalog,
};
use serde_json::json;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};
fn fail(code: &str, message: &str) -> Diagnostic {
    Diagnostic::new(code, "resolve", message)
}
fn same(a: &ir::Field, b: &ir::Field) -> bool {
    a.scan == b.scan && a.identity == b.identity
}
fn old_value(value: &ast::Value) -> old_ast::Value {
    match value {
        ast::Value::Literal(v) => old_ast::Value::Literal(v.clone()),
        ast::Value::Field(v) => old_ast::Value::Field(v.clone()),
        ast::Value::Parameter(v) => old_ast::Value::Parameter(v.clone()),
    }
}
fn expression_value(
    scope: &mut Scope,
    value: &crate::arithmetic_syntax::Expression,
    groups: Option<&Vec<ir::Field>>,
) -> Result<arithmetic_resolve::Expression> {
    let cell = RefCell::new(scope);
    let result = arithmetic_resolve::resolve(
        value,
        &mut |column| {
            let field = cell.borrow_mut().field(column)?;
            if groups.is_some_and(|groups| !groups.iter().any(|g| same(g, &field))) {
                return Err(fail(
                    "WFT-GROUPING",
                    "Computed projection contains an ungrouped field",
                )
                .at(&column.span));
            }
            Ok(field)
        },
        &mut |name| {
            if name.quoted {
                return Err(fail(
                    "WFT-PARAMETER",
                    "Source parameter names must be unquoted ASCII identifiers",
                )
                .at(&name.span));
            }
            let mut scope = cell.borrow_mut();
            let parameter: Parameter =
                scope.parameters.get(&name.value).cloned().ok_or_else(|| {
                    fail("WFT-PARAMETER", "Missing source parameter binding").at(&name.span)
                })?;
            scope.used.insert(name.value.clone());
            scope.caps.insert("parameter.named".into());
            Ok(parameter)
        },
    )?;
    fn capabilities(e: &arithmetic_resolve::Expression, caps: &mut BTreeSet<String>) {
        use arithmetic_resolve::{Domain, Kind};
        caps.insert(
            match e.domain {
                Domain::Integer => "arithmetic.exact.integer",
                Domain::Decimal { .. } => "arithmetic.exact.decimal",
            }
            .into(),
        );
        match &e.kind {
            Kind::Negate { operand } => {
                caps.insert("arithmetic.negate".into());
                capabilities(operand, caps)
            }
            Kind::Binary {
                operator,
                left,
                right,
            } => {
                caps.insert(format!("arithmetic.{operator}"));
                capabilities(left, caps);
                capabilities(right, caps)
            }
            _ => {}
        }
    }
    capabilities(&result, &mut cell.borrow_mut().caps);
    Ok(result)
}
fn predicate(scope: &mut Scope, p: &ast::Predicate, on: bool) -> Result<ir::Predicate> {
    if matches!(p, ast::Predicate::ArithmeticCompare { .. }) {
        scope.caps.insert("arithmetic.compareExact".into());
    }
    match p {
        ast::Predicate::StringIn{column,literals}=>{
            let field=scope.field(column)?;
            if field.logical_type.family!=Family::String || field.logical_type.nullable || field.logical_type.facets!=json!({}) {return Err(fail("WFT-TYPE","IN requires an exact required String Field"));}
            let values=literals.iter().map(|v|scope.value(&old_ast::Value::Literal(v.clone()),&field.logical_type)).collect::<Result<Vec<_>>>()?;
            scope.caps.insert("predicate.stringIn".into());
            Ok(ir::Predicate::StringIn{field,values})
        },
        ast::Predicate::NullTest{column,negated}=>{
            let field=scope.presence_field03(column)?;
            scope.caps.insert("predicate.nativeNull".into());scope.caps.insert("value.nativeNull".into());
            Ok(ir::Predicate::NullTest{field,negated:*negated})
        },
        ast::Predicate::ExtendedCompare { column, value, operator } => {
            let left = scope.field(column)?;
            if left.logical_type.family == Family::Boolean && *operator != ir::ComparisonOperator::NotEqual {
                return Err(fail("WFT-TYPE", "New scalar ordering operators exclude Boolean").at(&column.span));
            }
            let right = scope.value(&old_value(value), &left.logical_type)?;
            scope.caps.insert(operator.capability().into());
            // Explicit new-operator ON admission; the original predicate policy is unchanged.
            if on { scope.caps.insert("compare.scalarJoin".into()); }
            Ok(ir::Predicate::ScalarCompare { left, right, operator: *operator })
        }
        ast::Predicate::ArithmeticCompareExtended { left, right, operator } => {
            scope.caps.insert(operator.capability().into());
            scope.caps.insert("arithmetic.compareExact".into());
            if on { scope.caps.insert("compare.scalarJoin".into()); }
            Ok(ir::Predicate::ArithmeticCompareExtended {
                left: expression_value(scope, left, None)?,
                right: expression_value(scope, right, None)?,
                operator: *operator,
            })
        }
        ast::Predicate::ArithmeticCompare {
            left,
            right,
            greater,
        } => Ok(ir::Predicate::ArithmeticCompare {
            left: expression_value(scope, left, None)?,
            right: expression_value(scope, right, None)?,
            greater: *greater,
        }),
        ast::Predicate::Compare {columns,values,greater} if on && !greater && columns.len()==1 && values.len()==1 && matches!(values[0],ast::Value::Field(_)) && (scope.optional_field03(&columns[0])? || match &values[0] {ast::Value::Field(c)=>scope.optional_field03(c)?,_=>false}) => {
            let left=scope.presence_field03(&columns[0])?;
            let ast::Value::Field(column)=&values[0] else {unreachable!()};
            let right=scope.presence_field03(column)?;
            if left.logical_type.family!=Family::String || right.logical_type.family!=Family::String {
                return Ok(ir::Predicate::Legacy{predicate:scope.predicate(&old_ast::Predicate::Compare{columns:columns.clone(),values:values.iter().map(old_value).collect(),greater:*greater},on)?});
            }
            scope.caps.insert("compare.nullAwareStringEqual".into());scope.caps.insert("value.nativeNull".into());
            Ok(ir::Predicate::NullableStringEqual{left,right})
        },
        ast::Predicate::Compare {
            columns,
            values,
            greater,
        } => Ok(ir::Predicate::Legacy {
            predicate: scope.predicate(
                &old_ast::Predicate::Compare {
                    columns: columns.clone(),
                    values: values.iter().map(old_value).collect(),
                    greater: *greater,
                },
                on,
            )?,
        }),
        ast::Predicate::HasRelated { relationship, key } => Ok(ir::Predicate::Legacy {
            predicate: scope.predicate(
                &old_ast::Predicate::HasRelated {
                    relationship: relationship.clone(),
                    key: key.iter().map(old_value).collect(),
                },
                on,
            )?,
        }),
    }
}
pub(crate) fn resolve(
    catalog: &Catalog,
    q: ast::Query,
    parameters: Parameters,
    profile: Option<ir::ReadProfile>,
) -> Result<ir::Plan> {
    if profile.is_some() {
        return Err(fail(
            "WFT-PROFILE",
            "Existing named application-read subsets do not admit arithmetic 0.3",
        ));
    }
    let mut normalized = BTreeMap::new();
    if parameters.len() > 1024 {
        return Err(fail("WFT-LIMIT", "Source parameter count exceeds 1024"));
    }
    for (name, p) in parameters {
        if !name
            .as_bytes()
            .first()
            .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || normalized.insert(name.to_ascii_lowercase(), p).is_some()
        {
            return Err(fail(
                "WFT-PARAMETER",
                "Parameter names must be distinct ASCII identifiers after case folding",
            ));
        }
    }
    let mut s = Scope {
        catalog,
        records: vec![],
        parameters: normalized,
        used: BTreeSet::new(),
        caps: BTreeSet::new(),
        graph: vec![],
    };
    let source = s.add_source(&q.source)?;
    let mut joins = vec![];
    for (right, on) in &q.joins {
        let right = s.add_source(right)?;
        let on = on
            .iter()
            .map(|p| predicate(&mut s, p, true))
            .collect::<Result<Vec<_>>>()?;
        s.caps.insert("innerJoin".into());
        joins.push(ir::Join { right, on });
    }
    let filters = q
        .predicates
        .iter()
        .map(|p| predicate(&mut s, p, false))
        .collect::<Result<Vec<_>>>()?;
    if !filters.is_empty() {
        s.caps.insert("filter".into());
    }
    if filters.len() > 1 || joins.iter().any(|j| j.on.len() > 1) {
        s.caps.insert("and".into());
    }
    let groups = q
        .groups
        .iter()
        .map(|c| s.field(c))
        .collect::<Result<Vec<_>>>()?;
    if groups
        .iter()
        .enumerate()
        .any(|(i, g)| groups[..i].iter().any(|h| same(g, h)))
    {
        return Err(fail("WFT-GROUPING", "Repeated grouping field"));
    }
    let aggregate = q
        .outputs
        .iter()
        .any(|p| matches!(p.output, ast::Output::Count | ast::Output::CountDistinct(_) | ast::Output::Sum(_)))
        || !groups.is_empty();
    if aggregate {
        s.caps.insert("aggregate".into());
    }
    if !groups.is_empty() {
        s.caps.insert("group".into());
    }
    let mut outputs = vec![];
    let mut labels = BTreeMap::new();
    for p in &q.outputs {
        let (default, expressions) = match &p.output {
            ast::Output::Arithmetic(expression) => {
                let value = expression_value(
                    &mut s,
                    expression,
                    if aggregate { Some(&groups) } else { None },
                )?;
                (
                    String::new(),
                    vec![(
                        String::new(),
                        ir::Expression::Arithmetic { expression: value },
                    )],
                )
            }
            ast::Output::Entity(n) => {
                if aggregate {
                    return Err(fail(
                        "WFT-GROUPING",
                        "Whole-entity projection is excluded from aggregation",
                    ));
                }
                let (r, scan) = s.record(n)?;
                let e = catalog.entity_descriptor(r)?;
                s.descriptors(e.graph);
                s.caps.insert("project.entity".into());
                let expressions = e
                    .members
                    .into_iter()
                    .map(|m| {
                        (
                            m.name,
                            ir::Expression::Field {
                                scan: scan.clone(),
                                identity: m.identity,
                            },
                        )
                    })
                    .collect::<Vec<_>>();
                (String::new(), expressions)
            }
            ast::Output::Field(c) => {
                let (r, scan) = s.record_for_column(c)?;
                let (m, graph) = catalog.member_descriptor(r, &c.field)?;
                if aggregate {
                    let f = s.field(c)?;
                    if !groups.iter().any(|g| same(g, &f)) {
                        return Err(
                            fail("WFT-GROUPING", "Projected field is not grouped").at(&c.span)
                        );
                    }
                }
                s.descriptors(graph);
                (
                    m.name.clone(),
                    vec![(
                        m.name,
                        ir::Expression::Field {
                            scan,
                            identity: m.identity,
                        },
                    )],
                )
            }
            ast::Output::CountDistinct(c) => {
                let argument=s.field(c)?;
                if argument.logical_type.family!=Family::String || argument.logical_type.nullable || argument.logical_type.facets!=json!({}) {return Err(fail("WFT-TYPE","COUNT DISTINCT requires an exact required String Field"));}
                s.caps.insert("aggregate.countDistinct".into());
                ("count".into(),vec![("count".into(),ir::Expression::CountDistinct{argument,logical_type:LogicalType{family:Family::Integer,facets:json!({}),nullable:false}})])
            }
            ast::Output::Count => {
                s.caps.insert("aggregate.count".into());
                (
                    "count".into(),
                    vec![(
                        "count".into(),
                        ir::Expression::Count {
                            logical_type: LogicalType {
                                family: Family::Integer,
                                facets: json!({}),
                                nullable: false,
                            },
                        },
                    )],
                )
            }
            ast::Output::Sum(c) => {
                let argument = s.field(c)?;
                if !matches!(
                    argument.logical_type.family,
                    Family::Integer | Family::Decimal
                ) {
                    return Err(fail("WFT-TYPE", "SUM requires an exact numeric field").at(&c.span));
                }
                let facets = if argument.logical_type.family == Family::Decimal {
                    json!({"scale":argument.logical_type.facets["scale"]})
                } else {
                    json!({})
                };
                let t = LogicalType {
                    family: argument.logical_type.family.clone(),
                    facets,
                    nullable: groups.is_empty(),
                };
                s.caps.insert("sum".into());
                (
                    "sum".into(),
                    vec![(
                        "sum".into(),
                        ir::Expression::Sum {
                            argument,
                            logical_type: t,
                        },
                    )],
                )
            }
            ast::Output::Related {
                relationship,
                bound,
            } => {
                if aggregate {
                    return Err(fail("WFT-GROUPING", "RELATED_KEYS cannot be aggregated"));
                }
                let (r, scan) = s.record(&relationship.alias)?;
                let rel = catalog.relationship_read(r, &relationship.field)?;
                s.caps.insert("relationship.boundedKeys".into());
                if rel.inverse {
                    s.caps.insert("relationship.inverse".into());
                }
                (
                    relationship.field.value.clone(),
                    vec![(
                        relationship.field.value.clone(),
                        ir::Expression::RelatedKeys {
                            scan,
                            relationship: rel,
                            bound: *bound,
                        },
                    )],
                )
            }
        };
        for (label, expression) in expressions {
            let name = p
                .alias
                .as_ref()
                .map(|a| a.value.clone())
                .unwrap_or(if default.is_empty() {
                    label
                } else {
                    default.clone()
                });
            let implicit_field = p.alias.is_none() && matches!(p.output, ast::Output::Field(_))
                && matches!(&expression, ir::Expression::Field { identity, .. }
                    if s.graph.iter().any(|descriptor| descriptor.identity == *identity
                        && matches!(descriptor.shape, crate::application_model::Shape::Scalar { .. })));
            if let Some(previous) = labels.get(&name) {
                if !implicit_field || !previous {
                    return Err(fail("WFT-OUTPUT-NAME", "Repeated explicit or computed output name"));
                }
                s.caps.insert("project.positionedOutputs".into());
            } else {
                labels.insert(name.clone(), implicit_field);
            }
            outputs.push(ir::Output { name, expression });
        }
        if outputs.len() > 256 {
            return Err(fail("WFT-LIMIT", "Expanded output count exceeds 256"));
        }
    }
    if outputs.is_empty() {
        return Err(fail(
            "WFT-OUTPUT-NAME",
            "Projection expands to no output fields",
        ));
    }
    let order = q
        .order
        .iter()
        .map(|c| s.field(c))
        .collect::<Result<Vec<_>>>()?;
    if aggregate && order.iter().any(|f| !groups.iter().any(|g| same(f, g))) {
        return Err(fail("WFT-GROUPING", "Ordering field is not grouped"));
    }
    if !order.is_empty() {
        s.caps.insert("order.asc".into());
    }
    if q.limit.is_some() {
        s.caps.insert("limit".into());
    }
    if q.distinct {
        if aggregate || profile.is_some() || q.outputs.iter().any(|o| !matches!(o.output, ast::Output::Field(_))) {
            return Err(fail("WFT-CAPABILITY", "DISTINCT requires direct required String Field projections without aggregation or page profile"));
        }
        for output in &outputs {
            let ir::Expression::Field { scan: _, identity } = &output.expression else { return Err(fail("WFT-CAPABILITY", "Unsupported DISTINCT output")); };
            let descriptor = s.graph.iter().find(|d| &d.identity == identity).ok_or_else(|| fail("WFT-TYPE", "Missing DISTINCT descriptor"))?;
            if descriptor.availability.as_deref() != Some("required") || !matches!(&descriptor.shape, crate::application_model::Shape::Scalar { logical_type } if logical_type.family == Family::String && logical_type.facets == json!({}) && !logical_type.nullable) {
                return Err(fail("WFT-CAPABILITY", "DISTINCT supports required non-null exact String Fields only"));
            }
        }
        if order.iter().any(|field| !outputs.iter().any(|output| matches!(&output.expression, ir::Expression::Field { scan, identity } if scan == &field.scan && identity == &field.identity))) {
            return Err(fail("WFT-CAPABILITY", "DISTINCT ordering must reference an exact projected scan and Field identity"));
        }
        s.caps.insert("project.distinct".into());
    }
    let page_key = None;
    if s.used.len() != s.parameters.len() {
        return Err(fail("WFT-PARAMETER", "Surplus source parameter binding"));
    }
    s.caps.insert("project".into());
    if s.graph.iter().any(|d|d.availability.as_deref()==Some("absent-allowed")) {s.caps.insert("value.nativeNull".into());}
    Ok(ir::Plan {
        distinct: q.distinct,
        ir_version: "weft-ir/0.3.0".into(),
        module_pins: catalog.pins(),
        read_profile: profile,
        required_capabilities: s.caps.into_iter().collect(),
        type_graph: s.graph,
        source,
        page_key,
        joins,
        filters,
        groups,
        aggregate,
        outputs,
        order,
        limit: q.limit,
    })
}
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/logical-plan-v0.3.schema.json")]
struct PlanSchema03;
#[cfg(test)]
mod tests {
    use super::*;
    fn request(id: &str) -> serde_json::Value {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/application/fixtures/cases.json"
        ))
        .unwrap();
        cases
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == id)
            .unwrap()["request"]
            .clone()
    }
    fn run(id: &str, sql: &str, parameters: serde_json::Value) -> Result<ir::Plan> {
        let req = request(id);
        let catalog = Catalog::prepare(serde_json::from_value(req["modules"].clone()).unwrap())?;
        resolve(
            &catalog,
            ast::parse(sql)?,
            serde_json::from_value(parameters).unwrap(),
            None,
        )
    }
    #[test]
    fn arithmetic_join_projection_and_exact_domain() {
        let p=run("join-count","SELECT c.id+1 AS next,o.total*12.5000 AS scaled FROM Customer c JOIN Orders o ON o.customer_id=c.id WHERE o.total*2>1",json!({})).unwrap();
        assert_eq!(p.ir_version, "weft-ir/0.3.0");
        assert!(PlanSchema03::is_valid(&serde_json::to_value(&p).unwrap()));
        let original=serde_json::to_value(&p).unwrap();
        for (pointer,value) in [
            ("/irVersion",json!("weft-ir/0.2.0")),
            ("/readProfile",json!({"version":"weft-application-read/0.2.0","subset":"entity-page"})),
            ("/outputs/1/expression/expression/kind/operator",json!("/")),
            ("/outputs/1/expression/expression/kind/right/kind/value",json!(12.5)),
            ("/outputs/1/expression/expression/kind/left/kind/field/type/nullable",json!(true)),
        ] {
            let mut bad=original.clone();*bad.pointer_mut(pointer).unwrap()=value;
            assert!(!PlanSchema03::is_valid(&bad),"{pointer}");
        }
        let mut bad=original.clone();bad["outputs"][1]["expression"]["expression"]["domain"]["precision"]=json!(38);
        assert!(!PlanSchema03::is_valid(&bad));

        assert_eq!(p.joins.len(), 1);
        assert!(matches!(p.joins[0].on[0], ir::Predicate::Legacy { .. }));
        assert!(matches!(
            p.filters[0],
            ir::Predicate::ArithmeticCompare { .. }
        ));
        let ir::Expression::Arithmetic { expression } = &p.outputs[1].expression else {
            panic!()
        };
        assert_eq!(
            expression.domain,
            arithmetic_resolve::Domain::Decimal { scale: 6 }
        );
        assert!(p.required_capabilities.contains(&"arithmetic.*".into()));
    }
    #[test]
    fn grouped_leaves_and_parameter_occurrences() {
        assert!(run(
            "global-count",
            "SELECT c.id+1 AS next,COUNT(*) AS n FROM Customer c GROUP BY c.id",
            json!({})
        )
        .is_ok());
        assert_eq!(
            run(
                "global-count",
                "SELECT c.id+1 AS next,COUNT(*) AS n FROM Customer c",
                json!({})
            )
            .unwrap_err()
            .code,
            "WFT-GROUPING"
        );
        assert!(run(
            "global-count",
            "SELECT :n+1 AS next FROM Customer c WHERE c.id=:n",
            json!({"n":{"family":"integer","value":"2"}})
        )
        .is_ok());
        assert!(run(
            "global-count",
            "SELECT :n+1 AS next FROM Customer c WHERE c.name=:n",
            json!({"n":{"family":"integer","value":"2"}})
        )
        .is_err());
        assert_eq!(
            run(
                "global-count",
                "SELECT c.id+1 AS next FROM Customer c",
                json!({"unused":{"family":"integer","value":"2"}})
            )
            .unwrap_err()
            .code,
            "WFT-PARAMETER"
        );
    }
    #[test]
    fn additive_comparisons_retain_scope_constraints_and_closed_ir() {
        let p=run("join-count", "SELECT c.id FROM Customer c JOIN Orders o ON o.customer_id<=c.id WHERE o.total+1<>2 AND c.name>='é'",json!({})).unwrap();
        assert!(p.required_capabilities.contains(&"compare.scalarJoin".into()));
        let original=serde_json::to_value(p).unwrap();
        assert!(PlanSchema03::is_valid(&original));
        for op in ["equal", "greater", "!=", "lesser"] {
            let mut bad=original.clone();bad["joins"][0]["on"][0]["operator"]=json!(op);
            assert!(!PlanSchema03::is_valid(&bad));
        }
        let mut bad=original.clone();bad["filters"][0]["left"]["domain"]["family"]=json!("string");assert!(!PlanSchema03::is_valid(&bad));
        for pointer in ["/joins/0/on/0/left/type/nullable","/joins/0/on/0/right/field/type/nullable"] {let mut bad=original.clone();*bad.pointer_mut(pointer).unwrap()=json!(true);assert!(!PlanSchema03::is_valid(&bad));}
        let mut bad=original.clone();bad["joins"][0]["on"][0]["right"]["field"]["type"]["family"]=json!("string");assert!(!PlanSchema03::is_valid(&bad));
        let mut bad=original.clone();bad["joins"][0]["on"][0]["left"]["type"]["family"]=json!("boolean");bad["joins"][0]["on"][0]["right"]["field"]["type"]["family"]=json!("boolean");assert!(!PlanSchema03::is_valid(&bad));
        assert!(run("join-count","SELECT c.id FROM Customer c JOIN Orders o ON o.customer_id>c.id",json!({})).is_err());
        assert!(run("join-count","SELECT c.id FROM Customer c JOIN Orders o ON o.customer_id=1",json!({})).is_err());
        assert_eq!(run("join-count","SELECT c.id FROM Customer c WHERE c.id<c.name",json!({})).unwrap_err().code,"WFT-TYPE");
        assert_eq!(run("join-count","SELECT c.id FROM Customer c WHERE c.id<:n AND c.name<>:n",json!({"n":{"family":"integer","value":"2"}})).unwrap_err().code,"WFT-PARAMETER");
        // One parameter must satisfy every original occurrence's bounded source domain.
        assert!(run("join-count","SELECT c.id FROM Customer c WHERE c.id<:n AND c.id+1>=:n",json!({"n":{"family":"integer","value":"2"}})).is_ok());
    }

    #[test]
    fn null03_closed_ir_preserves_nonnullable_ideal_and_boolean_operator() {
        let plan=run("join-count","SELECT c.name FROM Customer c WHERE c.name IS NULL",json!({})).unwrap();let original=serde_json::to_value(plan).unwrap();assert!(PlanSchema03::is_valid(&original));
        for mutation in ["negated","extra","nullable"] {
            let mut bad=original.clone();match mutation {"negated"=>bad["filters"][0]["negated"]=json!(0),"extra"=>bad["filters"][0]["future"]=json!(true),_=>bad["filters"][0]["field"]["type"]["nullable"]=json!(true)};assert!(!PlanSchema03::is_valid(&bad));
        }
        assert!(run("join-count","SELECT c.id+1 AS n FROM Customer c WHERE (c.id+1) IS NULL",json!({})).is_err());
    }

    #[test]
    fn distinct03_closed_member_is_true_only_and_absent_on_old_plans() {
        // This separately selected required String declaration preserves every other fixture member.
        let mut request=request("join-count");
        let mut document:serde_json::Value=serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
        let elements=document["modules"][0]["elements"].as_array_mut().unwrap();
        elements.iter_mut().find(|e|e["name"]=="name"&&e["kind"]=="field").unwrap()["nullability"]=json!("required");
        let raw=document.to_string();request["modules"][0]["documentJson"]=json!(raw);request["modules"][0]["pin"]["sha256"]=json!(crate::json::sha256(raw.as_bytes()));
        let modules=serde_json::from_value(request["modules"].clone()).unwrap();
        let catalog=Catalog::prepare(modules).unwrap();
        let plan=resolve(&catalog,ast::parse("SELECT DISTINCT c.name FROM Customer c").unwrap(),Default::default(),None).unwrap();
        let original=serde_json::to_value(plan).unwrap();assert!(PlanSchema03::is_valid(&original));
        for value in [json!(false),json!(1),json!(null),json!("true")] {let mut bad=original.clone();bad["distinct"]=value;assert!(!PlanSchema03::is_valid(&bad));}
        let old=run("join-count","SELECT c.id FROM Customer c",json!({})).unwrap();let original=serde_json::to_value(old).unwrap();assert!(original.get("distinct").is_none());assert!(PlanSchema03::is_valid(&original));
    }

    #[test]
    fn count_distinct_and_string_in_closed_ir_refuse_domain_and_slot_mutations() {
        let mut req=request("join-count");
        let mut doc:serde_json::Value=serde_json::from_str(req["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
        doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["name"]=="name"&&e["kind"]=="field").unwrap()["nullability"]=json!("required");
        let raw=doc.to_string();req["modules"][0]["documentJson"]=json!(raw);req["modules"][0]["pin"]["sha256"]=json!(crate::json::sha256(raw.as_bytes()));
        let catalog=Catalog::prepare(serde_json::from_value(req["modules"].clone()).unwrap()).unwrap();
        let p=resolve(&catalog,ast::parse("SELECT c.name,COUNT(DISTINCT c.name) AS n FROM Customer c WHERE c.name IN ('a','a') GROUP BY c.name").unwrap(),Default::default(),None).unwrap();
        let original=serde_json::to_value(p).unwrap();assert!(PlanSchema03::is_valid(&original));
        for pointer in ["/outputs/1/expression/argument/type/family","/filters/0/field/type/family","/filters/0/values/0/type/family"] {
            let mut bad=original.clone();*bad.pointer_mut(pointer).unwrap()=json!("integer");assert!(!PlanSchema03::is_valid(&bad));
        }
        for pointer in ["/outputs/1/expression/argument/type/nullable","/filters/0/field/type/nullable","/filters/0/values/0/type/nullable"] {
            let mut bad=original.clone();*bad.pointer_mut(pointer).unwrap()=json!(true);assert!(!PlanSchema03::is_valid(&bad));
        }
        let mut bad=original.clone();bad["filters"][0]["values"]=json!([]);assert!(!PlanSchema03::is_valid(&bad));
        let mut bad=original.clone();bad["filters"][0]["values"][0]["kind"]=json!("parameter");assert!(!PlanSchema03::is_valid(&bad));
        let mut bad=original.clone();bad["outputs"][1]["expression"]["type"]["facets"]=json!({"integerWidth":"signed64"});assert!(!PlanSchema03::is_valid(&bad));
    }

}
