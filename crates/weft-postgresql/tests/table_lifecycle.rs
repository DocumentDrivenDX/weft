// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{json, Value};
fn source() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/raw-postgresql/table-lifecycle-model.json"
    ))
    .unwrap()
}
fn repin(request: &mut Value, source: &Value) {
    let text = serde_json::to_string(source).unwrap();
    request["binding"]["modelSha256"] = json!(weft_core::json::sha256(text.as_bytes()));
    request["binding"]["modelJson"] = json!(text);
}
fn request() -> Value {
    let mut result: Value = serde_json::from_str(include_str!(
        "../../../tests/raw-postgresql/table-lifecycle-read-template.json"
    ))
    .unwrap();
    repin(&mut result, &source());
    result
}
fn compile(request: Value) -> Value {
    serde_json::from_str(&weft_postgresql::raw_read::compile_json(
        &request.to_string(),
    ))
    .unwrap()
}
#[test]
fn native_table_read_retains_source_and_explicit_unqualified_host_obligations() {
    let r = request();
    let program = compile(r.clone());
    assert_eq!(program["status"], "compiled", "{program}");
    assert_eq!(program["version"], "weft.table-read/0.1");
    assert_eq!(program["binding"]["modelJson"], r["binding"]["modelJson"]);
    assert_eq!(
        program["obligations"]["tableLifecycle"]["sourceDigest"],
        r["binding"]["modelSha256"]
    );
    assert_eq!(program["obligations"]["tableLifecycle"]["module"], "demo");
    assert_eq!(
        program["obligations"]["tableLifecycle"]["tables"][0]["id"],
        "notes"
    );
    assert_eq!(
        program["obligations"]["tableLifecycle"]["tables"][0]["command"],
        "select"
    );
    assert_eq!(
        program["obligations"]["tableLifecycle"]["nativeExecutionQualified"],
        false
    );
    assert!(program["sql"]
        .as_str()
        .unwrap()
        .contains("\"plugin_demo\".\"notes\""));
    assert!(program["dbapiSql"].as_str().unwrap().contains("%s"));
    assert!(program["cursorParameters"]
        .as_array()
        .is_some_and(|v| !v.is_empty()));
}
#[test]
fn selected_table_policy_cannot_silently_use_a_legacy_profile() {
    for remove in ["nativeLifecycle", "verificationProfile"] {
        let mut r = request();
        if remove == "nativeLifecycle" {
            r["binding"].as_object_mut().unwrap().remove(remove);
        } else {
            r.as_object_mut().unwrap().remove(remove);
        }
        assert_eq!(compile(r)["status"], "blocked");
    }
    let mut r = request();
    let mut doc = source();
    doc["modules"][0]["extensions"]
        .as_object_mut()
        .unwrap()
        .remove("umf.tables");
    repin(&mut r, &doc);
    assert_eq!(compile(r)["status"], "blocked");
}
#[test]
fn changed_native_policy_remains_in_source_and_changes_custody_without_sql_reinterpretation() {
    let before = compile(request());
    let mut r = request();
    let mut doc = source();
    doc["modules"][0]["extensions"]["umf.tables"]["tables"][0]["policy"]["person"]["select"] =
        json!({"kind":"deny"});
    repin(&mut r, &doc);
    let after = compile(r.clone());
    assert_eq!(after["status"], "compiled", "{after}");
    assert_eq!(before["sql"], after["sql"]);
    assert_ne!(before["structuralDigest"], after["structuralDigest"]);
    assert_ne!(
        before["obligations"]["tableLifecycle"]["sourceDigest"],
        after["obligations"]["tableLifecycle"]["sourceDigest"]
    );
    assert_eq!(after["binding"]["modelJson"], r["binding"]["modelJson"]);
}
#[test]
fn native_context_and_mapping_tampering_refuses() {
    for pointer in [
        "/binding/nativeLifecycle/module",
        "/binding/nativeLifecycle/version",
        "/binding/nativeLifecycle/configurationDigest",
        "/binding/nativeLifecycle/deploymentDigest",
        "/binding/tables/0/native",
        "/binding/tables/0/columns/0/native",
    ] {
        let mut r = request();
        *r.pointer_mut(pointer).unwrap() = json!("forged");
        assert_eq!(compile(r)["status"], "blocked", "{pointer}");
    }
    let mut r = request();
    r["binding"]["schema"] = json!("public");
    assert_eq!(compile(r)["status"], "blocked");
    let mut r = request();
    r["binding"]["tables"][0]["key"] = json!(["owner"]);
    assert_eq!(compile(r)["status"], "blocked");
    let mut r = request();
    r["binding"]["tables"][0]["columns"][0]["nullable"] = json!(true);
    assert_eq!(compile(r)["status"], "blocked");
}
#[test]
fn unknown_selected_table_content_and_opaque_commands_refuse() {
    for (pointer, value) in [
        ("/modules/0/extensions/umf.tables", json!({"unknown":true})),
        (
            "/modules/0/extensions/umf.tables/tables/0/policy/person/select",
            json!({"kind":"future"}),
        ),
        (
            "/modules/0/extensions/umf.tables/commands",
            json!([{"unknown":true}]),
        ),
    ] {
        let mut r = request();
        let mut doc = source();
        *doc.pointer_mut(pointer).unwrap() = value;
        repin(&mut r, &doc);
        assert_eq!(compile(r)["status"], "blocked", "{pointer}");
    }
    let mut r = request();
    r["binding"]["nativeLifecycle"]["unknown"] = json!(true);
    assert_eq!(compile(r)["status"], "blocked");
}
#[test]
fn original_source_whitespace_is_preserved_and_pinned() {
    let mut r = request();
    let text = format!("\r\n{}\t", serde_json::to_string_pretty(&source()).unwrap());
    r["binding"]["modelJson"] = json!(text);
    r["binding"]["modelSha256"] = json!(weft_core::json::sha256(text.as_bytes()));
    let program = compile(r.clone());
    assert_eq!(program["status"], "compiled", "{program}");
    assert_eq!(program["binding"]["modelJson"], r["binding"]["modelJson"]);
    r["binding"]["modelSha256"] = json!("0".repeat(64));
    assert_eq!(compile(r)["status"], "blocked");
}
#[test]
fn original_fk_custody_is_required_for_native_relationship_bindings() {
    let doc: Value = serde_json::from_str(include_str!(
        "../../../tests/raw-postgresql/table-lifecycle-related-model.json"
    ))
    .unwrap();
    let mut r = request();
    r["parameters"] = json!([]);
    r["sql"]=json!("SELECT n.\"id\" AS id,p.\"parentOwner\" AS owner FROM demo.\"Note\" n LEFT JOIN demo.\"Parent\" p ON n.\"noteParentId\"=p.\"parentId\" ORDER BY n.\"id\" ASC LIMIT 10");
    r["binding"]["tables"] = json!([
     {"logical":["demo","Note"],"native":"notes","columns":[{"logical":"id","native":"id","carrier":"string","nullable":false},{"logical":"owner","native":"owner","carrier":"string","nullable":false},{"logical":"noteParentId","native":"parent_id","carrier":"string","nullable":false},{"logical":"caption","native":"caption","carrier":"string","nullable":true},{"logical":"amount","native":"amount","carrier":"decimal","nullable":false}],"key":["id"]},
     {"logical":["demo","Parent"],"native":"parents","columns":[{"logical":"parentId","native":"id","carrier":"string","nullable":false},{"logical":"parentOwner","native":"owner","carrier":"string","nullable":false}],"key":["parentId"]}
    ]);
    r["binding"]["relationships"] = json!([{"module":"demo","id":"parent","source":["demo","Note"],"target":["demo","Parent"],"pairs":[["noteParentId","parentId"]],"toOne":true}]);
    repin(&mut r, &doc);
    let program = compile(r.clone());
    assert_eq!(program["status"], "compiled", "{program}");
    assert_eq!(
        program["obligations"]["tableLifecycle"]["tables"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let mut forged = r.clone();
    forged["binding"]["relationships"][0]["pairs"] = json!([["owner", "parentId"]]);
    forged["sql"]=json!("SELECT n.\"id\" AS id FROM demo.\"Note\" n JOIN demo.\"Parent\" p ON n.\"owner\"=p.\"parentId\" ORDER BY n.\"id\" ASC LIMIT 10");
    assert_eq!(compile(forged)["status"], "blocked");
    let mut forged = r.clone();
    let mut changed = doc.clone();
    changed["modules"][0]["extensions"]["umf.tables"]["tables"][0]["foreignKeys"][0]
        .as_object_mut()
        .unwrap()
        .remove("relationship");
    repin(&mut forged, &changed);
    assert_eq!(compile(forged)["status"], "blocked");
    let mut forged = r.clone();
    let mut changed = doc.clone();
    changed["modules"][0]["extensions"]["umf.tables"]["tables"][0]["foreignKeys"][0]["target"]
        ["key"] = json!("different-key");
    repin(&mut forged, &changed);
    assert_eq!(compile(forged)["status"], "blocked");
}
#[test]
fn unselected_native_extension_is_retained_without_granting_or_requiring_its_profile() {
    let mut r = request();
    let mut doc = source();
    let mut other = doc["modules"][0].clone();
    other["id"] = json!("other");
    other["namespace"] = json!("other");
    other["extensions"] = json!({});
    for element in other["elements"].as_array_mut().unwrap() {
        if let Some(members) = element.get_mut("members").and_then(Value::as_array_mut) {
            for member in members {
                member["module"] = json!("other");
            }
        }
        if let Some(keys) = element.get_mut("keys").and_then(Value::as_array_mut) {
            for key in keys {
                for field in key["fields"].as_array_mut().unwrap() {
                    field["module"] = json!("other");
                }
            }
        }
    }
    doc["modules"].as_array_mut().unwrap().push(other);
    r["binding"]["tables"][0]["logical"] = json!(["other", "Note"]);
    r["sql"] = json!("SELECT n.\"id\" AS id FROM other.\"Note\" n ORDER BY n.\"id\" ASC LIMIT 10");
    r["parameters"] = json!([]);
    r["binding"]
        .as_object_mut()
        .unwrap()
        .remove("nativeLifecycle");
    r.as_object_mut().unwrap().remove("verificationProfile");
    repin(&mut r, &doc);
    let program = compile(r.clone());
    assert_eq!(program["status"], "compiled", "{program}");
    assert_eq!(program["version"], "weft.raw-read/0.1");
    assert_eq!(program["binding"]["modelJson"], r["binding"]["modelJson"]);
    assert!(program["obligations"].get("tableLifecycle").is_none());
}

#[test]
fn native_count_keeps_table_obligations_and_requires_complete_module_inventory() {
    let mut r = request();
    r["sql"] = json!("SELECT COUNT(*) AS total FROM demo.\"Note\" n WHERE n.\"owner\"=:person");
    r["keyset"] = json!(false);
    let program = compile(r.clone());
    assert_eq!(program["status"], "compiled", "{program}");
    assert_eq!(program["version"], "weft.table-read/0.1");
    assert_eq!(
        program["obligations"]["tableLifecycle"]["tables"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let mut extra = r.clone();
    let duplicate = extra["binding"]["tables"][0].clone();
    extra["binding"]["tables"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert_eq!(compile(extra)["status"], "blocked");
    let mut incomplete = r.clone();
    incomplete["binding"]["tables"][0]["columns"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert_eq!(compile(incomplete)["status"], "blocked");
    let mut other_profile = r;
    other_profile["verificationProfile"] = json!("source-exact/0.1");
    assert_eq!(compile(other_profile)["status"], "blocked");
}

#[test]
fn preserved_unknown_members_cannot_be_selected_as_known_table_meaning() {
    for pointer in [
        "/modules/0/extensions/umf.tables",
        "/modules/0/extensions/umf.tables/tables/0",
        "/modules/0/extensions/umf.tables/tables/0/record",
        "/modules/0/extensions/umf.tables/tables/0/columns/0",
        "/modules/0/extensions/umf.tables/tables/0/columns/0/nativeType",
        "/modules/0/extensions/umf.tables/tables/0/policy",
        "/modules/0/extensions/umf.tables/tables/0/policy/person",
        "/modules/0/extensions/umf.tables/tables/0/policy/person/select",
    ] {
        let mut r = request();
        let mut doc = source();
        doc.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("future".into(), json!(true));
        repin(&mut r, &doc);
        assert_eq!(compile(r)["status"], "blocked", "{pointer}");
    }
}
