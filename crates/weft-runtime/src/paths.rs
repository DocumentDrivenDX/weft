//! Explicit 0.4 composition; hosts still own execution and publication custody.
use weft_core::{
    backend03::{Plan04View, Registry},
    compile::{v04::Compiler, CompositionInput},
    error::Result,
    model::Catalog,
};

/// Compile only through the selected Backend03 path adapter. The existing runtime
/// entrypoint and registrations remain governed by their original interfaces.
/// This candidate composition supplies no native conformance or engine execution.
pub fn compile_json(request: &str) -> String {
    let mut factory =
        |_: &Catalog, _: Plan04View<'_>, _: CompositionInput<'_>| -> Result<Registry> {
            let mut registry = Registry::default();
            registry.register(weft_databricks::paths::Paths)?;
            Ok(registry)
        };
    Compiler::default().compile_json_with_factory(request, &mut factory)
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    #[test]
    fn explicit_adapter_compiles_original_fixture_and_requires_candidate_opt_in() {
        let request = include_str!(
            "../../../tests/ashlar-databricks/fixtures/original-commerce-path-request.json"
        );
        let output: Value = serde_json::from_str(&super::compile_json(request)).unwrap();
        assert_eq!(output["status"], "compiled", "{output}");
        assert_eq!(output["interfaceVersion"], "weft-compile/0.4.0");
        assert_eq!(output["backend"]["backendId"], "ashlar.databricks.paths");
        assert_eq!(output["backend"]["interfaceVersion"], "weft-backend/0.3.0");
        assert_eq!(output["qualification"]["status"], "candidate");
        let mut refused: Value = serde_json::from_str(request).unwrap();
        refused["options"]["allowCandidate"] = json!(false);
        let output: Value =
            serde_json::from_str(&super::compile_json(&refused.to_string())).unwrap();
        assert_eq!(output["status"], "blocked");
        assert_eq!(output["diagnostics"][0]["code"], "WFT-CAPABILITY");
    }

    #[test]
    fn explicit_namespace_preserves_both_version_fences() {
        let new_pair =
            json!({"interfaceVersion":"weft-compile/0.4.0","dialect":"weft-sql/0.4.0"}).to_string();
        let old_result: Value = serde_json::from_str(&crate::compile_json(&new_pair)).unwrap();
        assert_eq!(old_result["status"], "blocked");
        assert_eq!(old_result["diagnostics"][0]["code"], "WFT-VERSION");
        let old_pair =
            json!({"interfaceVersion":"weft-compile/0.3.0","dialect":"weft-sql/0.3.0"}).to_string();
        let new_result: Value = serde_json::from_str(&super::compile_json(&old_pair)).unwrap();
        assert_eq!(new_result["interfaceVersion"], "weft-compile/0.4.0");
        assert_eq!(new_result["status"], "blocked");
        assert_eq!(new_result["diagnostics"][0]["code"], "WFT-VERSION");
        let malformed: Value = serde_json::from_str(&super::compile_json("{")).unwrap();
        assert_eq!(malformed["status"], "blocked");
        assert_eq!(malformed["interfaceVersion"], "weft-compile/0.4.0");
    }
}
