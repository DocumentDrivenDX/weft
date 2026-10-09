use serde_json::{json,Value};
use weft_core::{frontend_json,json::sha256};
fn request(n:&str)->Value{
 let text=include_str!("../../../tests/fixtures/original-commerce-0.8/ontology.json");let d:Value=serde_json::from_str(text).unwrap();json!({"sql":format!("SELECT o.quantity FROM order_lines o WHERE o.quantity = {n}"),"dialect":"weft-sql/0.2.0","modules":[{"documentJson":text,"pin":{"documentId":d["id"],"revision":"original-commerce0.8","umfVersion":"0.8.0","sha256":sha256(text.as_bytes())},"selectedModuleIds":["domain"]}]})
}
#[test]fn mathematical_integer_literal_is_exact_and_not_i128_or_int64(){
 for token in ["9007199254740993","-0","99999999999999999999999999999999999999","100000000000000000000000000000000000000000000000000"]{
  let r=request(token);let out:Value=serde_json::from_str(&frontend_json(&r.to_string())).unwrap();assert_eq!(out["status"],"resolved","{out}");assert!(out["logicalPlan"]["requiredCapabilities"].as_array().unwrap().iter().any(|c|c=="type.integer.unbounded"));let p=&out["logicalPlan"]["filters"][0]["right"];assert_eq!(p["value"],token);assert_eq!(p["type"]["facets"],json!({}));assert_eq!(out["retainedModules"],r["modules"]);
 }
}
