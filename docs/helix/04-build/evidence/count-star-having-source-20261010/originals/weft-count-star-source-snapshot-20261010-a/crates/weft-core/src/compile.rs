//! Public, pure compile transport. Hosts own backend registration and execution.
use crate::{
    backend::{BindingInput, Plan, Registry, Representation, Status, Target},
    error::{Diagnostic, Result},
    model::ModuleInput,
};
use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    interface_version: String,
    dialect: String,
    sql: String,
    modules: Vec<ModuleInput>,
    target: TargetRequest,
    #[serde(default)]
    options: Options,
    parameters: Option<crate::application_resolve::Parameters>,
    read_profile: Option<crate::application_ir::ReadProfile>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TargetRequest {
    backend_id: String,
    backend_version: String,
    target_profile: String,
    binding_json: String,
    binding_sha256: String,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    #[serde(default)]
    allow_candidate: bool,
}
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request.schema.json")]
struct Request01;
#[jsonschema::validator(
    path = "../../docs/helix/02-design/contracts/compile-request-v0.2.schema.json"
)]
struct Request02;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request-v0.3.schema.json")]
struct Request03;
/// Validated request target supplied to trusted host composition code.
/// Binding bytes/digest, modules and SQL/plan have passed transport admission.
pub struct CompositionInput<'a> {
    pub backend_id: &'a str,
    pub backend_version: &'a str,
    pub target_profile: &'a str,
    pub binding_json: &'a str,
    pub binding_sha256: &'a str,
    pub allow_candidate: bool,
}
/// Called once per valid resolved request. Models cannot select executable code.
/// Hosts provide pure admission/registration; execution and IO stay outside.
pub type RegistryFactory<'a> =
    dyn FnMut(&crate::model::Catalog, Plan<'_>, CompositionInput<'_>) -> Result<Registry> + 'a;
#[derive(Default)]
pub struct Compiler {
    pub registry: Registry,
}
impl Compiler {
    pub fn compile_json(&self, raw: &str) -> String {
        self.compile_json_internal(raw, None)
    }
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
        let mut version = "weft-compile/0.1.0".to_string();
        let result = (|| -> Result<Value> {
            if raw.len() > 16 * 1024 * 1024 {
                return Err(Diagnostic::new(
                    "WFT-LIMIT",
                    "input",
                    "Request exceeds sixteen MiB",
                ));
            }
            let input = crate::json::checked_json(raw)
                .map_err(|code| Diagnostic::new(code, "input", "Invalid request JSON"))?;
            if !input["interfaceVersion"].is_string() || !input["dialect"].is_string() {
                return Err(Diagnostic::new(
                    "WFT-INPUT",
                    "input",
                    "Compile and dialect versions must be supplied strings",
                ));
            }
            if matches!(input["interfaceVersion"].as_str(), Some("weft-compile/0.2.0" | "weft-compile/0.3.0")) {
                version = input["interfaceVersion"].as_str().unwrap().into();
            }
            if !matches!(
                (
                    input["interfaceVersion"].as_str(),
                    input["dialect"].as_str()
                ),
                (Some("weft-compile/0.1.0"), Some("weft-sql/0.1.0"))
                    | (Some("weft-compile/0.2.0"), Some("weft-sql/0.2.0"))
                    | (Some("weft-compile/0.3.0"), Some("weft-sql/0.3.0"))
            ) {
                return Err(Diagnostic::new(
                    "WFT-VERSION",
                    "input",
                    "Unsupported or mismatched compile/dialect versions",
                ));
            }
            if !(if version == "weft-compile/0.1.0" {
                Request01::is_valid(&input)
            } else if version == "weft-compile/0.2.0" {
                Request02::is_valid(&input)
            } else {
                Request03::is_valid(&input)
            }) {
                return Err(Diagnostic::new(
                    "WFT-INPUT",
                    "input",
                    "Request violates its versioned envelope",
                ));
            }
            let req: Request = serde_json::from_value(input)
                .map_err(|_| Diagnostic::new("WFT-INPUT", "input", "Malformed request members"))?;
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
            if crate::json::sha256(req.target.binding_json.as_bytes()) != req.target.binding_sha256
            {
                return Err(Diagnostic::new(
                    "WFT-PIN",
                    "input",
                    "Binding byte digest mismatch",
                ));
            }
            crate::json::checked_json(&req.target.binding_json)
                .map_err(|code| Diagnostic::new(code, "input", "Invalid supplied binding JSON"))?;
            let catalog = crate::model::Catalog::prepare(req.modules)?;
            if req.interface_version == "weft-compile/0.1.0" {
                let query = crate::syntax::parse(&req.sql)?;
                let plan = crate::resolve::resolve(&catalog, query)?;
                self.emit(
                    &req.interface_version,
                    &req.dialect,
                    &catalog,
                    Plan::V01(&plan),
                    req.target,
                    req.options.allow_candidate,
                    factory,
                )
            } else if req.interface_version == "weft-compile/0.3.0" {
                let query = crate::arithmetic_query::parse(&req.sql)?;
                let plan = crate::arithmetic_application_resolve::resolve(
                    &catalog, query, req.parameters.unwrap_or_default(), None,
                )?;
                self.emit(
                    &req.interface_version, &req.dialect, &catalog, Plan::V03(&plan),
                    req.target, req.options.allow_candidate, factory,
                )
            } else {
                let query = crate::application_syntax::parse(&req.sql)?;
                let plan = crate::application_resolve::resolve(
                    &catalog,
                    query,
                    req.parameters.unwrap_or_default(),
                    req.read_profile,
                )?;
                self.emit(
                    &req.interface_version,
                    &req.dialect,
                    &catalog,
                    Plan::V02(&plan),
                    req.target,
                    req.options.allow_candidate,
                    factory,
                )
            }
        })();
        let output = match result {
            Ok(v) => v,
            Err(d) => {
                json!({"interfaceVersion":version,"status":"blocked","diagnostics":[public_diagnostic(d)]})
            }
        };
        // serde_json maps use Unicode scalar key order; arrays/value strings retain order/text.
        serde_json::to_string(&output).expect("serializable compiler response")
    }
    fn emit(
        &self,
        version: &str,
        dialect: &str,
        catalog: &crate::model::Catalog,
        plan: Plan<'_>,
        target: TargetRequest,
        allow_candidate: bool,
        factory: Option<&mut RegistryFactory<'_>>,
    ) -> Result<Value> {
        let composed = match factory {
            Some(factory) => Some(factory(
                catalog,
                plan,
                CompositionInput {
                    backend_id: &target.backend_id,
                    backend_version: &target.backend_version,
                    target_profile: &target.target_profile,
                    binding_json: &target.binding_json,
                    binding_sha256: &target.binding_sha256,
                    allow_candidate,
                },
            )?),
            None => None,
        };
        let registry = composed.as_ref().unwrap_or(&self.registry);
        let manifest = registry.manifest(&target.backend_id).ok_or_else(|| {
            Diagnostic::new(
                "WFT-BACKEND-MISSING",
                "capability",
                "Selected backend is not registered",
            )
        })?;
        let binding = BindingInput {
            profile: manifest.binding_profile.clone(),
            json: target.binding_json,
            sha256: target.binding_sha256,
        };
        let target = Target {
            backend_id: target.backend_id,
            backend_version: target.backend_version,
            profile_id: target.target_profile,
            allow_candidate,
        };
        let compiled = registry.compile(catalog, plan, &target, &binding)?;
        let candidate = compiled
            .qualifications
            .iter()
            .any(|q| q.assessment.status == Status::Candidate);
        let evidence: std::collections::BTreeSet<_> = compiled
            .qualifications
            .iter()
            .flat_map(|q| q.assessment.evidence.iter().cloned())
            .collect();
        let logical_plan = match plan {
            Plan::V01(p) => serde_json::to_value(p),
            Plan::V02(p) => serde_json::to_value(p),
            Plan::V03(p) => serde_json::to_value(p),
        }
        .expect("typed plan");
        let columns = if version == "weft-compile/0.1.0" {
            let crate::ir::Node::Project { outputs, .. } = &match plan {
                Plan::V01(p) => p,
                Plan::V02(_) | Plan::V03(_) => unreachable!(),
            }
            .root
            else {
                unreachable!()
            };
            compiled.emission.columns.iter().zip(outputs).map(|(c,o)|{
    let Representation::Scalar{logical_type,carrier,decoder}=&c.representation else{unreachable!()};
    json!({"position":c.position,"outputName":c.output_name,"logicalType":logical_type,"nullable":c.nullable,"carrier":carrier,"decoder":decoder,"sourceIdentities":c.source_identities,"aggregate":if matches!(o.expression,crate::ir::Expression::Sum{..}){Some("sum")}else{None}})
   }).collect::<Vec<_>>()
        } else {
            compiled
                .emission
                .columns
                .iter()
                .map(|c| serde_json::to_value(c).expect("typed column"))
                .collect()
        };
        Ok(
            json!({"interfaceVersion":version,"status":"compiled","diagnostics":[],"dialect":dialect,"compilerVersion":concat!("weft/",env!("CARGO_PKG_VERSION")),"modelPins":catalog.pins(),
   "backend":{"backendId":compiled.backend_id,"backendVersion":compiled.backend_version,"targetProfile":compiled.target_profile.id,"interfaceVersion":"weft-backend/0.2.0"},
   "targetContext":compiled.target_profile,"bindingSha256":compiled.binding_sha256,"logicalPlan":logical_plan,"sql":compiled.emission.sql,"parameters":compiled.emission.parameters,"columns":columns,"obligations":compiled.emission.obligations,
   "qualification":{"status":if candidate{"candidate"}else{"conformance-verified"},"evidence":evidence,"assumptions":[],"operations":compiled.qualifications}}),
        )
    }
}
#[path = "compile04.rs"]
pub mod v04;

fn public_diagnostic(d: Diagnostic) -> Value {
    let recovery = match d.code.as_str() {
        "WFT-BACKEND-MISSING" | "WFT-BACKEND-VERSION" | "WFT-CAPABILITY" => "change-profile",
        "WFT-BACKEND-FAILURE" | "WFT-EMIT" | "WFT-OBLIGATION" => "host-action",
        _ => "correct-input",
    };
    let mut value = serde_json::to_value(d).expect("typed diagnostic");
    value["recoverability"] = json!(recovery);
    value
}

#[path = "compile041.rs"]
pub mod v041;
