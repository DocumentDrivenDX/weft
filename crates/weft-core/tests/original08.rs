use serde_json::{json,Value};
use weft_core::{frontend_json,json::sha256};
fn request(sql:&str)->Value {
 let text=include_str!("../../../tests/fixtures/original-commerce-0.8/ontology.json");
 let source:Value=serde_json::from_str(text).unwrap();
 json!({"dialect":"weft-sql/0.2.0","sql":sql,"modules":[{"documentJson":text,"pin":{"documentId":source["id"],"revision":"original-commerce0.8","umfVersion":"0.8.0","sha256":sha256(text.as_bytes())},"selectedModuleIds":["domain"]}]})
}
fn run(r:&Value)->Value {serde_json::from_str(&frontend_json(&r.to_string())).unwrap()}
fn edit(r:&mut Value,f:impl FnOnce(&mut Value)) {
 let mut d:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();f(&mut d);
 let s=d.to_string();r["modules"][0]["pin"]["sha256"]=json!(sha256(s.as_bytes()));r["modules"][0]["documentJson"]=json!(s);
}
#[test] fn original08_authored_inverse_and_unknown_relationship_qualifier() {
 use weft_core::{model::{Catalog,ModuleInput},ir::{Identity,Span},syntax::Name};
 for unknown in [false,true] {
  let mut r=request("SELECT s.id FROM suppliers s");edit(&mut r,|d|{let rel=&mut d["modules"][0]["relationships"][0];rel["inverse"]=json!("products_supplied");if unknown{rel["inverseName"]=json!("unestablished");}});
  let inputs:Vec<ModuleInput>=serde_json::from_value(r["modules"].clone()).unwrap();let catalog=Catalog::prepare(inputs).unwrap();let record=catalog.record_by_identity(&Identity{document_id:"urn:umf:domain:commerce".into(),revision:"original-commerce0.8".into(),module:"domain".into(),element:"suppliers".into()}).unwrap();
  let result=catalog.relationship_read(&record,&Name{value:"products_supplied".into(),quoted:true,span:Span{start:0,end:0}});
  if unknown{assert_eq!(result.unwrap_err().code,"WFT-TYPE");}else{assert!(result.unwrap().inverse);}
 }
}
#[test] fn original08_string_decimal_join_count_and_source_custody() {
 for sql in ["SELECT p.id, p.unit_price FROM products p", "SELECT COUNT(*) AS n FROM products p", "SELECT p.id AS left_id, q.id AS right_id FROM products p JOIN products q ON p.id = q.id"] {
  let r=request(sql);let o=run(&r);assert_eq!(o["status"],"resolved","{o}");assert_eq!(o["retainedModules"],r["modules"]);assert_eq!(o["logicalPlan"]["modulePins"][0]["umfVersion"],"0.8.0");
 }
 let o=run(&request("SELECT o.quantity FROM order_lines o"));assert_eq!(o["status"],"blocked");assert_eq!(o["diagnostics"][0]["code"],"WFT-TYPE");assert!(o.get("logicalPlan").is_none());
}
#[test] fn original08_exact_owning_version_and_selected_meaning_refuse() {
 let mut r=request("SELECT p.id, p.unit_price FROM products p");edit(&mut r,|d| {let e=d["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="products").unwrap();e["members"][0]["futureMembership"]=json!({"meaning":"selected-membership"});});assert_eq!(run(&r)["diagnostics"][0]["code"],"WFT-TYPE");
 r["interfaceVersion"]=json!("weft-compile/0.2.0");r["target"]=json!({"backendId":"ashlar.databricks","backendVersion":"0.1.0-candidate","targetProfile":"dbsql-candidate","bindingJson":"{}","bindingSha256":sha256(b"{}")});let out:Value=serde_json::from_str(&weft_core::compile::Compiler::default().compile_json(&r.to_string())).unwrap();assert_eq!(out["diagnostics"][0]["code"],"WFT-TYPE");assert!(out.get("sql").is_none());
 let mut r=request("SELECT p.id, p.unit_price FROM products p");edit(&mut r,|d| {let e=d["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="products").unwrap();e["references"]=json!([{"module":"domain","element":"products.id","role":"future-record-meaning"}]);});assert_eq!(run(&r)["diagnostics"][0]["code"],"WFT-TYPE");r["interfaceVersion"]=json!("weft-compile/0.2.0");r["target"]=json!({"backendId":"ashlar.databricks","backendVersion":"0.1.0-candidate","targetProfile":"dbsql-candidate","bindingJson":"{}","bindingSha256":sha256(b"{}")});let out:Value=serde_json::from_str(&weft_core::compile::Compiler::default().compile_json(&r.to_string())).unwrap();assert_eq!(out["diagnostics"][0]["code"],"WFT-TYPE");assert!(out.get("sql").is_none());
 let mut r=request("SELECT p.id FROM products p");r["modules"][0]["pin"]["umfVersion"]=json!("0.7.0");assert_eq!(run(&r)["diagnostics"][0]["code"],"WFT-MODEL-VERSION");
 for meaning in [json!({"default":{"value":{"string":"invented"},"on":"missing"}}),json!({"allowedValues":[{"string":"one"}]}),json!({"facets":{"futureBound":{"meaning":1}}}),json!({"extensions":{"unknown":{"meaning":true}}})] {
  let mut r=request("SELECT p.id FROM products p");edit(&mut r,|d| {let fields=d["modules"][0]["elements"].as_array_mut().unwrap();let f=fields.iter_mut().find(|f|f["id"]=="products.id").unwrap();for(k,v)in meaning.as_object().unwrap(){f[k]=v.clone();}});
  let o=run(&r);assert_eq!(o["status"],"blocked","{o}");assert_eq!(o["diagnostics"][0]["code"],"WFT-TYPE","meaning={meaning}: {o}");
 }
 for meaning in [json!({"nullability":"unspecified"}),json!({"cardinality":"array","itemType":{"scalarType":"string"}})] {
  let mut r=request("SELECT p.id FROM products p");edit(&mut r,|d|{let f=d["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|f|f["id"]=="products.id").unwrap();for(k,v)in meaning.as_object().unwrap(){f[k]=v.clone();}});assert_eq!(run(&r)["status"],"blocked");
 }
 let mut r=request("SELECT p.unit_price FROM products p");edit(&mut r,|d|{let f=d["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|f|f["id"]=="products.unit_price").unwrap();f["facets"]["range"]=json!({"min":{"decimalToken":"1.00"}});});assert_eq!(run(&r)["diagnostics"][0]["code"],"WFT-TYPE");
 let mut r=request("SELECT p.id FROM products p");edit(&mut r,|d|{d["modules"][0]["elements"][0]["extensions"]=json!(false);});assert_eq!(run(&r)["diagnostics"][0]["code"],"WFT-MODEL");
}
