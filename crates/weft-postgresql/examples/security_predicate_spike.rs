//! Separate diagnostic transport; ordinary security compilation remains refused.
use std::io::{Read, Write};
use weft_core::error::{Diagnostic, Result};
fn refused() -> Diagnostic {
    Diagnostic::new("WFT-SECURITY-PREDICATE-SPIKE", "capability", "Bounded closed diagnostic input required")
}
fn run() -> Result<()> {
    let mut raw = String::new();
    std::io::stdin().take(16_000_001).read_to_string(&mut raw).map_err(|_| refused())?;
    if raw.len() > 16_000_000 { return Err(refused()); }
    let value = weft_core::json::checked_json(&raw).map_err(|_| refused())?;
    let object = value.as_object().ok_or_else(refused)?;
    if object.keys().any(|k| !matches!(k.as_str(), "request" | "assessment" | "cut" | "evaluationRequest"))
        || !object.contains_key("request") || !object.contains_key("assessment")
        || object.contains_key("cut") != object.contains_key("evaluationRequest") { return Err(refused()); }
    let input = &value["request"];
    let catalog = weft_core::model::Catalog::prepare_security(
        serde_json::from_value(input["modules"].clone()).map_err(|_| refused())?)?;
    let source = weft_core::security_source::SecuritySourcePacket::read(
        input["policyJson"].as_str().ok_or_else(refused)?,
        input["ontologyJson"].as_str().ok_or_else(refused)?, &catalog)?;
    let plan = weft_core::security_ir::SecurityLogicalPlan::read(source, &catalog)?;
    let mut output = weft_postgresql::security_predicate_spike::render(&plan, &catalog, &value["assessment"])?;
    if object.contains_key("cut") {
        let results = weft_core::security_evaluation::simulate_json(&plan, &catalog,
            &value["cut"].to_string(), &value["evaluationRequest"].to_string())?;
        output["simulationDisclosures"] = serde_json::to_value(results.iter().map(|r| &r.disclosure).collect::<Vec<_>>()).map_err(|_| refused())?;
        output["simulationDecisions"] = serde_json::json!(results.iter().map(|r| format!("{:?}", r.decision).to_lowercase()).collect::<Vec<_>>());
    }
    let encoded = serde_json::to_vec(&output).map_err(|_| refused())?;
    if encoded.len() > 16_000_000 { return Err(refused()); }
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&encoded).and_then(|_| stdout.write_all(b"\n")).map_err(|_| refused())
}
fn main() {
    if let Err(error) = run() {
        if let Ok(encoded) = serde_json::to_string(&error) { eprintln!("{encoded}"); }
        std::process::exit(1);
    }
}
