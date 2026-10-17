//! Additive original ordered/presence page profile. No legacy enrollment.
use crate::{candidate::Candidate,local_spark4_sdk::LocalSpark4Sdk};
use weft_core::{backend::*,error::{Diagnostic,Result}};
pub const VERSION:&str="0.1.0-local-spark4-sdk-pages-candidate";
pub struct LocalSpark4SdkPages;
impl Backend for LocalSpark4SdkPages {
 type Mapping=<Candidate as Backend>::Mapping;
 type TargetPlan=<Candidate as Backend>::TargetPlan;
 fn describe(&self)->Result<Manifest>{
  let mut m=LocalSpark4Sdk.describe()?;m.backend_version=VERSION.into();m.interface_version="weft-backend/0.2.8".into();
  m.language_profiles.push(LanguageProfile{dialect_profile:"weft-sql/0.2.0".into(),ir_version:"weft-ir/engine-ordered-page/0.1".into()});
  for c in &mut m.capabilities{c.language_profiles=m.language_profiles.clone();}
  for name in ["page.ordered","cursor.stateTuple","order.desc"] {if m.capabilities.iter().any(|c|c.id==name){continue;}let mut c=m.capabilities.iter().find(|c|c.id=="order.asc").unwrap().clone();c.id=name.into();c.logical_domain=serde_json::json!({"subset":"original complete selected Key ASC tie-break with exact full state/value cursor; explicit directions/null placement and binary String order"});m.capabilities.push(c);}
  Ok(m)
 }
 fn validate_binding(&self,c:&Context<'_>)->Result<Validated<Self::Mapping>>{
  match c.plan {
   Plan::Ordered(page)=>{
    let base=Context{catalog:c.catalog,plan:Plan::V02(page.base()),target:c.target,binding:c.binding,binding_value:c.binding_value,selection:c.selection};
    LocalSpark4Sdk.validate_binding(&base)
   },
   Plan::V02(_)=>LocalSpark4Sdk.validate_binding(c),
   _=>Err(Diagnostic::new("WFT-ORDERED-PAGE","lower","Original SDK application or ordered plan required")),
  }
 }
 fn assess(&self,c:&Context<'_>,m:&Self::Mapping)->Result<Vec<Assessment>>{LocalSpark4Sdk.assess(c,m)}
 fn lower(&self,c:&Context<'_>,m:&Self::Mapping)->Result<Self::TargetPlan>{match c.plan{Plan::Ordered(page)=>crate::candidate::lower_sdk_ordered(c,m,page),Plan::V02(_)=>Candidate.lower(c,m),_=>Err(Diagnostic::new("WFT-ORDERED-PAGE","lower","Original ordered plan required"))}}
 fn emit(&self,c:&Context<'_>,p:&Self::TargetPlan)->Result<Emission>{Candidate.emit(c,p)}
}
