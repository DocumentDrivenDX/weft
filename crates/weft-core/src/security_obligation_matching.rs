//! Conditional private correspondence. Required premise must be independently issued.
//! No native evidence, profile authentication, Permit, dispatch or release admission.
use std::collections::{BTreeMap,BTreeSet};
use crate::{backend::ObligationOwner,error::{Diagnostic,Result},security_obligation_custody::{ObligationCustody,ScopedDeclaration,ScopedSubject},security_obligation_sources::OwnerSourceDemands,security_semantic_coverage::CoverageScope};
fn fail()->Diagnostic {Diagnostic::new("WFT-SECURITY-LOWERING-UNSUPPORTED","capability","Scoped obligation correspondence refused")}
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) enum Subject { Semantic{source:String,scope:CoverageScope}, SelectedCapability{requirement:String} }
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) struct Atom {pub(crate) obligation:String,pub(crate) origin:String,pub(crate) subject:Subject,pub(crate) site:String,pub(crate) case:String}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Contract {pub(crate) owner:ObligationOwner,pub(crate) failure:String,pub(crate) prerequisites:BTreeSet<String>,pub(crate) site:String,pub(crate) semantic:bool}
/// Explicit trusted premise for the experiment, not an authenticated profile issuer.
/// Crate-local authoring only; never construct from the declaration under test.
#[derive(Clone)]
pub(crate) struct RequiredPremise {
 pub(crate) version:String,pub(crate) profile_id:String,
 pub(crate) registration_json:String,pub(crate) target:String,
 pub(crate) selected:BTreeSet<String>,
 pub(crate) selected_requirements:BTreeMap<String,BTreeSet<String>>,
 pub(crate) contracts:BTreeMap<String,Contract>,pub(crate) atoms:BTreeSet<Atom>,
}
#[allow(dead_code)]
pub(crate) struct Correspondence<'a,'d,'m,'c,'s> {
 demands:&'a OwnerSourceDemands<'d,'m,'c,'s>,custody:&'a ObligationCustody<'m>,required:&'a RequiredPremise,
 originals:Vec<ScopedDeclaration<'a>>,alternatives:BTreeMap<CoverageScope,BTreeSet<String>>,
}
#[allow(dead_code)]
impl Correspondence<'_, '_, '_, '_, '_> {
 pub(crate) fn alternatives(&self)->&BTreeMap<CoverageScope,BTreeSet<String>> {&self.alternatives}
 pub(crate) fn originals(&self)->&[ScopedDeclaration<'_>] {&self.originals}
}
struct Budget {work:usize,text:usize}
impl Budget {
 fn step(&mut self)->Result<()> {self.work=self.work.checked_sub(1).ok_or_else(fail)?;Ok(())}
 fn text(&mut self,s:&str)->Result<()> {self.step()?;self.text=self.text.checked_sub(s.len()).ok_or_else(fail)?;Ok(())}
 fn id(&mut self,s:&str)->Result<()> {self.text(s)?;if s.is_empty()||s.len()>4096||s.contains('\0'){return Err(fail());}Ok(())}
 fn scope(&mut self,q:&CoverageScope)->Result<()> {self.step()?;if let CoverageScope::ScanAction{scan,action}=q {self.id(scan)?;self.id(action)?;}Ok(())}
 fn subject(&mut self,s:&Subject)->Result<()> {match s {Subject::Semantic{source,scope}=>{self.id(source)?;self.scope(scope)?;},Subject::SelectedCapability{requirement}=>self.id(requirement)?}Ok(())}
 fn atom(&mut self,a:&Atom)->Result<()> {self.id(&a.obligation)?;self.id(&a.origin)?;self.subject(&a.subject)?;self.id(&a.site)?;self.id(&a.case)}
}
#[allow(dead_code)]
pub(crate) fn match_required<'a,'d,'m,'c,'s>(demands:&'a OwnerSourceDemands<'d,'m,'c,'s>,custody:&'a ObligationCustody<'m>,required:&'a RequiredPremise)->Result<Correspondence<'a,'d,'m,'c,'s>> {
 match_budget(demands,custody,required,&mut Budget{work:1_000_000,text:16_000_000})
}
fn match_budget<'a,'d,'m,'c,'s>(demands:&'a OwnerSourceDemands<'d,'m,'c,'s>,custody:&'a ObligationCustody<'m>,r:&'a RequiredPremise,b:&mut Budget)->Result<Correspondence<'a,'d,'m,'c,'s>> {
 let coverage=demands.coverage();let registration=coverage.declaration();
 custody.same_registration(registration)?;
 b.text(registration.manifest_json())?;b.text(&r.registration_json)?;b.id(&r.target)?;b.id(&r.profile_id)?;b.id(&r.version)?;
 if r.version!="weft.security.required-obligations/0.1.0"||r.registration_json!=registration.manifest_json()||r.target!=registration.target().id||r.selected!=*coverage.selected_capabilities()||r.selected.len()!=custody.selected_capabilities().len()||r.selected.iter().any(|id|!custody.selected_capabilities().contains(id.as_str())) {return Err(fail());}
 if r.atoms.is_empty()||r.atoms.len()>4096||r.contracts.is_empty()||r.contracts.len()>4096||r.selected_requirements.len()!=r.selected.len(){return Err(fail());}
 for selected in &r.selected {
  b.id(selected)?;
  let requirements=r.selected_requirements.get(selected).ok_or_else(fail)?;
  if requirements.len()>4096 {return Err(fail());}
  let mut has_edge=false;for candidates in coverage.assignments().values(){b.step()?;if candidates.contains(selected){has_edge=true;}}
  if !has_edge&&requirements.is_empty(){return Err(fail());}
  for requirement in requirements {
   b.id(requirement)?;let mut found=false;
   for a in &r.atoms {b.step()?;b.id(&a.origin)?;if let Subject::SelectedCapability{requirement:actual}=&a.subject {b.id(actual)?;if a.origin==*selected&&actual==requirement {found=true;break;}}}
   if !found{return Err(fail());}
  }
 }
 for (id,c) in &r.contracts {b.id(id)?;b.id(&c.failure)?;b.id(&c.site)?;if c.prerequisites.len()>4096{return Err(fail());}for p in &c.prerequisites {b.id(p)?;}}
 for atom in &r.atoms {b.atom(atom)?;if !r.selected.contains(&atom.origin)||!r.contracts.contains_key(&atom.obligation){return Err(fail());}}
 let originals=custody.project_scoped()?;
 if originals.len()!=r.contracts.len(){return Err(fail());}
 let mut declared=BTreeSet::new();
 for d in &originals {
  let expected=r.contracts.get(&d.original.id).ok_or_else(fail)?;
  b.id(&d.original.id)?;b.id(&d.original.failure_code)?;b.id(d.site)?;
  let semantic=matches!(d.subjects.first(),Some(ScopedSubject::Semantic{..}));
  if expected.owner!=d.original.owner||expected.failure!=d.original.failure_code||expected.site!=d.site||expected.semantic!=semantic||expected.prerequisites.len()!=d.prerequisites.len(){return Err(fail());}
  for p in &d.prerequisites {b.id(p)?;if !expected.prerequisites.contains(*p){return Err(fail());}}
  for origin in d.origins {for subject in &d.subjects {
   let owned=match subject {
    ScopedSubject::Semantic{source,scope}=>{
     b.id(source)?;b.scope(scope)?;
     if !demands.demands().get(*source).is_some_and(|scopes|scopes.contains(scope))||!demands.candidates(scope).is_some_and(|ids|ids.contains(*origin)){return Err(fail());}
     Subject::Semantic{source:(*source).to_owned(),scope:scope.clone()}
    },
    ScopedSubject::SelectedCapability{profile_requirement_id}=>{
     b.id(profile_requirement_id)?;
     if !r.selected_requirements.get(*origin).is_some_and(|ids|ids.contains(*profile_requirement_id)){return Err(fail());}
     Subject::SelectedCapability{requirement:(*profile_requirement_id).to_owned()}
    },
   };
   for case in &d.cases {
    // Charge the complete retained copy before allocating each expanded atom.
    b.id(&d.original.id)?;b.id(origin)?;b.subject(&owned)?;b.id(d.site)?;b.id(case)?;
    if declared.len()>=4096 {return Err(fail());}
    declared.insert(Atom{obligation:d.original.id.clone(),origin:(*origin).to_owned(),subject:owned.clone(),site:d.site.to_owned(),case:(*case).to_owned()});
   }
  } }
 }
 b.step()?;if declared!=r.atoms {return Err(fail());}
 let mut alternatives=BTreeMap::new();
 for (scope,candidates) in coverage.assignments() {
  b.scope(scope)?;let mut complete=BTreeSet::new();
  for candidate in candidates {
   b.id(candidate)?;let mut all=true;let mut population=false;
   for (source,scopes) in demands.demands() {b.id(source)?;if !scopes.contains(scope){continue;}population=true;
    // Exact atoms above preserve every obligation; this witnesses a whole origin.
    let mut found=false;for a in &declared {b.step()?;if a.origin==*candidate&&matches!(&a.subject,Subject::Semantic{source:s,scope:q} if s==source&&q==scope){found=true;break;}}
    if !found {all=false;break;}
   }
   if all&&population {b.id(candidate)?;complete.insert(candidate.clone());}
  }
  if complete.is_empty(){return Err(fail());}alternatives.insert(scope.clone(),complete);
 }
 Ok(Correspondence{demands,custody,required:r,originals,alternatives})
}

#[cfg(test)]
pub(crate) fn test_budget(demands:&OwnerSourceDemands<'_, '_, '_, '_>,custody:&ObligationCustody<'_>,required:&RequiredPremise,work:usize,text:usize)->Result<(usize,usize)> {
 let mut b=Budget{work,text};match_budget(demands,custody,required,&mut b)?;Ok((b.work,b.text))
}

/// Independently supplied requirement identity; never recovered from source text.
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) struct DemandInstance {
 pub(crate) source:String,pub(crate) scope:CoverageScope,
 pub(crate) kind:String,pub(crate) occurrence:String,
}
/// Separate private0.2 boundary. The nested0.1 premise remains source-level.
#[derive(Clone)]
pub(crate) struct InstancePremise {
 pub(crate) version:String,pub(crate) base:RequiredPremise,
 pub(crate) instances:BTreeMap<DemandInstance,BTreeMap<String,BTreeSet<Atom>>>,
}
#[allow(dead_code)]
pub(crate) struct InstanceCorrespondence<'a,'d,'m,'c,'s> {
 source:Correspondence<'a,'d,'m,'c,'s>,required:&'a InstancePremise,
 alternatives:BTreeMap<CoverageScope,BTreeSet<String>>,
}
#[allow(dead_code)]
impl InstanceCorrespondence<'_, '_, '_, '_, '_> {
 pub(crate) fn alternatives(&self)->&BTreeMap<CoverageScope,BTreeSet<String>> {&self.alternatives}
}
#[allow(dead_code)]
pub(crate) fn match_instances<'a,'d,'m,'c,'s>(demands:&'a OwnerSourceDemands<'d,'m,'c,'s>,custody:&'a ObligationCustody<'m>,required:&'a InstancePremise)->Result<InstanceCorrespondence<'a,'d,'m,'c,'s>> {
 instance_budget(demands,custody,required,&mut Budget{work:1_000_000,text:16_000_000})
}
fn instance_budget<'a,'d,'m,'c,'s>(demands:&'a OwnerSourceDemands<'d,'m,'c,'s>,custody:&'a ObligationCustody<'m>,r:&'a InstancePremise,b:&mut Budget)->Result<InstanceCorrespondence<'a,'d,'m,'c,'s>> {
 b.id(&r.version)?;
 if r.version!="weft.security.required-instances/0.2.0"||r.instances.is_empty()||r.instances.len()>4096{return Err(fail());}
 let source=match_required(demands,custody,&r.base)?;
 let mut projection=BTreeSet::new();let mut seen=BTreeSet::new();let mut identities=BTreeMap::new();let mut links=0usize;
 for (instance,origins) in &r.instances {
  b.id(&instance.source)?;b.scope(&instance.scope)?;b.id(&instance.kind)?;b.id(&instance.occurrence)?;
  if !demands.demands().get(&instance.source).is_some_and(|qs|qs.contains(&instance.scope))||origins.is_empty()||origins.len()>4096{return Err(fail());}
  b.step()?;projection.insert((instance.source.as_str(),&instance.scope));
  for (origin,atoms) in origins {
   b.id(origin)?;
   if atoms.is_empty()||atoms.len()>4096||!demands.candidates(&instance.scope).is_some_and(|cs|cs.contains(origin)){return Err(fail());}
   for atom in atoms {
    b.atom(atom)?;links=links.checked_add(1).ok_or_else(fail)?;if links>4096{return Err(fail());}
    if atom.origin!=*origin||!matches!(&atom.subject,Subject::Semantic{source:s,scope:q} if s==&instance.source&&q==&instance.scope)||!r.base.atoms.contains(atom)||!r.base.contracts.get(&atom.obligation).is_some_and(|c|c.semantic)||!seen.insert(atom){return Err(fail());}
    // Identical shared original declarations cannot acquire different kind meaning.
    let identity=(instance.kind.as_str(),instance.occurrence.as_str());
    if let Some(previous)=identities.insert(atom.obligation.as_str(),identity){if previous!=identity{return Err(fail());}}
   }
  }
 }
 // No actual demanded edge or globally required semantic atom may disappear.
 for (s,qs) in demands.demands(){b.id(s)?;for q in qs {b.scope(q)?;if !projection.contains(&(s.as_str(),*q)){return Err(fail());}}}
 for atom in &r.base.atoms {b.atom(atom)?;if matches!(atom.subject,Subject::Semantic{..})&&!seen.contains(atom){return Err(fail());}}
 let mut alternatives=BTreeMap::new();
 for (scope,candidates) in demands.coverage().assignments(){
  b.scope(scope)?;let mut complete=BTreeSet::new();
  for candidate in candidates {
   b.id(candidate)?;let mut all=true;let mut population=false;
   for (instance,origins) in &r.instances {b.step()?;if &instance.scope!=scope{continue;}population=true;
    // Every nonempty expansion above is exact and already matched globally.
    // One original must have a full expansion for EVERY kind/occurrence here.
    if !origins.contains_key(candidate){all=false;break;}
   }
   if all&&population {b.id(candidate)?;complete.insert(candidate.clone());}
  }
  if complete.is_empty(){return Err(fail());}alternatives.insert(scope.clone(),complete);
 }
 Ok(InstanceCorrespondence{source,required:r,alternatives})
}
#[cfg(test)]
pub(crate) fn test_instance_budget(demands:&OwnerSourceDemands<'_, '_, '_, '_>,custody:&ObligationCustody<'_>,required:&InstancePremise,work:usize,text:usize)->Result<(usize,usize)> {
 let mut b=Budget{work,text};instance_budget(demands,custody,required,&mut b)?;Ok((b.work,b.text))
}
