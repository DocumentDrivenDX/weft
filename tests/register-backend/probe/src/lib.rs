//! Test-only third-backend probe; no public compile envelope or production registration.
#[path = "../../fixture.rs"]
mod fixture;
use serde_json::{json, Value};
use weft_core::{
    backend::{BindingInput, Plan, Status},
    error::{Diagnostic, Result},
};
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn backend_json(raw: &str) -> String {
    fn run(raw: &str) -> Result<Value> {
        let request = weft_core::json::checked_json(raw)
            .map_err(|_| Diagnostic::new("WFT-INPUT", "input", "Invalid test probe JSON"))?;
        let sql = request["sql"]
            .as_str()
            .ok_or_else(|| Diagnostic::new("WFT-INPUT", "input", "Missing SQL"))?;
        let modules = serde_json::from_value(request["modules"].clone())
            .map_err(|_| Diagnostic::new("WFT-INPUT", "input", "Missing supplied modules"))?;
        let status = if request["candidate"] == true {
            Status::Candidate
        } else {
            Status::Supported
        };
        let behavior = match request["behavior"].as_str().unwrap_or("normal") {
            "normal" => fixture::Behavior::Normal,
            "missing-coverage" => fixture::Behavior::MissingCoverage,
            "missing-assessment" => fixture::Behavior::MissingAssessment,
            "empty-sql" => fixture::Behavior::EmptySql,
            "wrong-label" => fixture::Behavior::WrongLabel,
            "wrong-type" => fixture::Behavior::WrongType,
            "wrong-carrier" => fixture::Behavior::WrongCarrier,
            "bad-slots" => fixture::Behavior::BadSlots,
            "parameter-lexical" => fixture::Behavior::ParameterLexical,
            "lower-failure" => fixture::Behavior::LowerFailure,
            _ => {
                return Err(Diagnostic::new(
                    "WFT-INPUT",
                    "input",
                    "Unknown test behavior",
                ))
            }
        };
        let registry = fixture::registry(status, behavior);
        let mut target = fixture::target(request["allowCandidate"] == true);
        if let Some(id) = request["backendId"].as_str() {
            target.backend_id = id.into();
        }
        if let Some(version) = request["backendVersion"].as_str() {
            target.backend_version = version.into();
        }
        if let Some(profile) = request["targetProfile"].as_str() {
            target.profile_id = profile.into();
        }
        fn compile(
            registry: &weft_core::backend::Registry,
            catalog: &weft_core::model::Catalog,
            plan: Plan<'_>,
            target: &weft_core::backend::Target,
            request: &Value,
        ) -> Result<Value> {
            let binding = if request.get("bindingJson").is_some() {
                BindingInput {
                    profile: request["bindingProfile"]
                        .as_str()
                        .unwrap_or("test-third-binding/0.1.0")
                        .into(),
                    json: request["bindingJson"].as_str().unwrap_or("").into(),
                    sha256: request["bindingSha256"].as_str().unwrap_or("").into(),
                }
            } else {
                fixture::binding(catalog)
            };
            let compilation = registry.compile(catalog, plan, target, &binding)?;
            Ok(
                json!({"status":"emitted","probeVersion":"weft-backend-probe/0.1.0","compilation":compilation,"manifest":registry.manifest("test.third"),"retainedModules":catalog.inputs()}),
            )
        }
        match request["dialect"].as_str() {
            Some("weft-sql/0.1.0") => {
                let (c, p) = weft_core::prepare_and_resolve(sql, modules)?;
                compile(&registry, &c, Plan::V01(&p), &target, &request)
            }
            Some("weft-sql/0.2.0") => {
                let (c, p) = weft_core::prepare_and_resolve_application(
                    sql,
                    modules,
                    Default::default(),
                    None,
                )?;
                compile(&registry, &c, Plan::V02(&p), &target, &request)
            }
            _ => Err(Diagnostic::new(
                "WFT-VERSION",
                "input",
                "Probe needs an explicit supported dialect",
            )),
        }
    }
    serde_json::to_string(&match run(raw) {
        Ok(v) => v,
        Err(d) => json!({"status":"blocked","diagnostics":[d]}),
    })
    .unwrap()
}
/// Explicit test-build composition; input/model data never selects executable code.
pub fn fixture_registry() -> weft_core::backend::Registry {
    fixture::registry(
        weft_core::backend::Status::Supported,
        fixture::Behavior::Normal,
    )
}
/// Test-build registry with independently selectable supported/candidate declarations.
pub fn compile_fixture_registry() -> weft_core::backend::Registry {
    let mut registry = fixture_registry();
    let mut manifest = fixture::manifest(weft_core::backend::Status::Candidate);
    manifest.backend_id = "test.third.candidate".into();
    registry
        .register(fixture::Third {
            manifest,
            behavior: fixture::Behavior::Normal,
        })
        .unwrap();
    registry
}
