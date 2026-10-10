//! Independently authored deployment qualification duties. Not semantic applicability,
//! authenticated profile selection, execution evidence or physical admission.
use std::collections::{BTreeMap,BTreeSet};
use crate::{backend::ObligationOwner,error::{Diagnostic,Result},security_case_catalog::CaseCatalog,
 security_requirement_templates::{Profile,Selector,Template}};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-LOWERING-UNSUPPORTED","capability","Authored deployment catalog refused")}
pub(crate) const VERSION:&str="weft.security.deployment-duty-catalog/0.1.0";
// Explicit authored tuples: original case suffix, duty kind, host verifier?,
// prerequisite case suffixes. No interpretation of case names/layers/status.
const SEMANTIC:[(&str,&str,&[&str]);12]=[
 ("S01","ontology-membership-resolution",&[]),
 ("S02","qualified-typed-key-identity",&["S01"]),
 ("S03","selected-policy-validation",&[]),
 ("S04","require-forbid-composition",&["S03"]),
 ("S05","ontology-path-semantics",&["S01","S02"]),
 ("S06","complete-fact-eligibility",&["S04","S05"]),
 ("S07","disclosure-state-fidelity",&["S01"]),
 ("S08","operator-mode-fidelity",&["S07"]),
 ("S09","selected-meaning-preservation",&[]),
 ("S10","portable-consumer-validation",&["S09"]),
 ("S11","corrupted-source-validation",&["S03","S05","S07","S08"]),
 ("S12","formal-semantic-analysis",&["S11"]),
];
const BACKEND:[(&str,&str,bool,&[&str]);30]=[
 ("B01","native-membership-enforcement",false,&["S01","S02","S03","S04","S05","B13"]),
 ("B02","native-identity-fidelity",false,&["S01","S02","S05"]),
 ("B03","collection-operator-eligibility",false,&["S03","S04","S05","S06"]),
 ("B04","native-bypass-closure",false,&["B13","B14"]),
 ("B05","native-invoker-definer-closure",false,&["B04"]),
 ("B06","native-view-function-closure",false,&["B04"]),
 ("B07","native-disclosure-fidelity",false,&["S07","S10"]),
 ("B08","native-operator-semantics",false,&["S08","B01","B07"]),
 ("B09","unsupported-activation-refusal",false,&["S09","B04"]),
 ("B10","private-observation-closure",false,&["B04","B05","B06"]),
 ("B11","installation-drift-refusal",false,&["B04","B12"]),
 ("B12","receipt-source-custody",true,&["S10","B13","B14"]),
 ("B13","native-collision-resistant-identity",false,&["S02"]),
 ("B14","excluded-admin-boundary",false,&["S09"]),
 ("B15","authority-dependency-corruption-refusal",false,&["S03","S04","S06","B01"]),
 ("B16","finite-resource-time-budget-refusal",false,&["B08","B15"]),
 ("L01","write-old-new-state",false,&["B01","B04"]),
 ("L02","field-write-authority",false,&["L01","B07"]),
 ("L03","revocation-final-drain",false,&["B10","B12"]),
 ("L04","streaming-revocation",false,&["L03"]),
 ("L05","post-acknowledgment-admission",false,&["L03"]),
 ("L06","old-snapshot-refusal",false,&["L05"]),
 ("L07","historical-owner-current-authority",false,&["L05","B02"]),
 ("L08","native-history-eligibility",false,&["L07","B09"]),
 ("L09","cursor-cache-rebinding",false,&["L05","L06"]),
 ("L10","derived-copy-feed-propagation",false,&["L03","B07","B12"]),
 ("L11","missing-custody-refusal",false,&["B03","L03"]),
 ("L12","activation-rollback",false,&["B04","B09","B11"]),
 ("L13","authority-change-exhaustiveness",false,&["B13","L05"]),
 ("L14","effective-date-transition",false,&["L03","L13"]),
];
fn case_id(backend:&str,suffix:&str)->String{if suffix.starts_with('S'){suffix.into()}else{format!("{backend}.{suffix}")}}
fn template_id(backend:&str,suffix:&str)->String{format!("deployment:{backend}:{suffix}")}
/// Fixed-size compiler-authored candidate map. Four homes use the same duty
/// interfaces; their implementations/procedures/evidence remain backend-owned.
#[allow(dead_code)]
pub(crate) fn authored(catalog:&CaseCatalog)->Result<BTreeMap<String,Template>>{
 let backend=catalog.backend();let mut out=BTreeMap::new();let mut cases=BTreeSet::new();
 for (suffix,kind,host,dependencies) in SEMANTIC.iter().map(|(s,k,d)|(*s,*k,true,*d)).chain(BACKEND.iter().copied()){
  let case=case_id(backend,suffix);catalog.original_case(&case)?;cases.insert(case.clone());
  out.insert(template_id(backend,suffix),Template{selector:Selector::Selected,kind:kind.into(),
   owner:if host{ObligationOwner::Host}else{ObligationOwner::Backend},site:if host{"host"}else{"native"}.into(),
   failure:"WFT-SECURITY-QUALIFICATION-REQUIRED".into(),cases:BTreeSet::from([case]),
   prerequisites:dependencies.iter().map(|d|template_id(backend,d)).collect()});
 }
 if cases!=*catalog.required()||out.len()!=42{return Err(fail());}
 Ok(out)
}
/// Every authored capability, including unchosen/zero-edge entries, must retain
/// all exact Selected duties. Semantic templates remain separate trusted inputs.
/// Adding agreeing manifest obligations cannot weaken this independent map.
#[allow(dead_code)]
pub(crate) fn validate(profile:&Profile,catalog:&CaseCatalog)->Result<()> {
 let mut visits=1_000_000usize;let mut text=16_000_000usize;
 validate_charged(profile,catalog,&mut |bytes|{if visits==0||text<bytes{return Err(fail());}visits-=1;text-=bytes;Ok(())})
}
/// Called only after the template issuer's charged profile preflight. Fixed
/// authored record copies reserve512 text bytes each before map construction;
/// all remaining comparisons/lookups consume that same issuer visit ledger.
pub(crate) fn validate_charged(profile:&Profile,catalog:&CaseCatalog,charge:&mut impl FnMut(usize)->Result<()>)->Result<()> {
 if profile.templates.len()>4096||profile.capabilities.is_empty()||profile.capabilities.len()>4096{return Err(fail());}
 for _ in 0..42{charge(512)?;}
 let expected=authored(catalog)?;
 let mut selected=0;for t in profile.templates.values(){charge(0)?;if t.selector==Selector::Selected{selected+=1;}}
 if selected!=expected.len(){return Err(fail());}
 for (id,duty) in &expected{charge(0)?;if profile.templates.get(id)!=Some(duty){return Err(fail());}}
 for ids in profile.capabilities.values(){
  charge(0)?;if ids.len()>4096{return Err(fail());}
  for id in expected.keys(){charge(0)?;if !ids.contains(id){return Err(fail());}}
 }
 Ok(())
}
#[cfg(test)]
mod tests{
 use super::*;
 const RAW:&str=include_str!("../tests/security-required-case-plan.json");
 const PIN:&str="37308740d5d8b57f6f4647c1a44feee71cbfde02d8f5bee1334299654a610ba9";
 fn catalog(backend:&str)->CaseCatalog{CaseCatalog::read(RAW,PIN,backend,&"0".repeat(64)).unwrap()}
 fn profile(c:&CaseCatalog)->Profile{
  let templates=authored(c).unwrap();let ids=templates.keys().cloned().collect();
  Profile{version:"weft.security.requirement-templates/0.1.0".into(),id:"authored-deployment-controls".into(),registration_json:"{}".into(),target:"fixture".into(),
   templates,capabilities:BTreeMap::from([("selected".into(),ids)]),cases:c.required().clone()}
 }
 #[test]
 fn all_four_homes_retain_single_case_duties_owners_and_explicit_prerequisites(){
  for backend in ["pg-raw","truss","delta-raw","ashlar"]{
   let c=catalog(backend);let p=profile(&c);validate(&p,&c).unwrap();assert_eq!(VERSION,"weft.security.deployment-duty-catalog/0.1.0");
   // Separate authored golden table; not generated by the Rust catalog.
   let mut golden=BTreeMap::new();
   for row in include_str!("../tests/security-deployment-duty-catalog.txt").lines(){
    let parts=row.split('|').collect::<Vec<_>>();assert_eq!(parts.len(),5);
    let original=if parts[0].starts_with('S'){parts[0].to_owned()}else{format!("{backend}.{}",parts[0])};
    let prerequisites=parts[4].split(',').filter(|s|!s.is_empty()).map(|s|format!("deployment:{backend}:{s}")).collect();
    golden.insert(format!("deployment:{backend}:{}",parts[0]),Template{selector:Selector::Selected,kind:parts[1].into(),owner:if parts[2]=="host"{ObligationOwner::Host}else{assert_eq!(parts[2],"backend");ObligationOwner::Backend},site:parts[3].into(),failure:"WFT-SECURITY-QUALIFICATION-REQUIRED".into(),cases:BTreeSet::from([original]),prerequisites});
   }
   assert!(p.templates==golden);assert_eq!(golden.len(),42);
   let cases=p.templates.values().flat_map(|t|t.cases.iter().cloned()).collect::<BTreeSet<_>>();assert_eq!(cases,*c.required());
   assert_eq!(p.templates.values().filter(|t|t.owner==ObligationOwner::Host&&t.site=="host").count(),13);
   assert_eq!(p.templates.values().filter(|t|t.owner==ObligationOwner::Backend&&t.site=="native").count(),29);
   assert!(p.templates.values().all(|t|t.cases.len()==1&&t.failure=="WFT-SECURITY-QUALIFICATION-REQUIRED"));
   let d=&p.templates[&format!("deployment:{backend}:L06")];assert_eq!(d.kind,"old-snapshot-refusal");assert_eq!(d.cases,BTreeSet::from([format!("{backend}.L06")]));assert_eq!(d.prerequisites,BTreeSet::from([format!("deployment:{backend}:L05")]));
   assert_eq!(p.templates[&format!("deployment:{backend}:B12")].prerequisites,BTreeSet::from([format!("deployment:{backend}:S10"),format!("deployment:{backend}:B13"),format!("deployment:{backend}:B14")]));
  }
 }
 #[test]
 fn every_duty_and_capability_omission_and_contract_substitution_refuses(){
  let c=catalog("pg-raw");let p=profile(&c);
  for id in p.templates.keys(){
   let mut missing=p.clone();missing.templates.remove(id);assert!(validate(&missing,&c).is_err());
   let mut missing=p.clone();missing.capabilities.get_mut("selected").unwrap().remove(id);assert!(validate(&missing,&c).is_err());
   for mutation in 0..7{let mut altered=p.clone();let t=altered.templates.get_mut(id).unwrap();match mutation{
    0=>t.kind="foreign".into(),1=>t.owner=if t.owner==ObligationOwner::Host{ObligationOwner::Backend}else{ObligationOwner::Host},
    2=>t.site="backend".into(),3=>t.failure="WFT-OTHER".into(),4=>t.cases=BTreeSet::from(["S01".into(),"S02".into()]),
    5=>t.prerequisites=BTreeSet::from([id.clone()]),_=>t.selector=Selector::False,
   }assert!(validate(&altered,&c).is_err());}
  }
  let mut extra=p.clone();extra.templates.insert("catch-all".into(),p.templates.values().next().unwrap().clone());assert!(validate(&extra,&c).is_err());
  let mut unchosen=p.clone();unchosen.capabilities.insert("unchosen".into(),BTreeSet::new());assert!(validate(&unchosen,&c).is_err());
  let wrong=catalog("truss");assert!(validate(&p,&wrong).is_err());
 }
 #[test]
 fn independently_counted_deployment_preflight_uses_one_shared_ledger(){
  let c=catalog("pg-raw");let p=profile(&c);
  //42 fixed reservations +42 template counts +42 comparisons +1 capability
  // +42 membership lookups =169visits;42*512 reserved text bytes.
  for (mut visits,mut text,ok) in [(169usize,21504usize,true),(168,21504,false),(169,21503,false)]{
   assert_eq!(validate_charged(&p,&c,&mut |bytes|{if visits==0||text<bytes{return Err(fail());}visits-=1;text-=bytes;Ok(())}).is_ok(),ok);
   if ok{assert_eq!((visits,text),(0,0));}
  }
 }

}
