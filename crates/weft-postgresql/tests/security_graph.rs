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
