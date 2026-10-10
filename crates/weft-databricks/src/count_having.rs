//! Explicit Optional String COUNT DISTINCT / HAVING target; source domains and old profile are unchanged.
use crate::{arithmetic::Arithmetic,binding::Binding,candidate::TargetPlan};
use serde_json::json;
use weft_core::{backend::*,error::{Diagnostic,Result}};
pub const ID:&str="ashlar.databricks.count-having";
pub const PROFILE:&str="spark4-delta4-count-having-candidate";
pub const VERSION:&str="0.3.0-count-having-candidate";
pub struct CountHaving;
impl Backend for CountHaving {
    type Mapping=Binding;
    type TargetPlan=TargetPlan;
    fn describe(&self)->Result<Manifest>{
        let mut manifest=Arithmetic.describe()?;
        manifest.backend_id=ID.into();manifest.backend_version=VERSION.into();
        for target in &mut manifest.target_profiles {target.id=PROFILE.into();}
        for cap in &mut manifest.capabilities {
            cap.target_profiles=vec![PROFILE.into()];
            if cap.logical_domain.get("subset").and_then(|v|v.as_str())==Some("required scalar nonaggregate row queries; original Field facets preserved") {
                cap.logical_domain=json!({"subset":"scalar row queries and explicit required/native-null String distinct-count/HAVING composition; original Field facets and availability preserved"});
            }
        }
        for id in ["aggregate","group","aggregate.countDistinct","predicate.stringIn","aggregate.countDistinct.optional","aggregate.havingCountDistinctGreater"] {
            manifest.capabilities.push(Capability{id:id.into(),target_profiles:vec![PROFILE.into()],language_profiles:manifest.language_profiles.clone(),logical_domain:json!({"subset":"exact required/explicit-native-null String counts with required String groups and projected-count HAVING > nonnegative Integer literal; no optional group/IN or other aggregate widening"}),result_domain:json!({"count":"logical mathematicalInteger; native signed64 count capacity over complete pre-HAVING nonnull dedup bag; all-null/global-empty zero; exact nonnegative HAVING literal within separate signed64 threshold capacity","equality":"UTF8_BINARY; validated present-null count arguments ignored, missing refuses; duplicate required-String IN literal slots retained"}),constraints:vec!["All consumed source rows and pre-HAVING deduplicated count capacities are checked before user SQL".into()],obligations:vec![],status:Status::Candidate,evidence:vec![]});
        }
        Ok(manifest)
    }
    fn validate_binding(&self,context:&Context<'_>)->Result<Validated<Binding>>{
        if context.target.id!=PROFILE || context.binding_value.get("profile").and_then(|v|v.as_str())!=Some("ashlar-databricks-candidate/0.1.0") {
            return Err(Diagnostic::new("WFT-BINDING","lower","Explicit distinct-count target and unchanged candidate binding profile required"));
        }
        Arithmetic.validate_binding(context)
    }
    fn assess(&self,context:&Context<'_>,binding:&Binding)->Result<Vec<Assessment>>{Arithmetic.assess(context,binding)}
    fn lower(&self,context:&Context<'_>,binding:&Binding)->Result<TargetPlan>{Arithmetic.lower(context,binding)}
    fn emit(&self,context:&Context<'_>,plan:&TargetPlan)->Result<Emission>{Arithmetic.emit(context,plan)}
}
