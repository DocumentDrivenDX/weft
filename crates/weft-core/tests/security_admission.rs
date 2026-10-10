use serde_json::{json,Value};
use weft_core::{compile::Compiler,json::sha256};
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-response-v0.5.schema.json")]
struct SecurityResponse;
// Shared security goal: CONTRACT-007 source custody; native acceptance remains open.
fn request()->Value {
    let cases:Value=serde_json::from_str(include_str!("../../../docs/helix/03-test/fixtures/cases.json")).unwrap();
    let mut request=cases.as_array().unwrap().iter().find(|c|c["expected"]["status"]=="compiled").unwrap()["request"].clone();
    request["interfaceVersion"]=json!("weft-compile/0.5.0");request["dialect"]=json!("weft-sql/0.2.0");
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
fn refused(response:Value,code:&str){if response["interfaceVersion"]=="weft-compile/0.5.0"{assert!(SecurityResponse::is_valid(&response));}assert_eq!(response["status"],"blocked");assert_eq!(response["diagnostics"][0]["code"],code);for key in ["sql","parameters","logicalPlan"]{assert!(response.get(key).is_none());}}
#[test]
fn core08_source_custody_never_activates_uninterpreted_security(){let r=response(&request());assert_eq!(r["interfaceVersion"],"weft-compile/0.5.0");refused(r,"WFT-SECURITY-UNSUPPORTED");}
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
 use weft_core::{model::{Catalog,ModuleInput},
security_policy_types::SecurityPolicyTypes,
security_source::SecuritySourcePacket,};
 let r=request();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let  catalog=Catalog::prepare_security(inputs).unwrap();let packet=SecuritySourcePacket::read(r["security"]["policyJson"].as_str().unwrap(),r["security"]["ontologyJson"].as_str().unwrap(),&catalog
,
).unwrap();
let mut changed_inputs =
catalog.inputs
().to_vec();
    changed_inputs
[0].document_json.push(' '
);
    changed_inputs[0].pin.sha256 = sha256(changed_inputs[0].document_json.as_bytes());
    let catalog = Catalog::prepare_security(changed_inputs).unwrap(
);assert_eq!(SecurityPolicyTypes::read(&packet,&catalog).unwrap_err().code,"WFT-SECURITY-PIN");
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
fn rehashed_catalog_input_cannot_substitute_for_prepared_definitions() {
 use weft_core::{model::{Catalog,ModuleInput},security_source::SecuritySourcePacket,security_policy_types::SecurityPolicyTypes};
 let r=request(); let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();
 let catalog=Catalog::prepare_security(inputs).unwrap();
 let packet=SecuritySourcePacket::read(r["security"]["policyJson"].as_str().unwrap(),r["security"]["ontologyJson"].as_str().unwrap(),&catalog).unwrap();
 let mut inputs=catalog.inputs().to_vec();
 let mut doc:Value=serde_json::from_str(&inputs[0].document_json).unwrap();
 doc["modules"][0]["elements"][0]["description"]=json!("substituted source");
 inputs[0].document_json=doc.to_string();
 assert!(Catalog::prepare_security(inputs.clone()).is_err());
 inputs[0].pin.sha256=sha256(inputs[0].document_json.as_bytes());
 let fresh=Catalog::prepare_security(inputs).unwrap();
 assert_eq!(SecurityPolicyTypes::read(&packet,&fresh).unwrap_err().code,"WFT-SECURITY-PIN");
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
  let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let packet=SecuritySourcePacket::read(r["security"]["policyJson"].as_str().unwrap(),r["security"]["ontologyJson"].as_str().unwrap(),&catalog).unwrap();let closure=SecurityOntologyClosure::read(&packet,&catalog).unwrap();assert_eq!(closure.type_count(),5);assert_eq!(closure.field_count(),11);assert_eq!(catalog.inputs()[0].document_json,text);assert_eq!(serde_json::from_str::<Value>(&catalog.inputs()[0].document_json).unwrap(),document);
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
 use weft_core::{model::{Catalog,ModuleInput},
security_candidate_policy_types::SecurityCandidatePolicyTypes,
security_source::SecurityCandidateSourcePacket,};
 let (r,mut policy,mut ontology)=candidate_sources();let mut inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let mut doc:Value=serde_json::from_str(&inputs[0].document_json).unwrap();
 doc["modules"][0]["relationships"]=json!([{"id":"WorksOn","name":"WorksOn","source":[{"module":"m","element":"Staff"}],"target":[{"module":"m","element":"Project","key":"pk"}],"sourceMultiplicity":{"min":0,"max":"*"},"targetMultiplicity":{"min":0,"max":"*"},"targetLifecycle":"independent","directed":true}]);inputs[0].document_json=doc.to_string();inputs[0].pin.sha256=sha256(inputs[0].document_json.as_bytes());let catalog=Catalog::prepare_security(inputs.clone()).unwrap();
 let reference=json!({"documentId":"domain","moduleId":"m","relationshipId":"WorksOn"});ontology["associations"][1]=json!({"kind":"core-relationship","relationship":reference,"witness":{"kind":"opaque-existential"},"endpoints":[{"role":"staff","side":"source","target":{"documentId":"domain","moduleId":"m","elementId":"Staff"},"keyId":"pk"},{"role":"project","side":"target","target":{"documentId":"domain","moduleId":"m","elementId":"Project"},"keyId":"pk"}]});
 policy["rules"][1]["condition"]=json!({"op":"exists","association":reference,"as":"edge","where":{"op":"eq","left":{"kind":"variable","name":"edge","endpoint":"staff"},"right":{"kind":"subject","identity":true}}});
 let check=|p:&Value|{let packet=SecurityCandidateSourcePacket::read(&p.to_string(),&ontology.to_string(),&catalog)?;SecurityCandidatePolicyTypes::read(&packet,&catalog)};
 assert!(check(&policy).is_ok());
 {use weft_core::security_candidate_ir::{CandidateEndpointCarrier,CandidateExpression,
CandidateSide,
CandidateTerm,CandidateWitness,SecurityCandidateLogicalPlan,};use weft_core::security_ir::Binding;
 let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&ontology.to_string(),&catalog
,
).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();
 let CandidateExpression::Exists{slot,association,witness,condition
,
}=&plan.rules()[1].condition else{panic!()};assert_eq!(*slot,0);assert_eq!(serde_json::to_value(association).unwrap(),reference);assert_eq!(*witness,CandidateWitness::OpaqueExistential);
 let CandidateExpression::Equal(CandidateTerm::Endpoint{binding,association:a,role,target,key_id,carrier
,
},_
,
)=condition.as_ref() else{panic!()};assert_eq!(*binding,Binding::Variable(0));assert_eq!(a,association);assert_eq!(role,"staff");assert_eq!(target.element_id,"Staff");assert_eq!(key_id,"pk");assert_eq!(*carrier,CandidateEndpointCarrier::Incidence{side:CandidateSide::Source});assert_eq!(plan.source().policy(),&policy);
 {use weft_core::security_candidate_incidence::CandidateIncidenceValue;let value=CandidateIncidenceValue::read(&plan,&catalog,association,"staff",CandidateSide::Source,target,"pk",&[json!({"string":"s1"})]
,
).unwrap();assert_eq!(value.target(),target);assert_eq!(value.key_id(),"pk");assert_eq!(value.components().len(),1);assert_eq!(value.component_domains()[0]["scalarType"],"string");assert!(value.require_catalog(&catalog).is_ok());for (role,side,values) in [("staff",CandidateSide::Target,vec![json!({"string":"s1"})]),("missing",CandidateSide::Source,vec![json!({"string":"s1"})]
,
),("staff",CandidateSide::Source,vec![]),("staff",CandidateSide::Source,vec![json!({"integerToken":"1"})]
,
),("staff",CandidateSide::Source,vec![json!({"string":{"nested":true}})]
,
)
,
]{assert!(CandidateIncidenceValue::read(&plan,&catalog,association,role,side,target,"pk",&values).is_err());}assert!(CandidateIncidenceValue::read(&plan,&catalog,association,"staff",CandidateSide::Source,target,"other",&[json!({"string":"s1"})]).is_err());let mut wrong_target=target.clone();wrong_target.module_id="other".into();assert!(CandidateIncidenceValue::read(&plan,&catalog,association,"staff",CandidateSide::Source,&wrong_target,"pk",&[json!({"string":"s1"})]).is_err());let record=weft_core::security_association_ref::SecurityAssociationRef::Record(weft_core::security_ontology::SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"WorksOn".into()
,
}
,
);assert!(CandidateIncidenceValue::read(&plan,&catalog,&record,"staff",CandidateSide::Source,target,"pk",&[json!({"string":"s1"})]).is_err());let mut stale_inputs=catalog.inputs().to_vec();stale_inputs[0].document_json.push(' '
);
            stale_inputs[0].pin.sha256 =
                weft_core::json::sha256(stale_inputs[0].document_json.as_bytes());
            let stale = Catalog::prepare_security(stale_inputs).unwrap(
);assert!(value.require_catalog(&stale).is_err());assert_eq!(CandidateIncidenceValue::read(&plan,&stale,association,"staff",CandidateSide::Source,target,"pk",&[json!({"string":"s1"})]).unwrap_err().code,"WFT-SECURITY-PIN");}

 let dependencies=weft_core::security_candidate_dependencies::SecurityCandidateDependencies::derive(&plan,&catalog,&plan.rules()[1].target,"read"
,
).unwrap();assert_eq!(dependencies.incidences().len(),2);assert_eq!(dependencies.associations().len(),1);assert!(!dependencies.keys().keys().any(|r|r.element_id=="Assignment"||r.element_id=="WorksOn"));assert!(!dependencies.fields().keys().any(|r|r.element_id=="Assignment"||r.element_id=="WorksOn"));
 {use
std::collections::BTreeMap;
            use
weft_core::security_composition::{compose_candidate,
Decision,
Truth};let truths=BTreeMap::from([("reader".into(),Truth::True),("membership".into(),Truth::True)
,
]);let output=weft_core::security_ontology::SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"salary".into()
,
};let result=compose_candidate(&plan,&catalog,&plan.rules()[1].target,"read",std::slice::from_ref(&output),&truths
,
).unwrap();assert_eq!(result.decision,Decision::Permit);assert!(matches!(result.disclosure[0].1,weft_core::security_ir::Disposition::Withheld));let mut unknown=truths.clone();unknown.insert("membership".into(),Truth::Unknown);let result=compose_candidate(&plan,&catalog,&plan.rules()[1].target,"read",&[output],&unknown
,
).unwrap();assert_eq!(result.decision,Decision::Indeterminate);assert!(result.disclosure.is_empty());}

 let mut target_policy=policy.clone();let operand=json!({"kind":"variable","name":"edge","endpoint":"project"});target_policy["rules"][1]["condition"]["where"]=json!({"op":"eq","left":operand,"right":operand});let packet=SecurityCandidateSourcePacket::read(&target_policy.to_string(),&ontology.to_string(),&catalog
,
).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();let CandidateExpression::Exists{condition,..}=&plan.rules()[1].condition else{panic!()};let CandidateExpression::Equal(CandidateTerm::Endpoint{role,target,carrier,..},_
,
)=condition.as_ref() else{panic!()};assert_eq!(role,"project");assert_eq!(target.element_id,"Project");assert_eq!(*carrier,CandidateEndpointCarrier::Incidence{side:CandidateSide::Target});
 }
 for mutation in ["identity","attribute","role","namespace","unbound"]{let mut p=policy.clone();match mutation{
  "identity"=>
{
p["rules"][1]["condition"]["where"]["left"]=json!({"kind":"variable","name":"edge","identity":true})}
  "attribute"=>
{
p["rules"][1]["condition"]["where"]["left"]=json!({"kind":"variable","name":"edge","field":{"documentId":"domain","moduleId":"m","elementId":"active"}})}
  "role"=>p["rules"][1]["condition"]["where"]["left"]["endpoint"]=json!("missing"),
  "namespace"=>
{
p["rules"][1]["condition"]["association"]=json!({"documentId":"domain","moduleId":"m","elementId":"WorksOn"})}
  _=>p["rules"][1]["condition"]["where"]["left"]["name"]=json!("other")
,
};if matches!(mutation,"identity"|"attribute"){p["rules"][1]["condition"]["where"]["right"]=p["rules"][1]["condition"]["where"]["left"].clone();}assert!(check(&p).is_err(),"{mutation}");}
 let mut record_doc=doc.clone();record_doc["modules"][0]["relationships"][0]["associationRecord"]=json!({"module":"m","element":"Assignment"});let mut record_inputs=inputs.clone();record_inputs[0].document_json=record_doc.to_string();record_inputs[0].pin.sha256=sha256(record_inputs[0].document_json.as_bytes());let record_catalog=Catalog::prepare_security(record_inputs).unwrap();let mut record_ontology=ontology.clone();record_ontology["associations"][1]["witness"]=json!({"kind":"record-key","type":{"documentId":"domain","moduleId":"m","elementId":"Assignment"},"keyId":"pk"});
 for operand in [json!({"kind":"variable","name":"edge","identity":true}),json!({"kind":"variable","name":"edge","field":{"documentId":"domain","moduleId":"m","elementId":"active"}})
,
]{let mut p=policy.clone();p["rules"][1]["condition"]["where"]=json!({"op":"eq","left":operand,"right":operand});assert!(check(&p).is_err());let packet=SecurityCandidateSourcePacket::read(&p.to_string(),&record_ontology.to_string(),&record_catalog
,
).unwrap();assert!(SecurityCandidatePolicyTypes::read(&packet,&record_catalog).is_ok());let plan=weft_core::security_candidate_ir::SecurityCandidateLogicalPlan::read(packet,&record_catalog
,
).unwrap();let deps=weft_core::security_candidate_dependencies::SecurityCandidateDependencies::derive(&plan,&record_catalog,&plan.rules()[1].target,"read"
,
).unwrap();assert_eq!(deps.incidences().len(),2);assert!(deps.keys().keys().any(|r|r.element_id=="Assignment"));assert!(deps.fields().iter().find(|(r,_)|r.element_id=="Assignment").unwrap().1.iter().any(|f|f.element_id=="active"));}
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
 use weft_core::{model::{Catalog,ModuleInput},security_candidate_ir::{CandidateEndpointCarrier,CandidateExpression,CandidateTerm,CandidateWitness,SecurityCandidateLogicalPlan,},security_ir::Binding
,
        security_source::SecurityCandidateSourcePacket,
};
 let (r,policy,ontology)=candidate_sources();let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let  catalog=Catalog::prepare_security(inputs).unwrap();let source=format!("\n{}\n",policy);let packet=SecurityCandidateSourcePacket::read(&source,&ontology.to_string(),&catalog).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();assert_eq!(plan.source().policy_json(),source);
 let CandidateExpression::Exists{slot,witness,condition,..}=&plan.rules()[1].condition else{panic!()};assert_eq!(*slot,0);let CandidateWitness::RecordKey{owner,key_id,key}=witness else{panic!()};assert_eq!(owner.element_id,"Ownership");assert_eq!(key_id,"pk");assert_eq!(key["fields"][0]["element"],"ownerId");
 let CandidateExpression::And(args)=condition.as_ref() else{panic!()};let CandidateExpression::Equal(CandidateTerm::Endpoint{binding,carrier,..},_
,
)=&args[0] else{panic!()};assert_eq!(*binding,Binding::Variable(0));let CandidateEndpointCarrier::Members{fields}=carrier else{panic!()};assert_eq!(fields[0].element_id,"ownerResource");
 let CandidateExpression::Exists{slot,condition,..}=&args[1] else{panic!()};assert_eq!(*slot,1);let CandidateExpression::And(inner)=condition.as_ref() else{panic!()};let CandidateExpression::Equal(CandidateTerm::Endpoint{binding:left,..},CandidateTerm::Endpoint{binding:right,..}
,
)=&inner[1] else{panic!()};assert_eq!(*left,Binding::Variable(1));assert_eq!(*right,Binding::Variable(0));
 let mut siblings=policy.clone();let condition=policy["rules"][1]["condition"].clone();siblings["rules"][1]["condition"]=json!({"op":"and","args":[condition,condition]});let packet=SecurityCandidateSourcePacket::read(&siblings.to_string(),&ontology.to_string(),&catalog).unwrap();let sibling_plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();let CandidateExpression::And(branches)=&sibling_plan.rules()[1].condition else{panic!()};for (branch,expected) in branches.iter().zip([0,2]){let CandidateExpression::Exists{slot,..}=branch else{panic!()};assert_eq!(*slot,expected);}

let mut changed_inputs =
catalog.inputs
().to_vec();
    changed_inputs
[0].document_json.push(' '
);
    changed_inputs[0].pin.sha256 = sha256(changed_inputs[0].document_json.as_bytes());
    let catalog = Catalog::prepare_security(changed_inputs).unwrap(
);assert_eq!(plan.require_catalog(&catalog).unwrap_err().code,"WFT-SECURITY-PIN");
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
