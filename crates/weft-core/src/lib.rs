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
        if req.get("dialect").is_some_and(|v| v != "weft-sql/0.1.0") {
            return Err(Diagnostic::new(
                "WFT-VERSION",
                "input",
                "Frontend implements the explicit 0.1 dialect only",
            ));
        }
        let sql = req["sql"]
            .as_str()
            .ok_or_else(|| Diagnostic::new("WFT-INPUT", "input", "SQL must be a string"))?;
        let modules = serde_json::from_value(req["modules"].clone()).map_err(|_| {
            Diagnostic::new("WFT-INPUT", "input", "Malformed supplied module bundle")
        })?;
        let (catalog, plan) = prepare_and_resolve(sql, modules)?;
        Ok(
            serde_json::json!({"status":"resolved","logicalPlan":plan,"retainedModules":catalog.inputs,"diagnostics":[]}),
        )
    }
    let output = match run(request) {
        Ok(v) => v,
        Err(d) => serde_json::json!({"status":"blocked","diagnostics":[d]}),
    };
    serde_json::to_string(&output).expect("serializable frontend report")
}
