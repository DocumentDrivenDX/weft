// @covers US-005-AC1 @covers US-005-AC2 @covers US-005-AC3 @covers US-005-AC4
#[path = "../register-backend/fixture.rs"]
mod fixture;
use serde_json::{json, Value};
use weft_core::compile::Compiler;
fn base() -> Value {
    let cases: Value =
        serde_json::from_str(include_str!("../../docs/helix/03-test/fixtures/cases.json")).unwrap();
    cases[0]["request"].clone()
}
fn compiler() -> Compiler {
    Compiler {
        registry: fixture::registry(
            weft_core::backend::Status::Supported,
            fixture::Behavior::Normal,
        ),
    }
}
fn run(c: &Compiler, r: &Value) -> Value {
    serde_json::from_str(&c.compile_json(&r.to_string())).unwrap()
}
#[test]
fn strict_envelope_and_atomic_refusals() {
    let c = Compiler::default();
    let r = base();
    let out = run(&c, &r);
    assert_eq!(out["status"], "blocked");
    assert_eq!(out["diagnostics"][0]["code"], "WFT-BACKEND-MISSING");
    assert!(out.get("sql").is_none());
    assert!(out.get("logicalPlan").is_none());
    assert!(out.get("parameters").is_none());
    assert_eq!(out["diagnostics"][0]["recoverability"], "change-profile");
    for (key, val, code) in [
        ("extra", json!(true), "WFT-INPUT"),
        (
            "interfaceVersion",
            json!("weft-compile/0.3.0"),
            "WFT-VERSION",
        ),
        ("options", json!({"allowCandidate":1}), "WFT-INPUT"),
        ("parameters", json!({}), "WFT-INPUT"),
    ] {
        let mut r = base();
        r[key] = val;
        assert_eq!(run(&c, &r)["diagnostics"][0]["code"], code);
    }
    let out: Value = serde_json::from_str(&c.compile_json("{}")).unwrap();
    assert_eq!(out["diagnostics"][0]["code"], "WFT-INPUT");
    let out: Value = serde_json::from_str(&c.compile_json("{\"sql\":1,\"sql\":2}")).unwrap();
    assert_eq!(out["diagnostics"][0]["code"], "WFT-JSON-DUPLICATE");
}
#[test]
fn compiled_artifacts_keep_versions_types_pins_and_qualifications() {
    let (catalog, _) = weft_core::prepare_and_resolve(
        "SELECT c.name AS label FROM Customer c",
        fixture::modules(),
    )
    .unwrap();
    let b = fixture::binding(&catalog);
    for app in [false, true] {
        let mut r = base();
        r["sql"] = json!("SELECT c.name AS label FROM Customer c");
        r["interfaceVersion"] = json!(if app {
            "weft-compile/0.2.0"
        } else {
            "weft-compile/0.1.0"
        });
        r["dialect"] = json!(if app {
            "weft-sql/0.2.0"
        } else {
            "weft-sql/0.1.0"
        });
        r["target"] = json!({"backendId":"test.third","backendVersion":"0.1.0","targetProfile":"fixture-only","bindingJson":b.json,"bindingSha256":b.sha256});
        let out = run(&compiler(), &r);
        assert_eq!(out["status"], "compiled", "{out}");
        assert_eq!(
            out["sql"],
            "SELECT \"display_name\" AS \"label\" FROM \"fixture_customers\""
        );
        assert_eq!(out["modelPins"][0], r["modules"][0]["pin"]);
        assert_eq!(out["backend"]["interfaceVersion"], "weft-backend/0.2.0");
        assert_eq!(
            out["qualification"]["operations"].as_array().unwrap().len(),
            3
        );
        assert_eq!(out["columns"][0]["outputName"], "label");
    }
}
#[test]
fn exact_limits_and_binding_digest_precede_resolution() {
    let c = Compiler::default();
    let mut r = base();
    r["sql"] = json!("not supported");
    r["target"]["bindingSha256"] = json!("0".repeat(64));
    assert_eq!(run(&c, &r)["diagnostics"][0]["code"], "WFT-PIN");
    r = base();
    r["sql"] = json!(" ".repeat(65537));
    assert_eq!(run(&c, &r)["diagnostics"][0]["code"], "WFT-LIMIT");
    let out: Value =
        serde_json::from_str(&c.compile_json(&" ".repeat(16 * 1024 * 1024 + 1))).unwrap();
    assert_eq!(out["diagnostics"][0]["code"], "WFT-LIMIT");
}
