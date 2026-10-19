// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{json, Value};
fn source() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/raw-postgresql/table-operation-model.json"
    ))
    .unwrap()
}
fn request(id: &str) -> Value {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/raw-postgresql/table-operation-model.json"
    ))
    .unwrap();
    let template: Value = serde_json::from_str(include_str!(
        "../../../tests/raw-postgresql/table-lifecycle-read-template.json"
    ))
    .unwrap();
    let mut binding = template["binding"].clone();
    binding["modelJson"] = json!(source);
    binding["modelSha256"] = json!(weft_core::json::sha256(source.as_bytes()));
    let inputs = if id.starts_with("read_") {
        json!({"owner":{"string":"alice"}})
    } else if id == "remove" {
        json!({"id":{"string":"n1"}})
    } else {
        json!({"id":{"string":"n1"},"owner":{"string":"alice"}})
    };
    let call = if id == "create_batch" {
        json!({"kind":"batch","items":[inputs.clone(),inputs]})
    } else {
        json!({"kind":"single","inputs":inputs})
    };
    let mut r = json!({"version":"weft.table-operation/0.1","target":"postgresql18","module":"demo","operation":id,"binding":binding,"call":call});
    if !id.starts_with("read_") {
        r["idempotencyKey"] = json!("synthetic-replay-key")
    };
    r
}
fn repin(r: &mut Value, source: Value) {
    let text = serde_json::to_string(&source).unwrap();
    r["binding"]["modelJson"] = json!(text);
    r["binding"]["modelSha256"] = json!(weft_core::json::sha256(text.as_bytes()));
}
fn compile(r: Value) -> Value {
    serde_json::from_str(&weft_postgresql::table_operation::compile_json(
        &r.to_string(),
    ))
    .unwrap()
}
#[test]
fn all_declared_read_write_conflict_and_batch_shapes_compile_without_native_authority() {
    for id in [
        "read_many",
        "read_one",
        "read_optional",
        "read_page",
        "read_count",
        "create",
        "rename",
        "remove",
        "put_ignore",
        "put_update",
        "create_batch",
    ] {
        let r = request(id);
        let p = compile(r.clone());
        assert_eq!(p["status"], "compiled", "{id}: {p}");
        assert_eq!(p["version"], "weft.table-operation/0.1");
        assert_eq!(p["binding"], r["binding"]);
        assert_eq!(p["obligations"]["nativeExecutionQualified"], false);
        assert_eq!(p["obligations"]["transaction"]["writeLimitClipping"], false);
        let checked: Value = serde_json::from_str(&weft_postgresql::table_operation::check_json(
            &p.to_string(),
        ))
        .unwrap();
        assert_eq!(checked["status"], "checked", "{id}: {checked}");
        assert_eq!(
            p["parameterOrder"].as_array().unwrap().len(),
            p["parameterSources"].as_array().unwrap().len()
        );
    }
}
#[test]
fn source_original_sql_is_not_a_request_surface_and_legacy_guards_stay_closed() {
    let mut r = request("read_many");
    r["sql"] = json!("SELECT secret");
    assert_eq!(compile(r)["status"], "blocked");
    let mut r: Value = serde_json::from_str(include_str!(
        "../../../tests/raw-postgresql/table-lifecycle-read-template.json"
    ))
    .unwrap();
    let original = request("read_many");
    r["binding"] = original["binding"].clone();
    let p: Value =
        serde_json::from_str(&weft_postgresql::raw_read::compile_json(&r.to_string())).unwrap();
    assert_eq!(p["status"], "blocked");
}
#[test]
fn module_operation_mapping_inputs_and_replay_are_source_checked() {
    for pointer in [
        "/module",
        "/operation",
        "/binding/nativeLifecycle/module",
        "/binding/tables/0/native",
    ] {
        let mut r = request("create");
        *r.pointer_mut(pointer).unwrap() = json!("forged");
        assert_eq!(compile(r)["status"], "blocked", "{pointer}");
    }
    let mut missing = request("create");
    missing["call"]["inputs"]
        .as_object_mut()
        .unwrap()
        .remove("owner");
    assert_eq!(compile(missing)["status"], "blocked");
    let mut extra = request("create");
    extra["call"]["inputs"]["extra"] = json!({"string":"x"});
    assert_eq!(compile(extra)["status"], "blocked");
    let mut wrong = request("create");
    wrong["call"]["inputs"]["owner"] = json!({"integerToken":"1"});
    assert_eq!(compile(wrong)["status"], "blocked");
    for key in [Value::Null, json!(""), json!("x".repeat(257))] {
        let mut r = request("create");
        r["idempotencyKey"] = key;
        assert_eq!(compile(r)["status"], "blocked");
    }
    let mut read = request("read_many");
    read["idempotencyKey"] = json!("forged");
    assert_eq!(compile(read)["status"], "blocked");
}
#[test]
fn keyset_continuation_uses_exact_original_key_tuple_and_lookahead() {
    let first = compile(request("read_page"));
    assert!(first["sql"].as_str().unwrap().ends_with("LIMIT 101"));
    let mut r = request("read_page");
    r["call"]["after"] = json!([{"string":"n1"}]);
    let next = compile(r);
    assert_eq!(next["status"], "compiled", "{next}");
    assert_ne!(next["sql"], first["sql"]);
    assert!(next["parameterSources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["source"]["kind"] == "after"));
    for after in [
        json!([]),
        json!([{"string":"n1"},{"string":"n2"}]),
        json!([null]),
        json!([{"integerToken":"1"}]),
    ] {
        let mut r = request("read_page");
        r["call"]["after"] = after;
        assert_eq!(compile(r)["status"], "blocked");
    }
    let mut r = request("read_many");
    r["call"]["after"] = json!([{"string":"n1"}]);
    assert_eq!(compile(r)["status"], "blocked");
}
#[test]
fn literals_and_user_data_are_only_bound_values_and_native_text_capacity_is_exact() {
    let mut r = request("create");
    r["call"]["inputs"]["owner"] = json!({"string":"O'Reilly %_; DROP TABLE never;--"});
    let p = compile(r);
    assert_eq!(p["status"], "compiled", "{p}");
    assert!(!p["sql"].as_str().unwrap().contains("O'Reilly"));
    assert!(p["dbapiSql"].as_str().unwrap().contains("%s"));
    let mut d = source();
    d["modules"][0]["extensions"]["umf.tables"]["tables"][0]["columns"][1]["nativeType"] =
        json!({"kind":"varchar","length":2});
    for (text, status) in [
        ("😀é", "compiled"),
        ("abc", "blocked"),
        ("ab ", "blocked"),
        ("a\0", "blocked"),
    ] {
        let mut r = request("create");
        repin(&mut r, d.clone());
        r["call"]["inputs"]["owner"] = json!({"string":text});
        assert_eq!(compile(r)["status"], status, "{text:?}");
    }
}
#[test]
fn source_command_tampering_and_self_rehashed_programs_refuse() {
    for pointer in [
        "/future",
        "/body/future",
        "/body/values/0/future",
        "/parameters/0/future",
    ] {
        let mut r = request("create");
        let mut d = source();
        let command = &mut d["modules"][0]["extensions"]["umf.tables"]["commands"][5];
        if pointer == "/future" {
            command["future"] = json!(true)
        } else {
            let parent = pointer.rsplit_once('/').unwrap().0;
            command.pointer_mut(parent).unwrap()["future"] = json!(true)
        }
        repin(&mut r, d);
        assert_eq!(compile(r)["status"], "blocked", "{pointer}");
    }
    let p = compile(request("rename"));
    for key in [
        "sql",
        "module",
        "operation",
        "obligations",
        "parameterSources",
    ] {
        let mut changed = p.clone();
        changed[key] = json!("forged");
        changed.as_object_mut().unwrap().remove("structuralDigest");
        changed["structuralDigest"] = json!(weft_core::json::sha256(
            weft_core::mutation::canonical(&changed).as_bytes()
        ));
        let checked: Value = serde_json::from_str(&weft_postgresql::table_operation::check_json(
            &changed.to_string(),
        ))
        .unwrap();
        assert_eq!(checked["status"], "blocked", "{key}");
    }
}
#[test]
fn bulk_is_one_declared_operation_with_explicit_atomic_bounds_and_no_hidden_write_limit() {
    let p = compile(request("create_batch"));
    assert_eq!(p["status"], "compiled", "{p}");
    assert_eq!(p["obligations"]["transaction"]["batch"], true);
    assert!(!p["sql"].as_str().unwrap().contains("LIMIT"));
    assert!(p["sql"].as_str().unwrap().contains("RETURNING"));
    for items in [json!([]), json!([{"id":{"string":"n1"}}])] {
        let mut r = request("create_batch");
        r["call"]["items"] = items;
        assert_eq!(compile(r)["status"], "blocked");
    }
    let mut r = request("create");
    r["call"] = request("create_batch")["call"].clone();
    assert_eq!(compile(r)["status"], "blocked");
    let mut r = request("create_batch");
    let row = r["call"]["items"][0].clone();
    r["call"]["items"] = json!(vec![row; 101]);
    assert_eq!(compile(r)["status"], "blocked");
}

#[test]
fn exact_integer_decimal_and_noncanonical_json_integer_bounds_never_round_or_panic() {
    for (family, native, facets, carrier, values) in [
        (
            "integer",
            json!({"kind":"int2"}),
            json!({"integerWidth":{"bits":16,"signed":true}}),
            "integer",
            vec![
                ("-32768", true),
                ("32767", true),
                ("32768", false),
                ("-32769", false),
                ("1e0", true),
                ("1.5", false),
            ],
        ),
        (
            "decimal",
            json!({"kind":"numeric","precision":5,"scale":2}),
            json!({"precision":5,"scale":2}),
            "decimal",
            vec![
                ("999.99", true),
                ("-999.99", true),
                ("1.230", true),
                ("1000", false),
                ("1.234", false),
            ],
        ),
    ] {
        let mut d = source();
        d["modules"][0]["elements"][0]["scalarType"] = json!(family);
        d["modules"][0]["elements"][0]["facets"] = facets;
        d["modules"][0]["extensions"]["umf.tables"]["tables"][0]["columns"][0]["nativeType"] =
            native;
        for (token, accepted) in values {
            let mut r = request("create");
            repin(&mut r, d.clone());
            r["binding"]["tables"][0]["columns"][0]["carrier"] = json!(carrier);
            r["call"]["inputs"]["id"] = if family == "integer" {
                json!({"integerToken":token})
            } else {
                json!({"decimalToken":token})
            };
            let p = compile(r);
            assert_eq!(
                p["status"],
                if accepted { "compiled" } else { "blocked" },
                "{family} {token}: {p}"
            );
        }
    }
    let mut r = request("read_page");
    let d = serde_json::to_string(&source())
        .unwrap()
        .replace("\"maxRows\":100", "\"maxRows\":1e2");
    r["binding"]["modelJson"] = json!(d);
    r["binding"]["modelSha256"] = json!(weft_core::json::sha256(d.as_bytes()));
    let p = compile(r);
    assert_eq!(p["status"], "compiled", "{p}");
    assert!(p["sql"].as_str().unwrap().ends_with("LIMIT 101"));
    assert_eq!(p["result"]["maxRows"].as_u64(), Some(100));
    assert_eq!(p["declaration"]["body"]["maxRows"].as_u64(), Some(100));
    assert!(p["binding"]["modelJson"].as_str().unwrap().contains("1e2"));
}

#[test]
fn driver_numeric_values_are_canonical_without_losing_original_tokens() {
    for (family, native, facets, carrier, token, expected) in [
        (
            "integer",
            json!({"kind":"int2"}),
            json!({"integerWidth":{"bits":16,"signed":true}}),
            "integer",
            "0e999999999999999999999999999",
            "0",
        ),
        (
            "decimal",
            json!({"kind":"numeric","precision":5,"scale":2}),
            json!({"precision":5,"scale":2}),
            "decimal",
            "0e-999999999999999999999999999",
            "0",
        ),
        (
            "decimal",
            json!({"kind":"numeric","precision":5,"scale":2}),
            json!({"precision":5,"scale":2}),
            "decimal",
            "1.2300",
            "123e-2",
        ),
    ] {
        let mut d = source();
        d["modules"][0]["elements"][0]["scalarType"] = json!(family);
        d["modules"][0]["elements"][0]["facets"] = facets;
        d["modules"][0]["extensions"]["umf.tables"]["tables"][0]["columns"][0]["nativeType"] =
            native;
        let mut r = request("create");
        repin(&mut r, d);
        r["binding"]["tables"][0]["columns"][0]["carrier"] = json!(carrier);
        r["call"]["inputs"]["id"] = if family == "integer" {
            json!({"integerToken":token})
        } else {
            json!({"decimalToken":token})
        };
        let p = compile(r.clone());
        assert_eq!(p["status"], "compiled", "{p}");
        assert_eq!(p["nativeInputs"][0]["id"], expected);
        assert!(p["requestJson"].as_str().unwrap().contains(token));
        let checked: Value = serde_json::from_str(&weft_postgresql::table_operation::check_json(
            &p.to_string(),
        ))
        .unwrap();
        assert_eq!(checked["status"], "checked");
    }
}

#[test]
fn requested_read_size_can_only_lower_a_declared_many_or_page_bound() {
    for id in ["read_many", "read_page"] {
        let mut r = request(id);
        r["call"]["size"] = json!(2);
        let p = compile(r);
        assert_eq!(p["status"], "compiled", "{p}");
        assert_eq!(p["result"]["maxRows"], 2);
        assert_eq!(p["result"]["declaredMaxRows"], 100);
        assert!(p["sql"].as_str().unwrap().ends_with("LIMIT 3"));
    }
    for (id, size) in [
        ("read_page", 0),
        ("read_page", 101),
        ("read_one", 1),
        ("read_optional", 1),
        ("read_count", 1),
        ("create", 1),
        ("rename", 1),
    ] {
        let mut r = request(id);
        r["call"]["size"] = json!(size);
        assert_eq!(compile(r)["status"], "blocked", "{id}/{size}");
    }
}
