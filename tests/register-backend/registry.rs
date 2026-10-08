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

#[test]
fn capability_declaration_guards_refuse_at_their_exact_phase() {
    let capability_error = "Capability domains, constraints and qualified evidence must be explicit";
    let identity_error = "Capabilities must bind distinct IDs to declared target and language profiles";
    let mut cases = Vec::new();
    for field in ["logicalDomain", "resultDomain"] {
        for value in [json!({}), json!([]), json!(false), json!(null)] {
            cases.push((field, value, "WFT-CAPABILITY", capability_error));
        }
    }
    for field in ["constraints", "evidence"] {
        for value in [json!([""]), json!(["bad\u{0}id"]), json!(["fixture-scan-1", "fixture-scan-1"])] {
            cases.push((field, value, "WFT-CAPABILITY", capability_error));
        }
    }
    cases.push(("evidence", json!([]), "WFT-CAPABILITY", capability_error));
    cases.push(("evidence", json!(["undeclared"]), "WFT-CAPABILITY", capability_error));
    for value in [json!(""), json!("bad\u{0}id")] {
        cases.push(("id", value, "WFT-BACKEND-VERSION", identity_error));
    }
    for value in [json!([]), json!([""]), json!(["fixture-only", "fixture-only"]), json!(["undeclared"])] {
        cases.push(("targetProfiles", value, "WFT-BACKEND-VERSION", identity_error));
    }
    for value in [json!([]), json!([{"dialectProfile":"weft-sql/0.1.0","irVersion":"weft-ir/0.2.0"}])] {
        cases.push(("languageProfiles", value, "WFT-BACKEND-VERSION", identity_error));
    }
    assert_eq!(cases.len(), 24);
    for (field, value, code, message) in cases {
        let mut m = manifest();
        m["capabilities"][0][field] = value;
        let error = validate(&m).unwrap_err();
        assert_eq!(error.code, code, "{field}");
        assert_eq!(error.message, message, "{field}");
    }
    let obligation = json!({"id":"fixture-guard", "parameters":{}, "owner":"host", "failureCode":"WFT-GUARD"});
    let mut valid = manifest();
    valid["capabilities"][0]["obligations"] = json!([obligation.clone()]);
    assert!(validate(&valid).is_ok());
    for (field, value) in [
        ("id", json!("")), ("id", json!("bad\u{0}id")),
        ("parameters", json!([])), ("parameters", json!(null)),
        ("failureCode", json!("GUARD")), ("failureCode", json!("WFT-")),
        ("failureCode", json!("WFT-lower")), ("failureCode", json!("WFT-☃")),
    ] {
        let mut m = valid.clone();
        m["capabilities"][0]["obligations"][0][field] = value;
        let error = validate(&m).unwrap_err();
        assert_eq!(error.code, "WFT-OBLIGATION", "{field}");
        assert_eq!(error.message, "Capability obligations are malformed or ambiguous", "{field}");
    }
    valid["capabilities"][0]["obligations"] = json!([obligation.clone(), obligation]);
    let error = validate(&valid).unwrap_err();
    assert_eq!(error.code, "WFT-OBLIGATION");
    assert_eq!(error.message, "Capability obligations are malformed or ambiguous");
}

#[test]
fn manifest_collection_and_byte_limits_admit_the_boundary() {
    use weft_core::backend::validate_manifest;
    let baseline = validate(&manifest()).unwrap();
    let mut m = baseline.clone();
    m.target_profiles = (0..256).map(|i| { let mut p=baseline.target_profiles[0].clone();p.id=format!("target-{i}");p }).collect();
    m.capabilities[0].target_profiles = vec!["target-0".into()];
    assert!(validate_manifest(&m).is_ok());
    let mut extra=m.target_profiles[0].clone();extra.id="target-256".into();m.target_profiles.push(extra);
    let error=validate_manifest(&m).unwrap_err();
    assert_eq!(error.code,"WFT-BACKEND-VERSION");
    assert_eq!(error.message,"Backend manifest identity, collection bounds or evidence IDs are invalid");
    let mut m=baseline.clone();
    m.capabilities=(0..4096).map(|i|{let mut c=baseline.capabilities[0].clone();c.id=format!("cap-{i}");c}).collect();
    assert!(validate_manifest(&m).is_ok());
    let mut extra=m.capabilities[0].clone();extra.id="cap-4096".into();m.capabilities.push(extra);
    let error=validate_manifest(&m).unwrap_err();
    assert_eq!(error.message,"Backend manifest identity, collection bounds or evidence IDs are invalid");
    let mut raw=manifest().to_string();raw.extend(std::iter::repeat_n(' ',1024*1024-raw.len()));
    assert_eq!(raw.len(),1024*1024);
    assert!(validate_manifest_json(&raw).is_ok());
    raw.push(' ');
    let error=validate_manifest_json(&raw).unwrap_err();
    assert_eq!(error.code,"WFT-LIMIT");assert_eq!(error.message,"Backend manifest exceeds one MiB");
}

#[test]
fn manifest_identity_evidence_and_language_guards_are_independent() {
    for field in ["backendId","backendVersion","bindingProfile"] {
        for value in [json!(""),json!("invalid\u{0}identity")] {
            let mut m=manifest();m[field]=value;let error=validate(&m).unwrap_err();
            assert_eq!(error.code,"WFT-BACKEND-VERSION");
            assert_eq!(error.message,"Backend manifest identity, collection bounds or evidence IDs are invalid");
        }
    }
    for (field,value) in [("targetProfiles",json!([])),("capabilities",json!([])),("evidence",json!([""])),("evidence",json!(["bad\u{0}id"])),("evidence",json!(["fixture-scan-1","fixture-scan-1"]))] {
        let mut m=manifest();m[field]=value;let error=validate(&m).unwrap_err();
        assert_eq!(error.message,"Backend manifest identity, collection bounds or evidence IDs are invalid");
    }
    for duplicate in [false,true] {
        let mut m=manifest();
        m["languageProfiles"]=if duplicate {json!([m["languageProfiles"][0].clone(),m["languageProfiles"][0].clone()])} else {json!([])};
        let error=validate(&m).unwrap_err();assert_eq!(error.code,"WFT-BACKEND-VERSION");
        assert_eq!(error.message,"Unsupported backend interface or language/IR pair");
    }
    let mut m=manifest();m["languageProfiles"]=json!([m["languageProfiles"][0].clone()]);
    let error=validate(&m).unwrap_err();
    assert_eq!(error.message,"Capabilities must bind distinct IDs to declared target and language profiles");
}
