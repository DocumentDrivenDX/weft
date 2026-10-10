//! Source-bound compiler query-use profile. No native credential or qualification.
use crate::{error::{Diagnostic,Result},json::{checked_json,sha256},model::Catalog,ir::ModelPin,security_ir::{SecurityLogicalPlan,QueryOperator},security_ontology::{SecurityOntologyClosure,SecurityRef},security_composition::Composition};
use serde::{Deserialize,Serialize};
use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/security-query-profile-v0.1.schema.json")]
struct ProfileSchema;
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-QUERY-PROFILE","model","Security query profile admission refused")}
#[derive(Debug,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct BindingSource{backend_id:String,backend_version:String,target_profile:String,sha256:String}
#[derive(Debug,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Entry{target:SecurityRef,field:SecurityRef,operator:QueryOperator,original_action:String}
#[derive(Debug,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Spec{version:String,id:String,revision:String,action:String,model_pins:Vec<ModelPin>,policy_sha256:String,ontology_sha256:String,binding:BindingSource,targets:Vec<SecurityRef>,bindings:Vec<Entry>}
#[derive(Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct ClaimedUse{field:SecurityRef,operator:QueryOperator,#[serde(skip_serializing_if="Option::is_none")]original_action:Option<String>}
/// Retained exact bytes and qualified bindings are immutable. Read/resolve do not
/// authenticate who supplied the profile or establish installed native mappings.
#[derive(Debug)]
pub struct SecurityQueryProfile{spec:Spec,model_inputs:Vec<crate::model::ModuleInput>,raw:String,binding_json:String,policy_json:String,ontology_json:String,closure:SecurityOntologyClosure,bindings:BTreeMap<(SecurityRef,SecurityRef,QueryOperator),String>}
/// Immutable compiler lineage/profile artifact, never a native credential.
#[derive(Debug)]
pub struct SecurityProfiledQuery<'a>{query:&'a crate::security_query_uses::SecurityResolvedQuery,profile:&'a SecurityQueryProfile,uses:Vec<(crate::security_query_uses::QueryUse,Option<String>)>,obligations:Vec<crate::security_scan_obligations::ScanObligations>}
impl SecurityProfiledQuery<'_>{
 pub fn obligations(&self)->&[crate::security_scan_obligations::ScanObligations]{&self.obligations}
 pub fn query(&self)->&crate::security_query_uses::SecurityResolvedQuery{self.query}
 pub fn uses(&self)->&[(crate::security_query_uses::QueryUse,Option<String>)]{&self.uses}
 /// Versioned backend mapping input, never an execution credential. Exact
 /// source reuse is checked before retaining the original resolved plan.
 pub fn mapping_handoff_json(&self,plan:&SecurityLogicalPlan,catalog:&Catalog,binding_json:&str,backend_id:&str,backend_version:&str,target_profile:&str)->Result<String>{
  self.require_sources(plan,catalog,binding_json,backend_id,backend_version,target_profile)?;
  let scans:Vec<Value>=self.obligations.iter().map(|scan|{
   let actions:Vec<Value>=scan.actions().iter().map(|action|{
    let keys:Vec<Value>=action.keys().iter().map(|(target,(id,fields))|serde_json::json!({"target":target,"keyId":id,"fields":fields})).collect();
    let fields:Vec<Value>=action.fields().iter().map(|(target,fields)|serde_json::json!({"target":target,"fields":fields})).collect();
    serde_json::json!({"action":action.action(),"ruleIds":action.rule_ids(),"keys":keys,"fields":fields,"context":action.context(),"associations":action.associations()})
   }).collect();
   serde_json::json!({"scan":scan.scan(),"target":scan.target(),"actions":actions,"projectionFields":scan.projection_fields(),"queryFields":scan.query_fields()})
  }).collect();
  let uses:Vec<Value>=self.uses.iter().map(|(u,action)|serde_json::json!({"scan":u.scan,"target":u.target,"field":u.field,"operator":u.operator,"originalAction":action})).collect();
  let result=serde_json::json!({"version":"weft.security.mapping-handoff/0.2.0","profile":{"id":self.profile.id(),"revision":self.profile.revision(),"sha256":sha256(self.profile.source_json().as_bytes())},"modelPins":catalog.pins(),"policySha256":sha256(plan.source().policy_json().as_bytes()),"ontologySha256":sha256(plan.source().ontology_json().as_bytes()),"binding":{"backendId":backend_id,"backendVersion":backend_version,"targetProfile":target_profile,"sha256":sha256(binding_json.as_bytes())},"sqlSha256":sha256(self.query.sql().as_bytes()),"applicationPlan":self.query.application_plan(),"securityLogicalPlan":{"version":"weft.security.logical-ir/0.1.0","rules":plan.rules()},"uses":uses,"scans":scans}).to_string();
  if result.len()>16_000_000{return Err(fail());}Ok(result)
 }
 pub fn require_sources(&self,plan:&SecurityLogicalPlan,catalog:&Catalog,binding_json:&str,backend_id:&str,backend_version:&str,target_profile:&str)->Result<()>{self.profile.require_sources(plan,catalog,binding_json,backend_id,backend_version,target_profile)?;self.query.require_sources(plan,catalog)}
}
impl SecurityQueryProfile{
 pub fn id(&self)->&str{&self.spec.id}
 pub fn revision(&self)->&str{&self.spec.revision}
 pub fn source_json(&self)->&str{&self.raw}
 pub fn binding_json(&self)->&str{&self.binding_json}
 pub fn read(raw:&str,plan:&SecurityLogicalPlan,catalog:&Catalog,binding_json:&str,backend_id:&str,backend_version:&str,target_profile:&str)->Result<Self>{
  plan.require_catalog(catalog)?;if raw.len()>4_000_000||binding_json.len()>4_000_000{return Err(fail());}
  let value=checked_json(raw).map_err(|_|fail())?;if !ProfileSchema::is_valid(&value){return Err(fail());}
  let spec:Spec=serde_json::from_value(value).map_err(|_|fail())?;
  checked_json(binding_json).map_err(|_|fail())?;
  if spec.version!="weft.security.query-profile/0.1.0"||spec.model_pins!=catalog.pins()||spec.policy_sha256!=sha256(plan.source().policy_json().as_bytes())||spec.ontology_sha256!=sha256(plan.source().ontology_json().as_bytes())||spec.binding.backend_id!=backend_id||spec.binding.backend_version!=backend_version||spec.binding.target_profile!=target_profile||spec.binding.sha256!=sha256(binding_json.as_bytes()){return Err(fail());}
  let closure=SecurityOntologyClosure::read(plan.source(),catalog)?;let actions=plan.source().ontology()["actions"].as_array().ok_or_else(fail)?;
  if !actions.iter().any(|a|a==&spec.action)||spec.targets.iter().collect::<BTreeSet<_>>().len()!=spec.targets.len()||spec.targets.iter().any(|t|!closure.types.contains_key(t)){return Err(fail());}
  let mut expected=BTreeSet::new();for target in &spec.targets{for field in closure.types[target].source["fields"].as_array().ok_or_else(fail)?{let reference=SecurityRef::read(&field["ref"])?;for operator in [QueryOperator::Predicate,QueryOperator::Order,QueryOperator::Group,QueryOperator::Join,QueryOperator::Aggregate]{if field["queryUse"][operator.name()]=="original-authorized"{expected.insert((target.clone(),reference.clone(),operator));}}}}
  let mut bindings=BTreeMap::new();for entry in &spec.bindings{let key=(entry.target.clone(),entry.field.clone(),entry.operator);if !expected.contains(&key)||entry.original_action==spec.action||!actions.iter().any(|a|a==&entry.original_action)||bindings.insert(key,entry.original_action.clone()).is_some(){return Err(fail());}}
  if bindings.keys().cloned().collect::<BTreeSet<_>>()!=expected{return Err(fail());}
  Ok(Self{spec,model_inputs:catalog.inputs.clone(),raw:raw.into(),binding_json:binding_json.into(),policy_json:plan.source().policy_json().into(),ontology_json:plan.source().ontology_json().into(),closure,bindings})
 }
 pub fn require_sources(&self,plan:&SecurityLogicalPlan,catalog:&Catalog,binding_json:&str,backend_id:&str,backend_version:&str,target_profile:&str)->Result<()>{
  plan.require_catalog(catalog)?;
  if self.model_inputs!=catalog.inputs||self.spec.model_pins!=catalog.pins()||self.policy_json!=plan.source().policy_json()||self.ontology_json!=plan.source().ontology_json()||self.binding_json!=binding_json||self.spec.binding.backend_id!=backend_id||self.spec.binding.backend_version!=backend_version||self.spec.binding.target_profile!=target_profile{return Err(fail());}Ok(())
 }
 fn derive(&self,target:&SecurityRef,field:&SecurityRef,operator:QueryOperator)->Result<ClaimedUse>{
  if !self.spec.targets.contains(target){return Err(fail());}let binding=self.closure.types.get(target).ok_or_else(fail)?;
  let classification=binding.source["fields"].as_array().ok_or_else(fail)?.iter().find(|f|SecurityRef::read(&f["ref"]).as_ref().is_ok_and(|r|r==field)).ok_or_else(fail)?;
  let mode=classification["queryUse"][operator.name()].as_str().unwrap_or(if classification["protection"]=="protected"{"prohibited"}else{"disclosed"});
  let original_action=match mode{"disclosed"=>None,"original-authorized"=>Some(self.bindings.get(&(target.clone(),field.clone(),operator)).ok_or_else(fail)?.clone()),_=>return Err(fail())};
  Ok(ClaimedUse{field:field.clone(),operator,original_action})
 }

 /// Admit compiler-extracted scan/field/operator lineage. The caller cannot supply
 /// a forged Plan: SecurityResolvedQuery owns the result of source resolution.
 pub fn admit_resolved_query<'a>(&'a self,query:&'a crate::security_query_uses::SecurityResolvedQuery,plan:&SecurityLogicalPlan,catalog:&Catalog,binding_json:&str,backend_id:&str,backend_version:&str,target_profile:&str)->Result<SecurityProfiledQuery<'a>>{
  self.require_sources(plan,catalog,binding_json,backend_id,backend_version,target_profile)?;query.require_sources(plan,catalog)?;
  if query.scans().values().any(|t|!self.spec.targets.contains(t)){return Err(fail());}
  let uses=query.uses().iter().map(|u|Ok((u.clone(),self.derive(&u.target,&u.field,u.operator)?.original_action))).collect::<Result<Vec<_>>>()?;
  let obligations=crate::security_scan_obligations::derive(query,plan,catalog,&self.spec.action,&uses)?;
  Ok(SecurityProfiledQuery{query,profile:self,uses,obligations})
 }
 /// Construct or verify query annotations from this exact profile, preserving all
 /// other request meaning for the downstream strict evaluator to admit.
 pub fn bind_request_json(&self,plan:&SecurityLogicalPlan,catalog:&Catalog,binding_json:&str,backend_id:&str,backend_version:&str,target_profile:&str,request_json:&str)->Result<String>{
  self.require_sources(plan,catalog,binding_json,backend_id,backend_version,target_profile)?;
  if request_json.len()>4_000_000{return Err(fail());}let mut request=checked_json(request_json).map_err(|_|fail())?;
  if request["action"]!=self.spec.action{return Err(fail());}
  let targets:Vec<SecurityRef>=if let Some(value)=request.get("targets").filter(|v|!v.is_null()){serde_json::from_value(value.clone()).map_err(|_|fail())?}else{request["resources"].as_array().ok_or_else(fail)?.iter().map(|r|serde_json::from_value(r["type"].clone()).map_err(|_|fail())).collect::<Result<BTreeSet<_>>>()?.into_iter().collect()};
  if targets.is_empty()||targets.len()>256||targets.iter().collect::<BTreeSet<_>>().len()!=targets.len()||targets.iter().any(|t|!self.spec.targets.contains(t)){return Err(fail());}
  let queries:Vec<ClaimedUse>=if let Some(value)=request.get("queryUses").filter(|v|!v.is_null()){serde_json::from_value(value.clone()).map_err(|_|fail())?}else{vec![]};
  if queries.len()>4096{return Err(fail());}let mut derived=Vec::new();
  for query in queries{let mut binding:Option<ClaimedUse>=None;for target in &targets{let current=self.derive(target,&query.field,query.operator)?;if query.original_action.as_ref().is_some_and(|claimed|Some(claimed)!=current.original_action.as_ref())||binding.as_ref().is_some_and(|prior|prior.original_action!=current.original_action){return Err(fail());}binding=Some(current);}derived.push(serde_json::to_value(binding.ok_or_else(fail)?).map_err(|_|fail())?);}
  request.as_object_mut().ok_or_else(fail)?.insert("queryUses".into(),Value::Array(derived));let result=request.to_string();if result.len()>4_000_000{return Err(fail());}Ok(result)
 }
 /// Conditional logical simulation through source-bound annotations. Native
 /// authenticity, installed mapping checks and output release remain unqualified.
 pub fn simulate_json(&self,plan:&SecurityLogicalPlan,catalog:&Catalog,binding_json:&str,backend_id:&str,backend_version:&str,target_profile:&str,cut_json:&str,request_json:&str)->Result<Vec<Composition>>{
  let bound=self.bind_request_json(plan,catalog,binding_json,backend_id,backend_version,target_profile,request_json)?;
  crate::security_evaluation::simulate_json(plan,catalog,cut_json,&bound)
 }
}

#[cfg(test)]
mod tests{
 use super::*;
 use serde_json::json;
 fn fixture()->(Catalog,SecurityLogicalPlan,Value,String,Value){
  let corpus:Value=serde_json::from_str(include_str!("../tests/security-evaluation-oracle.json")).unwrap();let mut case=corpus["cases"].as_array().unwrap().iter().find(|c|c["name"]=="query-predicate-original-grant").unwrap().clone();
  let targets=case["request"]["targets"].clone();
  case["policy"]["rules"].as_array_mut().unwrap().push(json!({"id":"unrelated-update","effect":"permit","actions":["update"],"target":targets,"condition":{"op":"literal","value":true}}));
  let f:Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();let s=&f["resolution"]["documents"][0];let mut document=s["document"].clone();for e in document["modules"][0]["elements"].as_array_mut().unwrap(){e["name"]=e["id"].clone();if e["scalarType"]=="integer"{e["facets"]=json!({"integerWidth":{"bits":64,"signed":true}});}}let text=document.to_string();
  let inputs=serde_json::from_value(json!([{"documentJson":text,"pin":{"documentId":s["document"]["id"],"revision":s["revision"],"umfVersion":"0.8.0","sha256":sha256(text.as_bytes())},"selectedModuleIds":["m"]}])).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let packet=crate::security_source::SecuritySourcePacket::read(&case["policy"].to_string(),&case["ontology"].to_string(),&catalog).unwrap();let plan=SecurityLogicalPlan::read(packet,&catalog).unwrap();
  let binding=json!({"version":"fixture-only","layout":"opaque-unqualified-storage"}).to_string();
  let profile=json!({"version":"weft.security.query-profile/0.1.0","id":"fixture-profile","revision":"p1","action":"read","modelPins":catalog.pins(),"policySha256":sha256(plan.source().policy_json().as_bytes()),"ontologySha256":sha256(plan.source().ontology_json().as_bytes()),"binding":{"backendId":"fixture","backendVersion":"unqualified","targetProfile":"fixture-only","sha256":sha256(binding.as_bytes())},"targets":case["request"]["targets"],"bindings":[{"target":case["request"]["targets"][0],"field":case["request"]["queryUses"][0]["field"],"operator":"predicate","originalAction":"query-original"}]});
  (catalog,plan,profile,binding,case)
 }
 fn read(p:&Value,plan:&SecurityLogicalPlan,catalog:&Catalog,binding:&str)->Result<SecurityQueryProfile>{SecurityQueryProfile::read(&p.to_string(),plan,catalog,binding,"fixture","unqualified","fixture-only")}
 #[test]
 fn bound_action_is_derived_and_unrelated_granted_action_cannot_substitute(){
  let (catalog,plan,p,binding,case)=fixture();let profile=read(&p,&plan,&catalog,&binding).unwrap();assert_eq!(profile.source_json(),p.to_string());assert_eq!(profile.binding_json(),binding);
  let mut request=case["request"].clone();request["queryUses"][0].as_object_mut().unwrap().remove("originalAction");
  let bound:Value=serde_json::from_str(&profile.bind_request_json(&plan,&catalog,&binding,"fixture","unqualified","fixture-only",&request.to_string()).unwrap()).unwrap();assert_eq!(bound["queryUses"][0]["originalAction"],"query-original");assert_eq!(profile.simulate_json(&plan,&catalog,&binding,"fixture","unqualified","fixture-only",&case["cut"].to_string(),&request.to_string()).unwrap().len(),1);
  request["queryUses"][0]["originalAction"]=json!("update");
  // Conditional unprofiled evaluation accepts the separate granted label; this is
  // not native authority. The source-bound profile closes that substitution path.
  assert_eq!(crate::security_evaluation::simulate_json(&plan,&catalog,&case["cut"].to_string(),&request.to_string()).unwrap().len(),1);
  assert!(profile.simulate_json(&plan,&catalog,&binding,"fixture","unqualified","fixture-only",&case["cut"].to_string(),&request.to_string()).is_err());
  request["resources"]=json!([]);assert!(profile.bind_request_json(&plan,&catalog,&binding,"fixture","unqualified","fixture-only",&request.to_string()).is_err());
 }
 #[test]
 fn complete_qualified_bindings_unknown_meaning_and_exact_pins_are_required(){
  let (catalog,plan,p,binding,_)=fixture();
  for mutation in 0..11{let mut bad=p.clone();match mutation{
   0=>bad["bindings"]=json!([]),1=>{let entry=bad["bindings"][0].clone();bad["bindings"].as_array_mut().unwrap().push(entry);},
   2=>bad["bindings"][0]["operator"]=json!("aggregate"),3=>bad["bindings"][0]["field"]["documentId"]=json!("other"),
   4=>bad["bindings"][0]["originalAction"]=json!("read"),5=>bad["bindings"][0]["originalAction"]=json!("undeclared"),
   6=>bad["policySha256"]=json!("0".repeat(64)),7=>bad["ontologySha256"]=json!("0".repeat(64)),
   8=>bad["modelPins"][0]["revision"]=json!("stale"),9=>bad["binding"]["sha256"]=json!("0".repeat(64)),_=>bad["futureMeaning"]=json!(true)}
   assert!(read(&bad,&plan,&catalog,&binding).is_err(),"{mutation}");
  }
 }
 #[test]
 fn profile_reuse_is_exact_source_custody_not_a_native_mapping_claim(){
  let (catalog,plan,p,binding,_)=fixture();let profile=read(&p,&plan,&catalog,&binding).unwrap();
  assert!(profile.require_sources(&plan,&catalog,&format!("{binding} "),"fixture","unqualified","fixture-only").is_err());
  assert!(profile.require_sources(&plan,&catalog,&binding,"fixture","changed","fixture-only").is_err());
  let mut policy=plan.source().policy().clone();policy["rules"][0]["condition"]["value"]=json!(false);let packet=crate::security_source::SecuritySourcePacket::read(&policy.to_string(),plan.source().ontology_json(),&catalog).unwrap();let changed=SecurityLogicalPlan::read(packet,&catalog).unwrap();assert!(profile.require_sources(&changed,&catalog,&binding,"fixture","unqualified","fixture-only").is_err());
  let mut changed_catalog=catalog.clone();changed_catalog.inputs[0].selected_module_ids.clear();assert!(profile.require_sources(&plan,&changed_catalog,&binding,"fixture","unqualified","fixture-only").is_err());
 }
 #[test]
 fn mapping_handoff_preserves_plan_identity_dependencies_and_source_refusal(){
  let (catalog,plan,p,binding,_)=fixture();let profile=read(&p,&plan,&catalog,&binding).unwrap();
  let query=crate::security_query_uses::SecurityResolvedQuery::resolve("SELECT r.resourceId FROM Resource r WHERE r.salary=100",&catalog,&plan,BTreeMap::new(),None).unwrap();
  let admitted=profile.admit_resolved_query(&query,&plan,&catalog,&binding,"fixture","unqualified","fixture-only").unwrap();
  let raw=admitted.mapping_handoff_json(&plan,&catalog,&binding,"fixture","unqualified","fixture-only").unwrap();let value:Value=serde_json::from_str(&raw).unwrap();
  assert_eq!(value["version"],"weft.security.mapping-handoff/0.2.0");assert_eq!(value["applicationPlan"],serde_json::to_value(query.application_plan()).unwrap());
  assert_eq!(value["sqlSha256"],sha256(query.sql().as_bytes()));assert_eq!(value["profile"]["sha256"],sha256(profile.source_json().as_bytes()));
  assert_eq!(value["securityLogicalPlan"]["version"],"weft.security.logical-ir/0.1.0");assert_eq!(value["securityLogicalPlan"]["rules"],serde_json::to_value(plan.rules()).unwrap());
  let rules=&value["securityLogicalPlan"]["rules"];assert_eq!(rules[0]["effect"],"permit");assert_eq!(rules[0]["disclosure"][0][1],"withheld");assert_eq!(rules[1]["effect"],"require");
  let owner=&rules[1]["condition"]["exists"];assert_eq!(owner["slot"],0);assert_eq!(owner["association"]["elementId"],"Ownership");
  let assignment=&owner["condition"]["and"][1]["exists"];assert_eq!(assignment["slot"],1);assert_eq!(assignment["association"]["elementId"],"Assignment");
  let active=&assignment["condition"]["and"][2]["equal"];assert_eq!(active[0]["field"]["binding"]["variable"],1);assert_eq!(active[0]["field"]["domain"]["scalarType"],"boolean");assert_eq!(active[1]["constant"]["literal"]["boolean"],true);

  assert_eq!(value["uses"][0]["field"]["elementId"],"salary");assert_eq!(value["uses"][0]["originalAction"],"query-original");
  assert_eq!(value["scans"][0]["actions"].as_array().unwrap().len(),2);
  assert_eq!(raw,admitted.mapping_handoff_json(&plan,&catalog,&binding,"fixture","unqualified","fixture-only").unwrap());
  assert!(admitted.mapping_handoff_json(&plan,&catalog,&format!("{binding} "),"fixture","unqualified","fixture-only").is_err());
  assert!(admitted.mapping_handoff_json(&plan,&catalog,&binding,"fixture","changed","fixture-only").is_err());
 }
 #[test]
 fn mapping_handoff_keeps_self_join_occurrences_and_field_free_count_authority(){
  let (catalog,plan,p,binding,_)=fixture();let profile=read(&p,&plan,&catalog,&binding).unwrap();
  let query=crate::security_query_uses::SecurityResolvedQuery::resolve("SELECT COUNT(*) FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId",&catalog,&plan,BTreeMap::new(),None).unwrap();
  let admitted=profile.admit_resolved_query(&query,&plan,&catalog,&binding,"fixture","unqualified","fixture-only").unwrap();
  let value:Value=serde_json::from_str(&admitted.mapping_handoff_json(&plan,&catalog,&binding,"fixture","unqualified","fixture-only").unwrap()).unwrap();
  let scans=value["scans"].as_array().unwrap();assert_eq!(scans.len(),2);assert_eq!(scans[0]["scan"],"s0");assert_eq!(scans[1]["scan"],"s1");
  for scan in scans{
   assert_eq!(scan["target"]["elementId"],"Resource");assert!(scan["projectionFields"].as_array().unwrap().is_empty());
   let actions=scan["actions"].as_array().unwrap();assert_eq!(actions.len(),1);assert_eq!(actions[0]["action"],"read");assert_eq!(actions[0]["ruleIds"],serde_json::json!(["membership","reader"]));
   let keys=actions[0]["keys"].as_array().unwrap();assert_eq!(keys.iter().map(|k|k["target"]["elementId"].as_str().unwrap()).collect::<Vec<_>>(),vec!["Assignment","Ownership","Project","Resource","Staff"]);
   assert!(keys.iter().all(|k|k["target"]["documentId"]=="domain"&&k["keyId"]=="pk"&&k["fields"].as_array().unwrap().len()==1));
   assert_eq!(actions[0]["associations"].as_array().unwrap().iter().map(|a|a["elementId"].as_str().unwrap()).collect::<Vec<_>>(),vec!["Assignment","Ownership"]);
  }
  let uses=value["uses"].as_array().unwrap();assert_eq!(uses.len(),2);assert_eq!(uses[0]["scan"],"s0");assert_eq!(uses[1]["scan"],"s1");assert!(uses.iter().all(|u|u["operator"]=="join"&&u["originalAction"].is_null()));
 }
 #[test]
 fn actual_resolved_predicate_derives_profile_action_without_caller_annotation(){
  let (catalog,plan,p,binding,_)=fixture();let profile=read(&p,&plan,&catalog,&binding).unwrap();let query=crate::security_query_uses::SecurityResolvedQuery::resolve("SELECT r.resourceId FROM Resource r WHERE r.salary=100",&catalog,&plan,BTreeMap::new(),None).unwrap();
  let uses=profile.admit_resolved_query(&query,&plan,&catalog,&binding,"fixture","unqualified","fixture-only").unwrap();assert_eq!(uses.uses().len(),1);assert_eq!(uses.uses()[0].0.field.element_id,"salary");assert_eq!(uses.uses()[0].0.operator,QueryOperator::Predicate);assert_eq!(uses.uses()[0].1.as_deref(),Some("query-original"));
 }

 #[test]
 fn scan_obligations_preserve_correlated_keys_and_field_free_count_dependencies(){
  let (catalog,plan,p,binding,_)=fixture();let profile=read(&p,&plan,&catalog,&binding).unwrap();
  let query=crate::security_query_uses::SecurityResolvedQuery::resolve("SELECT COUNT(*) FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId",&catalog,&plan,BTreeMap::new(),None).unwrap();
  let admitted=profile.admit_resolved_query(&query,&plan,&catalog,&binding,"fixture","unqualified","fixture-only").unwrap();assert_eq!(admitted.obligations().len(),2);
  for (index,scan) in admitted.obligations().iter().enumerate(){
   assert_eq!(scan.scan(),format!("s{index}"));assert_eq!(scan.target().element_id,"Resource");assert!(scan.projection_fields().is_empty());assert_eq!(scan.query_fields().iter().map(|f|f.element_id.as_str()).collect::<Vec<_>>(),vec!["resourceId"]);
   assert_eq!(scan.actions().len(),1);let action=&scan.actions()[0];assert_eq!(action.action(),"read");assert_eq!(action.rule_ids(),["membership","reader"]);
   assert_eq!(action.associations().iter().map(|a|a.element_id.as_str()).collect::<Vec<_>>(),vec!["Assignment","Ownership"]);
   assert_eq!(action.keys().keys().map(|a|a.element_id.as_str()).collect::<Vec<_>>(),vec!["Assignment","Ownership","Project","Resource","Staff"]);
   assert!(action.keys().values().all(|(id,components)|id=="pk"&&components.len()==1));
   let assignment=action.fields().iter().find(|(t,_)|t.element_id=="Assignment").unwrap().1;assert_eq!(assignment.iter().map(|f|f.element_id.as_str()).collect::<Vec<_>>(),vec!["active","assignmentId","assignmentProject","assignmentStaff"]);
   let resource=action.fields().iter().find(|(t,_)|t.element_id=="Resource").unwrap().1;assert_eq!(resource.iter().map(|f|f.element_id.as_str()).collect::<Vec<_>>(),vec!["resourceId"]);assert!(action.context().is_empty());
  }
 }
 #[test]
 fn original_action_false_branches_and_context_are_retained_without_constant_fact_reads(){
  let (catalog,old,p,binding,_)=fixture();let mut policy=old.source().policy().clone();let mut ontology=old.source().ontology().clone();
  let salary=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});ontology["context"]=json!([salary]);
  let membership=policy["rules"][1]["condition"].clone();policy["rules"].as_array_mut().unwrap().push(json!({"id":"original-dependencies","effect":"require","actions":["query-original"],"target":p["targets"],"condition":{"op":"or","args":[{"op":"literal","value":false},membership,{"op":"eq","left":{"kind":"context","field":salary},"right":{"kind":"constant","field":salary,"value":{"integerToken":"100"}}}]}}));
  let packet=crate::security_source::SecuritySourcePacket::read(&policy.to_string(),&ontology.to_string(),&catalog).unwrap();let plan=SecurityLogicalPlan::read(packet,&catalog).unwrap();let mut p=p;p["policySha256"]=json!(sha256(plan.source().policy_json().as_bytes()));p["ontologySha256"]=json!(sha256(plan.source().ontology_json().as_bytes()));let profile=read(&p,&plan,&catalog,&binding).unwrap();
  let query=crate::security_query_uses::SecurityResolvedQuery::resolve("SELECT COUNT(*) FROM Resource r WHERE r.salary=100",&catalog,&plan,BTreeMap::new(),None).unwrap();let admitted=profile.admit_resolved_query(&query,&plan,&catalog,&binding,"fixture","unqualified","fixture-only").unwrap();
  let scan=&admitted.obligations()[0];assert!(scan.projection_fields().is_empty());assert_eq!(scan.query_fields().iter().map(|f|f.element_id.as_str()).collect::<Vec<_>>(),vec!["salary"]);assert_eq!(scan.actions().len(),2);
  let original=scan.actions().iter().find(|a|a.action()=="query-original").unwrap();assert!(original.rule_ids().contains(&"original-dependencies".to_string()));assert_eq!(original.associations().len(),2);assert_eq!(original.context().iter().map(|f|f.element_id.as_str()).collect::<Vec<_>>(),vec!["salary"]);
  assert!(!original.fields().values().any(|fields|fields.iter().any(|f|f.element_id=="salary"))); // Domain-only constant and context are not resource salary reads.
  let read=scan.actions().iter().find(|a|a.action()=="read").unwrap();assert!(read.context().is_empty());
 }

}
