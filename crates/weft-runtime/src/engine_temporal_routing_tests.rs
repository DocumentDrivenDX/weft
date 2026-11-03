// SPDX-License-Identifier: MIT OR Apache-2.0
//! Captured ABI1.2 admission controls shared with a pristine baseline test fixture.
use serde_json::{json, Value};
const LIMIT: usize = 4_194_304;
fn fixture() -> Value {
    weft_core::json::checked_json(include_str!(
        "../../../tests/engine-catalog/ordinary-abi12-public056.json"
    ))
    .unwrap()
}
fn pad(raw: &str, length: usize) -> String {
    assert!(raw.len() <= length);
    format!("{raw}{}", " ".repeat(length - raw.len()))
}
#[test]
fn temporal_route_ordinary_captured_read_above_four_mib_matches_prior_native_bytes() {
    let f = fixture();
    let raw = f["request"]["utf8"].as_str().unwrap();
    assert_eq!(
        weft_core::json::sha256(raw.as_bytes()),
        f["request"]["sha256"]
    );
    let request = weft_core::json::checked_json(raw).unwrap();
    let binding =
        weft_core::json::checked_json(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
    assert_eq!(binding["queryAbi"]["version"], "1.2");
    let packet =
        json!({"version":"weft.engine-read/0.1","compiler":"truss-runtime","request":request})
            .to_string();
    let before = super::engine_read::compile_json(&packet);
    assert_eq!(
        weft_core::json::checked_json(&before).unwrap()["status"],
        "compiled"
    );
    let padded = super::engine_read::compile_json(&pad(&packet, LIMIT + 1));
    eprintln!(
        "captured ABI1.2 ordinary native read result SHA256 {}",
        weft_core::json::sha256(before.as_bytes())
    );
    assert_eq!(
        padded, before,
        "oversized trailing whitespace must preserve ordinary native read result bytes"
    );
}
#[test]
fn temporal_route_ordinary_captured_page_preserves_prior_cap_diagnostic_bytes() {
    let f = fixture();
    let packet = f["enginePageInput"]["utf8"].as_str().unwrap();
    assert_eq!(
        weft_core::json::sha256(packet.as_bytes()),
        f["enginePageInput"]["sha256"]
    );
    let before = super::engine_read::compile_page_json(packet);
    assert_eq!(
        weft_core::json::checked_json(&before).unwrap(),
        f["expected"]["compile_engine_page_json"]["json"]
    );
    assert_eq!(
        super::engine_read::compile_page_json(&pad(packet, LIMIT)),
        before
    );
    let prior = json!({"version":"weft.engine-page/0.1","status":"blocked","diagnostics":[{"code":"WFT-ENGINE-PAGE","message":"bounded original page request required"}]}).to_string();
    assert_eq!(
        super::engine_read::compile_page_json(&pad(packet, LIMIT + 1)),
        prior
    );
}
#[test]
fn temporal_route_ordinary_oversized_invalid_packet_preserves_prior_diagnostic_bytes() {
    let before = super::engine_read::compile_json("{}");
    assert_eq!(
        super::engine_read::compile_json(&pad("{}", LIMIT + 1)),
        before
    );
}
#[test]
fn temporal_route_temporal_read_and_page_keep_exact_cap_and_never_fall_back() {
    let r = weft_core::json::checked_json(include_str!(
        "../../../tests/engine-catalog/temporal-request.json"
    ))
    .unwrap();
    for page in [false, true] {
        let mut packet = json!({"version":if page{"weft.engine-page/0.4"}else{"weft.engine-read/0.4"},"compiler":"truss-runtime-temporal","request":r});
        if page {
            packet["selectedKey"] = json!({"module":"m","record":"R","key":"pair"});
            packet["pageSize"] = json!(2);
        }
        let compile = |raw: &str| {
            if page {
                super::engine_read::compile_page_json(raw)
            } else {
                super::engine_read::compile_json(raw)
            }
        };
        let raw = packet.to_string();
        let before = compile(&raw);
        assert_eq!(
            weft_core::json::checked_json(&before).unwrap()["status"],
            "compiled"
        );
        assert_eq!(compile(&pad(&raw, LIMIT)), before);
        let over = weft_core::json::checked_json(&compile(&pad(&raw, LIMIT + 1))).unwrap();
        assert_eq!(over["diagnostics"][0]["code"], "WFT-LIMIT");
        assert_eq!(over["diagnostics"][0]["phase"], "input");
        assert!(over.get("artifact").is_none());
        // Valid JSON that exceeds the temporal node budget still selects the
        // temporal refusal path, even with both selectors after the dense leaf.
        let dense = format!(
            "{{\"extra\":[{}],\"compiler\":\"truss-runtime-temporal\",\"version\":\"{}\"}}",
            vec!["0"; 100_000].join(","),
            if page {
                "weft.engine-page/0.4"
            } else {
                "weft.engine-read/0.4"
            }
        );
        let refused = weft_core::json::checked_json(&compile(&dense)).unwrap();
        assert_eq!(refused["diagnostics"][0]["code"], "WFT-LIMIT");
        assert_eq!(refused["diagnostics"][0]["phase"], "input");
    }
}
