//! Original raw-read privilege analysis probes. No SQL is executed here.
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
type Result<T> = std::result::Result<T, &'static str>;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    version: String,
    request: Value,
    program: Value,
    probe_table: String,
}
fn sha(text: &str) -> String {
    weft_core::json::sha256(text.as_bytes())
}
fn ident(text: &str) -> Result<String> {
    if text.is_empty()
        || text.len() > 63
        || !text.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        || !text.starts_with("__weft_privilege_")
    {
        return Err("owned bounded probe table identifier required");
    };
    Ok(format!("\"{text}\""))
}
#[derive(Clone)]
struct Hole {
    start: usize,
    end: usize,
    position: usize,
    target: String,
}
/// Scan only compiler-emitted parameter holes. Quoted SQL bytes stay untouched.
fn holes(sql: &str) -> Result<Vec<Hole>> {
    let b = sql.as_bytes();
    let mut i = 0;
    let mut result = vec![];
    while i < b.len() {
        match b[i] {
            b'\'' | b'"' => {
                let quote = b[i];
                let escape = quote == b'\'' && i > 0 && matches!(b[i - 1], b'e' | b'E');
                i += 1;
                loop {
                    if i >= b.len() {
                        return Err("unterminated original SQL quote");
                    };
                    if escape && b[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if b[i] == quote {
                        if b.get(i + 1) == Some(&quote) {
                            i += 2;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    i += 1
                }
            }
            b'-' if b.get(i + 1) == Some(&b'-') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1
                }
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                i += 2;
                let mut depth = 1;
                while depth > 0 {
                    if i >= b.len() {
                        return Err("unterminated original SQL comment");
                    };
                    if b.get(i..i + 2) == Some(&b"/*"[..]) {
                        depth += 1;
                        i += 2
                    } else if b.get(i..i + 2) == Some(&b"*/"[..]) {
                        depth -= 1;
                        i += 2
                    } else {
                        i += 1
                    }
                }
            }
            b'$' => {
                let start = i;
                i += 1;
                let digits = i;
                while b.get(i).is_some_and(u8::is_ascii_digit) {
                    i += 1
                }
                if i == digits {
                    return Err("unestablished original dollar-quoted SQL profile");
                };
                let position = sql[digits..i]
                    .parse::<usize>()
                    .map_err(|_| "invalid original parameter position")?;
                if position == 0 {
                    return Err("parameter positions start at1");
                };
                let end = i;
                let tail = &sql[i..];
                let tail = tail
                    .strip_prefix("::pg_catalog.")
                    .ok_or("every owner parameter hole requires an explicit native cast")?;
                let length = tail
                    .bytes()
                    .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'[' | b']'))
                    .count();
                let target = &tail[..length];
                if !matches!(
                    target,
                    "text" | "bool" | "numeric" | "int8" | "text[]" | "bool[]" | "numeric[]"
                ) {
                    return Err("original parameter native cast outside analyzed profile");
                };
                result.push(Hole {
                    start,
                    end,
                    position,
                    target: format!("pg_catalog.{target}"),
                });
            }
            _ => i += 1,
        }
    }
    Ok(result)
}
fn replace(sql: &str, holes: &[Hole], table: &str) -> String {
    let mut output = String::new();
    let mut last = 0;
    for h in holes {
        output.push_str(&sql[last..h.start]);
        output.push_str(&format!(
            "(SELECT \"p{}\" FROM pg_temp.{table})",
            h.position
        ));
        last = h.end;
    }
    output.push_str(&sql[last..]);
    output
}
fn alternatives(parameter: &Value, target: &str) -> Result<Vec<(u32, String)>> {
    let carrier = parameter["carrier"]
        .as_str()
        .ok_or("original parameter carrier absent")?;
    let list = parameter.get("container").is_some();
    let expected = match (carrier, list) {
        ("string", false) => "pg_catalog.text",
        ("boolean", false) => "pg_catalog.bool",
        ("integer" | "decimal", false) => "pg_catalog.numeric",
        ("string", true) => "pg_catalog.text[]",
        ("boolean", true) => "pg_catalog.bool[]",
        ("integer" | "decimal", true) => "pg_catalog.numeric[]",
        _ => return Err("native probe carrier outside original profile"),
    };
    if target != expected && !(carrier == "integer" && !list && target == "pg_catalog.int8") {
        return Err("original cast and parameter carrier disagree");
    }
    let mut values = match (carrier, list) {
        ("string", false) => vec![(25, "pg_catalog.text")],
        ("boolean", false) => vec![(16, "pg_catalog.bool")],
        ("decimal", false) => vec![(1700, "pg_catalog.numeric")],
        ("integer", false) => vec![
            (21, "pg_catalog.int2"),
            (23, "pg_catalog.int4"),
            (20, "pg_catalog.int8"),
            (1700, "pg_catalog.numeric"),
        ],
        ("string", true) => vec![(1009, "pg_catalog.text[]")],
        ("boolean", true) => vec![(1000, "pg_catalog.bool[]")],
        ("decimal", true) => vec![(1231, "pg_catalog.numeric[]")],
        ("integer", true) => vec![
            (1005, "pg_catalog.int2[]"),
            (1007, "pg_catalog.int4[]"),
            (1016, "pg_catalog.int8[]"),
            (1231, "pg_catalog.numeric[]"),
        ],
        _ => unreachable!(),
    };
    // Unknown/NULL and empty arrays infer the explicit original cast target. Their
    // actual native protocol and type input/receive closure still need evidence.
    let mut result = vec![];
    if carrier == "string" || list || parameter["nullable"] == true {
        result.push((0, target.to_string()));
    }
    result.extend(values.drain(..).map(|(oid, t)| (oid, t.into())));
    Ok(result)
}
fn statement(
    path: String,
    role: &str,
    sql: &str,
    parameters: &[Value],
    order: &[Value],
    table: &str,
    output_count: usize,
) -> Result<Value> {
    if output_count == 0 {
        return Err("original output inventory absent");
    };
    let output_columns = (1..=output_count)
        .map(|i| format!("c{i}"))
        .collect::<Vec<_>>();
    let hs = holes(sql)?;
    let mut targets = BTreeMap::new();
    for h in &hs {
        if h.position > order.len() {
            return Err("original SQL hole exceeds parameter order");
        };
        if targets
            .insert(h.position, h.target.clone())
            .is_some_and(|old| old != h.target)
        {
            return Err("same original parameter position has incompatible casts");
        }
    }
    if targets.len() != order.len() {
        return Err("exact original parameter hole inventory required");
    }
    let mut definitions = vec![];
    for (index, name) in order.iter().enumerate() {
        let name = name.as_str().ok_or("original parameter name required")?;
        let matches = parameters
            .iter()
            .filter(|p| p["name"] == name)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err("unique original parameter descriptor required");
        };
        let target = &targets[&(index + 1)];
        definitions.push((name, matches[0], target, alternatives(matches[0], target)?));
    }
    let variant_count = 1 + definitions.iter().map(|d| d.3.len() - 1).sum::<usize>();
    if definitions.len().saturating_mul(variant_count) > 4096 {
        return Err("native privilege analysis input budget exceeded; no completeness issued");
    }
    let baseline = definitions
        .iter()
        .map(|d| d.3[0].clone())
        .collect::<Vec<_>>();
    let mut variants = vec![baseline.clone()];
    // Every occurrence is explicitly cast to its fixed owner type. Vary only the
    // pre-cast protocol type: all business operators see the same post-cast type.
    for (i, definition) in definitions.iter().enumerate() {
        for alternative in definition.3.iter().skip(1) {
            let mut types = baseline.clone();
            types[i] = alternative.clone();
            variants.push(types)
        }
    }
    let probe_sql = replace(sql, &hs, table);
    let mut probes = vec![];
    for types in variants {
        let columns=types.iter().enumerate().map(|(i,(oid,ty))|json!({"name":format!("p{}",i+1),"nativeType":ty,"protocolOid":oid,"inferredTargetType":definitions[i].2,"parameter":definitions[i].0,"source":definitions[i].1.get("source"),"carrier":definitions[i].1["carrier"]})).collect::<Vec<_>>();
        let setup = if columns.is_empty() {
            Value::Null
        } else {
            json!(format!(
                "CREATE TEMPORARY TABLE {table} ({}) ON COMMIT DROP",
                types
                    .iter()
                    .enumerate()
                    .map(|(i, (_, ty))| format!("\"p{}\" {ty}", i + 1))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        };
        probes.push(json!({"inputRelation":{"schema":"pg_temp","native":table.trim_matches('"'),"columns":columns,"probeOnly":true},"setupSql":setup,"outputColumns":output_columns,"sql":probe_sql,"sqlSha256":sha(&probe_sql),"analysisOnly":true,"executeBusinessSql":false,"typeIoMustResolveNatively":true,"probeSupportDependencies":"inputRelation-and-scalar-subselect-excluded-from-business-privileges"}));
    }
    Ok(
        json!({"path":path,"role":role,"sql":sql,"sqlSha256":sha(sql),"parameters":parameters,"parameterOrder":order,"explicitFixedCastProof":true,"protocolCoverage":"independent-precast-types-fixed-postcast-operators","probes":probes}),
    )
}
fn array<'a>(v: &'a Value, key: &str) -> Result<&'a [Value]> {
    v[key]
        .as_array()
        .map(Vec::as_slice)
        .ok_or("original array inventory absent")
}
fn build(raw: &str) -> Result<Value> {
    let request: Request = serde_json::from_value(
        weft_core::json::checked_json(raw).map_err(|_| "probe request framing refused")?,
    )
    .map_err(|_| "closed original privilege probe request required")?;
    if request.version != "weft.native-privilege-probes/0.1" {
        return Err("privilege probe request version differs");
    }
    let table = ident(&request.probe_table)?;
    let compiled = match request.request["version"].as_str() {
        Some("weft.raw-read/0.1") => crate::raw_read::compile_json(&request.request.to_string()),
        Some("weft.related-read/0.1") => {
            crate::related_read::compile_json(&request.request.to_string())
        }
        _ => {
            return Err(
                "unhandled original program role: search/action/bulk/engine require separate owner profiles",
            );
        }
    };
    let program: Value =
        serde_json::from_str(&compiled).map_err(|_| "original compiler output refused")?;
    if program["status"] != "compiled" || program != request.program {
        return Err("probe program differs from exact original compiler");
    }
    if !matches!(
        program["target"].as_str(),
        Some("postgresql18" | "lakebase18")
    ) {
        return Err("native probe target differs");
    }
    let mut parameters = array(&program, "parameters")?.to_vec();
    parameters.extend_from_slice(array(&program, "cursorParameters")?);
    let mut statements = vec![statement(
        "/sql".into(),
        "business-read",
        program["sql"].as_str().ok_or("original SQL absent")?,
        &parameters,
        array(&program, "parameterOrder")?,
        &table,
        array(&program, "outputs")?.len() + array(&program, "cursorOutputs")?.len(),
    )?];
    let mut locks = vec![];
    for (i, sql) in array(&program, "lockSql")?.iter().enumerate() {
        let sql = sql.as_str().ok_or("original lock SQL absent")?;
        locks.push(json!({"path":format!("/lockSql/{i}"),"role":"table-lock","sql":sql,"sqlSha256":sha(sql),"analysis":"native-lock-privilege","tablePrivilege":"SELECT","columnOnlyGrantsSufficient":false,"executeBusinessSql":false}));
    }
    for (kind, entries, role) in [
        ("nativeProjection", "columns", "projection-admission"),
        ("nativeRelationshipKeys", "keys", "key-integrity-admission"),
        (
            "nativeRelationships",
            "relationships",
            "relationship-integrity-admission",
        ),
        ("termFilter", "indexes", "term-index-admission"),
    ] {
        if let Some(obligation) = program["obligations"].get(kind) {
            for (i, item) in array(obligation, entries)?.iter().enumerate() {
                let admission = &item["nativeAdmission"];
                let sql = admission["sql"]
                    .as_str()
                    .ok_or("original admission SQL absent")?;
                statements.push(statement(
                    format!("/obligations/{kind}/{entries}/{i}/nativeAdmission/sql"),
                    role,
                    sql,
                    array(admission, "parameters")?,
                    array(admission, "dbapiParameterOrder")?,
                    &table,
                    1,
                )?);
            }
        }
    }
    let mut result = json!({"version":"weft.native-privilege-probes/0.1","status":"compiled","request":request.request,"program":program,"probeTable":request.probe_table,"sourceProgramDigest":request.program["structuralDigest"],"nativeProfile":{"postgresqlVersionNum":180006,"protocol":"psycopg3.3.6-extended-scalar-array-candidate"},"statements":statements,"locks":locks,"grantSemantics":request.program["privilegeInventory"],"complete":false,"unresolved":["native-analyzed-function-operator-cast-type-io-closure","independent-native-grant-sufficiency","runtime-metadata-control-closing-statements"],"nativeAuthority":false});
    let hash = sha(&weft_core::mutation::canonical(&result));
    result["structuralDigest"] =
        json!({"owner":"weft-postgresql-native-privilege-probes","version":"0.1","sha256":hash});
    let encoded = result.to_string();
    if encoded.len() > 4_194_304 || weft_core::json::checked_json(&encoded).is_err() {
        return Err("native privilege analysis artifact budget exceeded; no completeness issued");
    }
    Ok(result)
}
pub fn compile_json(raw: &str) -> String {
    build(raw).unwrap_or_else(|reason|json!({"version":"weft.native-privilege-probes/0.1","status":"blocked","diagnostics":[{"code":"WFT-NATIVE-PRIVILEGE-PROBES","message":reason}]})).to_string()
}
pub fn check_json(program: &str) -> String {
    let result = (|| {
        let value: Value =
            weft_core::json::checked_json(program).map_err(|_| "probe program framing refused")?;
        let raw = json!({"version":"weft.native-privilege-probes/0.1","request":value["request"],"program":value["program"],"probeTable":value["probeTable"]});
        let expected = build(&raw.to_string())?;
        if expected != value {
            return Err("owner privilege probe program differs");
        };
        Ok(expected)
    })();
    result.unwrap_or_else(|reason|json!({"version":"weft.native-privilege-probes/0.1","status":"blocked","diagnostics":[{"code":"WFT-NATIVE-PRIVILEGE-PROBES","message":reason}]})).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> Value {
        let mut r: Value = serde_json::from_str(include_str!(
            "../../../tests/raw-postgresql/read-request.json"
        ))
        .unwrap();
        r["parameters"] = json!([{"name":"bound","carrier":"integer","nullable":true}]);
        r["sql"] = json!(
            "SELECT o.id AS id FROM commerce.\"Order\" AS o WHERE o.id>:bound ORDER BY o.id LIMIT 5"
        );
        r
    }
    fn packet(request: Value) -> Value {
        let program: Value =
            serde_json::from_str(&crate::raw_read::compile_json(&request.to_string())).unwrap();
        assert_eq!(program["status"], "compiled", "{program}");
        json!({"version":"weft.native-privilege-probes/0.1","request":request,"program":program,"probeTable":"__weft_privilege_test"})
    }
    #[test]
    fn exact_original_read_emits_nonexecuting_typed_var_probes_and_closed_check() {
        let p = packet(request());
        let output = build(&p.to_string()).unwrap();
        if let Ok(path) = std::env::var("WEFT_PRIVILEGE_PROBE_OUTPUT") {
            std::fs::write(path, output.to_string()).unwrap();
        }
        assert_eq!(output["complete"], false);
        assert_eq!(output["nativeAuthority"], false);
        assert_eq!(
            output["sourceProgramDigest"],
            p["program"]["structuralDigest"]
        );
        let statements = output["statements"].as_array().unwrap();
        assert_eq!(statements.len(), 1);
        let s = &statements[0];
        assert_eq!(s["path"], "/sql");
        assert_eq!(s["sql"], p["program"]["sql"]);
        assert_eq!(s["probes"].as_array().unwrap().len(), 5);
        for probe in s["probes"].as_array().unwrap() {
            assert!(
                probe["sql"]
                    .as_str()
                    .unwrap()
                    .contains("SELECT \"p1\" FROM pg_temp.\"__weft_privilege_test\"")
            );
            assert!(
                probe["sql"]
                    .as_str()
                    .unwrap()
                    .contains("::pg_catalog.numeric")
            );
            assert_eq!(probe["executeBusinessSql"], false);
            assert_eq!(probe["analysisOnly"], true);
            assert_eq!(
                probe["inputRelation"]["columns"][0]["inferredTargetType"],
                "pg_catalog.numeric"
            );
        }
        assert_eq!(output["locks"][0]["tablePrivilege"], "SELECT");
        assert_eq!(output["locks"][0]["columnOnlyGrantsSufficient"], false);
        let checked: Value = serde_json::from_str(&check_json(&output.to_string())).unwrap();
        assert_eq!(checked, output);
        for path in [
            "/complete",
            "/statements/0/sql",
            "/statements/0/probes/0/sql",
            "/statements/0/probes/0/inputRelation/columns/0/protocolOid",
        ] {
            let mut forged = output.clone();
            *forged.pointer_mut(path).unwrap() = json!("forged");
            let q: Value = serde_json::from_str(&check_json(&forged.to_string())).unwrap();
            assert_eq!(q["status"], "blocked", "{path}");
        }
    }
    #[test]
    fn original_full_projection_admission_paths_and_stream_slots_are_all_retained() {
        let mut r: Value = serde_json::from_str(include_str!(
            "../../../tests/raw-read/postgresql-projection-request.json"
        ))
        .unwrap();
        r["keyset"] = json!(true);
        r["streamPageSize"] = json!(true);
        let p = packet(r);
        let output = build(&p.to_string()).unwrap();
        assert_eq!(output["statements"].as_array().unwrap().len(), 8);
        let business = &output["statements"][0];
        assert!(
            business["parameterOrder"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p == "__umf_page_size")
        );
        assert!(
            business["probes"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|p| p["inputRelation"]["columns"].as_array().unwrap())
                .any(|c| c["parameter"] == "__umf_page_size"
                    && c["inferredTargetType"] == "pg_catalog.int8")
        );
        for s in output["statements"].as_array().unwrap().iter().skip(1) {
            assert_eq!(s["role"], "projection-admission");
            let sql = p["program"].pointer(s["path"].as_str().unwrap()).unwrap();
            assert_eq!(sql, &s["sql"]);
            assert_eq!(s["sqlSha256"], sha(sql.as_str().unwrap()));
        }
    }
    #[test]
    fn emitted_holes_are_distinct_from_quoted_literals_and_identifier_bytes() {
        let sql = "SELECT E'\\\\$99\\\\' AS \"$88\",$1::pg_catalog.numeric -- $77\n/* $66 */";
        let h = holes(sql).unwrap();
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].position, 1);
        let q = replace(sql, &h, "\"__weft_privilege_test\"");
        assert!(q.contains("$99"));
        assert!(q.contains("\"$88\""));
        assert!(q.contains("-- $77"));
        for sql in [
            "SELECT $0::pg_catalog.text",
            "SELECT $1",
            "SELECT $1::untrusted.type",
            "SELECT $$unsafe$$",
            "SELECT 'unterminated",
        ] {
            assert!(holes(sql).is_err(), "{sql}");
        }
    }
    #[test]
    fn forged_original_requests_and_unsupported_roles_refuse() {
        let original = packet(request());
        for key in ["program", "probeTable", "version"] {
            let mut p = original.clone();
            p[key] = json!("forged");
            assert!(build(&p.to_string()).is_err());
        }
        let mut p = original.clone();
        p["extra"] = json!(true);
        assert!(build(&p.to_string()).is_err());
        let mut p = original;
        p["request"]["version"] = json!("weft.native-action/0.1");
        assert!(build(&p.to_string()).is_err());
    }
}

#[cfg(test)]
mod related_tests {
    use super::*;
    #[test]
    fn original_related_query_and_integrity_roles_remain_one_checked_program() {
        let mut source: Value = serde_json::from_str(include_str!(
            "../../../tests/raw-postgresql/read-request.json"
        ))
        .unwrap();
        source["parameters"] = json!([]);
        source["sql"] = json!(
            "SELECT o.id AS id FROM commerce.\"Order\" o WHERE o.status='open' ORDER BY o.id DESC LIMIT 2"
        );
        let mut target = source.clone();
        target["sql"] =
            json!("SELECT c.id AS id FROM commerce.\"Customer\" c ORDER BY c.id LIMIT 10");
        let request = json!({"version":"weft.related-read/0.1","source":source,"target":target,"relationship":{"module":"commerce","id":"customer","direction":"forward"}});
        let program: Value =
            serde_json::from_str(&crate::related_read::compile_json(&request.to_string())).unwrap();
        assert_eq!(program["status"], "compiled", "{program}");
        let p = json!({"version":"weft.native-privilege-probes/0.1","request":request,"program":program,"probeTable":"__weft_privilege_related"});
        let q = build(&p.to_string()).unwrap();
        let statements = q["statements"].as_array().unwrap();
        assert!(statements[0]["sql"].as_str().unwrap().contains("EXISTS"));
        assert!(
            statements
                .iter()
                .any(|s| s["role"] == "relationship-integrity-admission")
        );
        let expected_keys = p["program"]["obligations"]["nativeRelationshipKeys"]["keys"]
            .as_array()
            .map_or(0, Vec::len);
        assert_eq!(
            statements
                .iter()
                .filter(|s| s["role"] == "key-integrity-admission")
                .count(),
            expected_keys
        );
        assert_eq!(q["complete"], false);
    }
}
