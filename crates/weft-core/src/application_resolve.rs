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
struct Scope<'a> {
    catalog: &'a Catalog,
    records: Vec<(Name, Record, String)>,
    parameters: Parameters,
    used: BTreeSet<String>,
    caps: BTreeSet<String>,
    graph: Vec<Descriptor>,
}
impl<'a> Scope<'a> {
    fn source(&mut self, s: &ast::Query) -> Result<ir::Scan> {
        self.add_source(&s.source)
    }
    fn add_source(&mut self, s: &crate::syntax::Source) -> Result<ir::Scan> {
        if self
            .records
            .iter()
            .any(|(n, _, _)| n.value == s.alias.value)
        {
            return Err(fail("WFT-NAME-AMBIGUOUS", "Repeated source alias").at(&s.alias.span));
        }
        let record = self.catalog.record(s.namespace.as_ref(), &s.name)?;
        let occurrence = format!("s{}", self.records.len());
        let scan = ir::Scan {
            occurrence: occurrence.clone(),
            record: record.identity.clone(),
            pin: record.pin.clone(),
        };
        self.records.push((s.alias.clone(), record, occurrence));
        self.caps.insert("scan".into());
        Ok(scan)
    }
    fn record(&self, n: &Name) -> Result<(&Record, String)> {
        let (_, r, s) = self
            .records
            .iter()
            .find(|(alias, _, _)| alias.value == n.value)
            .ok_or_else(|| fail("WFT-NAME-MISSING", "Source alias is not in scope").at(&n.span))?;
        Ok((r, s.clone()))
    }
    fn field(&mut self, c: &Column) -> Result<ir::Field> {
        let (r, s) = self.record(&c.alias)?;
        let (id, t, _) = self.catalog.field(r, &c.field)?;
        if t.family == Family::Integer && t.facets == json!({}) { self.caps.insert("type.integer.unbounded".into()); }
        self.caps.insert(format!(
            "type.{}",
            match t.family {
                Family::Boolean => "boolean",
                Family::String => "string",
                Family::Integer => "integer",
                Family::Decimal => "decimal",
            }
        ));
        Ok(ir::Field {
            scan: s,
            identity: id,
            logical_type: t,
            span: c.span.clone(),
        })
    }
    fn descriptors(&mut self, graph: Vec<Descriptor>) {
        for d in graph {
            match &d.shape {
                Shape::Scalar { logical_type } => {
                    if logical_type.family == Family::Integer && logical_type.facets == json!({}) { self.caps.insert("type.integer.unbounded".into()); }
                    self.caps.insert(format!(
                        "type.{}",
                        match logical_type.family {
                            Family::Boolean => "boolean",
                            Family::String => "string",
                            Family::Integer => "integer",
                            Family::Decimal => "decimal",
                        }
                    ));
                }
                Shape::Sequence { .. } => {
                    self.caps.insert("value.sequence".into());
                }
                Shape::Map { .. } => {
                    self.caps.insert("value.map".into());
                }
                Shape::Structured { .. } => {
                    self.caps.insert("value.structured".into());
                }
                Shape::Record { .. } => {}
            }
            if d.availability.as_deref() == Some("absent-allowed") {
                self.caps.insert("value.presence".into());
            }
            if !self.graph.iter().any(|g| g.identity == d.identity) {
                self.graph.push(d);
            }
        }
    }
    fn value(&mut self, v: &ast::Value, t: &LogicalType) -> Result<ir::Value> {
        match v {
            ast::Value::Field(c) => {
                let f = self.field(c)?;
                if f.logical_type.family != t.family {
                    return Err(
                        fail("WFT-TYPE", "Compared fields require the same exact family")
                            .at(&c.span),
                    );
                }
                Ok(ir::Value::Field { field: f })
            }
            ast::Value::Literal(l) => {
                exact::literal(l, t)?;
                Ok(ir::Value::Literal {
                    value: l.value.clone(),
                    logical_type: t.clone(),
                    span: l.span.clone(),
                })
            }
            ast::Value::Parameter(n) => {
                if n.quoted {
                    return Err(fail(
                        "WFT-PARAMETER",
                        "Source parameter names must be unquoted ASCII identifiers",
                    )
                    .at(&n.span));
                }
                let p = self.parameters.get(&n.value).ok_or_else(|| {
                    fail("WFT-PARAMETER", "Missing source parameter binding").at(&n.span)
                })?;
                if p.family != t.family {
                    return Err(
                        fail("WFT-PARAMETER", "Source parameter family is incompatible")
                            .at(&n.span),
                    );
                }
                let kind = match p.family {
                    Family::String => LiteralKind::String,
                    Family::Boolean => LiteralKind::Boolean,
                    _ => LiteralKind::Number,
                };
                let lexical = match p.family {
                    Family::Boolean => p.value == "true" || p.value == "false",
                    Family::String => !p.value.contains('\0'),
                    _ => number_text(&p.value),
                };
                if !lexical {
                    return Err(fail(
                        "WFT-PARAMETER",
                        "Source parameter value violates the exact lexical grammar",
                    )
                    .at(&n.span));
                }
                exact::literal(
                    &Literal {
                        value: p.value.clone(),
                        kind,
                        span: n.span.clone(),
                    },
                    t,
                )?;
                self.used.insert(n.value.clone());
                self.caps.insert("parameter.named".into());
                Ok(ir::Value::Parameter {
                    name: n.value.clone(),
                    value: p.value.clone(),
                    logical_type: t.clone(),
                    span: n.span.clone(),
                })
            }
        }
    }
    fn predicate(&mut self, p: &ast::Predicate, on: bool) -> Result<ir::Predicate> {
        match p {
            ast::Predicate::Compare {
                columns,
                values,
                greater,
            } => {
                let fields = columns
                    .iter()
                    .map(|c| self.field(c))
                    .collect::<Result<Vec<_>>>()?;
                if on && (*greater || !matches!(values.as_slice(), [ast::Value::Field(_)])) {
                    return Err(fail("WFT-UNSUPPORTED", "JOIN ON requires field equality"));
                }
                let values = values
                    .iter()
                    .zip(&fields)
                    .map(|(v, f)| self.value(v, &f.logical_type))
                    .collect::<Result<Vec<_>>>()?;
                self.caps.insert(
                    if *greater {
                        "compare.lexicographicGreater"
                    } else {
                        "equal"
                    }
                    .into(),
                );
                if *greater {
                    Ok(ir::Predicate::LexicographicGreater {
                        columns: fields,
                        values,
                    })
                } else {
                    Ok(ir::Predicate::Equal {
                        left: fields.into_iter().next().unwrap(),
                        right: values.into_iter().next().unwrap(),
                    })
                }
            }
            ast::Predicate::HasRelated { relationship, key } => {
                if on {
                    return Err(fail("WFT-UNSUPPORTED", "HAS_RELATED is a WHERE predicate"));
                }
                let (r, scan) = self.record(&relationship.alias)?;
                let rel = self.catalog.relationship_read(r, &relationship.field)?;
                if key.len() != rel.target_key.types.len() {
                    return Err(fail("WFT-TYPE", "Related key tuple arity mismatch"));
                }
                if key.iter().any(|v| matches!(v, ast::Value::Field(_))) {
                    return Err(fail(
                        "WFT-UNSUPPORTED",
                        "Related key operands require literals or parameters",
                    ));
                }
                let key = key
                    .iter()
                    .zip(&rel.target_key.types)
                    .map(|(v, t)| self.value(v, t))
                    .collect::<Result<Vec<_>>>()?;
                self.caps.insert("relationship.exists".into());
                if rel.inverse {
                    self.caps.insert("relationship.inverse".into());
                }
                Ok(ir::Predicate::HasRelated {
                    scan,
                    relationship: rel,
                    key,
                })
            }
        }
    }
}
fn number_text(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    let mut parts = s.split('.');
    let whole = parts.next().unwrap_or("");
    !whole.is_empty()
        && whole.bytes().all(|b| b.is_ascii_digit())
        && match parts.next() {
            None => true,
            Some(f) => {
                !f.is_empty() && f.bytes().all(|b| b.is_ascii_digit()) && parts.next().is_none()
            }
        }
}
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
