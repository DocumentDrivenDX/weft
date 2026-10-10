//! Draft selected policy term checking, separately typed from admitted 0.1 plans.
use crate::{error::{Diagnostic,Result},model::Catalog,security_source::SecurityCandidateSourcePacket,security_candidate_ontology::SecurityCandidateOntologyClosure,security_association_ref::SecurityAssociationRef,security_ontology::{SecurityRef,domain,locate}};
use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-CANDIDATE-TYPE","model","Draft policy term checking refused")}
#[derive(Debug,PartialEq)]
enum TermType{Identity(SecurityRef,String),Scalar(Value)}
#[derive(Debug)]
pub struct SecurityCandidatePolicyTypes{checked_rules:usize,checked_nodes:usize}
impl SecurityCandidatePolicyTypes{
 pub fn checked_rules(&self)->usize{self.checked_rules}
 pub fn checked_nodes(&self)->usize{self.checked_nodes}
 pub fn read(packet:&SecurityCandidateSourcePacket,catalog:&Catalog)->Result<Self>{
  let closure=SecurityCandidateOntologyClosure::read(packet,catalog)?;
  let contexts:BTreeSet<_>=packet.ontology()["context"].as_array().ok_or_else(fail)?.iter().map(SecurityRef::read).collect::<Result<_>>()?;
  fn identity(r:&SecurityRef,c:&SecurityCandidateOntologyClosure)->Result<TermType>{let b=c.types.get(r).ok_or_else(fail)?;Ok(TermType::Identity(r.clone(),b.source["keyId"].as_str().ok_or_else(fail)?.into()))}
  fn witness<'a>(selector:&'a Value)->Result<SecurityRef>{if selector["kind"]=="record-members"{SecurityRef::read(&selector["type"])}else if selector["witness"]["kind"]=="record-key"{SecurityRef::read(&selector["witness"]["type"])}else{Err(fail())}}
  fn term(v:&Value,resource:&SecurityRef,vars:&BTreeMap<String,SecurityAssociationRef>,c:&SecurityCandidateOntologyClosure,contexts:&BTreeSet<SecurityRef>,catalog:&Catalog)->Result<TermType>{
   let variable=if v["kind"]=="variable"{let a=vars.get(v["name"].as_str().ok_or_else(fail)?).ok_or_else(fail)?;Some(c.associations.get(a).ok_or_else(fail)?)}else{None};
   if let Some(role)=v.get("endpoint"){
    let selector=variable.ok_or_else(fail)?;let endpoint=selector["endpoints"].as_array().ok_or_else(fail)?.iter().find(|e|e["role"]==*role).ok_or_else(fail)?;return identity(&SecurityRef::read(&endpoint["target"] )?,c);
   }
   let binding=match v["kind"].as_str(){Some("subject")=>Some(c.subject.clone()),Some("resource")=>Some(resource.clone()),Some("variable")=>Some(witness(variable.ok_or_else(fail)?)?),Some("context"|"constant")=>None,_=>return Err(fail())};
   if v.get("identity").is_some(){return identity(&binding.ok_or_else(fail)?,c);}
   let field=SecurityRef::read(&v["field"])?;if !c.types.values().any(|b|b.fields.contains(&field)){return Err(fail());}
   if v["kind"]=="context"{if !contexts.contains(&field){return Err(fail());}}
   else if v["kind"]!="constant"{if !c.types.get(&binding.ok_or_else(fail)?).ok_or_else(fail)?.fields.contains(&field){return Err(fail());}}
   let source=locate(catalog,&field)?;if source["nullability"]!="required"{return Err(fail());}if source.get("extensions").is_some_and(|e|e.as_object().is_none_or(|o|!o.is_empty())){return Err(fail());}
   crate::security_literals::validate_field(source)?;let d=domain(source)?;
   if v["kind"]=="constant"{crate::security_literals::check_literal(source,&v["value"]).map_err(|_|fail())?;}
   Ok(TermType::Scalar(d))
  }
  fn expression(e:&Value,resource:&SecurityRef,vars:&BTreeMap<String,SecurityAssociationRef>,c:&SecurityCandidateOntologyClosure,contexts:&BTreeSet<SecurityRef>,catalog:&Catalog,nodes:&mut usize)->Result<()>{
   *nodes+=1;if *nodes>1_000_000{return Err(fail());}
   match e["op"].as_str(){Some("literal")=>{},Some("eq")=>{if term(&e["left"],resource,vars,c,contexts,catalog)?!=term(&e["right"],resource,vars,c,contexts,catalog)?{return Err(fail());}},
    Some("and"|"or")=>{for x in e["args"].as_array().ok_or_else(fail)?{expression(x,resource,vars,c,contexts,catalog,nodes)?;}},Some("not")=>expression(&e["arg"],resource,vars,c,contexts,catalog,nodes)?,
    Some("exists")=>{let a=SecurityAssociationRef::read(&e["association"])?;if !c.associations.contains_key(&a){return Err(fail());}let name=e["as"].as_str().ok_or_else(fail)?;if vars.contains_key(name){return Err(fail());}let mut nested=vars.clone();nested.insert(name.into(),a);expression(&e["where"],resource,&nested,c,contexts,catalog,nodes)?;},_=>return Err(fail())}
   Ok(())
  }
  let actions:BTreeSet<_>=packet.ontology()["actions"].as_array().ok_or_else(fail)?.iter().map(|v|v.as_str().ok_or_else(fail)).collect::<Result<_>>()?;
  let rules=packet.policy()["rules"].as_array().ok_or_else(fail)?;let mut nodes=0;
  for rule in rules{
   let mut used=BTreeSet::new();for a in rule["actions"].as_array().ok_or_else(fail)?{let a=a.as_str().ok_or_else(fail)?;if !actions.contains(a)||!used.insert(a){return Err(fail());}}
   let mut targets=BTreeSet::new();for t in rule["target"].as_array().ok_or_else(fail)?{let r=SecurityRef::read(t)?;let b=closure.types.get(&r).ok_or_else(fail)?;if !targets.insert(r.clone()){return Err(fail());}expression(&rule["condition"],&r,&BTreeMap::new(),&closure,&contexts,catalog,&mut nodes)?;
    let mut outputs=BTreeSet::new();if let Some(ds)=rule["disclosure"].as_array(){for d in ds{let output=SecurityRef::read(&d["field"])?;if !b.fields.contains(&output)||!outputs.insert(output){return Err(fail());}let v=&d["disposition"];if v["kind"]=="transformed"{let declaration=SecurityRef::read(&v["field"])?;if !closure.types.values().any(|b|b.fields.contains(&declaration)){return Err(fail());}let field=locate(catalog,&declaration)?;domain(field)?;if field["nullability"]!="required"||field.get("extensions").is_some_and(|e|e.as_object().is_none_or(|o|!o.is_empty())){return Err(fail());}crate::security_literals::check_literal(field,&v["value"]).map_err(|_|fail())?;}}}
   }
  }
  Ok(Self{checked_rules:rules.len(),checked_nodes:nodes})
 }
}
