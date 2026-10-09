use crate::{
    application_ir as ir,
    application_model::{Descriptor, Shape},
    application_syntax as ast,
    error::{Diagnostic, Result},
    exact,
    ir::{Family, Identity, LogicalType},
    model::{Catalog, Record},
    syntax::{Column, Literal, LiteralKind, Name},
};
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    pub family: Family,
    pub value: String,
}
pub type Parameters = BTreeMap<String, Parameter>;
fn fail(code: &str, message: &str) -> Diagnostic {
    Diagnostic::new(code, "resolve", message)
}
use crate::application_scope::Scope;
fn same(a: &ir::Field, b: &ir::Field) -> bool {
    a.scan == b.scan && a.identity == b.identity
}
fn signature(fields: &[ir::Field]) -> Vec<(String, Identity)> {
    fields
        .iter()
        .map(|f| (f.scan.clone(), f.identity.clone()))
        .collect()
}
pub fn resolve(
    catalog: &Catalog,
    q: ast::Query,
    parameters: Parameters,
    profile: Option<ir::ReadProfile>,
) -> Result<ir::Plan> {
    if profile
        .as_ref()
        .is_some_and(|p| p.version != "weft-application-read/0.2.0")
    {
        return Err(fail(
            "WFT-VERSION",
            "Unsupported application recognizer profile version",
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
    let source = s.source(&q)?;
    let mut joins = vec![];
    for (right, on) in &q.joins {
        let right = s.add_source(right)?;
        let on = on
            .iter()
            .map(|p| s.predicate(p, true))
            .collect::<Result<Vec<_>>>()?;
        s.caps.insert("innerJoin".into());
        joins.push(ir::Join { right, on });
    }
    let filters = q
        .predicates
        .iter()
        .map(|p| s.predicate(p, false))
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
        .any(|p| matches!(p.output, ast::Output::Count | ast::Output::Sum(_)))
        || !groups.is_empty();
    if aggregate {
        s.caps.insert("aggregate".into());
    }
    if !groups.is_empty() {
        s.caps.insert("group".into());
    }
    let mut outputs = vec![];
    let mut labels = BTreeSet::new();
    for p in &q.outputs {
        let (default, expressions) = match &p.output {
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
                let (r, scan) = s.record(&c.alias)?;
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
            if !labels.insert(name.clone()) {
                return Err(fail("WFT-OUTPUT-NAME", "Repeated output name"));
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
    let mut page_key = None;
    if let Some(p) = &profile {
        match p.subset {
            ir::Subset::CountSummary => {
                if !q
                    .outputs
                    .iter()
                    .any(|p| matches!(p.output, ast::Output::Count))
                    || q.outputs.iter().any(|p| {
                        matches!(
                            p.output,
                            ast::Output::Sum(_)
                                | ast::Output::Entity(_)
                                | ast::Output::Related { .. }
                        )
                    })
                {
                    return Err(fail(
                        "WFT-PROFILE",
                        "count-summary requires COUNT(*) and optional grouped scalar projections",
                    ));
                }
                if filters
                    .iter()
                    .any(|f| !matches!(f, ir::Predicate::Equal { .. }))
                {
                    return Err(fail(
                        "WFT-PROFILE",
                        "count-summary excludes cursor and relationship predicates",
                    ));
                }
                if groups.is_empty() {
                    if !order.is_empty() || q.limit.is_some() {
                        return Err(fail(
                            "WFT-PROFILE",
                            "Global count-summary has one row and excludes ORDER BY and LIMIT",
                        ));
                    }
                } else if signature(&order) != signature(&groups) || q.limit.is_none() {
                    return Err(fail(
                        "WFT-PROFILE",
                        "Grouped count-summary requires complete grouping order and LIMIT",
                    ));
                }
            }
            ir::Subset::EntityPage | ir::Subset::RelatedEntityPage => {
                if aggregate || !joins.is_empty() {
                    return Err(fail(
                        "WFT-PROFILE",
                        "Entity-page profiles require a single nonaggregated source",
                    ));
                }
                let related = filters
                    .iter()
                    .any(|f| matches!(f, ir::Predicate::HasRelated { .. }))
                    || outputs
                        .iter()
                        .any(|o| matches!(o.expression, ir::Expression::RelatedKeys { .. }));
                if p.subset == ir::Subset::EntityPage && related {
                    return Err(fail(
                        "WFT-PROFILE",
                        "entity-page excludes relationship reads",
                    ));
                }
                if p.subset == ir::Subset::RelatedEntityPage && !related {
                    return Err(fail(
                        "WFT-PROFILE",
                        "related-entity-page requires a relationship read",
                    ));
                }
                let record = &s.records[0].1;
                let keys = record.value["keys"]
                    .as_array()
                    .ok_or_else(|| fail("WFT-PROFILE", "Entity page needs an authored key"))?;
                let mut qualifying = vec![];
                for key in keys {
                    let k = catalog.authored_key(record, key["id"].as_str().unwrap())?;
                    if order.iter().all(|f| f.scan == source.occurrence)
                        && k.fields == order.iter().map(|f| f.identity.clone()).collect::<Vec<_>>()
                    {
                        qualifying.push(k);
                    }
                }
                if qualifying.len() != 1 || q.limit.is_none() {
                    return Err(fail(
                        "WFT-PROFILE",
                        "Entity page requires unambiguous complete authored key order and LIMIT",
                    ));
                }
                for f in &filters {
                    if let ir::Predicate::LexicographicGreater { columns, values } = f {
                        if signature(columns) != signature(&order)
                            || values.iter().any(|v| matches!(v, ir::Value::Field { .. }))
                        {
                            return Err(fail("WFT-PROFILE","Cursor must compare the complete ordered key to literal or parameter tuple"));
                        }
                    }
                }
                page_key = qualifying.into_iter().next();
                s.caps.insert("key.uniqueStable".into());
            }
        }
    }
    if s.used.len() != s.parameters.len() {
        return Err(fail("WFT-PARAMETER", "Surplus source parameter binding"));
    }
    s.caps.insert("project".into());
    Ok(ir::Plan {
        ir_version: "weft-ir/0.2.0".into(),
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
