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
        if self.groups {
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
    fn additions(&self) -> Vec<String> {
        let mut v = vec!["value.nativeNull".into()];
        if self.exact_numbers {
            v.extend([
                "type.integer.unbounded".into(),
                "type.decimal.unbounded".into(),
            ]);
        }
        v
    }
}
pub struct Mapping {
    binding: Binding,
    original: String,
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
fn scalar_type(d: &Descriptor, exact_numbers: bool) -> Result<&LogicalType> {
    match &d.shape {
        Shape::Scalar { logical_type } => {
            let facets = &logical_type.facets;
            let accepted = match logical_type.family {
                Family::String | Family::Boolean => facets == &json!({}),
                Family::Integer => facets == &json!({}),
                Family::Decimal => exact_numbers && facets == &json!({}),
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
            ir_version: "weft-ir/0.2.0".into(),
        }];
        Ok(Manifest{backend_id:BACKEND.into(),backend_version:self.version().into(),interface_version:"weft-backend/0.2.0".into(),binding_profile:self.profile().into(),language_profiles:language.clone(),target_profiles:vec![TargetProfile{id:self.target().into(),engine:"postgresql".into(),engine_version:"18.6".into(),session_settings:json!({"server_encoding":"UTF8","search_path":"pg_catalog,pg_temp","read_context":"live"}),storage_layout_revision:if self.pages{"truss-runtime-keyset-query/0.4".into()}else if self.relationships{"truss-runtime-relationship-query/0.3".into()}else if self.exact_numbers{"truss-runtime-scalar-query/0.2".into()}else{"truss-runtime-scalar-query/0.1".into()},publication_revision:"caller-transaction/0.1".into()}],capabilities:["scan","project","filter","innerJoin","equal","and","type.string","type.boolean","type.integer","aggregate","aggregate.count","parameter.named","order.asc","limit","value.presence","value.nativeNull","value.structured","project.entity"].iter().map(|id|(*id).to_owned()).chain(if self.exact_numbers{vec!["type.decimal".into(),"type.integer.unbounded".into(),"type.decimal.unbounded".into()]}else{vec![]}).chain(if self.relationships{vec!["relationship.exists".into(),"relationship.boundedKeys".into(),"relationship.inverse".into()]}else{vec![]}).chain(if self.pages{vec!["key.uniqueStable".into(),"compare.lexicographicGreater".into()]}else{vec![]}).chain(if self.groups{vec!["group".into()]}else{vec![]}).map(|id|Capability{id:id.clone(),target_profiles:vec![self.target().into()],language_profiles:language.clone(),logical_domain:json!({"subset":"rootless scalar records; exact string/boolean and explicitly admitted mathematical integer/decimal domains; native-domain refusal remains host-owned"}),result_domain:json!({"transport":"exact text and presence envelopes"}),constraints:vec![if self.groups{"Required non-nullable string/boolean grouping; count-summary requires complete grouping order and LIMIT; no optional/numeric group keys or structured properties".into()}else if self.pages{"Required string authored-key entity pages, related entity pages and ungrouped count summary; no grouped summary or structured properties".into()}else if self.relationships{"Independent directed monomorphic relationships with required string keys only; no structured-property or page-key profile".into()}else{"No relationship or structured-property lowering; no page-key profile".into()}],obligations:vec![obligation()],status:Status::Candidate,evidence:vec![]}).collect(),evidence:vec![]})
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
                != if self.release_030 {
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
        let mut native_types = BTreeSet::new();
        let mut native_fields = BTreeSet::new();
        for r in &binding.records {
            native(&r.type_id)?;
            if !records.insert(key(&r.identity)) || !native_types.insert(&r.type_id) {
                return Err(fail("Duplicate record mapping"));
            }
            let original = c.catalog.record_by_identity(&r.identity)?;
            let descriptor = c.catalog.entity_descriptor(&original)?;
            if descriptor.members.len() != r.fields.len() {
                return Err(fail("Complete original record member mapping required"));
            }
            for member in &descriptor.members {
                let f = r
                    .fields
                    .iter()
                    .find(|f| f.identity == member.identity)
                    .ok_or_else(|| fail("Original member mapping missing"))?;
                native(&f.property_id)?;
                if !native_fields.insert(&f.property_id) {
                    return Err(fail("Duplicate native property mapping"));
                }
                let d = descriptor
                    .graph
                    .iter()
                    .find(|d| d.identity == f.identity)
                    .ok_or_else(|| fail("Original member descriptor missing"))?;
                scalar_type(d, self.exact_numbers)?;
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
                .any(|t| !fields.contains(&key(t)) && !records.contains(&key(t)))
        {
            return Err(fail("Resolved selection lacks complete runtime mapping"));
        }
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
                    if let app::Expression::RelatedKeys { relationship, .. } = &o.expression {
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
                    if authored.fields.is_empty()
                        || authored.fields.len() != authored.types.len()
                        || authored.types.iter().any(|t| {
                            t.family != Family::String || t.facets != json!({}) || t.nullable
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
                        .any(|t| t.family != Family::String || t.facets != json!({}) || t.nullable)
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
            },
            additional_capabilities: self.additions(),
            coverage: c.selection.clone(),
            obligations: vec![obligation()],
        })
    }
    fn assess(&self, c: &Context<'_>, _: &Mapping) -> Result<Vec<Assessment>> {
        let manifest = self.describe()?;
        let additions = self.additions();
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
                .map(|p| build.predicate(p))
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
            .map(|p| build.predicate(p))
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
                        Shape::Record { members } => {
                            let mut pairs = vec![];
                            for member in members {
                                let field = build.descriptor(&member.identity)?;
                                let Shape::Scalar { logical_type } = &field.shape else {
                                    return Err(unsupported("Scalar entity member required"));
                                };
                                let (value, _) =
                                    build.scalar(scan, &member.identity, logical_type)?;
                                pairs.push(literal(&member.name)?);
                                pairs.push(if field.availability.as_deref() == Some("required") {
                                    match logical_type.family {
                                        Family::Integer | Family::Decimal => {
                                            let (cell, _) = build.cell(scan, &member.identity)?;
                                            format!("pg_catalog.to_jsonb({cell}->>'text')")
                                        }
                                        _ => format!("pg_catalog.to_jsonb({value})"),
                                    }
                                } else {
                                    build.envelope(scan, &member.identity)?
                                });
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
    fn envelope(&self, scan: &str, id: &Identity) -> Result<String> {
        let d = self.descriptor(id)?;
        let Shape::Scalar { logical_type } = &d.shape else {
            return Err(unsupported("Scalar envelope required"));
        };
        let (cell, present) = self.cell(scan, id)?;
        let (value, _) = self.scalar(scan, id, logical_type)?;
        let jsonvalue = match logical_type.family {
            Family::Integer | Family::Decimal => format!("pg_catalog.to_jsonb({cell}->>'text')"),
            _ => format!("pg_catalog.to_jsonb({value})"),
        };
        Ok(format!("CASE WHEN NOT ({present}) THEN pg_catalog.jsonb_build_object('state','absent') WHEN {cell}->>'kind'='null' THEN pg_catalog.jsonb_build_object('state','null') ELSE pg_catalog.jsonb_build_object('state','value','value',{jsonvalue}) END"))
    }
    fn value(&mut self, v: &app::Value) -> Result<String> {
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
                    .push(logical_type.clone(), value.clone(), json!(v))?;
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
        let inner=format!("SELECT pg_catalog.jsonb_build_array({order}) AS k,pg_catalog.row_number() OVER (ORDER BY {order}) AS ord FROM {from} WHERE {} ORDER BY {order} LIMIT {}",predicates.join(" AND "),u32::from(bound)+1);
        Ok(format!("(SELECT pg_catalog.jsonb_build_object('items',COALESCE(pg_catalog.jsonb_agg(k ORDER BY ord) FILTER (WHERE ord<={bound}),'[]'::pg_catalog.jsonb),'truncated',pg_catalog.count(*)>{bound})::pg_catalog.text FROM ({inner}) AS weft_runtime_related_{}_bounded)",self.relationship_scope))
    }
    fn predicate(&mut self, p: &app::Predicate) -> Result<String> {
        match p {
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
                        .any(|f| f.logical_type.family != Family::String || f.logical_type.nullable)
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
                    .map(|v| self.value(v))
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
            _ => Err(unsupported("Runtime predicate requires typed equality")),
        }
    }
}
pub fn compile_json(request: &str) -> String {
    let version = weft_core::json::checked_json(request)
        .ok()
        .and_then(|v| v["target"]["backendVersion"].as_str().map(str::to_owned));
    let groups = version.as_deref() == Some("0.4.2");
    let release_030 = groups || version.as_deref() == Some("0.4.1");
    let pages = release_030 || version.as_deref() == Some("0.4.0");
    let relationships = pages || version.as_deref() == Some("0.3.0");
    let numeric = relationships || version.as_deref() == Some("0.2.0");
    let mut factory =
        |_: &Catalog, _: Plan<'_>, target: weft_core::compile::CompositionInput<'_>| {
            let backend = Runtime {
                exact_numbers: numeric,
                relationships,
                pages,
                release_030,
                groups,
            };
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
    if numeric {
        compiler.compile_json_with_exact_numbers_factory(request, &mut factory)
    } else {
        compiler.compile_json_with_factory(request, &mut factory)
    }
}
