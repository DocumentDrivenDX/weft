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

// @covers US-002-AC2 @covers US-002-AC4
#[test]
fn registration_description_errors_are_atomic() {
    struct Description(u8);
    impl Backend for Description {
        type Mapping=();type TargetPlan=();
        fn describe(&self)->weft_core::error::Result<Manifest> {
            match self.0 {
                0=>panic!("description panic control"),
                1=>Err(weft_core::error::Diagnostic::new("WFT-BINDING","binding","description refused")),
                2=>{let mut m=manifest(Status::Supported);m.interface_version="unknown".into();Ok(m)},
                _=>Ok(manifest(Status::Supported)),
            }
        }
        fn validate_binding(&self,_:&Context<'_>)->weft_core::error::Result<Validated<()>> {panic!("registration must not validate binding")}
        fn assess(&self,_:&Context<'_>,_:&())->weft_core::error::Result<Vec<Assessment>> {panic!("registration must not assess")}
        fn lower(&self,_:&Context<'_>,_:&())->weft_core::error::Result<()> {panic!("registration must not lower")}
        fn emit(&self,_:&Context<'_>,_:&())->weft_core::error::Result<Emission> {panic!("registration must not emit")}
    }
    for (mode,code) in [(0,"WFT-BACKEND-FAILURE"),(1,"WFT-BINDING"),(2,"WFT-BACKEND-VERSION")] {
        let mut r=Registry::default();let error=r.register(Description(mode)).unwrap_err();assert_eq!(error.code,code);
        assert!(r.manifest("test.third").is_none());
        r.register(Description(3)).unwrap();assert!(r.manifest("test.third").is_some());
        let error=r.register(Description(3)).unwrap_err();assert_eq!(error.code,"WFT-BACKEND-VERSION");
        assert_eq!(error.message,"Backend identity is already registered");
        assert_eq!(r.manifest("test.third").unwrap().backend_version,"0.1.0");
    }
}

// @covers US-002-AC2 @covers US-006-AC2 @covers US-006-AC4
#[test]
fn adapter_dispatch_guards_refuse_before_lowering() {
    let (catalog,plan)=weft_core::prepare_and_resolve("SELECT c.name FROM Customer c",modules()).unwrap();
    let registry=registry(Status::Supported,Behavior::Normal);
    for mode in 0..10 {
        let mut p=plan.clone();let mut t=target(false);let mut b=binding(&catalog);
        let (code,message)=match mode {
            0=>{t.backend_version="99".into();("WFT-BACKEND-VERSION","Selected backend version is not registered")},
            1=>{p.ir_version="99".into();("WFT-BACKEND-VERSION","Typed plan version or operation identities are invalid")},
            2=>{p.required_capabilities.push(p.required_capabilities[0].clone());("WFT-BACKEND-VERSION","Typed plan version or operation identities are invalid")},
            3=>{t.profile_id="missing".into();("WFT-BACKEND-VERSION","Selected target profile is not registered")},
            4=>{b.profile="missing".into();("WFT-BINDING","Binding profile does not match the selected registered backend")},
            5=>{p.module_pins[0].revision="stale".into();("WFT-PIN","Plan model pins do not match the supplied catalog")},
            6=>{b.json=" ".repeat(4*1024*1024+1);("WFT-LIMIT","Binding exceeds four MiB")},
            7=>{b.sha256="0".repeat(64);("WFT-PIN","Binding byte digest mismatch")},
            8=>{b.json="{".into();b.sha256=weft_core::json::sha256(b.json.as_bytes());("WFT-BINDING","Binding JSON is malformed or repeats members")},
            _=>{b.json="{\"x\":1,\"x\":2}".into();b.sha256=weft_core::json::sha256(b.json.as_bytes());("WFT-BINDING","Binding JSON is malformed or repeats members")},
        };
        let error=registry.compile(&catalog,Plan::V01(&p),&t,&b).unwrap_err();assert_eq!(error.code,code,"mode {mode}");assert_eq!(error.message,message,"mode {mode}");
    }
}

// @covers US-002-AC2 @covers US-002-AC3 @covers US-002-AC4
#[test]
fn capability_assessment_guards_prevent_lowering() {
    struct Altered { mode:u8, inner:Third }
    impl Backend for Altered {
        type Mapping=Mapping;type TargetPlan=Select;
        fn describe(&self)->weft_core::error::Result<Manifest>{self.inner.describe()}
        fn validate_binding(&self,c:&Context<'_>)->weft_core::error::Result<Validated<Mapping>>{
            let mut v=self.inner.validate_binding(c)?;
            if self.mode==0 {v.additional_capabilities=vec!["extra".into(),"extra".into()];}
            if self.mode==1 {v.additional_capabilities=(0..4097).map(|i|format!("extra-{i}")).collect();}
            Ok(v)
        }
        fn assess(&self,c:&Context<'_>,m:&Mapping)->weft_core::error::Result<Vec<Assessment>>{
            let mut a=self.inner.assess(c,m)?;
            match self.mode {
                2=>a.push(a[0].clone()),3=>a[0].id="unrequested".into(),4=>a[0].status=Status::Unsupported,
                5=>{let proof=a[0].evidence[0].clone();a[0].evidence.push(proof);},6=>a[0].evidence=vec!["untrusted-proof".into()],7=>a[0].evidence.clear(),_=>{}
            }
            Ok(a)
        }
        fn lower(&self,_:&Context<'_>,_:&Mapping)->weft_core::error::Result<Select>{panic!("guard must precede lowering")}
        fn emit(&self,_:&Context<'_>,_:&Select)->weft_core::error::Result<Emission>{panic!("guard must precede emission")}
    }
    let (catalog,plan)=weft_core::prepare_and_resolve("SELECT c.name FROM Customer c",modules()).unwrap();
    for mode in 0..8 {
        let mut r=Registry::default();r.register(Altered{mode,inner:Third{manifest:manifest(Status::Supported),behavior:Behavior::Normal}}).unwrap();
        let error=r.compile(&catalog,Plan::V01(&plan),&target(false),&binding(&catalog)).unwrap_err();
        assert_eq!(error.code,"WFT-CAPABILITY","mode {mode}");
        assert_eq!(error.message,match mode {0|1=>"Binding-derived capabilities must be distinct and bounded",2|3=>"Assessment has duplicate or unrequested operations",4=>"Selected operation is unsupported",_=>"Assessment has missing or undeclared qualification evidence"},"mode {mode}");
    }
}

// @covers US-002-AC4 @covers US-006-AC4
#[test]
fn obligation_merge_refuses_conflicts_and_preserves_requirements() {
    fn obligation(id:&str)->Obligation {Obligation{id:id.into(),parameters:json!({}),owner:ObligationOwner::Host,failure_code:"WFT-BINDING".into()}}
    struct Obligations {mode:u8,inner:Third}
    impl Backend for Obligations {
        type Mapping=Mapping;type TargetPlan=Select;
        fn describe(&self)->weft_core::error::Result<Manifest>{self.inner.describe()}
        fn validate_binding(&self,c:&Context<'_>)->weft_core::error::Result<Validated<Mapping>>{
            let mut v=self.inner.validate_binding(c)?;let mut o=obligation("z");
            match self.mode {0=>o.id.clear(),1=>o.parameters=json!([]),2=>o.failure_code="OTHER".into(),3=>o.failure_code="WFT-".into(),4=>o.failure_code="WFT-lower".into(),5=>o.failure_code="WFT-É".into(),_=>{}}
            v.obligations.push(o.clone());
            if self.mode==6 {o.parameters=json!({"changed":true});v.obligations.push(o.clone());}
            if self.mode==7 {o.owner=ObligationOwner::Backend;v.obligations.push(o.clone());}
            if self.mode==8 {v.obligations.extend([o,obligation("a")]);}
            Ok(v)
        }
        fn assess(&self,c:&Context<'_>,m:&Mapping)->weft_core::error::Result<Vec<Assessment>>{self.inner.assess(c,m)}
        fn lower(&self,c:&Context<'_>,m:&Mapping)->weft_core::error::Result<Select>{self.inner.lower(c,m)}
        fn emit(&self,c:&Context<'_>,p:&Select)->weft_core::error::Result<Emission>{self.inner.emit(c,p)}
    }
    let (catalog,plan)=weft_core::prepare_and_resolve("SELECT c.name FROM Customer c",modules()).unwrap();
    for mode in 0..9 {
        let mut r=Registry::default();r.register(Obligations{mode,inner:Third{manifest:manifest(Status::Supported),behavior:Behavior::Normal}}).unwrap();
        let out=r.compile(&catalog,Plan::V01(&plan),&target(false),&binding(&catalog));
        if mode==8 {assert_eq!(out.unwrap().emission.obligations,vec![obligation("a"),obligation("z")]);}
        else {let e=out.unwrap_err();assert_eq!(e.code,"WFT-OBLIGATION");assert_eq!(e.message,if mode<6 {"Backend returned a malformed obligation"} else {"Obligation ID has conflicting requirements"});}
    }
}

// @covers US-002-AC2 @covers US-002-AC4
#[test]
fn language_and_binding_root_guards_precede_backend_validation() {
    let (catalog,p01)=weft_core::prepare_and_resolve("SELECT c.name FROM Customer c",modules()).unwrap();
    let (_,p02)=weft_core::prepare_and_resolve_application("SELECT c.name FROM Customer c",modules(),Default::default(),None).unwrap();
    for (index,plan) in [Plan::V01(&p01),Plan::V02(&p02)].into_iter().enumerate() {
        let mut m=manifest(Status::Supported);
        m.language_profiles.remove(index);
        for capability in &mut m.capabilities {capability.language_profiles=m.language_profiles.clone();}
        let mut r=Registry::default();
        // Any accidental call into the fixture backend panics and changes the diagnostic.
        r.register(Third{manifest:m,behavior:Behavior::Panics}).unwrap();
        let e=r.compile(&catalog,plan,&target(false),&binding(&catalog)).unwrap_err();
        assert_eq!(e.code,"WFT-BACKEND-VERSION");
        assert_eq!(e.message,"Registered backend does not accept the selected dialect/IR pair");
        let r=registry(Status::Supported,Behavior::Panics);
        for root in ["null","true","false","0","\"binding\"","[]"] {
            let mut b=binding(&catalog);b.json=root.into();b.sha256=weft_core::json::sha256(b.json.as_bytes());
            let e=r.compile(&catalog,plan,&target(false),&b).unwrap_err();
            assert_eq!(e.code,"WFT-BINDING","root {root}");
            assert_eq!(e.message,"Binding root must be an object","root {root}");
        }
    }
}

// @covers US-002-AC2 @covers US-002-AC4
#[test]
fn selected_identity_coverage_is_revision_exact() {
    struct Partial { mode:u8, inner:Third }
    impl Backend for Partial {
        type Mapping=Mapping;type TargetPlan=Select;
        fn describe(&self)->weft_core::error::Result<Manifest>{self.inner.describe()}
        fn validate_binding(&self,c:&Context<'_>)->weft_core::error::Result<Validated<Mapping>> {
            let mut v=self.inner.validate_binding(c)?;
            let identities=match self.mode%2 {0=>&mut v.coverage.records,_=>&mut v.coverage.fields};
            assert!(!identities.is_empty(),"fixture must select this identity class");
            if self.mode<2 {identities.clear();} else {identities[0].revision="different-revision".into();}
            Ok(v)
        }
        fn assess(&self,_:&Context<'_>,_:&Mapping)->weft_core::error::Result<Vec<Assessment>>{panic!("incomplete mapping must not reach assessment")}
        fn lower(&self,_:&Context<'_>,_:&Mapping)->weft_core::error::Result<Select>{panic!("incomplete mapping must not lower")}
        fn emit(&self,_:&Context<'_>,_:&Select)->weft_core::error::Result<Emission>{panic!("incomplete mapping must not emit")}
    }
    let (catalog,p01)=weft_core::prepare_and_resolve("SELECT c.name FROM Customer c",modules()).unwrap();
    let (_,p02)=weft_core::prepare_and_resolve_application("SELECT c.name FROM Customer c",modules(),Default::default(),None).unwrap();
    for plan in [Plan::V01(&p01),Plan::V02(&p02)] {
        for mode in 0..4 {
            let mut r=Registry::default();r.register(Partial{mode,inner:Third{manifest:manifest(Status::Supported),behavior:Behavior::Normal}}).unwrap();
            let e=r.compile(&catalog,plan,&target(false),&binding(&catalog)).unwrap_err();
            assert_eq!(e.code,"WFT-BINDING","mode {mode}");
            assert_eq!(e.message,"Backend mapping omits a selected record, field, type or relationship identity","mode {mode}");
        }
    }
}

// @covers US-002-AC2 @covers US-002-AC4 @covers US-007-AC4
#[test]
fn selected_type_and_relationship_coverage_refuse_before_assessment() {
    struct Partial { mode:u8 }
    impl Backend for Partial {
        type Mapping=();type TargetPlan=();
        fn describe(&self)->weft_core::error::Result<Manifest>{Ok(manifest(Status::Supported))}
        fn validate_binding(&self,c:&Context<'_>)->weft_core::error::Result<Validated<()>> {
            let mut coverage=c.selection.clone();
            if self.mode<2 {
                assert!(!coverage.types.is_empty(),"fixture must select types");
                if self.mode==0 {coverage.types.clear();} else {coverage.types[0].revision="wrong".into();}
            } else {
                assert!(!coverage.relationships.is_empty(),"fixture must select relationships");
                if self.mode==2 {coverage.relationships.clear();} else {coverage.relationships[0].revision="wrong".into();}
            }
            Ok(Validated{mapping:(),coverage,additional_capabilities:vec![],obligations:vec![]})
        }
        fn assess(&self,_:&Context<'_>,_:&())->weft_core::error::Result<Vec<Assessment>>{panic!("incomplete coverage must precede assessment")}
        fn lower(&self,_:&Context<'_>,_:&())->weft_core::error::Result<()>{panic!("incomplete coverage must not lower")}
        fn emit(&self,_:&Context<'_>,_:&())->weft_core::error::Result<Emission>{panic!("incomplete coverage must not emit")}
    }
    let cases:Value=serde_json::from_str(include_str!("../application/fixtures/cases.json")).unwrap();
    for (id,modes) in [("whole-entity",0..2),("related-page",2..4)] {
        let request=&cases.as_array().unwrap().iter().find(|c|c["id"]==id).unwrap()["request"];
        let inputs=serde_json::from_value(request["modules"].clone()).unwrap();
        let (catalog,plan)=weft_core::prepare_and_resolve_application(request["sql"].as_str().unwrap(),inputs,Default::default(),None).unwrap();
        for mode in modes {
            let mut r=Registry::default();r.register(Partial{mode}).unwrap();
            let e=r.compile(&catalog,Plan::V02(&plan),&target(false),&binding(&catalog)).unwrap_err();
            assert_eq!(e.code,"WFT-BINDING","case {id}, mode {mode}");
            assert_eq!(e.message,"Backend mapping omits a selected record, field, type or relationship identity");
        }
    }
}

// @covers US-002-AC2 @covers US-002-AC3 @covers US-002-AC4
#[test]
fn capability_declarations_bind_selected_target_language_and_status() {
    struct Declaration { manifest:Manifest,inner:Third }
    impl Backend for Declaration {
        type Mapping=Mapping;type TargetPlan=Select;
        fn describe(&self)->weft_core::error::Result<Manifest>{Ok(self.manifest.clone())}
        fn validate_binding(&self,c:&Context<'_>)->weft_core::error::Result<Validated<Mapping>>{self.inner.validate_binding(c)}
        fn assess(&self,c:&Context<'_>,m:&Mapping)->weft_core::error::Result<Vec<Assessment>>{self.inner.assess(c,m)}
        fn lower(&self,_:&Context<'_>,_:&Mapping)->weft_core::error::Result<Select>{panic!("invalid declaration must not lower")}
        fn emit(&self,_:&Context<'_>,_:&Select)->weft_core::error::Result<Emission>{panic!("invalid declaration must not emit")}
    }
    let (catalog,p01)=weft_core::prepare_and_resolve("SELECT c.name FROM Customer c",modules()).unwrap();
    let (_,p02)=weft_core::prepare_and_resolve_application("SELECT c.name FROM Customer c",modules(),Default::default(),None).unwrap();
    for (index,plan) in [Plan::V01(&p01),Plan::V02(&p02)].into_iter().enumerate() {
        for mode in 0..4 {
            let mut m=manifest(Status::Supported);
            match mode {
                0=>{m.capabilities.remove(0);},
                1=>{let mut other=m.target_profiles[0].clone();other.id="other-target".into();m.target_profiles.push(other);m.capabilities[0].target_profiles=vec!["other-target".into()];},
                2=>{m.capabilities[0].language_profiles.remove(index);},
                _=>m.capabilities[0].status=Status::Unsupported,
            }
            let mut r=Registry::default();r.register(Declaration{manifest:m,inner:Third{manifest:manifest(Status::Supported),behavior:Behavior::Normal}}).unwrap();
            let e=r.compile(&catalog,plan,&target(false),&binding(&catalog)).unwrap_err();
            assert_eq!(e.code,"WFT-CAPABILITY","mode {mode}");
            assert_eq!(e.message,if mode==3 {"Selected operation is unsupported"} else {"Operation is not declared for the selected target/language profile"},"mode {mode}");
        }
    }
}

// @covers US-002-AC4 @covers US-006-AC4
#[test]
fn obligations_merge_across_declaration_assessment_and_emission() {
    fn requirement(value:u8)->Obligation {Obligation{id:"shared".into(),parameters:json!({"version":value}),owner:ObligationOwner::Host,failure_code:"WFT-BINDING".into()}}
    struct CrossPhase {mode:u8,inner:Third}
    impl Backend for CrossPhase {
        type Mapping=Mapping;type TargetPlan=Select;
        fn describe(&self)->weft_core::error::Result<Manifest>{
            let mut m=self.inner.describe()?;m.capabilities[0].obligations=vec![requirement(1)];Ok(m)
        }
        fn validate_binding(&self,c:&Context<'_>)->weft_core::error::Result<Validated<Mapping>>{self.inner.validate_binding(c)}
        fn assess(&self,c:&Context<'_>,m:&Mapping)->weft_core::error::Result<Vec<Assessment>>{
            let mut a=self.inner.assess(c,m)?;a[0].obligations=vec![requirement(if self.mode==0 {2} else {1})];Ok(a)
        }
        fn lower(&self,c:&Context<'_>,m:&Mapping)->weft_core::error::Result<Select>{
            assert_ne!(self.mode,0,"assessment conflict must refuse before lowering");self.inner.lower(c,m)
        }
        fn emit(&self,c:&Context<'_>,p:&Select)->weft_core::error::Result<Emission>{
            let mut e=self.inner.emit(c,p)?;e.obligations=vec![requirement(if self.mode==1 {2} else {1})];Ok(e)
        }
    }
    let (catalog,p01)=weft_core::prepare_and_resolve("SELECT c.name FROM Customer c",modules()).unwrap();
    let (_,p02)=weft_core::prepare_and_resolve_application("SELECT c.name FROM Customer c",modules(),Default::default(),None).unwrap();
    for plan in [Plan::V01(&p01),Plan::V02(&p02)] {
        for mode in 0..3 {
            let mut r=Registry::default();r.register(CrossPhase{mode,inner:Third{manifest:manifest(Status::Supported),behavior:Behavior::Normal}}).unwrap();
            let out=r.compile(&catalog,plan,&target(false),&binding(&catalog));
            if mode==2 {assert_eq!(out.unwrap().emission.obligations,vec![requirement(1)]);}
            else {let e=out.unwrap_err();assert_eq!(e.code,"WFT-OBLIGATION");assert_eq!(e.message,"Obligation ID has conflicting requirements");}
        }
    }
}
