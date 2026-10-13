#![cfg(feature = "ashlar-databricks-compatible-revisions-candidate")]
#[path = "../../../tests/ashlar-databricks/common.rs"]
mod common;
use serde_json::{json,Value};
#[test]
fn explicit_compatible_revision_runtime_registration() {
 let mut request=common::request("SELECT c.name FROM Customer c");
 request["target"]["backendVersion"]=json!(weft_databricks::compatible_profile::VERSION);
 request["target"]["targetProfile"]=json!(weft_databricks::compatible_profile::PROFILE);
 let mut binding:Value=serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
 binding["records"][0]["compatibleSchemaRevisions"]=json!(["fixture-schema-1","fixture-schema-2"]);
 let bytes=binding.to_string();request["target"]["bindingSha256"]=json!(weft_core::json::sha256(bytes.as_bytes()));request["target"]["bindingJson"]=json!(bytes);
 let response:Value=serde_json::from_str(&weft_runtime::compile_json(&request.to_string())).unwrap();
 assert_eq!(response["status"],"compiled","{response}");assert!(response.to_string().contains("binding.compatibleSchemaRevisions"));
 request["target"]["backendVersion"]=json!("0.1.0-candidate");let response:Value=serde_json::from_str(&weft_runtime::compile_json(&request.to_string())).unwrap();assert_eq!(response["status"],"blocked","{response}");
}
