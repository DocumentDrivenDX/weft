//! Fixed Truss runtime mapping over its independently authorized scalar source.
//! The host constructs this binding from accepted bytes and actual catalog IDs.
use crate::{Identifier, Parameters};
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    application_ir as app,
    application_model::{Descriptor, RelationshipRead, Shape},
    backend::*,
    error::{Diagnostic, Result},
    ir::{Family, Identity, LogicalType},
    model::Catalog,
};
pub const PROFILE: &str = "truss-runtime-binding/0.1";
pub const TARGET: &str = "truss-runtime-pg18.6/0.1";
pub const NUMERIC_PROFILE: &str = "truss-runtime-binding/0.2";
pub const NUMERIC_TARGET: &str = "truss-runtime-pg18.6/0.2";
pub const RELATIONSHIP_PROFILE: &str = "truss-runtime-binding/0.3";
pub const RELATIONSHIP_TARGET: &str = "truss-runtime-pg18.6/0.3";
pub const PAGE_PROFILE: &str = "truss-runtime-binding/0.4";
pub const PAGE_TARGET: &str = "truss-runtime-pg18.6/0.4";
const BACKEND: &str = "truss.postgresql.runtime";
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Binding {
    interface_version: String,
    schema: String,
    installation_id: String,
    release_id: String,
    release_sha256: String,
    catalog_revision: i32,
    maximum_scan_rows: u32,
    maximum_result_rows: u32,
    maximum_bytes: u32,
    documents: Vec<DocumentPin>,
    records: Vec<RecordBinding>,
    #[serde(default)]
    relationships: Vec<RelationshipBinding>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DocumentPin {
    document_id: String,
    revision: String,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RecordBinding {
    identity: Identity,
    type_id: String,
    fields: Vec<FieldBinding>,
    #[serde(default)]
    keys: Vec<KeyBinding>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct KeyBinding {
    key_id: String,
    property_ids: Vec<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FieldBinding {
    identity: Identity,
    property_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RelationshipBinding {
    identity: serde_json::Value,
    relationship_type_id: String,
    source_type_id: String,
    target_type_id: String,
}
pub struct Runtime {
    exact_numbers: bool,
    relationships: bool,
    pages: bool,
    release_030: bool,
    groups: bool,
    selected_metadata: bool,
    exact_scalars: bool,
    retained_metadata: bool,
    complete_entities: bool,
    entity_times: bool,
    import_loads: bool,
    numeric_facets: bool,
    typed_keys: bool,
    live_traversal: bool,
    facet_evolution: bool,
    scalar_containers: bool,
    container_queries: bool,
    typed_relationship_keys: bool,
    typed_key_pages: bool,
    record_evolution: bool,
    required_tightening: bool,
    relationship_tightening: bool,
    container_tightening: bool,
    relationship_names: bool,
    binary_equality: bool,
    optional_binary: bool,
    decimal_scale2: bool,
    field_names: bool,
    source_references: bool,
    timestamp_equality: bool,
    catalog_rebinding: bool,
    key_backfill: bool,
    keyed_rebinding: bool,
    recursive_containers: bool,
    owned_runtime: bool,
    required_rebinding: bool,
    parallel_edges: bool,
    engine_imports: bool,
    typed_page_mode: bool,
    continuation_filter: Option<usize>,
}
impl Runtime {
    fn profile(&self) -> &'static str {
        if self.pages {
            PAGE_PROFILE
        } else if self.relationships {
            RELATIONSHIP_PROFILE
        } else if self.exact_numbers {
            NUMERIC_PROFILE
        } else {
            PROFILE
        }
    }
    fn target(&self) -> &'static str {
        if self.pages {
            PAGE_TARGET
        } else if self.relationships {
            RELATIONSHIP_TARGET
        } else if self.exact_numbers {
            NUMERIC_TARGET
        } else {
            TARGET
        }
    }
    fn version(&self) -> &'static str {
        if self.owned_runtime { "0.4.35" } else if self.recursive_containers { "0.4.34" } else if self.keyed_rebinding { "0.4.33" } else if self.key_backfill { "0.4.32" } else if self.required_rebinding { "0.4.31" } else if self.catalog_rebinding { "0.4.30" } else if self.timestamp_equality { "0.4.29" } else if self.source_references { "0.4.28" } else if self.field_names { "0.4.27" } else if self.decimal_scale2 { "0.4.26" } else if self.optional_binary { "0.4.25" } else if self.binary_equality { "0.4.24" } else if self.relationship_names { "0.4.23" } else if self.parallel_edges { "0.4.22" } else if self.engine_imports { "0.4.21" } else if self.container_tightening { "0.4.20" } else if self.relationship_tightening { "0.4.19" } else if self.required_tightening { "0.4.18" } else if self.record_evolution { "0.4.17" } else if self.typed_key_pages { "0.4.16" } else if self.typed_relationship_keys { "0.4.15" } else if self.container_queries { "0.4.14" } else if self.scalar_containers { "0.4.13" } else if self.facet_evolution { "0.4.12" } else if self.live_traversal { "0.4.11" } else if self.typed_keys { "0.4.10" } else if self.numeric_facets { "0.4.9" } else if self.import_loads { "0.4.8" } else if self.entity_times { "0.4.7" } else if self.complete_entities {
            "0.4.6"
        } else if self.retained_metadata {
            "0.4.5"
        } else if self.exact_scalars {
            "0.4.4"
        } else if self.selected_metadata {
            "0.4.3"
        } else if self.groups {
            "0.4.2"
        } else if self.release_030 {
            "0.4.1"
        } else if self.pages {
            "0.4.0"
        } else if self.relationships {
            "0.3.0"
        } else if self.exact_numbers {
            "0.2.0"
        } else {
            "0.1.0"
        }
    }
    fn admit_projection(&self, descriptor: &Descriptor) -> Result<()> {
        if self.exact_scalars && matches!(descriptor.shape, Shape::ExactScalar { .. })
            && matches!(descriptor.availability.as_deref(), Some("required" | "absent-allowed")) {
            return Ok(());
        }
        if self.container_queries && matches!(descriptor.shape, Shape::Sequence { .. } | Shape::Map { .. })
            && matches!(descriptor.availability.as_deref(), Some("required" | "absent-allowed")) { return Ok(()); }
        scalar_type(descriptor, self.exact_numbers, self.numeric_facets, self.decimal_scale2).map(|_| ())
    }
    fn additions(&self) -> Vec<String> {
        let mut v = vec!["value.nativeNull".into()];
        if self.exact_numbers {
            v.extend([
                "type.integer.unbounded".into(),
                "type.decimal.unbounded".into(),
            ]);
        }
        if self.container_queries { v.extend(["value.sequence".into(), "value.map".into()]); }
        v
    }
}
pub struct Mapping {
    binding: Binding,
    original: String,
    relationship_capabilities: Vec<String>,
}
pub struct TargetPlan(Emission);
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
fn unsupported(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-CAPABILITY", "lower", message)
}
fn key(id: &Identity) -> String {
    json!(id).to_string()
}
fn native(id: &str) -> Result<()> {
    let n = id
        .parse::<i32>()
        .map_err(|_| fail("Native catalog ID required"))?;
    if n == 0 || n.to_string() != id {
        return Err(fail("Canonical nonzero native catalog ID required"));
    }
    Ok(())
}
fn literal(s: &str) -> Result<String> {
    if s.contains('\0') {
        return Err(fail("NUL cannot enter PostgreSQL text"));
    }
    Ok(format!(
        "E'{}'",
        s.replace('\\', "\\\\").replace('\'', "''")
    ))
}
fn obligation() -> Obligation {
    Obligation {
        id: "truss.runtime.context/0.1".into(),
        parameters: json!({"requirements":["original accepted catalog mapping and release","complete selected owner and property authority","bounded validated scalar source before predicates","same caller transaction","search_path=pg_catalog,pg_temp","final authority/catalog/release recheck","exact buffered decoding"]}),
        owner: ObligationOwner::Host,
        failure_code: "WFT-OBLIGATION".into(),
    }
}
fn scalar_type(d: &Descriptor, exact_numbers: bool, numeric_facets: bool, decimal_scale2: bool) -> Result<&LogicalType> {
    match &d.shape {
        Shape::Scalar { logical_type } => {
            let facets = &logical_type.facets;
            let accepted = match logical_type.family {
                Family::String | Family::Boolean => facets == &json!({}),
                Family::Integer => facets == &json!({}) || numeric_facets && facets == &json!({"integerWidth":{"bits":64,"signed":true}}),
                Family::Decimal => exact_numbers && (facets == &json!({}) || numeric_facets && (facets == &json!({"precision":18,"scale":4}) || decimal_scale2 && facets == &json!({"precision":18,"scale":2}))),
            };
            if !accepted
                || !matches!(
                    d.availability.as_deref(),
                    Some("required" | "absent-allowed")
                )
            {
                return Err(unsupported(
                    "Selected scalar facets or availability have no runtime mapping",
                ));
            }
            Ok(logical_type)
        }
        _ => Err(unsupported(
            "Runtime mapping currently requires rootless scalar fields",
        )),
    }
}
impl Mapping {
    fn record(&self, id: &Identity) -> Result<&RecordBinding> {
        self.binding
            .records
            .iter()
            .find(|r| r.identity == *id)
            .ok_or_else(|| fail("Accepted record mapping missing"))
    }
    fn field(&self, owner: &Identity, id: &Identity) -> Result<&FieldBinding> {
        self.record(owner)?
            .fields
            .iter()
            .find(|f| f.identity == *id)
            .ok_or_else(|| fail("Accepted owner/field mapping missing"))
    }
}
impl Backend for Runtime {
    type Mapping = Mapping;
    type TargetPlan = TargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let language = vec![LanguageProfile {
            dialect_profile: "weft-sql/0.2.0".into(),
            ir_version: if self.timestamp_equality { "weft-ir/0.2.6" } else if self.binary_equality { "weft-ir/0.2.5" } else if self.import_loads { "weft-ir/0.2.4" } else if self.entity_times { "weft-ir/0.2.3" } else if self.complete_entities { "weft-ir/0.2.2" } else if self.exact_scalars { "weft-ir/0.2.1" } else { "weft-ir/0.2.0" }.into(),
        }];
        Ok(Manifest{backend_id:BACKEND.into(),backend_version:self.version().into(),interface_version:if self.timestamp_equality{"weft-backend/0.2.6"}else if self.binary_equality{"weft-backend/0.2.5"}else if self.import_loads{"weft-backend/0.2.4"}else if self.entity_times{"weft-backend/0.2.3"}else if self.complete_entities{"weft-backend/0.2.2"}else if self.exact_scalars{"weft-backend/0.2.1"}else{"weft-backend/0.2.0"}.into(),binding_profile:self.profile().into(),language_profiles:language.clone(),target_profiles:vec![TargetProfile{id:self.target().into(),engine:"postgresql".into(),engine_version:"18.6".into(),session_settings:json!({"server_encoding":"UTF8","search_path":"pg_catalog,pg_temp","read_context":"live"}),storage_layout_revision:if self.pages{"truss-runtime-keyset-query/0.4".into()}else if self.relationships{"truss-runtime-relationship-query/0.3".into()}else if self.exact_numbers{"truss-runtime-scalar-query/0.2".into()}else{"truss-runtime-scalar-query/0.1".into()},publication_revision:"caller-transaction/0.1".into()}],capabilities:["scan","project","filter","innerJoin","equal","and","type.string","type.boolean","type.integer","aggregate","aggregate.count","parameter.named","order.asc","limit","value.presence","value.nativeNull","value.structured","project.entity"].iter().map(|id|(*id).to_owned()).chain(if self.complete_entities{vec!["project.nativeMetadata".into(),"relationship.completeKeys".into()]}else{vec![]}).chain(if self.exact_numbers{vec!["type.decimal".into(),"type.integer.unbounded".into(),"type.decimal.unbounded".into()]}else{vec![]}).chain(if self.relationships{vec!["relationship.exists".into(),"relationship.boundedKeys".into(),"relationship.inverse".into()]}else{vec![]}).chain(if self.pages{vec!["key.uniqueStable".into(),"compare.lexicographicGreater".into()]}else{vec![]}).chain(if self.groups{vec!["group".into()]}else{vec![]}).chain(if self.exact_scalars{vec!["type.timestamp".into(),"type.binary".into()]}else{vec![]}).chain(if self.decimal_scale2{vec!["type.decimal.scale2".into()]}else{vec![]}).chain(if self.optional_binary{vec!["compare.binary.optionalEqual".into()]}else{vec![]}).chain(if self.binary_equality{vec!["compare.binary.equal".into()]}else{vec![]}).chain(if self.timestamp_equality{vec!["compare.timestamp.instantEqual".into(),"compare.timestamp.optionalInstantEqual".into()]}else{vec![]}).chain(if self.container_queries{vec!["value.sequence".into(),"value.map".into()]}else{vec![]}).map(|id|Capability{id:id.clone(),target_profiles:vec![self.target().into()],language_profiles:language.clone(),logical_domain:json!({"subset":"rootless scalar records; exact string/boolean and explicitly admitted mathematical integer/decimal domains; native-domain refusal remains host-owned"}),result_domain:json!({"transport":"exact text and presence envelopes"}),constraints:vec![if self.recursive_containers{"Projection-only acyclic required-item sequence/map chains with exact scalar leaves; depth32; complete item dependency closure; original numeric and timestamp/binary tokens preserved; map keys unique and array order retained; no container comparisons, unnesting, cyclic items or record/composed ownership; inherited exact-instant timestamp equality remains umf-rfc3339-exact-instant/1.0.0".into()}else if self.timestamp_equality{"Explicit umf-rfc3339-exact-instant/1.0.0 equality over original stored and operand tokens; arbitrary fractional digits within1MiB; unknown local offset retained separately; no timestamp keys or native timestamp casts".into()}else if self.decimal_scale2{"Exact declared decimal18,2 or18,4 scalar/item/key/page domains; preserve original tokens and refuse nonrepresentable values without rounding; existing binary and optional-binary equality retained".into()}else if self.optional_binary{"Singular required or absent-allowed binary byte equality; absent and null yield SQL UNKNOWN/nonmatch; empty bytes remain a value; canonical typed binary parameters, filters, INNER joins and page filters; timestamps remain projection-only".into()}else if self.binary_equality{"Required singular binary byte equality, canonical base64 literals and explicitly typed binary parameters; INNER joins and page filters; timestamps remain projection-only".into()}else if self.typed_key_pages{"Explicit typed SQL-page entrypoint with required string/boolean/integer/decimal18,4 primary keys and lexical lossless continuations; legacy explicit entity-page profiles remain string-only; typed relationship keys and flat container projections retained".into()}else if self.typed_relationship_keys{"Required string/boolean/integer/decimal18,4 relationship keys; typed comparison/order with original numeric token projection; complete and bounded lists; flat container/entity projections; page keys remain required strings; no binary/timestamp relationship keys or nested values".into()}else if self.container_queries{"Projection-only one-level sequences/maps with required singular scalar item descriptors; exact numeric item text; outer presence; no nested structures, container comparisons or unnesting; complete rootless entity fields and original current item authority".into()}else if self.complete_entities{"Explicit complete entity metadata and complete required-string relationship key lists; maximum100 per list, excess refuses the whole query; original attribution evidence required; rootless scalar records only".into()}else if self.exact_scalars{"Projection-only timestamp/binary exact text tokens; no temporal/binary comparison, grouping, parameters or keys".into()}else if self.selected_metadata{"Complete original member mapping with only selected scalar semantics admitted; timestamp/binary projections unsupported; required non-nullable string/boolean grouping, with complete order and LIMIT for count-summary".into()}else if self.groups{"Required non-nullable string/boolean grouping; count-summary requires complete grouping order and LIMIT; no optional/numeric group keys or structured properties".into()}else if self.pages{"Required string authored-key entity pages, related entity pages and ungrouped count summary; no grouped summary or structured properties".into()}else if self.relationships{"Independent directed monomorphic relationships with required string keys only; no structured-property or page-key profile".into()}else{"No relationship or structured-property lowering; no page-key profile".into()}],obligations:vec![obligation()],status:Status::Candidate,evidence:vec![]}).collect(),evidence:vec![]})
    }
    fn validate_binding(&self, c: &Context<'_>) -> Result<Validated<Mapping>> {
        if c.binding.profile != self.profile()
            || weft_core::json::sha256(c.binding.json.as_bytes()) != c.binding.sha256
        {
            return Err(fail("Exact runtime binding pin required"));
        }
        let original_binding = weft_core::json::checked_json(&c.binding.json)
            .map_err(|_| fail("Runtime binding JSON refused"))?;
        if original_binding.get("relationships").is_some() != self.relationships {
            return Err(fail("Exact versioned relationship binding shape required"));
        }
        if original_binding["records"]
            .as_array()
            .is_none_or(|records| {
                records
                    .iter()
                    .any(|r| r.get("keys").is_some() != self.pages)
            })
        {
            return Err(fail("Exact versioned key binding shape required"));
        }
        let binding: Binding = serde_json::from_value(original_binding)
            .map_err(|_| fail("Runtime binding shape refused"))?;
        if binding.interface_version != self.profile()
            || binding.schema != "truss"
            || binding.installation_id.is_empty()
            || binding.release_id
                != if self.owned_runtime { "truss-catalog-runtime/0.65-candidate" } else if self.recursive_containers { "truss-catalog-runtime/0.64-candidate" } else if self.keyed_rebinding { "truss-catalog-runtime/0.63-candidate" } else if self.key_backfill { "truss-catalog-runtime/0.62-candidate" } else if self.required_rebinding { "truss-catalog-runtime/0.61-candidate" } else if self.catalog_rebinding { "truss-catalog-runtime/0.60-candidate" } else if self.timestamp_equality { "truss-catalog-runtime/0.59-candidate" } else if self.source_references { "truss-catalog-runtime/0.58-candidate" } else if self.field_names { "truss-catalog-runtime/0.57-candidate" } else if self.decimal_scale2 { "truss-catalog-runtime/0.56-candidate" } else if self.optional_binary { "truss-catalog-runtime/0.55-candidate" } else if self.binary_equality { "truss-catalog-runtime/0.54-candidate" } else if self.relationship_names { "truss-catalog-runtime/0.53-candidate" } else if self.parallel_edges { "truss-catalog-runtime/0.52-candidate" } else if self.engine_imports { "truss-catalog-runtime/0.51-candidate" } else if self.container_tightening { "truss-catalog-runtime/0.50-candidate" } else if self.relationship_tightening { "truss-catalog-runtime/0.49-candidate" } else if self.required_tightening { "truss-catalog-runtime/0.48-candidate" } else if self.record_evolution { "truss-catalog-runtime/0.47-candidate" } else if self.typed_key_pages { "truss-catalog-runtime/0.46-candidate" } else if self.typed_relationship_keys { "truss-catalog-runtime/0.45-candidate" } else if self.container_queries { "truss-catalog-runtime/0.44-candidate" } else if self.scalar_containers { "truss-catalog-runtime/0.43-candidate" } else if self.facet_evolution { "truss-catalog-runtime/0.42-candidate" } else if self.live_traversal { "truss-catalog-runtime/0.41-candidate" } else if self.typed_keys { "truss-catalog-runtime/0.40-candidate" } else if self.numeric_facets { "truss-catalog-runtime/0.39-candidate" } else if self.import_loads && binding.release_id == "truss-catalog-runtime/0.38-candidate" { "truss-catalog-runtime/0.38-candidate" } else if self.import_loads && binding.release_id == "truss-catalog-runtime/0.37-candidate" { "truss-catalog-runtime/0.37-candidate" } else if self.import_loads && binding.release_id == "truss-catalog-runtime/0.36-candidate" { "truss-catalog-runtime/0.36-candidate" } else if self.import_loads { "truss-catalog-runtime/0.35-candidate" } else if self.entity_times { "truss-catalog-runtime/0.34-candidate" } else if self.complete_entities {
                    "truss-catalog-runtime/0.33-candidate"
                } else if self.retained_metadata {
                    "truss-catalog-runtime/0.32-candidate"
                } else if self.selected_metadata {
                    "truss-catalog-runtime/0.31-candidate"
                } else if self.release_030 {
                    "truss-catalog-runtime/0.30-candidate"
                } else if self.pages {
                    "truss-catalog-runtime/0.29-candidate"
                } else if self.relationships {
                    "truss-catalog-runtime/0.28-candidate"
                } else if self.exact_numbers {
                    "truss-catalog-runtime/0.27-candidate"
                } else {
                    "truss-catalog-runtime/0.26-candidate"
                }
            || binding.release_sha256.len() != 64
            || !binding
                .release_sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
            || binding.catalog_revision < 1
            || !(1..=10000).contains(&binding.maximum_result_rows)
            || !(1..=10000).contains(&binding.maximum_scan_rows)
            || !(1..=16777216).contains(&binding.maximum_bytes)
            || binding.records.is_empty()
            || binding.records.len() > 1024
        {
            return Err(fail("Selected runtime context or limits refused"));
        }
        if binding.documents.len() != c.catalog.inputs().len()
            || c.catalog.inputs().iter().any(|input| {
                !binding.documents.iter().any(|d| {
                    d.document_id == input.pin.document_id
                        && d.revision == input.pin.revision
                        && d.sha256 == input.pin.sha256
                })
            })
        {
            return Err(fail(
                "Original model pins differ from runtime document selection",
            ));
        }
        let mut records = BTreeSet::new();
        let mut fields = BTreeSet::new();
        let mut item_dependencies = BTreeSet::new();
        let mut native_types = BTreeSet::new();
        let mut native_fields = BTreeSet::new();
        for r in &binding.records {
            native(&r.type_id)?;
            if !records.insert(key(&r.identity)) || !native_types.insert(&r.type_id) {
                return Err(fail("Duplicate record mapping"));
            }
            let original = c.catalog.record_by_identity(&r.identity)?;
            let members = if self.selected_metadata {
                c.catalog.record_members(&original)?
            } else {
                c.catalog.entity_descriptor(&original)?.members
            };
            if members.len() != r.fields.len() {
                return Err(fail("Complete original record member mapping required"));
            }
            // Entity selections carry the Record root and require its whole graph.
            if self.selected_metadata && c.selection.types.contains(&r.identity) {
                let entity = c.catalog.entity_descriptor(&original)?;
                for descriptor in &entity.graph {
                    if descriptor.identity != r.identity {
                        self.admit_projection(descriptor)?;
                    }
                }
            }
            for member in &members {
                let f = r.fields.iter().find(|f| f.identity == member.identity)
                    .ok_or_else(|| fail("Original member mapping missing"))?;
                native(&f.property_id)?;
                if !native_fields.insert(&f.property_id) {
                    return Err(fail("Duplicate native property mapping"));
                }
                if !self.selected_metadata || c.selection.fields.contains(&f.identity)
                    || c.selection.types.contains(&f.identity)
                {
                    let (_, graph) = c.catalog.member_descriptor_by_identity(&original, &f.identity)?;
                    let descriptor = graph.iter().find(|d| d.identity == f.identity)
                        .ok_or_else(|| fail("Original member descriptor missing"))?;
                    self.admit_projection(descriptor)?;
                    if let Shape::Sequence { item } | Shape::Map { item } = &descriptor.shape {
                        let mut current=item.clone();let mut path=BTreeSet::new();
                        loop {
                            if path.len()>=32 || !path.insert(key(&current)) {return Err(unsupported("Bounded acyclic item descriptor chain required"));}
                            let item_descriptor=graph.iter().find(|d| d.identity==current).ok_or_else(||fail("Original container item descriptor missing"))?;
                            if item_descriptor.availability.as_deref()!=Some("required") {return Err(unsupported("Required item descriptor required"));}
                            self.admit_projection(item_descriptor)?;item_dependencies.insert(key(&current));
                            match &item_descriptor.shape {
                                Shape::Sequence{item}|Shape::Map{item} if self.recursive_containers=>current=item.clone(),
                                Shape::Scalar{..}|Shape::ExactScalar{..}=>break,
                                _=>return Err(unsupported("Selected container item shape unavailable")),
                            }
                        }
                    }
                }
                fields.insert(key(&f.identity));
            }
        }
        if c.selection
            .records
            .iter()
            .any(|r| !records.contains(&key(r)))
            || c.selection.fields.iter().any(|f| !fields.contains(&key(f)))
            || (!self.relationships && !c.selection.relationships.is_empty())
            || c.selection
                .types
                .iter()
                .any(|t| !fields.contains(&key(t)) && !records.contains(&key(t)) && !item_dependencies.contains(&key(t)))
        {
            return Err(fail("Resolved selection lacks complete runtime mapping"));
        }
        let mut relationship_capabilities = BTreeSet::new();
        if self.relationships {
            let mut seen = BTreeSet::new();
            let mut native_seen = BTreeSet::new();
            for rel in &binding.relationships {
                native(&rel.relationship_type_id)?;
                native(&rel.source_type_id)?;
                native(&rel.target_type_id)?;
                if !seen.insert(rel.identity.to_string())
                    || !native_seen.insert(&rel.relationship_type_id)
                {
                    return Err(fail("Duplicate relationship mapping"));
                }
            }
            let Plan::V02(plan) = c.plan else {
                return Err(unsupported(
                    "Relationship mapping requires application-read IR",
                ));
            };
            let selected = plan
                .filters
                .iter()
                .filter_map(|p| {
                    if let app::Predicate::HasRelated { relationship, .. } = p {
                        Some(relationship)
                    } else {
                        None
                    }
                })
                .chain(plan.outputs.iter().filter_map(|o| {
                    if let app::Expression::RelatedKeys { relationship, .. } | app::Expression::CompleteRelatedKeys { relationship, .. } = &o.expression {
                        Some(relationship)
                    } else {
                        None
                    }
                }));
            for rel in selected {
                if rel.target_lifecycle != "independent" {
                    return Err(unsupported("Independent relationship required"));
                }
                let mapped = binding
                    .relationships
                    .iter()
                    .find(|r| r.identity == json!(rel.identity))
                    .ok_or_else(|| fail("Original relationship mapping missing"))?;
                let from = binding
                    .records
                    .iter()
                    .find(|r| r.identity == rel.from)
                    .ok_or_else(|| fail("Relationship source record missing"))?;
                let to = binding
                    .records
                    .iter()
                    .find(|r| r.identity == rel.to)
                    .ok_or_else(|| fail("Relationship target record missing"))?;
                if (if rel.inverse {
                    (&mapped.target_type_id, &mapped.source_type_id)
                } else {
                    (&mapped.source_type_id, &mapped.target_type_id)
                }) != (&from.type_id, &to.type_id)
                {
                    return Err(fail("Relationship endpoint mapping differs from original"));
                }
                for authored in [&rel.source_key, &rel.target_key] {
                    if self.typed_relationship_keys {
                        for t in &authored.types { relationship_capabilities.insert(match t.family { Family::String => "type.string", Family::Boolean => "type.boolean", Family::Integer => "type.integer", Family::Decimal => "type.decimal" }.to_string()); }
                    }
                    if authored.fields.is_empty()
                        || authored.fields.len() != authored.types.len()
                        || authored.types.iter().any(|t| {
                            t.nullable || match t.family {
                                Family::String => t.facets != json!({}),
                                Family::Boolean => !self.typed_relationship_keys || t.facets != json!({}),
                                Family::Integer => !self.typed_relationship_keys || (t.facets != json!({}) && t.facets != json!({"integerWidth":{"bits":64,"signed":true}})),
                                Family::Decimal => !self.typed_relationship_keys || (t.facets != json!({"precision":18,"scale":4}) && !(self.decimal_scale2 && t.facets == json!({"precision":18,"scale":2}))),
                            }
                        })
                    {
                        return Err(unsupported(
                            "Required string relationship key components required",
                        ));
                    }
                }
            }
        }
        if self.pages {
            let Plan::V02(plan) = c.plan else {
                return Err(unsupported("Page mapping requires application-read IR"));
            };
            if let Some(authored) = &plan.page_key {
                let r = binding
                    .records
                    .iter()
                    .find(|r| r.identity == plan.source.record)
                    .ok_or_else(|| fail("Page source record mapping missing"))?;
                let matches = r
                    .keys
                    .iter()
                    .filter(|k| k.key_id == authored.id)
                    .collect::<Vec<_>>();
                if matches.len() != 1
                    || authored.fields.len() != authored.types.len()
                    || authored
                        .types
                        .iter()
                        .any(|t| t.nullable || match t.family { Family::String=>t.facets!=json!({}),Family::Boolean=>!self.typed_page_mode || t.facets!=json!({}),Family::Integer=>!self.typed_page_mode || (t.facets!=json!({}) && t.facets!=json!({"integerWidth":{"bits":64,"signed":true}})),Family::Decimal=>!self.typed_page_mode || (t.facets!=json!({"precision":18,"scale":4}) && !(self.decimal_scale2 && t.facets==json!({"precision":18,"scale":2}))) })
                {
                    return Err(unsupported(
                        "Unique required string page key mapping required",
                    ));
                }
                let ids = authored
                    .fields
                    .iter()
                    .map(|id| {
                        r.fields
                            .iter()
                            .find(|f| f.identity == *id)
                            .map(|f| f.property_id.clone())
                            .ok_or_else(|| fail("Page key member mapping missing"))
                    })
                    .collect::<Result<Vec<_>>>()?;
                if matches[0].property_ids != ids {
                    return Err(fail("Page key mapping changes authored component order"));
                }
            }
        }
        Ok(Validated {
            mapping: Mapping {
                binding,
                original: c.binding.json.clone(),
                relationship_capabilities: relationship_capabilities.iter().cloned().collect(),
            },
            additional_capabilities: self.additions().into_iter().chain(relationship_capabilities).collect::<BTreeSet<_>>().into_iter().collect(),
            coverage: c.selection.clone(),
            obligations: vec![obligation()],
        })
    }
    fn assess(&self, c: &Context<'_>, m: &Mapping) -> Result<Vec<Assessment>> {
        let manifest = self.describe()?;
        let additions = self.additions().into_iter().chain(m.relationship_capabilities.iter().cloned()).collect::<Vec<_>>();
        c.plan
            .capabilities()
            .iter()
            .chain(additions.iter())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|id| {
                if !manifest.capabilities.iter().any(|cap| &cap.id == id) {
                    return Err(unsupported("Operation has no runtime lowering"));
                }
                Ok(Assessment {
                    id: id.clone(),
                    status: Status::Candidate,
                    evidence: vec![],
                    obligations: vec![obligation()],
                })
            })
            .collect()
    }
    fn lower(&self, c: &Context<'_>, m: &Mapping) -> Result<TargetPlan> {
        let Plan::V02(plan) = c.plan else {
            return Err(unsupported("Runtime requires application-read IR"));
        };
        if (!self.groups && !plan.groups.is_empty())
            || (!self.pages && (plan.read_profile.is_some() || plan.page_key.is_some()))
        {
            return Err(unsupported("Runtime page-key/group profile unavailable"));
        }
        for field in &plan.groups {
            let descriptor = plan.type_graph.iter().find(|d| d.identity == field.identity)
                .ok_or_else(|| unsupported("Original group descriptor required"))?;
            if descriptor.availability.as_deref() != Some("required")
                || field.logical_type.nullable
                || !matches!(field.logical_type.family, Family::String | Family::Boolean)
            {
                return Err(unsupported("Required non-null string/boolean group keys only"));
            }
        }
        let mut build = Build {
            mapping: m,
            plan,
            aliases: BTreeMap::new(),
            parameters: Parameters::default(),
            binding_slot: String::new(),
            relationship_scope: 0,
            typed_page_mode: self.typed_page_mode,
            timestamp_equality: self.timestamp_equality,
            recursive_containers: self.recursive_containers,
        };
        build
            .aliases
            .insert(plan.source.occurrence.clone(), plan.source.record.clone());
        for join in &plan.joins {
            build
                .aliases
                .insert(join.right.occurrence.clone(), join.right.record.clone());
        }
        let binding_slot = build.parameters.push(
            LogicalType {
                family: Family::String,
                facets: json!({}),
                nullable: false,
            },
            m.original.clone(),
            json!({"kind":"runtime-binding"}),
        )?;
        build.binding_slot = binding_slot.clone();
        let source = |s: &app::Scan| -> Result<String> {
            if let Some(page_key) = &plan.page_key {
                Ok(format!("\"truss\".\"query_page_objects\"({binding_slot}::pg_catalog.jsonb,{}::pg_catalog.int4,{}) AS {}",m.record(&s.record)?.type_id,literal(&page_key.id)?,Identifier::new(&s.occurrence)?.sql()))
            } else {
                Ok(format!("\"truss\".\"query_objects\"({binding_slot}::pg_catalog.jsonb,{}::pg_catalog.int4) AS {}",m.record(&s.record)?.type_id,Identifier::new(&s.occurrence)?.sql()))
            }
        };
        let mut from = source(&plan.source)?;
        for join in &plan.joins {
            let predicates = join
                .on
                .iter()
                .map(|p| build.predicate(p, false))
                .collect::<Result<Vec<_>>>()?;
            if predicates.is_empty() {
                return Err(unsupported("Typed join predicate required"));
            }
            from.push_str(&format!(
                " INNER JOIN {} ON {}",
                source(&join.right)?,
                predicates.join(" AND ")
            ));
        }
        let filters = plan
            .filters
            .iter()
            .enumerate().map(|(index, p)| build.predicate(p, self.continuation_filter == Some(index)))
            .collect::<Result<Vec<_>>>()?;
        let mut projections = vec![];
        let mut columns = vec![];
        for (i, o) in plan.outputs.iter().enumerate() {
            let (sql, representation, ids) = match &o.expression {
                app::Expression::Count { logical_type } => (
                    "pg_catalog.count(*)::pg_catalog.text".into(),
                    Representation::Scalar {
                        logical_type: logical_type.clone(),
                        carrier: ScalarCarrier::Text,
                        decoder: ScalarDecoder::ExactInteger,
                    },
                    vec![plan.source.record.clone()],
                ),
                app::Expression::Field { scan, identity } => {
                    let d = build.descriptor(identity)?;
                    match &d.shape {
                        Shape::Scalar { logical_type } => {
                            let (_value, _) = build.scalar(scan, identity, logical_type)?;
                            if d.availability.as_deref() == Some("required") {
                                (
                                    build.projected_scalar(scan, identity, logical_type)?,
                                    Representation::Scalar {
                                        logical_type: logical_type.clone(),
                                        carrier: ScalarCarrier::Text,
                                        decoder: decoder(&logical_type.family),
                                    },
                                    vec![identity.clone()],
                                )
                            } else {
                                (
                                    format!(
                                        "({})::pg_catalog.text",
                                        build.envelope(scan, identity)?
                                    ),
                                    Representation::Value {
                                        descriptor: identity.clone(),
                                        native_null: true,
                                        outer_join: None,
                                    },
                                    vec![identity.clone()],
                                )
                            }
                        }
                        Shape::Sequence { .. } | Shape::Map { .. } if self.container_queries => (
                            format!("({})::pg_catalog.text", build.projected_value(scan, identity)?),
                            Representation::Value { descriptor: identity.clone(), native_null: true, outer_join: None },
                            vec![identity.clone()],
                        ),
                        Shape::ExactScalar { .. } => (
                            format!("({})::pg_catalog.text", build.projected_value(scan, identity)?),
                            Representation::Value { descriptor: identity.clone(), native_null: true, outer_join: None },
                            vec![identity.clone()],
                        ),
                        Shape::Record { members } => {
                            let mut pairs = vec![];
                            for member in members {
                                let field = build.descriptor(&member.identity)?;
                                if !matches!(field.shape, Shape::Scalar { .. } | Shape::ExactScalar { .. }) && !(self.container_queries && matches!(field.shape, Shape::Sequence { .. } | Shape::Map { .. })) {
                                    return Err(unsupported("Scalar entity member required"));
                                }
                                pairs.push(literal(&member.name)?);
                                pairs.push(build.projected_value(scan, &member.identity)?);
                            }
                            (
                                format!(
                                    "pg_catalog.jsonb_build_object({})::pg_catalog.text",
                                    pairs.join(",")
                                ),
                                Representation::Value {
                                    descriptor: identity.clone(),
                                    native_null: false,
                                    outer_join: None,
                                },
                                vec![identity.clone()],
                            )
                        }
                        _ => return Err(unsupported("Structured property unavailable")),
                    }
                }
                app::Expression::NativeMetadata { scan, record, metadata, logical_type } if self.complete_entities => {
                    if build.aliases.get(scan) != Some(record) { return Err(fail("Metadata source occurrence changed")); }
                    let name = match metadata {
                        app::NativeMetadata::Version => "_version", app::NativeMetadata::CreatedBy => "_created_by",
                        app::NativeMetadata::UpdatedBy => "_updated_by",
                        app::NativeMetadata::CreatedAt => "_created_at", app::NativeMetadata::UpdatedAt => "_updated_at", app::NativeMetadata::OriginalAuthor => "_original_author",
                        app::NativeMetadata::OriginalAt => "_original_at",
                        app::NativeMetadata::LoadId => "_load_id",
                    };
                    let helper=if matches!(metadata,app::NativeMetadata::LoadId) {
                        if !self.import_loads { return Err(unsupported("Import load entity profile requires runtime0.4.8")); }
                        "query_object_load_id"
                    } else if matches!(metadata,app::NativeMetadata::CreatedAt|app::NativeMetadata::UpdatedAt) {
                        if !self.entity_times { return Err(unsupported("Timestamp entity profile requires runtime0.4.7")); }
                        "query_object_times"
                    } else { "query_object_attribution" };
                    let sql = format!("(\"truss\".\"{helper}\"({binding_slot}::pg_catalog.jsonb,{}::pg_catalog.int4,{}.id)->{})::pg_catalog.text", m.record(record)?.type_id, Identifier::new(scan)?.sql(), literal(name)?);
                    (sql, Representation::NativeMetadata { metadata: metadata.clone(), logical_type: logical_type.clone(), carrier: ScalarCarrier::Text,
                        decoder: if matches!(metadata, app::NativeMetadata::Version) { ScalarDecoder::ExactInteger } else { ScalarDecoder::Text } }, vec![record.clone()])
                },
                app::Expression::CompleteRelatedKeys { scan, relationship, bound } if self.complete_entities => (
                    build.complete_related(scan, relationship, *bound)?,
                    Representation::CompleteRelatedKeys { relationship: relationship.identity.clone(), key: relationship.target_key.clone(), bound: *bound },
                    vec![relationship.from.clone(), relationship.to.clone()],
                ),
                app::Expression::RelatedKeys {
                    scan,
                    relationship,
                    bound,
                } if self.relationships => (
                    build.related(scan, relationship, None, Some(*bound))?,
                    Representation::RelatedKeys {
                        relationship: relationship.identity.clone(),
                        key: relationship.target_key.clone(),
                        bound: *bound,
                    },
                    vec![relationship.from.clone()],
                ),
                _ => {
                    return Err(unsupported(
                        "Selected aggregate or relationship unavailable",
                    ))
                }
            };
            projections.push(format!("{sql} AS {}", Identifier::new(&o.name)?.sql()));
            columns.push(Column {
                position: i + 1,
                output_name: o.name.clone(),
                carrier_name: None,
                representation,
                source_identities: ids,
                nullable: false,
            });
        }
        let order = plan
            .order
            .iter()
            .map(|f| {
                build
                    .scalar(&f.scan, &f.identity, &f.logical_type)
                    .map(|v| format!("{} ASC", v.0))
            })
            .collect::<Result<Vec<_>>>()?;
        let mut sql = format!("SELECT {} FROM {from}", projections.join(","));
        if !filters.is_empty() {
            sql.push_str(&format!(" WHERE {}", filters.join(" AND ")));
        }
        if !plan.groups.is_empty() {
            let groups = plan.groups.iter().map(|f| {
                build.scalar(&f.scan, &f.identity, &f.logical_type).map(|v| v.0)
            }).collect::<Result<Vec<_>>>()?;
            sql.push_str(&format!(" GROUP BY {}", groups.join(",")));
        }
        if !order.is_empty() {
            sql.push_str(&format!(" ORDER BY {}", order.join(",")));
        }
        let limit = plan
            .limit
            .map(u32::from)
            .unwrap_or(m.binding.maximum_result_rows + 1)
            .min(m.binding.maximum_result_rows + 1);
        sql.push_str(&format!(" LIMIT {limit}"));
        Ok(TargetPlan(Emission {
            sql,
            parameters: build.parameters.into_slots(),
            columns,
            obligations: vec![obligation()],
        }))
    }
    fn emit(&self, _: &Context<'_>, p: &TargetPlan) -> Result<Emission> {
        Ok(Emission {
            sql: p.0.sql.clone(),
            parameters: p.0.parameters.clone(),
            columns: p.0.columns.clone(),
            obligations: p.0.obligations.clone(),
        })
    }
}
fn decoder(f: &Family) -> ScalarDecoder {
    match f {
        Family::String => ScalarDecoder::Text,
        Family::Boolean => ScalarDecoder::Boolean,
        Family::Integer => ScalarDecoder::ExactInteger,
        Family::Decimal => ScalarDecoder::ExactDecimal,
    }
}
struct Build<'a> {
    mapping: &'a Mapping,
    plan: &'a app::Plan,
    aliases: BTreeMap<String, Identity>,
    parameters: Parameters,
    binding_slot: String,
    relationship_scope: u32,
    typed_page_mode: bool,
    timestamp_equality: bool,
    recursive_containers: bool,
}
impl Build<'_> {
    fn descriptor(&self, id: &Identity) -> Result<&Descriptor> {
        self.plan
            .type_graph
            .iter()
            .find(|d| d.identity == *id)
            .ok_or_else(|| fail("Resolved descriptor missing"))
    }
    fn cell(&self, scan: &str, id: &Identity) -> Result<(String, String)> {
        let owner = self
            .aliases
            .get(scan)
            .ok_or_else(|| fail("Scan mapping missing"))?;
        let f = self.mapping.field(owner, id)?;
        let alias = Identifier::new(scan)?.sql();
        Ok((
            format!("{alias}.props->'{}'", f.property_id),
            format!("{alias}.props ? '{}'", f.property_id),
        ))
    }
    fn scalar(&self, scan: &str, id: &Identity, logical: &LogicalType) -> Result<(String, String)> {
        let (cell, present) = self.cell(scan, id)?;
        let sql = match logical.family {
            Family::String => format!("({cell}->>'text') COLLATE pg_catalog.\"C\""),
            Family::Boolean => format!("({cell}->>'value')::pg_catalog.bool"),
            Family::Integer | Family::Decimal => format!("({cell}->>'text')::pg_catalog.numeric"),
        };
        Ok((sql, present))
    }
    fn projected_scalar(&self, scan: &str, id: &Identity, logical: &LogicalType) -> Result<String> {
        if logical.family == Family::Decimal {
            let (cell, _) = self.cell(scan, id)?;
            Ok(format!("{cell}->>'text'"))
        } else {
            let (value, _) = self.scalar(scan, id, logical)?;
            Ok(format!("({value})::pg_catalog.text"))
        }
    }
    fn scalar_json(&self, scan: &str, id: &Identity, d: &Descriptor) -> Result<String> {
        let (cell, _) = self.cell(scan, id)?;
        match &d.shape {
            Shape::Sequence{..}|Shape::Map{..} if self.recursive_containers=>self.recursive_cell_json(&cell,&d.identity,0),
            Shape::ExactScalar { .. } => Ok(format!("pg_catalog.to_jsonb({cell}->>'text')")),
            Shape::Scalar { logical_type } => match logical_type.family {
                Family::Integer | Family::Decimal => Ok(format!("pg_catalog.to_jsonb({cell}->>'text')")),
                _ => {
                    let (value, _) = self.scalar(scan, id, logical_type)?;
                    Ok(format!("pg_catalog.to_jsonb({value})"))
                }
            },
            Shape::Sequence { .. } => {
                let value = "CASE WHEN member.value->>'kind'='boolean' THEN member.value->'value' ELSE pg_catalog.to_jsonb(member.value->>'text') END";
                Ok(format!("(SELECT COALESCE(pg_catalog.jsonb_agg({value} ORDER BY member.n),'[]'::pg_catalog.jsonb) FROM pg_catalog.jsonb_array_elements({cell}->'items') WITH ORDINALITY AS member(value,n))"))
            },
            Shape::Map { .. } => {
                let value = "CASE WHEN member.value->'value'->>'kind'='boolean' THEN member.value->'value'->'value' ELSE pg_catalog.to_jsonb(member.value->'value'->>'text') END";
                Ok(format!("(SELECT COALESCE(pg_catalog.jsonb_object_agg(member.value->>'key',{value}),'{{}}'::pg_catalog.jsonb) FROM pg_catalog.jsonb_array_elements({cell}->'entries') AS member(value))"))
            },
            _ => Err(unsupported("Scalar projection required")),
        }
    }
    fn recursive_cell_json(&self,cell:&str,id:&Identity,depth:usize)->Result<String>{
        if depth>32{return Err(unsupported("Container projection depth limit"));}
        let descriptor=self.descriptor(id)?;
        match &descriptor.shape{
            Shape::ExactScalar{..}=>Ok(format!("pg_catalog.to_jsonb({cell}->>'text')")),
            Shape::Scalar{logical_type}=>if logical_type.family==Family::Boolean{Ok(format!("{cell}->'value'"))}else{Ok(format!("pg_catalog.to_jsonb({cell}->>'text')"))},
            Shape::Sequence{item}=>{
                let alias=format!("nested_{depth}");let child=self.recursive_cell_json(&format!("{alias}.value"),item,depth+1)?;
                Ok(format!("(SELECT COALESCE(pg_catalog.jsonb_agg({child} ORDER BY {alias}.n),'[]'::pg_catalog.jsonb) FROM pg_catalog.jsonb_array_elements({cell}->'items') WITH ORDINALITY AS {alias}(value,n))"))
            },
            Shape::Map{item}=>{
                let alias=format!("nested_{depth}");let child=self.recursive_cell_json(&format!("({alias}.value->'value')"),item,depth+1)?;
                Ok(format!("(SELECT COALESCE(pg_catalog.jsonb_object_agg({alias}.value->>'key',{child}),'{{}}'::pg_catalog.jsonb) FROM pg_catalog.jsonb_array_elements({cell}->'entries') AS {alias}(value))"))
            },
            _=>Err(unsupported("Recursive container projection unavailable")),
        }
    }
    fn projected_value(&self, scan: &str, id: &Identity) -> Result<String> {
        let d = self.descriptor(id)?;
        if d.availability.as_deref() == Some("required") {
            self.scalar_json(scan, id, d)
        } else { self.envelope(scan, id) }
    }
    fn envelope(&self, scan: &str, id: &Identity) -> Result<String> {
        let d = self.descriptor(id)?;
        let (cell, present) = self.cell(scan, id)?;
        let jsonvalue = self.scalar_json(scan, id, d)?;
        Ok(format!("CASE WHEN NOT ({present}) THEN pg_catalog.jsonb_build_object('state','absent') WHEN {cell}->>'kind'='null' THEN pg_catalog.jsonb_build_object('state','null') ELSE pg_catalog.jsonb_build_object('state','value','value',{jsonvalue}) END"))
    }
    fn value(&mut self, v: &app::Value) -> Result<String> {
        self.value_with_origin(v, None)
    }
    fn value_with_origin(&mut self, v: &app::Value, origin: Option<serde_json::Value>) -> Result<String> {
        match v {
            app::Value::Field { field } => Ok(self
                .scalar(&field.scan, &field.identity, &field.logical_type)?
                .0),
            app::Value::Literal {
                value,
                logical_type,
                ..
            }
            | app::Value::Parameter {
                value,
                logical_type,
                ..
            } => {
                let slot = self
                    .parameters
                    .push(logical_type.clone(), value.clone(), origin.unwrap_or_else(|| json!(v)))?;
                Ok(match logical_type.family {
                    Family::String => format!("{slot}::pg_catalog.text COLLATE pg_catalog.\"C\""),
                    Family::Boolean => format!("{slot}::pg_catalog.bool"),
                    Family::Integer | Family::Decimal => format!("{slot}::pg_catalog.numeric"),
                })
            }
        }
    }
    fn related(
        &mut self,
        scan: &str,
        rel: &RelationshipRead,
        key_values: Option<&[app::Value]>,
        bound: Option<u16>,
    ) -> Result<String> {
        let mapped = self
            .mapping
            .binding
            .relationships
            .iter()
            .find(|r| r.identity == json!(rel.identity))
            .ok_or_else(|| fail("Relationship mapping missing"))?;
        if self.aliases.get(scan) != Some(&rel.from) {
            return Err(fail("Relationship source occurrence changed"));
        }
        self.relationship_scope += 1;
        let target = format!("weft_runtime_related_{}_target", self.relationship_scope);
        let edge = format!("weft_runtime_related_{}_edge", self.relationship_scope);
        self.aliases.insert(target.clone(), rel.to.clone());
        let target_alias = Identifier::new(&target)?.sql();
        let edge_alias = Identifier::new(&edge)?.sql();
        let outer = Identifier::new(scan)?.sql();
        let (source_column, target_column) = if rel.inverse {
            ("target_id", "source_id")
        } else {
            ("source_id", "target_id")
        };
        let from=format!("\"truss\".\"query_edges\"({}::pg_catalog.jsonb,{}::pg_catalog.int4) AS {edge_alias} JOIN \"truss\".\"query_objects\"({}::pg_catalog.jsonb,{}::pg_catalog.int4) AS {target_alias} ON {target_alias}.id={edge_alias}.{target_column}",self.binding_slot,mapped.relationship_type_id,self.binding_slot,self.mapping.record(&rel.to)?.type_id);
        let mut predicates = vec![format!("{edge_alias}.{source_column}={outer}.id")];
        let ordered = rel
            .target_key
            .fields
            .iter()
            .zip(&rel.target_key.types)
            .map(|(id, t)| self.scalar(&target, id, t).map(|v| v.0))
            .collect::<Result<Vec<_>>>()?;
        if let Some(values) = key_values {
            if values.len() != ordered.len() {
                return Err(fail("Relationship key arity mismatch"));
            }
            for (left, right) in ordered.iter().zip(values) {
                predicates.push(format!("({left})=({})", self.value(right)?));
            }
            return Ok(format!(
                "EXISTS (SELECT 1 FROM {from} WHERE {})",
                predicates.join(" AND ")
            ));
        }
        let bound = bound
            .filter(|b| (1..=1000).contains(b))
            .ok_or_else(|| unsupported("Related key bound requires 1 through 1000"))?;
        let order = ordered.join(",");
        let projected = rel.target_key.fields.iter().zip(&rel.target_key.types).map(|(id,t)| {
            if matches!(t.family, Family::Integer | Family::Decimal) {
                self.cell(&target,id).map(|(cell,_)|format!("{cell}->>'text'"))
            } else { self.scalar(&target,id,t).map(|v|v.0) }
        }).collect::<Result<Vec<_>>>()?.join(",");
        let inner=format!("SELECT pg_catalog.jsonb_build_array({projected}) AS k,pg_catalog.row_number() OVER (ORDER BY {order}) AS ord FROM {from} WHERE {} ORDER BY {order} LIMIT {}",predicates.join(" AND "),u32::from(bound)+1);
        Ok(format!("(SELECT pg_catalog.jsonb_build_object('items',COALESCE(pg_catalog.jsonb_agg(k ORDER BY ord) FILTER (WHERE ord<={bound}),'[]'::pg_catalog.jsonb),'truncated',pg_catalog.count(*)>{bound})::pg_catalog.text FROM ({inner}) AS weft_runtime_related_{}_bounded)",self.relationship_scope))
    }
    fn complete_related(&mut self, scan: &str, relationship: &RelationshipRead, bound: u16) -> Result<String> {
        if !(1..=100).contains(&bound) { return Err(unsupported("Complete relationship bound requires 1 through 100")); }
        let bounded = self.related(scan, relationship, None, Some(bound))?;
        Ok(format!("\"truss\".\"query_complete_related_keys\"(({bounded})::pg_catalog.jsonb,{bound}::pg_catalog.int4,{}::pg_catalog.int4)::pg_catalog.text", relationship.target_key.fields.len()))
    }
    fn predicate(&mut self, p: &app::Predicate, continuation: bool) -> Result<String> {
        match p {
            app::Predicate::ExactEqual { left, right } => {
                use weft_core::application_model::ExactScalarType;
                if left.scalar_type==ExactScalarType::Timestamp && !self.timestamp_equality {return Err(unsupported("Selected backend does not admit exact instant equality"));}
                let comparison=|text:String|if left.scalar_type==ExactScalarType::Binary {
                    format!("pg_catalog.decode({text},'base64')")
                } else {format!("\"truss\".\"query_timestamp_instant_key\"({text},'umf-rfc3339-exact-instant/1.0.0'::pg_catalog.text)")};
                let (cell, _) = self.cell(&left.scan, &left.identity)?;
                let l = comparison(format!("{cell}->>'text'"));
                let r = match right {
                    app::ExactValue::Field { field } => {
                        if field.scalar_type!=left.scalar_type {return Err(unsupported("Exact comparison families must match"));}
                        let (cell, _) = self.cell(&field.scan, &field.identity)?;
                        comparison(format!("{cell}->>'text'"))
                    },
                    app::ExactValue::Literal { value, scalar_type, .. } | app::ExactValue::Parameter { value, scalar_type, .. } => {
                        if scalar_type!=&left.scalar_type {return Err(unsupported("Exact comparison families must match"));}
                        let slot = if left.scalar_type==ExactScalarType::Binary {self.parameters.push_binary(value.clone(),json!(right))?} else {self.parameters.push_timestamp(value.clone(),json!(right))?};
                        comparison(format!("{slot}::pg_catalog.text"))
                    },
                };
                Ok(format!("({l} = {r})"))
            }
            app::Predicate::Equal { left, right } => {
                let (l, _) = self.scalar(&left.scan, &left.identity, &left.logical_type)?;
                let r = self.value(right)?;
                Ok(format!("({l} = {r})"))
            }
            app::Predicate::LexicographicGreater { columns, values } => {
                if self.plan.page_key.is_none() {
                    return Err(unsupported(
                        "Authored page profile required for continuation",
                    ));
                }
                if columns.is_empty()
                    || columns.len() != values.len()
                    || columns
                        .iter()
                        .any(|f| f.logical_type.nullable || (!self.typed_page_mode && f.logical_type.family != Family::String))
                {
                    return Err(unsupported(
                        "Required string continuation components required",
                    ));
                }
                let left = columns
                    .iter()
                    .map(|f| {
                        self.scalar(&f.scan, &f.identity, &f.logical_type)
                            .map(|v| v.0)
                    })
                    .collect::<Result<Vec<_>>>()?;
                let right = values
                    .iter()
                    .enumerate().map(|(component, v)| self.value_with_origin(v,
                        continuation.then(|| json!({"kind":"page-continuation","component":component}))))
                    .collect::<Result<Vec<_>>>()?;
                Ok(format!(
                    "(ROW({})>ROW({}))",
                    left.join(","),
                    right.join(",")
                ))
            }
            app::Predicate::HasRelated {
                scan,
                relationship,
                key,
            } => self.related(scan, relationship, Some(key), None),
        }
    }
}
pub fn compile_json(request: &str) -> String {
    compile_internal(request, None)
}
/// Authored-key SQL page composition over the same accepted Truss runtime binding.
pub fn compile_page_json(request: &str, page_size: u16, after: Option<Vec<String>>) -> String {
    compile_internal(request, Some((page_size, after)))
}
fn compile_internal(request: &str, page: Option<(u16, Option<Vec<String>>)>) -> String {
    let version = weft_core::json::checked_json(request)
        .ok()
        .and_then(|v| v["target"]["backendVersion"].as_str().map(str::to_owned));
    let owned_runtime=version.as_deref()==Some("0.4.35");
    let recursive_containers=owned_runtime || version.as_deref()==Some("0.4.34");
    let keyed_rebinding=recursive_containers || version.as_deref()==Some("0.4.33");
    let key_backfill=keyed_rebinding || version.as_deref()==Some("0.4.32");
    let required_rebinding=key_backfill || version.as_deref()==Some("0.4.31");
    let catalog_rebinding=required_rebinding || version.as_deref()==Some("0.4.30");
    let timestamp_equality=catalog_rebinding || version.as_deref()==Some("0.4.29");
    let source_references = timestamp_equality || version.as_deref() == Some("0.4.28");
    let field_names = source_references || version.as_deref() == Some("0.4.27");
    let decimal_scale2 = field_names || version.as_deref() == Some("0.4.26");
    let optional_binary = decimal_scale2 || version.as_deref() == Some("0.4.25");
    let binary_equality = optional_binary || version.as_deref() == Some("0.4.24");
    let relationship_names = binary_equality || version.as_deref() == Some("0.4.23");
    let parallel_edges = relationship_names || version.as_deref() == Some("0.4.22");
    let engine_imports = parallel_edges || version.as_deref() == Some("0.4.21");
    let container_tightening = engine_imports || version.as_deref() == Some("0.4.20");
    let relationship_tightening = container_tightening || version.as_deref() == Some("0.4.19");
    let required_tightening = relationship_tightening || version.as_deref() == Some("0.4.18");
    let record_evolution = required_tightening || version.as_deref() == Some("0.4.17");
    let typed_key_pages = record_evolution || version.as_deref() == Some("0.4.16");
    let typed_relationship_keys = typed_key_pages || version.as_deref() == Some("0.4.15");
    let container_queries = typed_relationship_keys || version.as_deref() == Some("0.4.14");
    let scalar_containers = container_queries || version.as_deref() == Some("0.4.13");
    let facet_evolution = scalar_containers || version.as_deref() == Some("0.4.12");
    let live_traversal = facet_evolution || version.as_deref() == Some("0.4.11");
    let typed_keys = live_traversal || version.as_deref() == Some("0.4.10");
    let numeric_facets = typed_keys || version.as_deref() == Some("0.4.9");
    let import_loads = numeric_facets || version.as_deref() == Some("0.4.8");
    let entity_times = import_loads || version.as_deref() == Some("0.4.7");
    let complete_entities = entity_times || version.as_deref() == Some("0.4.6");
    let retained_metadata = complete_entities || version.as_deref() == Some("0.4.5");
    let exact_scalars = retained_metadata || version.as_deref() == Some("0.4.4");
    let selected_metadata = exact_scalars || version.as_deref() == Some("0.4.3");
    let groups = selected_metadata || version.as_deref() == Some("0.4.2");
    let release_030 = groups || version.as_deref() == Some("0.4.1");
    let pages = release_030 || version.as_deref() == Some("0.4.0");
    let relationships = pages || version.as_deref() == Some("0.3.0");
    let numeric = relationships || version.as_deref() == Some("0.2.0");
    let mut factory =
        |_: &Catalog, plan: Plan<'_>, target: weft_core::compile::CompositionInput<'_>| {
            let backend = Runtime {
                exact_numbers: numeric,
                relationships,
                pages,
                release_030,
                groups,
                selected_metadata,
                exact_scalars,
                retained_metadata,
                complete_entities,
                entity_times,
                import_loads,
                numeric_facets,
                typed_keys,
                live_traversal,
                facet_evolution,
                scalar_containers,
                container_queries,
                typed_relationship_keys,
                typed_key_pages,
                record_evolution,
                required_tightening,
                relationship_tightening,
                container_tightening,
                relationship_names,
                binary_equality,
                optional_binary,
                decimal_scale2,
                field_names,
                source_references,
                timestamp_equality,
                catalog_rebinding,
                key_backfill,
                keyed_rebinding,
                recursive_containers,
                owned_runtime,
                required_rebinding,
                parallel_edges,
                engine_imports,
                typed_page_mode: typed_key_pages && page.is_some(),
                continuation_filter: if page.as_ref().is_some_and(|(_, after)| after.is_some()) {
                    match plan { Plan::V02(p) => Some(p.filters.len()-1), _ => return Err(fail("Page application plan required")) }
                } else { None },
            };
            if page.is_some() && !complete_entities { return Err(fail("SQL page transport requires runtime backend 0.4.6")); }
            if target.backend_id != BACKEND
                || target.backend_version != backend.version()
                || target.target_profile != backend.target()
            {
                return Err(fail("Explicit runtime backend/version/target required"));
            }
            let mut r = Registry::default();
            r.register(backend)?;
            Ok(r)
        };
    let compiler = weft_core::compile::Compiler::default();
    if let Some((size, after)) = page.clone() {
        if typed_key_pages { compiler.compile_typed_page_json_with_exact_numbers_factory(request,size,after,&mut factory) } else { compiler.compile_page_json_with_exact_numbers_factory(request, size, after, &mut factory) }
    } else if numeric {
        compiler.compile_json_with_exact_numbers_factory(request, &mut factory)
    } else {
        compiler.compile_json_with_factory(request, &mut factory)
    }
}
