mod security_budget;
pub mod security_scan_obligations;
pub mod security_query_uses;
pub mod security_query_profile;
pub mod security_evaluation;
pub mod security_composition;
pub mod security_ir;
pub mod security_literals;
pub mod security_policy_types;
pub mod security_ontology;
pub mod security_candidate_ontology;
pub mod security_candidate_policy_types;
pub mod security_candidate_ir;
pub mod security_candidate_dependencies;
pub mod security_candidate_incidence;
pub mod security_association_ref;
pub mod security_source;
pub mod application_ir;
pub mod application_model;
pub mod application_resolve;
pub mod application_syntax;
pub mod backend;
mod backend_emission;
pub mod compile;
pub mod error;
mod exact;
pub mod ir;
pub mod json;
pub mod model;
mod resolve;
pub mod syntax;
use error::{Diagnostic, Result};
use ir::LogicalPlan;
use model::{Catalog, ModuleInput};

/// Frontend boundary only. No target selection, backend lowering or host IO.
pub fn prepare_and_resolve(sql: &str, modules: Vec<ModuleInput>) -> Result<(Catalog, LogicalPlan)> {
    let catalog = Catalog::prepare(modules)?;
    let query = syntax::parse(sql)?;
    let plan = resolve::resolve(&catalog, query)?;
    Ok((catalog, plan))
}
/// Test transport for the frontend. Not CONTRACT-003's public compile artifact.
pub fn frontend_json(request: &str) -> String {
    fn run(request: &str) -> Result<serde_json::Value> {
        if request.len() > 16 * 1024 * 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "input",
                "Request exceeds byte limit",
            ));
        }
        let req = json::checked_json(request)
            .map_err(|c| Diagnostic::new(c, "input", "Invalid JSON input"))?;
        let app = req["dialect"] == "weft-sql/0.2.0";
        if req
            .get("dialect")
            .is_some_and(|v| v != "weft-sql/0.1.0" && !app)
        {
            return Err(Diagnostic::new(
                "WFT-VERSION",
                "input",
                "Unsupported explicit dialect version",
            ));
        }
        let sql = req["sql"]
            .as_str()
            .ok_or_else(|| Diagnostic::new("WFT-INPUT", "input", "SQL must be a string"))?;
        let modules = serde_json::from_value(req["modules"].clone()).map_err(|_| {
            Diagnostic::new("WFT-INPUT", "input", "Malformed supplied module bundle")
        })?;
        if app {
            let parameters = serde_json::from_value(
                req.get("parameters")
                    .cloned()
                    .unwrap_or(serde_json::json!({})),
            )
            .map_err(|_| Diagnostic::new("WFT-INPUT", "input", "Malformed typed parameter map"))?;
            let profile = serde_json::from_value(
                req.get("readProfile")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            )
            .map_err(|_| Diagnostic::new("WFT-INPUT", "input", "Malformed application profile"))?;
            let (catalog, plan) =
                prepare_and_resolve_application(sql, modules, parameters, profile)?;
            return Ok(
                serde_json::json!({"status":"resolved","logicalPlan":plan,"retainedModules":catalog.inputs(),"diagnostics":[]}),
            );
        }
        let (catalog, plan) = prepare_and_resolve(sql, modules)?;
        Ok(
            serde_json::json!({"status":"resolved","logicalPlan":plan,"retainedModules":catalog.inputs(),"diagnostics":[]}),
        )
    }
    let output = match run(request) {
        Ok(v) => v,
        Err(d) => serde_json::json!({"status":"blocked","diagnostics":[d]}),
    };
    serde_json::to_string(&output).expect("serializable frontend report")
}

/// Versioned application frontend. Does not execute or select storage.
pub fn prepare_and_resolve_application(
    sql: &str,
    modules: Vec<ModuleInput>,
    parameters: application_resolve::Parameters,
    profile: Option<application_ir::ReadProfile>,
) -> Result<(Catalog, application_ir::Plan)> {
    let catalog = Catalog::prepare(modules)?;
    let query = application_syntax::parse(sql)?;
    let plan = application_resolve::resolve(&catalog, query, parameters, profile)?;
    Ok((catalog, plan))
}
