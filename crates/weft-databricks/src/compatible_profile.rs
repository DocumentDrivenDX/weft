//! Host-admitted compatible physical revisions; no native qualification claimed.
use crate::{binding::Binding, candidate::{Candidate, validate_mapping}};
use serde_json::json;
use weft_core::{backend::*, error::Result};
pub const VERSION: &str = "0.1.1-compatible-revisions-candidate";
pub const PROFILE: &str = "dbsql-compatible-revisions-candidate";
pub const CAPABILITY: &str = "binding.compatibleSchemaRevisions";
pub struct CompatibleRevisions;
impl Backend for CompatibleRevisions {
 type Mapping = Binding;
 type TargetPlan = <Candidate as Backend>::TargetPlan;
 fn describe(&self) -> Result<Manifest> {
  let mut m=Candidate.describe()?;m.backend_version=VERSION.into();m.target_profiles[0].id=PROFILE.into();
  for cap in &mut m.capabilities {cap.target_profiles=vec![PROFILE.into()];}
  m.capabilities.push(Capability{id:CAPABILITY.into(),target_profiles:vec![PROFILE.into()],language_profiles:m.language_profiles.clone(),logical_domain:json!({"subset":"Object/Edge host-declared compatible physical revisions; original logical pins unchanged"}),result_domain:json!({"guard":"exact parameterized UTF8_BINARY membership; unselected revisions refuse before result publication"}),constraints:vec!["Host must establish accepted schema, field and key correspondence for every selected physical revision".into(),"Candidate only; no native qualification or production support".into()],obligations:vec![],status:Status::Candidate,evidence:vec![]});Ok(m)
 }
 fn validate_binding(&self,c:&Context<'_>)->Result<Validated<Binding>>{
  let mut v=validate_mapping(c,true)?;
  let selected:Vec<_>=v.mapping.records.iter().filter(|r|c.selection.records.contains(&r.logical)&&!r.compatible_schema_revisions.is_empty()).map(|r|json!({"logical":r.logical,"primarySchemaRevision":r.schema_revision,"compatibleSchemaRevisions":r.compatible_schema_revisions})).collect();
  if !selected.is_empty(){v.obligations.push(Obligation{id:"ashlar.compatibleSchemaRevisions".into(),parameters:json!({"records":selected,"modelPins":v.mapping.model_pins,"publication":v.mapping.publication,"requirements":["verify every selected physical revision is accepted in the same immutable publication","establish original logical field/key correspondence for every revision; the compiler does not prove compatibility","enforce every integrity check before buffered publication; never filter unselected revisions"]}),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into()});}Ok(v)
 }
 fn assess(&self,c:&Context<'_>,m:&Binding)->Result<Vec<Assessment>>{
  let mut a=Candidate.assess(c,m)?;
  if m.records.iter().any(|r|c.selection.records.contains(&r.logical)&&!r.compatible_schema_revisions.is_empty()) {a.push(Assessment{id:CAPABILITY.into(),status:Status::Candidate,evidence:vec![],obligations:vec![]});}Ok(a)
 }
 fn lower(&self,c:&Context<'_>,m:&Binding)->Result<Self::TargetPlan>{Candidate.lower(c,m)}
 fn emit(&self,c:&Context<'_>,p:&Self::TargetPlan)->Result<Emission>{Candidate.emit(c,p)}
}
