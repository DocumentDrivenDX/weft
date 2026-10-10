//! Owned 0.4 path plan foundation. Typed consistency is not model or native admission.
use crate::{
    application_ir::{Field, Scan},
    application_model::{AuthoredKey, Descriptor, RelationshipRead},
    arithmetic_plan::{Join, JoinKind, Predicate},
    ir::{Family, Identity, LogicalType, ModelPin, Span},
};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invalid {
    EmptyOccurrence,
    InvalidSpan,
    DiscontinuousPath,
    InvalidBound,
    WrongRoot,
    DuplicateOccurrence,
    InvalidStage,
    InvalidCount,
    InvalidPins,
    InvalidShape,
}
#[jsonschema::validator(
    path = "../../docs/helix/02-design/contracts/logical-plan-v0.4.schema.json"
)]
struct PlanSchema04;
type Result<T> = std::result::Result<T, Invalid>;
fn span(s: &Span) -> bool {
    s.start <= s.end
}
fn integer() -> LogicalType {
    LogicalType {
        family: Family::Integer,
        facets: serde_json::json!({}),
        nullable: false,
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathRead {
    start_scan: String,
    hops: [RelationshipRead; 2],
    span: Span,
    hop_spans: [Span; 2],
}
impl PathRead {
    pub fn new(
        start_scan: String,
        hops: [RelationshipRead; 2],
        expression_span: Span,
        hop_spans: [Span; 2],
    ) -> Result<Self> {
        if start_scan.is_empty() {
            return Err(Invalid::EmptyOccurrence);
        }
        if hop_spans[0].end > hop_spans[1].start
            || !span(&expression_span)
            || hop_spans
                .iter()
                .any(|s| !span(s) || s.start < expression_span.start || s.end > expression_span.end)
        {
            return Err(Invalid::InvalidSpan);
        }
        if hops[0].to != hops[1].from {
            return Err(Invalid::DiscontinuousPath);
        }
        for h in &hops {
            if h.identity.document_id != h.from.document_id
                || h.identity.revision != h.from.revision
                || h.from.document_id != h.to.document_id
                || h.from.revision != h.to.revision
            {
                return Err(Invalid::DiscontinuousPath);
            }
            for (key, endpoint) in [(&h.source_key, &h.from), (&h.target_key, &h.to)] {
                if key.id.is_empty()
                    || key.fields.is_empty()
                    || key.fields.len() != key.types.len()
                    || key.fields.iter().any(|f| {
                        f.document_id != endpoint.document_id
                            || f.revision != endpoint.revision
                            || f.element.is_empty()
                    })
                {
                    return Err(Invalid::DiscontinuousPath);
                }
            }
        }
        Ok(Self {
            start_scan,
            hops,
            span: expression_span,
            hop_spans,
        })
    }
    pub(crate) fn span(&self) -> &Span {
        &self.span
    }
    pub(crate) fn hop_spans(&self) -> &[Span; 2] {
        &self.hop_spans
    }
    pub fn start_scan(&self) -> &str {
        &self.start_scan
    }
    pub fn hops(&self) -> &[RelationshipRead; 2] {
        &self.hops
    }
    pub fn start_record(&self) -> &Identity {
        &self.hops[0].from
    }
    pub fn terminal_record(&self) -> &Identity {
        &self.hops[1].to
    }
    pub fn terminal_key(&self) -> &AuthoredKey {
        &self.hops[1].target_key
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathExpansion {
    occurrence: String,
    path: PathRead,
}
impl PathExpansion {
    pub fn new(occurrence: String, path: PathRead) -> Result<Self> {
        if occurrence.is_empty() {
            Err(Invalid::EmptyOccurrence)
        } else {
            Ok(Self { occurrence, path })
        }
    }
    pub fn occurrence(&self) -> &str {
        &self.occurrence
    }
    pub fn path(&self) -> &PathRead {
        &self.path
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PathExpression {
    RelatedPaths {
        path: PathRead,
        bound: u16,
    },
    CountDistinctPathTargets {
        #[serde(rename = "pathOccurrence")]
        path_occurrence: String,
        #[serde(rename = "type")]
        logical_type: LogicalType,
    },
}
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Expression {
    Legacy(crate::arithmetic_plan::Expression),
    Path(PathExpression),
}
impl Expression {
    pub fn related_paths(path: PathRead, bound: u16) -> Result<Self> {
        if !(1..=1000).contains(&bound) {
            return Err(Invalid::InvalidBound);
        }
        Ok(Self::Path(PathExpression::RelatedPaths { path, bound }))
    }
    pub fn count_distinct_path_targets(path_occurrence: String) -> Result<Self> {
        if path_occurrence.is_empty() {
            return Err(Invalid::EmptyOccurrence);
        }
        Ok(Self::Path(PathExpression::CountDistinctPathTargets {
            path_occurrence,
            logical_type: integer(),
        }))
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Output {
    pub name: String,
    pub expression: Expression,
}
#[derive(Debug, Clone, Serialize)]
pub struct Having {
    pub count: Expression,
    pub threshold: crate::application_ir::Value,
}
/// Resolver-owned inputs; only Plan::new produces a released typed plan.
#[derive(Debug, Clone)]
pub struct PlanParts {
    pub distinct: bool,
    pub module_pins: Vec<ModelPin>,
    pub required_capabilities: Vec<String>,
    pub type_graph: Vec<Descriptor>,
    pub source: Scan,
    pub page_key: Option<AuthoredKey>,
    pub joins: Vec<Join>,
    pub filters: Vec<Predicate>,
    pub groups: Vec<Field>,
    pub having: Vec<Having>,
    pub aggregate: bool,
    pub outputs: Vec<Output>,
    pub order: Vec<Field>,
    pub limit: Option<u16>,
    pub path_expansion: Option<PathExpansion>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    distinct: bool,
    ir_version: &'static str,
    module_pins: Vec<ModelPin>,
    read_profile: Option<()>,
    required_capabilities: Vec<String>,
    type_graph: Vec<Descriptor>,
    source: Scan,
    page_key: Option<AuthoredKey>,
    joins: Vec<Join>,
    filters: Vec<Predicate>,
    groups: Vec<Field>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    having: Vec<Having>,
    aggregate: bool,
    outputs: Vec<Output>,
    order: Vec<Field>,
    limit: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path_expansion: Option<PathExpansion>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    outer_join_scans: Vec<String>,
}
impl Plan {
    pub fn new(p: PlanParts) -> Result<Self> {
        if p.page_key.is_some() {
            return Err(Invalid::InvalidStage);
        }
        let mut scans = BTreeSet::new();
        for s in std::iter::once(&p.source).chain(p.joins.iter().map(|j| &j.right)) {
            if s.record.document_id != s.pin.document_id || s.record.revision != s.pin.revision {
                return Err(Invalid::InvalidPins);
            }
            if s.occurrence.is_empty() {
                return Err(Invalid::EmptyOccurrence);
            }
            if !scans.insert(s.occurrence.as_str()) {
                return Err(Invalid::DuplicateOccurrence);
            }
            if !p.module_pins.iter().any(|pin| {
                pin.document_id == s.pin.document_id
                    && pin.revision == s.pin.revision
                    && pin.sha256 == s.pin.sha256
                    && pin.umf_version == s.pin.umf_version
            }) {
                return Err(Invalid::InvalidPins);
            }
        }
        if let Some(e) = &p.path_expansion {
            if scans.contains(e.occurrence()) {
                return Err(Invalid::DuplicateOccurrence);
            }
        }
        let check_path = |path: &PathRead| -> Result<()> {
            let root = std::iter::once(&p.source)
                .chain(p.joins.iter().map(|j| &j.right))
                .find(|s| s.occurrence == path.start_scan)
                .ok_or(Invalid::WrongRoot)?;
            if root.record != *path.start_record() {
                return Err(Invalid::WrongRoot);
            }
            Ok(())
        };
        if let Some(e) = &p.path_expansion {
            check_path(&e.path)?
        }
        if p.limit.is_some_and(|n| n > 1000) {
            return Err(Invalid::InvalidBound);
        }
        for expression in p
            .outputs
            .iter()
            .map(|o| &o.expression)
            .chain(p.having.iter().map(|h| &h.count))
        {
            match expression {
                Expression::Path(PathExpression::RelatedPaths { path, bound }) => {
                    check_path(path)?;
                    if !(1..=1000).contains(bound) {
                        return Err(Invalid::InvalidBound);
                    }
                    if p.path_expansion.is_some()
                        || p.aggregate
                        || !p.groups.is_empty()
                        || !p.having.is_empty()
                    {
                        return Err(Invalid::InvalidStage);
                    }
                }
                Expression::Path(PathExpression::CountDistinctPathTargets {
                    path_occurrence,
                    logical_type,
                }) => {
                    if !p.aggregate
                        || *logical_type != integer()
                        || p.path_expansion.as_ref().map(|e| e.occurrence())
                            != Some(path_occurrence.as_str())
                    {
                        return Err(Invalid::InvalidCount);
                    }
                }
                Expression::Legacy(
                    crate::arithmetic_plan::Expression::Count { logical_type }
                    | crate::arithmetic_plan::Expression::CountDistinct { logical_type, .. },
                ) => {
                    if !p.aggregate || *logical_type != integer() {
                        return Err(Invalid::InvalidCount);
                    }
                }
                Expression::Legacy(crate::arithmetic_plan::Expression::Sum { .. }) => {
                    if !p.aggregate {
                        return Err(Invalid::InvalidCount);
                    }
                }
                Expression::Legacy(crate::arithmetic_plan::Expression::RelatedKeys {
                    scan,
                    relationship,
                    bound,
                }) => {
                    if !(1..=1000).contains(bound)
                        || p.aggregate
                        || p.path_expansion.is_some()
                        || !p.groups.is_empty()
                        || !p.having.is_empty()
                    {
                        return Err(Invalid::InvalidStage);
                    }
                    let root = std::iter::once(&p.source)
                        .chain(p.joins.iter().map(|j| &j.right))
                        .find(|s| s.occurrence == *scan)
                        .ok_or(Invalid::WrongRoot)?;
                    if root.record != relationship.from {
                        return Err(Invalid::WrongRoot);
                    }
                }
                _ => {}
            }
        }
        let outer_join_scans = p
            .joins
            .iter()
            .filter(|j| j.kind == Some(JoinKind::Left))
            .map(|j| j.right.occurrence.clone())
            .collect();
        let plan = Self {
            distinct: p.distinct,
            ir_version: "weft-ir/0.4.0",
            module_pins: p.module_pins,
            read_profile: None,
            required_capabilities: p.required_capabilities,
            type_graph: p.type_graph,
            source: p.source,
            page_key: p.page_key,
            joins: p.joins,
            filters: p.filters,
            groups: p.groups,
            having: p.having,
            aggregate: p.aggregate,
            outputs: p.outputs,
            order: p.order,
            limit: p.limit,
            path_expansion: p.path_expansion,
            outer_join_scans,
        };
        let value = serde_json::to_value(&plan).map_err(|_| Invalid::InvalidShape)?;
        if !PlanSchema04::is_valid(&value) {
            return Err(Invalid::InvalidShape);
        }
        Ok(plan)
    }
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }
    pub fn expansion(&self) -> Option<&PathExpansion> {
        self.path_expansion.as_ref()
    }
    pub fn pins(&self) -> &[ModelPin] {
        &self.module_pins
    }
    pub fn capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
    pub(crate) fn joins(&self) -> &[Join] {
        &self.joins
    }
    pub(crate) fn filters(&self) -> &[Predicate] {
        &self.filters
    }
    pub(crate) fn groups(&self) -> &[Field] {
        &self.groups
    }
    pub(crate) fn having(&self) -> &[Having] {
        &self.having
    }
    pub(crate) fn order(&self) -> &[Field] {
        &self.order
    }
    pub(crate) fn limit(&self) -> Option<u16> {
        self.limit
    }
    pub(crate) fn distinct(&self) -> bool {
        self.distinct
    }
    pub(crate) fn aggregate(&self) -> bool {
        self.aggregate
    }
    pub(crate) fn outer_join_scans(&self) -> &[String] {
        &self.outer_join_scans
    }
    pub(crate) fn type_graph(&self) -> &[Descriptor] {
        &self.type_graph
    }
    pub fn source(&self) -> &Scan {
        &self.source
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[jsonschema::validator(
        path = "../../docs/helix/02-design/contracts/logical-plan-v0.4.schema.json"
    )]
    struct Schema04;
    fn id(element: &str) -> Identity {
        Identity {
            document_id: "d".into(),
            revision: "r".into(),
            module: "m".into(),
            element: element.into(),
        }
    }
    fn hop(from: &str, to: &str) -> RelationshipRead {
        let key = AuthoredKey {
            id: "key".into(),
            fields: vec![id("field")],
            types: vec![LogicalType {
                family: Family::String,
                facets: serde_json::json!({}),
                nullable: false,
            }],
        };
        RelationshipRead {
            identity: crate::application_model::RelationshipIdentity {
                document_id: "d".into(),
                revision: "r".into(),
                module: "m".into(),
                relationship: format!("{from}-{to}"),
            },
            inverse: false,
            from: id(from),
            to: id(to),
            source_key: key.clone(),
            target_key: key,
            source_multiplicity: serde_json::json!({"min":0,"max":"*"}),
            target_multiplicity: serde_json::json!({"min":0,"max":"*"}),
            target_lifecycle: "independent".into(),
        }
    }
    fn path() -> PathRead {
        PathRead::new(
            "root".into(),
            [hop("Root", "Middle"), hop("Middle", "Terminal")],
            Span { start: 0, end: 20 },
            [Span { start: 1, end: 5 }, Span { start: 6, end: 10 }],
        )
        .unwrap()
    }
    fn parts() -> PlanParts {
        let pin = ModelPin {
            document_id: "d".into(),
            revision: "r".into(),
            umf_version: "0.8.0".into(),
            sha256: "a".repeat(64),
        };
        PlanParts {
            distinct: false,
            module_pins: vec![pin.clone()],
            required_capabilities: vec![],
            type_graph: vec![],
            source: Scan {
                occurrence: "root".into(),
                record: id("Root"),
                pin,
            },
            page_key: None,
            joins: vec![],
            filters: vec![],
            groups: vec![],
            having: vec![],
            aggregate: false,
            outputs: vec![Output {
                name: "paths".into(),
                expression: Expression::related_paths(path(), 2).unwrap(),
            }],
            order: vec![],
            limit: None,
            path_expansion: None,
        }
    }
    #[test]
    fn collection_and_count_schema_correspondence() {
        let value = serde_json::to_value(Plan::new(parts()).unwrap()).unwrap();
        assert!(Schema04::is_valid(&value));
        assert_eq!(value["readProfile"], serde_json::Value::Null);
        assert_eq!(
            value["outputs"][0]["expression"]["path"]["hops"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        let mut p = parts();
        p.aggregate = true;
        p.path_expansion = Some(PathExpansion::new("paths".into(), path()).unwrap());
        p.outputs[0].expression = Expression::count_distinct_path_targets("paths".into()).unwrap();
        assert!(Schema04::is_valid(
            &serde_json::to_value(Plan::new(p).unwrap()).unwrap()
        ));
    }
    #[test]
    fn alternative_intermediate_keys_and_parallel_path_structure_survive() {
        let first = hop("Root", "Middle");
        let mut second = hop("Middle", "Terminal");
        second.source_key.id = "alternative".into();
        let p = PathRead::new(
            "root".into(),
            [first, second],
            Span { start: 0, end: 20 },
            [Span { start: 1, end: 5 }, Span { start: 6, end: 10 }],
        )
        .unwrap();
        assert_eq!(p.hops()[0].to, p.hops()[1].from);
        assert_ne!(p.hops()[0].target_key.id, p.hops()[1].source_key.id);
        assert_eq!(p.terminal_record(), &id("Terminal"));
    }
    #[test]
    fn impossible_paths_and_spans_refuse() {
        assert!(PathRead::new(
            "root".into(),
            [hop("Root", "Middle"), hop("Other", "Terminal")],
            Span { start: 0, end: 20 },
            [Span { start: 1, end: 5 }, Span { start: 6, end: 10 }]
        )
        .is_err());
        assert!(PathRead::new(
            "root".into(),
            [hop("Root", "Middle"), hop("Middle", "Terminal")],
            Span { start: 0, end: 2 },
            [Span { start: 1, end: 5 }, Span { start: 6, end: 10 }]
        )
        .is_err());
        assert!(Expression::related_paths(path(), 0).is_err());
        assert!(Expression::related_paths(path(), 1001).is_err());
    }
    #[test]
    fn stage_root_count_and_collision_refuse() {
        let mut p = parts();
        p.aggregate = true;
        assert!(Plan::new(p).is_err());
        let mut p = parts();
        p.path_expansion = Some(PathExpansion::new("paths".into(), path()).unwrap());
        assert!(Plan::new(p).is_err());
        let mut p = parts();
        p.source.record = id("Other");
        assert!(Plan::new(p).is_err());
        let mut p = parts();
        p.aggregate = true;
        p.path_expansion = Some(PathExpansion::new("root".into(), path()).unwrap());
        p.outputs[0].expression = Expression::count_distinct_path_targets("root".into()).unwrap();
        assert!(Plan::new(p).is_err());
        let mut p = parts();
        p.outputs[0].expression =
            Expression::count_distinct_path_targets("missing".into()).unwrap();
        assert!(Plan::new(p).is_err());
    }
    #[test]
    fn legacy_aggregates_are_not_opaque() {
        for expression in [
            crate::arithmetic_plan::Expression::Sum {
                argument: Field {
                    scan: "root".into(),
                    identity: id("field"),
                    logical_type: integer(),
                    span: Span { start: 0, end: 1 },
                },
                logical_type: integer(),
            },
            crate::arithmetic_plan::Expression::Count {
                logical_type: integer(),
            },
            crate::arithmetic_plan::Expression::CountDistinct {
                argument: Field {
                    scan: "root".into(),
                    identity: id("field"),
                    logical_type: LogicalType {
                        family: Family::String,
                        facets: serde_json::json!({}),
                        nullable: false,
                    },
                    span: Span { start: 0, end: 1 },
                },
                logical_type: integer(),
            },
        ] {
            let mut p = parts();
            p.outputs[0].expression = Expression::Legacy(expression.clone());
            assert!(Plan::new(p.clone()).is_err());
            p.aggregate = true;
            assert!(Schema04::is_valid(
                &serde_json::to_value(Plan::new(p).unwrap()).unwrap()
            ));
        }
    }
    #[test]
    fn endpoint_key_and_paging_metadata_refuse() {
        let mut bad = hop("Root", "Middle");
        bad.source_key.fields[0].revision = "other".into();
        assert!(PathRead::new(
            "root".into(),
            [bad, hop("Middle", "Terminal")],
            Span { start: 0, end: 20 },
            [Span { start: 1, end: 5 }, Span { start: 6, end: 10 }]
        )
        .is_err());
        let mut p = parts();
        p.page_key = Some(hop("Root", "Middle").source_key);
        assert!(Plan::new(p).is_err());
        let mut p = parts();
        p.outputs[0].expression =
            Expression::Legacy(crate::arithmetic_plan::Expression::RelatedKeys {
                scan: "root".into(),
                relationship: hop("Root", "Middle"),
                bound: 1,
            });
        p.groups.push(Field {
            scan: "root".into(),
            identity: id("field"),
            logical_type: integer(),
            span: Span { start: 0, end: 1 },
        });
        assert!(Plan::new(p).is_err());
    }
    #[test]
    fn original_catalog_cross_module_key_is_preserved() {
        let mut doc: serde_json::Value = serde_json::from_str(include_str!(
            "../../../docs/helix/03-test/fixtures/sales.umf.json"
        ))
        .unwrap();
        let elements = doc["modules"][0]["elements"].as_array_mut().unwrap();
        let index = elements
            .iter()
            .position(|v| v["id"] == "customer-id")
            .unwrap();
        let field = elements.remove(index);
        doc["modules"][0]["elements"][0]["members"][0]["module"] = serde_json::json!("shared");
        doc["modules"][0]["elements"][0]["keys"] = serde_json::json!([{"id":"primary","name":"primary","fields":[{"module":"shared","element":"customer-id"}]}]);
        doc["modules"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"id":"shared","namespace":"shared","elements":[field],"relationships":[],"extensions":{}}));
        let text = doc.to_string();
        let catalog = crate::model::Catalog::prepare(vec![crate::model::ModuleInput {
            document_json: text.clone(),
            pin: ModelPin {
                document_id: "sales-fixture".into(),
                revision: "r".into(),
                umf_version: "0.7.0".into(),
                sha256: crate::json::sha256(text.as_bytes()),
            },
            selected_module_ids: vec!["sales".into(), "shared".into()],
        }])
        .unwrap();
        let record = catalog
            .record(
                None,
                &crate::syntax::Name {
                    value: "Customer".into(),
                    quoted: true,
                    span: Span { start: 0, end: 8 },
                },
            )
            .unwrap();
        let key = catalog.authored_key(&record, "primary").unwrap();
        assert_eq!(key.fields[0].module, "shared");
        assert_eq!(record.identity.module, "sales");
        let make = || RelationshipRead {
            identity: crate::application_model::RelationshipIdentity {
                document_id: "sales-fixture".into(),
                revision: "r".into(),
                module: "sales".into(),
                relationship: "self".into(),
            },
            inverse: false,
            from: record.identity.clone(),
            to: record.identity.clone(),
            source_key: key.clone(),
            target_key: key.clone(),
            source_multiplicity: serde_json::json!({"min":0,"max":"*"}),
            target_multiplicity: serde_json::json!({"min":0,"max":"*"}),
            target_lifecycle: "independent".into(),
        };
        assert!(PathRead::new(
            "root".into(),
            [make(), make()],
            Span { start: 0, end: 20 },
            [Span { start: 1, end: 5 }, Span { start: 6, end: 10 }]
        )
        .is_ok());
    }
    #[test]
    fn complete_local_schema_and_pin_refusals() {
        let mut p = parts();
        p.source.record.document_id = "other".into();
        assert!(Plan::new(p).is_err());
        let mut p = parts();
        p.limit = Some(0);
        assert!(Plan::new(p).is_err());
        let mut p = parts();
        p.outputs.clear();
        assert!(Plan::new(p).is_err());
        let mut p = parts();
        p.outputs = vec![p.outputs[0].clone(); 257];
        assert!(Plan::new(p).is_err());
        let mut p = parts();
        p.outputs[0].name.clear();
        assert!(Plan::new(p).is_err());
        let mut p = parts();
        p.aggregate = true;
        p.path_expansion = Some(PathExpansion::new("paths".into(), path()).unwrap());
        p.outputs[0].expression = Expression::count_distinct_path_targets("paths".into()).unwrap();
        p.having.push(Having {
            count: Expression::count_distinct_path_targets("paths".into()).unwrap(),
            threshold: crate::application_ir::Value::Literal {
                value: "1".into(),
                logical_type: integer(),
                span: Span { start: 0, end: 1 },
            },
        });
        assert!(Plan::new(p).is_err());
        assert!(PathRead::new(
            "root".into(),
            [hop("Root", "Middle"), hop("Middle", "Terminal")],
            Span { start: 0, end: 20 },
            [Span { start: 6, end: 10 }, Span { start: 1, end: 5 }]
        )
        .is_err());
    }
    #[test]
    fn pin_only_mutation_refuses_before_root_correspondence() {
        let mut p = parts();
        p.source.pin.document_id = "other-document".into();
        p.source.pin.revision = "other-revision".into();
        p.module_pins = vec![p.source.pin.clone()];
        assert!(matches!(Plan::new(p), Err(Invalid::InvalidPins)));
        let mut p = parts();
        let mut joined_pin = p.source.pin.clone();
        joined_pin.document_id = "other-document".into();
        p.module_pins.push(joined_pin.clone());
        p.joins.push(Join {
            kind: None,
            right: Scan {
                occurrence: "joined".into(),
                record: id("Joined"),
                pin: joined_pin,
            },
            on: vec![],
        });
        assert!(matches!(Plan::new(p), Err(Invalid::InvalidPins)));
    }
}
