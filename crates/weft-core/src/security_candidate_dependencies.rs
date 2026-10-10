//! Draft action dependencies only; no authenticated inventory or execution permit.
use crate::{error::{Diagnostic,Result},model::Catalog,security_candidate_ir::{SecurityCandidateLogicalPlan,CandidateExpression,CandidateTerm,CandidateWitness,CandidateEndpointCarrier,CandidateSide},security_candidate_ontology::SecurityCandidateOntologyClosure,security_association_ref::{SecurityAssociationRef,SecurityRelationshipRef},security_ontology::SecurityRef,security_ir::{Term,Binding}};
use std::collections::{BTreeMap,BTreeSet};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-CANDIDATE-DEPENDENCIES","model","Draft dependency closure refused")}
#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub struct CandidateIncidence{pub relationship:SecurityRelationshipRef,pub role:String,pub side:CandidateSide,pub target:SecurityRef,pub key_id:String}
#[derive(Debug)]
pub struct SecurityCandidateDependencies{keys:BTreeMap<SecurityRef,(String,Vec<SecurityRef>)>,fields:BTreeMap<SecurityRef,BTreeSet<SecurityRef>>,context:BTreeSet<SecurityRef>,associations:BTreeSet<SecurityAssociationRef>,incidences:BTreeSet<CandidateIncidence>}
impl SecurityCandidateDependencies{
 pub fn keys(&self)->&BTreeMap<SecurityRef,(String,Vec<SecurityRef>)>{&self.keys}
 pub fn fields(&self)->&BTreeMap<SecurityRef,BTreeSet<SecurityRef>>{&self.fields}
 pub fn context(&self)->&BTreeSet<SecurityRef>{&self.context}
 pub fn associations(&self)->&BTreeSet<SecurityAssociationRef>{&self.associations}
 pub fn incidences(&self)->&BTreeSet<CandidateIncidence>{&self.incidences}
 pub fn derive(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,target:&SecurityRef,action:&str)->Result<Self>{
  plan.require_catalog(catalog)?;let closure=SecurityCandidateOntologyClosure::read(plan.source(),catalog)?;
  if !closure.types.contains_key(target)||!plan.source().ontology()["actions"].as_array().ok_or_else(fail)?.iter().any(|a|a==action){return Err(fail());}
  let result=Self{keys:BTreeMap::new(),fields:BTreeMap::new(),context:BTreeSet::new(),associations:BTreeSet::new(),incidences:BTreeSet::new()};
  let retained=target.document_id.len()+target.module_id.len()+target.element_id.len();let text=16_000_000usize.checked_sub(retained).ok_or_else(fail)?;let mut c=Collector{closure:&closure,resource:target.clone(),variables:BTreeMap::new(),result,work:999_999,text};c.key(target)?;c.key(&closure.subject)?;
  for rule in plan.rules(){c.charge(0)?;if &rule.target==target&&rule.actions.iter().any(|a|a==action){c.expression(&rule.condition)?;}}
  Ok(c.result)
 }
}
struct Collector<'a>{closure:&'a SecurityCandidateOntologyClosure,resource:SecurityRef,variables:BTreeMap<usize,Option<SecurityRef>>,result:SecurityCandidateDependencies,work:usize,text:usize}
impl Collector<'_>{
 fn charge(&mut self,text:usize)->Result<()>{self.work=self.work.checked_sub(1).ok_or_else(fail)?;self.text=self.text.checked_sub(text).ok_or_else(fail)?;Ok(())}
 fn reference(&mut self,r:&SecurityRef)->Result<()>{self.charge(r.document_id.len()+r.module_id.len()+r.element_id.len())}
 fn key(&mut self,r:&SecurityRef)->Result<()>{self.reference(r)?;let closure=self.closure;let b=closure.types.get(r).ok_or_else(fail)?;let id=b.source["keyId"].as_str().ok_or_else(fail)?;self.charge(id.len())?;for f in &b.keys{self.reference(f)?;}let id=id.to_string();let fields=b.keys.clone();for f in &fields{self.field(r,f)?;}self.result.keys.insert(r.clone(),(id,fields));Ok(())}

 fn field(&mut self,r:&SecurityRef,f:&SecurityRef)->Result<()>{self.reference(r)?;self.reference(f)?;if !self.closure.types.get(r).ok_or_else(fail)?.fields.contains(f){return Err(fail());}self.result.fields.entry(r.clone()).or_default().insert(f.clone());Ok(())}
 fn endpoint(&mut self,a:&SecurityAssociationRef,role:&str,target:&SecurityRef,key_id:&str,carrier:&CandidateEndpointCarrier)->Result<()>{
  self.charge(role.len()+key_id.len())?;self.key(target)?;
  match (a,carrier){(SecurityAssociationRef::Record(owner),CandidateEndpointCarrier::Members{fields})=>{for f in fields{self.field(owner,f)?;}},(SecurityAssociationRef::Relationship(r),CandidateEndpointCarrier::Incidence{side})=>{self.charge(r.document_id.len()+r.module_id.len()+r.relationship_id.len())?;self.reference(target)?;self.result.incidences.insert(CandidateIncidence{relationship:r.clone(),role:role.into(),side:side.clone(),target:target.clone(),key_id:key_id.into()});},_=>return Err(fail())};Ok(())
 }
 fn term(&mut self,t:&CandidateTerm)->Result<()>{self.charge(0)?;match t{
  CandidateTerm::Endpoint{association,role,target,key_id,carrier,..}=>self.endpoint(association,role,target,key_id,carrier)?,
  CandidateTerm::Value(v)=>match v{Term::Identity{target,..}=>self.key(target)?,Term::Field{binding,field,..}=>{let owner=match binding{Binding::Subject=>self.closure.subject.clone(),Binding::Resource=>self.resource.clone(),Binding::Variable(slot)=>self.variables.get(slot).ok_or_else(fail)?.clone().ok_or_else(fail)?};self.field(&owner,field)?;},Term::Context{field,..}=>{self.reference(field)?;self.result.context.insert(field.clone());},Term::Constant{..}=>{},Term::Endpoint{..}=>return Err(fail())}
 };Ok(())}
 fn expression(&mut self,e:&CandidateExpression)->Result<()>{self.charge(0)?;match e{
  CandidateExpression::Literal(_)=>{},CandidateExpression::Equal(a,b)=>{self.term(a)?;self.term(b)?;},CandidateExpression::And(xs)|CandidateExpression::Or(xs)=>{for x in xs{self.expression(x)?;}},CandidateExpression::Not(x)=>self.expression(x)?,
  CandidateExpression::Exists{slot,association,witness,condition}=>{
   let owner=match witness{CandidateWitness::RecordKey{owner,..}=>{self.key(owner)?;let fields=self.closure.types.get(owner).ok_or_else(fail)?.fields.clone();for f in fields{self.field(owner,&f)?;}self.reference(owner)?;Some(owner.clone())},CandidateWitness::OpaqueExistential=>None};
   if self.variables.insert(*slot,owner).is_some(){return Err(fail());}match association{SecurityAssociationRef::Record(r)=>self.reference(r)?,SecurityAssociationRef::Relationship(r)=>self.charge(r.document_id.len()+r.module_id.len()+r.relationship_id.len())?};self.result.associations.insert(association.clone());
   let closure=self.closure;let selector=closure.associations.get(association).ok_or_else(fail)?;let endpoints=selector["endpoints"].as_array().ok_or_else(fail)?;
   for endpoint in endpoints{let target=SecurityRef::read(&endpoint["target"])?;let key_id=self.closure.types.get(&target).ok_or_else(fail)?.source["keyId"].as_str().ok_or_else(fail)?.to_string();let carrier=match association{SecurityAssociationRef::Record(_)=>CandidateEndpointCarrier::Members{fields:endpoint["fields"].as_array().ok_or_else(fail)?.iter().map(SecurityRef::read).collect::<Result<_>>()?},SecurityAssociationRef::Relationship(_)=>CandidateEndpointCarrier::Incidence{side:match endpoint["side"].as_str(){Some("source")=>CandidateSide::Source,Some("target")=>CandidateSide::Target,_=>return Err(fail())}}};self.endpoint(association,endpoint["role"].as_str().ok_or_else(fail)?,&target,&key_id,&carrier)?;}
   self.expression(condition)?;self.variables.remove(slot);
  }
 };Ok(())}
}

#[cfg(test)]
mod tests{
 use super::*;
 #[test]
 fn dependency_budgets_refuse_work_and_text_exhaustion(){
  let r=SecurityRef{document_id:"d".into(),module_id:"m".into(),element_id:"r".into()};let closure=SecurityCandidateOntologyClosure{types:BTreeMap::new(),subject:r.clone(),associations:BTreeMap::new(),context:std::collections::BTreeSet::new()};let result=SecurityCandidateDependencies{keys:BTreeMap::new(),fields:BTreeMap::new(),context:BTreeSet::new(),associations:BTreeSet::new(),incidences:BTreeSet::new()};let mut c=Collector{closure:&closure,resource:r,variables:BTreeMap::new(),result,work:1,text:3};assert!(c.charge(3).is_ok());assert_eq!(c.charge(0).unwrap_err().code,"WFT-SECURITY-CANDIDATE-DEPENDENCIES");c.work=1;assert_eq!(c.charge(1).unwrap_err().code,"WFT-SECURITY-CANDIDATE-DEPENDENCIES");
 }
 #[test]
 fn graph_endpoint_charges_every_retained_identifier_copy_before_issuance(){
  let target=SecurityRef{document_id:"d".into(),module_id:"m".into(),element_id:"r".into()};let field=SecurityRef{document_id:"d".into(),module_id:"m".into(),element_id:"f".into()};let closure=SecurityCandidateOntologyClosure{types:BTreeMap::from([(target.clone(),crate::security_ontology::TypeBinding{source:serde_json::json!({"keyId":"pk"}),fields:BTreeSet::from([field.clone()]),keys:vec![field]})]),subject:target.clone(),associations:BTreeMap::new(),context:std::collections::BTreeSet::new()};
  let association=SecurityAssociationRef::Relationship(SecurityRelationshipRef{document_id:"d".into(),module_id:"m".into(),relationship_id:"a".into()});
  for available in [22,23]{let result=SecurityCandidateDependencies{keys:BTreeMap::new(),fields:BTreeMap::new(),context:BTreeSet::new(),associations:BTreeSet::new(),incidences:BTreeSet::new()};let mut c=Collector{closure:&closure,resource:target.clone(),variables:BTreeMap::new(),result,work:100,text:available};let outcome=c.endpoint(&association,"s",&target,"pk",&CandidateEndpointCarrier::Incidence{side:CandidateSide::Source});assert_eq!(outcome.is_ok(),available==23);if outcome.is_ok(){
   let size=|r:&SecurityRef|r.document_id.len()+r.module_id.len()+r.element_id.len();let key_bytes:usize=c.result.keys.iter().map(|(r,(id,fs))|size(r)+id.len()+fs.iter().map(size).sum::<usize>()).sum();let field_bytes:usize=c.result.fields.iter().map(|(r,fs)|size(r)+fs.iter().map(size).sum::<usize>()).sum();let incidence_bytes:usize=c.result.incidences.iter().map(|i|i.relationship.document_id.len()+i.relationship.module_id.len()+i.relationship.relationship_id.len()+i.role.len()+size(&i.target)+i.key_id.len()).sum();assert_eq!(key_bytes+field_bytes+incidence_bytes,23);assert_eq!(c.text,0);
  }}
 }

}
