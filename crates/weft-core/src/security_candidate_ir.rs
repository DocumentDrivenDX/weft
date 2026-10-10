//! Separate draft typed IR. Not an admitted 0.1 plan or native execution permit.
use crate::{error::{Diagnostic,Result},model::Catalog,security_source::SecurityCandidateSourcePacket,security_candidate_policy_types::SecurityCandidatePolicyTypes,security_candidate_ontology::SecurityCandidateOntologyClosure,security_ontology::{SecurityRef,locate,domain},security_association_ref::SecurityAssociationRef,security_ir::{Term,Binding,Effect,Disposition}};
use serde_json::Value;
use std::collections::BTreeMap;
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-CANDIDATE-IR","model","Draft logical plan refused")}
#[derive(Debug,Clone,PartialEq,serde::Serialize)]
#[serde(rename_all="camelCase",rename_all_fields="camelCase")]
pub enum CandidateWitness{RecordKey{owner:SecurityRef,key_id:String,key:Value},OpaqueExistential}
#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord,serde::Serialize)]
#[serde(rename_all="lowercase")]
pub enum CandidateSide{Source,Target}
#[derive(Debug,Clone,PartialEq,serde::Serialize)]
#[serde(rename_all="camelCase",rename_all_fields="camelCase")]
pub enum CandidateEndpointCarrier{Members{fields:Vec<SecurityRef>},Incidence{side:CandidateSide}}
#[derive(Debug,Clone,PartialEq,serde::Serialize)]
#[serde(rename_all="camelCase",rename_all_fields="camelCase")]
pub enum CandidateTerm{Value(Term),Endpoint{binding:Binding,association:SecurityAssociationRef,role:String,target:SecurityRef,key_id:String,carrier:CandidateEndpointCarrier}}
#[derive(Debug,Clone,PartialEq,serde::Serialize)]
#[serde(rename_all="camelCase",rename_all_fields="camelCase")]
pub enum CandidateExpression{Literal(bool),Equal(CandidateTerm,CandidateTerm),And(Vec<CandidateExpression>),Or(Vec<CandidateExpression>),Not(Box<CandidateExpression>),Exists{slot:usize,association:SecurityAssociationRef,witness:CandidateWitness,condition:Box<CandidateExpression>}}
#[derive(Debug,Clone,PartialEq,serde::Serialize)]
#[serde(rename_all="camelCase")]
pub struct CandidateRule{pub id:String,pub effect:Effect,pub actions:Vec<String>,pub target:SecurityRef,pub condition:CandidateExpression,pub disclosure:Vec<(SecurityRef,Disposition)>}
fn witness(s:&Value,c:&SecurityCandidateOntologyClosure,catalog:&Catalog)->Result<CandidateWitness>{
 if s["kind"]=="core-relationship"&&s["witness"]["kind"]=="opaque-existential"{return Ok(CandidateWitness::OpaqueExistential);}
 let owner=SecurityRef::read(if s["kind"]=="record-members"{&s["type"]}else{&s["witness"]["type"]})?;let key_id=c.types.get(&owner).ok_or_else(fail)?.source["keyId"].as_str().ok_or_else(fail)?.to_string();let key=locate(catalog,&owner)?["keys"].as_array().ok_or_else(fail)?.iter().find(|k|k["id"]==key_id).ok_or_else(fail)?.clone();Ok(CandidateWitness::RecordKey{owner,key_id,key})
}
#[derive(Debug)]
pub struct SecurityCandidateLogicalPlan{rules:Vec<CandidateRule>,packet:SecurityCandidateSourcePacket}
impl SecurityCandidateLogicalPlan{
 pub fn rules(&self)->&[CandidateRule]{&self.rules}
 pub fn source(&self)->&SecurityCandidateSourcePacket{&self.packet}
 pub fn require_catalog(&self,catalog:&Catalog)->Result<()>{self.packet.require_catalog(catalog)}
 pub fn read(packet:SecurityCandidateSourcePacket,catalog:&Catalog)->Result<Self>{
  SecurityCandidatePolicyTypes::read(&packet,catalog)?;
  let closure=SecurityCandidateOntologyClosure::read(&packet,catalog)?;
  fn term(v:&Value,resource:&SecurityRef,variables:&BTreeMap<String,(usize,SecurityAssociationRef)>,closure:&SecurityCandidateOntologyClosure,catalog:&Catalog)->Result<CandidateTerm>{
   let variable=if v["kind"]=="variable"{Some(variables.get(v["name"].as_str().ok_or_else(fail)?).ok_or_else(fail)?)}else{None};
   if let Some(role)=v.get("endpoint"){
    let (slot,a)=variable.ok_or_else(fail)?;let selector=closure.associations.get(a).ok_or_else(fail)?;let endpoint=selector["endpoints"].as_array().ok_or_else(fail)?.iter().find(|e|e["role"]==*role).ok_or_else(fail)?;
    let target=SecurityRef::read(&endpoint["target"])?;let key_id=closure.types[&target].source["keyId"].as_str().ok_or_else(fail)?.into();
    let carrier=if selector["kind"]=="record-members"{CandidateEndpointCarrier::Members{fields:endpoint["fields"].as_array().ok_or_else(fail)?.iter().map(SecurityRef::read).collect::<Result<_>>()?}}else{CandidateEndpointCarrier::Incidence{side:match endpoint["side"].as_str(){Some("source")=>CandidateSide::Source,Some("target")=>CandidateSide::Target,_=>return Err(fail())}}};
    return Ok(CandidateTerm::Endpoint{binding:Binding::Variable(*slot),association:a.clone(),role:role.as_str().ok_or_else(fail)?.into(),target,key_id,carrier});
   }
   let (binding,target)=match v["kind"].as_str(){Some("subject")=>(Some(Binding::Subject),Some(closure.subject.clone())),Some("resource")=>(Some(Binding::Resource),Some(resource.clone())),Some("variable")=>{let (slot,a)=variable.ok_or_else(fail)?;let selector=closure.associations.get(a).ok_or_else(fail)?;let owner=match witness(selector,closure,catalog)?{CandidateWitness::RecordKey{owner,..}=>owner,CandidateWitness::OpaqueExistential=>return Err(fail())};(Some(Binding::Variable(*slot)),Some(owner))},Some("context"|"constant")=>(None,None),_=>return Err(fail())};
   if v.get("identity").is_some(){let target=target.ok_or_else(fail)?;let key_id=closure.types[&target].source["keyId"].as_str().ok_or_else(fail)?.into();return Ok(CandidateTerm::Value(Term::Identity{binding:binding.ok_or_else(fail)?,target,key_id}));}
   let field=SecurityRef::read(&v["field"])?;let shape=domain(locate(catalog,&field)?)?;
   Ok(CandidateTerm::Value(match v["kind"].as_str(){Some("constant")=>Term::Constant{field,domain:shape,literal:v["value"].clone()},Some("context")=>Term::Context{field,domain:shape},_=>Term::Field{binding:binding.ok_or_else(fail)?,field,domain:shape}}))
  }

  fn expression(v:&Value,resource:&SecurityRef,variables:&BTreeMap<String,(usize,SecurityAssociationRef)>,closure:&SecurityCandidateOntologyClosure,catalog:&Catalog,next_slot:&mut usize)->Result<CandidateExpression>{
   Ok(match v["op"].as_str(){
    Some("literal")=>CandidateExpression::Literal(v["value"].as_bool().ok_or_else(fail)?),
    Some("eq")=>CandidateExpression::Equal(term(&v["left"],resource,variables,closure,catalog)?,term(&v["right"],resource,variables,closure,catalog)?),
    Some("and"|"or")=>{let args=v["args"].as_array().ok_or_else(fail)?.iter().map(|a|expression(a,resource,variables,closure,catalog,next_slot)).collect::<Result<Vec<_>>>()?;if v["op"]=="and"{CandidateExpression::And(args)}else{CandidateExpression::Or(args)}},
    Some("not")=>CandidateExpression::Not(Box::new(expression(&v["arg"],resource,variables,closure,catalog,next_slot)?)),
    Some("exists")=>{let slot=*next_slot;*next_slot+=1;let association=SecurityAssociationRef::read(&v["association"])?;let descriptor=witness(closure.associations.get(&association).ok_or_else(fail)?,closure,catalog)?;let mut nested=variables.clone();nested.insert(v["as"].as_str().ok_or_else(fail)?.into(),(slot,association.clone()));CandidateExpression::Exists{slot,association,witness:descriptor,condition:Box::new(expression(&v["where"],resource,&nested,closure,catalog,next_slot)?)}},
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
    rules.push(CandidateRule{id:r["id"].as_str().ok_or_else(fail)?.into(),effect:match r["effect"].as_str(){Some("permit")=>Effect::Permit,Some("require")=>Effect::Require,Some("forbid")=>Effect::Forbid,_=>return Err(fail())},actions:r["actions"].as_array().ok_or_else(fail)?.iter().map(|a|a.as_str().map(String::from).ok_or_else(fail)).collect::<Result<_>>()?,target,condition,disclosure});
   }
  }
  Ok(Self{rules,packet})
 }
}
