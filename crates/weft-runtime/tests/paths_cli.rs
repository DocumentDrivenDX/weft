//! Produced compiler executable checks only; no SQL engine or host execution.
#![cfg(feature = "ashlar-databricks-paths")]
use serde_json::{json, Value};
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

const MAX: usize = 16 * 1024 * 1024;
fn run(raw: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_weft-paths"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(raw).unwrap();
    child.wait_with_output().unwrap()
}
fn protocol(raw: &str) -> Value {
    let result = run(raw.as_bytes());
    assert!(result.status.success(), "{:?}", result.stderr);
    assert!(result.stderr.is_empty());
    let expected = format!("{}\n", weft_runtime::paths::compile_json(raw));
    assert_eq!(result.stdout, expected.as_bytes());
    assert_eq!(result.stdout.iter().filter(|b| **b == b'\n').count(), 1);
    serde_json::from_slice(&result.stdout).unwrap()
}
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/ashlar-databricks/fixtures/original-commerce-path-request.json"
    ))
    .unwrap()
}

#[test]
fn produced_binary_preserves_original_collection_and_count_artifacts() {
    let mut request = fixture();
    for query in [
        "SELECT l.id, RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 2) AS paths FROM order_lines l ORDER BY l.id",
        "SELECT l.product_id, COUNT(*) AS rows, COUNT_DISTINCT_PATH_TARGETS(p) AS targets FROM order_lines l CROSS JOIN EXPAND_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\") AS p GROUP BY l.product_id",
    ] {
        request["sql"]=json!(query);
        let raw=format!(" \r\n{}\t\n",request);
        let result=protocol(&raw);
        assert_eq!(result["status"],"compiled");
        assert_eq!(result["qualification"]["status"],"candidate");
        assert_eq!(result["modelPins"],json!([request["modules"][0]["pin"].clone()]));
    }
}

#[test]
fn compiler_refusals_are_closed04_and_old_route_stays_separate() {
    let mut request = fixture();
    request["options"]["allowCandidate"] = json!(false);
    let output = protocol(&request.to_string());
    assert_eq!(output["diagnostics"][0]["code"], "WFT-CAPABILITY");
    for raw in [
        "{",
        "{}",
        "{\"interfaceVersion\":\"weft-compile/0.3.0\",\"dialect\":\"weft-sql/0.3.0\"}",
    ] {
        let result = protocol(raw);
        assert_eq!(result["interfaceVersion"], "weft-compile/0.4.0");
        assert_eq!(result["status"], "blocked");
        assert_eq!(result.as_object().unwrap().len(), 3);
        assert!(result.get("sql").is_none());
    }
    let old: Value =
        serde_json::from_str(&weft_runtime::compile_json(&fixture().to_string())).unwrap();
    assert_eq!(old["diagnostics"][0]["code"], "WFT-VERSION");
}

#[test]
fn native_transport_limits_utf8_before_conversion_and_emits_constant_errors() {
    for (raw, code) in [
        (vec![b' '; MAX + 1], "WEFT_CLI_INPUT_LIMIT\n"),
        (vec![0xff], "WEFT_CLI_UTF8\n"),
        (
            {
                let mut b = vec![b' '; MAX - 1];
                b.push(0xc3);
                b
            },
            "WEFT_CLI_UTF8\n",
        ),
    ] {
        let result = run(&raw);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert_eq!(result.stderr, code.as_bytes());
    }
    let mut exact = vec![b' '; MAX - 2];
    exact.extend_from_slice("é".as_bytes());
    let raw = String::from_utf8(exact).unwrap();
    let output = protocol(&raw);
    assert_eq!(output["status"], "blocked");
}
