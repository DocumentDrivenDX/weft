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
    holes_with_native_types(sql, &BTreeMap::new())
}
fn holes_with_native_types(
    sql: &str,
    native_types: &BTreeMap<String, String>,
) -> Result<Vec<Hole>> {
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
                let tail = sql[i..]
                    .strip_prefix("::")
                    .ok_or("every owner parameter hole requires an explicit native cast")?;
                let target = if let Some((_, native_type)) = native_types
                    .iter()
                    .find(|(cast, _)| tail.starts_with(cast.as_str()))
                {
                    native_type.clone()
                } else {
                    let tail = tail
                        .strip_prefix("pg_catalog.")
                        .ok_or("original parameter native cast outside analyzed profile")?;
                    let length = tail
                        .bytes()
                        .take_while(|c| {
                            c.is_ascii_alphanumeric() || matches!(c, b'_' | b'[' | b']')
                        })
                        .count();
                    let target = &tail[..length];
                    if !matches!(
                        target,
                        "text" | "bool" | "numeric" | "int8" | "text[]" | "bool[]" | "numeric[]"
                    ) {
                        return Err("original parameter native cast outside analyzed profile");
                    }
                    format!("pg_catalog.{target}")
                };
                result.push(Hole {
                    start,
                    end,
                    position,
                    target,
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
    if carrier == "vector" && !list {
        if parameter["nativeType"].as_str() != Some(target) {
            return Err("original vector cast and source-bound native type disagree");
        }
        return Ok(vec![(0, target.into()), (25, "pg_catalog.text".into())]);
    }
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
    statement_with_native_types(
        path,
        role,
        sql,
        parameters,
        order,
        table,
        output_count,
        &BTreeMap::new(),
    )
}
fn statement_with_native_types(
    path: String,
    role: &str,
    sql: &str,
    parameters: &[Value],
    order: &[Value],
    table: &str,
    output_count: usize,
    native_types: &BTreeMap<String, String>,
) -> Result<Value> {
    if output_count == 0 {
        return Err("original output inventory absent");
    };
    let output_columns = (1..=output_count)
        .map(|i| format!("c{i}"))
        .collect::<Vec<_>>();
    let hs = holes_with_native_types(sql, native_types)?;
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
fn positional_statement(
    path: String,
    role: &str,
    original: &Value,
    table: &str,
    output_count: usize,
    native_types: &BTreeMap<String, String>,
    vector_type: Option<&str>,
) -> Result<Value> {
    let parameters = array(original, "parameters")?
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let mut p = p.clone();
            p["name"] = json!(format!("p{}", i + 1));
            if p["array"] == true {
                p["container"] = json!("list");
            }
            if p["carrier"] == "vector" {
                p["nativeType"] = json!(vector_type.ok_or("source-bound vector type absent")?);
            }
            Ok(p)
        })
        .collect::<Result<Vec<_>>>()?;
    let order = parameters
        .iter()
        .map(|p| p["name"].clone())
        .collect::<Vec<_>>();
    statement_with_native_types(
        path,
        role,
        original["sql"].as_str().ok_or("original SQL absent")?,
        &parameters,
        &order,
        table,
        output_count,
        native_types,
    )
}
fn search_report(request: &Request, program: Value, table: &str) -> Result<Value> {
    let mut native_types = BTreeMap::new();
    let mut vector_type = None;
    if let Some(vector) = request.request.get("vectorBinding") {
        let schema = vector["extensionSchema"]
            .as_str()
            .ok_or("original vector schema absent")?;
        let dimensions = vector["dimensions"]
            .as_u64()
            .ok_or("original vector dimensions absent")?;
        let base = format!("\"{}\".vector", schema.replace('"', "\"\""));
        native_types.insert(format!("{base}({dimensions})"), base.clone());
        vector_type = Some(base);
    }
    let mut statements = vec![positional_statement(
        "/statement/sql".into(),
        "search-business-read",
        &program["statement"],
        table,
        array(&program, "outputs")?.len() + array(&program, "keyOutputs")?.len() + 1,
        &native_types,
        vector_type.as_deref(),
    )?];
    statements.push(positional_statement(
        "/nativeAdmission/sql".into(),
        "search-index-admission",
        &program["nativeAdmission"],
        table,
        1,
        &native_types,
        vector_type.as_deref(),
    )?);
    for (i, preflight) in array(&program, "preflights")?.iter().enumerate() {
        statements.push(positional_statement(
            format!("/preflights/{i}/sql"),
            "search-preflight",
            preflight,
            table,
            1,
            &native_types,
            vector_type.as_deref(),
        )?);
    }
    let mut locks = vec![];
    for (i, sql) in array(&program["selectionTemplate"], "lockSql")?
        .iter()
        .enumerate()
    {
        let sql = sql.as_str().ok_or("original search lock SQL absent")?;
        locks.push(json!({"path":format!("/selectionTemplate/lockSql/{i}"),"role":"table-lock","sql":sql,"sqlSha256":sha(sql),"analysis":"native-lock-privilege","tablePrivilege":"SELECT","columnOnlyGrantsSufficient":false,"executeBusinessSql":false}));
    }
    finish_report(
        json!({"version":"weft.native-privilege-probes/0.1","status":"compiled","request":request.request,"program":program,"probeTable":request.probe_table,"sourceProgramDigest":request.program["structuralDigest"],"nativeProfile":{"postgresqlVersionNum":180006,"protocol":"psycopg3.3.6-extended-scalar-array-vector-text-candidate"},"statements":statements,"locks":locks,"grantSemantics":null,"complete":false,"unresolved":["search-native-objects-and-grant-inventory","native-analyzed-function-operator-cast-type-io-closure","independent-native-grant-sufficiency","runtime-metadata-control-closing-statements"],"nativeAuthority":false}),
    )
}
fn quote_native(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
fn dml_statement(
    path: String,
    role: &str,
    original: &Value,
    table: &str,
    binding: &Value,
    entity: &Value,
    kind: &str,
    values: &Value,
) -> Result<Value> {
    let fields = array(&entity["key"], "fields")?;
    let mut result = positional_statement(
        path,
        role,
        original,
        table,
        fields.len(),
        &BTreeMap::new(),
        None,
    )?;
    let targets = array(binding, "tables")?
        .iter()
        .filter(|t| t["logical"] == entity["key"]["table"])
        .collect::<Vec<_>>();
    if targets.len() != 1 {
        return Err("unique original DML target binding required");
    }
    let target = targets[0];
    let native = quote_native(
        target["native"]
            .as_str()
            .ok_or("original native DML table absent")?,
    );
    let native = match binding["schema"].as_str() {
        Some(schema) => format!("{}.{native}", quote_native(schema)),
        None => native,
    };
    let mut projection = vec![];
    for (i, field) in fields.iter().enumerate() {
        let columns = array(target, "columns")?
            .iter()
            .filter(|c| c["logical"] == field["logical"])
            .collect::<Vec<_>>();
        if columns.len() != 1 {
            return Err("unique original RETURNING Key column required");
        }
        projection.push(format!(
            "{} AS \"c{}\"",
            quote_native(
                columns[0]["native"]
                    .as_str()
                    .ok_or("native RETURNING column absent")?
            ),
            i + 1
        ));
    }
    let suffix = &sha(&format!("{table}:{}", result["path"]))[..16];
    let event = format!("__weft_privilege_event_{suffix}");
    let rule = format!("__weft_privilege_rule_{suffix}");
    if table.trim_matches('"') == event {
        return Err("probe input/event identity collision");
    }
    let event_quoted = ident(&event)?;
    let rule_quoted = ident(&rule)?;
    let event_sql = format!(
        "CREATE TEMPORARY TABLE {event_quoted} AS SELECT {} FROM {native} WITH NO DATA",
        projection.join(", ")
    );
    let command = match kind {
        "set" => 2,
        "create" => 3,
        "delete" => 4,
        _ => return Err("original DML effect kind outside analyzed profile"),
    };
    for probe in result["probes"]
        .as_array_mut()
        .ok_or("DML probe inventory absent")?
    {
        let rule_sql = format!(
            "CREATE RULE {rule_quoted} AS ON INSERT TO pg_temp.{event_quoted} DO INSTEAD ({})",
            probe["sql"].as_str().ok_or("DML probe SQL absent")?
        );
        let mut setup = vec![];
        if let Some(input) = probe["setupSql"].as_str() {
            setup.push(json!(input));
        }
        setup.push(json!(event_sql));
        setup.push(json!(rule_sql));
        probe["analysisKind"] = json!("pg-rewrite-rule/0.1");
        probe["setupStatements"] = json!(setup);
        probe["analysisRelation"] = json!({"schema":"pg_temp","native":event,"probeOnly":true});
        probe["analysisRule"] = json!({"name":rule,"event":"INSERT","instead":true,"unconditional":true,"neverFire":true,"expectedBusinessCommandType":command});
        probe["probeSupportDependencies"] = json!(
            "input-relation-scalar-subselect-event-relation-rule-and-event-type-projection-excluded-from-business-privileges"
        );
    }
    result["writeSemantics"] = json!({"operation":kind,"entity":entity,"assignedFields":values,"returningKeyFields":fields,"nativeTable":native,"grantSufficiency":false});
    Ok(result)
}
fn mutation_report(request: &Request, program: Value, table: &str) -> Result<Value> {
    let bulk = request.request["version"] == "weft.native-bulk/0.1";
    let mut statements = vec![positional_statement(
        "/admission/sql".into(),
        "mutation-principal-admission",
        &program["admission"],
        table,
        4,
        &BTreeMap::new(),
        None,
    )?];
    if bulk {
        for (name, role) in [
            ("preview", "bulk-preview"),
            ("selection", "bulk-locked-selection"),
        ] {
            statements.push(positional_statement(
                format!("/{name}/sql"),
                role,
                &program[name],
                table,
                array(&program, "selectionFields")?.len(),
                &BTreeMap::new(),
                None,
            )?);
        }
        statements.push(dml_statement(
            "/update/sql".into(),
            "bulk-update",
            &program["update"],
            table,
            &program["binding"],
            &program["entity"],
            "set",
            &program["values"],
        )?);
    } else {
        for (name, role) in [
            ("preImages", "action-preimage"),
            ("postImages", "action-postimage"),
        ] {
            for (i, image) in array(&program, name)?.iter().enumerate() {
                statements.push(positional_statement(
                    format!("/{name}/{i}/statement/sql"),
                    role,
                    &image["statement"],
                    table,
                    array(image, "columns")?.len(),
                    &BTreeMap::new(),
                    None,
                )?);
            }
        }
        for (name, role) in [
            ("preconditions", "action-precondition"),
            ("postconditions", "action-postcondition"),
        ] {
            for (i, condition) in array(&program, name)?.iter().enumerate() {
                statements.push(positional_statement(
                    format!("/{name}/{i}/statement/sql"),
                    role,
                    &condition["statement"],
                    table,
                    2,
                    &BTreeMap::new(),
                    None,
                )?);
            }
        }
        for (i, effect) in array(&program, "effects")?.iter().enumerate() {
            if !effect["guard"].is_null() {
                statements.push(positional_statement(
                    format!("/effects/{i}/guard/sql"),
                    "action-change-guard",
                    &effect["guard"],
                    table,
                    1,
                    &BTreeMap::new(),
                    None,
                )?);
            }
            statements.push(dml_statement(
                format!("/effects/{i}/statement/sql"),
                "action-effect",
                &effect["statement"],
                table,
                &program["binding"],
                &effect["entity"],
                effect["kind"]
                    .as_str()
                    .ok_or("original effect kind absent")?,
                &effect["values"],
            )?);
        }
    }
    let mut locks = vec![];
    for (i, sql) in array(&program, "lockSql")?.iter().enumerate() {
        let sql = sql.as_str().ok_or("original mutation lock absent")?;
        locks.push(json!({"path":format!("/lockSql/{i}"),"role":"mutation-table-lock","sql":sql,"sqlSha256":sha(sql),"analysis":"native-lock-privilege","tablePrivilegeAlternatives":["INSERT","UPDATE","DELETE","TRUNCATE","MAINTAIN"],"columnOnlyGrantsSufficient":false,"executeBusinessSql":false}));
    }
    finish_report(
        json!({"version":"weft.native-privilege-probes/0.1","status":"compiled","request":request.request,"program":program,"probeTable":request.probe_table,"sourceProgramDigest":request.program["structuralDigest"],"nativeProfile":{"postgresqlVersionNum":180006,"protocol":"psycopg3.3.6-extended-scalar-array-dml-candidate"},"executionCoverage":if bulk{json!(["preview-only","atomic-execute"])}else{json!(["invoke-first-false","validate-collect-all"])},"statements":statements,"locks":locks,"grantSemantics":null,"complete":false,"unresolved":["mutation-original-write-grant-inventory","native-analyzed-function-operator-cast-type-io-closure","independent-native-grant-sufficiency","runtime-metadata-control-closing-reconciliation-statements"],"nativeAuthority":false}),
    )
}
fn finish_report(mut result: Value) -> Result<Value> {
    let hash = sha(&weft_core::mutation::canonical(&result));
    result["structuralDigest"] =
        json!({"owner":"weft-postgresql-native-privilege-probes","version":"0.1","sha256":hash});
    let encoded = result.to_string();
    if encoded.len() > 4_194_304 || weft_core::json::checked_json(&encoded).is_err() {
        return Err("native privilege analysis artifact budget exceeded; no completeness issued");
    }
    Ok(result)
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
        Some("weft.search/0.1") => crate::search::compile_json(&request.request.to_string()),
        Some("weft.native-action/0.1") => {
            crate::raw_action::compile_json(&request.request.to_string())
        }
        Some("weft.native-bulk/0.1") => crate::raw_bulk::compile_json(&request.request.to_string()),
        _ => {
            return Err(
                "unhandled original program role: engine requires a separate owner profile",
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
    if request.request["version"] == "weft.search/0.1" {
        return search_report(&request, program, &table);
    }
    if matches!(
        request.request["version"].as_str(),
        Some("weft.native-action/0.1" | "weft.native-bulk/0.1")
    ) {
        return mutation_report(&request, program, &table);
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
    let result = json!({"version":"weft.native-privilege-probes/0.1","status":"compiled","request":request.request,"program":program,"probeTable":request.probe_table,"sourceProgramDigest":request.program["structuralDigest"],"nativeProfile":{"postgresqlVersionNum":180006,"protocol":"psycopg3.3.6-extended-scalar-array-candidate"},"statements":statements,"locks":locks,"grantSemantics":request.program["privilegeInventory"],"complete":false,"unresolved":["native-analyzed-function-operator-cast-type-io-closure","independent-native-grant-sufficiency","runtime-metadata-control-closing-statements"],"nativeAuthority":false});
    finish_report(result)
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

#[cfg(test)]
mod source_semantic_field_tests {
    use super::*;
    #[test]
    fn related_constant_folded_source_field_is_retained_but_not_fabricated_as_native_used_column() {
        let mut source: Value = serde_json::from_str(include_str!(
            "../../../tests/raw-postgresql/read-request.json"
        ))
        .unwrap();
        source["parameters"] = json!([]);
        source["sql"] = json!(
            "SELECT o.id AS id FROM commerce.\"Order\" o WHERE NOT IN_EMPTY(o.status) ORDER BY o.id DESC LIMIT 2"
        );
        let mut target = source.clone();
        target["sql"] =
            json!("SELECT c.id AS id FROM commerce.\"Customer\" c ORDER BY c.id LIMIT 10");
        let native = source["binding"]["tables"][0]["native"].clone();
        let request = json!({"version":"weft.related-read/0.1","source":source,"target":target,"relationship":{"module":"commerce","id":"customer","direction":"forward"}});
        let program: Value =
            serde_json::from_str(&crate::related_read::compile_json(&request.to_string())).unwrap();
        assert_eq!(program["status"], "compiled", "{program}");
        let p = json!({"version":"weft.native-privilege-probes/0.1","request":request,"program":program,"probeTable":"__weft_privilege_source"});
        let q = build(&p.to_string()).unwrap();
        assert!(
            q["grantSemantics"]["semanticFields"]
                .as_array()
                .unwrap()
                .iter()
                .any(
                    |f| f["logicalTable"] == json!(["commerce", "Order"]) && f["field"] == "status"
                )
        );
        let used = q["grantSemantics"]["tables"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["native"] == native)
            .unwrap();
        assert!(
            !used["columns"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c == "status")
        );
        assert!(
            !q["statements"][0]["sql"]
                .as_str()
                .unwrap()
                .contains("\"status\"")
        );
        assert_eq!(q["complete"], false);
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;
    #[test]
    fn original_keyword_and_vector_program_paths_and_typmod_casts_are_preserved() {
        for raw in [
            include_str!("../../../tests/native-search/keyword-request.json"),
            include_str!("../../../tests/native-search/vector-request.json"),
        ] {
            let request: Value = serde_json::from_str(raw).unwrap();
            let program: Value = serde_json::from_str(&crate::search::compile_json(raw)).unwrap();
            assert_eq!(program["status"], "compiled", "{program}");
            let packet = json!({"version":"weft.native-privilege-probes/0.1","request":request,"program":program,"probeTable":"__weft_privilege_search"});
            let report = build(&packet.to_string()).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&check_json(&report.to_string())).unwrap(),
                report
            );
            assert_eq!(
                report["statements"].as_array().unwrap().len(),
                2 + array(&program, "preflights").unwrap().len()
            );
            for statement in report["statements"].as_array().unwrap() {
                assert_eq!(
                    program
                        .pointer(statement["path"].as_str().unwrap())
                        .unwrap(),
                    &statement["sql"]
                );
            }
            assert_eq!(
                report["locks"].as_array().unwrap().len(),
                array(&program["selectionTemplate"], "lockSql")
                    .unwrap()
                    .len()
            );
            assert_eq!(report["complete"], false);
            if program["statement"]["parameters"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["carrier"] == "vector")
            {
                let probes = report["statements"][0]["probes"].as_array().unwrap();
                assert_eq!(probes.len(), 2);
                assert_eq!(
                    probes[0]["inputRelation"]["columns"][0]["nativeType"],
                    "\"public\".vector"
                );
                assert!(
                    probes[0]["sql"]
                        .as_str()
                        .unwrap()
                        .contains("::\"public\".vector(3)")
                );
                assert!(
                    !probes[0]["setupSql"]
                        .as_str()
                        .unwrap()
                        .contains("vector(3)")
                );
                assert_eq!(probes[1]["inputRelation"]["columns"][0]["protocolOid"], 25);
            }
            let mut forged = packet;
            forged["program"]["statement"]["sql"] = json!("SELECT 1");
            assert!(build(&forged.to_string()).is_err());
        }
    }
}

#[cfg(test)]
mod mutation_tests {
    use super::*;
    fn action() -> Value {
        serde_json::from_str(include_str!(
            "../../../tests/native-actions/action-request.json"
        ))
        .unwrap()
    }
    fn packet(request: Value, bulk: bool) -> Value {
        let compiled = if bulk {
            crate::raw_bulk::compile_json(&request.to_string())
        } else {
            crate::raw_action::compile_json(&request.to_string())
        };
        let program: Value = serde_json::from_str(&compiled).unwrap();
        assert_eq!(program["status"], "compiled", "{program}");
        json!({"version":"weft.native-privilege-probes/0.1","request":request,"program":program,"probeTable":"__weft_privilege_mutation"})
    }
    #[test]
    fn original_action_branches_and_nonexecuting_update_insert_rules_are_closed() {
        for (index, name) in [
            (0, "approve"),
            (1, "guarded"),
            (2, "postFail"),
            (3, "createOrder"),
            (4, "firstFalse"),
        ] {
            let mut request = action();
            request["source"]["action"] = json!(name);
            request["source"]["actionPointer"] =
                json!(format!("/modules/0/extensions/umf.actions/actions/{index}"));
            let p = packet(request, false);
            let q = build(&p.to_string()).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&check_json(&q.to_string())).unwrap(),
                q
            );
            for statement in q["statements"].as_array().unwrap() {
                assert_eq!(
                    p["program"]
                        .pointer(statement["path"].as_str().unwrap())
                        .unwrap(),
                    &statement["sql"]
                );
                for probe in statement["probes"].as_array().unwrap() {
                    assert_eq!(probe["executeBusinessSql"], false);
                    if statement["role"] == "action-effect" {
                        assert_eq!(probe["analysisKind"], "pg-rewrite-rule/0.1");
                        assert_eq!(probe["analysisRule"]["neverFire"], true);
                        assert_eq!(
                            probe["analysisRule"]["expectedBusinessCommandType"],
                            if name == "createOrder" { 3 } else { 2 }
                        );
                        let setup = probe["setupStatements"].as_array().unwrap();
                        assert!(
                            setup[setup.len() - 2]
                                .as_str()
                                .unwrap()
                                .ends_with("WITH NO DATA")
                        );
                        assert!(
                            setup
                                .last()
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .starts_with("CREATE RULE ")
                        );
                    }
                }
            }
            assert_eq!(q["complete"], false);
            assert_eq!(q["locks"][0]["tablePrivilege"], Value::Null);
            let mut forged = q.clone();
            forged["statements"][0]["sql"] = json!("SELECT 1");
            assert_eq!(
                serde_json::from_str::<Value>(&check_json(&forged.to_string())).unwrap()["status"],
                "blocked"
            );
        }
    }
    #[test]
    fn bulk_inventories_only_actual_preview_selection_update_and_key_array_protocol() {
        let action = action();
        let request = json!({"version":"weft.native-bulk/0.1","selection":{"version":"weft.raw-read/0.1","target":"postgresql18","sql":"SELECT o.id AS id,o.status AS status FROM sales.order o ORDER BY o.id ASC","binding":action["binding"],"parameters":[],"rowCap":4},"action":action,"maxItems":3,"actionInputs":{},"parameters":[]});
        let p = packet(request, true);
        let q = build(&p.to_string()).unwrap();
        let statements = q["statements"].as_array().unwrap();
        assert_eq!(statements.len(), 4);
        assert_eq!(statements[3]["path"], "/update/sql");
        assert_eq!(statements[3]["parameters"][1]["container"], "list");
        assert!(
            statements[3]["probes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["inputRelation"]["columns"][1]["nativeType"] == "pg_catalog.text[]")
        );
        assert_eq!(
            serde_json::from_str::<Value>(&check_json(&q.to_string())).unwrap(),
            q
        );
        assert!(
            !statements
                .iter()
                .any(|s| s["path"].as_str().unwrap().contains("actionTemplate"))
        );
    }
}
