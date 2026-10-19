// SPDX-License-Identifier: MIT OR Apache-2.0
//! Weft-owned action designation over original typed queries. Never a permission.
use crate::{application_lineage::ApplicationLineage,error::{Diagnostic,Result},ir::{Identity,ModelPin},json::{checked_json,sha256},model::Catalog,security_ir::QueryOperator,security_ontology::SecurityRef};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::collections::{BTreeMap,BTreeSet};
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/current-query-designation-profile-v0.1-candidate.schema.json")]
struct ProfileSchema;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/current-query-designation-v0.1-candidate.schema.json")]
struct ReportSchema;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/current-query-designation-profile-v0.2-candidate.schema.json")]
struct SumProfileSchema;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/current-query-designation-v0.2-candidate.schema.json")]
struct SumReportSchema;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/current-query-designation-profile-v0.3-candidate.schema.json")]
struct BoundedProfileSchema;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/current-query-designation-v0.3-candidate.schema.json")]
struct BoundedReportSchema;
fn fail()->Diagnostic{Diagnostic::new("WFT-CURRENT-QUERY-DESIGNATION","model","Original current query designation refused")}
#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct SourceIdentity{pub document_id:String,pub revision:String}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Opaque{role:String,identity:SourceIdentity,media_type:String,utf8:String,sha256:String}
#[derive(Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct SourcePin{role:String,identity:SourceIdentity,sha256:String}
#[derive(Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct BindingScope{pub backend_id:String,pub backend_version:String,pub target_profile:String,pub source:SourceIdentity,pub sha256:String}
#[derive(Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Entry{target:SecurityRef,field:SecurityRef,operator:QueryOperator,original_action:String,#[serde(default,skip_serializing_if="Option::is_none")]operation:Option<String>}
#[derive(Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Profile{version:String,id:String,revision:String,model_pins:Vec<ModelPin>,sources:Vec<SourcePin>,binding:BindingScope,targets:Vec<SecurityRef>,primary_action:String,bindings:Vec<Entry>,#[serde(default,skip_serializing_if="Option::is_none")]capabilities:Option<Vec<String>>}
#[derive(Debug,Serialize)]
#[serde(rename_all="camelCase")]
struct UseDesignation{path:String,scan:String,target:SecurityRef,field:SecurityRef,field_identity:Identity,operator:QueryOperator,operation:String,original_action:Option<String>}
/// Only the factory can bind a profile to the original preparation. No serde/import API.
/// Reports cannot be imported as issued contexts.
/// ```compile_fail
/// use weft_core::current_query_designation::CurrentQueryDesignations;
/// fn import(raw:&str)->CurrentQueryDesignations<'_>{serde_json::from_str(raw).unwrap()}
/// ```
/// The original preparation cannot be replaced through the public API.
/// ```compile_fail
/// use weft_core::{application_lineage::ApplicationLineage,current_query_designation::CurrentQueryDesignations};
/// fn replace<'a>(d:&mut CurrentQueryDesignations<'a>,p:&'a ApplicationLineage){d.preparation=p;}
/// ```
/// Issued designations cannot outlive their actual original typed preparation.
/// ```compile_fail
/// use weft_core::{application_lineage,current_query_designation::{derive,CurrentQueryDesignations}};
/// fn detached(raw:&str)->CurrentQueryDesignations<'static>{
///   let original=application_lineage::prepare(raw).unwrap();derive(&original).unwrap()
/// }
/// ```
pub struct CurrentQueryDesignations<'a>{preparation:&'a ApplicationLineage,profile:Profile,uses:Vec<UseDesignation>,json:String}
impl CurrentQueryDesignations<'_>{
 pub fn json(&self)->&str{&self.json}
 pub fn preparation(&self)->&ApplicationLineage{self.preparation}
 pub fn primary_action(&self)->&str{&self.profile.primary_action}
 pub fn supports_bounded_unicode_strings(&self)->bool{self.profile.version=="weft.current-query.designation-profile/0.3-candidate" && self.profile.capabilities.as_ref().is_some_and(|v|v.len()==1 && v[0]=="bounded-unicode-scalars")}
 pub fn supports_live_original_integer_sum(&self)->bool{self.profile.version=="weft.current-query.designation-profile/0.2-candidate" && self.profile.capabilities.as_ref().is_some_and(|v|v.len()==1 && v[0]=="live-original-integer-sum")}
 pub fn binding_scope(&self)->&BindingScope{&self.profile.binding}
 pub fn original_action(&self,path:&str,scan:&str)->Result<Option<&str>>{self.uses.iter().find(|u|u.path==path&&u.scan==scan).map(|u|u.original_action.as_deref()).ok_or_else(fail)}
}
fn bounded(s:&str)->bool{!s.is_empty()&&s.len()<=4096&&!s.contains('\0')}
fn identity(c:&Catalog,r:&SecurityRef)->Result<Identity>{let pin=c.inputs().iter().find(|m|m.pin.document_id==r.document_id).ok_or_else(fail)?;Ok(Identity{document_id:r.document_id.clone(),revision:pin.pin.revision.clone(),module:r.module_id.clone(),element:r.element_id.clone()})}
fn reference(i:&Identity)->SecurityRef{SecurityRef{document_id:i.document_id.clone(),module_id:i.module.clone(),element_id:i.element.clone()}}
fn exact_keys(v:&Value,keys:&[&str])->bool{v.as_object().is_some_and(|m|m.len()==keys.len()&&keys.iter().all(|k|m.contains_key(*k)))}
/// Query-owned bounded action/purpose subset only; not whole UMF ontology admission.
fn purposes(source:&Opaque)->Result<BTreeMap<String,BTreeSet<String>>>{
 let v=checked_json(&source.utf8).map_err(|_|fail())?;
 if v["version"]!="0.2.0"||v["documentId"]!=source.identity.document_id||v["revision"]!=source.identity.revision{return Err(fail())}
 let actions=v["actions"].as_array().filter(|a|!a.is_empty()&&a.len()<=256).ok_or_else(fail)?;
 let mut expected=BTreeSet::new();for a in actions{let a=a.as_str().filter(|a|bounded(a)).ok_or_else(fail)?;if !expected.insert(a.to_owned()){return Err(fail())}}
 let entries=v["actionPurposes"].as_array().filter(|a|a.len()==expected.len()).ok_or_else(fail)?;let mut result=BTreeMap::new();
 for e in entries{
  if !exact_keys(e,&["action","purposes"]){return Err(fail())}let action=e["action"].as_str().filter(|a|expected.contains(*a)).ok_or_else(fail)?;
  let ps=e["purposes"].as_array().filter(|a|!a.is_empty()&&a.len()<=6).ok_or_else(fail)?;let mut set=BTreeSet::new();
  for p in ps{let p=p.as_str().filter(|p|matches!(*p,"read"|"field"|"original-authorized"|"reverse"|"vault-write"|"other")).ok_or_else(fail)?;if !set.insert(p.to_owned()){return Err(fail())}}
  if set.iter().any(|p|matches!(p.as_str(),"reverse"|"vault-write"|"other"))&&set.len()!=1{return Err(fail())}
  if result.insert(action.to_owned(),set).is_some(){return Err(fail())}
 }
 if result.keys().cloned().collect::<BTreeSet<_>>()!=expected{return Err(fail())}Ok(result)
}
pub(crate) fn encode(v:&Value,maximum:usize)->Result<String>{
 struct Buffer{bytes:Vec<u8>,left:usize}
 impl std::io::Write for Buffer{fn write(&mut self,b:&[u8])->std::io::Result<usize>{self.left=self.left.checked_sub(b.len()).ok_or_else(||std::io::Error::other("budget"))?;self.bytes.extend_from_slice(b);Ok(b.len())}fn flush(&mut self)->std::io::Result<()>{Ok(())}}
 let mut b=Buffer{bytes:Vec::new(),left:maximum};serde_json::to_writer(&mut b,v).map_err(|_|fail())?;String::from_utf8(b.bytes).map_err(|_|fail())
}
/// Select only original query-designation/native-binding and pinned source bytes.
/// No caller mode/action/AST override, policy evaluator or SQL callback is accepted.
pub fn derive(prepared:&ApplicationLineage)->Result<CurrentQueryDesignations<'_>>{
 let bounded_context=prepared.plan().ir_version=="weft-ir/application-lineage/0.3-candidate";
 if prepared.catalog().pins().iter().any(|p|p.umf_version!="0.8.0")||!matches!(prepared.plan().ir_version.as_str(),"weft-ir/application-lineage/0.2-candidate"|"weft-ir/application-lineage/0.3-candidate")||prepared.plan().filters.iter().any(|p|matches!(p,crate::application_ir::Predicate::HasRelated{..})){return Err(fail())}
 let request=checked_json(prepared.source_request_json()).map_err(|_|fail())?;
 let sources:Vec<Opaque>=serde_json::from_value(request["opaqueSources"].clone()).map_err(|_|fail())?;
 let get=|role:&str|->Result<&Opaque>{let all:Vec<_>=sources.iter().filter(|s|s.role==role).collect();if all.len()!=1{return Err(fail())}let s=all[0];if s.media_type!="application/json"||sha256(s.utf8.as_bytes())!=s.sha256{return Err(fail())}Ok(s)};
 let original=get("query-designation")?;let value=checked_json(&original.utf8).map_err(|_|fail())?;let sum_profile=value["version"]=="weft.current-query.designation-profile/0.2-candidate";let bounded_profile=value["version"]=="weft.current-query.designation-profile/0.3-candidate";if bounded_context!=bounded_profile{return Err(fail())}if !(if bounded_profile{BoundedProfileSchema::is_valid(&value)}else if sum_profile{SumProfileSchema::is_valid(&value)}else{ProfileSchema::is_valid(&value)}){return Err(fail())}let profile:Profile=serde_json::from_value(value.clone()).map_err(|_|fail())?;
 // Closed nested source/model references: no unknown member can be discarded by serde.
 if serde_json::to_value(&profile).map_err(|_|fail())?!=value||!matches!(profile.version.as_str(),"weft.current-query.designation-profile/0.1-candidate"|"weft.current-query.designation-profile/0.2-candidate"|"weft.current-query.designation-profile/0.3-candidate")||!bounded(&profile.id)||!bounded(&profile.revision)||original.identity!= (SourceIdentity{document_id:profile.id.clone(),revision:profile.revision.clone()})||profile.model_pins!=prepared.catalog().pins(){return Err(fail())}
 let required:BTreeSet<_>=["policy","ontology","catalog","owner"].into_iter().map(str::to_owned).collect();let mut seen=BTreeSet::new();
 for pin in &profile.sources{if !required.contains(&pin.role)||!seen.insert(pin.role.clone()){return Err(fail())}let s=get(&pin.role)?;if s.identity!=pin.identity||s.sha256!=pin.sha256{return Err(fail())}}
 if seen!=required{return Err(fail())}
 let binding=get("native-binding")?;
 if binding.identity!=profile.binding.source||binding.sha256!=profile.binding.sha256||![&profile.binding.backend_id,&profile.binding.backend_version,&profile.binding.target_profile].into_iter().all(|s|bounded(s)){return Err(fail())}
 let _=checked_json(&binding.utf8).map_err(|_|fail())?;
 let action_purposes=purposes(get("ontology")?)?;
 if !bounded(&profile.primary_action)||!action_purposes.get(&profile.primary_action).is_some_and(|p|p.contains("read")){return Err(fail())}
 if profile.targets.is_empty()||profile.targets.len()>1024||profile.bindings.len()>4096{return Err(fail())}
 let mut targets=BTreeSet::new();for t in &profile.targets{if !targets.insert(t.clone()){return Err(fail())}prepared.catalog().record_by_identity(&identity(prepared.catalog(),t)?).map_err(|_|fail())?;}
 let scans=crate::application_property_uses::scans(prepared.plan())?;if scans.values().any(|i|!targets.contains(&reference(i))){return Err(fail())}
 let mut entries=BTreeMap::new();
 for (index,e) in profile.bindings.iter().enumerate(){
  if (sum_profile||bounded_profile) && !matches!((e.operator,e.operation.as_deref()),(QueryOperator::Predicate,Some("equal"|"order"))|(QueryOperator::Join,Some("equal"))|(QueryOperator::Group,Some("group"))|(QueryOperator::Order,Some("order"))|(QueryOperator::Aggregate,Some("countDistinct"|"sum"))){return Err(fail())}

  if bounded_profile && e.operation.as_deref()==Some("sum"){return Err(fail())}
  if !targets.contains(&e.target)||e.original_action==profile.primary_action||!action_purposes.get(&e.original_action).is_some_and(|p|p.contains("original-authorized")){return Err(fail())}
  let record=prepared.catalog().record_by_identity(&identity(prepared.catalog(),&e.target)?).map_err(|_|fail())?;
  prepared.catalog().member_descriptor_by_identity(&record,&identity(prepared.catalog(),&e.field)?).map_err(|_|fail())?;
  if entries.insert((e.target.clone(),e.field.clone(),e.operator,e.operation.clone()),index).is_some(){return Err(fail())}
 }
 let mut uses=Vec::new();let mut remaining=8_000_000usize;
 crate::application_property_uses::visit(prepared.plan(),prepared.catalog(),|u|{
  if uses.len()>=4096{return Err(fail())}let target=reference(scans.get(u.scan).ok_or_else(fail)?);let field=reference(u.identity);
  let action=entries.get(&(target.clone(),field.clone(),u.operator,if sum_profile||bounded_profile{Some(u.operation.to_owned())}else{None})).map(|i|profile.bindings[*i].original_action.clone());
  let row=UseDesignation{path:u.path,scan:u.scan.to_owned(),target,field,field_identity:u.identity.clone(),operator:u.operator,operation:u.operation.into(),original_action:action};
  let value=serde_json::to_value(&row).map_err(|_|fail())?;let bytes=encode(&value,remaining)?;remaining=remaining.checked_sub(bytes.len()).ok_or_else(fail)?;uses.push(row);Ok(())
 })?;
 let mut report=json!({"version":"weft.current-query.designation/0.1-candidate","authority":false,"securityReady":false,"sourceRequestSha256":sha256(prepared.source_request_json().as_bytes()),"sourceProfileJson":original.utf8,"sourceProfileSha256":original.sha256,"modelPins":prepared.catalog().pins(),"binding":profile.binding,"sources":profile.sources,"primaryAction":profile.primary_action,"primaryActionRequiredPurpose":"read","originalActionRequiredPurpose":"original-authorized","uses":uses,"unresolved":["UMF whole-source semantic admission","joint ontology/catalog query modes","per-occurrence effective value and paired native scope","authenticated native source/caller/current authority and guarded release"]});
 if sum_profile{report["version"]=json!("weft.current-query.designation/0.2-candidate");report["capabilities"]=json!(profile.capabilities);}
 if bounded_profile{report["version"]=json!("weft.current-query.designation/0.3-candidate");report["capabilities"]=json!(profile.capabilities);}
 if !(if bounded_profile{BoundedReportSchema::is_valid(&report)}else if sum_profile{SumReportSchema::is_valid(&report)}else{ReportSchema::is_valid(&report)}){return Err(fail())}
 let json=encode(&report,16_000_000)?;Ok(CurrentQueryDesignations{preparation:prepared,profile,uses,json})
}


/// Source-bound computation inputs issued only from the actual typed context.
/// This is not a native credential and cannot be imported as an issued object.
/// The host must still authenticate and admit every original captured source.
pub fn native_request_json(g:&crate::current_query_operands::CurrentQueryOperands<'_>)->Result<String>{
 use crate::{application_ir as app,current_query_operands::{Input,Kind}};
 let d=g.designation();let p=d.preparation();let plan=p.plan();
 let scan_map=crate::application_property_uses::scans(plan)?;
 let scans=std::iter::once(&plan.source).chain(plan.joins.iter().map(|j|&j.right)).map(|s|json!({"scan":s.occurrence,"record":s.record,"type":reference(&s.record)})).collect::<Vec<_>>();
 let mut projections=Vec::new();
 for (i,o) in plan.outputs.iter().enumerate(){
  if let app::Expression::Field{scan,identity}=&o.expression{
   let record=p.catalog().record_by_identity(scan_map.get(scan).ok_or_else(fail)?)?;
   let descriptor=p.catalog().member_descriptor_by_identity(&record,identity)?;
   projections.push(json!({"path":format!("outputs/{i}/expression"),"scan":scan,"field":identity,"descriptor":descriptor}));
  }
 }
 let mut routes=Vec::new();
 crate::application_property_uses::visit(plan,p.catalog(),|u|{
  if routes.len()>=4096{return Err(fail())}
  routes.push(json!({"scan":u.scan,"path":u.path,"type":reference(scan_map.get(u.scan).ok_or_else(fail)?),"field":reference(u.identity),"operator":u.operator,"operation":u.operation,"required":if u.operation=="sum" && d.supports_live_original_integer_sum(){vec!["nullity"]}else{u.required.into_iter().collect::<Vec<_>>()}}));Ok(())
 })?;
 let mut operations=Vec::new();
 for op in g.operations(){
  let (kind,direction,boolean)=match op.kind(){
   Kind::Equal=>("equal",None,None),Kind::LexicographicGreater=>("lexicographic-greater",None,None),Kind::Group=>("group",None,None),
   Kind::Order(direction)=>("order",Some(match direction{crate::application_lineage::OrderDirection::Asc=>"asc",crate::application_lineage::OrderDirection::Desc=>"desc"}),None),
   Kind::Count=>("count",None,None),Kind::CountDistinct=>("count-distinct",None,None),Kind::Sum=>("sum",None,None),Kind::Boolean(value)=>("boolean",None,Some(value))};
  let operands=op.operands().iter().map(|operand|{let input=match operand.input(){
   Input::Field{scan,identity,logical_type,original_action}=>json!({"kind":"field","scan":scan,"identity":identity,"logicalType":logical_type,"originalAction":original_action}),
   Input::Literal{value,logical_type}=>json!({"kind":"literal","value":value,"logicalType":logical_type}),
   Input::Parameter{name,value,logical_type}=>json!({"kind":"parameter","name":name,"value":value,"logicalType":logical_type})};
   json!({"path":operand.path(),"input":input})}).collect::<Vec<_>>();
  operations.push(json!({"path":op.path(),"operator":op.operator(),"kind":kind,"direction":direction,"boolean":boolean,"operands":operands,"outputType":op.output_type()}));
 }
 let mut packet=json!({"version":"weft.current-query.native-input/0.1-candidate","authority":false,"securityReady":false,"sourceRequestSha256":sha256(p.source_request_json().as_bytes()),"designationSha256":sha256(d.json().as_bytes()),"modelPins":p.catalog().pins(),"binding":d.binding_scope(),"sources":d.profile.sources,"primaryAction":d.primary_action(),"scans":scans,"projections":projections,"operations":operations,"routes":routes});
 if d.supports_bounded_unicode_strings(){packet["version"]=json!("weft.current-query.native-input/0.3-candidate");packet["capabilities"]=json!(d.profile.capabilities);}
 if d.supports_live_original_integer_sum(){packet["version"]=json!("weft.current-query.native-input/0.2-candidate");packet["capabilities"]=json!(d.profile.capabilities);}
 encode(&packet,16_000_000)
}

#[cfg(test)]
mod tests{
 use super::*;
 // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
 #[test]
 fn native_request_derives_original_routes_operations_and_literals_without_report_import(){
  let r=request();let p=crate::application_lineage::prepare(&r.to_string()).unwrap();let d=derive(&p).unwrap();let g=crate::current_query_operands::derive(&d).unwrap();
  let output:Value=serde_json::from_str(&native_request_json(&g).unwrap()).unwrap();
  assert_eq!(output["primaryAction"],d.primary_action());assert_eq!(output["designationSha256"],sha256(d.json().as_bytes()));
  assert_eq!(output["sourceRequestSha256"],sha256(p.source_request_json().as_bytes()));
  assert_eq!(output["routes"][0]["required"],json!(["equality"]));assert_eq!(output["routes"][0]["type"]["elementId"],"Resource");
  assert!(output["routes"][0].get("originalAction").is_none());
  assert_eq!(output["operations"][0]["operands"][0]["input"]["originalAction"],"query-original-salary-predicate");
  assert_eq!(output["operations"][0]["operands"][1]["input"]["value"],"100");
  assert_eq!(output["operations"][1]["direction"],"desc");
  assert_eq!(output["projections"][0]["path"],"outputs/0/expression");assert_eq!(output["authority"],false);assert_eq!(output["securityReady"],false);
 }
 fn sum_request()->Value{serde_json::from_str(include_str!("../../../tests/current-query-designation/fixtures/live-original-integer-sum.request.json")).unwrap()}
 // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
 #[test]
 fn explicit_sum_context_preserves_original_action_presence_dependency_and_nullable_math_type(){
  let r=sum_request();let p=crate::application_lineage::prepare(&r.to_string()).unwrap();let d=derive(&p).unwrap();assert!(d.supports_live_original_integer_sum());
  let g=crate::current_query_operands::derive(&d).unwrap();assert_eq!(g.operations()[0].kind(),crate::current_query_operands::Kind::Sum);
  assert_eq!(d.original_action("outputs/0/argument","s0").unwrap(),Some("query-original-salary-sum"));
  let report:Value=serde_json::from_str(d.json()).unwrap();assert_eq!(report["version"],"weft.current-query.designation/0.2-candidate");
  let native:Value=serde_json::from_str(&native_request_json(&g).unwrap()).unwrap();assert_eq!(native["version"],"weft.current-query.native-input/0.2-candidate");
  assert_eq!(native["routes"][0]["required"],json!(["nullity"]));assert_eq!(native["routes"][0]["path"],"outputs/0/argument");assert_eq!(native["operations"][0]["kind"],"sum");
  assert_eq!(native["operations"][0]["outputType"],json!({"family":"integer","facets":{},"nullable":true}));assert_eq!(native["projections"],json!([]));
  assert_eq!(native["authority"],false);assert_eq!(native["securityReady"],false);
 }
 // @covers US-009-AC2
 #[test]
 fn live_original_sum_does_not_narrow_unsupported_input_domains(){
  for facets in [json!({"integerWidth":{"bits":32,"signed":true}}),json!({"integerWidth":{"bits":64,"signed":false}}),json!({})]{
   let mut r=sum_request();let mut document:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
   document["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="salary").unwrap()["facets"]=facets;
   let raw=document.to_string();r["modules"][0]["documentJson"]=json!(raw);r["modules"][0]["pin"]["sha256"]=json!(sha256(raw.as_bytes()));
   let pins=json!([r["modules"][0]["pin"].clone()]);profile(&mut r,|p|p["modelPins"]=pins);
   let parent=crate::application_lineage::prepare(&r.to_string()).unwrap();let d=derive(&parent).unwrap();let error=match crate::current_query_operands::derive(&d){Ok(_)=>panic!("Unsupported SUM domain admitted"),Err(error)=>error};assert_eq!(error.code,"WFT-CURRENT-QUERY-OPERANDS");
  }
 }
 // @covers US-009-AC2 @covers US-009-AC3
 #[test]
 fn sum_capability_has_no_legacy_unknown_or_missing_action_fallback(){
  for op in [json!("arithmetic"),Value::Null]{let mut r=sum_request();profile(&mut r,|p|p["bindings"][2]["operation"]=op);assert!(result(&r).is_err());}
  let mut absent=sum_request();profile(&mut absent,|p|{p["bindings"][2].as_object_mut().unwrap().remove("operation");});assert!(result(&absent).is_err());
  let mut distinct=sum_request();distinct["sql"]=json!("SELECT COUNT(DISTINCT r.salary) AS n FROM Resource r");let parent=crate::application_lineage::prepare(&distinct.to_string()).unwrap();let selected=derive(&parent).unwrap();assert_eq!(selected.original_action("outputs/0/argument","s0").unwrap(),None);

  for capability in [json!([]),json!(["sum"]),json!(["live-original-integer-sum","live-original-integer-sum"]),Value::Null]{let mut r=sum_request();profile(&mut r,|p|p["capabilities"]=capability);assert!(result(&r).is_err());}
  let mut r=sum_request();profile(&mut r,|p|p["version"]=json!("weft.current-query.designation-profile/0.1-candidate"));assert!(result(&r).is_err());
  let mut r=sum_request();profile(&mut r,|p|{p.as_object_mut().unwrap().remove("capabilities");});assert!(result(&r).is_err());
  let mut r=sum_request();profile(&mut r,|p|{p["bindings"].as_array_mut().unwrap().retain(|b|b["operator"]!="aggregate");});
  let p=crate::application_lineage::prepare(&r.to_string()).unwrap();let d=derive(&p).unwrap();assert!(crate::current_query_operands::derive(&d).is_err());
  let mut r=request();r["sql"]=json!("SELECT SUM(r.salary) AS total FROM Resource r");let p=crate::application_lineage::prepare(&r.to_string()).unwrap();let d=derive(&p).unwrap();assert!(!d.supports_live_original_integer_sum());assert!(crate::current_query_operands::derive(&d).is_err());
 }
 fn request()->Value{serde_json::from_str(include_str!("../../../tests/current-query-designation/fixtures/mixed-original-uses.request.json")).unwrap()}
 fn source_index(r:&Value,role:&str)->usize{r["opaqueSources"].as_array().unwrap().iter().position(|s|s["role"]==role).unwrap()}
 fn profile(r:&mut Value,edit:impl FnOnce(&mut Value)){let i=source_index(r,"query-designation");let mut p:Value=serde_json::from_str(r["opaqueSources"][i]["utf8"].as_str().unwrap()).unwrap();edit(&mut p);let raw=p.to_string();r["opaqueSources"][i]["utf8"]=json!(raw);r["opaqueSources"][i]["sha256"]=json!(sha256(raw.as_bytes()));}
 fn ontology(r:&mut Value,edit:impl FnOnce(&mut Value)){let i=source_index(r,"ontology");let mut p:Value=serde_json::from_str(r["opaqueSources"][i]["utf8"].as_str().unwrap()).unwrap();edit(&mut p);let raw=p.to_string();let digest=sha256(raw.as_bytes());r["opaqueSources"][i]["utf8"]=json!(raw);r["opaqueSources"][i]["sha256"]=json!(digest);profile(r,|p|{let s=p["sources"].as_array_mut().unwrap().iter_mut().find(|s|s["role"]=="ontology").unwrap();s["sha256"]=json!(digest)});}
 fn result(r:&Value)->Result<Value>{let p=crate::application_lineage::prepare(&r.to_string())?;let d=derive(&p)?;serde_json::from_str(d.json()).map_err(|_|fail())}
 // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
 #[test]
 fn actual_context_issues_per_occurrence_actions_without_mode_or_permission(){
  let r=request();let raw=r.to_string();let p=crate::application_lineage::prepare(&raw).unwrap();let d=derive(&p).unwrap();assert!(std::ptr::eq(d.preparation(),&p));
  assert_eq!(d.original_action("filters/0/left","s0").unwrap(),Some("query-original-salary-predicate"));assert_eq!(d.original_action("order/0","s0").unwrap(),Some("query-original-token-order"));assert!(d.original_action("order/0","s1").is_err());
  let report:Value=serde_json::from_str(d.json()).unwrap();assert_eq!(report["authority"],false);assert_eq!(report["securityReady"],false);assert_eq!(report["sourceRequestSha256"],sha256(raw.as_bytes()));assert_eq!(p.order_directions(),&[crate::application_lineage::OrderDirection::Desc]);
  for u in report["uses"].as_array().unwrap(){assert!(u.get("mode").is_none());assert!(u.get("permitted").is_none());}
  let salary=p.plan().type_graph.iter().find(|t|t.identity.element=="salary").unwrap();let crate::application_model::Shape::Scalar{logical_type}= &salary.shape else{panic!("Original scalar required")};assert_eq!(logical_type.family,crate::ir::Family::Integer);
 }
 // @covers US-009-AC1 @covers US-009-AC2
 #[test]
 fn profile_overrides_legacy_versions_wrong_pins_missing_roles_and_field_forgery_refuse(){
  for field in ["mode","originalAction","applicationPlan"]{let mut r=request();profile(&mut r,|p|p[field]=json!("caller"));assert!(result(&r).is_err());}
  for version in ["weft.security.query-profile/0.1.0","weft.security.query-profile/0.2-candidate"]{let mut r=request();profile(&mut r,|p|p["version"]=json!(version));assert!(result(&r).is_err());}
  for role in ["policy","ontology","catalog","owner","native-binding","query-designation"]{let mut r=request();let i=source_index(&r,role);r["opaqueSources"].as_array_mut().unwrap().remove(i);assert!(result(&r).is_err(),"{role}");}
  let mut r=request();profile(&mut r,|p|p["modelPins"][0]["sha256"]=json!("0".repeat(64)));assert!(result(&r).is_err());
  let mut r=request();profile(&mut r,|p|p["binding"]["sha256"]=json!("0".repeat(64)));assert!(result(&r).is_err());
  let mut r=request();profile(&mut r,|p|p["bindings"][0]["field"]["elementId"]=json!("salaryMaskLabel"));assert!(result(&r).is_err());
  let mut r=request();profile(&mut r,|p|{let e=p["bindings"][0].clone();p["bindings"].as_array_mut().unwrap().push(e)});assert!(result(&r).is_err());
  let mut r=request();let i=source_index(&r,"ontology");let mut duplicate=r["opaqueSources"][i].clone();duplicate["identity"]["revision"]=json!("other");r["opaqueSources"].as_array_mut().unwrap().push(duplicate);assert!(result(&r).is_err());
 }
 // @covers US-009-AC2 @covers US-009-AC3
 #[test]
 fn exact_action_purposes_refuse_unrelated_actions_and_incomplete_or_unknown_closure(){
  for action in ["read","update","reverse","vault-write","not-declared"]{let mut r=request();profile(&mut r,|p|p["bindings"][0]["originalAction"]=json!(action));assert!(result(&r).is_err(),"{action}");}
  for change in 0..5{let mut r=request();ontology(&mut r,|o|{match change{0=>{o.as_object_mut().unwrap().remove("actionPurposes");},1=>{o["actionPurposes"].as_array_mut().unwrap().pop();},2=>{let e=o["actionPurposes"][0].clone();o["actionPurposes"].as_array_mut().unwrap().push(e);},3=>{o["actionPurposes"][0]["purposes"]=json!(["unknown"]);},_=>{o["actionPurposes"][0]["purposes"]=json!(["read","reverse"]);}}});assert!(result(&r).is_err());}
 }
 // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
 #[test]
 fn omitted_designation_is_not_permission_and_false_zero_keeps_uses(){
  let mut r=request();profile(&mut r,|p|p["bindings"]=json!([]));r["sql"]=json!("SELECT COUNT(DISTINCT r.token) FROM Resource r WHERE FALSE AND r.salary=100 LIMIT 0");let report=result(&r).unwrap();assert_eq!(report["uses"].as_array().unwrap().len(),2);assert!(report["uses"].as_array().unwrap().iter().all(|u|u["originalAction"].is_null()));assert_eq!(report["authority"],false);
  r["interfaceVersion"]=json!("weft-application-lineage/0.1-candidate");assert!(result(&r).is_err());
 }
}

#[cfg(test)]
mod bounded_tests {
 use super::*;
 fn request()->Value{serde_json::from_str(include_str!("../../../tests/current-query-designation/fixtures/bounded-unicode-strings.request.json")).unwrap()}
 fn profile(r:&mut Value,change:impl FnOnce(&mut Value)){
  let s=r["opaqueSources"].as_array_mut().unwrap().iter_mut().find(|s|s["role"]=="query-designation").unwrap();
  let mut p:Value=serde_json::from_str(s["utf8"].as_str().unwrap()).unwrap();change(&mut p);let raw=p.to_string();s["sha256"]=json!(sha256(raw.as_bytes()));s["utf8"]=json!(raw);
 }
 fn result(r:&Value)->Result<Value>{let p=crate::application_lineage::prepare(&r.to_string())?;let d=derive(&p)?;serde_json::from_str(d.json()).map_err(|_|fail())}
 #[test]
 fn bounded_designation_is_version_paired_and_retains_original_operands(){
  let r=request();let p=crate::application_lineage::prepare(&r.to_string()).unwrap();let d=derive(&p).unwrap();
  assert!(d.supports_bounded_unicode_strings());assert!(!d.supports_live_original_integer_sum());
  let report:Value=serde_json::from_str(d.json()).unwrap();assert_eq!(report["version"],"weft.current-query.designation/0.3-candidate");assert_eq!(report["capabilities"],json!(["bounded-unicode-scalars"]));
  let g=crate::current_query_operands::derive(&d).unwrap();let native:Value=serde_json::from_str(&native_request_json(&g).unwrap()).unwrap();
  assert_eq!(native["version"],"weft.current-query.native-input/0.3-candidate");assert_eq!(native["capabilities"],json!(["bounded-unicode-scalars"]));
  assert_eq!(native["operations"][0]["operands"][1]["input"]["value"],"😀");
  assert_eq!(report["sourceRequestSha256"],sha256(p.source_request_json().as_bytes()));
  assert_eq!(report["authority"],false);assert_eq!(report["securityReady"],false);
 }
 #[test]
 fn bounded_capability_cannot_upgrade_old_source_or_select_sum(){
  for cap in [json!([]),json!(["live-original-integer-sum"]),json!(["bounded-unicode-scalars","bounded-unicode-scalars"]),json!(["bounded-unicode-scalars","live-original-integer-sum"])] {
   let mut r=request();profile(&mut r,|p|p["capabilities"]=cap);assert!(result(&r).is_err());
  }
  for version in ["weft.current-query.designation-profile/0.1-candidate","weft.current-query.designation-profile/0.2-candidate"] {
   let mut r=request();profile(&mut r,|p|p["version"]=json!(version));assert!(result(&r).is_err());
  }
  for operation in [None,Some("sum"),Some("unknown")] {
   let mut r=request();profile(&mut r,|p|{if let Some(op)=operation {p["bindings"][0]["operation"]=json!(op)}else{p["bindings"][0].as_object_mut().unwrap().remove("operation");}});assert!(result(&r).is_err());
  }
  let mut r=request();r["sql"]=json!("SELECT SUM(r.salary) AS total FROM Resource r");r["parameters"]=json!({});
  let refused=(||{let p=crate::application_lineage::prepare(&r.to_string())?;let d=derive(&p)?;crate::current_query_operands::derive(&d).map(|_|())})();assert!(refused.is_err());
 }
}

#[cfg(feature="current-presence-context-candidate")]
pub(crate) fn require_presence_read_purpose(raw:&str,identity:&SourceIdentity,action:&str)->Result<()> {
 let source=Opaque{role:"ontology".into(),identity:identity.clone(),media_type:"application/json".into(),utf8:raw.into(),sha256:sha256(raw.as_bytes())};
 if purposes(&source)?.get(action).is_some_and(|p|p.contains("read")){Ok(())}else{Err(fail())}
}
