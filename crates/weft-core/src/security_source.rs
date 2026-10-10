//! Bounded source shape/pin admission only. Not full ontology/domain interpretation or authorization.
use crate::{error::{Diagnostic,Result},json::{checked_json,sha256},model::Catalog};
use serde_json::Value;
use std::collections::BTreeSet;
#[jsonschema::validator(path="../../spec/upstream/umf-security-policy-0.1.0-selected.schema.json")]
struct PolicySchema;
#[jsonschema::validator(path="../../spec/upstream/umf-security-ontology-0.1.0-selected.schema.json")]
struct OntologySchema;
#[jsonschema::validator(path="../../spec/upstream/umf-security-policy-0.2.0-draft-selected.schema.json")]
struct CandidatePolicySchema;
#[jsonschema::validator(path="../../spec/upstream/umf-security-ontology-0.2.0-draft-selected.schema.json")]
struct CandidateOntologySchema;
fn fail(code:&str)->Diagnostic{Diagnostic::new(code,"model","Security source admission refused")}
/// Source bytes and parsed trees are immutable snapshots, not trust credentials.
#[derive(Debug)]
pub struct SecuritySourcePacket{model_pins:Vec<crate::ir::ModelPin>,policy_json:String,ontology_json:String,policy:Value,ontology:Value}
impl SecuritySourcePacket{
 pub(crate) fn require_catalog(&self,catalog:&Catalog)->Result<()>{
  if catalog.pins()!=self.model_pins||catalog.inputs().len()!=catalog.documents.len(){return Err(fail("WFT-SECURITY-PIN"));}
  for (input,document) in catalog.inputs().iter().zip(&catalog.documents){
   if sha256(input.document_json.as_bytes())!=input.pin.sha256||input.pin.document_id!=document["id"]||input.pin.umf_version!=document["umf"]||checked_json(&input.document_json).map_err(|_|fail("WFT-SECURITY-PIN"))?!=*document{return Err(fail("WFT-SECURITY-PIN"));}
  }Ok(())
 }
 pub fn policy(&self)->&Value{&self.policy}
 pub fn ontology(&self)->&Value{&self.ontology}
 pub fn policy_json(&self)->&str{&self.policy_json}
 pub fn ontology_json(&self)->&str{&self.ontology_json}
 pub fn read(policy_json:&str,ontology_json:&str,catalog:&Catalog)->Result<Self>{Self::read_profile(policy_json,ontology_json,catalog,false)}
 fn read_profile(policy_json:&str,ontology_json:&str,catalog:&Catalog,candidate:bool)->Result<Self>{
  if policy_json.len()>4_000_000||ontology_json.len()>4_000_000{return Err(fail("WFT-SECURITY-LIMIT"));}
  let policy=checked_json(policy_json).map_err(|_|fail("WFT-SECURITY-SOURCE"))?;
  let ontology=checked_json(ontology_json).map_err(|_|fail("WFT-SECURITY-SOURCE"))?;
  fn bounds(expr:&Value,depth:usize,nodes:&mut usize)->Result<()>{
   *nodes+=1;if depth>16||*nodes>4096{return Err(fail("WFT-SECURITY-LIMIT"));}
   match expr["op"].as_str(){Some("and"|"or")=>{if let Some(args)=expr["args"].as_array(){for e in args{bounds(e,depth+1,nodes)?;}}},Some("not")=>bounds(&expr["arg"],depth+1,nodes)?,Some("exists")=>bounds(&expr["where"],depth+1,nodes)?,_=>{}};Ok(())
  }
  let mut nodes=0;if let Some(rules)=policy["rules"].as_array(){for rule in rules{bounds(&rule["condition"],1,&mut nodes)?;}}
  if !(if candidate{CandidatePolicySchema::is_valid(&policy)&&CandidateOntologySchema::is_valid(&ontology)}else{PolicySchema::is_valid(&policy)&&OntologySchema::is_valid(&ontology)}){return Err(fail("WFT-SECURITY-SOURCE"));}
  if policy["ontology"]["documentId"]!=ontology["documentId"]||policy["ontology"]["revision"]!=ontology["revision"]{return Err(fail("WFT-SECURITY-PIN"));}
  if catalog.inputs().len()!=catalog.documents.len(){return Err(fail("WFT-SECURITY-PIN"));}
  for (input,document) in catalog.inputs().iter().zip(&catalog.documents){
   if input.document_json.len()>4*1024*1024||sha256(input.document_json.as_bytes())!=input.pin.sha256||input.pin.document_id!=document["id"]||input.pin.umf_version!=document["umf"]||checked_json(&input.document_json).map_err(|_|fail("WFT-SECURITY-PIN"))?!=*document{return Err(fail("WFT-SECURITY-PIN"));}
  }
  let docs=ontology["documents"].as_array().unwrap();let mut ids=BTreeSet::new();
  if docs.len()!=catalog.inputs().len(){return Err(fail("WFT-SECURITY-PIN"));}
  for doc in docs{
   let id=doc["documentId"].as_str().unwrap();
   if !ids.insert(id)||!catalog.inputs().iter().any(|m|m.pin.document_id==id&&doc["revision"]==m.pin.revision&&m.pin.umf_version=="0.8.0"){return Err(fail("WFT-SECURITY-PIN"));}
  }
  let mut rule_ids=BTreeSet::new();for rule in policy["rules"].as_array().unwrap(){
   if !rule_ids.insert(rule["id"].as_str().unwrap()){return Err(fail("WFT-SECURITY-SOURCE"));}
   if rule["effect"]!="permit"&&rule.get("disclosure").is_some(){return Err(fail("WFT-SECURITY-SOURCE"));}
  }
  Ok(Self{model_pins:catalog.pins(),policy_json:policy_json.into(),ontology_json:ontology_json.into(),policy,ontology})
 }
}

/// Separate draft 0.2 shape/pin snapshot. Cannot be passed to admitted 0.1 plans.
/// No ontology interpretation, source authentication or public compiler activation.
#[derive(Debug)]
pub struct SecurityCandidateSourcePacket{packet:SecuritySourcePacket}
impl SecurityCandidateSourcePacket{
 pub fn read(policy_json:&str,ontology_json:&str,catalog:&Catalog)->Result<Self>{Ok(Self{packet:SecuritySourcePacket::read_profile(policy_json,ontology_json,catalog,true)?})}
 pub fn policy(&self)->&Value{self.packet.policy()}
 pub fn ontology(&self)->&Value{self.packet.ontology()}
 pub fn policy_json(&self)->&str{self.packet.policy_json()}
 pub fn ontology_json(&self)->&str{self.packet.ontology_json()}
 pub fn require_catalog(&self,catalog:&Catalog)->Result<()>{self.packet.require_catalog(catalog)}
}
