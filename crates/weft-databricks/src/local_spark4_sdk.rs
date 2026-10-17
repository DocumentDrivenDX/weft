//! Explicit SDK scalar presence transport. Legacy profiles keep their bytes.
use crate::{candidate::Candidate, local_spark4::LocalSpark4};
use weft_core::{backend::*, error::{Diagnostic, Result}};
pub const VERSION: &str = "0.1.0-local-spark4-sdk-candidate";
pub const PROFILE: &str = "spark4.0.1-delta4.0.0-sdk-candidate";
pub struct LocalSpark4Sdk;
impl Backend for LocalSpark4Sdk {
 type Mapping=<Candidate as Backend>::Mapping;
 type TargetPlan=<Candidate as Backend>::TargetPlan;
 fn describe(&self)->Result<Manifest>{
  let mut m=LocalSpark4.describe()?;m.backend_version=VERSION.into();m.interface_version="weft-backend/0.2.8".into();m.language_profiles.push(LanguageProfile{dialect_profile:"weft-sql/0.2.0".into(),ir_version:"weft-ir/engine-navigation/0.2".into()});m.language_profiles.push(LanguageProfile{dialect_profile:"weft-sql/0.2.0".into(),ir_version:"weft-ir/engine-ordered-read/0.1".into()});m.target_profiles[0].id=PROFILE.into();
  for c in &mut m.capabilities {c.language_profiles=m.language_profiles.clone();c.target_profiles=vec![PROFILE.into()];for o in &mut c.obligations {if o.id=="ashlar.nativeProfile"{o.parameters["targetProfile"]=PROFILE.into();}}
   if c.id=="value.presence" {c.logical_domain=serde_json::json!({"subset":"original scalar Field presence; missing and explicit null remain distinct; exact text or Boolean values"});}
  }
  let mut native=m.capabilities.iter().find(|c|c.id=="value.presence").unwrap().clone();native.id="value.nativeNull".into();m.capabilities.push(native);let mut descending=m.capabilities.iter().find(|c|c.id=="order.asc").unwrap().clone();descending.id="order.desc".into();m.capabilities.push(descending);Ok(m)
 }
 fn validate_binding(&self,c:&Context<'_>)->Result<Validated<Self::Mapping>>{
  if !c.catalog.sdk_scalar_presence(){return Err(Diagnostic::new("WFT-SDK-PRESENCE","lower","Trusted original SDK presence preparation required"));}
  let Plan::V02(p)=c.plan else{return Err(Diagnostic::new("WFT-SDK-PRESENCE","lower","Original application scalar Field plan required"))};
  for o in &p.outputs {match &o.expression {
   weft_core::application_ir::Expression::Field{identity,..}=>{
    if !p.type_graph.iter().any(|d|d.identity==*identity && matches!(&d.shape,weft_core::application_model::Shape::Scalar{..})) {return Err(Diagnostic::new("WFT-SDK-PRESENCE","lower","Original scalar Field descriptor required"));}
   },
   weft_core::application_ir::Expression::RelatedKeys{..}=>{},
   _=>return Err(Diagnostic::new("WFT-SDK-PRESENCE","lower","Computed outputs require separately admitted expression presence semantics")),
  }}
  let mut v=LocalSpark4.validate_binding(c)?;v.additional_capabilities.push("value.nativeNull".into());for o in &mut v.obligations {if o.id=="ashlar.nativeProfile"{o.parameters["targetProfile"]=PROFILE.into();}}Ok(v)
 }
 fn assess(&self,c:&Context<'_>,m:&Self::Mapping)->Result<Vec<Assessment>>{let mut a=Candidate.assess(c,m)?;a.push(Assessment{id:"value.nativeNull".into(),status:Status::Candidate,evidence:vec![],obligations:vec![]});Ok(a)}
 fn lower(&self,c:&Context<'_>,m:&Self::Mapping)->Result<Self::TargetPlan>{Candidate.lower(c,m)}
 fn emit(&self,c:&Context<'_>,p:&Self::TargetPlan)->Result<Emission>{Candidate.emit(c,p)}
}
