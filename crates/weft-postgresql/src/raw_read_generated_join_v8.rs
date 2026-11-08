// SPDX-License-Identifier: MIT OR Apache-2.0
//! Closed compiler-free required to-one inner-join issuer for generated read v8.
use super::{Request, field_key, identity_component, native_type_carrier_oid};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use weft_core::raw_read::{Binding, Table};
type Result<T> = std::result::Result<T, &'static str>;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RequestV8 {
    version: String,
    target: String,
    binding: Binding,
    module: String,
    native_source3: Value,
}

fn obj<'a>(
    value: &'a Value,
    required: &[&str],
    allowed: &[&str],
    error: &'static str,
) -> Result<&'a Map<String, Value>> {
    let object = value.as_object().ok_or(error)?;
    if required.iter().any(|key| !object.contains_key(*key))
        || object.keys().any(|key| !allowed.contains(&key.as_str()))
    {
        return Err(error);
    }
    Ok(object)
}
fn ident(value: &str) -> Result<String> {
    if value.is_empty() || value.len() > 63 || value.contains('\0') {
        return Err("generated join native identifier outside profile");
    }
    Ok(format!("\"{}\"", value.replace('"', "\"\"").replace('%', "%%")))
}
fn path_ident(schema: &str, table: &str) -> Result<String> {
    Ok(format!("{}.{}", ident(schema)?, ident(table)?))
}
fn record_path(module: &str, record: &str) -> String {
    format!(
        "{}/{}",
        identity_component(module),
        identity_component(record)
    )
}
fn relation_path(module: &str, relationship: &str) -> String {
    format!(
        "{}/{}",
        identity_component(module),
        identity_component(relationship)
    )
}
fn value_ref(value: &Value) -> Result<(String, String)> {
    obj(
        value,
        &["module", "element"],
        &["module", "element", "key"],
        "generated join Field/Record ref differs",
    )?;
    Ok((
        value["module"]
            .as_str()
            .ok_or("generated join ref module absent")?
            .into(),
        value["element"]
            .as_str()
            .ok_or("generated join ref element absent")?
            .into(),
    ))
}
fn ref_json(module: &str, element: &str) -> Value {
    json!({"module":module,"element":element})
}
fn read_type(value: &Value) -> Result<(&'static str, u64)> {
    native_type_carrier_oid(value)
}
fn qualified_fragment(fragment: &str, owner: &str) -> Result<String> {
    if owner == "root" {
        return Ok(fragment.to_owned());
    }
    if owner != "target" || fragment.matches("\"r\".").count() != 1 {
        return Err("generated join inherited fragment alias differs");
    }
    Ok(fragment.replace("\"r\".", "\"t\"."))
}

fn verify_source3(request: &RequestV8) -> Result<()> {
    let source = &request.native_source3;
    obj(
        source,
        &[
            "version",
            "bundleDigest",
            "sourceDigest",
            "configurationDigest",
            "configuration",
            "expectedCatalog",
            "operations",
            "relationships",
        ],
        &[
            "version",
            "bundleDigest",
            "sourceDigest",
            "configurationDigest",
            "configuration",
            "expectedCatalog",
            "operations",
            "relationships",
        ],
        "generated read v8 nativeSource3 shape differs",
    )?;
    if source["version"] != "umf.native-table-source/3" {
        return Err("generated read v8 nativeSource3 version differs");
    }
    let lifecycle = request
        .binding
        .native_lifecycle
        .as_ref()
        .ok_or("generated read v8 requires native lifecycle binding")?;
    if lifecycle.version != "umf.table-lifecycle/0.1"
        || lifecycle.module != request.module
        || lifecycle.configuration_digest
            != source["configurationDigest"]
                .as_str()
                .ok_or("generated read v8 native configuration digest absent")?
        || lifecycle.deployment_digest
            != source["bundleDigest"]
                .as_str()
                .ok_or("generated read v8 native bundle digest absent")?
        || source["sourceDigest"].as_str() != Some(request.binding.model_sha256.as_str())
        || source["configuration"]["module"] != request.module
        || source["configuration"]["schema"].as_str() != request.binding.schema.as_deref()
        || request.binding.model_sha256
            != weft_core::json::sha256(request.binding.model_json.as_bytes())
    {
        return Err("generated read v8 nativeSource3 source/deployment pins differ");
    }
    let catalog = &source["expectedCatalog"];
    obj(
        catalog,
        &["tables", "checks", "keys", "foreignKeys"],
        &["tables", "checks", "keys", "foreignKeys"],
        "generated read v8 native catalog shape differs",
    )?;
    if !catalog["tables"].is_array()
        || !catalog["checks"].is_array()
        || !catalog["keys"].is_array()
        || !catalog["foreignKeys"].is_array()
        || !source["relationships"].is_array()
        || !source["operations"].is_array()
    {
        return Err("generated read v8 nativeSource3 inventories differ");
    }
    // Check every emitted native table/column against the exact original binding.
    let mut table_ids = BTreeSet::new();
    for table in catalog["tables"].as_array().unwrap() {
        obj(
            table,
            &["id", "name", "record", "columns"],
            &["id", "name", "record", "columns"],
            "generated read v8 native table row differs",
        )?;
        if !table_ids.insert(
            table["id"]
                .as_str()
                .ok_or("generated native table ID absent")?
                .to_owned(),
        ) {
            return Err("generated read v8 native table identity duplicated");
        }
        for column in table["columns"]
            .as_array()
            .ok_or("generated native column inventory absent")?
        {
            obj(
                column,
                &["name", "field", "nativeType"],
                &["name", "field", "nativeType"],
                "generated read v8 native column row differs",
            )?;
        }
    }
    for table in &request.binding.tables {
        let matches = catalog["tables"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|native| {
                native["name"] == table.native
                    && native["record"]
                        == json!({"module":table.logical[0],"element":table.logical[1]})
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err("generated read v8 native table/binding correspondence differs");
        }
        let native = matches[0];
        let native_columns = native["columns"]
            .as_array()
            .ok_or("generated native columns absent")?;
        let mut seen = BTreeSet::new();
        for column in &table.columns {
            let source_field = source_field_for(request, table, &column.logical)?;
            let matches = native_columns
                .iter()
                .filter(|item| item["field"] == source_field && item["name"] == column.native)
                .collect::<Vec<_>>();
            if matches.len() != 1 || !seen.insert(field_key(&source_field)?) {
                return Err("generated read v8 bound column/native projection differs");
            }
        }
    }
    Ok(())
}

fn source_field_for(request: &RequestV8, table: &Table, logical: &str) -> Result<Value> {
    let document: Value = serde_json::from_str(&request.binding.model_json)
        .map_err(|_| "generated read v8 original UMF JSON invalid")?;
    let module = document["modules"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["id"] == table.logical[0]))
        .ok_or("generated read v8 original module absent")?;
    let record = module["elements"]
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item["id"] == table.logical[1] && item["kind"] == "record")
        })
        .ok_or("generated read v8 original Record absent")?;
    let member_refs = record["members"]
        .as_array()
        .ok_or("generated read v8 Record members absent")?;
    let elements = module["elements"]
        .as_array()
        .ok_or("generated read v8 module elements absent")?;
    let candidates = member_refs
        .iter()
        .filter(|reference| {
            let id = reference["element"].as_str();
            elements
                .iter()
                .find(|element| element["id"].as_str() == id)
                .is_some_and(|field| {
                    field["name"].as_str().or_else(|| field["id"].as_str()) == Some(logical)
                })
        })
        .collect::<Vec<_>>();
    if candidates.len() != 1 {
        return Err("generated read v8 logical Field mapping ambiguous");
    }
    Ok(candidates[0].clone())
}

fn unsupported(reason: &str) -> Value {
    json!({"status":"unsupported","reason":reason})
}

fn issue_output_value_codec(
    request: &RequestV8,
    source_ref: &Value,
    output: &Value,
) -> Result<Value> {
    let model: Value = serde_json::from_str(&request.binding.model_json)
        .map_err(|_| "generated join original model JSON invalid")?;
    if let Ok(mut codec) = weft_core::table_query::generated_codec_for_member(&model, source_ref) {
        let steps = codec["steps"]
            .as_array_mut()
            .ok_or("generated join output codec steps absent")?;
        if steps.len() != 4 {
            return Err("generated join output codec instruction count differs");
        }
        steps[0]["nullable"] = output["nullable"].clone();
        return Ok(codec);
    }
    let module_id = source_ref["module"]
        .as_str()
        .ok_or("generated join output Field module absent")?;
    let field_id = source_ref["element"]
        .as_str()
        .ok_or("generated join output Field ID absent")?;
    let module = model["modules"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["id"] == module_id))
        .ok_or("generated join output Field module unresolved")?;
    let fields = module["elements"]
        .as_array()
        .ok_or("generated join output Field declarations absent")?
        .iter()
        .filter(|item| item["id"] == field_id && item["kind"] == "field")
        .collect::<Vec<_>>();
    if fields.len() != 1 {
        return Err("generated join output Field declaration ambiguous");
    }
    let field = fields[0];
    let facets = field
        .get("facets")
        .filter(|value| !value.is_null())
        .cloned()
        .unwrap_or_else(|| json!({}));
    let allowed = field
        .get("allowedValues")
        .filter(|value| !value.is_null())
        .cloned()
        .unwrap_or_else(|| json!([]));
    let references = field
        .get("references")
        .filter(|value| !value.is_null())
        .cloned()
        .unwrap_or_else(|| json!([]));
    let extensions = field
        .get("extensions")
        .filter(|value| !value.is_null())
        .cloned()
        .unwrap_or_else(|| json!({}));
    if facets.as_object().is_none_or(|value| !value.is_empty())
        || allowed.as_array().is_none_or(|value| !value.is_empty())
        || references.as_array().is_none_or(|value| !value.is_empty())
        || extensions.as_object().is_none_or(|value| !value.is_empty())
    {
        return Err("output-result-codec-outside-profile");
    }
    let carrier = output["carrier"]
        .as_str()
        .ok_or("generated join output carrier absent")?;
    let native_type = &output["nativeType"];
    let oid = output["oid"]
        .as_u64()
        .ok_or("generated join output OID absent")?;
    let profile = match (carrier, native_type["kind"].as_str(), oid) {
        ("uuid", Some("uuid"), 2950) if field["scalarType"] == "uuid" => "weft.postgresql18.uuid/1",
        ("bits128", Some("bytea"), 17) if field["scalarType"] == "bits128" => {
            "weft.postgresql18.bytea-16/1"
        }
        ("date", Some("date"), 1082) if field["scalarType"] == "date" => "weft.postgresql18.date/1",
        ("timestamp", Some("timestamptz"), 1184) if field["scalarType"] == "timestamp" => {
            "weft.postgresql18.utc-timestamp/1"
        }
        _ => return Err("output-result-codec-outside-profile"),
    };
    if (field["nullability"] == "required") != (output["nullable"] == false) {
        return Err("generated join result nullability differs from source Field");
    }
    Ok(json!({
        "profile":"weft.query-result-codec/1",
        "carrier":carrier,
        "nullable":output["nullable"],
        "nativeType":native_type,
        "oid":oid,
        "decoder":{"op":"decode-native-result","profile":profile},
        "refinements":{"profile":"weft.query-result-refinements/1","facets":{},"allowedValues":[]}
    }))
}

fn issue_field_slots(
    request: &RequestV8,
    owner: &str,
    record: &Value,
    algebra: &Value,
    binding_table: &Table,
    native_table: &Value,
    ordinal_base: usize,
) -> Result<(Vec<Value>, BTreeMap<String, Value>)> {
    let fields = algebra["fields"]
        .as_object()
        .ok_or("generated join base fields absent")?;
    let columns = native_table["columns"]
        .as_array()
        .ok_or("generated join native columns absent")?;
    let mut slots = Vec::new();
    let mut by_id = BTreeMap::new();
    for (field_id, public_field) in fields {
        let source_ref = public_field
            .get("sourceField")
            .ok_or("generated join Field source ref absent")?;
        let (_, element_id) = value_ref(source_ref)?;
        let bound_rows = binding_table
            .columns
            .iter()
            .filter(|column| {
                source_field_for(request, binding_table, &column.logical).is_ok_and(|resolved| {
                    resolved["element"] == element_id && resolved["module"] == source_ref["module"]
                })
            })
            .collect::<Vec<_>>();
        if bound_rows.len() != 1 {
            return Err("generated join Field binding ambiguous");
        }
        let bound = bound_rows[0];
        let native_rows = columns
            .iter()
            .filter(|item| item["field"] == *source_ref && item["name"] == bound.native)
            .collect::<Vec<_>>();
        if native_rows.len() != 1 {
            return Err("generated join Field native mapping differs");
        }
        let metadata = super::ordinary_page_field_output(
            field_id,
            public_field,
            source_ref,
            bound,
            native_rows[0],
        )?;
        let value_codec = issue_output_value_codec(request, source_ref, &metadata["output"])?;
        let (carrier, oid) = read_type(&metadata["output"]["nativeType"])?;
        if metadata["output"]["carrier"] != carrier || metadata["output"]["oid"] != oid {
            return Err("generated join Field output type/OID differs");
        }
        let ordinal = ordinal_base + slots.len();
        let mut slot = metadata;
        slot["output"]["valueCodec"] = value_codec;
        slot["owner"] = json!(owner);
        slot["ownerRecord"] = record.clone();
        slot["ordinal"] = json!(ordinal);
        let original_alias = public_field["output"]["name"]
            .as_str()
            .ok_or("generated join base output alias absent")?;
        let original_select = public_field["selectSql"]
            .as_str()
            .ok_or("generated join base select SQL absent")?;
        let suffix = format!(" AS {}", ident(original_alias)?);
        let expression = original_select
            .strip_suffix(&suffix)
            .ok_or("generated join base select alias differs")?;
        let expression = qualified_fragment(expression, owner)?;
        let alias = format!("__weft_field_{ordinal}");
        slot["output"]["name"] = json!(alias);
        slot["selectSql"] = json!(format!("{expression} AS {}", ident(&alias)?));
        slot["frameSelectSql"] = json!(format!("f.{}", ident(&format!("__weft_field_{ordinal}"))?));
        if by_id.insert(field_id.clone(), slot.clone()).is_some() {
            return Err("generated join owner Field ID duplicated");
        }
        slots.push(slot);
    }
    Ok((slots, by_id))
}

fn issue_predicates(
    owner: &str,
    algebra: &Value,
    fields: &BTreeMap<String, Value>,
) -> Result<Value> {
    let predicates = algebra["predicates"]
        .as_object()
        .ok_or("generated join base predicates absent")?;
    let mut output = Map::new();
    for (id, predicate) in predicates {
        let field_id = predicate["fieldId"]
            .as_str()
            .ok_or("generated join predicate Field ID absent")?;
        let _field = fields
            .get(field_id)
            .ok_or("generated join predicate Field absent")?;
        let op = predicate["operator"]
            .as_str()
            .ok_or("generated join predicate operator absent")?;
        let source_sql = predicate["sql"]
            .as_str()
            .ok_or("generated join predicate SQL absent")?;
        let expression = qualified_fragment(source_sql, owner)?;
        let mut value = json!({"field":{"owner":owner,"fieldId":field_id},"fieldId":field_id,"operator":op,"sql":expression,"arity":predicate["arity"]});
        if predicate["arity"] == 1 {
            value["codec"] = predicate
                .get("codec")
                .cloned()
                .ok_or("generated join predicate codec absent")?;
        } else if predicate["arity"] != 0 {
            return Err("generated join predicate arity differs");
        }
        output.insert(id.clone(), value);
    }
    Ok(Value::Object(output))
}
fn issue_orders(owner: &str, algebra: &Value, fields: &BTreeMap<String, Value>) -> Result<Value> {
    let orders = algebra["orders"]
        .as_object()
        .ok_or("generated join base orders absent")?;
    let mut output = Map::new();
    for (id, order) in orders {
        let field_id = order["fieldId"]
            .as_str()
            .ok_or("generated join order Field ID absent")?;
        let _field = fields
            .get(field_id)
            .ok_or("generated join order Field absent")?;
        let direction = order["direction"]
            .as_str()
            .ok_or("generated join order direction absent")?;
        if !matches!(direction, "asc" | "desc") || order["nulls"] != "last" {
            return Err("generated join order meaning outside closed vocabulary");
        }
        let source_sql = order["sql"]
            .as_str()
            .ok_or("generated join base order SQL absent")?;
        let sql = qualified_fragment(source_sql, owner)?;
        output.insert(id.clone(), json!({"field":{"owner":owner,"fieldId":field_id},"fieldId":field_id,"direction":direction,"nulls":"last","sql":sql}));
    }
    Ok(Value::Object(output))
}

fn table_by_record<'a>(binding: &'a Binding, reference: &Value) -> Result<&'a Table> {
    let path = vec![
        reference["module"]
            .as_str()
            .ok_or("generated join Record module absent")?
            .to_owned(),
        reference["element"]
            .as_str()
            .ok_or("generated join Record ID absent")?
            .to_owned(),
    ];
    let found = binding
        .tables
        .iter()
        .filter(|table| table.logical == path)
        .collect::<Vec<_>>();
    if found.len() != 1 {
        return Err("generated join native table binding missing or ambiguous");
    }
    Ok(found[0])
}
fn native_table<'a>(catalog: &'a Value, table: &Table) -> Result<&'a Value> {
    let record = json!({"module":table.logical[0],"element":table.logical[1]});
    let found = catalog["tables"]
        .as_array()
        .ok_or("generated join native tables absent")?
        .iter()
        .filter(|item| item["name"] == table.native && item["record"] == record)
        .collect::<Vec<_>>();
    if found.len() != 1 {
        return Err("generated join native table witness missing or ambiguous");
    }
    Ok(found[0])
}

fn eligible_descriptor(
    request: &RequestV8,
    root: &Value,
    root_algebra: &Value,
    row: &Value,
) -> Result<Value> {
    let logical = row
        .get("logical")
        .ok_or("generated join admitted logical cut absent")?;
    obj(
        logical,
        &[
            "module",
            "id",
            "source",
            "target",
            "sourceMultiplicity",
            "targetMultiplicity",
            "targetLifecycle",
            "directed",
        ],
        &[
            "module",
            "id",
            "source",
            "target",
            "sourceMultiplicity",
            "targetMultiplicity",
            "targetLifecycle",
            "directed",
        ],
        "generated join logical relationship shape differs",
    )?;
    let root_record = json!({"module":root["module"],"element":root["record"]});
    let source_ref = &logical["source"];
    let target_ref = &logical["target"];
    let source = value_ref(source_ref)?;
    let target = value_ref(target_ref)?;
    if source_ref != &root_record {
        return Err("generated join source is not this root");
    }
    if source == target {
        return Err("generated join self/alias relationship is outside profile");
    }
    if logical["directed"] != true
        || logical["targetLifecycle"] != "independent"
        || logical["targetMultiplicity"] != json!({"min":1,"max":1})
        || logical["target"]["key"].as_str().is_none()
    {
        return Err("generated join relationship is not required directed to-one");
    }
    let native = &request.native_source3;
    let target_table = table_by_record(&request.binding, target_ref)?;
    let root_table = table_by_record(&request.binding, &root_record)?;
    let catalog = &native["expectedCatalog"];
    let root_native = native_table(catalog, root_table)?;
    let target_native = native_table(catalog, target_table)?;
    let target_root = root["__targetAlgebra"].clone();
    let root_record_json = root_record.clone();
    let (mut root_slots, root_fields) = issue_field_slots(
        request,
        "root",
        &root_record_json,
        root_algebra,
        root_table,
        root_native,
        0,
    )?;
    let target_record_json = ref_json(&target.0, &target.1);
    let (target_slots, target_fields) = issue_field_slots(
        request,
        "target",
        &target_record_json,
        &target_root,
        target_table,
        target_native,
        root_slots.len(),
    )?;
    let mut field_slots = Vec::new();
    field_slots.append(&mut root_slots);
    field_slots.extend(target_slots);
    let predicates = json!({"root":issue_predicates("root",root_algebra,&root_fields)?,"target":issue_predicates("target",&target_root,&target_fields)?});
    let orders = json!({"root":issue_orders("root",root_algebra,&root_fields)?,"target":issue_orders("target",&target_root,&target_fields)?});
    let native_binding = row
        .get("binding")
        .ok_or("generated join binding witness absent")?;
    let native_physical = row
        .get("native")
        .ok_or("generated join native witness absent")?;
    let pairs = native_binding["pairs"]
        .as_array()
        .filter(|pairs| !pairs.is_empty())
        .ok_or("generated join pair list absent")?;
    if native_binding["sourceRecord"] != json!([source.0, source.1])
        || native_binding["targetRecord"] != json!([target.0, target.1])
        || native_binding["toOne"] != true
    {
        return Err("generated join relationship binding direction differs");
    }
    let binding_relations = request
        .binding
        .relationships
        .iter()
        .filter(|item| {
            item.id == logical["id"].as_str().unwrap_or("")
                && item.module.as_deref().unwrap_or(&request.module) == request.module
        })
        .collect::<Vec<_>>();
    if binding_relations.len() != 1 {
        return Err("generated join exact raw relationship binding absent or ambiguous");
    }
    let binding_relation = binding_relations[0];
    if binding_relation.source != vec![source.0.clone(), source.1.clone()]
        || binding_relation.target != vec![target.0.clone(), target.1.clone()]
        || !binding_relation.to_one
        || binding_relation.pairs.len() != pairs.len()
    {
        return Err("generated join raw relationship binding differs");
    }
    let key_id = logical["target"]["key"].as_str().unwrap();
    let target_key = &native_physical["targetKey"];
    let foreign_key = &native_physical["foreignKey"];
    let key_rows = catalog["keys"]
        .as_array()
        .ok_or("generated join verified native Keys absent")?;
    let fk_rows = catalog["foreignKeys"]
        .as_array()
        .ok_or("generated join verified native Foreign Keys absent")?;
    if key_rows.iter().filter(|item| *item == target_key).count() != 1
        || fk_rows.iter().filter(|item| *item == foreign_key).count() != 1
        || target_key["id"] != key_id
        || target_key["table"] != target_native["id"]
        || foreign_key["table"] != root_native["id"]
        || foreign_key["targetKey"] != target_key["name"]
    {
        return Err("generated join verified native Key/Foreign Key witness differs");
    }
    let model: Value = serde_json::from_str(&request.binding.model_json)
        .map_err(|_| "generated join original model JSON invalid")?;
    let source_module = model["modules"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["id"] == source.0))
        .ok_or("generated join source module absent")?;
    let source_tables = source_module["extensions"]["umf.tables"]["tables"]
        .as_array()
        .ok_or("generated join original native table declarations absent")?;
    let fk_decl = source_tables
        .iter()
        .filter(|table| table["id"] == root_native["id"])
        .flat_map(|table| table["foreignKeys"].as_array().into_iter().flatten())
        .filter(|item| {
            item["id"] == foreign_key["id"]
                && item["name"] == foreign_key["name"]
                && item["relationship"]["id"] == logical["id"]
                && item["relationship"]["module"] == logical["module"]
        })
        .collect::<Vec<_>>();
    if fk_decl.len() != 1 {
        return Err("generated join original native Foreign Key declaration differs");
    }
    let mut on = Vec::new();
    let mut source_fields = Vec::new();
    let mut target_fields_refs = Vec::new();
    let mut source_columns = Vec::new();
    let mut target_columns = Vec::new();
    for (index, pair) in pairs.iter().enumerate() {
        obj(
            pair,
            &["sourceField", "targetField", "sourceColumn", "targetColumn"],
            &["sourceField", "targetField", "sourceColumn", "targetColumn"],
            "generated join native pair shape differs",
        )?;
        let source_field = &pair["sourceField"];
        let target_field = &pair["targetField"];
        let source_ref_pair = value_ref(source_field)?;
        let target_ref_pair = value_ref(target_field)?;
        let source_column = &pair["sourceColumn"];
        let target_column = &pair["targetColumn"];
        if source_column["field"] != *source_field
            || target_column["field"] != *target_field
            || source_column["tableId"] != root_native["id"]
            || target_column["tableId"] != target_native["id"]
            || source_column["tableName"] != root_native["name"]
            || target_column["tableName"] != target_native["name"]
        {
            return Err("generated join physical pair ownership differs");
        }
        let root_col = root_native["columns"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| {
                item["field"] == *source_field && item["name"] == source_column["columnName"]
            })
            .collect::<Vec<_>>();
        let target_col = target_native["columns"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| {
                item["field"] == *target_field && item["name"] == target_column["columnName"]
            })
            .collect::<Vec<_>>();
        if root_col.len() != 1 || target_col.len() != 1 {
            return Err("generated join exact physical columns absent or ambiguous");
        }
        let source_bound = root_table
            .columns
            .iter()
            .filter(|item| {
                item.native == source_column["columnName"]
                    && source_field_for(request, root_table, &item.logical).is_ok_and(|resolved| {
                        resolved["element"] == source_ref_pair.1
                            && resolved["module"] == source_ref_pair.0
                    })
            })
            .collect::<Vec<_>>();
        let target_bound = target_table
            .columns
            .iter()
            .filter(|item| {
                item.native == target_column["columnName"]
                    && source_field_for(request, target_table, &item.logical).is_ok_and(
                        |resolved| {
                            resolved["element"] == target_ref_pair.1
                                && resolved["module"] == target_ref_pair.0
                        },
                    )
            })
            .collect::<Vec<_>>();
        if source_bound.len() != 1
            || target_bound.len() != 1
            || source_bound[0].carrier != target_bound[0].carrier
        {
            return Err("generated join binding pair carrier/column differs");
        }
        if binding_relation.pairs[index]
            != [
                source_bound[0].logical.clone(),
                target_bound[0].logical.clone(),
            ]
        {
            return Err("generated join raw binding pair order differs");
        }
        if source_bound[0].nullable {
            return Err("nullable-source-pair");
        }
        let source_native_type = &root_col[0]["nativeType"];
        let target_native_type = &target_col[0]["nativeType"];
        if source_native_type != target_native_type {
            return Err("generated join native pair type differs");
        }
        let left = format!(
            "{}.{}",
            ident("r")?,
            ident(
                source_column["columnName"]
                    .as_str()
                    .ok_or("generated join source column name absent")?
            )?
        );
        let right = format!(
            "{}.{}",
            ident("t")?,
            ident(
                target_column["columnName"]
                    .as_str()
                    .ok_or("generated join target column name absent")?
            )?
        );
        let equality = if source_bound[0].carrier == weft_core::raw_read::Carrier::String {
            format!("{left} COLLATE pg_catalog.\"C\" = {right} COLLATE pg_catalog.\"C\"")
        } else {
            format!("{left} = {right}")
        };
        on.push(equality);
        source_fields.push(source_field.clone());
        target_fields_refs.push(target_field.clone());
        source_columns.push(source_column["columnName"].clone());
        target_columns.push(target_column["columnName"].clone());
        let _ = index;
    }
    let fk_field_rows = fk_decl[0]["fields"]
        .as_array()
        .ok_or("generated join original Foreign Key fields absent")?;
    if on.is_empty() || &source_fields != fk_field_rows {
        return Err("generated join pair order differs from original Foreign Key");
    }
    let target_module = model["modules"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["id"] == target.0))
        .ok_or("generated join target module absent")?;
    let target_record = target_module["elements"]
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item["id"] == target.1 && item["kind"] == "record")
        })
        .ok_or("generated join target Record absent")?;
    let target_key_rows = target_record["keys"]
        .as_array()
        .ok_or("generated join target Key inventory absent")?
        .iter()
        .filter(|item| item["id"] == key_id)
        .collect::<Vec<_>>();
    if target_key_rows.len() != 1
        || target_fields_refs
            != target_key_rows[0]["fields"]
                .as_array()
                .cloned()
                .ok_or("generated join target Key fields absent")?
        || target_columns
            != target_key["columns"]
                .as_array()
                .cloned()
                .ok_or("generated join native target Key columns absent")?
    {
        return Err("generated join target Key field/column order differs");
    }
    let join_sql = format!(
        "INNER JOIN {} AS \"t\" ON {}",
        path_ident(
            request
                .binding
                .schema
                .as_deref()
                .ok_or("generated join requires qualified schema")?,
            target_table.native.as_str()
        )?,
        on.join(" AND ")
    );
    let root_from = root_algebra["fromSql"]
        .as_str()
        .ok_or("generated join root FROM absent")?;
    if !root_from.starts_with("FROM ") || !root_from.ends_with(" AS \"r\"") {
        return Err("generated join root alias differs");
    }
    let primary_key_ties = root_algebra["keyTieOrder"]
        .as_array()
        .cloned()
        .ok_or("generated join root Key ties absent")?;
    if primary_key_ties.is_empty() {
        return Err("generated join root has no stable Key tie order");
    }
    let statement_source = format!(
        "SELECT ROW_NUMBER() OVER (ORDER BY {{{{ORDER_BY}}}}) AS \"__weft_ordinal\"{{{{PROJECTION}}}} {} {} WHERE ({{{{FILTER}}}}) ORDER BY {{{{ORDER_BY}}}} LIMIT %s",
        root_from, join_sql
    );
    let guard = root["nativeReadGuard"]["template"]["sql"]
        .as_str()
        .ok_or("generated join inherited server byte guard absent")?;
    if guard.matches("{{SOURCE_SQL}}").count() != 1
        || guard.matches("{{RETURN_COLUMNS}}").count() != 1
        || root["nativeReadGuard"]["profile"] != "pg18-composed-native-frame/1"
    {
        return Err("generated join inherited server byte guard template differs");
    }
    let sql_template = guard
        .replace("{{SOURCE_SQL}}", &statement_source)
        .replace("{{RETURN_COLUMNS}}", "f.*");
    let template_sha = weft_core::json::sha256(sql_template.as_bytes());
    let descriptor = json!({
        "profile":"weft.generated-required-inner-join/1",
        "relationship":{"module":logical["module"],"id":logical["id"],"source":logical["source"],"target":logical["target"],"sourceMultiplicity":logical["sourceMultiplicity"],"targetMultiplicity":logical["targetMultiplicity"],"targetLifecycle":logical["targetLifecycle"],"directed":logical["directed"]},
        "binding":native_binding,
        "native":native_physical,
        "aliases":{"root":"r","target":"t"},
        "joinSql":join_sql,
        "fields":field_slots,
        "selection":{"allowed":field_slots.iter().map(|field| json!({"owner":field["owner"],"fieldId":field["fieldId"]})).collect::<Vec<_>>(),"ordered":true,"ownerRequired":true,"nullMeansDefault":true,"emptyAllowed":true,"emptyPublicTuple":[]},
        "defaultSelection":root_algebra["defaultSelection"].as_array().ok_or("generated join root default selection absent")?.iter().map(|field_id| json!({"owner":"root","fieldId":field_id})).collect::<Vec<_>>(),
        "predicates":predicates,
        "orders":orders,
        "statement":{"profile":"weft.generated-required-inner-join-statement/1","terminal":"all","sourceSqlTemplate":statement_source,"sqlTemplate":sql_template,"templateSha256":template_sha,"parameterSlots":{"projection":"PROJECTION","filter":"FILTER","order":"ORDER_BY","limit":"%s"},"projectionSlot":{"name":"PROJECTION","emptyValue":"","nonemptyPrefix":", "},"ordinalOutput":{"name":"__weft_ordinal","carrier":"integer","nullable":false,"nativeType":{"kind":"int8"},"oid":20},"parameterOrder":["predicateValues","limit"],"predicateValueOrder":"owner-qualified-expression-depth-first-left-to-right","maxRows":1000,"limit":{"minimum":1,"maximum":1000,"nullMeans":1000},"serverByteGuard":true,"byteGuard":{"profile":root["nativeReadGuard"]["profile"],"byteMetric":root["nativeReadGuard"]["byteMetric"],"maxNativeBytes":root["nativeReadGuard"]["maxNativeBytes"],"sqlstate":root["nativeReadGuard"]["sqlstate"],"partialFrames":root["nativeReadGuard"]["partialFrames"],"templateSha256":root["nativeReadGuard"]["template"]["sha256"]},"rootKeyTieOrder":primary_key_ties,"visibleTargetSemantics":"only-visible-matching-target-rows"}
    });
    // Validate a closed representative expansion with the PostgreSQL parser.
    let probe = statement_source
        .replace("{{ORDER_BY}}", "\"r\".\"id\" ASC NULLS LAST")
        .replace("{{PROJECTION}}", ", \"r\".\"id\" AS \"__weft_field_0\"")
        .replace("{{FILTER}}", "TRUE");
    let probe = probe.replace("LIMIT %s", "LIMIT 1");
    let _ = super::parse_compiled(&probe)?;
    let empty_probe = statement_source
        .replace("{{ORDER_BY}}", "\"r\".\"id\" ASC NULLS LAST")
        .replace("{{PROJECTION}}", "")
        .replace("{{FILTER}}", "TRUE")
        .replace("LIMIT %s", "LIMIT 1");
    let _ = super::parse_compiled(&empty_probe)?;
    // Output fields retain issuer ordinals; no call-supplied owner or native metadata is trusted.
    if descriptor["fields"]
        .as_array()
        .is_none_or(|values| values.is_empty())
    {
        return Err("generated join output inventory absent");
    }
    Ok(descriptor)
}

fn build_v8(request: RequestV8, binding_digest: String) -> Result<Value> {
    if request.version != "weft.generated-read-algebra-request/8" {
        return Err("generated read algebra v8 request profile differs");
    }
    if !matches!(request.target.as_str(), "postgresql18" | "lakebase18") {
        return Err("generated read v8 target outside profile");
    }
    verify_source3(&request)?;
    let base_request = Request {
        version: "weft.generated-read-algebra-request/7".into(),
        target: request.target.clone(),
        binding: request.binding.clone(),
        module: request.module.clone(),
    };
    let mut output = super::build_v7_query_names(base_request, binding_digest)?;
    let source_document: Value = serde_json::from_str(&request.binding.model_json)
        .map_err(|_| "generated read v8 source model JSON invalid")?;
    let module = source_document["modules"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["id"] == request.module))
        .ok_or("generated read v8 source module absent")?;
    let original_relationships = module
        .get("relationships")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let projected = request.native_source3["relationships"]
        .as_array()
        .ok_or("generated read v8 Relationship inventory absent")?;
    if projected.len() != original_relationships.len() {
        return Err("generated read v8 Relationship inventory incomplete");
    }
    let mut rows = BTreeMap::new();
    for row in projected {
        obj(
            row,
            &["declaration", "logicalIdentity", "status"],
            &[
                "declaration",
                "logicalIdentity",
                "status",
                "logical",
                "binding",
                "native",
                "reasons",
            ],
            "generated read v8 Relationship row shape differs",
        )?;
        let identity = &row["logicalIdentity"];
        let module_id = identity["module"]
            .as_str()
            .ok_or("generated read v8 Relationship module absent")?;
        let relationship_id = identity["id"]
            .as_str()
            .ok_or("generated read v8 Relationship ID absent")?;
        if module_id != request.module
            || !original_relationships
                .iter()
                .any(|item| item == &row["declaration"] && item["id"] == relationship_id)
        {
            return Err("generated read v8 Relationship declaration differs from original source");
        }
        let key = relation_path(module_id, relationship_id);
        if rows.insert(key, row).is_some() {
            return Err("generated read v8 Relationship identity duplicated");
        }
    }
    let root_snapshot = output["roots"].clone();
    let roots = output["roots"]
        .as_object_mut()
        .ok_or("generated read v8 roots absent")?;
    for (root_key, root) in roots.iter_mut() {
        let root_record = json!({"module":root["module"],"element":root["record"]});
        let mut catalog_rows = BTreeMap::new();
        for (relationship_key, row) in &rows {
            let entry = if row["status"] == "refused" {
                let reasons = row["reasons"]
                    .as_array()
                    .filter(|items| !items.is_empty())
                    .ok_or("generated read v8 refused relationship reasons absent")?;
                unsupported(&format!(
                    "native-source3-refused:{}",
                    reasons
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(",")
                ))
            } else if row["status"] != "admitted" {
                return Err("generated read v8 Relationship status differs");
            } else if row["declaration"]["source"]
                .as_array()
                .is_none_or(|items| items.len() != 1)
                || row["declaration"]["target"]
                    .as_array()
                    .is_none_or(|items| items.len() != 1)
                || row["logical"]["module"] != row["logicalIdentity"]["module"]
                || row["logical"]["id"] != row["logicalIdentity"]["id"]
                || row["logical"]["source"] != row["declaration"]["source"][0]
                || row["logical"]["target"]["module"] != row["declaration"]["target"][0]["module"]
                || row["logical"]["target"]["element"] != row["declaration"]["target"][0]["element"]
                || row["logical"]["target"]["key"] != row["declaration"]["target"][0]["key"]
                || row["logical"]["sourceMultiplicity"] != row["declaration"]["sourceMultiplicity"]
                || row["logical"]["targetMultiplicity"] != row["declaration"]["targetMultiplicity"]
                || row["logical"]["targetLifecycle"] != row["declaration"]["targetLifecycle"]
                || row["logical"]["directed"] != row["declaration"]["directed"]
            {
                return Err(
                    "generated read v8 logical relationship cut differs from original declaration",
                );
            } else if row["logical"]["source"] != root_record {
                unsupported("relationship-source-does-not-match-root")
            } else {
                let target_module = row["logical"]["target"]["module"]
                    .as_str()
                    .ok_or("generated read v8 target module absent")?;
                let target_record = row["logical"]["target"]["element"]
                    .as_str()
                    .ok_or("generated read v8 target Record absent")?;
                let target_path = record_path(target_module, target_record);
                let target_binding = request
                    .binding
                    .tables
                    .iter()
                    .filter(|table| {
                        table.logical == [target_module.to_owned(), target_record.to_owned()]
                    })
                    .count();
                if target_binding != 1 {
                    unsupported("target-record-not-bound")
                } else if let Some(target_algebra) = root_snapshot.get(&target_path) {
                    let mut enriched_root = root.clone();
                    enriched_root["__targetAlgebra"] = target_algebra.clone();
                    match eligible_descriptor(&request, &enriched_root, root, row) {
                        Ok(value) => value,
                        Err("generated join source is not this root") => {
                            unsupported("relationship-source-does-not-match-root")
                        }
                        Err("generated join self/alias relationship is outside profile") => {
                            unsupported("self-or-alias-relationship-outside-profile")
                        }
                        Err("generated join relationship is not required directed to-one") => {
                            unsupported("relationship-not-required-to-one")
                        }
                        Err("nullable-source-pair") => unsupported("nullable-source-foreign-key"),
                        Err("output-result-codec-outside-profile") => {
                            unsupported("output-result-codec-outside-profile")
                        }
                        Err(_) => {
                            return Err(
                                "generated read v8 admitted Relationship native correspondence differs",
                            );
                        }
                    }
                } else {
                    unsupported("target-root-algebra-unavailable")
                }
            };
            catalog_rows.insert(relationship_key.clone(), entry);
        }
        root["requiredInnerJoins"] = json!({"profile":"weft.generated-required-inner-join-catalog/1","relationships":catalog_rows});
        let _ = root_key;
    }
    output["version"] = json!("weft.generated-read-algebra/8");
    output["readiness"] = json!("weft.generated-read-executable/8");
    output["executorAbi"] = json!({"major":1,"minor":8});
    // Rebind only the inherited Page feedback slot over the complete `/8` root.
    for root in output["roots"].as_object_mut().unwrap().values_mut() {
        if root["ordinaryPage"]["status"] == "eligible" {
            let mut preimage = root.clone();
            preimage["ordinaryPage"]["continuation"]["fixed"]
                .as_object_mut()
                .ok_or("generated v8 ordinary Page digest slot absent")?
                .remove("issuedProgramDigest");
            let digest =
                weft_core::json::sha256(weft_core::mutation::canonical(&preimage).as_bytes());
            root["ordinaryPage"]["continuation"]["fixed"]["issuedProgramDigest"] = json!(digest);
        }
    }
    Ok(output)
}

pub(super) fn prepare_generated_read_algebra_v8_json(raw: &str) -> Result<String> {
    if raw.len() > 8_388_608 {
        return Err("generated read algebra request capacity exceeded");
    }
    let value =
        weft_core::json::checked_json(raw).map_err(|_| "generated read algebra JSON invalid")?;
    let binding_value = value
        .get("binding")
        .ok_or("generated read algebra binding absent")?;
    let digest = weft_core::json::sha256(weft_core::mutation::canonical(binding_value).as_bytes());
    let request: RequestV8 = serde_json::from_value(value)
        .map_err(|_| "generated read algebra v8 request shape differs")?;
    let result = build_v8(request, digest)?;
    let output = weft_core::mutation::canonical(&result);
    if output.len() > 8_388_608 {
        return Err("generated read algebra artifact capacity exceeded");
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> Value {
        serde_json::from_str(include_str!(
            "../../../tests/raw-postgresql/generated-read-v8/required-inner-join-request.json"
        ))
        .unwrap()
    }
    fn issue(value: &Value) -> Result<Value> {
        let raw = weft_core::mutation::canonical(value);
        let output = prepare_generated_read_algebra_v8_json(&raw)?;
        serde_json::from_str(&output).map_err(|_| "generated read v8 output JSON invalid")
    }

    #[test]
    fn v8_issues_one_owned_required_inner_join_and_preserves_v7_page_relay() {
        let input = request();
        let output = issue(&input).unwrap();
        assert_eq!(output["version"], "weft.generated-read-algebra/8");
        assert_eq!(output["executorAbi"], json!({"major":1,"minor":8}));
        let task = &output["roots"]["demo/Task"];
        let descriptor = &task["requiredInnerJoins"]["relationships"]["demo/TaskNote"];
        if std::env::var_os("WEFT_TEST_CAPTURE_GENERATED_READ_V8_JOIN").is_some() {
            eprintln!(
                "WEFT_GENERATED_READ_V8_JOIN_JSON={}",
                weft_core::mutation::canonical(descriptor)
            );
            eprintln!(
                "WEFT_GENERATED_READ_V8_ALGEBRA_JSON={}",
                weft_core::mutation::canonical(&output)
            );
        }
        assert_eq!(
            descriptor["profile"],
            "weft.generated-required-inner-join/1"
        );
        assert_eq!(
            descriptor["relationship"]["targetMultiplicity"],
            json!({"min":1,"max":1})
        );
        assert_eq!(descriptor["aliases"], json!({"root":"r","target":"t"}));
        assert!(descriptor["joinSql"].as_str().unwrap().contains(
            "\"r\".\"task_id\" COLLATE pg_catalog.\"C\" = \"t\".\"id\" COLLATE pg_catalog.\"C\""
        ));
        assert_eq!(
            descriptor["fields"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|field| field["owner"] == "root")
                .count(),
            3
        );
        assert_eq!(
            descriptor["fields"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|field| field["owner"] == "target")
                .count(),
            5
        );
        assert_eq!(descriptor["selection"]["emptyAllowed"], true);
        assert_eq!(descriptor["selection"]["emptyPublicTuple"], json!([]));
        assert_eq!(descriptor["statement"]["projectionSlot"]["emptyValue"], "");
        assert_eq!(descriptor["statement"]["terminal"], "all");
        assert_eq!(descriptor["statement"]["ordinalOutput"]["oid"], 20);
        assert_eq!(descriptor["statement"]["serverByteGuard"], true);
        assert_eq!(
            descriptor["statement"]["byteGuard"]["maxNativeBytes"],
            8_388_608
        );
        assert!(
            descriptor["fields"]
                .as_array()
                .unwrap()
                .iter()
                .all(|field| field["output"]["valueCodec"]["profile"]
                    == "weft.query-codec-instructions/1")
        );
        assert_eq!(task["ordinaryPage"]["status"], "eligible");
        assert_eq!(task["forwardConnectionSupport"]["status"], "eligible");
        let mut page_preimage = task.clone();
        page_preimage["ordinaryPage"]["continuation"]["fixed"]
            .as_object_mut()
            .unwrap()
            .remove("issuedProgramDigest");
        assert_eq!(
            task["ordinaryPage"]["continuation"]["fixed"]["issuedProgramDigest"],
            weft_core::json::sha256(weft_core::mutation::canonical(&page_preimage).as_bytes())
        );
        let note =
            &output["roots"]["demo/Note"]["requiredInnerJoins"]["relationships"]["demo/TaskNote"];
        assert_eq!(note["status"], "unsupported");
        assert_eq!(note["reason"], "relationship-source-does-not-match-root");
    }

    #[test]
    fn v8_resolves_same_named_fields_by_record_owner() {
        let raw = include_str!(
            "../../../tests/raw-postgresql/generated-read-v8/same-name-fields-request.json"
        );
        let value: Value = serde_json::from_str(raw).unwrap();
        let output = issue(&value).unwrap();
        if std::env::var_os("WEFT_TEST_CAPTURE_GENERATED_READ_V8_SAME_NAME").is_some() {
            eprintln!(
                "WEFT_GENERATED_READ_V8_SAME_NAME_JSON={}",
                weft_core::mutation::canonical(&output)
            );
        }
        let descriptor =
            &output["roots"]["demo/Task"]["requiredInnerJoins"]["relationships"]["demo/TaskNote"];
        assert_eq!(
            descriptor["profile"],
            "weft.generated-required-inner-join/1"
        );
        let fields = descriptor["fields"].as_array().unwrap();
        for (owner, field_id, source_id) in [
            ("root", "demo/taskOwner", "taskOwner"),
            ("target", "demo/owner", "owner"),
            ("root", "demo/taskValue", "taskValue"),
            ("target", "demo/payload", "payload"),
        ] {
            let slot = fields
                .iter()
                .find(|field| field["owner"] == owner && field["fieldId"] == field_id)
                .unwrap();
            assert_eq!(slot["sourceField"]["element"], source_id);
        }
        assert_ne!(
            fields
                .iter()
                .find(|f| f["owner"] == "root" && f["sourceField"]["element"] == "taskValue")
                .unwrap()["ordinal"],
            fields
                .iter()
                .find(|f| f["owner"] == "target" && f["sourceField"]["element"] == "payload")
                .unwrap()["ordinal"]
        );
    }

    #[test]
    fn result_only_codecs_cover_uuid_bits128_date_and_utc_timestamp() {
        let mut request_value = request();
        let mut model: Value =
            serde_json::from_str(request_value["binding"]["modelJson"].as_str().unwrap()).unwrap();
        let task_value_index = model["modules"][0]["elements"]
            .as_array()
            .unwrap()
            .iter()
            .position(|field| field["id"] == "taskValue")
            .unwrap();
        let source_ref = json!({"module":"demo","element":"taskValue"});
        let cases = [
            ("uuid", "uuid", 2950, "weft.postgresql18.uuid/1"),
            ("bits128", "bytea", 17, "weft.postgresql18.bytea-16/1"),
            ("date", "date", 1082, "weft.postgresql18.date/1"),
            (
                "timestamp",
                "timestamptz",
                1184,
                "weft.postgresql18.utc-timestamp/1",
            ),
        ];
        for (carrier, native_kind, oid, decoder) in cases {
            model["modules"][0]["elements"][task_value_index]["scalarType"] = json!(carrier);
            request_value["binding"]["modelJson"] = json!(weft_core::mutation::canonical(&model));
            let request: RequestV8 = serde_json::from_value(request_value.clone()).unwrap();
            let output = json!({"carrier":carrier,"nullable":false,"nativeType":{"kind":native_kind},"oid":oid});
            let codec = issue_output_value_codec(&request, &source_ref, &output).unwrap();
            assert_eq!(codec["profile"], "weft.query-result-codec/1");
            assert_eq!(codec["decoder"]["profile"], decoder);
            assert_eq!(codec["nativeType"]["kind"], native_kind);
            assert_eq!(codec["oid"], oid);
            assert_eq!(codec["refinements"]["allowedValues"], json!([]));
        }
        model["modules"][0]["elements"][task_value_index]["scalarType"] = json!("uuid");
        model["modules"][0]["elements"][task_value_index]["facets"] = json!({"precision":4});
        request_value["binding"]["modelJson"] = json!(weft_core::mutation::canonical(&model));
        let request: RequestV8 = serde_json::from_value(request_value).unwrap();
        let output =
            json!({"carrier":"uuid","nullable":false,"nativeType":{"kind":"uuid"},"oid":2950});
        assert_eq!(
            issue_output_value_codec(&request, &source_ref, &output).unwrap_err(),
            "output-result-codec-outside-profile"
        );
    }

    #[test]
    fn v8_refuses_nullable_source_pairs_without_disabling_base_page_or_relay() {
        let raw = include_str!(
            "../../../tests/raw-postgresql/generated-read-v8/nullable-source-request.json"
        );
        let value: Value = serde_json::from_str(raw).unwrap();
        let output = issue(&value).unwrap();
        let task = &output["roots"]["demo/Task"];
        let descriptor = &task["requiredInnerJoins"]["relationships"]["demo/TaskNote"];
        assert_eq!(descriptor["status"], "unsupported");
        assert_eq!(descriptor["reason"], "nullable-source-foreign-key");
        assert_eq!(task["ordinaryPage"]["status"], "eligible");
        assert_eq!(task["forwardConnectionSupport"]["status"], "eligible");
    }

    #[test]
    fn v8_escapes_percent_in_issued_native_identifiers() {
        assert_eq!(ident("table%name").unwrap(), "\"table%%name\"");
    }

    #[test]
    fn v8_rejects_tampered_native_or_relationship_cut_before_emitting_join() {
        let original = request();
        let mut changed = original.clone();
        changed["nativeSource3"]["relationships"][0]["native"]["foreignKey"]["name"] =
            json!("other_fk");
        assert!(issue(&changed).is_err());
        let mut changed = original.clone();
        changed["nativeSource3"]["relationships"][0]["logical"]["targetMultiplicity"]["min"] =
            json!(0);
        assert!(issue(&changed).is_err());
        let mut changed = original.clone();
        changed["nativeSource3"]["relationships"][0]["binding"]["pairs"][0]["sourceColumn"]["columnName"] =
            json!("other_id");
        assert!(issue(&changed).is_err());
        let mut changed = original;
        changed["binding"]["relationships"][0]["pairs"][0][0] = json!("taskOwner");
        assert!(issue(&changed).is_err());
    }
}
