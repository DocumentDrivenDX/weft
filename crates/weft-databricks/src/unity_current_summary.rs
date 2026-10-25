// SPDX-License-Identifier: MIT OR Apache-2.0
//! Source-bound exact finite summaries for Unity Catalog current reads.
//! This additive request owner grants no host authority and performs no I/O.
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use super::unity_current_predicate;
use weft_core::raw_read;

type Result<T> = std::result::Result<T, &'static str>;
const CORE_VERSION: &str = "0.10.0";
const REQUEST_VERSION: &str = "weft.unity-current-read-request/0.3";
const RESPONSE_VERSION: &str = "weft.unity-current-selection/0.4";
const COMPILER_VERSION: &str = "weft-databricks-unity-current-read/0.3";
const LIMIT_SQL: usize = 65_536;

fn exact(v: &Value, required: &[&str], optional: &[&str]) -> Result<()> {
    unity_current_predicate::exact(v, required, optional)
}
fn key(module: &str, element: &str) -> String { format!("{module}\0{element}") }
fn ident(s: &str) -> Result<String> { crate::raw_read::ident(s) }
fn json_digest(value: &Value) -> String {
    weft_core::json::sha256(weft_core::mutation::canonical(value).as_bytes())
}

fn compile(raw: &str) -> Result<Value> {
    let request = weft_core::json::checked_json(raw).map_err(|_| "invalid or duplicate JSON")?;
    exact(&request, &["version", "source", "expectedTarget", "read", "parameters"], &[])?;
    exact(&request["source"], &["version", "utf8", "sha256"], &[])?;
    exact(&request["expectedTarget"], &["kind", "warehouseId", "catalog", "schema", "table", "tableUuid", "runtimeVersion", "columns"], &[])?;
    exact(&request["read"], &["kind", "tableRef", "predicate", "summary", "frameCapBytes", "deadlineMs"], &[])?;
    if request["version"] != REQUEST_VERSION || request["source"]["version"] != CORE_VERSION
        || request["source"]["sha256"].as_str().is_none_or(|d| d.len() != 64 || !d.bytes().all(|b| b.is_ascii_hexdigit()))
        || request["source"]["utf8"].as_str().is_none_or(|s| weft_core::json::sha256(s.as_bytes()) != request["source"]["sha256"])
    { return Err("source/request version or digest differs"); }
    let source_doc = weft_core::json::checked_json(request["source"]["utf8"].as_str().unwrap()).map_err(|_| "invalid or duplicate UMF source JSON")?;
    if source_doc["umf"] != CORE_VERSION || source_doc["modules"].as_array().is_none() { return Err("UMF core document required"); }
    let target = &request["expectedTarget"];
    if target["kind"] != "unity-catalog" { return Err("Unity Catalog target required"); }
    for field in ["warehouseId", "catalog", "schema", "table", "tableUuid", "runtimeVersion"] {
        let value = target[field].as_str().filter(|s| !s.is_empty() && s.len() <= 255 && !s.contains('\0')).ok_or("bounded target identity required")?;
        let _ = value;
    }
    let read = &request["read"];
    if read["kind"] != "finite-summary" || read["frameCapBytes"].as_u64().is_none_or(|n| n == 0 || n > 1_048_576)
        || read["deadlineMs"].as_u64().is_none_or(|n| n == 0 || n > 30_000)
    { return Err("finite-summary frame/deadline bound required"); }
    let source = unity_current_predicate::source_profile(&source_doc, &read["tableRef"])?;
    let summary = &read["summary"];
    let kind = summary["kind"].as_str().ok_or("summary kind required")?;
    let parameters = request["parameters"].as_array().ok_or("parameter array required")?;
    if parameters.len() > 8 { return Err("summary parameter descriptor bound exceeded"); }
    let mut param_map = BTreeMap::new();
    let mut internal_params = Vec::new();
    for p in parameters {
        let carrier = p["carrier"].as_str().ok_or("parameter carrier required")?;
        if !matches!(carrier, "string" | "boolean") { return Err("summary parameter carrier outside profile"); }
        if carrier == "string" { exact(p, &["name", "carrier", "nullable", "maxUtf8Bytes"], &[])?; }
        else { exact(p, &["name", "carrier", "nullable"], &[])?; }
        let name = p["name"].as_str().filter(|n| unity_current_predicate::valid_parameter_name(n)).ok_or("bounded parameter name required")?;
        let nullable = p["nullable"].as_bool().ok_or("parameter nullability required")?;
        let max = if carrier == "string" { Some(u16::try_from(p["maxUtf8Bytes"].as_u64().filter(|n| *n > 0 && *n <= 4096).ok_or("STRING parameter byte bound required")?).map_err(|_| "parameter bound")?) } else { None };
        if param_map.insert(name.to_owned(), (carrier.to_owned(), nullable, max)).is_some() { return Err("duplicate parameter name"); }
        internal_params.push(json!({"name":name,"carrier":carrier,"nullable":nullable}));
    }

    let mut used = Vec::new();
    let mut predicate_nodes = 0usize;
    let mut bound = BTreeSet::new();
    let predicate_sql = if read["predicate"].is_null() {
        if !parameters.is_empty() { return Err("unfiltered summaries require no parameters"); }
        None
    } else {
        Some(unity_current_predicate::validate_predicate(&source, &read["predicate"], 1, &mut predicate_nodes, &param_map, &mut used, &mut bound)?)
    };
    if used.iter().any(|name| !param_map.contains_key(name))
        || parameters.iter().any(|p| !used.iter().any(|name| p["name"] == *name))
    {
        return Err("summary predicate parameter coverage differs");
    }
    let (group_ids, measure_ids, order_spec, group_fields, measure_fields, group_cap, exists_mode) =
        validate_summary(&source, summary, &mut bound)?;
    if bound.len() > 9 { return Err("summary selected Field bound exceeded"); }

    let target_columns = target["columns"].as_array().ok_or("target ColumnSet required")?;
    if target_columns.len() != bound.len() { return Err("target ColumnSet must exactly bind predicate/group/distinct Fields"); }
    let mut columns = BTreeMap::<String, (String, String, bool, String)>::new();
    let mut native_names = BTreeSet::new();
    for column in target_columns {
        exact(column, &["fieldRef", "nativeName", "nativeType", "nullable"], &[])?;
        let (module, element) = unity_current_predicate::checked_ref(&column["fieldRef"])?;
        let identity = key(&module, &element);
        let (logical, carrier, nullable) = source.field(&column["fieldRef"])?;
        let native_type = if carrier == "string" { "STRING" } else { "BOOLEAN" };
        let native = column["nativeName"].as_str().filter(|s| !s.is_empty() && s.len() <= 255 && !s.contains('\0')).ok_or("bounded native column name required")?;
        if !bound.contains(&identity) || column["nativeType"] != native_type || column["nullable"] != nullable
            || !native_names.insert(native.to_owned()) || columns.insert(identity, (native.to_owned(), carrier, nullable, logical)).is_some()
        { return Err("target native ColumnSet differs from exact source Field closure"); }
    }
    let relation = format!("{}.{}", ident(&source.table_module)?, ident(&source.record_name)?);
    let mut projection = Vec::new();
    let mut group_exprs = Vec::new();
    let mut output_rows = Vec::new();
    let mut output_ord = 0usize;
    if exists_mode {
        output_rows.push(json!({"id":"exists","name":"__weft_out_0","role":"exists","carrier":"boolean","nullable":false,"decoder":{"kind":"native-boolean"}}));
    }
    for (group_id, field_ref, identity) in &group_fields {
        let (logical, carrier, nullable) = source.field(field_ref)?;
        let (native, _, _, _) = columns.get(identity).ok_or("group native ColumnSet binding absent")?;
        let alias = format!("__weft_out_{output_ord}"); output_ord += 1;
        projection.push(format!("r.{} AS {}", ident(&logical)?, ident(&alias)?));
        group_exprs.push(format!("r.{}", ident(&logical)?));
        let mut out = json!({"id":group_id,"name":alias,"role":"group","fieldRef":field_ref,"nativeName":native,"carrier":carrier,"nullable":nullable,"decoder":{"kind":if carrier == "string" {"native-string"} else {"native-boolean"}}});
        if carrier == "string" { out["collation"] = json!("UTF8_BINARY"); }
        output_rows.push(out);
    }
    for (measure_id, measure_kind, field_ref, identity) in &measure_fields {
        let alias = format!("__weft_out_{output_ord}"); output_ord += 1;
        let expression = match measure_kind.as_str() {
            "count-rows" => "COUNT(*)".to_owned(),
            "count-distinct" => {
                let rf = field_ref.as_ref().ok_or("distinct Field required")?;
                let (logical, carrier, _) = source.field(rf)?;
                let _ = columns.get(identity.as_ref().ok_or("distinct target binding required")?).ok_or("distinct target binding absent")?;
                let expr = format!("r.{}", ident(&logical)?);
                // The closed Weft AST carries the direct Field. Its exact
                // lowerer inserts UTF8_BINARY for STRING distinctness.
                let _ = carrier;
                format!("COUNT(DISTINCT {expr})")
            }
            _ => return Err("summary measure outside exact count union"),
        };
        projection.push(format!("{expression} AS {}", ident(&alias)?));
        let mut output = json!({"id":measure_id,"name":alias,"role":"measure","function":measure_kind,"carrier":"integer","nullable":false,"decoder":{"kind":"canonical-nonnegative-integer-text","nativeAggregateType":"BIGINT"}});
        if measure_kind == "count-distinct" {
            output["fieldRef"] = field_ref.clone().ok_or("distinct Field provenance required")?;
        }
        output_rows.push(output);
    }
    let row_cap = if group_fields.is_empty() { 1u16 } else { u16::try_from(group_cap + 1).map_err(|_| "group sentinel bound")? };
    let sql = if exists_mode {
        let where_sql = predicate_sql.map(|p| format!(" WHERE {p}")).unwrap_or_default();
        format!("SELECT EXISTS (SELECT 1 FROM {relation} AS r{where_sql} LIMIT 1) AS \"__weft_out_0\"")
    } else {
        let mut out = format!("SELECT {} FROM {relation} AS r", projection.join(", "));
        if let Some(p) = predicate_sql { out.push_str(&format!(" WHERE {p}")); }
        if !group_exprs.is_empty() { out.push_str(&format!(" GROUP BY {}", group_exprs.join(", "))); }
        if !order_spec.is_empty() {
            let id_to_alias = output_rows.iter().map(|row| (row["id"].as_str().unwrap().to_owned(), row["name"].as_str().unwrap().to_owned())).collect::<BTreeMap<_,_>>();
            let mut ordering = Vec::new(); let mut seen = BTreeSet::new();
            for order in &order_spec {
                let id = order["output"].as_str().ok_or("summary order output id required")?;
                if !seen.insert(id.to_owned()) { return Err("duplicate summary order output"); }
                let alias = id_to_alias.get(id).ok_or("summary order output absent")?;
                ordering.push(format!("{} {} NULLS {}", ident(alias)?, if order["direction"] == "desc" {"DESC"} else {"ASC"}, if order["nulls"] == "first" {"FIRST"} else {"LAST"}));
            }
            for (id, _, _) in &group_fields {
                if seen.insert(id.clone()) { let alias = id_to_alias.get(id).ok_or("group output absent")?; ordering.push(format!("{} ASC NULLS LAST", ident(alias)?)); }
            }
            out.push_str(&format!(" ORDER BY {}", ordering.join(", ")));
        } else if !group_fields.is_empty() {
            let id_to_alias = output_rows.iter().map(|row| (row["id"].as_str().unwrap().to_owned(), row["name"].as_str().unwrap().to_owned())).collect::<BTreeMap<_,_>>();
            out.push_str(&format!(" ORDER BY {}", group_fields.iter().map(|(id,_,_)| format!("{} ASC NULLS LAST", ident(id_to_alias[id].as_str()).unwrap())).collect::<Vec<_>>().join(", ")));
        }
        out
    };
    let sql = if exists_mode { sql } else { format!("{sql} LIMIT {row_cap}") };
    if sql.len() > LIMIT_SQL { return Err("final lowered SQL byte limit exceeded"); }
    let mut raw_columns = Vec::new();
    for (identity, (native, carrier, nullable, logical)) in &columns {
        let mut parts=identity.split('\0'); let module=parts.next().unwrap(); let element=parts.next().unwrap();
        let _ = (module, element);
        raw_columns.push(json!({"logical":logical,"native":native,"carrier":carrier,"nullable":nullable}));
    }
    let internal = json!({"version":"weft.raw-read/0.1","target":"unity-catalog-current","sql":sql,
        "parameters":internal_params,"rowCap":row_cap,"keyset":false,
        "binding":{"version":"umf.raw-read-binding/0.1","schema":target["schema"],"tables":[{"logical":[source.table_module,source.record_name],"native":target["table"],"columns":raw_columns,"key":[]}],"relationships":[],"modelSha256":request["source"]["sha256"],"modelJson":request["source"]["utf8"]}});
    let admitted = raw_read::admit_finite_current_summary_json(&internal.to_string()).map_err(|_| "derived summary request failed shared source admission")?;
    let lowered = super::raw_read::lower_current_finite_summary_shaped(&admitted, target["catalog"].as_str().unwrap()).map_err(|_| "exact summary lowering refused")?;
    let expected_lowered_columns = columns.iter().map(|(_, (native, carrier, nullable, logical))|
        json!({"logical":logical,"native":native,"carrier":carrier,"nullable":nullable})
    ).collect::<Vec<_>>();
    if lowered["status"] != "compiled" || lowered["version"] != RESPONSE_VERSION || !lowered_outputs_match(lowered.get("outputs").and_then(Value::as_array), &output_rows)
        || lowered["parameterOrder"] != json!(used)
        || lowered["referencedTables"].as_array().is_none_or(|a| a.len() != 1 || a[0]["native"] != target["table"])
        || lowered["referencedTables"][0]["logical"] != json!([source.table_module,source.record_name])
        || lowered["referencedTables"][0]["columns"] != json!(expected_lowered_columns)
        || lowered["nativeChecks"] != json!([]) || lowered["snapshotTables"] != json!([])
        || lowered["lockSql"] != json!([]) || lowered["cursorParameters"] != json!([])
        || lowered["cursorOutputs"] != json!([]) || lowered["cursorOrdering"] != json!([])
    { return Err("exact summary lowerer output shape differs"); }
    let final_sql = lowered["sql"].as_str().filter(|s| !s.is_empty()).ok_or("final SQL missing")?;
    if final_sql.len() > LIMIT_SQL { return Err("final lowered SQL byte limit exceeded"); }
    let (summary_kind, fetch_rows, cardinality) = if exists_mode { ("exists", 1u16, json!({"kind":"exact-one"})) }
        else if group_fields.is_empty() { ("global", 1u16, json!({"kind":"exact-one"})) }
        else { ("grouped", row_cap, json!({"kind":"group-overflow-lookahead","maximumGroups":group_cap,"fetchRows":row_cap})) };
    let ordering = normalized_order(&order_spec, &group_fields)?;
    let binary_collation = bound.iter().any(|identity| {
        let mut parts = identity.split('\0');
        let field = json!({"module":parts.next().unwrap_or(""),"element":parts.next().unwrap_or("")});
        source.field(&field).is_ok_and(|(_, carrier, _)| carrier == "string")
    });
    let compiled = json!({"version":RESPONSE_VERSION,"status":"compiled","sourceSha256":request["source"]["sha256"],
        "expectedTargetDigest":json_digest(target),"readDigest":json_digest(read),"compilerVersion":COMPILER_VERSION,
        "sql":final_sql,"sqlSha256":weft_core::json::sha256(final_sql.as_bytes()),"parameterOrder":lowered["parameterOrder"],
        "parameters":parameters,"outputs":output_rows,"rowCap":fetch_rows,"frameCapBytes":read["frameCapBytes"],"deadlineMs":read["deadlineMs"],
        "expectedTarget":target,"summary":{"kind":summary_kind,"groupOutputIds":group_ids,"measureOutputIds":measure_ids,"ordering":ordering,"fetchRows":fetch_rows},
        "obligations":{"transaction":"current-read","principal":"native-person","tableIdentity":"native-current-uuid","nativeAdmissionEachOperation":true,
          "continuation":false,"temporalQualification":"held","summaryProfile":"bounded-current-summary/0.1","binaryStringCollation":binary_collation,"resultCardinality":cardinality}});
    let digest = json_digest(&compiled);
    let mut output = compiled;
    output["structuralDigest"] = json!({"owner":"weft-databricks-unity-current-read","version":"0.4","sha256":digest});
    Ok(output)
}

fn lowered_outputs_match(actual: Option<&Vec<Value>>, expected: &[Value]) -> bool {
    let Some(actual) = actual else { return false; };
    if actual.len() != expected.len() { return false; }
    actual.iter().zip(expected).all(|(got, want)| {
        match want["role"].as_str() {
            Some("exists") => exact(got, &["name", "role", "carrier", "nullable", "decoder"], &[]).is_ok()
                && got["name"] == want["name"] && got["role"] == "exists" && got["carrier"] == "boolean"
                && got["nullable"] == false && got["decoder"] == want["decoder"],
            Some("group") => {
                let string = want["carrier"] == "string";
                exact(got, &["name", "role", "carrier", "nullable", "decoder"], if string { &["collation"] } else { &[] }).is_ok()
                    && got["name"] == want["name"] && got["role"] == "group"
                    && got["carrier"] == want["carrier"] && got["nullable"] == want["nullable"]
                    && got["decoder"] == want["decoder"]
                    && (!string || got["collation"] == "UTF8_BINARY")
            }
            Some("measure") => exact(got, &["name", "carrier", "nullable", "decoder"], &[]).is_ok()
                && got["name"] == want["name"] && got["carrier"] == "integer"
                && got["nullable"] == false && got["decoder"] == want["decoder"],
            _ => false,
        }
    })
}

fn validate_summary(source: &unity_current_predicate::Source, summary: &Value, bound: &mut BTreeSet<String>) -> Result<(Vec<String>, Vec<String>, Vec<Value>, Vec<(String, Value, String)>, Vec<(String, String, Option<Value>, Option<String>)>, u64, bool)> {
    match summary["kind"].as_str().ok_or("summary kind required")? {
        "exists" => { exact(summary, &["kind"], &[])?; Ok((vec![], vec![], vec![], vec![], vec![], 1, true)) }
        "aggregate" => {
            exact(summary, &["kind", "groups", "measures", "order", "groupCap"], &[])?;
            let groups = summary["groups"].as_array().filter(|a| a.len() <= 3).ok_or("summary group array required")?;
            let measures = summary["measures"].as_array().filter(|a| !a.is_empty()).ok_or("summary measure array required")?;
            let order = summary["order"].as_array().ok_or("summary ordering array required")?;
            let cap = summary["groupCap"].as_u64().ok_or("summary groupCap required")?;
            if (groups.is_empty() && cap != 1) || (!groups.is_empty() && !(1..=1000).contains(&cap)) { return Err("summary groupCap outside profile"); }
            if groups.is_empty() && !order.is_empty() { return Err("global summary ordering must be empty"); }
            let mut ids=BTreeSet::new(); let mut group_ids=Vec::new(); let mut measure_ids=Vec::new(); let mut group_fields=Vec::new(); let mut measure_fields=Vec::new(); let mut field_ids=BTreeSet::new();
            for g in groups { exact(g, &["id", "fieldRef"], &[])?; let id=g["id"].as_str().filter(|s| unity_current_predicate::valid_parameter_name(s)).ok_or("bounded group id required")?.to_owned(); if !ids.insert(id.clone()){return Err("duplicate summary output id");} let (module,element)=unity_current_predicate::checked_ref(&g["fieldRef"])?; let identity=key(&module,&element); let (_,carrier,_)=source.field(&g["fieldRef"])?; if !matches!(carrier.as_str(),"string"|"boolean") || !field_ids.insert(identity.clone()){return Err("duplicate or unsupported group Field");} bound.insert(identity.clone()); group_ids.push(id.clone()); group_fields.push((id,g["fieldRef"].clone(),identity)); }
            for m in measures { let kind=m["kind"].as_str().ok_or("measure kind required")?; let id=m["id"].as_str().filter(|s| unity_current_predicate::valid_parameter_name(s)).ok_or("bounded measure id required")?.to_owned(); if !ids.insert(id.clone()){return Err("duplicate summary output id");} match kind { "count-rows"=>{exact(m,&["id","kind"],&[])?; measure_fields.push((id.clone(),kind.to_owned(),None,None));}, "count-distinct"=>{exact(m,&["id","kind","fieldRef"],&[])?; let (module,element)=unity_current_predicate::checked_ref(&m["fieldRef"])?; let identity=key(&module,&element); let (_,carrier,_)=source.field(&m["fieldRef"])?; if !matches!(carrier.as_str(),"string"|"boolean"){return Err("distinct measure requires STRING/BOOLEAN Field");} bound.insert(identity.clone()); measure_fields.push((id.clone(),kind.to_owned(),Some(m["fieldRef"].clone()),Some(identity)));}, _=>return Err("measure kind outside exact count union") } measure_ids.push(id); }
            if group_fields.len()+measure_fields.len()>8{return Err("summary output bound exceeded");}
            for o in order { exact(o,&["output","direction","nulls"],&[])?; let id=o["output"].as_str().ok_or("order output required")?; if !ids.contains(id)||!matches!(o["direction"].as_str(),Some("asc"|"desc"))||!matches!(o["nulls"].as_str(),Some("first"|"last")){return Err("invalid summary order entry");} }
            let mut seen=BTreeSet::new(); if order.iter().any(|o| !seen.insert(o["output"].as_str().unwrap().to_owned())) { return Err("duplicate summary order output"); }
            Ok((group_ids,measure_ids,order.clone(),group_fields,measure_fields,cap,false))
        }
        _=>Err("summary kind outside closed union")
    }
}

fn normalized_order(order: &[Value], groups: &[(String, Value, String)]) -> Result<Vec<Value>> {
    let mut out=order.to_vec(); let mut seen=out.iter().filter_map(|v|v["output"].as_str().map(str::to_owned)).collect::<BTreeSet<_>>();
    for (id,_,_) in groups { if seen.insert(id.clone()) { out.push(json!({"output":id,"direction":"asc","nulls":"last"})); } }
    Ok(out)
}

pub(crate) fn compile_json(raw: &str) -> String {
    match compile(raw) { Ok(value)=>value.to_string(), Err(message)=>json!({"version":RESPONSE_VERSION,"status":"blocked","diagnostics":[{"code":"WFT-UNITY-CURRENT-SUMMARY","message":message}]}).to_string() }
}
