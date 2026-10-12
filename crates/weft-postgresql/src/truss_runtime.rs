//! Fixed Truss runtime mapping over its independently authorized scalar source.
//! The host constructs this binding from accepted bytes and actual catalog IDs.
use crate::{Identifier, Parameters};
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    application_ir as app,
    application_model::{Descriptor, Shape},
    backend::*,
    error::{Diagnostic, Result},
    ir::{Family, Identity, LogicalType},
    model::Catalog,
};
pub const PROFILE: &str = "truss-runtime-binding/0.1";
pub const TARGET: &str = "truss-runtime-pg18.6/0.1";
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
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FieldBinding {
    identity: Identity,
    property_id: String,
}
pub struct Runtime;
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
fn scalar_type(d: &Descriptor) -> Result<&LogicalType> {
    match &d.shape {
        Shape::Scalar { logical_type } => {
            let facets = &logical_type.facets;
            let accepted = match logical_type.family {
                Family::String | Family::Boolean => facets == &json!({}),
                Family::Integer => facets == &json!({}),
                Family::Decimal => false,
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
        Ok(Manifest{backend_id:BACKEND.into(),backend_version:"0.1.0".into(),interface_version:"weft-backend/0.2.0".into(),binding_profile:PROFILE.into(),language_profiles:language.clone(),target_profiles:vec![TargetProfile{id:TARGET.into(),engine:"postgresql".into(),engine_version:"18.6".into(),session_settings:json!({"server_encoding":"UTF8","search_path":"pg_catalog,pg_temp","read_context":"live"}),storage_layout_revision:"truss-runtime-scalar-query/0.1".into(),publication_revision:"caller-transaction/0.1".into()}],capabilities:["scan","project","filter","innerJoin","equal","and","type.string","type.boolean","type.integer","aggregate","aggregate.count","parameter.named","order.asc","limit","value.presence","value.nativeNull","value.structured","project.entity"].iter().map(|id|Capability{id:(*id).into(),target_profiles:vec![TARGET.into()],language_profiles:language.clone(),logical_domain:json!({"subset":"rootless scalar string/boolean/mathematical-integer records; equality joins/filters, counts, exact complete records"}),result_domain:json!({"transport":"exact text and presence envelopes"}),constraints:vec!["No relationship or structured-property lowering; no page-key profile".into()],obligations:vec![obligation()],status:Status::Candidate,evidence:vec![]}).collect(),evidence:vec![]})
    }
    fn validate_binding(&self, c: &Context<'_>) -> Result<Validated<Mapping>> {
        if c.binding.profile != PROFILE
            || weft_core::json::sha256(c.binding.json.as_bytes()) != c.binding.sha256
        {
            return Err(fail("Exact runtime binding pin required"));
        }
        let binding: Binding = serde_json::from_value(
            weft_core::json::checked_json(&c.binding.json)
                .map_err(|_| fail("Runtime binding JSON refused"))?,
        )
        .map_err(|_| fail("Runtime binding shape refused"))?;
        if binding.interface_version != PROFILE
            || binding.schema != "truss"
            || binding.installation_id.is_empty()
            || binding.release_id != "truss-catalog-runtime/0.26-candidate"
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
                scalar_type(d)?;
                fields.insert(key(&f.identity));
            }
        }
        if c.selection
            .records
            .iter()
            .any(|r| !records.contains(&key(r)))
            || c.selection.fields.iter().any(|f| !fields.contains(&key(f)))
            || !c.selection.relationships.is_empty()
            || c.selection
                .types
                .iter()
                .any(|t| !fields.contains(&key(t)) && !records.contains(&key(t)))
        {
            return Err(fail("Resolved selection lacks complete runtime mapping"));
        }
        Ok(Validated {
            mapping: Mapping {
                binding,
                original: c.binding.json.clone(),
            },
            additional_capabilities: vec!["value.nativeNull".into()],
            coverage: c.selection.clone(),
            obligations: vec![obligation()],
        })
    }
    fn assess(&self, c: &Context<'_>, _: &Mapping) -> Result<Vec<Assessment>> {
        let manifest = self.describe()?;
        c.plan
            .capabilities()
            .iter()
            .chain(std::iter::once(&"value.nativeNull".to_owned()))
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
        if plan.read_profile.is_some() || !plan.groups.is_empty() || plan.page_key.is_some() {
            return Err(unsupported("Runtime page-key/group profile unavailable"));
        }
        let mut build = Build {
            mapping: m,
            plan,
            aliases: BTreeMap::new(),
            parameters: Parameters::default(),
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
        let source = |s: &app::Scan| -> Result<String> {
            Ok(format!("\"truss\".\"query_objects\"({binding_slot}::pg_catalog.jsonb,{}::pg_catalog.int4) AS {}",m.record(&s.record)?.type_id,Identifier::new(&s.occurrence)?.sql()))
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
                            let (value, _) = build.scalar(scan, identity, logical_type)?;
                            if d.availability.as_deref() == Some("required") {
                                (
                                    format!("({value})::pg_catalog.text"),
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
                                        Family::Integer | Family::Decimal => format!(
                                            "pg_catalog.to_jsonb(({value})::pg_catalog.text)"
                                        ),
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
    fn predicate(&mut self, p: &app::Predicate) -> Result<String> {
        match p {
            app::Predicate::Equal { left, right } => {
                let (l, _) = self.scalar(&left.scan, &left.identity, &left.logical_type)?;
                let r = self.value(right)?;
                Ok(format!("({l} = {r})"))
            }
            _ => Err(unsupported("Runtime predicate requires typed equality")),
        }
    }
}
pub fn compile_json(request: &str) -> String {
    let mut factory =
        |_: &Catalog, _: Plan<'_>, target: weft_core::compile::CompositionInput<'_>| {
            if target.backend_id != BACKEND
                || target.backend_version != "0.1.0"
                || target.target_profile != TARGET
            {
                return Err(fail("Explicit runtime backend/version/target required"));
            }
            let mut r = Registry::default();
            r.register(Runtime)?;
            Ok(r)
        };
    weft_core::compile::Compiler::default().compile_json_with_factory(request, &mut factory)
}
