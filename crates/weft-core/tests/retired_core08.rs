// SPDX-License-Identifier: MIT OR Apache-2.0
//! Retired UMF core formats remain explicit negative controls only.
use serde_json::{json, Value};
use weft_core::{frontend_json, json::sha256};

#[test]
fn core_08_fixture_is_rejected_by_current_admission() {
    let text = include_str!("../../../tests/fixtures/original-commerce-0.8/ontology.json");
    let source: Value = serde_json::from_str(text).unwrap();
    let request = json!({
        "dialect": "weft-sql/0.2.0",
        "sql": "SELECT c.id FROM customers c",
        "modules": [{
            "documentJson": text,
            "pin": {
                "documentId": source["id"],
                "revision": "historical-commerce0.8",
                "umfVersion": "0.8.0",
                "sha256": sha256(text.as_bytes())
            },
            "selectedModuleIds": ["domain"]
        }]
    });
    let response: Value = serde_json::from_str(&frontend_json(&request.to_string())).unwrap();
    assert_eq!(response["status"], "blocked");
    assert_eq!(response["diagnostics"][0]["code"], "WFT-MODEL-VERSION");
}
