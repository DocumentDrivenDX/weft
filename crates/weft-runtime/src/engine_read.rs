// SPDX-License-Identifier: MIT OR Apache-2.0
//! Engine catalog custody. Engines retain native admission and execution.
//! This module never lowers SQL or interprets engine storage or authority.
use serde::Deserialize;
use serde_json::{json,Value};
type Result<T>=std::result::Result<T,&'static str>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct Request { version:String, compiler:String, request:Value,#[serde(default)]order_key:Option<Value> }
fn encode(v:&Value)->String{weft_core::mutation::canonical(v)}
fn related_output(column:&Value,artifact:&Value)->Result<Value>{
 let truss=artifact["backend"]["backendId"]=="truss.postgresql.runtime";
 if (!truss&&artifact["backend"]["backendId"]!="ashlar.databricks") || column["nullable"]!=false {return Err("original admitted engine required related output required")}
 let representation=&column["representation"];
 let bound=representation["bound"].as_u64().filter(|n|(1..=1000).contains(n)).ok_or("bounded original related output required")?;
 if representation["kind"]!="relatedKeys" {return Err("bounded original related representation required")}
 let key=&representation["key"];let fields=key["fields"].as_array().ok_or("original related Key Fields required")?;let types=key["types"].as_array().ok_or("original related Key types required")?;
 let supported=|t:&Value|{if t["nullable"]!=false{return false}let facets=&t["facets"];match t["family"].as_str(){Some("string")=>facets==&json!({}),Some("boolean") if truss=>facets==&json!({}),Some("integer") if truss=>facets==&json!({})||facets==&json!({"integerWidth":{"bits":64,"signed":true}}),Some("decimal") if truss=>[json!({"precision":18,"scale":4}),json!({"precision":18,"scale":2}),json!({"precision":12,"scale":2})].contains(facets),_=>false}};
 if fields.is_empty()||fields.len()!=types.len()||types.iter().any(|t|!supported(t)){return Err("original required related Key profile differs from engine decoder")}
 let logical=if artifact["logicalPlan"].get("base").is_some(){&artifact["logicalPlan"]["base"]}else{&artifact["logicalPlan"]};
 let expressions=logical["outputs"].as_array().ok_or("original application output inventory required")?;
 let selected=expressions.iter().filter(|output|output["name"]==column["outputName"]).collect::<Vec<_>>();
 if selected.len()!=1{return Err("unique original related expression required")}
 let expression=&selected[0]["expression"];let relationship=&expression["relationship"];
 if expression["op"]!="relatedKeys"||expression["bound"]!=bound||relationship["identity"]!=representation["relationship"]||relationship["targetKey"]!=*key{return Err("original related expression/representation correspondence differs")}
 let inverse=relationship["inverse"].as_bool().ok_or("original relationship direction required")?;let target=&relationship["to"];
 if ["documentId","revision","module","element"].iter().any(|name|target[*name].as_str().is_none_or(str::is_empty)){return Err("original related target Record required")}
 let pins=artifact["modelPins"].as_array().ok_or("original related model pins required")?;
 if !pins.iter().any(|pin|pin["documentId"]==target["documentId"]&&pin["revision"]==target["revision"]){return Err("original related target outside model pins")}
 let mut components=vec![];
 for (field,logical_type) in fields.iter().zip(types){if field["documentId"]!=target["documentId"]||field["revision"]!=target["revision"]||field["module"].as_str().is_none_or(str::is_empty)||field["element"].as_str().is_none_or(str::is_empty){return Err("original related Key Field pin differs")};components.push(json!({"field":{"module":field["module"],"element":field["element"]},"carrier":logical_type["family"],"nullable":false}));}
 let id=key["id"].as_str().filter(|id|!id.is_empty()).ok_or("original authored Key id required")?;
 Ok(json!({"name":column["outputName"],"carrier":"related-keys","nullable":false,"decoder":{"profile":if truss{"truss-related-keys/0.1"}else{"ashlar-related-keys-string/0.1"},"bound":bound,"relationship":{"module":relationship["identity"]["module"],"id":relationship["identity"]["relationship"],"direction":if inverse{"inverse"}else{"forward"}},"key":{"module":target["module"],"record":target["element"],"key":id,"components":components},"target":target,"representation":representation,"modelPins":pins}}))
}
#[cfg(feature="ashlar-sdk-ordered-page-candidate")]
pub(crate) fn compile_with_order(method:&str,request:&Value,key:Option<&Value>)->Result<Value>{
 if !matches!(method,"ashlar-spark4-sdk"|"ashlar-spark4-sdk-pages"){if key.is_some(){return Err("legacy engine cannot accept orderKey")};return compile_original(method,request)}
 let selected=key.map(|v|serde_json::from_value::<weft_core::application_ordered_page::SelectedKey>(v.clone())).transpose().map_err(|_|"closed original orderKey required")?;
 let mut registry=weft_core::backend::Registry::default();if method=="ashlar-spark4-sdk"{registry.register(weft_databricks::local_spark4_sdk::LocalSpark4Sdk)}else{registry.register(weft_databricks::local_spark4_sdk_pages::LocalSpark4SdkPages)}.map_err(|_|"original SDK backend registration refused")?;
 let raw=weft_core::compile::Compiler{registry}.compile_sdk_read_json(&encode(request),selected.as_ref(),None,None);
 let mut out=wrap_artifact(method,request,weft_core::json::checked_json(&raw).map_err(|_|"original ordered read framing refused")?)?;
 if out["status"]=="compiled"{if let Some(key)=key{out.as_object_mut().unwrap().remove("structuralDigest");out["orderKey"]=key.clone();seal_ordered(&mut out);}}
 Ok(out)
}
#[cfg(feature="ashlar-sdk-ordered-page-candidate")]
fn seal_ordered(out:&mut Value){out["structuralDigest"]=json!({"owner":"weft-runtime-engine-read","version":"0.2","sha256":weft_core::json::sha256(encode(out).as_bytes())});}
pub(crate) fn compile_original(method:&str,request:&Value)->Result<Value>{
 let raw=match method{
  "truss-runtime"=>weft_postgresql::truss_runtime::compile_json(&encode(request)),
  #[cfg(feature="ashlar-sdk-ordered-page-candidate")]
  "truss-runtime-binary"=>weft_postgresql::truss_runtime::compile_json(&encode(request)),
  "registered"=>crate::compile_json(&encode(request)),
  #[cfg(feature="ashlar-sdk-ordered-page-candidate")]
  "ashlar-spark4-sdk-pages"=>{
    let mut registry=weft_core::backend::Registry::default();registry.register(weft_databricks::local_spark4_sdk_pages::LocalSpark4SdkPages).map_err(|_|"original ordered page registration refused")?;
    weft_core::compile::Compiler{registry}.compile_sdk_presence_json(&encode(request))
  },
  #[cfg(feature="ashlar-sdk-ordered-page-candidate")]
  "ashlar-spark4-sdk"=>{
    let mut registry=weft_core::backend::Registry::default();
    registry.register(weft_databricks::local_spark4_sdk::LocalSpark4Sdk).map_err(|_|"original SDK presence registration refused")?;
    weft_core::compile::Compiler {registry}.compile_sdk_presence_json(&encode(request))
  },
  "ashlar-spark4-fields"=>{
    let mut registry=weft_core::backend::Registry::default();
    registry.register(weft_databricks::local_spark4_fields::LocalSpark4Fields).map_err(|_|"original direct Field registration refused")?;
    weft_core::compile::Compiler {registry}.compile_field_projection_json(&encode(request))
  },
  "ashlar-spark4"=>{
    let mut registry=weft_core::backend::Registry::default();
    registry.register(weft_databricks::local_spark4::LocalSpark4).map_err(|_|"original local Spark backend registration refused")?;
    weft_core::compile::Compiler { registry }.compile_json(&encode(request))
  },
  "ashlar-spark4-pages"=>{
    let registry=if request["target"]["backendId"]==weft_databricks::managed_current_core_xxhash64::ID {
      let mut registry=weft_core::backend::Registry::default();
      registry.register(weft_databricks::authored_pages::managed_current_core_xxhash64()).map_err(|_|"managed current-core xxhash64 page backend registration refused")?;
      registry
    } else if request["target"]["backendId"]==weft_databricks::managed_current_core::ID {
      let mut registry=weft_core::backend::Registry::default();
      registry.register(weft_databricks::authored_pages::managed_current_core()).map_err(|_|"managed current-core page backend registration refused")?;
      registry
    } else {local_authored_page_target(request)?};
    weft_core::compile::Compiler { registry }.compile_json(&encode(request))
  },
  "ashlar-dbsql-20261016-pages"=>{
    let mut registry=weft_core::backend::Registry::default();registry.register(weft_databricks::authored_pages::managed()).map_err(|_|"original managed page backend registration refused")?;
    weft_core::compile::Compiler { registry }.compile_json(&encode(request))
  },
  "ashlar-dbsql-20261016"=>{
    let mut registry=weft_core::backend::Registry::default();
    registry.register(weft_databricks::managed_20261016::Managed20261016).map_err(|_|"original managed build registration refused")?;
    weft_core::compile::Compiler { registry }.compile_json(&encode(request))
  },
  _=>return Err("unrecognized original engine compiler")};
 let artifact=weft_core::json::checked_json(&raw).map_err(|_|"original compiler output refused")?;
 wrap_artifact(method,request,artifact)
}
fn local_authored_page_target(request:&Value)->Result<weft_core::backend::Registry>{
 let target=&request["target"];
 let mut registry=weft_core::backend::Registry::default();
 if target["backendId"]=="ashlar.databricks"&&target["backendVersion"]==weft_databricks::authored_pages::LOCAL_VERSION&&target["targetProfile"]==weft_databricks::local_spark4::PROFILE {
  registry.register(weft_databricks::authored_pages::local()).map_err(|_|"original local Spark4 page target registration refused")?;
 }else if target["backendId"]=="ashlar.databricks"&&target["backendVersion"]==weft_databricks::authored_pages::LOCAL42_VERSION&&target["targetProfile"]==weft_databricks::local_spark42::PROFILE {
  registry.register(weft_databricks::authored_pages::local42()).map_err(|_|"original local Spark4.2 page target registration refused")?;
 }else{return Err("exact Ashlar authored-page target required")}
 Ok(registry)
}
fn logical_plan_scalar<'a>(artifact:&'a Value,identity:&Value)->Result<&'a Value>{
 let plan=&artifact["logicalPlan"];let graph=plan["typeGraph"].as_array().ok_or("original scalar sibling type graph required")?;let nodes=graph.iter().filter(|d|d["identity"]==*identity).collect::<Vec<_>>();if nodes.len()!=1{return Err("unique original scalar sibling required")};Ok(nodes[0])
}
pub(crate) fn wrap_artifact(method:&str,request:&Value,artifact:Value)->Result<Value>{
 if artifact["status"]!="compiled" {return Ok(json!({"version":"weft.engine-read/0.1","status":"blocked","original":artifact}))}
 if request.get("exactParameters").is_some_and(|p|p.as_object().is_none_or(|m|!m.is_empty())){return Err("engine catalog exact binary/temporal parameter profile not selected")}
 let parameters=request["parameters"].as_object().ok_or("complete declared parameter object required")?;
 for parameter in parameters.values(){let p=parameter.as_object().ok_or("original scalar parameter required")?;if p.len()!=2||!p.contains_key("family")||!p.contains_key("value")||!matches!(p["family"].as_str(),Some("string"|"boolean"|"integer"|"decimal"))||!p["value"].is_string(){return Err("closed non-temporal scalar parameter required")}}
 let sdk=matches!(method,"ashlar-spark4-sdk"|"ashlar-spark4-sdk-pages");
 let binary=method=="truss-runtime-binary";
 let logical=if artifact["logicalPlan"].get("base").is_some(){&artifact["logicalPlan"]["base"]}else{&artifact["logicalPlan"]};
 let visible=if artifact["logicalPlan"].get("base").is_some(){logical["outputs"].as_array().ok_or("original visible outputs required")?.len()}else{artifact["columns"].as_array().ok_or("original columns required")?.len()};
 let columns=artifact["columns"].as_array().ok_or("complete engine output inventory required")?;let mut outputs=vec![];
 if binary && request["modules"].as_array().is_none_or(|m|m.is_empty()||m.iter().any(|m|m["pin"]["umfVersion"]!="0.10.0")){return Err("explicit current Core10 binary/scalar state domain required")}
 if binary && (request["interfaceVersion"]!="weft-compile/0.2.7" || artifact["backend"]["backendId"]!="truss.postgresql.runtime" || artifact["backend"]["backendVersion"]!="0.4.41" || artifact["backend"]["targetProfile"]!="truss-runtime-pg18.6/0.5") {return Err("original pinned Truss binary domain profile required")}
 for column in columns.iter().take(visible){
 if binary {let expressions=logical["outputs"].as_array().ok_or("original direct binary-profile expressions required")?;let selected=expressions.iter().filter(|o|o["name"]==column["outputName"]).collect::<Vec<_>>();if selected.len()!=1||selected[0]["expression"]["op"]!="field"||column["sourceIdentities"]!=json!([selected[0]["expression"]["identity"]]){return Err("binary-profile projection must retain original direct Field identity")}}
 let representation=&column["representation"];if representation["kind"]=="relatedKeys"{let mut output=related_output(column,&artifact)?;if sdk {output["missing"]=json!(false);}outputs.push(output);continue;}let logical=match representation["kind"].as_str(){Some("scalar")=>&representation["logicalType"],Some("value")=>{let descriptor=logical["typeGraph"].as_array().ok_or("original output type graph required")?.iter().find(|d|d["identity"]==representation["descriptor"]).ok_or("original output descriptor absent")?;if binary && descriptor["kind"]=="exactScalar" && descriptor["scalarType"]=="binary" {
 if representation!=&json!({"kind":"value","descriptor":descriptor["identity"],"nativeNull":true})||column["sourceIdentities"]!=json!([descriptor["identity"]])||column["nullable"]!=false{return Err("original direct binary source representation required")}
 let identity=&descriptor["identity"];let modules=request["modules"].as_array().ok_or("original binary source modules required")?;
 let source=modules.iter().find(|m|m["pin"]["documentId"]==identity["documentId"]&&m["pin"]["revision"]==identity["revision"]&&m["pin"]["umfVersion"]=="0.10.0").ok_or("binary Core10 state domain not established")?;
 let doc=weft_core::json::checked_json(source["documentJson"].as_str().ok_or("current Core10 binary source required")?).map_err(|_|"original binary source framing refused")?;
 if doc["umf"]!="0.10.0"{return Err("binary Core10 source domain required")}
 let optional=match descriptor["availability"].as_str(){Some("required")=>false,Some("absent-allowed")=>true,_=>return Err("binary availability domain not established")};
 outputs.push(json!({"name":column["outputName"],"carrier":"binary","nullable":optional,"missing":optional,"source":identity,"representation":representation,"modelPins":artifact["modelPins"],"stateDomain":{"version":"umf.core08-binary/0.1","states":if optional{json!(["missing","null","value"])}else{json!(["value"])}},"decoder":{"profile":"truss-binary/0.1"}}));continue;
 }
 if descriptor["kind"]!="scalar" || !matches!(descriptor["type"]["family"].as_str(),Some("string"|"boolean"|"integer"|"decimal")){return Err("engine output carrier profile unsupported")};if sdk||binary {
 if representation["nativeNull"]!=true || !column["sourceIdentities"].as_array().is_some_and(|ids|ids==&vec![descriptor["identity"].clone()]) {return Err("original direct Field representation/source differs")}
 outputs.push(json!({"name":column["outputName"],"carrier":descriptor["type"]["family"],"nullable":descriptor["type"]["nullable"]==true||(binary&&descriptor["availability"]=="absent-allowed"),"missing":descriptor["availability"]=="absent-allowed","source":descriptor["identity"],"representation":representation,"modelPins":artifact["modelPins"]}));
 }else{outputs.push(json!({"name":column["outputName"],"carrier":descriptor["type"]["family"],"nullable":descriptor["type"]["nullable"]==true||descriptor["availability"]=="absent-allowed"}));}continue;},_=>return Err("engine output representation unsupported")};
  if !matches!(logical["family"].as_str(),Some("string"|"boolean"|"integer"|"decimal")){return Err("engine output carrier profile unsupported")};if binary {
 let ids=column["sourceIdentities"].as_array().filter(|ids|ids.len()==1).ok_or("original binary-profile sibling must be a direct Field")?;let identity=&ids[0];
 let descriptor=logical_plan_scalar(&artifact,identity)?;
 if descriptor["type"]!=*logical||descriptor["kind"]!="scalar"{return Err("original scalar sibling descriptor differs")}
 let optional=descriptor["availability"]=="absent-allowed";if !optional&&descriptor["availability"]!="required"{return Err("original scalar sibling availability unavailable")}
 outputs.push(json!({"name":column["outputName"],"carrier":logical["family"],"nullable":logical["nullable"]==true||optional,"missing":optional,"source":identity,"representation":representation,"modelPins":artifact["modelPins"]}));
 }else{outputs.push(json!({"name":column["outputName"],"carrier":logical["family"],"nullable":logical["nullable"]}));}
 }
 let version=if sdk||binary {"0.2"}else{"0.1"};
 let mut out=json!({"version":format!("weft.engine-read/{version}"),"status":"compiled","compiler":method,"request":request,"artifact":artifact,"outputs":outputs});
 out["structuralDigest"]=json!({"owner":"weft-runtime-engine-read","version":version,"sha256":weft_core::json::sha256(encode(&out).as_bytes())});Ok(out)
}
fn checked_program(program:&Value)->Result<(&str,&Value)>{
 if !matches!(program["version"].as_str(),Some("weft.engine-read/0.1"|"weft.engine-read/0.2"))||program["status"]!="compiled"{return Err("original compiled engine catalog required")}
 let method=program["compiler"].as_str().ok_or("original compiler required")?;
 let request=&program["request"];
 let expected=if let Some(packet)=program.get("relatedRequest"){crate::engine_related_read::compile_value(packet)?}else{
 #[cfg(feature="ashlar-sdk-ordered-page-candidate")]let out=compile_with_order(method,request,program.get("orderKey"))?;
 #[cfg(not(feature="ashlar-sdk-ordered-page-candidate"))]let out=compile_original(method,request)?;
 out};
 if expected!=*program{return Err("original engine program differs from its compiler")}
 Ok((method,request))
}
fn bind(program:&Value,request:&Value)->Result<Value>{
 let(method,original)=checked_program(program)?;
 let mut expected=original.clone();expected["parameters"]=request["parameters"].clone();
 if expected!=*request{return Err("engine model, binding, SQL or compiler context drift")}
 let before=original["parameters"].as_object().ok_or("original parameter inventory required")?;
 let after=request["parameters"].as_object().ok_or("current parameter inventory required")?;
 if before.keys().ne(after.keys())||before.iter().any(|(name,value)|after[name]["family"]!=value["family"]){return Err("engine parameter inventory or family differs")}
 if let Some(packet)=program.get("relatedRequest"){let mut packet=packet.clone();packet["source"]["parameters"]=request["parameters"].clone();packet["target"]["parameters"]=request["parameters"].clone();crate::engine_related_read::compile_value(&packet)}else{
 #[cfg(feature="ashlar-sdk-ordered-page-candidate")]return compile_with_order(method,request,program.get("orderKey"));
 #[cfg(not(feature="ashlar-sdk-ordered-page-candidate"))]compile_original(method,request)}
}
fn response(result:Result<Value>)->String{result.unwrap_or_else(|code|json!({"version":"weft.engine-read/0.1","status":"blocked","diagnostics":[{"code":"WFT-ENGINE-READ","message":code}]})).to_string()}
pub fn compile_json(raw:&str)->String{response((||{let v=weft_core::json::checked_json(raw).map_err(|_|"request framing refused")?;let r:Request=serde_json::from_value(v).map_err(|_|"closed engine catalog request required")?;if r.version!=if matches!(r.compiler.as_str(),"ashlar-spark4-sdk"|"ashlar-spark4-sdk-pages"|"truss-runtime-binary"){"weft.engine-read/0.2"}else{"weft.engine-read/0.1"}{return Err("engine catalog version differs")};{
 #[cfg(feature="ashlar-sdk-ordered-page-candidate")]return compile_with_order(&r.compiler,&r.request,r.order_key.as_ref());
 #[cfg(not(feature="ashlar-sdk-ordered-page-candidate"))]{if r.order_key.is_some(){return Err("ordered read feature not selected")};compile_original(&r.compiler,&r.request)}
 }})())}
pub fn check_json(program:&str,current_request:&str)->String{response((||{let program=weft_core::json::checked_json(program).map_err(|_|"program framing refused")?;let request=weft_core::json::checked_json(current_request).map_err(|_|"request framing refused")?;bind(&program,&request)})())}
#[cfg(test)]mod tests{
 use super::*;
 fn request()->Value{let mut r:Value=serde_json::from_str(include_str!("../../../tests/truss-decimal12/fixtures/genuine-northwind-compile-request.json")).unwrap();r["interfaceVersion"]=json!("weft-compile/0.2.7");r["timestampComparisonProfile"]=json!("umf-rfc3339-exact-instant/1.0.0");r["target"]["backendVersion"]=json!("0.4.41");let mut b:Value=serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();let old=b["documents"][0]["sha256"].as_str().unwrap().to_owned();let current=r["modules"][0]["pin"]["sha256"].as_str().unwrap().to_owned();fn replace(v:&mut Value,old:&str,new:&str){match v{Value::String(s) if s==old=>*s=new.to_owned(),Value::Array(a)=>for x in a{replace(x,old,new)},Value::Object(o)=>for x in o.values_mut(){replace(x,old,new)},_=>{}}}replace(&mut b,&old,&current);b["documents"][0]["revision"]=json!(r["modules"][0]["pin"]["revision"]);b["queryAbi"]=json!({"identity":"truss-postgresql-logical-query","version":"1.1","sha256":weft_core::json::sha256(include_bytes!("../../../tests/truss-date/query-abi1.1.json"))});let raw=b.to_string();r["target"]["bindingJson"]=json!(raw);r["target"]["bindingSha256"]=json!(weft_core::json::sha256(raw.as_bytes()));r}
 fn program(r:&Value)->Value{serde_json::from_str(&compile_json(&json!({"version":"weft.engine-read/0.1","compiler":"truss-runtime","request":r}).to_string())).unwrap()}
 #[test]fn original_compiler_and_current_binding_agree_without_losing_decimal_spelling(){let r=request();let p=program(&r);assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["request"],r);let mut current=r.clone();current["parameters"]["price"]["value"]=json!("-0002.5000");let a:Value=serde_json::from_str(&check_json(&p.to_string(),&current.to_string())).unwrap();assert_eq!(a["status"],"compiled","{a}");assert_eq!(a["request"]["parameters"]["price"]["value"],"-0002.5000");assert!(a["artifact"]["parameters"].as_array().unwrap().iter().any(|s|s["value"]=="-0002.5000"));}
 #[test]fn changed_context_family_and_program_never_match(){let r=request();let p=program(&r);for mode in 0..5{let mut current=r.clone();let mut original=p.clone();match mode{0=>current["sql"]=json!("SELECT 1"),1=>current["target"]["bindingSha256"]=json!("0".repeat(64)),2=>current["parameters"]["price"]["family"]=json!("string"),3=>current["parameters"]["extra"]=json!({"family":"string","value":"private"}),_=>original["artifact"]["sql"]=json!("SELECT 1")};let a:Value=serde_json::from_str(&check_json(&original.to_string(),&current.to_string())).unwrap();assert_eq!(a["status"],"blocked");assert!(a.get("artifact").is_none());}}
 #[test]fn native_core010_engine_requests_keep_original_profiles_and_sources(){
  for (method,raw) in [("truss-runtime",include_str!("../../../tests/engine-catalog/native-truss-request.json")),("ashlar-spark4",include_str!("../../../tests/engine-catalog/native-ashlar-spark4-request.json")),("ashlar-dbsql-20261016",include_str!("../../../tests/engine-catalog/native-ashlar-dbsql-20261016-request.json"))]{
   let request:Value=serde_json::from_str(raw).unwrap();let out:Value=serde_json::from_str(&compile_json(&json!({"version":"weft.engine-read/0.1","compiler":method,"request":request}).to_string())).unwrap();
   assert_eq!(out["status"],"compiled","{out}");assert_eq!(out["request"],request);assert_eq!(out["artifact"]["modelPins"][0]["umfVersion"],request["modules"][0]["pin"]["umfVersion"]);
   if method=="ashlar-spark4" {let obligation=out["artifact"]["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.nativeProfile").unwrap();assert_eq!(obligation["parameters"]["engineVersion"],"4.0.1");assert_eq!(obligation["parameters"]["deltaVersion"],"4.0.0");assert_eq!(out["artifact"]["backend"]["targetProfile"],"spark4.0.1-delta4.0.0-candidate");}
   let mut changed=request.clone();changed["modules"][0]["documentJson"]=json!(format!("{} ",request["modules"][0]["documentJson"].as_str().unwrap()));let checked:Value=serde_json::from_str(&check_json(&out.to_string(),&changed.to_string())).unwrap();assert_eq!(checked["status"],"blocked");
  }
 }
 #[test]fn blocked_original_diagnostics_are_retained(){let mut r=request();r["parameters"]["price"]["value"]=json!("0.001");let p=program(&r);assert_eq!(p["status"],"blocked");assert_eq!(p["original"]["status"],"blocked");assert!(p["original"]["diagnostics"].as_array().is_some_and(|d|!d.is_empty()));}
}

/// Original authored-key page composition. Read catalog custody alone is never
/// relabeled as a page; retain the complete native page response and recompile it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct PageRequest { version:String, compiler:String, request:Value, page_size:u16,#[serde(default)]selected_key:Option<Value> }
fn compile_page_original(method:&str,request:&Value,size:u16,after:Option<Vec<String>>)->Result<Value>{
 if !matches!(method,"truss-runtime"|"truss-runtime-binary"|"ashlar-spark4-pages"|"ashlar-dbsql-20261016-pages") || !(1..=999).contains(&size){return Err("explicit original authored-key page compiler required")}
 let base=compile_original(method,request)?;
 if base["status"]!="compiled" {return Ok(json!({"version":"weft.engine-page/0.1","status":"blocked","original":base}))}
 let raw=if matches!(method,"truss-runtime"|"truss-runtime-binary"){weft_postgresql::truss_runtime::compile_page_json(&encode(request),size,after.clone())}else{
  let mut factory=|_:&weft_core::model::Catalog,_:weft_core::backend::Plan<'_>,_:weft_core::compile::CompositionInput<'_>|{
    let target_id=request["target"]["backendId"].as_str().unwrap_or("");
    if method=="ashlar-spark4-pages" && target_id==weft_databricks::managed_current_core_xxhash64::ID {
     let mut registry=weft_core::backend::Registry::default();
     registry.register(weft_databricks::authored_pages::managed_current_core_xxhash64()).map_err(|_|weft_core::error::Diagnostic::new("WFT-ENGINE-READ","engine", "managed current-core xxhash64 page target registration refused"))?;
     Ok(registry)
    } else if method=="ashlar-spark4-pages" && target_id==weft_databricks::managed_current_core::ID {
     let mut registry=weft_core::backend::Registry::default();
     registry.register(weft_databricks::authored_pages::managed_current_core()).map_err(|_|weft_core::error::Diagnostic::new("WFT-ENGINE-READ","engine", "managed current-core page target registration refused"))?;
     Ok(registry)
    } else if method=="ashlar-spark4-pages" {local_authored_page_target(request).map_err(|e|weft_core::error::Diagnostic::new("WFT-ENGINE-READ","engine",e))}else{
     let mut registry=weft_core::backend::Registry::default();
     registry.register(weft_databricks::authored_pages::managed()).map_err(|_|weft_core::error::Diagnostic::new("WFT-ENGINE-READ","engine", "original managed page target registration refused"))?;
     Ok(registry)
   }
  };
  weft_core::compile::Compiler::default().compile_typed_page_json_with_exact_numbers_factory(&encode(request),size,after.clone(),&mut factory)
 };
 let page=weft_core::json::checked_json(&raw).map_err(|_|"original page compiler output refused")?;
 if page["status"]!="compiled" {return Ok(json!({"version":"weft.engine-page/0.1","status":"blocked","original":page}))}
 let visible=base["outputs"].as_array().ok_or("complete visible output signature required")?.len();
 if page["interfaceVersion"]!="weft-sql-page/0.2" || page["page"]["visibleOutputs"]!=visible || page["page"]["pageSize"]!=size {return Err("original typed page output and bound differ")}
 let version=if method=="truss-runtime-binary"{"0.2"}else{"0.1"};
 let mut out=json!({"version":format!("weft.engine-page/{version}"),"status":"compiled","compiler":method,"request":request,"pageSize":size,"after":after,"page":page["page"],"artifact":page["artifact"],"outputs":base["outputs"]});
 out["structuralDigest"]=json!({"owner":"weft-runtime-engine-page","version":version,"sha256":weft_core::json::sha256(encode(&out).as_bytes())});Ok(out)
}
fn binary_page(request:&Value,size:u16,after:Option<Vec<String>>,key:Option<&Value>)->Result<Value>{
 let key=key.ok_or("original selectedKey required for binary page")?;
 if key.as_object().is_none_or(|m|m.len()!=3||!["module","record","key"].iter().all(|k|m.get(*k).is_some_and(|v|v.as_str().is_some_and(|s|!s.is_empty())))){return Err("closed original selectedKey required")}
 let mut out=compile_page_original("truss-runtime-binary",request,size,after)?;if out["status"]!="compiled"{return Ok(out)}
 let plan=&out["artifact"]["logicalPlan"];if key["module"]!=plan["source"]["record"]["module"]||key["record"]!=plan["source"]["record"]["element"]||key["key"]!=plan["pageKey"]["id"]{return Err("selectedKey differs from original native authored page Key")}
 out.as_object_mut().unwrap().remove("structuralDigest");out["selectedKey"]=key.clone();out["structuralDigest"]=json!({"owner":"weft-runtime-engine-page","version":"0.2","sha256":weft_core::json::sha256(encode(&out).as_bytes())});Ok(out)
}
fn page_response(result:Result<Value>)->String{result.unwrap_or_else(|code|json!({"version":"weft.engine-page/0.1","status":"blocked","diagnostics":[{"code":"WFT-ENGINE-PAGE","message":code}]})).to_string()}
pub fn compile_page_json(raw:&str)->String{
 #[cfg(feature="ashlar-sdk-ordered-page-candidate")]
 if weft_core::json::checked_json(raw).is_ok_and(|v|v["version"]=="weft.engine-page/0.2"&&v["compiler"]!="truss-runtime-binary"){return crate::engine_ordered_page::compile_json(raw);}
 page_response((||{
 if raw.len()>4_194_304{return Err("bounded original page request required")}
 let value=weft_core::json::checked_json(raw).map_err(|_|"page request framing refused")?;
 let r:PageRequest=serde_json::from_value(value).map_err(|_|"closed page catalog request required")?;
 if r.version!=if r.compiler=="truss-runtime-binary"{"weft.engine-page/0.2"}else{"weft.engine-page/0.1"}{return Err("page catalog version differs")}
 if r.compiler=="truss-runtime-binary"{binary_page(&r.request,r.page_size,None,r.selected_key.as_ref())}else{if r.selected_key.is_some(){return Err("legacy page rejects selectedKey")};compile_page_original(&r.compiler,&r.request,r.page_size,None)}
})())}
pub fn check_page_json(program:&str,current_request:&str,after_json:&str)->String{
 #[cfg(feature="ashlar-sdk-ordered-page-candidate")]
 if weft_core::json::checked_json(program).is_ok_and(|v|v["version"]=="weft.engine-page/0.2"&&v["compiler"]!="truss-runtime-binary"){return crate::engine_ordered_page::check_json(program,current_request,after_json);}
 page_response((||{
 if program.len()>4_194_304||current_request.len()>4_194_304||after_json.len()>1_048_576{return Err("bounded original page program, current request and continuation required")}
 let program=weft_core::json::checked_json(program).map_err(|_|"page program framing refused")?;
 let current=weft_core::json::checked_json(current_request).map_err(|_|"current page request framing refused")?;
 let after=weft_core::json::checked_json(after_json).map_err(|_|"page continuation framing refused")?;
 let after:Option<Vec<String>>=serde_json::from_value(after).map_err(|_|"original lexical page continuation required")?;
 if after.as_ref().is_some_and(|values|values.is_empty()||values.len()>1024||values.iter().any(|v|v.len()>1_048_576)){return Err("bounded complete page continuation required")}
 if program["version"]!=if program["compiler"]=="truss-runtime-binary"{"weft.engine-page/0.2"}else{"weft.engine-page/0.1"} || program["status"]!="compiled" || !program["after"].is_null(){return Err("original first-page catalog required")}
 if let Some(packet)=program.get("relatedRequest"){
  if crate::engine_related_read::compile_page_value(packet,None)?!=program{return Err("original related page differs from its compiler")}
  let read=crate::engine_related_read::page_read_packet(packet)?;let base=crate::engine_related_read::compile_value(&read)?;let checked=bind(&base,&current)?;
  if checked["status"]!="compiled"{return Ok(json!({"version":"weft.engine-page/0.1","status":"blocked","original":checked}))}
  let mut packet=packet.clone();packet["source"]["parameters"]=current["parameters"].clone();packet["target"]["parameters"]=current["parameters"].clone();return crate::engine_related_read::compile_page_value(&packet,after)
 }
 let method=program["compiler"].as_str().ok_or("original page compiler required")?;
 let size=program["pageSize"].as_u64().and_then(|v|u16::try_from(v).ok()).ok_or("original page bound required")?;
 let original=&program["request"];
 if (if method=="truss-runtime-binary"{binary_page(original,size,None,program.get("selectedKey"))?}else{compile_page_original(method,original,size,None)?})!=program{return Err("original page program differs from its compiler")}
 // The existing read checker owns exact source/context and parameter inventory.
 let base=compile_original(method,original)?;let checked=bind(&base,&current)?;
 if checked["status"]!="compiled"{return Ok(json!({"version":"weft.engine-page/0.1","status":"blocked","original":checked}))}
 if method=="truss-runtime-binary"{binary_page(&current,size,after,program.get("selectedKey"))}else{compile_page_original(method,&current,size,after)}
})())}
#[cfg(test)]mod page_tests{
 use super::*;
 fn request()->Value{let mut r:Value=serde_json::from_str(include_str!("../../../tests/engine-catalog/native-truss-request.json")).unwrap();r["sql"]=json!(r["sql"].as_str().unwrap().strip_suffix(" LIMIT 10").unwrap());r}
 fn program(r:&Value)->Value{serde_json::from_str(&compile_page_json(&json!({"version":"weft.engine-page/0.1","compiler":"truss-runtime","request":r,"pageSize":2}).to_string())).unwrap()}
 #[test]fn complete_required_primary_key_asc_page_order_is_owner_proven_redundant(){
  let original=request();let base=program(&original);assert_eq!(base["status"],"compiled","{base}");
  for suffix in ["ORDER BY r.id","ORDER BY r.id ASC","ORDER BY r.id ASC NULLS LAST","ORDER BY r.id ASC NULLS FIRST"]{let mut r=original.clone();r["sql"]=json!(format!("{} {suffix}",original["sql"].as_str().unwrap()));let p=program(&r);assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["request"],r);assert_eq!(p["artifact"]["sql"],base["artifact"]["sql"]);}
  for suffix in ["ORDER BY r.status","ORDER BY r.id,r.id","ORDER BY other.id","ORDER BY r.id DESC","ORDER BY r.id LIMIT 1"]{let mut r=original.clone();r["sql"]=json!(format!("{} {suffix}",original["sql"].as_str().unwrap()));let p=program(&r);assert_ne!(p["status"],"compiled","{suffix}: {p}");}
 }
 #[test]fn original_bounded_related_keys_page_preserves_entity_cursor_and_representation(){
  let mut r:Value=serde_json::from_str(include_str!("../../../tests/engine-catalog/native-ashlar-related-page-request.json")).unwrap();
  r["timestampComparisonProfile"]=json!("umf-rfc3339-exact-instant/1.0.0");
  let p=compile_page_original("ashlar-spark4-pages",&r,2,None).unwrap();assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["request"],r);
  assert_eq!(p["outputs"].as_array().unwrap().len(),2);assert_eq!(p["outputs"][1]["decoder"]["profile"],"ashlar-related-keys-string/0.1");assert_eq!(p["outputs"][1]["decoder"]["bound"],2);
  assert_eq!(p["artifact"]["columns"].as_array().unwrap().len(),3);assert_eq!(p["page"]["keyOutputs"],json!([2]));assert_eq!(p["outputs"][1]["decoder"]["representation"],p["artifact"]["columns"][1]["representation"]);
  let a:Value=serde_json::from_str(&check_page_json(&p.to_string(),&r.to_string(),"[\"o-1\"]")).unwrap();assert_eq!(a["status"],"compiled","{a}");assert_eq!(a["after"],json!(["o-1"]));assert_eq!(a["outputs"],p["outputs"]);assert_ne!(a["artifact"]["sql"],p["artifact"]["sql"]);
  let mut forged=p.clone();forged["outputs"][1]["decoder"]["bound"]=json!(3);let refused:Value=serde_json::from_str(&check_page_json(&forged.to_string(),&r.to_string(),"null")).unwrap();assert_eq!(refused["status"],"blocked");
 }
 #[test]fn original_page_retains_hidden_key_and_native_continuation(){let r=request();let p=program(&r);assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["outputs"].as_array().unwrap().len(),2);assert_eq!(p["artifact"]["columns"].as_array().unwrap().len(),3);assert_eq!(p["page"]["keyOutputs"],json!([2]));let a:Value=serde_json::from_str(&check_page_json(&p.to_string(),&r.to_string(),"[\"o-1\"]")).unwrap();assert_eq!(a["status"],"compiled","{a}");assert_eq!(a["after"],json!(["o-1"]));assert_ne!(a["artifact"]["sql"],p["artifact"]["sql"]);assert!(a["artifact"]["parameters"].as_array().unwrap().iter().any(|v|v["value"]=="o-1"));}
 #[test]fn original_ashlar_page_compilation_retains_snapshot_key_obligation(){for (method,version,raw) in [("ashlar-spark4-pages",weft_databricks::authored_pages::LOCAL_VERSION,include_str!("../../../tests/engine-catalog/native-ashlar-spark4-request.json")),("ashlar-dbsql-20261016-pages",weft_databricks::authored_pages::MANAGED_VERSION,include_str!("../../../tests/engine-catalog/native-ashlar-dbsql-20261016-request.json"))]{let mut r:Value=serde_json::from_str(raw).unwrap();assert!(compile_page_original(method,&r,2,None).is_err()||compile_page_original(method,&r,2,None).unwrap()["status"]!="compiled");r["interfaceVersion"]=json!("weft-compile/0.2.7");r["timestampComparisonProfile"]=json!("umf-rfc3339-exact-instant/1.0.0");r["target"]["backendVersion"]=json!(version);if method=="ashlar-spark4-pages"{r["target"]["backendId"]=json!("ashlar.databricks");r["target"]["targetProfile"]=json!(weft_databricks::local_spark4::PROFILE);}let p=compile_page_original(method,&r,2,None).unwrap();assert_eq!(p["status"],"compiled","{p}");assert!(p["artifact"]["obligations"].as_array().unwrap().iter().any(|o|o["id"]=="ashlar.candidate.keyIntegrity"));let a:Value=serde_json::from_str(&check_page_json(&p.to_string(),&r.to_string(),"[\"bar\"]")).unwrap();assert_eq!(a["status"],"compiled","{a}");}}
 #[test]fn current_spark42_authored_pages_require_exact_target_and_preserve_continuation(){
  let mut r:Value=serde_json::from_str(include_str!("../../../tests/engine-catalog/native-ashlar-spark4-request.json")).unwrap();
  r["interfaceVersion"]=json!("weft-compile/0.2.7");r["timestampComparisonProfile"]=json!("umf-rfc3339-exact-instant/1.0.0");r["target"]["backendVersion"]=json!(weft_databricks::authored_pages::LOCAL42_VERSION);r["target"]["targetProfile"]=json!(weft_databricks::local_spark42::PROFILE);
  let p=compile_page_original("ashlar-spark4-pages",&r,2,None).unwrap();assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["version"],"weft.engine-page/0.1");assert_eq!(p["artifact"]["backend"]["targetProfile"],weft_databricks::local_spark42::PROFILE);
  let native=p["artifact"]["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.nativeProfile").unwrap();assert_eq!(native["parameters"]["engineVersion"],"4.2.0");assert_eq!(native["parameters"]["deltaVersion"],"4.4.1");
  let next:Value=serde_json::from_str(&check_page_json(&p.to_string(),&r.to_string(),"[\"bar\"]")).unwrap();assert_eq!(next["status"],"compiled","{next}");assert_eq!(next["after"],json!(["bar"]));
  for (version,profile) in [(weft_databricks::authored_pages::LOCAL_VERSION,weft_databricks::local_spark42::PROFILE),(weft_databricks::authored_pages::LOCAL42_VERSION,weft_databricks::local_spark4::PROFILE)]{let mut bad=r.clone();bad["target"]["backendVersion"]=json!(version);bad["target"]["targetProfile"]=json!(profile);assert!(compile_page_original("ashlar-spark4-pages",&bad,2,None).is_err());}
 }
 #[test]fn managed_current_core_pages_compile_four_uc_pins_and_preserve_2_plus_1_continuation(){
  let mut r:Value=serde_json::from_str(include_str!("../../../tests/engine-catalog/native-ashlar-spark4-request.json")).unwrap();
  r["interfaceVersion"]=json!("weft-compile/0.2.7");r["timestampComparisonProfile"]=json!("umf-rfc3339-exact-instant/1.0.0");
  let mut binding:Value=serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();
  binding["profile"]=json!("ashlar-managed-current-core/0.1");
  binding["publication"]["tables"]=json!([
   {"name":["client_dev","managed_current_test","object_current"],"uuid":"00000000-0000-4000-8000-000000000001","version":11},
   {"name":["client_dev","managed_current_test","edge_current"],"uuid":"00000000-0000-4000-8000-000000000002","version":12},
   {"name":["client_dev","managed_current_test","tombstone"],"uuid":"00000000-0000-4000-8000-000000000003","version":13},
   {"name":["client_dev","managed_current_test","whole_source_history"],"uuid":"00000000-0000-4000-8000-000000000004","version":14}
  ]);
  let raw=binding.to_string();let digest=weft_core::json::sha256(raw.as_bytes());
  r["target"]=json!({"backendId":weft_databricks::managed_current_core::ID,"backendVersion":weft_databricks::authored_pages::MANAGED_CURRENT_VERSION,"targetProfile":weft_databricks::managed_current_core::PROFILE,"bindingJson":raw,"bindingSha256":digest});
  let p=compile_page_original("ashlar-spark4-pages",&r,2,None).unwrap();
  assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["version"],"weft.engine-page/0.1");
  assert_eq!(p["artifact"]["backend"]["backendId"],weft_databricks::managed_current_core::ID);
  assert_eq!(p["artifact"]["backend"]["targetProfile"],weft_databricks::managed_current_core::PROFILE);
  assert!(p["artifact"]["sql"].as_str().unwrap().contains("`client_dev`.`managed_current_test`.`object_current` VERSION AS OF 11"));
  let publication=p["artifact"]["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.candidate.publication").unwrap();
  assert!(publication["parameters"]["nativeProfile"]["warehouseRelease"].is_null());
  assert_eq!(publication["parameters"]["nativeProfile"]["engine"],"databricks-serverless-spark-jobs");
  let next:Value=serde_json::from_str(&check_page_json(&p.to_string(),&r.to_string(),"[\"bar\"]")).unwrap();
  assert_eq!(next["status"],"compiled","{next}");assert_eq!(next["version"],"weft.engine-page/0.1");assert_eq!(next["after"],json!(["bar"]));
  let next_publication=next["artifact"]["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.candidate.publication").unwrap();
  assert!(next_publication["parameters"]["nativeProfile"]["warehouseRelease"].is_null());
  assert_eq!(next_publication["parameters"]["nativeProfile"]["engine"],"databricks-serverless-spark-jobs");
  assert_ne!(next["artifact"]["sql"],p["artifact"]["sql"]);
  let mut xxhash64=r.clone();let mut v4_binding:Value=serde_json::from_str(xxhash64["target"]["bindingJson"].as_str().unwrap()).unwrap();
  v4_binding["layoutRevision"]=json!("ashlar-delta/0.4-xxhash64");v4_binding["layoutSha256"]=json!("b".repeat(64));v4_binding["lookupHashProfile"]=json!("spark-xxhash64-typed-key/0.1");
  let raw=v4_binding.to_string();let digest=weft_core::json::sha256(raw.as_bytes());
  xxhash64["target"]=json!({"backendId":weft_databricks::managed_current_pages_xxhash64::ID,"backendVersion":weft_databricks::authored_pages::MANAGED_CURRENT_XXHASH64_VERSION,"targetProfile":weft_databricks::managed_current_pages_xxhash64::PROFILE,"bindingJson":raw,"bindingSha256":digest});
  let v4=compile_page_original("ashlar-spark4-pages",&xxhash64,2,None).unwrap();assert_eq!(v4["status"],"compiled","{v4}");
  assert_eq!(v4["artifact"]["backend"]["backendId"],weft_databricks::managed_current_pages_xxhash64::ID);assert_eq!(v4["artifact"]["backend"]["backendVersion"],weft_databricks::authored_pages::MANAGED_CURRENT_XXHASH64_VERSION);assert_eq!(v4["artifact"]["backend"]["targetProfile"],weft_databricks::managed_current_pages_xxhash64::PROFILE);
  assert_eq!(v4["artifact"]["backend"]["interfaceVersion"],"weft-backend/0.2.7");
  assert_eq!(v4["artifact"]["targetContext"]["id"],weft_databricks::managed_current_pages_xxhash64::PROFILE);
  let v4_pub=v4["artifact"]["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.candidate.publication").unwrap();assert_eq!(v4_pub["parameters"]["layoutRevision"],"ashlar-delta/0.4-xxhash64");assert_eq!(v4_pub["parameters"]["layoutSha256"],"b".repeat(64));assert_eq!(v4_pub["parameters"]["nativeProfile"]["lookupHashProfile"],"spark-xxhash64-typed-key/0.1");
  let v4_native=v4["artifact"]["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.nativeProfile").unwrap();assert!(v4_native["parameters"]["requirements"].as_array().unwrap().iter().any(|r|r=="verify the selected Databricks serverless Spark Jobs runtime/build, authenticated principal, Unity Catalog catalog/schema/table UUIDs and Delta protocol before SQL"));
  assert_eq!(v4["artifact"]["sql"],p["artifact"]["sql"]);assert_eq!(v4["artifact"]["parameters"],p["artifact"]["parameters"]);assert_eq!(v4["outputs"],p["outputs"]);assert_eq!(v4["artifact"]["targetContext"]["storageLayoutRevision"],"ashlar-delta/0.4-xxhash64");
  let v4_next:Value=serde_json::from_str(&check_page_json(&v4.to_string(),&xxhash64.to_string(),"[\"bar\"]")).unwrap();assert_eq!(v4_next["status"],"compiled","{v4_next}");assert_eq!(v4_next["artifact"]["sql"],next["artifact"]["sql"]);
  let mut crossed=r.clone();crossed["target"]["backendId"]=json!(weft_databricks::managed_current_pages_xxhash64::ID);crossed["target"]["backendVersion"]=json!(weft_databricks::authored_pages::MANAGED_CURRENT_XXHASH64_VERSION);crossed["target"]["targetProfile"]=json!(weft_databricks::managed_current_pages_xxhash64::PROFILE);assert_ne!(compile_page_original("ashlar-spark4-pages",&crossed,2,None).unwrap()["status"],"compiled");
  let mut crossed_back=xxhash64.clone();crossed_back["target"]["backendId"]=json!(weft_databricks::managed_current_core::ID);crossed_back["target"]["backendVersion"]=json!(weft_databricks::authored_pages::MANAGED_CURRENT_VERSION);crossed_back["target"]["targetProfile"]=json!(weft_databricks::managed_current_pages::PROFILE);assert_ne!(compile_page_original("ashlar-spark4-pages",&crossed_back,2,None).unwrap()["status"],"compiled");
  let mut wrong=r.clone();wrong["target"]["targetProfile"]=json!(weft_databricks::local_spark42::PROFILE);
  assert_ne!(compile_page_original("ashlar-spark4-pages",&wrong,2,None).unwrap()["status"],"compiled");
 }
 #[test]fn blocked_page_composition_keeps_its_frame_and_original_diagnostics(){let mut r=request();r["sql"]=json!("SELECT unknown FROM absent");let value:Value=serde_json::from_str(&compile_page_json(&json!({"version":"weft.engine-page/0.1","compiler":"truss-runtime","request":r,"pageSize":1}).to_string())).unwrap();assert_eq!(value["version"],"weft.engine-page/0.1");assert_eq!(value["status"],"blocked");assert!(value["original"].is_object());assert!(value.get("artifact").is_none());}
 #[test]fn oversized_page_inputs_refuse_before_parsing(){let large=" ".repeat(4_194_305);let after=" ".repeat(1_048_577);for result in [compile_page_json(&large),check_page_json(&large,"{}","null"),check_page_json("{}",&large,"null"),check_page_json("{}","{}",&after)]{let value:Value=serde_json::from_str(&result).unwrap();assert_eq!(value["status"],"blocked");assert!(value["artifact"].is_null());}}
 #[test]fn page_tampering_and_context_drift_refuse(){let r=request();let p=program(&r);assert_eq!(p["status"],"compiled","{p}");for mode in 0..6{let mut p=p.clone();let mut current=r.clone();match mode{0=>p["pageSize"]=json!(3),1=>p["page"]["keyOutputs"]=json!([0]),2=>p["artifact"]["sql"]=json!("SELECT 1"),3=>current["sql"]=json!("SELECT 1"),4=>current["parameters"]["status"]["family"]=json!("integer"),_=>p["after"]=json!(["o-1"])}let checked:Value=serde_json::from_str(&check_page_json(&p.to_string(),&current.to_string(),"null")).unwrap();assert_eq!(checked["status"],"blocked","{checked}");}for after in ["[]","[1]","{}","[\"a\",\"b\"]"]{let a:Value=serde_json::from_str(&check_page_json(&p.to_string(),&r.to_string(),after)).unwrap();assert_eq!(a["status"],"blocked","{a}");}}
}

#[cfg(test)]
mod related_output_tests {
 use super::*;
 fn original()->Value {
  let request:Value=serde_json::from_str(include_str!("../../../tests/engine-catalog/native-ashlar-related-request.json")).unwrap();
  let out:Value=serde_json::from_str(&compile_json(&json!({"version":"weft.engine-read/0.1","compiler":"ashlar-spark4","request":request}).to_string())).unwrap();
  assert_eq!(out["status"],"compiled","{out}");out
 }
 fn truss_related_request(family:&str,facets:Value)->Value {
  // Derived compiler-only fixture; no accepted native publication is claimed.
  let mut bundle:Value=serde_json::from_str(include_str!("../../../tests/truss-query-abi/fixtures/native-query-bundle.json")).unwrap();
  let mut doc:Value=serde_json::from_str(bundle["modules"][0]["documentJson"].as_str().unwrap()).unwrap();let field=doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="A-value").unwrap();field["scalarType"]=json!(family);field["facets"]=facets;
  let source=doc.to_string();let sha=weft_core::json::sha256(source.as_bytes());bundle["modules"][0]["documentJson"]=json!(source);bundle["modules"][0]["pin"]["sha256"]=json!(sha);
  let mut binding:Value=serde_json::from_str(bundle["bindingJson"].as_str().unwrap()).unwrap();binding["documents"][0]["sha256"]=json!(sha);let raw=binding.to_string();
  json!({"interfaceVersion":"weft-compile/0.2.6","dialect":"weft-sql/0.2.0","timestampComparisonProfile":"umf-rfc3339-exact-instant/1.0.0","modules":bundle["modules"],"sql":"SELECT RELATED_KEYS(a.children,4) AS related FROM \"A\" a","parameters":{},"exactParameters":{},"options":{"allowCandidate":true},"target":{"backendId":"truss.postgresql.runtime","backendVersion":"0.4.39","targetProfile":"truss-runtime-pg18.6/0.5","bindingJson":raw,"bindingSha256":weft_core::json::sha256(raw.as_bytes())}})
 }
 #[test]fn original_truss_related_wrapper_retains_native_key_types_and_representation(){
  for (family,facets) in [("string",json!({})),("boolean",json!({})),("integer",json!({})),("integer",json!({"integerWidth":{"bits":64,"signed":true}})),("decimal",json!({"precision":18,"scale":4})),("decimal",json!({"precision":18,"scale":2})),("decimal",json!({"precision":12,"scale":2}))]{let request=truss_related_request(family,facets);let p=compile_original("truss-runtime",&request).unwrap();assert_eq!(p["status"],"compiled","{p}");let d=&p["outputs"][0]["decoder"];assert_eq!(d["profile"],"truss-related-keys/0.1");assert_eq!(d["key"]["components"][0]["carrier"],family);assert_eq!(d["representation"],p["artifact"]["columns"][0]["representation"]);assert_eq!(d["modelPins"],p["artifact"]["modelPins"]);assert_eq!(p["request"],request);
   for mode in 0..4{let mut artifact=p["artifact"].clone();match mode{0=>artifact["columns"][0]["representation"]["key"]["types"][0]["nullable"]=json!(true),1=>artifact["columns"][0]["representation"]["key"]["types"][0]["facets"]=json!({"precision":13,"scale":2}),2=>artifact["columns"][0]["representation"]["bound"]=json!(5),_=>artifact["backend"]["backendId"]=json!("other")};assert!(related_output(&artifact["columns"][0],&artifact).is_err(),"{mode}");}
  }
 }
 #[test]fn original_required_scalar_null_placement_normalizes_only_after_owner_proof(){
  let base=original();
  for placement in ["FIRST","LAST"]{let mut request=base["request"].clone();request["sql"]=json!(format!("{} ASC NULLS {placement}",request["sql"].as_str().unwrap()));let p=compile_original("ashlar-spark4",&request).unwrap();assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["request"],request);assert_eq!(p["artifact"]["sql"],base["artifact"]["sql"]);}
  for sql in ["SELECT r.id AS id,COUNT(*) AS total FROM sales.\"order\" r GROUP BY r.id ORDER BY r.id NULLS LAST","SELECT r.id AS id FROM sales.\"order\" r LEFT JOIN sales.customer c ON r.id=c.customerId ORDER BY r.id NULLS LAST","SELECT r.id AS id FROM sales.\"order\" r ORDER BY r.id NULLS UNKNOWN"]{let mut request=base["request"].clone();request["sql"]=json!(sql);let p=compile_original("ashlar-spark4",&request).unwrap();assert_ne!(p["status"],"compiled","{p}");}
  let mut request=base["request"].clone();let mut doc:Value=serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();let status=doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|f|f["id"]=="status").unwrap();status["nullability"]=json!("absent-allowed");let source=doc.to_string();let hash=weft_core::json::sha256(source.as_bytes());request["modules"][0]["documentJson"]=json!(source);request["modules"][0]["pin"]["revision"]=json!(hash);request["modules"][0]["pin"]["sha256"]=json!(hash);request["sql"]=json!("SELECT r.status AS status FROM sales.\"order\" r ORDER BY r.status NULLS LAST");let p=compile_original("ashlar-spark4",&request).unwrap();assert_ne!(p["status"],"compiled","{p}");
 }
 #[test]fn ashlar_related_original_preserves_native_representation_and_source_custody(){
  let p=original();let output=&p["outputs"][1];let native=&p["artifact"]["columns"][1];
  assert_eq!(output["carrier"],"related-keys");assert_eq!(output["decoder"]["profile"],"ashlar-related-keys-string/0.1");
  assert_eq!(output["decoder"]["representation"],native["representation"]);
  assert_eq!(output["decoder"]["modelPins"],p["artifact"]["modelPins"]);
  assert_eq!(output["decoder"]["relationship"]["direction"],"forward");
  assert_eq!(output["decoder"]["bound"],2);
  assert_eq!(output["decoder"]["key"]["record"],"customer");
  assert_eq!(output["decoder"]["key"]["components"].as_array().unwrap().len(),1);
  let checked:Value=serde_json::from_str(&check_json(&p.to_string(),&p["request"].to_string())).unwrap();assert_eq!(checked,p);
 }
 #[test]fn ashlar_related_metadata_faults_refuse_instead_of_guessing(){
  let p=original();let original_column=&p["artifact"]["columns"][1];
  for mode in 0..10 {let mut a=p["artifact"].clone();let mut c=original_column.clone();
   match mode {
    0=>c["representation"]["kind"]=json!("completeRelatedKeys"),
    1=>c["representation"]["bound"]=json!(0),
    2=>c["representation"]["key"]["types"][0]["family"]=json!("integer"),
    3=>c["nullable"]=json!(true),
    4=>a["logicalPlan"]["outputs"][1]["expression"]["relationship"]["inverse"]=Value::Null,
    5=>a["logicalPlan"]["outputs"][1]["expression"]["relationship"]["to"]["documentId"]=json!("foreign"),
    6=>a["modelPins"]=json!([]),
    7=>c["representation"]["key"]["fields"][0]["revision"]=json!("foreign"),
    8=>a["backend"]["backendId"]=json!("other"),
    _=>a["logicalPlan"]["outputs"][1]["expression"]["relationship"]["targetKey"]["id"]=json!("different")
   };assert!(related_output(&c,&a).is_err(),"mode {mode}");
  }
  let mut forged=p.clone();forged["outputs"][1]["decoder"]["bound"]=json!(1);
  let checked:Value=serde_json::from_str(&check_json(&forged.to_string(),&p["request"].to_string())).unwrap();assert_eq!(checked["status"],"blocked");
 }
}

#[cfg(all(test,feature="ashlar-databricks-candidate"))]
mod original_field_projection_tests {
 use super::*;
 fn request()->Value{serde_json::from_str(include_str!("../../../tests/engine-catalog/ashlar-fields-only-request.json")).unwrap()}
 #[test]fn exact_original_field_decimal38_and_uint64_do_not_widen_old_profile(){
  let request=request();let p=compile_original("ashlar-spark4-fields",&request).unwrap();
  assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["request"],request);
  let columns=p["artifact"]["columns"].as_array().unwrap();assert_eq!(columns.len(),5);
  assert_eq!(columns[4]["representation"]["logicalType"]["facets"],json!({"precision":38,"scale":6}));
  assert_eq!(columns[3]["representation"]["logicalType"]["facets"],json!({"integerWidth":{"bits":64,"signed":false}}));
  let mut old=request.clone();old["target"]["backendVersion"]=json!(weft_databricks::local_spark4::VERSION);old["target"]["targetProfile"]=json!(weft_databricks::local_spark4::PROFILE);
  assert_ne!(compile_original("ashlar-spark4",&old).unwrap()["status"],"compiled");
  assert_ne!(compile_original("ashlar-spark4-fields",&old).unwrap()["status"],"compiled");
 }
 #[test]fn field_only_original_profile_refuses_broader_semantics(){
  let base=request();
  for sql in [
   "SELECT r.wideAmount AS value FROM sales.\"order\" r WHERE r.id='a' ORDER BY r.id LIMIT 10",
   "SELECT r.wideAmount AS value FROM sales.\"order\" r JOIN sales.customer c ON r.id=c.customerId ORDER BY r.id LIMIT 10",
   "SELECT COUNT(*) AS value FROM sales.\"order\" r ORDER BY r.id LIMIT 10",
   "SELECT r AS value FROM sales.\"order\" r ORDER BY r.id LIMIT 10",
   "SELECT r.wideAmount AS value FROM sales.\"order\" r ORDER BY r.wideAmount LIMIT 10",
   "SELECT r.wideAmount AS value FROM sales.\"order\" r ORDER BY r.id",
   "SELECT r.wideAmount AS value FROM (SELECT r.id FROM sales.\"order\" r) r ORDER BY r.id LIMIT 10"
  ]{let mut r=base.clone();r["sql"]=json!(sql);assert_ne!(compile_original("ashlar-spark4-fields",&r).unwrap()["status"],"compiled","{sql}");}
  for optional in [true,false] {
   let mut r=base.clone();let mut doc:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
   let field=doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|f|f["id"]=="wideAmount").unwrap();
   if optional{field["nullability"]=json!("absent-allowed");}else{field["cardinality"]=json!("array");}
   let source=doc.to_string();let hash=weft_core::json::sha256(source.as_bytes());
   let old_hash=r["modules"][0]["pin"]["revision"].as_str().unwrap().to_owned();
   r["modules"][0]["documentJson"]=json!(source);r["modules"][0]["pin"]["revision"]=json!(hash);r["modules"][0]["pin"]["sha256"]=json!(hash);
   let binding=r["target"]["bindingJson"].as_str().unwrap().replace(&old_hash,&hash);
   r["target"]["bindingSha256"]=json!(weft_core::json::sha256(binding.as_bytes()));r["target"]["bindingJson"]=json!(binding);
   let out=compile_original("ashlar-spark4-fields",&r).unwrap();assert_ne!(out["status"],"compiled","{out}");
   if optional {assert!(out["original"]["diagnostics"].as_array().unwrap().iter().any(|d|d["code"]=="WFT-FIELD-PROFILE"),"{out}");}
  }
  for (key,value) in [("interfaceVersion",json!("weft-compile/0.2.5")),("parameters",json!({"amount":{"family":"decimal","facets":{"precision":38,"scale":6},"nullable":false}}))] {
   let mut r=base.clone();r[key]=value;assert_ne!(compile_original("ashlar-spark4-fields",&r).unwrap()["status"],"compiled");
  }
 }
}

#[cfg(all(test,feature="ashlar-sdk-ordered-page-candidate"))]
mod sdk_presence_tests {
 use super::*;
 fn request()->Value{serde_json::from_str(include_str!("../../../tests/engine-catalog/ashlar-sdk-presence-request.json")).unwrap()}
 fn repin_source(r:&mut Value, old_revision:&str, old_hash:&str, new_revision:&str, new_hash:&str, version:&str){
  fn replace(value:&mut Value, old_revision:&str, old_hash:&str, new_revision:&str, new_hash:&str, version:&str){
   match value {
    Value::String(s) if s==old_revision => *s=new_revision.to_owned(),
    Value::String(s) if s==old_hash => *s=new_hash.to_owned(),
    Value::String(s) if s=="0.10.0" => *s=version.to_owned(),
    Value::Array(items) => for item in items { replace(item,old_revision,old_hash,new_revision,new_hash,version); },
    Value::Object(items) => for item in items.values_mut() { replace(item,old_revision,old_hash,new_revision,new_hash,version); },
    _ => {}
   }
  }
  let pin=&mut r["modules"][0]["pin"];
  pin["revision"]=json!(new_revision);pin["sha256"]=json!(new_hash);pin["umfVersion"]=json!(version);
  let mut binding:Value=serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();
  replace(&mut binding,old_revision,old_hash,new_revision,new_hash,version);
  let text=encode(&binding);r["target"]["bindingSha256"]=json!(weft_core::json::sha256(text.as_bytes()));r["target"]["bindingJson"]=json!(text);
 }
 #[test]fn original_scalar_presence_keeps_null_missing_and_source_distinct(){
  let r=request();let packet=json!({"version":"weft.engine-read/0.2","compiler":"ashlar-spark4-sdk","request":r});
  let p:Value=serde_json::from_str(&compile_json(&packet.to_string())).unwrap();assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["version"],"weft.engine-read/0.2");assert_eq!(p["request"],r);
  assert_eq!(p["outputs"][0]["missing"],false);assert_eq!(p["outputs"][0]["nullable"],false);
  assert_eq!(p["outputs"][1]["missing"],true);assert_eq!(p["outputs"][1]["nullable"],true);
  for (i,o) in p["outputs"].as_array().unwrap().iter().enumerate(){assert_eq!(o["representation"],p["artifact"]["columns"][i]["representation"]);assert_eq!(o["modelPins"],p["artifact"]["modelPins"]);assert_eq!(o["source"],p["artifact"]["columns"][i]["sourceIdentities"][0]);}
  let checked:Value=serde_json::from_str(&check_json(&p.to_string(),&r.to_string())).unwrap();assert_eq!(checked,p);
  assert!(p["artifact"]["sql"].as_str().unwrap().contains("'null'"));
  if let Ok(path)=std::env::var("WEFT_SDK_PRESENCE_SAMPLE"){std::fs::write(path,serde_json::to_string_pretty(&p).unwrap()).unwrap();}
 }
 #[test]fn new_presence_profile_requires_original_core010_domain(){
  let mut r=request();let mut doc:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();doc["umf"]=json!("0.7.0");let text=encode(&doc);let hash=weft_core::json::sha256(text.as_bytes());let old_revision=r["modules"][0]["pin"]["revision"].as_str().unwrap().to_owned();let old_hash=r["modules"][0]["pin"]["sha256"].as_str().unwrap().to_owned();r["modules"][0]["documentJson"]=json!(text);repin_source(&mut r,&old_revision,&old_hash,&hash,&hash,"0.7.0");
  let p=compile_original("ashlar-spark4-sdk",&r).unwrap();assert_eq!(p["status"],"blocked","{p}");assert!(p["original"]["diagnostics"].as_array().unwrap().iter().any(|d|d["code"]=="WFT-INPUT"),"{p}");
 }
 #[test]fn retired_core08_sources_refuse_and_temporal_carriers_remain_held(){
  let mut legacy=request();
  let mut source:Value=serde_json::from_str(legacy["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
  source["umf"]=json!("0.8.0");
  let text=encode(&source);let hash=weft_core::json::sha256(text.as_bytes());
  let old_revision=legacy["modules"][0]["pin"]["revision"].as_str().unwrap().to_owned();let old_hash=legacy["modules"][0]["pin"]["sha256"].as_str().unwrap().to_owned();
  legacy["modules"][0]["documentJson"]=json!(text);repin_source(&mut legacy,&old_revision,&old_hash,&hash,&hash,"0.8.0");
  let refused=compile_original("ashlar-spark4-sdk",&legacy).unwrap();assert_eq!(refused["status"],"blocked","{refused}");assert!(refused["original"]["diagnostics"].as_array().unwrap().iter().any(|d|d["code"]=="WFT-INPUT"),"{refused}");
  for temporal in ["date","timestamp","local-date-time"] {
   let mut r=request();let mut doc:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();doc["modules"][0]["elements"][1]["scalarType"]=json!(temporal);
   let old=r["modules"][0]["pin"]["revision"].as_str().unwrap().to_string();let text=doc.to_string();let hash=weft_core::json::sha256(text.as_bytes());r["modules"][0]["documentJson"]=json!(text);r["modules"][0]["pin"]["revision"]=json!(hash);r["modules"][0]["pin"]["sha256"]=json!(hash);let binding=r["target"]["bindingJson"].as_str().unwrap().replace(&old,&hash);r["target"]["bindingSha256"]=json!(weft_core::json::sha256(binding.as_bytes()));r["target"]["bindingJson"]=json!(binding);
   assert_ne!(compile_original("ashlar-spark4-sdk",&r).unwrap()["status"],"compiled","Temporal carrier must remain held");
  }
 }
 #[test]fn old_versions_forged_states_source_and_computed_outputs_refuse(){
  let r=request();for version in ["weft.engine-read/0.1","weft.engine-read/0.3"] {let p:Value=serde_json::from_str(&compile_json(&json!({"version":version,"compiler":"ashlar-spark4-sdk","request":r}).to_string())).unwrap();assert_ne!(p["status"],"compiled");}
  let p=compile_original("ashlar-spark4-sdk",&r).unwrap();for key in ["missing","nullable","source","representation","modelPins"] {let mut forged=p.clone();forged["outputs"][1][key]=Value::Null;let checked:Value=serde_json::from_str(&check_json(&forged.to_string(),&r.to_string())).unwrap();assert_eq!(checked["status"],"blocked");}
  let mut computed=r.clone();computed["sql"]=json!("SELECT COUNT(*) AS value FROM sales.\"order\" r");assert_ne!(compile_original("ashlar-spark4-sdk",&computed).unwrap()["status"],"compiled");
  let mut legacy=r.clone();legacy["target"]["backendVersion"]=json!(weft_databricks::local_spark4::VERSION);legacy["target"]["targetProfile"]=json!(weft_databricks::local_spark4::PROFILE);assert_ne!(compile_original("ashlar-spark4",&legacy).unwrap()["status"],"compiled");
 }
}

#[cfg(all(test,feature="ashlar-sdk-ordered-page-candidate"))]mod binary_domain_tests{
 use super::*;
 fn request()->Value{serde_json::from_str(include_str!("../../../tests/engine-catalog/native-truss-binary-request.json")).unwrap()}
 #[test]fn current_core10_binary_state_domain_is_distinct_from_ashlar_envelopes(){
  let r=request();let out=compile_original("truss-runtime-binary",&r).unwrap();assert_eq!(out["status"],"compiled","{out}");assert_eq!(out["version"],"weft.engine-read/0.2");assert_eq!(out["outputs"][1]["carrier"],"binary");assert_eq!(out["outputs"][1]["stateDomain"]["states"],json!(["value"]));assert_eq!(out["outputs"][1]["nullable"],false);assert_eq!(out["outputs"][2]["stateDomain"]["states"],json!(["missing","null","value"]));assert_eq!(out["outputs"][2]["nullable"],true);assert_eq!(out["outputs"][2]["missing"],true);
  assert_eq!(bind(&out,&r).unwrap(),out);assert!(compile_original("truss-runtime",&r).is_err());
  let mut forged=out.clone();forged["outputs"][1]["stateDomain"]["states"]=json!(["null","value"]);forged.as_object_mut().unwrap().remove("structuralDigest");forged["structuralDigest"]=json!({"owner":"weft-runtime-engine-read","version":"0.2","sha256":weft_core::json::sha256(encode(&forged).as_bytes())});assert!(bind(&forged,&r).is_err());
  if let Ok(dir)=std::env::var("WEFT_BINARY_SAMPLE_DIR"){std::fs::create_dir_all(&dir).unwrap();std::fs::write(format!("{dir}/read.json"),serde_json::to_string_pretty(&out).unwrap()).unwrap();}
 }
 #[test]fn original_optional_scalar_sibling_preserves_missing_and_null(){
  let mut r=request();let mut doc:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();let field=doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="A-optional").unwrap();field["scalarType"]=json!("string");let text=doc.to_string();let hash=weft_core::json::sha256(text.as_bytes());r["modules"][0]["documentJson"]=json!(text);r["modules"][0]["pin"]["sha256"]=json!(hash);let mut binding:Value=serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();binding["documents"][0]["sha256"]=json!(hash);let text=binding.to_string();r["target"]["bindingSha256"]=json!(weft_core::json::sha256(text.as_bytes()));r["target"]["bindingJson"]=json!(text);
  let out=compile_original("truss-runtime-binary",&r).unwrap();assert_eq!(out["status"],"compiled","{out}");assert_eq!(out["outputs"][2]["carrier"],"string");assert_eq!(out["outputs"][2]["missing"],true);assert_eq!(out["outputs"][2]["nullable"],true);assert_eq!(out["outputs"][0]["missing"],false);
  if let Ok(dir)=std::env::var("WEFT_BINARY_SAMPLE_DIR"){std::fs::create_dir_all(&dir).unwrap();std::fs::write(format!("{dir}/optional-scalar-sibling.json"),serde_json::to_string_pretty(&out).unwrap()).unwrap();}
 }
 #[test]fn binary_projection_page_delegates_original_scalar_key_cursor(){
  let captured:Value=serde_json::from_str(include_str!("../../../tests/engine-catalog/native-truss-binary-page-request.json")).unwrap();assert_ne!(compile_page_original("truss-runtime-binary",&captured,2,None).unwrap()["status"],"compiled","captured nonprimary ordering remains an original owner refusal");let mut r=captured.clone();r["sql"]=json!("SELECT a.\"A-label\" AS label,a.blob AS blob,a.optional_blob AS optional_blob FROM \"A\" a");let key=json!({"module":"m","record":"A","key":"a-primary"});assert!(binary_page(&r,2,None,None).is_err());assert!(binary_page(&r,2,None,Some(&json!({"module":"m","record":"A","key":"z-alt"}))).is_err());let p=binary_page(&r,2,None,Some(&key)).unwrap();assert_eq!(p["selectedKey"],key);assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["version"],"weft.engine-page/0.2");assert_eq!(p["outputs"][1]["carrier"],"binary");let checked:Value=serde_json::from_str(&check_page_json(&p.to_string(),&r.to_string(),"null")).unwrap();assert_eq!(checked,p);
  if let Ok(dir)=std::env::var("WEFT_BINARY_SAMPLE_DIR"){std::fs::create_dir_all(&dir).unwrap();std::fs::write(format!("{dir}/page.json"),serde_json::to_string_pretty(&p).unwrap()).unwrap();}
 }
}

#[cfg(all(test,not(feature="ashlar-sdk-ordered-page-candidate")))]mod sdk_feature_boundary{
 use super::*;
 #[test]fn new_engine_profiles_require_explicit_candidate_feature(){
  let request:Value=serde_json::from_str(include_str!("../../../tests/engine-catalog/ashlar-sdk-presence-request.json")).unwrap();
  for method in ["ashlar-spark4-sdk","ashlar-spark4-sdk-pages","truss-runtime-binary"]{let out:Value=serde_json::from_str(&compile_json(&json!({"version":"weft.engine-read/0.2","compiler":method,"request":request}).to_string())).unwrap();assert_eq!(out["status"],"blocked");assert!(out.get("artifact").is_none());}
 }
}
