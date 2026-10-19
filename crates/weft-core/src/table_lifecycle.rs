// SPDX-License-Identifier: MIT OR Apache-2.0
//! Conditional native table custody. UMF owns extension meaning; hosts prove
//! original source deployment, policies, privileges, principal and generation.
use crate::raw_read::{Binding, Carrier};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, &'static str>;

#[jsonschema::validator(path = "../../spec/upstream/umf-tables-0.1.0.schema.json")]
struct TablePayload;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeLifecycle {
    pub version: String,
    pub module: String,
    pub configuration_digest: String,
    pub deployment_digest: String,
}

#[derive(Debug)]
pub struct AdmittedLifecycle {
    context: NativeLifecycle,
    source_digest: String,
    tables: BTreeMap<Vec<String>, Value>,
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn query_name(value: &Value) -> Option<&str> {
    value["name"].as_str().or_else(|| value["id"].as_str())
}
impl AdmittedLifecycle {
    pub fn obligation(&self, selected: &BTreeSet<Vec<String>>) -> Result<Value> {
        let tables = selected
            .iter()
            .map(|path| {
                self.tables
                    .get(path)
                    .cloned()
                    .ok_or("selected table outside native lifecycle module")
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(
            json!({"version":"umf.table-lifecycle/0.1","module":self.context.module,
            "sourceDigest":self.source_digest,"configurationDigest":self.context.configuration_digest,
            "deploymentDigest":self.context.deployment_digest,"tables":tables,
            "sourceInterpretation":"original-umf-deployment-compiler",
            "installedAuthority":"approved-forward-ledger-and-source-derived-native-witness",
            "generationLock":"shared-before-admission-through-transaction",
            "principal":"verified-person-or-batch","roleScope":"native-transitive-current",
            "policy":"source-command-policy-and-native-forced-rls",
            "privileges":"source-grants-and-native-effective-closure",
            "cursorScope":["module","sourceDigest","configurationDigest","deploymentDigest","ledgerHeadDigest","principal","roleScopeDigest"],
            "nativeExecutionQualified":false}),
        )
    }
}

pub(crate) fn admit(
    binding: &Binding,
    doc: &Value,
    records: &BTreeMap<Vec<String>, crate::model::Record>,
) -> Result<Option<AdmittedLifecycle>> {
    let modules = doc["modules"].as_array().ok_or("original modules absent")?;
    let selected = records
        .values()
        .map(|r| r.identity.module.as_str())
        .collect::<BTreeSet<_>>();
    let has_tables = modules.iter().any(|m| {
        m["id"].as_str().is_some_and(|id| selected.contains(id))
            && m["extensions"].get("umf.tables").is_some()
    });
    let Some(context) = &binding.native_lifecycle else {
        if has_tables {
            return Err("selected native table declaration requires explicit lifecycle admission");
        }
        return Ok(None);
    };
    if !has_tables
        || doc["umf"] != "0.10.0"
        || context.version != "umf.table-lifecycle/0.1"
        || !digest(&context.configuration_digest)
        || !digest(&context.deployment_digest)
        || selected.len() != 1
        || !selected.contains(context.module.as_str())
        || binding.schema.as_deref().is_none_or(|s| {
            s.is_empty()
                || s == "public"
                || s == "information_schema"
                || s.to_ascii_lowercase().starts_with("pg_")
        })
        || binding.native_columns.is_some()
        || binding.native_catalog_fence.is_some()
    {
        return Err("native table lifecycle source/context differs");
    }
    let (module_index, module) = modules
        .iter()
        .enumerate()
        .find(|(_, m)| m["id"] == context.module)
        .ok_or("native module absent")?;
    if module["extensions"]
        .as_object()
        .is_none_or(|m| m.len() != 1)
        || doc["vocabularies"]["umf.tables"]["version"] != "0.1.0"
    {
        return Err("native table lifecycle selected vocabulary composition unsupported");
    }
    let payload = &module["extensions"]["umf.tables"];
    if !TablePayload::is_valid(payload)
        || payload["commands"].as_array().is_none_or(|v| !v.is_empty())
    {
        return Err("native table lifecycle payload or command meaning unsupported");
    }
    let declared = payload["tables"].as_array().ok_or("native tables absent")?;
    if declared.len() != binding.tables.len() {
        return Err("complete native module binding inventory required");
    }
    let elements = module["elements"]
        .as_array()
        .ok_or("native module elements absent")?;
    let mut result = BTreeMap::new();
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for (index, table) in declared.iter().enumerate() {
        if table["record"]["module"] != context.module {
            return Err("native table Record must belong to its owning module");
        }
        let record = elements
            .iter()
            .find(|e| e["id"] == table["record"]["element"] && e["kind"] == "record")
            .ok_or("native source Record absent")?;
        if record["extensions"]
            .as_object()
            .is_none_or(|v| !v.is_empty())
        {
            return Err("native table Record extension composition unsupported");
        }
        let path = vec![
            module["namespace"]
                .as_str()
                .ok_or("native namespace absent")?
                .to_owned(),
            query_name(record)
                .ok_or("native Record name absent")?
                .to_owned(),
        ];
        let bound = binding
            .tables
            .iter()
            .find(|t| t.logical == path)
            .ok_or("native Record missing exact binding")?;
        let id = table["id"].as_str().ok_or("native table ID absent")?;
        let native = table["name"].as_str().ok_or("native table name absent")?;
        if !ids.insert(id)
            || !names.insert(native)
            || native.to_ascii_lowercase().starts_with("umf_lifecycle_")
            || bound.native != native
        {
            return Err("native table mapping/identity differs");
        }
        let columns = table["columns"].as_array().ok_or("native columns absent")?;
        let members = record["members"]
            .as_array()
            .ok_or("native members absent")?;
        if columns.len() != bound.columns.len() || columns.len() != members.len() {
            return Err("complete native column binding inventory required");
        }
        let mut fields = BTreeMap::new();
        let mut physical = BTreeSet::new();
        for column in columns {
            let reference = &column["field"];
            if reference["module"] != context.module || !members.contains(reference) {
                return Err("native column outside owning Record/module");
            }
            let field = elements
                .iter()
                .find(|e| e["id"] == reference["element"] && e["kind"] == "field")
                .ok_or("native source Field absent")?;
            let logical = query_name(field).ok_or("native Field name absent")?;
            let c = bound
                .columns
                .iter()
                .find(|c| c.logical == logical)
                .ok_or("native Field missing exact binding")?;
            let family = match c.carrier {
                Carrier::Integer => "integer",
                Carrier::Decimal => "decimal",
                Carrier::String => "string",
                Carrier::Boolean => "boolean",
                Carrier::Uuid => "uuid",
                Carrier::Bits128 => "bits128",
            };
            let compatible = matches!(
                (family, column["nativeType"]["kind"].as_str()),
                ("string", Some("text" | "varchar"))
                    | ("integer", Some("int2" | "int4" | "int8"))
                    | ("decimal", Some("numeric"))
                    | ("boolean", Some("boolean"))
                    | ("uuid", Some("uuid"))
                    | ("bits128", Some("bytea"))
            );
            if !compatible
                || field["scalarType"] != family
                || field["cardinality"] != "one"
                || !matches!(
                    field["nullability"].as_str(),
                    Some("required" | "absent-allowed")
                )
                || c.nullable != (field["nullability"] == "absent-allowed")
                || column["name"] != c.native
                || !physical.insert(c.native.as_str())
                || fields
                    .insert(reference.clone().to_string(), logical.to_owned())
                    .is_some()
                || field["extensions"]
                    .as_object()
                    .is_none_or(|v| !v.is_empty())
            {
                return Err("native column type/mapping/availability differs");
            }
        }
        let primary = record["keys"]
            .as_array()
            .ok_or("native Keys absent")?
            .iter()
            .filter(|k| k["primary"] == true)
            .collect::<Vec<_>>();
        if primary.len() != 1 {
            return Err("native lifecycle requires one original primary Key");
        }
        let key = primary[0]["fields"]
            .as_array()
            .ok_or("native Key fields absent")?
            .iter()
            .map(|f| {
                fields
                    .get(&f.to_string())
                    .cloned()
                    .ok_or("native Key Field missing mapping")
            })
            .collect::<Result<Vec<_>>>()?;
        if bound.key != key {
            return Err("native binding Key differs from original primary Key");
        }
        if result.insert(path,json!({"id":id,"record":table["record"],"native":native,"command":"select","policySourcePath":format!("/modules/{module_index}/extensions/umf.tables/tables/{index}/policy")})).is_some(){return Err("duplicate native Record mapping")}
    }
    for relationship in &binding.relationships {
        if relationship.module.as_deref() != Some(context.module.as_str()) {
            return Err("native relationship module custody required");
        }
        let candidates = declared
            .iter()
            .flat_map(|table| {
                table["foreignKeys"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(move |fk| (table, fk))
            })
            .filter(|(_, fk)| {
                fk["relationship"]["module"] == context.module
                    && fk["relationship"]["id"] == relationship.id
            })
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            return Err("native relationship requires one original FK declaration");
        }
        let (source, fk) = candidates[0];
        let logical_relationship = module["relationships"]
            .as_array()
            .ok_or("native logical relationships absent")?
            .iter()
            .find(|r| r["id"] == relationship.id)
            .ok_or("native logical relationship absent")?;
        if logical_relationship["target"][0]["key"] != fk["target"]["key"] {
            return Err("native FK target Key differs from logical relationship");
        }
        let source_path = result
            .iter()
            .find(|(_, t)| t["record"] == source["record"])
            .map(|(path, _)| path)
            .ok_or("native relationship source absent")?;
        let target_path = result
            .iter()
            .find(|(_, t)| t["record"] == fk["target"]["record"])
            .map(|(path, _)| path)
            .ok_or("native relationship target absent")?;
        let target = elements
            .iter()
            .find(|e| e["id"] == fk["target"]["record"]["element"] && e["kind"] == "record")
            .ok_or("native relationship target Record absent")?;
        let keys = target["keys"]
            .as_array()
            .ok_or("native target Keys absent")?
            .iter()
            .filter(|k| k["id"] == fk["target"]["key"])
            .collect::<Vec<_>>();
        if keys.len() != 1 {
            return Err("native FK requires one original target Key");
        }
        let from = fk["fields"]
            .as_array()
            .ok_or("native FK source fields absent")?;
        let to = keys[0]["fields"]
            .as_array()
            .ok_or("native FK target fields absent")?;
        if from.len() != to.len() {
            return Err("native FK tuple arity differs");
        }
        let logical = |reference: &Value| -> Result<String> {
            if reference["module"] != context.module {
                return Err("native FK Field outside module");
            }
            let field = elements
                .iter()
                .find(|e| e["id"] == reference["element"] && e["kind"] == "field")
                .ok_or("native FK Field absent")?;
            Ok(query_name(field)
                .ok_or("native FK Field name absent")?
                .to_owned())
        };
        let pairs = from
            .iter()
            .zip(to)
            .map(|(a, b)| Ok([logical(a)?, logical(b)?]))
            .collect::<Result<Vec<_>>>()?;
        if relationship.source != *source_path
            || relationship.target != *target_path
            || relationship.pairs != pairs
        {
            return Err("native relationship binding differs from original FK");
        }
    }
    Ok(Some(AdmittedLifecycle {
        context: context.clone(),
        source_digest: binding.model_sha256.clone(),
        tables: result,
    }))
}
