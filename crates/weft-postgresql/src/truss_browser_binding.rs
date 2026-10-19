// SPDX-License-Identifier: MIT OR Apache-2.0
//! Browser-local catalog correspondence, never native binding or execution authority.
use serde::Deserialize;
use std::collections::BTreeSet;
use weft_core::{application_model::Shape,ir::{Identity,ModelPin,LogicalType,Family},model::Catalog,mutation::{Artifact,MResult,MutationDiagnostic}};
fn fail()->MutationDiagnostic{MutationDiagnostic{code:"WFT-MUTATION-BROWSER-BINDING".into(),phase:"binding".into(),path:String::new(),message:"Exact complete browser catalog mapping required".into()}}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct BrowserMapping {
 interface_version:String,pglite_version:String,postgresql_version:String,store:Store,catalog_revision:String,document:ModelPin,
 pub(crate) maximum_scan_rows:u32,record:Record,
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Store{profile:String,origin:String,namespace:String,store_id:String,incarnation:String,layout_sha256:String}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Record{identity:Identity,type_id:String,fields:Vec<Field>,key:Key}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Field{identity:Identity,property_id:String}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Key{key_id:String,property_ids:Vec<String>}
fn native(value:&str)->bool{value.parse::<i32>().is_ok_and(|n|n>0&&n.to_string()==value)}
fn uuid(value:&str)->bool{value.len()==36&&value.bytes().enumerate().all(|(i,c)|if [8,13,18,23].contains(&i){c==b'-'}else{c.is_ascii_digit()||(b'a'..=b'f').contains(&c)})}
impl BrowserMapping {
 pub(crate) fn admit(artifact:&Artifact,model:&Artifact,pin:&ModelPin,catalog:&Catalog)->MResult<Self>{
  let value=artifact.read()?;
  for identity in std::iter::once(&value["record"]["identity"]).chain(value["record"]["fields"].as_array().into_iter().flatten().map(|f|&f["identity"])) {
   if identity.as_object().is_none_or(|o|o.len()!=4||!["documentId","revision","module","element"].iter().all(|k|o.contains_key(*k))){return Err(fail());}
  }
  let mapping:Self=serde_json::from_value(value).map_err(|_|fail())?;
  let s=&mapping.store;
  if mapping.interface_version!="truss-browser-binding/0.1"||mapping.pglite_version!="0.5.10"||mapping.postgresql_version!="18.6"||mapping.document!=*pin||pin.sha256!=model.sha256
   ||!native(&mapping.catalog_revision)||!(1..=10000).contains(&mapping.maximum_scan_rows)
   ||!matches!((s.profile.as_str(),s.layout_sha256.as_str()),
    ("truss-browser-layout/0.1","c203d09418fb4eddda829d791dda11c20341227c2d1d82e86d4c24afda5e7294")
    |("truss-browser-layout/0.2","1ffa72c51518e809c773e7f9207c1de1598f331c15de61e02dda0abe0fc475e0"))
   ||s.origin.is_empty()||s.origin.len()>4096||!s.namespace.starts_with("truss-browser:")||s.namespace.len()>78||s.namespace.len()<=14||!s.namespace[14..].bytes().enumerate().all(|(i,c)|c.is_ascii_lowercase()||c.is_ascii_digit()||(i>0&&c==b'-'))
   ||!uuid(&s.store_id)||!uuid(&s.incarnation)||!native(&mapping.record.type_id)
   ||mapping.record.fields.is_empty()||mapping.record.fields.len()>64{return Err(fail());}
  let record=catalog.record_by_identity(&mapping.record.identity).map_err(|_|fail())?;
  let entity=catalog.entity_descriptor(&record).map_err(|_|fail())?;
  if entity.members.len()!=mapping.record.fields.len(){return Err(fail());}
  let mut properties=BTreeSet::new();let mut identities=BTreeSet::new();
  for field in &mapping.record.fields{
   if !native(&field.property_id)||!properties.insert(&field.property_id)||!identities.insert(serde_json::to_string(&field.identity).map_err(|_|fail())?)
    ||entity.members.get(properties.len()-1).is_none_or(|m|m.identity!=field.identity){return Err(fail());}
   let descriptor=entity.graph.iter().find(|d|d.identity==field.identity).ok_or_else(fail)?;
   if descriptor.availability.as_deref()!=Some("required")||!matches!(&descriptor.shape,Shape::Scalar{logical_type} if logical_type.family==Family::String&&!logical_type.nullable){return Err(fail());}
  }
  let key=catalog.authored_key(&record,&mapping.record.key.key_id).map_err(|_|fail())?;
  let keys=record.value["keys"].as_array().ok_or_else(fail)?;
  if keys.len()!=1||keys[0]["primary"]!=true||key.fields.len()!=1{return Err(fail());}
  let expected=key.fields.iter().map(|identity|mapping.record.fields.iter().find(|f|f.identity==*identity).map(|f|f.property_id.clone()).ok_or_else(fail)).collect::<MResult<Vec<_>>>()?;
  if expected!=mapping.record.key.property_ids{return Err(fail());}
  Ok(mapping)
 }
 pub(crate) fn type_id(&self,record:&Identity)->MResult<String>{if *record!=self.record.identity{return Err(fail());}Ok(self.record.type_id.clone())}
 pub(crate) fn scalar(&self,record:&Identity,field:&Identity,logical:&LogicalType)->MResult<(String,String,String)>{
  self.type_id(record)?;
  if logical.family!=Family::String||logical.nullable{return Err(fail());}
  let property=&self.record.fields.iter().find(|f|f.identity==*field).ok_or_else(fail)?.property_id;
  let cell=format!("\"weft_mutation\".props->'{property}'");
  Ok((format!("({cell}->>'text') COLLATE pg_catalog.\"C\""),format!("({cell})->>'text'"),format!("truss.valid_scalar_cell(({cell}),'string',FALSE) IS TRUE")))
 }
}

#[cfg(test)]
mod layout_compatibility_tests {
 use super::*;
 use serde_json::{json,Value};
 use weft_core::{json::sha256,model::ModuleInput};
 fn fixture()->(Artifact,ModelPin,Catalog,Value){
  let source:Value=serde_json::from_str(include_str!("../../../tests/mutation/fixtures/local-products-input.json")).unwrap();
  let model:Artifact=serde_json::from_value(source["request"]["model"].clone()).unwrap();
  let pin:ModelPin=serde_json::from_value(source["request"]["modelPin"].clone()).unwrap();
  let document:Value=serde_json::from_str(&model.utf8).unwrap();
  let catalog=Catalog::prepare_security(vec![ModuleInput{document_json:model.utf8.clone(),pin:pin.clone(),selected_module_ids:document["modules"].as_array().unwrap().iter().map(|m|m["id"].as_str().unwrap().to_owned()).collect()}]).unwrap();
  let identity=|element:&str|json!({"documentId":pin.document_id,"revision":pin.revision,"module":"sales","element":element});
  let binding=json!({"interfaceVersion":"truss-browser-binding/0.1","pgliteVersion":"0.5.10","postgresqlVersion":"18.6",
   "store":{"profile":"truss-browser-layout/0.2","origin":"https://layout.example","namespace":"truss-browser:layout-compat","storeId":"00000000-0000-4000-8000-000000000001","incarnation":"00000000-0000-4000-8000-000000000002","layoutSha256":"1ffa72c51518e809c773e7f9207c1de1598f331c15de61e02dda0abe0fc475e0"},
   "catalogRevision":"1","document":pin,"maximumScanRows":1000,"record":{"identity":identity("products"),"typeId":"1","fields":[{"identity":identity("product-id"),"propertyId":"1"},{"identity":identity("product-name"),"propertyId":"2"}],"key":{"keyId":"pk","propertyIds":["1"]}}});
  (model,pin,catalog,binding)
 }
 fn artifact(value:&Value)->Artifact{let utf8=value.to_string();Artifact{media_type:"application/json".into(),sha256:sha256(utf8.as_bytes()),utf8}}
 #[test]
 fn only_complete_known_layout_pairs_admit(){
  let (model,pin,catalog,original)=fixture();
  let old="c203d09418fb4eddda829d791dda11c20341227c2d1d82e86d4c24afda5e7294";
  let new="1ffa72c51518e809c773e7f9207c1de1598f331c15de61e02dda0abe0fc475e0";
  for (profile,layout,expected) in [("truss-browser-layout/0.1",old,true),("truss-browser-layout/0.2",new,true),("truss-browser-layout/0.1",new,false),("truss-browser-layout/0.2",old,false),("truss-browser-layout/0.3",new,false),("truss-browser-layout/0.2","unknown",false)]{
   let mut value=original.clone();value["store"]["profile"]=json!(profile);value["store"]["layoutSha256"]=json!(layout);
   assert_eq!(BrowserMapping::admit(&artifact(&value),&model,&pin,&catalog).is_ok(),expected,"{profile}/{layout}");
  }
 }
 #[test]
 fn new_pair_preserves_other_mapping_checks(){
  let (model,pin,catalog,original)=fixture();
  for (path,replacement) in [("/pgliteVersion",json!("0.5.11")),("/store/origin",json!("")),("/maximumScanRows",json!(10001)),("/record/typeId",json!("01")),("/record/key/propertyIds",json!(["2"]))]{
   let mut value=original.clone();*value.pointer_mut(path).unwrap()=replacement;
   assert!(BrowserMapping::admit(&artifact(&value),&model,&pin,&catalog).is_err(),"{path}");
  }
 }
}
