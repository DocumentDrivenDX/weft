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
    assert!(sql.contains("MAX(1)"));
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

fn application(sql: &str) -> Value {
    let mut request = common::request(sql);
    request["interfaceVersion"] = json!("weft-compile/0.2.0");
    request["dialect"] = json!("weft-sql/0.2.0");
    request
}
#[test]
fn application_entity_and_count_keep_logical_metadata() {
    for sql in [
        "SELECT c.* FROM Customer c",
        "SELECT COUNT(*) AS total FROM Customer c",
        "SELECT c.name, COUNT(*) AS total FROM Customer c GROUP BY c.name",
        common::SALES_SQL,
    ] {
        let response = run(&application(sql));
        assert_eq!(response["status"], "compiled", "{response}");
        assert!(response["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["id"] == "ashlar.candidate.scalarIntegrity"));
        if sql.contains("COUNT") {
            let representation =
                &response["columns"].as_array().unwrap().last().unwrap()["representation"];
            assert_eq!(representation["decoder"], "exact-integer");
            assert_eq!(representation["logicalType"]["nullable"], false);
        }
        if sql == "SELECT c.* FROM Customer c" {
            assert_eq!(response["columns"].as_array().unwrap().len(), 3);
            assert_eq!(response["columns"][0]["outputName"], "id");
            assert_eq!(response["columns"][1]["outputName"], "name");
            assert_eq!(response["columns"][2]["outputName"], "active");
        }
    }
}
#[test]
fn application_page_retains_authored_key_and_named_cursor() {
    let mut request =
        application("SELECT c.* FROM Customer c WHERE c.id > :after ORDER BY c.id ASC LIMIT 2");
    let mut document: Value =
        serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    document["modules"][0]["elements"][0]["keys"] = json!([{"id":"primary","name":"primary","primary":true,"fields":[{"module":"sales","element":"customer-id"}]}]);
    let raw = document.to_string();
    let digest = sha256(raw.as_bytes());
    request["modules"][0]["documentJson"] = json!(raw);
    request["modules"][0]["pin"]["sha256"] = json!(digest);
    let mut binding: Value =
        serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
    binding["modelPins"][0]["sha256"] = json!(digest);
    let raw = binding.to_string();
    request["target"]["bindingJson"] = json!(raw);
    request["target"]["bindingSha256"] = json!(sha256(raw.as_bytes()));
    request["parameters"] = json!({"after":{"family":"integer","value":"7"}});
    request["readProfile"] =
        json!({"version":"weft-application-read/0.2.0","subset":"entity-page"});
    let response = run(&request);
    assert_eq!(response["status"], "compiled", "{response}");
    assert!(response["sql"].as_str().unwrap().contains("ORDER BY"));
    assert!(response["sql"].as_str().unwrap().contains("LIMIT 2"));
    assert!(response["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == "ashlar.candidate.keyIntegrity"));
}

fn optional_request(column: bool) -> Value {
    let mut request = application("SELECT c.name FROM Customer c");
    let mut document: Value =
        serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    let field = document["modules"][0]["elements"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["id"] == "customer-name")
        .unwrap();
    field["nullability"] = json!("absent-allowed");
    let raw = document.to_string();
    let digest = sha256(raw.as_bytes());
    request["modules"][0]["documentJson"] = json!(raw);
    request["modules"][0]["pin"]["sha256"] = json!(digest);
    let mut binding: Value =
        serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
    binding["modelPins"][0]["sha256"] = json!(digest);
    if column {
        binding["records"][0]["kind"] = json!("nodeProjection");
        binding["records"][0]["properties"][1]["home"] = json!({"kind":"column","value":"group_value","present":"group_present","nativeType":"STRING"});
    }
    let raw = binding.to_string();
    request["target"]["bindingJson"] = json!(raw);
    request["target"]["bindingSha256"] = json!(sha256(raw.as_bytes()));
    request
}

#[test]
fn optional_scalar_preserves_presence_envelope_in_both_owner_homes() {
    for column in [false, true] {
        let response = run(&optional_request(column));
        assert_eq!(response["status"], "compiled", "{response}");
        assert_eq!(response["columns"][0]["representation"]["kind"], "value");
        assert_eq!(
            response["columns"][0]["representation"]["nativeNull"],
            false
        );
        assert_eq!(response["columns"][0]["nullable"], false);
        let sql = response["sql"].as_str().unwrap();
        assert!(sql.contains("'absent'"));
        assert!(sql.contains("to_json(named_struct"));
        let check = response["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["id"] == "ashlar.candidate.scalarIntegrity")
            .unwrap()["parameters"]["checks"][0]["sql"]
            .as_str()
            .unwrap();
        if column {
            assert!(check.contains("group_present` = FALSE"));
            assert!(check.contains("group_value` IS NULL"));
        } else {
            assert!(check.contains("RLIKE '^OBJECT'"));
            assert!(check.contains("IS NULL"));
        }
    }
}
