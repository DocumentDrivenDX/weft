//! Explicit engine-pinned registration for native qualification review.
//! This registration remains candidate until its versioned support audit passes.
//! Native correctness never establishes production authorization or stored-data custody.
use crate::candidate::Candidate;
use serde_json::json;
use weft_core::{backend::*, error::Result};

pub const VERSION: &str = "0.1.0-native-review";
pub const PROFILE: &str = "pg17.9-native-review";
pub struct NativeReview;
fn execution_profile() -> Obligation {
    Obligation {
        id: "truss.nativeProfile".into(),
        parameters: json!({
            "targetProfile":PROFILE,
            "engineVersion":"17.9",
            "settings":{"server_encoding":"UTF8","client_encoding":"UTF8","standard_conforming_strings":"on","transaction_isolation":"repeatable read","lc_collate":"C","lc_ctype":"C","comparison":"explicit C","arithmetic":"exact-or-error"},
            "requirements":[
                "verify the exact executing native engine and relevant settings before any integrity or user SQL",
                "retain the same admitted catalog/publication context across checks and buffered result publication",
                "refuse mismatched or unavailable profile evidence; never silently choose a newer engine or another profile"
            ],
            "qualification":"native-review; no supported declaration or production adoption"
        }),
        owner: ObligationOwner::Host,
        failure_code: "WFT-OBLIGATION".into(),
    }
}
impl Backend for NativeReview {
    type Mapping = <Candidate as Backend>::Mapping;
    type TargetPlan = <Candidate as Backend>::TargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let mut manifest = Candidate.describe()?;
        manifest.backend_version = VERSION.into();
        let profile = &mut manifest.target_profiles[0];
        profile.id = PROFILE.into();
        profile.engine_version = "17.9".into();
        profile.session_settings = json!({"server_encoding":"UTF8","client_encoding":"UTF8","standard_conforming_strings":"on","transaction_isolation":"repeatable read","lc_collate":"C","lc_ctype":"C","comparison":"explicit C","arithmetic":"exact-or-error"});
        profile.storage_layout_revision = "weft-truss-fixtures/0.1".into();
        profile.publication_revision = "host-verified-native-review/0.1".into();
        for capability in &mut manifest.capabilities {
            capability.target_profiles = vec![PROFILE.into()];
            capability.obligations.push(execution_profile());
            capability
                .constraints
                .push("Exact engine/settings review scope; candidate opt-in still required".into());
        }
        Ok(manifest)
    }
    fn validate_binding(&self, context: &Context<'_>) -> Result<Validated<Self::Mapping>> {
        let mut validated = Candidate.validate_binding(context)?;
        validated.obligations.push(execution_profile());
        Ok(validated)
    }
    fn assess(&self, context: &Context<'_>, mapping: &Self::Mapping) -> Result<Vec<Assessment>> {
        Candidate.assess(context, mapping)
    }
    fn lower(&self, context: &Context<'_>, mapping: &Self::Mapping) -> Result<Self::TargetPlan> {
        Candidate.lower(context, mapping)
    }
    fn emit(&self, context: &Context<'_>, plan: &Self::TargetPlan) -> Result<Emission> {
        Candidate.emit(context, plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use weft_core::compile::Compiler;
    // @covers US-002-AC2 @covers US-006-AC1 @covers US-006-AC3
    #[test]
    fn engine_pinned_registration_retains_candidate_gate_and_exact_emission() {
        let manifest = NativeReview.describe().unwrap();
        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.backend_version, VERSION);
        assert_eq!(manifest.target_profiles[0].id, PROFILE);
        assert_eq!(manifest.target_profiles[0].engine_version, "17.9");
        assert!(manifest.evidence.is_empty());
        assert!(manifest
            .capabilities
            .iter()
            .all(|cap| cap.status == Status::Candidate
                && cap.evidence.is_empty()
                && cap.target_profiles == vec![PROFILE.to_string()]));
        let mut registry = Registry::default();
        registry.register(NativeReview).unwrap();
        let compiler = Compiler { registry };
        let mut old_registry = Registry::default();
        old_registry.register(Candidate).unwrap();
        let old = Compiler {
            registry: old_registry,
        };
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/compiler-cases.json"
        ))
        .unwrap();
        let cases: Vec<Value> = fixture.as_array().unwrap().clone();
        for case in cases {
            let mut request = case["request"].clone();
            let before: Value =
                serde_json::from_str(&old.compile_json(&request.to_string())).unwrap();
            assert_eq!(before["status"], "compiled");
            request["target"]["backendVersion"] = json!(VERSION);
            request["target"]["targetProfile"] = json!(PROFILE);
            let after: Value =
                serde_json::from_str(&compiler.compile_json(&request.to_string())).unwrap();
            assert_eq!(after["status"], "compiled", "{after}");
            for member in [
                "sql",
                "parameters",
                "columns",
                "logicalPlan",
                "modelPins",
                "bindingSha256",
            ] {
                assert_eq!(before[member], after[member], "Changed {member}");
            }
            let obligations = after["obligations"].as_array().unwrap();
            assert!(obligations.iter().any(|o| o["id"] == "truss.nativeProfile"
                && o["owner"] == "host"
                && o["parameters"]["engineVersion"] == "17.9"
                && o["parameters"]["targetProfile"] == PROFILE));
            for obligation in before["obligations"].as_array().unwrap() {
                assert!(obligations.contains(obligation));
            }
            assert_eq!(after["qualification"]["status"], "candidate");
            for (member, value, code) in [
                ("targetProfile", "unregistered", "WFT-BACKEND-VERSION"),
                ("backendVersion", "stale", "WFT-BACKEND-VERSION"),
            ] {
                let mut stale = request.clone();
                stale["target"][member] = json!(value);
                let refused: Value =
                    serde_json::from_str(&compiler.compile_json(&stale.to_string())).unwrap();
                assert_eq!(refused["status"], "blocked");
                assert!(refused.get("sql").is_none());
                assert_eq!(refused["diagnostics"][0]["code"], code);
            }
            request["options"]["allowCandidate"] = json!(false);
            let refused: Value =
                serde_json::from_str(&compiler.compile_json(&request.to_string())).unwrap();
            assert_eq!(refused["diagnostics"][0]["code"], "WFT-CAPABILITY");
            assert!(refused.get("sql").is_none());
        }
    }
}
