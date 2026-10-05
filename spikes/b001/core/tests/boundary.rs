use serde_json::{json, Value};
use weft_spike_core::compile_json;
#[test]
fn numeric_corpus_uses_independent_expected_domains() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../../../docs/helix/03-test/fixtures/cases.json"
    ))
    .unwrap();
    let mut count = 0;
    for case in cases
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["category"] == "numeric")
    {
        let mut request = case["request"].clone();
        request["target"]["backendId"] = json!("spike.synthetic");
        let response: Value = serde_json::from_str(&compile_json(&request.to_string())).unwrap();
        assert_eq!(
            response["status"], case["expected"]["status"],
            "{}",
            case["id"]
        );
        if response["status"] == "blocked" {
            assert_eq!(response["code"], case["expected"]["code"], "{}", case["id"]);
            assert!(response.get("sql").is_none());
        }
        count += 1;
    }
    assert_eq!(count, 575);
}
#[test]
fn duplicate_keys_are_not_discarded() {
    let response: Value = serde_json::from_str(&compile_json(r#"{"sql":"a","sql":"b"}"#)).unwrap();
    assert_eq!(response["code"], "WFT-JSON-DUPLICATE");
}
#[test]
fn malformed_or_large_requests_refuse_atomically() {
    for (input, code) in [
        ("{".to_string(), "WFT-INPUT"),
        (" ".repeat(16 * 1024 * 1024 + 1), "WFT-LIMIT"),
    ] {
        let response: Value = serde_json::from_str(&compile_json(&input)).unwrap();
        assert_eq!(response["code"], code);
        assert!(response.get("sql").is_none());
    }
}
