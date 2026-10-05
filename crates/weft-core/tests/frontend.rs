use serde_json::Value;
use weft_core::frontend_json;
// @covers US-001-AC1 @covers US-001-AC2 @covers US-001-AC3 @covers US-001-AC4
#[test]
fn independent_corpus_resolution_and_refusals() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../docs/helix/03-test/fixtures/cases.json"
    ))
    .unwrap();
    for case in corpus.as_array().unwrap() {
        let response: Value =
            serde_json::from_str(&frontend_json(&case["request"].to_string())).unwrap();
        let expected = if case["expected"]["status"] == "compiled" {
            "resolved"
        } else {
            "blocked"
        };
        assert_eq!(response["status"], expected, "{}: {}", case["id"], response);
        if expected == "blocked" {
            assert_eq!(
                response["diagnostics"][0]["code"], case["expected"]["code"],
                "{}",
                case["id"]
            );
            assert!(response.get("logicalPlan").is_none());
        } else {
            assert_eq!(
                response["retainedModules"], case["request"]["modules"],
                "{}",
                case["id"]
            );
        }
    }
}

fn base(sql: &str) -> Value {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../docs/helix/03-test/fixtures/cases.json"
    ))
    .unwrap();
    let mut request = corpus[0]["request"].clone();
    request["sql"] = serde_json::json!(sql);
    request
}
fn run(req: &Value) -> Value {
    serde_json::from_str(&frontend_json(&req.to_string())).unwrap()
}
fn model_edit(req: &mut Value, edit: impl FnOnce(&mut Value)) {
    let mut doc: Value =
        serde_json::from_str(req["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    edit(&mut doc);
    let text = doc.to_string();
    req["modules"][0]["documentJson"] = serde_json::json!(text);
    req["modules"][0]["pin"]["sha256"] =
        serde_json::json!(weft_core::json::sha256(text.as_bytes()));
}
#[test]
fn utf8_spans_and_exact_quoted_names() {
    let sql = "SELECT \"é😀\".\"na😀me\" FROM Customer \"é😀\"";
    let mut req = base(sql);
    model_edit(&mut req, |d| {
        d["modules"][0]["elements"][3]["name"] = serde_json::json!("na😀me")
    });
    let r = run(&req);
    assert_eq!(r["status"], "resolved");
    let e = &r["logicalPlan"]["root"]["outputs"][0]["expression"];
    let start = e["span"]["start"].as_u64().unwrap() as usize;
    let end = e["span"]["end"].as_u64().unwrap() as usize;
    assert_eq!(&sql[start..end], "\"é😀\".\"na😀me\"");
    assert_eq!(e["identity"]["element"], "customer-name");
}
#[test]
fn on_scope_self_joins_and_group_only() {
    let r = run(&base(
        "SELECT c.id FROM Customer c JOIN Customer d ON c.id = d.id",
    ));
    assert_eq!(r["status"], "resolved");
    assert_eq!(
        r["logicalPlan"]["root"]["input"]["on"]["left"]["scan"],
        "s0"
    );
    assert_eq!(
        r["logicalPlan"]["root"]["input"]["on"]["right"]["scan"],
        "s1"
    );
    let r=run(&base("SELECT c.id FROM Customer c JOIN Orders o ON o.customer_id = later.id JOIN Customer later ON later.id = c.id"));
    assert_eq!(r["diagnostics"][0]["code"], "WFT-NAME-MISSING");
    let r = run(&base("SELECT c.active FROM Customer c GROUP BY c.active"));
    assert_eq!(r["status"], "resolved");
    assert_eq!(
        r["logicalPlan"]["root"]["input"]["aggregates"],
        serde_json::json!([])
    );
}
#[test]
fn schema_identity_and_dependency_guards() {
    let mut req = base("SELECT c.name FROM Customer c");
    model_edit(&mut req, |d| {
        d["modules"][0]["elements"][2]["id"] = serde_json::json!("customer-name")
    });
    assert_eq!(run(&req)["diagnostics"][0]["code"], "WFT-MODEL");
    let mut req = base("SELECT c.name FROM Customer c");
    model_edit(&mut req, |d| {
        d["modules"][0]["elements"][0]["members"][0]["element"] = serde_json::json!("missing")
    });
    assert_eq!(run(&req)["diagnostics"][0]["code"], "WFT-MODEL");
    let mut req = base("SELECT c.name FROM Customer c");
    model_edit(&mut req, |d| d["vocabularies"] = serde_json::json!([]));
    assert_eq!(run(&req)["diagnostics"][0]["code"], "WFT-MODEL");
    let mut req = base("SELECT c.name FROM Customer c");
    model_edit(&mut req, |d| {
        d["extensions"]["future.vendor"]["huge"] =
            serde_json::from_str::<Value>("1234567890123456789012345678901234567890").unwrap()
    });
    assert_eq!(run(&req)["status"], "resolved");
    assert_eq!(run(&req)["retainedModules"], req["modules"]);
}
