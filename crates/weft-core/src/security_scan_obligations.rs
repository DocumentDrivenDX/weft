//! Conservative per-scan fact obligations for physical mapping qualification.
//! These immutable inventories do not authenticate facts or authorize execution.
use crate::{error::{Diagnostic,Result},model::Catalog,security_ir::{SecurityLogicalPlan,Expression,Term,Binding},security_ontology::{SecurityOntologyClosure,SecurityRef},security_query_uses::SecurityResolvedQuery};
use std::collections::{BTreeMap,BTreeSet};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-SCAN-OBLIGATIONS","model","Security scan obligation closure refused")}
#[derive(Debug,PartialEq,Eq)]
pub struct ActionObligations{action:String,rule_ids:Vec<String>,keys:BTreeMap<SecurityRef,(String,Vec<SecurityRef>)>,fields:BTreeMap<SecurityRef,BTreeSet<SecurityRef>>,context:BTreeSet<SecurityRef>,associations:BTreeSet<SecurityRef>}
impl ActionObligations{
 pub fn action(&self)->&str{&self.action}
 pub fn rule_ids(&self)->&[String]{&self.rule_ids}
 /// Full ordered identity components, never labels or digests.
 pub fn keys(&self)->&BTreeMap<SecurityRef,(String,Vec<SecurityRef>)>{&self.keys}
 pub fn fields(&self)->&BTreeMap<SecurityRef,BTreeSet<SecurityRef>>{&self.fields}
 pub fn context(&self)->&BTreeSet<SecurityRef>{&self.context}
 /// Complete private association inventories; empty sets require completeness evidence.
 pub fn associations(&self)->&BTreeSet<SecurityRef>{&self.associations}
}
#[derive(Debug,PartialEq,Eq)]
pub struct ScanObligations{scan:String,target:SecurityRef,actions:Vec<ActionObligations>,projection_fields:BTreeSet<SecurityRef>,query_fields:BTreeSet<SecurityRef>}
impl ScanObligations{
 pub fn scan(&self)->&str{&self.scan}
 pub fn target(&self)->&SecurityRef{&self.target}
 pub fn actions(&self)->&[ActionObligations]{&self.actions}
 pub fn projection_fields(&self)->&BTreeSet<SecurityRef>{&self.projection_fields}
 pub fn query_fields(&self)->&BTreeSet<SecurityRef>{&self.query_fields}
}
struct Budget{remaining:usize,text_remaining:usize}
impl Budget{
 fn step(&mut self)->Result<()>{if self.remaining==0{return Err(fail());}self.remaining-=1;Ok(())}
 fn text(&mut self,size:usize)->Result<()>{if size>self.text_remaining{return Err(fail());}self.text_remaining-=size;Ok(())}
 fn reference(&mut self,r:&SecurityRef)->Result<()>{self.text(r.document_id.len()+r.module_id.len()+r.element_id.len())}
}
struct Collector<'a>{resource:SecurityRef,variables:BTreeMap<usize,SecurityRef>,closure:&'a SecurityOntologyClosure,result:ActionObligations,budget:&'a mut Budget}
impl Collector<'_>{
 fn step(&mut self)->Result<()>{self.budget.step()}
 fn key(&mut self,target:&SecurityRef)->Result<()>{self.step()?;let binding=self.closure.types.get(target).ok_or_else(fail)?;let id=binding.source["keyId"].as_str().ok_or_else(fail)?.to_string();self.budget.reference(target)?;self.budget.text(id.len())?;for field in &binding.keys{self.budget.step()?;self.budget.reference(field)?;}self.result.keys.insert(target.clone(),(id,binding.keys.clone()));self.budget.reference(target)?;for field in &binding.keys{self.budget.reference(field)?;}self.result.fields.entry(target.clone()).or_default().extend(binding.keys.iter().cloned());Ok(())}
 fn field(&mut self,target:&SecurityRef,field:&SecurityRef)->Result<()>{self.step()?;if !self.closure.types.get(target).ok_or_else(fail)?.fields.contains(field){return Err(fail());}self.budget.reference(target)?;self.budget.reference(field)?;self.result.fields.entry(target.clone()).or_default().insert(field.clone());Ok(())}
 fn term(&mut self,term:&Term)->Result<()>{self.step()?;match term{
  Term::Identity{target,..}=>self.key(target)?,
  Term::Endpoint{association,role,target,..}=>{let association=association.record().ok_or_else(fail)?;let binding=self.closure.types.get(association).ok_or_else(fail)?;let endpoint=binding.source["endpoints"].as_array().ok_or_else(fail)?.iter().find(|e|e["role"]==*role).ok_or_else(fail)?;let fields=endpoint["fields"].as_array().ok_or_else(fail)?.iter().map(SecurityRef::read).collect::<Result<Vec<_>>>()?;for field in fields{self.field(association,&field)?;}self.key(target)?;},
  Term::Field{binding,field,..}=>{let target=match binding{Binding::Resource=>self.resource.clone(),Binding::Subject=>self.closure.subject().clone(),Binding::Variable(slot)=>self.variables.get(slot).ok_or_else(fail)?.clone()};self.field(&target,field)?;},
  Term::Context{field,..}=>{self.budget.reference(field)?;self.result.context.insert(field.clone());},
  Term::Constant{..}=>{} // A constant's domain reference is not a live fact read.
 }Ok(())}
 fn expression(&mut self,expression:&Expression)->Result<()>{self.step()?;match expression{
  Expression::Literal(_)=>{},Expression::Equal(a,b)=>{self.term(a)?;self.term(b)?;},
  Expression::And(args)|Expression::Or(args)=>{for arg in args{self.expression(arg)?;}},
  Expression::Not(arg)=>self.expression(arg)?,
  Expression::Exists{slot,association,condition}=>{let association=association.record().ok_or_else(fail)?;
   self.budget.reference(association)?;if self.variables.insert(*slot,association.clone()).is_some(){return Err(fail());}
   self.budget.reference(association)?;self.result.associations.insert(association.clone());self.key(association)?;
   // Match complete fact-cut admission: preserve every classified association
   // field and ordered endpoint key, not just a chosen SQL join column.
   let binding=self.closure.types.get(association).ok_or_else(fail)?;
   self.budget.reference(association)?;for field in &binding.fields{self.budget.step()?;self.budget.reference(field)?;}self.result.fields.entry(association.clone()).or_default().extend(binding.fields.iter().cloned());
   let targets=binding.source["endpoints"].as_array().ok_or_else(fail)?.iter().map(|e|SecurityRef::read(&e["target"])).collect::<Result<Vec<_>>>()?;
   for target in targets{self.key(&target)?;}self.expression(condition)?;self.variables.remove(slot);
  }
 }Ok(())}
}
pub(crate) fn derive(query:&SecurityResolvedQuery,plan:&SecurityLogicalPlan,catalog:&Catalog,primary_action:&str,uses:&[(crate::security_query_uses::QueryUse,Option<String>)])->Result<Vec<ScanObligations>>{
 query.require_sources(plan,catalog)?;let closure=SecurityOntologyClosure::read(plan.source(),catalog)?;let mut budget=Budget{remaining:1_000_000,text_remaining:16_000_000};let mut scans=Vec::new();
 for (scan,target) in query.scans(){
  let mut actions=BTreeSet::from([primary_action.to_string()]);for (usage,original_action) in uses{if &usage.scan==scan{if let Some(action)=original_action{actions.insert(action.clone());}}}
  let mut obligations=Vec::new();for action in actions{
   budget.reference(target)?;budget.text(action.len())?;let mut collector=Collector{resource:target.clone(),variables:BTreeMap::new(),closure:&closure,budget:&mut budget,result:ActionObligations{action:action.clone(),rule_ids:Vec::new(),keys:BTreeMap::new(),fields:BTreeMap::new(),context:BTreeSet::new(),associations:BTreeSet::new()}};
   collector.key(target)?;collector.key(closure.subject())?;
   for rule in plan.rules(){collector.step()?;if &rule.target!=target||!rule.actions.contains(&action){continue;}collector.budget.text(rule.id.len())?;
    collector.result.rule_ids.push(rule.id.clone());collector.expression(&rule.condition)?;
   }
   collector.result.rule_ids.sort();obligations.push(collector.result);
  }
  budget.step()?;budget.text(scan.len())?;budget.reference(target)?;for item in query.projections().iter().filter(|p|&p.scan==scan){budget.step()?;budget.reference(&item.field)?;}for item in query.uses().iter().filter(|u|&u.scan==scan){budget.step()?;budget.reference(&item.field)?;}
  scans.push(ScanObligations{scan:scan.clone(),target:target.clone(),actions:obligations,projection_fields:query.projections().iter().filter(|p|&p.scan==scan).map(|p|p.field.clone()).collect(),query_fields:query.uses().iter().filter(|u|&u.scan==scan).map(|u|u.field.clone()).collect()});
 }Ok(scans)
}

#[cfg(test)]
mod tests{
 use super::*;
 #[test]
 fn record_inventory_collector_refuses_graph_namespaces_before_inventing_reads(){
  let target=SecurityRef{document_id:"d".into(),module_id:"m".into(),element_id:"same".into()};
  let closure=SecurityOntologyClosure{types:BTreeMap::new(),fields:BTreeMap::new(),subject:target.clone(),context:BTreeSet::new()};
  let mut budget=Budget{remaining:100,text_remaining:100};
  let mut collector=Collector{resource:target,variables:BTreeMap::new(),closure:&closure,budget:&mut budget,result:ActionObligations{action:"read".into(),rule_ids:vec![],keys:BTreeMap::new(),fields:BTreeMap::new(),context:BTreeSet::new(),associations:BTreeSet::new()}};
  let association=crate::security_association_ref::SecurityAssociationRef::read(&serde_json::json!({"documentId":"d","moduleId":"m","relationshipId":"same"})).unwrap();
  let expr=Expression::Exists{slot:0,association,condition:Box::new(Expression::Literal(true))};
  assert_eq!(collector.expression(&expr).unwrap_err().code,"WFT-SECURITY-SCAN-OBLIGATIONS");
  assert!(collector.result.associations.is_empty());assert!(collector.result.keys.is_empty());assert!(collector.result.fields.is_empty());assert!(collector.variables.is_empty());
 }

 #[test]
 fn inventory_work_and_identifier_text_limits_refuse_before_underflow(){
  let mut work=Budget{remaining:1,text_remaining:3};assert!(work.step().is_ok());assert!(work.step().is_err());assert!(work.text(3).is_ok());assert!(work.text(1).is_err());
 }
}
