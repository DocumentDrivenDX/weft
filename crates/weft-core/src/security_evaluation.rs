//! Bounded logical evaluation of explicit simulated facts. No native admission or credentials.
use crate::{error::{Diagnostic,Result},json::checked_json,model::Catalog,security_ir::{SecurityLogicalPlan,Expression,Term,Binding,QueryOperator},security_composition::{Truth,Composition,Decision,not,and,or,compose_with_budget},security_ontology::{SecurityOntologyClosure,SecurityRef,locate},security_literals::ScalarLiteral,security_budget::PayloadBudget};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};
use std::sync::Arc;
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-EVALUATION","model","Security logical evaluation refused")}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct FieldValue{field:SecurityRef,value:Value}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Fact{#[serde(rename="type")]type_ref:SecurityRef,key:Vec<Value>,fields:Vec<FieldValue>,absent:Vec<SecurityRef>}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Coverage{#[serde(rename="type")]type_ref:SecurityRef,complete:bool,fields:Vec<SecurityRef>}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Cut{trusted:bool,generation:String,expected_generation:String,policy_id:String,policy_revision:String,ontology_document_id:String,ontology_revision:String,subjects:Vec<Fact>,facts:Vec<Fact>,coverage:Vec<Coverage>,context:Vec<FieldValue>,max_facts:usize,max_steps:usize}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Request{action:String,resources:Vec<Fact>,output:Vec<SecurityRef>,targets:Option<Vec<SecurityRef>>,query_uses:Option<Vec<QueryUse>>}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct QueryUse{field:SecurityRef,operator:QueryOperator,original_action:Option<String>}
// Typed skeletons must never reinterpret ordinary opaque literal object keys.
fn take_array(v:&mut Value,key:&str)->Result<Vec<Value>> {
 let member=v.as_object_mut().and_then(|o|o.get_mut(key)).ok_or_else(fail)?;
 if !member.is_array(){return Err(fail());}
 match std::mem::replace(member,Value::Array(vec![])){Value::Array(values)=>Ok(values),_=>Err(fail())}
}
fn read_field_value(v:Value)->Result<FieldValue>{
 let Value::Object(mut o)=v else{return Err(fail());};
 if o.len()!=2{return Err(fail());}
 let field=SecurityRef::read(&o.remove("field").ok_or_else(fail)?)?;
 let value=o.remove("value").ok_or_else(fail)?;Ok(FieldValue{field,value})
}
fn read_fact(v:Value)->Result<Fact>{
 let Value::Object(mut o)=v else{return Err(fail());};if o.len()!=4{return Err(fail());}
 let type_ref=SecurityRef::read(&o.remove("type").ok_or_else(fail)?)?;
 let Value::Array(key)=o.remove("key").ok_or_else(fail)? else{return Err(fail());};
 let Value::Array(fields)=o.remove("fields").ok_or_else(fail)? else{return Err(fail());};
 let Value::Array(absent)=o.remove("absent").ok_or_else(fail)? else{return Err(fail());};
 Ok(Fact{type_ref,key,fields:fields.into_iter().map(read_field_value).collect::<Result<Vec<_>>>()?,absent:absent.iter().map(SecurityRef::read).collect::<Result<Vec<_>>>()?})
}
fn read_cut(mut v:Value)->Result<Cut>{
 if !strict_cut(&v){return Err(fail());}
 let subjects=take_array(&mut v,"subjects")?;let facts=take_array(&mut v,"facts")?;let context=take_array(&mut v,"context")?;
 let mut cut:Cut=serde_json::from_value(v).map_err(|_|fail())?;
 cut.subjects=subjects.into_iter().map(read_fact).collect::<Result<Vec<_>>>()?;
 cut.facts=facts.into_iter().map(read_fact).collect::<Result<Vec<_>>>()?;
 cut.context=context.into_iter().map(read_field_value).collect::<Result<Vec<_>>>()?;Ok(cut)
}
fn read_request(mut v:Value)->Result<Request>{
 let resources=take_array(&mut v,"resources")?;let mut request:Request=serde_json::from_value(v).map_err(|_|fail())?;
 request.resources=resources.into_iter().map(read_fact).collect::<Result<Vec<_>>>()?;Ok(request)
}
fn read_scoped_rows(mut v:Value)->Result<ScopedRows>{
 if !strict_scoped_rows(&v){return Err(fail());}
 let rows=take_array(&mut v,"rows")?;let mut request:ScopedRows=serde_json::from_value(v).map_err(|_|fail())?;
 for row in rows {let Value::Object(row)=row else{return Err(fail());};
  request.rows.push(row.into_iter().map(|(scan,fact)|Ok((scan,read_fact(fact)?))).collect::<Result<BTreeMap<_,_>>>()?);
 }Ok(request)
}
fn query_mode<'a>(closure:&'a SecurityOntologyClosure,target:&SecurityRef,q:&QueryUse)->Result<&'a str>{
 let field=closure.types.get(target).ok_or_else(fail)?.source["fields"].as_array().ok_or_else(fail)?.iter().find(|f|SecurityRef::read(&f["ref"]).as_ref().is_ok_and(|r|r==&q.field)).ok_or_else(fail)?;
 Ok(field["queryUse"][q.operator.name()].as_str().unwrap_or(if field["protection"]=="protected"{"prohibited"}else{"disclosed"}))
}
#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
struct Identity{target:SecurityRef,key_id:String,components:Vec<ScalarLiteral>}
#[derive(Debug)]
struct NormalFact{identity:Identity,fields:BTreeMap<SecurityRef,Option<ScalarLiteral>>,absent:BTreeSet<SecurityRef>}
#[derive(Debug,PartialEq)]
enum TermValue{Identity(Identity),Scalar(Option<ScalarLiteral>)}
struct Budget{steps:usize,limit:usize,payload:PayloadBudget}
impl Budget{
 fn charge(&mut self,count:usize)->Result<()>{self.steps=self.steps.checked_add(count).ok_or_else(fail)?;if self.steps>self.limit{return Err(fail());}Ok(())}
 fn step(&mut self)->Result<()>{self.charge(1)}
 fn copy_scalar(&mut self,value:&ScalarLiteral)->Result<()>{self.payload.copy_scalar(value)}
 fn copy_identity(&mut self,value:&Identity)->Result<()>{for part in &value.components{self.copy_scalar(part)?;}Ok(())}
 fn literal(&mut self,catalog:&Catalog,r:&SecurityRef,v:&Value)->Result<Option<ScalarLiteral>>{
  self.step()?;self.payload.literal(locate(catalog,r)?,v)
 }
}
fn fact(input:Fact,closure:&SecurityOntologyClosure,catalog:&Catalog,budget:&mut Budget)->Result<NormalFact>{
 budget.step()?;let b=closure.types.get(&input.type_ref).ok_or_else(fail)?;
 if input.key.len()!=b.keys.len()||input.key.len()>64||input.fields.len()>4096||input.absent.len()>4096{return Err(fail());}
 let components=input.key.iter().zip(&b.keys).map(|(v,r)|budget.literal(catalog,r,v)?.ok_or_else(fail)).collect::<Result<Vec<_>>>()?;
 let mut fields=BTreeMap::new();for value in input.fields{if !b.fields.contains(&value.field)||fields.contains_key(&value.field){return Err(fail());}fields.insert(value.field.clone(),budget.literal(catalog,&value.field,&value.value)?);}
 let mut absent=BTreeSet::new();for r in input.absent{budget.step()?;if !b.fields.contains(&r)||fields.contains_key(&r)||locate(catalog,&r)?["nullability"]!="absent-allowed"||!absent.insert(r){return Err(fail());}}
 for (r,key) in b.keys.iter().zip(&components){if fields.get(r).is_some_and(|v|v.as_ref()!=Some(key)){return Err(fail());}}
 Ok(NormalFact{identity:Identity{target:input.type_ref,key_id:b.source["keyId"].as_str().ok_or_else(fail)?.into(),components},fields,absent})
}
struct Evaluator<'a>{catalog:&'a Catalog,closure:SecurityOntologyClosure,subject:NormalFact,facts:BTreeMap<SecurityRef,Vec<Arc<NormalFact>>>,coverage:BTreeMap<SecurityRef,BTreeSet<SecurityRef>>,context:BTreeMap<SecurityRef,Option<ScalarLiteral>>,budget:Budget}
impl Evaluator<'_>{
 fn complete(&self,r:&SecurityRef)->Result<()>{if self.coverage.contains_key(r){Ok(())}else{Err(fail())}}
 fn preflight(&mut self,expr:&Expression,target:&SecurityRef,resources:&[&NormalFact],variables:&BTreeMap<usize,SecurityRef>)->Result<()>{
  self.budget.step()?;
  match expr{
   Expression::Literal(_)=>{},Expression::Equal(a,b)=>{self.preflight_term(a,target,resources,variables)?;self.preflight_term(b,target,resources,variables)?;},
   Expression::And(args)|Expression::Or(args)=>for arg in args{self.preflight(arg,target,resources,variables)?;},Expression::Not(arg)=>self.preflight(arg,target,resources,variables)?,
   Expression::Exists{slot,association,condition}=>{let association=association.record().ok_or_else(fail)?;self.complete(association)?;let mut vars=variables.clone();vars.insert(*slot,association.clone());self.preflight(condition,target,resources,&vars)?;}
  }Ok(())
 }
 fn preflight_term(&mut self,term:&Term,target:&SecurityRef,resources:&[&NormalFact],variables:&BTreeMap<usize,SecurityRef>)->Result<()>{
  self.budget.step()?;
  let (binding,fields,endpoint_target)=match term{
   Term::Constant{..}=>return Ok(()),Term::Context{field,..}=>return if self.context.contains_key(field){Ok(())}else{Err(fail())},
   Term::Identity{binding,..}=>(binding,vec![],None),Term::Field{binding,field,..}=>(binding,vec![field.clone()],None),
   Term::Endpoint{binding,association,role,target,..}=>{let association=association.record().ok_or_else(fail)?;
    let endpoint=self.closure.types[association].source["endpoints"].as_array().ok_or_else(fail)?.iter().find(|e|e["role"]==*role).ok_or_else(fail)?;
    (binding,endpoint["fields"].as_array().ok_or_else(fail)?.iter().map(SecurityRef::read).collect::<Result<Vec<_>>>()?,Some(target))}
  };
  let type_ref=match binding{Binding::Subject=>&self.subject.identity.target,Binding::Resource=>target,Binding::Variable(slot)=>variables.get(slot).ok_or_else(fail)?};self.complete(type_ref)?;if let Some(target)=endpoint_target{self.complete(target)?;}
  let rows:Vec<&NormalFact>=match binding{Binding::Subject=>vec![&self.subject],Binding::Resource=>resources.to_vec(),Binding::Variable(_)=>self.facts.get(type_ref).map(|v|v.iter().map(Arc::as_ref).collect()).unwrap_or_default()};
  for field in fields{if !self.coverage[type_ref].contains(&field){return Err(fail());}for row in &rows{self.budget.step()?;if !row.fields.contains_key(&field)&&!row.absent.contains(&field){return Err(fail());}}}
  Ok(())
 }
 fn term(&mut self,t:&Term,resource:&NormalFact,variables:&BTreeMap<usize,&NormalFact>)->Result<Option<TermValue>>{
  self.budget.step()?;
  let binding=match t{Term::Constant{field,literal,..}=>return Ok(Some(TermValue::Scalar(self.budget.literal(self.catalog,field,literal)?))),Term::Context{field,..}=>{if let Some(Some(value))=self.context.get(field){self.budget.copy_scalar(value)?;}return Ok(self.context.get(field).cloned().map(TermValue::Scalar))},Term::Identity{binding,..}|Term::Endpoint{binding,..}|Term::Field{binding,..}=>binding};
  let row=match binding{Binding::Subject=>&self.subject,Binding::Resource=>resource,Binding::Variable(slot)=>variables.get(slot).copied().ok_or_else(fail)?};
  Ok(match t{
   Term::Identity{..}=>{self.budget.copy_identity(&row.identity)?;Some(TermValue::Identity(row.identity.clone()))},Term::Field{field,..}=>{if let Some(Some(value))=row.fields.get(field){self.budget.copy_scalar(value)?;}row.fields.get(field).cloned().map(TermValue::Scalar)},
   Term::Endpoint{association,role,target,key_id,..}=>{let association=association.record().ok_or_else(fail)?;
    let endpoint=self.closure.types[association].source["endpoints"].as_array().ok_or_else(fail)?.iter().find(|e|e["role"]==*role).ok_or_else(fail)?;
    let refs=endpoint["fields"].as_array().ok_or_else(fail)?.iter().map(SecurityRef::read).collect::<Result<Vec<_>>>()?;let mut components=Vec::new();for r in refs{let Some(Some(value))=row.fields.get(&r)else{return Ok(None)};self.budget.copy_scalar(value)?;components.push(value.clone());}
    Some(TermValue::Identity(Identity{target:target.clone(),key_id:key_id.clone(),components}))
   },_=>return Err(fail())})
 }
 fn expression(&mut self,expr:&Expression,resource:&NormalFact,variables:&BTreeMap<usize,&NormalFact>)->Result<Truth>{
  self.budget.step()?;
  Ok(match expr{
   Expression::Literal(v)=>if *v{Truth::True}else{Truth::False},Expression::Equal(a,b)=>match (self.term(a,resource,variables)?,self.term(b,resource,variables)?){(Some(a),Some(b))=>if a==b{Truth::True}else{Truth::False},_=>Truth::Unknown},
   Expression::Not(arg)=>not(self.expression(arg,resource,variables)?),Expression::And(args)|Expression::Or(args)=>{let values=args.iter().map(|a|self.expression(a,resource,variables)).collect::<Result<Vec<_>>>()?;if matches!(expr,Expression::And(_)){and(&values)?}else{or(&values)?}},
   Expression::Exists{slot,association,condition}=>{let association=association.record().ok_or_else(fail)?;
    // Relations remain immutable and shared, including nested same-type quantifiers.
    let count=self.facts.get(association).map(Vec::len).unwrap_or(0);self.budget.charge(count)?;
    let rows=self.facts.get(association).cloned().unwrap_or_default();
    let mut found=false;let mut unknown=false;for row in &rows{self.budget.step()?;let mut nested=variables.clone();nested.insert(*slot,row);match self.expression(condition,resource,&nested)?{Truth::True=>found=true,Truth::Unknown=>unknown=true,_=>{}}}
    if found{Truth::True}else if unknown{Truth::Unknown}else{Truth::False}
   }
  })
 }
}
/// Conditional logical simulation. A `trusted` flag and matching pins are only
/// caller assertions here; this API never admits native authority or releases data.
/// Output contains dispositions only. Query execution, writes, host cuts and binding parity
/// require later stages; unsupported request members refuse rather than disappearing.
pub fn simulate_json(plan:&SecurityLogicalPlan,catalog:&Catalog,cut_json:&str,request_json:&str)->Result<Vec<Composition>>{
 plan.require_catalog(catalog)?;if cut_json.len()+request_json.len()>4_000_000{return Err(fail());}
 let cut=read_cut(checked_json(cut_json).map_err(|_|fail())?)?;
 let request=read_request(checked_json(request_json).map_err(|_|fail())?)?;
 if !cut.trusted||cut.generation.is_empty()||cut.generation!=cut.expected_generation||cut.policy_id!=plan.source().policy()["id"]||cut.policy_revision!=plan.source().policy()["revision"]||cut.ontology_document_id!=plan.source().ontology()["documentId"]||cut.ontology_revision!=plan.source().ontology()["revision"]||cut.max_facts==0||cut.max_facts>10000||cut.max_steps==0||cut.max_steps>1_000_000||cut.subjects.len()!=1||cut.subjects.len()+cut.facts.len()+request.resources.len()>cut.max_facts||cut.coverage.len()>512||cut.context.len()>256{return Err(fail());}
 let closure=SecurityOntologyClosure::read(plan.source(),catalog)?;
 if !plan.source().ontology()["actions"].as_array().ok_or_else(fail)?.iter().any(|a|a==&request.action){return Err(fail());}
 let mut budget=Budget{steps:0,limit:cut.max_steps,payload:PayloadBudget::new("WFT-SECURITY-EVALUATION")};
 let subject=fact(cut.subjects.into_iter().next().ok_or_else(fail)?,&closure,catalog,&mut budget)?;if subject.identity.target!=closure.subject{return Err(fail());}
 let mut facts:BTreeMap<SecurityRef,Vec<Arc<NormalFact>>>=BTreeMap::new();let mut identities=BTreeSet::new();for input in cut.facts{let row=fact(input,&closure,catalog,&mut budget)?;budget.copy_identity(&row.identity)?;if !identities.insert(row.identity.clone()){return Err(fail());}facts.entry(row.identity.target.clone()).or_default().push(Arc::new(row));}
 let resources=request.resources.into_iter().map(|r|fact(r,&closure,catalog,&mut budget)).collect::<Result<Vec<_>>>()?;
 let mut coverage=BTreeMap::new();let mut seen=BTreeSet::new();for entry in cut.coverage{budget.step()?;let b=closure.types.get(&entry.type_ref).ok_or_else(fail)?;let fields:BTreeSet<_>=entry.fields.iter().cloned().collect();if !seen.insert(entry.type_ref.clone())||entry.fields.len()>4096||fields.len()!=entry.fields.len()||!fields.is_subset(&b.fields){return Err(fail());}if entry.complete{coverage.insert(entry.type_ref,fields);}}
 let mut context=BTreeMap::new();for entry in cut.context{if !closure.context.contains(&entry.field)||context.contains_key(&entry.field){return Err(fail());}context.insert(entry.field.clone(),budget.literal(catalog,&entry.field,&entry.value)?);}
 let targets=request.targets.unwrap_or_else(||resources.iter().map(|r|r.identity.target.clone()).collect::<BTreeSet<_>>().into_iter().collect());
 if targets.is_empty()||targets.len()>256||targets.iter().collect::<BTreeSet<_>>().len()!=targets.len()||targets.iter().any(|t|!closure.types.contains_key(t)||!coverage.contains_key(t))||resources.iter().any(|r|!targets.contains(&r.identity.target)){return Err(fail());}
 if request.output.len()>4096||request.output.iter().collect::<BTreeSet<_>>().len()!=request.output.len()||targets.iter().any(|t|request.output.iter().any(|f|!closure.types[t].fields.contains(f))){return Err(fail());}
 let queries=request.query_uses.unwrap_or_default();if queries.len()>4096{return Err(fail());}
 for q in &queries{if q.original_action.as_ref().is_some_and(|a|!plan.source().ontology()["actions"].as_array().unwrap().iter().any(|v|v==a)){return Err(fail());}}
 let mut evaluator=Evaluator{catalog,closure,subject,facts,coverage,context,budget};
 for target in &targets{
  let rows=resources.iter().filter(|r|&r.identity.target==target).collect::<Vec<_>>();let mut actions=BTreeSet::from([request.action.clone()]);
  for q in &queries{evaluator.budget.step()?;match query_mode(&evaluator.closure,target,q)?{
   "prohibited"=>return Err(fail()),"disclosed"=>{},"original-authorized"=>{
    let action=q.original_action.as_ref().ok_or_else(fail)?;if action==&request.action||!evaluator.coverage[target].contains(&q.field){return Err(fail());}
    for row in &rows{evaluator.budget.step()?;if !row.fields.contains_key(&q.field)&&!row.absent.contains(&q.field){return Err(fail());}}
    actions.insert(action.clone());
   },_=>return Err(fail())}}
  for rule in plan.rules().iter().filter(|r|&r.target==target&&r.actions.iter().any(|a|actions.contains(a))){evaluator.preflight(&rule.condition,target,&rows,&BTreeMap::new())?;}
 }
 let mut results=Vec::new();for resource in &resources{
  let mut truths=BTreeMap::new();for rule in plan.rules().iter().filter(|r|r.target==resource.identity.target&&r.actions.contains(&request.action)){truths.insert(rule.id.clone(),evaluator.expression(&rule.condition,resource,&BTreeMap::new())?);}
  let mut selected=request.output.clone();for q in &queries{if query_mode(&evaluator.closure,&resource.identity.target,q)?=="disclosed"&&!selected.contains(&q.field){selected.push(q.field.clone());}}
  let mut result=compose_with_budget(plan,catalog,&resource.identity.target,&request.action,&selected,&truths,&mut evaluator.budget.payload)?;
  if matches!(result.decision,Decision::Indeterminate|Decision::Conflict){return Err(fail());}if result.decision==Decision::Permit{
   for q in &queries{
    evaluator.budget.step()?;if query_mode(&evaluator.closure,&resource.identity.target,q)?=="original-authorized"{
     let action=q.original_action.as_ref().ok_or_else(fail)?;let mut original_truths=BTreeMap::new();for rule in plan.rules().iter().filter(|r|r.target==resource.identity.target&&r.actions.contains(action)){original_truths.insert(rule.id.clone(),evaluator.expression(&rule.condition,resource,&BTreeMap::new())?);}
     if compose_with_budget(plan,catalog,&resource.identity.target,action,&[],&original_truths,&mut evaluator.budget.payload)?.decision!=Decision::Permit{return Err(fail());}
    }else if result.disclosure.iter().any(|(field,d)|field==&q.field&&matches!(d,crate::security_ir::Disposition::Withheld)){return Err(fail());}
   }
   for (field,disposition) in &result.disclosure{if matches!(disposition,crate::security_ir::Disposition::Original)&&(!evaluator.coverage[&resource.identity.target].contains(field)||!resource.fields.contains_key(field)&&!resource.absent.contains(field)){return Err(fail());}}
   result.disclosure.retain(|(field,_)|request.output.contains(field));results.push(result);
  }
 }Ok(results)
}

#[cfg(test)]
mod tests{
 use super::*;
 #[test]
 fn record_fact_evaluator_refuses_graph_variants_instead_of_empty_relations(){
  let f:Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();let (catalog,plan)=plan(&f["policy"]);
  let closure=SecurityOntologyClosure::read(plan.source(),&catalog).unwrap();let target=closure.subject().clone();
  let reference=|element:&str|SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:element.into()};
  let row=NormalFact{identity:Identity{target:target.clone(),key_id:"pk".into(),components:vec![ScalarLiteral::String("s".into())]},fields:BTreeMap::from([(reference("staffId"),Some(ScalarLiteral::String("s".into())))]),absent:BTreeSet::new()};
  let assignment=Arc::new(NormalFact{identity:Identity{target:reference("Assignment"),key_id:"pk".into(),components:vec![ScalarLiteral::String("a".into())]},fields:BTreeMap::from([(reference("assignmentId"),Some(ScalarLiteral::String("a".into()))),(reference("assignmentStaff"),Some(ScalarLiteral::String("s".into()))),(reference("assignmentProject"),Some(ScalarLiteral::String("p".into()))),(reference("active"),Some(ScalarLiteral::Boolean(true)))]),absent:BTreeSet::new()});
  let coverage=closure.types.iter().map(|(r,b)|(r.clone(),b.fields.clone())).collect();
  let mut evaluator=Evaluator{catalog:&catalog,closure,subject:NormalFact{identity:row.identity.clone(),fields:row.fields.clone(),absent:BTreeSet::new()},facts:BTreeMap::from([(reference("Assignment"),vec![assignment.clone()])]),coverage,context:BTreeMap::new(),budget:Budget{steps:0,limit:100,payload:PayloadBudget::new("WFT-SECURITY-EVALUATION")}};
  let record=crate::security_association_ref::SecurityAssociationRef::Record(reference("Assignment"));
  let graph=crate::security_association_ref::SecurityAssociationRef::read(&serde_json::json!({"documentId":"domain","moduleId":"m","relationshipId":"Assignment"})).unwrap();
  let variables=BTreeMap::from([(0,reference("Assignment"))]);let rows=BTreeMap::from([(0,assignment.as_ref())]);
  for (association,accepted) in [(record,true),(graph,false)]{
   let expr=Expression::Exists{slot:0,association:association.clone(),condition:Box::new(Expression::Literal(true))};
   let term=Term::Endpoint{binding:Binding::Variable(0),association,role:"staff".into(),target:target.clone(),key_id:"pk".into()};
   if accepted{
    assert!(evaluator.preflight(&expr,&target,&[],&BTreeMap::new()).is_ok());
    assert_eq!(evaluator.expression(&expr,&row,&BTreeMap::new()).unwrap(),Truth::True);
    assert!(evaluator.preflight_term(&term,&target,&[],&variables).is_ok());
    assert_eq!(evaluator.term(&term,&row,&rows).unwrap(),Some(TermValue::Identity(row.identity.clone())));
   }else{
    assert_eq!(evaluator.preflight(&expr,&target,&[],&BTreeMap::new()).unwrap_err().code,"WFT-SECURITY-EVALUATION");
    assert_eq!(evaluator.expression(&expr,&row,&BTreeMap::new()).unwrap_err().code,"WFT-SECURITY-EVALUATION");
    assert_eq!(evaluator.preflight_term(&term,&target,&[],&variables).unwrap_err().code,"WFT-SECURITY-EVALUATION");
    assert_eq!(evaluator.term(&term,&row,&rows).unwrap_err().code,"WFT-SECURITY-EVALUATION");
    evaluator.facts.clear();assert_eq!(evaluator.expression(&expr,&row,&BTreeMap::new()).unwrap_err().code,"WFT-SECURITY-EVALUATION");
   }
  }
 }

 fn plan(policy:&Value)->(Catalog,SecurityLogicalPlan){plan_with_ontology(policy,None)}
 pub(super) fn plan_with_ontology(policy:&Value,ontology:Option<&Value>)->(Catalog,SecurityLogicalPlan){
  let f:Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();let s=&f["resolution"]["documents"][0];let text=s["document"].to_string();
  let inputs=serde_json::from_value(serde_json::json!([{"documentJson":text,"pin":{"documentId":s["document"]["id"],"revision":s["revision"],"umfVersion":"0.8.0","sha256":crate::json::sha256(text.as_bytes())},"selectedModuleIds":["m"]}])).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let packet=crate::security_source::SecuritySourcePacket::read(&policy.to_string(),&ontology.unwrap_or(&f["resolution"]["ontology"]).to_string(),&catalog).unwrap();let plan=SecurityLogicalPlan::read(packet,&catalog).unwrap();(catalog,plan)
 }
 #[test]
 fn portable_project_membership_and_whole_collection_refusal_correspondence(){
  let corpus:Value=serde_json::from_str(include_str!("../tests/security-evaluation-oracle.json")).unwrap();assert_eq!(corpus["cases"].as_array().unwrap().len(),75);
  for case in corpus["cases"].as_array().unwrap(){let (catalog,plan)=plan_with_ontology(&case["policy"],case.get("ontology"));let result=simulate_json(&plan,&catalog,&case["cut"].to_string(),&case["request"].to_string());
   if case["expected"]["status"]=="refused"{assert!(result.is_err(),"{}",case["name"]);}else{let rows=result.unwrap_or_else(|e|panic!("{}: {:?}",case["name"],e));assert_eq!(rows.len(),case["expected"]["rowCount"].as_u64().unwrap() as usize,"{}",case["name"]);assert!(rows.iter().all(|r|r.decision==Decision::Permit));}
  }
 }
 #[test]
 fn unknown_cut_members_and_key_field_mismatch_refuse(){
  let corpus:Value=serde_json::from_str(include_str!("../tests/security-evaluation-oracle.json")).unwrap();let case=&corpus["cases"][0];let (catalog,plan)=plan(&case["policy"]);
  let mut cut=case["cut"].clone();cut["futureMeaning"]=Value::Bool(true);assert!(simulate_json(&plan,&catalog,&cut.to_string(),&case["request"].to_string()).is_err());
  let mut request=case["request"].clone();request["resources"][0]["key"][0]=serde_json::json!({"string":"other"});assert!(simulate_json(&plan,&catalog,&case["cut"].to_string(),&request.to_string()).is_err());
 }
 #[test]
 fn nested_same_type_relation_and_bag_multiplicity_are_preserved(){
  let corpus:Value=serde_json::from_str(include_str!("../tests/security-evaluation-oracle.json")).unwrap();let case=&corpus["cases"][0];let mut policy=case["policy"].clone();let association=serde_json::json!({"documentId":"domain","moduleId":"m","elementId":"Assignment"});
  policy["rules"][1]["condition"]=serde_json::json!({"op":"exists","association":association,"as":"a","where":{"op":"exists","association":association,"as":"b","where":{"op":"eq","left":{"kind":"variable","name":"a","identity":true},"right":{"kind":"variable","name":"b","identity":true}}}});
  let (catalog,plan)=plan(&policy);let mut request=case["request"].clone();let resource=request["resources"][0].clone();request["resources"].as_array_mut().unwrap().push(resource);
  assert_eq!(simulate_json(&plan,&catalog,&case["cut"].to_string(),&request.to_string()).unwrap().len(),2);
  let mut cut=case["cut"].clone();cut["facts"].as_array_mut().unwrap().retain(|f|f["type"]["elementId"]!="Assignment");assert_eq!(simulate_json(&plan,&catalog,&cut.to_string(),&request.to_string()).unwrap().len(),0);
 }
 #[test]
 fn repeated_large_identity_work_refuses_without_returning_an_admitted_prefix(){
  let corpus:Value=serde_json::from_str(include_str!("../tests/security-evaluation-oracle.json")).unwrap();let case=&corpus["cases"][0];let mut policy=case["policy"].clone();
  policy["rules"][1]["condition"]=serde_json::json!({"op":"exists","association":{"documentId":"domain","moduleId":"m","elementId":"Assignment"},"as":"a","where":{"op":"eq","left":{"kind":"variable","name":"a","identity":true},"right":{"kind":"variable","name":"a","identity":true}}});
  let (catalog,plan)=plan(&policy);let mut cut=case["cut"].clone();let token="x".repeat(1_000_000);let assignment=cut["facts"].as_array_mut().unwrap().iter_mut().find(|f|f["type"]["elementId"]=="Assignment").unwrap();assignment["key"][0]=serde_json::json!({"string":token});assignment["fields"].as_array_mut().unwrap().iter_mut().find(|f|f["field"]["elementId"]=="assignmentId").unwrap()["value"]=serde_json::json!({"string":token});
  for count in [1,7,8]{let mut request=case["request"].clone();request["resources"]=Value::Array(vec![request["resources"][0].clone();count]);let result=simulate_json(&plan,&catalog,&cut.to_string(),&request.to_string());if count<=7{assert_eq!(result.unwrap().len(),count);}else{assert!(result.is_err());}}
 }

 // @covers US-008-AC2
 #[test]
 fn transformed_disclosures_share_the_collection_normalization_budget(){
  let corpus:Value=serde_json::from_str(include_str!("../tests/security-evaluation-oracle.json")).unwrap();let case=&corpus["cases"][0];let mut policy=case["policy"].clone();
  let mut output=policy["rules"][0]["disclosure"][0]["field"].clone();output["elementId"]=serde_json::json!("resourceId");
  policy["rules"][0]["disclosure"][0]["disposition"]=serde_json::json!({"kind":"transformed","transform":"constant","version":"0.1.0","field":output,"value":{"string":"x".repeat(1_000_000)}});
  let (catalog,plan)=plan(&policy);
  for count in [1,3,4,17]{let mut request=case["request"].clone();request["resources"]=Value::Array(vec![request["resources"][0].clone();count]);
   let result=simulate_json(&plan,&catalog,&case["cut"].to_string(),&request.to_string());
   if count<=3{let rows=result.unwrap();assert_eq!(rows.len(),count);assert!(rows.iter().all(|r|r.disclosure.iter().any(|(_,d)|matches!(d,crate::security_ir::Disposition::Transformed{literal,..} if literal["string"].as_str().unwrap().len()==1_000_000))));}
   else{assert_eq!(result.unwrap_err().code,"WFT-SECURITY-EVALUATION");}
  }
 }

}

#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct ScopedRows{version:String,rows:Vec<BTreeMap<String,Fact>>}
// Serde structs also accept positional sequences; the selected wire grammar does not.
fn strict_field_value(v:&Value)->bool{v.is_object()&&SecurityRef::read(&v["field"]).is_ok()}
fn strict_fact(v:&Value)->bool{
 v.is_object()&&SecurityRef::read(&v["type"]).is_ok()&&v["key"].is_array()
 &&v["fields"].as_array().is_some_and(|a|a.iter().all(strict_field_value))
 &&v["absent"].as_array().is_some_and(|a|a.iter().all(|r|SecurityRef::read(r).is_ok()))
}
fn strict_cut(v:&Value)->bool{
 v.is_object()&&["subjects","facts"].iter().all(|k|v[*k].as_array().is_some_and(|a|a.iter().all(strict_fact)))
 &&v["context"].as_array().is_some_and(|a|a.iter().all(strict_field_value))
 &&v["coverage"].as_array().is_some_and(|a|a.iter().all(|c|c.is_object()&&SecurityRef::read(&c["type"]).is_ok()&&c["fields"].as_array().is_some_and(|a|a.iter().all(|r|SecurityRef::read(r).is_ok()))))
}
fn strict_scoped_rows(v:&Value)->bool{v.is_object()&&v["rows"].as_array().is_some_and(|a|a.iter().all(|r|r.as_object().is_some_and(|r|r.values().all(strict_fact))))}
/// Evaluate actual owner rule scopes from one caller-declared simulated cut.
/// No issuer authentication, native query execution, or release authority.
pub(crate) fn check_owner_rows(ctx:&crate::security_backend::SecurityBackendContext<'_>,contract:&crate::security_lowering::SecurityResultContract,contract_json:&str,hash:&str,batch_json:&str,cut_json:&str,rows_json:&str)->Result<()> {
 ctx.check_result_cells(contract,contract_json,hash,batch_json)?;
 let plan=ctx.logical_plan();let catalog=ctx.catalog();plan.require_catalog(catalog)?;
 if cut_json.len().checked_add(rows_json.len()).ok_or_else(fail)?>4_000_000{return Err(fail());}
 let parse=|raw:&str|crate::json::checked_json_bounded(raw,4_000_000,64,1_000_000).map_err(|_|fail());
 let cut_value=parse(cut_json)?;let rows_value=parse(rows_json)?;
 if !strict_cut(&cut_value)||!strict_scoped_rows(&rows_value){return Err(fail());}
 let cut=read_cut(cut_value)?;
 let request=read_scoped_rows(rows_value)?;
 let batch=crate::security_result_cells::parse(batch_json)?;let cells=batch["rows"].as_array().ok_or_else(fail)?;
 if request.version!="weft.security.scoped-facts/0.1.0"||request.rows.len()>4096||request.rows.len()!=cells.len(){return Err(fail());}
 let resource_count=request.rows.iter().try_fold(0usize,|n,row|n.checked_add(row.len()).ok_or_else(fail))?;
 if !cut.trusted||cut.generation.is_empty()||cut.generation!=cut.expected_generation||cut.policy_id!=plan.source().policy()["id"]||cut.policy_revision!=plan.source().policy()["revision"]||cut.ontology_document_id!=plan.source().ontology()["documentId"]||cut.ontology_revision!=plan.source().ontology()["revision"]||cut.max_facts==0||cut.max_facts>10000||cut.max_steps==0||cut.max_steps>1_000_000||cut.subjects.len()!=1||cut.subjects.len()+cut.facts.len()+resource_count>cut.max_facts||cut.coverage.len()>512||cut.context.len()>256{return Err(fail());}
 let closure=SecurityOntologyClosure::read(plan.source(),catalog)?;
 let mut budget=Budget{steps:0,limit:cut.max_steps,payload:PayloadBudget::new("WFT-SECURITY-EVALUATION")};
 let subject=fact(cut.subjects.into_iter().next().ok_or_else(fail)?,&closure,catalog,&mut budget)?;if subject.identity.target!=closure.subject{return Err(fail());}
 let mut facts:BTreeMap<SecurityRef,Vec<Arc<NormalFact>>>=BTreeMap::new();let mut identities=BTreeSet::new();for input in cut.facts{let row=fact(input,&closure,catalog,&mut budget)?;budget.copy_identity(&row.identity)?;if !identities.insert(row.identity.clone()){return Err(fail());}facts.entry(row.identity.target.clone()).or_default().push(Arc::new(row));}
 let mut coverage=BTreeMap::new();let mut seen=BTreeSet::new();for entry in cut.coverage{budget.step()?;let b=closure.types.get(&entry.type_ref).ok_or_else(fail)?;let fields:BTreeSet<_>=entry.fields.iter().cloned().collect();if !seen.insert(entry.type_ref.clone())||entry.fields.len()>4096||fields.len()!=entry.fields.len()||!fields.is_subset(&b.fields){return Err(fail());}if entry.complete{coverage.insert(entry.type_ref,fields);}}
 let mut context=BTreeMap::new();for entry in cut.context{if !closure.context.contains(&entry.field)||context.contains_key(&entry.field){return Err(fail());}context.insert(entry.field.clone(),budget.literal(catalog,&entry.field,&entry.value)?);}
 let mut rows=Vec::new();for input in request.rows{
  if input.len()!=ctx.requirements().scans().len(){return Err(fail());}let mut row=BTreeMap::new();
  for (scan,input) in input{budget.step()?;let actual=ctx.requirements().scans().iter().find(|s|s.inventory().scan()==scan).ok_or_else(fail)?;
   if &input.type_ref!=actual.inventory().target(){return Err(fail());}row.insert(scan,fact(input,&closure,catalog,&mut budget)?);
  }rows.push(row);
 }
 let mut evaluator=Evaluator{catalog,closure,subject,facts,coverage,context,budget};
 // Repeated resource identities are permitted for bags/self-joins, with one value assignment.
 let mut coherent:BTreeMap<&Identity,&NormalFact>=BTreeMap::new();
 for value in std::iter::once(&evaluator.subject).chain(evaluator.facts.values().flatten().map(Arc::as_ref)).chain(rows.iter().flat_map(|r|r.values())){
  evaluator.budget.step()?;
  if let Some(prior)=coherent.insert(&value.identity,value){
   evaluator.budget.charge(prior.fields.len()+value.fields.len()+prior.absent.len()+value.absent.len())?;
   for scalar in prior.fields.values().chain(value.fields.values()).flatten(){evaluator.budget.copy_scalar(scalar)?;}
   if prior.fields!=value.fields||prior.absent!=value.absent{return Err(fail());}
  }
 }
 // Eagerly preflight every actual rule, including empty result batches.
 for scan in ctx.requirements().scans(){
  evaluator.complete(scan.inventory().target())?;let resources:Vec<_>=rows.iter().map(|row|row.get(scan.inventory().scan()).ok_or_else(fail)).collect::<Result<_>>()?;
  for action in scan.actions(){for rule in action.rules(){evaluator.preflight(&rule.condition,scan.inventory().target(),&resources,&BTreeMap::new())?;}}
 }
 let mut metadata=crate::security_result_check::Budget::new();
 let mut truths=Vec::new();for row in &rows{
  let mut scoped=crate::security_backend::SecuritySimulatedRowTruths::new();
  for scan in ctx.requirements().scans(){let resource=row.get(scan.inventory().scan()).ok_or_else(fail)?;let mut actions=BTreeMap::new();
   for action in scan.actions(){let mut values=BTreeMap::new();for rule in action.rules(){metadata.text(&rule.id).map_err(|_|fail())?;values.insert(rule.id.clone(),evaluator.expression(&rule.condition,resource,&BTreeMap::new())?);}metadata.text(action.inventory().action()).map_err(|_|fail())?;actions.insert(action.inventory().action().into(),values);}
   metadata.text(scan.inventory().scan()).map_err(|_|fail())?;scoped.insert(scan.inventory().scan().into(),actions);
  }truths.push(scoped);
 }
 ctx.check_simulated_result_selection(contract,contract_json,hash,batch_json,&truths)?;
 for (row,cells) in rows.iter().zip(cells){for (output,cell) in ctx.requirements().outputs().iter().zip(cells.as_array().ok_or_else(fail)?){
  if cell["disposition"]!="original"{continue;}
  let crate::application_ir::Expression::Field{scan,identity}=&output.output().expression else{return Err(fail());};
  let field=SecurityRef{document_id:identity.document_id.clone(),module_id:identity.module.clone(),element_id:identity.element.clone()};
  let resource=row.get(scan).ok_or_else(fail)?;evaluator.budget.step()?;
  if !evaluator.coverage[&resource.identity.target].contains(&field){return Err(fail());}
  let expected=resource.fields.get(&field).ok_or_else(fail)?;let actual=evaluator.budget.literal(catalog,&field,&cell["value"])?;
  if let Some(v)=expected{evaluator.budget.copy_scalar(v)?;}if let Some(v)=&actual{evaluator.budget.copy_scalar(v)?;}
  if expected!=&actual{return Err(fail());}
 }}
 Ok(())
}
#[cfg(test)]
mod faithful_fact_tests {
 use super::*;
 use serde_json::json;
 #[test]
 fn actual_simulation_refuses_raw_value_wrapped_keys_fields_and_context(){
  let corpus=checked_json(include_str!("../tests/security-evaluation-oracle.json")).unwrap();let case=&corpus["cases"][0];
  let source=checked_json(include_str!("../tests/security-source-fixture.json")).unwrap();let mut ontology=source["resolution"]["ontology"].clone();
  let active=json!({"documentId":"domain","moduleId":"m","elementId":"active"});ontology["context"]=json!([active.clone()]);
  let (catalog,plan)=super::tests::plan_with_ontology(&case["policy"],Some(&ontology));
  let mut base_cut=case["cut"].clone();base_cut["context"]=json!([{"field":active,"value":{"boolean":true}}]);let base_request=case["request"].clone();
  assert!(simulate_json(&plan,&catalog,&base_cut.to_string(),&base_request.to_string()).is_ok());
  for position in 0..7 {
   let mut cut=base_cut.clone();let mut request=base_request.clone();
   let member=match position {0=>&mut cut["subjects"][0]["key"][0],1=>&mut cut["subjects"][0]["fields"][0]["value"],2=>&mut cut["facts"][0]["key"][0],3=>&mut cut["facts"][0]["fields"][0]["value"],4=>&mut request["resources"][0]["key"][0],5=>&mut request["resources"][0]["fields"][0]["value"],_=>&mut cut["context"][0]["value"]};
   *member=json!({"$serde_json::private::RawValue":member.to_string()});
   assert!(simulate_json(&plan,&catalog,&cut.to_string(),&request.to_string()).is_err(),"position {position}");
  }
 }
 #[test]
 fn raw_literal_objects_cannot_masquerade_through_typed_fact_consumers(){
  let reference=json!({"documentId":"domain","moduleId":"m","elementId":"field"});
  for key in ["$serde_json::private::RawValue","$serde_json::private::Number"]{
   let raw=format!("{{\"{key}\":\"{{\\\"integerToken\\\":\\\"1\\\"}}\",\"sibling\":true}}");
   let literal=checked_json(&raw).unwrap();
   let f=json!({"type":reference,"key":[literal.clone()],"fields":[{"field":reference,"value":literal.clone()}],"absent":[]});
   let fact=read_fact(f.clone()).unwrap();assert_eq!(fact.key[0],literal);assert_eq!(fact.fields[0].value,literal);
   let request=read_request(json!({"action":"read","resources":[f.clone()],"output":[]})).unwrap();assert_eq!(request.resources[0].fields[0].value,literal);
   let scoped=read_scoped_rows(json!({"version":"weft.security.scoped-facts/0.1.0","rows":[{"r":f.clone()}]})).unwrap();assert_eq!(scoped.rows[0]["r"].key[0],literal);
   let cut=read_cut(json!({"trusted":true,"generation":"one","expectedGeneration":"one","policyId":"policy","policyRevision":"one","ontologyDocumentId":"ontology","ontologyRevision":"one","subjects":[f.clone()],"facts":[f],"coverage":[],"context":[{"field":reference,"value":literal.clone()}],"maxFacts":10,"maxSteps":100})).unwrap();
   assert_eq!(cut.subjects[0].key[0],literal);assert_eq!(cut.facts[0].fields[0].value,literal);assert_eq!(cut.context[0].value,literal);
   let domain=json!({"kind":"field","scalarType":"integer","cardinality":"one","nullability":"required","facets":{"integerWidth":{"bits":64,"signed":true}}});
   assert!(crate::security_literals::normalized_literal(&domain,&json!({"integerToken":"1"})).is_ok());
   assert!(crate::security_literals::normalized_literal(&domain,&literal).is_err());
  }
 }
}
