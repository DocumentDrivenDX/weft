// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use serde_json::json;
fn fixture()->Value{crate::current_presence::tests::fixture()}
fn profile(q:&mut Value,f:impl FnOnce(&mut Value)){
 let s=q["opaqueSources"].as_array_mut().unwrap().iter_mut().find(|s|s["role"]=="presence-designation").unwrap();let mut p:Value=serde_json::from_str(s["utf8"].as_str().unwrap()).unwrap();f(&mut p);let raw=p.to_string();s["sha256"]=json!(sha256(raw.as_bytes()));s["utf8"]=json!(raw);
}
fn ontology(q:&mut Value,f:impl FnOnce(&mut Value)){
 let s=q["opaqueSources"].as_array_mut().unwrap().iter_mut().find(|s|s["role"]=="ontology").unwrap();let mut o:Value=serde_json::from_str(s["utf8"].as_str().unwrap()).unwrap();f(&mut o);let raw=o.to_string();let hash=sha256(raw.as_bytes());s["sha256"]=json!(hash);s["utf8"]=json!(raw);profile(q,|p|p["sources"].as_array_mut().unwrap().iter_mut().find(|s|s["role"]=="ontology").unwrap()["sha256"]=json!(hash));
}
fn model(q:&mut Value,f:impl FnOnce(&mut Value)){
 let m=&mut q["modules"][0];let mut d:Value=serde_json::from_str(m["documentJson"].as_str().unwrap()).unwrap();f(&mut d);let raw=d.to_string();m["pin"]["sha256"]=json!(sha256(raw.as_bytes()));m["documentJson"]=json!(raw);let pins:Vec<_>=q["modules"].as_array().unwrap().iter().map(|m|m["pin"].clone()).collect();profile(q,|p|p["modelPins"]=json!(pins));
}
fn r(id:&str)->Value{json!({"documentId":"domain","moduleId":"m","elementId":id})}
fn graph(q:&mut Value,record_witness:bool){
 model(q,|d|{
  let mut declaration=json!({"id":"Assignment","name":"same-as-record-namespace","source":[{"module":"m","element":"Assignment"},{"module":"m","element":"Assignment"}],"target":[{"module":"m","element":"Project","key":"pk"},{"module":"m","element":"Project","key":"pk"}],"sourceMultiplicity":{"min":0,"max":"*"},"targetMultiplicity":{"min":0,"max":"*"},"targetLifecycle":"independent","directed":true});
  if record_witness{declaration["associationRecord"]=json!({"module":"m","element":"Ownership"})}d["modules"][0]["relationships"]=json!([declaration]);
 });
 ontology(q,|o|o["associations"].as_array_mut().unwrap().push(json!({"kind":"core-relationship","relationship":{"documentId":"domain","moduleId":"m","relationshipId":"Assignment"},"witness":if record_witness{json!({"kind":"record-key","type":r("Ownership"),"keyId":"pk"})}else{json!({"kind":"opaque-existential"})},"endpoints":[{"role":"assignment","side":"source","target":r("Assignment"),"keyId":"pk"},{"role":"project","side":"target","target":r("Project"),"keyId":"pk"}]})));
}
fn report(q:&Value)->Value{
 let original=crate::application_lineage::prepare(&q.to_string()).unwrap();let context=crate::current_presence::prepare(&original).unwrap();let issued=derive(&context).unwrap();assert!(std::ptr::eq(issued.context(),&context));assert!(std::ptr::eq(issued.model_requirements().context(),&context));let value:Value=serde_json::from_str(issued.json()).unwrap();assert_eq!(value["modelRequirementsSha256"],sha256(issued.model_requirements().json().as_bytes()));assert_eq!(value["sourceRequestJson"],original.source_request_json());assert_eq!(value["complete"],false);assert_eq!(value["authority"],false);assert_eq!(value["securityReady"],false);assert!(ReportSchema::is_valid(&value));value
}
fn refuses_at(q:&Value,stage:&str){
 let original=match crate::application_lineage::prepare(&q.to_string()){Ok(v)=>v,Err(_)=>{assert_eq!(stage,"parent");return}};assert_ne!(stage,"parent");let context=crate::current_presence::prepare(&original).unwrap();assert!(derive(&context).is_err());
}
fn refuses(q:&Value){refuses_at(q,"graph")}
#[test]
fn record_members_have_independent_full_owner_target_key_and_component_mapping(){
 let value=report(&fixture());let entries=value["associations"].as_array().unwrap();assert_eq!(entries.len(),2);
 let actual:Vec<_>=entries.iter().map(|a|json!({"ordinal":a["ordinal"],"owner":a["owner"]["element"],"key":a["key"]["id"],"endpoints":a["endpoints"].as_array().unwrap().iter().map(|e|json!({"role":e["role"],"target":e["target"]["element"],"key":e["key"]["fields"][0]["element"],"member":e["members"][0]["field"]["element"]})).collect::<Vec<_>>()})).collect();
 assert_eq!(actual,vec![json!({"ordinal":0,"owner":"Ownership","key":"pk","endpoints":[{"role":"resource","target":"Resource","key":"resourceId","member":"ownerResource"},{"role":"project","target":"Project","key":"projectId","member":"ownerProject"}]}),json!({"ordinal":1,"owner":"Assignment","key":"pk","endpoints":[{"role":"staff","target":"Staff","key":"staffId","member":"assignmentStaff"},{"role":"project","target":"Project","key":"projectId","member":"assignmentProject"}]})]);
 for entry in entries{assert!(entry["reference"].get("elementId").is_some());assert!(entry["reference"].get("relationshipId").is_none());for endpoint in entry["endpoints"].as_array().unwrap(){let member=&endpoint["members"][0];assert_eq!(member["field"],member["descriptors"][0]["identity"]);assert_eq!(endpoint["key"]["record"],endpoint["target"]);}}
}
#[test]
fn graph_namespace_and_all_duplicate_declaration_ordinals_are_preserved(){
 let mut q=fixture();graph(&mut q,false);let value=report(&q);let a=&value["associations"][2];assert_eq!(a["reference"],json!({"documentId":"domain","moduleId":"m","relationshipId":"Assignment"}));assert!(a["reference"].get("elementId").is_none());assert_eq!(a["sources"].as_array().unwrap().len(),2);assert_eq!(a["targets"].as_array().unwrap().len(),2);assert_eq!(a["endpoints"][0]["declarationOrdinals"],json!([0,1]));assert_eq!(a["endpoints"][1]["declarationOrdinals"],json!([0,1]));assert_eq!(a["targets"][0]["key"]["fields"][0]["element"],"projectId");assert_eq!(a["witness"],json!({"kind":"opaque-existential"}));assert_eq!(a["associationRecord"],Value::Null);
}
#[test]
fn record_key_witness_is_distinct_from_relationship_and_opaque_domain(){
 let mut q=fixture();graph(&mut q,true);let a=report(&q)["associations"][2].clone();assert_eq!(a["witness"]["kind"],"record-key");assert_eq!(a["witness"]["record"]["element"],"Ownership");assert_eq!(a["witness"]["key"]["fields"][0]["element"],"ownerId");assert_eq!(a["witness"]["record"],a["associationRecord"]);assert_ne!(a["witness"]["record"]["element"],a["reference"]["relationshipId"]);
}
#[test]
fn source_key_is_ontology_selected_not_primary_or_unknown_annotation_default(){
 let mut q=fixture();graph(&mut q,false);
 model(&mut q,|d|{let assignment=d["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="Assignment").unwrap();assignment["keys"][0]["primary"]=json!(true);assignment["keys"].as_array_mut().unwrap().push(json!({"id":"alt","name":"Alternate","fields":[{"module":"m","element":"assignmentProject"}],"primary":false}));d["modules"][0]["relationships"][0]["source"][0]["key"]=json!("pk");});
 ontology(&mut q,|o|{o["entities"].as_array_mut().unwrap().iter_mut().find(|e|e["type"]["elementId"]=="Assignment").unwrap()["keyId"]=json!("alt");o["associations"][1]["keyId"]=json!("alt");o["associations"][2]["endpoints"][0]["keyId"]=json!("alt");});
 let a=report(&q)["associations"][2].clone();assert_eq!(a["sources"][0]["metadata"]["key"],"pk");assert!(a["sources"][0].get("key").is_none());assert_eq!(a["endpoints"][0]["key"]["id"],"alt");assert_eq!(a["endpoints"][0]["key"]["fields"][0]["element"],"assignmentProject");
}
#[test]
fn unknown_annotations_direction_and_multiple_branches_are_metadata_not_admission(){
 let mut q=fixture();graph(&mut q,false);model(&mut q,|d|{let a=&mut d["modules"][0]["relationships"][0];a["directed"]=json!(false);a["source"].as_array_mut().unwrap().push(json!({"module":"m","element":"Ownership"}));a["target"].as_array_mut().unwrap().push(json!({"module":"m","element":"Resource","key":"pk"}));a["x-annotation"]=serde_json::from_str("{\"n\":1.000000000000000001,\"unicode\":\"雪\"}").unwrap();});ontology(&mut q,|o|o["associations"][2]["x-note"]=json!("annotation does not itself establish required unsupported meaning"));
 let value=report(&q);let a=&value["associations"][2];assert_eq!(a["authored"]["directed"],false);assert_eq!(a["sources"][2]["record"]["element"],"Ownership");assert_eq!(a["targets"][2]["record"]["element"],"Resource");assert_eq!(a["authored"]["x-annotation"]["n"].to_string(),"1.000000000000000001");assert_eq!(a["authored"]["x-annotation"]["unicode"],"雪");assert!(a["selection"].get("x-note").is_some());assert!(value["unresolved"][0].as_str().unwrap().contains("required-meaning verdict"));
}
#[test]
fn repeated_unadmitted_selections_are_not_silently_deduplicated(){
 let mut q=fixture();graph(&mut q,false);ontology(&mut q,|o|{let a=o["associations"][2].clone();o["associations"].as_array_mut().unwrap().push(a)});let v=report(&q);assert_eq!(v["associations"].as_array().unwrap().len(),4);assert_eq!(v["associations"][2]["reference"],v["associations"][3]["reference"]);assert_eq!(v["associations"][3]["ordinal"],3);assert_eq!(v["complete"],false);
}
#[test]
fn structural_endpoint_key_namespace_and_witness_breaks_refuse_without_defaults(){
 for case in 0..10{
  let mut q=fixture();graph(&mut q,false);
  if case<4{model(&mut q,|d|{let a=&mut d["modules"][0]["relationships"][0];match case{0=>{a["target"][0].as_object_mut().unwrap().remove("key");},1=>a["target"][0]["key"]=Value::Null,2=>a["target"][0]["key"]=json!("unknown"),_=>a["associationRecord"]=Value::Null}})}
  else{ontology(&mut q,|o|{let a=&mut o["associations"][2];match case{4=>a["endpoints"][1]["keyId"]=json!("other"),5=>a["endpoints"][1]["side"]=json!("source"),6=>a["relationship"]=r("Assignment"),7=>a["witness"]=json!({"kind":"record-key","type":r("Ownership"),"keyId":"pk"}),8=>a["endpoints"][0]["target"]=r("Staff"),_=>a["witness"]=json!({"kind":"future-unknown"})}})}
  refuses_at(&q,if matches!(case,0|1|3){"parent"}else{"graph"});
 }
 let mut q=fixture();graph(&mut q,true);ontology(&mut q,|o|o["associations"][2]["witness"]=json!({"kind":"opaque-existential"}));refuses(&q);
 let mut q=fixture();ontology(&mut q,|o|o["associations"][0]["endpoints"][0]["fields"][0]=r("salary"));refuses(&q);
}
#[test]
fn existing_model_report_bytes_remain_exact_against_actual039_native_baseline(){
 let raw=include_str!("../../../tests/current-query-presence/fixtures/unadmitted-context.request.json");assert_eq!(sha256(raw.as_bytes()),"2ebd655178b71bb49c48ff6997587cf51cd2a419a1793f95a24a6ec82d5dfe24");let original=crate::application_lineage::prepare(raw).unwrap();let context=crate::current_presence::prepare(&original).unwrap();let result=derive(&context).unwrap();assert_eq!(sha256(result.model_requirements().json().as_bytes()),"b5285598b2a6da07b6f6070adb468dd55ba496842a64f11b078eebe8fcf9924e");assert_eq!(result.model_requirements().record_requirements().len(),5);assert_eq!(result.json().len(),30_718);assert_eq!(sha256(result.json().as_bytes()),"fea5b5468a1d9e2e79d2a96dd1107777a6a56022dad892de4d4ebd92b24b870a");
}
#[test]
fn borrowed_metadata_amplification_has_whole_report_positive_and_refusal_controls(){
 for (count,expected) in [(14,true),(15,false)]{
  let mut q=fixture();graph(&mut q,false);model(&mut q,|d|d["modules"][0]["relationships"][0]["x-padding"]=json!("x".repeat(1_000_000)));ontology(&mut q,|o|{let a=o["associations"][2].clone();o["associations"].as_array_mut().unwrap().extend(std::iter::repeat_n(a,count-1))});let raw=q.to_string();assert!(raw.len()<4_000_000);let original=crate::application_lineage::prepare(&raw).unwrap();let context=crate::current_presence::prepare(&original).unwrap();let result=derive(&context);assert_eq!(result.is_ok(),expected,"{count}");if let Ok(issued)=result{assert!(issued.json().len()>15_000_000&&issued.json().len()<=LIMIT)}assert_eq!(original.source_request_json(),raw);
 }
}
#[test]
fn actual_scalar_model_report_above_eight_mib_is_owned_not_reparsed_by_input_scanner(){
 let raw=include_str!("../../../tests/current-query-presence/fixtures/model-requirements-scalar-capacity.request.json");assert_eq!(raw.len(),597_992);assert_eq!(sha256(raw.as_bytes()),"9979943c2cba3bbcdb297084906194fc4a58532600a582c3faa18a13737a1dc5");let original=crate::application_lineage::prepare(raw).unwrap();let context=crate::current_presence::prepare(&original).unwrap();let result=derive(&context).unwrap();assert_eq!(result.model_requirements().json().len(),9_833_820);assert_eq!(sha256(result.model_requirements().json().as_bytes()),"23d1451bd831aed015c63c6df054dce2ec4165543d115e806f7071d1c0aadea9");assert!(std::ptr::eq(result.model_requirements().context(),&context));let value:Value=serde_json::from_str(result.json()).unwrap();assert_eq!(value["modelRequirementsSha256"],"23d1451bd831aed015c63c6df054dce2ec4165543d115e806f7071d1c0aadea9");assert_eq!(value["associations"].as_array().unwrap().len(),2);assert_eq!(result.model_requirements().record_requirements().values().map(|r|r["fields"].as_array().unwrap().len()).sum::<usize>(),1062);
}
#[test]
fn cumulative_branch_match_payload_and_first_declared_count_failure_are_bounded(){
 for (count,expected) in [(4,true),(40,false)]{
  let mut q=fixture();graph(&mut q,false);model(&mut q,|d|{let a=&mut d["modules"][0]["relationships"][0];a["source"]=json!(vec![json!({"module":"m","element":"Assignment"});128]);a["target"]=json!(vec![json!({"module":"m","element":"Project","key":"pk"});128]);});
  let rev="r".repeat(1024);q["modules"][0]["pin"]["revision"]=json!(rev);let pin=q["modules"][0]["pin"].clone();profile(&mut q,|p|p["modelPins"][0]=pin);ontology(&mut q,|o|{o["documents"][0]["revision"]=json!(rev);let a=o["associations"][2].clone();o["associations"].as_array_mut().unwrap().extend(std::iter::repeat_n(a,count-1))});
  let original=crate::application_lineage::prepare(&q.to_string()).unwrap();let context=crate::current_presence::prepare(&original).unwrap();let result=derive(&context);assert_eq!(result.is_ok(),expected,"{count}");if let Ok(issued)=result{let v:Value=serde_json::from_str(issued.json()).unwrap();assert_eq!(v["associations"][2]["endpoints"][0]["declarationOrdinals"].as_array().unwrap().len(),128);assert_eq!(v["associations"][2]["endpoints"][1]["declarationOrdinals"].as_array().unwrap().len(),128);}
 }
 for (branches,expected) in [(4096,true),(4097,false)]{let mut q=fixture();graph(&mut q,false);model(&mut q,|d|d["modules"][0]["relationships"][0]["source"]=json!(vec![json!({"module":"m","element":"Assignment"});branches]));let original=crate::application_lineage::prepare(&q.to_string()).unwrap();let context=crate::current_presence::prepare(&original).unwrap();assert_eq!(derive(&context).is_ok(),expected,"{branches}");}
}

#[test]
fn encoded_byte_limit_has_exact_positive_and_first_overflow(){
 let allowed="x".repeat(LIMIT-2);assert_eq!(encode(&allowed).unwrap().len(),LIMIT);assert!(encode(&"x".repeat(LIMIT-1)).is_err());
}
#[test]
fn full_association_inventory_bound_preserves_occurrences_and_refuses_first_extra(){
 for (count,expected) in [(4096,true),(4097,false)]{
  let mut q=fixture();ontology(&mut q,|o|{let mut a=o["associations"][0].clone();a["endpoints"].as_array_mut().unwrap().truncate(1);o["associations"]=json!(vec![a;count]);});let original=crate::application_lineage::prepare(&q.to_string()).unwrap();let context=crate::current_presence::prepare(&original).unwrap();let result=derive(&context);assert_eq!(result.is_ok(),expected,"{count}");if let Ok(issued)=result{let v:Value=serde_json::from_str(issued.json()).unwrap();assert_eq!(v["associations"].as_array().unwrap().len(),4096);assert_eq!(v["associations"][4095]["ordinal"],4095);}
 }
}

// Exercise intermediate guards directly: a final report encoder must not mask
// a missing pre-retention budget check. These are retained-payload guarantees,
// not a bound on temporary allocations during original Key resolution.
#[test]
fn intermediate_detail_budget_refuses_before_retaining_match_ordinal() {
 let q=fixture();let raw=q.to_string();
 let original=crate::application_lineage::prepare(&raw).unwrap();
 let context=crate::current_presence::prepare(&original).unwrap();
 let model=current_presence_model_requirements::derive(&context).unwrap();
 let mut resolver=Resolver{catalog:original.catalog(),records:model.record_requirements(),keys:BTreeMap::new(),key_budget:PayloadBudget::result_phase(CODE),detail_budget:PayloadBudget::result_phase(CODE)};
 // A JSON string contributes its two quotes to the exact 16M budget.
 resolver.charge(&"x".repeat(LIMIT-2)).unwrap();
 let mut ordinals=Vec::new();
 assert!(resolver.matches(&mut ordinals,0).is_err());
 assert!(ordinals.is_empty(),"failed charge must not retain the ordinal");
}
#[test]
fn intermediate_key_budget_refuses_before_retaining_new_cache_entry() {
 let q=fixture();let raw=q.to_string();
 let original=crate::application_lineage::prepare(&raw).unwrap();
 let context=crate::current_presence::prepare(&original).unwrap();
 let model=current_presence_model_requirements::derive(&context).unwrap();
 let mut resolver=Resolver{catalog:original.catalog(),records:model.record_requirements(),keys:BTreeMap::new(),key_budget:PayloadBudget::result_phase(CODE),detail_budget:PayloadBudget::result_phase(CODE)};
 let reference=SecurityRef::read(&r("Resource")).unwrap();
 // Positive control proves this is a valid original Key, not a resolution refusal.
 let positive=resolver.key(&reference,"pk").unwrap();
 assert_eq!(positive.0.fields[0].element,"resourceId");
 resolver.keys.clear();resolver.key_budget=PayloadBudget::result_phase(CODE);
 resolver.key_budget.copy_json(&"x".repeat(LIMIT-2)).unwrap();
 assert!(resolver.key(&reference,"pk").is_err());
 assert!(resolver.keys.is_empty(),"failed charge must not retain the Key");
}

#[test]
fn unselected_declared_target_key_is_resolved_completely_without_entity_defaults() {
 for invalid_last_field in [false,true] {
  let mut q=fixture();graph(&mut q,false);
  model(&mut q,|d|{
   let elements=d["modules"][0]["elements"].as_array_mut().unwrap();
   let template=elements.iter().find(|e|e["id"]=="projectId").unwrap().clone();
   let mut members=Vec::new();
   for ordinal in 0..256 {
    let id=format!("unselectedKey{ordinal}");let mut field=template.clone();
    field["id"]=json!(id);field["name"]=json!(id);
    if invalid_last_field&&ordinal==255{field["nullability"]=json!("absent-allowed")}
    elements.push(field);members.push(json!({"module":"m","element":id}));
   }
   elements.push(json!({"id":"UnselectedTarget","name":"UnselectedTarget","kind":"record","members":members,"keys":[{"id":"authoredComposite","name":"Authored composite","fields":members,"primary":false}],"extensions":{}}));
   d["modules"][0]["relationships"][0]["target"].as_array_mut().unwrap().push(json!({"module":"m","element":"UnselectedTarget","key":"authoredComposite"}));
  });
  let revision="r".repeat(4096);q["modules"][0]["pin"]["revision"]=json!(revision);
  let pin=q["modules"][0]["pin"].clone();profile(&mut q,|p|p["modelPins"][0]=pin);
  ontology(&mut q,|o|o["documents"][0]["revision"]=json!(revision));
  let raw=q.to_string();assert!(raw.len()<4_000_000);
  let original=crate::application_lineage::prepare(&raw).unwrap();
  let context=crate::current_presence::prepare(&original).unwrap();
  // This extra authored branch is deliberately absent from selected entities.
  let model=current_presence_model_requirements::derive(&context).unwrap();
  assert_eq!(model.record_requirements().len(),5);
  let issued=derive(&context);
  if invalid_last_field {assert!(issued.is_err(),"unselected authored Key must still be validated");continue}
  let issued=issued.unwrap();let value:Value=serde_json::from_str(issued.json()).unwrap();
  let branch=&value["associations"][2]["targets"][2];
  assert_eq!(branch["record"]["element"],"UnselectedTarget");
  assert_eq!(branch["key"]["id"],"authoredComposite");
  let fields=branch["key"]["fields"].as_array().unwrap();assert_eq!(fields.len(),256);
  for (ordinal,field) in fields.iter().enumerate(){assert_eq!(field["element"],format!("unselectedKey{ordinal}"));assert_eq!(field["revision"],revision)}
  assert!(serde_json::to_vec(&branch["key"]).unwrap().len()>1_000_000);
  assert_eq!(value["associations"][2]["endpoints"][1]["declarationOrdinals"],json!([0,1]));
  assert_eq!(value["sourceRequestJson"],raw);assert_eq!(value["authority"],false);
 }
}

#[test]
fn intermediate_output_budget_is_inclusive_and_refuses_before_retention() {
 let payload="x".repeat(LIMIT-2);let mut entries=Vec::new();
 let mut budget=PayloadBudget::result_phase(CODE);
 retain(&mut budget,&mut entries,payload.as_str()).unwrap();
 assert_eq!(entries.len(),1);assert_eq!(entries[0],payload);
 assert!(retain(&mut budget,&mut entries,"").is_err());
 assert_eq!(entries.len(),1,"refused entry must not be retained");
 assert_eq!(entries[0],payload,"previous retained entries must remain unchanged");
}
