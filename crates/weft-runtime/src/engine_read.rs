//! Engine catalog custody. Engines retain native admission and execution.
//! This module never lowers SQL or interprets engine storage or authority.
use serde::Deserialize;
use serde_json::{json,Value};
type Result<T>=std::result::Result<T,&'static str>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct Request { version:String, compiler:String, request:Value }
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
 let expressions=artifact["logicalPlan"]["outputs"].as_array().ok_or("original application output inventory required")?;
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
fn compile_original(method:&str,request:&Value)->Result<Value>{
 let raw=match method{
  "truss-runtime"=>weft_postgresql::truss_runtime::compile_json(&encode(request)),
  "registered"=>crate::compile_json(&encode(request)),
  "ashlar-spark4"=>{
    let mut registry=weft_core::backend::Registry::default();
    registry.register(weft_databricks::local_spark4::LocalSpark4).map_err(|_|"original local Spark backend registration refused")?;
    weft_core::compile::Compiler { registry }.compile_json(&encode(request))
  },
  "ashlar-spark4-pages"=>{
    let mut registry=weft_core::backend::Registry::default();registry.register(weft_databricks::authored_pages::local()).map_err(|_|"original local page backend registration refused")?;
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
pub(crate) fn wrap_artifact(method:&str,request:&Value,artifact:Value)->Result<Value>{
 if artifact["status"]!="compiled" {return Ok(json!({"version":"weft.engine-read/0.1","status":"blocked","original":artifact}))}
 if request.get("exactParameters").is_some_and(|p|p.as_object().is_none_or(|m|!m.is_empty())){return Err("engine catalog exact binary/temporal parameter profile not selected")}
 let parameters=request["parameters"].as_object().ok_or("complete declared parameter object required")?;
 for parameter in parameters.values(){let p=parameter.as_object().ok_or("original scalar parameter required")?;if p.len()!=2||!p.contains_key("family")||!p.contains_key("value")||!matches!(p["family"].as_str(),Some("string"|"boolean"|"integer"|"decimal"))||!p["value"].is_string(){return Err("closed non-temporal scalar parameter required")}}
 let columns=artifact["columns"].as_array().ok_or("complete engine output inventory required")?;let mut outputs=vec![];
 for column in columns{let representation=&column["representation"];if representation["kind"]=="relatedKeys"{outputs.push(related_output(column,&artifact)?);continue;}let logical=match representation["kind"].as_str(){Some("scalar")=>&representation["logicalType"],Some("value")=>{let descriptor=artifact["logicalPlan"]["typeGraph"].as_array().ok_or("original output type graph required")?.iter().find(|d|d["identity"]==representation["descriptor"]).ok_or("original output descriptor absent")?;if descriptor["kind"]!="scalar" || !matches!(descriptor["type"]["family"].as_str(),Some("string"|"boolean"|"integer"|"decimal")){return Err("engine output carrier profile unsupported")};outputs.push(json!({"name":column["outputName"],"carrier":descriptor["type"]["family"],"nullable":descriptor["type"]["nullable"]==true||descriptor["availability"]=="absent-allowed"}));continue;},_=>return Err("engine output representation unsupported")};
  if !matches!(logical["family"].as_str(),Some("string"|"boolean"|"integer"|"decimal")){return Err("engine output carrier profile unsupported")};outputs.push(json!({"name":column["outputName"],"carrier":logical["family"],"nullable":logical["nullable"]}));
 }
 let mut out=json!({"version":"weft.engine-read/0.1","status":"compiled","compiler":method,"request":request,"artifact":artifact,"outputs":outputs});
 out["structuralDigest"]=json!({"owner":"weft-runtime-engine-read","version":"0.1","sha256":weft_core::json::sha256(encode(&out).as_bytes())});Ok(out)
}
fn checked_program(program:&Value)->Result<(&str,&Value)>{
 if program["version"]!="weft.engine-read/0.1"||program["status"]!="compiled"{return Err("original compiled engine catalog required")}
 let method=program["compiler"].as_str().ok_or("original compiler required")?;
 let request=&program["request"];
 let expected=if let Some(packet)=program.get("relatedRequest"){crate::engine_related_read::compile_value(packet)?}else{compile_original(method,request)?};
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
 if let Some(packet)=program.get("relatedRequest"){let mut packet=packet.clone();packet["source"]["parameters"]=request["parameters"].clone();packet["target"]["parameters"]=request["parameters"].clone();crate::engine_related_read::compile_value(&packet)}else{compile_original(method,request)}
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
  for (method,raw) in [("truss-runtime",include_str!("../../../tests/engine-catalog/native-truss-request.json")),("ashlar-spark4",include_str!("../../../tests/engine-catalog/native-ashlar-spark4-request.json")),("ashlar-dbsql-20261016",include_str!("../../../tests/engine-catalog/native-ashlar-dbsql-20261016-request.json"))]{
   let request:Value=serde_json::from_str(raw).unwrap();let out:Value=serde_json::from_str(&compile_json(&json!({"version":"weft.engine-read/0.1","compiler":method,"request":request}).to_string())).unwrap();
   assert_eq!(out["status"],"compiled","{out}");assert_eq!(out["request"],request);assert_eq!(out["artifact"]["modelPins"][0]["umfVersion"],"0.8.0");
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
struct PageRequest { version:String, compiler:String, request:Value, page_size:u16 }
fn compile_page_original(method:&str,request:&Value,size:u16,after:Option<Vec<String>>)->Result<Value>{
 if !matches!(method,"truss-runtime"|"ashlar-spark4-pages"|"ashlar-dbsql-20261016-pages") || !(1..=999).contains(&size){return Err("explicit original authored-key page compiler required")}
 let base=compile_original(method,request)?;
 if base["status"]!="compiled" {return Ok(json!({"version":"weft.engine-page/0.1","status":"blocked","original":base}))}
 let raw=if method=="truss-runtime"{weft_postgresql::truss_runtime::compile_page_json(&encode(request),size,after.clone())}else{
  let mut factory=|_:&weft_core::model::Catalog,_:weft_core::backend::Plan<'_>,_:weft_core::compile::CompositionInput<'_>|{
   let mut registry=weft_core::backend::Registry::default();
   if method=="ashlar-spark4-pages"{registry.register(weft_databricks::authored_pages::local())?;}else{registry.register(weft_databricks::authored_pages::managed())?;}
   Ok(registry)
  };
  weft_core::compile::Compiler::default().compile_typed_page_json_with_exact_numbers_factory(&encode(request),size,after.clone(),&mut factory)
 };
 let page=weft_core::json::checked_json(&raw).map_err(|_|"original page compiler output refused")?;
 if page["status"]!="compiled" {return Ok(json!({"version":"weft.engine-page/0.1","status":"blocked","original":page}))}
 let visible=base["outputs"].as_array().ok_or("complete visible output signature required")?.len();
 if page["interfaceVersion"]!="weft-sql-page/0.2" || page["page"]["visibleOutputs"]!=visible || page["page"]["pageSize"]!=size {return Err("original typed page output and bound differ")}
 let mut out=json!({"version":"weft.engine-page/0.1","status":"compiled","compiler":method,"request":request,"pageSize":size,"after":after,"page":page["page"],"artifact":page["artifact"],"outputs":base["outputs"]});
 out["structuralDigest"]=json!({"owner":"weft-runtime-engine-page","version":"0.1","sha256":weft_core::json::sha256(encode(&out).as_bytes())});Ok(out)
}
fn page_response(result:Result<Value>)->String{result.unwrap_or_else(|code|json!({"version":"weft.engine-page/0.1","status":"blocked","diagnostics":[{"code":"WFT-ENGINE-PAGE","message":code}]})).to_string()}
pub fn compile_page_json(raw:&str)->String{page_response((||{
 if raw.len()>4_194_304{return Err("bounded original page request required")}
 let value=weft_core::json::checked_json(raw).map_err(|_|"page request framing refused")?;
 let r:PageRequest=serde_json::from_value(value).map_err(|_|"closed page catalog request required")?;
 if r.version!="weft.engine-page/0.1"{return Err("page catalog version differs")}
 compile_page_original(&r.compiler,&r.request,r.page_size,None)
})())}
pub fn check_page_json(program:&str,current_request:&str,after_json:&str)->String{page_response((||{
 if program.len()>4_194_304||current_request.len()>4_194_304||after_json.len()>1_048_576{return Err("bounded original page program, current request and continuation required")}
 let program=weft_core::json::checked_json(program).map_err(|_|"page program framing refused")?;
 let current=weft_core::json::checked_json(current_request).map_err(|_|"current page request framing refused")?;
 let after=weft_core::json::checked_json(after_json).map_err(|_|"page continuation framing refused")?;
 let after:Option<Vec<String>>=serde_json::from_value(after).map_err(|_|"original lexical page continuation required")?;
 if after.as_ref().is_some_and(|values|values.is_empty()||values.len()>1024||values.iter().any(|v|v.len()>1_048_576)){return Err("bounded complete page continuation required")}
 if program["version"]!="weft.engine-page/0.1" || program["status"]!="compiled" || !program["after"].is_null(){return Err("original first-page catalog required")}
 if let Some(packet)=program.get("relatedRequest"){
  if crate::engine_related_read::compile_page_value(packet,None)?!=program{return Err("original related page differs from its compiler")}
  let read=crate::engine_related_read::page_read_packet(packet)?;let base=crate::engine_related_read::compile_value(&read)?;let checked=bind(&base,&current)?;
  if checked["status"]!="compiled"{return Ok(json!({"version":"weft.engine-page/0.1","status":"blocked","original":checked}))}
  let mut packet=packet.clone();packet["source"]["parameters"]=current["parameters"].clone();packet["target"]["parameters"]=current["parameters"].clone();return crate::engine_related_read::compile_page_value(&packet,after)
 }
 let method=program["compiler"].as_str().ok_or("original page compiler required")?;
 let size=program["pageSize"].as_u64().and_then(|v|u16::try_from(v).ok()).ok_or("original page bound required")?;
 let original=&program["request"];
 if compile_page_original(method,original,size,None)?!=program{return Err("original page program differs from its compiler")}
 // The existing read checker owns exact source/context and parameter inventory.
 let base=compile_original(method,original)?;let checked=bind(&base,&current)?;
 if checked["status"]!="compiled"{return Ok(json!({"version":"weft.engine-page/0.1","status":"blocked","original":checked}))}
 compile_page_original(method,&current,size,after)
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
  let r:Value=serde_json::from_str(include_str!("../../../tests/engine-catalog/native-ashlar-related-page-request.json")).unwrap();
  let p=compile_page_original("ashlar-spark4-pages",&r,2,None).unwrap();assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["request"],r);
  assert_eq!(p["outputs"].as_array().unwrap().len(),2);assert_eq!(p["outputs"][1]["decoder"]["profile"],"ashlar-related-keys-string/0.1");assert_eq!(p["outputs"][1]["decoder"]["bound"],2);
  assert_eq!(p["artifact"]["columns"].as_array().unwrap().len(),3);assert_eq!(p["page"]["keyOutputs"],json!([2]));assert_eq!(p["outputs"][1]["decoder"]["representation"],p["artifact"]["columns"][1]["representation"]);
  let a:Value=serde_json::from_str(&check_page_json(&p.to_string(),&r.to_string(),"[\"o-1\"]")).unwrap();assert_eq!(a["status"],"compiled","{a}");assert_eq!(a["after"],json!(["o-1"]));assert_eq!(a["outputs"],p["outputs"]);assert_ne!(a["artifact"]["sql"],p["artifact"]["sql"]);
  let mut forged=p.clone();forged["outputs"][1]["decoder"]["bound"]=json!(3);let refused:Value=serde_json::from_str(&check_page_json(&forged.to_string(),&r.to_string(),"null")).unwrap();assert_eq!(refused["status"],"blocked");
 }
 #[test]fn original_page_retains_hidden_key_and_native_continuation(){let r=request();let p=program(&r);assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["outputs"].as_array().unwrap().len(),2);assert_eq!(p["artifact"]["columns"].as_array().unwrap().len(),3);assert_eq!(p["page"]["keyOutputs"],json!([2]));let a:Value=serde_json::from_str(&check_page_json(&p.to_string(),&r.to_string(),"[\"o-1\"]")).unwrap();assert_eq!(a["status"],"compiled","{a}");assert_eq!(a["after"],json!(["o-1"]));assert_ne!(a["artifact"]["sql"],p["artifact"]["sql"]);assert!(a["artifact"]["parameters"].as_array().unwrap().iter().any(|v|v["value"]=="o-1"));}
 #[test]fn original_ashlar_page_compilation_retains_snapshot_key_obligation(){for (method,version,raw) in [("ashlar-spark4-pages",weft_databricks::authored_pages::LOCAL_VERSION,include_str!("../../../tests/engine-catalog/native-ashlar-spark4-request.json")),("ashlar-dbsql-20261016-pages",weft_databricks::authored_pages::MANAGED_VERSION,include_str!("../../../tests/engine-catalog/native-ashlar-dbsql-20261016-request.json"))]{let mut r:Value=serde_json::from_str(raw).unwrap();assert_ne!(compile_page_original(method,&r,2,None).unwrap()["status"],"compiled");r["interfaceVersion"]=json!("weft-compile/0.2.2");r["target"]["backendVersion"]=json!(version);let p=compile_page_original(method,&r,2,None).unwrap();assert_eq!(p["status"],"compiled","{p}");assert!(p["artifact"]["obligations"].as_array().unwrap().iter().any(|o|o["id"]=="ashlar.candidate.keyIntegrity"));let a:Value=serde_json::from_str(&check_page_json(&p.to_string(),&r.to_string(),"[\"bar\"]")).unwrap();assert_eq!(a["status"],"compiled","{a}");}}
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
