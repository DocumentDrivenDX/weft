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

// @covers US-006-AC4 @covers US-002-AC4
#[test]
fn plugin_failures_are_atomic_at_the_public_boundary() {
    use fixture::Behavior;
    let (catalog,_)=weft_core::prepare_and_resolve("SELECT c.name AS label FROM Customer c",fixture::modules()).unwrap();
    let binding=fixture::binding(&catalog);
    for app in [false,true] {
        let mut request=base();
        request["sql"]=json!("SELECT c.name AS label FROM Customer c");
        request["interfaceVersion"]=json!(if app {"weft-compile/0.2.0"} else {"weft-compile/0.1.0"});
        request["dialect"]=json!(if app {"weft-sql/0.2.0"} else {"weft-sql/0.1.0"});
        request["target"]=json!({"backendId":"test.third","backendVersion":"0.1.0","targetProfile":"fixture-only","bindingJson":binding.json,"bindingSha256":binding.sha256});
        for (behavior,code) in [
            (Behavior::Panics,"WFT-BACKEND-FAILURE"),
            (Behavior::MissingCoverage,"WFT-BINDING"),
            (Behavior::MissingAssessment,"WFT-CAPABILITY"),
            (Behavior::LowerFailure,"WFT-CAPABILITY"),
            (Behavior::EmptySql,"WFT-EMIT"),
            (Behavior::WrongLabel,"WFT-EMIT"),
            (Behavior::MissingColumns,"WFT-EMIT"),
            (Behavior::WrongCarrier,"WFT-EMIT"),
            (Behavior::WrongType,"WFT-EMIT"),
            (Behavior::WrongNullable,"WFT-EMIT"),
            (Behavior::UnknownSource,"WFT-EMIT"),
            (Behavior::BadSlots,"WFT-EMIT"),
            (Behavior::ParameterLexical,"WFT-EMIT"),
        ] {
            let c=Compiler{registry:fixture::registry(weft_core::backend::Status::Supported,behavior)};
            let response=run(&c,&request);
            assert_eq!(response["status"],"blocked");assert_eq!(response["diagnostics"][0]["code"],code);
            for key in ["sql","parameters","logicalPlan","columns","obligations"] {assert!(response.get(key).is_none(),"{key}: {response}");}
        }
    }
}

// @covers US-006-AC2 @covers US-006-AC4
#[test]
fn request_admission_branches_refuse_atomically() {
    fn check(raw: &str, code: &str, message: &str, version: &str) {
        let out: Value = serde_json::from_str(&Compiler::default().compile_json(raw)).unwrap();
        assert_eq!(out["status"], "blocked");
        assert_eq!(out["interfaceVersion"],version);
        assert_eq!(out["diagnostics"][0]["code"],code);
        assert_eq!(out["diagnostics"][0]["message"],message);
        for key in ["sql","parameters","logicalPlan","columns","obligations"] { assert!(out.get(key).is_none(),"{key}"); }
    }
    for key in ["interfaceVersion","dialect"] {
        let mut r = base();r.as_object_mut().unwrap().remove(key);
        check(&r.to_string(),"WFT-INPUT","Compile and dialect versions must be supplied strings","weft-compile/0.1.0");
        let mut r = base();r[key]=json!(1);
        check(&r.to_string(),"WFT-INPUT","Compile and dialect versions must be supplied strings","weft-compile/0.1.0");
    }
    for (interface,dialect,output) in [
        ("weft-compile/0.1.0","weft-sql/0.2.0","weft-compile/0.1.0"),
        ("weft-compile/0.2.0","weft-sql/0.1.0","weft-compile/0.2.0"),
        ("weft-compile/99","weft-sql/0.1.0","weft-compile/0.1.0"),
    ] {
        let mut r=base();r["interfaceVersion"]=json!(interface);r["dialect"]=json!(dialect);
        check(&r.to_string(),"WFT-VERSION","Unsupported or mismatched compile/dialect versions",output);
    }
    for app in [false,true] {
        let mut r=base();
        if app {r["interfaceVersion"]=json!("weft-compile/0.2.0");r["dialect"]=json!("weft-sql/0.2.0");}
        let output=if app {"weft-compile/0.2.0"} else {"weft-compile/0.1.0"};
        r["extra"]=json!(true);
        check(&r.to_string(),"WFT-INPUT","Request violates its versioned envelope",output);
    }
    for (raw,code) in [("{","WFT-INPUT"),("{\"x\":1,\"x\":2}","WFT-JSON-DUPLICATE")] {
        let mut r=base();r["target"]["bindingJson"]=json!(raw);r["target"]["bindingSha256"]=json!(weft_core::json::sha256(raw.as_bytes()));
        check(&r.to_string(),code,"Invalid supplied binding JSON","weft-compile/0.1.0");
    }
}

// @covers US-002-AC1 @covers US-002-AC4 @covers US-006-AC2
#[test]
fn supplied_factory_selection_and_failure_do_not_fall_back() {
    use weft_core::{backend::{Plan,Registry,Status},error::Diagnostic};
    for app in [false,true] {
        let (catalog,_) = weft_core::prepare_and_resolve("SELECT c.name AS label FROM Customer c",fixture::modules()).unwrap();
        let binding=fixture::binding(&catalog);
        let mut r=base();r["sql"]=json!("SELECT c.name AS label FROM Customer c");
        if app {r["interfaceVersion"]=json!("weft-compile/0.2.0");r["dialect"]=json!("weft-sql/0.2.0");}
        r["target"]=json!({"backendId":"test.third","backendVersion":"0.1.0","targetProfile":"fixture-only","bindingJson":binding.json,"bindingSha256":binding.sha256});
        let host=compiler(); // Deliberately has a working registry: failure must not use it.
        for mode in [0,1,2] {
            let mut calls=0;
            let mut factory=|catalog: &weft_core::model::Catalog, plan: Plan<'_>, input: weft_core::compile::CompositionInput<'_>| {
                calls+=1;
                assert_eq!(catalog.pins()[0],fixture::modules()[0].pin);
                assert_eq!(matches!(plan,Plan::V02(_)),app);
                assert_eq!(input.backend_id,"test.third");assert_eq!(input.binding_sha256,binding.sha256);
                match mode {0=>Err(Diagnostic::new("WFT-BINDING","binding","Host composition refused")),1=>Ok(Registry::default()),_=>Ok(fixture::registry(Status::Supported,fixture::Behavior::Normal))}
            };
            let out:Value=serde_json::from_str(&host.compile_json_with_factory(&r.to_string(),&mut factory)).unwrap();
            assert_eq!(calls,1);
            if mode==2 {assert_eq!(out["status"],"compiled");assert_eq!(out["columns"][0]["outputName"],"label");}
            else {
                assert_eq!(out["status"],"blocked");
                assert_eq!(out["diagnostics"][0]["code"],if mode==0 {"WFT-BINDING"} else {"WFT-BACKEND-MISSING"});
                for key in ["sql","parameters","logicalPlan","columns","obligations"] {assert!(out.get(key).is_none(),"{key}");}
            }
        }
        let mut calls=0;
        let mut factory=|_: &weft_core::model::Catalog, _: Plan<'_>, _: weft_core::compile::CompositionInput<'_>| {calls+=1;Ok(Registry::default())};
        r["target"]["bindingSha256"]=json!("0".repeat(64));
        let out:Value=serde_json::from_str(&host.compile_json_with_factory(&r.to_string(),&mut factory)).unwrap();
        assert_eq!(out["diagnostics"][0]["code"],"WFT-PIN");assert_eq!(calls,0);
    }
}
