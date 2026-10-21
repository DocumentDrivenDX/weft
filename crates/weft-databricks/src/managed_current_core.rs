// SPDX-License-Identifier: MIT OR Apache-2.0
//! Candidate Backend03 target for managed Unity Catalog Spark Jobs.
//! Native runtime, identity, publication and source authority remain host obligations.
use crate::{binding::Binding, paths::PathTargetPlan, paths_keys_having_spark42::CountStarHavingSpark42};
use weft_core::{backend03::*, backend::{Obligation,ObligationOwner}, error::{Diagnostic, Result}};

pub const ID: &str = "ashlar.databricks.managed-uc-spark";
pub const VERSION: &str = "0.1.0-managed-current-core-candidate";
pub const PROFILE: &str = "ashlar-managed-current-core/0.1";
pub struct ManagedCurrentCore;
pub(crate) fn refine_managed_publication(obligations:&mut [Obligation]) {
 for obligation in obligations.iter_mut().filter(|item|item.id=="ashlar.candidate.publication") {
  obligation.parameters["nativeProfile"]=serde_json::json!({
   "targetProfile":PROFILE,
   "engine":"databricks-serverless-spark-jobs",
   "runtime":"host-observed",
   "comparison":"UTF8_BINARY",
   "arithmetic":"ANSI exact-or-error",
   "requirements":[
    "require host-observed selected serverless Spark Jobs runtime/build, authenticated principal, UC namespace, table UUIDs and Delta protocol",
    "resolve and verify the complete pinned manifest vector for exactly object_current, edge_current, tombstone and whole_source_history",
    "keep the six-role installation inventory separate from the four-role manifest data vector",
    "bind publication opening and closing checks to the same observed UC tables, versions, schema and retained data files"
   ]
  });
 }
}
fn obligation() -> Obligation { Obligation { id:"ashlar.nativeProfile".into(), owner:ObligationOwner::Host, failure_code:"WFT-OBLIGATION".into(), parameters:serde_json::json!({
    "targetProfile":PROFILE,
    "requirements":["verify the selected Databricks serverless Spark Jobs runtime/build, authenticated principal, Unity Catalog catalog/schema/table UUIDs and Delta protocol before SQL","bind exactly the four graph-role tables and complete pinned publication vector; retain six-role installation inventory separately","retain original source and publication authority over opening, execution and closing checks","preserve exact UTF8_BINARY comparison, typed results and original compiler SQL/parameters","recheck runtime, principal, table identity, protocol, retention, source and publication custody before releasing buffered results"],
    "qualification":"managed current-core candidate; host must fill all values from the selected observed runtime and publication"
}), } }
impl Backend for ManagedCurrentCore {
 type Mapping=Binding; type TargetPlan=PathTargetPlan;
 fn describe(&self)->Result<Manifest>{
  let mut m=CountStarHavingSpark42.describe()?;m.backend_id=ID.into();m.backend_version=VERSION.into();m.binding_profile=PROFILE.into();
  let p=&mut m.target_profiles[0];p.id=PROFILE.into();p.engine="databricks-serverless-spark-jobs".into();p.engine_version="host-observed".into();p.session_settings=serde_json::json!({"comparison":"UTF8_BINARY","runtime":"host-observed"});p.storage_layout_revision="ashlar-delta/0.3".into();p.publication_revision="ashlar-managed-current-core/0.1".into();
  for c in &mut m.capabilities {c.target_profiles=vec![PROFILE.into()];c.obligations.retain(|o|o.id!="ashlar.nativeProfile");c.obligations.push(obligation());c.constraints.push("Managed native runtime and publication custody are host obligations; this target does not qualify a runtime".into());}
  Ok(m)
 }
 fn validate_binding(&self,c:&Context<'_>)->Result<Validated<Binding>> { if c.plan.ir_version()!="weft-ir/0.4.1" {return Err(Diagnostic::new("WFT-CAPABILITY","capability","Current-core grouped Backend03 requires weft-ir/0.4.1"));} let mut validated=crate::paths_keys::PathsKeys.validate_binding_managed_current(c,PROFILE)?;refine_managed_publication(&mut validated.obligations);Ok(validated) }
 fn assess(&self,c:&Context<'_>,b:&Binding)->Result<Vec<Assessment>>{CountStarHavingSpark42.assess(c,b)}
 fn lower(&self,c:&Context<'_>,b:&Binding)->Result<PathTargetPlan>{CountStarHavingSpark42.lower(c,b)}
 fn emit(&self,c:&Context<'_>,p:&PathTargetPlan)->Result<Emission>{let mut e=CountStarHavingSpark42.emit(c,p)?;e.obligations.retain(|o|o.id!="ashlar.nativeProfile");refine_managed_publication(&mut e.obligations);e.obligations.push(obligation());Ok(e)}
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn managed_profile_is_distinct_and_leaves_native_facts_to_the_host() {
  let manifest=ManagedCurrentCore.describe().unwrap();
  assert_eq!(manifest.backend_id,ID);
  assert_eq!(manifest.backend_version,VERSION);
  assert_eq!(manifest.binding_profile,"ashlar-managed-current-core/0.1");
  assert_eq!(manifest.target_profiles.len(),1);
  let profile=&manifest.target_profiles[0];
  assert_eq!(profile.id,PROFILE);
  assert_eq!(profile.storage_layout_revision,"ashlar-delta/0.3");
  assert_eq!(profile.engine_version,"host-observed");
  assert!(profile.session_settings.get("deltaJars").is_none());
  assert!(profile.session_settings.get("spark.sql.legacy.parameterSubstitution.constantsOnly").is_none());
  let cap=manifest.capabilities.iter().find(|c|c.id=="aggregate.havingCountStarGreater").unwrap();
  assert!(cap.obligations.iter().any(|o|o.id=="ashlar.nativeProfile"));
 }
}
