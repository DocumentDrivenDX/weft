// SPDX-License-Identifier: MIT OR Apache-2.0
//! Pure compiler controls. Synthetic capture pins do not qualify native execution.
use serde_json::{json, Value};
fn request() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/engine-catalog/temporal-request.json"
    ))
    .unwrap()
}
fn compile(r: &Value) -> Value {
    serde_json::from_str(&super::engine_read::compile_json(
        &json!({"version":"weft.engine-read/0.4","compiler":"truss-runtime-temporal","request":r})
            .to_string(),
    ))
    .unwrap()
}
#[test]
fn temporal_read_compiles_all_source_members_before_filters() {
    let r = request();
    let p = compile(&r);
    assert_eq!(p["status"], "compiled", "{p}");
    let a = &p["artifact"];
    assert_eq!(
        a["sourceDomainChecks"]["version"],
        "weft.truss-temporal-domain-checks/0.2"
    );
    assert_eq!(a["sourceDomainChecks"]["maxScanRows"], 10000);
    let sources = &a["sourceDomainChecks"]["sources"];
    assert_eq!(sources.as_array().unwrap().len(), 1);
    let slot = sources[0]["requirementsParameter"].as_u64().unwrap() as usize - 1;
    let req: Value =
        serde_json::from_str(a["parameters"][slot]["value"].as_str().unwrap()).unwrap();
    assert_eq!(
        req["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["kind"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["date", "instant", "local"]
    );
    assert_eq!(p["outputs"][2]["missing"], true);
    assert_eq!(p["outputs"][2]["nullable"], true);
    assert!(a["sql"]
        .as_str()
        .unwrap()
        .contains("person_query_temporal_objects"));
}
fn check(p: &Value, r: &Value, page: bool, shape: bool, after: &str) -> Value {
    serde_json::from_str(&super::engine_temporal::check_json(
        &p.to_string(),
        &r.to_string(),
        if page && !shape { Some(after) } else { None },
        page,
        shape,
    ))
    .unwrap()
}
fn page(r: &Value) -> Value {
    serde_json::from_str(&super::engine_read::compile_page_json(&json!({"version":"weft.engine-page/0.4","compiler":"truss-runtime-temporal","request":r,"selectedKey":{"module":"m","record":"R","key":"pair"},"pageSize":2}).to_string())).unwrap()
}
fn rewrite_model(r: &mut Value, f: impl FnOnce(&mut Value)) {
    let mut doc: Value =
        serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    f(&mut doc);
    let text = doc.to_string();
    let hash = weft_core::json::sha256(text.as_bytes());
    r["modules"][0]["documentJson"] = json!(text);
    r["modules"][0]["pin"]["sha256"] = json!(hash.clone());
    r["modules"][0]["pin"]["revision"] = json!(hash.clone());
    let mut b: Value = serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();
    b["documents"][0]["sha256"] = json!(hash.clone());
    b["documents"][0]["revision"] = json!(hash.clone());
    b["records"][0]["identity"]["revision"] = json!(hash.clone());
    for f in b["records"][0]["fields"].as_array_mut().unwrap() {
        f["identity"]["revision"] = json!(hash.clone())
    }
    let raw = b.to_string();
    r["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
    r["target"]["bindingJson"] = json!(raw);
}
#[test]
fn temporal_four_checkers_return_whole_program_and_revalidate_current_values() {
    let r = request();
    for is_page in [false, true] {
        let p = if is_page { page(&r) } else { compile(&r) };
        assert_eq!(p["status"], "compiled", "{p}");
        let out = check(&p, &r, is_page, true, "null");
        assert_eq!(out, json!({"status":"checked","program":p}));
        let mut current = r.clone();
        current["temporalParameters"]["since"]["value"] = json!("1970-01-01T00:00:00.000001+00:00");
        let out = check(&p, &current, is_page, false, "null");
        assert_eq!(out["status"], "checked", "{out}");
        assert_eq!(out["program"]["request"], current);
        assert_ne!(out["program"]["structuralDigest"], p["structuralDigest"]);
        assert_eq!(
            out["program"]["artifact"]["sourceDomainChecks"],
            p["artifact"]["sourceDomainChecks"]
        );
    }
}
#[test]
fn temporal_whole_program_tamper_refuses_even_resealed() {
    let r = request();
    for is_page in [false, true] {
        let p = if is_page { page(&r) } else { compile(&r) };
        for pointer in [
            "/artifact/sql",
            "/artifact/parameters/0/value",
            "/artifact/parameters/1/position",
            "/artifact/columns/0/sourceIdentities/0/element",
            "/artifact/sourceDomainChecks/maxBytes",
            "/artifact/sourceDomainChecks/sources/0/bindingParameter",
            "/artifact/sourceDomainChecks/queryAbi/sha256",
            "/outputs/0/decoder/kind",
            "/request/target/bindingJson",
        ] {
            let mut forged = p.clone();
            *forged.pointer_mut(pointer).unwrap() = json!("forged");
            forged.as_object_mut().unwrap().remove("structuralDigest");
            let sha = weft_core::json::sha256(weft_core::mutation::canonical(&forged).as_bytes());
            forged["structuralDigest"] = json!({"owner":if is_page{"weft-runtime-engine-page"}else{"weft-runtime-engine-read"},"version":"0.4","sha256":sha});
            for shape in [false, true] {
                let out = check(&forged, &r, is_page, shape, "null");
                assert_eq!(out["status"], "blocked", "{pointer}: {out}");
                assert!(out.get("program").is_none());
            }
        }
        let mut unknown = p.clone();
        unknown["extra"] = json!({"extension":"must not disappear"});
        assert_eq!(
            check(&unknown, &r, is_page, true, "null")["status"],
            "blocked"
        );
    }
}
#[test]
fn temporal_current_context_shape_and_cross_kind_refuse() {
    let r = request();
    let p = compile(&r);
    for pointer in [
        "/sql",
        "/temporalProfile",
        "/temporalParameters/since/kind",
        "/modules/0/documentJson",
        "/target/bindingSha256",
    ] {
        let mut current = r.clone();
        *current.pointer_mut(pointer).unwrap() = json!("changed");
        assert_eq!(
            check(&p, &current, false, false, "null")["status"],
            "blocked"
        );
        assert_eq!(
            check(&p, &current, false, true, "null")["status"],
            "blocked"
        );
    }
    let mut current = r.clone();
    current["parameters"]["since"] = json!({"family":"string","value":"1969-12-31T23:59:59.5Z"});
    assert_eq!(compile(&current)["status"], "blocked");
}
#[test]
fn temporal_strict075_tokens_and_pre_epoch_normalization() {
    for (kind, token, canonical) in [
        ("date", "2000-02-29", "2000-02-29"),
        (
            "local",
            "1969-12-31T23:59:59.5",
            "1969-12-31T23:59:59.500000",
        ),
        (
            "instant",
            "1969-12-31T23:59:59.5Z",
            "1969-12-31T23:59:59.500000Z",
        ),
        (
            "instant",
            "1970-01-01T01:00:00+01:00",
            "1970-01-01T00:00:00.000000Z",
        ),
    ] {
        assert_eq!(
            weft_core::application_temporal::canonical(kind, token).unwrap(),
            canonical
        )
    }
    for (kind, token) in [
        ("instant", "2024-01-01T00:00:00.1234560Z"),
        ("instant", "2024-01-01T00:00:00.0000000Z"),
        ("instant", "2024-01-01T00:00:00-00:00"),
        ("instant", "0001-01-01T00:00:00+00:01"),
        ("instant", "9999-12-31T23:59:59-00:01"),
        ("instant", "2024-01-01T00:00:60Z"),
        ("instant", "2024-01-01T00:00:00+00:00:00"),
        ("local", "2024-01-01T00:00:00Z"),
        ("date", "1900-02-29"),
    ] {
        assert!(
            weft_core::application_temporal::canonical(kind, token).is_err(),
            "{kind}: {token}"
        );
    }
}
#[test]
fn temporal_source_guard_is_complete_for_unprojected_filtered_and_later_rows() {
    let mut r = request();
    r["sql"] = json!("SELECT r.id FROM R r WHERE r.label='excluded' LIMIT 1");
    r["temporalParameters"] = json!({});
    let p = compile(&r);
    assert_eq!(p["status"], "compiled", "{p}");
    let a = &p["artifact"];
    let req: Value = serde_json::from_str(a["parameters"][1]["value"].as_str().unwrap()).unwrap();
    assert_eq!(req["fields"].as_array().unwrap().len(), 3);
    let sql = a["sql"].as_str().unwrap();
    assert!(sql.starts_with("WITH \"__weft_temporal_source\" AS MATERIALIZED"));
    assert!(sql.find("person_query_temporal_objects").unwrap() < sql.find(" WHERE ").unwrap());
    for forbidden in [
        "person_query_seek_objects",
        "person_query_selected_objects",
        "::pg_catalog.timestamp",
        "::pg_catalog.timestamptz",
    ] {
        assert!(!sql.contains(forbidden))
    }
} // Actual late-row execution belongs to the native worker.
#[test]
fn temporal_pages_bind_full_string_key_tuple_and_presence() {
    let r = request();
    let p = page(&r);
    assert_eq!(p["status"], "compiled", "{p}");
    assert_eq!(p["page"]["sort"].as_array().unwrap().len(), 4);
    assert_eq!(p["outputs"].as_array().unwrap().len(), 5);
    let after = json!({"version":"weft.engine-temporal-after/0.2","selectedKey":{"module":"m","record":"R","key":"pair"},"sortValues":[{"state":"present","value":"1969-12-31T23:59:59.500000"},{"state":"present","value":"1969-12-31"},{"state":"present","value":"prefix"},{"state":"present","value":"b"}],"keyValues":["prefix","b"]});
    for state in [
        json!({"state":"present","value":"1969-12-31T23:59:59.500000"}),
        json!({"state":"null"}),
        json!({"state":"missing"}),
    ] {
        let mut after = after.clone();
        after["sortValues"][0] = state;
        let out = check(&p, &r, true, false, &after.to_string());
        assert_eq!(out["status"], "checked", "{out}");
        assert_eq!(out["program"]["after"], after);
        assert_eq!(out["program"]["outputs"], p["outputs"]);
        assert!(out["program"]["artifact"]["sql"]
            .as_str()
            .unwrap()
            .contains("IS NOT DISTINCT FROM"));
    }
    for pointer in [
        "/version",
        "/selectedKey/key",
        "/sortValues/0/value",
        "/keyValues/1",
    ] {
        let mut bad = after.clone();
        *bad.pointer_mut(pointer).unwrap() = json!("forged");
        let out = check(&p, &r, true, false, &bad.to_string());
        assert_eq!(out["status"], "blocked", "{pointer}: {out}")
    }
    let mut bad = after.clone();
    bad["sortValues"].as_array_mut().unwrap().pop();
    assert_eq!(
        check(&p, &r, true, false, &bad.to_string())["status"],
        "blocked"
    );
}
#[test]
fn temporal_source_refinements_foreign_meaning_and_key_domains_refuse() {
    for change in [
        "refinement",
        "vocabulary",
        "kind",
        "temporal-key",
        "numeric-key",
    ] {
        let mut r = request();
        rewrite_model(&mut r, |doc| match change {
            "refinement" => {
                doc["modules"][0]["elements"][2]["allowedValues"] = json!(["2000-01-01"])
            }
            "vocabulary" => doc["vocabularies"]["umf.access"]["version"] = json!("unknown"),
            "kind" => {
                doc["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"][3]["temporal"] =
                    json!("local")
            }
            "temporal-key" => {
                doc["modules"][0]["elements"][6]["keys"][0]["fields"][0]["element"] = json!("day")
            }
            _ => doc["modules"][0]["elements"][0]["scalarType"] = json!("integer"),
        });
        let p = page(&r);
        assert_eq!(p["status"], "blocked", "{change}: {p}");
    }
}
#[test]
fn temporal_framing_limits_and_ordinary_checker_refusal() {
    let r = request();
    let p = compile(&r);
    let raw = p.to_string();
    let current = r.to_string();
    for bad in [
        format!("{{\"x\":1,\"x\":2}}"),
        "{\"x\":\"\\ud800\"}".into(),
        " ".repeat(4_194_305),
    ] {
        let out: Value = serde_json::from_str(&super::engine_temporal::check_json(
            &bad, &current, None, false, true,
        ))
        .unwrap();
        assert_eq!(out["status"], "blocked");
        assert!(out.get("program").is_none());
    }
    let ordinary: Value =
        serde_json::from_str(&super::engine_read::check_json(&raw, &current)).unwrap();
    assert_eq!(ordinary["status"], "blocked");
    let out: Value = serde_json::from_str(&super::engine_temporal::check_json(
        &page(&r).to_string(),
        &current,
        Some(&" ".repeat(1_048_577)),
        true,
        false,
    ))
    .unwrap();
    assert_eq!(out["diagnostics"][0]["code"], "WFT-LIMIT");
}
fn reseal(p: &mut Value, is_page: bool) {
    p.as_object_mut().unwrap().remove("structuralDigest");
    let sha = weft_core::json::sha256(weft_core::mutation::canonical(p).as_bytes());
    p["structuralDigest"] = json!({"owner":if is_page{"weft-runtime-engine-page"}else{"weft-runtime-engine-read"},"version":"0.4","sha256":sha});
}
fn leaves(v: &Value, path: String, out: &mut Vec<String>) {
    match v {
        Value::Object(m) if !m.is_empty() => {
            for (key, value) in m {
                leaves(
                    value,
                    format!("{path}/{}", key.replace('~', "~0").replace('/', "~1")),
                    out,
                )
            }
        }
        Value::Array(a) if !a.is_empty() => {
            for (i, value) in a.iter().enumerate() {
                leaves(value, format!("{path}/{i}"), out)
            }
        }
        _ => out.push(path),
    }
}
#[test]
fn temporal_every_program_leaf_tamper_is_refused_with_old_and_resealed_digest() {
    let r = request();
    let mut checked = 0;
    for is_page in [false, true] {
        let original = if is_page { page(&r) } else { compile(&r) };
        assert_eq!(original["status"], "compiled", "{original}");
        let mut paths = vec![];
        leaves(&original, String::new(), &mut paths);
        for pointer in paths {
            for resealed in [false, true] {
                if resealed && pointer.starts_with("/structuralDigest/") {
                    continue;
                }
                let mut p = original.clone();
                let leaf = p.pointer_mut(&pointer).unwrap();
                *leaf = match leaf {
                    Value::String(s) => json!(format!("{s}_tampered")),
                    Value::Bool(b) => json!(!*b),
                    Value::Number(n) => json!(n.as_u64().unwrap_or(0) + 1),
                    _ => json!({"unexpected":true}),
                };
                if resealed {
                    reseal(&mut p, is_page)
                }
                for shape in [true, false] {
                    let out = check(&p, &r, is_page, shape, "null");
                    assert_eq!(
                        out["status"], "blocked",
                        "{pointer}, resealed={resealed}, shape={shape}: {out}"
                    );
                    assert!(out.get("program").is_none());
                    checked += 1;
                }
            }
        }
    }
    eprintln!("Whole-program leaf rejection checks: {checked}");
    assert!(
        checked > 1000,
        "Bounded full-leaf inventory unexpectedly incomplete: {checked}"
    );
}
fn binding_change(r: &mut Value, f: impl FnOnce(&mut Value)) {
    let mut b: Value = serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();
    f(&mut b);
    let text = b.to_string();
    r["target"]["bindingSha256"] = json!(weft_core::json::sha256(text.as_bytes()));
    r["target"]["bindingJson"] = json!(text);
}
#[test]
fn temporal_coherent_inner_binding_refusals_preserve_original_diagnostics() {
    for case in [
        "abi-missing",
        "abi-ordinary",
        "abi-extra",
        "identity-extra",
        "field-identity-extra",
        "key-property-order",
        "keys-missing",
        "release",
        "fields",
        "documents",
        "duplicates",
        "zero-scan",
        "over-scan",
        "over-result",
        "over-bytes",
        "zero-bytes",
        "lookahead",
    ] {
        let mut r = request();
        binding_change(&mut r, |b| match case {
            "abi-missing" => {
                b.as_object_mut().unwrap().remove("queryAbi");
            }
            "abi-ordinary" => b["queryAbi"]["version"] = json!("1.2"),
            "abi-extra" => b["queryAbi"]["extra"] = json!(true),
            "identity-extra" => b["records"][0]["identity"]["foreignMeaning"] = json!(true),
            "field-identity-extra" => {
                b["records"][0]["fields"][0]["identity"]["foreignMeaning"] = json!(true)
            }
            "key-property-order" => b["records"][0]["keys"][0]["propertyIds"] = json!(["2", "1"]),
            "keys-missing" => b["records"][0]["keys"] = json!([]),
            "release" => b["releaseId"] = json!("truss-catalog-runtime/0.75-candidate"),
            "fields" => {
                b["records"][0]["fields"].as_array_mut().unwrap().pop();
            }
            "documents" => b["documents"][0]["revision"] = json!("foreign"),
            "duplicates" => b["records"][0]["fields"][1]["propertyId"] = json!("1"),
            "zero-scan" => b["maximumScanRows"] = json!(0),
            "over-scan" => b["maximumScanRows"] = json!(10001),
            "over-result" => b["maximumResultRows"] = json!(10001),
            "over-bytes" => b["maximumBytes"] = json!(16777217),
            "zero-bytes" => b["maximumBytes"] = json!(0),
            _ => b["maximumResultRows"] = json!(2),
        });
        let out = page(&r);
        assert_eq!(out["status"], "blocked", "{case}: {out}");
        assert!(out.get("artifact").is_none());
        assert!(out.get("program").is_none());
        assert_eq!(out["diagnostics"], out["original"]["diagnostics"]);
    }
}
#[test]
fn temporal_current_whole_result_matches_independent_fresh_compilation() {
    let mut r = request();
    r["parameters"] = json!({"label":{"family":"string","value":"before"}});
    r["sql"]=json!("SELECT r.day AS day,r.at AS at,r.wall AS wall FROM R r WHERE r.label = :label AND r.at >= :since ORDER BY r.wall DESC NULLS LAST");
    for is_page in [false, true] {
        let p = if is_page { page(&r) } else { compile(&r) };
        assert_eq!(p["status"], "compiled", "{p}");
        let mut current = r.clone();
        current["parameters"]["label"]["value"] = json!("after");
        current["temporalParameters"]["since"]["value"] = json!("1970-01-01T00:00:00+01:00");
        let out = check(&p, &current, is_page, false, "null");
        assert_eq!(
            out,
            json!({"status":"checked","program":if is_page{page(&current)}else{compile(&current)}})
        );
        assert_eq!(
            check(&p, &current, is_page, true, "null")["status"],
            "blocked"
        );
        for map in ["parameters", "temporalParameters"] {
            let mut bad = current.clone();
            let name = if map == "parameters" {
                "label"
            } else {
                "since"
            };
            let value = bad[map].as_object_mut().unwrap().remove(name).unwrap();
            bad[map]["equal_size_renamed"] = value;
            assert_eq!(check(&p, &bad, is_page, false, "null")["status"], "blocked");
            let mut bad = current.clone();
            bad[map][name]["extra"] = json!(true);
            assert_eq!(check(&p, &bad, is_page, false, "null")["status"], "blocked");
        }
    }
}
#[test]
fn temporal_continuation_directions_metadata_and_repeated_sources() {
    for direction in ["ASC", "DESC"] {
        for nulls in ["FIRST", "LAST"] {
            let mut r = request();
            r["sql"]=json!(format!("SELECT r.day AS __weft_temporal_hidden_2 FROM R r WHERE r.at>=:since ORDER BY r.wall {direction} NULLS {nulls},r.wall ASC NULLS LAST"));
            let p = page(&r);
            assert_eq!(p["status"], "compiled", "{p}");
            assert_eq!(
                p["page"]["sort"][0]["direction"],
                direction.to_ascii_lowercase()
            );
            assert_eq!(p["page"]["sort"][0]["nulls"], nulls.to_ascii_lowercase());
            assert_eq!(
                p["page"]["sort"][0]["outputPosition"],
                p["page"]["sort"][1]["outputPosition"]
            );
            let mut after = json!({"version":"weft.engine-temporal-after/0.2","selectedKey":{"module":"m","record":"R","key":"pair"},"sortValues":[{"state":"missing"},{"state":"missing"},{"state":"present","value":"a"},{"state":"present","value":"b"}],"keyValues":["a","b"]});
            let out = check(&p, &r, true, false, &after.to_string());
            assert_eq!(out["status"], "checked", "{out}");
            let selected = weft_core::application_temporal::KeyRef {
                module: "m".into(),
                record: "R".into(),
                key: "pair".into(),
            };
            let fresh = weft_postgresql::truss_runtime::temporal::compile_value(
                &r,
                Some((&selected, 2)),
                &after,
            )
            .unwrap();
            let mut expected = p.clone();
            expected["artifact"] = fresh["artifact"].clone();
            expected["outputs"] = fresh["outputs"].clone();
            expected["page"] = fresh["page"].clone();
            expected["after"] = after.clone();
            reseal(&mut expected, true);
            assert_eq!(out["program"], expected);
            after["sortValues"][1] = json!({"state":"null"});
            assert_eq!(
                check(&p, &r, true, false, &after.to_string())["status"],
                "blocked"
            );
        }
    }
}
#[test]
fn temporal_all_ordinary_parameter_families_keep_their_domains() {
    let mut r = request();
    rewrite_model(&mut r, |doc| {
        for (name, kind, facets) in [
            ("flag", "boolean", json!({})),
            (
                "count",
                "integer",
                json!({"integerWidth":{"bits":64,"signed":true}}),
            ),
            ("amount", "decimal", json!({"precision":12,"scale":2})),
        ] {
            doc["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":name,"name":name,"kind":"field","scalarType":kind,"cardinality":"one","nullability":"required","facets":facets,"extensions":{}}));
            doc["modules"][0]["elements"][6]["members"]
                .as_array_mut()
                .unwrap()
                .push(json!({"module":"m","element":name}));
            doc["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"].as_array_mut().unwrap().push(json!({"field":{"module":"m","element":name},"row":{"missing":false,"null":false},"create":{"missing":false,"null":false},"mutable":true,"generated":false}));
        }
    });
    binding_change(&mut r, |b| {
        let mut identity = b["records"][0]["identity"].clone();
        for (i, name) in ["flag", "count", "amount"].iter().enumerate() {
            identity["element"] = json!(name);
            b["records"][0]["fields"]
                .as_array_mut()
                .unwrap()
                .push(json!({"identity":identity,"propertyId":(i+7).to_string()}));
        }
    });
    r["sql"]=json!("SELECT r.day,r.label,r.flag,r.count,r.amount FROM R r WHERE r.at >= :since AND r.label = :label AND r.flag = :flag AND r.count >= :count AND r.amount <= :amount ORDER BY r.wall DESC NULLS LAST");
    r["parameters"] = json!({"label":{"family":"string","value":"a"},"flag":{"family":"boolean","value":"false"},"count":{"family":"integer","value":"-2"},"amount":{"family":"decimal","value":"10.00"}});
    let p = compile(&r);
    assert_eq!(p["status"], "compiled", "{p}");
    let mut current = r.clone();
    for (key, value) in [
        ("label", "b"),
        ("flag", "true"),
        ("count", "9223372036854775807"),
        ("amount", "0.01"),
    ] {
        current["parameters"][key]["value"] = json!(value)
    }
    let out = check(&p, &current, false, false, "null");
    assert_eq!(out, json!({"status":"checked","program":compile(&current)}));
    for (key, value) in [
        ("flag", "1"),
        ("count", "9223372036854775808"),
        ("amount", "0.001"),
    ] {
        let mut bad = current.clone();
        bad["parameters"][key]["value"] = json!(value);
        let out = check(&p, &bad, false, false, "null");
        assert_eq!(out["status"], "blocked", "{key}: {out}");
        assert_eq!(out["diagnostics"], out["original"]["diagnostics"]);
    }
}
#[test]
fn temporal_dependency_graph_preserves_authentic_public056_ordinary_abi12() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/engine-catalog/ordinary-abi12-public056.json"
    ))
    .unwrap();
    let request = fixture["request"]["utf8"].as_str().unwrap();
    let packet = fixture["enginePageInput"]["utf8"].as_str().unwrap();
    assert_eq!(
        weft_core::json::sha256(request.as_bytes()),
        fixture["request"]["sha256"]
    );
    assert_eq!(
        weft_core::json::sha256(packet.as_bytes()),
        fixture["enginePageInput"]["sha256"]
    );
    let r: Value = serde_json::from_str(request).unwrap();
    let binding: Value =
        serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();
    assert_eq!(binding["queryAbi"]["version"], "1.2");
    assert_eq!(r["target"]["backendVersion"], "0.4.43");
    for (name, result) in [
        (
            "compile_truss_runtime_json",
            weft_postgresql::truss_runtime::compile_json(request),
        ),
        (
            "compile_truss_runtime_page_json",
            weft_postgresql::truss_runtime::compile_page_json(
                request,
                fixture["pageSize"].as_u64().unwrap() as u16,
                None,
            ),
        ),
        (
            "compile_engine_page_json",
            super::engine_read::compile_page_json(packet),
        ),
    ] {
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, fixture["expected"][name]["json"], "{name}");
    }
    let checked = &fixture["expected"]["check_engine_page_json"];
    let program = checked["inputs"]["program"]["utf8"].as_str().unwrap();
    let current = checked["inputs"]["request"]["utf8"].as_str().unwrap();
    let after = checked["inputs"]["afterJson"]["utf8"].as_str().unwrap();
    let out: Value = serde_json::from_str(&super::engine_read::check_page_json(
        program, current, after,
    ))
    .unwrap();
    assert_eq!(out, checked["result"]["json"]);
    assert_eq!(
        check(
            &serde_json::from_str(program).unwrap(),
            &r,
            true,
            true,
            "null"
        )["status"],
        "blocked"
    );
    let mut changed = r.clone();
    binding_change(&mut changed, |b| b["queryAbi"]["version"] = json!("1.3"));
    let out: Value = serde_json::from_str(&weft_postgresql::truss_runtime::compile_json(
        &changed.to_string(),
    ))
    .unwrap();
    assert_eq!(out["status"], "blocked");
    assert!(out.get("sql").is_none());
}
#[test]
fn temporal_red_controls_limit_must_remain_an_integer_literal() {
    let mut r = request();
    r["sql"] = json!("SELECT r.day FROM R r LIMIT '1'");
    r["temporalParameters"] = json!({});
    let out = compile(&r);
    assert_eq!(
        out["status"], "blocked",
        "String LIMIT must retain the ordinary parser refusal: {out}"
    );
}
#[test]
fn temporal_same_kind_comparisons_cover_all_six_operators() {
    for (field, kind, token) in [
        ("day", "date", "2000-02-29"),
        ("at", "instant", "1969-12-31T23:59:59.5Z"),
        ("wall", "local", "1969-12-31T23:59:59.5"),
    ] {
        for op in ["=", "<>", "<", "<=", ">", ">="] {
            let mut r = request();
            r["sql"]=json!(format!("SELECT r.{field} FROM R r WHERE (r.{field} {op} :value AND TRUE) OR NOT FALSE ORDER BY r.{field}"));
            r["temporalParameters"] = json!({"value":{"kind":kind,"value":token}});
            let p = compile(&r);
            assert_eq!(p["status"], "compiled", "{field} {op}: {p}");
            assert!(p["artifact"]["sql"]
                .as_str()
                .unwrap()
                .contains("query_temporal_canonical_text"));
            assert_eq!(p["artifact"]["parameters"][2]["value"], token);
            assert_eq!(
                p["artifact"]["parameters"][2]["logicalType"]["temporalKind"],
                kind
            );
            let mut bad = r.clone();
            bad["temporalParameters"]["value"]["kind"] =
                json!(if kind == "date" { "local" } else { "date" });
            assert_eq!(compile(&bad)["status"], "blocked");
        }
    }
}
#[test]
fn temporal_alternate_selected_key_and_page_boundaries_are_exact() {
    let mut r = request();
    rewrite_model(&mut r, |doc| {
        doc["modules"][0]["elements"][6]["keys"].as_array_mut().unwrap().push(json!({"id":"alternate","name":"Alternative composite","fields":[{"module":"m","element":"id2"},{"module":"m","element":"label"}],"primary":false}));
        doc["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"][5]["mutable"] =
            json!(false);
        doc["modules"][0]["extensions"]["umf.access"]["views"][0]["selectedKey"]["key"] =
            json!("alternate");
    });
    binding_change(&mut r, |b| {
        b["records"][0]["keys"]
            .as_array_mut()
            .unwrap()
            .push(json!({"keyId":"alternate","propertyIds":["2","6"]}))
    });
    assert_eq!(
        page(&r)["status"],
        "blocked",
        "The old primary must never substitute for the selected alternate"
    );
    for size in [1, 999] {
        let packet = json!({"version":"weft.engine-page/0.4","compiler":"truss-runtime-temporal","request":r,"selectedKey":{"module":"m","record":"R","key":"alternate"},"pageSize":size});
        let p: Value =
            serde_json::from_str(&super::engine_read::compile_page_json(&packet.to_string()))
                .unwrap();
        assert_eq!(p["status"], "compiled", "{p}");
        assert!(p["artifact"]["sql"]
            .as_str()
            .unwrap()
            .contains("E'alternate'"));
        assert!(p["artifact"]["sql"]
            .as_str()
            .unwrap()
            .ends_with(&format!("LIMIT {}", size + 1)));
        assert_eq!(
            p["artifact"]["sourceDomainChecks"]["sources"][0]["selectedKey"],
            packet["selectedKey"]
        );
        let sort = p["page"]["sort"].as_array().unwrap();
        assert_eq!(sort[sort.len() - 2]["field"]["element"], "id2");
        assert_eq!(sort[sort.len() - 1]["field"]["element"], "label");
    }
}
#[test]
fn temporal_exact_utf8_input_bounds_accept_then_refuse_one_more_byte() {
    let r = request();
    let p = page(&r);
    assert_eq!(p["status"], "compiled", "{p}");
    let program = p.to_string();
    let request = r.to_string();
    let pad = |text: &str, bound: usize| format!("{}{}", text, " ".repeat(bound - text.len()));
    for (first, second) in [
        (pad(&program, 4_194_304), request.clone()),
        (program.clone(), pad(&request, 4_194_304)),
    ] {
        let out: Value = serde_json::from_str(&super::engine_temporal::check_json(
            &first, &second, None, true, true,
        ))
        .unwrap();
        assert_eq!(out, json!({"status":"checked","program":p}));
        let mut first = first;
        let mut second = second;
        if first.len() == 4_194_304 {
            first.push(' ')
        } else {
            second.push(' ')
        }
        let out: Value = serde_json::from_str(&super::engine_temporal::check_json(
            &first, &second, None, true, true,
        ))
        .unwrap();
        assert_eq!(out["diagnostics"][0]["code"], "WFT-LIMIT");
        assert!(out.get("program").is_none());
    }
    let after = pad("null", 1_048_576);
    let out: Value = serde_json::from_str(&super::engine_temporal::check_json(
        &program,
        &request,
        Some(&after),
        true,
        false,
    ))
    .unwrap();
    assert_eq!(out["status"], "checked", "{out}");
    let out: Value = serde_json::from_str(&super::engine_temporal::check_json(
        &program,
        &request,
        Some(&(after + " ")),
        true,
        false,
    ))
    .unwrap();
    assert_eq!(out["diagnostics"][0]["code"], "WFT-LIMIT");
}
#[test]
fn temporal_selected_key_suffix_is_exact_including_null_placement() {
    for (nulls, slots) in [("FIRST", 4), ("LAST", 2)] {
        let mut r = request();
        r["temporalParameters"] = json!({});
        r["sql"] = json!(format!(
            "SELECT r.day FROM R r ORDER BY r.id ASC NULLS {nulls},r.id2 ASC NULLS LAST"
        ));
        let p = page(&r);
        assert_eq!(p["status"], "compiled", "{p}");
        let sort = p["page"]["sort"].as_array().unwrap();
        assert_eq!(sort.len(), slots);
        assert_eq!(sort[0]["nulls"], nulls.to_ascii_lowercase());
        assert_eq!(sort[slots - 2]["field"]["element"], "id");
        assert_eq!(sort[slots - 2]["nulls"], "last");
        assert_eq!(sort[slots - 1]["field"]["element"], "id2");
    }
}
