//! Immutable, type-admitted security logical IR. No fact trust, authorization or physical lowering.
use crate::{error::{Diagnostic,Result},model::Catalog,security_source::SecuritySourcePacket,security_policy_types::SecurityPolicyTypes,security_ontology::{SecurityRef,SecurityOntologyClosure,locate,domain}};
use serde_json::Value;
use crate::security_association_ref::SecurityAssociationRef;
use std::collections::BTreeMap;
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-IR","model","Security logical IR admission refused")}
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,serde::Serialize,serde::Deserialize)]
#[serde(rename_all="lowercase")]
pub enum QueryOperator{Predicate,Order,Group,Join,Aggregate}
impl QueryOperator{pub(crate) fn name(&self)->&'static str{match self{Self::Predicate=>"predicate",Self::Order=>"order",Self::Group=>"group",Self::Join=>"join",Self::Aggregate=>"aggregate"}}}
#[derive(Debug,Clone,PartialEq,Eq,serde::Serialize)]
#[serde(rename_all="camelCase")]
pub enum Effect{Permit,Require,Forbid}
#[derive(Debug,Clone,PartialEq,Eq,serde::Serialize)]
#[serde(rename_all="camelCase")]
pub enum Binding{Subject,Resource,Variable(usize)}
#[derive(Debug,Clone,PartialEq,serde::Serialize)]
#[serde(rename_all="camelCase",rename_all_fields="camelCase")]
pub enum Term{
 Identity{binding:Binding,target:SecurityRef,key_id:String},
 Endpoint{binding:Binding,association:SecurityAssociationRef,role:String,target:SecurityRef,key_id:String},
 Field{binding:Binding,field:SecurityRef,domain:Value},
 Context{field:SecurityRef,domain:Value},
 Constant{field:SecurityRef,domain:Value,literal:Value},
}
#[derive(Debug,Clone,PartialEq,serde::Serialize)]
#[serde(rename_all="camelCase",rename_all_fields="camelCase")]
pub enum Expression{
 Literal(bool),Equal(Term,Term),And(Vec<Expression>),Or(Vec<Expression>),Not(Box<Expression>),
 Exists{slot:usize,association:SecurityAssociationRef,condition:Box<Expression>},
}
#[derive(Debug,Clone,PartialEq,serde::Serialize)]
#[serde(rename_all="camelCase",rename_all_fields="camelCase")]
pub enum Disposition{Original,Withheld,Transformed{transform:String,version:String,output_field:SecurityRef,domain:Value,literal:Value}}
#[derive(Debug,Clone,PartialEq,serde::Serialize)]
#[serde(rename_all="camelCase")]
pub struct Rule{pub id:String,pub effect:Effect,pub actions:Vec<String>,pub target:SecurityRef,pub condition:Expression,pub disclosure:Vec<(SecurityRef,Disposition)>}
/// Rules can only be obtained through admission against the exact source/catalog snapshot.
/// Returned references are immutable; caller-constructed enums are not admitted plans.
#[derive(Debug)]
pub struct SecurityLogicalPlan{rules:Vec<Rule>,packet:SecuritySourcePacket}
impl SecurityLogicalPlan{
 pub fn rules(&self)->&[Rule]{&self.rules}
 pub fn source(&self)->&SecuritySourcePacket{&self.packet}
 pub fn require_catalog(&self,catalog:&Catalog)->Result<()>{self.packet.require_catalog(catalog)}
 pub fn read(packet:SecuritySourcePacket,catalog:&Catalog)->Result<Self>{
  SecurityPolicyTypes::read(&packet,catalog)?;
  let closure=SecurityOntologyClosure::read(&packet,catalog)?;
  fn term(v:&Value,resource:&SecurityRef,variables:&BTreeMap<String,(usize,SecurityRef)>,closure:&SecurityOntologyClosure,catalog:&Catalog)->Result<Term>{
   let (binding,target)=match v["kind"].as_str(){
    Some("subject")=>(Some(Binding::Subject),Some(closure.subject.clone())),
    Some("resource")=>(Some(Binding::Resource),Some(resource.clone())),
    Some("variable")=>{let (slot,r)=variables.get(v["name"].as_str().ok_or_else(fail)?).ok_or_else(fail)?;(Some(Binding::Variable(*slot)),Some(r.clone()))},
    Some("context"|"constant")=>(None,None),_=>return Err(fail())};
   if v.get("identity").is_some(){let target=target.ok_or_else(fail)?;let key_id=closure.types[&target].source["keyId"].as_str().ok_or_else(fail)?.into();return Ok(Term::Identity{binding:binding.ok_or_else(fail)?,target,key_id});}
   if let Some(role)=v.get("endpoint"){
    let association=target.ok_or_else(fail)?;let endpoint=closure.types[&association].source["endpoints"].as_array().ok_or_else(fail)?.iter().find(|e|e["role"]==*role).ok_or_else(fail)?;
    let target=SecurityRef::read(&endpoint["target"])?;let key_id=closure.types[&target].source["keyId"].as_str().ok_or_else(fail)?.into();
    return Ok(Term::Endpoint{binding:binding.ok_or_else(fail)?,association:SecurityAssociationRef::Record(association),role:role.as_str().ok_or_else(fail)?.into(),target,key_id});
   }
   let field=SecurityRef::read(&v["field"])?;let shape=domain(locate(catalog,&field)?)?;
   Ok(match v["kind"].as_str(){Some("constant")=>Term::Constant{field,domain:shape,literal:v["value"].clone()},Some("context")=>Term::Context{field,domain:shape},_=>Term::Field{binding:binding.ok_or_else(fail)?,field,domain:shape}})
  }
  fn expression(v:&Value,resource:&SecurityRef,variables:&BTreeMap<String,(usize,SecurityRef)>,closure:&SecurityOntologyClosure,catalog:&Catalog,next_slot:&mut usize)->Result<Expression>{
   Ok(match v["op"].as_str(){
    Some("literal")=>Expression::Literal(v["value"].as_bool().ok_or_else(fail)?),
    Some("eq")=>Expression::Equal(term(&v["left"],resource,variables,closure,catalog)?,term(&v["right"],resource,variables,closure,catalog)?),
    Some("and"|"or")=>{let args=v["args"].as_array().ok_or_else(fail)?.iter().map(|a|expression(a,resource,variables,closure,catalog,next_slot)).collect::<Result<Vec<_>>>()?;if v["op"]=="and"{Expression::And(args)}else{Expression::Or(args)}},
    Some("not")=>Expression::Not(Box::new(expression(&v["arg"],resource,variables,closure,catalog,next_slot)?)),
    Some("exists")=>{let slot=*next_slot;*next_slot+=1;let association=SecurityAssociationRef::read(&v["association"])?;let record=association.record().ok_or_else(fail)?.clone();let mut nested=variables.clone();nested.insert(v["as"].as_str().ok_or_else(fail)?.into(),(slot,record));Expression::Exists{slot,association,condition:Box::new(expression(&v["where"],resource,&nested,closure,catalog,next_slot)?)}},
    _=>return Err(fail())})
  }
  let mut rules=Vec::new();
  for r in packet.policy()["rules"].as_array().ok_or_else(fail)?{
   for target in r["target"].as_array().ok_or_else(fail)?{
    let target=SecurityRef::read(target)?;let mut next_slot=0;
    let condition=expression(&r["condition"],&target,&BTreeMap::new(),&closure,catalog,&mut next_slot)?;
    let mut disclosure=Vec::new();if let Some(ds)=r["disclosure"].as_array(){for d in ds{
     let field=SecurityRef::read(&d["field"])?;let disp=&d["disposition"];
     let disposition=match disp["kind"].as_str(){Some("original")=>Disposition::Original,Some("withheld")=>Disposition::Withheld,Some("transformed")=>{let output_field=SecurityRef::read(&disp["field"])?;Disposition::Transformed{transform:disp["transform"].as_str().ok_or_else(fail)?.into(),version:disp["version"].as_str().ok_or_else(fail)?.into(),domain:domain(locate(catalog,&output_field)?)?,output_field,literal:disp["value"].clone()}},_=>return Err(fail())};disclosure.push((field,disposition));
    }}
    rules.push(Rule{id:r["id"].as_str().ok_or_else(fail)?.into(),effect:match r["effect"].as_str(){Some("permit")=>Effect::Permit,Some("require")=>Effect::Require,Some("forbid")=>Effect::Forbid,_=>return Err(fail())},actions:r["actions"].as_array().ok_or_else(fail)?.iter().map(|a|a.as_str().map(String::from).ok_or_else(fail)).collect::<Result<_>>()?,target,condition,disclosure});
   }
  }
  Ok(Self{rules,packet})
 }
}

#[cfg(test)]
mod tests{
 use super::*;
 #[test]
 fn existential_ir_preserves_record_and_relationship_reference_shapes(){
  for field in ["elementId","relationshipId"]{
   let mut reference=serde_json::json!({"documentId":"d","moduleId":"m"});reference[field]=serde_json::json!("a");
   let association=SecurityAssociationRef::read(&reference).unwrap();
   assert_eq!(association.record().is_some(),field=="elementId");
   let expr=Expression::Exists{slot:7,association:association.clone(),condition:Box::new(Expression::Literal(true))};
   assert_eq!(serde_json::to_value(expr).unwrap()["exists"]["association"],reference);
   let term=Term::Endpoint{binding:Binding::Variable(7),association,role:"owner".into(),target:SecurityRef{document_id:"d".into(),module_id:"m".into(),element_id:"Target".into()},key_id:"selected".into()};
   let encoded=serde_json::to_value(term).unwrap();assert_eq!(encoded["endpoint"]["association"],reference);assert_eq!(encoded["endpoint"]["keyId"],"selected");
  }
 }

 #[test]
 fn plan_reuse_refuses_mutated_prepared_definitions(){
  let fixture:Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();
  let source=&fixture["resolution"]["documents"][0];let text=source["document"].to_string();
  let inputs=serde_json::from_value(serde_json::json!([{"documentJson":text,"pin":{"documentId":source["document"]["id"],"revision":source["revision"],"umfVersion":"0.8.0","sha256":crate::json::sha256(text.as_bytes())},"selectedModuleIds":["m"]}])).unwrap();
  let mut catalog=Catalog::prepare_security(inputs).unwrap();let packet=SecuritySourcePacket::read(&fixture["policy"].to_string(),&fixture["resolution"]["ontology"].to_string(),&catalog).unwrap();let plan=SecurityLogicalPlan::read(packet,&catalog).unwrap();
  catalog.documents[0]["modules"][0]["elements"][0]["name"]=serde_json::json!("changed");assert!(plan.require_catalog(&catalog).is_err());
 }
}
