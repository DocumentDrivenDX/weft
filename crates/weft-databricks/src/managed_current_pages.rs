// SPDX-License-Identifier: MIT OR Apache-2.0
//! Managed current-core Backend02 typed page backend, used by EngineRead/AshlarTarget.
use crate::candidate::Candidate;
use weft_core::{backend::*,error::Result};
pub const VERSION:&str="0.1.0-managed-current-core-pages-candidate";
pub const PROFILE:&str="ashlar-managed-current-core/0.1";
pub struct ManagedCurrentPages;
fn native()->Obligation{Obligation{id:"ashlar.nativeProfile".into(),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into(),parameters:serde_json::json!({
 "targetProfile":PROFILE,
 "requirements":["verify selected Databricks serverless Spark Jobs runtime/build, authenticated principal, UC catalog/schema/table UUIDs and Delta protocol before SQL","bind exactly the four graph-role tables in the complete pinned publication vector and keep the six-role installation inventory separate","retain original source and publication authority through each SDK page and continuation","preserve the original typed request, selected key, continuation identity, UTF8 ordering and exact value representation","recheck the selected runtime, principal, UC identities, protocol, retention, source and publication before releasing buffered pages"],
 "qualification":"managed current-core typed page candidate; host fills values from the selected observed runtime/publication"
})}}
impl Backend for ManagedCurrentPages{
 type Mapping=<Candidate as Backend>::Mapping;type TargetPlan=<Candidate as Backend>::TargetPlan;
 fn describe(&self)->Result<Manifest>{let mut m=Candidate.describe()?;m.backend_id="ashlar.databricks.managed-uc-spark".into();m.backend_version=VERSION.into();m.binding_profile="ashlar-managed-current-core/0.1".into();m.target_profiles[0].id=PROFILE.into();m.target_profiles[0].engine="databricks-serverless-spark-jobs".into();m.target_profiles[0].engine_version="host-observed".into();m.target_profiles[0].session_settings=serde_json::json!({"comparison":"UTF8_BINARY","runtime":"host-observed"});m.target_profiles[0].storage_layout_revision="ashlar-delta/0.3".into();m.target_profiles[0].publication_revision="ashlar-managed-current-core/0.1".into();for c in &mut m.capabilities{c.target_profiles=vec![PROFILE.into()];c.obligations.retain(|o|o.id!="ashlar.nativeProfile");c.obligations.push(native());}Ok(m)}
 fn validate_binding(&self,c:&Context<'_>)->Result<Validated<Self::Mapping>>{if c.target.id!=PROFILE{return Err(weft_core::error::Diagnostic::new("WFT-BINDING","lower","Exact managed current-core page target required"));}let mut validated=Candidate.validate_binding(c)?;crate::managed_current_core::refine_managed_publication(&mut validated.obligations);Ok(validated)}
 fn assess(&self,c:&Context<'_>,m:&Self::Mapping)->Result<Vec<Assessment>>{Candidate.assess(c,m)}
 fn lower(&self,c:&Context<'_>,m:&Self::Mapping)->Result<Self::TargetPlan>{Candidate.lower(c,m)}
 fn emit(&self,c:&Context<'_>,p:&Self::TargetPlan)->Result<Emission>{let mut e=Candidate.emit(c,p)?;e.obligations.retain(|o|o.id!="ashlar.nativeProfile");crate::managed_current_core::refine_managed_publication(&mut e.obligations);e.obligations.push(native());Ok(e)}
}
