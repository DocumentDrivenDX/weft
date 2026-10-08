// @covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 @covers US-004-AC4
// Compiler component assertions; native/host acceptance is separate.
mod common;
use serde_json::{json, Value};
use weft_core::{backend::Registry, compile::Compiler, json::sha256};
use weft_databricks::candidate::Candidate;
fn compiler() -> Compiler {
    let mut registry = Registry::default();
    registry.register(Candidate).unwrap();
    Compiler { registry }
}
fn run(request: &Value) -> Value {
    serde_json::from_str(&compiler().compile_json(&request.to_string())).unwrap()
}

#[test]
fn sales_compiles_with_pins_exact_carriers_and_prequery_integrity() {
    let request = common::request(common::SALES_SQL);
    let response = run(&request);
    assert_eq!(response["status"], "compiled", "{response}");
    assert_eq!(response["qualification"]["status"], "candidate");
    let sql = response["sql"].as_str().unwrap();
    assert!(sql.contains("VERSION AS OF 0"));
    assert!(sql.contains("SUM("));
    assert!(sql.contains("raise_error('WFT-NUMERIC-DOMAIN')"));
    assert!(sql.contains("COUNT("));
    assert!(!sql.contains("DISTINCT"));
    assert!(!sql.contains("weft-synthetic"));
    assert!(!sql.contains("fixture-schema-1"));
    assert_eq!(response["columns"][0]["outputName"], "name");
    assert_eq!(response["columns"][1]["outputName"], "total");
    assert_eq!(response["columns"][1]["decoder"], "exact-decimal");
    let integrity = response["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == "ashlar.candidate.scalarIntegrity")
        .unwrap();
    assert_eq!(
        integrity["parameters"]["checks"].as_array().unwrap().len(),
        4
    );
    assert_eq!(integrity["parameters"]["phase"], "before-user-query");
    assert!(response["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == "ashlar.candidate.publication"));
    assert_eq!(run(&request), response);
}
#[test]
fn missing_mapping_and_candidate_optout_return_no_sql() {
    let mut request = common::request(common::SALES_SQL);
    request["options"]["allowCandidate"] = json!(false);
    let result = run(&request);
    assert_eq!(result["status"], "blocked");
    assert!(result.get("sql").is_none());
    let mut request = common::request(common::SALES_SQL);
    let mut binding: Value =
        serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
    binding["records"][0]["properties"]
        .as_array_mut()
        .unwrap()
        .remove(1);
    let raw = binding.to_string();
    request["target"]["bindingJson"] = json!(raw);
    request["target"]["bindingSha256"] = json!(sha256(raw.as_bytes()));
    let result = run(&request);
    assert_eq!(result["status"], "blocked");
    assert!(result.get("sql").is_none());
}
#[test]
fn unicode_quotes_literal_and_boolean_are_bound_in_typed_slots() {
    let response=run(&common::request("SELECT c.name FROM Customer c WHERE c.name = 'x''; DROP TABLE fake;--é' AND c.active = TRUE"));
    assert_eq!(response["status"], "compiled", "{response}");
    assert!(!response["sql"].as_str().unwrap().contains("DROP TABLE"));
    assert!(response["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["value"] == "x'; DROP TABLE fake;--é"));
    assert!(response["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["value"] == "true"));
}

#[test]
fn global_sum_retains_nullable_exact_type_and_empty_input_branch() {
    let response = run(&common::request(
        "SELECT SUM(o.total) AS total FROM Orders o",
    ));
    assert_eq!(response["status"], "compiled", "{response}");
    assert_eq!(response["columns"][0]["nullable"], true);
    assert_eq!(
        response["columns"][0]["logicalType"]["facets"],
        json!({"scale":2})
    );
    assert_eq!(response["columns"][0]["decoder"], "exact-decimal");
    assert!(!response["sql"].as_str().unwrap().contains("GROUP BY"));
}
