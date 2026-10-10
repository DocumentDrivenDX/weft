//! Declared policy term types/scope. Exact literal/facet value validity and lowering remain separate.
use crate::{error::{Diagnostic,Result},model::Catalog,security_source::SecuritySourcePacket,security_ontology::{SecurityOntologyClosure,SecurityRef,domain,locate}};
use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-TYPE","model","Security policy type admission refused")}
#[derive(Debug,PartialEq)]
enum TermType{Identity(SecurityRef,String),Scalar(Value)}
/// A successful result records declared term checking only, never authorization.
#[derive(Debug)]
pub struct SecurityPolicyTypes{checked_rules:usize,checked_nodes:usize}
impl SecurityPolicyTypes{
 pub fn checked_rules(&self)->usize{self.checked_rules}
 pub fn checked_nodes(&self)->usize{self.checked_nodes}
 pub fn read(packet:&SecuritySourcePacket,catalog:&Catalog)->Result<Self>{
  let admitted=SecurityOntologyClosure::read(packet,catalog)?;let closure=&admitted;
  fn literal_shape(field:&Value,value:&Value)->Result<()>{
   crate::security_literals::validate_field(field)?;
   crate::security_literals::check_literal(field,value).map_err(|_|fail())
  }
  fn term(value:&Value,resource:&SecurityRef,variables:&BTreeMap<String,SecurityRef>,closure:&SecurityOntologyClosure,catalog:&Catalog)->Result<TermType>{
   let binding=match value["kind"].as_str(){Some("subject")=>Some(&closure.subject),Some("resource")=>Some(resource),Some("variable")=>Some(variables.get(value["name"].as_str().ok_or_else(fail)?).ok_or_else(fail)?),Some("context"|"constant")=>None,_=>return Err(fail())};
   if value.get("identity").is_some(){let r=binding.ok_or_else(fail)?;let b=closure.types.get(r).ok_or_else(fail)?;return Ok(TermType::Identity(r.clone(),b.source["keyId"].as_str().unwrap().into()));}
   if let Some(role)=value.get("endpoint"){
    let b=closure.types.get(binding.ok_or_else(fail)?).ok_or_else(fail)?;
    let endpoint=b.source["endpoints"].as_array().ok_or_else(fail)?.iter().find(|e|e["role"]==*role).ok_or_else(fail)?;
    let target=SecurityRef::read(&endpoint["target"])?;let b=closure.types.get(&target).ok_or_else(fail)?;
    return Ok(TermType::Identity(target,b.source["keyId"].as_str().unwrap().into()));
   }
   let r=SecurityRef::read(&value["field"])?;
   if value["kind"]=="context"{if !closure.context.contains(&r){return Err(fail());}}
   else if value["kind"]!="constant"{if !closure.types.get(binding.ok_or_else(fail)?).ok_or_else(fail)?.fields.contains(&r){return Err(fail());}}
   let field=locate(catalog,&r)?;let shape=domain(field)?;
   if value["kind"]=="constant"{literal_shape(field,&value["value"])?;}
   Ok(TermType::Scalar(shape))
  }
  fn expression(expr:&Value,resource:&SecurityRef,variables:&BTreeMap<String,SecurityRef>,closure:&SecurityOntologyClosure,catalog:&Catalog,steps:&mut usize)->Result<()>{
   *steps+=1;if *steps>1_000_000{return Err(fail());}
   match expr["op"].as_str(){
    Some("literal")=>{},
    Some("eq")=>{if term(&expr["left"],resource,variables,closure,catalog)?!=term(&expr["right"],resource,variables,closure,catalog)?{return Err(fail());}},
    Some("and"|"or")=>{for e in expr["args"].as_array().unwrap(){expression(e,resource,variables,closure,catalog,steps)?;}},
    Some("not")=>expression(&expr["arg"],resource,variables,closure,catalog,steps)?,
    Some("exists")=>{
     let association=SecurityRef::read(&expr["association"])?;let b=closure.types.get(&association).ok_or_else(fail)?;
     if !b.source["endpoints"].is_array(){return Err(fail());}
     let name=expr["as"].as_str().ok_or_else(fail)?;if variables.contains_key(name){return Err(fail());}
     let mut nested=variables.clone();nested.insert(name.into(),association);expression(&expr["where"],resource,&nested,closure,catalog,steps)?;
    },_=>return Err(fail())
   };Ok(())
  }
  let ontology=packet.ontology();let mut actions=BTreeSet::new();for a in ontology["actions"].as_array().unwrap(){if !actions.insert(a.as_str().unwrap()){return Err(fail());}}
  let mut nodes=0;let rules=packet.policy()["rules"].as_array().unwrap();
  for rule in rules{
   let mut unique=BTreeSet::new();for a in rule["actions"].as_array().unwrap(){let name=a.as_str().unwrap();if !actions.contains(name)||!unique.insert(name){return Err(fail());}}
   let mut targets=BTreeSet::new();for target in rule["target"].as_array().unwrap(){
    let r=SecurityRef::read(target)?;let binding=closure.types.get(&r).ok_or_else(fail)?;if !targets.insert(r.clone()){return Err(fail());}
    expression(&rule["condition"],&r,&BTreeMap::new(),closure,catalog,&mut nodes)?;
    let mut fields=BTreeSet::new();if let Some(disclosures)=rule["disclosure"].as_array(){for disclosure in disclosures{
     let output=SecurityRef::read(&disclosure["field"])?;if !binding.fields.contains(&output)||!fields.insert(output){return Err(fail());}
     let disposition=&disclosure["disposition"];if disposition["kind"]=="transformed"{let r=SecurityRef::read(&disposition["field"])?;let field=locate(catalog,&r)?;domain(field)?;literal_shape(field,&disposition["value"])?;}
    }}
   }
  }
  Ok(Self{checked_rules:rules.len(),checked_nodes:nodes})
 }
}
