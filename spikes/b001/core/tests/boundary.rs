use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use weft_spike_core::compile_json;
#[test]
fn numeric_corpus_uses_independent_expected_domains() {
    let original_corpus = include_bytes!("../../fixtures/cases-umf070.json");
    assert_eq!(
        format!("{:x}", Sha256::digest(original_corpus)),
        "5d13b1d0d77074193e678321406f57a9ce34e49cd413649f0f7f7210262c6306"
    );
    let cases: Value = serde_json::from_slice(original_corpus).unwrap();
    let mut count = 0;
    let mut compiled = 0;
    let mut blocked = 0;
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
            blocked += 1;
            assert_eq!(response["code"], case["expected"]["code"], "{}", case["id"]);
            assert!(response.get("sql").is_none());
        } else {
            compiled += 1;
        }
        count += 1;
    }
    assert_eq!(count, 575);
    assert_eq!(compiled, 288);
    assert_eq!(blocked, 287);
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
