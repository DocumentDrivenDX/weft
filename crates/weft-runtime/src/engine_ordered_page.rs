//! Additive engine0.2 custody. Native SQL belongs only to the original backend.
use serde::Deserialize;
use serde_json::{json,Value};
use weft_core::application_ordered_page::{SelectedKey,Cursor};
type Result<T>=std::result::Result<T,&'static str>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct Page {version:String,compiler:String,request:Value,page_size:u16,selected_key:SelectedKey}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Relationship{module:String,id:String,direction:String}
#[derive(Deserialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct Related {version:String,compiler:String,source:Value,target:Value,relationship:Relationship,#[serde(default)]page_size:Option<u16>,#[serde(default)]selected_key:Option<SelectedKey>,source_selected_key:SelectedKey,#[serde(default)]order_key:Option<SelectedKey>}
fn registry(method:&str)->Result<weft_core::backend::Registry>{
 let mut r=weft_core::backend::Registry::default();match method{
  "ashlar-spark4-sdk"=>r.register(weft_databricks::local_spark4_sdk::LocalSpark4Sdk),
  "ashlar-spark4-sdk-pages"=>r.register(weft_databricks::local_spark4_sdk_pages::LocalSpark4SdkPages),
  _=>return Err("explicit original SDK engine0.2 compiler required"),
 }.map_err(|_|"original SDK engine registration refused")?;Ok(r)
}
fn navigation(r:&Related)->Result<weft_core::application_navigation::Input>{
 let sql=r.source["sql"].as_str().filter(|s|s.len()<=65536).ok_or("bounded original source SQL required")?;
 if r.relationship.module.is_empty()||r.relationship.id.is_empty()||!matches!(r.relationship.direction.as_str(),"forward"|"inverse"){return Err("canonical original relationship required")}
 let mut context=r.source.clone();context.as_object_mut().ok_or("original source request required")?.insert("sql".into(),r.target["sql"].clone());if context!=r.target{return Err("original source/target context differs")}
 Ok(weft_core::application_navigation::Input{source_sql:sql.into(),relationship_module:r.relationship.module.clone(),relationship_id:r.relationship.id.clone(),inverse:r.relationship.direction=="inverse"})
}
pub(crate) fn related_read(packet:&Value)->Result<Value>{
 let r:Related=serde_json::from_value(packet.clone()).map_err(|_|"closed original engine0.2 Related read required")?;
 if r.version!="weft.engine-related-read/0.2"||r.selected_key.is_some()||r.page_size.is_some(){return Err("ordinary Related read cannot select page metadata")}
 let input=navigation(&r)?;let compiler=weft_core::compile::Compiler{registry:registry(&r.compiler)?};
 let artifact=weft_core::json::checked_json(&compiler.compile_sdk_read_json(&r.target.to_string(),r.order_key.as_ref(),Some(&input),Some(&r.source_selected_key))).map_err(|_|"original Related output framing refused")?;
 let mut out=crate::engine_read::wrap_artifact(&r.compiler,&r.target,artifact)?;
 if out["status"]!="compiled"{return Ok(out)}
 out.as_object_mut().unwrap().remove("structuralDigest");out["relatedRequest"]=packet.clone();if let Some(key)=&r.order_key{out["orderKey"]=json!(key);}digest(&mut out,"weft-runtime-engine-read");Ok(out)
}
fn digest(out:&mut Value,owner:&str){out["structuralDigest"]=json!({"owner":owner,"version":"0.2","sha256":weft_core::json::sha256(weft_core::mutation::canonical(out).as_bytes())});}
pub(crate) fn compile_value(packet:&Value,after:Option<Cursor>)->Result<Value>{
 let (method,request,size,selected,nav,source_key)=if packet["version"]=="weft.engine-related-page/0.2" {
  let r:Related=serde_json::from_value(packet.clone()).map_err(|_|"closed original Related page0.2 required")?;let nav=navigation(&r)?;if r.order_key.is_some(){return Err("page cannot carry orderKey")};
  (r.compiler,r.target,r.page_size.ok_or("original page bound required")?,r.selected_key.ok_or("original selected Key required")?,Some(nav),Some(r.source_selected_key))
 }else{
  let r:Page=serde_json::from_value(packet.clone()).map_err(|_|"closed original page0.2 required")?;if r.version!="weft.engine-page/0.2"{return Err("original page0.2 version differs")}
  (r.compiler,r.request,r.page_size,r.selected_key,None,None)
 };
 if method!="ashlar-spark4-sdk-pages"{return Err("explicit original ordered page compiler required")}
 let compiler=weft_core::compile::Compiler{registry:registry(&method)?};
 let native=weft_core::json::checked_json(&compiler.compile_sdk_ordered_json(&request.to_string(),&selected,size,after.clone(),nav.as_ref(),source_key.as_ref())).map_err(|_|"original ordered page output framing refused")?;
 if native["status"]!="compiled"{return Ok(json!({"version":"weft.engine-page/0.2","status":"blocked","original":native}))}
 if native["interfaceVersion"]!="weft-sql-page/0.3"||native["page"]["selectedKey"]!=json!(selected)||native["page"]["pageSize"]!=size{return Err("original ordered page identity/bound differs")}
 let wrapped=crate::engine_read::wrap_artifact(&method,&request,native["artifact"].clone())?;
 if wrapped["status"]!="compiled"{return Ok(wrapped)}
 let mut out=json!({"version":"weft.engine-page/0.2","status":"compiled","compiler":method,"request":request,"pageSize":size,"selectedKey":selected,"after":after,"page":native["page"],"artifact":native["artifact"],"outputs":wrapped["outputs"]});
 if nav.is_some(){out["relatedRequest"]=packet.clone();}
 digest(&mut out,"weft-runtime-engine-page");Ok(out)
}
fn response(result:Result<Value>)->String{result.unwrap_or_else(|m|json!({"version":"weft.engine-page/0.2","status":"blocked","diagnostics":[{"code":"WFT-ENGINE-PAGE","message":m}]})).to_string()}
pub fn compile_json(raw:&str)->String{response((||{if raw.len()>4_194_304{return Err("bounded original page input required")};let p=weft_core::json::checked_json(raw).map_err(|_|"original page framing refused")?;compile_value(&p,None)})())}
pub fn check_json(program:&str,current:&str,after:&str)->String{response((||{
 if program.len()>4_194_304||current.len()>4_194_304||after.len()>65536{return Err("bounded original page/context/cursor required")}
 let p=weft_core::json::checked_json(program).map_err(|_|"original page framing refused")?;let current=weft_core::json::checked_json(current).map_err(|_|"current request framing refused")?;
 let after:Option<Cursor>=serde_json::from_value(weft_core::json::checked_json(after).map_err(|_|"cursor framing refused")?).map_err(|_|"closed full typed original cursor required")?;
 if p["version"]!="weft.engine-page/0.2"||p["status"]!="compiled"||!p["after"].is_null(){return Err("original first page0.2 required")}
 let mut packet=if let Some(r)=p.get("relatedRequest"){r.clone()}else{json!({"version":"weft.engine-page/0.2","compiler":p["compiler"],"request":p["request"],"selectedKey":p["selectedKey"],"pageSize":p["pageSize"]})};
 if compile_value(&packet,None)?!=p{return Err("original page/metadata differs from compiler")}
 let original=&p["request"];let mut expected=original.clone();expected["parameters"]=current["parameters"].clone();if expected!=current{return Err("original page source/model/binding/order drift")}
 let old=original["parameters"].as_object().ok_or("original parameter inventory required")?;let new=current["parameters"].as_object().ok_or("current parameter inventory required")?;
 if old.keys().ne(new.keys())||old.iter().any(|(k,v)|new[k]["family"]!=v["family"]){return Err("original parameter inventory/family differs")}
 if packet.get("source").is_some(){packet["source"]["parameters"]=current["parameters"].clone();packet["target"]["parameters"]=current["parameters"].clone();}else{packet["request"]=current;}
 compile_value(&packet,after)
})())}

#[cfg(test)]
mod tests {
 use super::*;
 fn packet()->Value{serde_json::from_str(include_str!("../../../tests/engine-catalog/ashlar-sdk-ordered-page-request.json")).unwrap()}
 #[test]fn original_related_keys_output_keeps_distinct_sealed_decoder(){
  let input:Value=serde_json::from_str(include_str!("../../../tests/engine-catalog/ashlar-sdk-ordered-related-request.json")).unwrap();let mut request=input["source"].clone();request["sql"]=json!("SELECT r.id AS id,RELATED_KEYS(r.customers,2) AS customer_keys FROM sales.\"order\" r LIMIT 3");
  let program=crate::engine_read::compile_with_order(input["compiler"].as_str().unwrap(),&request,None).unwrap();assert_eq!(program["status"],"compiled","{program}");assert_eq!(program["outputs"][1]["carrier"],"related-keys");assert_eq!(program["outputs"][1]["missing"],false);assert_eq!(program["outputs"][1]["decoder"]["profile"],"ashlar-related-keys-string/0.1");
  if let Ok(dir)=std::env::var("WEFT_SDK_ORDERED_SAMPLE_DIR"){std::fs::create_dir_all(&dir).unwrap();std::fs::write(format!("{dir}/with-keys.json"),serde_json::to_string_pretty(&program).unwrap()).unwrap();}
 }
 #[test]fn ordinary_order_key_is_required_exact_and_recompiled(){
  let p=packet();let mut request=p["request"].clone();request["sql"]=json!("SELECT r.id AS id,r.status AS status FROM sales.\"order\" r ORDER BY r.score DESC NULLS LAST LIMIT 4");
  let input=json!({"version":"weft.engine-read/0.2","compiler":p["compiler"],"request":request,"orderKey":p["selectedKey"]});
  let out:Value=serde_json::from_str(&crate::engine_read::compile_json(&input.to_string())).unwrap();assert_eq!(out["status"],"compiled","{out}");assert_eq!(out["orderKey"],input["orderKey"]);assert!(out["artifact"]["sql"].as_str().unwrap().ends_with("LIMIT 4"));assert!(out["artifact"]["sql"].as_str().unwrap().contains("DESC NULLS LAST"));
  let checked:Value=serde_json::from_str(&crate::engine_read::check_json(&out.to_string(),&request.to_string())).unwrap();assert_eq!(checked,out);
  let mut forged=out.clone();let identity=forged["outputs"][1]["source"].clone();forged["outputs"][1]["missing"]=json!(false);forged["outputs"][1]["nullable"]=json!(false);
  for node in forged["artifact"]["logicalPlan"]["typeGraph"].as_array_mut().unwrap(){if node["identity"]==identity{node["availability"]=json!("required");node["type"]["nullable"]=json!(false);}}
  forged.as_object_mut().unwrap().remove("structuralDigest");digest(&mut forged,"weft-runtime-engine-read");let refused:Value=serde_json::from_str(&crate::engine_read::check_json(&forged.to_string(),&request.to_string())).unwrap();assert_eq!(refused["status"],"blocked","coupled metadata/graph forgery must recompile against untouched source document");
  for key in [Value::Null,json!({"module":"sales","record":"customer","key":"pk"})]{let mut bad=input.clone();bad["orderKey"]=key;let out:Value=serde_json::from_str(&crate::engine_read::compile_json(&bad.to_string())).unwrap();assert_ne!(out["status"],"compiled");}
  if let Ok(dir)=std::env::var("WEFT_SDK_ORDERED_SAMPLE_DIR"){std::fs::create_dir_all(&dir).unwrap();std::fs::write(format!("{dir}/ordered-read.json"),serde_json::to_string_pretty(&out).unwrap()).unwrap();}
 }
 #[test]fn related_source_and_target_keep_separate_authored_order_keys(){
  let input:Value=serde_json::from_str(include_str!("../../../tests/engine-catalog/ashlar-sdk-ordered-related-request.json")).unwrap();let out=related_read(&input).unwrap();assert_eq!(out["status"],"compiled","{out}");assert_eq!(out["relatedRequest"],input);assert_eq!(out["orderKey"],input["orderKey"]);let sql=out["artifact"]["sql"].as_str().unwrap();assert!(sql.contains("DESC NULLS LAST"));assert!(sql.contains("DESC NULLS FIRST"));assert!(sql.contains("LIMIT 2"));assert!(sql.ends_with("LIMIT 3"));
  for key in [Value::Null,json!({"module":"sales","record":"customer","key":"pk"})]{let mut bad=input.clone();bad["sourceSelectedKey"]=key;assert!(related_read(&bad).is_err()||related_read(&bad).unwrap()["status"]!="compiled");}
  let mut page=input.clone();page["version"]=json!("weft.engine-related-page/0.2");page.as_object_mut().unwrap().remove("orderKey");page["target"]["sql"]=json!("SELECT r.customerId AS id FROM sales.customer r ORDER BY r.customerName DESC NULLS FIRST");page["selectedKey"]=input["orderKey"].clone();page["pageSize"]=json!(2);let paged=compile_value(&page,None).unwrap();assert_eq!(paged["status"],"compiled","{paged}");
  if let Ok(dir)=std::env::var("WEFT_SDK_ORDERED_SAMPLE_DIR"){std::fs::create_dir_all(&dir).unwrap();for(name,v)in[("related-read.json",&out),("related-page.json",&paged),("related-read-request.json",&input),("related-page-request.json",&page)]{std::fs::write(format!("{dir}/{name}"),serde_json::to_string_pretty(v).unwrap()).unwrap();}}
 }
 #[test]fn original_mixed_nullish_order_keeps_selected_composite_key_and_full_states(){
  let input=packet();let p=compile_value(&input,None).unwrap();assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["selectedKey"],input["selectedKey"]);assert_eq!(p["request"],input["request"]);assert_eq!(p["page"]["keySlots"],json!([1,2]));assert_eq!(p["page"]["key"]["id"],"ordered-key");assert_eq!(p["page"]["sortSlots"][0]["direction"],"desc");assert_eq!(p["page"]["sortSlots"][0]["nulls"],"last");assert_eq!(p["page"]["sortSlots"][0]["missing"],true);
  assert_eq!(p["outputs"].as_array().unwrap().len(),2);assert_eq!(p["artifact"]["columns"].as_array().unwrap().len(),8);assert!(p["artifact"]["sql"].as_str().unwrap().contains("DESC NULLS LAST"));assert!(p["artifact"]["sql"].as_str().unwrap().ends_with("LIMIT 3"));
  let cursor=json!({"key":[{"state":"present","carrier":"string","value":"A"},{"state":"present","carrier":"string","value":"3"}],"sortValues":[{"state":"missing"},{"state":"present","carrier":"string","value":"A"},{"state":"present","carrier":"string","value":"3"}]});
  let next:Value=serde_json::from_str(&check_json(&p.to_string(),&input["request"].to_string(),&cursor.to_string())).unwrap();assert_eq!(next["status"],"compiled","{next}");assert_eq!(next["after"],cursor);assert_eq!(next["outputs"],p["outputs"]);assert!(next["artifact"]["sql"].as_str().unwrap().contains(" OR "));
  if let Ok(dir)=std::env::var("WEFT_SDK_ORDERED_SAMPLE_DIR"){std::fs::create_dir_all(&dir).unwrap();for(name,v)in[("page.json",&p),("after-missing.json",&next)]{std::fs::write(format!("{dir}/{name}"),serde_json::to_string_pretty(v).unwrap()).unwrap();}}
 }
 #[test]fn repeated_field_sort_uses_explicit_selected_key_slots_not_first_field_match(){
  let mut input=packet();input["request"]["sql"]=json!("SELECT r.id AS id FROM sales.\"order\" r ORDER BY r.id DESC");let p=compile_value(&input,None).unwrap();assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["page"]["keySlots"],json!([1,2]));assert_eq!(p["page"]["sortSlots"][0]["field"],p["page"]["sortSlots"][2]["field"]);assert_eq!(p["page"]["sortSlots"][0]["direction"],"desc");assert_eq!(p["page"]["sortSlots"][2]["direction"],"asc");
 }
 #[test]fn cursor_shape_state_identity_order_and_old_profile_fail_closed(){
  let input=packet();let p=compile_value(&input,None).unwrap();assert_eq!(p["status"],"compiled","{p}");
  for cursor in [json!([]),json!({"key":[],"sortValues":[]}),json!({"key":[{"state":"null"}],"sortValues":[{"state":"null"}]}),json!({"key":[{"state":"present","carrier":"string","value":"A"},{"state":"present","carrier":"string","value":"1"}],"sortValues":[{"state":"present","carrier":"integer","value":"2"},{"state":"present","carrier":"string","value":"B"},{"state":"present","carrier":"string","value":"1"}]})]{let n:Value=serde_json::from_str(&check_json(&p.to_string(),&input["request"].to_string(),&cursor.to_string())).unwrap();assert_eq!(n["status"],"blocked","{n}");}
  for key in ["selectedKey","page","artifact","outputs"]{let mut forged=p.clone();forged[key]=Value::Null;let n:Value=serde_json::from_str(&check_json(&forged.to_string(),&input["request"].to_string(),"null")).unwrap();assert_eq!(n["status"],"blocked");}
  let mut old=input.clone();old["compiler"]=json!("ashlar-spark4-pages");assert!(compile_value(&old,None).is_err());
  let mut left=input.clone();left["request"]["interfaceVersion"]=json!("weft-compile/0.3.0");left["request"]["sql"]=json!("SELECT r.id AS id FROM sales.\"order\" r LEFT JOIN sales.\"order\" other ON r.id=other.id");assert_ne!(compile_value(&left,None).unwrap()["status"],"compiled");
 }
}
