// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded, keyless Unity Catalog current-read compilation.
//!
//! This module accepts source identities and expected native metadata only.
//! It grants no permission and performs no connection, catalog, or warehouse I/O.
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use weft_core::raw_read;

type Result<T> = std::result::Result<T, &'static str>;
const REQUEST_VERSION: &str = "weft.unity-current-read-request/0.1";
const RESPONSE_VERSION: &str = "weft.unity-current-selection/0.2";
const CORE_VERSION: &str = "0.10.0";
const COMPILER_VERSION: &str = "weft-databricks-unity-current-read/0.1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SourcePacket {
    version: String,
    utf8: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct FieldRef {
    module: String,
    element: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct TargetColumn {
    field_ref: FieldRef,
    native_name: String,
    native_type: String,
    nullable: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ExpectedTarget {
    kind: String,
    warehouse_id: String,
    catalog: String,
    schema: String,
    table: String,
    table_uuid: String,
    runtime_version: String,
    columns: Vec<TargetColumn>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ReadSpec {
    kind: String,
    table_ref: FieldRef,
    projection: Vec<FieldRef>,
    predicate: Predicate,
    row_cap: u16,
    frame_cap_bytes: u32,
    deadline_ms: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Predicate {
    kind: String,
    field_ref: FieldRef,
    parameter: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Parameter {
    name: String,
    carrier: String,
    nullable: bool,
    max_utf8_bytes: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Request {
    version: String,
    source: SourcePacket,
    expected_target: ExpectedTarget,
    read: ReadSpec,
    parameters: Vec<Parameter>,
}

fn exact(value: &Value, required: &[&str], optional: &[&str]) -> Result<()> {
    let object = value.as_object().ok_or("closed JSON object required")?;
    if required.iter().any(|key| !object.contains_key(*key))
        || object
            .keys()
            .any(|key| !required.contains(&key.as_str()) && !optional.contains(&key.as_str()))
    {
        return Err("unknown or missing selected request member");
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= 255 && !value.contains('\0')
}

fn quote_logical(value: &str) -> Result<String> {
    if !valid_identifier(value) {
        return Err("bounded source identifier required");
    }
    Ok(format!("\"{}\"", value.replace('"', "\"\"")))
}

fn ref_value(reference: &FieldRef) -> Value {
    json!({"module": reference.module, "element": reference.element})
}

fn locate<'a>(model: &'a Value, reference: &FieldRef) -> Result<(&'a Value, &'a Value)> {
    let modules = model["modules"].as_array().ok_or("source modules required")?;
    let matches = modules
        .iter()
        .filter(|module| module["id"] == reference.module)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err("source module absent or ambiguous");
    }
    let module = matches[0];
    let elements = module["elements"].as_array().ok_or("source elements required")?;
    let matches = elements
        .iter()
        .filter(|element| element["id"] == reference.element)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err("source element absent or ambiguous");
    }
    Ok((module, matches[0]))
}

fn compile(raw: &str) -> Result<Value> {
    let value = weft_core::json::checked_json(raw).map_err(|_| "invalid or duplicate JSON")?;
    exact(
        &value,
        &["version", "source", "expectedTarget", "read", "parameters"],
        &[],
    )?;
    if value.get("continuation").is_some() {
        return Err("continuation is outside finite current-read/0.1");
    }
    let request: Request = serde_json::from_value(value.clone())
        .map_err(|_| "closed current-read request required")?;
    if request.version != REQUEST_VERSION
        || request.source.version != CORE_VERSION
        || request.source.sha256.len() != 64
        || !request.source.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        || weft_core::json::sha256(request.source.utf8.as_bytes()) != request.source.sha256
    {
        return Err("source/request version or digest differs");
    }
    let model = weft_core::json::checked_json(&request.source.utf8)
        .map_err(|_| "invalid or duplicate UMF source JSON")?;
    if model["umf"] != CORE_VERSION
        || model["id"].as_str().is_none_or(str::is_empty)
        || model["modules"].as_array().is_none()
    {
        return Err("UMF source core document required");
    }

    let target = &request.expected_target;
    if target.kind != "unity-catalog"
        || [
            &target.warehouse_id,
            &target.catalog,
            &target.schema,
            &target.table,
            &target.table_uuid,
            &target.runtime_version,
        ]
        .iter()
        .any(|value| !valid_identifier(value))
    {
        return Err("bounded expected Unity Catalog target required");
    }

    let read = &request.read;
    if read.kind != "finite-current"
        || read.predicate.kind != "eq"
        || read.predicate.parameter != "p1"
        || read.row_cap == 0
        || read.row_cap > 1000
        || read.frame_cap_bytes == 0
        || read.frame_cap_bytes > 1_048_576
        || read.deadline_ms == 0
        || read.deadline_ms > 30_000
        || read.projection.is_empty()
        || read.projection.len() > 8
    {
        return Err("finite-current read outside bounded profile");
    }
    if request.parameters.len() != 1
        || request.parameters[0].name != "p1"
        || request.parameters[0].carrier != "string"
        || request.parameters[0].nullable
        || request.parameters[0].max_utf8_bytes != 4096
    {
        return Err("exact non-null bounded STRING parameter p1 required");
    }

    let (table_module, record) = locate(&model, &read.table_ref)?;
    if record["kind"] != "record"
        || record["name"].as_str().is_none_or(str::is_empty)
        || record["extensions"].as_object().is_none_or(|extensions| !extensions.is_empty())
    {
        return Err("selected source Record outside finite-read profile");
    }
    let members = record["members"].as_array().ok_or("Record members required")?;
    let mut source_fields = BTreeMap::<String, String>::new();
    let mut source_refs = BTreeSet::new();
    for member in members {
        let module_id = member["module"].as_str().ok_or("Field module required")?;
        let element_id = member["element"].as_str().ok_or("Field element required")?;
        let reference = FieldRef {
            module: module_id.to_owned(),
            element: element_id.to_owned(),
        };
        let (field_module, field) = locate(&model, &reference)?;
        if field["kind"] != "field" {
            return Err("Record member is not a Field");
        }
        let key = format!("{module_id}\0{element_id}");
        if !source_refs.insert(key.clone()) {
            return Err("duplicate source Field identity");
        }
        let name = field["name"].as_str().ok_or("selected Field name required")?;
        if field["scalarType"] != "string"
            || field["cardinality"] != "one"
            || field["nullability"] != "required"
            || field["extensions"].as_object().is_none_or(|extensions| !extensions.is_empty())
        {
            continue;
        }
        if field_module["id"] != module_id || source_fields.insert(key, name.to_owned()).is_some() {
            return Err("Field owner differs from selected source reference");
        }
    }

    let mut columns = BTreeMap::<String, &TargetColumn>::new();
    for column in &target.columns {
        if column.nullable
            || column.native_type != "STRING"
            || !valid_identifier(&column.native_name)
        {
            return Err("only non-null native STRING columns are admitted");
        }
        let key = format!("{}\0{}", column.field_ref.module, column.field_ref.element);
        if !source_fields.contains_key(&key) || columns.insert(key, column).is_some() {
            return Err("target column does not uniquely bind a selected source Field");
        }
    }
    if table_module["id"] != read.table_ref.module {
        return Err("table selector owner differs");
    }
    let mut projected = BTreeSet::new();
    for field_ref in &read.projection {
        let key = format!("{}\0{}", field_ref.module, field_ref.element);
        if !source_fields.contains_key(&key)
            || !columns.contains_key(&key)
            || !projected.insert(key)
        {
            return Err("projection Field is absent, ambiguous, or unbound");
        }
    }
    let predicate_key = format!(
        "{}\0{}",
        read.predicate.field_ref.module, read.predicate.field_ref.element
    );
    if !source_fields.contains_key(&predicate_key) || !columns.contains_key(&predicate_key) {
        return Err("predicate Field is absent or unbound");
    }
    let mut required_columns = projected.clone();
    required_columns.insert(predicate_key.clone());
    if columns.keys().cloned().collect::<BTreeSet<_>>() != required_columns {
        return Err("expected target columns must exactly cover projected and predicate Fields");
    }

    let record_name = record["name"].as_str().unwrap();
    let query_relation = format!(
        "{}.{}",
        quote_logical(&read.table_ref.module)?,
        quote_logical(record_name)?
    );
    let alias = "r";
    let mut projection = Vec::with_capacity(read.projection.len());
    for (ordinal, reference) in read.projection.iter().enumerate() {
        let key = format!("{}\0{}", reference.module, reference.element);
        let logical = source_fields.get(&key).ok_or("projection Field missing")?;
        let logical_q = quote_logical(logical)?;
        projection.push(format!("{alias}.{logical_q} AS __weft_out_{ordinal}"));
    }
    let predicate_source = source_fields
        .get(&predicate_key)
        .ok_or("predicate source Field missing")?;
    let predicate_column = format!("{alias}.{}", quote_logical(predicate_source)?);
    let sql = format!(
        "SELECT {} FROM {query_relation} AS {alias} WHERE {predicate_column} = :p1",
        projection.join(", ")
    );

    let raw_columns = target
        .columns
        .iter()
        .map(|column| {
            let key = format!("{}\0{}", column.field_ref.module, column.field_ref.element);
            let logical = source_fields.get(&key).ok_or("target source Field missing")?;
            Ok(json!({
                "logical": logical,
                "native": column.native_name,
                "carrier": "string",
                "nullable": false
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    let internal = json!({
        "version":"weft.raw-read/0.1",
        "target":"unity-catalog-current",
        "sql":sql,
        "parameters":[{"name":"p1","carrier":"string","nullable":false}],
        "rowCap":read.row_cap,
        "keyset":false,
        "binding":{
            "version":"umf.raw-read-binding/0.1",
            "schema":target.schema,
            "tables":[{
                "logical":[read.table_ref.module,record_name],
                "native":target.table,
                "columns":raw_columns,
                "key":[]
            }],
            "relationships":[],
            "modelSha256":request.source.sha256,
            "modelJson":request.source.utf8
        }
    });
    let admitted = raw_read::admit_finite_current_keyless_json(&internal.to_string())
        .map_err(|_| "derived finite current selection failed shared source admission")?;
    let lowered = super::raw_read::lower_current_finite_keyless_shaped(&admitted, &target.catalog)
        .map_err(|_| "derived finite current selection failed shared lowering")?;
    if lowered["status"] != "compiled"
        || lowered["parameterOrder"] != json!(["p1"])
        || lowered["outputs"].as_array().is_none_or(|outputs| {
            outputs.len() != read.projection.len()
                || outputs.iter().enumerate().any(|(ordinal, output)| {
                    output["name"] != format!("__weft_out_{ordinal}")
                        || output["carrier"] != "string"
                        || output["nullable"] != false
                })
        })
        || lowered["referencedTables"].as_array().is_none_or(|tables| {
            tables.len() != 1
                || tables[0]["native"] != target.table
                || tables[0]["logical"] != json!([read.table_ref.module, record["name"]])
        })
        || lowered["snapshotTables"] != json!([])
        || lowered["nativeChecks"] != json!([])
        || lowered["cursorParameters"] != json!([])
        || lowered["cursorOutputs"] != json!([])
        || lowered["cursorOrdering"] != json!([])
    {
        return Err("shared lowering output differs from finite current request");
    }

    let outputs = read
        .projection
        .iter()
        .map(|field| {
            let key = format!("{}\0{}", field.module, field.element);
            let column = columns.get(&key).ok_or("projection target column missing")?;
            Ok(json!({
                "fieldRef":ref_value(field),
                "nativeName":column.native_name,
                "carrier":"string",
                "nullable":false
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut output = json!({
        "version":RESPONSE_VERSION,
        "status":"compiled",
        "sourceSha256":request.source.sha256,
        "expectedTargetDigest":weft_core::json::sha256(weft_core::mutation::canonical(&value["expectedTarget"]).as_bytes()),
        "readDigest":weft_core::json::sha256(weft_core::mutation::canonical(&value["read"]).as_bytes()),
        "compilerVersion":COMPILER_VERSION,
        "sql":lowered["sql"],
        "sqlSha256":weft_core::json::sha256(lowered["sql"].as_str().unwrap_or_default().as_bytes()),
        "parameterOrder":["p1"],
        "outputs":outputs,
        "rowCap":read.row_cap,
        "frameCapBytes":read.frame_cap_bytes,
        "deadlineMs":read.deadline_ms,
        "expectedTarget":value["expectedTarget"],
        "obligations":{
            "transaction":"current-read",
            "principal":"native-person",
            "tableIdentity":"native-current-uuid",
            "nativeAdmissionEachOperation":true,
            "continuation":false,
            "temporalQualification":"held"
        }
    });
    let digest = weft_core::json::sha256(weft_core::mutation::canonical(&output).as_bytes());
    output["structuralDigest"] = json!({"owner":"weft-databricks-unity-current-read","version":"0.2","sha256":digest});
    Ok(output)
}

/// Compile a source-pinned finite Unity Catalog current-read request.
/// This function never connects to a warehouse or establishes authorization.
fn compile_v01_json(raw: &str) -> String {
    match compile(raw) {
        Ok(value) => value.to_string(),
        Err(reason) => json!({
            "version":RESPONSE_VERSION,
            "status":"blocked",
            "diagnostics":[{"code":"WFT-UNITY-CURRENT-READ","message":reason}]
        })
        .to_string(),
    }
}

/// Dispatch the exact checked request version to its owner while preserving the 0.1 route.
pub fn compile_json(raw: &str) -> String {
    let version = weft_core::json::checked_json(raw)
        .ok()
        .and_then(|value| value.get("version").and_then(Value::as_str).map(str::to_owned));
    if version.as_deref() == Some("weft.unity-current-read-request/0.2") {
        super::unity_current_predicate::compile_json(raw)
    } else if version.as_deref() == Some("weft.unity-current-read-request/0.3") {
        super::unity_current_summary::compile_json(raw)
    } else if version.as_deref() == Some("weft.unity-current-read-request/0.4") {
        super::unity_current_sum::compile_json(raw)
    } else if version.as_deref() == Some("weft.unity-current-read-request/0.5") {
        super::unity_current_avg::compile_json(raw)
    } else {
        compile_v01_json(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn source() -> Value {
        json!({
            "umf":"0.10.0","id":"finite-read-fixture",
            "vocabularies":{},
            "modules":[{"id":"sales","namespace":"sales","elements":[
                {"id":"Customer","name":"Customer","kind":"record","members":[
                    {"module":"sales","element":"customer_id"},
                    {"module":"sales","element":"display_name"}
                ],"extensions":{}},
                {"id":"customer_id","name":"customer_id","kind":"field","scalarType":"string","cardinality":"one","nullability":"required","extensions":{}},
                {"id":"display_name","name":"display_name","kind":"field","scalarType":"string","cardinality":"one","nullability":"required","extensions":{}}
            ],"extensions":{}}]
        })
    }

    fn request() -> Value {
        let source = source();
        let source_utf8 = source.to_string();
        json!({
            "version":REQUEST_VERSION,
            "source":{"version":CORE_VERSION,"utf8":source_utf8,"sha256":weft_core::json::sha256(source_utf8.as_bytes())},
            "expectedTarget":{"kind":"unity-catalog","warehouseId":"warehouse-1","catalog":"main","schema":"sales","table":"customers","tableUuid":"table-uuid-1","runtimeVersion":"16.4","columns":[
                {"fieldRef":{"module":"sales","element":"customer_id"},"nativeName":"customer_id","nativeType":"STRING","nullable":false},
                {"fieldRef":{"module":"sales","element":"display_name"},"nativeName":"display_name","nativeType":"STRING","nullable":false}
            ]},
            "read":{"kind":"finite-current","tableRef":{"module":"sales","element":"Customer"},"projection":[{"module":"sales","element":"display_name"}],"predicate":{"kind":"eq","fieldRef":{"module":"sales","element":"customer_id"},"parameter":"p1"},"rowCap":10,"frameCapBytes":65536,"deadlineMs":5000},
            "parameters":[{"name":"p1","carrier":"string","nullable":false,"maxUtf8Bytes":4096}]
        })
    }

    fn summary_request(summary: Value, columns: Value, predicate: Value) -> Value {
        let mut value = request();
        value["version"] = json!("weft.unity-current-read-request/0.3");
        value["expectedTarget"]["columns"] = columns;
        value["read"] = json!({
            "kind":"finite-summary",
            "tableRef":{"module":"sales","element":"Customer"},
            "predicate":predicate,
            "summary":summary,
            "frameCapBytes":65536,
            "deadlineMs":5000
        });
        value["parameters"] = json!([]);
        value
    }

    fn summary_column(element: &str, native: &str) -> Value {
        json!({"fieldRef":{"module":"sales","element":element},"nativeName":native,"nativeType":"STRING","nullable":false})
    }

    #[test]
    fn public_owner_route_compiles_unfiltered_global_count_with_empty_columnset() {
        let input = summary_request(json!({"kind":"aggregate","groups":[],"measures":[{"id":"rows","kind":"count-rows"}],"order":[],"groupCap":1}), json!([]), Value::Null);
        let output: Value = serde_json::from_str(&compile_json(&input.to_string())).unwrap();
        assert_eq!(output["version"], "weft.unity-current-selection/0.4");
        assert_eq!(output["status"], "compiled");
        assert_eq!(output["rowCap"], 1);
        assert_eq!(output["summary"], json!({"kind":"global","groupOutputIds":[],"measureOutputIds":["rows"],"ordering":[],"fetchRows":1}));
        assert_eq!(output["outputs"][0]["function"], "count-rows");
        assert!(output["sql"].as_str().unwrap().contains("COUNT(*)"));
        assert!(!output["sql"].as_str().unwrap().contains("LIMIT 1)"));
    }

    #[test]
    fn public_owner_route_compiles_exists_and_binary_distinct_grouped_count() {
        let exists = summary_request(json!({"kind":"exists"}), json!([]), Value::Null);
        let exists_out: Value = serde_json::from_str(&compile_json(&exists.to_string())).unwrap();
        assert_eq!(exists_out["status"], "compiled");
        assert_eq!(exists_out["outputs"], json!([{"id":"exists","name":"__weft_out_0","role":"exists","carrier":"boolean","nullable":false,"decoder":{"kind":"native-boolean"}}]));
        assert!(exists_out["sql"].as_str().unwrap().contains("SELECT EXISTS"));

        let grouped = summary_request(json!({
            "kind":"aggregate",
            "groups":[{"id":"display","fieldRef":{"module":"sales","element":"display_name"}}],
            "measures":[{"id":"unique","kind":"count-distinct","fieldRef":{"module":"sales","element":"customer_id"}}],
            "order":[{"output":"unique","direction":"asc","nulls":"last"}],
            "groupCap":1000
        }), json!([summary_column("customer_id","customer_id"),summary_column("display_name","display_name")]), Value::Null);
        let grouped_out: Value = serde_json::from_str(&compile_json(&grouped.to_string())).unwrap();
        assert_eq!(grouped_out["status"], "compiled");
        assert_eq!(grouped_out["rowCap"], 1001);
        assert_eq!(grouped_out["outputs"][1]["fieldRef"], json!({"module":"sales","element":"customer_id"}));
        assert!(grouped_out["sql"].as_str().unwrap().contains("COUNT(DISTINCT"));
        assert!(grouped_out["sql"].as_str().unwrap().contains("COLLATE UTF8_BINARY"));
        assert!(grouped_out["sql"].as_str().unwrap().contains("ORDER BY COUNT(DISTINCT"));
    }

    #[test]
    fn summary_owner_refuses_unknown_measure_and_columnset_drift() {
        let mut bad = summary_request(json!({"kind":"aggregate","groups":[],"measures":[{"id":"rows","kind":"sum"}],"order":[],"groupCap":1}), json!([]), Value::Null);
        assert_eq!(serde_json::from_str::<Value>(&compile_json(&bad.to_string())).unwrap()["status"], "blocked");
        bad = summary_request(json!({"kind":"aggregate","groups":[],"measures":[{"id":"rows","kind":"count-rows"}],"order":[{"output":"rows","direction":"asc","nulls":"last"}],"groupCap":1}), json!([]), Value::Null);
        assert_eq!(serde_json::from_str::<Value>(&compile_json(&bad.to_string())).unwrap()["status"], "blocked");
        bad = summary_request(json!({"kind":"exists"}), json!([summary_column("customer_id","customer_id")]), Value::Null);
        assert_eq!(serde_json::from_str::<Value>(&compile_json(&bad.to_string())).unwrap()["status"], "blocked");
    }

    fn derived_keyless_raw_request() -> Value {
        let model = source();
        let model_json = model.to_string();
        json!({
            "version":"weft.raw-read/0.1",
            "target":"unity-catalog-current",
            "sql":"SELECT r.display_name AS display_name FROM sales.Customer AS r WHERE r.customer_id = :p1",
            "parameters":[{"name":"p1","carrier":"string","nullable":false}],
            "rowCap":10,
            "keyset":false,
            "binding":{
                "version":"umf.raw-read-binding/0.1",
                "schema":"sales",
                "tables":[{"logical":["sales","Customer"],"native":"customers","columns":[
                    {"logical":"customer_id","native":"customer_id","carrier":"string","nullable":false},
                    {"logical":"display_name","native":"display_name","carrier":"string","nullable":false}
                ],"key":[]}],
                "relationships":[],
                "modelSha256":weft_core::json::sha256(model_json.as_bytes()),
                "modelJson":model_json
            }
        })
    }

    #[test]
    fn finite_current_read_compiles_without_a_key_or_value() {
        let result: Value = serde_json::from_str(&compile_json(&request().to_string())).unwrap();
        assert_eq!(result["version"], RESPONSE_VERSION);
        assert_eq!(result["status"], "compiled");
        assert_eq!(result["parameterOrder"], json!(["p1"]));
        assert_eq!(result["outputs"].as_array().unwrap().len(), 1);
        assert!(result["sql"].as_str().unwrap().contains("LIMIT 10"));
        assert!(result.get("nativeChecks").is_none());
        assert!(!result["sql"].as_str().unwrap().contains("customer-42"));
        assert_eq!(result["obligations"]["continuation"], false);
    }

    #[test]
    fn selected_limits_unknown_fields_and_continuations_refuse() {
        let mut bad = request();
        bad["read"]["rowCap"] = json!(1001);
        assert_eq!(serde_json::from_str::<Value>(&compile_json(&bad.to_string())).unwrap()["status"], "blocked");
        let mut bad = request();
        bad["read"]["offset"] = json!(1);
        assert_eq!(serde_json::from_str::<Value>(&compile_json(&bad.to_string())).unwrap()["status"], "blocked");
        let mut bad = request();
        bad["continuation"] = Value::Null;
        assert_eq!(serde_json::from_str::<Value>(&compile_json(&bad.to_string())).unwrap()["status"], "blocked");
    }

    #[test]
    fn source_and_expected_native_shape_are_independent_pins() {
        let mut bad = request();
        bad["expectedTarget"]["columns"][0]["nativeType"] = json!("BIGINT");
        assert_eq!(serde_json::from_str::<Value>(&compile_json(&bad.to_string())).unwrap()["status"], "blocked");
        let mut bad = request();
        bad["expectedTarget"]["columns"][0]["nativeType"] = json!("string");
        assert_eq!(serde_json::from_str::<Value>(&compile_json(&bad.to_string())).unwrap()["status"], "blocked");
        let mut bad = request();
        bad["source"]["sha256"] = json!("0".repeat(64));
        assert_eq!(serde_json::from_str::<Value>(&compile_json(&bad.to_string())).unwrap()["status"], "blocked");
        let mut bad = request();
        bad["read"]["projection"][0]["element"] = json!("not-a-field");
        assert_eq!(serde_json::from_str::<Value>(&compile_json(&bad.to_string())).unwrap()["status"], "blocked");
    }

    #[test]
    fn physical_identifiers_are_quoted_and_values_never_enter_sql() {
        let mut request = request();
        request["expectedTarget"]["table"] = json!("customer`archive");
        let response: Value = serde_json::from_str(&compile_json(&request.to_string())).unwrap();
        assert_eq!(response["status"], "compiled");
        assert!(response["sql"].as_str().unwrap().contains("`customer``archive`"));
        assert!(!response["sql"].as_str().unwrap().contains("customer-42"));
        assert_eq!(response["parameterOrder"], json!(["p1"]));
    }

    #[test]
    fn duplicate_json_members_refuse() {
        let request = request();
        let text = request.to_string();
        let duplicate = text.replacen("\"rowCap\":10", "\"rowCap\":10,\"rowCap\":10", 1);
        assert_eq!(serde_json::from_str::<Value>(&compile_json(&duplicate)).unwrap()["status"], "blocked");
    }

    #[test]
    fn core_keyless_admission_is_separate_and_current_only() {
        let raw = derived_keyless_raw_request().to_string();
        assert!(raw_read::admit_json(&raw).is_err(), "ordinary raw-read still requires an authored Key");
        assert!(raw_read::admit_finite_current_keyless_json(&raw).is_ok());

        let mut bad: Value = serde_json::from_str(&raw).unwrap();
        bad["target"] = json!("spark-delta4");
        assert!(raw_read::admit_finite_current_keyless_json(&bad.to_string()).is_err());
        let mut bad: Value = serde_json::from_str(&raw).unwrap();
        bad["keyset"] = json!(true);
        assert!(raw_read::admit_finite_current_keyless_json(&bad.to_string()).is_err());
        let mut bad: Value = serde_json::from_str(&raw).unwrap();
        bad["snapshot"] = json!(true);
        assert!(raw_read::admit_finite_current_keyless_json(&bad.to_string()).is_err());
        let mut bad: Value = serde_json::from_str(&raw).unwrap();
        bad["continuation"] = json!(null);
        assert!(raw_read::admit_finite_current_keyless_json(&bad.to_string()).is_err());
        let mut bad: Value = serde_json::from_str(&raw).unwrap();
        bad["binding"]["nativeLifecycle"] = json!({
            "version":"umf.table-lifecycle/0.3",
            "module":"sales",
            "configurationDigest":"0".repeat(64),
            "deploymentDigest":"1".repeat(64)
        });
        assert!(raw_read::admit_finite_current_keyless_json(&bad.to_string()).is_err());
    }
}
