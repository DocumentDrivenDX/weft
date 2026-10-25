// SPDX-License-Identifier: MIT OR Apache-2.0
//! Owned, pinned frontend admission for the experimental raw-read profile.
//! Backends borrow this artifact; JSON, UMF identity and SQL parsing stay in core.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use sqlparser::{
    ast::{Query, Statement},
    dialect::PostgreSqlDialect,
    parser::Parser,
};
use std::collections::{BTreeMap, HashMap};

type Result<T> = std::result::Result<T, &'static str>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeDocumentMode {
    Runtime,
    Administrative,
    FiniteUnityCurrentKeyless,
    FiniteUnityCurrentPredicate,
    FiniteUnityCurrentSummary,
    FiniteUnityCurrentSum,
    FiniteUnityCurrentAvg,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Carrier {
    Integer,
    Decimal,
    String,
    Boolean,
    Uuid,
    Bits128,
    Date,
    Timestamp,
    #[serde(rename = "localTimestamp")]
    LocalTimestamp,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Column {
    pub logical: String,
    pub native: String,
    pub carrier: Carrier,
    pub nullable: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Table {
    pub logical: Vec<String>,
    pub native: String,
    pub columns: Vec<Column>,
    pub key: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Relationship {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    pub source: Vec<String>,
    pub target: Vec<String>,
    pub pairs: Vec<[String; 2]>,
    pub to_one: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaBinding {
    Qualified,
    SearchPath,
}
/// Explicit native deployment facts; never a grant or authentication credential.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeCatalogFence {
    pub version: String,
    pub schema: String,
    pub view: String,
    pub owner: String,
    pub owner_profile: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Binding {
    pub version: String,
    pub schema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_binding: Option<SchemaBinding>,
    pub tables: Vec<Table>,
    pub relationships: Vec<Relationship>,
    pub model_sha256: String,
    pub model_json: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_columns: Option<Json>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_catalog_fence: Option<NativeCatalogFence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_lifecycle: Option<crate::table_lifecycle::NativeLifecycle>,
}
pub(crate) fn validate_catalog_fence(binding: &Binding, target: &str) -> Result<()> {
    if let Some(fence) = &binding.native_catalog_fence {
        if !matches!(target, "postgresql18" | "lakebase18")
            || fence.version != "umf.pg-ts-fence/0.1"
            || fence.owner_profile != "non-login-maintainer/0.1"
        {
            return Err("unsupported native catalog fence profile");
        }
        for name in [&fence.schema, &fence.view, &fence.owner] {
            if name.is_empty() || name.len() > 63 || name.contains('\0') {
                return Err("native catalog fence identifier outside profile");
            }
        }
    }
    Ok(())
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterContainer {
    List,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    pub carrier: Carrier,
    pub nullable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<ParameterContainer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_nullable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u16>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Request {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_profile: Option<String>,
    pub target: String,
    pub sql: String,
    pub binding: Binding,
    pub parameters: Vec<Parameter>,
    pub row_cap: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshots: Option<BTreeMap<String, u64>>,
    #[serde(default)]
    pub keyset: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub stream_page_size: bool,
    #[serde(default, skip_serializing_if = "snapshot_false")]
    pub snapshot: bool,
}
fn snapshot_false(value: &bool) -> bool {
    !*value
}
pub(crate) fn validate_model(binding: &Binding) -> Result<()> {
    let model = prepare_model(binding)?;
    if model.lifecycle.is_some() {
        return Err("native table action profile is not yet admitted");
    }
    Ok(())
}

/// Resolve a logical Record through the same original binding admission used by reads.
/// This exposes source facts only and grants no native execution authority.
pub fn binding_record(binding: &Binding, logical_table: &[String]) -> Result<crate::model::Record> {
    prepare_model(binding)?
        .records
        .get(logical_table)
        .cloned()
        .ok_or("table outside admitted binding")
}

#[derive(Debug)]
pub(crate) struct AdmittedModel {
    pub(crate) document: Json,
    pub(crate) records: BTreeMap<Vec<String>, crate::model::Record>,
    pub(crate) lifecycle: Option<crate::table_lifecycle::AdmittedLifecycle>,
}

fn prepare_model(binding: &Binding) -> Result<AdmittedModel> {
    prepare_operation_model(binding, None)
}
fn prepare_temporal_model(binding: &Binding) -> Result<AdmittedModel> {
    prepare_native_model(binding, None, false, true)
}
pub(crate) fn prepare_operation_model(
    binding: &Binding,
    operation: Option<&str>,
) -> Result<AdmittedModel> {
    prepare_native_model(binding, operation, false, false)
}
pub(crate) fn prepare_seed_model(binding: &Binding) -> Result<AdmittedModel> {
    prepare_native_model(binding, None, true, false)
}
/// Admit the selected seed source for the distinct installer-owned migration
/// effect compiler. No native lifecycle is synthesized or returned.
pub(crate) fn prepare_administrative_seed_model(binding: &Binding) -> Result<AdmittedModel> {
    let doc = crate::json::checked_json(&binding.model_json).map_err(|_| "model JSON refused")?;
    prepare_native_document(binding, doc, None, true, true, NativeDocumentMode::Administrative)
}
fn prepare_native_model(
    binding: &Binding,
    operation: Option<&str>,
    seed: bool,
    temporal_read: bool,
) -> Result<AdmittedModel> {
    let doc = crate::json::checked_json(&binding.model_json).map_err(|_| "model JSON refused")?;
    prepare_native_document(binding, doc, operation, seed, temporal_read, NativeDocumentMode::Runtime)
}
pub(crate) fn prepare_operation_document(
    binding: &Binding,
    doc: Json,
    operation: Option<&str>,
) -> Result<AdmittedModel> {
    prepare_native_document(binding, doc, operation, false, false, NativeDocumentMode::Runtime)
}
/// Admit original operation meaning for a Weft-owned administrative compiler.
/// This keeps the normal operation source/field/type checks while deliberately
/// bypassing runtime lifecycle admission; the caller must establish a separate
/// administrative authority and must not serialize this as a runtime program.
pub(crate) fn prepare_administrative_operation_document(
    binding: &Binding,
    doc: Json,
    operation: &str,
) -> Result<AdmittedModel> {
    prepare_native_document(binding, doc, Some(operation), false, true, NativeDocumentMode::Administrative)
}
fn prepare_native_document(
    binding: &Binding,
    doc: Json,
    operation: Option<&str>,
    seed: bool,
    temporal_read: bool,
    mode: NativeDocumentMode,
) -> Result<AdmittedModel> {
    if binding.tables.iter().flat_map(|t| &t.columns).any(|c| {
        matches!(
            c.carrier,
            Carrier::Date | Carrier::Timestamp | Carrier::LocalTimestamp
        )
    }) && ((operation.is_none() && !seed && !temporal_read)
        || binding.native_lifecycle.is_none() && !temporal_read)
    {
        return Err("temporal carriers require declared native lifecycle operations");
    }
    if crate::json::sha256(binding.model_json.as_bytes()) != binding.model_sha256 {
        return Err("model source pin mismatch");
    }
    let modules = doc["modules"].as_array().ok_or("model modules")?;
    let version = doc["umf"].as_str().ok_or("model UMF version")?;
    let input = crate::model::ModuleInput {
        document_json: binding.model_json.clone(),
        pin: crate::ir::ModelPin {
            document_id: doc["id"].as_str().ok_or("model ID")?.into(),
            revision: binding.model_sha256.clone(),
            umf_version: version.into(),
            sha256: binding.model_sha256.clone(),
        },
        selected_module_ids: modules
            .iter()
            .map(|m| m["id"].as_str().unwrap_or("").to_owned())
            .collect(),
    };
    if version != "0.10.0" {
        return Err("raw-read supports UMF core 0.10.0 sources only");
    }
    let catalog = if operation.is_some() || seed {
        crate::model::Catalog::prepare_table_operation010(vec![input])
    } else {
        crate::model::Catalog::prepare_raw_read010(vec![input])
    }
    .map_err(|_| "UMF core envelope/profile refused")?;
    let n = |value: &str| crate::syntax::Name {
        value: value.into(),
        quoted: true,
        span: crate::ir::Span { start: 0, end: 0 },
    };
    let mut records = BTreeMap::new();
    let mut columns_by_table: BTreeMap<Vec<String>, BTreeMap<String, &Column>> = BTreeMap::new();
    for table in &binding.tables {
        let columns = columns_by_table.entry(table.logical.clone()).or_default();
        for column in &table.columns {
            columns.entry(column.logical.clone()).or_insert(column);
        }
    }
    let mut relationships_by_id: HashMap<String, Vec<&Json>> = HashMap::new();
    let mut relationships_by_module_id: HashMap<(String, String), Vec<&Json>> = HashMap::new();
    for module in modules {
        let module_id = module["id"].as_str().ok_or("model module ID")?;
        for relationship in module["relationships"].as_array().into_iter().flatten() {
            let id = relationship["id"].as_str().ok_or("model relationship ID")?;
            relationships_by_id
                .entry(id.to_owned())
                .or_default()
                .push(relationship);
            relationships_by_module_id
                .entry((module_id.to_owned(), id.to_owned()))
                .or_default()
                .push(relationship);
        }
    }
    for table in &binding.tables {
        if table.logical.len() != 2 {
            return Err("logical table path");
        }
        let record = catalog
            .record(Some(&n(&table.logical[0])), &n(&table.logical[1]))
            .map_err(|_| "table absent/ambiguous in UMF")?;
        let members = record.value["members"].as_array().ok_or("record members")?;
        let mut members_by_name: HashMap<String, Vec<(&Json, &Json)>> = HashMap::new();
        for member in members {
            let module_id = member["module"].as_str().ok_or("member module ID")?;
            let element_id = member["element"].as_str().ok_or("member element ID")?;
            let field = catalog
                .local_element_by_ids(record.document, module_id, element_id)
                .ok_or("missing member")?;
            if let Some(name) = catalog.query_name(field) {
                members_by_name
                    .entry(name.to_owned())
                    .or_default()
                    .push((field, member));
            }
        }
        let mut field_ids = BTreeMap::new();
        for c in &table.columns {
            let matches = members_by_name.get(&c.logical);
            if matches.is_none_or(|found| found.len() != 1) {
                return Err("column absent/ambiguous in UMF");
            };
            let (field, member) = matches.unwrap()[0];
            (if operation.is_some() || seed {
                crate::model::selected010_table_operation(&record.pin, field)
            } else {
                crate::model::selected010_raw_read(&record.pin, field)
            })
            .map_err(|_| "selected UMF 0.10 field has unsupported meaning")?;
            let family = match c.carrier {
                Carrier::Integer => "integer",
                Carrier::Decimal => "decimal",
                Carrier::String => "string",
                Carrier::Boolean => "boolean",
                Carrier::Uuid => "uuid",
                Carrier::Bits128 => "bits128",
                Carrier::Date => "date",
                Carrier::Timestamp => "timestamp",
                Carrier::LocalTimestamp => "timestamp",
            };
            if matches!(c.carrier, Carrier::Uuid | Carrier::Bits128)
                && (field.get("facets").is_some_and(|v| v != &json!({}))
                    || field.get("semanticTypes").is_some())
            {
                return Err("UMF 0.10 UUID/bits128 native refinements or semantic references are unsupported");
            }
            if field["kind"] != "field"
                || field["cardinality"] != "one"
                || field["scalarType"] != family
            {
                return Err("column meaning outside scalar read profile");
            }
            field_ids.insert(c.logical.clone(), member.clone());
        }
        let key: Vec<Json> = table
            .key
            .iter()
            .map(|k| field_ids.get(k).cloned().ok_or("key field absent"))
            .collect::<Result<_>>()?;
        let core_key_declared = record.value["keys"]
            .as_array()
            .is_some_and(|keys| keys.iter().any(|k| k["fields"] == json!(key)));
        let native_key_declared = if matches!(doc["vocabularies"]["umf.tables"]["version"].as_str(), Some("0.2.0" | "0.3.0")) {
            doc["modules"]
                .as_array()
                .into_iter()
                .flatten()
                .flat_map(|module| {
                    module["extensions"]["umf.tables"]["tables"]
                        .as_array()
                        .into_iter()
                        .flatten()
                })
                .any(|native_table| {
                    native_table["name"] == table.native
                        && native_table["record"]["module"] == record.identity.module
                        && native_table["record"]["element"] == record.identity.element
                        && native_table["nativePrimaryKey"]["fields"] == json!(key)
                        && native_table["nativePrimaryKey"]["comparison"] == "postgresql-date/0.1"
                })
        } else {
            false
        };
        if !core_key_declared && !native_key_declared && !matches!(mode, NativeDocumentMode::FiniteUnityCurrentKeyless | NativeDocumentMode::FiniteUnityCurrentPredicate | NativeDocumentMode::FiniteUnityCurrentSummary | NativeDocumentMode::FiniteUnityCurrentSum | NativeDocumentMode::FiniteUnityCurrentAvg) {
            return Err("binding key not declared in UMF");
        }
        if records.insert(table.logical.clone(), record).is_some() {
            return Err("ambiguous duplicate logical table binding");
        }
    }
    for rel in &binding.relationships {
        let source = records
            .get(&rel.source)
            .ok_or("relationship source binding")?;
        let target = records
            .get(&rel.target)
            .ok_or("relationship target binding")?;
        let declared = if let Some(module) = &rel.module {
            relationships_by_module_id
                .get(&(module.clone(), rel.id.clone()))
                .map(Vec::as_slice)
                .unwrap_or(&[])
        } else {
            relationships_by_id
                .get(&rel.id)
                .map(Vec::as_slice)
                .unwrap_or(&[])
        };
        if declared.len() != 1 {
            return Err("relationship identity absent/ambiguous in UMF");
        };
        let d = declared[0];
        if d.as_object().is_none_or(|m| {
            m.keys().any(|k| {
                !matches!(
                    k.as_str(),
                    "id" | "name"
                        | "title"
                        | "description"
                        | "source"
                        | "target"
                        | "sourceMultiplicity"
                        | "targetMultiplicity"
                        | "targetLifecycle"
                        | "directed"
                        | "inverse"
                        | "associationRecord"
                        | "extensions"
                )
            })
        }) || !matches!(d["targetLifecycle"].as_str(), Some("owned" | "independent"))
            || d["source"].as_array().is_none_or(|items| {
                items.iter().any(|r| {
                    r.as_object().is_none_or(|m| {
                        m.keys()
                            .any(|k| !matches!(k.as_str(), "module" | "element"))
                    })
                })
            })
            || d["target"].as_array().is_none_or(|items| {
                items.iter().any(|r| {
                    r.as_object().is_none_or(|m| {
                        m.keys()
                            .any(|k| !matches!(k.as_str(), "module" | "element" | "key"))
                    })
                })
            })
            || ["sourceMultiplicity", "targetMultiplicity"]
                .iter()
                .any(|key| {
                    d[*key]
                        .as_object()
                        .is_none_or(|m| m.keys().any(|k| !matches!(k.as_str(), "min" | "max")))
                })
        {
            return Err("selected original raw relationship contains unsupported meaning");
        }
        let target_key = target.value["keys"]
            .as_array()
            .and_then(|keys| keys.iter().find(|k| k["id"] == d["target"][0]["key"]))
            .ok_or("relationship target Key absent")?;
        let target_columns = columns_by_table
            .get(&rel.target)
            .ok_or("relationship target binding")?;
        let target_fields = target_key["fields"]
            .as_array()
            .ok_or("relationship target Key fields absent")?
            .iter()
            .map(|reference| {
                let module = reference["module"]
                    .as_str()
                    .ok_or("relationship target Field module absent")?;
                let element = reference["element"]
                    .as_str()
                    .ok_or("relationship target Field absent")?;
                let field = catalog
                    .local_element_by_ids(target.document, module, element)
                    .ok_or("relationship target Field absent")?;
                let logical = catalog
                    .query_name(field)
                    .ok_or("relationship target Field name absent")?;
                if !target_columns.contains_key(logical) {
                    return Err("relationship target Key field unbound");
                }
                Ok(logical.to_string())
            })
            .collect::<Result<Vec<_>>>()?;
        for pair in &rel.pairs {
            let source_column = columns_by_table
                .get(&rel.source)
                .and_then(|columns| columns.get(&pair[0]))
                .ok_or("relationship source Field unbound")?;
            let target_column = columns_by_table
                .get(&rel.target)
                .and_then(|columns| columns.get(&pair[1]))
                .ok_or("relationship target Field unbound")?;
            if source_column.carrier != target_column.carrier {
                return Err("relationship native logical carrier mismatch");
            }
        }
        if rel.pairs.iter().map(|p| p[1].clone()).collect::<Vec<_>>() != target_fields
            || d.get("extensions")
                .is_some_and(|e| e.as_object().is_none_or(|m| !m.is_empty()))
            || d["directed"] != true
            || d["targetMultiplicity"]["max"] != 1
            || !rel.to_one
            || d.get("associationRecord").is_some()
            || d["source"].as_array().is_none_or(|a| a.len() != 1)
            || d["target"].as_array().is_none_or(|a| a.len() != 1)
            || d["source"][0]["module"] != source.identity.module
            || d["source"][0]["element"] != source.identity.element
            || d["target"][0]["module"] != target.identity.module
            || d["target"][0]["element"] != target.identity.element
        {
            return Err("relationship endpoints/multiplicity differ from UMF");
        }
    }
    let lifecycle = if mode == NativeDocumentMode::Administrative {
        None
    } else if seed {
        crate::table_lifecycle::admit_seed(binding, &doc, &records)?
    } else {
        crate::table_lifecycle::admit(binding, &doc, &records, operation)?
    };
    Ok(AdmittedModel {
        document: doc,
        records,
        lifecycle,
    })
}

#[derive(Debug)]
pub struct AdmittedRead {
    request: Request,
    query: Query,
    model: AdmittedModel,
}
impl AdmittedRead {
    pub fn native_lifecycle(&self) -> Option<&crate::table_lifecycle::AdmittedLifecycle> {
        self.model.lifecycle.as_ref()
    }
    pub fn request(&self) -> &Request {
        &self.request
    }
    pub fn query(&self) -> &Query {
        &self.query
    }
    /// The exact parsed source whose bytes and core semantics passed admission.
    pub fn original_document(&self) -> &Json {
        &self.model.document
    }
    /// Resolve a logical binding path through the admitted Catalog, never by ID guessing.
    pub fn original_record(&self, logical_table: &[String]) -> Result<&crate::model::Record> {
        self.model
            .records
            .get(logical_table)
            .ok_or("table outside admitted binding")
    }
    /// Resolve even an unbound projection Field through original Record membership.
    pub fn original_field_ref(
        &self,
        logical_table: &[String],
        logical_column: &str,
    ) -> Result<Json> {
        let record = self.original_record(logical_table)?;
        let modules = self.model.document["modules"]
            .as_array()
            .ok_or("model modules")?;
        let members = record.value["members"].as_array().ok_or("record members")?;
        let mut found = None;
        for member in members {
            let module = modules
                .iter()
                .find(|m| m["id"] == member["module"])
                .ok_or("missing member module")?;
            let field = module["elements"]
                .as_array()
                .ok_or("module elements")?
                .iter()
                .find(|f| f["id"] == member["element"])
                .ok_or("missing member")?;
            let query_name = field["name"].as_str().or_else(|| {
                (self.model.document["umf"] == "0.10.0")
                    .then(|| field["id"].as_str())
                    .flatten()
            });
            if field["kind"] == "field" && query_name == Some(logical_column) {
                if found.is_some() {
                    return Err("column absent/ambiguous in UMF");
                }
                found = Some(member.clone());
            }
        }
        found.ok_or("column absent/ambiguous in UMF")
    }
}
pub fn admit_json(raw: &str) -> Result<AdmittedRead> {
    admit_json_mode(raw, FiniteCurrentAdmission::Ordinary)
}

/// Admit only the derived finite-current request owned by the Unity source-bound compiler.
pub fn admit_finite_current_keyless_json(raw: &str) -> Result<AdmittedRead> {
    admit_json_mode(raw, FiniteCurrentAdmission::Keyless)
}

/// Admit a derived finite-current request for the distinct source-bound predicate owner.
/// Its caller validates the closed predicate language before constructing this request.
pub fn admit_finite_current_predicate_json(raw: &str) -> Result<AdmittedRead> {
    admit_json_mode(raw, FiniteCurrentAdmission::Predicate)
}

/// Admit a derived request for the distinct bounded Unity current-summary owner.
/// Empty selected ColumnSets are admitted only in this owner mode.
pub fn admit_finite_current_summary_json(raw: &str) -> Result<AdmittedRead> {
    admit_json_mode(raw, FiniteCurrentAdmission::Summary)
}

/// Separate source-bound integer/decimal SUM admission. Only this mode keeps
/// original native ColumnSet custody and the bounded 39-field exact union.
pub fn admit_finite_current_sum_json(raw: &str) -> Result<AdmittedRead> {
    admit_json_mode(raw, FiniteCurrentAdmission::Sum)
}

pub fn admit_finite_current_avg_json(raw: &str) -> Result<AdmittedRead> {
    admit_json_mode(raw, FiniteCurrentAdmission::Avg)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FiniteCurrentAdmission { Ordinary, Keyless, Predicate, Summary, Sum, Avg }

fn admit_json_mode(raw: &str, admission: FiniteCurrentAdmission) -> Result<AdmittedRead> {
    let allow_keyless_current_read = admission != FiniteCurrentAdmission::Ordinary;
    let v = crate::json::checked_json(raw).map_err(|_| "invalid or duplicate JSON")?;
    if admission == FiniteCurrentAdmission::Keyless {
        if !exact_object_keys(
            &v,
            &["version", "target", "sql", "parameters", "rowCap", "keyset", "binding"],
            &[],
        ) || !exact_object_keys(
            &v["binding"],
            &["version", "schema", "tables", "relationships", "modelSha256", "modelJson"],
            &[],
        ) || v["binding"]["tables"].as_array().is_none_or(|tables| {
            tables.len() != 1
                || !exact_object_keys(tables.first().unwrap(), &["logical", "native", "columns", "key"], &[])
                || tables[0]["logical"].as_array().is_none_or(|path| path.len() != 2)
                || tables[0]["columns"].as_array().is_none_or(|columns| columns.is_empty() || columns.len() > 9)
                || tables[0]["key"] != json!([])
        }) || !v["binding"]["relationships"].as_array().is_some_and(|items| items.is_empty())
            || v["parameters"].as_array().is_none_or(|parameters| {
                parameters.len() != 1
                    || parameters[0]["name"] != "p1"
                    || parameters[0]["carrier"] != "string"
                    || parameters[0]["nullable"] != false
                    || parameters[0].get("container").is_some()
            })
        {
            return Err("keyless admission requires the exact derived finite-current request shape");
        }
    } else if matches!(admission, FiniteCurrentAdmission::Predicate | FiniteCurrentAdmission::Summary | FiniteCurrentAdmission::Sum | FiniteCurrentAdmission::Avg) {
        if !exact_object_keys(
            &v,
            &["version", "target", "sql", "parameters", "rowCap", "keyset", "binding"],
            &[],
        ) || !exact_object_keys(
            &v["binding"],
            &["version", "schema", "tables", "relationships", "modelSha256", "modelJson"],
            if matches!(admission, FiniteCurrentAdmission::Sum | FiniteCurrentAdmission::Avg) { &["nativeColumns"] } else { &[] },
        ) || v["binding"]["tables"].as_array().is_none_or(|tables| {
            tables.len() != 1
                || !exact_object_keys(tables.first().unwrap(), &["logical", "native", "columns", "key"], &[])
                || tables[0]["logical"].as_array().is_none_or(|path| path.len() != 2)
                || tables[0]["columns"].as_array().is_none_or(|columns| {
                    (admission != FiniteCurrentAdmission::Summary && columns.is_empty())
                        || columns.len() > if matches!(admission, FiniteCurrentAdmission::Sum | FiniteCurrentAdmission::Avg) { 39 } else { 9 }
                })
                || tables[0]["key"] != json!([])
        }) || !v["binding"]["relationships"].as_array().is_some_and(|items| items.is_empty())
            || v["parameters"].as_array().is_none_or(|parameters| {
                parameters.len() > 8 || parameters.iter().any(|parameter| {
                    !exact_object_keys(parameter, &["name", "carrier", "nullable"], &[])
                        || !matches!(parameter["carrier"].as_str(), Some("string" | "boolean"))
                        || parameter["name"].as_str().is_none_or(|name| name.is_empty() || name.len() > 64 || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'))
                        || parameter["nullable"].as_bool().is_none()
                })
            })
        {
            return Err("predicate admission requires the exact derived finite-current request shape");
        }
    }
    let request: Request = serde_json::from_value(v).map_err(|_| "invalid read request")?;
    if allow_keyless_current_read
        && (request.version != "weft.raw-read/0.1"
            || request.target != "unity-catalog-current"
            || request.keyset
            || request.snapshot
            || request.snapshots.is_some()
            || request.stream_page_size
            || request.verification_profile.is_some()
            || (request.binding.native_columns.is_some() && !matches!(admission, FiniteCurrentAdmission::Sum | FiniteCurrentAdmission::Avg))
            || request.binding.native_catalog_fence.is_some()
            || request.binding.native_lifecycle.is_some()
            || request.binding.tables.len() != 1
            || !request.binding.tables[0].key.is_empty())
    {
        return Err("keyless admission is reserved for finite Unity Catalog current reads");
    }
    if matches!(admission, FiniteCurrentAdmission::Sum | FiniteCurrentAdmission::Avg) {
        let envelope = request.binding.native_columns.as_ref().ok_or("finite numeric aggregate requires original native ColumnSet custody")?;
        if !exact_object_keys(envelope, &["target", "sets"], &[])
            || envelope["target"].as_object().is_none_or(|target| {
                target.keys().any(|k| !matches!(k.as_str(), "platform" | "majorVersion"))
                    || target.get("platform").and_then(Json::as_str) != Some("unity-catalog")
                    || target.get("majorVersion").and_then(Json::as_str).is_none_or(|v| v.is_empty())
            })
            || envelope["sets"].as_array().is_none_or(|sets| sets.len() != 1)
        {
            return Err("finite SUM native ColumnSet custody envelope differs");
        }
    }
    if matches!(admission, FiniteCurrentAdmission::Predicate | FiniteCurrentAdmission::Summary)
        && request.binding.tables[0].columns.iter().any(|column| {
            !matches!(column.carrier, Carrier::String | Carrier::Boolean)
        })
    {
        return Err("current predicate admission is limited to STRING/BOOLEAN columns");
    }
    if matches!(admission, FiniteCurrentAdmission::Sum | FiniteCurrentAdmission::Avg)
        && request.binding.tables[0].columns.iter().any(|column| {
            !matches!(column.carrier, Carrier::String | Carrier::Boolean | Carrier::Integer | Carrier::Decimal)
        })
    {
        return Err("finite SUM selected fields exceed its exact scalar profile");
    }
    if request
        .binding
        .tables
        .iter()
        .flat_map(|t| &t.columns)
        .any(|c| matches!(c.carrier, Carrier::Uuid | Carrier::Bits128))
        && !matches!(request.target.as_str(), "postgresql18" | "lakebase18")
    {
        return Err("UMF 0.10 UUID/bits128 raw carriers are currently PostgreSQL-only");
    }
    validate_catalog_fence(&request.binding, &request.target)?;
    if let Some(profile) = &request.verification_profile {
        if !matches!(profile.as_str(), "source-exact/0.1" | "table-lifecycle/0.1")
            || !matches!(request.target.as_str(), "postgresql18" | "lakebase18")
        {
            return Err("unsupported source verification profile");
        }
        if profile == "source-exact/0.1" && request.binding.native_columns.is_none() {
            return Err("source-exact verification requires original native ColumnSet facts");
        }
    }
    if request.snapshot
        && (!request.keyset || !matches!(request.target.as_str(), "spark-delta4" | "unity-catalog"))
    {
        return Err("snapshot pages require an explicit Delta snapshot target");
    }
    let temporal = request.version == "weft.raw-read/0.2"
        && request.temporal_profile.as_deref() == Some("umf.weft-binding-temporal/0.1");
    if (request.version != "weft.raw-read/0.1" && !temporal)
        || (request.version == "weft.raw-read/0.1" && request.temporal_profile.is_some())
        || request.sql.len() > 65536
        || request.parameters.len() > 64
        || request.binding.tables.len() > 256
    {
        return Err("unsupported raw-read version or input budget");
    }
    for parameter in &request.parameters {
        if temporal && parameter.container.is_some() && parameter.carrier == Carrier::Timestamp {
            return Err("instant list parameters are outside the exact temporal profile");
        }
        if parameter.container.is_some() {
            if !matches!(
                request.target.as_str(),
                "postgresql18" | "lakebase18" | "spark-delta4"
            ) || parameter.nullable
                || parameter.element_nullable.is_none()
                || !parameter.max_items.is_some_and(|n| n > 0 && n <= 1000)
            {
                return Err(
                    "list parameter requires an exact bounded nonnullable membership profile",
                );
            }
        } else if parameter.element_nullable.is_some() || parameter.max_items.is_some() {
            return Err("scalar parameter cannot carry list facets");
        }
    }
    if request
        .binding
        .tables
        .iter()
        .flat_map(|t| &t.columns)
        .any(|c| {
            matches!(
                c.carrier,
                Carrier::Date | Carrier::Timestamp | Carrier::LocalTimestamp
            )
        })
        && !temporal
    {
        return Err("temporal carriers require the explicit raw-read/0.2 temporal profile");
    }
    if temporal && request.binding.native_columns.is_none() {
        return Err("temporal raw reads require the original native ColumnSet");
    }
    let model = if allow_keyless_current_read {
        let doc = crate::json::checked_json(&request.binding.model_json)
            .map_err(|_| "model JSON refused")?;
        let model = prepare_native_document(
            &request.binding, doc, None, false, false, match admission { FiniteCurrentAdmission::Keyless => NativeDocumentMode::FiniteUnityCurrentKeyless, FiniteCurrentAdmission::Predicate => NativeDocumentMode::FiniteUnityCurrentPredicate, FiniteCurrentAdmission::Summary => NativeDocumentMode::FiniteUnityCurrentSummary, FiniteCurrentAdmission::Sum => NativeDocumentMode::FiniteUnityCurrentSum, FiniteCurrentAdmission::Avg => NativeDocumentMode::FiniteUnityCurrentAvg, FiniteCurrentAdmission::Ordinary => unreachable!() },
        )?;
        if model.lifecycle.is_some() {
            return Err("keyless current reads do not admit a native lifecycle");
        }
        model
    } else if temporal {
        prepare_temporal_model(&request.binding)?
    } else {
        prepare_model(&request.binding)?
    };
    if temporal {
        validate_temporal_selectors(&request.binding, &model.document)?;
    }
    if model.lifecycle.is_some()
        != (request.verification_profile.as_deref() == Some("table-lifecycle/0.1"))
    {
        return Err("native table lifecycle requires its explicit verification profile");
    }
    let mut statements =
        Parser::parse_sql(&PostgreSqlDialect {}, &request.sql).map_err(|_| "SQL parse refused")?;
    if statements.len() != 1 {
        return Err("exactly one SELECT required");
    }
    let Statement::Query(query) = statements.remove(0) else {
        return Err("exactly one SELECT required");
    };
    Ok(AdmittedRead {
        request,
        query: *query,
        model,
    })
}

fn validate_temporal_selectors(binding: &Binding, doc: &Json) -> Result<()> {
    // A temporal value may participate in ordering, but the temporal profile
    // does not admit it as a row identity or Key tie-breaker.
    for table in &binding.tables {
        validate_authored_key_match(table, doc)?;
        if table.key.iter().any(|key| {
            table.columns.iter().any(|column| {
                column.logical == *key
                    && matches!(
                        column.carrier,
                        Carrier::Date | Carrier::Timestamp | Carrier::LocalTimestamp
                    )
            })
        }) {
            return Err("temporal Fields cannot be selected as Keys");
        }
    }
    let modules = doc["modules"]
        .as_array()
        .ok_or("temporal source modules absent")?;
    for table in &binding.tables {
        for column in &table.columns {
            let member = match column.carrier {
                Carrier::Date | Carrier::Timestamp | Carrier::LocalTimestamp => Some(column),
                _ => None,
            };
            let Some(column) = member else { continue };
            let module = table
                .logical
                .first()
                .map(String::as_str)
                .ok_or("temporal source module absent")?;
            let record_name = table
                .logical
                .get(1)
                .map(String::as_str)
                .ok_or("temporal source Record absent")?;
            let module_matches = modules
                .iter()
                .filter(|m| m["id"] == module)
                .collect::<Vec<_>>();
            if module_matches.len() != 1 {
                return Err("temporal source module is absent or ambiguous");
            }
            let module_doc = module_matches[0];
            let records = module_doc["elements"]
                .as_array()
                .ok_or("temporal source elements absent")?
                .iter()
                .filter(|e| e["kind"] == "record" && e["name"] == record_name)
                .collect::<Vec<_>>();
            if records.len() != 1 {
                return Err("temporal source Record is absent or ambiguous");
            }
            let record = records[0];
            let id = record["id"]
                .as_str()
                .ok_or("temporal source Record identity absent")?;
            let fields = module_doc["elements"]
                .as_array()
                .ok_or("temporal source elements absent")?;
            let members = record["members"]
                .as_array()
                .ok_or("temporal source Record members absent")?;
            let mut matching_fields = vec![];
            for member in members.iter().filter(|m| m["module"] == module) {
                let member_id = member["element"]
                    .as_str()
                    .ok_or("temporal source member identity absent")?;
                let owners = fields
                    .iter()
                    .filter(|f| f["kind"] == "field" && f["id"] == member_id)
                    .collect::<Vec<_>>();
                if owners.len() != 1 {
                    return Err("temporal source member Field is absent or ambiguous");
                }
                if owners[0]["name"] == column.logical {
                    matching_fields.push(owners[0]);
                }
            }
            if matching_fields.len() != 1 {
                return Err("temporal source Field name is absent or ambiguous");
            }
            let field = matching_fields[0];
            let field_id = field["id"]
                .as_str()
                .ok_or("temporal source Field identity absent")?;
            let scalar = field["scalarType"]
                .as_str()
                .ok_or("temporal source Field scalar absent")?;
            let meaning = authored_temporal_meaning(doc, module, id, field_id, scalar)?;
            let valid = match column.carrier {
                Carrier::Date => field["scalarType"] == "date" && meaning.is_none(),
                Carrier::Timestamp => {
                    field["scalarType"] == "timestamp" && meaning == Some("instant")
                }
                Carrier::LocalTimestamp => {
                    field["scalarType"] == "timestamp" && meaning == Some("local")
                }
                _ => true,
            };
            if !valid {
                return Err("temporal carrier differs from authored access meaning");
            }
        }
    }
    Ok(())
}

fn validate_authored_key_match(table: &Table, doc: &Json) -> Result<()> {
    let module = table
        .logical
        .first()
        .map(String::as_str)
        .ok_or("temporal source module absent")?;
    let record_name = table
        .logical
        .get(1)
        .map(String::as_str)
        .ok_or("temporal source Record absent")?;
    let modules = doc["modules"]
        .as_array()
        .ok_or("temporal source modules absent")?;
    let module_matches = modules
        .iter()
        .filter(|m| m["id"] == module)
        .collect::<Vec<_>>();
    if module_matches.len() != 1 {
        return Err("temporal source module is absent or ambiguous");
    }
    let module_doc = module_matches[0];
    let records = module_doc["elements"]
        .as_array()
        .ok_or("temporal source elements absent")?
        .iter()
        .filter(|e| e["kind"] == "record" && e["name"] == record_name)
        .collect::<Vec<_>>();
    if records.len() != 1 {
        return Err("temporal source Record is absent or ambiguous");
    }
    let record = records[0];
    let record_id = record["id"]
        .as_str()
        .ok_or("temporal source Record identity absent")?;
    let selected_fields = authored_selected_key_fields(doc, module, record_id)?;
    let members = record["members"]
        .as_array()
        .ok_or("temporal source Record members absent")?;
    let elements = module_doc["elements"]
        .as_array()
        .ok_or("temporal source elements absent")?;
    let mut bound_fields = Vec::with_capacity(table.key.len());
    for logical in &table.key {
        let columns = table
            .columns
            .iter()
            .filter(|c| c.logical == *logical)
            .collect::<Vec<_>>();
        if columns.len() != 1 {
            return Err("bound Key column absent or ambiguous");
        }
        let mut found = vec![];
        for member in members.iter().filter(|m| m["module"] == module) {
            let field_id = member["element"]
                .as_str()
                .ok_or("temporal source member identity absent")?;
            let owners = elements
                .iter()
                .filter(|e| e["kind"] == "field" && e["id"] == field_id)
                .collect::<Vec<_>>();
            if owners.len() != 1 {
                return Err("temporal source member Field is absent or ambiguous");
            }
            if owners[0]["name"] == *logical {
                found.push(member.clone());
            }
        }
        if found.len() != 1 {
            return Err("bound Key source Field absent or ambiguous");
        }
        bound_fields.push(found.remove(0));
    }
    if bound_fields != selected_fields {
        return Err("bound Key differs from authored selectedKey");
    }
    Ok(())
}

fn authored_selected_key_fields(doc: &Json, module: &str, record_id: &str) -> Result<Vec<Json>> {
    if !exact_object_keys(&doc["vocabularies"]["umf.access"], &["version"], &[])
        || doc["vocabularies"]["umf.access"]["version"] != "0.1.0"
    {
        return Err("selected access vocabulary version unsupported");
    }
    let modules = doc["modules"]
        .as_array()
        .ok_or("temporal source modules absent")?;
    let module_matches = modules
        .iter()
        .filter(|m| m["id"] == module)
        .collect::<Vec<_>>();
    if module_matches.len() != 1 {
        return Err("temporal source module is absent or ambiguous");
    }
    let module_doc = module_matches[0];
    let views = module_doc["extensions"]["umf.access"]["views"]
        .as_array()
        .ok_or("temporal source access views absent")?;
    let matching_views = views
        .iter()
        .filter(|v| v["record"]["module"] == module && v["record"]["element"] == record_id)
        .collect::<Vec<_>>();
    if matching_views.len() != 1 {
        return Err("temporal source access view is absent or ambiguous");
    }
    let view = matching_views[0];
    if !exact_object_keys(view, &["fields", "record", "selectedKey"], &[])
        || !exact_reference(&view["record"])
        || !exact_object_keys(&view["selectedKey"], &["element", "key", "module"], &[])
        || view["selectedKey"]["module"] != module
        || view["selectedKey"]["element"] != record_id
        || view["selectedKey"]["key"]
            .as_str()
            .is_none_or(str::is_empty)
    {
        return Err("selected temporal RecordView shape unsupported");
    }
    let record = module_doc["elements"]
        .as_array()
        .and_then(|elements| {
            elements
                .iter()
                .find(|e| e["id"] == record_id && e["kind"] == "record")
        })
        .ok_or("selected temporal Record absent")?;
    let keys = record["keys"]
        .as_array()
        .ok_or("selected temporal Record Keys absent")?;
    let selected_keys = keys
        .iter()
        .filter(|k| k["id"] == view["selectedKey"]["key"])
        .collect::<Vec<_>>();
    if selected_keys.len() != 1
        || selected_keys[0]["fields"].as_array().is_none_or(|fields| {
            fields.is_empty()
                || fields.iter().any(|f| {
                    !exact_reference(f)
                        || !record["members"]
                            .as_array()
                            .is_some_and(|members| members.contains(f))
                })
        })
    {
        return Err("selected temporal RecordView Key reference unsupported");
    }
    Ok(selected_keys[0]["fields"].as_array().unwrap().clone())
}

/// Resolve a selected temporal meaning only from a closed authored access view.
/// Missing `temporal` is distinct from a present malformed value; unknown
/// selected FieldView members refuse, while other FieldViews remain untouched.
pub fn authored_temporal_meaning(
    doc: &Json,
    module: &str,
    record_id: &str,
    field_id: &str,
    scalar: &str,
) -> std::result::Result<Option<&'static str>, &'static str> {
    if !matches!(scalar, "date" | "timestamp") {
        return Ok(None);
    }
    if !exact_object_keys(&doc["vocabularies"]["umf.access"], &["version"], &[])
        || doc["vocabularies"]["umf.access"]["version"] != "0.1.0"
    {
        return Err("selected access vocabulary version unsupported");
    }
    let modules = doc["modules"]
        .as_array()
        .ok_or("temporal source modules absent")?;
    let module_matches = modules
        .iter()
        .filter(|m| m["id"] == module)
        .collect::<Vec<_>>();
    if module_matches.len() != 1 {
        return Err("temporal source module is absent or ambiguous");
    }
    let module_doc = module_matches[0];
    let access = &module_doc["extensions"]["umf.access"];
    if !access.is_object() {
        return Err("temporal source access extension absent");
    }
    let views = access["views"]
        .as_array()
        .ok_or("temporal source access views absent")?;
    let matching_views = views
        .iter()
        .filter(|v| v["record"]["module"] == module && v["record"]["element"] == record_id)
        .collect::<Vec<_>>();
    if matching_views.len() != 1 {
        return Err("temporal source access view is absent or ambiguous");
    }
    let view = matching_views[0];
    if !exact_object_keys(view, &["fields", "record", "selectedKey"], &[])
        || !exact_reference(&view["record"])
        || !exact_object_keys(&view["selectedKey"], &["element", "key", "module"], &[])
        || view["selectedKey"]["module"] != module
        || view["selectedKey"]["element"] != record_id
        || view["selectedKey"]["key"]
            .as_str()
            .is_none_or(str::is_empty)
    {
        return Err("selected temporal RecordView shape unsupported");
    }
    authored_selected_key_fields(doc, module, record_id)?;
    let selected = view["fields"]
        .as_array()
        .ok_or("temporal source access Fields absent")?
        .iter()
        .filter(|f| f["field"]["module"] == module && f["field"]["element"] == field_id)
        .collect::<Vec<_>>();
    if selected.len() != 1 {
        return Err("temporal source access Field is absent or ambiguous");
    }
    let field = selected[0];
    if !exact_object_keys(
        field,
        &["create", "field", "generated", "mutable", "row"],
        &["temporal"],
    ) || !exact_reference(&field["field"])
        || !exact_presence(&field["create"])
        || !exact_presence(&field["row"])
        || !field["generated"].is_boolean()
        || !field["mutable"].is_boolean()
    {
        return Err("selected temporal FieldView shape unsupported");
    }
    match (field.get("temporal"), scalar) {
        (None, "date") => Ok(None),
        (Some(value), "timestamp") if value.as_str() == Some("instant") => Ok(Some("instant")),
        (Some(value), "timestamp") if value.as_str() == Some("local") => Ok(Some("local")),
        _ => Err("unknown, malformed, or mismatched temporal access selector"),
    }
}
fn exact_reference(value: &Json) -> bool {
    value.as_object().is_some_and(|m| {
        m.len() == 2
            && value["module"].as_str().is_some_and(|s| !s.is_empty())
            && value["element"].as_str().is_some_and(|s| !s.is_empty())
    })
}
fn exact_presence(value: &Json) -> bool {
    value.as_object().is_some_and(|m| {
        m.len() == 2 && value["missing"].is_boolean() && value["null"].is_boolean()
    })
}
fn exact_object_keys(value: &Json, required: &[&str], optional: &[&str]) -> bool {
    value.as_object().is_some_and(|m| {
        required.iter().all(|k| m.contains_key(*k))
            && m.keys()
                .all(|k| required.contains(&k.as_str()) || optional.contains(&k.as_str()))
    })
}

#[cfg(test)]
mod source_identity_tests {
    use super::*;

    #[test]
    fn temporal_columns_require_matching_authored_access_meaning() {
        let field_view = |id: &str, temporal: Option<&str>| {
            let mut v = json!({"field":{"module":"m","element":id},"create":{"missing":false,"null":false},"generated":false,"mutable":false,"row":{"missing":false,"null":false}});
            if let Some(t) = temporal {
                v["temporal"] = json!(t)
            }
            v
        };
        let doc = json!({"vocabularies":{"umf.access":{"version":"0.1.0"}},"modules":[{"id":"m","elements":[
            {"id":"R","name":"R","kind":"record","members":[{"module":"m","element":"id"},{"module":"m","element":"day"},{"module":"m","element":"instant"},{"module":"m","element":"local"}],"keys":[{"id":"k","fields":[{"module":"m","element":"id"}]}]},
            {"id":"id","name":"id","kind":"field","scalarType":"integer"},
            {"id":"day","name":"day","kind":"field","scalarType":"date"},
            {"id":"instant","name":"instant","kind":"field","scalarType":"timestamp"},
            {"id":"local","name":"local","kind":"field","scalarType":"timestamp"}],
            "extensions":{"umf.access":{"views":[{"record":{"module":"m","element":"R"},"selectedKey":{"module":"m","element":"R","key":"k"},"fields":[
                field_view("id",None),field_view("day",None),field_view("instant",Some("instant")),field_view("local",Some("local"))]}]}}}]});
        let table = Table {
            logical: vec!["m".into(), "R".into()],
            native: "r".into(),
            key: vec!["id".into()],
            columns: vec![
                Column {
                    logical: "id".into(),
                    native: "id".into(),
                    carrier: Carrier::Integer,
                    nullable: false,
                },
                Column {
                    logical: "day".into(),
                    native: "day".into(),
                    carrier: Carrier::Date,
                    nullable: true,
                },
                Column {
                    logical: "instant".into(),
                    native: "instant".into(),
                    carrier: Carrier::Timestamp,
                    nullable: true,
                },
                Column {
                    logical: "local".into(),
                    native: "local".into(),
                    carrier: Carrier::LocalTimestamp,
                    nullable: true,
                },
            ],
        };
        let binding = Binding {
            version: "umf.raw-read-binding/0.1".into(),
            schema: None,
            schema_binding: None,
            tables: vec![table],
            relationships: vec![],
            model_sha256: String::new(),
            model_json: String::new(),
            native_columns: None,
            native_catalog_fence: None,
            native_lifecycle: None,
        };
        validate_temporal_selectors(&binding, &doc).expect("all explicit source meanings match");
        for (index, wrong) in [
            (1, Carrier::Timestamp),
            (2, Carrier::LocalTimestamp),
            (3, Carrier::Timestamp),
        ] {
            let mut mismatch = binding.clone();
            mismatch.tables[0].columns[index].carrier = wrong;
            assert!(
                validate_temporal_selectors(&mismatch, &doc).is_err(),
                "column {index} meaning mismatch"
            );
        }
        for key in ["day", "instant", "local"] {
            let mut temporal_key = binding.clone();
            temporal_key.tables[0].key = vec![key.to_string()];
            assert!(
                validate_temporal_selectors(&temporal_key, &doc).is_err(),
                "temporal Key {key} must refuse"
            );
        }
        let mut relationship_target_key = binding.clone();
        relationship_target_key.tables.push(Table {
            logical: vec!["m".into(), "Target".into()],
            native: "target".into(),
            key: vec!["day".into()],
            columns: vec![Column {
                logical: "day".into(),
                native: "day".into(),
                carrier: Carrier::Date,
                nullable: false,
            }],
        });
        relationship_target_key.relationships.push(Relationship {
            id: "target".into(),
            module: Some("m".into()),
            source: vec!["m".into(), "R".into()],
            target: vec!["m".into(), "Target".into()],
            pairs: vec![["instant".into(), "day".into()]],
            to_one: true,
        });
        assert!(
            validate_temporal_selectors(&relationship_target_key, &doc).is_err(),
            "relationship target temporal Keys must refuse"
        );
        let mut altered = doc.clone();
        altered["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"][2]
            .as_object_mut()
            .unwrap()
            .remove("temporal");
        assert!(validate_temporal_selectors(&binding, &altered).is_err());
        let mut malformed_date = doc.clone();
        malformed_date["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"][1]
            ["temporal"] = json!(7);
        assert!(
            validate_temporal_selectors(&binding, &malformed_date).is_err(),
            "present malformed date selector must not mean absent"
        );
        let mut future_date = doc.clone();
        future_date["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"][1]
            ["futureTemporalRule"] = json!({"must":"not ignore"});
        assert!(
            validate_temporal_selectors(&binding, &future_date).is_err(),
            "unknown selected FieldView refinement must refuse"
        );
        let mut malformed_row = doc.clone();
        malformed_row["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"][1]["row"]
            ["null"] = json!("maybe");
        assert!(
            validate_temporal_selectors(&binding, &malformed_row).is_err(),
            "malformed selected FieldView presence must refuse"
        );
        let mut wrong_access_version = doc.clone();
        wrong_access_version["vocabularies"]["umf.access"]["version"] = json!("0.2.0");
        assert!(
            validate_temporal_selectors(&binding, &wrong_access_version).is_err(),
            "unknown selected access vocabulary version must refuse"
        );
        let mut duplicate_view = doc.clone();
        let view = duplicate_view["modules"][0]["extensions"]["umf.access"]["views"][0].clone();
        duplicate_view["modules"][0]["extensions"]["umf.access"]["views"]
            .as_array_mut()
            .unwrap()
            .push(view);
        assert!(validate_temporal_selectors(&binding, &duplicate_view).is_err());
        let mut same_name_elsewhere = doc.clone();
        same_name_elsewhere["modules"][0]["elements"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":"other","name":"instant","kind":"field","scalarType":"timestamp"}));
        assert!(validate_temporal_selectors(&binding, &same_name_elsewhere).is_ok());
        let mut duplicate_member = doc.clone();
        duplicate_member["modules"][0]["elements"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["id"] == "R")
            .unwrap()["members"]
            .as_array_mut()
            .unwrap()
            .push(json!({"module":"m","element":"instant"}));
        assert!(validate_temporal_selectors(&binding, &duplicate_member).is_err());
        let mut duplicate_selection = doc.clone();
        let selected = duplicate_selection["modules"][0]["extensions"]["umf.access"]["views"][0]
            ["fields"][1]
            .clone();
        duplicate_selection["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"]
            .as_array_mut()
            .unwrap()
            .push(selected);
        assert!(validate_temporal_selectors(&binding, &duplicate_selection).is_err());
    }

    #[test]
    fn raw_read_refuses_legacy_owning_core_versions() {
        let current =
            include_str!("../../../tests/raw-read/postgresql-source-exact-010-request.json");
        for version in ["0.7.0", "0.8.0"] {
            let mut request: Json = serde_json::from_str(current).unwrap();
            let mut doc: Json =
                serde_json::from_str(request["binding"]["modelJson"].as_str().unwrap()).unwrap();
            doc["umf"] = json!(version);
            let source = serde_json::to_string(&doc).unwrap();
            request["binding"]["modelSha256"] = json!(crate::json::sha256(source.as_bytes()));
            request["binding"]["modelJson"] = json!(source);
            let error = admit_json(&request.to_string()).unwrap_err();
            assert!(
                error.contains("UMF core 0.10.0 sources only"),
                "version {version}: {error}"
            );
        }
    }

    #[test]
    fn original_umf010_uuid_and_bits128_source_is_retained_and_pinned() {
        let mut request: Json =
            serde_json::from_str(include_str!("../../../tests/raw-read/source-identity.json"))
                .unwrap();
        let mut doc: Json =
            serde_json::from_str(request["binding"]["modelJson"].as_str().unwrap()).unwrap();
        doc["umf"] = json!("0.10.0");
        let elements = doc["modules"][0]["elements"].as_array_mut().unwrap();
        for field in elements {
            match field["id"].as_str() {
                Some("Order_id") => field["scalarType"] = json!("uuid"),
                Some("Order_status") => field["scalarType"] = json!("bits128"),
                _ => {}
            }
        }
        let source = serde_json::to_string(&doc).unwrap();
        let digest = crate::json::sha256(source.as_bytes());
        request["binding"]["modelSha256"] = json!(digest);
        request["binding"]["modelJson"] = json!(source);
        request["binding"]["tables"]
            .as_array_mut()
            .unwrap()
            .truncate(1);
        request["binding"]["tables"][0]["columns"][0]["carrier"] = json!("uuid");
        request["binding"]["tables"][0]["columns"][2]["carrier"] = json!("bits128");
        request["binding"]["relationships"] = json!([]);
        request["target"] = json!("postgresql18");
        request["sql"] = json!("SELECT o.id AS id, o.status AS status FROM commerce.Order AS o WHERE o.id = :id AND o.status = :opaque");
        request["parameters"] = json!([
            {"name":"id","carrier":"uuid","nullable":false},
            {"name":"opaque","carrier":"bits128","nullable":false}
        ]);
        request["keyset"] = json!(false);
        request.as_object_mut().unwrap().remove("snapshots");

        let admitted =
            admit_json(&request.to_string()).expect("UMF 0.10 exact UUID/128 source admitted");
        assert_eq!(admitted.request().binding.model_sha256, digest);
        assert_eq!(
            crate::json::sha256(admitted.request().binding.model_json.as_bytes()),
            digest
        );
        assert_eq!(admitted.original_document(), &doc);
        let table = vec!["commerce".into(), "Order".into()];
        let record = admitted.original_record(&table).unwrap();
        assert_eq!(record.pin.umf_version, "0.10.0");
        assert_eq!(
            admitted.original_field_ref(&table, "id").unwrap(),
            json!({"module":"commerce","element":"Order_id"})
        );
        assert_eq!(
            admitted.original_field_ref(&table, "status").unwrap(),
            json!({"module":"commerce","element":"Order_status"})
        );
    }

    #[test]
    fn resolved_source_identity_is_not_logical_name_guessing() {
        let mut request: Json =
            serde_json::from_str(include_str!("../../../tests/raw-read/source-identity.json"))
                .unwrap();
        let mut doc: Json =
            serde_json::from_str(request["binding"]["modelJson"].as_str().unwrap()).unwrap();
        fn remap(value: &mut Json) {
            match value {
                Json::String(s) => {
                    *s = match s.as_str() {
                        "commerce" => "module-identity".into(),
                        "Order" => "record-order-identity".into(),
                        "Customer" => "record-customer-identity".into(),
                        _ => s.clone(),
                    };
                }
                Json::Array(a) => a.iter_mut().for_each(remap),
                Json::Object(o) => o.values_mut().for_each(remap),
                _ => {}
            }
        }
        remap(&mut doc);
        doc["modules"][0]["namespace"] = json!("commerce");
        for element in doc["modules"][0]["elements"].as_array_mut().unwrap() {
            if element["id"] == "record-order-identity" {
                element["name"] = json!("Order");
            }
            if element["id"] == "record-customer-identity" {
                element["name"] = json!("Customer");
            }
        }
        // Status is a real original member omitted from the flat scalar binding.
        request["binding"]["tables"][0]["columns"]
            .as_array_mut()
            .unwrap()
            .retain(|c| c["logical"] != "status");
        let source = serde_json::to_string(&doc).unwrap();
        request["binding"]["modelSha256"] = json!(crate::json::sha256(source.as_bytes()));
        request["binding"]["modelJson"] = json!(source);
        let admitted = admit_json(&request.to_string()).unwrap();
        let table = vec!["commerce".into(), "Order".into()];
        let record = admitted.original_record(&table).unwrap();
        assert_eq!(record.identity.module, "module-identity");
        assert_eq!(record.identity.element, "record-order-identity");
        assert_eq!(admitted.original_document(), &doc);
        assert_eq!(
            admitted.original_field_ref(&table, "status").unwrap(),
            json!({"module":"module-identity", "element":"Order_status"})
        );
        assert!(admitted
            .original_record(&["module-identity".into(), "record-order-identity".into()])
            .is_err());
        assert!(admitted.original_field_ref(&table, "missing").is_err());
        assert!(admitted
            .original_field_ref(&table, "Customer_name")
            .is_err());
        // Callers receive owned references, not mutation access to the retained source.
        let mut reference = admitted.original_field_ref(&table, "status").unwrap();
        reference["element"] = json!("tampered");
        assert_eq!(
            admitted.original_field_ref(&table, "status").unwrap()["element"],
            "Order_status"
        );
    }
}
