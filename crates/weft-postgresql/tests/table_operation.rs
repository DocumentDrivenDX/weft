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
fn prepared_operation_module_reuses_source_and_returns_compact_call_programs() {
    let first = request("read_many");
    let source = json!({
        "version":"weft.table-operation-module/0.1",
        "target":first["target"],
        "module":first["module"],
        "binding":first["binding"],
    });
    let module =
        weft_postgresql::table_operation::PreparedOperationModule::prepare(&source.to_string())
            .unwrap();
    let operation = module.prepare_operation("read_many").unwrap();
    let bind = |inputs: Value| json!({"call":{"kind":"single","inputs":inputs}}).to_string();
    let compiled: Value =
        serde_json::from_str(&operation.bind_json(&bind(json!({"owner":{"string":"alice"}}))))
            .unwrap();
    let changed: Value =
        serde_json::from_str(&operation.bind_json(&bind(json!({"owner":{"string":"bob"}}))))
            .unwrap();
    assert_eq!(compiled["status"], "compiled");
    assert_eq!(compiled["sql"], changed["sql"]);
    assert_eq!(compiled["nativeInputs"][0]["owner"], json!("alice"));
    assert_eq!(changed["nativeInputs"][0]["owner"], json!("bob"));
    assert_eq!(
        compiled["binding"],
        json!({"modelSha256":first["binding"]["modelSha256"]})
    );
    for retired in [
        "requestJson",
        "requestSha256",
        "declaration",
        "structuralDigest",
    ] {
        assert!(
            compiled.get(retired).is_none(),
            "compact program retained {retired}"
        );
    }
    assert_eq!(compiled["callKind"], "single");
    assert_eq!(compiled["declarationDigest"].as_str().unwrap().len(), 64);
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
    assert!(first["sql"].as_str().unwrap().contains("LIMIT 101)"));
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
fn plain_insert_batch_emits_ordered_insert_only_merge_with_source_ordinals() {
    let r = request("create_batch");
    let program = compile(r.clone());
    assert_eq!(program["status"], "compiled", "{program}");
    let batch = &program["nativeBatch"];
    assert_eq!(batch["version"], "weft.table-insert-batch/0.1");
    assert_eq!(batch["ordinalColumn"], "umf_input_ordinal");
    assert_eq!(batch["parameterOrder"], json!(["nativeRows"]));
    assert_eq!(batch["dbapiParameterOrder"], json!(["nativeRows"]));
    let sql = batch["sql"].as_str().unwrap();
    assert!(sql.contains("jsonb_array_elements($1::pg_catalog.jsonb) WITH ORDINALITY"));
    assert!(sql.contains("MERGE INTO"));
    assert!(sql.contains("ON FALSE WHEN NOT MATCHED THEN INSERT"));
    assert!(sql.contains("ORDER BY umf_input_ordinal"));
    assert!(sql.contains("RETURNING i.umf_input_ordinal AS umf_input_ordinal"));
    assert!(sql.contains("pg_catalog.count(*) AS effects"));
    assert!(!sql.contains("count(DISTINCT"));
    assert!(sql.contains("umf_input_ordinal"));
    assert!(!sql.contains("WHEN MATCHED"));
    assert_eq!(
        serde_json::from_str::<Value>(&weft_postgresql::table_operation::check_json(
            &program.to_string()
        ))
        .unwrap()["status"],
        "checked"
    );

    let mut no_key_output = r;
    let mut source: Value =
        serde_json::from_str(no_key_output["binding"]["modelJson"].as_str().unwrap()).unwrap();
    let command = source["modules"][0]["extensions"]["umf.tables"]["commands"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|item| item["id"] == "create_batch")
        .unwrap();
    command["body"]["returning"] = json!([{"module":"demo","element":"owner"}]);
    repin(&mut no_key_output, source);
    let no_key_program = compile(no_key_output);
    assert_eq!(no_key_program["status"], "compiled", "{no_key_program}");
    assert!(no_key_program.get("nativeBatch").is_some());
    assert!(no_key_program["nativeBatch"]["sql"]
        .as_str()
        .unwrap()
        .contains("RETURNING i.umf_input_ordinal AS umf_input_ordinal"));
    assert!(!no_key_program["nativeBatch"]["sql"]
        .as_str()
        .unwrap()
        .contains("umf_batch_key_"));

    let mut literal_batch = request("create_batch");
    let mut literal_source: Value =
        serde_json::from_str(literal_batch["binding"]["modelJson"].as_str().unwrap()).unwrap();
    let command = literal_source["modules"][0]["extensions"]["umf.tables"]["commands"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|item| item["id"] == "create_batch")
        .unwrap();
    command["parameters"]
        .as_array_mut()
        .unwrap()
        .retain(|parameter| parameter["id"] != "owner");
    command["body"]["values"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|assignment| assignment["field"]["element"] == "owner")
        .unwrap()["value"] = json!({"kind":"literal","value":{"string":"fixed"}});
    repin(&mut literal_batch, literal_source);
    for item in literal_batch["call"]["items"].as_array_mut().unwrap() {
        item.as_object_mut().unwrap().remove("owner");
    }
    let literal_program = compile(literal_batch);
    assert_eq!(literal_program["status"], "compiled", "{literal_program}");
    assert_eq!(
        literal_program["nativeBatch"]["parameterOrder"],
        json!(["nativeRows", "nativeLiterals"])
    );
    assert_eq!(
        literal_program["nativeBatch"]["dbapiParameterOrder"],
        json!(["nativeRows", "nativeLiterals"])
    );
    let literal_sql = literal_program["nativeBatch"]["sql"].as_str().unwrap();
    assert!(literal_sql.contains("CROSS JOIN umf_batch_literals AS l"));
    assert!(literal_sql.contains("VALUES (i.\"slot0\", i.\"slot1\")"));

    let mut empty_returning = request("create_batch");
    let mut empty_source: Value =
        serde_json::from_str(empty_returning["binding"]["modelJson"].as_str().unwrap()).unwrap();
    let command = empty_source["modules"][0]["extensions"]["umf.tables"]["commands"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|item| item["id"] == "create_batch")
        .unwrap();
    command["body"]["returning"] = json!([]);
    repin(&mut empty_returning, empty_source);
    let empty_program = compile(empty_returning);
    assert_eq!(empty_program["status"], "compiled", "{empty_program}");
    assert!(empty_program.get("nativeBatch").is_some());
    let empty_sql = empty_program["nativeBatch"]["sql"].as_str().unwrap();
    assert!(!empty_sql.contains(", ,"));
    assert!(empty_sql.contains("SELECT m.umf_input_ordinal, m.umf_affected FROM"));
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
    assert!(p["sql"].as_str().unwrap().contains("LIMIT 101)"));
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
        assert!(p["sql"].as_str().unwrap().contains("LIMIT 3)"));
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

#[test]
fn read_frames_bound_uncompressed_bytes_and_preserve_unprojected_order() {
    for id in [
        "read_many",
        "read_one",
        "read_optional",
        "read_page",
        "read_count",
    ] {
        let p = compile(request(id));
        assert_eq!(p["status"], "compiled", "{p}");
        assert_eq!(
            p["result"]["nativeReadGuard"],
            json!({
                "profile":"pg18-read-frame/0.1", "maxNativeBytes":8388608,
                "sqlstate":"22012", "includesLookahead":true, "partialFrames":false
            })
        );
        let sql = p["sql"].as_str().unwrap();
        assert!(sql.starts_with("WITH umf_read AS MATERIALIZED ("));
        assert!(sql.contains("pg_catalog.octet_length("));
        assert!(!sql.contains("pg_column_size"));
        assert!(sql.contains("pg_catalog.count(*) - pg_catalog.count(*)"));
    }
    let mut r = request("read_many");
    let mut model = source();
    model["modules"][0]["extensions"]["umf.tables"]["commands"][0]["body"]["fields"] =
        json!([{"module":"demo","element":"owner"}]);
    repin(&mut r, model);
    let p = compile(r);
    assert_eq!(p["status"], "compiled", "{p}");
    assert_eq!(p["outputs"].as_array().unwrap().len(), 1);
    let sql = p["sql"].as_str().unwrap();
    assert!(sql.contains("t.\"id\" AS \"umf_sort_0\""));
    assert!(sql.ends_with("ORDER BY r.\"umf_sort_0\" ASC"));
    assert!(compile(request("create"))["result"]["nativeReadGuard"].is_null());
}

#[test]
fn mutation_frames_bound_returning_octets_and_effects_atomically() {
    for id in [
        "create",
        "rename",
        "remove",
        "put_ignore",
        "put_update",
        "create_batch",
    ] {
        let p = compile(request(id));
        assert_eq!(p["status"], "compiled", "{p}");
        assert_eq!(
            p["result"]["nativeWriteFrameGuard"],
            json!({
                "profile":"pg18-write-frame/0.1","maxNativeBytes":8388608,
                "sqlstate":"22012","includesLookahead":false,"partialFrames":false
            })
        );
        let sql = p["sql"].as_str().unwrap();
        assert!(sql.contains("AND COALESCE(pg_catalog.sum("));
        assert!(sql.contains("pg_catalog.octet_length("));
        assert!(sql.contains("<= 8388608"));
        assert!(!sql.contains("pg_column_size"));
        assert!(!sql.contains(" LIMIT "));
    }
    assert!(compile(request("read_many"))["result"]["nativeWriteFrameGuard"].is_null());
}

#[test]
fn temporal_operations_are_exact_and_do_not_expand_raw_read_admission() {
    let mut r = request("create");
    let mut source = source();
    for (id, family, native) in [("day", "date", "date"), ("at", "timestamp", "timestamptz")] {
        let reference = json!({"module":"demo","element":id});
        source["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":id,"kind":"field","scalarType":family,"cardinality":"one","nullability":"required","extensions":{}}));
        source["modules"][0]["elements"][2]["members"]
            .as_array_mut()
            .unwrap()
            .push(reference.clone());
        source["modules"][0]["extensions"]["umf.tables"]["tables"][0]["columns"]
            .as_array_mut()
            .unwrap()
            .push(json!({"field":reference,"name":id,"nativeType":{"kind":native}}));
        r["binding"]["tables"][0]["columns"]
            .as_array_mut()
            .unwrap()
            .push(json!({"logical":id,"native":id,"carrier":family,"nullable":false}));
        let op = source["modules"][0]["extensions"]["umf.tables"]["commands"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|v| v["id"] == "create")
            .unwrap();
        op["parameters"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":id,"field":reference}));
        op["body"]["values"]
            .as_array_mut()
            .unwrap()
            .push(json!({"field":reference,"value":{"kind":"parameter","parameter":id}}));
        op["body"]["returning"]
            .as_array_mut()
            .unwrap()
            .push(reference);
    }
    repin(&mut r, source);
    r["call"]["inputs"]["day"] = json!({"date":"2000-02-29"});
    r["call"]["inputs"]["at"] = json!({"timestamp":"2026-01-01T00:30:00.123456000+01:00"});
    let program = compile(r.clone());
    assert_eq!(program["status"], "compiled", "{program}");
    assert_eq!(
        program["nativeInputs"][0]["at"],
        "2025-12-31T23:30:00.123456Z"
    );
    assert_eq!(program["nativeInputs"][0]["day"], "2000-02-29");
    assert!(program["sql"]
        .as_str()
        .unwrap()
        .contains("AT TIME ZONE 'UTC'"));
    for (field, value) in [
        ("day", json!({"date":"1900-02-29"})),
        ("day", json!({"date":"infinity"})),
        ("at", json!({"timestamp":"2026-01-01T00:00:00.1234567Z"})),
        ("at", json!({"timestamp":"2026-01-01T00:00:00-00:00"})),
    ] {
        let mut bad = r.clone();
        bad["call"]["inputs"][field] = value;
        assert_ne!(compile(bad)["status"], "compiled");
    }
    let raw = json!({"version":"weft.raw-read/0.1","target":"postgresql18","binding":r["binding"],"sql":"SELECT day FROM demo.Note","parameters":[],"maxRows":10,"keyset":false,"snapshot":false});
    assert!(weft_core::raw_read::admit_json(&raw.to_string()).is_err());
    let bound: weft_core::raw_read::Binding = serde_json::from_value(r["binding"].clone()).unwrap();
    assert!(weft_core::raw_read::binding_record(&bound, &["demo".into(), "Note".into()]).is_err());
}

#[test]
fn declared_domains_enforce_original_enumerations_ranges_and_literal_inputs() {
    let mut r = request("create");
    let mut model = source();
    model["modules"][0]["elements"][1]["allowedValues"] =
        json!([{"string":"alice"},{"string":"bob"}]);
    model["modules"][0]["elements"][1]["facets"] =
        json!({"length":{"unit":"unicode-scalar","min":1,"max":8}});
    repin(&mut r, model);
    assert_eq!(compile(r.clone())["status"], "compiled");
    r["call"]["inputs"]["owner"] = json!({"string":"other"});
    assert_eq!(compile(r.clone())["status"], "blocked");
    let binding: weft_core::raw_read::Binding =
        serde_json::from_value(r["binding"].clone()).unwrap();
    assert!(
        weft_core::raw_read::binding_record(&binding, &["demo".into(), "Note".into()]).is_err()
    );
}

#[test]
fn transaction_clock_values_are_typed_and_never_become_input_slots() {
    let mut r = request("create");
    let mut model = source();
    let reference = json!({"module":"demo","element":"at"});
    model["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":"at","kind":"field","scalarType":"timestamp","cardinality":"one","nullability":"required","extensions":{}}));
    model["modules"][0]["elements"][2]["members"]
        .as_array_mut()
        .unwrap()
        .push(reference.clone());
    model["modules"][0]["extensions"]["umf.tables"]["tables"][0]["columns"]
        .as_array_mut()
        .unwrap()
        .push(json!({"field":reference,"name":"at","nativeType":{"kind":"timestamptz"}}));
    r["binding"]["tables"][0]["columns"]
        .as_array_mut()
        .unwrap()
        .push(json!({"logical":"at","native":"at","carrier":"timestamp","nullable":false}));
    let command = model["modules"][0]["extensions"]["umf.tables"]["commands"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["id"] == "create")
        .unwrap();
    command["body"]["values"]
        .as_array_mut()
        .unwrap()
        .push(json!({"field":reference,"value":{"kind":"currentTimestamp"}}));
    command["body"]["returning"]
        .as_array_mut()
        .unwrap()
        .push(reference);
    repin(&mut r, model.clone());
    let program = compile(r.clone());
    assert_eq!(program["status"], "compiled", "{program}");
    assert!(program["sql"]
        .as_str()
        .unwrap()
        .contains("CURRENT_TIMESTAMP"));
    assert_eq!(program["nativeInputs"][0].as_object().unwrap().len(), 2);
    let command = model["modules"][0]["extensions"]["umf.tables"]["commands"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["id"] == "create")
        .unwrap();
    command["body"]["values"][2]["field"] = json!({"module":"demo","element":"owner"});
    repin(&mut r, model);
    assert_eq!(compile(r)["status"], "blocked");
}

#[test]
fn batch_principal_is_native_and_has_no_caller_slot() {
    let mut r = request("create_batch");
    let mut model = source();
    let command = model["modules"][0]["extensions"]["umf.tables"]["commands"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["id"] == "create_batch")
        .unwrap();
    command["parameters"] = json!([{"id":"id","field":{"module":"demo","element":"id"}}]);
    command["body"]["values"][1]["value"] = json!({"kind":"currentPrincipal"});
    repin(&mut r, model);
    r["call"]["items"] = json!([{"id":{"string":"a"}},{"id":{"string":"b"}}]);
    let p = compile(r.clone());
    assert_eq!(p["status"], "compiled", "{p}");
    assert!(p["nativeBatch"]["sql"]
        .as_str()
        .unwrap()
        .contains("CURRENT_USER::pg_catalog.text"));
    assert_eq!(p["nativeBatch"]["parameterOrder"], json!(["nativeRows"]));
    r["call"]["items"][0]["owner"] = json!({"string":"forged"});
    assert_eq!(compile(r)["status"], "blocked");
}

#[test]
fn mixed_direction_cursor_carries_complete_order_with_original_key_suffix() {
    let mut r = request("read_page");
    let mut model = source();
    let command = model["modules"][0]["extensions"]["umf.tables"]["commands"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["id"] == "read_page")
        .unwrap();
    command["body"]["orderBy"] = json!([{"field":{"module":"demo","element":"owner"},"direction":"desc"},{"field":{"module":"demo","element":"id"},"direction":"asc"}]);
    repin(&mut r, model);
    r["call"]["after"] = json!([{"string":"alice"},{"string":"n1"}]);
    let p = compile(r.clone());
    assert_eq!(p["status"], "compiled", "{p}");
    assert_eq!(p["nativeAfter"], json!(["alice", "n1"]));
    assert_eq!(p["result"]["cursor"]["directions"], json!(["desc", "asc"]));
    let sql = p["sql"].as_str().unwrap();
    assert!(sql.contains(" < "));
    assert!(sql.contains(" > "));
    assert!(sql.contains(" OR "));
    assert!(sql.contains(" DESC"));
    r["call"]["after"] = json!([{"string":"n1"}]);
    assert_eq!(compile(r)["status"], "blocked");
}

fn g02_request(expression: Value) -> Value {
    let mut r = request("rename");
    let mut model: Value =
        serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();
    let commands = model["modules"][0]["extensions"]["umf.tables"]["commands"]
        .as_array_mut()
        .unwrap();
    commands.push(json!({
        "version":"umf.table-operation/0.2",
        "id":"backfill_owner",
        "parameters":[],
        "consistency":"transaction",
        "replay":"caller-key-required",
        "body":{"kind":"update","table":"notes","values":[{"field":{"module":"demo","element":"owner"},"value":{"kind":"rowExpression","expression":expression}}],"where":{"op":"isNull","field":{"module":"demo","element":"owner"}},"returning":[],"maxRows":100}
    }));
    repin(&mut r, model);
    r["version"] = json!("weft.table-operation/0.2");
    r["operation"] = json!("backfill_owner");
    r["call"] = json!({"kind":"single","inputs":{}});
    r["idempotencyKey"] = json!("g02-replay-key");
    r
}

#[test]
fn g02_row_expression_compiles_closed_ast_with_private_full_effects_guard() {
    let expression = json!({"op":"concat","args":[
        {"op":"field","field":{"module":"demo","element":"id"}},
        {"op":"literal","value":{"string":" "}},
        {"op":"literal","value":{"string":"x"}}
    ]});
    let r = g02_request(expression);
    let p = compile(r.clone());
    assert_eq!(p["status"], "compiled", "{p}");
    assert_eq!(p["version"], "weft.table-operation/0.2");
    assert_eq!(
        p["result"]["nativeExactValueGuard"],
        json!({
            "profile":"pg18-row-expression-exact/0.1","sqlstate":"22012","scope":"all-direct-target-rows",
            "nullResult":"statement-error-and-transaction-rollback","nonfiniteInput":"statement-error-and-transaction-rollback",
            "lossyAssignment":"statement-error-and-transaction-rollback","privateResults":true
        })
    );
    let sql = p["sql"].as_str().unwrap();
    assert!(sql.contains("OPERATOR(pg_catalog.||)"), "{sql}");
    for qualified in [
        "OPERATOR(pg_catalog.<=)",
        "OPERATOR(pg_catalog.-)",
        "OPERATOR(pg_catalog./)",
        "OPERATOR(pg_catalog.=)",
    ] {
        assert!(
            sql.contains(qualified),
            "missing qualified G02 operator {qualified}: {sql}"
        );
    }
    assert!(
        sql.contains("umf_native_exact")
            && sql.contains("umf_native_nonnull")
            && sql.contains("bool_and"),
        "{sql}"
    );
    assert!(
        sql.ends_with(
            "SELECT e.umf_affected FROM umf_bound AS b CROSS JOIN umf_effects AS e WHERE b.ok OPERATOR(pg_catalog.=) 1"
        ),
        "{sql}"
    );
    assert_eq!(p["outputs"], json!([]));
    assert_eq!(p["status"], "compiled");
    let checked: Value = serde_json::from_str(&weft_postgresql::table_operation::check_json(
        &p.to_string(),
    ))
    .unwrap();
    assert_eq!(checked["status"], "checked", "{checked}");
    assert_eq!(p["version"], "weft.table-operation/0.2");
    assert_eq!(p["binding"], r["binding"]);
}

#[test]
fn g02_refuses_target_self_reference_old_dispatch_and_bad_ast_shapes() {
    let target = json!({"module":"demo","element":"owner"});
    for expression in [
        json!({"op":"field","field":target.clone()}),
        json!({"op":"rawSql","sql":"x"}),
        json!({"op":"concat","args":[{"op":"field","field":{"module":"demo","element":"id"}}]}),
        json!({"op":"concat","args":[{"op":"field","field":{"module":"demo","element":"id"}},{"op":"literal","value":{"string":"\0"}}]}),
        json!({"op":"conditional","when":{"op":"isNull","field":target.clone()},"then":{"op":"field","field":{"module":"demo","element":"id"}},"else":{"op":"field","field":{"module":"demo","element":"id"}}}),
        json!({"op":"literal","value":{"date":"2025-02-30"}}),
        json!({"op":"literal","value":{"uuid":"not-a-uuid"}}),
    ] {
        assert_eq!(compile(g02_request(expression))["status"], "blocked");
    }
    let mut wrong = g02_request(json!({"op":"field","field":{"module":"demo","element":"id"}}));
    wrong["version"] = json!("weft.table-operation/0.1");
    assert_eq!(compile(wrong)["status"], "blocked");

    let null_result = compile(g02_request(json!({"op":"literal","value":null})));
    assert_eq!(
        null_result["status"], "compiled",
        "all-NULL expression uses target context and is checked at runtime"
    );
    assert!(null_result["sql"]
        .as_str()
        .unwrap()
        .contains("umf_native_nonnull"));
}

fn g02_numeric_request(expression: Value) -> Value {
    let mut r = request("rename");
    let mut model: Value =
        serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();
    let module = &mut model["modules"][0];
    let elements = module["elements"].as_array_mut().unwrap();
    elements.push(json!({"id":"sourceAmount","kind":"field","scalarType":"decimal","cardinality":"one","nullability":"required","facets":{"precision":10,"scale":2},"extensions":{}}));
    elements.push(json!({"id":"amount","kind":"field","scalarType":"decimal","cardinality":"one","nullability":"absent-allowed","facets":{"precision":12,"scale":2},"extensions":{}}));
    elements.push(json!({"id":"integerTarget","kind":"field","scalarType":"integer","cardinality":"one","nullability":"absent-allowed","extensions":{}}));
    elements.push(json!({"id":"narrowInteger","kind":"field","scalarType":"integer","cardinality":"one","nullability":"absent-allowed","extensions":{}}));
    elements.push(json!({"id":"unusedAmount","kind":"field","scalarType":"decimal","cardinality":"one","nullability":"required","facets":{"precision":8,"scale":2},"extensions":{}}));
    elements.iter_mut().find(|e| e["id"] == "Note").unwrap()["members"]
        .as_array_mut()
        .unwrap()
        .extend([
            json!({"module":"demo","element":"sourceAmount"}),
            json!({"module":"demo","element":"amount"}),
            json!({"module":"demo","element":"integerTarget"}),
            json!({"module":"demo","element":"narrowInteger"}),
            json!({"module":"demo","element":"unusedAmount"}),
        ]);
    let table = &mut module["extensions"]["umf.tables"]["tables"][0];
    table["columns"].as_array_mut().unwrap().extend([
        json!({"field":{"module":"demo","element":"sourceAmount"},"name":"source_amount","nativeType":{"kind":"numeric","precision":10,"scale":2}}),
        json!({"field":{"module":"demo","element":"amount"},"name":"amount","nativeType":{"kind":"numeric","precision":12,"scale":2}}),
        json!({"field":{"module":"demo","element":"integerTarget"},"name":"integer_target","nativeType":{"kind":"int4"}}),
        json!({"field":{"module":"demo","element":"narrowInteger"},"name":"narrow_integer","nativeType":{"kind":"int2"}}),
        json!({"field":{"module":"demo","element":"unusedAmount"},"name":"unused_amount","nativeType":{"kind":"numeric","precision":8,"scale":2}})
    ]);
    let columns = r["binding"]["tables"][0]["columns"].as_array_mut().unwrap();
    columns.extend([
        json!({"logical":"sourceAmount","native":"source_amount","carrier":"decimal","nullable":false}),
        json!({"logical":"amount","native":"amount","carrier":"decimal","nullable":true}),
        json!({"logical":"integerTarget","native":"integer_target","carrier":"integer","nullable":true}),
        json!({"logical":"narrowInteger","native":"narrow_integer","carrier":"integer","nullable":true}),
        json!({"logical":"unusedAmount","native":"unused_amount","carrier":"decimal","nullable":false})
    ]);
    module["extensions"]["umf.tables"]["commands"].as_array_mut().unwrap().push(json!({
        "version":"umf.table-operation/0.2","id":"backfill_amount","parameters":[],"consistency":"transaction","replay":"caller-key-required",
        "body":{"kind":"update","table":"notes","values":[{"field":{"module":"demo","element":"amount"},"value":{"kind":"rowExpression","expression":expression}}],"where":{"op":"isNull","field":{"module":"demo","element":"amount"}},"returning":[],"maxRows":100}
    }));
    module["extensions"]["umf.tables"]["commands"].as_array_mut().unwrap().push(json!({
        "version":"umf.table-operation/0.2","id":"backfill_narrow","parameters":[],"consistency":"transaction","replay":"caller-key-required",
        "body":{"kind":"update","table":"notes","values":[{"field":{"module":"demo","element":"narrowInteger"},"value":{"kind":"rowExpression","expression":expression}}],"where":{"op":"isNull","field":{"module":"demo","element":"narrowInteger"}},"returning":[],"maxRows":100}
    }));
    // An integer-target command exercises static integrality admission without
    // changing the independently compiled decimal assignment above.
    module["extensions"]["umf.tables"]["commands"].as_array_mut().unwrap().push(json!({
        "version":"umf.table-operation/0.2","id":"backfill_integer","parameters":[],"consistency":"transaction","replay":"caller-key-required",
        "body":{"kind":"update","table":"notes","values":[{"field":{"module":"demo","element":"integerTarget"},"value":{"kind":"rowExpression","expression":expression}}],"where":{"op":"isNull","field":{"module":"demo","element":"integerTarget"}},"returning":[],"maxRows":100}
    }));
    repin(&mut r, model);
    r["version"] = json!("weft.table-operation/0.2");
    r["operation"] = json!("backfill_amount");
    r["call"] = json!({"kind":"single","inputs":{}});
    r["idempotencyKey"] = json!("g02-numeric-replay-key");
    r
}

#[test]
fn g02_numeric_and_conditional_algebra_uses_unconstrained_qualified_intermediates() {
    let source_field = json!({"op":"field","field":{"module":"demo","element":"sourceAmount"}});
    let literal = json!({"op":"literal","value":{"decimalToken":"1.25"}});
    let cases = [
        source_field.clone(),
        json!({"op":"literal","value":{"decimalToken":"1.25"}}),
        json!({"op":"coalesce","args":[{"op":"literal","value":null},source_field.clone()]}),
        json!({"op":"add","left":source_field.clone(),"right":literal.clone()}),
        json!({"op":"subtract","left":source_field.clone(),"right":literal.clone()}),
        json!({"op":"multiply","left":source_field.clone(),"right":literal.clone()}),
        json!({"op":"conditional","when":{"op":"gt","field":{"module":"demo","element":"sourceAmount"},"value":{"kind":"literal","value":{"decimalToken":"0"}}},"then":source_field.clone(),"else":literal.clone()}),
        json!({"op":"conditional","when":{"op":"in","field":{"module":"demo","element":"sourceAmount"},"values":[{"kind":"literal","value":{"decimalToken":"1.00"}}]},"then":source_field.clone(),"else":{"op":"field","field":{"module":"demo","element":"unusedAmount"}}}),
    ];
    for (index, expression) in cases.into_iter().enumerate() {
        let mut r = g02_numeric_request(expression);
        r["operation"] = json!("backfill_amount");
        let p = compile(r);
        assert_eq!(p["status"], "compiled", "case {index}: {p}");
        let sql = p["sql"].as_str().unwrap();
        assert!(
            sql.contains("umf_native_finite") && sql.contains("bool_and"),
            "{sql}"
        );
        if index >= 3 {
            assert!(
                sql.contains("::pg_catalog.numeric") && sql.contains("OPERATOR(pg_catalog."),
                "{sql}"
            );
        }
        if index == 3 {
            assert!(
                sql.contains("OPERATOR(pg_catalog.+)"),
                "numeric addition must be explicitly qualified: {sql}"
            );
        }
    }
    let expression = json!({"op":"add","left":source_field,"right":literal});
    let mut r = g02_numeric_request(expression);
    r["operation"] = json!("backfill_integer");
    assert_eq!(
        compile(r)["status"],
        "blocked",
        "fractional arithmetic must not target an integer carrier"
    );

    let narrow = json!({"op":"subtract","left":{"op":"literal","value":{"integerToken":"100000"}},"right":{"op":"literal","value":{"integerToken":"99999"}}});
    let mut r = g02_numeric_request(narrow);
    r["operation"] = json!("backfill_narrow");
    let p = compile(r);
    assert_eq!(
        p["status"], "compiled",
        "wide operands whose result fits int2 must stay numeric until assignment: {p}"
    );
    assert!(p["sql"].as_str().unwrap().contains("::pg_catalog.numeric"));

    let unused = json!({"op":"conditional","when":{"op":"all"},"then":{"op":"field","field":{"module":"demo","element":"sourceAmount"}},"else":{"op":"field","field":{"module":"demo","element":"unusedAmount"}}});
    let mut r = g02_numeric_request(unused);
    r["operation"] = json!("backfill_amount");
    let p = compile(r);
    let sql = p["sql"].as_str().unwrap();
    assert_eq!(p["status"], "compiled", "{p}");
    assert!(
        sql.contains("t.\"unused_amount\"") && sql.contains("'NaN'::pg_catalog.numeric"),
        "all numeric refs must be finite, including unused conditional branches: {sql}"
    );

    let mut nested = json!({"op":"field","field":{"module":"demo","element":"sourceAmount"}});
    for _ in 0..12 {
        nested = json!({"op":"coalesce","args":[nested,{"op":"literal","value":null}]});
    }
    let mut r = g02_numeric_request(nested);
    r["operation"] = json!("backfill_amount");
    let p = compile(r);
    assert_eq!(p["status"], "compiled", "{p}");
    assert!(
        p["sql"].as_str().unwrap().len() < 30_000,
        "COALESCE lowering should keep bounded AST SQL linear"
    );

    let numeric_literal = |token: &str| json!({"op":"literal","value":{"integerToken":token}});
    for (token, normalized) in [
        ("1.0", "1"),
        ("1e2", "100"),
        ("-0.0", "0"),
        ("0e999999999999999999999999999999", "0"),
    ] {
        let mut r = g02_numeric_request(numeric_literal(token));
        r["operation"] = json!("backfill_amount");
        let p = compile(r);
        assert_eq!(p["status"], "compiled", "integerToken {token}: {p}");
        assert_eq!(
            p["parameterSources"][0]["source"]["nativeValue"],
            normalized
        );

        let mut r = g02_numeric_request(numeric_literal(token));
        r["operation"] = json!("backfill_integer");
        let p = compile(r);
        assert_eq!(
            p["status"], "compiled",
            "integral integerToken {token} must type-check for integer targets: {p}"
        );
    }
    let cancelled_integral = format!("0.{}99999e2005", "0".repeat(2000));
    for tag in ["integerToken", "decimalToken"] {
        let mut value = serde_json::Map::new();
        value.insert(tag.to_owned(), json!(cancelled_integral));
        let expression = json!({"op":"literal","value":Value::Object(value)});
        for operation in ["backfill_amount", "backfill_integer"] {
            let mut request = g02_numeric_request(expression.clone());
            request["operation"] = json!(operation);
            let program = compile(request);
            assert_eq!(
                program["status"], "compiled",
                "{tag} {operation}: {program}"
            );
            assert_eq!(
                program["parameterSources"][0]["source"]["nativeValue"],
                "99999"
            );
        }
    }
    let cancelled_fraction = format!("0.{}99999e2004", "0".repeat(2000));
    let mut decimal = serde_json::Map::new();
    decimal.insert("decimalToken".to_owned(), json!(cancelled_fraction));
    let mut request = g02_numeric_request(json!({"op":"literal","value":Value::Object(decimal)}));
    request["operation"] = json!("backfill_amount");
    let program = compile(request);
    assert_eq!(program["status"], "compiled", "{program}");
    assert_eq!(
        program["parameterSources"][0]["source"]["nativeValue"],
        "9999.9"
    );

    let mut decimal = serde_json::Map::new();
    decimal.insert("decimalToken".to_owned(), json!(cancelled_fraction));
    let mut request = g02_numeric_request(json!({"op":"literal","value":Value::Object(decimal)}));
    request["operation"] = json!("backfill_integer");
    assert_eq!(compile(request)["status"], "blocked");

    for token in ["1.2", "1e1000"] {
        let mut r = g02_numeric_request(numeric_literal(token));
        r["operation"] = json!("backfill_amount");
        assert_eq!(
            compile(r)["status"],
            "blocked",
            "nonintegral or over-budget integerToken {token} must refuse"
        );
    }
    for token in ["1.0", "100e-2", "-0.00e999999999999999999999999999999"] {
        let mut r = g02_numeric_request(json!({"op":"literal","value":{"decimalToken":token}}));
        r["operation"] = json!("backfill_integer");
        let p = compile(r);
        assert_eq!(
            p["status"], "compiled",
            "integral decimalToken {token} must type-check for integer targets: {p}"
        );
        if token.starts_with('-') {
            assert_eq!(p["parameterSources"][0]["source"]["nativeValue"], "0");
        }
    }
    let mut fractional_decimal =
        g02_numeric_request(json!({"op":"literal","value":{"decimalToken":"1.2"}}));
    fractional_decimal["operation"] = json!("backfill_integer");
    assert_eq!(
        compile(fractional_decimal)["status"],
        "blocked",
        "fractional decimalToken must refuse an integer target"
    );

    let deep_condition = |not_count: usize| {
        let mut condition = json!({"op":"eq","field":{"module":"demo","element":"sourceAmount"},"value":{"kind":"literal","value":{"integerToken":"1"}}});
        for _ in 0..not_count {
            condition = json!({"op":"not","arg":condition});
        }
        json!({"op":"conditional","when":condition,"then":{"op":"field","field":{"module":"demo","element":"sourceAmount"}},"else":{"op":"literal","value":{"decimalToken":"0"}}})
    };
    let mut at_limit = g02_numeric_request(deep_condition(14));
    at_limit["operation"] = json!("backfill_amount");
    assert_eq!(
        compile(at_limit)["status"],
        "compiled",
        "predicate operand at AST depth 16 is admitted"
    );
    let mut over_limit = g02_numeric_request(deep_condition(15));
    over_limit["operation"] = json!("backfill_amount");
    assert_eq!(
        compile(over_limit)["status"],
        "blocked",
        "predicate operand at AST depth 17 refuses"
    );

    let exponent = json!({"op":"literal","value":{"decimalToken":"1e10"}});
    let mut r = g02_numeric_request(exponent);
    r["operation"] = json!("backfill_amount");
    assert_eq!(
        compile(r)["status"],
        "compiled",
        "positive exponent should be admitted with bounded numeric precision"
    );
}

#[test]
fn g02_intrinsic_null_types_and_null_safe_comparisons_remain_closed() {
    let null_literal = json!({"op":"literal","value":null});
    let text_concat =
        g02_request(json!({"op":"concat","args":[null_literal.clone(),null_literal.clone()]}));
    let compiled = compile(text_concat);
    assert_eq!(compiled["status"], "compiled", "{compiled}");
    assert!(compiled["sql"]
        .as_str()
        .unwrap()
        .contains("OPERATOR(pg_catalog.||)"));

    let numeric_add = json!({"op":"add","left":null_literal.clone(),"right":null_literal.clone()});
    let mut request = g02_numeric_request(numeric_add.clone());
    request["operation"] = json!("backfill_amount");
    let compiled = compile(request);
    assert_eq!(compiled["status"], "compiled", "{compiled}");
    assert!(compiled["sql"]
        .as_str()
        .unwrap()
        .contains("OPERATOR(pg_catalog.+)"));

    let large_numeric = json!({"op":"literal","value":{"decimalToken":"1e999"}});
    let null_plus_large = json!({"op":"add","left":null_literal.clone(),"right":large_numeric});
    let mut request = g02_numeric_request(null_plus_large);
    request["operation"] = json!("backfill_amount");
    let compiled = compile(request);
    assert_eq!(
        compiled["status"], "compiled",
        "NULL preserves the other numeric operand bound: {compiled}"
    );

    let mut wrong_concat_target = g02_numeric_request(
        json!({"op":"concat","args":[null_literal.clone(),null_literal.clone()]}),
    );
    wrong_concat_target["operation"] = json!("backfill_amount");
    assert_eq!(compile(wrong_concat_target)["status"], "blocked");
    assert_eq!(compile(g02_request(numeric_add))["status"], "blocked");

    let predicate = |op: &str| json!({"op":op,"field":{"module":"demo","element":"sourceAmount"},"value":{"kind":"literal","value":null}});
    for (op, not_wrapped) in [("eq", false), ("ne", false), ("eq", true), ("ne", true)] {
        let condition = if not_wrapped {
            json!({"op":"not","arg":predicate(op)})
        } else {
            predicate(op)
        };
        let expression = json!({"op":"conditional","when":condition,"then":{"op":"field","field":{"module":"demo","element":"sourceAmount"}},"else":{"op":"literal","value":{"decimalToken":"0"}}});
        let mut request = g02_numeric_request(expression);
        request["operation"] = json!("backfill_amount");
        let compiled = compile(request);
        assert_eq!(compiled["status"], "compiled", "{compiled}");
        let sql = compiled["sql"].as_str().unwrap();
        assert!(sql.contains("OPERATOR(pg_catalog.=)"), "{sql}");
        assert!(
            sql.contains("IS TRUE OR (") && sql.contains(" IS NULL AND "),
            "{sql}"
        );
        if op == "ne" || not_wrapped {
            assert!(sql.contains("NOT (("), "{sql}");
        }
    }
}
