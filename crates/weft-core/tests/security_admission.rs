use serde_json::{json,Value};
use weft_core::{compile::Compiler,json::sha256};
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/security-compile-response-v0.1.schema.json")]
struct SecurityResponse;
// Shared security goal: CONTRACT-006 source custody; native acceptance remains open.
fn request()->Value {
    let cases:Value=serde_json::from_str(include_str!("../../../docs/helix/03-test/fixtures/cases.json")).unwrap();
    let mut request=cases.as_array().unwrap().iter().find(|c|c["expected"]["status"]=="compiled").unwrap()["request"].clone();
    request["interfaceVersion"]=json!("weft-security-compile/0.1.0");request["dialect"]=json!("weft-sql/0.2.0");
    for module in request["modules"].as_array_mut().unwrap(){
        let mut doc:Value=serde_json::from_str(module["documentJson"].as_str().unwrap()).unwrap();doc["umf"]=json!("0.8.0");
        let text=doc.to_string();module["documentJson"]=json!(text);module["pin"]["umfVersion"]=json!("0.8.0");module["pin"]["sha256"]=json!(sha256(text.as_bytes()));
    }
    let fixture:Value=serde_json::from_str(include_str!("security-source-fixture.json")).unwrap();
    let doc=&fixture["resolution"]["documents"][0];let text=doc["document"].to_string();
    request["modules"]=json!([{"documentJson":text,"pin":{"documentId":doc["document"]["id"],"revision":doc["revision"],"umfVersion":"0.8.0","sha256":sha256(text.as_bytes())},"selectedModuleIds":["m"]}]);
    request["security"]=json!({"version":"umf.security/0.1.0","policyJson":fixture["policy"].to_string(),"ontologyJson":fixture["resolution"]["ontology"].to_string()});request
}
fn response(request:&Value)->Value{serde_json::from_str(&Compiler::default().compile_json(&request.to_string())).unwrap()}
fn refused(response:Value,code:&str){if response["interfaceVersion"]=="weft-security-compile/0.1.0"{assert!(SecurityResponse::is_valid(&response));}assert_eq!(response["status"],"blocked");assert_eq!(response["diagnostics"][0]["code"],code);for key in ["sql","parameters","logicalPlan"]{assert!(response.get(key).is_none());}}
#[test]
fn core08_source_custody_never_activates_uninterpreted_security(){let r=response(&request());assert_eq!(r["interfaceVersion"],"weft-security-compile/0.1.0");refused(r,"WFT-SECURITY-UNSUPPORTED");}
#[test]
fn missing_security_and_prior_core_pin_do_not_downgrade(){let mut r=request();r.as_object_mut().unwrap().remove("security");refused(response(&r),"WFT-INPUT");let mut r=request();r["modules"][0]["pin"]["umfVersion"]=json!("0.7.0");refused(response(&r),"WFT-INPUT");}
#[test]
fn invalid_source_digest_refuses_before_security_gate(){let mut r=request();r["modules"][0]["pin"]["sha256"]=json!("0".repeat(64));refused(response(&r),"WFT-PIN");}
#[test]
fn prior_transport_cannot_carry_security_or_core08(){let mut r=request();r["interfaceVersion"]=json!("weft-compile/0.2.0");refused(response(&r),"WFT-INPUT");}

#[test]
fn unsupported_security_does_not_invoke_backend_factory(){
    let mut called=false;
    let raw=Compiler::default().compile_json_with_factory(&request().to_string(),&mut |_,_,_|{called=true;panic!("security activated backend composition")});
    assert!(!called);refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-UNSUPPORTED");
}

#[test]
fn source_unknown_meaning_and_stale_ontology_refuse(){
 let mut r=request();let mut policy:Value=serde_json::from_str(r["security"]["policyJson"].as_str().unwrap()).unwrap();policy["future"]=json!(true);r["security"]["policyJson"]=json!(policy.to_string());refused(response(&r),"WFT-SECURITY-SOURCE");
 let mut r=request();let mut ontology:Value=serde_json::from_str(r["security"]["ontologyJson"].as_str().unwrap()).unwrap();ontology["documents"][0]["revision"]=json!("other");r["security"]["ontologyJson"]=json!(ontology.to_string());refused(response(&r),"WFT-SECURITY-PIN");
}
#[test]
fn opaque_native_archive_is_retained_without_authorizing(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecuritySourcePacket};
 let r=request();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();
 let mut policy:Value=serde_json::from_str(r["security"]["policyJson"].as_str().unwrap()).unwrap();policy["native"]=json!({"future":{"trusted":true,"role":"admin"}});let raw=policy.to_string();
 let packet=SecuritySourcePacket::read(&raw,r["security"]["ontologyJson"].as_str().unwrap(),&catalog).unwrap();assert_eq!(packet.policy_json(),raw);assert_eq!(packet.policy()["native"],policy["native"]);
}

#[test]
fn duplicate_json_and_deep_expressions_refuse_before_lowering(){
 let mut r=request();r["security"]["policyJson"]=json!("{\"vocabulary\":\"umf.security\",\"vocabulary\":\"umf.security\"}");refused(response(&r),"WFT-SECURITY-SOURCE");
 let mut r=request();let mut policy:Value=serde_json::from_str(r["security"]["policyJson"].as_str().unwrap()).unwrap();let mut expr=json!({"op":"literal","value":true});for _ in 0..16{expr=json!({"op":"not","arg":expr});}policy["rules"][0]["condition"]=expr;r["security"]["policyJson"]=json!(policy.to_string());refused(response(&r),"WFT-SECURITY-LIMIT");
}
#[test]
fn nested_unknown_meaning_and_missing_document_closure_refuse(){
 let mut r=request();let mut policy:Value=serde_json::from_str(r["security"]["policyJson"].as_str().unwrap()).unwrap();policy["rules"][0]["condition"]["future"]=json!(true);r["security"]["policyJson"]=json!(policy.to_string());refused(response(&r),"WFT-SECURITY-SOURCE");
 let mut r=request();let mut ontology:Value=serde_json::from_str(r["security"]["ontologyJson"].as_str().unwrap()).unwrap();ontology["documents"][0]["documentId"]=json!("other");r["security"]["ontologyJson"]=json!(ontology.to_string());refused(response(&r),"WFT-SECURITY-PIN");
}

#[test]
fn incomplete_classification_and_wrong_endpoint_type_refuse(){
 let mut r=request();let mut o:Value=serde_json::from_str(r["security"]["ontologyJson"].as_str().unwrap()).unwrap();o["entities"][2]["fields"].as_array_mut().unwrap().pop();r["security"]["ontologyJson"]=json!(o.to_string());refused(response(&r),"WFT-SECURITY-ONTOLOGY");
 let mut r=request();let mut o:Value=serde_json::from_str(r["security"]["ontologyJson"].as_str().unwrap()).unwrap();o["associations"][0]["endpoints"][0]["target"]["elementId"]=json!("salary");r["security"]["ontologyJson"]=json!(o.to_string());refused(response(&r),"WFT-SECURITY-ONTOLOGY");
}
#[test]
fn unknown_core_domain_and_invalid_key_refuse(){
 let mut r=request();let mut d:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();let field=d["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="salary").unwrap();field["facets"]=json!({"future":true});let text=d.to_string();r["modules"][0]["documentJson"]=json!(text);r["modules"][0]["pin"]["sha256"]=json!(sha256(text.as_bytes()));refused(response(&r),"WFT-SECURITY-ONTOLOGY");
 let mut r=request();let mut o:Value=serde_json::from_str(r["security"]["ontologyJson"].as_str().unwrap()).unwrap();o["entities"][0]["keyId"]=json!("missing");r["security"]["ontologyJson"]=json!(o.to_string());refused(response(&r),"WFT-SECURITY-ONTOLOGY");
}

#[test]
fn correlated_fixture_has_exact_ontology_closure(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecuritySourcePacket,security_ontology::SecurityOntologyClosure};
 let r=request();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();
 let packet=SecuritySourcePacket::read(r["security"]["policyJson"].as_str().unwrap(),r["security"]["ontologyJson"].as_str().unwrap(),&catalog).unwrap();
 let closure=SecurityOntologyClosure::read(&packet,&catalog).unwrap();assert_eq!(closure.type_count(),5);assert_eq!(closure.field_count(),11);assert_eq!(closure.subject().document_id,"domain");assert_eq!(closure.subject().element_id,"Staff");
}
#[test]
fn duplicate_types_wrong_document_endpoint_arity_and_domain_refuse(){
 for mutation in 0..4{
  let mut r=request();let mut o:Value=serde_json::from_str(r["security"]["ontologyJson"].as_str().unwrap()).unwrap();
  match mutation{
   0=>{let duplicate=o["entities"][0].clone();o["entities"].as_array_mut().unwrap().push(duplicate);},
   1=>o["entities"][0]["fields"][0]["ref"]["documentId"]=json!("other"),
   2=>{o["associations"][0]["endpoints"][0]["fields"].as_array_mut().unwrap().clear();},
   3=>o["associations"][1]["endpoints"][0]["fields"][0]["elementId"]=json!("active"),_=>unreachable!()
  }
  r["security"]["ontologyJson"]=json!(o.to_string());let actual=response(&r);assert!(actual["status"]=="blocked");assert!(matches!(actual["diagnostics"][0]["code"].as_str(),Some("WFT-SECURITY-ONTOLOGY"|"WFT-SECURITY-SOURCE")));assert!(actual.get("sql").is_none());
 }
}
#[test]
fn same_local_ids_in_distinct_documents_do_not_merge(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecuritySourcePacket,security_ontology::SecurityOntologyClosure};
 let mut r=request();let mut doc:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();doc["id"]=json!("other");let text=doc.to_string();let mut module=r["modules"][0].clone();module["documentJson"]=json!(text);module["pin"]["documentId"]=json!("other");module["pin"]["sha256"]=json!(sha256(text.as_bytes()));r["modules"].as_array_mut().unwrap().push(module);
 let mut ontology:Value=serde_json::from_str(r["security"]["ontologyJson"].as_str().unwrap()).unwrap();let mut pin=ontology["documents"][0].clone();pin["documentId"]=json!("other");ontology["documents"].as_array_mut().unwrap().push(pin);
 let mut entity=ontology["entities"][0].clone();entity["type"]["documentId"]=json!("other");entity["fields"][0]["ref"]["documentId"]=json!("other");ontology["entities"].as_array_mut().unwrap().push(entity);
 let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let packet=SecuritySourcePacket::read(r["security"]["policyJson"].as_str().unwrap(),&ontology.to_string(),&catalog).unwrap();let closure=SecurityOntologyClosure::read(&packet,&catalog).unwrap();assert_eq!(closure.type_count(),6);assert_eq!(closure.field_count(),12);assert_eq!(closure.subject().document_id,"domain");
}

#[test]
fn unknown_length_unit_and_inapplicable_scalar_facet_refuse(){
 for facet in [json!({"length":{"min":0,"unit":"future"}}),json!({"collectionSize":{"min":0}})]{
  let mut r=request();let mut d:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();let f=d["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|f|f["id"]=="staffId").unwrap();f["facets"]=facet;let text=d.to_string();r["modules"][0]["documentJson"]=json!(text);r["modules"][0]["pin"]["sha256"]=json!(sha256(text.as_bytes()));refused(response(&r),"WFT-SECURITY-ONTOLOGY");
 }
}

#[test]
fn policy_variable_scope_identity_and_actions_refuse(){
 for mutation in 0..4{
  let mut r=request();let mut p:Value=serde_json::from_str(r["security"]["policyJson"].as_str().unwrap()).unwrap();
  match mutation{
   0=>p["rules"][1]["condition"]["where"]["args"][1]["as"]=json!("o"),
   1=>p["rules"][1]["condition"]["where"]["args"][0]["left"]["name"]=json!("unbound"),
   2=>p["rules"][1]["condition"]["where"]["args"][0]["left"]["endpoint"]=json!("project"),
   3=>p["rules"][0]["actions"]=json!(["unknown"]),_=>unreachable!()
  }
  r["security"]["policyJson"]=json!(p.to_string());refused(response(&r),"WFT-SECURITY-TYPE");
 }
}
#[test]
fn target_membership_and_literal_wrapper_refuse(){
 let mut r=request();let mut p:Value=serde_json::from_str(r["security"]["policyJson"].as_str().unwrap()).unwrap();p["rules"][0]["disclosure"][0]["field"]["elementId"]=json!("staffId");r["security"]["policyJson"]=json!(p.to_string());refused(response(&r),"WFT-SECURITY-TYPE");
 let mut r=request();let mut p:Value=serde_json::from_str(r["security"]["policyJson"].as_str().unwrap()).unwrap();p["rules"][1]["condition"]["where"]["args"][1]["where"]["args"][2]["right"]["value"]=json!({"string":"true"});r["security"]["policyJson"]=json!(p.to_string());refused(response(&r),"WFT-SECURITY-TYPE");
}

#[test]
fn correlated_declared_term_types_are_checked_without_activation(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecuritySourcePacket,security_policy_types::SecurityPolicyTypes};
 let r=request();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let packet=SecuritySourcePacket::read(r["security"]["policyJson"].as_str().unwrap(),r["security"]["ontologyJson"].as_str().unwrap(),&catalog).unwrap();let types=SecurityPolicyTypes::read(&packet,&catalog).unwrap();assert_eq!(types.checked_rules(),2);assert_eq!(types.checked_nodes(),9);
}
#[test]
fn source_packet_cannot_be_reused_with_changed_model_bytes(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecuritySourcePacket,security_policy_types::SecurityPolicyTypes};
 let r=request();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let mut catalog=Catalog::prepare_security(inputs).unwrap();let packet=SecuritySourcePacket::read(r["security"]["policyJson"].as_str().unwrap(),r["security"]["ontologyJson"].as_str().unwrap(),&catalog).unwrap();catalog.inputs[0].document_json.push(' ');assert_eq!(SecurityPolicyTypes::read(&packet,&catalog).unwrap_err().code,"WFT-SECURITY-PIN");
}
#[test]
fn scalar_operand_domains_and_endpoint_roles_refuse(){
 for mutation in 0..2{
  let mut r=request();let mut p:Value=serde_json::from_str(r["security"]["policyJson"].as_str().unwrap()).unwrap();
  if mutation==0{p["rules"][1]["condition"]["where"]["args"][1]["where"]["args"][2]["right"]=json!({"kind":"resource","field":{"documentId":"domain","moduleId":"m","elementId":"salary"}});}else{p["rules"][1]["condition"]["where"]["args"][0]["left"]["endpoint"]=json!("missing");}
  r["security"]["policyJson"]=json!(p.to_string());refused(response(&r),"WFT-SECURITY-TYPE");
 }
}

#[test]
fn rehashed_catalog_input_cannot_substitute_for_prepared_definitions(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecuritySourcePacket};
 let r=request();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let mut catalog=Catalog::prepare_security(inputs).unwrap();let mut doc:Value=serde_json::from_str(&catalog.inputs[0].document_json).unwrap();doc["modules"][0]["elements"][0]["description"]=json!("substituted source");catalog.inputs[0].document_json=doc.to_string();catalog.inputs[0].pin.sha256=sha256(catalog.inputs[0].document_json.as_bytes());let error=SecuritySourcePacket::read(r["security"]["policyJson"].as_str().unwrap(),r["security"]["ontologyJson"].as_str().unwrap(),&catalog).unwrap_err();assert_eq!(error.code,"WFT-SECURITY-PIN");
}

#[test]
fn admitted_ir_retains_correlated_binding_and_qualified_domains(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecuritySourcePacket,security_ir::{SecurityLogicalPlan,Expression,Term,Binding,Effect,Disposition}};
 let r=request();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();
 let raw=r["security"]["policyJson"].as_str().unwrap();let packet=SecuritySourcePacket::read(raw,r["security"]["ontologyJson"].as_str().unwrap(),&catalog).unwrap();
 let plan=SecurityLogicalPlan::read(packet,&catalog).unwrap();assert_eq!(plan.source().policy_json(),raw);assert_eq!(plan.rules().len(),2);
 assert_eq!(plan.rules()[0].effect,Effect::Permit);assert_eq!(plan.rules()[0].disclosure[0].1,Disposition::Withheld);
 let Expression::Exists{slot:0,association,condition}=&plan.rules()[1].condition else{panic!("missing ownership correlation")};assert_eq!(association.record().unwrap().element_id,"Ownership");
 let Expression::And(args)=condition.as_ref() else{panic!()};
 let Expression::Equal(Term::Endpoint{binding:Binding::Variable(0),target,key_id,..},Term::Identity{binding:Binding::Resource,..})=&args[0] else{panic!("missing resource identity")};assert_eq!(target.element_id,"Resource");assert_eq!(key_id,"pk");
 let Expression::Exists{slot:1,condition,..}=&args[1] else{panic!("missing assignment correlation")};let Expression::And(args)=condition.as_ref() else{panic!()};
 let Expression::Equal(Term::Endpoint{binding:Binding::Variable(1),target:left,..},Term::Endpoint{binding:Binding::Variable(0),target:right,..})=&args[1] else{panic!("lost outer binding")};assert_eq!(left,right);assert_eq!(left.element_id,"Project");
 let Expression::Equal(Term::Field{domain:left,..},Term::Constant{domain:right,literal,..})=&args[2] else{panic!()};assert_eq!(left,right);assert_eq!(*literal,json!({"boolean":true}));
 assert!(plan.require_catalog(&catalog).is_ok());
}
#[test]
fn ir_admission_cannot_skip_type_checking(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecuritySourcePacket,security_ir::SecurityLogicalPlan};
 let r=request();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();
 let mut policy:Value=serde_json::from_str(r["security"]["policyJson"].as_str().unwrap()).unwrap();policy["rules"][1]["condition"]["where"]["args"][0]["left"]["name"]=json!("unbound");
 let packet=SecuritySourcePacket::read(&policy.to_string(),r["security"]["ontologyJson"].as_str().unwrap(),&catalog).unwrap();assert!(SecurityLogicalPlan::read(packet,&catalog).is_err());
}

fn query_profile_request()->Value{
 let mut r=request();let mut doc:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();for element in doc["modules"][0]["elements"].as_array_mut().unwrap(){element["name"]=element["id"].clone();if element["scalarType"]=="integer"{element["facets"]=json!({"integerWidth":{"bits":64,"signed":true}});}}let text=doc.to_string();r["modules"][0]["documentJson"]=json!(text);r["modules"][0]["pin"]["sha256"]=json!(sha256(text.as_bytes()));r["sql"]=json!("SELECT r.resourceId FROM Resource r");let p=json!({"version":"weft.security.query-profile/0.1.0","id":"compile-profile-fixture","revision":"p1","action":"read","modelPins":r["modules"].as_array().unwrap().iter().map(|m|m["pin"].clone()).collect::<Vec<_>>(),"policySha256":sha256(r["security"]["policyJson"].as_str().unwrap().as_bytes()),"ontologySha256":sha256(r["security"]["ontologyJson"].as_str().unwrap().as_bytes()),"binding":{"backendId":r["target"]["backendId"],"backendVersion":r["target"]["backendVersion"],"targetProfile":r["target"]["targetProfile"],"sha256":r["target"]["bindingSha256"]},"targets":[{"documentId":"domain","moduleId":"m","elementId":"Resource"}],"bindings":[]});r["security"]["queryProfileJson"]=json!(p.to_string());r
}
#[test]
fn admitted_query_profile_never_activates_backend_before_native_lowering(){
 let r=query_profile_request();let mut called=false;let raw=Compiler::default().compile_json_with_factory(&r.to_string(),&mut |_,_,_|{called=true;panic!("profile activated backend")});assert!(!called);refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-UNSUPPORTED");
}
#[test]
fn stale_query_profile_source_refuses_before_backend_factory(){
 let mut r=query_profile_request();let mut p:Value=serde_json::from_str(r["security"]["queryProfileJson"].as_str().unwrap()).unwrap();p["ontologySha256"]=json!("0".repeat(64));r["security"]["queryProfileJson"]=json!(p.to_string());let mut called=false;let raw=Compiler::default().compile_json_with_factory(&r.to_string(),&mut |_,_,_|{called=true;panic!("stale profile activated backend")});assert!(!called);refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-QUERY-PROFILE");
}

#[test]
fn actual_sql_protected_predicate_refuses_without_backend_factory(){
 let mut r=query_profile_request();r["sql"]=json!("SELECT r.resourceId FROM Resource r WHERE r.salary=100");let mut called=false;let raw=Compiler::default().compile_json_with_factory(&r.to_string(),&mut |_,_,_|{called=true;panic!("prohibited SQL use activated backend")});assert!(!called);refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-QUERY-PROFILE");
}
#[test]
fn count_cannot_escape_profile_target_admission_by_having_no_field_uses(){
 let mut r=query_profile_request();r["sql"]=json!("SELECT COUNT(*) FROM Staff s");refused(response(&r),"WFT-SECURITY-QUERY-PROFILE");
}


#[test]
fn original_boolean_primary_metadata_preserves_explicit_key_selection_and_refusal_gate(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecuritySourcePacket,security_ontology::SecurityOntologyClosure};
 for primary in [true,false]{
  let mut r=request();let mut document:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
  for record in document["modules"][0]["elements"].as_array_mut().unwrap(){if record["kind"]=="record"{record["keys"][0]["primary"]=json!(primary);}}
  let text=document.to_string();r["modules"][0]["documentJson"]=json!(text);r["modules"][0]["pin"]["sha256"]=json!(sha256(text.as_bytes()));
  let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let packet=SecuritySourcePacket::read(r["security"]["policyJson"].as_str().unwrap(),r["security"]["ontologyJson"].as_str().unwrap(),&catalog).unwrap();let closure=SecurityOntologyClosure::read(&packet,&catalog).unwrap();assert_eq!(closure.type_count(),5);assert_eq!(closure.field_count(),11);assert_eq!(catalog.inputs[0].document_json,text);assert_eq!(serde_json::from_str::<Value>(&catalog.inputs[0].document_json).unwrap(),document);
  refused(response(&r),"WFT-SECURITY-UNSUPPORTED");
 }
 let mut r=request();let mut document:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();let staff=document["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="Resource").unwrap();let mut secondary=staff["keys"][0].clone();secondary["id"]=json!("secondary");secondary["name"]=json!("Secondary");secondary["primary"]=json!(true);secondary["fields"]=json!([{"module":"m","element":"salary"}]);staff["keys"][0]["primary"]=json!(false);staff["keys"].as_array_mut().unwrap().push(secondary.clone());
 let text=document.to_string();r["modules"][0]["documentJson"]=json!(text);r["modules"][0]["pin"]["sha256"]=json!(sha256(text.as_bytes()));refused(response(&r),"WFT-SECURITY-UNSUPPORTED");
 {use weft_core::security_ir::{SecurityLogicalPlan,Expression,Term,Binding};let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let packet=SecuritySourcePacket::read(r["security"]["policyJson"].as_str().unwrap(),r["security"]["ontologyJson"].as_str().unwrap(),&catalog).unwrap();let plan=SecurityLogicalPlan::read(packet,&catalog).unwrap();let Expression::Exists{condition,..}=&plan.rules()[1].condition else{panic!()};let Expression::And(args)=condition.as_ref() else{panic!()};let Expression::Equal(Term::Endpoint{key_id:endpoint,..},Term::Identity{binding:Binding::Resource,key_id:resource,..})=&args[0] else{panic!()};assert_eq!(endpoint,"pk");assert_eq!(resource,"pk");}
 for key_index in [0,1]{for carrier in [Value::Null,json!("true"),json!(1)]{let mut changed=document.clone();changed["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="Resource").unwrap()["keys"][key_index]["primary"]=carrier;let mut invalid=r.clone();let text=changed.to_string();invalid["modules"][0]["documentJson"]=json!(text);invalid["modules"][0]["pin"]["sha256"]=json!(sha256(text.as_bytes()));refused(response(&invalid),"WFT-MODEL");}}
 document["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="Resource").unwrap()["keys"][0]["primary"]=json!(true);let text=document.to_string();r["modules"][0]["documentJson"]=json!(text);r["modules"][0]["pin"]["sha256"]=json!(sha256(text.as_bytes()));refused(response(&r),"WFT-SECURITY-ONTOLOGY");
}


fn candidate_sources()->(Value,Value,Value){
 let r=request();let mut policy:Value=serde_json::from_str(r["security"]["policyJson"].as_str().unwrap()).unwrap();let mut ontology:Value=serde_json::from_str(r["security"]["ontologyJson"].as_str().unwrap()).unwrap();
 policy["version"]=json!("0.2.0");policy["revision"]=json!("policy-draft2");policy["ontology"]["revision"]=json!("ontology-draft2");ontology["version"]=json!("0.2.0");ontology["revision"]=json!("ontology-draft2");
 let original=ontology["associations"].as_array().unwrap().clone();
 for association in &original{let mut entity=association.clone();entity.as_object_mut().unwrap().remove("endpoints");ontology["entities"].as_array_mut().unwrap().push(entity);}
 ontology["associations"]=Value::Array(original.iter().map(|a|json!({"kind":"record-members","type":a["type"],"keyId":a["keyId"],"endpoints":a["endpoints"]})).collect());
 (r,policy,ontology)
}
#[test]
fn draft_source_custody_preserves_bytes_and_cannot_downgrade_into_admitted_packet(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::{SecuritySourcePacket,SecurityCandidateSourcePacket}};
 let (r,mut policy,ontology)=candidate_sources();policy["native"]=json!({"future":{"opaque":[1,true,"kept"]}});
 let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();
 let p=format!("\n{}\n",serde_json::to_string_pretty(&policy).unwrap());let o=ontology.to_string();let packet=SecurityCandidateSourcePacket::read(&p,&o,&catalog).unwrap();
 assert_eq!(packet.policy_json(),p);assert_eq!(packet.ontology_json(),o);assert_eq!(packet.policy(),&policy);assert_eq!(packet.ontology(),&ontology);assert!(packet.require_catalog(&catalog).is_ok());
 assert_eq!(SecuritySourcePacket::read(&p,&o,&catalog).unwrap_err().code,"WFT-SECURITY-SOURCE");
 for (p,o) in [(r["security"]["policyJson"].as_str().unwrap().to_string(),o.clone()),(p.clone(),r["security"]["ontologyJson"].as_str().unwrap().to_string())]{assert_eq!(SecurityCandidateSourcePacket::read(&p,&o,&catalog).unwrap_err().code,"WFT-SECURITY-SOURCE");}
}
#[test]
fn draft_source_custody_refuses_unknown_semantics_pin_mismatch_and_duplicate_keys(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket};
 let (r,policy,ontology)=candidate_sources();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();
 for mutation in ["unknown","version","mixed-ref","pin","duplicate-rule"]{
  let mut p=policy.clone();let mut o=ontology.clone();
  match mutation{"unknown"=>o["associations"][0]["future"]=json!(true),"version"=>p["version"]=json!("0.3.0"),"mixed-ref"=>p["rules"][1]["condition"]["association"]["relationshipId"]=json!("Ownership"),"pin"=>o["documents"][0]["revision"]=json!("stale"),_=>{let rule=p["rules"][0].clone();p["rules"].as_array_mut().unwrap().push(rule);}}
  let error=SecurityCandidateSourcePacket::read(&p.to_string(),&o.to_string(),&catalog).unwrap_err();assert_eq!(error.code,if mutation=="pin"{"WFT-SECURITY-PIN"}else{"WFT-SECURITY-SOURCE"});
 }
 let duplicate=policy.to_string().replacen("\"version\":\"0.2.0\"","\"version\":\"0.2.0\",\"version\":\"0.2.0\"",1);assert_ne!(duplicate,policy.to_string());assert_eq!(SecurityCandidateSourcePacket::read(&duplicate,&ontology.to_string(),&catalog).unwrap_err().code,"WFT-SECURITY-SOURCE");
}
#[test]
fn draft_graph_selector_is_shape_custody_only_and_expression_bounds_still_apply(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket};
 let (r,mut policy,mut ontology)=candidate_sources();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();
 let relationship=json!({"documentId":"domain","moduleId":"m","relationshipId":"WorksOn"});ontology["associations"][1]=json!({"kind":"core-relationship","relationship":relationship,"witness":{"kind":"opaque-existential"},"endpoints":[{"role":"staff","side":"source","target":{"documentId":"domain","moduleId":"m","elementId":"Staff"},"keyId":"pk"},{"role":"project","side":"target","target":{"documentId":"domain","moduleId":"m","elementId":"Project"},"keyId":"pk"}]});policy["rules"][1]["condition"]["where"]["args"][1]["association"]=relationship;
 // Relationship presence and opaque attribute admissibility belong to later closure.
 assert!(SecurityCandidateSourcePacket::read(&policy.to_string(),&ontology.to_string(),&catalog).is_ok());
 let mut condition=json!({"op":"literal","value":true});for _ in 0..16{condition=json!({"op":"not","arg":condition});}policy["rules"][0]["condition"]=condition;assert_eq!(SecurityCandidateSourcePacket::read(&policy.to_string(),&ontology.to_string(),&catalog).unwrap_err().code,"WFT-SECURITY-LIMIT");
}

#[test]
fn draft_selected_ontology_closes_raw_members_and_refuses_semantic_substitutions(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket,security_candidate_ontology::SecurityCandidateOntologyClosure};
 let (r,policy,ontology)=candidate_sources();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();
 let close=|o:&Value|{let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&o.to_string(),&catalog).unwrap();SecurityCandidateOntologyClosure::read(&packet,&catalog)};
 let c=close(&ontology).unwrap();assert_eq!(c.type_count(),5);assert_eq!(c.association_count(),2);assert_eq!(c.subject().element_id,"Staff");
 for mutation in ["key","owner","target","role","context","action","duplicate"]{let mut o=ontology.clone();match mutation{
  "key"=>o["associations"][0]["keyId"]=json!("other"),
  "owner"=>o["associations"][0]["endpoints"][0]["fields"][0]["elementId"]=json!("StaffId"),
  "target"=>o["associations"][0]["endpoints"][0]["target"]["elementId"]=json!("Missing"),
  "role"=>{let role=o["associations"][0]["endpoints"][0]["role"].clone();o["associations"][0]["endpoints"][1]["role"]=role;},
  "context"=>o["context"].as_array_mut().unwrap().push(json!({"documentId":"domain","moduleId":"m","elementId":"Missing"})),
  "action"=>{let a=o["actions"][0].clone();o["actions"].as_array_mut().unwrap().push(a);},
  _=>{let a=o["associations"][0].clone();o["associations"].as_array_mut().unwrap().push(a);}}
  assert!(close(&o).is_err(),"{mutation}");
 }
}
#[test]
fn draft_graph_selected_closure_requires_actual_direction_target_key_and_witness_owner(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket,security_candidate_ontology::SecurityCandidateOntologyClosure};
 let (mut r,policy,mut ontology)=candidate_sources();let relationship=json!({"documentId":"domain","moduleId":"m","relationshipId":"WorksOn"});
 ontology["associations"][1]=json!({"kind":"core-relationship","relationship":relationship,"witness":{"kind":"record-key","type":{"documentId":"domain","moduleId":"m","elementId":"Assignment"},"keyId":"pk"},"endpoints":[{"role":"staff","side":"source","target":{"documentId":"domain","moduleId":"m","elementId":"Staff"},"keyId":"pk"},{"role":"project","side":"target","target":{"documentId":"domain","moduleId":"m","elementId":"Project"},"keyId":"pk"}]});
 let mut doc:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
 doc["modules"][0]["relationships"]=json!([{"id":"WorksOn","name":"WorksOn","source":[{"module":"m","element":"Staff"}],"target":[{"module":"m","element":"Project","key":"pk"}],"sourceMultiplicity":{"min":0,"max":"*"},"targetMultiplicity":{"min":0,"max":2},"targetLifecycle":"independent","directed":true,"associationRecord":{"module":"m","element":"Assignment"}}]);
 for mutation in ["valid","opaque","integer-spelling","inverted-bounds","lifecycle","absent","direction","target-key","witness-owner","qualifier","duplicate","side","role","opaque-owner"]{let mut d=doc.clone();let mut o=ontology.clone();match mutation{
  "integer-spelling"=>d["modules"][0]["relationships"][0]["targetMultiplicity"]=serde_json::from_str("{\"min\":0.0,\"max\":2e0}").unwrap(),
  "lifecycle"=>d["modules"][0]["relationships"][0]["targetLifecycle"]=json!("future-magic"),
  "inverted-bounds"=>d["modules"][0]["relationships"][0]["targetMultiplicity"]=serde_json::from_str("{\"min\":3e0,\"max\":2.0}").unwrap(),
  "opaque"=>{d["modules"][0]["relationships"][0].as_object_mut().unwrap().remove("associationRecord");o["associations"][1]["witness"]=json!({"kind":"opaque-existential"});},
  "opaque-owner"=>o["associations"][1]["witness"]=json!({"kind":"opaque-existential"}),
  "side"=>o["associations"][1]["endpoints"][0]["side"]=json!("target"),
  "role"=>o["associations"][1]["endpoints"][0]["role"]=json!("project"),
  "absent"=>d["modules"][0]["relationships"]=json!([]),"direction"=>d["modules"][0]["relationships"][0]["directed"]=json!(false),
  "target-key"=>d["modules"][0]["relationships"][0]["target"][0]["key"]=json!("other"),"witness-owner"=>d["modules"][0]["relationships"][0]["associationRecord"]["element"]=json!("Staff"),
  "qualifier"=>d["modules"][0]["relationships"][0]["associationRecord"]["key"]=json!("pk"),
  "duplicate"=>{let a=d["modules"][0]["relationships"][0].clone();d["modules"][0]["relationships"].as_array_mut().unwrap().push(a);},_=>{}}
  let text=d.to_string();r["modules"][0]["documentJson"]=json!(text);r["modules"][0]["pin"]["sha256"]=json!(sha256(text.as_bytes()));
  let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&o.to_string(),&catalog).unwrap();let result=SecurityCandidateOntologyClosure::read(&packet,&catalog);
  assert_eq!(result.is_ok(),matches!(mutation,"valid"|"opaque"|"integer-spelling"),"{mutation}");
 }
}

#[test]
fn draft_selected_ownership_and_single_document_raw_endpoints_are_not_inferred(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket,security_candidate_ontology::SecurityCandidateOntologyClosure};
 let (r,policy,ontology)=candidate_sources();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();
 for mutation in ["baseline","shared-owner","second-document","cross-document"]{
  let mut ins=inputs.clone();let mut o=ontology.clone();let mut doc:Value=serde_json::from_str(&ins[0].document_json).unwrap();
  if mutation=="shared-owner"{let record=doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="Assignment").unwrap();record["members"].as_array_mut().unwrap().push(json!({"module":"m","element":"salary"}));let entity=o["entities"].as_array_mut().unwrap().iter_mut().find(|e|e["type"]["elementId"]=="Assignment").unwrap();entity["fields"].as_array_mut().unwrap().push(json!({"ref":{"documentId":"domain","moduleId":"m","elementId":"salary"},"protection":"unprotected"}));ins[0].document_json=doc.to_string();ins[0].pin.sha256=sha256(ins[0].document_json.as_bytes());}
  if matches!(mutation,"second-document"|"cross-document"){
   doc["id"]=json!("other");let mut extra=ins[0].clone();extra.document_json=doc.to_string();extra.pin.document_id="other".into();extra.pin.sha256=sha256(extra.document_json.as_bytes());o["documents"].as_array_mut().unwrap().push(json!({"documentId":"other","revision":extra.pin.revision}));ins.push(extra);
   let mut entity=o["entities"].as_array().unwrap().iter().find(|e|e["type"]["elementId"]=="Project").unwrap().clone();entity["type"]["documentId"]=json!("other");for f in entity["fields"].as_array_mut().unwrap(){f["ref"]["documentId"]=json!("other");}o["entities"].as_array_mut().unwrap().push(entity);
   if mutation=="cross-document"{let endpoint=o["associations"].as_array_mut().unwrap().iter_mut().flat_map(|a|a["endpoints"].as_array_mut().unwrap()).find(|e|e["target"]["elementId"]=="Project").unwrap();endpoint["target"]["documentId"]=json!("other");}
  }
  let catalog=Catalog::prepare_security(ins).unwrap();let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&o.to_string(),&catalog).unwrap();let result=SecurityCandidateOntologyClosure::read(&packet,&catalog);
  assert_eq!(result.is_ok(),matches!(mutation,"baseline"|"second-document"),"{mutation}");
 }
}

#[test]
fn draft_unselected_alternate_keys_retain_core_integrity_without_switching_identity(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket,security_candidate_ontology::SecurityCandidateOntologyClosure};
 let (r,policy,ontology)=candidate_sources();let original:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();
 for mutation in ["valid","id","name","set","owner","repeat","nullable"]{
  let mut inputs=original.clone();let mut doc:Value=serde_json::from_str(&inputs[0].document_json).unwrap();
  let record=doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="Resource").unwrap();
  let mut alt=json!({"id":"salary-key","name":"SalaryKey","fields":[{"module":"m","element":"salary"}],"primary":true});
  match mutation{"id"=>alt["id"]=json!("pk"),"name"=>alt["name"]=json!("Key"),"set"=>alt["fields"]=record["keys"][0]["fields"].clone(),"owner"=>alt["fields"][0]["element"]=json!("staffId"),"repeat"=>{let f=alt["fields"][0].clone();alt["fields"].as_array_mut().unwrap().push(f);},_=>{}}
  record["keys"].as_array_mut().unwrap().push(alt);
  if mutation=="nullable"{doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="salary").unwrap()["nullability"]=json!("absent-allowed");}
  inputs[0].document_json=doc.to_string();inputs[0].pin.sha256=sha256(inputs[0].document_json.as_bytes());
  let prepared=Catalog::prepare_security(inputs);if mutation=="repeat"{assert_eq!(prepared.unwrap_err().code,"WFT-MODEL");continue;}let catalog=prepared.unwrap_or_else(|e|panic!("{mutation}: {e:?}"));let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&ontology.to_string(),&catalog).unwrap();let closure=SecurityCandidateOntologyClosure::read(&packet,&catalog);
  assert_eq!(closure.is_ok(),mutation=="valid","{mutation}");assert_eq!(packet.ontology()["entities"][2]["keyId"],"pk");
 }
}

#[test]
fn draft_policy_checks_raw_correlated_terms_and_intrinsic_types(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket,security_candidate_policy_types::SecurityCandidatePolicyTypes};
 let (r,policy,ontology)=candidate_sources();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();
 let check=|p:&Value|{let packet=SecurityCandidateSourcePacket::read(&p.to_string(),&ontology.to_string(),&catalog)?;SecurityCandidatePolicyTypes::read(&packet,&catalog)};
 let checked=check(&policy).unwrap();assert_eq!(checked.checked_rules(),2);assert_eq!(checked.checked_nodes(),9);
 let mut p=policy.clone();p["rules"][0]["condition"]=json!({"op":"eq","left":{"kind":"resource","field":{"documentId":"domain","moduleId":"m","elementId":"salary"}},"right":{"kind":"constant","field":{"documentId":"domain","moduleId":"m","elementId":"salary"},"value":{"integerToken":"9007199254740993"}}});assert!(check(&p).is_ok());
 p["rules"][0]["condition"]["right"]["value"]["integerToken"]=json!("not-an-integer");assert!(check(&p).is_err());
 let mut p=policy.clone();p["rules"][1]["condition"]["where"]["args"][1]["as"]=p["rules"][1]["condition"]["as"].clone();assert!(check(&p).is_err());
 let mut p=policy.clone();p["rules"][1]["condition"]["association"]["elementId"]=json!("Staff");assert!(check(&p).is_err());
}
#[test]
fn draft_graph_policy_endpoint_terms_do_not_invent_opaque_identity_or_attributes(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket,security_candidate_policy_types::SecurityCandidatePolicyTypes};
 let (r,mut policy,mut ontology)=candidate_sources();let mut inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let mut doc:Value=serde_json::from_str(&inputs[0].document_json).unwrap();
 doc["modules"][0]["relationships"]=json!([{"id":"WorksOn","name":"WorksOn","source":[{"module":"m","element":"Staff"}],"target":[{"module":"m","element":"Project","key":"pk"}],"sourceMultiplicity":{"min":0,"max":"*"},"targetMultiplicity":{"min":0,"max":"*"},"targetLifecycle":"independent","directed":true}]);inputs[0].document_json=doc.to_string();inputs[0].pin.sha256=sha256(inputs[0].document_json.as_bytes());let catalog=Catalog::prepare_security(inputs.clone()).unwrap();
 let reference=json!({"documentId":"domain","moduleId":"m","relationshipId":"WorksOn"});ontology["associations"][1]=json!({"kind":"core-relationship","relationship":reference,"witness":{"kind":"opaque-existential"},"endpoints":[{"role":"staff","side":"source","target":{"documentId":"domain","moduleId":"m","elementId":"Staff"},"keyId":"pk"},{"role":"project","side":"target","target":{"documentId":"domain","moduleId":"m","elementId":"Project"},"keyId":"pk"}]});
 policy["rules"][1]["condition"]=json!({"op":"exists","association":reference,"as":"edge","where":{"op":"eq","left":{"kind":"variable","name":"edge","endpoint":"staff"},"right":{"kind":"subject","identity":true}}});
 let check=|p:&Value|{let packet=SecurityCandidateSourcePacket::read(&p.to_string(),&ontology.to_string(),&catalog)?;SecurityCandidatePolicyTypes::read(&packet,&catalog)};
 assert!(check(&policy).is_ok());
 {use weft_core::security_candidate_ir::{SecurityCandidateLogicalPlan,CandidateExpression,CandidateTerm,CandidateEndpointCarrier,CandidateSide,CandidateWitness};use weft_core::security_ir::Binding;
 let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&ontology.to_string(),&catalog).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();
 let CandidateExpression::Exists{slot,association,witness,condition}=&plan.rules()[1].condition else{panic!()};assert_eq!(*slot,0);assert_eq!(serde_json::to_value(association).unwrap(),reference);assert_eq!(*witness,CandidateWitness::OpaqueExistential);
 let CandidateExpression::Equal(CandidateTerm::Endpoint{binding,association:a,role,target,key_id,carrier},_)=condition.as_ref() else{panic!()};assert_eq!(*binding,Binding::Variable(0));assert_eq!(a,association);assert_eq!(role,"staff");assert_eq!(target.element_id,"Staff");assert_eq!(key_id,"pk");assert_eq!(*carrier,CandidateEndpointCarrier::Incidence{side:CandidateSide::Source});assert_eq!(plan.source().policy(),&policy);
 {use weft_core::security_candidate_incidence::CandidateIncidenceValue;let value=CandidateIncidenceValue::read(&plan,&catalog,association,"staff",CandidateSide::Source,target,"pk",&[json!({"string":"s1"})]).unwrap();assert_eq!(value.target(),target);assert_eq!(value.key_id(),"pk");assert_eq!(value.components().len(),1);assert_eq!(value.component_domains()[0]["scalarType"],"string");assert!(value.require_catalog(&catalog).is_ok());for (role,side,values) in [("staff",CandidateSide::Target,vec![json!({"string":"s1"})]),("missing",CandidateSide::Source,vec![json!({"string":"s1"})]),("staff",CandidateSide::Source,vec![]),("staff",CandidateSide::Source,vec![json!({"integerToken":"1"})]),("staff",CandidateSide::Source,vec![json!({"string":{"nested":true}})])]{assert!(CandidateIncidenceValue::read(&plan,&catalog,association,role,side,target,"pk",&values).is_err());}assert!(CandidateIncidenceValue::read(&plan,&catalog,association,"staff",CandidateSide::Source,target,"other",&[json!({"string":"s1"})]).is_err());let mut wrong_target=target.clone();wrong_target.module_id="other".into();assert!(CandidateIncidenceValue::read(&plan,&catalog,association,"staff",CandidateSide::Source,&wrong_target,"pk",&[json!({"string":"s1"})]).is_err());let record=weft_core::security_association_ref::SecurityAssociationRef::Record(weft_core::security_ontology::SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"WorksOn".into()});assert!(CandidateIncidenceValue::read(&plan,&catalog,&record,"staff",CandidateSide::Source,target,"pk",&[json!({"string":"s1"})]).is_err());let mut stale=catalog.clone();stale.inputs[0].document_json.push(' ');assert!(value.require_catalog(&stale).is_err());assert_eq!(CandidateIncidenceValue::read(&plan,&stale,association,"staff",CandidateSide::Source,target,"pk",&[json!({"string":"s1"})]).unwrap_err().code,"WFT-SECURITY-PIN");}

 let dependencies=weft_core::security_candidate_dependencies::SecurityCandidateDependencies::derive(&plan,&catalog,&plan.rules()[1].target,"read").unwrap();assert_eq!(dependencies.incidences().len(),2);assert_eq!(dependencies.associations().len(),1);assert!(!dependencies.keys().keys().any(|r|r.element_id=="Assignment"||r.element_id=="WorksOn"));assert!(!dependencies.fields().keys().any(|r|r.element_id=="Assignment"||r.element_id=="WorksOn"));
 {use weft_core::security_composition::{compose_candidate,Truth,Decision};use std::collections::BTreeMap;let truths=BTreeMap::from([("reader".into(),Truth::True),("membership".into(),Truth::True)]);let output=weft_core::security_ontology::SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"salary".into()};let result=compose_candidate(&plan,&catalog,&plan.rules()[1].target,"read",std::slice::from_ref(&output),&truths).unwrap();assert_eq!(result.decision,Decision::Permit);assert!(matches!(result.disclosure[0].1,weft_core::security_ir::Disposition::Withheld));let mut unknown=truths.clone();unknown.insert("membership".into(),Truth::Unknown);let result=compose_candidate(&plan,&catalog,&plan.rules()[1].target,"read",&[output],&unknown).unwrap();assert_eq!(result.decision,Decision::Indeterminate);assert!(result.disclosure.is_empty());}

 let mut target_policy=policy.clone();let operand=json!({"kind":"variable","name":"edge","endpoint":"project"});target_policy["rules"][1]["condition"]["where"]=json!({"op":"eq","left":operand,"right":operand});let packet=SecurityCandidateSourcePacket::read(&target_policy.to_string(),&ontology.to_string(),&catalog).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();let CandidateExpression::Exists{condition,..}=&plan.rules()[1].condition else{panic!()};let CandidateExpression::Equal(CandidateTerm::Endpoint{role,target,carrier,..},_)=condition.as_ref() else{panic!()};assert_eq!(role,"project");assert_eq!(target.element_id,"Project");assert_eq!(*carrier,CandidateEndpointCarrier::Incidence{side:CandidateSide::Target});
 }
 for mutation in ["identity","attribute","role","namespace","unbound"]{let mut p=policy.clone();match mutation{
  "identity"=>p["rules"][1]["condition"]["where"]["left"]=json!({"kind":"variable","name":"edge","identity":true}),
  "attribute"=>p["rules"][1]["condition"]["where"]["left"]=json!({"kind":"variable","name":"edge","field":{"documentId":"domain","moduleId":"m","elementId":"active"}}),
  "role"=>p["rules"][1]["condition"]["where"]["left"]["endpoint"]=json!("missing"),
  "namespace"=>p["rules"][1]["condition"]["association"]=json!({"documentId":"domain","moduleId":"m","elementId":"WorksOn"}),
  _=>p["rules"][1]["condition"]["where"]["left"]["name"]=json!("other")};if matches!(mutation,"identity"|"attribute"){p["rules"][1]["condition"]["where"]["right"]=p["rules"][1]["condition"]["where"]["left"].clone();}assert!(check(&p).is_err(),"{mutation}");}
 let mut record_doc=doc.clone();record_doc["modules"][0]["relationships"][0]["associationRecord"]=json!({"module":"m","element":"Assignment"});let mut record_inputs=inputs.clone();record_inputs[0].document_json=record_doc.to_string();record_inputs[0].pin.sha256=sha256(record_inputs[0].document_json.as_bytes());let record_catalog=Catalog::prepare_security(record_inputs).unwrap();let mut record_ontology=ontology.clone();record_ontology["associations"][1]["witness"]=json!({"kind":"record-key","type":{"documentId":"domain","moduleId":"m","elementId":"Assignment"},"keyId":"pk"});
 for operand in [json!({"kind":"variable","name":"edge","identity":true}),json!({"kind":"variable","name":"edge","field":{"documentId":"domain","moduleId":"m","elementId":"active"}})]{let mut p=policy.clone();p["rules"][1]["condition"]["where"]=json!({"op":"eq","left":operand,"right":operand});assert!(check(&p).is_err());let packet=SecurityCandidateSourcePacket::read(&p.to_string(),&record_ontology.to_string(),&record_catalog).unwrap();assert!(SecurityCandidatePolicyTypes::read(&packet,&record_catalog).is_ok());let plan=weft_core::security_candidate_ir::SecurityCandidateLogicalPlan::read(packet,&record_catalog).unwrap();let deps=weft_core::security_candidate_dependencies::SecurityCandidateDependencies::derive(&plan,&record_catalog,&plan.rules()[1].target,"read").unwrap();assert_eq!(deps.incidences().len(),2);assert!(deps.keys().keys().any(|r|r.element_id=="Assignment"));assert!(deps.fields().iter().find(|(r,_)|r.element_id=="Assignment").unwrap().1.iter().any(|f|f.element_id=="active"));}
}

#[test]
fn draft_constants_and_result_declarations_require_classified_required_operands(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket,security_candidate_policy_types::SecurityCandidatePolicyTypes};
 let (r,policy,ontology)=candidate_sources();let original:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();
 for mutation in ["valid","unclassified-constant","unclassified-result","nullable"]{
  let mut p=policy.clone();let mut inputs=original.clone();let mut doc:Value=serde_json::from_str(&inputs[0].document_json).unwrap();
  doc["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":"unclassifiedSalary","kind":"field","scalarType":"integer","cardinality":"one","nullability":"required","extensions":{}}));
  let field=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});let mut declaration=field.clone();if mutation.starts_with("unclassified"){declaration["elementId"]=json!("unclassifiedSalary");}
  let value=if mutation=="nullable"{doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|f|f["id"]=="salary").unwrap()["nullability"]=json!("absent-allowed");Value::Null}else{json!({"integerToken":"1"})};
  p["rules"][0]["condition"]=json!({"op":"eq","left":{"kind":"resource","field":field},"right":{"kind":"constant","field":if mutation=="unclassified-constant"{declaration.clone()}else{field.clone()},"value":value}});
  p["rules"][0]["disclosure"]=json!([{"field":field,"disposition":{"kind":"transformed","transform":"constant","version":"0.1.0","field":if mutation=="unclassified-result"{declaration}else{field},"value":{"integerToken":"1"}}}]);
  if mutation=="nullable"{p["rules"][0].as_object_mut().unwrap().remove("disclosure");}
  inputs[0].document_json=doc.to_string();inputs[0].pin.sha256=sha256(inputs[0].document_json.as_bytes());let catalog=Catalog::prepare_security(inputs).unwrap();let packet=SecurityCandidateSourcePacket::read(&p.to_string(),&ontology.to_string(),&catalog).unwrap();let result=SecurityCandidatePolicyTypes::read(&packet,&catalog);assert_eq!(result.is_ok(),mutation=="valid","{mutation}");
 }
}

#[test]
fn draft_raw_ir_retains_nested_witness_slots_ordered_carriers_and_source_custody(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket,security_candidate_ir::{SecurityCandidateLogicalPlan,CandidateExpression,CandidateTerm,CandidateWitness,CandidateEndpointCarrier},security_ir::Binding};
 let (r,policy,ontology)=candidate_sources();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let mut catalog=Catalog::prepare_security(inputs).unwrap();let source=format!("\n{}\n",policy);let packet=SecurityCandidateSourcePacket::read(&source,&ontology.to_string(),&catalog).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();assert_eq!(plan.source().policy_json(),source);
 let CandidateExpression::Exists{slot,witness,condition,..}=&plan.rules()[1].condition else{panic!()};assert_eq!(*slot,0);let CandidateWitness::RecordKey{owner,key_id,key}=witness else{panic!()};assert_eq!(owner.element_id,"Ownership");assert_eq!(key_id,"pk");assert_eq!(key["fields"][0]["element"],"ownerId");
 let CandidateExpression::And(args)=condition.as_ref() else{panic!()};let CandidateExpression::Equal(CandidateTerm::Endpoint{binding,carrier,..},_)=&args[0] else{panic!()};assert_eq!(*binding,Binding::Variable(0));let CandidateEndpointCarrier::Members{fields}=carrier else{panic!()};assert_eq!(fields[0].element_id,"ownerResource");
 let CandidateExpression::Exists{slot,condition,..}=&args[1] else{panic!()};assert_eq!(*slot,1);let CandidateExpression::And(inner)=condition.as_ref() else{panic!()};let CandidateExpression::Equal(CandidateTerm::Endpoint{binding:left,..},CandidateTerm::Endpoint{binding:right,..})=&inner[1] else{panic!()};assert_eq!(*left,Binding::Variable(1));assert_eq!(*right,Binding::Variable(0));
 let mut siblings=policy.clone();let condition=policy["rules"][1]["condition"].clone();siblings["rules"][1]["condition"]=json!({"op":"and","args":[condition,condition]});let packet=SecurityCandidateSourcePacket::read(&siblings.to_string(),&ontology.to_string(),&catalog).unwrap();let sibling_plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();let CandidateExpression::And(branches)=&sibling_plan.rules()[1].condition else{panic!()};for (branch,expected) in branches.iter().zip([0,2]){let CandidateExpression::Exists{slot,..}=branch else{panic!()};assert_eq!(*slot,expected);}
 catalog.inputs[0].document_json.push(' ');assert_eq!(plan.require_catalog(&catalog).unwrap_err().code,"WFT-SECURITY-PIN");
}

#[test]
fn draft_dependencies_separate_stored_context_and_constant_channels(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket,security_candidate_ir::SecurityCandidateLogicalPlan,security_candidate_dependencies::SecurityCandidateDependencies,security_ontology::SecurityRef};
 let (r,policy,ontology)=candidate_sources();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let field=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});let salary=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"salary".into()};
 for mode in ["constant","context"]{let mut p=policy.clone();p["rules"].as_array_mut().unwrap().truncate(1);let mut o=ontology.clone();
  let constant=json!({"kind":"constant","field":field,"value":{"integerToken":"1"}});p["rules"][0]["condition"]=if mode=="constant"{json!({"op":"eq","left":constant,"right":constant})}else{o["context"].as_array_mut().unwrap().push(field.clone());json!({"op":"eq","left":{"kind":"resource","field":field},"right":{"kind":"context","field":field}})};
  let packet=SecurityCandidateSourcePacket::read(&p.to_string(),&o.to_string(),&catalog).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();let target=&plan.rules()[0].target;let deps=SecurityCandidateDependencies::derive(&plan,&catalog,target,"read").unwrap();assert_eq!(deps.fields().get(target).unwrap().contains(&salary),mode=="context");assert_eq!(deps.context().contains(&salary),mode=="context");assert!(deps.associations().is_empty());assert!(deps.incidences().is_empty());assert!(SecurityCandidateDependencies::derive(&plan,&catalog,target,"undeclared").is_err());
 }
 let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&ontology.to_string(),&catalog).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();let deps=SecurityCandidateDependencies::derive(&plan,&catalog,&plan.rules()[1].target,"read").unwrap();assert_eq!(deps.associations().len(),2);assert!(deps.incidences().is_empty());let assignment=deps.fields().iter().find(|(r,_)|r.element_id=="Assignment").unwrap();assert!(assignment.1.iter().any(|f|f.element_id=="active"));assert!(deps.keys().keys().any(|r|r.element_id=="Assignment"));
}

#[test]
fn draft_composition_matches_independent_truth_corpus_without_record_plan_cast(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket,security_candidate_ir::SecurityCandidateLogicalPlan,security_composition::{compose_candidate,Truth,Decision},security_ontology::SecurityRef};use std::collections::BTreeMap;
 let (r,mut policy,ontology)=candidate_sources();let mut forbid=policy["rules"][1].clone();forbid["id"]=json!("forbid");forbid["effect"]=json!("forbid");policy["rules"].as_array_mut().unwrap().push(forbid);let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&ontology.to_string(),&catalog).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();let corpus:Value=serde_json::from_str(include_str!("security-composition-oracle.json")).unwrap();let field=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"salary".into()};let truth=|v:&Value|match v.as_str().unwrap(){"T"=>Truth::True,"F"=>Truth::False,"U"=>Truth::Unknown,_=>panic!()};
 assert_eq!(corpus["decisions"].as_array().unwrap().len(),27);for v in corpus["decisions"].as_array().unwrap(){let values=BTreeMap::from([("reader".into(),truth(&v["permit"])),("membership".into(),truth(&v["require"])),("forbid".into(),truth(&v["forbid"]))]);let result=compose_candidate(&plan,&catalog,&plan.rules()[0].target,"read",std::slice::from_ref(&field),&values).unwrap();let decision=match result.decision{Decision::Permit=>"permit",Decision::Deny=>"deny",Decision::Indeterminate=>"indeterminate",Decision::Conflict=>"conflict"};assert_eq!(v["decision"],decision);if result.decision!=Decision::Permit{assert!(result.disclosure.is_empty());}}
 assert!(compose_candidate(&plan,&catalog,&plan.rules()[0].target,"read",&[field],&BTreeMap::new()).is_err());
}

#[test]
fn draft_disclosure_bridge_preserves_missing_masks_exact_conflicts_and_withheld(){
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket,security_candidate_ir::SecurityCandidateLogicalPlan,security_composition::{compose_candidate,Truth,Decision},security_ontology::SecurityRef,security_ir::Disposition};use std::collections::BTreeMap;
 let (r,policy,ontology)=candidate_sources();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let field=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});let output=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"salary".into()};
 for mode in ["missing","equal","conflict","withheld"]{let mut p=policy.clone();let mut truths=BTreeMap::from([("reader".into(),Truth::True),("membership".into(),Truth::True)]);
  if mode=="missing"{p["rules"][0].as_object_mut().unwrap().remove("disclosure");}else{
   p["rules"][0]["disclosure"]=json!([{"field":field,"disposition":{"kind":"transformed","transform":"constant","version":"0.1.0","field":field,"value":{"integerToken":"9007199254740993"}}}]);let mut second=p["rules"][0].clone();second["id"]=json!("second");second["disclosure"][0]["disposition"]["value"]=json!({"integerToken":if mode=="equal"{"9007199254740993.0e0"}else{"9007199254740992"}});p["rules"].as_array_mut().unwrap().push(second);truths.insert("second".into(),Truth::True);
   if mode=="withheld"{let mut withheld=p["rules"][0].clone();withheld["id"]=json!("withheld");withheld["disclosure"][0]["disposition"]=json!({"kind":"withheld"});p["rules"].as_array_mut().unwrap().push(withheld);truths.insert("withheld".into(),Truth::True);}
  }
  let packet=SecurityCandidateSourcePacket::read(&p.to_string(),&ontology.to_string(),&catalog).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();let result=compose_candidate(&plan,&catalog,&plan.rules()[0].target,"read",std::slice::from_ref(&output),&truths).unwrap();let expected=match mode{"missing"=>Decision::Indeterminate,"conflict"=>Decision::Conflict,_=>Decision::Permit};assert_eq!(result.decision,expected,"{mode}");if expected!=Decision::Permit{assert!(result.disclosure.is_empty());}else if mode=="withheld"{assert!(matches!(result.disclosure[0].1,Disposition::Withheld));}else{assert!(matches!(result.disclosure[0].1,Disposition::Transformed{..}));}
 }
}

// CONTRACT-006 security compile 0.2 registration boundary; all physical/native acceptance stays open.
fn security04_request()->Value { let mut value=query_profile_request();value["interfaceVersion"]=json!("weft-security-compile/0.2.0");value }
fn registration_manifest(request:&Value)->Value {
 json!({"interfaceVersion":"weft-security-backend/0.1.0","backendId":request["target"]["backendId"],"backendVersion":request["target"]["backendVersion"],"bindingProfile":"registration-fixture","sourceProfiles":[{"dialect":"weft-sql/0.2.0","applicationIr":"weft-ir/0.2.0","policy":"0.1.0","ontology":"0.1.0","securityIr":"weft.security.logical-ir/0.1.0"}],"targetProfiles":[{"id":request["target"]["targetProfile"],"engine":"fixture-only","engineVersion":"unqualified","sessionSettings":{},"storageLayoutRevision":"fixture","publicationRevision":"fixture"}],"capabilities":[{"id":"declaration-only","targetProfiles":[request["target"]["targetProfile"]],"languageProfiles":[{"dialectProfile":"weft-sql/0.2.0","irVersion":"weft-ir/0.2.0"}],"logicalDomain":{"fixture":true},"resultDomain":{"fixture":true},"constraints":[],"obligations":[],"status":"candidate","evidence":[]}],"evidence":[]})
}
struct RegistrationFixture(String);
impl weft_core::security_backend::SecurityBackend for RegistrationFixture {
 fn manifest_json(&self)->&str { &self.0 }
 fn lower(&self,_:&weft_core::security_backend::SecurityBackendContext<'_>)->weft_core::error::Result<weft_core::security_lowering::SecurityLowering> { panic!("Unqualified security lowering was dispatched") }
}
#[test]
fn security04_callback_receives_actual_owner_objects_once_and_never_emits_partial_output(){
 use weft_core::security_backend::SecurityRegistry;
 for sql in ["SELECT r.resourceId FROM Resource r","SELECT COUNT(*) FROM Resource r"] {
  let mut req=security04_request();req["sql"]=json!(sql);let mut calls=0;
  let manifest=registration_manifest(&req).to_string();
  let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |context|{
   calls+=1;
   assert!(std::ptr::eq(context.query(),context.profiled_query().query()));
   assert_eq!(context.query().sql(),sql);
   assert_eq!(context.logical_plan().source().policy_json(),req["security"]["policyJson"].as_str().unwrap());
   assert_eq!(context.binding_json(),req["target"]["bindingJson"].as_str().unwrap());
   assert_eq!(context.backend_id(),req["target"]["backendId"].as_str().unwrap());
   assert_eq!(context.backend_version(),req["target"]["backendVersion"].as_str().unwrap());
   assert_eq!(context.target_profile(),req["target"]["targetProfile"].as_str().unwrap());
   assert!(!context.profiled_query().obligations().is_empty());
   context.profiled_query().require_sources(context.logical_plan(),context.catalog(),context.binding_json(),context.backend_id(),context.backend_version(),context.target_profile())?;
   let mut registry=SecurityRegistry::default();registry.register(RegistrationFixture(manifest.clone()))?;Ok(registry)
  });
  assert_eq!(calls,1);let response:Value=serde_json::from_str(&raw).unwrap();assert_eq!(response["interfaceVersion"],"weft-security-compile/0.2.0");
  refused(response.clone(),"WFT-SECURITY-LOWERING-UNSUPPORTED");assert_eq!(response.as_object().unwrap().len(),3);
 }
}
#[test]
fn security04_owner_refusals_never_dispatch_registration(){
 let baseline=security04_request();
 let mut missing=baseline.clone();missing["security"].as_object_mut().unwrap().remove("queryProfileJson");
 let mut digest=baseline.clone();digest["modules"][0]["pin"]["sha256"]=json!("0".repeat(64));
 let mut stale=baseline.clone();let mut profile:Value=serde_json::from_str(stale["security"]["queryProfileJson"].as_str().unwrap()).unwrap();profile["ontologySha256"]=json!("0".repeat(64));stale["security"]["queryProfileJson"]=json!(profile.to_string());
 let mut sql=baseline.clone();sql["sql"]=json!("SELECT r.resourceId FROM Resource r WHERE r.salary=100");
 let mut unknown=baseline.clone();let mut policy:Value=serde_json::from_str(unknown["security"]["policyJson"].as_str().unwrap()).unwrap();policy["future"]=json!(true);unknown["security"]["policyJson"]=json!(policy.to_string());
 for (req,code) in [(missing,"WFT-INPUT"),(digest,"WFT-PIN"),(stale,"WFT-SECURITY-QUERY-PROFILE"),(sql,"WFT-SECURITY-QUERY-PROFILE"),(unknown,"WFT-SECURITY-SOURCE")] {
  let mut calls=0;let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |_|{calls+=1;panic!("Refused owner source reached callback")});assert_eq!(calls,0);refused(serde_json::from_str(&raw).unwrap(),code);
 }
}
#[test]
fn security04_requires_explicit_exact_registration_and_never_falls_back(){
 use weft_core::security_backend::SecurityRegistry;
 let req=security04_request();refused(response(&req),"WFT-SECURITY-BACKEND-REQUIRED");
 let mut ordinary_calls=0;let raw=Compiler::default().compile_json_with_factory(&req.to_string(),&mut |_,_,_|{ordinary_calls+=1;panic!("Ordinary security fallback")});assert_eq!(ordinary_calls,0);refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
 let mut calls=0;let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |_|{calls+=1;Ok(SecurityRegistry::default())});assert_eq!(calls,1);refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
 for member in ["backendId","backendVersion"] {
  let mut manifest=registration_manifest(&req);manifest[member]=json!("other");let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |_|{let mut registry=SecurityRegistry::default();registry.register(RegistrationFixture(manifest.to_string()))?;Ok(registry)});refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
 }
 let mut manifest=registration_manifest(&req);manifest["targetProfiles"][0]["id"]=json!("other");manifest["capabilities"][0]["targetProfiles"]=json!(["other"]);
 let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |_|{let mut registry=SecurityRegistry::default();registry.register(RegistrationFixture(manifest.to_string()))?;Ok(registry)});refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-VERSION");
 let mut calls=0;let raw=Compiler::default().compile_json_with_security_factory(&query_profile_request().to_string(),&mut |_|{calls+=1;panic!("Legacy source reached security callback")});assert_eq!(calls,0);refused(serde_json::from_str(&raw).unwrap(),"WFT-VERSION");
}
#[test]
fn security_manifest_closed_versions_bounds_and_references_refuse(){
 use weft_core::security_backend::{validate_security_manifest_json,SecurityRegistry};
 let original=registration_manifest(&security04_request());assert!(validate_security_manifest_json(&original.to_string()).is_ok());
 let edits:Vec<Box<dyn Fn(&mut Value)>>=vec![
 Box::new(|v|v["interfaceVersion"]=json!("weft-backend/0.2.0")),Box::new(|v|v["extra"]=json!(true)),Box::new(|v|v["sourceProfiles"]=json!([])),Box::new(|v|v["sourceProfiles"][0]["ontology"]=json!("0.2.0")),Box::new(|v|v["sourceProfiles"][0]["extra"]=json!(true)),Box::new(|v|{let p=v["sourceProfiles"][0].clone();v["sourceProfiles"].as_array_mut().unwrap().push(p);}),
 Box::new(|v|v["bindingProfile"]=json!("")),Box::new(|v|v["backendId"]=json!("x\0y")),Box::new(|v|v["backendVersion"]=json!("界".repeat(4097))),Box::new(|v|v["targetProfiles"]=json!([])),Box::new(|v|{let p=v["targetProfiles"][0].clone();v["targetProfiles"].as_array_mut().unwrap().push(p);}),Box::new(|v|v["targetProfiles"][0]["extra"]=json!(true)),Box::new(|v|v["capabilities"]=json!([])),Box::new(|v|v["capabilities"][0]["targetProfiles"]=json!(["unknown"])),Box::new(|v|v["capabilities"][0]["languageProfiles"]=json!([{"dialectProfile":"weft-sql/0.1.0","irVersion":"weft-ir/0.1.0"}])),Box::new(|v|v["capabilities"][0]["status"]=json!("supported")),Box::new(|v|v["capabilities"][0]["evidence"]=json!(["undeclared"])),Box::new(|v|v["capabilities"][0]["obligations"]=json!([{"id":"o","parameters":{},"owner":"backend","failureCode":"WFT-bad"}])),
 ];
 for edit in edits {let mut changed=original.clone();edit(&mut changed);assert_eq!(validate_security_manifest_json(&changed.to_string()).unwrap_err().code,"WFT-SECURITY-BACKEND-VERSION");}
 let duplicate=original.to_string().replacen('{',"{\"backendId\":\"duplicate\",",1);assert!(validate_security_manifest_json(&duplicate).is_err());assert!(validate_security_manifest_json(&" ".repeat(1024*1024+1)).is_err());
 let mut registry=SecurityRegistry::default();registry.register(RegistrationFixture(original.to_string())).unwrap();assert!(registry.register(RegistrationFixture(original.to_string())).is_err());
}

#[test]
fn security_manifest_positional_struct_carriers_refuse_before_registration(){
 use weft_core::security_backend::{validate_security_manifest_json,SecurityRegistry};
 let mut original=registration_manifest(&security04_request());
 original["capabilities"][0]["obligations"]=json!([{"id":"obligation","parameters":{},"owner":"backend","failureCode":"WFT-FIXTURE"}]);
 let opaque=json!({"future":{"array":[null,{"nested":[true,"meaning",7]}],"object":{}}});
 original["capabilities"][0]["logicalDomain"]=opaque.clone();original["capabilities"][0]["resultDomain"]=opaque.clone();
 original["targetProfiles"][0]["sessionSettings"]=opaque.clone();original["capabilities"][0]["obligations"][0]["parameters"]=opaque.clone();
 let parsed=validate_security_manifest_json(&original.to_string()).unwrap();
 assert_eq!(parsed.capabilities[0].logical_domain,opaque);assert_eq!(parsed.capabilities[0].result_domain,opaque);
 assert_eq!(parsed.target_profiles[0].session_settings,opaque);assert_eq!(parsed.capabilities[0].obligations[0].parameters,opaque);
 let positional=|v:&Value,keys:&[&str]|json!(keys.iter().map(|k|v[*k].clone()).collect::<Vec<_>>());
 let mut accepted=Vec::new();
 for carrier in ["manifest","source","target","capability","language","obligation"]{
  let mut changed=original.clone();match carrier{
   "manifest"=>changed=positional(&changed,&["interfaceVersion","backendId","backendVersion","sourceProfiles","bindingProfile","targetProfiles","capabilities","evidence"]),
   "source"=>changed["sourceProfiles"][0]=positional(&changed["sourceProfiles"][0],&["dialect","applicationIr","policy","ontology","securityIr"]),
   "target"=>changed["targetProfiles"][0]=positional(&changed["targetProfiles"][0],&["id","engine","engineVersion","sessionSettings","storageLayoutRevision","publicationRevision"]),
   "capability"=>changed["capabilities"][0]=positional(&changed["capabilities"][0],&["id","targetProfiles","languageProfiles","logicalDomain","resultDomain","constraints","obligations","status","evidence"]),
   "language"=>changed["capabilities"][0]["languageProfiles"][0]=positional(&changed["capabilities"][0]["languageProfiles"][0],&["dialectProfile","irVersion"]),
   "obligation"=>changed["capabilities"][0]["obligations"][0]=positional(&changed["capabilities"][0]["obligations"][0],&["id","parameters","owner","failureCode"]),_=>unreachable!()
  }
  let error=match validate_security_manifest_json(&changed.to_string()){Ok(_)=>{accepted.push(carrier);continue;},Err(error)=>error};assert_eq!(error.code,"WFT-SECURITY-BACKEND-VERSION");assert_eq!(error.phase,"capability");
  let mut registry=SecurityRegistry::default();assert!(registry.register(RegistrationFixture(changed.to_string())).is_err());
 }
 assert!(accepted.is_empty(),"Accepted positional carriers: {accepted:?}");
}

#[test]
fn manifest_status_and_candidate_option_never_upgrade_unqualified_lowering(){
 use weft_core::security_backend::SecurityRegistry;
 for status in ["supported","candidate","unsupported"] { for allow in [false,true] {
  let mut req=security04_request();req["options"]=json!({"allowCandidate":allow});let mut manifest=registration_manifest(&req);manifest["capabilities"][0]["status"]=json!(status);
  if status=="supported" {manifest["evidence"]=json!(["declared-fixture-only"]);manifest["capabilities"][0]["evidence"]=json!(["declared-fixture-only"]);}
  let mut calls=0;let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |_|{calls+=1;let mut registry=SecurityRegistry::default();registry.register(RegistrationFixture(manifest.to_string()))?;Ok(registry)});assert_eq!(calls,1);refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-LOWERING-UNSUPPORTED");
 }}
 let req=security04_request();let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |_|Err(weft_core::error::Diagnostic::new("WFT-SECURITY-BACKEND-VERSION","capability","Failed registration fixture")));refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-VERSION");
}

#[test]
fn security_callback_malformed_diagnostics_still_produce_bounded_error_refusals(){
 use weft_core::error::Diagnostic;
 let req=security04_request();
 let edits:Vec<Box<dyn Fn(&mut Diagnostic)>>=vec![
 Box::new(|d|d.severity="warning".into()),Box::new(|d|d.severity="info".into()),Box::new(|d|d.phase="invalid".into()),Box::new(|d|d.code="".into()),Box::new(|d|d.code="WFT-lowercase".into()),Box::new(|d|d.code="WFT-".into()),Box::new(|d|d.message="".into()),Box::new(|d|d.message="界".repeat(4097)),Box::new(|d|d.message="x\0y".into()),Box::new(|d|d.source_span=Some(weft_core::ir::Span{start:2,end:1})),Box::new(|d|d.source_span=Some(weft_core::ir::Span{start:0,end:usize::MAX}))
 ];
 for edit in edits {
  let mut diagnostic=Diagnostic::new("WFT-SECURITY-BACKEND-VERSION","capability","Fixture refusal");edit(&mut diagnostic);let mut calls=0;
  let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |_|{calls+=1;Err(diagnostic.clone())});assert_eq!(calls,1);let value:Value=serde_json::from_str(&raw).unwrap();refused(value.clone(),"WFT-SECURITY-BACKEND-VERSION");assert_eq!(value["interfaceVersion"],"weft-security-compile/0.2.0");assert_eq!(value.as_object().unwrap().len(),3);assert_eq!(value["diagnostics"][0]["severity"],"error");assert_eq!(value["diagnostics"][0]["phase"],"capability");assert!(value["diagnostics"][0].get("sourceSpan").is_none());assert!(raw.len()<1024);
 }
}

#[test]
fn owner_requirements_keep_output_occurrences_actions_and_complete_rule_objects() {
 use weft_core::{security_backend::SecurityRegistry, security_requirements::SecurityOperatorMode};
 for sql in ["SELECT r.resourceId AS first, r.resourceId AS second FROM Resource r", "SELECT COUNT(*) FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId", "SELECT r.resourceId FROM Resource r WHERE r.salary=100", "SELECT SUM(r.salary) FROM Resource r"] {
  let mut req=security04_request();req["sql"]=json!(sql);let mut calls=0;
  let mut ontology:Value=serde_json::from_str(req["security"]["ontologyJson"].as_str().unwrap()).unwrap();
  if !ontology["actions"].as_array().unwrap().iter().any(|a|a=="query-original") { ontology["actions"].as_array_mut().unwrap().push(json!("query-original")); }
  let target=json!({"documentId":"domain","moduleId":"m","elementId":"Resource"});
  let field=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});
  for entity in ontology["entities"].as_array_mut().unwrap() { if entity["type"]==target { for classified in entity["fields"].as_array_mut().unwrap() { if classified["ref"]==field { classified["queryUse"]=json!({"predicate":"original-authorized","aggregate":"original-authorized"}); } } } }
  let mut policy:Value=serde_json::from_str(req["security"]["policyJson"].as_str().unwrap()).unwrap();
  let mut false_permit=policy["rules"][0].clone();false_permit["id"]=json!("retained-false-permit");false_permit["condition"]=json!({"op":"literal","value":false});
  let mut forbid=false_permit.clone();forbid["id"]=json!("retained-forbid");forbid["effect"]=json!("forbid");forbid.as_object_mut().unwrap().remove("disclosure");
  policy["rules"].as_array_mut().unwrap().extend([false_permit,forbid]);
  req["security"]["ontologyJson"]=json!(ontology.to_string());req["security"]["policyJson"]=json!(policy.to_string());
  let mut profile:Value=serde_json::from_str(req["security"]["queryProfileJson"].as_str().unwrap()).unwrap();
  profile["ontologySha256"]=json!(sha256(ontology.to_string().as_bytes()));profile["policySha256"]=json!(sha256(policy.to_string().as_bytes()));
  profile["bindings"]=json!([{"target":target,"field":field,"operator":"predicate","originalAction":"query-original"},{"target":target,"field":field,"operator":"aggregate","originalAction":"query-original"}]);
  req["security"]["queryProfileJson"]=json!(profile.to_string());
  let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |context| {
   calls+=1;let requirements=context.requirements();
   assert_eq!(requirements.primary_action(),"read");
   assert_eq!(requirements.outputs().len(),context.query().application_plan().outputs.len());
   for (index, output) in requirements.outputs().iter().enumerate() {
    assert_eq!(output.position(),index+1);
    assert!(std::ptr::eq(output.output(),&context.query().application_plan().outputs[index]));
   }
   assert_eq!(requirements.scans().len(),context.query().scans().len());
   for scan in requirements.scans() {
    assert_eq!(context.query().scans()[scan.inventory().scan()],*scan.inventory().target());
    for action in scan.actions() {
     assert_eq!(action.rules().len(),action.inventory().rule_ids().len());
     if action.inventory().action()=="read" {
      assert!(action.rules().iter().any(|r|r.id=="retained-false-permit" && r.effect==weft_core::security_ir::Effect::Permit && r.condition==weft_core::security_ir::Expression::Literal(false)));
      assert!(action.rules().iter().any(|r|r.id=="retained-forbid" && r.effect==weft_core::security_ir::Effect::Forbid));
     }
     for rule in action.rules() {
      assert!(context.logical_plan().rules().iter().any(|actual|std::ptr::eq(actual,*rule)));
      assert_eq!(&rule.target,scan.inventory().target());
      assert!(rule.actions.iter().any(|a|a==action.inventory().action()));
     }
    }
   }
   assert_eq!(requirements.operators().len(),context.profiled_query().uses().len());
   for (operator,(actual, action)) in requirements.operators().iter().zip(context.profiled_query().uses()) {
    assert!(std::ptr::eq(operator.usage(),actual));
    assert_eq!(operator.mode(),match action {Some(a)=>SecurityOperatorMode::OriginalAuthorized(a),None=>SecurityOperatorMode::Disclosed});
   }
   if sql.contains("first") {
    assert_eq!(requirements.outputs().iter().map(|o|o.output().name.as_str()).collect::<Vec<_>>(),vec!["first","second"]);
    assert_eq!(context.query().projections().len(),1);
   }
   if sql.contains("COUNT") {
    assert_eq!(requirements.scans().len(),2);
    assert!(matches!(requirements.outputs()[0].output().expression,weft_core::application_ir::Expression::Count{..}));
    assert!(requirements.scans().iter().all(|s|s.inventory().projection_fields().is_empty()));
    assert_eq!(requirements.operators().len(),2);
   }
   if sql.contains("SUM") {
    match &requirements.outputs()[0].output().expression {
     weft_core::application_ir::Expression::Sum{argument,logical_type}=>{
      assert_eq!(argument.identity.element,"salary");assert_eq!(argument.scan,"s0");
      assert_eq!(serde_json::to_value(logical_type).unwrap(),json!({"family":"integer","nullable":true,"facets":{}}));
      assert_ne!(serde_json::to_value(logical_type).unwrap(),serde_json::to_value(&argument.logical_type).unwrap());
     }, _=>panic!("SUM requirement was erased")
    }
    assert!(requirements.operators().iter().any(|o|o.usage().operator==weft_core::security_ir::QueryOperator::Aggregate));
   }
   if sql.contains("salary") {
    assert!(requirements.operators().iter().any(|o|o.mode()==SecurityOperatorMode::OriginalAuthorized("query-original")));
    assert!(requirements.scans()[0].actions().iter().any(|a|a.inventory().action()=="query-original"));
   }
   Ok(SecurityRegistry::default())
  });
  assert_eq!(calls,1,"{sql}: {raw}");refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
 }
}

fn direct_result_contract(ctx:&weft_core::security_backend::SecurityBackendContext<'_>)->weft_core::security_lowering::SecurityResultContract {
 use weft_core::security_lowering::*;
 SecurityResultContract{version:"weft.security.result-contract/0.1.0".into(),encoding:"weft.security.cells/0.1.0".into(),columns:ctx.requirements().outputs().iter().map(|o|{
  let weft_core::application_ir::Expression::Field{identity,..}=&o.output().expression else {panic!("Direct fixture expected")};
  let field=SecurityIdentity{document_id:identity.document_id.clone(),revision:identity.revision.clone(),module:identity.module.clone(),element:identity.element.clone()};
  SecurityResultColumn{position:o.position(),output_name:o.output().name.clone(),source_fields:vec![field.clone()],outcomes:vec![SecurityResultOutcome::Original{id:"original".into(),domain:SecurityResultDomain::Model{field}}]}
 }).collect()}
}
#[test]
fn result_declaration_checks_ordered_original_domains_without_opening_lowering() {
 use weft_core::{security_backend::SecurityRegistry,security_lowering::*};
 let mut req=security04_request();req["sql"]=json!("SELECT r.resourceId AS a, r.resourceId AS b FROM Resource r");let mut calls=0;
 let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |ctx|{
  calls+=1;let baseline=direct_result_contract(ctx);ctx.check_result_declaration(&baseline)?;
  for mutation in 0..8 {let mut bad=baseline.clone();match mutation {
   0=>bad.columns.swap(0,1),1=>{bad.columns.pop();},2=>bad.columns[0].output_name="wrong".into(),
   3=>bad.columns[0].source_fields[0].revision="other".into(),
   4=>bad.columns[0].outcomes=vec![SecurityResultOutcome::Absent{id:"absent".into()}],
   5=>{let duplicate=bad.columns[0].outcomes[0].clone();bad.columns[0].outcomes.push(duplicate);},
   6=>{let SecurityResultOutcome::Original{domain,..}=&mut bad.columns[0].outcomes[0] else{unreachable!()};let SecurityResultDomain::Model{field}=domain else{unreachable!()};field.revision="other".into();},
   _=>bad.encoding="unknown".into()
  }assert_eq!(ctx.check_result_declaration(&bad).unwrap_err().code,"WFT-SECURITY-LOWERING-UNSUPPORTED");}
  Ok(SecurityRegistry::default())
 });assert_eq!(calls,1);refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
}
#[test]
fn result_declaration_requires_all_false_permit_transform_sources() {
 use weft_core::{security_backend::SecurityRegistry,security_lowering::*};
 let mut req=security04_request();let mut policy:Value=serde_json::from_str(req["security"]["policyJson"].as_str().unwrap()).unwrap();
 let field=json!({"documentId":"domain","moduleId":"m","elementId":"resourceId"});
 for id in ["mask-a","mask-b"] {let mut rule=policy["rules"][0].clone();rule["id"]=json!(id);rule["condition"]=json!({"op":"literal","value":false});rule["disclosure"]=json!([{"field":field,"disposition":{"kind":"transformed","transform":"constant","version":"0.1.0","field":field,"value":{"string":"hidden"}}}]);policy["rules"].as_array_mut().unwrap().push(rule);}
 req["security"]["policyJson"]=json!(policy.to_string());let mut profile:Value=serde_json::from_str(req["security"]["queryProfileJson"].as_str().unwrap()).unwrap();profile["policySha256"]=json!(sha256(policy.to_string().as_bytes()));req["security"]["queryProfileJson"]=json!(profile.to_string());
 let mut calls=0;let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |ctx|{
  calls+=1;let mut baseline=direct_result_contract(ctx);let field=baseline.columns[0].source_fields[0].clone();let mut target=field.clone();target.element="Resource".into();
  let transformed=SecurityResultOutcome::Transformed{id:"mask".into(),transform:SecurityTransform::Constant{version:"0.1.0".into(),output_field:field.clone(),literal:json!({"string":"hidden"})},disposition_sources:["mask-a","mask-b"].iter().map(|id|SecurityDispositionSource{rule_id:(*id).into(),target:target.clone(),field:field.clone()}).collect(),domain:SecurityResultDomain::Model{field}};
  baseline.columns[0].outcomes.push(transformed);ctx.check_result_declaration(&baseline)?;
  for mutation in 0..8 {let mut bad=baseline.clone();let SecurityResultOutcome::Transformed{transform,disposition_sources,domain,..}=&mut bad.columns[0].outcomes[1] else{unreachable!()};match mutation{
   0=>{disposition_sources.pop();},1=>disposition_sources[1].rule_id="mask-a".into(),2=>disposition_sources[0].target.revision="other".into(),
   3=>{let SecurityTransform::Constant{literal,..}=transform;*literal=json!({"string":"different"});},
   4=>{let SecurityResultDomain::Model{field}=domain else{unreachable!()};field.element="salary".into();},
   5=>{let SecurityTransform::Constant{literal,..}=transform;*literal=json!({"integerToken":"1"});},
   6|7=>{let name=if mutation==6{"missing"}else{"Resource"};let SecurityTransform::Constant{output_field,..}=transform;output_field.element=name.into();let SecurityResultDomain::Model{field}=domain else{unreachable!()};field.element=name.into();},_=>unreachable!()
  }assert_eq!(ctx.check_result_declaration(&bad).unwrap_err().code,"WFT-SECURITY-LOWERING-UNSUPPORTED");}
  Ok(SecurityRegistry::default())
 });assert_eq!(calls,1);refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
}

#[test]
fn result_declaration_preserves_numeric_classes_even_with_withholding() {
 use weft_core::{security_backend::SecurityRegistry,security_lowering::*};
 let mut req=security04_request();let mut policy:Value=serde_json::from_str(req["security"]["policyJson"].as_str().unwrap()).unwrap();
 let input=json!({"documentId":"domain","moduleId":"m","elementId":"resourceId"});let output=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});
 for (id,token) in [("number-a","9007199254740993"),("number-b","9007199254740993.0e0"),("number-c","9007199254740992")] {
  let mut r=policy["rules"][0].clone();r["id"]=json!(id);r["condition"]=json!({"op":"literal","value":false});r["disclosure"]=json!([{"field":input,"disposition":{"kind":"transformed","transform":"constant","version":"0.1.0","field":output,"value":{"integerToken":token}}}]);policy["rules"].as_array_mut().unwrap().push(r);
 }
 let mut hidden=policy["rules"][0].clone();hidden["id"]=json!("hide-key");hidden["disclosure"]=json!([{"field":input,"disposition":{"kind":"withheld"}}]);policy["rules"].as_array_mut().unwrap().push(hidden);
 req["security"]["policyJson"]=json!(policy.to_string());let mut profile:Value=serde_json::from_str(req["security"]["queryProfileJson"].as_str().unwrap()).unwrap();profile["policySha256"]=json!(sha256(policy.to_string().as_bytes()));req["security"]["queryProfileJson"]=json!(profile.to_string());
 let mut calls=0;let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |ctx| {
  calls+=1;let mut contract=direct_result_contract(ctx);let input=contract.columns[0].source_fields[0].clone();let mut target=input.clone();target.element="Resource".into();let mut output=input.clone();output.element="salary".into();
  contract.columns[0].outcomes.push(SecurityResultOutcome::Withheld{id:"withheld".into()});
  for (id,token,sources) in [("equal","9007199254740993",vec!["number-a","number-b"]),("unequal","9007199254740992",vec!["number-c"])] {
   contract.columns[0].outcomes.push(SecurityResultOutcome::Transformed{id:id.into(),transform:SecurityTransform::Constant{version:"0.1.0".into(),output_field:output.clone(),literal:json!({"integerToken":token})},disposition_sources:sources.into_iter().map(|s|SecurityDispositionSource{rule_id:s.into(),target:target.clone(),field:input.clone()}).collect(),domain:SecurityResultDomain::Model{field:output.clone()}});
  }
  ctx.check_result_declaration(&contract)?;
  let bytes=serde_json::to_string(&contract).unwrap();let hash=sha256(bytes.as_bytes());
  let mut batch=json!({"version":"weft.security.cells/0.1.0","resultContractSha256":hash,"rows":[[{"outcomeId":"equal","disposition":"transformed","value":{"integerToken":"9007199254740993.0e0"}}],[{"outcomeId":"withheld","disposition":"withheld"}]]});
  ctx.check_result_cells(&contract,&bytes,&hash,&batch.to_string())?;
  batch["rows"][0][0]["value"]=json!({"integerToken":"9007199254740992"});assert_eq!(ctx.check_result_cells(&contract,&bytes,&hash,&batch.to_string()).unwrap_err().code,"WFT-SECURITY-RESULT");
  batch["rows"][0][0]["value"]=json!({"integerToken":"9223372036854775808"});assert_eq!(ctx.check_result_cells(&contract,&bytes,&hash,&batch.to_string()).unwrap_err().code,"WFT-SECURITY-RESULT");
  batch["rows"][0][0]["value"]=json!({"integerToken":"9007199254740993"});batch["rows"][1][0]["value"]=Value::Null;assert_eq!(ctx.check_result_cells(&contract,&bytes,&hash,&batch.to_string()).unwrap_err().code,"WFT-SECURITY-RESULT");
  let mut missing=contract.clone();missing.columns[0].outcomes.pop();assert!(ctx.check_result_declaration(&missing).is_err());
  let mut wrong=contract.clone();let SecurityResultOutcome::Transformed{disposition_sources,..}=&mut wrong.columns[0].outcomes[2] else{unreachable!()};disposition_sources.pop();assert!(ctx.check_result_declaration(&wrong).is_err());
  Ok(SecurityRegistry::default())
 });assert_eq!(calls,1,"{raw}");refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
}
#[test]
fn result_declaration_refuses_original_with_unresolved_optional_presence() {
 use weft_core::security_backend::SecurityRegistry;
 let mut req=security04_request();req["sql"]=json!("SELECT r.salary FROM Resource r");
 let mut doc:Value=serde_json::from_str(req["modules"][0]["documentJson"].as_str().unwrap()).unwrap();for e in doc["modules"][0]["elements"].as_array_mut().unwrap(){if e["id"]=="salary"{e["nullability"]=json!("absent-allowed");}}
 req["modules"][0]["documentJson"]=json!(doc.to_string());req["modules"][0]["pin"]["sha256"]=json!(sha256(doc.to_string().as_bytes()));
 let mut ontology:Value=serde_json::from_str(req["security"]["ontologyJson"].as_str().unwrap()).unwrap();for e in ontology["entities"].as_array_mut().unwrap(){if let Some(fields)=e["fields"].as_array_mut(){for f in fields{if f["ref"]["elementId"]=="salary"{f["protection"]=json!("unprotected");}}}}
 let mut policy:Value=serde_json::from_str(req["security"]["policyJson"].as_str().unwrap()).unwrap();policy["rules"][0].as_object_mut().unwrap().remove("disclosure");req["security"]["policyJson"]=json!(policy.to_string());req["security"]["ontologyJson"]=json!(ontology.to_string());
 let mut profile:Value=serde_json::from_str(req["security"]["queryProfileJson"].as_str().unwrap()).unwrap();profile["modelPins"]=json!(req["modules"].as_array().unwrap().iter().map(|m|m["pin"].clone()).collect::<Vec<_>>());profile["policySha256"]=json!(sha256(policy.to_string().as_bytes()));profile["ontologySha256"]=json!(sha256(ontology.to_string().as_bytes()));req["security"]["queryProfileJson"]=json!(profile.to_string());
 let mut calls=0;let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |ctx|{calls+=1;let contract=direct_result_contract(ctx);assert!(ctx.check_result_declaration(&contract).is_err());Ok(SecurityRegistry::default())});assert_eq!(calls,1,"{raw}");refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
}

#[test]
fn result_declaration_accepts_256_outcomes_and_refuses_257_without_truncation() {
 use weft_core::{security_backend::SecurityRegistry,security_lowering::*};
 for count in [255usize,256usize] {
  let mut req=security04_request();let mut policy:Value=serde_json::from_str(req["security"]["policyJson"].as_str().unwrap()).unwrap();let template=policy["rules"][0].clone();let input=json!({"documentId":"domain","moduleId":"m","elementId":"resourceId"});let output=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});
  policy["rules"]=json!((0..count).map(|n|{let mut r=template.clone();r["id"]=json!(format!("mask-{n}"));r["condition"]=json!({"op":"literal","value":false});r["disclosure"]=json!([{"field":input,"disposition":{"kind":"transformed","transform":"constant","version":"0.1.0","field":output,"value":{"integerToken":n.to_string()}}}]);r}).collect::<Vec<_>>());
  req["security"]["policyJson"]=json!(policy.to_string());let mut profile:Value=serde_json::from_str(req["security"]["queryProfileJson"].as_str().unwrap()).unwrap();profile["policySha256"]=json!(sha256(policy.to_string().as_bytes()));req["security"]["queryProfileJson"]=json!(profile.to_string());
  let mut calls=0;let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |ctx|{
   calls+=1;let mut contract=direct_result_contract(ctx);let input=contract.columns[0].source_fields[0].clone();let mut target=input.clone();target.element="Resource".into();let mut output=input.clone();output.element="salary".into();
   for n in 0..count {contract.columns[0].outcomes.push(SecurityResultOutcome::Transformed{id:format!("mask-{n}"),transform:SecurityTransform::Constant{version:"0.1.0".into(),output_field:output.clone(),literal:json!({"integerToken":n.to_string()})},disposition_sources:vec![SecurityDispositionSource{rule_id:format!("mask-{n}"),target:target.clone(),field:input.clone()}],domain:SecurityResultDomain::Model{field:output.clone()}});}
   if count==255 {ctx.check_result_declaration(&contract)?;}else{contract.columns[0].outcomes.pop();assert_eq!(contract.columns[0].outcomes.len(),256);assert!(ctx.check_result_declaration(&contract).is_err());}
   Ok(SecurityRegistry::default())
  });assert_eq!(calls,1,"{raw}");refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
 }
}

#[test]
fn result_declaration_cannot_skip_unresolved_aggregates_after_valid_fields() {
 use weft_core::{security_backend::SecurityRegistry,security_lowering::*};
 for sql in ["SELECT COUNT(*) FROM Resource r","SELECT r.resourceId, COUNT(*) FROM Resource r GROUP BY r.resourceId"] {
  let mut req=security04_request();req["sql"]=json!(sql);let mut calls=0;
  let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |ctx|{
   calls+=1;let mut contract=SecurityResultContract{version:"weft.security.result-contract/0.1.0".into(),encoding:"weft.security.cells/0.1.0".into(),columns:vec![]};
   for o in ctx.requirements().outputs() {let (fields,domain)=match &o.output().expression {
    weft_core::application_ir::Expression::Field{identity,..}=>{let field=SecurityIdentity{document_id:identity.document_id.clone(),revision:identity.revision.clone(),module:identity.module.clone(),element:identity.element.clone()};(vec![field.clone()],SecurityResultDomain::Model{field})},
    weft_core::application_ir::Expression::Count{logical_type}=>(vec![],SecurityResultDomain::Scalar{r#type:logical_type.clone()}),_=>panic!("Unexpected aggregate fixture")
   };contract.columns.push(SecurityResultColumn{position:o.position(),output_name:o.output().name.clone(),source_fields:fields,outcomes:vec![SecurityResultOutcome::Original{id:"original".into(),domain}]});}
   assert!(ctx.check_result_declaration(&contract).is_err());Ok(SecurityRegistry::default())
  });assert_eq!(calls,1,"{sql}: {raw}");refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
 }
}

#[test]
fn result_declaration_keeps_null_transform_distinct_from_protected_absence() {
 use weft_core::{security_backend::SecurityRegistry,security_lowering::*};
 for null_mask in [false,true] {
  let mut req=security04_request();req["sql"]=json!("SELECT r.salary FROM Resource r");
  let mut doc:Value=serde_json::from_str(req["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
  for e in doc["modules"][0]["elements"].as_array_mut().unwrap(){if e["id"]=="salary"{e["nullability"]=json!("absent-allowed");}}
  req["modules"][0]["documentJson"]=json!(doc.to_string());req["modules"][0]["pin"]["sha256"]=json!(sha256(doc.to_string().as_bytes()));
  let mut policy:Value=serde_json::from_str(req["security"]["policyJson"].as_str().unwrap()).unwrap();
  policy["rules"][0].as_object_mut().unwrap().remove("disclosure");
  if null_mask {
   let field=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});
   policy["rules"][0]["disclosure"]=json!([{"field":field,"disposition":{"kind":"transformed","transform":"constant","version":"0.1.0","field":field,"value":null}}]);
  }
  req["security"]["policyJson"]=json!(policy.to_string());
  let mut profile:Value=serde_json::from_str(req["security"]["queryProfileJson"].as_str().unwrap()).unwrap();
  profile["modelPins"]=json!(req["modules"].as_array().unwrap().iter().map(|m|m["pin"].clone()).collect::<Vec<_>>());profile["policySha256"]=json!(sha256(policy.to_string().as_bytes()));req["security"]["queryProfileJson"]=json!(profile.to_string());
  let mut calls=0;let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |ctx| {
   calls+=1;let mut contract=direct_result_contract(ctx);
   // Protected fields never receive an invented Original default.
   assert_eq!(ctx.check_result_declaration(&contract).unwrap_err().code,"WFT-SECURITY-LOWERING-UNSUPPORTED");
   if null_mask {
    let field=contract.columns[0].source_fields[0].clone();let mut target=field.clone();target.element="Resource".into();
    let rule_id=ctx.requirements().scans()[0].actions().iter().find(|a|a.inventory().action()==ctx.requirements().primary_action()).unwrap().rules()[0].id.clone();
    contract.columns[0].outcomes=vec![SecurityResultOutcome::Transformed{id:"null-mask".into(),transform:SecurityTransform::Constant{version:"0.1.0".into(),output_field:field.clone(),literal:Value::Null},disposition_sources:vec![SecurityDispositionSource{rule_id,target,field:field.clone()}],domain:SecurityResultDomain::Model{field}}];
    ctx.check_result_declaration(&contract)?;
    let bytes=serde_json::to_string(&contract).unwrap();let hash=sha256(bytes.as_bytes());
    let mut batch=json!({"version":"weft.security.cells/0.1.0","resultContractSha256":hash,"rows":[[{"outcomeId":"null-mask","disposition":"transformed","value":null}]]});
    ctx.check_result_cells(&contract,&bytes,&hash,&batch.to_string())?;
    batch["rows"][0][0]["value"]=json!({"integerToken":"0"});assert_eq!(ctx.check_result_cells(&contract,&bytes,&hash,&batch.to_string()).unwrap_err().code,"WFT-SECURITY-RESULT");
    for replacement in [SecurityResultOutcome::Absent{id:"absent".into()},SecurityResultOutcome::Withheld{id:"withheld".into()}] {
     let mut bad=contract.clone();bad.columns[0].outcomes[0]=replacement;
     assert_eq!(ctx.check_result_declaration(&bad).unwrap_err().code,"WFT-SECURITY-LOWERING-UNSUPPORTED");
    }
   }
   Ok(SecurityRegistry::default())
  });assert_eq!(calls,1,"{raw}");refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
 }
}

#[test]
fn result_cells_require_exact_custody_width_tags_and_domains() {
 use weft_core::security_backend::SecurityRegistry;
 let mut req=security04_request();req["sql"]=json!("SELECT r.resourceId AS first, r.resourceId AS second FROM Resource r");
 let mut calls=0;let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |ctx|{
  calls+=1;let contract=direct_result_contract(ctx);let bytes=serde_json::to_string(&contract).unwrap();let hash=sha256(bytes.as_bytes());
  let cell=json!({"outcomeId":"original","disposition":"original","value":{"string":"resource-a"}});
  let good=json!({"version":"weft.security.cells/0.1.0","resultContractSha256":hash,"rows":[[cell,cell]]});
  ctx.check_result_cells(&contract,&bytes,&hash,&good.to_string())?;
  // Every large cell is independently valid; only their aggregate ledger refuses.
  let large=json!({"outcomeId":"original","disposition":"original","value":{"string":"x".repeat(1_000_000)}});
  let mut aggregate=good.clone();aggregate["rows"]=json!([[large,large]]);
  ctx.check_result_cells(&contract,&bytes,&hash,&aggregate.to_string())?;
  aggregate["rows"]=json!(vec![vec![large.clone(),large.clone()];3]);
  ctx.check_result_cells(&contract,&bytes,&hash,&aggregate.to_string())?;
  aggregate["rows"]=json!(vec![vec![large.clone(),large.clone()];4]);
  let aggregate_bytes=aggregate.to_string();assert!(aggregate_bytes.len()<32*1024*1024);
  assert_eq!(ctx.check_result_cells(&contract,&bytes,&hash,&aggregate_bytes).unwrap_err().code,"WFT-LIMIT");
  let mut empty=good.clone();empty["rows"]=json!([]);ctx.check_result_cells(&contract,&bytes,&hash,&empty.to_string())?;
  for mutation in 0..10 {let mut bad=good.clone();match mutation {
   0=>bad["rows"][0].as_array_mut().unwrap().pop().map(|_|()).unwrap(),
   1=>{bad["rows"][0].as_array_mut().unwrap().push(cell.clone());},
   2=>bad["rows"][0][0]["outcomeId"]=json!("unknown"),
   3=>bad["rows"][0][0]["disposition"]=json!("withheld"),
   4=>bad["rows"][0][0]["value"]=Value::Null,
   5=>bad["rows"][0][0]["value"]=json!({"integerToken":"1"}),
   6=>{bad["rows"][0][0].as_object_mut().unwrap().remove("value");},
   7=>bad["rows"][0][0]["extra"]=json!(false),
   8=>bad["resultContractSha256"]=json!("0".repeat(64)),
   9=>bad["extra"]=json!(false),_=>unreachable!()
  }assert_eq!(ctx.check_result_cells(&contract,&bytes,&hash,&bad.to_string()).unwrap_err().code,"WFT-SECURITY-RESULT");}
  assert_eq!(ctx.check_result_cells(&contract,&bytes,&"0".repeat(64),&good.to_string()).unwrap_err().code,"WFT-SECURITY-RESULT");
  let changed=bytes.replace("first","foreign");assert_eq!(ctx.check_result_cells(&contract,&changed,&sha256(changed.as_bytes()),&good.to_string()).unwrap_err().code,"WFT-SECURITY-RESULT");
  let duplicate=bytes.replacen("\"version\":","\"version\":\"duplicate\",\"version\":",1);
  assert_eq!(ctx.check_result_cells(&contract,&duplicate,&sha256(duplicate.as_bytes()),&good.to_string()).unwrap_err().code,"WFT-JSON-DUPLICATE");
  let duplicate_batch=good.to_string().replacen("\"rows\":","\"rows\":[],\"rows\":",1);
  assert_eq!(ctx.check_result_cells(&contract,&bytes,&hash,&duplicate_batch).unwrap_err().code,"WFT-JSON-DUPLICATE");
  Ok(SecurityRegistry::default())
 });assert_eq!(calls,1,"{raw}");refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
}

#[test]
fn simulated_result_selection_enforces_complete_scoped_folds_without_release() {
 use weft_core::{security_backend::{SecurityRegistry,SecuritySimulatedRowTruths},security_lowering::*,security_composition::Truth};
 use std::collections::BTreeMap;
 for mode in 0..3 {
  let join=mode==1; let secondary=mode==2;
  let mut req=security04_request();req["sql"]=json!(if join{"SELECT r.salary AS first, s.salary AS second FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId"}else{"SELECT r.salary FROM Resource r"});
  let mut policy:Value=serde_json::from_str(req["security"]["policyJson"].as_str().unwrap()).unwrap();let field=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});
  for (id,disposition) in [("raw-permit",json!({"kind":"original"})),("mask-zero",json!({"kind":"transformed","transform":"constant","version":"0.1.0","field":field,"value":{"integerToken":"0"}})),("mask-one",json!({"kind":"transformed","transform":"constant","version":"0.1.0","field":field,"value":{"integerToken":"1"}}))] {
   let mut rule=policy["rules"][0].clone();rule["id"]=json!(id);rule["disclosure"]=json!([{"field":field,"disposition":disposition}]);policy["rules"].as_array_mut().unwrap().push(rule);
  }
  let mut forbid=policy["rules"][0].clone();forbid["id"]=json!("block");forbid["effect"]=json!("forbid");forbid.as_object_mut().unwrap().remove("disclosure");policy["rules"].as_array_mut().unwrap().push(forbid);
  req["security"]["policyJson"]=json!(policy.to_string());let mut profile:Value=serde_json::from_str(req["security"]["queryProfileJson"].as_str().unwrap()).unwrap();profile["policySha256"]=json!(sha256(policy.to_string().as_bytes()));req["security"]["queryProfileJson"]=json!(profile.to_string());
  if secondary {
   req["sql"]=json!("SELECT r.salary FROM Resource r WHERE r.salary=100");
   let mut ontology:Value=serde_json::from_str(req["security"]["ontologyJson"].as_str().unwrap()).unwrap();ontology["actions"].as_array_mut().unwrap().push(json!("query-original"));
   for entity in ontology["entities"].as_array_mut().unwrap(){if let Some(fields)=entity["fields"].as_array_mut(){for f in fields{if f["ref"]==field{f["queryUse"]=json!({"predicate":"original-authorized"});}}}}
   req["security"]["ontologyJson"]=json!(ontology.to_string());
   let mut rule=policy["rules"][0].clone();rule["id"]=json!("secondary-permit");rule["actions"]=json!(["query-original"]);rule.as_object_mut().unwrap().remove("disclosure");policy["rules"].as_array_mut().unwrap().push(rule);
   req["security"]["policyJson"]=json!(policy.to_string());profile["policySha256"]=json!(sha256(policy.to_string().as_bytes()));
   profile["bindings"]=json!([{"target":{"documentId":"domain","moduleId":"m","elementId":"Resource"},"field":field,"operator":"predicate","originalAction":"query-original"}]);
  }
  // Valid source whitespace isolates the selection source-byte ledger from cells.
  let ontology=format!("{}{}"," ".repeat(5000),req["security"]["ontologyJson"].as_str().unwrap());req["security"]["ontologyJson"]=json!(ontology);profile["ontologySha256"]=json!(sha256(ontology.as_bytes()));req["security"]["queryProfileJson"]=json!(profile.to_string());
  let mut calls=0;let response=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |ctx| {
   calls+=1;let mut contract=direct_result_contract(ctx);
   for column in &mut contract.columns {let field=column.source_fields[0].clone();let mut target=field.clone();target.element="Resource".into();column.outcomes.push(SecurityResultOutcome::Withheld{id:"hidden".into()});
    for (id,token) in [("mask-zero","0"),("mask-one","1")] {column.outcomes.push(SecurityResultOutcome::Transformed{id:id.into(),transform:SecurityTransform::Constant{version:"0.1.0".into(),output_field:field.clone(),literal:json!({"integerToken":token})},disposition_sources:vec![SecurityDispositionSource{rule_id:id.into(),target:target.clone(),field:field.clone()}],domain:SecurityResultDomain::Model{field:field.clone()}});}
   }
   ctx.check_result_declaration(&contract)?;let bytes=serde_json::to_string(&contract).unwrap();let hash=sha256(bytes.as_bytes());
   let scopes:Vec<_>=ctx.requirements().scans().iter().map(|s|s.inventory().scan().to_owned()).collect();
   let mut truth:SecuritySimulatedRowTruths=BTreeMap::new();for scan in ctx.requirements().scans(){let mut actions=BTreeMap::new();for a in scan.actions(){let rules=a.rules().iter().map(|r|(r.id.clone(),if r.id=="membership"||r.id=="raw-permit"||r.id=="secondary-permit"{Truth::True}else{Truth::False})).collect();actions.insert(a.inventory().action().into(),rules);}truth.insert(scan.inventory().scan().into(),actions);}
   let cell=|id:&str,disposition:&str,value:Option<Value>|{let mut c=json!({"outcomeId":id,"disposition":disposition});if let Some(v)=value{c["value"]=v;}c};
   let original=cell("original","original",Some(json!({"integerToken":"5"})));let mask=cell("mask-zero","transformed",Some(json!({"integerToken":"0"})));let hidden=cell("hidden","withheld",None);
   let batch=|cells:Vec<Value>|json!({"version":"weft.security.cells/0.1.0","resultContractSha256":hash,"rows":[cells]}).to_string();
   let raw_batch=batch(vec![original.clone();contract.columns.len()]);let mask_batch=batch(vec![mask.clone();contract.columns.len()]);let hidden_batch=batch(vec![hidden;contract.columns.len()]);
   ctx.check_simulated_result_selection(&contract,&bytes,&hash,&raw_batch,&[truth.clone()])?;
   let mut two_rows:Value=serde_json::from_str(&raw_batch).unwrap();let second_row=two_rows["rows"][0].clone();two_rows["rows"].as_array_mut().unwrap().push(second_row);let two_bytes=two_rows.to_string();ctx.check_result_cells(&contract,&bytes,&hash,&two_bytes)?;
   ctx.check_simulated_result_selection(&contract,&bytes,&hash,&two_bytes,&[truth.clone(),truth.clone()])?;
   let mut later=truth.clone();later.get_mut(&scopes[0]).unwrap().get_mut("read").unwrap().insert("membership".into(),Truth::False);
   assert_eq!(ctx.check_simulated_result_selection(&contract,&bytes,&hash,&two_bytes,&[truth.clone(),later]).unwrap_err().code,"WFT-SECURITY-RESULT-SELECTION");
   if secondary {let mut bad=truth.clone();bad.get_mut(&scopes[0]).unwrap().get_mut("query-original").unwrap().insert("secondary-permit".into(),Truth::False);assert_eq!(ctx.check_simulated_result_selection(&contract,&bytes,&hash,&raw_batch,&[bad]).unwrap_err().code,"WFT-SECURITY-RESULT-SELECTION");}
   if mode==0 {let mut many:Value=serde_json::from_str(&raw_batch).unwrap();many["rows"]=json!(vec![vec![original.clone()];4096]);let many_bytes=many.to_string();ctx.check_result_declaration(&contract)?;ctx.check_result_cells(&contract,&bytes,&hash,&many_bytes)?;let err=ctx.check_simulated_result_selection(&contract,&bytes,&hash,&many_bytes,&vec![truth.clone();4096]).unwrap_err();assert_eq!(err.code,"WFT-LIMIT");assert_eq!(err.phase,"result");}

   let mut masked=truth.clone();for actions in masked.values_mut(){actions.get_mut("read").unwrap().insert("mask-zero".into(),Truth::True);}
   ctx.check_simulated_result_selection(&contract,&bytes,&hash,&mask_batch,&[masked.clone()])?;
   // Shape/domain-valid cells cannot select another potential envelope outcome.
   assert_eq!(ctx.check_simulated_result_selection(&contract,&bytes,&hash,&raw_batch,&[masked.clone()]).unwrap_err().code,"WFT-SECURITY-RESULT-SELECTION");
   assert_eq!(ctx.check_simulated_result_selection(&contract,&bytes,&hash,&mask_batch,&[truth.clone()]).unwrap_err().code,"WFT-SECURITY-RESULT-SELECTION");
   let mut withheld=masked.clone();for actions in withheld.values_mut(){let r=actions.get_mut("read").unwrap();r.insert("reader".into(),Truth::True);r.insert("mask-one".into(),Truth::True);}
   ctx.check_simulated_result_selection(&contract,&bytes,&hash,&hidden_batch,&[withheld.clone()])?;
   assert_eq!(ctx.check_simulated_result_selection(&contract,&bytes,&hash,&raw_batch,&[withheld]).unwrap_err().code,"WFT-SECURITY-RESULT-SELECTION");
   for mutation in 0..7 {let mut bad=truth.clone();let rules=bad.get_mut(scopes.last().unwrap()).unwrap().get_mut("read").unwrap();match mutation{
    0=>{rules.insert("membership".into(),Truth::Unknown);},1=>{rules.insert("membership".into(),Truth::False);},2=>{rules.insert("raw-permit".into(),Truth::False);},3=>{rules.insert("block".into(),Truth::True);},4=>{rules.insert("mask-zero".into(),Truth::True);rules.insert("mask-one".into(),Truth::True);},5=>{rules.remove("membership");},6=>{rules.remove("membership");rules.insert("foreign".into(),Truth::True);},_=>unreachable!()
   }assert_eq!(ctx.check_simulated_result_selection(&contract,&bytes,&hash,&raw_batch,&[bad]).unwrap_err().code,"WFT-SECURITY-RESULT-SELECTION");}
   assert_eq!(ctx.check_simulated_result_selection(&contract,&bytes,&hash,&raw_batch,&[]).unwrap_err().code,"WFT-SECURITY-RESULT-SELECTION");
   let mut extra=truth.clone();extra.insert("foreign-scan".into(),BTreeMap::new());assert_eq!(ctx.check_simulated_result_selection(&contract,&bytes,&hash,&raw_batch,&[extra]).unwrap_err().code,"WFT-SECURITY-RESULT-SELECTION");
   let mut extra=truth.clone();extra.get_mut(&scopes[0]).unwrap().insert("foreign-action".into(),BTreeMap::new());assert_eq!(ctx.check_simulated_result_selection(&contract,&bytes,&hash,&raw_batch,&[extra]).unwrap_err().code,"WFT-SECURITY-RESULT-SELECTION");
   if join {let mut mixed=truth.clone();mixed.get_mut(&scopes[1]).unwrap().get_mut("read").unwrap().insert("mask-zero".into(),Truth::True);let mixed_batch=batch(vec![original,mask]);ctx.check_simulated_result_selection(&contract,&bytes,&hash,&mixed_batch,&[mixed.clone()])?;
    let first=mixed.remove(&scopes[0]).unwrap();let second=mixed.remove(&scopes[1]).unwrap();mixed.insert(scopes[0].clone(),second);mixed.insert(scopes[1].clone(),first);
    assert_eq!(ctx.check_simulated_result_selection(&contract,&bytes,&hash,&mixed_batch,&[mixed]).unwrap_err().code,"WFT-SECURITY-RESULT-SELECTION");
   }
   Ok(SecurityRegistry::default())
  });assert_eq!(calls,1,"{response}");refused(serde_json::from_str(&response).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
 }
}

#[test]
fn owner_scoped_fact_selection_evaluates_membership_and_original_values_without_release(){
 use weft_core::{security_backend::SecurityRegistry,security_lowering::*};
 for join in [false,true]{for masked in [false,true]{
  let mut req=security04_request();req["sql"]=json!(if join{"SELECT r.resourceId AS first, s.resourceId AS second FROM Resource r JOIN Resource s ON r.resourceId=r.resourceId"}else{"SELECT r.resourceId FROM Resource r"});
  if masked{let mut policy:Value=serde_json::from_str(req["security"]["policyJson"].as_str().unwrap()).unwrap();let mut rule=policy["rules"][0].clone();rule["id"]=json!("conditional-mask");let field=json!({"documentId":"domain","moduleId":"m","elementId":"resourceId"});let salary=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});
   rule["condition"]=json!({"op":"eq","left":{"kind":"resource","field":salary},"right":{"kind":"constant","field":salary,"value":{"integerToken":"100"}}});
   rule["disclosure"]=json!([{"field":field,"disposition":{"kind":"transformed","transform":"constant","version":"0.1.0","field":field,"value":{"string":"masked"}}}]);policy["rules"].as_array_mut().unwrap().push(rule);req["security"]["policyJson"]=json!(policy.to_string());let mut profile:Value=serde_json::from_str(req["security"]["queryProfileJson"].as_str().unwrap()).unwrap();profile["policySha256"]=json!(sha256(policy.to_string().as_bytes()));req["security"]["queryProfileJson"]=json!(profile.to_string());
  }
  let corpus:Value=serde_json::from_str(include_str!("security-evaluation-oracle.json")).unwrap();let case=&corpus["cases"][0];let mut cut=case["cut"].clone();let first=case["request"]["resources"][0].clone();let mut second=first.clone();second["key"][0]=json!({"string":"r2"});for f in second["fields"].as_array_mut().unwrap(){match f["field"]["elementId"].as_str().unwrap(){"resourceId"=>f["value"]=json!({"string":"r2"}),"salary"=>f["value"]=json!({"integerToken":"200"}),_=>{}}}
  let mut owner=cut["facts"][0].clone();owner["key"][0]=json!({"string":"o2"});for f in owner["fields"].as_array_mut().unwrap(){match f["field"]["elementId"].as_str().unwrap(){"ownerId"=>f["value"]=json!({"string":"o2"}),"ownerResource"=>f["value"]=json!({"string":"r2"}),_=>{}}}cut["facts"].as_array_mut().unwrap().push(owner);
  let mut calls=0;let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |ctx|{
   calls+=1;let mut contract=direct_result_contract(ctx);
   if masked{for c in &mut contract.columns{let field=c.source_fields[0].clone();let mut target=field.clone();target.element="Resource".into();c.outcomes.push(SecurityResultOutcome::Transformed{id:"mask".into(),transform:SecurityTransform::Constant{version:"0.1.0".into(),output_field:field.clone(),literal:json!({"string":"masked"})},disposition_sources:vec![SecurityDispositionSource{rule_id:"conditional-mask".into(),target,field:field.clone()}],domain:SecurityResultDomain::Model{field}});}}
   let bytes=serde_json::to_string(&contract).unwrap();let hash=sha256(bytes.as_bytes());let scans:Vec<_>=ctx.requirements().scans().iter().map(|s|s.inventory().scan().to_owned()).collect();
   let original=|v:&str|json!({"outcomeId":"original","disposition":"original","value":{"string":v}});let mask=json!({"outcomeId":"mask","disposition":"transformed","value":{"string":"masked"}});
   let mut scoped=serde_json::Map::new();scoped.insert(scans[0].clone(),first.clone());if join{scoped.insert(scans[1].clone(),second.clone());}
   let rows=json!({"version":"weft.security.scoped-facts/0.1.0","rows":[scoped]});let mut cells=vec![if masked{mask}else{original("r1")}];if join{cells.push(original("r2"));}
   let batch=json!({"version":"weft.security.cells/0.1.0","resultContractSha256":hash,"rows":[cells]});let cut_bytes=cut.to_string();
   ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&cut_bytes,&rows.to_string())?;
   let positional=|v:&Value,keys:&[&str]|json!(keys.iter().map(|k|v[*k].clone()).collect::<Vec<_>>());
   for mutation in 0..11{let mut c=cut.clone();let mut r=rows.clone();match mutation{
    0=>r=positional(&r,&["version","rows"]),
    1=>c=positional(&c,&["trusted","generation","expectedGeneration","policyId","policyRevision","ontologyDocumentId","ontologyRevision","subjects","facts","coverage","context","maxFacts","maxSteps"]),
    2=>r["rows"][0][&scans[0]]=positional(&r["rows"][0][&scans[0]],&["type","key","fields","absent"]),
    3=>r["rows"][0][&scans[0]]["fields"][0]=positional(&r["rows"][0][&scans[0]]["fields"][0],&["field","value"]),
    4=>c["subjects"][0]=positional(&c["subjects"][0],&["type","key","fields","absent"]),
    5=>c["facts"][0]=positional(&c["facts"][0],&["type","key","fields","absent"]),
    6=>c["coverage"][0]=positional(&c["coverage"][0],&["type","complete","fields"]),
    7=>c["coverage"][0]["type"]=positional(&c["coverage"][0]["type"],&["documentId","moduleId","elementId"]),
    8=>r["rows"][0][&scans[0]]["type"]=positional(&r["rows"][0][&scans[0]]["type"],&["documentId","moduleId","elementId"]),
    9=>c["coverage"][0]["fields"][0]=positional(&c["coverage"][0]["fields"][0],&["documentId","moduleId","elementId"]),
    10=>r["rows"][0][&scans[0]]["fields"][0]["field"]=positional(&r["rows"][0][&scans[0]]["fields"][0]["field"],&["documentId","moduleId","elementId"]),_=>unreachable!()
   }assert_eq!(ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&c.to_string(),&r.to_string()).unwrap_err().code,"WFT-SECURITY-EVALUATION");}
   // Domain-valid Original substitution refuses value correspondence (or the actual mask).
   let mut bad=batch.clone();bad["rows"][0][0]=original("other");ctx.check_result_cells(&contract,&bytes,&hash,&bad.to_string())?;assert!(ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&bad.to_string(),&cut_bytes,&rows.to_string()).is_err());
   for mutation in 0..7{let mut bad=cut.clone();match mutation{
    0=>bad["facts"]=json!([]),1=>bad["facts"][1]["fields"][3]["value"]=json!({"boolean":false}),2=>bad["coverage"][4]["complete"]=json!(false),3=>bad["expectedGeneration"]=json!("stale"),4=>bad["maxSteps"]=json!(1),5=>bad["facts"][1]["fields"].as_array_mut().unwrap().clear(),6=>{let duplicate=bad["facts"][0].clone();bad["facts"].as_array_mut().unwrap().push(duplicate);},_=>unreachable!()
   }assert!(ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&bad.to_string(),&rows.to_string()).is_err());}
   let mut missing=rows.clone();missing["rows"][0].as_object_mut().unwrap().remove(&scans[0]);assert!(ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&cut_bytes,&missing.to_string()).is_err());
   let mut foreign=rows.clone();let fact=foreign["rows"][0].as_object_mut().unwrap().remove(&scans[0]).unwrap();foreign["rows"][0]["foreign-scan"]=fact;assert!(ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&cut_bytes,&foreign.to_string()).is_err());
   let mut later_rows=rows.clone();let second_row=rows["rows"][0].clone();later_rows["rows"].as_array_mut().unwrap().push(second_row);let mut later_batch=batch.clone();let second_cells=batch["rows"][0].clone();later_batch["rows"].as_array_mut().unwrap().push(second_cells);
   ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&later_batch.to_string(),&cut_bytes,&later_rows.to_string())?;
   // Same identity cannot supply another field assignment in a later bag occurrence.
   later_rows["rows"][1][&scans[0]]["fields"][1]["value"]=json!({"integerToken":"999"});assert_eq!(ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&later_batch.to_string(),&cut_bytes,&later_rows.to_string()).unwrap_err().code,"WFT-SECURITY-EVALUATION");
   if join{let mut swapped=rows.clone();swapped["rows"][0][&scans[0]]=second.clone();swapped["rows"][0][&scans[1]]=first.clone();assert!(ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&cut_bytes,&swapped.to_string()).is_err());}
   if !masked{
    let mut missing_value=rows.clone();missing_value["rows"][0][&scans[0]]["fields"].as_array_mut().unwrap().retain(|f|f["field"]["elementId"]!="resourceId");
    assert_eq!(ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&cut_bytes,&missing_value.to_string()).unwrap_err().code,"WFT-SECURITY-EVALUATION");
    let mut missing_coverage=cut.clone();missing_coverage["coverage"][2]["fields"].as_array_mut().unwrap().retain(|f|f["elementId"]!="resourceId");
    assert_eq!(ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&missing_coverage.to_string(),&rows.to_string()).unwrap_err().code,"WFT-SECURITY-EVALUATION");
   }
   let mut population=cut.clone();population["facts"].as_array_mut().unwrap().push(first.clone());ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&population.to_string(),&rows.to_string())?;
   population["facts"][3]["fields"][1]["value"]=json!({"integerToken":"1e2"});ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&population.to_string(),&rows.to_string())?;
   population["facts"][3]["fields"][1]["value"]=json!({"integerToken":"999"});assert_eq!(ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&population.to_string(),&rows.to_string()).unwrap_err().code,"WFT-SECURITY-EVALUATION");
   let mut subject_population=cut.clone();let subject=cut["subjects"][0].clone();subject_population["facts"].as_array_mut().unwrap().push(subject);ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&subject_population.to_string(),&rows.to_string())?;
   subject_population["facts"][3]["fields"]=json!([]);assert_eq!(ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&subject_population.to_string(),&rows.to_string()).unwrap_err().code,"WFT-SECURITY-EVALUATION");
   let empty_rows=json!({"version":"weft.security.scoped-facts/0.1.0","rows":[]});let mut empty_batch=batch.clone();empty_batch["rows"]=json!([]);ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&empty_batch.to_string(),&cut_bytes,&empty_rows.to_string())?;
   let mut incomplete=cut.clone();incomplete["coverage"][4]["complete"]=json!(false);assert!(ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&empty_batch.to_string(),&incomplete.to_string(),&empty_rows.to_string()).is_err());
   Ok(SecurityRegistry::default())
  });assert_eq!(calls,1,"{raw}");refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
 }}
}

#[test]
fn evaluated_owner_truth_metadata_refuses_before_unbounded_identifier_expansion(){
 use weft_core::security_backend::SecurityRegistry;
 let mut req=security04_request();let mut policy:Value=serde_json::from_str(req["security"]["policyJson"].as_str().unwrap()).unwrap();let mut base=policy["rules"][0].clone();base.as_object_mut().unwrap().remove("disclosure");policy["rules"]=json!([base]);
 for n in 0..128{let mut rule=policy["rules"][0].clone();rule["id"]=json!(format!("{n:03}{}","x".repeat(125)));rule["condition"]=json!({"op":"literal","value":false});policy["rules"].as_array_mut().unwrap().push(rule);}
 req["security"]["policyJson"]=json!(policy.to_string());let mut profile:Value=serde_json::from_str(req["security"]["queryProfileJson"].as_str().unwrap()).unwrap();profile["policySha256"]=json!(sha256(policy.to_string().as_bytes()));req["security"]["queryProfileJson"]=json!(profile.to_string());
 let corpus:Value=serde_json::from_str(include_str!("security-evaluation-oracle.json")).unwrap();let case=&corpus["cases"][0];let mut cut=case["cut"].clone();cut["maxFacts"]=json!(10000);cut["maxSteps"]=json!(1000000);let resource=case["request"]["resources"][0].clone();
 let mut calls=0;let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |ctx|{
  calls+=1;let contract=direct_result_contract(ctx);ctx.check_result_declaration(&contract)?;let bytes=serde_json::to_string(&contract).unwrap();let hash=sha256(bytes.as_bytes());let scan=ctx.requirements().scans()[0].inventory().scan();
  let cell=json!({"outcomeId":"original","disposition":"original","value":{"string":"r1"}});let row=json!({scan:resource});
  for count in [512usize,1024usize]{let rows=json!({"version":"weft.security.scoped-facts/0.1.0","rows":vec![row.clone();count]});let batch=json!({"version":"weft.security.cells/0.1.0","resultContractSha256":hash,"rows":vec![vec![cell.clone()];count]});let rows_bytes=rows.to_string();assert!(rows_bytes.len()+cut.to_string().len()<4_000_000);ctx.check_result_cells(&contract,&bytes,&hash,&batch.to_string())?;
   let result=ctx.check_simulated_fact_selection(&contract,&bytes,&hash,&batch.to_string(),&cut.to_string(),&rows_bytes);if count==512{result?;}else{let error=result.unwrap_err();assert_eq!(error.code,"WFT-SECURITY-EVALUATION");assert_eq!(error.phase,"model");}
  }Ok(SecurityRegistry::default())
 });assert_eq!(calls,1,"{raw}");refused(serde_json::from_str(&raw).unwrap(),"WFT-SECURITY-BACKEND-REQUIRED");
}
