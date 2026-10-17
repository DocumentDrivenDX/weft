//! Original ColumnSet/binding custody shared by explicit Databricks profiles.
//! Core admits the UMF model first. These facts grant no native permission or
//! cross-profile qualification; each caller owns physical type and runtime gates.
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use weft_core::{raw_read,search};
type Result<T>=std::result::Result<T,&'static str>;
pub(crate) type Columns=std::collections::BTreeMap<String,(String,String,bool)>;
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub(crate) struct BindingDocument{pub(crate) source_id:String,pub(crate) document_json:String,pub(crate) document_sha256:String}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub(crate) struct NativeColumns{pub(crate) column_set:search::Source,pub(crate) binding:BindingDocument}
pub(crate) fn field_key(v:&Value)->String{weft_core::mutation::canonical(v)}
fn exact_keys(v:&Value,required:&[&str],optional:&[&str])->bool{
 v.as_object().is_some_and(|m|required.iter().all(|k|m.contains_key(*k))&&m.keys().all(|k|required.contains(&k.as_str())||optional.contains(&k.as_str())))
}
pub(crate) fn native_type(raw:&str)->Result<String>{
 if raw.len()>100||!raw.is_ascii(){return Err("bounded native type spelling required")}
 let chars=raw.chars().collect::<Vec<_>>();
 for (i,c) in chars.iter().enumerate(){if c.is_ascii_whitespace(){
   let before=chars[..i].iter().rev().find(|c|!c.is_ascii_whitespace());let after=chars[i+1..].iter().find(|c|!c.is_ascii_whitespace());
   if before.is_some_and(|c|c.is_ascii_alphanumeric()||*c=='_')&&after.is_some_and(|c|c.is_ascii_alphanumeric()||*c=='_'){return Err("native type tokens cannot be joined across whitespace")}
 }}
 Ok(chars.into_iter().filter(|c|!c.is_ascii_whitespace()).collect::<String>().to_ascii_lowercase())
}
pub(crate) fn admit_columns(model_sha256:&str,doc:&Value,target:&Value,custody:&NativeColumns,table:&raw_read::Table,record:&Value)->Result<Columns>{
 let source=&custody.column_set;
 if doc["vocabularies"]["umf.access"]["version"]!="0.1.0"||source.document_id!=doc["id"]||source.core_version!="0.8.0"||source.document_sha256!=model_sha256{return Err("ColumnSet original model source differs")}
 let mut sets=vec![];
 for (mi,module) in doc["modules"].as_array().unwrap().iter().enumerate(){
   for (ci,set) in module["extensions"]["umf.access"]["columns"].as_array().into_iter().flatten().enumerate(){
     if source.pointer==format!("/modules/{mi}/extensions/umf.access/columns/{ci}")&&source.module==module["id"]{sets.push(set)}
   }
 }
 if sets.len()!=1{return Err("unique source-pinned original ColumnSet required")}
 let set=sets[0];let b=&custody.binding;
 if !exact_keys(set,&["binding","target","columns"],&[])||!exact_keys(&set["binding"],&["sourceId","module"],&[])||b.source_id.is_empty(){return Err("closed original ColumnSet required")}
 let binding=weft_core::json::checked_json(&b.document_json).map_err(|_|"original binding JSON refused")?;
 if weft_core::json::sha256(b.document_json.as_bytes())!=b.document_sha256||binding["umf"]!="0.8.0"||binding["vocabularies"]["umf.binding"]["version"]!="0.2.0"||binding["id"].as_str().is_none_or(str::is_empty)||set["binding"]!=json!({"sourceId":b.source_id,"module":record["module"]})||set["target"]!=*target{return Err("ColumnSet binding/source/target differs")}
 let payload=&binding["extensions"]["umf.binding"];
 if !exact_keys(payload,&["profile","logical","target","elements","fields","relationships","indexes"],&[])||!exact_keys(&payload["logical"],&["documentId","coreVersion"],&[])||!exact_keys(&payload["target"],&["system","version"],&["subset"]){return Err("closed original binding profile required")}
 if payload["profile"]!="umf-binding-2"||payload["logical"]!=json!({"documentId":doc["id"],"coreVersion":"0.8.0"})||payload["target"]["system"]!=target["platform"]||payload["target"]["version"]!=target["majorVersion"]||payload["target"]["subset"]!="typed-access-columns/0.1"{return Err("original binding logical/native target differs")}
 let elements=payload["elements"].as_array().ok_or("binding element inventory absent")?.iter().filter(|v|v["module"]==record["module"]&&v["element"]==record["element"]).collect::<Vec<_>>();
 if elements.len()!=1||!exact_keys(elements[0],&["module","element","table"],&[])||elements[0]["table"]!=table.native{return Err("original native Record table differs")}
 let record_body=doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==record["module"]).and_then(|m|m["elements"].as_array().unwrap().iter().find(|e|e["id"]==record["element"])).ok_or("original ColumnSet Record absent")?;
 let mut columns=std::collections::BTreeMap::new();let mut physical=std::collections::BTreeSet::new();
 for column in set["columns"].as_array().ok_or("ColumnSet columns absent")? {
   if !exact_keys(column,&["field","type","notNull","generation"],&[])||!exact_keys(&column["field"],&["module","element"],&[]){return Err("closed original NativeColumn required")}
   let field=&column["field"];if !record_body["members"].as_array().unwrap().contains(field){continue}
   let fields=payload["fields"].as_array().ok_or("binding field inventory absent")?.iter().filter(|v|v["module"]==field["module"]&&v["element"]==field["element"]).collect::<Vec<_>>();
   if fields.len()!=1||!exact_keys(fields[0],&["module","element","storage","column"],&[])||fields[0]["storage"]!="column"{return Err("original physical Field column absent/ambiguous")}
   let native=fields[0]["column"].as_str().ok_or("original native column name absent")?;if native.is_empty()||native.len()>255||native.contains('\0'){return Err("bounded native identifier required")};
   let ty=native_type(column["type"].as_str().ok_or("original native column type absent")?)?;
   let not_null=column["notNull"].as_bool().ok_or("original ColumnSet notNull required")?;
   if !(column["generation"].is_null()||matches!(column["generation"].as_str(),Some("identity"|"stored-expression"|"default")))||column.get("generation").is_none(){return Err("original native generation unsupported")}
   if !physical.insert(native.to_string())||columns.insert(field_key(field),(native.to_string(),ty,not_null)).is_some(){return Err("ambiguous original ColumnSet column inventory")}
 }
 Ok(columns)
}
pub(crate) fn row_null(doc:&Value,record:&Value,reference:&Value,original_columns:&Columns)->Result<bool>{
 let views=doc["modules"].as_array().unwrap().iter().flat_map(|m|m["extensions"]["umf.access"]["views"].as_array().into_iter().flatten()).filter(|v|v["record"]==*record).collect::<Vec<_>>();
 if views.len()!=1{return Err("unique original current Record view required")}

   let matches=views[0]["fields"].as_array().ok_or("original current view fields absent")?.iter().filter(|f|f["field"]==*reference).collect::<Vec<_>>();
   if matches.len()!=1{return Err("unique original current Field view required")}
   let nullable=matches[0]["row"]["null"].as_bool().ok_or("original current row null policy required")?;
   let owner=doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==reference["module"]).ok_or("row view module absent")?;
   let f=owner["elements"].as_array().unwrap().iter().find(|f|f["id"]==reference["element"]).ok_or("row view Field absent")?;
   if matches[0]["row"]["missing"]!=false||!matches!(f["nullability"].as_str(),Some("required"|"absent-allowed"))||(nullable&&f["nullability"]!="absent-allowed"){return Err("current row view contradicts core/relational presence")}
   let (_,_,not_null)=original_columns.get(&field_key(reference)).ok_or("current Field absent from original ColumnSet")?;
   if *not_null==nullable{return Err("original ColumnSet notNull contradicts row view")}
   for module in doc["modules"].as_array().unwrap(){for set in module["extensions"]["umf.access"]["columns"].as_array().into_iter().flatten(){for column in set["columns"].as_array().into_iter().flatten(){if column["field"]==*reference&&column["notNull"]!=!nullable{return Err("another authored ColumnSet contradicts original row view")}}}}
   Ok(nullable)
}
