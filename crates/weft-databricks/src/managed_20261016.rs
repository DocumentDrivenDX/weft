// SPDX-License-Identifier: MIT OR Apache-2.0
//! Explicit observed managed build candidate; never replaces historical qualification.
use crate::candidate::Candidate;
use serde_json::json;
use weft_core::{backend::*,error::Result};
pub const VERSION:&str="0.1.0-managed-20261016-candidate";
pub const PROFILE:&str="dbsql2026.39-build-20261016-candidate";
pub struct Managed20261016;
fn obligation()->Obligation{Obligation{
 id:"ashlar.nativeProfile".into(),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into(),
 parameters:json!({"targetProfile":PROFILE,"engineVersion":"2026.39",
 "settings":{"ansi_mode":true,"comparison":"explicit UTF8_BINARY","arithmetic":"ANSI exact-or-error","resultTransport":"JSON_ARRAY exact STRING","warehouseBuild":{"u_build_hash":"55233303686df68c7c3acb934c0a573947b3fae8","r_build_hash":"ec864f9e1ceaa3eb7afec091ca45053cf08a0afa"}},
 "requirements":["verify the exact executing native engine and relevant settings before any integrity or user SQL","retain the same admitted catalog/publication context across checks and buffered result publication","refuse mismatched or unavailable profile evidence; never silently choose a newer engine or another profile"],
 "qualification":"explicit managed build candidate; only separately recorded native subsets, original publication and authority obligations apply"})}}
impl Backend for Managed20261016{
 type Mapping=<Candidate as Backend>::Mapping;type TargetPlan=<Candidate as Backend>::TargetPlan;
 fn describe(&self)->Result<Manifest>{let mut m=Candidate.describe()?;m.backend_version=VERSION.into();
 let p=&mut m.target_profiles[0];p.id=PROFILE.into();p.engine_version="2026.39".into();p.session_settings=obligation().parameters["settings"].clone();p.storage_layout_revision="ashlar-delta/0.3".into();p.publication_revision="host-verified-managed-build-candidate/0.1".into();
 for c in &mut m.capabilities{c.target_profiles=vec![PROFILE.into()];c.obligations.push(obligation());c.constraints.push("Exact observed managed build candidate; no inherited native qualification or production authority".into());}Ok(m)}
 fn validate_binding(&self,c:&Context<'_>)->Result<Validated<Self::Mapping>>{let mut v=Candidate.validate_binding(c)?;v.obligations.push(obligation());Ok(v)}
 fn assess(&self,c:&Context<'_>,m:&Self::Mapping)->Result<Vec<Assessment>>{Candidate.assess(c,m)}
 fn lower(&self,c:&Context<'_>,m:&Self::Mapping)->Result<Self::TargetPlan>{Candidate.lower(c,m)}
 fn emit(&self,c:&Context<'_>,p:&Self::TargetPlan)->Result<Emission>{Candidate.emit(c,p)}
}
