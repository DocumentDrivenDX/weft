//! Explicit direct required primitive Field profile. No parameter/Key semantics.
use crate::{candidate::Candidate,local_spark4::LocalSpark4};
use weft_core::{backend::*,error::{Diagnostic,Result}};
pub const VERSION:&str="0.1.0-local-spark4-fields-candidate";
pub const PROFILE:&str="spark4.0.1-delta4.0.0-fields-candidate";
pub struct LocalSpark4Fields;
impl Backend for LocalSpark4Fields {
 type Mapping=<Candidate as Backend>::Mapping;type TargetPlan=<Candidate as Backend>::TargetPlan;
 fn describe(&self)->Result<Manifest>{let mut m=LocalSpark4.describe()?;m.backend_version=VERSION.into();m.target_profiles[0].id=PROFILE.into();m.capabilities.retain(|c|["scan","project","order.asc","limit","type.string","type.boolean","type.integer","type.decimal"].contains(&c.id.as_str()));for c in &mut m.capabilities {c.target_profiles=vec![PROFILE.into()];for o in &mut c.obligations {if o.id=="ashlar.nativeProfile" {o.parameters["targetProfile"]=serde_json::json!(PROFILE);}}}Ok(m)}
 fn validate_binding(&self,c:&Context<'_>)->Result<Validated<Self::Mapping>>{
  let Plan::V02(p)=c.plan else{return Err(Diagnostic::new("WFT-FIELD-PROFILE","lower","Direct Field plan required"))};
  if p.navigation.is_some() || !p.joins.is_empty() || !p.filters.is_empty() || p.aggregate || p.order.len()!=1 || p.limit.is_none() || p.page_key.is_some() || p.order[0].logical_type.family!=weft_core::ir::Family::String || p.order[0].logical_type.nullable {
   return Err(Diagnostic::new("WFT-FIELD-PROFILE","lower","Direct Field plan required"));
  }
  let owner=c.catalog.record_by_identity(&p.source.record)?;
  for output in &p.outputs {
   let weft_core::application_ir::Expression::Field{scan,identity}= &output.expression else{return Err(Diagnostic::new("WFT-FIELD-PROFILE","lower","Direct Field output required"))};
   let (_,graph)=c.catalog.member_descriptor_by_identity(&owner,identity)?;
   let selected=graph.iter().find(|d|d.identity==*identity).ok_or_else(||Diagnostic::new("WFT-FIELD-PROFILE","lower","Original Field descriptor required"))?;
   if scan!=&p.source.occurrence || selected.availability.as_deref()!=Some("required") || !matches!(&selected.shape,weft_core::application_model::Shape::Scalar{logical_type} if !logical_type.nullable) {
    return Err(Diagnostic::new("WFT-FIELD-PROFILE","lower","Original required scalar Field only"));
   }
  }
  let mut v=LocalSpark4.validate_binding(c)?;
  for o in &mut v.obligations {if o.id=="ashlar.nativeProfile" {o.parameters["targetProfile"]=serde_json::json!(PROFILE);}}
  Ok(v)
 }
 fn assess(&self,c:&Context<'_>,m:&Self::Mapping)->Result<Vec<Assessment>>{Candidate.assess(c,m)}
 fn lower(&self,c:&Context<'_>,m:&Self::Mapping)->Result<Self::TargetPlan>{Candidate.lower(c,m)}
 fn emit(&self,c:&Context<'_>,p:&Self::TargetPlan)->Result<Emission>{Candidate.emit(c,p)}
}
