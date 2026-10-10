//! Typed rule occurrence custody. No backend profile issuance or native admission.
use std::collections::BTreeMap;
use crate::{error::{Diagnostic,Result},security_ir::{Rule,Expression,Term,Disposition},security_ontology::SecurityRef,security_obligation_sources::{OwnerSourceDemands,OwnerEventKind}};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-LOWERING-UNSUPPORTED","capability","Rule occurrence issuance refused")}
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) enum RulePath {Rule,Condition(Vec<usize>),Operand(Vec<usize>,u8),Disclosure(usize)}
#[derive(Debug)]
pub(crate) enum RulePayload<'a>{Rule(&'a Rule),Condition(&'a Expression),Operand(&'a Term),Disclosure{field:&'a SecurityRef,disposition:&'a Disposition}}
#[allow(dead_code)]
pub(crate) struct RuleOccurrences<'a,'d,'m,'c,'s>{owner:&'a OwnerSourceDemands<'d,'m,'c,'s>,entries:BTreeMap<(&'a str,RulePath),RulePayload<'a>>}
#[allow(dead_code)]
impl<'a,'d,'m,'c,'s> RuleOccurrences<'a,'d,'m,'c,'s>{
 pub(crate) fn owner(&self)->&'a OwnerSourceDemands<'d,'m,'c,'s>{self.owner}
 pub(crate) fn entries(&self)->&BTreeMap<(&'a str,RulePath),RulePayload<'a>>{&self.entries}
}
struct Budget{work:usize,text:usize}
impl Budget{fn charge(&mut self,bytes:usize)->Result<()>{if self.work==0||self.text<bytes{return Err(fail());}self.work-=1;self.text-=bytes;Ok(())}}
struct Builder<'a>{budget:Budget,entries:BTreeMap<(&'a str,RulePath),RulePayload<'a>>}
impl<'a> Builder<'a>{
 fn insert(&mut self,source:&'a str,path:RulePath,payload:RulePayload<'a>)->Result<()>{
  self.budget.charge(source.len())?;if self.entries.len()>=4096{return Err(fail());}
  if self.entries.insert((source,path),payload).is_some(){return Err(fail());}Ok(())
 }
 fn path(&mut self,p:&[usize])->Result<Vec<usize>>{if p.len()>64{return Err(fail());}self.budget.charge(p.len().checked_mul(std::mem::size_of::<usize>()).ok_or_else(fail)?)?;Ok(p.to_vec())}
 fn child(&mut self,source:&'a str,e:&'a Expression,path:&mut Vec<usize>,position:usize)->Result<()>{
  if path.len()>=64{return Err(fail());}self.budget.charge(std::mem::size_of::<usize>())?;path.push(position);self.expression(source,e,path)?;path.pop();Ok(())
 }
 fn expression(&mut self,source:&'a str,e:&'a Expression,path:&mut Vec<usize>)->Result<()>{
  let owned=self.path(path)?;self.insert(source,RulePath::Condition(owned),RulePayload::Condition(e))?;
  match e {
   Expression::Literal(_)=>{},
   Expression::Equal(left,right)=>{for (position,term) in [(0,left),(1,right)]{let owned=self.path(path)?;self.insert(source,RulePath::Operand(owned,position),RulePayload::Operand(term))?;}},
   Expression::And(args)|Expression::Or(args)=>{for (i,arg) in args.iter().enumerate(){self.child(source,arg,path,i)?;}},
   Expression::Not(arg)=>self.child(source,arg,path,0)?,
   Expression::Exists{condition,..}=>self.child(source,condition,path,0)?,
  }Ok(())
 }
 fn rule(&mut self,source:&'a str,rule:&'a Rule)->Result<()>{
  self.insert(source,RulePath::Rule,RulePayload::Rule(rule))?;self.expression(source,&rule.condition,&mut Vec::new())?;
  for (i,(field,disposition)) in rule.disclosure.iter().enumerate(){self.budget.charge(std::mem::size_of::<usize>())?;self.insert(source,RulePath::Disclosure(i),RulePayload::Disclosure{field,disposition})?;}Ok(())
 }
}
#[allow(dead_code)]
pub(crate) fn issue<'a,'d,'m,'c,'s>(owner:&'a OwnerSourceDemands<'d,'m,'c,'s>)->Result<RuleOccurrences<'a,'d,'m,'c,'s>>{issue_budget(owner,1_000_000,16_000_000).map(|(out,_,_)|out)}
fn issue_budget<'a,'d,'m,'c,'s>(owner:&'a OwnerSourceDemands<'d,'m,'c,'s>,work:usize,text:usize)->Result<(RuleOccurrences<'a,'d,'m,'c,'s>,usize,usize)>{
 let mut b=Builder{budget:Budget{work,text},entries:BTreeMap::new()};
 for (source,kind) in owner.events(){b.budget.charge(source.len())?;if *kind==OwnerEventKind::Rule&&!owner.rule_events().contains_key(source){return Err(fail());}}
 for (source,rule) in owner.rule_events(){
  b.budget.charge(source.len())?;
  if owner.events().get(source)!=Some(&OwnerEventKind::Rule)||!owner.demands().get(source).is_some_and(|scopes|!scopes.is_empty()){return Err(fail());}
  b.rule(source,rule)?;
 }
 let work=b.budget.work;let text=b.budget.text;Ok((RuleOccurrences{owner,entries:b.entries},work,text))
}
#[cfg(test)]
pub(crate) fn test_budget<'a,'d,'m,'c,'s>(owner:&'a OwnerSourceDemands<'d,'m,'c,'s>,work:usize,text:usize)->Result<(usize,usize)>{issue_budget(owner,work,text).map(|(_,w,t)|(w,t))}
#[cfg(test)]
mod tests{
 use super::*;use crate::security_ir::Effect;
 fn rule(condition:Expression)->Rule{Rule{id:"r".into(),effect:Effect::Permit,actions:vec!["read".into()],target:SecurityRef{document_id:"d".into(),module_id:"m".into(),element_id:"t".into()},condition,disclosure:vec![]}}
 #[test]
 fn exact_paths_keep_false_empty_nodes_operands_and_repeated_disclosures(){
  let field=SecurityRef{document_id:"d".into(),module_id:"m".into(),element_id:"f".into()};
  let domain=serde_json::json!({"kind":"boolean"});let mut r=rule(Expression::And(vec![Expression::Literal(false),Expression::Or(vec![]),Expression::Not(Box::new(Expression::Equal(Term::Context{field:field.clone(),domain:domain.clone()},Term::Constant{field:field.clone(),domain,literal:serde_json::json!({"boolean":true})})))]));
  r.disclosure=vec![(field.clone(),Disposition::Withheld),(field.clone(),Disposition::Withheld),(field.clone(),Disposition::Original),(field.clone(),Disposition::Transformed{transform:"literal".into(),version:"v1".into(),output_field:field,domain:serde_json::json!({"kind":"boolean"}),literal:serde_json::json!({"boolean":false})})];
  let mut b=Builder{budget:Budget{work:100,text:10000},entries:BTreeMap::new()};b.rule("s",&r).unwrap();
  let expected=vec![RulePath::Rule,RulePath::Condition(vec![]),RulePath::Condition(vec![0]),RulePath::Condition(vec![1]),RulePath::Condition(vec![2]),RulePath::Condition(vec![2,0]),RulePath::Operand(vec![2,0],0),RulePath::Operand(vec![2,0],1),RulePath::Disclosure(0),RulePath::Disclosure(1),RulePath::Disclosure(2),RulePath::Disclosure(3)];
  assert_eq!(b.entries.keys().map(|(_,p)|p.clone()).collect::<std::collections::BTreeSet<_>>(),expected.into_iter().collect());
  let RulePayload::Condition(node)=&b.entries[&("s",RulePath::Condition(vec![0]))] else {panic!()};let Expression::And(args)=&r.condition else {panic!()};assert!(std::ptr::eq(*node,&args[0]));
  for i in [0,1,2,3]{let RulePayload::Disclosure{field,disposition}=&b.entries[&("s",RulePath::Disclosure(i))] else{panic!()};assert!(std::ptr::eq(*field,&r.disclosure[i].0));assert!(std::ptr::eq(*disposition,&r.disclosure[i].1));}
 }
 #[test]
 fn exact_occurrence_and_depth_boundaries_refuse_without_returning_inventory(){
  for (size,ok) in [(4094,true),(4095,false)]{let r=rule(Expression::And((0..size).map(|_|Expression::Literal(false)).collect()));let mut b=Builder{budget:Budget{work:1_000_000,text:16_000_000},entries:BTreeMap::new()};assert_eq!(b.rule("s",&r).is_ok(),ok);if ok{assert_eq!(b.entries.len(),4096);}}
  for (depth,ok) in [(64,true),(65,false)]{let mut e=Expression::Literal(false);for _ in 0..depth{e=Expression::Not(Box::new(e));}let r=rule(e);let mut b=Builder{budget:Budget{work:1_000_000,text:16_000_000},entries:BTreeMap::new()};assert_eq!(b.rule("s",&r).is_ok(),ok);}
 }
 #[test]
 fn independent_small_ledger_accounts_for_stack_and_each_owned_path(){
  let field=SecurityRef{document_id:"d".into(),module_id:"m".into(),element_id:"f".into()};
  let r=rule(Expression::Not(Box::new(Expression::Equal(Term::Context{field:field.clone(),domain:serde_json::json!({"kind":"boolean"})},Term::Constant{field,domain:serde_json::json!({"kind":"boolean"}),literal:serde_json::json!({"boolean":false})}))));
  // Five entry source bytes; stack push + three one-index copies.10 charges.
  let exact=5+4*std::mem::size_of::<usize>();
  for (work,text,ok) in [(10,exact,true),(9,exact,false),(10,exact-1,false)]{
   let mut b=Builder{budget:Budget{work,text},entries:BTreeMap::new()};assert_eq!(b.rule("s",&r).is_ok(),ok);
   if ok {assert_eq!(b.entries.len(),5);assert_eq!(b.budget.work,0);assert_eq!(b.budget.text,0);}
  }
 }

}
