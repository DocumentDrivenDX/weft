//! Engine catalog custody. Engines retain native admission and execution.
//! This module never lowers SQL or interprets engine storage or authority.
use serde::Deserialize;
use serde_json::{json,Value};
type Result<T>=std::result::Result<T,&'static str>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct Request { version:String, compiler:String, request:Value }
fn encode(v:&Value)->String{weft_core::mutation::canonical(v)}
fn compile_original(method:&str,request:&Value)->Result<Value>{
 let raw=match method{
  "truss-runtime"=>weft_postgresql::truss_runtime::compile_json(&encode(request)),
  "registered"=>crate::compile_json(&encode(request)),
  "ashlar-spark4"=>{
    let mut registry=weft_core::backend::Registry::default();
    registry.register(weft_databricks::local_spark4::LocalSpark4).map_err(|_|"original local Spark backend registration refused")?;
    weft_core::compile::Compiler { registry }.compile_json(&encode(request))
  },
  _=>return Err("unrecognized original engine compiler")};
 let artifact=weft_core::json::checked_json(&raw).map_err(|_|"original compiler output refused")?;
 if artifact["status"]!="compiled" {return Ok(json!({"version":"weft.engine-read/0.1","status":"blocked","original":artifact}))}
 if request.get("exactParameters").is_some_and(|p|p.as_object().is_none_or(|m|!m.is_empty())){return Err("engine catalog exact binary/temporal parameter profile not selected")}
 let parameters=request["parameters"].as_object().ok_or("complete declared parameter object required")?;
 for parameter in parameters.values(){let p=parameter.as_object().ok_or("original scalar parameter required")?;if p.len()!=2||!p.contains_key("family")||!p.contains_key("value")||!matches!(p["family"].as_str(),Some("string"|"boolean"|"integer"|"decimal"))||!p["value"].is_string(){return Err("closed non-temporal scalar parameter required")}}
 let columns=artifact["columns"].as_array().ok_or("complete engine output inventory required")?;let mut outputs=vec![];
 for column in columns{let representation=&column["representation"];let logical=match representation["kind"].as_str(){Some("scalar")=>&representation["logicalType"],Some("value")=>{let descriptor=artifact["logicalPlan"]["typeGraph"].as_array().ok_or("original output type graph required")?.iter().find(|d|d["identity"]==representation["descriptor"]).ok_or("original output descriptor absent")?;if descriptor["kind"]!="scalar" || !matches!(descriptor["type"]["family"].as_str(),Some("string"|"boolean"|"integer"|"decimal")){return Err("engine output carrier profile unsupported")};outputs.push(json!({"name":column["outputName"],"carrier":descriptor["type"]["family"],"nullable":descriptor["type"]["nullable"]==true||descriptor["availability"]=="absent-allowed"}));continue;},_=>return Err("engine output representation unsupported")};
  if !matches!(logical["family"].as_str(),Some("string"|"boolean"|"integer"|"decimal")){return Err("engine output carrier profile unsupported")};outputs.push(json!({"name":column["outputName"],"carrier":logical["family"],"nullable":logical["nullable"]}));
 }
 let mut out=json!({"version":"weft.engine-read/0.1","status":"compiled","compiler":method,"request":request,"artifact":artifact,"outputs":outputs});
 out["structuralDigest"]=json!({"owner":"weft-runtime-engine-read","version":"0.1","sha256":weft_core::json::sha256(encode(&out).as_bytes())});Ok(out)
}
fn checked_program(program:&Value)->Result<(&str,&Value)>{
 if program["version"]!="weft.engine-read/0.1"||program["status"]!="compiled"{return Err("original compiled engine catalog required")}
 let method=program["compiler"].as_str().ok_or("original compiler required")?;
 let request=&program["request"];
 if compile_original(method,request)?!=*program{return Err("original engine program differs from its compiler")}
 Ok((method,request))
}
fn bind(program:&Value,request:&Value)->Result<Value>{
 let(method,original)=checked_program(program)?;
 let mut expected=original.clone();expected["parameters"]=request["parameters"].clone();
 if expected!=*request{return Err("engine model, binding, SQL or compiler context drift")}
 let before=original["parameters"].as_object().ok_or("original parameter inventory required")?;
 let after=request["parameters"].as_object().ok_or("current parameter inventory required")?;
 if before.keys().ne(after.keys())||before.iter().any(|(name,value)|after[name]["family"]!=value["family"]){return Err("engine parameter inventory or family differs")}
 compile_original(method,request)
}
fn response(result:Result<Value>)->String{result.unwrap_or_else(|code|json!({"version":"weft.engine-read/0.1","status":"blocked","diagnostics":[{"code":"WFT-ENGINE-READ","message":code}]})).to_string()}
pub fn compile_json(raw:&str)->String{response((||{let v=weft_core::json::checked_json(raw).map_err(|_|"request framing refused")?;let r:Request=serde_json::from_value(v).map_err(|_|"closed engine catalog request required")?;if r.version!="weft.engine-read/0.1"{return Err("engine catalog version differs")};compile_original(&r.compiler,&r.request)})())}
pub fn check_json(program:&str,current_request:&str)->String{response((||{let program=weft_core::json::checked_json(program).map_err(|_|"program framing refused")?;let request=weft_core::json::checked_json(current_request).map_err(|_|"request framing refused")?;bind(&program,&request)})())}
#[cfg(test)]mod tests{
 use super::*;
 fn request()->Value{let mut r:Value=serde_json::from_str(include_str!("../../../tests/truss-decimal12/fixtures/genuine-northwind-compile-request.json")).unwrap();r["target"]["backendVersion"]=json!("0.4.39");r}
 fn program(r:&Value)->Value{serde_json::from_str(&compile_json(&json!({"version":"weft.engine-read/0.1","compiler":"truss-runtime","request":r}).to_string())).unwrap()}
 #[test]fn original_compiler_and_current_binding_agree_without_losing_decimal_spelling(){let r=request();let p=program(&r);assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["request"],r);let mut current=r.clone();current["parameters"]["price"]["value"]=json!("-0002.5000");let a:Value=serde_json::from_str(&check_json(&p.to_string(),&current.to_string())).unwrap();assert_eq!(a["status"],"compiled","{a}");assert_eq!(a["request"]["parameters"]["price"]["value"],"-0002.5000");assert!(a["artifact"]["parameters"].as_array().unwrap().iter().any(|s|s["value"]=="-0002.5000"));}
 #[test]fn changed_context_family_and_program_never_match(){let r=request();let p=program(&r);for mode in 0..5{let mut current=r.clone();let mut original=p.clone();match mode{0=>current["sql"]=json!("SELECT 1"),1=>current["target"]["bindingSha256"]=json!("0".repeat(64)),2=>current["parameters"]["price"]["family"]=json!("string"),3=>current["parameters"]["extra"]=json!({"family":"string","value":"private"}),_=>original["artifact"]["sql"]=json!("SELECT 1")};let a:Value=serde_json::from_str(&check_json(&original.to_string(),&current.to_string())).unwrap();assert_eq!(a["status"],"blocked");assert!(a.get("artifact").is_none());}}
 #[test]fn native_core08_engine_requests_keep_original_profiles_and_sources(){
  for (method,raw) in [("truss-runtime",include_str!("../../../tests/engine-catalog/native-truss-request.json")),("ashlar-spark4",include_str!("../../../tests/engine-catalog/native-ashlar-spark4-request.json"))]{
   let request:Value=serde_json::from_str(raw).unwrap();let out:Value=serde_json::from_str(&compile_json(&json!({"version":"weft.engine-read/0.1","compiler":method,"request":request}).to_string())).unwrap();
   assert_eq!(out["status"],"compiled","{out}");assert_eq!(out["request"],request);assert_eq!(out["artifact"]["modelPins"][0]["umfVersion"],"0.8.0");
   if method=="ashlar-spark4" {let obligation=out["artifact"]["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.nativeProfile").unwrap();assert_eq!(obligation["parameters"]["engineVersion"],"4.0.1");assert_eq!(obligation["parameters"]["deltaVersion"],"4.0.0");assert_eq!(out["artifact"]["backend"]["targetProfile"],"spark4.0.1-delta4.0.0-candidate");}
   let mut changed=request.clone();changed["modules"][0]["documentJson"]=json!(format!("{} ",request["modules"][0]["documentJson"].as_str().unwrap()));let checked:Value=serde_json::from_str(&check_json(&out.to_string(),&changed.to_string())).unwrap();assert_eq!(checked["status"],"blocked");
  }
 }
 #[test]fn blocked_original_diagnostics_are_retained(){let mut r=request();r["parameters"]["price"]["value"]=json!("0.001");let p=program(&r);assert_eq!(p["status"],"blocked");assert_eq!(p["original"]["status"],"blocked");assert!(p["original"]["diagnostics"].as_array().is_some_and(|d|!d.is_empty()));}
}
