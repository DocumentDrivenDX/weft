//! Evidence-qualified compiler registration for explicit native fixture/layout domains.
//! Supported semantics remain conditional on host integrity/publication obligations.
use crate::{candidate::Candidate, native_profile::NativeReview};
use serde_json::json;
use weft_core::{
    backend::*,
    error::{Diagnostic, Result},
};
pub const VERSION: &str = "0.1.0-qualified";
pub const PROFILE: &str = "dbsql2026.39-qualified";
pub const EVIDENCE: &str = "weft-native-qualification/ashlar/0.1.0#sha256:b5a3675c3bf0b10301ce1411368b80491446d8bbfb128f089fd431ea1eff3d6b";
pub struct Qualified;
fn profile_obligation() -> Result<Obligation> {
    let manifest = NativeReview.describe()?;
    let mut obligation = manifest.capabilities[0]
        .obligations
        .iter()
        .find(|o| o.id == "ashlar.nativeProfile")
        .cloned()
        .ok_or_else(|| {
            Diagnostic::new(
                "WFT-BACKEND-VERSION",
                "capability",
                "Native profile obligation missing",
            )
        })?;
    obligation.parameters["targetProfile"] = json!(PROFILE);
    obligation.parameters["qualification"] = json!(
        "evidence-qualified compiler semantics; host execution and publication remain conditional"
    );
    Ok(obligation)
}
impl Backend for Qualified {
    type Mapping = <Candidate as Backend>::Mapping;
    type TargetPlan = <Candidate as Backend>::TargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let mut manifest = NativeReview.describe()?;
        manifest.backend_version = VERSION.into();
        manifest.target_profiles[0].id = PROFILE.into();
        manifest.target_profiles[0].publication_revision =
            "host-verified-native-qualified/0.1".into();
        manifest.evidence = vec![EVIDENCE.into()];
        let obligation = profile_obligation()?;
        for capability in &mut manifest.capabilities {
            capability.status = Status::Supported;
            capability.evidence = vec![EVIDENCE.into()];
            capability.target_profiles = vec![PROFILE.into()];
            capability.constraints = vec![
                "Only binding-admitted versioned logical/storage domains in the hashed qualification record".into(),
                "Host must verify native profile, integrity, authorization and one admitted read context before buffered publication; no engine fallback".into(),
                "Native-tested compiler semantics do not attest existing application data or installed storage runtime adoption".into()
            ];
            capability
                .obligations
                .retain(|o| o.id != "ashlar.nativeProfile");
            capability.obligations.push(obligation.clone());
            if capability.result_domain.get("qualification").is_some() {
                capability.result_domain["qualification"] =
                    json!("supported native compiler semantics under host obligations");
            }
        }
        Ok(manifest)
    }
    fn validate_binding(&self, context: &Context<'_>) -> Result<Validated<Self::Mapping>> {
        let mut validated = Candidate.validate_binding(context)?;
        validated.obligations.push(profile_obligation()?);
        Ok(validated)
    }
    fn assess(&self, context: &Context<'_>, mapping: &Self::Mapping) -> Result<Vec<Assessment>> {
        let manifest = self.describe()?;
        let mut assessments = Candidate.assess(context, mapping)?;
        for assessment in &mut assessments {
            let declaration = manifest
                .capabilities
                .iter()
                .find(|c| c.id == assessment.id)
                .ok_or_else(|| {
                    Diagnostic::new(
                        "WFT-CAPABILITY",
                        "capability",
                        "Operation has no qualified declaration",
                    )
                })?;
            // A refusal is never upgraded; the qualified declaration owns its domain.
            if assessment.status != Status::Unsupported {
                assessment.status = declaration.status.clone();
                assessment.evidence = declaration.evidence.clone();
            }
        }
        Ok(assessments)
    }
    fn lower(&self, context: &Context<'_>, mapping: &Self::Mapping) -> Result<Self::TargetPlan> {
        Candidate.lower(context, mapping)
    }
    fn emit(&self, context: &Context<'_>, plan: &Self::TargetPlan) -> Result<Emission> {
        Candidate.emit(context, plan)
    }
}
