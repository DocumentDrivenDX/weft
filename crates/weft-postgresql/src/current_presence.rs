// SPDX-License-Identifier: MIT OR Apache-2.0
//! Conditional tagged projection over a held native TEXT-cell materialization.
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::collections::{BTreeMap,BTreeSet};
use weft_core::{application_ir::Expression,current_presence::CurrentPresencePreparation,error::{Diagnostic,Result},ir::Identity,json::{checked_json,sha256},security_ontology::SecurityRef};
fn fail()->Diagnostic{Diagnostic::new("WFT-CURRENT-PRESENCE-SQL","capability","Tagged current projection lowering refused")}
fn identifier(s:&str)->Result<String>{if s.is_empty()||s.len()>63||s.contains('\0'){return Err(fail())}Ok(format!("\"{}\"",s.replace('"',"\"\"")))}
fn key(id:&Identity)->(&str,&str,&str,&str){(&id.document_id,&id.revision,&id.module,&id.element)}
fn literal(s:&str)->String{format!("'{}'",s.replace('\'',"''"))}
#[derive(Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Relation{schema:String,table:String,eligible_column:String,row_key_column:String}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
struct Reference{id:String,revision:String}
#[derive(Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Domain{document_id:String,revision:String}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
struct Transform{id:String,version:String}
#[derive(Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Column{field:SecurityRef,storage_effect:String,#[serde(default,skip_serializing_if="Option::is_none")]storage_transform:Option<Transform>,#[serde(default,skip_serializing_if="Option::is_none")]value_profile:Option<String>}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
struct Header{version:String,policy:Reference,protection:Reference,domains:Vec<Domain>,columns:Vec<Column>}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
struct Cell{field:Identity,column:String}
#[derive(Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Binding{version:String,capabilities:Vec<String>,source_request_sha256:String,preparation_sha256:String,source_binding_sha256:String,relation:Relation,header:Header,cells:Vec<Cell>}
/// Only the actual typed presence parent can issue this artifact. JSON is data,
/// never native admission; no uniform scalar logical type is assigned to cells.
pub struct CurrentPresenceSql<'a>{context:&'a CurrentPresencePreparation<'a>,sql:String,json:String}
impl CurrentPresenceSql<'_>{
 pub fn context(&self)->&CurrentPresencePreparation<'_>{self.context}
 pub fn sql(&self)->&str{&self.sql}
 pub fn json(&self)->&str{&self.json}
}
fn encode(value:&Value)->Result<String>{
 struct Buffer{bytes:Vec<u8>,left:usize}
 impl std::io::Write for Buffer{fn write(&mut self,b:&[u8])->std::io::Result<usize>{self.left=self.left.checked_sub(b.len()).ok_or_else(||std::io::Error::other("budget"))?;self.bytes.extend_from_slice(b);Ok(b.len())}fn flush(&mut self)->std::io::Result<()>{Ok(())}}
 let mut out=Buffer{bytes:Vec::new(),left:16_000_000};serde_json::to_writer(&mut out,value).map_err(|_|fail())?;String::from_utf8(out.bytes).map_err(|_|fail())
}
/// Compiles only an explicit new binding and context; never executes or admits it.
pub fn lower<'a>(context:&'a CurrentPresencePreparation<'a>,raw:&str)->Result<CurrentPresenceSql<'a>>{
 if raw.len()>4_000_000{return Err(fail())}
 let value=checked_json(raw).map_err(|_|fail())?;let binding:Binding=serde_json::from_value(value.clone()).map_err(|_|fail())?;
 let original=context.preparation();let plan=original.plan();
 if serde_json::to_value(&binding).map_err(|_|fail())?!=value||binding.version!="weft.postgresql.current-presence/0.1-candidate"
  ||binding.capabilities!=["tagged-current-projection"]||binding.source_request_sha256!=sha256(original.source_request_json().as_bytes())
  ||binding.preparation_sha256!=sha256(context.json().as_bytes())||binding.source_binding_sha256!=context.binding_scope().sha256
  ||plan.outputs.is_empty()||plan.outputs.len()>256||binding.cells.is_empty()||binding.cells.len()>256{return Err(fail())}
 if !plan.joins.is_empty()||!plan.filters.is_empty()||!plan.groups.is_empty()||!plan.order.is_empty()||plan.limit.is_some()||plan.aggregate{return Err(fail())}
 let prep=checked_json(context.json()).map_err(|_|fail())?;
 for (header,role) in [(&binding.header.policy,"policy"),(&binding.header.protection,"catalog")]{
  let pins=prep["sources"].as_array().ok_or_else(fail)?;let pin=pins.iter().find(|p|p["role"]==role).ok_or_else(fail)?;
  if pin["identity"]["documentId"]!=header.id||pin["identity"]["revision"]!=header.revision{return Err(fail())}
 }
 if binding.header.version!="umf.security.disclosure/0.2.0"||binding.header.columns.len()!=binding.cells.len(){return Err(fail())}
 let pins=original.catalog().pins();let mut domains=BTreeSet::new();let mut domain_documents=BTreeSet::new();
 for d in &binding.header.domains{if !domain_documents.insert(&d.document_id)||!domains.insert((&d.document_id,&d.revision))||!pins.iter().any(|p|p.document_id==d.document_id&&p.revision==d.revision){return Err(fail())}}
 let mut cells=BTreeMap::new();let mut names=BTreeSet::new();names.insert(binding.relation.eligible_column.as_str());
 if !names.insert(&binding.relation.row_key_column){return Err(fail())}
 for (index,(cell,column)) in binding.cells.iter().zip(&binding.header.columns).enumerate(){
  let field=SecurityRef{document_id:cell.field.document_id.clone(),module_id:cell.field.module.clone(),element_id:cell.field.element.clone()};
  if field!=column.field||!domains.contains(&(&cell.field.document_id,&cell.field.revision))||cells.insert(key(&cell.field),(index,&cell.column)).is_some()||!names.insert(&cell.column){return Err(fail())}
  match column.storage_effect.as_str(){"canonical" if column.storage_transform.is_none()=>{},"stored-transformed" if column.storage_transform.as_ref().is_some_and(|t|!t.id.is_empty()&&!t.version.is_empty()&&t.id.len()<=4096&&t.version.len()<=4096&&!t.id.contains('\0')&&!t.version.contains('\0'))=>{},_=>return Err(fail())}
  if column.value_profile.as_deref().is_some_and(|p|p!="umf-gregorian-date/1.0.0"){return Err(fail())}
 }
 let lineage=checked_json(original.json()).map_err(|_|fail())?;let requirements=lineage["readRequirements"]["fields"].as_array().ok_or_else(fail)?;
 let mut select=Vec::new();let mut columns=Vec::new();let mut selected=BTreeSet::new();
 for (position,output) in plan.outputs.iter().enumerate(){
  let Expression::Field{scan,identity}=&output.expression else{return Err(fail())};if scan!=&plan.source.occurrence{return Err(fail())}
  let (header_column,column)=cells.get(&key(identity)).ok_or_else(fail)?;selected.insert(key(identity));
  let requirement=requirements.iter().find(|r|r["path"]==format!("outputs/{position}/expression")).ok_or_else(fail)?;
  let availability=requirement["descriptor"]["availability"].as_str().ok_or_else(fail)?;
  let alternatives=match availability{"required"=>vec!["original","withheld","transformed"],"absent-allowed"=>vec!["original","absent","withheld","transformed"],_=>return Err(fail())};
  select.push(format!("s.{} AS {}",identifier(column)?,identifier(&format!("output_{position}"))?));
  columns.push(json!({"position":position,"name":output.name,"sourceField":identity,"headerColumn":header_column,
   "type":{"kind":"tagged-umf-disclosure-cell","originalDomain":requirement["descriptor"],"transformedDomain":"per-cell outputDomain resolved against the held header domains","alternatives":alternatives,"authorization":"actual held owner selection required"},
   "encoding":"weft.umf-disclosure-cell-text/0.1-candidate","nativeType":"text"}));
 }
 if selected.len()!=binding.cells.len(){return Err(fail())}
 let key=identifier(&binding.relation.row_key_column)?;
 select.push(format!("jsonb_build_object('version','weft.stage-row-tuple/0.1-candidate','rows',jsonb_build_array(jsonb_build_object('scan',{},'key',s.{})))::text AS \"__weft_source_rows\"",literal(&plan.source.occurrence),key));
 let sql=format!("SELECT {} FROM {}.{} AS s WHERE s.{} IS TRUE",select.join(", "),identifier(&binding.relation.schema)?,identifier(&binding.relation.table)?,identifier(&binding.relation.eligible_column)?);
 let artifact=json!({"version":"weft.postgresql.current-presence.sql/0.1-candidate","capabilities":["tagged-current-projection"],"authority":false,"securityReady":false,"nativeAdmission":"required","sourceRequestSha256":binding.source_request_sha256,"preparationSha256":binding.preparation_sha256,"materializationBindingSha256":sha256(raw.as_bytes()),"sql":sql,"parameters":[],"resultColumns":columns,"disclosureHeader":binding.header,
 "privateResultColumns":[{"position":columns.len(),"name":"__weft_source_rows","purpose":"source-row-contribution","encoding":"jsonb-text:weft.stage-row-tuple/0.1-candidate","maxKeyUtf8Bytes":256,"nativeType":"text","scans":[plan.source.occurrence]}],
 "obligations":["actual authenticated owner batch/header/source/caller/current cut","native owned TEXT cells and BOOL eligibility plus unique nonnull TEXT row keys","independent complete eligible population and exact statement/result custody","whole UMF V2 header/cell/domain/selected-occurrence validation","closing authority revalidation and guarded release; cleanup on every outcome"]});
 let json=encode(&artifact)?;Ok(CurrentPresenceSql{context,sql,json})
}

#[cfg(test)]
mod tests{
 use super::*;
 fn request()->Value{serde_json::from_str(include_str!("../../../tests/current-query-presence/fixtures/unadmitted-context.request.json")).unwrap()}
 fn binding(context:&CurrentPresencePreparation<'_>)->Value{
  let prep:Value=serde_json::from_str(context.json()).unwrap();let role=|name:&str|prep["sources"].as_array().unwrap().iter().find(|s|s["role"]==name).unwrap()["identity"].clone();
  let policy=role("policy");let catalog=role("catalog");let field=match &context.preparation().plan().outputs[0].expression{Expression::Field{identity,..}=>identity.clone(),_=>panic!()};
  json!({"version":"weft.postgresql.current-presence/0.1-candidate","capabilities":["tagged-current-projection"],"sourceRequestSha256":prep["sourceRequestSha256"],"preparationSha256":sha256(context.json().as_bytes()),"sourceBindingSha256":context.binding_scope().sha256,
   "relation":{"schema":"private_stage","table":"held_cells","eligibleColumn":"eligible","rowKeyColumn":"private_key"},
   "header":{"version":"umf.security.disclosure/0.2.0","policy":{"id":policy["documentId"],"revision":policy["revision"]},"protection":{"id":catalog["documentId"],"revision":catalog["revision"]},"domains":[{"documentId":field.document_id,"revision":field.revision}],"columns":[{"field":{"documentId":field.document_id,"moduleId":field.module,"elementId":field.element},"storageEffect":"stored-transformed","storageTransform":{"id":"vault","version":"v1"}}]},"cells":[{"field":field,"column":"token_cell"}]})
 }
 #[test]
 fn tagged_cell_sql_has_distinct_codec_header_and_actual_parent(){
  let original=weft_core::application_lineage::prepare(&request().to_string()).unwrap();let context=weft_core::current_presence::prepare(&original).unwrap();let input=binding(&context);let artifact=lower(&context,&input.to_string()).unwrap();
  assert!(std::ptr::eq(artifact.context(),&context));assert!(artifact.sql().contains("s.\"token_cell\" AS \"output_0\""));assert!(!artifact.sql().contains("COALESCE"));assert!(!artifact.sql().contains("->>"));
  let report:Value=serde_json::from_str(artifact.json()).unwrap();assert_eq!(report["authority"],false);assert_eq!(report["nativeAdmission"],"required");assert_eq!(report["disclosureHeader"],input["header"]);assert_eq!(report["resultColumns"][0]["type"]["alternatives"],json!(["original","absent","withheld","transformed"]));assert!(report["resultColumns"][0].get("logicalType").is_none());
 }
 #[test]
 fn old_profiles_changed_pins_headers_and_field_mappings_refuse(){
  let original=weft_core::application_lineage::prepare(&request().to_string()).unwrap();let context=weft_core::current_presence::prepare(&original).unwrap();
  for case in 0..7{let mut input=binding(&context);match case{
   0=>input["version"]=json!("weft.postgresql.current-values/0.5-candidate"),1=>input["preparationSha256"]=json!("0".repeat(64)),2=>input["header"]["policy"]["revision"]=json!("stale"),3=>input["header"]["domains"][0]["revision"]=json!("stale"),4=>input["cells"][0]["field"]["element"]=json!("salary"),5=>input["cells"][0]["column"]=json!("eligible"),_=>input["header"]["columns"][0]["storageTransform"]=Value::Null};assert!(lower(&context,&input.to_string()).is_err(),"{case}");}
 }
 #[test]
 fn repeated_projection_positions_share_one_cell_mapping_and_header(){
  let mut input=request();input["sql"]=json!("SELECT r.token AS first, r.token AS second FROM Resource r");let original=weft_core::application_lineage::prepare(&input.to_string()).unwrap();let context=weft_core::current_presence::prepare(&original).unwrap();let artifact=lower(&context,&binding(&context).to_string()).unwrap();let report:Value=serde_json::from_str(artifact.json()).unwrap();assert_eq!(report["resultColumns"].as_array().unwrap().len(),2);assert_eq!(report["resultColumns"][0]["headerColumn"],report["resultColumns"][1]["headerColumn"]);assert_eq!(report["disclosureHeader"]["columns"].as_array().unwrap().len(),1);
 }
 #[test]
 fn identifiers_are_quoted_and_native_text_impossibilities_refuse(){
  let original=weft_core::application_lineage::prepare(&request().to_string()).unwrap();let context=weft_core::current_presence::prepare(&original).unwrap();let mut input=binding(&context);
  input["relation"]["schema"]=json!("schema\"; SELECT 1; --");input["cells"][0]["column"]=json!("cell\"name");
  let artifact=lower(&context,&input.to_string()).unwrap();assert!(artifact.sql().contains("\"schema\"\"; SELECT 1; --\""));assert!(artifact.sql().contains("s.\"cell\"\"name\""));
  input["header"]["columns"][0]["storageTransform"]["id"]=json!("bad\0id");assert!(lower(&context,&input.to_string()).is_err());
  let input=binding(&context);let mut f=frame(&context,&input);f["rows"][0]["key"]=json!("bad\0key");assert!(check(&context,&input.to_string(),&f.to_string()).is_err());
 }
 #[test]
 fn exact_cell_bytes_and_private_scan_custody_are_required(){
  let original=weft_core::application_lineage::prepare(&request().to_string()).unwrap();let context=weft_core::current_presence::prepare(&original).unwrap();let input=binding(&context);
  let mut f=frame(&context,&input);let cell=f["resultRows"][0][0].as_str().unwrap();f["resultRows"][0][0]=json!(format!(" {cell}"));assert!(check(&context,&input.to_string(),&f.to_string()).is_err());
  let mut f=frame(&context,&input);let mut tuple:Value=serde_json::from_str(f["resultRows"][0][1].as_str().unwrap()).unwrap();tuple["rows"][0]["scan"]=json!("s1");f["resultRows"][0][1]=json!(tuple.to_string());assert!(check(&context,&input.to_string(),&f.to_string()).is_err());
 }
 #[test]
 fn actual_atomic_producer_batches_preserve_pinned_headers_cells_and_empty_denials(){
  let corpus:Value=serde_json::from_str(include_str!("../../../tests/current-query-presence/fixtures/actual-atomic-transport.json")).unwrap();
  assert_eq!(corpus["sourcePacketSha256"],"123be2df9ef80b9a1a25904ea6b886924ac8d63b52de88b4c5a6b34fdc020acb");
  let mut rendered=0;let mut refused=0;let mut empty=0;let mut states=BTreeSet::new();let mut captured_artifacts=Vec::new();
  for case in corpus["cases"].as_array().unwrap(){
   let result=&case["result"];assert_eq!(result["authority"],false);assert_eq!(result["complete"],false);
   if result["status"]=="refused"{assert!(result["batch"].is_null());assert_eq!(result["resources"],json!([]));assert!(case["inspectionStatus"].is_null());refused+=1;continue}
   assert_eq!(case["inspectionStatus"],"validated-disclosure");let exact:Value=serde_json::from_str(case["batchJson"].as_str().unwrap()).unwrap();assert_eq!(exact,result["batch"]);
   let original=weft_core::application_lineage::prepare(&case["request"].to_string()).unwrap();let context=weft_core::current_presence::prepare(&original).unwrap();let mut input=binding(&context);
   let mut header=exact.clone();header.as_object_mut().unwrap().remove("rows");input["header"]=header;
   let artifact=lower(&context,&input.to_string()).unwrap();let compiled:Value=serde_json::from_str(artifact.json()).unwrap();assert_eq!(compiled["disclosureHeader"],input["header"]);
   let rows=exact["rows"].as_array().unwrap();let resources=result["resources"].as_array().unwrap();assert_eq!(rows.len(),resources.len());
   let mut expected=Vec::new();let mut returned=Vec::new();
   for (row,resource) in rows.iter().zip(resources){
    // Logical-fixture locator only; real native private keys require host custody.
    let key=resource.to_string();let cells:Vec<_>=row.as_array().unwrap().iter().map(Value::to_string).collect();
    for cell in row.as_array().unwrap(){states.insert(format!("{}:{}",cell["disposition"].as_str().unwrap(),cell.get("value").map_or("none",|v|if v.is_null(){"null"}else{"value"})));}
    expected.push(json!({"key":key,"cells":cells}));returned.push(json!([cells[0],json!({"version":"weft.stage-row-tuple/0.1-candidate","rows":[{"scan":"s0","key":key}]}).to_string()]));
   }
   let mut captured=frame(&context,&input);captured["rows"]=json!(expected);captured["resultRows"]=json!(returned);captured["expectedRowCount"]=json!(rows.len().to_string());
   let report=check(&context,&input.to_string(),&captured.to_string()).unwrap_or_else(|e|panic!("{}: {}",case["id"],e.code));let report:Value=serde_json::from_str(&report).unwrap();assert_eq!(report["authority"],false);assert_eq!(report["matched"],true);
   captured_artifacts.push(json!({"id":case["id"],"binding":input,"compiled":compiled,"frame":captured,"report":report}));if rows.is_empty(){empty+=1}rendered+=1;
  }
  assert_eq!((rendered,refused,empty),(41,25,35));assert_eq!(states.len(),5);
  if let Ok(path)=std::env::var("WEFT_PRESENCE_TEST_CAPTURE"){std::fs::write(path,serde_json::to_vec(&captured_artifacts).unwrap()).unwrap();}
 }
 #[test]
 fn required_absence_and_native_row_budget_boundaries_refuse(){
  let mut req=request();req["sql"]=json!("SELECT r.resourceId FROM Resource r");let original=weft_core::application_lineage::prepare(&req.to_string()).unwrap();let context=weft_core::current_presence::prepare(&original).unwrap();let input=binding(&context);let mut f=frame(&context,&input);
  let error=check(&context,&input.to_string(),&f.to_string()).unwrap_err();assert_eq!(error.code,"WFT-CURRENT-PRESENCE-TRANSPORT");assert_eq!(error.message,"Tagged current projection transport refused");
  let mut rows=Vec::new();let mut result=Vec::new();let cell=json!({"disposition":"original","value":{"string":"r"}}).to_string();
  for i in 0..4096{let key=format!("k{i}");rows.push(json!({"key":key,"cells":[cell]}));result.push(json!([cell,json!({"version":"weft.stage-row-tuple/0.1-candidate","rows":[{"scan":"s0","key":key}]}).to_string()]));}
  f["rows"]=json!(rows);f["resultRows"]=json!(result);f["expectedRowCount"]=json!("4096");assert!(check(&context,&input.to_string(),&f.to_string()).is_ok());
  let mut extra=f["rows"][0].clone();extra["key"]=json!("extra");f["rows"].as_array_mut().unwrap().push(extra);f["expectedRowCount"]=json!("4097");assert!(check(&context,&input.to_string(),&f.to_string()).is_err());
 }
 fn frame(context:&CurrentPresencePreparation<'_>,input:&Value)->Value{
  // Declared unadmitted transport outcomes, not authorized Token disclosures.
  let domain=json!({"documentId":"domain","moduleId":"m","elementId":"token"});
  let cells=vec![json!({"disposition":"original","value":null}),json!({"disposition":"absent"}),json!({"disposition":"withheld"}),json!({"disposition":"transformed","selection":{"ruleIndex":0,"disclosureIndex":0},"transform":{"id":"nullify","version":"v1"},"outputDomain":domain,"value":null}),json!({"disposition":"transformed","selection":{"ruleIndex":1,"disclosureIndex":0},"transform":{"id":"mask","version":"v1"},"outputDomain":domain,"value":{"string":"masked"}})];
  let rows:Vec<_>=cells.iter().enumerate().map(|(i,c)|json!({"key":format!("k{i}"),"cells":[c.to_string()]})).collect();
  let result:Vec<_>=rows.iter().rev().map(|r|json!([r["cells"][0],json!({"version":"weft.stage-row-tuple/0.1-candidate","rows":[{"scan":"s0","key":r["key"]}]}).to_string()])).collect();
  let raw=input.to_string();let artifact=lower(context,&raw).unwrap();json!({"version":"weft.current-presence.transport-input/0.1-candidate","sourceRequestSha256":input["sourceRequestSha256"],"preparationSha256":input["preparationSha256"],"materializationBindingSha256":sha256(raw.as_bytes()),"compiledArtifactSha256":sha256(artifact.json().as_bytes()),"expectedRowCount":"5","header":input["header"],"rows":rows,"resultRows":result})
 }
 #[test]
 fn transport_retains_five_states_and_row_bags_with_qualified_report(){
  let original=weft_core::application_lineage::prepare(&request().to_string()).unwrap();let context=weft_core::current_presence::prepare(&original).unwrap();let input=binding(&context);let frame=frame(&context,&input);let raw=frame.to_string();let report:Value=serde_json::from_str(&check(&context,&input.to_string(),&raw).unwrap()).unwrap();assert_eq!(report["matched"],true);assert_eq!(report["authority"],false);assert_eq!(report["transportInputSha256"],sha256(raw.as_bytes()));
  let mut empty=frame;empty["rows"]=json!([]);empty["resultRows"]=json!([]);empty["expectedRowCount"]=json!("0");assert!(check(&context,&input.to_string(),&empty.to_string()).is_ok());
 }
 #[test]
 fn transport_rejects_weakened_scalar_cells_missing_or_duplicate_rows_and_bad_sources(){
  let original=weft_core::application_lineage::prepare(&request().to_string()).unwrap();let context=weft_core::current_presence::prepare(&original).unwrap();let input=binding(&context);
  for case in 0..10{let mut f=frame(&context,&input);match case{
   0=>f["resultRows"][0][0]=json!("null"),
   1=>f["resultRows"][0]=f["resultRows"][1].clone(),
   2=>{f["resultRows"].as_array_mut().unwrap().pop();},
   3=>{f["rows"][4]["cells"][0]=json!(r#"{"disposition":"original"}"#);f["resultRows"]=json!([]);},
   4=>f["rows"][1]["cells"][0]=json!(r#"{"disposition":"absent","value":null}"#),
   5=>f["header"]["protection"]["revision"]=json!("stale"),
   6=>f["expectedRowCount"]=json!("05"),
   7=>f["compiledArtifactSha256"]=json!("0".repeat(64)),
   8=>f["version"]=json!("weft.current-query.refinement-input/0.2-candidate"),
   _=>{for row in f["rows"].as_array_mut().unwrap(){row["cells"][0]=json!("null");}for row in f["resultRows"].as_array_mut().unwrap(){row[0]=json!("null");}},
  }assert!(check(&context,&input.to_string(),&f.to_string()).is_err(),"{case}");}
 }

}

#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
struct InputRow{key:String,cells:Vec<String>}
#[derive(Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Frame{version:String,source_request_sha256:String,preparation_sha256:String,materialization_binding_sha256:String,compiled_artifact_sha256:String,expected_row_count:String,header:Header,rows:Vec<InputRow>,result_rows:Vec<Vec<String>>}
fn exact(value:&Value,keys:&[&str])->bool{value.as_object().is_some_and(|m|m.len()==keys.len()&&keys.iter().all(|k|m.contains_key(*k)))}
fn cell_shape(raw:&str,absence:bool,domains:&BTreeSet<&str>)->Result<()>{
 let v=checked_json(raw).map_err(|_|fail())?;let kind=v["disposition"].as_str().ok_or_else(fail)?;
 match kind{
  "absent" if absence&&exact(&v,&["disposition"])=>{},"withheld" if exact(&v,&["disposition"])=>{},
  "original" if exact(&v,&["disposition","value"])&&(v["value"].is_null()||v["value"].is_object())=>{},
  "transformed"=>{
   let keys=if v.get("valueProfile").is_some(){vec!["disposition","selection","outputDomain","transform","value","valueProfile"]}else{vec!["disposition","selection","outputDomain","transform","value"]};
   if !exact(&v,&keys)||!exact(&v["selection"],&["ruleIndex","disclosureIndex"])||!exact(&v["transform"],&["id","version"])||!(v["value"].is_null()||v["value"].is_object()){return Err(fail())}
   for k in ["ruleIndex","disclosureIndex"]{if !v["selection"][k].as_u64().is_some_and(|n|n<4096){return Err(fail())}}
   for k in ["id","version"]{if !v["transform"][k].as_str().is_some_and(|s|!s.is_empty()&&s.len()<=4096&&!s.contains('\0')){return Err(fail())}}
   let domain:SecurityRef=serde_json::from_value(v["outputDomain"].clone()).map_err(|_|fail())?;
   if serde_json::to_value(&domain).map_err(|_|fail())?!=v["outputDomain"]||!domains.contains(domain.document_id.as_str())||domain.module_id.is_empty()||domain.element_id.is_empty(){return Err(fail())}
   if v.get("valueProfile").is_some_and(|p|p!="umf-gregorian-date/1.0.0"){return Err(fail())}
  },_=>return Err(fail()),
 }Ok(())
}
/// Checks exact transport and complete row multiplicity, not UMF domain/policy
/// truth. Native hosts must independently validate the actual owner batch/cut.
pub fn check<'a>(context:&'a CurrentPresencePreparation<'a>,binding_json:&str,frame_json:&str)->Result<String>{
 check_transport(context,binding_json,frame_json).map_err(|_|Diagnostic::new("WFT-CURRENT-PRESENCE-TRANSPORT","capability","Tagged current projection transport refused"))
}
fn check_transport<'a>(context:&'a CurrentPresencePreparation<'a>,binding_json:&str,frame_json:&str)->Result<String>{
 let artifact=lower(context,binding_json)?;
 if frame_json.len()>16_000_000{return Err(fail())}
 let raw=checked_json(frame_json).map_err(|_|fail())?;let frame:Frame=serde_json::from_value(raw.clone()).map_err(|_|fail())?;
 if serde_json::to_value(&frame).map_err(|_|fail())?!=raw||frame.version!="weft.current-presence.transport-input/0.1-candidate"||frame.rows.len()>4096||frame.result_rows.len()>4096
  ||frame.source_request_sha256!=sha256(context.preparation().source_request_json().as_bytes())||frame.preparation_sha256!=sha256(context.json().as_bytes())||frame.materialization_binding_sha256!=sha256(binding_json.as_bytes())||frame.compiled_artifact_sha256!=sha256(artifact.json().as_bytes())||frame.expected_row_count!=frame.rows.len().to_string(){return Err(fail())}
 let output=checked_json(artifact.json()).map_err(|_|fail())?;
 if serde_json::to_value(&frame.header).map_err(|_|fail())?!=output["disclosureHeader"]{return Err(fail())}
 let columns=output["resultColumns"].as_array().ok_or_else(fail)?;let mut absent=vec![false;frame.header.columns.len()];let mut indexes=Vec::new();
 for column in columns{
  let index=column["headerColumn"].as_u64().and_then(|n|usize::try_from(n).ok()).filter(|i|*i<absent.len()).ok_or_else(fail)?;
  absent[index]=column["type"]["alternatives"].as_array().ok_or_else(fail)?.iter().any(|v|v=="absent");indexes.push(index);
 }
 let domains:BTreeSet<_>=frame.header.domains.iter().map(|d|d.document_id.as_str()).collect();let mut expected=BTreeMap::new();let mut cell_count=0usize;
 for row in &frame.rows{
  if row.key.is_empty()||row.key.len()>256||row.key.contains('\0')||row.cells.len()!=frame.header.columns.len()||expected.insert(row.key.as_str(),&row.cells).is_some(){return Err(fail())}
  cell_count=cell_count.checked_add(row.cells.len()).filter(|n|*n<=16384).ok_or_else(fail)?;
  for (cell,allowed) in row.cells.iter().zip(&absent){cell_shape(cell,*allowed,&domains)?;}
 }
 if frame.result_rows.len()!=expected.len(){return Err(fail())}
 let scan=&context.preparation().plan().source.occurrence;let mut seen=BTreeSet::new();
 for row in &frame.result_rows{
  if row.len()!=columns.len()+1{return Err(fail())}
  let lineage=checked_json(row.last().ok_or_else(fail)?).map_err(|_|fail())?;
  if !exact(&lineage,&["version","rows"])||lineage["version"]!="weft.stage-row-tuple/0.1-candidate"{return Err(fail())}
  let tuple=lineage["rows"].as_array().filter(|a|a.len()==1).ok_or_else(fail)?;
  if !exact(&tuple[0],&["scan","key"])||tuple[0]["scan"]!=*scan{return Err(fail())}
  let key=tuple[0]["key"].as_str().ok_or_else(fail)?;let cells=expected.get(key).ok_or_else(fail)?;
  if !seen.insert(key.to_owned()){return Err(fail())}
  for (position,index) in indexes.iter().enumerate(){if row[position]!=cells[*index]{return Err(fail())}}
 }
 encode(&json!({"version":"weft.current-presence.transport-result/0.1-candidate","matched":true,"authority":false,"securityReady":false,"nativeAdmission":"required","sourceRequestSha256":frame.source_request_sha256,"preparationSha256":frame.preparation_sha256,"materializationBindingSha256":frame.materialization_binding_sha256,"compiledArtifactSha256":frame.compiled_artifact_sha256,"transportInputSha256":sha256(frame_json.as_bytes()),"unresolved":["actual complete authenticated native population/header/owner batch","whole UMF cell/domain/selection/value validation","closing authority checks and guarded release"]}))
}
