//! Private semantic declaration matching. No binding, native or dispatch admission.
use crate::{backend::Status,error::{Diagnostic,Result},security_backend::{SecurityBackendContext,SecurityManifest},security_ir::{Expression,Term},security_ontology::SecurityRef};
use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-LOWERING-UNSUPPORTED","capability","Selected semantic coverage refused")}
struct Budget{work:usize,text:usize}
impl Budget{
 fn step(&mut self)->Result<()>{self.work=self.work.checked_sub(1).ok_or_else(fail)?;Ok(())}
 fn text(&mut self,s:&str)->Result<()>{self.step()?;self.text=self.text.checked_sub(s.len()).ok_or_else(fail)?;Ok(())}
 fn value(&mut self,v:&Value,depth:usize)->Result<()>{self.step()?;if depth>64{return Err(fail());}match v{
  Value::String(s)=>self.text(s)?,Value::Number(n)=>self.text(n.as_str())?,Value::Array(a)=>{for v in a{self.value(v,depth+1)?;}},Value::Object(o)=>{for (k,v) in o{self.text(k)?;self.value(v,depth+1)?;}},_=>{}}Ok(())}
}
fn object(v:&Value,keys:&[&str])->Result<()>{let o=v.as_object().ok_or_else(fail)?;if o.len()!=keys.len()||keys.iter().any(|k|!o.contains_key(*k)){return Err(fail());}Ok(())}
#[derive(Debug,PartialEq,Eq,PartialOrd,Ord)]
struct Target{reference:SecurityRef,revision:String}
#[derive(Debug)]
struct Domain{targets:BTreeSet<Target>,scans:usize,rules:usize,nodes:usize,depth:usize,exists:usize,outputs:usize}
fn bound(v:&Value,key:&str,max:u64)->Result<usize>{let n=v[key].as_u64().ok_or_else(fail)?;if n==0||n>max{return Err(fail());}Ok(n as usize)}
fn identity(v:&Value)->Result<String>{let s=v.as_str().ok_or_else(fail)?;if s.is_empty()||s.contains('\0')||s.len()>4096{return Err(fail());}Ok(s.into())}
#[cfg(test)]
fn parse(logical:&Value,result:&Value)->Result<Domain>{parse_budget(logical,result,&mut Budget{work:1_000_000,text:16_000_000})}
fn parse_budget(logical:&Value,result:&Value,b:&mut Budget)->Result<Domain>{
 b.value(logical,0)?;b.value(result,0)?;
 object(logical,&["version","policyProfile","applicationProfile","targets","limits"])?;
 if logical["version"]!="weft.security.logical-domain/0.1.0"||logical["policyProfile"]!="weft.security.owner-record-semantics/0.1.0"||logical["applicationProfile"]!="weft.security.owner-relational-plan/0.1.0"{return Err(fail());}
 object(result,&["version","profile","cellEncoding"])?;
 if result["version"]!="weft.security.result-domain/0.1.0"||result["profile"]!="weft.security.required-direct-cells/0.1.0"||result["cellEncoding"]!="weft.security.cells/0.1.0"{return Err(fail());}
 let entries=logical["targets"].as_array().ok_or_else(fail)?;if entries.is_empty()||entries.len()>4096{return Err(fail());}
 let mut targets=BTreeSet::new();for entry in entries{
  b.step()?;object(entry,&["documentId","revision","moduleId","elementId"])?;
  let target=Target{reference:SecurityRef{document_id:identity(&entry["documentId"])?,module_id:identity(&entry["moduleId"])?,element_id:identity(&entry["elementId"])?},revision:identity(&entry["revision"])?};
  if !targets.insert(target){return Err(fail());}
 }
 let l=&logical["limits"];object(l,&["maxScans","maxRulesPerAction","maxConditionNodes","maxConditionDepth","maxExistsDepth","maxOutputs"])?;
 Ok(Domain{targets,scans:bound(l,"maxScans",256)?,rules:bound(l,"maxRulesPerAction",4096)?,nodes:bound(l,"maxConditionNodes",1_000_000)?,depth:bound(l,"maxConditionDepth",64)?,exists:bound(l,"maxExistsDepth",64)?,outputs:bound(l,"maxOutputs",4096)?})
}
struct Measure{nodes:usize,depth:usize,exists:usize}
impl Measure{
 fn node(&mut self,b:&mut Budget)->Result<()>{b.step()?;self.nodes=self.nodes.checked_add(1).ok_or_else(fail)?;Ok(())}
 fn term(&mut self,_term:&Term,b:&mut Budget)->Result<()>{self.node(b)}
 fn expression(&mut self,e:&Expression,depth:usize,exists:usize,b:&mut Budget)->Result<()>{
  self.node(b)?;if depth>64||exists>64{return Err(fail());}self.depth=self.depth.max(depth);self.exists=self.exists.max(exists);
  match e{Expression::Literal(_)=>{},Expression::Equal(a,c)=>{self.term(a,b)?;self.term(c,b)?;},Expression::And(args)|Expression::Or(args)=>{for a in args{self.expression(a,depth+1,exists,b)?;}},Expression::Not(a)=>self.expression(a,depth+1,exists,b)?,Expression::Exists{association,condition,..}=>{if association.record().is_none(){return Err(fail());}self.exists=self.exists.max(exists+1);self.expression(condition,depth+1,exists+1,b)?;}}
  Ok(())
 }
}
struct OwnerIndex<'a>{revisions:BTreeMap<String,String>,elements:BTreeMap<SecurityRef,&'a Value>}
impl<'a> OwnerIndex<'a>{
 fn build(ctx:&'a SecurityBackendContext<'_>,b:&mut Budget)->Result<Self>{
  let mut revisions=BTreeMap::new();let mut elements=BTreeMap::new();
  for input in ctx.catalog().inputs(){b.text(&input.document_json)?;b.text(&input.pin.document_id)?;b.text(&input.pin.revision)?;if revisions.insert(input.pin.document_id.clone(),input.pin.revision.clone()).is_some(){return Err(fail());}}
  for (doc,input) in ctx.catalog().documents.iter().zip(ctx.catalog().inputs()){b.step()?;let document=doc["id"].as_str().ok_or_else(fail)?;
   let mut selected=BTreeSet::new();for id in &input.selected_module_ids{b.text(id)?;selected.insert(id.as_str());}
   for module in doc["modules"].as_array().ok_or_else(fail)?{b.step()?;let mid=module["id"].as_str().ok_or_else(fail)?;b.text(mid)?;if !selected.contains(mid){continue;}
    for element in module["elements"].as_array().ok_or_else(fail)?{b.step()?;let eid=element["id"].as_str().ok_or_else(fail)?;b.text(document)?;b.text(mid)?;b.text(eid)?;
     let key=SecurityRef{document_id:document.into(),module_id:mid.into(),element_id:eid.into()};if elements.insert(key,element).is_some(){return Err(fail());}
    }
   }
  }Ok(Self{revisions,elements})
 }
 fn includes(&self,d:&Domain,r:&SecurityRef,b:&mut Budget)->Result<bool>{for t in &d.targets{b.step()?;if &t.reference==r&&self.revisions.get(&r.document_id)==Some(&t.revision){return Ok(true);}}Ok(false)}
}
/// Each scope requires a complete capability, never a union of fragments.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CoverageScope { ScanAction { scan: String, action: String }, Application }
type Assignments = BTreeMap<CoverageScope, BTreeSet<String>>;
/// Actual immutable registration and context custody; no public constructor.
#[allow(dead_code)]
pub(crate) struct OwnerCoverage<'m, 'c, 's> {
 declaration: crate::security_backend::RegisteredSecurityDeclaration<'m>,
 context: &'c SecurityBackendContext<'s>,
 assignments: Assignments,
 selected: BTreeSet<String>,
}
#[allow(dead_code)]
impl OwnerCoverage<'_, '_, '_> {
 pub(crate) fn assignments(&self)->&Assignments { &self.assignments }
 pub(crate) fn selected_capabilities(&self)->&BTreeSet<String> { &self.selected }
 pub(crate) fn context(&self)->&SecurityBackendContext<'_> { self.context }
 pub(crate) fn declaration(&self)->&crate::security_backend::RegisteredSecurityDeclaration<'_> { &self.declaration }
}
#[allow(dead_code)]
pub(crate) fn issue<'m,'c,'s>(ctx:&'c SecurityBackendContext<'s>,declaration:crate::security_backend::RegisteredSecurityDeclaration<'m>,ids:&[String],allow:bool)->Result<OwnerCoverage<'m,'c,'s>> {
 let mut b=Budget{work:1_000_000,text:16_000_000};
 b.text(declaration.manifest_json())?;
 if declaration.target().id!=ctx.target_profile(){return Err(fail());}
 let assignments=assign_with_budget(ctx,declaration.manifest(),ids,allow,&mut b)?;
 let mut selected=BTreeSet::new();for id in ids{b.text(id)?;selected.insert(id.clone());}
 Ok(OwnerCoverage{declaration,context:ctx,assignments,selected})
}
fn retain(assignments:&mut Assignments,scope:CoverageScope,candidates:BTreeSet<String>,edges:&mut usize,b:&mut Budget)->Result<()> {
 b.step()?;if candidates.is_empty()||assignments.len()>=4096{return Err(fail());}
 *edges=edges.checked_add(candidates.len()).ok_or_else(fail)?;
 if *edges>4096||assignments.insert(scope,candidates).is_some(){return Err(fail());}Ok(())
}
/// Only correspondence with actual owner objects; no dispatch or native admission.
#[allow(dead_code)]
pub(crate) fn check_selected(ctx:&SecurityBackendContext<'_>,m:&SecurityManifest,ids:&[String],allow:bool)->Result<()>{check_with_budget(ctx,m,ids,allow,&mut Budget{work:1_000_000,text:16_000_000})}
fn check_with_budget(ctx:&SecurityBackendContext<'_>,m:&SecurityManifest,ids:&[String],allow:bool,b:&mut Budget)->Result<()>{assign_with_budget(ctx,m,ids,allow,b).map(|_|())}
fn assign_with_budget(ctx:&SecurityBackendContext<'_>,m:&SecurityManifest,ids:&[String],allow:bool,b:&mut Budget)->Result<Assignments>{
 b.step()?;
 if m.interface_version!="weft-security-backend/0.1.0"||m.source_profiles.len()!=1||m.backend_id!=ctx.backend_id()||m.backend_version!=ctx.backend_version()||ids.is_empty()||ids.len()>4096||m.capabilities.len()>4096||m.target_profiles.len()>256{return Err(fail());}
 let p=&m.source_profiles[0];for s in [&p.dialect,&p.application_ir,&p.policy,&p.ontology,&p.security_ir]{b.text(s)?;}
 if (&*p.dialect,&*p.application_ir,&*p.policy,&*p.ontology,&*p.security_ir)!=("weft-sql/0.2.0","weft-ir/0.2.0","0.1.0","0.1.0","weft.security.logical-ir/0.1.0"){return Err(fail());}
 let mut target=false;for t in &m.target_profiles{b.text(&t.id)?;if t.id==ctx.target_profile(){target=true;}}if !target{return Err(fail());}
 b.text(ctx.logical_plan().source().policy_json())?;b.text(ctx.logical_plan().source().ontology_json())?;
 let index=OwnerIndex::build(ctx,b)?;let mut caps=BTreeMap::new();for c in &m.capabilities{b.text(&c.id)?;if caps.insert(c.id.as_str(),c).is_some(){return Err(fail());}}
 let mut seen=BTreeSet::new();let mut domains=Vec::new();
 for id in ids{b.text(id)?;if !seen.insert(id){return Err(fail());}let c=caps.get(id.as_str()).ok_or_else(fail)?;
  let mut applicable=false;for t in &c.target_profiles{b.text(t)?;if t==ctx.target_profile(){applicable=true;}}
  if !applicable||c.language_profiles.len()!=1||c.language_profiles[0].dialect_profile!="weft-sql/0.2.0"||c.language_profiles[0].ir_version!="weft-ir/0.2.0"||!c.constraints.is_empty()||c.status==Status::Unsupported||(c.status==Status::Candidate&&!allow){return Err(fail());}
  let d=parse_budget(&c.logical_domain,&c.result_domain,b)?;
  for t in &d.targets{b.step()?;if index.revisions.get(&t.reference.document_id)!=Some(&t.revision)||index.elements.get(&t.reference).is_none_or(|e|e["kind"]!="record"){return Err(fail());}}
  domains.push((c.id.as_str(),d));
 }
 // Fixed profiles are compatible by exact parser identity, not feature union.
 let mut assignments=BTreeMap::new();let mut edges=0;
 let req=ctx.requirements();for scan in req.scans(){b.step()?;for action in scan.actions(){b.step()?;
  let mut measure=Measure{nodes:0,depth:0,exists:0};for rule in action.rules(){b.step()?;measure.expression(&rule.condition,1,0,b)?;for _ in &rule.disclosure{b.step()?;}}
  let mut candidates=BTreeSet::new();for (id,d) in &domains{b.step()?;if index.includes(d,scan.inventory().target(),b)?&&action.rules().len()<=d.rules&&measure.nodes<=d.nodes&&measure.depth<=d.depth&&measure.exists<=d.exists{b.text(id)?;candidates.insert((*id).to_string());}}
  b.text(scan.inventory().scan())?;b.text(action.inventory().action())?;
  retain(&mut assignments,CoverageScope::ScanAction{scan:scan.inventory().scan().to_string(),action:action.inventory().action().to_string()},candidates,&mut edges,b)?;
 }}
 // Whole relational plan retains all occurrences and field-free computations.
 let app=ctx.query().application_plan();for output in req.outputs(){b.step()?;
  let crate::application_ir::Expression::Field{identity,..}=&output.output().expression else{return Err(fail());};
  b.text(&identity.document_id)?;b.text(&identity.revision)?;b.text(&identity.module)?;b.text(&identity.element)?;
  let r=SecurityRef{document_id:identity.document_id.clone(),module_id:identity.module.clone(),element_id:identity.element.clone()};let e=index.elements.get(&r).ok_or_else(fail)?;b.value(e,0)?;
  if index.revisions.get(&identity.document_id)!=Some(&identity.revision)||crate::security_ontology::domain(e).map_err(|_|fail())?["nullability"]!="required"{return Err(fail());}
 }
 let mut candidates=BTreeSet::new();for (id,d) in &domains{b.step()?;if req.scans().len()>d.scans||app.outputs.len()>d.outputs{continue;}let mut all=true;for s in req.scans(){b.step()?;if !index.includes(d,s.inventory().target(),b)?{all=false;break;}}if all{b.text(id)?;candidates.insert((*id).to_string());}}
 retain(&mut assignments,CoverageScope::Application,candidates,&mut edges,b)?;Ok(assignments)
}
#[cfg(test)]
mod tests{
 use super::*;use serde_json::json;
 fn logical()->Value{json!({"version":"weft.security.logical-domain/0.1.0","policyProfile":"weft.security.owner-record-semantics/0.1.0","applicationProfile":"weft.security.owner-relational-plan/0.1.0","targets":[{"documentId":"domain","revision":"schema-1","moduleId":"m","elementId":"Resource"}],"limits":{"maxScans":256,"maxRulesPerAction":4096,"maxConditionNodes":1000000,"maxConditionDepth":64,"maxExistsDepth":64,"maxOutputs":4096}})}
 fn result()->Value{json!({"version":"weft.security.result-domain/0.1.0","profile":"weft.security.required-direct-cells/0.1.0","cellEncoding":"weft.security.cells/0.1.0"})}
 #[test]
 fn closed_domain_grammar_refuses_unknown_positional_duplicate_and_numeric_meaning(){
  let l=logical();let r=result();assert!(parse(&l,&r).is_ok());
  for key in ["version","policyProfile","applicationProfile"]{let mut v=l.clone();v[key]=json!("unknown");assert!(parse(&v,&r).is_err());}
  for key in ["version","profile","cellEncoding"]{let mut v=r.clone();v[key]=json!("unknown");assert!(parse(&l,&v).is_err());}
  for key in ["maxScans","maxRulesPerAction","maxConditionNodes","maxConditionDepth","maxExistsDepth","maxOutputs"]{for value in [json!(0),json!(-1),json!(1.5),json!(1000001),json!("1"),Value::Null]{let mut v=l.clone();v["limits"][key]=value;assert!(parse(&v,&r).is_err());}}
  for mut v in [l.clone(),l.clone()]{v["targets"].as_array_mut().unwrap().push(l["targets"][0].clone());assert!(parse(&v,&r).is_err());}
  let mut v=l.clone();v["unknown"]=json!(true);assert!(parse(&v,&r).is_err());let mut v=r.clone();v["unknown"]=json!(true);assert!(parse(&l,&v).is_err());
  let mut v=l.clone();v["targets"][0]=json!(["domain","schema-1","m","Resource"]);assert!(parse(&v,&r).is_err());
  let mut v=l.clone();v["limits"]=json!([256,4096,1000000,64,64,4096]);assert!(parse(&v,&r).is_err());
 }
 fn with_context(sql:&str,run:impl FnOnce(&SecurityBackendContext<'_>,SecurityManifest)){with_mutated_context(sql,|_|{},run)}
 fn with_mutated_context(sql:&str,mutate:impl FnOnce(&mut Value),run:impl FnOnce(&SecurityBackendContext<'_>,SecurityManifest)){with_bound_context(sql,mutate,json!([]),run)}
 fn with_bound_context(sql:&str,mutate:impl FnOnce(&mut Value),bindings:Value,run:impl FnOnce(&SecurityBackendContext<'_>,SecurityManifest)){
  let mut f:Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();mutate(&mut f);let source=&f["resolution"]["documents"][0];let mut doc=source["document"].clone();for e in doc["modules"][0]["elements"].as_array_mut().unwrap(){e["name"]=e["id"].clone();if e["scalarType"]=="integer"{e["facets"]=json!({"integerWidth":{"bits":64,"signed":true}});}}
  let text=doc.to_string();let inputs=serde_json::from_value(json!([{"documentJson":text,"pin":{"documentId":doc["id"],"revision":source["revision"],"umfVersion":"0.8.0","sha256":crate::json::sha256(text.as_bytes())},"selectedModuleIds":["m"]}])).unwrap();
  let catalog=crate::model::Catalog::prepare_security(inputs).unwrap();let policy=f["policy"].to_string();let ontology=f["resolution"]["ontology"].to_string();let packet=crate::security_source::SecuritySourcePacket::read(&policy,&ontology,&catalog).unwrap();let plan=crate::security_ir::SecurityLogicalPlan::read(packet,&catalog).unwrap();
  let p=json!({"version":"weft.security.query-profile/0.1.0","id":"coverage","revision":"p1","action":"read","modelPins":catalog.pins(),"policySha256":crate::json::sha256(policy.as_bytes()),"ontologySha256":crate::json::sha256(ontology.as_bytes()),"binding":{"backendId":"fixture","backendVersion":"v1","targetProfile":"fixture","sha256":crate::json::sha256(b"{}")},"targets":[{"documentId":"domain","moduleId":"m","elementId":"Resource"}],"bindings":bindings}).to_string();
  let profile=crate::security_query_profile::SecurityQueryProfile::read(&p,&plan,&catalog,"{}","fixture","v1","fixture").unwrap();let query=crate::security_query_uses::SecurityResolvedQuery::resolve(sql,&catalog,&plan,Default::default(),None).unwrap();let uses=profile.admit_resolved_query(&query,&plan,&catalog,"{}","fixture","v1","fixture").unwrap();let context=SecurityBackendContext::new(&catalog,&plan,&query,&uses,"{}","fixture","v1","fixture").unwrap();
  let manifest=crate::security_backend::validate_security_manifest_json(&json!({"interfaceVersion":"weft-security-backend/0.1.0","backendId":"fixture","backendVersion":"v1","sourceProfiles":[{"dialect":"weft-sql/0.2.0","applicationIr":"weft-ir/0.2.0","policy":"0.1.0","ontology":"0.1.0","securityIr":"weft.security.logical-ir/0.1.0"}],"bindingProfile":"fixture-uninterpreted","targetProfiles":[{"id":"fixture","engine":"fixture","engineVersion":"v1","sessionSettings":{},"storageLayoutRevision":"v1","publicationRevision":"v1"}],"capabilities":[{"id":"all","targetProfiles":["fixture"],"languageProfiles":[{"dialectProfile":"weft-sql/0.2.0","irVersion":"weft-ir/0.2.0"}],"logicalDomain":logical(),"resultDomain":result(),"constraints":[],"obligations":[],"status":"supported","evidence":["fixture-only"]}],"evidence":["fixture-only"]}).to_string()).unwrap();run(&context,manifest);
 }
 #[test]
 fn actual_owner_coverage_keeps_whole_conditions_selection_and_output_positions(){
  with_context("SELECT r.salary AS first_value,r.salary AS second_value FROM Resource r",|ctx,m|{
   let ids=vec!["all".into()];assert!(check_selected(ctx,&m,&ids,false).is_ok());
   for ids in [vec![],vec!["missing".into()],vec!["all".into(),"all".into()]]{assert!(check_selected(ctx,&m,&ids,true).is_err());}
   let mut v=m.clone();v.capabilities[0].status=Status::Candidate;assert!(check_selected(ctx,&v,&ids,false).is_err());assert!(check_selected(ctx,&v,&ids,true).is_ok());v.capabilities[0].status=Status::Unsupported;assert!(check_selected(ctx,&v,&ids,true).is_err());
   let mut v=m.clone();v.capabilities[0].constraints.push("unknown".into());assert!(check_selected(ctx,&v,&ids,true).is_err());
   let mut v=m.clone();v.capabilities[0].logical_domain["limits"]["maxOutputs"]=json!(1);assert!(check_selected(ctx,&v,&ids,true).is_err());
   let mut v=m.clone();v.capabilities[0].logical_domain["limits"]["maxConditionNodes"]=json!(1);assert!(check_selected(ctx,&v,&ids,true).is_err());
   let mut v=m.clone();v.capabilities[0].logical_domain["targets"][0]["revision"]=json!("foreign");assert!(check_selected(ctx,&v,&ids,true).is_err());
   let mut v=m.clone();let mut extra=v.capabilities[0].clone();extra.id="unknown".into();extra.logical_domain["unknown"]=json!(true);v.capabilities.push(extra);assert!(check_selected(ctx,&v,&ids,true).is_ok());assert!(check_selected(ctx,&v,&vec!["all".into(),"unknown".into()],true).is_err());
  });
 }
 #[test]
 fn field_free_count_and_selfjoin_keep_actual_plan_requirements(){
  with_context("SELECT COUNT(*) FROM Resource r",|ctx,m|{assert_eq!(ctx.requirements().scans().len(),1);assert!(ctx.requirements().operators().is_empty());assert!(check_selected(ctx,&m,&vec!["all".into()],true).is_err());});
  with_context("SELECT r.salary FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId",|ctx,m|{assert_eq!(ctx.requirements().scans().len(),2);let ids=vec!["all".into()];assert!(check_selected(ctx,&m,&ids,true).is_ok());let mut v=m;v.capabilities[0].logical_domain["limits"]["maxScans"]=json!(1);assert!(check_selected(ctx,&v,&ids,true).is_err());});
 }

 #[test]
 fn partial_capabilities_source_substitution_and_unknown_selected_targets_refuse(){
  with_context("SELECT r.salary FROM Resource r",|ctx,m|{
   let ids=vec!["all".into()];
   for i in 0..6{let mut v=m.clone();match i{0=>v.interface_version="unknown".into(),1=>v.source_profiles[0].dialect="unknown".into(),2=>v.source_profiles[0].application_ir="unknown".into(),3=>v.source_profiles[0].policy="unknown".into(),4=>v.source_profiles[0].ontology="unknown".into(),_=>v.source_profiles[0].security_ir="unknown".into()};assert_eq!(check_selected(ctx,&v,&ids,true).unwrap_err().code,"WFT-SECURITY-LOWERING-UNSUPPORTED");}
   for target in ["missing","salary"]{let mut v=m.clone();v.capabilities[0].logical_domain["targets"][0]["elementId"]=json!(target);let d=check_selected(ctx,&v,&ids,true).unwrap_err();assert_eq!((d.code.as_str(),d.phase.as_str()),("WFT-SECURITY-LOWERING-UNSUPPORTED","capability"));}
   let mut v=m.clone();v.capabilities[0].logical_domain["limits"]["maxRulesPerAction"]=json!(1);let mut second=v.capabilities[0].clone();second.id="second".into();v.capabilities.push(second);assert!(check_selected(ctx,&v,&vec!["all".into(),"second".into()],true).is_err());
   v.capabilities[1].logical_domain["limits"]["maxRulesPerAction"]=json!(2);assert!(check_selected(ctx,&v,&vec!["all".into(),"second".into()],true).is_ok());
  });
 }
 #[test]
 fn false_branch_retains_whole_correlated_condition_at_exact_declared_bounds(){
  with_mutated_context("SELECT r.salary FROM Resource r",|f|{let condition=f["policy"]["rules"][1]["condition"].clone();f["policy"]["rules"][1]["condition"]=json!({"op":"and","args":[{"op":"literal","value":false},condition]});},|ctx,m|{
   let ids=vec!["all".into()];let mut v=m;
   for (name,exact) in [("maxConditionNodes",19),("maxConditionDepth",6),("maxExistsDepth",2)]{v.capabilities[0].logical_domain["limits"][name]=json!(exact);assert!(check_selected(ctx,&v,&ids,true).is_ok());v.capabilities[0].logical_domain["limits"][name]=json!(exact-1);assert!(check_selected(ctx,&v,&ids,true).is_err());v.capabilities[0].logical_domain["limits"][name]=json!(exact);}
  });
 }
 #[test]
 fn entire_component_budget_refuses_before_lookup_copy_or_matching_overrun(){
  with_context("SELECT r.salary AS a,r.salary AS b FROM Resource r",|ctx,m|{
   let ids=vec!["all".into()];let mut b=Budget{work:1_000_000,text:16_000_000};check_with_budget(ctx,&m,&ids,true,&mut b).unwrap();let used_work=1_000_000-b.work;let used_text=16_000_000-b.text;
   assert!(check_with_budget(ctx,&m,&ids,true,&mut Budget{work:used_work,text:used_text}).is_ok());
   for (work,text) in [(used_work-1,used_text),(used_work,used_text-1),(0,used_text),(used_work,0)]{assert_eq!(check_with_budget(ctx,&m,&ids,true,&mut Budget{work,text}).unwrap_err().code,"WFT-SECURITY-LOWERING-UNSUPPORTED");}
   assert!(OwnerIndex::build(ctx,&mut Budget{work:0,text:used_text}).is_err());assert!(OwnerIndex::build(ctx,&mut Budget{work:used_work,text:0}).is_err());
   assert!(parse_budget(&logical(),&result(),&mut Budget{work:0,text:used_text}).is_err());assert!(parse_budget(&logical(),&result(),&mut Budget{work:used_work,text:0}).is_err());
  });
 } #[test]
 fn projected_sources_require_actual_complete_owner_inventory() {
  fn project(ctx:&SecurityBackendContext<'_>,mut m:SecurityManifest,sources:Vec<String>)->Result<Vec<crate::security_lowering::SecurityAdmissionObligation>> {
   struct Backend(String);
   impl crate::security_backend::SecurityBackend for Backend {
    fn manifest_json(&self)->&str{&self.0}
    fn lower(&self,_:&SecurityBackendContext<'_>)->Result<crate::security_lowering::SecurityLowering>{panic!("source correspondence must not lower")}
   }
   m.capabilities[0].obligations=vec![crate::backend::Obligation{id:"lineage".into(),parameters:json!({"version":"weft.security.admission-obligation/0.1.0","semanticSources":sources,"enforcementSite":"host","prerequisites":[],"evidenceCaseIds":["unqualified-fixture-case"]}),owner:crate::backend::ObligationOwner::Host,failure_code:"WFT-LINEAGE".into()}];
   let mut registry=crate::security_backend::SecurityRegistry::default();let profiles:Vec<_>=m.source_profiles.iter().map(|p|json!({"dialect":p.dialect,"applicationIr":p.application_ir,"policy":p.policy,"ontology":p.ontology,"securityIr":p.security_ir})).collect();
   let raw=json!({"interfaceVersion":m.interface_version,"backendId":m.backend_id,"backendVersion":m.backend_version,"sourceProfiles":profiles,"bindingProfile":m.binding_profile,"targetProfiles":m.target_profiles,"capabilities":m.capabilities,"evidence":m.evidence}).to_string();
   registry.register(Backend(raw)).unwrap();
   let declaration=registry.declaration_for("fixture","v1","fixture").unwrap();
   crate::security_obligation_custody::ObligationCustody::collect(declaration,&["all".into()],false)?.project_for_context(ctx)
  }
  with_context("SELECT r.salary AS first_value,r.salary AS second_value FROM Resource r WHERE r.resourceId = 'resource-1' ORDER BY r.resourceId",|ctx,m|{
   let required=crate::security_obligation_sources::derive(ctx).unwrap();
   for (work,text) in [(0,16_000_000),(1_000_000,0),(3,16_000_000)] { assert!(crate::security_obligation_sources::test_budget(ctx,work,text).is_err()); }

   assert!(project(ctx,m.clone(),required.iter().cloned().collect()).is_ok());
   // Independently authored from this fixture's policy, ontology, Key definitions
   // and SQL; do not consult derive(), requirement scans, fields or operators.
   let mut expected=BTreeSet::new();
   let mut put=|tokens:&[&str]|{expected.insert(serde_json::to_string(tokens).unwrap());};
   put(&["primary-action","read"]);
   for (kind,raw) in [("policy",ctx.logical_plan().source().policy_json()),("ontology",ctx.logical_plan().source().ontology_json()),("query",ctx.query().sql())]{put(&[kind,&crate::json::sha256(raw.as_bytes())]);}
   put(&["model","domain","schema-1","0.8.0",&ctx.catalog().inputs()[0].pin.sha256]);put(&["module","domain","m"]);
   put(&["scan","s0","domain","m","Resource"]);put(&["action","s0","read"]);
   for rule in ["reader","membership"]{put(&["rule","s0","read",rule]);}
   for (owner,key_field) in [("Staff","staffId"),("Project","projectId"),("Resource","resourceId"),("Ownership","ownerId"),("Assignment","assignmentId")]{
    put(&["key","s0","read","domain","m",owner,"pk"]);
    put(&["key-field","s0","read","domain","m",owner,"pk","1","domain","m",key_field]);
   }
   for (owner,fields) in [("Staff",vec!["staffId"]),("Project",vec!["projectId"]),("Resource",vec!["resourceId"]),("Ownership",vec!["ownerId","ownerResource","ownerProject"]),("Assignment",vec!["assignmentId","assignmentStaff","assignmentProject","active"])]{
    for field in fields{put(&["field","s0","read","domain","m",owner,"domain","m",field]);}
   }
   for association in ["Ownership","Assignment"]{put(&["association","s0","read","domain","m",association]);}
   put(&["projection","s0","domain","m","salary"]);put(&["query-field","s0","domain","m","resourceId"]);
   for (position,operator) in [("1","predicate"),("2","order")]{put(&["operator",position,"s0","domain","m","Resource","domain","m","resourceId",operator,"disclosed"]);}
   put(&["output","1","first_value"]);put(&["output","2","second_value"]);
   assert_eq!(required,expected);
   let mut noncanonical:Vec<_>=required.iter().cloned().collect();
   noncanonical[0]=serde_json::to_string_pretty(&serde_json::from_str::<Vec<String>>(&noncanonical[0]).unwrap()).unwrap();
   assert!(project(ctx,m.clone(),noncanonical).is_err());

   let kinds:BTreeSet<String>=required.iter().map(|id|serde_json::from_str::<Vec<String>>(id).unwrap()[0].clone()).collect();
   for kind in ["policy","ontology","query","model","module","primary-action","scan","action","rule","key","key-field","field","association","projection","query-field","operator","output"]{assert!(kinds.contains(kind),"missing actual {kind} witness");}
   // Every issued entry is independently required: dropping any one refuses.
   for missing in &required {
    let incomplete=required.iter().filter(|s|*s!=missing).cloned().collect();
    assert!(project(ctx,m.clone(),incomplete).is_err(),"omitted {missing}");
   }
   let mut extra:Vec<_>=required.iter().cloned().collect();extra.push("[\"scan\",\"unrelated\"]".into());assert!(project(ctx,m.clone(),extra).is_err());
   let mut substituted:Vec<_>=required.iter().cloned().collect();substituted[0]="foreign-source".into();assert!(project(ctx,m.clone(),substituted).is_err());
   let outputs:Vec<_>=required.iter().map(|id|serde_json::from_str::<Vec<String>>(id).unwrap()).filter(|p|p[0]=="output").collect();
   assert_eq!(outputs.len(),2);assert_ne!(outputs[0][1],outputs[1][1]);
  });
 }
 #[test]
 fn source_paths_preserve_selfjoin_occurrences_and_field_free_count() {
  with_context("SELECT r.salary FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId",|ctx,_|{
   let ids=crate::security_obligation_sources::derive(ctx).unwrap();
   let paths:Vec<Vec<String>>=ids.iter().map(|id|serde_json::from_str(id).unwrap()).collect();
   let scans:Vec<_>=paths.iter().filter(|p|p[0]=="scan").collect();assert_eq!(scans.len(),2);assert_ne!(scans[0][1],scans[1][1]);
   for scan in scans {assert!(paths.iter().any(|p|p[0]=="action"&&p[1]==scan[1]));assert!(paths.iter().any(|p|p[0]=="key-field"&&p[1]==scan[1]));}
  });
  with_context("SELECT COUNT(*) FROM Resource r",|ctx,_|{
   let ids=crate::security_obligation_sources::derive(ctx).unwrap();
   let paths:Vec<Vec<String>>=ids.iter().map(|id|serde_json::from_str(id).unwrap()).collect();
   assert_eq!(paths.iter().filter(|p|p[0]=="scan").count(),1);
   assert_eq!(paths.iter().filter(|p|p[0]=="output").count(),1);
   assert!(paths.iter().any(|p|p[0]=="action"));
  });
 }
 #[test]
 fn actual_context_and_original_authorized_modes_keep_source_channels() {
  let active=json!({"documentId":"domain","moduleId":"m","elementId":"active"});
  with_mutated_context("SELECT r.salary FROM Resource r",|f|{
   f["resolution"]["ontology"]["context"]=json!([active.clone()]);
   let old=f["policy"]["rules"][1]["condition"].clone();
   f["policy"]["rules"][1]["condition"]=json!({"op":"and","args":[old,{"op":"eq","left":{"kind":"context","field":active},"right":{"kind":"constant","field":active,"value":{"boolean":true}}}]});
  },|ctx,m|{
   let ids=crate::security_obligation_sources::derive(ctx).unwrap();
   for expected in [json!(["context","s0","read","domain","m","active"]),json!(["field","s0","read","domain","m","Assignment","domain","m","active"])]{assert!(ids.contains(&expected.to_string()));}
   with_registered_coverage(ctx,m,&["all".into()],|coverage|{
    let issued=crate::security_obligation_sources::issue_demands(coverage).unwrap();
    use crate::security_obligation_sources::OwnerEventKind as E;
    for (tokens,event) in [(json!(["context","s0","read","domain","m","active"]),E::Context),(json!(["field","s0","read","domain","m","Assignment","domain","m","active"]),E::Field)] {
     let id=tokens.to_string();assert_eq!(issued.events().get(&id),Some(&event));
     assert_eq!(issued.demands()[&id].iter().map(|q|(*q).clone()).collect::<BTreeSet<_>>(),BTreeSet::from([scan_scope("s0","read")]));
    }
   });
  });
  let bindings=json!([{"target":{"documentId":"domain","moduleId":"m","elementId":"Resource"},"field":{"documentId":"domain","moduleId":"m","elementId":"salary"},"operator":"predicate","originalAction":"query-original"}]);
  with_bound_context("SELECT r.salary FROM Resource r WHERE r.salary > 0 ORDER BY r.resourceId",|f|{
   f["resolution"]["ontology"]["actions"].as_array_mut().unwrap().push(json!("query-original"));
   f["resolution"]["ontology"]["entities"][2]["fields"][1]["queryUse"]=json!({"predicate":"original-authorized"});
  },bindings,|ctx,_|{
   let ids=crate::security_obligation_sources::derive(ctx).unwrap();
   for action in ["read","query-original"]{assert!(ids.contains(&json!(["action","s0",action]).to_string()));}
   let paths:Vec<Vec<String>>=ids.iter().map(|id|serde_json::from_str(id).unwrap()).filter(|p:&Vec<String>|p[0]=="operator").collect();
   assert_eq!(paths.len(),2);
   assert!(paths.iter().any(|p|p[9]=="predicate"&&p[10]=="original-authorized"&&p[11]=="query-original"));
   assert!(paths.iter().any(|p|p[9]=="order"&&p[10]=="disclosed"&&p.len()==11));
  });
 }
 #[test]
 fn composite_key_sources_retain_actual_same_domain_member_order() {
  for reversed in [false,true] {
   with_mutated_context("SELECT r.salary FROM Resource r",|f|{
    let elements=f["resolution"]["documents"][0]["document"]["modules"][0]["elements"].as_array_mut().unwrap();
    for (old,new) in [("staffId","staffId2"),("assignmentStaff","assignmentStaff2")] {
     let mut field=elements.iter().find(|e|e["id"]==old).unwrap().clone();field["id"]=json!(new);elements.push(field);
    }
    for (owner,member) in [("Staff","staffId2"),("Assignment","assignmentStaff2")] {
     elements.iter_mut().find(|e|e["id"]==owner).unwrap()["members"].as_array_mut().unwrap().push(json!({"module":"m","element":member}));
    }
    let fields=if reversed{vec!["staffId2","staffId"]}else{vec!["staffId","staffId2"]};
    elements.iter_mut().find(|e|e["id"]=="Staff").unwrap()["keys"][0]["fields"]=json!(fields.iter().map(|id|json!({"module":"m","element":id})).collect::<Vec<_>>());
    let ontology=&mut f["resolution"]["ontology"];
    ontology["entities"][0]["fields"].as_array_mut().unwrap().push(json!({"ref":{"documentId":"domain","moduleId":"m","elementId":"staffId2"},"protection":"unprotected"}));
    ontology["associations"][1]["fields"].as_array_mut().unwrap().push(json!({"ref":{"documentId":"domain","moduleId":"m","elementId":"assignmentStaff2"},"protection":"unprotected"}));
    let fields=if reversed{vec!["assignmentStaff2","assignmentStaff"]}else{vec!["assignmentStaff","assignmentStaff2"]};
    ontology["associations"][1]["endpoints"][0]["fields"]=json!(fields.iter().map(|id|json!({"documentId":"domain","moduleId":"m","elementId":id})).collect::<Vec<_>>());
   },|ctx,_|{
    let ids=crate::security_obligation_sources::derive(ctx).unwrap();
    let ordered=if reversed{["staffId2","staffId"]}else{["staffId","staffId2"]};
    for (position,field) in ordered.iter().enumerate(){assert!(ids.contains(&json!(["key-field","s0","read","domain","m","Staff","pk",(position+1).to_string(),"domain","m",field]).to_string()));}
   });
  }
 }

 #[test]
 fn issued_allocations_keep_all_complete_candidates_without_fragment_union(){
  with_context("SELECT r.salary AS a,r.salary AS b FROM Resource r",|ctx,mut m|{
   let mut partial=m.capabilities[0].clone();partial.id="partial".into();partial.logical_domain["limits"]["maxRulesPerAction"]=json!(1);m.capabilities.push(partial);
   let mut second=m.capabilities[0].clone();second.id="second".into();m.capabilities.push(second);
   let ids=vec!["second".into(),"partial".into(),"all".into()];
   let a=assign_with_budget(ctx,&m,&ids,true,&mut Budget{work:1_000_000,text:16_000_000}).unwrap();
   assert_eq!(a[&CoverageScope::ScanAction{scan:"s0".into(),action:"read".into()}],BTreeSet::from(["all".into(),"second".into()]));
   assert_eq!(a[&CoverageScope::Application],BTreeSet::from(["all".into(),"partial".into(),"second".into()]));
   let reversed=ids.iter().rev().cloned().collect::<Vec<_>>();assert_eq!(a,assign_with_budget(ctx,&m,&reversed,true,&mut Budget{work:1_000_000,text:16_000_000}).unwrap());
   let mut b=Budget{work:1_000_000,text:16_000_000};assign_with_budget(ctx,&m,&ids,true,&mut b).unwrap();let (w,t)=(1_000_000-b.work,16_000_000-b.text);
   assert!(assign_with_budget(ctx,&m,&ids,true,&mut Budget{work:w,text:t}).is_ok());
   for (work,text) in [(w-1,t),(w,t-1)]{assert!(assign_with_budget(ctx,&m,&ids,true,&mut Budget{work,text}).is_err());}
  });
 }
 #[test]
 fn issued_selfjoin_allocations_preserve_scopes_and_whole_application(){
  with_context("SELECT r.salary FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId",|ctx,mut m|{
   let mut narrow=m.capabilities[0].clone();narrow.id="one-scan".into();narrow.logical_domain["limits"]["maxScans"]=json!(1);m.capabilities.push(narrow);
   let a=assign_with_budget(ctx,&m,&["all".into(),"one-scan".into()],true,&mut Budget{work:1_000_000,text:16_000_000}).unwrap();assert_eq!(a.len(),3);
   for scan in ["s0","s1"]{assert_eq!(a[&CoverageScope::ScanAction{scan:scan.into(),action:"read".into()}],BTreeSet::from(["all".into(),"one-scan".into()]));}
   assert_eq!(a[&CoverageScope::Application],BTreeSet::from(["all".into()]));
   assert!(assign_with_budget(ctx,&m,&["one-scan".into()],true,&mut Budget{work:1_000_000,text:16_000_000}).is_err());
  });
 }
 #[test]
 fn allocation_issuer_borrows_actual_registration_and_context_after_callback_drift(){
  use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
  struct Backend{raw:String,drift:Arc<AtomicBool>}
  impl crate::security_backend::SecurityBackend for Backend{
   fn manifest_json(&self)->&str{if self.drift.load(Ordering::SeqCst){"{}"}else{&self.raw}}
   fn lower(&self,_:&SecurityBackendContext<'_>)->Result<crate::security_lowering::SecurityLowering>{panic!("allocation must not lower")}
  }
  with_context("SELECT r.salary FROM Resource r",|ctx,mut m|{
   let mut unrelated=m.capabilities[0].clone();unrelated.id="staff-only".into();unrelated.logical_domain["targets"][0]["elementId"]=json!("Staff");unrelated.obligations.clear();m.capabilities.push(unrelated);
   let profiles:Vec<_>=m.source_profiles.iter().map(|p|json!({"dialect":p.dialect,"applicationIr":p.application_ir,"policy":p.policy,"ontology":p.ontology,"securityIr":p.security_ir})).collect();
   let raw=json!({"interfaceVersion":m.interface_version,"backendId":m.backend_id,"backendVersion":m.backend_version,"sourceProfiles":profiles,"bindingProfile":m.binding_profile,"targetProfiles":m.target_profiles,"capabilities":m.capabilities,"evidence":m.evidence}).to_string();
   let drift=Arc::new(AtomicBool::new(false));let mut registry=crate::security_backend::SecurityRegistry::default();registry.register(Backend{raw:raw.clone(),drift:drift.clone()}).unwrap();drift.store(true,Ordering::SeqCst);
   let declaration=registry.declaration_for("fixture","v1","fixture").unwrap();let assigned=issue(ctx,declaration,&["all".into()],true).unwrap();
   assert!(std::ptr::eq(assigned.context(),ctx));assert_eq!(assigned.declaration().manifest_json(),raw);assert_eq!(assigned.assignments().len(),2);
   assert_eq!(assigned.assignments()[&CoverageScope::Application],BTreeSet::from(["all".into()]));
   assert_eq!(assigned.selected_capabilities(),&BTreeSet::from(["all".into()]));
   let other=issue(ctx,registry.declaration_for("fixture","v1","fixture").unwrap(),&["all".into(),"staff-only".into()],true).unwrap();
   assert_eq!(assigned.assignments(),other.assignments());assert_ne!(assigned.selected_capabilities(),other.selected_capabilities());
   assert_eq!(other.selected_capabilities(),&BTreeSet::from(["all".into(),"staff-only".into()]));
  });
 }
 #[test]
 fn allocation_relation_bounds_refuse_empty_duplicate_scope_and_overflow(){
  let mut a=BTreeMap::new();let mut edges=0;let candidates=BTreeSet::from(["a".into()]);
  let mut b=Budget{work:1,text:0};retain(&mut a,CoverageScope::Application,candidates.clone(),&mut edges,&mut b).unwrap();assert_eq!(b.work,0);
  assert!(retain(&mut BTreeMap::new(),CoverageScope::Application,candidates.clone(),&mut 0,&mut Budget{work:0,text:0}).is_err());
  assert!(retain(&mut a,CoverageScope::Application,candidates.clone(),&mut edges,&mut Budget{work:1,text:0}).is_err());
  assert!(retain(&mut BTreeMap::new(),CoverageScope::Application,BTreeSet::new(),&mut 0,&mut Budget{work:1,text:0}).is_err());
  let mut edges=4095;assert!(retain(&mut BTreeMap::new(),CoverageScope::Application,candidates.clone(),&mut edges,&mut Budget{work:1,text:0}).is_ok());
  let mut edges=4096;assert!(retain(&mut BTreeMap::new(),CoverageScope::Application,candidates,&mut edges,&mut Budget{work:1,text:0}).is_err());
 }

 #[test]
 fn issued_allocations_keep_original_authorized_action_separate_from_read(){
  let bindings=json!([{"target":{"documentId":"domain","moduleId":"m","elementId":"Resource"},"field":{"documentId":"domain","moduleId":"m","elementId":"salary"},"operator":"predicate","originalAction":"query-original"}]);
  with_bound_context("SELECT r.salary FROM Resource r WHERE r.salary > 0",|f|{
   f["resolution"]["ontology"]["actions"].as_array_mut().unwrap().push(json!("query-original"));
   f["resolution"]["ontology"]["entities"][2]["fields"][1]["queryUse"]=json!({"predicate":"original-authorized"});
  },bindings,|ctx,mut m|{
   let mut partial=m.capabilities[0].clone();partial.id="one-rule".into();partial.logical_domain["limits"]["maxRulesPerAction"]=json!(1);m.capabilities.push(partial);
   let a=assign_with_budget(ctx,&m,&["all".into(),"one-rule".into()],true,&mut Budget{work:1_000_000,text:16_000_000}).unwrap();
   assert_eq!(a[&CoverageScope::ScanAction{scan:"s0".into(),action:"read".into()}],BTreeSet::from(["all".into()]));
   assert_eq!(a[&CoverageScope::ScanAction{scan:"s0".into(),action:"query-original".into()}],BTreeSet::from(["all".into(),"one-rule".into()]));
   assert_eq!(a.len(),3); // Coverage of an action with zero rules is not Permit.
  });
 }

 fn with_registered_coverage(ctx:&SecurityBackendContext<'_>,m:SecurityManifest,ids:&[String],run:impl FnOnce(&OwnerCoverage<'_, '_, '_>)) {
  struct Backend(String);
  impl crate::security_backend::SecurityBackend for Backend {
   fn manifest_json(&self)->&str { &self.0 }
   fn lower(&self,_:&SecurityBackendContext<'_>)->Result<crate::security_lowering::SecurityLowering>{panic!("demand issuance must not lower")}
  }
  let profiles:Vec<_>=m.source_profiles.iter().map(|p|json!({"dialect":p.dialect,"applicationIr":p.application_ir,"policy":p.policy,"ontology":p.ontology,"securityIr":p.security_ir})).collect();
  let raw=json!({"interfaceVersion":m.interface_version,"backendId":m.backend_id,"backendVersion":m.backend_version,"sourceProfiles":profiles,"bindingProfile":m.binding_profile,"targetProfiles":m.target_profiles,"capabilities":m.capabilities,"evidence":m.evidence}).to_string();
  let mut registry=crate::security_backend::SecurityRegistry::default();registry.register(Backend(raw)).unwrap();
  let coverage=issue(ctx,registry.declaration_for("fixture","v1","fixture").unwrap(),ids,true).unwrap();run(&coverage);
 }
 fn scan_scope(scan:&str,action:&str)->CoverageScope{CoverageScope::ScanAction{scan:scan.into(),action:action.into()}}
 #[test]
 fn issued_source_demands_match_independent_complete_fixture_map_and_exact_budgets(){
  with_context("SELECT r.salary AS first_value,r.salary AS second_value FROM Resource r WHERE r.resourceId = 'resource-1' ORDER BY r.resourceId",|ctx,m|{
   with_registered_coverage(ctx,m,&["all".into()],|coverage|{
    let issued=crate::security_obligation_sources::issue_demands(coverage).unwrap();
    assert!(std::ptr::eq(issued.coverage(),coverage));
    let read=scan_scope("s0","read");let app=CoverageScope::Application;
    let mut expected=BTreeMap::new();
    let mut put=|tokens:&[&str],scopes:Vec<CoverageScope>|{expected.insert(serde_json::to_string(tokens).unwrap(),scopes.into_iter().collect::<BTreeSet<_>>());};
    put(&["primary-action","read"],vec![read.clone(),app.clone()]);
    for (kind,raw) in [("policy",ctx.logical_plan().source().policy_json()),("ontology",ctx.logical_plan().source().ontology_json()),("query",ctx.query().sql())]{put(&[kind,&crate::json::sha256(raw.as_bytes())],vec![read.clone(),app.clone()]);}
    put(&["model","domain","schema-1","0.8.0",&ctx.catalog().inputs()[0].pin.sha256],vec![read.clone(),app.clone()]);put(&["module","domain","m"],vec![read.clone(),app.clone()]);
    put(&["scan","s0","domain","m","Resource"],vec![read.clone(),app.clone()]);put(&["action","s0","read"],vec![read.clone()]);
    for rule in ["reader","membership"]{put(&["rule","s0","read",rule],vec![read.clone()]);}
    for (owner,key_field) in [("Staff","staffId"),("Project","projectId"),("Resource","resourceId"),("Ownership","ownerId"),("Assignment","assignmentId")]{
     put(&["key","s0","read","domain","m",owner,"pk"],vec![read.clone()]);put(&["key-field","s0","read","domain","m",owner,"pk","1","domain","m",key_field],vec![read.clone()]);
    }
    for (owner,fields) in [("Staff",vec!["staffId"]),("Project",vec!["projectId"]),("Resource",vec!["resourceId"]),("Ownership",vec!["ownerId","ownerResource","ownerProject"]),("Assignment",vec!["assignmentId","assignmentStaff","assignmentProject","active"])]{for field in fields{put(&["field","s0","read","domain","m",owner,"domain","m",field],vec![read.clone()]);}}
    for association in ["Ownership","Assignment"]{put(&["association","s0","read","domain","m",association],vec![read.clone()]);}
    put(&["projection","s0","domain","m","salary"],vec![read.clone(),app.clone()]);put(&["query-field","s0","domain","m","resourceId"],vec![read.clone(),app.clone()]);
    for (pos,op) in [("1","predicate"),("2","order")]{put(&["operator",pos,"s0","domain","m","Resource","domain","m","resourceId",op,"disclosed"],vec![read.clone(),app.clone()]);}
    put(&["output","1","first_value"],vec![read.clone(),app.clone()]);put(&["output","2","second_value"],vec![read.clone(),app.clone()]);
    let actual:BTreeMap<_,_>=issued.demands().iter().map(|(id,scopes)|(id.clone(),scopes.iter().map(|s|(*s).clone()).collect::<BTreeSet<_>>())).collect();assert_eq!(actual,expected);
    // Independent golden fixture categories; production never parses source IDs.
    use crate::security_obligation_sources::OwnerEventKind as E;
    let kinds=BTreeMap::from([("primary-action",E::PrimaryAction),("policy",E::Policy),("ontology",E::Ontology),("query",E::Query),("model",E::Model),("module",E::Module),("scan",E::Scan),("projection",E::Projection),("query-field",E::QueryField),("action",E::Action),("rule",E::Rule),("key",E::Key),("key-field",E::KeyField),("field",E::Field),("context",E::Context),("association",E::Association),("operator",E::Operator),("output",E::Output)]);
    let expected_events:BTreeMap<_,_>=expected.keys().map(|id|{let golden:Vec<String>=serde_json::from_str(id).unwrap();(id.clone(),kinds[golden[0].as_str()])}).collect();
    assert_eq!(issued.events(),&expected_events);
    let expected_rules:BTreeMap<_,_>=ctx.requirements().scans().iter().flat_map(|scan|scan.actions().iter().flat_map(move |action|action.rules().iter().map(move |rule|(serde_json::to_string(&["rule",scan.inventory().scan(),action.inventory().action(),&rule.id]).unwrap(),*rule)))).collect();
    assert_eq!(issued.rule_events().keys().collect::<Vec<_>>(),expected_rules.keys().collect::<Vec<_>>());

    use crate::security_rule_occurrences::{RulePath as P,RulePayload};
    let occurrences=crate::security_rule_occurrences::issue(&issued).unwrap();assert!(std::ptr::eq(occurrences.owner(),&issued));
    let reader=json!(["rule","s0","read","reader"]).to_string();let membership=json!(["rule","s0","read","membership"]).to_string();
    let mut golden_occurrences=BTreeSet::from([(reader.as_str(),P::Rule),(reader.as_str(),P::Condition(vec![])),(reader.as_str(),P::Disclosure(0)),(membership.as_str(),P::Rule)]);
    for path in [vec![],vec![0],vec![0,0],vec![0,1],vec![0,1,0],vec![0,1,0,0],vec![0,1,0,1],vec![0,1,0,2]]{golden_occurrences.insert((membership.as_str(),P::Condition(path)));}
    for path in [vec![0,0],vec![0,1,0,0],vec![0,1,0,1],vec![0,1,0,2]] {for operand in [0,1]{golden_occurrences.insert((membership.as_str(),P::Operand(path.clone(),operand)));}}
    assert_eq!(occurrences.entries().keys().cloned().collect::<BTreeSet<_>>(),golden_occurrences);assert_eq!(occurrences.entries().len(),20);
    use crate::security_ir::Expression as X;
    let actual_membership=issued.rule_events()[&membership];
    let X::Exists{slot,association,condition}= &actual_membership.condition else{panic!()};assert_eq!(*slot,0);assert_eq!(association.record().unwrap().element_id,"Ownership");
    let RulePayload::Condition(retained)=&occurrences.entries()[&(membership.as_str(),P::Condition(vec![0]))] else{panic!()};assert!(std::ptr::eq(*retained,condition.as_ref()));
    let X::And(outer)=condition.as_ref() else{panic!()};
    let X::Exists{slot,association,condition:inner}=&outer[1] else{panic!()};assert_eq!(*slot,1);assert_eq!(association.record().unwrap().element_id,"Assignment");
    for (path,node) in [(vec![0,1],&outer[1]),(vec![0,1,0],inner.as_ref())]{let RulePayload::Condition(retained)=&occurrences.entries()[&(membership.as_str(),P::Condition(path))] else{panic!()};assert!(std::ptr::eq(*retained,node));}
    let X::And(inner_args)=inner.as_ref() else{panic!()};
    for (path,node) in [(vec![0,0],&outer[0]),(vec![0,1,0,0],&inner_args[0]),(vec![0,1,0,1],&inner_args[1]),(vec![0,1,0,2],&inner_args[2])] {
     let X::Equal(left,right)=node else{panic!()};assert!(!std::ptr::eq(left,right));
     for (position,term) in [(0,left),(1,right)]{let RulePayload::Operand(retained)=&occurrences.entries()[&(membership.as_str(),P::Operand(path.clone(),position))] else{panic!()};assert!(std::ptr::eq(*retained,term));}
    }

    for source in [&reader,&membership]{let RulePayload::Rule(rule)=&occurrences.entries()[&(source.as_str(),P::Rule)] else {panic!()};assert!(std::ptr::eq(*rule,issued.rule_events()[source]));}
    let (rw,rt)=crate::security_rule_occurrences::test_budget(&issued,1_000_000,16_000_000).unwrap();let w=1_000_000-rw;let t=16_000_000-rt;
    assert!(crate::security_rule_occurrences::test_budget(&issued,w,t).is_ok());assert!(crate::security_rule_occurrences::test_budget(&issued,w-1,t).is_err());assert!(crate::security_rule_occurrences::test_budget(&issued,w,t-1).is_err());
    let golden_rule_ids:BTreeSet<_>=["reader","membership"].iter().map(|name|json!(["rule","s0","read",name]).to_string()).collect();
    assert_eq!(issued.rule_events().keys().cloned().collect::<BTreeSet<_>>(),golden_rule_ids);

    for (id,rule) in &expected_rules {assert!(std::ptr::eq(issued.rule_events()[id],*rule));}

    assert_eq!(issued.events().values().filter(|k|**k==E::Output).count(),2);
    assert_eq!(issued.events().values().filter(|k|**k==E::Operator).count(),2);

    assert_eq!(actual.keys().cloned().collect::<BTreeSet<_>>(),crate::security_obligation_sources::derive(ctx).unwrap());
    // Independently count the old source-only ledger from the hand-authored map.
    // Demand-only operator comparisons must not consume this legacy ledger.
    let mut legacy_work=3;let mut legacy_text=ctx.logical_plan().source().policy_json().len()+ctx.logical_plan().source().ontology_json().len()+ctx.query().sql().len();
    for id in expected.keys(){let tokens:Vec<String>=serde_json::from_str(id).unwrap();legacy_work+=tokens.len()+1;legacy_text+=tokens.iter().map(String::len).sum::<usize>()+id.len();}
    assert!(crate::security_obligation_sources::test_budget(ctx,legacy_work,legacy_text).is_ok());
    assert!(crate::security_obligation_sources::test_budget(ctx,legacy_work-1,legacy_text).is_err());
    assert!(crate::security_obligation_sources::test_budget(ctx,legacy_work,legacy_text-1).is_err());
    for scope in coverage.assignments().keys(){assert!(std::ptr::eq(issued.candidates(scope).unwrap(),&coverage.assignments()[scope]));}
    let (_,rw,rt)=crate::security_obligation_sources::test_demands(ctx,coverage.assignments(),1_000_000,16_000_000).unwrap();let w=1_000_000-rw;let t=16_000_000-rt;
    assert!(crate::security_obligation_sources::test_demands(ctx,coverage.assignments(),w,t).is_ok());
    for (work,text) in [(w-1,t),(w,t-1)]{assert!(crate::security_obligation_sources::test_demands(ctx,coverage.assignments(),work,text).is_err());}
    for missing in coverage.assignments().keys(){let mut incomplete=coverage.assignments().clone();incomplete.remove(missing);assert!(crate::security_obligation_sources::test_demands(ctx,&incomplete,1_000_000,16_000_000).is_err());}
   });
  });
 }
 #[test]
 fn issued_source_demands_accumulate_distinct_original_actions_on_one_field(){
  let target=json!({"documentId":"domain","moduleId":"m","elementId":"Resource"});let field=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});
  let bindings=json!([{"target":target,"field":field,"operator":"predicate","originalAction":"original-predicate"},{"target":target,"field":field,"operator":"order","originalAction":"original-order"}]);
  with_bound_context("SELECT r.salary FROM Resource r WHERE r.salary > 0 ORDER BY r.salary",|f|{
   for action in ["original-predicate","original-order"]{f["resolution"]["ontology"]["actions"].as_array_mut().unwrap().push(json!(action));}
   f["resolution"]["ontology"]["entities"][2]["fields"][1]["queryUse"]=json!({"predicate":"original-authorized","order":"original-authorized"});
  },bindings,|ctx,m|{with_registered_coverage(ctx,m,&["all".into()],|coverage|{
   let issued=crate::security_obligation_sources::issue_demands(coverage).unwrap();
   let expected=BTreeSet::from([scan_scope("s0","read"),scan_scope("s0","original-predicate"),scan_scope("s0","original-order"),CoverageScope::Application]);
   for missing in &expected {let mut incomplete=coverage.assignments().clone();incomplete.remove(missing);assert!(crate::security_obligation_sources::test_demands(ctx,&incomplete,1_000_000,16_000_000).is_err(),"missing {missing:?}");}
   let id=json!(["query-field","s0","domain","m","salary"]).to_string();assert_eq!(issued.demands()[&id].iter().map(|s|(*s).clone()).collect::<BTreeSet<_>>(),expected);
   for (pos,op,action) in [("1","predicate","original-predicate"),("2","order","original-order")]{
    let id=json!(["operator",pos,"s0","domain","m","Resource","domain","m","salary",op,"original-authorized",action]).to_string();
    assert_eq!(issued.demands()[&id].iter().map(|s|(*s).clone()).collect::<BTreeSet<_>>(),BTreeSet::from([scan_scope("s0","read"),scan_scope("s0",action),CoverageScope::Application]));
   }
  });});
 }

 #[test]
 fn rule_event_custody_retains_both_scan_and_action_coordinates() {
  let target=json!({"documentId":"domain","moduleId":"m","elementId":"Resource"});let field=json!({"documentId":"domain","moduleId":"m","elementId":"salary"});
  let bindings=json!([{"target":target,"field":field,"operator":"predicate","originalAction":"query-original"}]);
  with_bound_context("SELECT r.salary FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId WHERE r.salary > 0 AND s.salary > 0",|f|{
   f["resolution"]["ontology"]["actions"].as_array_mut().unwrap().push(json!("query-original"));
   f["resolution"]["ontology"]["entities"][2]["fields"][1]["queryUse"]=json!({"predicate":"original-authorized"});
   for rule in f["policy"]["rules"].as_array_mut().unwrap(){rule["actions"].as_array_mut().unwrap().push(json!("query-original"));}
  },bindings,|ctx,m|{with_registered_coverage(ctx,m,&["all".into()],|coverage|{
   let issued=crate::security_obligation_sources::issue_demands(coverage).unwrap();let mut expected=BTreeSet::new();
   for scan in ["s0","s1"] {for action in ["read","query-original"] {for name in ["reader","membership"] {
    let id=json!(["rule",scan,action,name]).to_string();expected.insert(id.clone());
    let actual=ctx.logical_plan().rules().iter().find(|r|r.id==name).unwrap();assert!(std::ptr::eq(issued.rule_events()[&id],actual));
    assert_eq!(issued.demands()[&id].iter().map(|q|(*q).clone()).collect::<BTreeSet<_>>(),BTreeSet::from([scan_scope(scan,action)]));
   }}}
   assert_eq!(issued.rule_events().keys().cloned().collect::<BTreeSet<_>>(),expected);
   let occurrences=crate::security_rule_occurrences::issue(&issued).unwrap();assert_eq!(occurrences.entries().len(),80);
   for source in expected {let crate::security_rule_occurrences::RulePayload::Rule(rule)=&occurrences.entries()[&(source.as_str(),crate::security_rule_occurrences::RulePath::Rule)] else {panic!()};assert!(std::ptr::eq(*rule,issued.rule_events()[&source]));}
  });});
 }
 #[test]
 fn issued_source_demands_keep_output_selfjoin_and_false_branch_ownership(){
  with_mutated_context("SELECT s.salary AS chosen FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId",|f|{let old=f["policy"]["rules"][1]["condition"].clone();f["policy"]["rules"][1]["condition"]=json!({"op":"and","args":[{"op":"literal","value":false},old]});},|ctx,m|{with_registered_coverage(ctx,m,&["all".into()],|coverage|{
   let issued=crate::security_obligation_sources::issue_demands(coverage).unwrap();

   let occurrences=crate::security_rule_occurrences::issue(&issued).unwrap();assert_eq!(occurrences.entries().len(),44);
   let membership=ctx.logical_plan().rules().iter().find(|r|r.id=="membership").unwrap();
   let crate::security_ir::Expression::And(args)=&membership.condition else {panic!("expected populated false branch")};
   assert!(matches!(args.first(),Some(crate::security_ir::Expression::Literal(false))));assert_eq!(args.len(),2);
   for scan in ["s0","s1"] {
    let id=json!(["rule",scan,"read","membership"]).to_string();
    assert!(std::ptr::eq(issued.rule_events()[&id],membership));
    assert_eq!(issued.demands()[&id].iter().map(|q|(*q).clone()).collect::<BTreeSet<_>>(),BTreeSet::from([scan_scope(scan,"read")]));
   }
   let output=json!(["output","1","chosen"]).to_string();assert_eq!(issued.demands()[&output].iter().map(|s|(*s).clone()).collect::<BTreeSet<_>>(),BTreeSet::from([scan_scope("s1","read"),CoverageScope::Application]));
   for scan in ["s0","s1"]{let field=json!(["field",scan,"read","domain","m","Assignment","domain","m","active"]).to_string();assert_eq!(issued.demands()[&field].iter().map(|s|(*s).clone()).collect::<BTreeSet<_>>(),BTreeSet::from([scan_scope(scan,"read")]));}
  });});
 }

 #[test]
 fn issued_source_demands_preserve_disjoint_candidates_and_zero_edge_selection(){
  let bindings=json!([{"target":{"documentId":"domain","moduleId":"m","elementId":"Resource"},"field":{"documentId":"domain","moduleId":"m","elementId":"salary"},"operator":"predicate","originalAction":"query-original"}]);
  with_bound_context("SELECT r.salary AS a,r.salary AS b FROM Resource r WHERE r.salary > 0",|f|{
   f["resolution"]["ontology"]["actions"].as_array_mut().unwrap().push(json!("query-original"));f["resolution"]["ontology"]["entities"][2]["fields"][1]["queryUse"]=json!({"predicate":"original-authorized"});
  },bindings,|ctx,mut m|{
   let mut read=m.capabilities[0].clone();read.id="read-only".into();read.logical_domain["limits"]["maxOutputs"]=json!(1);
   let mut app=m.capabilities[0].clone();app.id="application-only".into();app.logical_domain["limits"]["maxRulesPerAction"]=json!(1);
   let mut original=m.capabilities[0].clone();original.id="original-only".into();original.logical_domain["limits"]["maxRulesPerAction"]=json!(1);original.logical_domain["limits"]["maxOutputs"]=json!(1);
   let mut unused=m.capabilities[0].clone();unused.id="unused-staff".into();unused.logical_domain["targets"]=json!([{"documentId":"domain","revision":"schema-1","moduleId":"m","elementId":"Staff"}]);
   m.capabilities=vec![read,app,original,unused];
   let selected=vec!["read-only".into(),"application-only".into(),"original-only".into(),"unused-staff".into()];
   with_registered_coverage(ctx,m,&selected,|coverage|{
    let issued=crate::security_obligation_sources::issue_demands(coverage).unwrap();
    assert_eq!(issued.coverage().selected_capabilities(),&selected.iter().cloned().collect());
    assert_eq!(issued.candidates(&scan_scope("s0","read")).unwrap(),&BTreeSet::from(["read-only".into()]));
    assert_eq!(issued.candidates(&CoverageScope::Application).unwrap(),&BTreeSet::from(["application-only".into()]));
    assert_eq!(issued.candidates(&scan_scope("s0","query-original")).unwrap(),&BTreeSet::from(["application-only".into(),"original-only".into(),"read-only".into()]));
    assert!(coverage.assignments().values().all(|ids|!ids.contains("unused-staff")));
    let op=json!(["operator","1","s0","domain","m","Resource","domain","m","salary","predicate","original-authorized","query-original"]).to_string();
    assert_eq!(issued.demands()[&op].iter().map(|s|(*s).clone()).collect::<BTreeSet<_>>(),BTreeSet::from([scan_scope("s0","read"),scan_scope("s0","query-original"),CoverageScope::Application]));
   });
  });
 }
 #[test]
 fn scoped_matching_uses_actual_owner_demands_and_independent_required_contracts(){
  use crate::security_obligation_matching::{RequiredPremise,Contract,Atom,Subject,match_required};
  use crate::security_obligation_custody::ObligationCustody;
  use crate::backend::{Obligation,ObligationOwner};
  with_context("SELECT r.salary FROM Resource r",|ctx,mut m|{
   let mut owner_sources=BTreeMap::new();
   with_registered_coverage(ctx,m.clone(),&["all".into()],|coverage|{
    let demands=crate::security_obligation_sources::issue_demands(coverage).unwrap();
    owner_sources=demands.demands().iter().map(|(source,scopes)|(source.clone(),scopes.iter().map(|q|(*q).clone()).collect::<BTreeSet<_>>())).collect();
   });
   // Declaration template is authored separately from the required profile below.
   let mut obligations=Vec::new();let mut index=0;
   for (source,scopes) in &owner_sources {for scope in scopes {
    let q=match scope {CoverageScope::Application=>json!({"kind":"application"}),CoverageScope::ScanAction{scan,action}=>json!({"kind":"scan-action","scan":scan,"action":action})};
    obligations.push(Obligation{id:format!("semantic-{index}"),owner:ObligationOwner::Host,failure_code:"WFT-TEST-SEMANTIC".into(),parameters:json!({"version":"weft.security.admission-obligation/0.2.0","subjects":[{"kind":"semantic","source":source,"scope":q}],"enforcementSite":"host","prerequisites":[],"evidenceCaseIds":["semantic-case"]})});index+=1;
   }}
   m.capabilities[0].obligations=obligations;
   let mut unused=m.capabilities[0].clone();unused.id="unused-staff".into();unused.logical_domain["targets"]=json!([{"documentId":"domain","revision":"schema-1","moduleId":"m","elementId":"Staff"}]);
   unused.obligations=vec![Obligation{id:"deployment".into(),owner:ObligationOwner::Backend,failure_code:"WFT-TEST-DEPLOY".into(),parameters:json!({"version":"weft.security.admission-obligation/0.2.0","subjects":[{"kind":"selected-capability","profileRequirementId":"installed-profile"}],"enforcementSite":"native","prerequisites":[],"evidenceCaseIds":["deployment-case"]})}];
   m.capabilities.push(unused);
   let mut shared_m=m.clone();let mut also=shared_m.capabilities[0].clone();also.id="also-complete".into();shared_m.capabilities.push(also);
   let mut empty_m=m.clone();empty_m.capabilities[1].obligations.clear();
   with_registered_coverage(ctx,m,&["all".into(),"unused-staff".into()],|coverage|{
    let demands=crate::security_obligation_sources::issue_demands(coverage).unwrap();
    let custody=ObligationCustody::collect(*coverage.declaration(),&["all".into(),"unused-staff".into()],true).unwrap();
    // Explicit independently authored fixture profile; this is not authenticated.
    let mut r=RequiredPremise {version:"weft.security.required-obligations/0.1.0".into(),profile_id:"fixture-required-profile".into(),registration_json:coverage.declaration().manifest_json().into(),target:"fixture".into(),selected:BTreeSet::from(["all".into(),"unused-staff".into()]),selected_requirements:BTreeMap::from([("all".into(),BTreeSet::new()),("unused-staff".into(),BTreeSet::from(["installed-profile".into()]))]),contracts:BTreeMap::new(),atoms:BTreeSet::new()};
    let mut index=0;
    for (source,scopes) in &owner_sources {for scope in scopes {
     let id=format!("semantic-{index}");index+=1;
     r.contracts.insert(id.clone(),Contract{owner:ObligationOwner::Host,failure:"WFT-TEST-SEMANTIC".into(),prerequisites:BTreeSet::new(),site:"host".into(),semantic:true});
     r.atoms.insert(Atom{obligation:id,origin:"all".into(),subject:Subject::Semantic{source:source.clone(),scope:scope.clone()},site:"host".into(),case:"semantic-case".into()});
    }}
    r.contracts.insert("deployment".into(),Contract{owner:ObligationOwner::Backend,failure:"WFT-TEST-DEPLOY".into(),prerequisites:BTreeSet::new(),site:"native".into(),semantic:false});
    r.atoms.insert(Atom{obligation:"deployment".into(),origin:"unused-staff".into(),subject:Subject::SelectedCapability{requirement:"installed-profile".into()},site:"native".into(),case:"deployment-case".into()});
    with_registered_coverage(ctx,shared_m,&["all".into(),"also-complete".into(),"unused-staff".into()],|shared_coverage|{
     let shared_demands=crate::security_obligation_sources::issue_demands(shared_coverage).unwrap();
     let shared_custody=ObligationCustody::collect(*shared_coverage.declaration(),&["all".into(),"also-complete".into(),"unused-staff".into()],true).unwrap();
     let mut shared_required=r.clone();
     shared_required.registration_json=shared_coverage.declaration().manifest_json().into();
     shared_required.selected.insert("also-complete".into());shared_required.selected_requirements.insert("also-complete".into(),BTreeSet::new());
     for atom in &r.atoms {if atom.origin=="all" {let mut a=atom.clone();a.origin="also-complete".into();shared_required.atoms.insert(a);}}
     let matched=match_required(&shared_demands,&shared_custody,&shared_required).unwrap();
     let typed=fixture_instances(&shared_required);
     let typed_match=crate::security_obligation_matching::match_instances(&shared_demands,&shared_custody,&typed).unwrap();
     assert_eq!(typed_match.alternatives(),matched.alternatives());
     assert_eq!(matched.originals().len(),r.contracts.len());
     assert_eq!(matched.alternatives(),&BTreeMap::from([(scan_scope("s0","read"),BTreeSet::from(["all".into(),"also-complete".into()])),(CoverageScope::Application,BTreeSet::from(["all".into(),"also-complete".into()]))]));
     for declaration in matched.originals() {if declaration.original.id.starts_with("semantic-") {assert_eq!(declaration.origins,["all","also-complete"]);}}
     // A complete alternative does not excuse any shared original's other origin.
     for atom in shared_required.atoms.iter().filter(|a|a.origin=="also-complete") {
      let mut missing=shared_required.clone();missing.atoms.remove(atom);
      assert!(match_required(&shared_demands,&shared_custody,&missing).is_err());
     }
    });
    with_registered_coverage(ctx,empty_m,&["all".into(),"unused-staff".into()],|empty_coverage|{
     let empty_demands=crate::security_obligation_sources::issue_demands(empty_coverage).unwrap();
     let empty_custody=ObligationCustody::collect(*empty_coverage.declaration(),&["all".into(),"unused-staff".into()],true).unwrap();
     assert!(empty_custody.selected_capabilities().contains("unused-staff"));
     assert!(empty_custody.original("deployment").is_err());
     let mut still_required=r.clone();still_required.registration_json=empty_coverage.declaration().manifest_json().into();
     assert!(match_required(&empty_demands,&empty_custody,&still_required).is_err());
    });
    let matched=match_required(&demands,&custody,&r).unwrap();
    assert_eq!(matched.originals().len(),r.contracts.len());
    assert_eq!(matched.alternatives(),&BTreeMap::from([(scan_scope("s0","read"),BTreeSet::from(["all".into()])),(CoverageScope::Application,BTreeSet::from(["all".into()]))]));
    let (remaining_work,remaining_text)=crate::security_obligation_matching::test_budget(&demands,&custody,&r,1_000_000,16_000_000).unwrap();
    let work=1_000_000-remaining_work;let text=16_000_000-remaining_text;
    assert!(crate::security_obligation_matching::test_budget(&demands,&custody,&r,work,text).is_ok());
    assert!(crate::security_obligation_matching::test_budget(&demands,&custody,&r,work-1,text).is_err());
    assert!(crate::security_obligation_matching::test_budget(&demands,&custody,&r,work,text-1).is_err());
    // Both swapped scopes remain actually eligible; only original atom identity changes.
    let mut pair=None;
    for a in &r.atoms {for b in &r.atoms {
     if let (Subject::Semantic{source:sa,scope:qa},Subject::Semantic{source:sb,scope:qb})=(&a.subject,&b.subject) {
      if sa==sb&&qa!=qb {pair=Some((a.clone(),b.clone()));break;}
     }
    }if pair.is_some(){break;}}
    let (a,b)=pair.unwrap();let mut swap_a=a.clone();let mut swap_b=b.clone();
    swap_a.subject=b.subject.clone();swap_b.subject=a.subject.clone();
    let mut bad=r.clone();bad.atoms.remove(&a);bad.atoms.remove(&b);bad.atoms.insert(swap_a);bad.atoms.insert(swap_b);
    assert_eq!(bad.atoms.len(),r.atoms.len());
    assert_eq!(bad.atoms.iter().map(|a|a.subject.clone()).collect::<BTreeSet<_>>(),r.atoms.iter().map(|a|a.subject.clone()).collect());
    assert!(match_required(&demands,&custody,&bad).is_err());
    // Every-atom omission, foreign extra, full-contract drift and zero-edge duty.
    for atom in &r.atoms {let mut bad=r.clone();bad.atoms.remove(atom);assert!(match_required(&demands,&custody,&bad).is_err());}
    let mut bad=r.clone();bad.contracts.get_mut("deployment").unwrap().failure="changed".into();assert!(match_required(&demands,&custody,&bad).is_err());
    let mut bad=r.clone();bad.contracts.get_mut("deployment").unwrap().prerequisites.insert("semantic-0".into());assert!(match_required(&demands,&custody,&bad).is_err());
    let mut bad=r.clone();bad.selected_requirements.remove("unused-staff");assert!(match_required(&demands,&custody,&bad).is_err());
    let mut bad=r.clone();bad.selected_requirements.get_mut("unused-staff").unwrap().clear();assert!(match_required(&demands,&custody,&bad).is_err());
    let mut bad=r.clone();bad.selected.remove("unused-staff");assert!(match_required(&demands,&custody,&bad).is_err());
    let wrong_custody=ObligationCustody::collect(*coverage.declaration(),&["all".into()],true).unwrap();assert!(match_required(&demands,&wrong_custody,&r).is_err());
    let mut bad=r.clone();let mut extra=bad.atoms.first().unwrap().clone();extra.case="foreign-case".into();bad.atoms.insert(extra);assert!(match_required(&demands,&custody,&bad).is_err());
   });
  });
 }
 fn fixture_instances(base:&crate::security_obligation_matching::RequiredPremise)->crate::security_obligation_matching::InstancePremise {
  use crate::security_obligation_matching::{InstancePremise,DemandInstance,Subject};
  let mut r=InstancePremise{version:"weft.security.required-instances/0.2.0".into(),base:base.clone(),instances:BTreeMap::new()};
  // Fixed independently authored fixture kinds; never a production issuer.
  for atom in &base.atoms {if let Subject::Semantic{source,scope}=&atom.subject {
   let instance=DemandInstance{source:source.clone(),scope:scope.clone(),kind:if atom.obligation=="privacy-kind" {"privacy".into()}else{"codec".into()},occurrence:atom.obligation.clone()};
   r.instances.entry(instance).or_default().entry(atom.origin.clone()).or_default().insert(atom.clone());
  }}r
 }
 fn matching_population(ctx:&SecurityBackendContext<'_>,m:SecurityManifest,authored:&BTreeMap<(String,CoverageScope),String>,expected:&BTreeMap<(String,CoverageScope),String>,succeeds:bool){
  matching_population_instances(ctx,m,authored,expected,None,succeeds,succeeds);
 }
 fn matching_population_instances(ctx:&SecurityBackendContext<'_>,m:SecurityManifest,authored:&BTreeMap<(String,CoverageScope),String>,expected:&BTreeMap<(String,CoverageScope),String>,extra_origin:Option<&str>,succeeds:bool,instance_succeeds:bool){
  matching_population_occurrences(ctx,m,authored,expected,extra_origin,false,false,succeeds,instance_succeeds);
 }
 fn matching_population_occurrences(ctx:&SecurityBackendContext<'_>,mut m:SecurityManifest,authored:&BTreeMap<(String,CoverageScope),String>,expected:&BTreeMap<(String,CoverageScope),String>,extra_origin:Option<&str>,same_kind:bool,shared_primary:bool,succeeds:bool,instance_succeeds:bool){
  use crate::security_obligation_matching::{RequiredPremise,Contract,Atom,Subject,match_required};
  use crate::security_obligation_custody::ObligationCustody;
  use crate::backend::{Obligation,ObligationOwner};
  let mut a=m.capabilities[0].clone();a.id="cap-a".into();a.obligations.clear();
  let mut b=a.clone();b.id="cap-b".into();m.capabilities=vec![a,b];
  for (index,((source,scope),origin)) in authored.iter().enumerate(){
   let q=match scope {CoverageScope::Application=>json!({"kind":"application"}),CoverageScope::ScanAction{scan,action}=>json!({"kind":"scan-action","scan":scan,"action":action})};
   m.capabilities.iter_mut().find(|c|&c.id==origin).unwrap().obligations.push(Obligation{id:format!("demand-{index}"),owner:ObligationOwner::Host,failure_code:"WFT-POPULATION".into(),parameters:json!({"version":"weft.security.admission-obligation/0.2.0","subjects":[{"kind":"semantic","source":source,"scope":q}],"enforcementSite":"host","prerequisites":[],"evidenceCaseIds":["fixture-case","fixture-case-two"]})});
  }
  if shared_primary {
   let (index,_)=authored.iter().enumerate().find(|(_,((_,scope),_))|*scope==scan_scope("s0","read")).unwrap();
   let copy=m.capabilities[0].obligations.iter().find(|o|o.id==format!("demand-{index}")).unwrap().clone();
   m.capabilities[1].obligations.push(copy);
  }
  if let Some(origin)=extra_origin {
   let (index,_)=authored.iter().enumerate().find(|(_,((_,scope),_))|*scope==scan_scope("s0","read")).unwrap();
   let original=m.capabilities.iter().flat_map(|c|c.obligations.iter()).find(|o|o.id==format!("demand-{index}")).unwrap();
   let mut privacy=original.clone();privacy.id="privacy-kind".into();privacy.failure_code="WFT-PRIVACY".into();privacy.parameters["evidenceCaseIds"]=json!(["privacy-case"]);
   m.capabilities.iter_mut().find(|c|c.id==origin).unwrap().obligations.push(privacy);
  }
  with_registered_coverage(ctx,m,&["cap-a".into(),"cap-b".into()],|coverage|{
   let demands=crate::security_obligation_sources::issue_demands(coverage).unwrap();
   let custody=ObligationCustody::collect(*coverage.declaration(),&["cap-a".into(),"cap-b".into()],true).unwrap();
   let mut r=RequiredPremise{version:"weft.security.required-obligations/0.1.0".into(),profile_id:"population-profile".into(),registration_json:coverage.declaration().manifest_json().into(),target:"fixture".into(),selected:BTreeSet::from(["cap-a".into(),"cap-b".into()]),selected_requirements:BTreeMap::from([("cap-a".into(),BTreeSet::new()),("cap-b".into(),BTreeSet::new())]),contracts:BTreeMap::new(),atoms:BTreeSet::new()};
   // Expected profile is authored from its explicit input, never from decoded custody.
   for (index,((source,scope),origin)) in expected.iter().enumerate(){
    let id=format!("demand-{index}");
    r.contracts.insert(id.clone(),Contract{owner:ObligationOwner::Host,failure:"WFT-POPULATION".into(),prerequisites:BTreeSet::new(),site:"host".into(),semantic:true});
    let atom=Atom{obligation:id,origin:origin.clone(),subject:Subject::Semantic{source:source.clone(),scope:scope.clone()},site:"host".into(),case:"fixture-case".into()};
    r.atoms.insert(atom.clone());let mut second=atom;second.case="fixture-case-two".into();r.atoms.insert(second);
   }
   if shared_primary {
    let (index,((source,scope),_))=expected.iter().enumerate().find(|(_,((_,q),_))|*q==scan_scope("s0","read")).unwrap();
    for case in ["fixture-case","fixture-case-two"] {r.atoms.insert(Atom{obligation:format!("demand-{index}"),origin:"cap-b".into(),subject:Subject::Semantic{source:source.clone(),scope:scope.clone()},site:"host".into(),case:case.into()});}
    assert_eq!(custody.capability_sources(&format!("demand-{index}")).unwrap(),["cap-a","cap-b"]);
   }
   if let Some(origin)=extra_origin {
    let ((source,scope),_)=expected.iter().find(|((_,q),_)|*q==scan_scope("s0","read")).unwrap();
    r.contracts.insert("privacy-kind".into(),Contract{owner:ObligationOwner::Host,failure:"WFT-PRIVACY".into(),prerequisites:BTreeSet::new(),site:"host".into(),semantic:true});
    r.atoms.insert(Atom{obligation:"privacy-kind".into(),origin:origin.into(),subject:Subject::Semantic{source:source.clone(),scope:scope.clone()},site:"host".into(),case:"privacy-case".into()});
   }
   let mut instances=fixture_instances(&r);
   if same_kind {
    let key=instances.instances.keys().find(|i|i.kind=="privacy").unwrap().clone();
    let origins=instances.instances.remove(&key).unwrap();let mut second=key.clone();second.kind="codec".into();instances.instances.insert(second,origins);
    assert!(instances.instances.keys().any(|a|instances.instances.keys().any(|b|a.source==b.source&&a.scope==b.scope&&a.kind==b.kind&&a.occurrence!=b.occurrence)));
   }
   let instance_matched=crate::security_obligation_matching::match_instances(&demands,&custody,&instances);
   assert_eq!(instance_matched.is_ok(),instance_succeeds);
   if instance_succeeds {
    assert_eq!(instance_matched.unwrap().alternatives(),&BTreeMap::from([(scan_scope("s0","read"),BTreeSet::from(["cap-a".into()])),(CoverageScope::Application,BTreeSet::from(["cap-b".into()]))]));
    let (remaining_work,remaining_text)=crate::security_obligation_matching::test_instance_budget(&demands,&custody,&instances,1_000_000,16_000_000).unwrap();
    let work=1_000_000-remaining_work;let text=16_000_000-remaining_text;
    assert!(crate::security_obligation_matching::test_instance_budget(&demands,&custody,&instances,work,text).is_ok());
    assert!(crate::security_obligation_matching::test_instance_budget(&demands,&custody,&instances,work-1,text).is_err());
    assert!(crate::security_obligation_matching::test_instance_budget(&demands,&custody,&instances,work,text-1).is_err());
    for instance in instances.instances.keys(){let mut missing=instances.clone();missing.instances.remove(instance);assert!(crate::security_obligation_matching::match_instances(&demands,&custody,&missing).is_err());}
    // A real two-case expansion remains nonempty after this single omission.
    let mut partial=instances.clone();
    let atoms=partial.instances.values_mut().flat_map(|origins|origins.values_mut()).find(|atoms|atoms.len()==2).unwrap();
    let omitted=atoms.first().unwrap().clone();atoms.remove(&omitted);assert_eq!(atoms.len(),1);
    assert!(match_required(&demands,&custody,&partial.base).is_ok());
    assert!(crate::security_obligation_matching::match_instances(&demands,&custody,&partial).is_err());
    if shared_primary {
     let key=instances.instances.keys().find(|i|i.kind=="codec"&&instances.instances[*i].len()==2).unwrap().clone();
     let mut inconsistent=instances.clone();
     let atoms=inconsistent.instances.get_mut(&key).unwrap().remove("cap-b").unwrap();
     let privacy=inconsistent.instances.keys().find(|i|i.kind=="privacy"&&i.source==key.source&&i.scope==key.scope).unwrap().clone();
     inconsistent.instances.get_mut(&privacy).unwrap().insert("cap-b".into(),atoms);
     // cap-a still covers both demanded instances; other guards/partition still hold.
     assert!(inconsistent.instances.values().filter(|origins|origins.contains_key("cap-a")).count()>0);
     assert!(match_required(&demands,&custody,&inconsistent.base).is_ok());
     assert!(crate::security_obligation_matching::match_instances(&demands,&custody,&inconsistent).is_err());
    }
    let mut legacy=instances.clone();legacy.version="weft.security.required-instances/0.1.0".into();assert!(crate::security_obligation_matching::match_instances(&demands,&custody,&legacy).is_err());
   }
   let matched=match_required(&demands,&custody,&r);
   assert_eq!(matched.is_ok(),succeeds);
   if succeeds {
    let matched=matched.unwrap();
    assert_eq!(matched.alternatives(),&BTreeMap::from([(scan_scope("s0","read"),BTreeSet::from(["cap-a".into()])),(CoverageScope::Application,BTreeSet::from(["cap-b".into()]))]));
   }
  });
 }
 #[test]
 fn scoped_matching_refuses_agreeing_fragments_and_foreign_subjects_but_allows_distinct_complete_scopes(){
  with_context("SELECT r.salary FROM Resource r",|ctx,m|{
   let mut sources=BTreeMap::new();
   with_registered_coverage(ctx,m.clone(),&["all".into()],|coverage|{
    let demands=crate::security_obligation_sources::issue_demands(coverage).unwrap();
    sources=demands.demands().iter().map(|(s,qs)|(s.clone(),qs.iter().map(|q|(*q).clone()).collect::<BTreeSet<_>>())).collect();
   });
   let mut split=BTreeMap::new();
   for (index,(source,scopes)) in sources.iter().enumerate(){for scope in scopes {split.insert((source.clone(),scope.clone()),if index%2==0 {"cap-a".into()}else{"cap-b".into()});}}
   // Both declarations and independently authored requirements agree exactly.
   assert_eq!(split.values().cloned().collect::<BTreeSet<_>>(),BTreeSet::from(["cap-a".into(),"cap-b".into()]));
   matching_population(ctx,m.clone(),&split,&split,false);
   let complete:BTreeMap<_,_>=sources.iter().flat_map(|(source,scopes)|scopes.iter().map(move|scope|((source.clone(),scope.clone()),if *scope==CoverageScope::Application {"cap-b".into()}else{"cap-a".into()}))).collect();
   matching_population(ctx,m.clone(),&complete,&complete,true);
   let mut foreign_source=complete.clone();foreign_source.insert(("foreign-source".into(),scan_scope("s0","read")),"cap-a".into());
   matching_population(ctx,m.clone(),&foreign_source,&foreign_source,false);
   let mut foreign_scope=complete.clone();let source=sources.keys().next().unwrap().clone();foreign_scope.insert((source,scan_scope("foreign-occurrence","read")),"cap-a".into());
   matching_population(ctx,m,&foreign_scope,&foreign_scope,false);
  });
 }

 #[test]
 fn scoped_instance_matching_requires_all_kinds_at_one_origin(){
  with_context("SELECT r.salary FROM Resource r",|ctx,m|{
   let mut sources=BTreeMap::new();
   with_registered_coverage(ctx,m.clone(),&["all".into()],|coverage|{
    let demands=crate::security_obligation_sources::issue_demands(coverage).unwrap();
    sources=demands.demands().iter().map(|(s,qs)|(s.clone(),qs.iter().map(|q|(*q).clone()).collect::<BTreeSet<_>>())).collect();
   });
   let complete:BTreeMap<_,_>=sources.iter().flat_map(|(source,scopes)|scopes.iter().map(move|scope|((source.clone(),scope.clone()),if *scope==CoverageScope::Application {"cap-b".into()}else{"cap-a".into()}))).collect();
   // Both source-level relations match; only the full kind-instance choice differs.
   matching_population_instances(ctx,m.clone(),&complete,&complete,Some("cap-b"),true,false);
   matching_population_instances(ctx,m.clone(),&complete,&complete,Some("cap-a"),true,true);
   matching_population_occurrences(ctx,m.clone(),&complete,&complete,Some("cap-b"),true,false,true,false);
   matching_population_occurrences(ctx,m.clone(),&complete,&complete,Some("cap-a"),true,false,true,true);
   matching_population_occurrences(ctx,m,&complete,&complete,Some("cap-a"),false,true,true,true);
  });
 }

}
