// SPDX-License-Identifier: MIT OR Apache-2.0
//! Explicit independent Spark4/Delta4 local development registration.
//! This registration remains candidate until its versioned support audit passes.
//! Native correctness never establishes production authorization or stored-data custody.
use crate::candidate::Candidate;
use serde_json::json;
use weft_core::{backend::*, error::Result};

pub const VERSION: &str = "0.1.0-local-spark4-candidate";
pub const PROFILE: &str = "spark4.0.1-delta4.0.0-candidate";
pub struct LocalSpark4;
fn execution_profile() -> Obligation {
    Obligation {
        id: "ashlar.nativeProfile".into(),
        parameters: json!({
            "targetProfile":PROFILE,
            "engineVersion":"4.0.1",
            "deltaVersion":"4.0.0",
            "settings":{"spark.sql.ansi.enabled":"true","spark.sql.session.timeZone":"UTC","comparison":"explicit UTF8_BINARY","resultTransport":"exact native STRING"},
            "requirements":[
                "verify the exact executing native engine and relevant settings before any integrity or user SQL",
                "retain the same admitted catalog/publication context across checks and buffered result publication",
                "refuse mismatched or unavailable profile evidence; never silently choose a newer engine or another profile"
            ],
            "qualification":"local development candidate; native publication, authorization and complete pin custody remain mandatory"
        }),
        owner: ObligationOwner::Host,
        failure_code: "WFT-OBLIGATION".into(),
    }
}
impl Backend for LocalSpark4 {
    type Mapping = <Candidate as Backend>::Mapping;
    type TargetPlan = <Candidate as Backend>::TargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let mut manifest = Candidate.describe()?;
        manifest.backend_version = VERSION.into();
        let profile = &mut manifest.target_profiles[0];
        profile.id = PROFILE.into();
        profile.engine_version = "4.0.1".into();
        profile.session_settings = json!({"spark.sql.ansi.enabled":"true","spark.sql.session.timeZone":"UTC","comparison":"explicit UTF8_BINARY","resultTransport":"exact native STRING","deltaVersion":"4.0.0"});
        profile.storage_layout_revision = "ashlar-delta/0.3".into();
        profile.publication_revision = "host-verified-local-spark4/0.1".into();
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

