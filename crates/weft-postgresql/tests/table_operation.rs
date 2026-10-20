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
    let module = weft_postgresql::table_operation::PreparedOperationModule::prepare(&source.to_string()).unwrap();
    let operation = module.prepare_operation("read_many").unwrap();
    let bind = |inputs: Value| json!({"call":{"kind":"single","inputs":inputs}}).to_string();
    let compiled: Value = serde_json::from_str(&operation.bind_json(&bind(json!({"owner":{"string":"alice"}})))).unwrap();
    let changed: Value = serde_json::from_str(&operation.bind_json(&bind(json!({"owner":{"string":"bob"}})))).unwrap();
    assert_eq!(compiled["status"], "compiled");
    assert_eq!(compiled["sql"], changed["sql"]);
    assert_eq!(compiled["nativeInputs"][0]["owner"], json!("alice"));
    assert_eq!(changed["nativeInputs"][0]["owner"], json!("bob"));
    assert_eq!(compiled["binding"], json!({"modelSha256":first["binding"]["modelSha256"]}));
    for retired in ["requestJson", "requestSha256", "declaration", "structuralDigest"] {
        assert!(compiled.get(retired).is_none(), "compact program retained {retired}");
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
        serde_json::from_str::<Value>(
            &weft_postgresql::table_operation::check_json(&program.to_string())
        )
        .unwrap()["status"],
        "checked"
    );

    let mut no_key_output = r;
    let mut source: Value = serde_json::from_str(no_key_output["binding"]["modelJson"].as_str().unwrap()).unwrap();
    let command = source["modules"][0]["extensions"]["umf.tables"]["commands"]
        .as_array_mut().unwrap().iter_mut().find(|item| item["id"] == "create_batch").unwrap();
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
    let mut literal_source: Value = serde_json::from_str(
        literal_batch["binding"]["modelJson"].as_str().unwrap(),
    )
    .unwrap();
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
    assert_eq!(literal_program["nativeBatch"]["parameterOrder"], json!(["nativeRows", "nativeLiterals"]));
    assert_eq!(literal_program["nativeBatch"]["dbapiParameterOrder"], json!(["nativeRows", "nativeLiterals"]));
    let literal_sql = literal_program["nativeBatch"]["sql"].as_str().unwrap();
    assert!(literal_sql.contains("CROSS JOIN umf_batch_literals AS l"));
    assert!(literal_sql.contains("VALUES (i.\"slot0\", i.\"slot1\")"));

    let mut empty_returning = request("create_batch");
    let mut empty_source: Value = serde_json::from_str(
        empty_returning["binding"]["modelJson"].as_str().unwrap(),
    )
    .unwrap();
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
    let mut r=request("create");let mut source=source();
    for (id,family,native) in [("day","date","date"),("at","timestamp","timestamptz")] {
        let reference=json!({"module":"demo","element":id});
        source["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":id,"kind":"field","scalarType":family,"cardinality":"one","nullability":"required","extensions":{}}));
        source["modules"][0]["elements"][2]["members"].as_array_mut().unwrap().push(reference.clone());
        source["modules"][0]["extensions"]["umf.tables"]["tables"][0]["columns"].as_array_mut().unwrap().push(json!({"field":reference,"name":id,"nativeType":{"kind":native}}));
        r["binding"]["tables"][0]["columns"].as_array_mut().unwrap().push(json!({"logical":id,"native":id,"carrier":family,"nullable":false}));
        let op=source["modules"][0]["extensions"]["umf.tables"]["commands"].as_array_mut().unwrap().iter_mut().find(|v|v["id"]=="create").unwrap();
        op["parameters"].as_array_mut().unwrap().push(json!({"id":id,"field":reference}));
        op["body"]["values"].as_array_mut().unwrap().push(json!({"field":reference,"value":{"kind":"parameter","parameter":id}}));
        op["body"]["returning"].as_array_mut().unwrap().push(reference);
    }
    repin(&mut r,source);
    r["call"]["inputs"]["day"]=json!({"date":"2000-02-29"});
    r["call"]["inputs"]["at"]=json!({"timestamp":"2026-01-01T00:30:00.123456000+01:00"});
    let program=compile(r.clone());assert_eq!(program["status"],"compiled","{program}");
    assert_eq!(program["nativeInputs"][0]["at"],"2025-12-31T23:30:00.123456Z");
    assert_eq!(program["nativeInputs"][0]["day"],"2000-02-29");
    assert!(program["sql"].as_str().unwrap().contains("AT TIME ZONE 'UTC'"));
    for (field,value) in [("day",json!({"date":"1900-02-29"})),("day",json!({"date":"infinity"})),("at",json!({"timestamp":"2026-01-01T00:00:00.1234567Z"})),("at",json!({"timestamp":"2026-01-01T00:00:00-00:00"}))] {
        let mut bad=r.clone();bad["call"]["inputs"][field]=value;assert_ne!(compile(bad)["status"],"compiled");
    }
    let raw=json!({"version":"weft.raw-read/0.1","target":"postgresql18","binding":r["binding"],"sql":"SELECT day FROM demo.Note","parameters":[],"maxRows":10,"keyset":false,"snapshot":false});
    assert!(weft_core::raw_read::admit_json(&raw.to_string()).is_err());
    let bound:weft_core::raw_read::Binding=serde_json::from_value(r["binding"].clone()).unwrap();
    assert!(weft_core::raw_read::binding_record(&bound,&["demo".into(),"Note".into()]).is_err());
}

#[test]
fn declared_domains_enforce_original_enumerations_ranges_and_literal_inputs() {
    let mut r=request("create");let mut model=source();
    model["modules"][0]["elements"][1]["allowedValues"]=json!([{"string":"alice"},{"string":"bob"}]);
    model["modules"][0]["elements"][1]["facets"]=json!({"length":{"unit":"unicode-scalar","min":1,"max":8}});
    repin(&mut r,model);
    assert_eq!(compile(r.clone())["status"],"compiled");
    r["call"]["inputs"]["owner"]=json!({"string":"other"});
    assert_eq!(compile(r.clone())["status"],"blocked");
    let binding:weft_core::raw_read::Binding=serde_json::from_value(r["binding"].clone()).unwrap();
    assert!(weft_core::raw_read::binding_record(&binding,&["demo".into(),"Note".into()]).is_err());
}

#[test]
fn transaction_clock_values_are_typed_and_never_become_input_slots() {
    let mut r=request("create");let mut model=source();let reference=json!({"module":"demo","element":"at"});
    model["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":"at","kind":"field","scalarType":"timestamp","cardinality":"one","nullability":"required","extensions":{}}));
    model["modules"][0]["elements"][2]["members"].as_array_mut().unwrap().push(reference.clone());
    model["modules"][0]["extensions"]["umf.tables"]["tables"][0]["columns"].as_array_mut().unwrap().push(json!({"field":reference,"name":"at","nativeType":{"kind":"timestamptz"}}));
    r["binding"]["tables"][0]["columns"].as_array_mut().unwrap().push(json!({"logical":"at","native":"at","carrier":"timestamp","nullable":false}));
    let command=model["modules"][0]["extensions"]["umf.tables"]["commands"].as_array_mut().unwrap().iter_mut().find(|c|c["id"]=="create").unwrap();
    command["body"]["values"].as_array_mut().unwrap().push(json!({"field":reference,"value":{"kind":"currentTimestamp"}}));
    command["body"]["returning"].as_array_mut().unwrap().push(reference);
    repin(&mut r,model.clone());let program=compile(r.clone());
    assert_eq!(program["status"],"compiled","{program}");
    assert!(program["sql"].as_str().unwrap().contains("CURRENT_TIMESTAMP"));
    assert_eq!(program["nativeInputs"][0].as_object().unwrap().len(),2);
    let command=model["modules"][0]["extensions"]["umf.tables"]["commands"].as_array_mut().unwrap().iter_mut().find(|c|c["id"]=="create").unwrap();
    command["body"]["values"][2]["field"]=json!({"module":"demo","element":"owner"});
    repin(&mut r,model);assert_eq!(compile(r)["status"],"blocked");
}

#[test]
fn batch_principal_is_native_and_has_no_caller_slot() {
    let mut r = request("create_batch");
    let mut model = source();
    let command = model["modules"][0]["extensions"]["umf.tables"]["commands"].as_array_mut().unwrap().iter_mut().find(|c| c["id"] == "create_batch").unwrap();
    command["parameters"] = json!([{"id":"id","field":{"module":"demo","element":"id"}}]);
    command["body"]["values"][1]["value"] = json!({"kind":"currentPrincipal"});
    repin(&mut r, model);
    r["call"]["items"] = json!([{"id":{"string":"a"}},{"id":{"string":"b"}}]);
    let p = compile(r.clone());
    assert_eq!(p["status"], "compiled", "{p}");
    assert!(p["nativeBatch"]["sql"].as_str().unwrap().contains("CURRENT_USER::pg_catalog.text"));
    assert_eq!(p["nativeBatch"]["parameterOrder"], json!(["nativeRows"]));
    r["call"]["items"][0]["owner"] = json!({"string":"forged"});
    assert_eq!(compile(r)["status"], "blocked");
}

#[test]
fn mixed_direction_cursor_carries_complete_order_with_original_key_suffix() {
    let mut r=request("read_page");let mut model=source();
    let command=model["modules"][0]["extensions"]["umf.tables"]["commands"].as_array_mut().unwrap().iter_mut().find(|c|c["id"]=="read_page").unwrap();
    command["body"]["orderBy"]=json!([{"field":{"module":"demo","element":"owner"},"direction":"desc"},{"field":{"module":"demo","element":"id"},"direction":"asc"}]);
    repin(&mut r,model);r["call"]["after"]=json!([{"string":"alice"},{"string":"n1"}]);
    let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");
    assert_eq!(p["nativeAfter"],json!(["alice","n1"]));
    assert_eq!(p["result"]["cursor"]["directions"],json!(["desc","asc"]));
    let sql=p["sql"].as_str().unwrap();assert!(sql.contains(" < "));assert!(sql.contains(" > "));assert!(sql.contains(" OR "));assert!(sql.contains(" DESC"));
    r["call"]["after"]=json!([{"string":"n1"}]);assert_eq!(compile(r)["status"],"blocked");
}
