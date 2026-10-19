// SPDX-License-Identifier: MIT OR Apache-2.0
//! Original native accepted model/binding; compiler checks are not native admission.
use serde_json::{json,Value};
fn request(sql:&str)->Value {
 let bundle:Value=serde_json::from_str(include_str!("fixtures/native-query-bundle.json")).unwrap();
 let raw=bundle["bindingJson"].as_str().unwrap();
 json!({"interfaceVersion":"weft-compile/0.2.6","dialect":"weft-sql/0.2.0","timestampComparisonProfile":"umf-rfc3339-exact-instant/1.0.0",
  "modules":bundle["modules"],"sql":sql,"parameters":{},"exactParameters":{},"options":{"allowCandidate":true},
  "target":{"backendId":"truss.postgresql.runtime","backendVersion":"0.4.38","targetProfile":"truss-runtime-pg18.6/0.5","bindingJson":raw,"bindingSha256":weft_core::json::sha256(raw.as_bytes())}})
}
fn run(r:&Value)->Value{serde_json::from_str(&weft_postgresql::truss_runtime::compile_json(&r.to_string())).unwrap()}
fn alter(r:&mut Value,change:impl FnOnce(&mut Value)) {
 let mut binding:Value=serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();change(&mut binding);
 let raw=binding.to_string();r["target"]["bindingSha256"]=json!(weft_core::json::sha256(raw.as_bytes()));r["target"]["bindingJson"]=json!(raw);
}
#[test]
fn original_named_owned_and_independent_queries_compile(){
 assert_eq!(weft_core::json::sha256(include_bytes!("fixtures/query-abi.json")),weft_postgresql::truss_runtime::QUERY_ABI_SHA256);
 for sql in ["SELECT COUNT(*) FROM \"A\" a","SELECT a.* FROM \"A\" a","SELECT RELATED_KEYS(a.children, 4) FROM \"A\" a","SELECT RELATED_KEYS(a.\"A-B\", 2) FROM \"A\" a"] {
  let mut r=request(sql);if sql.contains("a.*"){r["readProfile"]=json!({"version":"weft-truss-entity-read/0.3.0","subset":"complete-entity"});}
  let out=run(&r);assert_eq!(out["status"],"compiled","{sql}: {out}");assert_eq!(out["backend"]["backendVersion"],"0.4.38");
 }
}
#[test]
fn legacy_modes_refuse_new_binding_and_any_abi_field(){
 let old:Value=serde_json::from_str(include_str!("../../docs/helix/04-build/evidence/mutation-current-truss-20261014/native-query-bundle.json")).unwrap();
 let mut original=request("SELECT COUNT(*) FROM products p");original["modules"]=old["modules"].clone();
 original["target"]["backendVersion"]=json!("0.4.35");original["target"]["targetProfile"]=json!("truss-runtime-pg18.6/0.4");
 let raw=old["bindingJson"].as_str().unwrap();original["target"]["bindingJson"]=json!(raw);original["target"]["bindingSha256"]=json!(weft_core::json::sha256(raw.as_bytes()));
 let positive=run(&original);assert_eq!(positive["status"],"compiled","Original closed target must remain usable: {positive}");

 for version in ["0.4.35","0.4.36","0.4.37"] {
  let mut r=request("SELECT COUNT(*) FROM \"A\" a");r["target"]["backendVersion"]=json!(version);r["target"]["targetProfile"]=json!("truss-runtime-pg18.6/0.4");
  assert_eq!(run(&r)["status"],"blocked");
  for pin in [Value::Null,json!({})] {
   alter(&mut r,|b|{b["interfaceVersion"]=json!("truss-runtime-binding/0.4");b["queryAbi"]=pin;});
   let out=run(&r);assert_eq!(out["status"],"blocked");assert!(out.get("sql").is_none());
  }
 }
}
#[test]
fn missing_changed_or_extended_abi_refuses(){
 for field in ["absent","null","identity","version","sha256","extra"] {
  let mut r=request("SELECT COUNT(*) FROM \"A\" a");alter(&mut r,|b|match field {
   "absent"=>{b.as_object_mut().unwrap().remove("queryAbi");},"null"=>b["queryAbi"]=Value::Null,
   "extra"=>b["queryAbi"]["extra"]=json!(true),other=>b["queryAbi"][other]=json!("changed")});
  let out=run(&r);assert_eq!(out["status"],"blocked","{field}: {out}");assert!(out.get("sql").is_none());
 }
}
#[test]
fn same_abi_preserves_actual_changed_runtime_in_exact_native_parameter(){
 let mut r=request("SELECT COUNT(*) FROM \"A\" a");alter(&mut r,|b|{b["releaseId"]=json!("truss-catalog-runtime/0.70-candidate");b["releaseSha256"]=json!("ab".repeat(32));});
 let out=run(&r);assert_eq!(out["status"],"compiled","{out}");
 let raw=r["target"]["bindingJson"].as_str().unwrap();
 assert!(out["parameters"].as_array().unwrap().iter().any(|p|p["value"].as_str()==Some(raw)),"Original runtime identity must reach native query_scope unchanged");
 assert!(out["sql"].as_str().unwrap().contains("query_objects"));
 assert!(out["obligations"].as_array().unwrap().iter().any(|o|o["id"]=="truss.runtime.context/0.1"));
}
