//! Explicit ordered LEFT stages; original source domains and old profiles stay unchanged.
use crate::{arithmetic::Arithmetic,binding::Binding,candidate::TargetPlan};
use serde_json::json;
use weft_core::{backend::*,error::{Diagnostic,Result}};
pub const ID:&str="ashlar.databricks.left-join";
pub const PROFILE:&str="spark4-delta4-left-join-candidate";
pub const VERSION:&str="0.3.0-left-join-candidate";
pub struct LeftJoin;
impl Backend for LeftJoin {
    type Mapping=Binding;
    type TargetPlan=TargetPlan;
    fn describe(&self)->Result<Manifest>{
        let mut manifest=Arithmetic.describe()?;
        manifest.backend_id=ID.into();manifest.backend_version=VERSION.into();
        for target in &mut manifest.target_profiles {target.id=PROFILE.into();}
        for cap in &mut manifest.capabilities {
            cap.target_profiles=vec![PROFILE.into()];
            cap.constraints.push("This explicit profile admits direct scalar String row projection and String equality INNER/LEFT ON only; no aggregate, arithmetic, filter, group, limit, or unmatched-scan ordering composition".into());
            if cap.logical_domain.get("subset").and_then(|v|v.as_str())==Some("required scalar nonaggregate row queries; original Field facets preserved") {
                cap.logical_domain=json!({"subset":"direct scalar String row queries with ordered INNER/LEFT String-equality joins and scan-qualified unmatched presence; original Field domains/availability preserved"});
            }
        }
        for id in ["join.left","value.outerJoinPresence"] {
            manifest.capabilities.push(Capability{id:id.into(),target_profiles:vec![PROFILE.into()],language_profiles:manifest.language_profiles.clone(),logical_domain:json!({"subset":"direct scalar String outputs and scalar String equality ON; complete source guards; each unmatched right scan retained separately from original Field/native-null semantics"}),result_domain:json!({"bags":"all matching occurrence combinations or exactly one unmatched extension per left prefix","carrier":"non-null tagged JSON; unmatched absent, explicitly permitted matched native-null null, matched exact String value","match":"original pinned object-current BIGINT id nullness under complete native schema/non-null source guard"}),constraints:vec!["No aggregate/arithmetic/filter/group/limit or unmatched-order widening".into(),"Mandatory outerJoin.matchIntegrity binds every LEFT right scan to the full pinned source and real physical identity schema/non-null guard, including unselected scans".into()],obligations:vec![],status:Status::Candidate,evidence:vec![]});
        }
        Ok(manifest)
    }
    fn validate_binding(&self,c:&Context<'_>)->Result<Validated<Binding>>{
        if c.target.id!=PROFILE||c.binding_value.get("profile").and_then(|v|v.as_str())!=Some("ashlar-databricks-candidate/0.1.0"){return Err(Diagnostic::new("WFT-BINDING","lower","Explicit LEFT target and original binding profile required"));}
        Arithmetic.validate_binding(c)
    }
    fn assess(&self,c:&Context<'_>,b:&Binding)->Result<Vec<Assessment>>{Arithmetic.assess(c,b)}
    fn lower(&self,c:&Context<'_>,b:&Binding)->Result<TargetPlan>{Arithmetic.lower(c,b)}
    fn emit(&self,c:&Context<'_>,p:&TargetPlan)->Result<Emission>{Arithmetic.emit(c,p)}
}
