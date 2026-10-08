// @covers US-006-AC4
// Public boundary refusals and exact JSON node-count boundaries.
use serde_json::{json,Value};
use weft_core::{compile::Compiler,json::checked_json};
fn refused(raw:&str,code:&str){
 let response:Value=serde_json::from_str(&Compiler::default().compile_json(raw)).unwrap();
 assert_eq!(response["status"],"blocked");
 assert_eq!(response["diagnostics"][0]["code"],code);
 for key in ["sql","parameters","logicalPlan","result","hostObligations"] {assert!(response.get(key).is_none(),"{key}");}
}
#[test]
fn json_node_and_request_byte_limits(){
 let at=format!("[{}]",vec!["0";99_999].join(","));
 assert!(checked_json(&at).is_ok()); // Root plus children = exactly 100000 nodes.
 let over=format!("[{}]",vec!["0";100_000].join(","));
 assert_eq!(checked_json(&over).unwrap_err(),"WFT-LIMIT");refused(&over,"WFT-LIMIT");
 let huge=" ".repeat(16*1024*1024+1);refused(&huge,"WFT-LIMIT");
 let nested=format!("{}0{}","[".repeat(200),"]".repeat(200));
 assert!(checked_json(&nested).is_err());
 let response:Value=serde_json::from_str(&Compiler::default().compile_json(&nested)).unwrap();
 assert_eq!(response["status"],"blocked");assert!(response.get("sql").is_none());
 println!("RESOURCE_REPORT nodeAt=100000 nodeOver=100001 requestOver=16777217 nesting=200");
}
#[test]
fn generated_truncated_json_and_nested_duplicates(){
 let mut state=0x574546540705u64;
 for case in 0..2000 {
  // Fixed xorshift generator, no wall clock or platform entropy.
  state^=state<<13;state^=state>>7;state^=state<<17;
  let raw=format!("{{\"case-{case}\": [{state},");
  assert_eq!(checked_json(&raw).unwrap_err(),"WFT-INPUT");refused(&raw,"WFT-INPUT");
  let duplicate=format!("{{\"outer\":[{{\"k-{state}\":1,\"k-{state}\":2}}]}}");
  assert_eq!(checked_json(&duplicate).unwrap_err(),"WFT-JSON-DUPLICATE");refused(&duplicate,"WFT-JSON-DUPLICATE");
 }
 println!("RESOURCE_REPORT generator=weft-json-xorshift/0.1.0 seed=95955044271877 generatedCases=2000 atomicAssertions=4000");
}
#[test]
fn sql_and_binding_byte_limits(){
 let cases:Value=serde_json::from_str(include_str!("../../docs/helix/03-test/fixtures/cases.json")).unwrap();
 let mut r=cases[0]["request"].clone();r["sql"]=json!(" ".repeat(65537));refused(&r.to_string(),"WFT-LIMIT");
 let mut r=cases[0]["request"].clone();r["target"]["bindingJson"]=json!(" ".repeat(4*1024*1024+1));refused(&r.to_string(),"WFT-LIMIT");
 println!("RESOURCE_REPORT sqlOver=65537 bindingOver=4194305");
}
