// SPDX-License-Identifier: MIT OR Apache-2.0
//! Separately typed, source-bound presence preparation. Never native permission.
use crate::{application_lineage::ApplicationLineage,current_query_designation::{BindingScope,SourceIdentity},error::{Diagnostic,Result},ir::ModelPin,json::{checked_json,sha256},security_ontology::SecurityRef};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::collections::BTreeSet;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/current-presence-profile-v0.1-candidate.schema.json")]
struct ProfileSchema;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/current-presence-preparation-v0.1-candidate.schema.json")]
struct ReportSchema;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/current-presence-native-input-v0.1-candidate.schema.json")]
struct NativeSchema;
const PROFILE:&str="weft.current-presence.designation-profile/0.1-candidate";
const CAPABILITY:&str="tagged-current-projection";
const PRODUCER:&str="umf.security.logical-disclosure-batch/0.1-candidate";
const ENTRYPOINT:&str="evaluateCandidateCatalogReadDisclosureBatchWithinOperation";
fn fail()->Diagnostic{Diagnostic::new("WFT-CURRENT-PRESENCE-CONTEXT","model","Source-bound presence preparation refused")}
fn text(s:&str)->bool{!s.is_empty()&&s.len()<=4096&&!s.contains('\0')}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Source{role:String,identity:SourceIdentity,media_type:String,utf8:String,sha256:String}
#[derive(Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Pin{role:String,identity:SourceIdentity,sha256:String}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
struct Producer{protocol:String,entrypoint:String}
#[derive(Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Profile{version:String,id:String,revision:String,model_pins:Vec<ModelPin>,sources:Vec<Pin>,binding:BindingScope,targets:Vec<SecurityRef>,primary_action:String,capabilities:Vec<String>,producer:Producer}
/// An original .4 preparation and a new explicit profile, never a uniform scalar context.
/// No deserializer or public constructor can import an issued object.
/// ```compile_fail
/// use weft_core::current_presence::CurrentPresencePreparation;
/// fn imported(s:&str)->CurrentPresencePreparation<'_>{serde_json::from_str(s).unwrap()}
/// ```
/// It cannot be returned as the old uniform context.
/// ```compile_fail
/// use weft_core::{application_lineage::ApplicationLineage,current_presence,current_query_designation::CurrentQueryDesignations,error::Result};
/// fn old<'a>(p:&'a ApplicationLineage)->Result<CurrentQueryDesignations<'a>>{current_presence::prepare(p)}
/// ```
/// It cannot outlive its actual parent.
/// ```compile_fail
/// use weft_core::{application_lineage,current_presence::{self,CurrentPresencePreparation}};
/// fn detached<'a>(s:&str)->CurrentPresencePreparation<'a>{let p=application_lineage::prepare(s).unwrap();current_presence::prepare(&p).unwrap()}
/// ```
pub struct CurrentPresencePreparation<'a>{original:&'a ApplicationLineage,profile:Profile,json:String}
impl CurrentPresencePreparation<'_>{
 pub fn preparation(&self)->&ApplicationLineage{self.original}
 pub fn json(&self)->&str{&self.json}
 pub fn primary_action(&self)->&str{&self.profile.primary_action}
 pub fn binding_scope(&self)->&BindingScope{&self.profile.binding}
 /// Exact candidate producer contract required of the host; not a verified invocation.
 pub fn producer_protocol(&self)->&str{&self.profile.producer.protocol}
 /// Source-bound data for an independent native owner. No caller/cut or cells are supplied.
 pub fn native_request_json(&self)->Result<String>{
  let mut value=checked_json(&self.json).map_err(|_|fail())?;
  value["version"]=json!("weft.current-presence.native-input/0.1-candidate");
  value["preparationSha256"]=json!(sha256(self.json.as_bytes()));
  value["sourceRequestJson"]=json!(self.original.source_request_json());
  if !NativeSchema::is_valid(&value){return Err(fail())}
  crate::current_query_designation::encode(&value,16_000_000).map_err(|_|fail())
 }
}
/// Requires the actual immutable .4 parent and a separately named source profile.
/// The old current-query constructor remains closed to this parent/type.
pub fn prepare(original:&ApplicationLineage)->Result<CurrentPresencePreparation<'_>>{
 if original.plan().ir_version!="weft-ir/application-lineage/0.4-candidate"
  || !original.plan().required_capabilities.iter().any(|c|c==CAPABILITY)
  || original.catalog().pins().iter().any(|p|p.umf_version!="0.8.0"){return Err(fail())}
 let request=checked_json(original.source_request_json()).map_err(|_|fail())?;
 let sources:Vec<Source>=serde_json::from_value(request["opaqueSources"].clone()).map_err(|_|fail())?;
 let get=|role:&str|->Result<&Source>{
  let mut found=sources.iter().filter(|s|s.role==role);let source=found.next().ok_or_else(fail)?;
  if found.next().is_some()||source.media_type!="application/json"||sha256(source.utf8.as_bytes())!=source.sha256{return Err(fail())}Ok(source)
 };
 let source=get("presence-designation")?;let value=checked_json(&source.utf8).map_err(|_|fail())?;
 if !ProfileSchema::is_valid(&value){return Err(fail())}
 let profile:Profile=serde_json::from_value(value.clone()).map_err(|_|fail())?;
 if serde_json::to_value(&profile).map_err(|_|fail())?!=value||profile.version!=PROFILE
  || profile.capabilities!=[CAPABILITY]||profile.producer.protocol!=PRODUCER||profile.producer.entrypoint!=ENTRYPOINT
  || !text(&profile.id)||!text(&profile.revision)||source.identity!=(SourceIdentity{document_id:profile.id.clone(),revision:profile.revision.clone()})
  || profile.model_pins!=original.catalog().pins(){return Err(fail())}
 let required:BTreeSet<_>=["policy","ontology","catalog","owner"].into_iter().collect();let mut seen=BTreeSet::new();
 for pin in &profile.sources{
  if !required.contains(pin.role.as_str())||!seen.insert(pin.role.as_str()){return Err(fail())}
  let input=get(&pin.role)?;if input.identity!=pin.identity||input.sha256!=pin.sha256{return Err(fail())}
 }
 if seen!=required{return Err(fail())}
 let binding=get("native-binding")?;
 if binding.identity!=profile.binding.source||binding.sha256!=profile.binding.sha256
  || ![&profile.binding.backend_id,&profile.binding.backend_version,&profile.binding.target_profile].into_iter().all(|s|text(s))
  || !checked_json(&binding.utf8).map_err(|_|fail())?.is_object(){return Err(fail())}
 let ontology=get("ontology")?;
 if !text(&profile.primary_action){return Err(fail())}
 crate::current_query_designation::require_presence_read_purpose(&ontology.utf8,&ontology.identity,&profile.primary_action).map_err(|_|fail())?;
 let record=&original.plan().source.record;
 let target=SecurityRef{document_id:record.document_id.clone(),module_id:record.module.clone(),element_id:record.element.clone()};
 if profile.targets!=[target]{return Err(fail())}
 let lineage=checked_json(original.json()).map_err(|_|fail())?;
 let report=json!({"version":"weft.current-presence.preparation/0.1-candidate","capabilities":[CAPABILITY],"authority":false,"securityReady":false,"nativeAdmission":"required",
  "sourceRequestSha256":sha256(original.source_request_json().as_bytes()),"lineageSha256":sha256(original.json().as_bytes()),
  "sourceProfileJson":source.utf8,"sourceProfileSha256":source.sha256,"modelPins":original.catalog().pins(),"sources":profile.sources,
  "binding":profile.binding,"primaryAction":profile.primary_action,"producer":profile.producer,"readRequirements":lineage["readRequirements"],
  "unresolved":["whole-source UMF semantic admission","actual canonical owner-fold invocation and selected source occurrences","native caller/source/current-cut authentication","materialization/result codec and guarded release"]});
 if !ReportSchema::is_valid(&report){return Err(fail())}
 let json=crate::current_query_designation::encode(&report,16_000_000).map_err(|_|fail())?;
 Ok(CurrentPresencePreparation{original,profile,json})
}

#[cfg(test)]
pub(crate) mod tests{
 use super::*;
 pub(crate) fn fixture()->Value{
  let mut request:Value=serde_json::from_str(include_str!("../../../tests/current-query-presence/fixtures/optional-token-lineage.request.json")).unwrap();
  let pins:Vec<Value>=request["opaqueSources"].as_array().unwrap().iter().map(|s|json!({"role":s["role"],"identity":s["identity"],"sha256":s["sha256"]})).collect();
  let ontology:Value=serde_json::from_str(request["opaqueSources"].as_array().unwrap().iter().find(|s|s["role"]=="ontology").unwrap()["utf8"].as_str().unwrap()).unwrap();
  let action=ontology["actionPurposes"].as_array().unwrap().iter().find(|a|a["purposes"].as_array().unwrap().iter().any(|p|p=="read")).unwrap()["action"].clone();
  // Authored unadmitted data fixture, never a repaired/accepted Truss binding.
  let native=json!({"version":"weft.presence-test-source/0.1-candidate","scope":"unadmitted fixture"}).to_string();
  let source_identity=json!({"documentId":"presence-test-native-source","revision":"test-r1"});
  request["opaqueSources"].as_array_mut().unwrap().push(json!({"role":"native-binding","identity":source_identity,"mediaType":"application/json","sha256":sha256(native.as_bytes()),"utf8":native}));
  let model_pins:Vec<_>=request["modules"].as_array().unwrap().iter().map(|m|m["pin"].clone()).collect();
  let profile=json!({"version":PROFILE,"id":"presence-profile","revision":"presence-r1","modelPins":model_pins,"sources":pins,
   "binding":{"backendId":"test.presence-source","backendVersion":"0.1-candidate","targetProfile":"unadmitted-fixture","source":source_identity,"sha256":sha256(native.as_bytes())},
   "targets":[{"documentId":"domain","moduleId":"m","elementId":"Resource"}],"primaryAction":action,"capabilities":[CAPABILITY],"producer":{"protocol":PRODUCER,"entrypoint":ENTRYPOINT}});
  let raw=profile.to_string();request["opaqueSources"].as_array_mut().unwrap().push(json!({"role":"presence-designation","identity":{"documentId":"presence-profile","revision":"presence-r1"},"mediaType":"application/json","sha256":sha256(raw.as_bytes()),"utf8":raw}));request
 }
 fn change(request:&mut Value,f:impl FnOnce(&mut Value)){
  let source=request["opaqueSources"].as_array_mut().unwrap().iter_mut().find(|s|s["role"]=="presence-designation").unwrap();
  let mut profile:Value=serde_json::from_str(source["utf8"].as_str().unwrap()).unwrap();f(&mut profile);let raw=profile.to_string();source["utf8"]=json!(raw);source["sha256"]=json!(sha256(raw.as_bytes()));
 }
 #[test]
 fn separate_presence_context_retains_parent_and_exact_sources_without_uniform_issuance(){
  let request=fixture();let original=crate::application_lineage::prepare(&request.to_string()).unwrap();let context=prepare(&original).unwrap();
  assert!(std::ptr::eq(context.preparation(),&original));assert!(crate::current_query_designation::derive(&original).is_err());
  let report:Value=serde_json::from_str(context.json()).unwrap();assert_eq!(report["authority"],false);assert_eq!(report["securityReady"],false);assert_eq!(report["nativeAdmission"],"required");
  assert_eq!(report["readRequirements"]["fields"][0]["descriptor"]["availability"],"absent-allowed");
  let native:Value=serde_json::from_str(&context.native_request_json().unwrap()).unwrap();
  assert_eq!(native["version"],"weft.current-presence.native-input/0.1-candidate");assert_eq!(native["preparationSha256"],sha256(context.json().as_bytes()));assert_eq!(native["sourceRequestJson"],original.source_request_json());
 }
 #[test]
 fn earlier_lineage_versions_never_select_the_presence_registration(){
  for version in ["0.1-candidate","0.2-candidate","0.3-candidate"]{
   let mut request=fixture();request["interfaceVersion"]=json!(format!("weft-application-lineage/{version}"));
   request["sql"]=json!("SELECT r.resourceId FROM Resource r");
   let original=crate::application_lineage::prepare(&request.to_string()).unwrap();
   assert!(prepare(&original).is_err(),"{version}");
  }
 }
 #[test]
 fn legacy_profile_wrong_capability_producer_scope_and_source_pins_refuse(){
  for case in 0..8{
   let mut request=fixture();change(&mut request,|p|match case{
    0=>p["version"]=json!("weft.current-query.designation-profile/0.3-candidate"),
    1=>p["capabilities"]=json!([]),
    2=>p["producer"]["protocol"]=json!("umf.security.logical-disclosure-selection/0.1-candidate"),
    3=>p["producer"]["entrypoint"]=json!("evaluateCandidateCatalogScan"),
    4=>{p["sources"].as_array_mut().unwrap().pop();},
    5=>p["binding"]["sha256"]=json!("0".repeat(64)),
    6=>p["targets"][0]["elementId"]=json!("Project"),
    _=>p["extra"]=json!(true),
   });let original=crate::application_lineage::prepare(&request.to_string()).unwrap();assert!(prepare(&original).is_err(),"{case}");
  }
 }
 #[cfg(feature="current-presence-model-requirements-candidate")]
 #[test]
 fn model_requirements_retain_original_parent_and_all_record_members(){
  let request=fixture();let original=crate::application_lineage::prepare(&request.to_string()).unwrap();let context=prepare(&original).unwrap();
  let requirements=crate::current_presence_model_requirements::derive(&context).unwrap();assert!(std::ptr::eq(requirements.context(),&context));
  let report:Value=serde_json::from_str(requirements.json()).unwrap();assert_eq!(report["complete"],false);assert_eq!(report["authority"],false);assert_eq!(report["securityReady"],false);
  assert_eq!(report["sourceRequestJson"],original.source_request_json());assert_eq!(report["preparationSha256"],sha256(context.json().as_bytes()));
  assert_eq!(report["modelPins"],serde_json::to_value(original.catalog().pins()).unwrap());assert_eq!(report["records"].as_array().unwrap().len(),5);
  // Independently authored complete expected Field/Key mapping, not generated
  // from the source under test or from the implementation's traversal.
  let mut actual=std::collections::BTreeMap::new();
  for r in report["records"].as_array().unwrap(){actual.insert(r["record"]["element"].as_str().unwrap().to_owned(),json!({"fields":r["fields"].as_array().unwrap().iter().map(|f|f["field"]["element"].clone()).collect::<Vec<_>>(),"key":r["selectedKey"]["fields"].as_array().unwrap().iter().map(|f|f["element"].clone()).collect::<Vec<_>>()}));}
  assert_eq!(serde_json::to_value(actual).unwrap(),json!({
   "Staff":{"fields":["staffId"],"key":["staffId"]},
   "Project":{"fields":["projectId"],"key":["projectId"]},
   "Resource":{"fields":["resourceId","salary","token"],"key":["resourceId"]},
   "Ownership":{"fields":["ownerId","ownerResource","ownerProject"],"key":["ownerId"]},
   "Assignment":{"fields":["assignmentId","assignmentStaff","assignmentProject","active"],"key":["assignmentId"]}
  }));
  let resource=report["records"].as_array().unwrap().iter().find(|r|r["record"]["element"]=="Resource").unwrap();
  assert_eq!(resource["fields"].as_array().unwrap().len(),3);assert_eq!(resource["selectedKey"]["fields"][0]["element"],"resourceId");
  let token=resource["fields"].as_array().unwrap().iter().find(|f|f["field"]["element"]=="token").unwrap();assert_eq!(token["descriptors"][0]["availability"],"absent-allowed");
 }
 #[cfg(feature="current-presence-model-requirements-candidate")]
 fn change_ontology(request:&mut Value,f:impl FnOnce(&mut Value)){
  let source=request["opaqueSources"].as_array_mut().unwrap().iter_mut().find(|s|s["role"]=="ontology").unwrap();
  let mut ontology:Value=serde_json::from_str(source["utf8"].as_str().unwrap()).unwrap();f(&mut ontology);let raw=ontology.to_string();let digest=sha256(raw.as_bytes());source["utf8"]=json!(raw);source["sha256"]=json!(digest);
  change(request,|p|p["sources"].as_array_mut().unwrap().iter_mut().find(|s|s["role"]=="ontology").unwrap()["sha256"]=json!(digest));
 }
 #[cfg(feature="current-presence-model-requirements-candidate")]
 #[test]
 fn model_requirements_refuse_incomplete_foreign_duplicate_and_mismatched_records(){
  for case in 0..7{
   let mut request=fixture();change_ontology(&mut request,|o|match case{
    0=>{o["entities"][2]["fields"].as_array_mut().unwrap().pop();},
    1=>o["entities"][2]["fields"][0]["ref"]["elementId"]=json!("staffId"),
    2=>{let e=o["entities"][0].clone();o["entities"].as_array_mut().unwrap().push(e);},
    3=>o["entities"][2]["keyId"]=json!("missing"),
    4=>o["associations"][0]["keyId"]=json!("missing"),
    5=>o["entities"][2]["type"]["documentId"]=json!("foreign"),
    _=>{let f=o["entities"][2]["fields"][0].clone();o["entities"][2]["fields"].as_array_mut().unwrap().push(f);},
   });let original=crate::application_lineage::prepare(&request.to_string()).unwrap();let context=prepare(&original).unwrap();
   assert!(crate::current_presence_model_requirements::derive(&context).is_err(),"case {case}");
  }
 }
 #[cfg(feature="current-presence-model-requirements-candidate")]
 #[test]
 fn model_requirements_preserve_unknown_graph_source_without_claiming_closure(){
  let mut request=fixture();change_ontology(&mut request,|o|o["x-future-graph"]=json!({"targetKey":null,"direction":"unknown","precision":"9007199254740993.000000001"}));
  let original=crate::application_lineage::prepare(&request.to_string()).unwrap();let context=prepare(&original).unwrap();
  let requirements=crate::current_presence_model_requirements::derive(&context).unwrap();let report:Value=serde_json::from_str(requirements.json()).unwrap();
  assert_eq!(report["sourceRequestJson"],original.source_request_json());assert_eq!(report["complete"],false);assert!(report["unresolved"].as_array().unwrap().iter().any(|s|s.as_str().unwrap().contains("graph direction")));
 }

 #[cfg(feature="current-presence-model-requirements-candidate")]
 #[test]
 fn model_requirements_keep_composite_key_order_instead_of_member_order(){
  let mut request=fixture();let module=&mut request["modules"][0];
  let mut document:Value=serde_json::from_str(module["documentJson"].as_str().unwrap()).unwrap();
  let resource=document["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="Resource").unwrap();
  resource["keys"][0]["fields"]=json!([{"module":"m","element":"salary"},{"module":"m","element":"resourceId"}]);
  let raw=document.to_string();module["documentJson"]=json!(raw);module["pin"]["sha256"]=json!(sha256(raw.as_bytes()));let pin=module["pin"].clone();
  change(&mut request,|p|p["modelPins"][0]=pin);
  let original=crate::application_lineage::prepare(&request.to_string()).unwrap();let context=prepare(&original).unwrap();
  let requirements=crate::current_presence_model_requirements::derive(&context).unwrap();let report:Value=serde_json::from_str(requirements.json()).unwrap();
  let r=report["records"].as_array().unwrap().iter().find(|r|r["record"]["element"]=="Resource").unwrap();
  assert_eq!(r["fields"][0]["field"]["element"],"resourceId");assert_eq!(r["selectedKey"]["fields"][0]["element"],"salary");assert_eq!(r["selectedKey"]["fields"][1]["element"],"resourceId");
  // This changes compiler model requirements only. It does not repair or admit
  // the fixture ontology's endpoint incidence for the new composite Key.
  assert_eq!(report["complete"],false);
 }
 #[cfg(feature="current-presence-model-requirements-candidate")]
 #[test]
 fn model_requirements_equal_bytes_do_not_replace_actual_context_custody(){
  let request=fixture().to_string();let first=crate::application_lineage::prepare(&request).unwrap();let second=crate::application_lineage::prepare(&request).unwrap();
  let a=prepare(&first).unwrap();let b=prepare(&second).unwrap();let requirements=crate::current_presence_model_requirements::derive(&a).unwrap();
  let other=crate::current_presence_model_requirements::derive(&b).unwrap();assert_eq!(requirements.json(),other.json());
  assert!(std::ptr::eq(requirements.context(),&a));assert!(!std::ptr::eq(requirements.context(),&b));
 }

 #[cfg(feature="current-presence-model-requirements-candidate")]
 #[test]
 fn model_requirements_context_hashes_do_not_feed_back_into_persistent_binding(){
  let first=fixture();let mut second=first.clone();change_ontology(&mut second,|o|o["x-original-order-control"]=json!({"revision":"independent-r2"}));
  let binding=|r:&Value|r["opaqueSources"].as_array().unwrap().iter().find(|s|s["role"]=="native-binding").unwrap().clone();
  let descriptor=binding(&first);assert_eq!(descriptor,binding(&second));
  // The same independently supplied descriptor precedes both actual preparations.
  // Neither construction imports a report or substitutes an earlier context.
  let a=crate::application_lineage::prepare(&first.to_string()).unwrap();let b=crate::application_lineage::prepare(&second.to_string()).unwrap();
  let ca=prepare(&a).unwrap();let cb=prepare(&b).unwrap();
  let qa=crate::current_presence_model_requirements::derive(&ca).unwrap();let qb=crate::current_presence_model_requirements::derive(&cb).unwrap();
  assert_eq!(ca.binding_scope().sha256,descriptor["sha256"].as_str().unwrap());assert_eq!(cb.binding_scope().sha256,ca.binding_scope().sha256);
  let ra:Value=serde_json::from_str(qa.json()).unwrap();let rb:Value=serde_json::from_str(qb.json()).unwrap();
  assert_ne!(ra["sourceRequestSha256"],rb["sourceRequestSha256"]);assert_ne!(ra["preparationSha256"],rb["preparationSha256"]);
  assert_eq!(binding(&serde_json::from_str(ra["sourceRequestJson"].as_str().unwrap()).unwrap()),descriptor);
  assert_eq!(binding(&serde_json::from_str(rb["sourceRequestJson"].as_str().unwrap()).unwrap()),descriptor);
  assert!(std::ptr::eq(qa.context(),&ca));assert!(!std::ptr::eq(qa.context(),&cb));assert_eq!(ra["complete"],false);assert_eq!(rb["authority"],false);
 }

}
