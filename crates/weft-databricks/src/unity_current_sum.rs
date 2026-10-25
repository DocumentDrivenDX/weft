// SPDX-License-Identifier: MIT OR Apache-2.0
//! Source-bound exact finite signed-limb SUM for Unity Catalog current reads.
//! This additive request owner grants no host authority and performs no I/O.
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use super::unity_current_predicate;
use weft_core::raw_read;

type Result<T> = std::result::Result<T, &'static str>;
const CORE_VERSION: &str = "0.10.0";
const REQUEST_VERSION: &str = "weft.unity-current-read-request/0.4";
const RESPONSE_VERSION: &str = "weft.unity-current-selection/0.5";
const COMPILER_VERSION: &str = "weft-databricks-unity-current-read/0.4";
const LIMIT_SQL: usize = 65_536;

fn exact(v: &Value, required: &[&str], optional: &[&str]) -> Result<()> {
    unity_current_predicate::exact(v, required, optional)
}
fn key(module: &str, element: &str) -> String { format!("{module}\0{element}") }
fn ident(s: &str) -> Result<String> { unity_current_predicate::logical_ident(s) }
fn json_digest(value: &Value) -> String {
    weft_core::json::sha256(weft_core::mutation::canonical(value).as_bytes())
}

fn compile(raw: &str) -> Result<Value> {
    let request = weft_core::json::checked_json(raw).map_err(|_| "invalid or duplicate JSON")?;
    exact(&request, &["version", "source", "expectedTarget", "nativeColumns", "read", "parameters"], &[])?;
    exact(&request["source"], &["version", "utf8", "sha256"], &[])?;
    exact(&request["expectedTarget"], &["kind", "warehouseId", "catalog", "schema", "table", "tableUuid", "runtimeVersion", "columns", "columnSetTarget"], &[])?;
    exact(&request["nativeColumns"], &["target", "sets"], &[])?;
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
    if read["kind"] != "finite-sum" || read["frameCapBytes"].as_u64().is_none_or(|n| n == 0 || n > 1_048_576)
        || read["deadlineMs"].as_u64().is_none_or(|n| n == 0 || n > 30_000)
    { return Err("finite-summary frame/deadline bound required"); }
    let source = unity_current_predicate::source_profile_for_sum(&source_doc, &read["tableRef"])?;
    let column_target = &target["columnSetTarget"];
    exact(column_target, &["platform", "majorVersion"], &[])?;
    if column_target["platform"] != "unity-catalog"
        || column_target["majorVersion"].as_str().is_none_or(|s| s.is_empty() || s.len() > 64)
        || request["nativeColumns"]["target"] != *column_target
        || request["nativeColumns"]["sets"].as_array().is_none_or(|sets| sets.len() != 1)
    { return Err("original Unity ColumnSet target/envelope differs"); }
    let summary = &read["summary"];
    let kind = summary["kind"].as_str().ok_or("summary kind required")?;
    let parameters = request["parameters"].as_array().ok_or("parameter array required")?;
    if parameters.len() > 8 { return Err("SUM parameter descriptor bound exceeded"); }
    let mut param_map = BTreeMap::new();
    let mut internal_params = Vec::new();
    for p in parameters {
        let carrier = p["carrier"].as_str().ok_or("parameter carrier required")?;
        if !matches!(carrier, "string" | "boolean") { return Err("SUM parameter carrier outside predicate profile"); }
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
        return Err("SUM predicate parameter coverage differs");
    }
    let (group_ids, measure_ids, order_spec, group_fields, measure_fields, group_cap, exists_mode) =
        validate_summary(&source, summary, &mut bound)?;
    if bound.len() > 39 { return Err("SUM selected Field bound exceeded"); }

    let target_columns = target["columns"].as_array().ok_or("target ColumnSet required")?;
    if target_columns.len() != bound.len() { return Err("target ColumnSet must exactly bind predicate/group/distinct Fields"); }
    let mut columns = BTreeMap::<String, (String, String, bool, String)>::new();
    let mut native_names = BTreeSet::new();
    for column in target_columns {
        exact(column, &["fieldRef", "nativeName", "nativeType", "nullable"], &["sourceFacets"])?;
        let (module, element) = unity_current_predicate::checked_ref(&column["fieldRef"])?;
        let identity = key(&module, &element);
        let (logical, carrier, nullable) = match source.field(&column["fieldRef"]) {
            Ok((logical, carrier, nullable)) => (logical, carrier, nullable),
            Err(_) => {
                let field = unity_current_predicate::sum_field(&source, &column["fieldRef"])?;
                (field.name.clone(), field.carrier.clone(), field.nullable)
            }
        };
        let native_type = match carrier.as_str() {
            "string" => "STRING".to_owned(), "boolean" => "BOOLEAN".to_owned(),
            "integer" => match column["sourceFacets"]["integerWidth"]["bits"].as_u64() {
                Some(8) => "TINYINT".to_owned(), Some(16) => "SMALLINT".to_owned(),
                Some(32) => "INT".to_owned(), Some(64) => "BIGINT".to_owned(),
                _ => return Err("SUM integer width facet missing or unsupported"),
            },
            "decimal" => {
                let p=column["sourceFacets"]["precision"].as_u64().filter(|p| (1..=38).contains(p)).ok_or("SUM decimal precision facet required")?;
                let s=column["sourceFacets"]["scale"].as_u64().filter(|s| *s<=p).ok_or("SUM decimal scale facet required")?;
                format!("DECIMAL({p},{s})")
            }, _ => return Err("target source carrier outside SUM profile"),
        };
        let native = column["nativeName"].as_str().filter(|s| !s.is_empty() && s.len() <= 255 && !s.contains('\0')).ok_or("bounded native column name required")?;
        let authored_native = column["nativeType"].as_str().ok_or("native type spelling required")?;
        let actual_type = weft_core::column_custody::native_type(authored_native).map_err(|_| "target native type tokenizer refused")?;
        let expected_type = weft_core::column_custody::native_type(&native_type).map_err(|_| "derived SUM native type invalid")?;
        let facets_match = match carrier.as_str() {
            "integer" => {
                let source_facets = &unity_current_predicate::sum_field(&source, &column["fieldRef"])?.facets;
                column.get("sourceFacets") == Some(source_facets)
                    && source_facets.as_object().is_some_and(|f| f.len()==1)
                    && source_facets["integerWidth"]["signed"] == true
                    && matches!(source_facets["integerWidth"]["bits"].as_u64(), Some(8|16|32|64))
            },
            "decimal" => {
                let source_facets = &unity_current_predicate::sum_field(&source, &column["fieldRef"])?.facets;
                column.get("sourceFacets") == Some(source_facets)
                    && source_facets.as_object().is_some_and(|f| f.len()==2)
                    && (1..=38).contains(&source_facets["precision"].as_u64().unwrap_or(0))
                    && source_facets["scale"].as_u64().is_some_and(|s| s <= source_facets["precision"].as_u64().unwrap_or(0))
            },
            _ => column.get("sourceFacets").is_none(),
        };
        if !facets_match || !bound.contains(&identity) || actual_type != expected_type || column["nullable"] != nullable
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
        if measure_kind != "sum" { return Err("finite SUM owner encountered non-SUM measure"); }
        let rf = field_ref.as_ref().ok_or("SUM Field provenance required")?;
        let source_field = unity_current_predicate::sum_field(&source, rf)?;
        let identity = identity.as_ref().ok_or("SUM Field identity required")?;
        let (native, carrier, nullable, logical) = columns.get(identity).ok_or("SUM target Field binding absent")?;
        let facets = &source_field.facets;
        if carrier != &source_field.carrier || logical != &source_field.name || nullable != &source_field.nullable {
            return Err("SUM source and target Field crosswalk differs");
        }
        projection.push(format!("SUM(r.{}) AS {}", ident(logical)?, ident(&alias)?));
        let (decoder, source_facets) = if carrier == "integer" {
            (json!({"kind":"canonical-signed-integer-text","maximumDigits":57}), facets.clone())
        } else {
            (json!({"kind":"canonical-fixed-scale-decimal-text","scale":facets["scale"],"maximumCoefficientDigits":57}), facets.clone())
        };
        let output = json!({"id":measure_id,"name":alias,"role":"measure","function":"sum","fieldRef":rf,
            "nativeName":native,"carrier":carrier,"nullable":true,"sourceFacets":source_facets,"decoder":decoder});
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
        "parameters":internal_params.clone(),"rowCap":row_cap,"keyset":false,
        "binding":{"version":"umf.raw-read-binding/0.1","schema":target["schema"],"tables":[{"logical":[source.table_module,source.record_name],"native":target["table"],"columns":raw_columns,"key":[]}],"relationships":[],"modelSha256":request["source"]["sha256"],"modelJson":request["source"]["utf8"],"nativeColumns":request["nativeColumns"]}});
    let admitted = raw_read::admit_finite_current_sum_json(&internal.to_string()).map_err(|error| {
        #[cfg(test)]
        eprintln!("finite-current SUM source admission rejection: {error}");
        "derived SUM request failed shared source admission"
    })?;
    verify_original_column_custody(&admitted, &source_doc, target, &request["nativeColumns"], &target_columns)?;
    let lowered = super::raw_read::lower_current_finite_sum_shaped(&admitted, target["catalog"].as_str().unwrap()).map_err(|_| "exact SUM lowering refused")?;
    let expected_lowered_columns = columns.iter().map(|(_, (native, carrier, nullable, logical))|
        json!({"logical":logical,"native":native,"carrier":carrier,"nullable":nullable})
    ).collect::<Vec<_>>();
    if lowered["status"] != "compiled" || lowered["version"] != RESPONSE_VERSION || !lowered_outputs_match(lowered.get("outputs").and_then(Value::as_array), &output_rows)
        || lowered["parameterOrder"] != json!(used)
        || lowered["parameters"] != json!(internal_params)
        || lowered["referencedTables"].as_array().is_none_or(|a| a.len() != 1 || a[0]["native"] != target["table"])
        || lowered["referencedTables"][0]["logical"] != json!([source.table_module,source.record_name])
        || lowered["referencedTables"][0]["columns"] != json!(expected_lowered_columns)
        || lowered["nativeChecks"] != json!([]) || lowered["snapshotTables"] != json!([])
        || lowered["lockSql"] != json!([]) || lowered["cursorParameters"] != json!([])
        || lowered["cursorOutputs"] != json!([]) || lowered["cursorOrdering"] != json!([])
    { return Err("exact SUM lowerer output shape differs"); }
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
        "expectedTarget":target,"nativeColumns":request["nativeColumns"],"summary":{"kind":summary_kind,"groupOutputIds":group_ids,"measureOutputIds":measure_ids,"ordering":ordering,"fetchRows":fetch_rows},
        "arithmetic":{"algorithm":"signed-coefficient-base10^19/1","maximumNonNullInputsPerMeasurePerGroup":"9223372036854775807","populationCheck":"exact-decimal-ones","rounding":"none","overflow":"WFT-CAPABILITY","result":"canonical-text","publication":"all-rows-after-complete-validation"},
        "obligations":{"transaction":"current-read","principal":"native-person","tableIdentity":"native-current-uuid","nativeAdmissionEachOperation":true,
          "continuation":false,"temporalQualification":"held","summaryProfile":"bounded-current-exact-sum/0.1","binaryStringCollation":binary_collation,"resultCardinality":cardinality}});
    let digest = json_digest(&compiled);
    let mut output = compiled;
    output["structuralDigest"] = json!({"owner":"weft-databricks-unity-current-read","version":"0.5","sha256":digest});
    Ok(output)
}

fn verify_original_column_custody(
    admitted: &raw_read::AdmittedRead,
    doc: &Value,
    target: &Value,
    envelope_value: &Value,
    selected: &[Value],
) -> Result<()> {
    let envelope: weft_core::column_custody::NativeColumnsEnvelope =
        serde_json::from_value(envelope_value.clone()).map_err(|_| "closed nativeColumns envelope required")?;
    if envelope.vector_extension.is_some() || envelope.sets.len() != 1 || envelope.target != target["columnSetTarget"] {
        return Err("finite SUM accepts exactly one original non-vector ColumnSet");
    }
    let binding = &admitted.request().binding;
    let table = binding.tables.first().ok_or("SUM table binding absent")?;
    let records = doc["modules"].as_array().ok_or("original model modules absent")?.iter()
        .filter(|m| m["id"] == table.logical[0])
        .flat_map(|m| m["elements"].as_array().into_iter().flatten())
        .filter(|e| e["kind"] == "record" && e["name"] == table.logical[1]).collect::<Vec<_>>();
    if records.len() != 1 { return Err("unique original Record required for ColumnSet custody"); }
    let record = json!({"module":table.logical[0],"element":records[0]["id"]});
    let original = weft_core::column_custody::admit_columns(
        &binding.model_sha256, doc, &target["columnSetTarget"], &envelope.sets[0], table, &record, CORE_VERSION,
    ).map_err(|_| "original ColumnSet/native binding custody refused")?;
    if records[0]["members"].as_array().is_none() { return Err("original Record member inventory absent"); }
    for selected_column in selected {
        let reference = &selected_column["fieldRef"];
        let key = weft_core::column_custody::field_key(reference);
        let (name, native_type, not_null) = original.get(&key).ok_or("selected Field absent from original ColumnSet")?;
        let expected_type = weft_core::column_custody::native_type(selected_column["nativeType"].as_str().ok_or("native type spelling required")?)
            .map_err(|_| "target native type spelling refused")?;
        if selected_column["nativeName"].as_str() != Some(name.as_str()) || expected_type != *native_type
            || weft_core::column_custody::row_null(doc, &record, reference, &original).map_err(|_| "original RowView nullability custody refused")? == *not_null
        { return Err("target selected column differs from exact original ColumnSet/RowView"); }
    }
    Ok(())
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
            Some("measure") => exact(got, &["name", "role", "function", "fieldRef", "nativeName", "carrier", "nullable", "sourceFacets", "decoder"], &[]).is_ok()
                && got["name"] == want["name"] && got["role"] == "measure" && got["function"] == "sum"
                && got["fieldRef"] == want["fieldRef"] && got["nativeName"] == want["nativeName"]
                && got["carrier"] == want["carrier"] && got["nullable"] == true
                && got["sourceFacets"] == want["sourceFacets"] && got["decoder"] == want["decoder"],
            _ => false,
        }
    })
}

fn validate_summary(source: &unity_current_predicate::Source, summary: &Value, bound: &mut BTreeSet<String>) -> Result<(Vec<String>, Vec<String>, Vec<Value>, Vec<(String, Value, String)>, Vec<(String, String, Option<Value>, Option<String>)>, u64, bool)> {
    exact(summary, &["kind", "groups", "measures", "order", "groupCap"], &[])?;
    if summary["kind"] != "aggregate" { return Err("finite SUM requires aggregate summary"); }
    let groups = summary["groups"].as_array().filter(|a| a.len() <= 3).ok_or("SUM group array required")?;
    let measures = summary["measures"].as_array().filter(|a| !a.is_empty()).ok_or("SUM measure array required")?;
    let order = summary["order"].as_array().ok_or("SUM ordering array required")?;
    let cap = summary["groupCap"].as_u64().ok_or("SUM groupCap required")?;
    if (groups.is_empty() && (cap != 1 || !order.is_empty())) || (!groups.is_empty() && !(1..=1000).contains(&cap)) {
        return Err("SUM groupCap or global ordering outside profile");
    }
    let mut ids=BTreeSet::new(); let mut group_ids=Vec::new(); let mut measure_ids=Vec::new();
    let mut group_fields=Vec::new(); let mut measure_fields=Vec::new(); let mut group_field_ids=BTreeSet::new();
    for g in groups {
        exact(g, &["id", "fieldRef"], &[])?;
        let id=g["id"].as_str().filter(|s| unity_current_predicate::valid_parameter_name(s)).ok_or("bounded group id required")?.to_owned();
        if !ids.insert(id.clone()) { return Err("duplicate SUM output id"); }
        let (module,element)=unity_current_predicate::checked_ref(&g["fieldRef"])?;
        let identity=key(&module,&element);
        let (_,carrier,_)=source.field(&g["fieldRef"])?;
        if !matches!(carrier.as_str(),"string"|"boolean") || !group_field_ids.insert(identity.clone()) { return Err("duplicate or unsupported SUM group Field"); }
        bound.insert(identity.clone()); group_ids.push(id.clone()); group_fields.push((id,g["fieldRef"].clone(),identity));
    }
    for m in measures {
        exact(m, &["id", "kind", "fieldRef"], &[])?;
        if m["kind"] != "sum" { return Err("finite SUM supports only SUM measures"); }
        let id=m["id"].as_str().filter(|s| unity_current_predicate::valid_parameter_name(s)).ok_or("bounded SUM id required")?.to_owned();
        if !ids.insert(id.clone()) { return Err("duplicate SUM output id"); }
        let (module,element)=unity_current_predicate::checked_ref(&m["fieldRef"])?;
        let identity=key(&module,&element);
        let field=unity_current_predicate::sum_field(source,&m["fieldRef"])?;
        if !matches!(field.carrier.as_str(),"integer"|"decimal") { return Err("SUM requires original signed integer or decimal Field"); }
        bound.insert(identity.clone());
        measure_fields.push((id.clone(),"sum".to_owned(),Some(m["fieldRef"].clone()),Some(identity)));
        measure_ids.push(id);
    }
    if group_fields.len()+measure_fields.len()>8 { return Err("SUM output bound exceeded"); }
    for o in order {
        exact(o, &["output", "direction", "nulls"], &[])?;
        let id=o["output"].as_str().ok_or("SUM order output required")?;
        if !ids.contains(id) || !matches!(o["direction"].as_str(),Some("asc"|"desc")) || !matches!(o["nulls"].as_str(),Some("first"|"last")) { return Err("invalid SUM order entry"); }
    }
    let mut seen=BTreeSet::new();
    if order.iter().any(|o| !seen.insert(o["output"].as_str().unwrap().to_owned())) { return Err("duplicate SUM order output"); }
    Ok((group_ids,measure_ids,order.clone(),group_fields,measure_fields,cap,false))
}

fn normalized_order(order: &[Value], groups: &[(String, Value, String)]) -> Result<Vec<Value>> {
    let mut out=order.to_vec(); let mut seen=out.iter().filter_map(|v|v["output"].as_str().map(str::to_owned)).collect::<BTreeSet<_>>();
    for (id,_,_) in groups { if seen.insert(id.clone()) { out.push(json!({"output":id,"direction":"asc","nulls":"last"})); } }
    Ok(out)
}

pub(crate) fn compile_json(raw: &str) -> String {
    match compile(raw) { Ok(value)=>value.to_string(), Err(message)=>json!({"version":RESPONSE_VERSION,"status":"blocked","diagnostics":[{"code":"WFT-UNITY-CURRENT-SUM","message":message}]}).to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn original_unity_request() -> Value {
        let model_text = include_str!("../tests/fixtures/original-unity-model.json");
        let binding_text = include_str!("../tests/fixtures/original-unity-binding.json");
        let model: Value = serde_json::from_str(model_text).unwrap();
        let set = model["modules"][0]["extensions"]["umf.access"]["columns"][0].clone();
        let model_sha = weft_core::json::sha256(model_text.as_bytes());
        let binding_sha = weft_core::json::sha256(binding_text.as_bytes());
        let column_set_target = set["target"].clone();
        json!({
            "version": REQUEST_VERSION,
            "source": {"version": CORE_VERSION, "utf8": model_text, "sha256": model_sha},
            "expectedTarget": {
                "kind":"unity-catalog", "warehouseId":"warehouse-source-fixture",
                "catalog":"main", "schema":"search", "table":"docs",
                "tableUuid":"source-fixture-uuid", "runtimeVersion":"2026-10-16",
                "columnSetTarget":column_set_target,
                "columns":[
                    {"fieldRef":{"module":"search","element":"count"},"nativeName":"count","nativeType":"BIGINT","nullable":false,"sourceFacets":{"integerWidth":{"bits":64,"signed":true}}},
                    {"fieldRef":{"module":"search","element":"amount"},"nativeName":"amount","nativeType":"DECIMAL(38,10)","nullable":false,"sourceFacets":{"precision":38,"scale":10}}
                ]
            },
            "nativeColumns": {
                "target":column_set_target,
                "sets":[{
                    "columnSet":{"documentId":model["id"],"coreVersion":CORE_VERSION,"documentSha256":model_sha,
                        "module":"search","pointer":"/modules/0/extensions/umf.access/columns/0"},
                    "binding":{"sourceId":"uc-search-binding","documentJson":binding_text,"documentSha256":binding_sha}
                }]
            },
            "read":{"kind":"finite-sum","tableRef":{"module":"search","element":"Doc"},"predicate":null,
                "summary":{"kind":"aggregate","groups":[],"measures":[
                    {"id":"sum_count","kind":"sum","fieldRef":{"module":"search","element":"count"}},
                    {"id":"sum_amount","kind":"sum","fieldRef":{"module":"search","element":"amount"}}
                ],"order":[],"groupCap":1},"frameCapBytes":65536,"deadlineMs":5000},
            "parameters":[]
        })
    }

    // Derive source-authored variants from the pinned original pair.  This
    // updates the source bytes, their source digest and the original
    // ColumnSet type/facets together; it does not manufacture a native
    // observation or bypass source custody.
    fn retarget_source_field(request: &mut Value, field_id: &str, native_type: &str, facets: Value) {
        let mut model: Value = serde_json::from_str(request["source"]["utf8"].as_str().unwrap()).unwrap();
        let module = &mut model["modules"][0];
        let element = module["elements"].as_array_mut().unwrap().iter_mut()
            .find(|e| e["id"] == field_id).unwrap();
        element["facets"] = facets.clone();
        let column = module["extensions"]["umf.access"]["columns"][0]["columns"].as_array_mut().unwrap().iter_mut()
            .find(|c| c["field"]["element"] == field_id).unwrap();
        column["type"] = json!(native_type);
        let source_text = serde_json::to_string(&model).unwrap();
        let digest = weft_core::json::sha256(source_text.as_bytes());
        request["source"]["utf8"] = json!(source_text);
        request["source"]["sha256"] = json!(digest);
        request["nativeColumns"]["sets"][0]["columnSet"]["documentSha256"] = json!(digest);
        let target_column = request["expectedTarget"]["columns"].as_array_mut().unwrap().iter_mut()
            .find(|c| c["fieldRef"]["element"] == field_id).unwrap();
        target_column["nativeType"] = json!(native_type);
        target_column["sourceFacets"] = facets;
    }

    #[test]
    fn exported_current_read_dispatch_compiles_original_unity_numeric_source() {
        let request = original_unity_request();
        let response: Value = serde_json::from_str(&super::super::unity_current_read::compile_json(&request.to_string())).unwrap();
        assert_eq!(response["version"], RESPONSE_VERSION);
        assert_eq!(response["status"], "compiled", "{response}");
        assert_eq!(response["obligations"]["summaryProfile"], "bounded-current-exact-sum/0.1");
        assert_eq!(response["outputs"].as_array().unwrap().len(), 2);
        assert_eq!(response["outputs"][0]["sourceFacets"]["integerWidth"]["bits"], 64);
        assert_eq!(response["outputs"][1]["sourceFacets"]["scale"], 10);
        assert_eq!(response["arithmetic"]["algorithm"], "signed-coefficient-base10^19/1");
        assert!(response["sql"].as_str().unwrap().contains("TRY_SUM"));
        assert!(!response["sql"].as_str().unwrap().contains("SUM(r."));
        // Current Unity responses omit internal raw-lowerer checks entirely.
        assert!(response.get("nativeChecks").is_none());
    }

    #[test]
    fn exported_current_read_refuses_missing_original_columnset_custody() {
        let mut request = original_unity_request();
        request.as_object_mut().unwrap().remove("nativeColumns");
        let response: Value = serde_json::from_str(&super::super::unity_current_read::compile_json(&request.to_string())).unwrap();
        assert_eq!(response["status"], "blocked");
        assert_eq!(response["diagnostics"][0]["code"], "WFT-UNITY-CURRENT-SUM");
    }

    #[test]
    fn exported_current_read_refuses_changed_decimal_source_facets() {
        let mut request = original_unity_request();
        request["expectedTarget"]["columns"][1]["sourceFacets"]["scale"] = json!(9);
        let response: Value = serde_json::from_str(&super::super::unity_current_read::compile_json(&request.to_string())).unwrap();
        assert_eq!(response["status"], "blocked");
    }

    #[test]
    fn exported_current_read_compiles_predicated_string_group_and_two_numeric_measures() {
        let mut request = original_unity_request();
        request["expectedTarget"]["columns"].as_array_mut().unwrap().push(json!({
            "fieldRef":{"module":"search","element":"body"},
            "nativeName":"body","nativeType":"STRING","nullable":false
        }));
        request["read"]["predicate"] = json!({"kind":"comparison","op":"eq",
            "fieldRef":{"module":"search","element":"body"},"operand":{"parameter":"p1"}});
        request["read"]["summary"]["groups"] = json!([{"id":"body_group","fieldRef":{"module":"search","element":"body"}}]);
        request["read"]["summary"]["groupCap"] = json!(20);
        request["parameters"] = json!([{"name":"p1","carrier":"string","nullable":false,"maxUtf8Bytes":512}]);
        let response: Value = serde_json::from_str(&super::super::unity_current_read::compile_json(&request.to_string())).unwrap();
        assert_eq!(response["status"], "compiled", "{response}");
        assert_eq!(response["outputs"].as_array().unwrap().len(), 3);
        assert_eq!(response["rowCap"], 21);
        assert_eq!(response["parameterOrder"], json!(["p1"]));
        assert_eq!(response["summary"]["groupOutputIds"], json!(["body_group"]));
    }

    #[test]
    fn exported_current_read_compiles_all_signed_integer_widths_with_original_columnset_custody() {
        for (bits, native) in [(8,"TINYINT"),(16,"SMALLINT"),(32,"INT"),(64,"BIGINT")] {
            let mut request = original_unity_request();
            retarget_source_field(&mut request, "count", native, json!({"integerWidth":{"bits":bits,"signed":true}}));
            let response: Value = serde_json::from_str(&super::super::unity_current_read::compile_json(&request.to_string())).unwrap();
            assert_eq!(response["status"], "compiled", "{bits}: {response}");
            assert_eq!(response["outputs"][0]["sourceFacets"]["integerWidth"]["bits"], bits);
        }
    }

    #[test]
    fn exported_current_read_compiles_decimal_scale_zero_ten_and_thirty_eight() {
        for (precision, scale) in [(38,0),(38,10),(38,38)] {
            let mut request = original_unity_request();
            retarget_source_field(&mut request, "amount", &format!("DECIMAL({precision},{scale})"), json!({"precision":precision,"scale":scale}));
            let response: Value = serde_json::from_str(&super::super::unity_current_read::compile_json(&request.to_string())).unwrap();
            assert_eq!(response["status"], "compiled", "{precision},{scale}: {response}");
            assert_eq!(response["outputs"][1]["decoder"]["scale"], scale);
        }
    }
}
