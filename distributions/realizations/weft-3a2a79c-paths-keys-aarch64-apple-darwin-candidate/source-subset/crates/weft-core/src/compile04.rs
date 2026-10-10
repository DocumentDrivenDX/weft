//! Explicit 0.4 transport. The existing Compiler and Backend02 factory stay intact.
use super::{public_diagnostic, CompositionInput, Request};
use crate::{
    backend03::{BindingInput, Plan04View, Registry, Status, Target},
    error::{Diagnostic, Result},
    model::Catalog,
};
use serde_json::{json, Value};

#[jsonschema::validator(
    path = "../../docs/helix/02-design/contracts/compile-request-v0.4.schema.json"
)]
struct Request04;

/// Hosts select registered code explicitly; SQL and models never provide code.
pub type RegistryFactory<'a> =
    dyn FnMut(&Catalog, Plan04View<'_>, CompositionInput<'_>) -> Result<Registry> + 'a;

/// Pure compiler for the exact compile/language 0.4 pair and Backend03.
///
/// The caller supplies registration and request configuration explicitly. A
/// compiled response describes obligations; it does not execute SQL, observe
/// native schemas, or discharge the host's publication and release checks.
#[derive(Default)]
pub struct Compiler {
    pub registry: Registry,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn request() -> Value {
        let document = include_str!("../../../tests/fixtures/original-commerce-0.8/ontology.json");
        json!({
            "interfaceVersion":"weft-compile/0.4.0", "dialect":"weft-sql/0.4.0",
            "sql":"SELECT l.id FROM order_lines l",
            "modules":[{"documentJson":document,"pin":{
                "documentId":"urn:umf:domain:commerce","revision":"original",
                "umfVersion":"0.8.0","sha256":crate::json::sha256(document.as_bytes())
            },"selectedModuleIds":["domain"]}],
            "target":{"backendId":"transport-test","backendVersion":"1.0.0",
                "targetProfile":"test04","bindingJson":"{}","bindingSha256":crate::json::sha256(b"{}")}
        })
    }
    fn output(value: &Value, calls: &Cell<usize>) -> Value {
        let mut factory = |_: &Catalog, _: Plan04View<'_>, _: CompositionInput<'_>| {
            calls.set(calls.get() + 1);
            Ok(Registry::default())
        };
        serde_json::from_str(
            &Compiler::default().compile_json_with_factory(&value.to_string(), &mut factory),
        )
        .unwrap()
    }
    #[test]
    fn exact_envelope_and_resolution_precede_composition() {
        let calls = Cell::new(0);
        let mut cases = vec![];
        let mut value = request();
        value["dialect"] = json!("weft-sql/0.3.0");
        cases.push(value);
        let mut value = request();
        value["readProfile"] = json!("entity-page");
        cases.push(value);
        let mut value = request();
        value["unknownPathSetting"] = json!(true);
        cases.push(value);
        let mut value = request();
        value["target"]["bindingSha256"] = json!("0".repeat(64));
        cases.push(value);
        let mut value = request();
        value["sql"] = json!("SELECT");
        cases.push(value);
        let mut value = request();
        value["parameters"] = json!({"unused":{"family":"integer","value":"1"}});
        cases.push(value);
        let mut value = request();
        value["modules"][0]["pin"]["sha256"] = json!("0".repeat(64));
        cases.push(value);
        let mut value = request();
        value["target"]["bindingJson"] = json!(r#"{"a":1,"a":2}"#);
        value["target"]["bindingSha256"] =
            json!(crate::json::sha256(r#"{"a":1,"a":2}"#.as_bytes()));
        cases.push(value);
        let mut value = request();
        value["sql"] = json!("SELECT RELATED_PATHS(l.\"order_lines.product_id\", \"missing.relationship\", 20) AS paths FROM order_lines l");
        cases.push(value);
        for value in cases {
            let result = output(&value, &calls);
            assert_eq!(result["status"], "blocked");
            assert!(crate::backend03::validate_response_schema(&result));
        }
        assert_eq!(calls.get(), 0);
        let result = output(&request(), &calls);
        assert_eq!(calls.get(), 1);
        assert_eq!(result["diagnostics"][0]["code"], "WFT-BACKEND-MISSING");
        assert!(crate::backend03::validate_response_schema(&result));
    }
    #[test]
    fn old_transport_keeps_refusing_new_pair() {
        let result: Value = serde_json::from_str(
            &super::super::Compiler::default().compile_json(&request().to_string()),
        )
        .unwrap();
        assert_eq!(result["status"], "blocked");
        assert_eq!(result["diagnostics"][0]["code"], "WFT-VERSION");
    }
    #[test]
    fn transport_byte_budgets_precede_composition() {
        let calls = Cell::new(0);
        let mut value = request();
        let at_limit = format!("{}{{}}", " ".repeat(4 * 1024 * 1024 - 2));
        value["target"]["bindingJson"] = json!(at_limit);
        value["target"]["bindingSha256"] = json!(crate::json::sha256(at_limit.as_bytes()));
        let accepted = output(&value, &calls);
        assert_eq!(accepted["diagnostics"][0]["code"], "WFT-BACKEND-MISSING");
        assert_eq!(calls.get(), 1);
        let oversized = format!("{at_limit} ");
        value["target"]["bindingJson"] = json!(oversized);
        value["target"]["bindingSha256"] = json!(crate::json::sha256(oversized.as_bytes()));
        let refused = output(&value, &calls);
        assert_eq!(refused["diagnostics"][0]["code"], "WFT-LIMIT");
        assert_eq!(calls.get(), 1);
        assert!(crate::backend03::validate_response_schema(&refused));
        let mut value = request();
        value["sql"] = json!(format!(
            "SELECT \"{}\" FROM order_lines l",
            "é".repeat(32768)
        ));
        let refused = output(&value, &calls);
        assert_eq!(refused["diagnostics"][0]["code"], "WFT-LIMIT");
        assert_eq!(calls.get(), 1);
    }
    #[test]
    fn explicit_candidate_fixture_preserves_original_path_metadata() {
        use crate::backend03::tests::{manifest, path_emission, FixtureBackend};
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };
        let mut value = request();
        value["sql"] = json!("SELECT RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 20) AS paths FROM order_lines l");
        value["target"]["backendId"] = json!("fixture-only-03");
        value["target"]["backendVersion"] = json!("0.1.0");
        value["target"]["targetProfile"] = json!("fixture-only");
        value["options"] = json!({"allowCandidate":true});
        let composed = Cell::new(0);
        let assessed = Arc::new(AtomicUsize::new(0));
        let mut factory = |catalog: &Catalog, view: Plan04View<'_>, input: CompositionInput<'_>| {
            composed.set(composed.get() + 1);
            assert_eq!(catalog.pins()[0].revision, "original");
            assert_eq!(input.binding_json, "{}");
            assert_eq!(input.binding_sha256, crate::json::sha256(b"{}"));
            let (emission, edges) = path_emission(view);
            let mut registry = Registry::default();
            registry.register(FixtureBackend {
                manifest: manifest(view),
                emission,
                edges,
                calls: assessed.clone(),
            })?;
            Ok(registry)
        };
        let result: Value = serde_json::from_str(
            &Compiler::default().compile_json_with_factory(&value.to_string(), &mut factory),
        )
        .unwrap();
        assert!(crate::backend03::validate_response_schema(&result));
        assert_eq!(result["status"], "compiled");
        assert_eq!(result["qualification"]["status"], "candidate");
        assert_eq!(result["backend"]["interfaceVersion"], "weft-backend/0.3.0");
        assert_eq!(
            result["modelPins"],
            json!(value["modules"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| m["pin"].clone())
                .collect::<Vec<_>>())
        );
        assert_eq!(result["logicalPlan"]["irVersion"], "weft-ir/0.4.0");
        assert_eq!(composed.get(), 1);
        assert_eq!(assessed.load(Ordering::SeqCst), 1);
        value["options"]["allowCandidate"] = json!(false);
        let refused: Value = serde_json::from_str(
            &Compiler::default().compile_json_with_factory(&value.to_string(), &mut factory),
        )
        .unwrap();
        assert_eq!(refused["status"], "blocked");
        assert_eq!(refused["diagnostics"][0]["code"], "WFT-CAPABILITY");
        assert_eq!(assessed.load(Ordering::SeqCst), 1);
        assert_eq!(composed.get(), 2);
    }
    #[test]
    fn malformed_json_and_foreign_diagnostics_have_closed_refusals() {
        for raw in [
            "{",
            r#"{"interfaceVersion":"weft-compile/0.4.0","interfaceVersion":"weft-compile/0.4.0"}"#,
        ] {
            let result: Value =
                serde_json::from_str(&Compiler::default().compile_json(raw)).unwrap();
            assert_eq!(result["status"], "blocked");
            assert!(crate::backend03::validate_response_schema(&result));
        }
        let mut factory =
            |_: &Catalog, _: Plan04View<'_>, _: CompositionInput<'_>| -> Result<Registry> {
                Err(Diagnostic::new(
                    "WFT-TEST",
                    "foreign-phase",
                    "unreleased fixture diagnostic",
                ))
            };
        let result: Value = serde_json::from_str(
            &Compiler::default().compile_json_with_factory(&request().to_string(), &mut factory),
        )
        .unwrap();
        assert_eq!(result["diagnostics"][0]["code"], "WFT-BACKEND-FAILURE");
        assert!(crate::backend03::validate_response_schema(&result));
        assert!(!result.to_string().contains("unreleased fixture diagnostic"));
        let mut panicking = |_: &Catalog,
                             _: Plan04View<'_>,
                             _: CompositionInput<'_>|
         -> Result<Registry> { panic!("fixture panic payload") };
        let result: Value = serde_json::from_str(
            &Compiler::default().compile_json_with_factory(&request().to_string(), &mut panicking),
        )
        .unwrap();
        assert_eq!(result["diagnostics"][0]["code"], "WFT-BACKEND-FAILURE");
        assert!(crate::backend03::validate_response_schema(&result));
        assert!(!result.to_string().contains("fixture panic payload"));
    }
}
impl Compiler {
    /// Compile with this instance's explicitly registered Backend03 adapters.
    pub fn compile_json(&self, raw: &str) -> String {
        self.compile_json_internal(raw, None)
    }
    /// Compose registered adapters once, after the complete request and typed
    /// query have passed admission. Panic payloads are excluded from the response;
    /// the process panic hook and its logging policy remain owned by the host.
    pub fn compile_json_with_factory(
        &self,
        raw: &str,
        factory: &mut RegistryFactory<'_>,
    ) -> String {
        self.compile_json_internal(raw, Some(factory))
    }
    fn compile_json_internal(
        &self,
        raw: &str,
        factory: Option<&mut RegistryFactory<'_>>,
    ) -> String {
        let result = self.admit_and_emit(raw, factory);
        let mut output = match result {
            Ok(value) => value,
            Err(diagnostic) => json!({
                "interfaceVersion":"weft-compile/0.4.0", "status":"blocked",
                "diagnostics":[public_diagnostic(diagnostic)]
            }),
        };
        if !crate::backend03::validate_response_schema(&output) {
            output = json!({
                "interfaceVersion":"weft-compile/0.4.0", "status":"blocked",
                "diagnostics":[public_diagnostic(Diagnostic::new(
                    "WFT-BACKEND-FAILURE", "capability", "Invalid Backend03 diagnostic envelope",
                ))]
            });
        }
        serde_json::to_string(&output).expect("typed compile response")
    }
    fn admit_and_emit(
        &self,
        raw: &str,
        factory: Option<&mut RegistryFactory<'_>>,
    ) -> Result<Value> {
        if raw.len() > 16 * 1024 * 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "input",
                "Request exceeds sixteen MiB",
            ));
        }
        let input = crate::json::checked_json(raw)
            .map_err(|code| Diagnostic::new(code, "input", "Invalid request JSON"))?;
        if input["interfaceVersion"] != "weft-compile/0.4.0" || input["dialect"] != "weft-sql/0.4.0"
        {
            return Err(Diagnostic::new(
                "WFT-VERSION",
                "input",
                "This entrypoint requires the exact 0.4 compile and dialect pair",
            ));
        }
        if !Request04::is_valid(&input) {
            return Err(Diagnostic::new(
                "WFT-INPUT",
                "input",
                "Request is outside the closed 0.4 envelope",
            ));
        }
        let req: Request = serde_json::from_value(input)
            .map_err(|_| Diagnostic::new("WFT-INPUT", "input", "Malformed typed request"))?;
        if req.sql.len() > 65536 {
            return Err(Diagnostic::new("WFT-LIMIT", "input", "SQL exceeds 64 KiB"));
        }
        if req.target.binding_json.len() > 4 * 1024 * 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "input",
                "Binding exceeds four MiB",
            ));
        }
        if crate::json::sha256(req.target.binding_json.as_bytes()) != req.target.binding_sha256 {
            return Err(Diagnostic::new(
                "WFT-PIN",
                "input",
                "Binding byte digest mismatch",
            ));
        }
        crate::json::checked_json(&req.target.binding_json)
            .map_err(|code| Diagnostic::new(code, "input", "Invalid supplied binding JSON"))?;
        let catalog = Catalog::prepare(req.modules)?;
        let query = crate::path_query::parse(&req.sql)?;
        let plan = crate::path_application_resolve::resolve(
            &catalog,
            query,
            req.parameters.unwrap_or_default(),
        )?;
        let view = Plan04View::new(&plan);
        let composed = match factory {
            Some(factory) => Some(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    factory(
                        &catalog,
                        view,
                        CompositionInput {
                            backend_id: &req.target.backend_id,
                            backend_version: &req.target.backend_version,
                            target_profile: &req.target.target_profile,
                            binding_json: &req.target.binding_json,
                            binding_sha256: &req.target.binding_sha256,
                            allow_candidate: req.options.allow_candidate,
                        },
                    )
                }))
                .map_err(|_| {
                    Diagnostic::new(
                        "WFT-BACKEND-FAILURE",
                        "capability",
                        "Backend03 composition panicked",
                    )
                })??,
            ),
            None => None,
        };
        let registry = composed.as_ref().unwrap_or(&self.registry);
        let manifest = registry.manifest(&req.target.backend_id).ok_or_else(|| {
            Diagnostic::new(
                "WFT-BACKEND-MISSING",
                "capability",
                "Selected Backend03 is not registered",
            )
        })?;
        let binding = BindingInput {
            profile: manifest.binding_profile.clone(),
            json: req.target.binding_json,
            sha256: req.target.binding_sha256,
        };
        let target = Target {
            backend_id: req.target.backend_id,
            backend_version: req.target.backend_version,
            profile_id: req.target.target_profile,
            allow_candidate: req.options.allow_candidate,
        };
        let compiled = registry.compile(&catalog, view, &target, &binding)?;
        let candidate = compiled
            .qualifications
            .iter()
            .any(|q| q.assessment.status == Status::Candidate);
        let evidence: std::collections::BTreeSet<_> = compiled
            .qualifications
            .iter()
            .flat_map(|q| q.assessment.evidence.iter().cloned())
            .collect();
        let response = json!({
            "interfaceVersion":"weft-compile/0.4.0", "status":"compiled", "diagnostics":[],
            "dialect":"weft-sql/0.4.0", "compilerVersion":concat!("weft/",env!("CARGO_PKG_VERSION")),
            "modelPins":catalog.pins(), "logicalPlan":plan,
            "backend":{"backendId":compiled.backend_id,"backendVersion":compiled.backend_version,
                "targetProfile":compiled.target_profile.id,"interfaceVersion":"weft-backend/0.3.0"},
            "targetContext":compiled.target_profile,"bindingSha256":compiled.binding_sha256,
            "sql":compiled.emission.sql,"parameters":compiled.emission.parameters,
            "columns":compiled.emission.columns,"obligations":compiled.emission.obligations,
            "qualification":{"status":if candidate {"candidate"} else {"conformance-verified"},
                "evidence":evidence,"assumptions":[],"operations":compiled.qualifications}
        });
        if !crate::backend03::validate_response_schema(&response) {
            return Err(Diagnostic::new(
                "WFT-EMIT",
                "emit",
                "Backend03 result is outside the closed 0.4 response",
            ));
        }
        Ok(response)
    }
}
