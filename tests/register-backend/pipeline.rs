// @covers US-002-AC1 @covers US-002-AC2 @covers US-002-AC3 @covers US-002-AC4
mod fixture;
use fixture::*;
use serde_json::{json, Value};
use weft_core::backend::*;
#[test]
fn third_backend_registration_for_both_plan_versions() {
    let (c, p) =
        weft_core::prepare_and_resolve("SELECT c.name AS label FROM Customer c", modules())
            .unwrap();
    let b = binding(&c);
    let r = registry(Status::Supported, Behavior::Normal);
    let out = r.compile(&c, Plan::V01(&p), &target(false), &b).unwrap();
    assert_eq!(
        out.emission.sql,
        "SELECT \"display_name\" AS \"label\" FROM \"fixture_customers\""
    );
    let (c, p) = weft_core::prepare_and_resolve_application(
        "SELECT c.name AS label FROM Customer c",
        modules(),
        Default::default(),
        None,
    )
    .unwrap();
    let out = r.compile(&c, Plan::V02(&p), &target(false), &b).unwrap();
    assert_eq!(out.emission.columns[0].output_name, "label");
}
#[test]
fn candidate_qualification_and_no_upgrade() {
    let (c, p) =
        weft_core::prepare_and_resolve("SELECT c.name FROM Customer c", modules()).unwrap();
    let b = binding(&c);
    let r = registry(Status::Candidate, Behavior::Normal);
    assert_eq!(
        r.compile(&c, Plan::V01(&p), &target(false), &b)
            .unwrap_err()
            .code,
        "WFT-CAPABILITY"
    );
    let out = r.compile(&c, Plan::V01(&p), &target(true), &b).unwrap();
    assert!(out
        .qualifications
        .iter()
        .all(|a| a.assessment.status == Status::Candidate));
    let r = registry(Status::Candidate, Behavior::Upgrade);
    assert_eq!(
        r.compile(&c, Plan::V01(&p), &target(true), &b)
            .unwrap_err()
            .code,
        "WFT-CAPABILITY"
    );
}
#[test]
fn atomic_refusals_and_no_implicit_selection() {
    let (c, p) =
        weft_core::prepare_and_resolve("SELECT c.name FROM Customer c", modules()).unwrap();
    let b = binding(&c);
    for (behavior, code) in [
        (Behavior::MissingCoverage, "WFT-BINDING"),
        (Behavior::MissingAssessment, "WFT-CAPABILITY"),
        (Behavior::Panics, "WFT-BACKEND-FAILURE"),
        (Behavior::EmptySql, "WFT-EMIT"),
        (Behavior::LowerFailure, "WFT-CAPABILITY"),
    ] {
        assert_eq!(
            registry(Status::Supported, behavior)
                .compile(&c, Plan::V01(&p), &target(false), &b)
                .unwrap_err()
                .code,
            code
        );
    }
    let mut t = target(false);
    t.backend_id = "unregistered".into();
    assert_eq!(
        registry(Status::Supported, Behavior::Normal)
            .compile(&c, Plan::V01(&p), &t, &b)
            .unwrap_err()
            .code,
        "WFT-BACKEND-MISSING"
    );
}
#[test]
fn binding_digest_revision_injection_and_selected_unknown_refuse() {
    let (c, p) =
        weft_core::prepare_and_resolve("SELECT c.name FROM Customer c", modules()).unwrap();
    let r = registry(Status::Supported, Behavior::Normal);
    for edit in [
        json!({"field":{"identity":{"documentId":"sales-fixture","revision":c.pins()[0].revision,"module":"sales","element":"customer-name"},"column":"display_name; DROP TABLE x"}}),
        json!({"record":{"identity":{"documentId":"sales-fixture","revision":"stale","module":"sales","element":"customer"},"table":"fixture_customers"}}),
        json!({"sql":"SELECT * FROM privileged"}),
    ] {
        let mut b = binding(&c);
        let mut v: Value = serde_json::from_str(&b.json).unwrap();
        for (k, x) in edit.as_object().unwrap() {
            v[k] = x.clone();
        }
        b.json = v.to_string();
        b.sha256 = weft_core::json::sha256(b.json.as_bytes());
        assert_eq!(
            r.compile(&c, Plan::V01(&p), &target(false), &b)
                .unwrap_err()
                .code,
            "WFT-BINDING"
        );
    }
    let mut b = binding(&c);
    b.sha256 = "0".repeat(64);
    assert_eq!(
        r.compile(&c, Plan::V01(&p), &target(false), &b)
            .unwrap_err()
            .code,
        "WFT-PIN"
    );
}

#[test]
fn emitter_cannot_change_outputs_or_parameter_domains() {
    let (c, p) =
        weft_core::prepare_and_resolve("SELECT c.name FROM Customer c", modules()).unwrap();
    let b = binding(&c);
    for behavior in [
        Behavior::WrongLabel,
        Behavior::MissingColumns,
        Behavior::WrongCarrier,
        Behavior::WrongType,
        Behavior::BadSlots,
        Behavior::ParameterLexical,
        Behavior::UnknownSource,
        Behavior::WrongNullable,
    ] {
        assert_eq!(
            registry(Status::Supported, behavior)
                .compile(&c, Plan::V01(&p), &target(false), &b)
                .unwrap_err()
                .code,
            "WFT-EMIT"
        );
    }
}
