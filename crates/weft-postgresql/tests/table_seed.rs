// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{Value,json};
fn request()->Value {
 let mut source:Value=serde_json::from_str(include_str!("../../../tests/raw-postgresql/table-operation-model.json")).unwrap();
 let row=json!({"table":"notes","values":[{"field":{"module":"demo","element":"id"},"value":{"string":"seed"}},{"field":{"module":"demo","element":"owner"},"value":{"string":"alice"}}]});
 source["modules"][0]["extensions"]["umf.tables"]["seedSets"]=json!([{"id":"initial","conflict":"error","rows":vec![row.clone();1005]},{"id":"later","conflict":"ignore","rows":[row]}]);
 let mut binding:Value=serde_json::from_str::<Value>(include_str!("../../../tests/raw-postgresql/table-lifecycle-read-template.json")).unwrap()["binding"].clone();
 let text=source.to_string();binding["modelJson"]=json!(text);binding["modelSha256"]=json!(weft_core::json::sha256(text.as_bytes()));
 json!({"version":"weft.table-seed/0.1","target":"postgresql18","module":"demo","seedSets":["initial","later"],"binding":binding})
}
fn compile(request:Value)->Value{serde_json::from_str(&weft_postgresql::table_seed::compile_json(&request.to_string())).unwrap()}
#[test]
fn seed_selection_retains_original_source_order_and_bounded_native_chunks(){
 let r=request();let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");
 assert_eq!(p["sourceRowCount"],1006);assert_eq!(p["steps"].as_array().unwrap().len(),3);
 assert_eq!(p["steps"][0]["sourceRows"]["count"],1000);assert_eq!(p["steps"][1]["sourceRows"]["count"],5);
 assert!(p["steps"][2]["sql"].as_str().unwrap().contains("ON CONFLICT (\"id\") DO NOTHING"));
 assert!(!p["steps"][0]["sql"].as_str().unwrap().contains("alice"));
 assert_eq!(serde_json::from_str::<Value>(&weft_postgresql::table_seed::check_json(&p.to_string())).unwrap()["status"],"checked");
 let mut forged=p;forged["steps"][0]["sql"]=json!("DROP TABLE notes");forged.as_object_mut().unwrap().remove("structuralDigest");forged["structuralDigest"]=json!(weft_core::json::sha256(weft_core::mutation::canonical(&forged).as_bytes()));
 assert_eq!(serde_json::from_str::<Value>(&weft_postgresql::table_seed::check_json(&forged.to_string())).unwrap()["status"],"blocked");
 for selection in [json!(["missing"]),json!(["later","initial"]),json!(["initial","initial"]),json!([])]{let mut bad=r.clone();bad["seedSets"]=selection;assert_eq!(compile(bad)["status"],"blocked");}
}
#[test]
fn seed_literals_obey_domains_required_fields_unknown_meaning_and_original_capacity(){
 let r=request();
 for (field,value) in [("owner",json!({"integerToken":"3"})),("id",Value::Null)]{
  let mut bad=r.clone();let mut source:Value=serde_json::from_str(bad["binding"]["modelJson"].as_str().unwrap()).unwrap();
  let values=source["modules"][0]["extensions"]["umf.tables"]["seedSets"][0]["rows"][0]["values"].as_array_mut().unwrap();
  values.iter_mut().find(|v|v["field"]["element"]==field).unwrap()["value"]=value;
  let text=source.to_string();bad["binding"]["modelJson"]=json!(text);bad["binding"]["modelSha256"]=json!(weft_core::json::sha256(text.as_bytes()));
  assert_eq!(compile(bad)["status"],"blocked");
 }
 let mut bad=r;bad["sql"]=json!("INSERT arbitrary");assert_eq!(compile(bad)["status"],"blocked");
}
