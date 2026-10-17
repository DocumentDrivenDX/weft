//! Original ColumnSet/binding custody shared by explicit target profiles.
//! Core admits the UMF model first. These facts grant no native permission or
//! cross-profile qualification; each caller owns physical type and runtime gates.
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use crate::{raw_read,search};
type Result<T>=std::result::Result<T,&'static str>;
pub type Columns=std::collections::BTreeMap<String,(String,String,bool)>;
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct BindingDocument{pub source_id:String,pub document_json:String,pub document_sha256:String}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct NativeColumns{pub column_set:search::Source,pub binding:BindingDocument}
/// Closed original-source envelope; each target independently admits its profile.
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct NativeColumnsEnvelope{pub target:Value,pub sets:Vec<NativeColumns>,#[serde(default,skip_serializing_if="Option::is_none")]pub vector_extension:Option<VectorExtension>}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct VectorExtension{pub schema:String,pub version:String}
pub fn field_key(v:&Value)->String{crate::mutation::canonical(v)}
fn exact_keys(v:&Value,required:&[&str],optional:&[&str])->bool{
 v.as_object().is_some_and(|m|required.iter().all(|k|m.contains_key(*k))&&m.keys().all(|k|required.contains(&k.as_str())||optional.contains(&k.as_str())))
}
pub fn native_type(raw:&str)->Result<String>{
 if raw.len()>100||!raw.is_ascii(){return Err("bounded native type spelling required")}
 let chars=raw.chars().collect::<Vec<_>>();
 for (i,c) in chars.iter().enumerate(){if c.is_ascii_whitespace(){
   let before=chars[..i].iter().rev().find(|c|!c.is_ascii_whitespace());let after=chars[i+1..].iter().find(|c|!c.is_ascii_whitespace());
   if before.is_some_and(|c|c.is_ascii_alphanumeric()||*c=='_')&&after.is_some_and(|c|c.is_ascii_alphanumeric()||*c=='_'){return Err("native type tokens cannot be joined across whitespace")}
 }}
 Ok(chars.into_iter().filter(|c|!c.is_ascii_whitespace()).collect::<String>().to_ascii_lowercase())
}
/// Resolve exact source custody before a target selects Record-relevant sets.
pub fn original_column_set<'a>(model_sha256:&str,doc:&'a Value,target:&Value,custody:&NativeColumns)->Result<&'a Value>{
 let source=&custody.column_set;
 if doc["vocabularies"]["umf.access"]["version"]!="0.1.0"||source.document_id!=doc["id"]||source.core_version!="0.8.0"||source.document_sha256!=model_sha256{return Err("ColumnSet original model source differs")}
 let mut sets=vec![];
 for (mi,module) in doc["modules"].as_array().unwrap().iter().enumerate(){
   for (ci,set) in module["extensions"]["umf.access"]["columns"].as_array().into_iter().flatten().enumerate(){
     if source.pointer==format!("/modules/{mi}/extensions/umf.access/columns/{ci}")&&source.module==module["id"]{sets.push(set)}
   }
 }
 if sets.len()!=1{return Err("unique source-pinned original ColumnSet required")}
 let set=sets[0];
 if set["target"]!=*target||!exact_keys(set,&["binding","target","columns"],&[])||!exact_keys(&set["binding"],&["sourceId","module"],&[])||set["columns"].as_array().is_none()||custody.binding.source_id.is_empty()||set["binding"]["sourceId"]!=custody.binding.source_id||set["binding"]["module"].as_str().is_none_or(str::is_empty){return Err("original ColumnSet source/target differs")}
 crate::json::checked_json(&custody.binding.document_json).map_err(|_|"original binding JSON refused")?;
 if crate::json::sha256(custody.binding.document_json.as_bytes())!=custody.binding.document_sha256{return Err("original binding source pin differs")}
 Ok(set)
}
pub fn admit_columns(model_sha256:&str,doc:&Value,target:&Value,custody:&NativeColumns,table:&raw_read::Table,record:&Value)->Result<Columns>{
 admit_columns_inner(model_sha256,doc,target,custody,table,record,false)
}
/// Admit explicitly authored collation facts for the source-exact PG profile.
pub fn admit_columns_with_collation(model_sha256:&str,doc:&Value,target:&Value,custody:&NativeColumns,table:&raw_read::Table,record:&Value)->Result<Columns>{
 admit_columns_inner(model_sha256,doc,target,custody,table,record,true)
}
fn admit_columns_inner(model_sha256:&str,doc:&Value,target:&Value,custody:&NativeColumns,table:&raw_read::Table,record:&Value,with_collation:bool)->Result<Columns>{
 let set=original_column_set(model_sha256,doc,target,custody)?;let b=&custody.binding;
 if !exact_keys(set,&["binding","target","columns"],&[])||!exact_keys(&set["binding"],&["sourceId","module"],&[])||b.source_id.is_empty(){return Err("closed original ColumnSet required")}
 let binding=crate::json::checked_json(&b.document_json).map_err(|_|"original binding JSON refused")?;
 if crate::json::sha256(b.document_json.as_bytes())!=b.document_sha256||binding["umf"]!="0.8.0"||binding["vocabularies"]["umf.binding"]["version"]!="0.2.0"||binding["id"].as_str().is_none_or(str::is_empty)||set["binding"]!=json!({"sourceId":b.source_id,"module":record["module"]})||set["target"]!=*target{return Err("ColumnSet binding/source/target differs")}
 let payload=&binding["extensions"]["umf.binding"];
 if !exact_keys(payload,&["profile","logical","target","elements","fields","relationships","indexes"],&[])||!exact_keys(&payload["logical"],&["documentId","coreVersion"],&[])||!exact_keys(&payload["target"],&["system","version"],&["subset"]){return Err("closed original binding profile required")}
 if payload["profile"]!="umf-binding-2"||payload["logical"]!=json!({"documentId":doc["id"],"coreVersion":"0.8.0"})||payload["target"]["system"]!=target["platform"]||payload["target"]["version"]!=target["majorVersion"]||payload["target"]["subset"]!="typed-access-columns/0.1"{return Err("original binding logical/native target differs")}
 let elements=payload["elements"].as_array().ok_or("binding element inventory absent")?.iter().filter(|v|v["module"]==record["module"]&&v["element"]==record["element"]).collect::<Vec<_>>();
 if elements.len()!=1||!exact_keys(elements[0],&["module","element","table"],&[])||elements[0]["table"]!=table.native{return Err("original native Record table differs")}
 let record_body=doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==record["module"]).and_then(|m|m["elements"].as_array().unwrap().iter().find(|e|e["id"]==record["element"])).ok_or("original ColumnSet Record absent")?;
 let mut columns=std::collections::BTreeMap::new();let mut physical=std::collections::BTreeSet::new();
 for column in set["columns"].as_array().ok_or("ColumnSet columns absent")? {
   if !exact_keys(column,&["field","type","notNull","generation"],if with_collation {&["collation"]}else{&[]})||!exact_keys(&column["field"],&["module","element"],&[]){return Err("closed original NativeColumn required")}
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
pub fn row_null(doc:&Value,record:&Value,reference:&Value,original_columns:&Columns)->Result<bool>{
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

/// Required original qualified collation identity; no native default is inferred.
pub fn collation(doc:&Value,custody:&NativeColumns,field:&Value)->Result<Value>{
 let set=doc.pointer(&custody.column_set.pointer).ok_or("original ColumnSet pointer absent")?;
 let values=set["columns"].as_array().ok_or("original ColumnSet columns absent")?.iter().filter(|c|c["field"]==*field).collect::<Vec<_>>();
 if values.len()!=1{return Err("original collation Field absent/ambiguous")}
 let value=values[0].get("collation").ok_or("source-exact original collation fact required")?;
 if !value.is_null() && (!exact_keys(value,&["schema","name"],&[])||["schema","name"].iter().any(|key|value[*key].as_str().is_none_or(|v|v.is_empty()||v.len()>63||v.contains('\0')))){return Err("closed qualified original collation required")}
 Ok(value.clone())
}

/// Source generation retained independently of target type admission.
pub fn generation(doc:&Value,custody:&NativeColumns,field:&Value)->Result<Value>{
 let set=doc.pointer(&custody.column_set.pointer).ok_or("original ColumnSet pointer absent")?;
 let values=set["columns"].as_array().ok_or("original ColumnSet columns absent")?.iter().filter(|c|c["field"]==*field).collect::<Vec<_>>();
 if values.len()!=1{return Err("original generation Field absent/ambiguous")}
 let value=values[0].get("generation").ok_or("original generation absent")?;
 if !(value.is_null()||matches!(value.as_str(),Some("identity"|"stored-expression"|"default"))){return Err("original generation unsupported")}
 Ok(value.clone())
}

/// Validate the original Vector declaration without granting search or native
/// execution authority. The caller still owns source admission and physical
/// storage/decoder qualification. The original document is only borrowed.
pub fn vector_declaration<'a>(doc:&'a Value,record:&Value,field:&Value)->Result<&'a Value>{
 fn locate<'a>(doc:&'a Value,r:&Value)->Result<&'a Value>{
  if !exact_keys(r,&["module","element"],&[]){return Err("exact original Field/Record reference required")}
  doc["modules"].as_array().ok_or("original modules absent")?.iter().find(|m|m["id"]==r["module"])
   .and_then(|m|m["elements"].as_array()).and_then(|e|e.iter().find(|e|e["id"]==r["element"]))
   .ok_or("original Vector declaration reference absent")
 }
 let body=locate(doc,record)?;let members=body["members"].as_array().ok_or("original Record members absent")?;
 if body["kind"]!="record"||!members.contains(field){return Err("Vector Field outside original Record")}
 let indexes=doc["modules"].as_array().ok_or("original modules absent")?.iter()
  .flat_map(|m|m["extensions"]["umf.access"]["search"].as_array().into_iter().flatten())
  .filter(|i|i["kind"]=="vector"&&i["record"]==*record&&i["vector"]["field"]==*field).collect::<Vec<_>>();
 if indexes.len()!=1{return Err("unique original Vector index declaration required")}
 let index=indexes[0];let vector=&index["vector"];
 if !exact_keys(index,&["id","record","kind","exactness","filterable","vector"],&[])
  ||index["id"].as_str().is_none_or(str::is_empty)||!matches!(index["exactness"].as_str(),Some("exact"|"approximate"))
  ||!exact_keys(vector,&["field","model","dimensions","metric","computedBy"],&["source"])
  ||!exact_keys(&vector["model"],&["id","version"],&[])
  ||vector["model"]["id"].as_str().is_none_or(str::is_empty)||vector["model"]["version"].as_str().is_none_or(str::is_empty)
  ||!matches!(vector["metric"].as_str(),Some("cosine"|"dot"|"l2"))
  ||!matches!(vector["computedBy"].as_str(),Some("host"|"target")) {return Err("original Vector index declaration malformed")}
 let filterable=index["filterable"].as_array().ok_or("original filterable Field inventory absent")?;let mut seen=std::collections::BTreeSet::new();
 for reference in filterable {if !members.contains(reference)||locate(doc,reference)?["kind"]!="field"||!seen.insert(field_key(reference)){return Err("original filterable Field outside Record or repeated")}}
 if vector["computedBy"]=="target"&&vector.get("source").is_none(){return Err("target-computed Vector requires original source text")}
 if let Some(source)=vector.get("source") {let text=locate(doc,source)?;
  if !members.contains(source)||text["kind"]!="field"||text["cardinality"]!="one"||text["scalarType"]!="string" {return Err("original Vector source must be one text Field of its Record")}
 }
 let dimensions=vector["dimensions"].as_u64().filter(|n|*n>0).ok_or("positive original Vector dimensions required")?;
 let embedding=locate(doc,field)?;let leaf=locate(doc,&embedding["itemType"])?;
 if embedding["kind"]!="field"||embedding["cardinality"]!="array"||embedding["facets"]!=json!({"collectionSize":{"min":dimensions,"max":dimensions}})
  ||leaf["kind"]!="field"||leaf["scalarType"]!="float"||leaf["cardinality"]!="one"||leaf["nullability"]!="required"
  ||leaf.get("allowedValues").is_some()||leaf["extensions"].as_object().is_none_or(|m|!m.is_empty())
  ||leaf.get("facets").is_some_and(|f|f.as_object().is_none_or(|m|!m.is_empty())) {return Err("original fixed float32 Vector Field declaration differs")}
 Ok(vector)
}

#[cfg(test)]
mod vector_declaration_tests {
 use super::*;
 fn admitted(mut change:impl FnMut(&mut Value))->raw_read::AdmittedRead{
  let mut r:Value=serde_json::from_str(include_str!("../../../tests/native-columns/vector-request.json")).unwrap();
  let mut doc:Value=serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();change(&mut doc);
  let source=doc.to_string();r["binding"]["modelSha256"]=json!(crate::json::sha256(source.as_bytes()));r["binding"]["modelJson"]=json!(source);
  raw_read::admit_json(&r.to_string()).unwrap()
 }
 #[test]
 fn valid_original_declaration_is_borrowed_without_changing_extension_content(){
  let a=admitted(|_|{});let before=a.original_document().clone();let record=json!({"module":"commerce","element":"Order"});let field=json!({"module":"commerce","element":"embedding"});
  let vector=vector_declaration(a.original_document(),&record,&field).unwrap();assert_eq!(vector["dimensions"],3);assert_eq!(vector["model"],json!({"id":"spark-fixture","version":"1"}));
  assert_eq!(a.original_document(),&before);
  let mut unknown=before.clone();unknown["modules"][0]["extensions"]["vendor.future"]=json!({"opaque":[1,null,{"content":"retained"}]});let exact=unknown.clone();vector_declaration(&unknown,&record,&field).unwrap();assert_eq!(unknown,exact);
 }
 #[test]
 fn malformed_selected_vector_meaning_refuses_without_source_mutation(){
  for fault in 0..7{let a=admitted(|doc|{let v=&mut doc["modules"][0]["extensions"]["umf.access"]["search"][0]["vector"];match fault{
   0=>v["metric"]=json!("unknown"),1=>{v.as_object_mut().unwrap().remove("model");},2=>v["model"]["version"]=json!(""),
   3=>v["computedBy"]=json!("unknown"),4=>{v["computedBy"]=json!("target");v.as_object_mut().unwrap().remove("source");},
   5=>{v["computedBy"]=json!("target");v["source"]=json!({"module":"commerce","element":"big"});},
   _=>v["model"]["unknownMeaning"]=json!(true),
  }});let before=a.original_document().clone();assert!(vector_declaration(a.original_document(),&json!({"module":"commerce","element":"Order"}),&json!({"module":"commerce","element":"embedding"})).is_err(),"fault {fault}");assert_eq!(a.original_document(),&before);}
 }
}
