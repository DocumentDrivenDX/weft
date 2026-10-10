//! Exact ontology reference/key/classification closure. Does not authorize native execution.
use crate::{error::{Diagnostic,Result},model::Catalog,security_source::SecuritySourcePacket};
use serde_json::{json,Value};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord,serde::Serialize,serde::Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct SecurityRef{pub document_id:String,pub module_id:String,pub element_id:String}
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-ONTOLOGY","model","Security ontology closure refused")}
fn members(v:&Value,known:&[&str])->Result<()>{let object=v.as_object().ok_or_else(fail)?;if object.keys().any(|k|!known.contains(&k.as_str())){return Err(fail());}Ok(())}
impl SecurityRef{
 pub(crate) fn read(v:&Value)->Result<Self>{members(v,&["documentId","moduleId","elementId"])?;Ok(Self{document_id:v["documentId"].as_str().ok_or_else(fail)?.into(),module_id:v["moduleId"].as_str().ok_or_else(fail)?.into(),element_id:v["elementId"].as_str().ok_or_else(fail)?.into()})}
 fn local(document:&str,v:&Value)->Result<Self>{members(v,&["module","element"])?;Ok(Self{document_id:document.into(),module_id:v["module"].as_str().ok_or_else(fail)?.into(),element_id:v["element"].as_str().ok_or_else(fail)?.into()})}
}
#[derive(Debug,Clone)]
pub(crate) struct TypeBinding{pub source:Value,pub fields:BTreeSet<SecurityRef>,pub keys:Vec<SecurityRef>}
#[derive(Debug)]
pub struct SecurityOntologyClosure{pub(crate) types:BTreeMap<SecurityRef,TypeBinding>,pub(crate) fields:BTreeMap<SecurityRef,Value>,pub(crate) subject:SecurityRef,pub(crate) context:BTreeSet<SecurityRef>}
pub(crate) fn locate<'a>(catalog:&'a Catalog,r:&SecurityRef)->Result<&'a Value>{
 let doc=catalog.documents.iter().find(|d|d["id"]==r.document_id).ok_or_else(fail)?;
 doc["modules"].as_array().ok_or_else(fail)?.iter().find(|m|m["id"]==r.module_id).ok_or_else(fail)?["elements"].as_array().ok_or_else(fail)?.iter().find(|e|e["id"]==r.element_id).ok_or_else(fail)
}
pub(crate) fn domain(field:&Value)->Result<Value>{
 members(field,&["id","kind","name","description","extensions","scalarType","nullability","cardinality","facets","references","title","aliases","examples","allowedValues","default"])?;
 if field["kind"]!="field"||field["cardinality"]!="one"||!matches!(field["scalarType"].as_str(),Some("boolean"|"string"|"integer"|"decimal"|"binary"))||field["references"].as_array().is_some_and(|r|!r.is_empty()){return Err(fail());}
 let facets=crate::security_literals::normalized_facets(field)?;
 members(&facets,&["length","precision","scale","integerWidth","range","collectionSize"])?;
 for (group,keys) in [("length",&["min","max","unit"][..]),("integerWidth",&["bits","signed"][..]),("range",&["min","max","minInclusive","maxInclusive"][..]),("collectionSize",&["min","max"][..])]{if let Some(value)=facets.get(group){members(value,keys)?;}}
 if !matches!(field["nullability"].as_str(),Some("required"|"absent-allowed"|"unspecified")){return Err(fail());}
 let kind=field["scalarType"].as_str().unwrap();
 if facets.get("collectionSize").is_some()||((facets.get("precision").is_some()||facets.get("scale").is_some())&&kind!="decimal")||(facets.get("integerWidth").is_some()&&kind!="integer")||(facets.get("range").is_some()&&!matches!(kind,"integer"|"decimal")){return Err(fail());}
 if let Some(length)=facets.get("length"){
  if length["unit"]!=if kind=="string"{"unicode-scalar"}else if kind=="binary"{"byte"}else{return Err(fail());}{return Err(fail());}
 }
 if let Some(width)=facets.get("integerWidth"){
  let bits=width["bits"].as_u64().ok_or_else(fail)?;if bits==0||bits>9_007_199_254_740_991||!width["signed"].is_boolean(){return Err(fail());}
 }
 if field["scalarType"]=="decimal"{
  let precision=facets["precision"].as_u64().ok_or_else(fail)?;let scale=facets["scale"].as_u64().ok_or_else(fail)?;
  if precision==0||precision>9_007_199_254_740_991||scale>precision{return Err(fail());}
 }
 Ok(json!({"scalarType":field["scalarType"],"cardinality":field["cardinality"],"nullability":field["nullability"],"facets":facets,"allowedValues":field.get("allowedValues").unwrap_or(&Value::Null)}))
}
pub(crate) fn close_entities(entities:&[Value],catalog:&Catalog)->Result<(BTreeMap<SecurityRef,TypeBinding>,BTreeMap<SecurityRef,Value>)>{
 let mut types=BTreeMap::new();let mut fields=BTreeMap::new();
  for entity in entities{
   let reference=SecurityRef::read(&entity["type"])?;let record=locate(catalog,&reference)?;if record["kind"]!="record"{return Err(fail());}
   let mut declared=BTreeSet::new();for f in entity["fields"].as_array().ok_or_else(fail)?{
    let r=SecurityRef::read(&f["ref"])?;if !declared.insert(r.clone()){return Err(fail());}domain(locate(catalog,&r)?)?;crate::security_literals::validate_field(locate(catalog,&r)?)?;fields.insert(r.clone(),locate(catalog,&r)?.clone());
   }
   let mut record_members=BTreeSet::new();for member in record["members"].as_array().ok_or_else(fail)?{if !record_members.insert(SecurityRef::local(&reference.document_id,member)?){return Err(fail());}}
   if record_members!=declared{return Err(fail());}
   let definitions=record["keys"].as_array().ok_or_else(fail)?;let matching:Vec<_>=definitions.iter().filter(|k|k["id"]==entity["keyId"]).collect();if matching.len()!=1{return Err(fail());}let key=matching[0];members(key,&["id","name","fields","primary"])?;
   if definitions.iter().any(|k|k.get("primary").is_some_and(|p|!p.is_boolean()))||definitions.iter().filter(|k|k["primary"]==true).count()>1{return Err(fail());}
   let mut keys=Vec::new();let mut unique=BTreeSet::new();for f in key["fields"].as_array().ok_or_else(fail)?{
    let r=SecurityRef::local(&reference.document_id,f)?;
    if !declared.contains(&r)||!unique.insert(r.clone())||locate(catalog,&r)?["nullability"]!="required"{return Err(fail());}keys.push(r);
   }
   if keys.is_empty()||types.insert(reference,TypeBinding{source:entity.clone(),fields:declared,keys}).is_some(){return Err(fail());}
  }
 Ok((types,fields))
}
impl SecurityOntologyClosure{
 pub fn subject(&self)->&SecurityRef{&self.subject}
 pub fn type_count(&self)->usize{self.types.len()}
 pub fn field_count(&self)->usize{self.fields.len()}
 pub fn read(packet:&SecuritySourcePacket,catalog:&Catalog)->Result<Self>{
  packet.require_catalog(catalog)?;
  let ontology=packet.ontology();
  let entities:Vec<Value>=ontology["entities"].as_array().ok_or_else(fail)?.iter().chain(ontology["associations"].as_array().ok_or_else(fail)?.iter()).cloned().collect();
  let (types,mut fields)=close_entities(&entities,catalog)?;
  let subject=SecurityRef::read(&ontology["subject"])?;
  if !ontology["entities"].as_array().unwrap().iter().any(|e|SecurityRef::read(&e["type"]).ok().as_ref()==Some(&subject)){return Err(fail());}
  let mut context=BTreeSet::new();for field in ontology["context"].as_array().ok_or_else(fail)?{let r=SecurityRef::read(field)?;if !context.insert(r.clone()){return Err(fail());}domain(locate(catalog,&r)?)?;crate::security_literals::validate_field(locate(catalog,&r)?)?;fields.insert(r.clone(),locate(catalog,&r)?.clone());}
  for association in ontology["associations"].as_array().unwrap(){
   let a=types.get(&SecurityRef::read(&association["type"])?).ok_or_else(fail)?;let mut roles=BTreeSet::new();
   for endpoint in association["endpoints"].as_array().ok_or_else(fail)?{
    if !roles.insert(endpoint["role"].as_str().ok_or_else(fail)?){return Err(fail());}
    let target=types.get(&SecurityRef::read(&endpoint["target"])?).ok_or_else(fail)?;
    let values=endpoint["fields"].as_array().ok_or_else(fail)?;if values.len()!=target.keys.len(){return Err(fail());}let mut unique=BTreeSet::new();
    for (value,key) in values.iter().zip(&target.keys){let r=SecurityRef::read(value)?;if !a.fields.contains(&r)||!unique.insert(r.clone())||domain(locate(catalog,&r)?)?!=domain(locate(catalog,key)?)?{return Err(fail());}}
   }
  }
  Ok(Self{types,fields,subject,context})
 }
}
