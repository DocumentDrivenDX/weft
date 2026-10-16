//! Original model requirements for native correspondence; never native admission.
use crate::{current_presence::CurrentPresencePreparation,error::{Diagnostic,Result},ir::Identity,json::{checked_json,sha256},security_ontology::{authored_key_fields,SecurityRef}};
use serde_json::{json,Value};
use std::collections::{BTreeMap,BTreeSet};
fn fail()->Diagnostic{Diagnostic::new("WFT-CURRENT-PRESENCE-MODEL-REQUIREMENTS","model","Original model requirements refused")}
/// Borrows the actual issued context; importing a report cannot issue this type.
/// ```compile_fail
/// use weft_core::current_presence_model_requirements::CurrentPresenceModelRequirements;
/// fn imported(s:&str)->CurrentPresenceModelRequirements<'_, '_>{serde_json::from_str(s).unwrap()}
/// ```
/// The issued requirements cannot escape their actual borrowed context.
/// ```compile_fail
/// use weft_core::{current_presence::CurrentPresencePreparation,current_presence_model_requirements::{derive,CurrentPresenceModelRequirements}};
/// fn escaped(c:&CurrentPresencePreparation<'_>)->CurrentPresenceModelRequirements<'static,'static>{derive(c).unwrap()}
/// ```
pub struct CurrentPresenceModelRequirements<'context,'parent>{context:&'context CurrentPresencePreparation<'parent>,json:String}
impl<'context,'parent> CurrentPresenceModelRequirements<'context,'parent>{
 pub fn context(&self)->&'context CurrentPresencePreparation<'parent>{self.context}
 pub fn json(&self)->&str{&self.json}
}
fn identity(context:&CurrentPresencePreparation<'_>,r:&SecurityRef)->Result<Identity>{
 let pins=context.preparation().catalog().pins();let matching:Vec<_>=pins.iter().filter(|p|p.document_id==r.document_id).collect();
 if matching.len()!=1{return Err(fail())}
 Ok(Identity{document_id:r.document_id.clone(),revision:matching[0].revision.clone(),module:r.module_id.clone(),element:r.element_id.clone()})
}
/// Resolve Record/member/selected-Key requirements using the original compiler.
/// Retains complete opaque source bytes. Graph/codec/native/witness closure is
/// required separately and is never inferred from this incomplete data report.
pub fn derive<'context,'parent>(context:&'context CurrentPresencePreparation<'parent>)->Result<CurrentPresenceModelRequirements<'context,'parent>>{
 let parent=context.preparation();let catalog=parent.catalog();
 let request=checked_json(parent.source_request_json()).map_err(|_|fail())?;
 let sources=request["opaqueSources"].as_array().ok_or_else(fail)?;
 let mut ontologies=sources.iter().filter(|s|s["role"]=="ontology");let source=ontologies.next().ok_or_else(fail)?;
 if ontologies.next().is_some(){return Err(fail())}
 let raw=source["utf8"].as_str().ok_or_else(fail)?;let ontology=checked_json(raw).map_err(|_|fail())?;
 let entities=ontology["entities"].as_array().ok_or_else(fail)?;
 let associations=ontology["associations"].as_array().ok_or_else(fail)?;
 if entities.is_empty()||entities.len()>4096||associations.len()>4096{return Err(fail())}
 let mut records=BTreeMap::new();let mut selected=BTreeMap::new();
 for entity in entities{
  let reference=SecurityRef::read(&entity["type"])?;
  let record=catalog.record_by_identity(&identity(context,&reference)?)?;
  let key=entity["keyId"].as_str().ok_or_else(fail)?;
  let key_fields=authored_key_fields(catalog,&reference,key)?;
  let mut fields=Vec::new();let mut seen=BTreeSet::new();
  for field in entity["fields"].as_array().ok_or_else(fail)?{
   let r=SecurityRef::read(&field["ref"])?;if !seen.insert(r.clone()){return Err(fail())}
   let id=identity(context,&r)?;let (_,descriptors)=catalog.member_descriptor_by_identity(&record,&id)?;
   fields.push(json!({"field":id,"descriptors":descriptors}));
  }
  let mut members=BTreeSet::new();
  for member in record.value["members"].as_array().ok_or_else(fail)?{
   let r=SecurityRef{document_id:reference.document_id.clone(),module_id:member["module"].as_str().ok_or_else(fail)?.into(),element_id:member["element"].as_str().ok_or_else(fail)?.into()};
   if !members.insert(r){return Err(fail())}
  }
  if seen!=members{return Err(fail())}
  let ordered:Vec<_>=key_fields.iter().map(|r|identity(context,r)).collect::<Result<_>>()?;
  let value=json!({"record":record.identity,"fields":fields,"selectedKey":{"id":key,"fields":ordered}});
  if records.insert(reference.clone(),value).is_some(){return Err(fail())}
  selected.insert(reference,key.to_owned());
 }
 // Record-backed associations reuse their entity's explicit selected Key.
 // Other graph forms remain original source obligations, never traversal defaults.
 for association in associations{
  if association["kind"]=="record-members"{
   let r=SecurityRef::read(&association["type"])?;
   if selected.get(&r).map(String::as_str)!=association["keyId"].as_str(){return Err(fail())}
  }
 }
 let report=json!({"version":"weft.current-presence.model-requirements/0.1-candidate","scope":"original-record-field-key-requirements-only",
  "complete":false,"authority":false,"securityReady":false,"nativeAdmission":"required",
  "sourceRequestJson":parent.source_request_json(),"sourceRequestSha256":sha256(parent.source_request_json().as_bytes()),
  "preparationSha256":sha256(context.json().as_bytes()),"modelPins":catalog.pins(),"ontologySha256":sha256(raw.as_bytes()),
  "records":records.into_values().collect::<Vec<_>>(),
  "unresolved":["whole-source UMF semantic admission","original graph direction, association Record and endpoint-Key correspondence","context-field native correspondence","accepted property/home/native identity and original codec correspondence","actual authenticated operation and continuously held native source/authority witness","closing guards and guarded release"]});
 let json=crate::current_query_designation::encode(&report,16_000_000).map_err(|_|fail())?;
 Ok(CurrentPresenceModelRequirements{context,json})
}
