//! Metadata correspondence only. Native query meaning requires backend conformance.
use super::*;
use crate::ir::{Family, Identity};
use serde_json::{json, Value};
pub(crate) fn validate_response_schema(value: &Value) -> bool {
    if value["interfaceVersion"] == "weft-compile/0.4.1" {
        static VALIDATOR041: std::sync::OnceLock<Option<jsonschema::Validator>> = std::sync::OnceLock::new();
        VALIDATOR041.get_or_init(|| {
            let plan:Value=serde_json::from_str(include_str!("../../../docs/helix/02-design/contracts/logical-plan-v0.4.1.schema.json")).ok()?;
            let response:Value=serde_json::from_str(include_str!("../../../docs/helix/02-design/contracts/compile-response-v0.4.1.schema.json")).ok()?;
            let registry=jsonschema::Registry::new().add("https://github.com/DocumentDrivenDX/weft/raw/main/docs/helix/02-design/contracts/logical-plan-v0.4.1.schema.json",plan).ok()?.prepare().ok()?;
            jsonschema::options().with_registry(&registry).build(&response).ok()
        }).as_ref().is_some_and(|validator|validator.is_valid(value))
    } else {
        static VALIDATOR: std::sync::OnceLock<jsonschema::Validator> = std::sync::OnceLock::new();
        VALIDATOR.get_or_init(|| {
    let plan: Value = serde_json::from_str(include_str!(
        "../../../docs/helix/02-design/contracts/logical-plan-v0.4.schema.json"
    ))
    .expect("committed plan04 schema");
    let response: Value = serde_json::from_str(include_str!(
        "../../../docs/helix/02-design/contracts/compile-response-v0.4.schema.json"
    ))
    .expect("committed response04 schema");
    let registry=jsonschema::Registry::new().add("https://github.com/DocumentDrivenDX/weft/raw/main/docs/helix/02-design/contracts/logical-plan-v0.4.schema.json",plan).expect("fixed schema URI").prepare().expect("closed local schema resources");
    jsonschema::options()
        .with_registry(&registry)
        .build(&response)
        .expect("committed response04 schema")
    }).is_valid(value)
    }
}

fn integer() -> crate::ir::LogicalType {
    crate::ir::LogicalType {
        family: Family::Integer,
        facets: json!({}),
        nullable: false,
    }
}
fn push<T: PartialEq>(v: &mut Vec<T>, x: T) {
    if !v.contains(&x) {
        v.push(x)
    }
}
pub(super) fn selection(plan: Plan04View<'_>) -> Result<Selection> {
    let mut s = Selection::default();
    push(&mut s.records, plan.source().record.clone());
    for j in plan.joins() {
        push(&mut s.records, j.right.record.clone())
    }
    fn walk(v: &Value, s: &mut Selection) {
        match v {
            Value::Object(m) => {
                if let Some(identity) = m.get("identity") {
                    if let Ok(id) = serde_json::from_value::<Identity>(identity.clone()) {
                        if m.get("kind").is_some() {
                            push(&mut s.types, id)
                        } else {
                            push(&mut s.fields, id)
                        }
                    }
                }
                for k in ["record", "from", "to"] {
                    if let Some(v) = m.get(k) {
                        if let Ok(id) = serde_json::from_value::<Identity>(v.clone()) {
                            push(&mut s.records, id)
                        }
                    }
                }
                if m.contains_key("relationship") && m.contains_key("documentId") {
                    if let (Some(d), Some(r), Some(module), Some(rel)) = (
                        m.get("documentId").and_then(Value::as_str),
                        m.get("revision").and_then(Value::as_str),
                        m.get("module").and_then(Value::as_str),
                        m.get("relationship").and_then(Value::as_str),
                    ) {
                        push(
                            &mut s.relationships,
                            crate::application_model::RelationshipIdentity {
                                document_id: d.into(),
                                revision: r.into(),
                                module: module.into(),
                                relationship: rel.into(),
                            },
                        )
                    }
                }
                if let Some(Value::Array(fields)) = m.get("fields") {
                    for f in fields {
                        if let Ok(id) = serde_json::from_value::<Identity>(f.clone()) {
                            push(&mut s.fields, id)
                        }
                    }
                }
                for v in m.values() {
                    walk(v, s)
                }
            }
            Value::Array(a) => {
                for v in a {
                    walk(v, s)
                }
            }
            _ => {}
        }
    }
    walk(&plan.serialized(), &mut s);
    // Constructor capability metadata is not a substitute for selected operations.
    let mut needed = vec![];
    for o in plan.outputs() {
        match o.expression {
            ExpressionView::RelatedPaths { .. } => {
                needed.extend(["relationship.twoHopPaths", "result.pathOccurrences"])
            }
            ExpressionView::CountDistinctPathTargets { .. } => {
                needed.push("aggregate.pathTargetDistinctCount")
            }
            ExpressionView::Legacy(crate::arithmetic_plan::Expression::Count { .. }) => {
                needed.push("aggregate.count")
            }
            ExpressionView::Legacy(crate::arithmetic_plan::Expression::CountDistinct {
                ..
            }) => needed.push("aggregate.countDistinct"),
            ExpressionView::Legacy(crate::arithmetic_plan::Expression::RelatedKeys { .. }) => {
                needed.push("relationship.boundedKeys")
            }
            _ => {}
        }
    }
    for h in plan.having() {
        match h.count {
            ExpressionView::Legacy(crate::arithmetic_plan::Expression::Count { .. }) => {
                if plan.ir_version() != "weft-ir/0.4.1"
                    || plan.groups().is_empty()
                    || !plan.outputs().any(|o| {
                        matches!(
                            o.expression,
                            ExpressionView::Legacy(
                                crate::arithmetic_plan::Expression::Count { .. }
                            )
                        )
                    })
                {
                    return Err(fail(
                        "WFT-CAPABILITY",
                        "Row-count HAVING version/group/projection correspondence differs",
                    ));
                }
                needed.push("aggregate.havingCountStarGreater");
            }
            ExpressionView::Legacy(crate::arithmetic_plan::Expression::CountDistinct {
                ..
            }) => needed.push("aggregate.havingCountDistinctGreater"),
            _ => return Err(fail("WFT-CAPABILITY", "Unknown HAVING count expression")),
        }
    }
    if plan.expansion().is_some() {
        needed.extend(["relationship.twoHopPaths", "relationship.pathExpansion"])
    }
    if plan.aggregate() {
        needed.push("aggregate")
    }
    if !plan.groups().is_empty() {
        needed.push("group")
    }
    if !plan.outer_join_scans().is_empty() {
        needed.extend(["join.left", "value.outerJoinPresence"])
    }
    if needed
        .iter()
        .any(|id| !plan.capabilities().iter().any(|c| c == id))
    {
        return Err(fail(
            "WFT-CAPABILITY",
            "Typed plan omits selected operation capabilities",
        ));
    }
    Ok(s)
}
fn same<T: Serialize>(left: &T, right: &Value) -> bool {
    serde_json::to_value(left).ok().as_ref() == Some(right)
}
pub(super) fn validate(
    plan: Plan04View<'_>,
    emitted: &Emission,
    selection: &Selection,
    qualifications: &[crate::backend::Qualification],
    target: &TargetProfile,
    manifest: &Manifest,
    binding: &BindingInput,
    edge_sources: &[NativeEdgeSource],
    record_sources: &[NativeRecordSource],
) -> Result<()> {
    bounded_json(emitted, 4 * 1024 * 1024)?;
    if emitted.sql.trim().is_empty()
        || emitted.sql.len() > 1024 * 1024
        || emitted.sql.contains('\0')
        || emitted.parameters.len() > 1024
    {
        return Err(fail("WFT-EMIT", "Invalid bounded SQL/parameters"));
    }
    for (i, p) in emitted.parameters.iter().enumerate() {
        if p.position != i + 1 || !p.origin.is_object() || p.logical_type.nullable {
            return Err(fail("WFT-EMIT", "Invalid parameter positions/origins"));
        }
        validate_parameter(p, qualifications)?;
    }
    let left: Vec<_> = plan
        .joins()
        .iter()
        .filter(|j| j.kind == Some(crate::arithmetic_plan::JoinKind::Left))
        .collect();
    if left.is_empty() {
        if emitted
            .obligations
            .iter()
            .any(|o| o.id == "outerJoin.matchIntegrity")
        {
            return Err(fail(
                "WFT-OBLIGATION",
                "Inner-only plan invents outer join custody",
            ));
        }
    } else {
        let o = emitted
            .obligations
            .iter()
            .find(|o| o.id == "outerJoin.matchIntegrity")
            .ok_or_else(|| fail("WFT-OBLIGATION", "LEFT plan lacks complete match integrity"))?;
        let maps = o.parameters["scans"]
            .as_array()
            .ok_or_else(|| fail("WFT-OBLIGATION", "LEFT map must be an array"))?;
        if record_sources.len() != left.len() || maps.len() != left.len() {
            return Err(fail("WFT-OBLIGATION", "LEFT map coverage mismatch"));
        }
        for ((map, join), source) in maps.iter().zip(left).zip(record_sources) {
            if map["scan"] != join.right.occurrence
                || !same(&join.right.record, &map["record"])
                || source.scan != join.right.occurrence
                || source.record != join.right.record
                || !same(&source.table, &map["table"])
                || map["identityColumn"] != source.identity_column
                || map["nativeType"] != source.native_type
            {
                return Err(fail(
                    "WFT-OBLIGATION",
                    "LEFT map changes immutable source declaration",
                ));
            }
        }
    }
    let outputs: Vec<_> = plan.outputs().collect();
    if outputs.len() != emitted.columns.len() {
        return Err(fail("WFT-EMIT", "Column count mismatch"));
    }
    let mut carriers = std::collections::BTreeSet::new();
    let positioned = plan
        .capabilities()
        .iter()
        .any(|c| c == "project.positionedOutputs");
    let repeated = outputs
        .iter()
        .enumerate()
        .any(|(i, o)| outputs[..i].iter().any(|previous| previous.name == o.name));
    if positioned != repeated {
        return Err(fail(
            "WFT-EMIT",
            "Positioned admission must equal repeated logical labels",
        ));
    }
    if positioned {
        if !qualifications.iter().any(|q| {
            q.assessment.id == "project.positionedOutputs"
                && q.assessment.status != Status::Unsupported
        }) {
            return Err(fail(
                "WFT-EMIT",
                "Positioned outputs lack capability admission",
            ));
        }
        let map=json!(emitted.columns.iter().map(|c|json!({"position":c.position,"outputName":c.output_name,"carrierName":c.carrier_name,"sourceIdentities":c.source_identities})).collect::<Vec<_>>());
        if !emitted.obligations.iter().any(|o| {
            o.id == "weft.output.positioned"
                && o.owner == crate::backend::ObligationOwner::Host
                && o.failure_code == "WFT-OBLIGATION"
                && o.parameters.as_object().is_some_and(|m| m.len() == 2)
                && o.parameters["profile"] == "weft-positioned-output/0.3.0"
                && o.parameters["columns"] == map
        }) {
            return Err(fail(
                "WFT-EMIT",
                "Positioned outputs require exact ordered host custody map",
            ));
        }
    }
    let mut paths = vec![];
    let mut count_kinds = std::collections::BTreeSet::new();
    for (i, (output, column)) in outputs.iter().zip(&emitted.columns).enumerate() {
        if column.position != i + 1
            || column.output_name != output.name
            || column.source_identities.is_empty()
            || column.source_identities.iter().any(|id| {
                !selection.records.contains(id)
                    && !selection.fields.contains(id)
                    && !selection.types.contains(id)
            })
        {
            return Err(fail("WFT-EMIT", "Output order/name/lineage mismatch"));
        }
        if positioned {
            let name = column
                .carrier_name
                .as_ref()
                .ok_or_else(|| fail("WFT-EMIT", "Missing positioned carrier"))?;
            if name.is_empty() || name.len() > 128 || name.contains('\0') || !carriers.insert(name)
            {
                return Err(fail("WFT-EMIT", "Invalid physical output names"));
            }
        } else if column.carrier_name.is_some() {
            return Err(fail("WFT-EMIT", "Undeclared positioned output"));
        }
        match output.expression {
            ExpressionView::RelatedPaths { path, bound } => {
                let Representation::RelatedPaths {
                    path: wire,
                    start_record,
                    bound: b,
                    edge_encoding,
                    outer_join,
                } = &column.representation
                else {
                    return Err(fail("WFT-EMIT", "Path output representation missing"));
                };
                let expected_outer = if plan
                    .outer_join_scans()
                    .iter()
                    .any(|s| s == path.start_scan())
                {
                    Some(crate::backend::OuterJoin {
                        scan: path.start_scan().into(),
                        record: path.start_record().clone(),
                    })
                } else {
                    None
                };
                if !same(wire, &path.serialized())
                    || start_record != path.start_record()
                    || *b != bound
                    || edge_encoding != "signed64-decimal/0.1"
                    || outer_join != &expected_outer
                    || column.nullable
                {
                    return Err(fail("WFT-EMIT", "Path descriptor/role mismatch"));
                }
                paths.push(json!({"path":path.serialized(),"edgeEncoding":"signed64-decimal/0.1","bound":bound}));
            }
            ExpressionView::CountDistinctPathTargets { occurrence } => {
                let e = plan
                    .expansion()
                    .ok_or_else(|| fail("WFT-EMIT", "No active path expansion"))?;
                let Representation::Scalar {
                    logical_type,
                    carrier,
                    decoder,
                    path_target: Some(pt),
                } = &column.representation
                else {
                    return Err(fail("WFT-EMIT", "Distinct target descriptor missing"));
                };
                if occurrence != e.occurrence
                    || pt.path_occurrence != occurrence
                    || pt.record != e.path.hops()[1].to
                    || !same(
                        &pt.key,
                        &serde_json::to_value(&e.path.hops()[1].target_key).unwrap(),
                    )
                    || *logical_type != integer()
                    || !matches!(carrier, crate::backend::ScalarCarrier::Text)
                    || !matches!(decoder, crate::backend::ScalarDecoder::ExactInteger)
                    || column.nullable
                {
                    return Err(fail("WFT-EMIT", "Distinct target identity/codec mismatch"));
                }
                count_kinds.insert("targetDistinct");
            }
            ExpressionView::Legacy(e) => {
                if matches!(
                    &column.representation,
                    Representation::RelatedPaths { .. }
                        | Representation::Scalar {
                            path_target: Some(_),
                            ..
                        }
                ) {
                    return Err(fail(
                        "WFT-EMIT",
                        "Forged path result on ordinary expression",
                    ));
                }
                validate_legacy(plan, e, column, qualifications)?;
                if matches!(e, crate::arithmetic_plan::Expression::Count { .. })
                    && plan.expansion().is_some()
                {
                    count_kinds.insert("pathRows");
                }
            }
        }
    }
    if let Some(e) = plan.expansion() {
        paths.push(json!({"path":e.path.serialized(),"edgeEncoding":"signed64-decimal/0.1"}))
    }
    if paths.is_empty()
        && emitted
            .obligations
            .iter()
            .any(|o| o.id == "ashlar.path.occurrenceIntegrity")
    {
        return Err(fail(
            "WFT-OBLIGATION",
            "No selected path for occurrence integrity",
        ));
    }
    if count_kinds.is_empty()
        && emitted
            .obligations
            .iter()
            .any(|o| o.id == "ashlar.path.countCapacity")
    {
        return Err(fail(
            "WFT-OBLIGATION",
            "No selected path count for capacity",
        ));
    }
    if !paths.is_empty() {
        let o = emitted
            .obligations
            .iter()
            .find(|o| o.id == "ashlar.path.occurrenceIntegrity")
            .ok_or_else(|| fail("WFT-OBLIGATION", "Missing path integrity"))?;
        if o.parameters["paths"] != json!(paths) {
            return Err(fail("WFT-OBLIGATION", "Path inventory mismatch"));
        }
        let checks = o.parameters["checks"]
            .as_array()
            .ok_or_else(|| fail("WFT-OBLIGATION", "Missing path checks"))?;
        let schemas = o.parameters["edgeSchemas"]
            .as_array()
            .ok_or_else(|| fail("WFT-OBLIGATION", "Missing native schema descriptors"))?;
        if schemas.len() != paths.len() * 2 {
            return Err(fail("WFT-OBLIGATION", "Native schema inventory mismatch"));
        }

        for (index, path) in paths.iter().enumerate() {
            let kinds: std::collections::BTreeSet<_> = checks
                .iter()
                .filter(|c| c["pathIndex"] == json!(index))
                .filter_map(|c| c["kind"].as_str())
                .collect();
            let expected: std::collections::BTreeSet<_> = if path.get("bound").is_some() {
                vec!["edgeEncoding", "intermediateIdentity", "collectionEncoding"]
            } else {
                vec!["edgeEncoding", "intermediateIdentity"]
            }
            .into_iter()
            .collect();
            if kinds != expected
                || checks
                    .iter()
                    .filter(|c| c["pathIndex"] == json!(index))
                    .count()
                    != expected.len()
            {
                return Err(fail(
                    "WFT-OBLIGATION",
                    "Incomplete/duplicate path check coverage",
                ));
            }
            for hop in 0..2 {
                let schema = &schemas[index * 2 + hop];
                if schema["pathIndex"] != json!(index)
                    || schema["hop"] != json!(hop)
                    || schema["relationship"] != path["path"]["hops"][hop]["identity"]
                    || !edge_sources.iter().any(|e| {
                        same(&e.relationship, &schema["relationship"])
                            && same(&e.table, &schema["table"])
                            && e.identity_column == schema["identityColumn"]
                            && e.native_type == schema["nativeType"]
                    })
                {
                    return Err(fail("WFT-OBLIGATION", "Native schema pin/hop mismatch"));
                }
            }
        }
        if checks.len()
            != paths
                .iter()
                .map(|p| if p.get("bound").is_some() { 3 } else { 2 })
                .sum::<usize>()
        {
            return Err(fail("WFT-OBLIGATION", "Extra path checks"));
        }
    }
    if !count_kinds.is_empty() {
        let o = emitted
            .obligations
            .iter()
            .find(|o| o.id == "ashlar.path.countCapacity")
            .ok_or_else(|| fail("WFT-OBLIGATION", "Missing count capacity"))?;
        let e = plan.expansion().unwrap();
        let checks = o.parameters["checks"]
            .as_array()
            .ok_or_else(|| fail("WFT-OBLIGATION", "Invalid count checks"))?;
        let kinds: std::collections::BTreeSet<_> =
            checks.iter().filter_map(|c| c["kind"].as_str()).collect();
        if kinds != count_kinds
            || checks.len() != count_kinds.len()
            || checks.iter().any(|c| c["pathOccurrence"] != e.occurrence)
        {
            return Err(fail(
                "WFT-OBLIGATION",
                "Count occurrence/check inventory mismatch",
            ));
        }
    }
    let response = json!({"interfaceVersion":if plan.ir_version()=="weft-ir/0.4.1" {"weft-compile/0.4.1"}else{"weft-compile/0.4.0"},"status":"compiled","diagnostics":[],"dialect":if plan.ir_version()=="weft-ir/0.4.1" {"weft-sql/0.4.1"}else{"weft-sql/0.4.0"},"compilerVersion":"weft/0.1.0","modelPins":plan.pins(),"backend":{"backendId":manifest.backend_id,"backendVersion":manifest.backend_version,"targetProfile":target.id,"interfaceVersion":"weft-backend/0.3.0"},"targetContext":target,"bindingSha256":binding.sha256,"logicalPlan":plan.serialized(),"sql":emitted.sql,"parameters":emitted.parameters,"columns":emitted.columns,"obligations":emitted.obligations,"qualification":{"status":"candidate","evidence":[],"assumptions":[],"operations":qualifications}});
    if !validate_response_schema(&response) {
        return Err(fail(
            "WFT-EMIT",
            "Emission violates closed response04 schema",
        ));
    }
    Ok(())
}

fn scalar_matches(column: &Column, expected: &LogicalType) -> bool {
    let parts = match &column.representation {
        Representation::Scalar {
            logical_type,
            carrier,
            decoder,
            path_target: None,
        } => Some((logical_type, carrier, decoder)),
        Representation::Legacy(crate::backend::Representation::Scalar {
            logical_type,
            carrier,
            decoder,
        }) => Some((logical_type, carrier, decoder)),
        _ => None,
    };
    parts.is_some_and(|(t, c, d)| {
        t == expected
            && column.nullable == expected.nullable
            && matches!(
                (&expected.family, c, d),
                (
                    Family::String,
                    crate::backend::ScalarCarrier::Text,
                    crate::backend::ScalarDecoder::Text
                ) | (
                    Family::Boolean,
                    crate::backend::ScalarCarrier::Boolean | crate::backend::ScalarCarrier::Text,
                    crate::backend::ScalarDecoder::Boolean
                ) | (
                    Family::Integer,
                    crate::backend::ScalarCarrier::Text,
                    crate::backend::ScalarDecoder::ExactInteger
                ) | (
                    Family::Decimal,
                    crate::backend::ScalarCarrier::Text,
                    crate::backend::ScalarDecoder::ExactDecimal
                )
            )
    })
}
fn validate_legacy(
    plan: Plan04View<'_>,
    expression: &crate::arithmetic_plan::Expression,
    column: &Column,
    qualifications: &[crate::backend::Qualification],
) -> Result<()> {
    use crate::arithmetic_plan::Expression as E;
    let valid = match expression {
        E::Count { logical_type }
        | E::CountDistinct { logical_type, .. }
        | E::Sum { logical_type, .. } => scalar_matches(column, logical_type),
        E::Arithmetic { expression } => {
            let (family, facets) = match expression.domain {
                crate::arithmetic_resolve::Domain::Integer => (Family::Integer, json!({})),
                crate::arithmetic_resolve::Domain::Decimal { scale } => {
                    (Family::Decimal, json!({"scale":scale}))
                }
            };
            scalar_matches(
                column,
                &LogicalType {
                    family,
                    facets,
                    nullable: false,
                },
            )
        }
        E::RelatedKeys {
            relationship,
            bound,
            ..
        } => {
            matches!(&column.representation,Representation::Legacy(crate::backend::Representation::RelatedKeys {relationship:r,key,bound:b}) if r==&relationship.identity && key.id==relationship.target_key.id && key.fields==relationship.target_key.fields && key.types==relationship.target_key.types && b==bound && !column.nullable)
        }
        E::Field { scan, identity } => {
            let descriptor = plan
                .type_graph()
                .iter()
                .find(|d| &d.identity == identity)
                .ok_or_else(|| fail("WFT-EMIT", "Selected Field descriptor missing"))?;
            let outer = plan
                .joins()
                .iter()
                .find(|j| {
                    j.right.occurrence == *scan
                        && j.kind == Some(crate::arithmetic_plan::JoinKind::Left)
                })
                .map(|j| crate::backend::OuterJoin {
                    scan: scan.clone(),
                    record: j.right.record.clone(),
                });
            if column.carrier_name.is_some()
                && column.source_identities.as_slice() != std::slice::from_ref(identity)
            {
                return Err(fail("WFT-EMIT", "Positioned Field lineage mismatch"));
            }
            match (&descriptor.shape, &column.representation) {
                (crate::application_model::Shape::Scalar { logical_type }, _)
                    if descriptor.availability.as_deref() == Some("required")
                        && outer.is_none() =>
                {
                    scalar_matches(column, logical_type)
                }
                (
                    _,
                    Representation::Legacy(crate::backend::Representation::Value {
                        descriptor: d,
                        native_null,
                        outer_join,
                    }),
                ) => {
                    d == identity
                        && outer_join == &outer
                        && !column.nullable
                        && (outer.is_none()
                            || !native_null
                            || descriptor.availability.as_deref() == Some("absent-allowed"))
                        && (!native_null
                            || qualifications.iter().any(|q| {
                                q.assessment.id == "value.nativeNull"
                                    && q.assessment.status != Status::Unsupported
                            }))
                }
                _ => false,
            }
        }
    };
    if valid {
        Ok(())
    } else {
        Err(fail(
            "WFT-EMIT",
            "Ordinary result changes original type/presence/relationship codec",
        ))
    }
}

fn validate_parameter(
    p: &ParameterSlot,
    qualifications: &[crate::backend::Qualification],
) -> Result<()> {
    let admitted = |ids: &[&str]| {
        qualifications.iter().any(|q| {
            ids.contains(&q.assessment.id.as_str()) && q.assessment.status != Status::Unsupported
        })
    };
    if p.logical_type.family == Family::Decimal && p.logical_type.facets.get("precision").is_none()
    {
        let scale = p.logical_type.facets["scale"]
            .as_u64()
            .ok_or_else(|| fail("WFT-EMIT", "Exact decimal scale missing"))?;
        if !admitted(&["arithmetic.exact.decimal"])
            || p.logical_type.facets != json!({"scale":scale})
        {
            return Err(fail(
                "WFT-EMIT",
                "Unbounded decimal parameter lacks admission",
            ));
        }
        let domain = crate::arithmetic_resolve::token_domain(
            &p.value,
            Some(&Family::Decimal),
            &crate::ir::Span { start: 0, end: 0 },
        )
        .map_err(|_| fail("WFT-EMIT", "Decimal token is not exact base ten"))?;
        if domain != (crate::arithmetic_resolve::Domain::Decimal { scale }) {
            return Err(fail("WFT-EMIT", "Decimal token scale changed"));
        }
        Ok(())
    } else {
        validate_parameter_domain(
            &p.value,
            &p.logical_type,
            admitted(&[
                "type.integer.unbounded",
                "arithmetic.exact.integer",
                "aggregate.havingCountDistinctGreater",
                "aggregate.havingCountStarGreater",
            ]),
        )
    }
}
fn validate_parameter_domain(value: &str, t: &LogicalType, unbounded_admitted: bool) -> Result<()> {
    if t.family == Family::Integer && t.facets == serde_json::json!({}) && !unbounded_admitted {
        return Err(fail(
            "WFT-EMIT",
            "Integer parameter needs an explicit bounded width",
        ));
    }
    let kind = match t.family {
        Family::String => {
            if value.contains('\0') {
                return Err(fail("WFT-EMIT", "String parameter contains NUL"));
            }
            crate::syntax::LiteralKind::String
        }
        Family::Boolean => {
            if value != "true" && value != "false" {
                return Err(fail(
                    "WFT-EMIT",
                    "Boolean parameter is not canonical exact text",
                ));
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
                return Err(fail(
                    "WFT-EMIT",
                    "Numeric parameter is not exact base-ten text",
                ));
            }
            if t.family == Family::Integer && t.facets != serde_json::json!({}) {
                let w = &t.facets["integerWidth"];
                if !w["bits"].as_u64().is_some_and(|b| (1..=64).contains(&b))
                    || !w["signed"].is_boolean()
                {
                    return Err(fail(
                        "WFT-EMIT",
                        "Integer parameter needs an explicit bounded width",
                    ));
                }
            } else if t.family == Family::Decimal {
                let p = t.facets["precision"].as_u64();
                let s = t.facets["scale"].as_u64();
                if !p.is_some_and(|p| (1..=28).contains(&p)) || !s.is_some_and(|s| s <= p.unwrap())
                {
                    return Err(fail(
                        "WFT-EMIT",
                        "Decimal parameter needs an explicit exact domain",
                    ));
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
    .map_err(|_| {
        fail(
            "WFT-EMIT",
            "Backend parameter value exceeds its declared logical domain",
        )
    })?;
    Ok(())
}
