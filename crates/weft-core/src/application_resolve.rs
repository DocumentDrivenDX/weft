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
#[derive(Debug,Clone,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ExactParameter { pub scalar_type:crate::application_model::ExactScalarType, pub value:String }
pub type ExactParameters=BTreeMap<String,ExactParameter>;

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
    resolve_inner(catalog,q,parameters,profile,ExactParameters::new(),false,false,false)
}
pub(crate) fn resolve_with_exact(catalog:&Catalog,q:ast::Query,parameters:Parameters,exact_parameters:ExactParameters,profile:Option<ir::ReadProfile>,timestamp_equality:bool)->Result<ir::Plan> {
    resolve_inner(catalog,q,parameters,profile,exact_parameters,true,timestamp_equality,false)
}
pub(crate) fn resolve_bounded_current_strings(catalog:&Catalog,q:ast::Query,parameters:Parameters)->Result<ir::Plan> {
    resolve_inner(catalog,q,parameters,None,ExactParameters::new(),false,false,true)
}
fn resolve_inner(
    catalog: &Catalog,
    q: ast::Query,
    parameters: Parameters,
    profile: Option<ir::ReadProfile>,
    exact_parameters:ExactParameters,
    binary_equality:bool,
    timestamp_equality:bool,
    bounded_current_strings:bool,
) -> Result<ir::Plan> {
    if catalog.bounded_current_strings != bounded_current_strings {
        return Err(fail("WFT-VERSION", "Catalog requires its explicitly versioned bounded-string resolver"));
    }
    let complete_entity = profile.as_ref().is_some_and(|p| p.subset == ir::Subset::CompleteEntity);
    if profile.as_ref().is_some_and(|p| match p.subset {
        ir::Subset::CompleteEntity => !matches!(p.version.as_str(), "weft-truss-entity-read/0.1.0" | "weft-truss-entity-read/0.2.0" | "weft-truss-entity-read/0.3.0"),
        _ => p.version != "weft-application-read/0.2.0",
    }) {
        return Err(fail("WFT-VERSION", "Unsupported application recognizer profile version"));
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
    let mut normalized_exact=ExactParameters::new();
    if normalized.len()+exact_parameters.len()>1024 {return Err(fail("WFT-LIMIT","Combined source parameter count exceeds1024"));}
    for (name,value) in exact_parameters {
        if !name.as_bytes().first().is_some_and(|b|b.is_ascii_alphabetic()||*b==b'_') || !name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_')
            || normalized.contains_key(&name.to_ascii_lowercase()) || normalized_exact.insert(name.to_ascii_lowercase(),value).is_some() {
            return Err(fail("WFT-PARAMETER","Parameter names must be distinct across atomic and exact maps"));
        }
    }
    let mut s = Scope {
        catalog,
        records: vec![],
        parameters: normalized,
        exact_parameters:normalized_exact,
        binary_equality,
        timestamp_equality,
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
        .any(|p| matches!(p.output, ast::Output::Count | ast::Output::CountDistinct(_) | ast::Output::Sum(_)))
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
                let record_identity = r.identity.clone();
                let relationships = if complete_entity { catalog.entity_relationships(r)? } else { vec![] };
                let e = catalog.entity_descriptor(r)?;
                s.descriptors(e.graph);
                s.caps.insert("project.entity".into());
                let mut expressions = e
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
                if complete_entity {
                    for (name, metadata, family, nullable) in [
                        ("_version", ir::NativeMetadata::Version, Family::Integer, false),
                        ("_created_by", ir::NativeMetadata::CreatedBy, Family::String, false),
                        ("_updated_by", ir::NativeMetadata::UpdatedBy, Family::String, false),
                        ("_original_author", ir::NativeMetadata::OriginalAuthor, Family::String, true),
                        ("_original_at", ir::NativeMetadata::OriginalAt, Family::String, true),
                    ] {
                        if expressions.iter().any(|(label, _)| label.eq_ignore_ascii_case(name)) {
                            return Err(fail("WFT-NAME-AMBIGUOUS", "Authored member collides with Truss entity metadata"));
                        }
                        expressions.push((name.into(), ir::Expression::NativeMetadata {
                            scan: scan.clone(), record: record_identity.clone(), metadata,
                            logical_type: LogicalType { family, facets: json!({}), nullable },
                        }));
                    }
                    if profile.as_ref().is_some_and(|p| matches!(p.version.as_str(),"weft-truss-entity-read/0.2.0"|"weft-truss-entity-read/0.3.0")) {
                        for (name, metadata) in [("_created_at",ir::NativeMetadata::CreatedAt),("_updated_at",ir::NativeMetadata::UpdatedAt)] {
                            if expressions.iter().any(|(label,_)|label.eq_ignore_ascii_case(name)) { return Err(fail("WFT-NAME-AMBIGUOUS","Authored member collides with Truss entity timestamp")); }
                            expressions.push((name.into(),ir::Expression::NativeMetadata{scan:scan.clone(),record:record_identity.clone(),metadata,logical_type:LogicalType{family:Family::String,facets:json!({}),nullable:false}}));
                        }
                    }
                    if profile.as_ref().is_some_and(|p|p.version=="weft-truss-entity-read/0.3.0") {
                        if expressions.iter().any(|(label,_)|label.eq_ignore_ascii_case("_load_id")) { return Err(fail("WFT-NAME-AMBIGUOUS","Authored member collides with Truss import load metadata")); }
                        expressions.push(("_load_id".into(),ir::Expression::NativeMetadata{scan:scan.clone(),record:record_identity.clone(),metadata:ir::NativeMetadata::LoadId,logical_type:LogicalType{family:Family::String,facets:json!({}),nullable:true}}));
                    }
                    s.caps.insert("project.nativeMetadata".into());
                    for (name, relationship) in relationships {
                        s.relationship_capabilities(&relationship);
                        if expressions.iter().any(|(label, _)| label.eq_ignore_ascii_case(&name)) {
                            return Err(fail("WFT-NAME-AMBIGUOUS", "Relationship label collides with an entity output"));
                        }
                        if relationship.inverse { s.caps.insert("relationship.inverse".into()); }
                        expressions.push((name, ir::Expression::CompleteRelatedKeys { scan: scan.clone(), relationship, bound: 100 }));
                        s.caps.insert("relationship.completeKeys".into());
                    }
                }
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
            ast::Output::CountDistinct(c)=>{
                let argument=s.field(c)?;
                let (record,_)=s.record_for_column(c)?;let (_,graph)=catalog.member_descriptor_by_identity(record,&argument.identity)?;
                if graph.iter().find(|d|d.identity==argument.identity).and_then(|d|d.availability.as_deref())!=Some("required"){return Err(fail("WFT-TYPE","COUNT DISTINCT requires established present scalar operands; absence is not NULL"));}
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
                s.relationship_capabilities(&rel);
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
            ir::Subset::CompleteEntity => {
                if aggregate || !joins.is_empty() || !q.outputs.iter().any(|p| matches!(p.output, ast::Output::Entity(_))) {
                    return Err(fail("WFT-PROFILE", "Complete entity requires a nonaggregated single-source entity projection"));
                }
            }
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
    if s.used.len() != s.parameters.len()+s.exact_parameters.len() {
        return Err(fail("WFT-PARAMETER", "Surplus source parameter binding"));
    }
    s.caps.insert("project".into());
    let extended=q.limit==Some(0)||q.predicates.iter().any(|p|matches!(p,ast::Predicate::Boolean{..}))||q.outputs.iter().any(|o|matches!(o.output,ast::Output::CountDistinct(_)));
    Ok(ir::Plan {
        ir_version: if extended {"weft-ir/application-lineage/0.2-candidate"}else if timestamp_equality { "weft-ir/0.2.6" } else if binary_equality { "weft-ir/0.2.5" } else if profile.as_ref().is_some_and(|p|p.version=="weft-truss-entity-read/0.3.0") { "weft-ir/0.2.4" } else if profile.as_ref().is_some_and(|p|p.version=="weft-truss-entity-read/0.2.0") { "weft-ir/0.2.3" } else if complete_entity { "weft-ir/0.2.2" } else { "weft-ir/0.2.0" }.into(),
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

/// Resolved page with compiler-owned hidden authored key outputs.
/// The caller must remove exactly these trailing outputs after keyset lookahead.
/// Generated AST spans are diagnostic anchors, not authored SQL provenance.
/// This source increment is not a public page compiler: page emission must label
/// continuation bindings explicitly before exposing a backend artifact.
pub struct PagePlan {
    pub plan: ir::Plan,
    pub visible_outputs: usize,
    pub key_outputs: Vec<usize>,
    pub page_size: u16,
    pub typed_keys: bool,
    /// Exact generated continuation values; never authored SQL provenance.
    pub continuation_values: Vec<ir::Value>,
}

pub fn resolve_page(
    catalog: &Catalog,
    query: ast::Query,
    parameters: Parameters,
    page_size: u16,
    after: Option<&[String]>,
) -> Result<PagePlan> {
    resolve_page_inner(catalog,query,parameters,page_size,after,false,ExactParameters::new(),false,false,false)
}

pub fn resolve_typed_page(catalog:&Catalog,query:ast::Query,parameters:Parameters,page_size:u16,after:Option<&[String]>) -> Result<PagePlan> {
    resolve_page_inner(catalog,query,parameters,page_size,after,true,ExactParameters::new(),false,false,false)
}

pub(crate) fn resolve_page_with_exact(catalog:&Catalog,query:ast::Query,parameters:Parameters,exact_parameters:ExactParameters,page_size:u16,after:Option<&[String]>,typed_keys:bool,timestamp_equality:bool,decimal12:bool)->Result<PagePlan> {
    resolve_page_inner(catalog,query,parameters,page_size,after,typed_keys,exact_parameters,true,timestamp_equality,decimal12)
}
fn resolve_page_inner(catalog:&Catalog,mut query:ast::Query,parameters:Parameters,page_size:u16,after:Option<&[String]>,typed_keys:bool,exact_parameters:ExactParameters,binary_equality:bool,timestamp_equality:bool,decimal12:bool) -> Result<PagePlan> {
    if !(1..=999).contains(&page_size) || !query.order.is_empty() || query.limit.is_some()
        || !query.joins.is_empty() || !query.groups.is_empty()
        || query.outputs.iter().any(|p| matches!(p.output, ast::Output::Count | ast::Output::CountDistinct(_) | ast::Output::Sum(_) | ast::Output::Related { .. }))
    {
        return Err(fail("WFT-PROFILE", "SQL page requires a single nonaggregated source without caller ORDER BY or LIMIT"));
    }
    let record = catalog.record(query.source.namespace.as_ref(), &query.source.name)?;
    let primary = record.value["keys"].as_array().ok_or_else(|| fail("WFT-PROFILE", "SQL page requires an authored primary key"))?
        .iter().filter(|key| key["primary"] == true).collect::<Vec<_>>();
    if primary.len() != 1 { return Err(fail("WFT-PROFILE", "SQL page requires exactly one authored primary key")); }
    let key = catalog.authored_key(&record, primary[0]["id"].as_str().unwrap())?;
    if key.fields.is_empty() || key.types.iter().any(|t| t.nullable || match t.family { Family::String => t.facets != json!({}), Family::Boolean => !typed_keys || t.facets != json!({}), Family::Integer => !typed_keys || (t.facets != json!({}) && t.facets != json!({"integerWidth":{"bits":64,"signed":true}})), Family::Decimal => !typed_keys || (t.facets != json!({"precision":18,"scale":4}) && !(binary_equality && t.facets == json!({"precision":18,"scale":2})) && !(decimal12 && t.facets == json!({"precision":12,"scale":2}))) }) {
        return Err(fail("WFT-PROFILE", "SQL page requires complete required-string key components"));
    }
    let mut columns = Vec::new();
    for (index, identity) in key.fields.iter().enumerate() {
        let (member, _) = catalog.member_descriptor_by_identity(&record, identity)?;
        let column = Column { unqualified: false, alias: query.source.alias.clone(),
            field: Name { value: member.name, quoted: true, span: query.source.name.span.clone() }, span: query.source.name.span.clone() };
        query.outputs.push(ast::Projection { output: ast::Output::Field(column.clone()),
            alias: Some(Name { value: format!("__weft_page_key_{index}"), quoted: true, span: query.source.name.span.clone() }) });
        columns.push(column);
    }
    if let Some(after) = after {
        if after.len() != columns.len() { return Err(fail("WFT-TYPE", "SQL page continuation key arity differs")); }
        query.predicates.push(ast::Predicate::Compare { columns: columns.clone(), greater: true,
            values: after.iter().zip(&key.types).map(|(value,t)| ast::Value::Literal(Literal { value: value.clone(), kind: match t.family { Family::String=>LiteralKind::String,Family::Boolean=>LiteralKind::Boolean,_=>LiteralKind::Number },
                span: query.source.name.span.clone() })).collect() });
    }
    query.order = columns;
    query.limit = Some(page_size + 1);
    let mut plan = resolve_inner(catalog, query, parameters, Some(ir::ReadProfile { version: "weft-application-read/0.2.0".into(), subset: ir::Subset::EntityPage }),exact_parameters,binary_equality,timestamp_equality,false)?;
    plan.ir_version = if timestamp_equality { "weft-ir/0.2.6" } else if binary_equality { "weft-ir/0.2.5" } else { "weft-ir/0.2.2" }.into();
    let visible_outputs = plan.outputs.len() - key.fields.len();
    let key_outputs = (visible_outputs..plan.outputs.len()).collect();
    let continuation_values = if after.is_some() {
        match plan.filters.last() {
            Some(ir::Predicate::LexicographicGreater { values, .. }) => values.clone(),
            _ => return Err(fail("WFT-IR", "Generated continuation predicate missing")),
        }
    } else { Vec::new() };
    Ok(PagePlan { plan, visible_outputs, key_outputs, page_size, typed_keys, continuation_values })
}
