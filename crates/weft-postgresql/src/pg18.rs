//! Production transport for host-selected original storage definitions on PG18.
//! Compilation establishes static admissions, never database or authorization facts.
use weft_core::{
    backend::{BindingInput, Registry},
    compile::CompositionInput,
    error::{Diagnostic, Result},
    model::Catalog,
};
pub const TARGET_PROFILE: &str = "truss-pg18.6-live/0.1";
pub const COMPOSITION_VERSION: &str = "weft-truss-pg18-composition/0.1.0";
fn registry(
    catalog: &Catalog,
    target: CompositionInput<'_>,
    composition: &str,
) -> Result<Registry> {
    if target.backend_id != "truss.postgresql.original"
        || target.backend_version != "0.2.0"
        || target.target_profile != TARGET_PROFILE
    {
        return Err(Diagnostic::new(
            "WFT-BACKEND-VERSION",
            "compose",
            "Explicit Truss PG18 original backend/version/target required",
        ));
    }
    let selected = crate::storage_composition::configuration(
        composition,
        catalog,
        COMPOSITION_VERSION,
        None,
        crate::storage_composition::scalar_expression,
        crate::storage_composition::scalar_expression,
    )?;
    let binding = BindingInput {
        profile: selected.binding_profile.clone(),
        json: target.binding_json.into(),
        sha256: target.binding_sha256.into(),
    };
    let mut registry = Registry::default();
    registry.register(selected.backend(catalog, &binding)?.with_pg18_target())?;
    Ok(registry)
}
/// The host supplies data-only storage declarations separately from SQL/models.
/// Only the statically selected Rust scalar procedure is executable. Candidate
/// opt-in and all emitted host obligations remain mandatory; there is no fallback.
pub fn compile_json(request: &str, composition: &str) -> String {
    let mut factory =
        |catalog: &Catalog, _: weft_core::backend::Plan<'_>, target: CompositionInput<'_>| {
            registry(catalog, target, composition)
        };
    weft_core::compile::Compiler::default().compile_json_with_factory(request, &mut factory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    #[test]
    fn explicit_pg18_transport_compiles_original_scalar_filter() {
        // Static owner fixture bytes exercise admission, not native runtime evidence.
        let inputs: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-boolean-inputs.json"
        ))
        .unwrap();
        let mut composition = inputs[1]["composition"].clone();
        composition["interfaceVersion"] = json!(COMPOSITION_VERSION);
        for property in composition["properties"].as_array_mut().unwrap() {
            property["nativeTree"] = json!(false);
        }
        let mut request = inputs[1]["requests"][1]["request"].clone();
        request["target"]["backendVersion"] = json!("0.2.0");
        request["target"]["targetProfile"] = json!(TARGET_PROFILE);
        let response: Value = serde_json::from_str(&compile_json(
            &request.to_string(),
            &composition.to_string(),
        ))
        .unwrap();
        assert_eq!(response["status"], "compiled", "{response}");
        assert!(response["sql"].is_string(), "{response}");
        request["target"]["targetProfile"] = json!("pg17.9-candidate");
        let refused: Value = serde_json::from_str(&compile_json(
            &request.to_string(),
            &composition.to_string(),
        ))
        .unwrap();
        assert_eq!(refused["status"], "blocked");
        assert!(refused.get("sql").is_none());
    }
}
