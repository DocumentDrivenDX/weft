//! Explicit COUNT DISTINCT / IN target; source domains and old profile are unchanged.
use crate::{arithmetic::Arithmetic,binding::Binding,candidate::TargetPlan};
use serde_json::json;
use weft_core::{backend::*,error::{Diagnostic,Result}};
pub const ID:&str="ashlar.databricks.count-distinct";
pub const PROFILE:&str="spark4-delta4-count-distinct-candidate";
pub const VERSION:&str="0.3.0-count-distinct-candidate";
pub struct CountDistinct;
impl Backend for CountDistinct {
    type Mapping=Binding;
    type TargetPlan=TargetPlan;
    fn describe(&self)->Result<Manifest>{
        let mut manifest=Arithmetic.describe()?;
        manifest.backend_id=ID.into();manifest.backend_version=VERSION.into();
        for target in &mut manifest.target_profiles {target.id=PROFILE.into();}
        for cap in &mut manifest.capabilities {
            cap.target_profiles=vec![PROFILE.into()];
            if cap.logical_domain.get("subset").and_then(|v|v.as_str())==Some("required scalar nonaggregate row queries; original Field facets preserved") {
                cap.logical_domain=json!({"subset":"required scalar row queries and exact required String distinct-count composition; original Field facets preserved"});
            }
        }
        for id in ["aggregate","group","aggregate.countDistinct","predicate.stringIn"] {
            manifest.capabilities.push(Capability{id:id.into(),target_profiles:vec![PROFILE.into()],language_profiles:manifest.language_profiles.clone(),logical_domain:json!({"subset":"exact required String DISTINCT counts/groups and finite original literal String IN; no nullable/numeric arguments or other aggregate widening"}),result_domain:json!({"count":"logical mathematicalInteger; native signed64 capacity guard over same full dedup bag; empty global zero","equality":"UTF8_BINARY; duplicate IN literal slots retained"}),constraints:vec!["All original source checks and deduplicated native capacity checks precede user SQL".into()],obligations:vec![],status:Status::Candidate,evidence:vec![]});
        }
        Ok(manifest)
    }
    fn validate_binding(&self,context:&Context<'_>)->Result<Validated<Binding>>{
        if context.target.id!=PROFILE || context.binding_value.get("profile").and_then(|v|v.as_str())!=Some("ashlar-databricks-candidate/0.1.0") {
            return Err(Diagnostic::new("WFT-BINDING","bind","Explicit distinct-count target and unchanged candidate binding profile required"));
        }
        Arithmetic.validate_binding(context)
    }
    fn assess(&self,context:&Context<'_>,binding:&Binding)->Result<Vec<Assessment>>{Arithmetic.assess(context,binding)}
    fn lower(&self,context:&Context<'_>,binding:&Binding)->Result<TargetPlan>{Arithmetic.lower(context,binding)}
    fn emit(&self,context:&Context<'_>,plan:&TargetPlan)->Result<Emission>{Arithmetic.emit(context,plan)}
}
