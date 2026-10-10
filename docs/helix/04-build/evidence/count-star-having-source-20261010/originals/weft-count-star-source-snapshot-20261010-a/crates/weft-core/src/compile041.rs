//! Explicit 0.4 transport. The existing Compiler and Backend02 factory stay intact.
use super::{public_diagnostic, CompositionInput, Request};
use crate::{
    backend03::{BindingInput, Plan04View, Registry, Status, Target},
    error::{Diagnostic, Result},
    model::Catalog,
};
use serde_json::{json, Value};

#[jsonschema::validator(
    path = "../../docs/helix/02-design/contracts/compile-request-v0.4.1.schema.json"
)]
struct Request041;

/// Hosts select registered code explicitly; SQL and models never provide code.
pub type RegistryFactory<'a> =
    dyn FnMut(&Catalog, Plan04View<'_>, CompositionInput<'_>) -> Result<Registry> + 'a;

/// Pure compiler for the exact compile/language 0.4.1 pair and Backend03.
///
/// The caller supplies registration and request configuration explicitly. A
/// compiled response describes obligations; it does not execute SQL, observe
/// native schemas, or discharge the host's publication and release checks.
#[derive(Default)]
pub struct Compiler {
    pub registry: Registry,
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
                "interfaceVersion":"weft-compile/0.4.1", "status":"blocked",
                "diagnostics":[public_diagnostic(diagnostic)]
            }),
        };
        if !crate::backend03::validate_response_schema(&output) {
            output = json!({
                "interfaceVersion":"weft-compile/0.4.1", "status":"blocked",
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
        if input["interfaceVersion"] != "weft-compile/0.4.1" || input["dialect"] != "weft-sql/0.4.1"
        {
            return Err(Diagnostic::new(
                "WFT-VERSION",
                "input",
                "This entrypoint requires the exact 0.4.1 compile and dialect pair",
            ));
        }
        if !Request041::is_valid(&input) {
            return Err(Diagnostic::new(
                "WFT-INPUT",
                "input",
                "Request is outside the closed 0.4.1 envelope",
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
        let query = crate::path_query::parse041(&req.sql)?;
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
            "interfaceVersion":"weft-compile/0.4.1", "status":"compiled", "diagnostics":[],
            "dialect":"weft-sql/0.4.1", "compilerVersion":concat!("weft/",env!("CARGO_PKG_VERSION")),
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
                "Backend03 result is outside the closed 0.4.1 response",
            ));
        }
        Ok(response)
    }
}
