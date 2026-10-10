//! Pinned original required-case custody, not execution or profile authentication.
use std::collections::{BTreeMap,BTreeSet};
use serde_json::Value;
use crate::{error::{Diagnostic,Result},json::{checked_json_bounded_charged,sha256}};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-LOWERING-UNSUPPORTED","capability","Required case catalog admission refused")}
const BACKENDS:[&str;4]=["pg-raw","truss","delta-raw","ashlar"];
struct Budget{work:usize,text:usize}
impl Budget{
 fn charge(&mut self,bytes:usize)->Result<()> {if self.work==0||self.text<bytes{return Err(fail());}self.work-=1;self.text-=bytes;Ok(())}
 fn id(&mut self,s:&str)->Result<()> {self.charge(s.len())?;if s.is_empty()||s.len()>4096||s.contains('\0'){return Err(fail());}Ok(())}
 fn value(&mut self,v:&Value)->Result<()> {self.charge(0)?;match v{Value::String(s)=>self.id(s)?,Value::Array(a)=>for v in a{self.value(v)?;},Value::Object(o)=>for(k,v)in o{self.id(k)?;self.value(v)?;},_=>{}}Ok(())}
}
fn keys(v:&Value,names:&[&str])->Result<()> {let o=v.as_object().ok_or_else(fail)?;if o.len()!=names.len()||names.iter().any(|n|!o.contains_key(*n)){return Err(fail());}Ok(())}
fn digest(s:&str)->bool{s.len()==64&&s.bytes().all(|x|x.is_ascii_digit()||(b'a'..=b'f').contains(&x))}
fn expected(b:&str)->BTreeSet<String>{
 if b=="semantic"{(1..=12).map(|i|format!("S{i:02}")).collect()}else{(1..=16).map(|i|format!("{b}.B{i:02}")).chain((1..=14).map(|i|format!("{b}.L{i:02}"))).collect()}
}
fn string_list(v:&Value,nonempty:bool,max:usize)->Result<BTreeSet<&str>>{
 let values=v.as_array().ok_or_else(fail)?;if values.len()>max||(nonempty&&values.is_empty()){return Err(fail());}let mut unique=BTreeSet::new();
 for v in values{let s=v.as_str().ok_or_else(fail)?;if s.is_empty()||!unique.insert(s){return Err(fail());}}Ok(unique)
}
/// Immutable full-plan snapshot. IDs select requirements only, never evidence.
#[allow(dead_code)]
pub(crate) struct CaseCatalog {
 raw:String,plan:Value,backend:String,plan_sha256:String,registration_sha256:String,
 required:BTreeSet<String>,indices:BTreeMap<String,usize>,
}
#[allow(dead_code)]
impl CaseCatalog{
 /// Both expected hashes and backend mapping are independently trusted host premises.
 /// An agreeing caller-supplied hash does not authenticate this catalog.
 pub(crate) fn read(raw:&str,expected_plan_sha256:&str,backend:&str,expected_registration_sha256:&str)->Result<Self>{Self::read_budget(raw,expected_plan_sha256,backend,expected_registration_sha256,1_000_000,16_000_000).map(|(c,_,_)|c)}
 fn read_budget(raw:&str,pin:&str,backend:&str,registration:&str,work:usize,text:usize)->Result<(Self,usize,usize)>{
  let mut b=Budget{work,text};b.charge(raw.len())?;b.id(pin)?;b.id(backend)?;b.id(registration)?;
  if raw.len()>4_000_000||!digest(pin)||!digest(registration)||!BACKENDS.contains(&backend)||sha256(raw.as_bytes())!=pin{return Err(fail());}
  let plan=checked_json_bounded_charged(raw,4_000_000,64,&mut b.work).map_err(|_|fail())?;b.value(&plan)?;
  keys(&plan,&["version","scope","cases","runnerContract"])?;if plan["version"]!="0.1.0"{return Err(fail());}
  let rows=plan["cases"].as_array().ok_or_else(fail)?;if rows.len()!=132{return Err(fail());}
  let mut all=BTreeMap::<&str,BTreeSet<&str>>::new();let mut indices=BTreeMap::new();
  for (i,c) in rows.iter().enumerate(){
   b.charge(0)?;keys(c,&["id","backend","covers","name","assertion","required","layer","status","command","evidence","testSource","oracleSource","implementationSources","timeoutMs","assertionIds"])?;
   let id=c["id"].as_str().ok_or_else(fail)?;let home=c["backend"].as_str().ok_or_else(fail)?;
   if home!="semantic"&&!BACKENDS.contains(&home){return Err(fail());}
   if c["required"]!=true||!all.entry(home).or_default().insert(id){return Err(fail());}
   for key in ["name","assertion","layer","status"]{if c[key].as_str().is_none_or(str::is_empty){return Err(fail());}}
   string_list(&c["covers"],true,256)?;let assertions=string_list(&c["assertionIds"],true,256)?;
   let expected_assertions=if id=="S10"{BTreeSet::from(["S10","S10:disclosure"])}else{BTreeSet::from([id])};if assertions!=expected_assertions{return Err(fail());}
   if !c["command"].is_null(){let argv=c["command"].as_array().ok_or_else(fail)?;if argv.is_empty()||argv.len()>256||argv.iter().any(|v|v.as_str().is_none_or(str::is_empty)){return Err(fail());}}
   for key in ["testSource","oracleSource","evidence"]{if !c[key].is_null()&&c[key].as_str().is_none_or(str::is_empty){return Err(fail());}}
   string_list(&c["implementationSources"],false,4096)?;
   if c["timeoutMs"].as_u64().is_none_or(|n|n==0||n>900_000){return Err(fail());}
   if home=="semantic"||home==backend{b.id(id)?;indices.insert(id.to_owned(),i);}
  }
  for home in ["semantic"].into_iter().chain(BACKENDS){let actual=all.get(home).ok_or_else(fail)?;let required=expected(home);if actual.iter().copied().collect::<BTreeSet<_>>()!=required.iter().map(String::as_str).collect(){return Err(fail());}}
  if all.len()!=5||indices.len()!=42{return Err(fail());}
  let mut required=BTreeSet::new();for id in indices.keys(){b.id(id)?;required.insert(id.clone());}
  // Frozen source and all selected identities are copied only after their charges.
  b.charge(raw.len())?;b.id(backend)?;b.id(pin)?;b.id(registration)?;
  let remaining=(b.work,b.text);Ok((Self{raw:raw.to_owned(),plan,backend:backend.to_owned(),plan_sha256:pin.to_owned(),registration_sha256:registration.to_owned(),required,indices},remaining.0,remaining.1))
 }
 pub(crate) fn required(&self)->&BTreeSet<String>{&self.required}
 pub(crate) fn plan_sha256(&self)->&str{&self.plan_sha256}
 pub(crate) fn registration_sha256(&self)->&str{&self.registration_sha256}
 pub(crate) fn backend(&self)->&str{&self.backend}
 pub(crate) fn original_case(&self,id:&str)->Result<&Value>{let index=self.indices.get(id).ok_or_else(fail)?;self.plan["cases"].get(*index).ok_or_else(fail)}
}
#[cfg(test)]
mod tests{
 use super::*;
 const RAW:&str=include_str!("../tests/security-required-case-plan.json");
 const PIN:&str="37308740d5d8b57f6f4647c1a44feee71cbfde02d8f5bee1334299654a610ba9";
 fn reg()->String{sha256(b"fixture-registration")}
 #[test]
 fn original_plan_keeps_all_four_backend_inventories_and_full_assertions(){
  assert_eq!(sha256(RAW.as_bytes()),PIN);
  for backend in BACKENDS{
   let c=CaseCatalog::read(RAW,PIN,backend,&reg()).unwrap();assert_eq!(c.required.len(),42);
   let ids=(1..=12).map(|i|format!("S{i:02}")).chain((1..=16).map(|i|format!("{backend}.B{i:02}"))).chain((1..=14).map(|i|format!("{backend}.L{i:02}"))).collect::<BTreeSet<_>>();assert_eq!(c.required,ids);
   assert_eq!(c.original_case("S10").unwrap()["assertionIds"],serde_json::json!(["S10","S10:disclosure"]));
   let id=format!("{backend}.B10");let case=c.original_case(&id).unwrap();assert!(case["assertion"].as_str().unwrap().contains("diagnostic"));
   assert!(c.original_case("foreign").is_err());let other=if backend=="truss"{"pg-raw.B10"}else{"truss.B10"};assert!(c.original_case(other).is_err());assert_eq!(c.raw,RAW);
  }
  let c=CaseCatalog::read(RAW,PIN,"pg-raw",&reg()).unwrap();assert_eq!(c.original_case("pg-raw.B10").unwrap()["status"],"counterexample-found");assert!(c.original_case("pg-raw.B10").unwrap()["command"].is_null());
 }
 #[test]
 fn exact_source_pins_and_all_original_case_omissions_refuse(){
  assert!(CaseCatalog::read(RAW,&"0".repeat(64),"pg-raw",&reg()).is_err());assert!(CaseCatalog::read(RAW,PIN,"foreign",&reg()).is_err());assert!(CaseCatalog::read(RAW,PIN,"pg-raw","short").is_err());
  let source:Value=serde_json::from_str(RAW).unwrap();
  // Fresh mutant hashes isolate structural completeness from exact source pin refusal.
  for index in 0..132{let mut shortened=source.clone();shortened["cases"].as_array_mut().unwrap().remove(index);let raw=shortened.to_string();assert!(CaseCatalog::read(&raw,&sha256(raw.as_bytes()),"pg-raw",&reg()).is_err());}
  for field in ["required","assertionIds","backend","id","assertion"]{let mut bad=source.clone();bad["cases"][0][field]=match field{"required"=>Value::Bool(false),"assertionIds"=>serde_json::json!(["foreign"]),"assertion"=>Value::String(String::new()),_=>Value::String("foreign".into())};let raw=bad.to_string();assert!(CaseCatalog::read(&raw,&sha256(raw.as_bytes()),"pg-raw",&reg()).is_err());}
  let mut repeated=source.clone();repeated["cases"][1]=repeated["cases"][0].clone();let raw=repeated.to_string();assert_eq!(repeated["cases"].as_array().unwrap().len(),132);assert!(CaseCatalog::read(&raw,&sha256(raw.as_bytes()),"pg-raw",&reg()).is_err());
  let mut secondary=source.clone();secondary["cases"].as_array_mut().unwrap().iter_mut().find(|c|c["id"]=="S10").unwrap()["assertionIds"]=serde_json::json!(["S10"]);let raw=secondary.to_string();assert!(CaseCatalog::read(&raw,&sha256(raw.as_bytes()),"pg-raw",&reg()).is_err());
  let duplicate=RAW.replacen("\"version\": \"0.1.0\"","\"version\": \"0.1.0\", \"version\": \"0.1.0\"",1);assert_ne!(duplicate,RAW);assert!(CaseCatalog::read(&duplicate,&sha256(duplicate.as_bytes()),"pg-raw",&reg()).is_err());
 }
 #[test]
 fn catalog_exact_minus_one_ledgers_and_raw_snapshot_bounds(){
  let (_,work,text)=CaseCatalog::read_budget(RAW,PIN,"pg-raw",&reg(),1_000_000,16_000_000).unwrap();let exact_work=1_000_000-work;let exact_text=16_000_000-text;
  assert!(CaseCatalog::read_budget(RAW,PIN,"pg-raw",&reg(),exact_work,exact_text).is_ok());assert!(CaseCatalog::read_budget(RAW,PIN,"pg-raw",&reg(),exact_work-1,exact_text).is_err());assert!(CaseCatalog::read_budget(RAW,PIN,"pg-raw",&reg(),exact_work,exact_text-1).is_err());
  let large=" ".repeat(4_000_001);assert!(CaseCatalog::read(&large,&sha256(large.as_bytes()),"pg-raw",&reg()).is_err());
 }
}
