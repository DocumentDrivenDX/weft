//! Backend registration refuses ambiguous or unqualified declarations before code is selected.
// @covers US-002-AC2 @covers US-002-AC3 @covers US-002-AC4
use serde_json::{json, Value};
use weft_core::backend::{validate_manifest_json, Manifest};
fn manifest() -> Value {
    json!({
     "backendId":"test.third","backendVersion":"0.1.0","interfaceVersion":"weft-backend/0.2.0",
     "languageProfiles":[{"dialectProfile":"weft-sql/0.1.0","irVersion":"weft-ir/0.1.0"},{"dialectProfile":"weft-sql/0.2.0","irVersion":"weft-ir/0.2.0"}],
     "bindingProfile":"test-third-binding/0.1.0",
     "targetProfiles":[{"id":"fixture-only","engine":"synthetic","engineVersion":"test-1","sessionSettings":{"comparison":"unicode-scalar"},"storageLayoutRevision":"fixture-1","publicationRevision":"fixture-1"}],
     "capabilities":[{"id":"scan","targetProfiles":["fixture-only"],"languageProfiles":[{"dialectProfile":"weft-sql/0.1.0","irVersion":"weft-ir/0.1.0"},{"dialectProfile":"weft-sql/0.2.0","irVersion":"weft-ir/0.2.0"}],"logicalDomain":{"record":"explicit mapping"},"resultDomain":{"bag":"preserved"},"constraints":[],"obligations":[],"status":"supported","evidence":["fixture-scan-1"]}],
     "evidence":["fixture-scan-1"]
    })
}
fn validate(v: &Value) -> Result<Manifest, weft_core::error::Diagnostic> {
    validate_manifest_json(&v.to_string())
}
#[test]
fn manifest_versions_and_profile_capability_binding() {
    let m = validate(&manifest()).unwrap();
    assert_eq!(m.backend_id, "test.third");
    assert_eq!(m.capabilities[0].target_profiles, vec!["fixture-only"]);
    for change in [
        json!({"interfaceVersion":"weft-backend/0.1.0"}),
        json!({"loadCode":"https://evil.test/plugin.js"}),
        json!({"languageProfiles":[{"dialectProfile":"weft-sql/0.2.0","irVersion":"weft-ir/0.1.0"}]}),
    ] {
        let mut m = manifest();
        for (k, v) in change.as_object().unwrap() {
            m[k] = v.clone();
        }
        assert!(validate(&m).is_err());
    }
    let mut m = manifest();
    m["capabilities"][0]["targetProfiles"] = json!(["undeclared"]);
    assert!(validate(&m).is_err());
}
#[test]
fn supported_requires_declared_evidence_candidate_remains_distinct() {
    let mut m = manifest();
    m["capabilities"][0]["evidence"] = json!([]);
    assert!(validate(&m).is_err());
    m["capabilities"][0]["status"] = json!("candidate");
    assert!(validate(&m).is_ok());
    m["capabilities"][0]["status"] = json!("supported");
    m["capabilities"][0]["evidence"] = json!(["unknown-evidence"]);
    assert!(validate(&m).is_err());
}
#[test]
fn duplicate_declarations_and_unknown_shapes_refuse() {
    let mut m = manifest();
    let second = m["targetProfiles"][0].clone();
    m["targetProfiles"].as_array_mut().unwrap().push(second);
    assert!(validate(&m).is_err());
    let mut m = manifest();
    let second = m["capabilities"][0].clone();
    m["capabilities"].as_array_mut().unwrap().push(second);
    assert!(validate(&m).is_err());
    let mut m = manifest();
    m["capabilities"][0]["logicalDomain"] = json!(true);
    assert!(validate(&m).is_err());
    let mut m = manifest();
    m["capabilities"][0]["obligations"] = json!([{"id":"exact-or-error","parameters":{},"owner":"query","failureCode":"WFT-NUMERIC-DOMAIN"}]);
    assert!(validate(&m).is_err());
    let raw = manifest().to_string().replace(
        "\"backendId\":\"test.third\"",
        "\"backendId\":\"test.third\",\"backendId\":\"test.other\"",
    );
    assert!(validate_manifest_json(&raw).is_err());
}

#[test]
fn target_profiles_require_each_identity_and_object_settings() {
    // A manifest is a declaration, not a native qualification receipt.
    let admitted = validate(&manifest()).unwrap();
    assert_eq!(admitted.target_profiles[0].engine_version, "test-1");
    assert_eq!(admitted.target_profiles[0].session_settings, json!({"comparison":"unicode-scalar"}));
    for field in ["id", "engine", "engineVersion", "storageLayoutRevision", "publicationRevision"] {
        for invalid in ["", "invalid\0identity"] {
            let mut m = manifest();
            m["targetProfiles"][0][field] = json!(invalid);
            let error = validate(&m).unwrap_err();
            assert_eq!(error.code, "WFT-BACKEND-VERSION", "{field}");
            assert_eq!(error.message, "Target profiles must be distinct, pinned and structurally explicit", "{field}");
        }
    }
    for settings in [json!(null), json!(false), json!(1), json!("ANSI"), json!([])] {
        let mut m = manifest();
        m["targetProfiles"][0]["sessionSettings"] = settings;
        let error = validate(&m).unwrap_err();
        assert_eq!(error.code, "WFT-BACKEND-VERSION");
        assert_eq!(error.message, "Target profiles must be distinct, pinned and structurally explicit");
    }
}
