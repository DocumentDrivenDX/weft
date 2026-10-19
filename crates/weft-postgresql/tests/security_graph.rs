// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{Value,json};
use weft_core::{model::{Catalog,ModuleInput},ir::ModelPin,json::sha256,security_source::SecuritySourcePacket,security_ir::SecurityLogicalPlan};
use weft_postgresql::{security_graph_binding::{GraphSource,GraphBinding},security_predicate_spike::render_graph};
const REQUEST:&str=include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/mapping-request.json");
const PHYSICAL:&str=include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/adoc.json");
fn fixture()->(Catalog,SecurityLogicalPlan,GraphSource){
 let r:Value=serde_json::from_str(REQUEST).unwrap();let modules:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare_security(modules).unwrap();
 let packet=SecuritySourcePacket::read(r["policyJson"].as_str().unwrap(),r["ontologyJson"].as_str().unwrap(),&catalog).unwrap();let plan=SecurityLogicalPlan::read(packet,&catalog).unwrap();
 let source=GraphSource{version:"weft.truss.security.graph-source/0.1-candidate".into(),native_schema_revision:"1".into(),physical_module:ModuleInput{document_json:PHYSICAL.into(),pin:ModelPin{document_id:"adoc".into(),revision:"r1".into(),umf_version:"0.8.0".into(),sha256:sha256(PHYSICAL.as_bytes())},selected_module_ids:vec!["m".into()]},inventory_json:include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/native-inventory.json").into(),correspondence_json:include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/binding-correspondence.json").into(),logical_resolution_json:include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/resolution.json").into(),logical_transport_json:include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/transport.json").into()};(catalog,plan,source)
}
#[test]
fn original_reified_graph_correspondence_preserves_namespaces_and_guards_all_endpoints(){
 let (catalog,plan,source)=fixture();let binding=GraphBinding::read(&plan,&catalog,&source).unwrap();assert_eq!(binding.integrity_checks().len(),4);
 let output=render_graph(&plan,&catalog,&source).unwrap();assert_eq!(output["securityReady"],false);assert_eq!(output["graphSourceSha256"],binding.source_sha256());
 let sql=output["rules"].to_string();assert!(sql.contains("truss.edge"));assert!(sql.contains("pg_catalog.count(*)=1"));assert!(sql.contains("e.target_id=v"));assert!(sql.contains("e.source_id=v"));
 for check in binding.integrity_checks(){let sql=check["sql"].as_str().unwrap();assert!(!sql.contains("active"));assert!(sql.contains("LEFT JOIN truss.object"));assert!(sql.contains("pg_catalog.count(*)=1"));}
 assert_eq!(catalog.pins()[0].document_id,"domain");assert_eq!(source.physical_module.pin.document_id,"adoc");
}
#[test]
fn original_graph_source_pin_changes_and_ambiguous_mappings_refuse(){
 let (catalog,plan,source)=fixture();
 for mutation in 0..9 {let mut changed=source.clone();let mut c:Value=serde_json::from_str(&changed.correspondence_json).unwrap();match mutation {
  0=>changed.physical_module.document_json.push(' '),
  1=>changed.inventory_json.push(' '),
  2=>changed.logical_resolution_json.push(' '),
  3=>changed.logical_transport_json.push(' '),
  4=>c["entities"][0]["nativePropertyId"]=json!("7"),
  5=>c["endpoints"][0]["direction"]=json!("outgoing"),
  6=>c["endpoints"][0]["runtimeCardinality"]=json!("schema_guaranteed"),
  7=>{let row=c["endpoints"][0].clone();c["endpoints"].as_array_mut().unwrap().push(row);},
  _=>c["entities"][0]["nativeType"]["documentId"]=json!("domain")
 };changed.correspondence_json=c.to_string();assert!(GraphBinding::read(&plan,&catalog,&changed).is_err(),"mutation {mutation}");}
}

fn graph_request(sql: &str) -> String {
 let (_, _, source) = fixture();
 let mut request: Value = serde_json::from_str(REQUEST).unwrap();
 request["sql"] = json!(sql);
 let binding = serde_json::to_string(&source).unwrap();
 request["bindingJson"] = json!(binding);
 let mut profile: Value = serde_json::from_str(request["queryProfileJson"].as_str().unwrap()).unwrap();
 profile["binding"]["sha256"] = json!(sha256(binding.as_bytes()));
 request["queryProfileJson"] = json!(profile.to_string());
 request.to_string()
}

#[test]
fn graph_query_preserves_actual_output_occurrences_contract_and_subject_slot() {
 use weft_core::security_mapping_transport::with_security_mapping_context;
 use weft_postgresql::security_graph_query::prepare_graph_query_checked;
 for (sql, names) in [
  ("SELECT r.resourceId FROM Resource r", vec!["resourceId"]),
  ("SELECT r.resourceId, r.salary FROM Resource r", vec!["resourceId", "salary"]),
  ("SELECT r.salary AS masked, r.resourceId AS second, r.resourceId AS third FROM Resource r", vec!["masked", "second", "third"]),
 ] {
  with_security_mapping_context(&graph_request(sql), |ctx| {
   let source: GraphSource = serde_json::from_str(ctx.binding_json()).unwrap();
   let query = prepare_graph_query_checked(ctx, &source)?;
   let value = query.value();
   assert_eq!(value["securityReady"], false);
   assert_eq!(value["nativeAdmission"], "required");
   assert_eq!(value["contextSha256"], ctx.execution_query_context_sha256());
   assert_eq!(value["execution"]["subjectSlot"]["ordinal"], 2);
   assert_eq!(value["execution"]["subjectSlot"]["supplier"], "original_authenticated_guarded_host_only");
   let bindings=query.parameter_bindings();
   assert_eq!(bindings.policy().len(),1);
   assert_eq!(bindings.policy()[0].ordinal(),1);
   assert_eq!(bindings.policy()[0].source_path(),"logicalRules/1/condition/where/args/1/where/args/2/right");
   assert_eq!(bindings.policy()[0].codec(),weft_postgresql::security_graph_parameters::GraphPolicyParameterCodec::Boolean);
   assert_eq!(bindings.subject().ordinal(),2);
   assert_eq!(bindings.subject().native_type_id(),value["execution"]["subjectSlot"]["nativeTypeId"].as_i64().unwrap());
   assert_eq!(bindings.subject().key_id(),"pk");
   assert_eq!(bindings.subject().key_fields()[0].element_id,"staffId");
   assert_eq!(value["integrityChecks"].as_array().unwrap().len(), 4);
   assert!(value["truthCheck"]["sql"].as_str().unwrap().contains("IS NULL"));
   let columns = &query.result_contract().columns;
   assert_eq!(columns.len(), names.len());
   for (i, (column, name)) in columns.iter().zip(names).enumerate() {
    assert_eq!(column.position, i+1);
    assert_eq!(column.output_name, name);
   }
   ctx.check_result_declaration(query.result_contract())?;
   Ok(())
  }).unwrap();
 }
}

#[test]
fn graph_query_refuses_unsupported_operations_and_context_substitution() {
 use weft_core::security_mapping_transport::with_security_mapping_context;
 use weft_postgresql::security_graph_query::prepare_graph_query_checked;
 for sql in [
  "SELECT r.resourceId FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId",
  "SELECT r.resourceId FROM Resource r WHERE r.resourceId='RA'",
  "SELECT COUNT(*) FROM Resource r",
  "SELECT r.resourceId FROM Resource r GROUP BY r.resourceId",
  "SELECT r.resourceId FROM Resource r ORDER BY r.resourceId",
  "SELECT r.resourceId FROM Resource r LIMIT 1",
 ] {
  let result = with_security_mapping_context(&graph_request(sql), |ctx| {
   let source: GraphSource = serde_json::from_str(ctx.binding_json()).unwrap();
   prepare_graph_query_checked(ctx, &source).map(|_| ())
  });
  assert!(result.is_err(), "unsupported query admitted: {sql}");
 }
 with_security_mapping_context(&graph_request("SELECT r.resourceId FROM Resource r"), |ctx| {
  let mut source: GraphSource = serde_json::from_str(ctx.binding_json()).unwrap();
  source.native_schema_revision = "2".into();
  assert!(prepare_graph_query_checked(ctx, &source).is_err());
  Ok(())
 }).unwrap();
}

#[test]
fn graph_result_checks_complete_ordered_batch_and_rejects_hidden_values() {
 use weft_core::security_mapping_transport::with_security_mapping_context;
 use weft_postgresql::security_graph_query::prepare_graph_query_checked;
 with_security_mapping_context(&graph_request("SELECT r.resourceId, r.salary FROM Resource r"), |ctx| {
  let source: GraphSource = serde_json::from_str(ctx.binding_json()).unwrap();
  let query = prepare_graph_query_checked(ctx, &source)?;
  let value = query.value();
  let bytes = value["resultContractJson"].as_str().unwrap();
  let hash = value["resultContractSha256"].as_str().unwrap();
  assert_eq!(sha256(bytes.as_bytes()), hash);
  let row = json!([{"outcomeId":"original","disposition":"original","value":{"string":"RA"}}, {"outcomeId":"withheld","disposition":"withheld"}]);
  let batch = json!({"version":"weft.security.cells/0.1.0", "resultContractSha256":hash, "rows":[row]});
  ctx.check_result_cells(query.result_contract(), bytes, hash, &batch.to_string())?;
  let mut empty = batch.clone(); empty["rows"] = json!([]);
  ctx.check_result_cells(query.result_contract(), bytes, hash, &empty.to_string())?;
  for mutation in 0..7 {
   let mut bad = batch.clone();
   match mutation {
    0 => bad["rows"][0][1]["value"] = json!({"integer":"100"}),
    1 => bad["rows"][0][1]["outcomeId"] = json!("original"),
    2 => bad["rows"][0] = json!([bad["rows"][0][0]]),
    3 => bad["rows"][0][0]["extra"] = json!(true),
    4 => bad["resultContractSha256"] = json!("0".repeat(64)),
    5 => bad["rows"][0] = json!([bad["rows"][0][1],bad["rows"][0][0]]),
    _ => {let corrupt=json!([{"outcomeId":"original","disposition":"original","value":{"string":"RB"}},{"outcomeId":"withheld","disposition":"withheld","value":{"integer":"100"}}]);bad["rows"].as_array_mut().unwrap().push(corrupt);},
   }
   let error = ctx.check_result_cells(query.result_contract(), bytes, hash, &bad.to_string()).expect_err("complete corrupt batch must refuse");
   assert_eq!(error.phase, "result");
  }
  Ok(())
 }).unwrap();
}
#[test]
fn graph_ordinals_preserve_distinct_constant_sources_and_exact_transformation_payloads() {
 use weft_core::security_mapping_transport::with_security_mapping_context;
 use weft_postgresql::{security_graph_query::prepare_graph_query_checked,security_graph_parameters::GraphPolicyParameterCodec};
 let mut request:Value=serde_json::from_str(&graph_request("SELECT r.resourceId, r.salary FROM Resource r")).unwrap();
 let field=json!({"documentId":"domain","moduleId":"m","elementId":"resourceId"});
 let salary=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});
 let mut policy:Value=serde_json::from_str(request["policyJson"].as_str().unwrap()).unwrap();
 policy["rules"][0]["condition"]=json!({"op":"eq","left":{"kind":"constant","field":field,"value":{"string":"RA"}},"right":{"kind":"constant","field":field,"value":{"string":"RA"}}});
 policy["rules"][0]["disclosure"][0]["disposition"]=json!({"kind":"transformed","transform":"constant","version":"0.1.0","field":salary,"value":{"integerToken":"1e2"}});
 let bytes=policy.to_string();request["policyJson"]=json!(bytes);
 let mut source:GraphSource=serde_json::from_str(request["bindingJson"].as_str().unwrap()).unwrap();
 let mut correspondence:Value=serde_json::from_str(&source.correspondence_json).unwrap();correspondence["logicalSourceDigests"]["policy"]=json!(sha256(bytes.as_bytes()));source.correspondence_json=correspondence.to_string();
 let binding=serde_json::to_string(&source).unwrap();request["bindingJson"]=json!(binding);
 let mut profile:Value=serde_json::from_str(request["queryProfileJson"].as_str().unwrap()).unwrap();profile["policySha256"]=json!(sha256(bytes.as_bytes()));profile["binding"]["sha256"]=json!(sha256(binding.as_bytes()));request["queryProfileJson"]=json!(profile.to_string());
 with_security_mapping_context(&request.to_string(),|ctx|{
  let query=prepare_graph_query_checked(ctx,&source)?;let bindings=query.parameter_bindings();
  assert_eq!(bindings.policy().iter().map(|p|p.codec()).collect::<Vec<_>>(),vec![GraphPolicyParameterCodec::Text,GraphPolicyParameterCodec::Text,GraphPolicyParameterCodec::Boolean,GraphPolicyParameterCodec::CoreLiteralJson]);
  assert_eq!(bindings.policy().iter().map(|p|p.ordinal()).collect::<Vec<_>>(),vec![1,2,3,4]);
  assert_eq!(bindings.policy()[0].source_path(),"logicalRules/0/condition/left");assert_eq!(bindings.policy()[1].source_path(),"logicalRules/0/condition/right");
  assert_eq!(bindings.policy()[3].source_path(),"logicalRules/0/disclosure/0/transformed");assert_eq!(bindings.policy()[3].field().element_id,"salary");
  assert_eq!(bindings.subject().ordinal(),5);assert_eq!(query.value()["execution"]["policyParameters"],json!(["RA","RA",true,{"integerToken":"1e2"}]));
  ctx.check_result_declaration(query.result_contract())?;Ok(())
 }).unwrap();
}

#[test]
fn public_relationship_correspondence_uses_original_installed_catalog_and_old_reader_refuses(){
 let inputs=serde_json::from_str(include_str!("security-public-relationship/modules.json")).unwrap();
 let logical=Catalog::prepare_security(inputs).unwrap();
 let packet=weft_core::security_source::SecuritySourcePacket::read(include_str!("security-public-relationship/policy.json"),include_str!("security-public-relationship/ontology.json"),&logical).unwrap();
 let plan=SecurityLogicalPlan::read(packet,&logical).unwrap();
 let original:GraphSource=serde_json::from_str(include_str!("security-public-relationship/graph-source.json")).unwrap();
 assert!(GraphBinding::read(&plan,&logical,&original).is_err());
 GraphBinding::read_relationship_candidate(&plan,&logical,&original).unwrap();
 for field in ["nativeRelationshipId","sourceTypeId","targetTypeId"] {
  let mut source=original.clone();let mut c:serde_json::Value=serde_json::from_str(&source.correspondence_json).unwrap();c["queryRelationships"][0][field]=serde_json::json!("1");source.correspondence_json=c.to_string();
  assert!(GraphBinding::read_relationship_candidate(&plan,&logical,&source).is_err(),"{field}");
 }
 for side in ["logicalRelationship","nativeRelationship"] {
  let mut source=original.clone();let mut c:serde_json::Value=serde_json::from_str(&source.correspondence_json).unwrap();c["queryRelationships"][0][side]["relationshipId"]=serde_json::json!("missing");source.correspondence_json=c.to_string();
  assert!(GraphBinding::read_relationship_candidate(&plan,&logical,&source).is_err());
 }
 let mut source=original.clone();let mut c:serde_json::Value=serde_json::from_str(&source.correspondence_json).unwrap();let entry=c["queryRelationships"][0].clone();c["queryRelationships"].as_array_mut().unwrap().push(entry);source.correspondence_json=c.to_string();assert!(GraphBinding::read_relationship_candidate(&plan,&logical,&source).is_err());
 let mut source=original;source.version="weft.truss.security.graph-source/0.1-candidate".into();assert!(GraphBinding::read_relationship_candidate(&plan,&logical,&source).is_err());
}
