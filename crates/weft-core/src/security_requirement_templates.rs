//! Conditional owner/template expansion. No authenticated profile or native admission.
use std::collections::{BTreeMap,BTreeSet};
use crate::{backend::ObligationOwner,error::{Diagnostic,Result},security_obligation_sources::{OwnerSourceDemands,OwnerEventKind},security_obligation_matching::{RequiredPremise,InstancePremise,DemandInstance,Contract,Atom,Subject},security_rule_occurrences::{RulePath,RulePayload},security_ir::{Effect,Expression,Term,Disposition},security_semantic_coverage::CoverageScope};
fn fail()->Diagnostic {Diagnostic::new("WFT-SECURITY-LOWERING-UNSUPPORTED","capability","Independent template issuance refused")}
/// Structural applicability is dispatched on actual typed payloads, never source text.
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) enum Selector {
 Event(OwnerEventKind), Selected,
 Permit,Require,Forbid,True,False,Equal,And,Or,Not,Exists,
 Identity,Endpoint,StoredOperand,ContextOperand,Constant,Original,Withheld,Transformed,
}
#[derive(Clone,PartialEq,Eq)]
pub(crate) struct Template {
 pub(crate) selector:Selector,pub(crate) kind:String,pub(crate) owner:ObligationOwner,
 pub(crate) site:String,pub(crate) failure:String,pub(crate) cases:BTreeSet<String>,
 /// Initial subset: prerequisites instantiate at the identical selector/occurrence.
 pub(crate) prerequisites:BTreeSet<String>,
}
/// Independently authored trusted premise. Not deserialized from manifest obligations.
/// Closed case inventory and template completeness still require authenticated registration.
#[derive(Clone)]
pub(crate) struct Profile {
 pub(crate) version:String,pub(crate) id:String,pub(crate) registration_json:String,pub(crate) target:String,
 pub(crate) templates:BTreeMap<String,Template>,
 pub(crate) capabilities:BTreeMap<String,BTreeSet<String>>,
 pub(crate) cases:BTreeSet<String>,
}
#[allow(dead_code)]
pub(crate) struct Issued<'a,'d,'m,'c,'s> {
 owner:&'a OwnerSourceDemands<'d,'m,'c,'s>,profile:&'a Profile,required:InstancePremise,
}
#[allow(dead_code)]
impl Issued<'_, '_, '_, '_, '_> {
 pub(crate) fn required(&self)->&InstancePremise {&self.required}
}
/// Separate catalog-bound result; ordinary0.1 trusted-case issuance stays unchanged.
#[allow(dead_code)]
pub(crate) struct CatalogIssued<'a,'d,'m,'c,'s>{issued:Issued<'a,'d,'m,'c,'s>,catalog:&'a crate::security_case_catalog::CaseCatalog}
#[allow(dead_code)]
impl CatalogIssued<'_, '_, '_, '_, '_>{
 pub(crate) fn required(&self)->&InstancePremise{self.issued.required()}
 pub(crate) fn pending_cases(&self)->&BTreeSet<String>{self.catalog.required()}
 pub(crate) fn original_case(&self,id:&str)->Result<&serde_json::Value>{self.catalog.original_case(id)}
}
/// Hash/source correspondence is conditional on the independently trusted pins.
/// This does not authenticate either premise or verify a native case execution.
#[allow(dead_code)]
pub(crate) fn issue_catalog_bound<'a,'d,'m,'c,'s>(owner:&'a OwnerSourceDemands<'d,'m,'c,'s>,profile:&'a Profile,catalog:&'a crate::security_case_catalog::CaseCatalog)->Result<CatalogIssued<'a,'d,'m,'c,'s>>{
 catalog_preflight(owner,profile,catalog)?;
 Ok(CatalogIssued{issued:issue(owner,profile)?,catalog})
}
fn catalog_preflight(owner:&OwnerSourceDemands<'_, '_, '_, '_>,profile:&Profile,catalog:&crate::security_case_catalog::CaseCatalog)->Result<()> {
 let actual=owner.coverage().declaration().manifest_json();
 // Hash only bounded registered source; exact case IDs remain trusted premises.
 if actual.len()>1024*1024||profile.registration_json!=actual||crate::json::sha256(actual.as_bytes())!=catalog.registration_sha256()||profile.cases!=*catalog.required(){return Err(fail());}Ok(())
}
/// Stricter independent deployment-duty boundary; earlier experimental issuance
/// remains unchanged. All deployment duties stay pending without native evidence.
#[allow(dead_code)]
pub(crate) struct DeploymentIssued<'a,'d,'m,'c,'s>{catalog:CatalogIssued<'a,'d,'m,'c,'s>}
#[allow(dead_code)]
impl DeploymentIssued<'_, '_, '_, '_, '_>{
 pub(crate) fn deployment_version(&self)->&'static str{crate::security_deployment_catalog::VERSION}
 pub(crate) fn required(&self)->&InstancePremise{self.catalog.required()}
 pub(crate) fn pending_cases(&self)->&BTreeSet<String>{self.catalog.pending_cases()}
 pub(crate) fn original_case(&self,id:&str)->Result<&serde_json::Value>{self.catalog.original_case(id)}
}
#[allow(dead_code)]
pub(crate) fn issue_deployment_bound<'a,'d,'m,'c,'s>(owner:&'a OwnerSourceDemands<'d,'m,'c,'s>,profile:&'a Profile,catalog:&'a crate::security_case_catalog::CaseCatalog)->Result<DeploymentIssued<'a,'d,'m,'c,'s>>{
 catalog_preflight(owner,profile,catalog)?;
 let issued=issue_budget_deployment(owner,profile,1_000_000,16_000_000,Some(catalog))?.0;
 Ok(DeploymentIssued{catalog:CatalogIssued{issued,catalog}})
}
struct Budget{work:usize,text:usize}
impl Budget {
 fn charge(&mut self,bytes:usize)->Result<()> {if self.work==0||self.text<bytes{return Err(fail());}self.work-=1;self.text-=bytes;Ok(())}
 fn id(&mut self,s:&str)->Result<()> {self.charge(s.len())?;if s.is_empty()||s.len()>4096||s.contains('\0'){return Err(fail());}Ok(())}
 fn scope(&mut self,s:&CoverageScope)->Result<()> {self.charge(0)?;if let CoverageScope::ScanAction{scan,action}=s{self.id(scan)?;self.id(action)?;}Ok(())}
 fn encode(&mut self,tokens:&[&str])->Result<String> {
  let mut size=2usize+tokens.len().saturating_sub(1);
  for s in tokens{self.id(s)?;size=size.checked_add(2).ok_or_else(fail)?;for x in s.bytes(){size=size.checked_add(match x{b'"'|b'\\'|8|9|10|12|13=>2,0..=31=>6,_=>1}).ok_or_else(fail)?;}}
  if size>4096{return Err(fail());}self.charge(size)?;let out=serde_json::to_string(tokens).map_err(|_|fail())?;if out.len()!=size{return Err(fail());}Ok(out)
 }
 fn atom(&mut self,a:&Atom)->Result<()> {self.id(&a.obligation)?;self.id(&a.origin)?;self.id(&a.site)?;self.id(&a.case)?;match &a.subject{Subject::Semantic{source,scope}=>{self.id(source)?;self.scope(scope)?;},Subject::SelectedCapability{requirement}=>self.id(requirement)?}Ok(())}
}
fn node(payload:&RulePayload<'_>)->Selector {
 match payload {
  RulePayload::Rule(r)=>match r.effect{Effect::Permit=>Selector::Permit,Effect::Require=>Selector::Require,Effect::Forbid=>Selector::Forbid},
  RulePayload::Condition(e)=>match e{Expression::Literal(true)=>Selector::True,Expression::Literal(false)=>Selector::False,Expression::Equal(..)=>Selector::Equal,Expression::And(..)=>Selector::And,Expression::Or(..)=>Selector::Or,Expression::Not(..)=>Selector::Not,Expression::Exists{..}=>Selector::Exists},
  RulePayload::Operand(t)=>match t{Term::Identity{..}=>Selector::Identity,Term::Endpoint{..}=>Selector::Endpoint,Term::Field{..}=>Selector::StoredOperand,Term::Context{..}=>Selector::ContextOperand,Term::Constant{..}=>Selector::Constant},
  RulePayload::Disclosure{disposition,..}=>match disposition{Disposition::Original=>Selector::Original,Disposition::Withheld=>Selector::Withheld,Disposition::Transformed{..}=>Selector::Transformed},
 }
}
fn path(p:&RulePath,b:&mut Budget)->Result<String> {
 let (tag,indices,operand)=match p{RulePath::Rule=>("rule",&[][..],None),RulePath::Condition(v)=>("condition",v.as_slice(),None),RulePath::Operand(v,i)=>("operand",v.as_slice(),Some(*i as usize)),RulePath::Disclosure(i)=>("disclosure",&[][..],Some(*i))};
 b.id(tag)?;let mut tokens=vec![tag.to_owned()];
 for i in indices.iter().copied().chain(operand){b.charge(20)?;tokens.push(i.to_string());}
 b.encode(&tokens.iter().map(String::as_str).collect::<Vec<_>>())
}
fn validate(p:&Profile,b:&mut Budget)->Result<()> {
 b.id(&p.version)?;b.id(&p.id)?;b.id(&p.target)?;b.charge(p.registration_json.len())?;
 if p.version!="weft.security.requirement-templates/0.1.0"||p.templates.is_empty()||p.templates.len()>4096||p.cases.is_empty()||p.cases.len()>4096||p.capabilities.is_empty()||p.capabilities.len()>4096{return Err(fail());}
 for case in &p.cases{b.id(case)?;}
 for (id,t) in &p.templates{
  b.id(id)?;b.id(&t.kind)?;b.id(&t.site)?;b.id(&t.failure)?;
  if t.cases.is_empty()||t.cases.len()>4096||t.prerequisites.len()>4096{return Err(fail());}
  for c in &t.cases{b.id(c)?;if !p.cases.contains(c){return Err(fail());}}
  for dependency in &t.prerequisites{b.id(dependency)?;if p.templates.get(dependency).is_none_or(|d|d.selector!=t.selector){return Err(fail());}}
  // Bounded explicit traversal rejects cycles and excessive prerequisite paths.
  let mut stack=vec![(id.as_str(),Vec::<&str>::new())];
  while let Some((current,ancestors))=stack.pop(){b.id(current)?;if ancestors.len()>=64||ancestors.contains(&current){return Err(fail());}
   let next=p.templates.get(current).ok_or_else(fail)?;for d in &next.prerequisites{b.charge((ancestors.len()+1)*std::mem::size_of::<&str>())?;let mut a=ancestors.clone();a.push(current);stack.push((d,a));}
  }
 }
 for (cap,ids) in &p.capabilities{
  b.id(cap)?;if ids.is_empty()||ids.len()>4096{return Err(fail());}
  let mut qualified=BTreeSet::new();for id in ids{b.id(id)?;let t=p.templates.get(id).ok_or_else(fail)?;for dependency in &t.prerequisites{b.id(dependency)?;if !ids.contains(dependency){return Err(fail());}}if t.selector==Selector::Selected{for case in &t.cases{b.id(case)?;qualified.insert(case);}}}
  if qualified!=p.cases.iter().collect(){return Err(fail());}
 }
 Ok(())
}
struct Builder<'a>{profile:&'a Profile,budget:Budget,result:InstancePremise,links:usize}
impl Builder<'_>{
 fn identity(&mut self,template:&str,source:&str,scope:&CoverageScope,occurrence:&str)->Result<String> {
  match scope{CoverageScope::Application=>self.budget.encode(&["template",&self.profile.id,template,source,"application",occurrence]),CoverageScope::ScanAction{scan,action}=>self.budget.encode(&["template",&self.profile.id,template,source,"scan-action",scan,action,occurrence])}
 }
 fn emit(&mut self,selector:Selector,source:&str,scope:&CoverageScope,occurrence:&str,candidates:&BTreeSet<String>)->Result<()> {
  let mut applicable=false;
  for (template,t) in &self.profile.templates{
   self.budget.id(template)?;if t.selector!=selector{continue;}
   let mut origins=Vec::new();for cap in candidates{self.budget.id(cap)?;if self.profile.capabilities.get(cap).is_some_and(|ids|ids.contains(template)){origins.push(cap);}}
   if origins.is_empty(){if selector==Selector::Selected{continue;}return Err(fail());}applicable=true;
   let id=self.identity(template,source,scope,occurrence)?;let mut prerequisites=BTreeSet::new();
   for dependency in &t.prerequisites{prerequisites.insert(self.identity(dependency,source,scope,occurrence)?);}
   self.budget.id(&id)?;self.budget.id(&t.site)?;self.budget.id(&t.failure)?;for d in &prerequisites{self.budget.id(d)?;}
   if self.result.base.contracts.len()>=4096{return Err(fail());}
   let contract=Contract{owner:t.owner.clone(),failure:t.failure.clone(),prerequisites,site:t.site.clone(),semantic:selector!=Selector::Selected};
   if self.result.base.contracts.insert(id.clone(),contract).is_some(){return Err(fail());}
   for origin in origins{
    let subject=if selector==Selector::Selected{self.budget.id(template)?;Subject::SelectedCapability{requirement:template.clone()}}else{self.budget.id(source)?;self.budget.scope(scope)?;Subject::Semantic{source:source.to_owned(),scope:scope.clone()}};
    for case in &t.cases{
     self.budget.id(&id)?;self.budget.id(origin)?;self.budget.id(&t.site)?;self.budget.id(case)?;self.budget.charge(0)?;
     match &subject{Subject::Semantic{source,scope}=>{self.budget.id(source)?;self.budget.scope(scope)?;},Subject::SelectedCapability{requirement}=>self.budget.id(requirement)?}
     let atom=Atom{obligation:id.clone(),origin:origin.clone(),subject:subject.clone(),site:t.site.clone(),case:case.clone()};self.budget.atom(&atom)?;
     self.links=self.links.checked_add(1).ok_or_else(fail)?;if self.links>4096{return Err(fail());}
     self.result.base.atoms.insert(atom.clone());
     if selector==Selector::Selected{self.budget.id(template)?;self.result.base.selected_requirements.get_mut(origin).ok_or_else(fail)?.insert(template.clone());}else{
      self.budget.id(source)?;self.budget.scope(scope)?;self.budget.id(&t.kind)?;self.budget.id(template)?;self.budget.id(occurrence)?;
      let occurrence=self.budget.encode(&[template,occurrence])?;
      let instance=DemandInstance{source:source.to_owned(),scope:scope.clone(),kind:t.kind.clone(),occurrence};
      if !self.result.instances.contains_key(&instance)&&self.result.instances.len()>=4096{return Err(fail());}
      self.budget.id(origin)?;self.budget.atom(&atom)?;self.result.instances.entry(instance).or_default().entry(origin.clone()).or_default().insert(atom);
     }
    }
   }
  }
  if !applicable{return Err(fail());}Ok(())
 }
}
#[allow(dead_code)]
pub(crate) fn issue<'a,'d,'m,'c,'s>(owner:&'a OwnerSourceDemands<'d,'m,'c,'s>,profile:&'a Profile)->Result<Issued<'a,'d,'m,'c,'s>>{issue_budget(owner,profile,1_000_000,16_000_000).map(|(out,_,_)|out)}
fn issue_budget<'a,'d,'m,'c,'s>(owner:&'a OwnerSourceDemands<'d,'m,'c,'s>,profile:&'a Profile,work:usize,text:usize)->Result<(Issued<'a,'d,'m,'c,'s>,usize,usize)> {
 issue_budget_deployment(owner,profile,work,text,None)
}
fn issue_budget_deployment<'a,'d,'m,'c,'s>(owner:&'a OwnerSourceDemands<'d,'m,'c,'s>,profile:&'a Profile,work:usize,text:usize,deployment:Option<&crate::security_case_catalog::CaseCatalog>)->Result<(Issued<'a,'d,'m,'c,'s>,usize,usize)> {
 let mut budget=Budget{work,text};validate(profile,&mut budget)?;
 // The same charged preflight bounds all IDs/maps before deployment comparison.
 if let Some(catalog)=deployment{crate::security_deployment_catalog::validate_charged(profile,catalog,&mut |bytes|budget.charge(bytes))?;}
 let coverage=owner.coverage();budget.charge(coverage.declaration().manifest_json().len())?;
 if profile.registration_json!=coverage.declaration().manifest_json()||profile.target!=coverage.declaration().target().id{return Err(fail());}
 for cap in coverage.selected_capabilities(){budget.id(cap)?;if !profile.capabilities.contains_key(cap){return Err(fail());}}
 // Charge every retained header/selection copy before allocating it.
 budget.id("weft.security.required-obligations/0.1.0")?;budget.id(&profile.id)?;budget.charge(profile.registration_json.len())?;budget.id(&profile.target)?;
 for cap in coverage.selected_capabilities(){budget.id(cap)?;budget.id(cap)?;}
 let base=RequiredPremise{version:"weft.security.required-obligations/0.1.0".into(),profile_id:profile.id.clone(),registration_json:profile.registration_json.clone(),target:profile.target.clone(),selected:coverage.selected_capabilities().clone(),selected_requirements:coverage.selected_capabilities().iter().map(|c|(c.clone(),BTreeSet::new())).collect(),contracts:BTreeMap::new(),atoms:BTreeSet::new()};
 budget.id("weft.security.required-instances/0.2.0")?;
 let mut b=Builder{profile,budget,result:InstancePremise{version:"weft.security.required-instances/0.2.0".into(),base,instances:BTreeMap::new()},links:0};
 for (source,kind) in owner.events(){
  b.budget.id(source)?;let scopes=owner.demands().get(source).ok_or_else(fail)?;if scopes.is_empty(){return Err(fail());}
  for scope in scopes{b.emit(Selector::Event(*kind),source,scope,"event",owner.candidates(scope).ok_or_else(fail)?)?;}
 }
 // Structural rules expand all nodes, including false/empty branches and disclosures.
 let rules=crate::security_rule_occurrences::issue(owner)?;
 for ((source,address),payload) in rules.entries(){let occurrence=path(address,&mut b.budget)?;for scope in owner.demands().get(*source).ok_or_else(fail)?{b.emit(node(payload),source,scope,&occurrence,owner.candidates(scope).ok_or_else(fail)?)?;}}
 for cap in coverage.selected_capabilities(){b.budget.id(cap)?;b.emit(Selector::Selected,cap,&CoverageScope::Application,"deployment",&BTreeSet::from([cap.clone()]))?;}
 let remaining=(b.budget.work,b.budget.text);Ok((Issued{owner,profile,required:b.result},remaining.0,remaining.1))
}
#[cfg(test)]
pub(crate) fn test_budget(owner:&OwnerSourceDemands<'_, '_, '_, '_>,p:&Profile,work:usize,text:usize)->Result<(usize,usize)>{issue_budget(owner,p,work,text).map(|(_,w,t)|(w,t))}
#[cfg(test)]
mod tests {
 use super::*;
 fn profile()->Profile{
  let t=Template{selector:Selector::Selected,kind:"deployment".into(),owner:ObligationOwner::Backend,site:"native".into(),failure:"WFT-TEST".into(),cases:BTreeSet::from(["c1".into(),"c2".into()]),prerequisites:BTreeSet::new()};
  Profile{version:"weft.security.requirement-templates/0.1.0".into(),id:"p".into(),registration_json:"{}".into(),target:"t".into(),templates:BTreeMap::from([("a-duty".into(),t.clone()),("b-duty".into(),t)]),capabilities:BTreeMap::from([("a".into(),BTreeSet::from(["a-duty".into()])),("b".into(),BTreeSet::from(["b-duty".into()]))]),cases:BTreeSet::from(["c1".into(),"c2".into()])}
 }
 fn builder(p:&Profile)->Builder<'_>{Builder{profile:p,budget:Budget{work:1_000_000,text:16_000_000},links:0,result:InstancePremise{version:"weft.security.required-instances/0.2.0".into(),base:RequiredPremise{version:"weft.security.required-obligations/0.1.0".into(),profile_id:p.id.clone(),registration_json:p.registration_json.clone(),target:p.target.clone(),selected:BTreeSet::from(["a".into()]),selected_requirements:BTreeMap::from([("a".into(),BTreeSet::new())]),contracts:BTreeMap::new(),atoms:BTreeSet::new()},instances:BTreeMap::new()}}}
 #[test]
 fn unselected_deployment_template_is_not_selected_capability_duty(){
  let p=profile();validate(&p,&mut Budget{work:1000,text:10000}).unwrap();let mut b=builder(&p);
  b.emit(Selector::Selected,"a",&CoverageScope::Application,"deployment",&BTreeSet::from(["a".into()])).unwrap();
  assert_eq!(b.result.base.contracts.keys().cloned().collect::<Vec<_>>(),vec!["[\"template\",\"p\",\"a-duty\",\"a\",\"application\",\"deployment\"]"]);
  assert_eq!(b.result.base.atoms.len(),2);assert_eq!(b.result.base.selected_requirements["a"],BTreeSet::from(["a-duty".into()]));assert!(b.result.instances.is_empty());
 }
 #[test]
 fn transitive_missing_dependency_refuses_instead_of_panicking(){
  let mut p=profile();p.templates.get_mut("a-duty").unwrap().prerequisites.insert("b-duty".into());p.templates.get_mut("b-duty").unwrap().prerequisites.insert("missing".into());
  assert!(validate(&p,&mut Budget{work:1000,text:10000}).is_err());
 }
 #[test]
 fn exact_atom_boundary_and_typed_address_distinctions(){
  let mut p=profile();p.templates.remove("b-duty");p.templates.get_mut("a-duty").unwrap().selector=Selector::False;p.templates.get_mut("a-duty").unwrap().cases=BTreeSet::from(["c1".into()]);
  let mut b=builder(&p);let candidates=BTreeSet::from(["a".into()]);
  for i in 0..4096{b.emit(Selector::False,&format!("source-{i}"),&CoverageScope::Application,"node",&candidates).unwrap();}
  assert_eq!(b.result.instances.len(),4096);assert_eq!(b.result.base.atoms.len(),4096);assert!(b.emit(Selector::False,"overflow",&CoverageScope::Application,"node",&candidates).is_err());
  let mut b=builder(&p);
  for address in [RulePath::Condition(vec![0]),RulePath::Condition(vec![1]),RulePath::Operand(vec![0],0),RulePath::Operand(vec![0],1),RulePath::Disclosure(0)]{
   let address=path(&address,&mut b.budget).unwrap();b.emit(Selector::False,"source",&CoverageScope::Application,&address,&candidates).unwrap();
  }
  assert_eq!(b.result.instances.len(),5);assert_eq!(b.result.base.contracts.len(),5);
 }

 #[test]
 fn aggregate_link_boundary_includes_shared_origins_cases_and_deployment(){
  let mut p=profile();p.templates.remove("b-duty");p.templates.get_mut("a-duty").unwrap().selector=Selector::False;
  let mut deployment=p.templates["a-duty"].clone();deployment.selector=Selector::Selected;p.templates.insert("deployment".into(),deployment);
  for ids in p.capabilities.values_mut(){*ids=BTreeSet::from(["a-duty".into(),"deployment".into()]);}
  let mut b=builder(&p);b.result.base.selected.insert("b".into());b.result.base.selected_requirements.insert("b".into(),BTreeSet::new());
  let candidates=BTreeSet::from(["a".into(),"b".into()]);
  for i in 0..1023{b.emit(Selector::False,&format!("source-{i}"),&CoverageScope::Application,"node",&candidates).unwrap();}
  for cap in ["a","b"]{b.emit(Selector::Selected,cap,&CoverageScope::Application,"deployment",&BTreeSet::from([cap.into()])).unwrap();}
  assert_eq!(b.links,4096);assert_eq!(b.result.base.atoms.len(),4096);assert_eq!(b.result.base.contracts.len(),1025);assert_eq!(b.result.instances.len(),1023);
  assert!(b.emit(Selector::False,"overflow",&CoverageScope::Application,"node",&candidates).is_err());
 }
 #[test]
 fn independently_counted_small_encoding_and_retention_ledgers(){
  for (work,text,ok) in [(3,11,true),(2,11,false),(3,10,false)]{let mut b=Budget{work,text};assert_eq!(b.encode(&["a","b"]).is_ok(),ok);if ok{assert_eq!((b.work,b.text),(0,0));}}
  // Tag10 + fixed index allowance20 + token bytes11 + JSON18 =59, five visits.
  for (work,text,ok) in [(5,59,true),(4,59,false),(5,58,false)]{let mut b=Budget{work,text};assert_eq!(path(&RulePath::Disclosure(2),&mut b).is_ok(),ok);if ok{assert_eq!((b.work,b.text),(0,0));}}
  // One Selected template, one origin, two cases:13 header visits/176 bytes,
  // then12 visits/148 bytes per retained case (includes full atom copy).
  let mut p=profile();p.templates.remove("b-duty");
  for (work,text,ok) in [(37,472,true),(36,472,false),(37,471,false)]{
   let mut b=builder(&p);b.budget=Budget{work,text};assert_eq!(b.emit(Selector::Selected,"a",&CoverageScope::Application,"deployment",&BTreeSet::from(["a".into()])).is_ok(),ok);
   if ok{assert_eq!((b.budget.work,b.budget.text),(0,0));assert_eq!(b.result.base.atoms.len(),2);}
  }
 }
}
