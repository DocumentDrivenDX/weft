// @covers US-007-AC1 @covers US-007-AC2 @covers US-007-AC3
// @covers US-007-AC4 @covers US-007-AC5 @covers US-007-AC6
use serde_json::{json, Value};
use weft_core::frontend_json;
fn corpus() -> Value {
    serde_json::from_str(include_str!("fixtures/cases.json")).unwrap()
}
fn run(r: &Value) -> Value {
    serde_json::from_str(&frontend_json(&r.to_string())).unwrap()
}
#[test]
fn application_acceptance_and_refusals() {
    for c in corpus().as_array().unwrap() {
        let r = run(&c["request"]);
        assert_eq!(r["status"], c["expected"]["status"], "{}: {}", c["id"], r);
        if r["status"] == "blocked" {
            assert_eq!(
                r["diagnostics"][0]["code"], c["expected"]["code"],
                "{}: {}",
                c["id"], r
            );
            assert!(r.get("logicalPlan").is_none());
        }
    }
}
#[test]
fn plans_keep_domains_presence_join_bags_and_cursors() {
    let cases = corpus();
    let find = |id: &str| {
        cases
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == id)
            .unwrap()
    };
    let r = run(&find("whole-entity")["request"]);
    let p = &r["logicalPlan"];
    assert_eq!(
        p["outputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["id", "name", "active", "nickname", "tags", "address"]
    );
    assert!(p["requiredCapabilities"]
        .as_array()
        .unwrap()
        .contains(&json!("value.presence")));
    assert!(p["requiredCapabilities"]
        .as_array()
        .unwrap()
        .contains(&json!("value.sequence")));
    assert_eq!(p["typeGraph"][4]["availability"], "absent-allowed");
    let r = run(&find("composite-cursor")["request"]);
    assert_eq!(
        r["logicalPlan"]["filters"][0]["columns"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(r["logicalPlan"]["filters"][0]["op"], "lexicographicGreater");
    let r = run(&find("join-count")["request"]);
    assert_eq!(r["logicalPlan"]["joins"].as_array().unwrap().len(), 1);
    assert_eq!(r["logicalPlan"]["outputs"][0]["expression"]["op"], "count");
    assert_eq!(
        r["logicalPlan"]["outputs"][0]["expression"]["type"]["facets"],
        json!({})
    );
}
#[test]
fn parameters_intersect_all_use_domains() {
    let mut r = corpus()[0]["request"].clone();
    r["sql"] =
        json!("SELECT c.id FROM Customer c WHERE c.id=:id AND c.id=:id ORDER BY c.id LIMIT 10");
    r["parameters"] = json!({"id":{"family":"integer","value":"18446744073709551615"}});
    assert_eq!(run(&r)["status"], "resolved");
    let mut d: Value =
        serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    d["modules"][0]["elements"][4]["scalarType"] = json!("integer");
    d["modules"][0]["elements"][4]["facets"] = json!({"integerWidth":{"bits":8,"signed":false}});
    let text = d.to_string();
    r["modules"][0]["documentJson"] = json!(text);
    r["modules"][0]["pin"]["sha256"] = json!(weft_core::json::sha256(text.as_bytes()));
    r["sql"] =
        json!("SELECT c.id FROM Customer c WHERE c.id=:id AND c.active=:id ORDER BY c.id LIMIT 10");
    assert_eq!(run(&r)["diagnostics"][0]["code"], "WFT-NUMERIC-DOMAIN");
    r["parameters"]["id"]["value"] = json!("255");
    assert_eq!(run(&r)["status"], "resolved");
}
#[test]
fn original_valid_queries_remain_valid_in_explicit_02() {
    let cases: Value =
        serde_json::from_str(include_str!("../../docs/helix/03-test/fixtures/cases.json")).unwrap();
    for c in cases
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["expected"]["status"] == "compiled")
    {
        let mut r = c["request"].clone();
        r["dialect"] = json!("weft-sql/0.2.0");
        assert_eq!(run(&r)["status"], "resolved", "{}", c["id"]);
    }
}
#[test]
fn explicit_profile_version_and_parameter_map_bounds() {
    let mut r = corpus()[0]["request"].clone();
    r["readProfile"]["version"] = json!("weft-application-read/0.3.0");
    assert_eq!(run(&r)["diagnostics"][0]["code"], "WFT-VERSION");
    r["readProfile"]["version"] = json!("weft-application-read/0.2.0");
    let parameters = (0..1025)
        .map(|n| (format!("p{n}"), json!({"family":"integer","value":"1"})))
        .collect::<serde_json::Map<_, _>>();
    r["parameters"] = json!(parameters);
    assert_eq!(run(&r)["diagnostics"][0]["code"], "WFT-LIMIT");
}
