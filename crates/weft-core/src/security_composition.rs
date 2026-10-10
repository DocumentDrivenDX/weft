//! Pure composition of caller-supplied rule truths against an admitted plan.
//! This never authenticates facts or authorizes native effects/publication.
use crate::{error::{Diagnostic,Result},model::Catalog,security_ir::{SecurityLogicalPlan,Effect,Disposition},security_ontology::{SecurityRef,SecurityOntologyClosure,locate},security_literals::ScalarLiteral,security_budget::PayloadBudget};
use std::collections::{BTreeMap,BTreeSet};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-COMPOSITION","model","Security composition admission refused")}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Truth{True,False,Unknown}
pub fn not(value:Truth)->Truth{match value{Truth::True=>Truth::False,Truth::False=>Truth::True,Truth::Unknown=>Truth::Unknown}}
pub fn and(values:&[Truth])->Result<Truth>{if values.is_empty()||values.len()>64{return Err(fail());}Ok(if values.contains(&Truth::False){Truth::False}else if values.contains(&Truth::Unknown){Truth::Unknown}else{Truth::True})}
pub fn or(values:&[Truth])->Result<Truth>{if values.is_empty()||values.len()>64{return Err(fail());}Ok(if values.contains(&Truth::True){Truth::True}else if values.contains(&Truth::Unknown){Truth::Unknown}else{Truth::False})}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Decision{Permit,Deny,Indeterminate,Conflict}
#[derive(Debug,PartialEq)]
pub struct Composition{pub decision:Decision,pub disclosure:Vec<(SecurityRef,Disposition)>}
fn refuse(decision:Decision)->Composition{Composition{decision,disclosure:vec![]}}
/// All scope-matching truths must be supplied exactly once; extras/missing values refuse.
/// Caller truths are simulation inputs, not admitted authority cuts or credentials.
pub fn compose(plan:&SecurityLogicalPlan,catalog:&Catalog,target:&SecurityRef,action:&str,output_fields:&[SecurityRef],truths:&BTreeMap<String,Truth>)->Result<Composition>{
 compose_with_budget(plan,catalog,target,action,output_fields,truths,&mut PayloadBudget::new("WFT-SECURITY-COMPOSITION"))
}
pub(crate) fn compose_with_budget(plan:&SecurityLogicalPlan,catalog:&Catalog,target:&SecurityRef,action:&str,output_fields:&[SecurityRef],truths:&BTreeMap<String,Truth>,budget:&mut PayloadBudget)->Result<Composition>{
 plan.require_catalog(catalog)?;
 let ontology=SecurityOntologyClosure::read(plan.source(),catalog)?;
 let binding=ontology.types.get(target).ok_or_else(fail)?;
 if output_fields.len()>4096||output_fields.iter().collect::<BTreeSet<_>>().len()!=output_fields.len()||output_fields.iter().any(|f|!binding.fields.contains(f)){return Err(fail());}
 if !plan.source().ontology()["actions"].as_array().ok_or_else(fail)?.iter().any(|a|a==action){return Err(fail());}
 let rules:Vec<_>=plan.rules().iter().filter(|r|&r.target==target&&r.actions.iter().any(|a|a==action)).map(|r|CompositionRule{id:r.id.clone(),effect:r.effect.clone(),disclosure:&r.disclosure}).collect();
 compose_rules(binding,&rules,catalog,output_fields,truths,budget)
}
/// A common pure fold over already source-checked rule metadata; conditions are
/// evaluated elsewhere and supplied truths remain simulation inputs.
struct CompositionRule<'a>{id:String,effect:Effect,disclosure:&'a [(SecurityRef,Disposition)]}
fn compose_rules(binding:&crate::security_ontology::TypeBinding,rules:&[CompositionRule<'_>],catalog:&Catalog,output_fields:&[SecurityRef],truths:&BTreeMap<String,Truth>,budget:&mut PayloadBudget)->Result<Composition>{
 let ids:BTreeSet<_>=rules.iter().map(|r|r.id.clone()).collect();if ids!=truths.keys().cloned().collect(){return Err(fail());}
 if truths.values().any(|t|*t==Truth::Unknown){return Ok(refuse(Decision::Indeterminate));}
 if !rules.iter().any(|r|r.effect==Effect::Permit&&truths[&r.id]==Truth::True)||rules.iter().any(|r|r.effect==Effect::Require&&truths[&r.id]!=Truth::True||r.effect==Effect::Forbid&&truths[&r.id]==Truth::True){return Ok(refuse(Decision::Deny));}
 let mut obligations:BTreeMap<SecurityRef,Vec<&Disposition>>=BTreeMap::new();
 for r in rules.iter().filter(|r|r.effect==Effect::Permit&&truths[&r.id]==Truth::True){for (field,d) in r.disclosure{obligations.entry(field.clone()).or_default().push(d);}}
 let mut disclosure=Vec::new();
 for reference in output_fields{
  let field=binding.source["fields"].as_array().ok_or_else(fail)?.iter().find(|f|SecurityRef::read(&f["ref"]).as_ref().is_ok_and(|r|r==reference)).ok_or_else(fail)?;let values=obligations.get(reference).map(Vec::as_slice).unwrap_or(&[]);
  if values.is_empty()&&field["protection"]=="protected"{return Ok(refuse(Decision::Indeterminate));}
  let disposition=if values.iter().any(|d|matches!(d,Disposition::Withheld)){Disposition::Withheld}else{
   let mut selected:Option<&Disposition>=None;let mut identity:Option<(String,String,SecurityRef,Option<ScalarLiteral>)>=None;
   for d in values{if let Disposition::Transformed{transform,version,output_field,literal,..}=d{
    let current=(transform.clone(),version.clone(),output_field.clone(),budget.literal(locate(catalog,output_field)?,literal)?);
    if identity.as_ref().is_some_and(|id|id!=&current){return Ok(refuse(Decision::Conflict));}
    if selected.is_none(){selected=Some(d);identity=Some(current);}
   }}
   if let Some(selected)=selected{budget.copy_json(selected)?;selected.clone()}else{Disposition::Original}
  };
  disclosure.push((reference.clone(),disposition));
 }
 Ok(Composition{decision:Decision::Permit,disclosure})
}

/// Separate draft composition. Never admits native authority or converts a
/// draft graph plan into the original Record-only admitted plan.
pub fn compose_candidate(plan:&crate::security_candidate_ir::SecurityCandidateLogicalPlan,catalog:&Catalog,target:&SecurityRef,action:&str,output_fields:&[SecurityRef],truths:&BTreeMap<String,Truth>)->Result<Composition>{
 plan.require_catalog(catalog)?;let ontology=crate::security_candidate_ontology::SecurityCandidateOntologyClosure::read(plan.source(),catalog)?;let binding=ontology.types.get(target).ok_or_else(fail)?;
 if output_fields.len()>4096||output_fields.iter().collect::<BTreeSet<_>>().len()!=output_fields.len()||output_fields.iter().any(|f|!binding.fields.contains(f)){return Err(fail());}
 if !plan.source().ontology()["actions"].as_array().ok_or_else(fail)?.iter().any(|a|a==action){return Err(fail());}
 let rules:Vec<_>=plan.rules().iter().filter(|r|&r.target==target&&r.actions.iter().any(|a|a==action)).map(|r|CompositionRule{id:r.id.clone(),effect:r.effect.clone(),disclosure:&r.disclosure}).collect();
 compose_rules(binding,&rules,catalog,output_fields,truths,&mut PayloadBudget::new("WFT-SECURITY-COMPOSITION"))
}

#[cfg(test)]
mod tests{
 use super::*;

 fn fields()->[SecurityRef;1]{[SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"salary".into()}]}
 fn truth(v:&serde_json::Value)->Truth{match v.as_str().unwrap(){"T"=>Truth::True,"F"=>Truth::False,"U"=>Truth::Unknown,_=>panic!()}}
 // @covers US-008-AC2
 #[test]
 fn composition_charges_retained_disclosure_at_copy_boundary(){
  let f:serde_json::Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();let mut policy=f["policy"].clone();let field=policy["rules"][0]["disclosure"][0]["field"].clone();
  policy["rules"][0]["disclosure"][0]["disposition"]=serde_json::json!({"kind":"transformed","transform":"constant","version":"0.1.0","field":field,"value":{"integerToken":"1"}});
  let (catalog,plan)=fixture(policy);let target=&plan.rules()[0].target;let truths=BTreeMap::from([("reader".into(),Truth::True),("membership".into(),Truth::True)]);
  let baseline=compose(&plan,&catalog,target,"read",&fields(),&truths).unwrap();
  let cost=serde_json::to_vec(&baseline.disclosure[0].1).unwrap().len();
  for extra in [0,1]{
   let mut budget=PayloadBudget::new("WFT-SECURITY-EVALUATION");
   budget.copy_json(&serde_json::Value::String("x".repeat(crate::security_budget::COPY_LIMIT-cost-2+extra))).unwrap();
   let result=compose_with_budget(&plan,&catalog,target,"read",&fields(),&truths,&mut budget);
   if extra==0{assert_eq!(result.unwrap(),baseline);}else{assert_eq!(result.unwrap_err().code,"WFT-SECURITY-EVALUATION");}
  }
 }
 #[test]
 fn portable_typescript_truth_corpus_correspondence(){
  let corpus:serde_json::Value=serde_json::from_str(include_str!("../tests/security-composition-oracle.json")).unwrap();
  assert_eq!(corpus["truthVectors"].as_array().unwrap().len(),120);
  for v in corpus["truthVectors"].as_array().unwrap(){let args:Vec<_>=v["args"].as_array().unwrap().iter().map(truth).collect();assert_eq!(and(&args).unwrap(),truth(&v["and"]));assert_eq!(or(&args).unwrap(),truth(&v["or"]));for (value,expected) in args.iter().zip(v["not"].as_array().unwrap()){assert_eq!(not(*value),truth(expected));}}
 }
 #[test]
 fn complete_strong_kleene_tables_and_bounds(){
  use Truth::*;let values=[True,False,Unknown];
  let and_table=[[True,False,Unknown],[False,False,False],[Unknown,False,Unknown]];
  let or_table=[[True,True,True],[True,False,Unknown],[True,Unknown,Unknown]];
  for (i,a) in values.iter().enumerate(){for (j,b) in values.iter().enumerate(){assert_eq!(and(&[*a,*b]).unwrap(),and_table[i][j]);assert_eq!(or(&[*a,*b]).unwrap(),or_table[i][j]);assert_eq!(not(and(&[*a,*b]).unwrap()),or(&[not(*a),not(*b)]).unwrap());}}
  assert_eq!(not(Unknown),Unknown);assert!(and(&[]).is_err());assert!(or(&[True;65]).is_err());
 }
 fn fixture(policy:serde_json::Value)->(Catalog,SecurityLogicalPlan){
  let f:serde_json::Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();let s=&f["resolution"]["documents"][0];let text=s["document"].to_string();
  let inputs=serde_json::from_value(serde_json::json!([{"documentJson":text,"pin":{"documentId":s["document"]["id"],"revision":s["revision"],"umfVersion":"0.8.0","sha256":crate::json::sha256(text.as_bytes())},"selectedModuleIds":["m"]}])).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let packet=crate::security_source::SecuritySourcePacket::read(&policy.to_string(),&f["resolution"]["ontology"].to_string(),&catalog).unwrap();let plan=SecurityLogicalPlan::read(packet,&catalog).unwrap();(catalog,plan)
 }
 #[test]
 fn all_permit_require_forbid_truths_and_rule_order(){
  let f:serde_json::Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();let mut p=f["policy"].clone();let mut forbid=p["rules"][1].clone();forbid["id"]=serde_json::json!("forbid");forbid["effect"]=serde_json::json!("forbid");p["rules"].as_array_mut().unwrap().push(forbid);
  let (catalog,plan)=fixture(p.clone());p["rules"].as_array_mut().unwrap().reverse();let (reverse_catalog,reverse_plan)=fixture(p);
  let target=&plan.rules()[0].target;
  for permit in [Truth::True,Truth::False,Truth::Unknown]{for require in [Truth::True,Truth::False,Truth::Unknown]{for forbid in [Truth::True,Truth::False,Truth::Unknown]{
   let truths=BTreeMap::from([("reader".into(),permit),("membership".into(),require),("forbid".into(),forbid)]);
   let result=compose(&plan,&catalog,target,"read",&fields(),&truths).unwrap();
   let corpus:serde_json::Value=serde_json::from_str(include_str!("../tests/security-composition-oracle.json")).unwrap();let vector=corpus["decisions"].as_array().unwrap().iter().find(|v|truth(&v["permit"])==permit&&truth(&v["require"])==require&&truth(&v["forbid"])==forbid).unwrap();let observed=match result.decision{Decision::Permit=>"permit",Decision::Deny=>"deny",Decision::Indeterminate=>"indeterminate",Decision::Conflict=>"conflict"};assert_eq!(vector["decision"],observed);
   let expected=if [permit,require,forbid].contains(&Truth::Unknown){Decision::Indeterminate}else if permit==Truth::True&&require==Truth::True&&forbid==Truth::False{Decision::Permit}else{Decision::Deny};assert_eq!(result.decision,expected);assert_eq!(result,compose(&reverse_plan,&reverse_catalog,target,"read",&fields(),&truths).unwrap());if expected!=Decision::Permit{assert!(result.disclosure.is_empty());}
  }}}
  assert!(compose(&plan,&catalog,target,"read",&fields(),&BTreeMap::new()).is_err());
 }
 #[test]
 fn missing_protected_disposition_false_permit_and_mask_conflict(){
  let f:serde_json::Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();let mut p=f["policy"].clone();
  p["rules"][0].as_object_mut().unwrap().remove("disclosure");let (c,plan)=fixture(p);let truths=BTreeMap::from([("reader".into(),Truth::True),("membership".into(),Truth::True)]);assert_eq!(compose(&plan,&c,&plan.rules()[0].target,"read",&fields(),&truths).unwrap().decision,Decision::Indeterminate);
  let key=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"resourceId".into()};
  let visible=compose(&plan,&c,&plan.rules()[0].target,"read",std::slice::from_ref(&key),&truths).unwrap();assert_eq!(visible.decision,Decision::Permit);assert_eq!(visible.disclosure,vec![(key.clone(),Disposition::Original)]);
  assert!(compose(&plan,&c,&plan.rules()[0].target,"read",&[key.clone(),key],&truths).is_err());
  let mut p=f["policy"].clone();let field=p["rules"][0]["disclosure"][0]["field"].clone();
  for (i,value) in ["x","y"].iter().enumerate(){let mut r=p["rules"][0].clone();r["id"]=serde_json::json!(format!("transform{i}"));let mut output=field.clone();output["elementId"]=serde_json::json!("resourceId");r["disclosure"][0]["disposition"]=serde_json::json!({"kind":"transformed","transform":"constant","version":"0.1.0","field":output,"value":{"string":value}});p["rules"].as_array_mut().unwrap().push(r);}
  let (c,plan)=fixture(p);let mut truths=truths;truths.insert("transform0".into(),Truth::True);truths.insert("transform1".into(),Truth::True);
  let target=&plan.rules()[0].target;assert_eq!(compose(&plan,&c,target,"read",&fields(),&truths).unwrap().decision,Decision::Permit); // Withheld dominates conflicting transforms.
  truths.insert("reader".into(),Truth::False);assert_eq!(compose(&plan,&c,target,"read",&fields(),&truths).unwrap().decision,Decision::Conflict);
  truths.insert("transform1".into(),Truth::False);let result=compose(&plan,&c,target,"read",&fields(),&truths).unwrap();assert_eq!(result.decision,Decision::Permit);assert!(result.disclosure.iter().any(|(_,d)|matches!(d,Disposition::Transformed{..})));
 }
 #[test]
 fn transform_identity_uses_exact_value_and_qualified_output_domain(){
  let f:serde_json::Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();
  for (left,right,expected) in [("9007199254740993","9007199254740993.0e0",Decision::Permit),("-0","0",Decision::Permit),("9007199254740992","9007199254740993",Decision::Conflict)]{
   let mut p=f["policy"].clone();let field=p["rules"][0]["disclosure"][0]["field"].clone();
   p["rules"][0]["disclosure"][0]["disposition"]=serde_json::json!({"kind":"transformed","transform":"constant","version":"0.1.0","field":field,"value":{"integerToken":left}});
   let mut second=p["rules"][0].clone();second["id"]=serde_json::json!("second");second["disclosure"][0]["disposition"]["value"]=serde_json::json!({"integerToken":right});p["rules"].as_array_mut().unwrap().push(second);
   let (catalog,plan)=fixture(p);let truths=BTreeMap::from([("reader".into(),Truth::True),("membership".into(),Truth::True),("second".into(),Truth::True)]);assert_eq!(compose(&plan,&catalog,&plan.rules()[0].target,"read",&fields(),&truths).unwrap().decision,expected);
  }
 }
 #[test]
 fn bounded_disposition_oracle_correspondence(){
  let oracle:serde_json::Value=serde_json::from_str(include_str!("../tests/security-disposition-oracle.json")).unwrap();
  let f:serde_json::Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();
  assert_eq!(oracle["cases"].as_array().unwrap().len(),432);
  let mut seen=BTreeSet::new();
  for vector in oracle["cases"].as_array().unwrap(){
   let slots=vector["slots"].as_array().unwrap();assert_eq!(slots.len(),3);let protected=vector["protected"].as_bool().unwrap();
   assert!(seen.insert((slots.iter().map(|s|s.as_u64().unwrap()).collect::<Vec<_>>(),protected)));
   let mut policy=f["policy"].clone();let template=policy["rules"][0].clone();policy["rules"]=serde_json::json!([policy["rules"][1].clone()]);
   let mut reference=template["disclosure"][0]["field"].clone();if !protected{reference["elementId"]=serde_json::json!("resourceId");}
   let mut output=reference.clone();output["elementId"]=serde_json::json!("resourceId");
   let mut truths=BTreeMap::from([("membership".into(),Truth::True)]);
   for (index,slot) in slots.iter().enumerate(){
    let slot=slot.as_u64().unwrap();let mut rule=template.clone();let id=format!("permit{index}");rule["id"]=serde_json::json!(id);truths.insert(id,Truth::True);
    rule.as_object_mut().unwrap().remove("disclosure");
    if slot!=0{
     let disposition=match slot{1=>serde_json::json!({"kind":"original"}),2=>serde_json::json!({"kind":"withheld"}),3..=5=>serde_json::json!({"kind":"transformed","transform":"constant","version":"0.1.0","field":output,"value":{"string":format!("identity{slot}")}}),_=>panic!("invalid oracle")};
     rule["disclosure"]=serde_json::json!([{"field":reference,"disposition":disposition}]);
    }
    policy["rules"].as_array_mut().unwrap().push(rule);
   }
   let selected=SecurityRef::read(&reference).unwrap();let (catalog,plan)=fixture(policy.clone());
   let result=compose(&plan,&catalog,&plan.rules()[0].target,"read",std::slice::from_ref(&selected),&truths).unwrap();
   let observed=match result.decision{Decision::Permit=>"permit",Decision::Deny=>"deny",Decision::Indeterminate=>"indeterminate",Decision::Conflict=>"conflict"};assert_eq!(vector["decision"],observed,"{vector}");
   if result.decision!=Decision::Permit{assert!(result.disclosure.is_empty());}else{
    assert_eq!(result.disclosure.len(),1);assert_eq!(result.disclosure[0].0,selected);
    let kind=match &result.disclosure[0].1{Disposition::Original=>1,Disposition::Withheld=>2,Disposition::Transformed{literal,..}=>literal["string"].as_str().unwrap().strip_prefix("identity").unwrap().parse::<u64>().unwrap()};assert_eq!(vector["disposition"],kind,"{vector}");
   }
   policy["rules"].as_array_mut().unwrap().reverse();let (reversed_catalog,reversed_plan)=fixture(policy);
   assert_eq!(result,compose(&reversed_plan,&reversed_catalog,&reversed_plan.rules()[0].target,"read",std::slice::from_ref(&selected),&truths).unwrap());
  }
 }

}
