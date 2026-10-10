//! Pure direct-field declaration correspondence. Never authorizes result selection.
use crate::{application_ir::Expression, error::{Diagnostic,Result}, security_backend::SecurityBackendContext, security_ir::{Disposition,Effect}, security_literals::{normalized_literal,ScalarLiteral}, security_lowering::{SecurityIdentity,SecurityResultContract,SecurityResultDomain,SecurityResultOutcome,SecurityTransform}, security_ontology::{SecurityOntologyClosure,SecurityRef,locate,domain}};
use serde_json::Value;
use std::collections::BTreeSet;
fn fail()->Diagnostic { Diagnostic::new("WFT-SECURITY-LOWERING-UNSUPPORTED","capability","Security result declaration correspondence refused") }
fn same(a:&SecurityIdentity,b:&crate::ir::Identity)->bool { a.document_id==b.document_id&&a.revision==b.revision&&a.module==b.module&&a.element==b.element }
fn matches(a:&SecurityIdentity,b:&SecurityRef,ctx:&SecurityBackendContext<'_>)->bool { a.document_id==b.document_id&&a.module==b.module_id&&a.element==b.element_id&&ctx.catalog().pins().iter().any(|p|p.document_id==a.document_id&&p.revision==a.revision) }
fn model_domain(d:&SecurityResultDomain,field:&SecurityRef,ctx:&SecurityBackendContext<'_>)->bool { matches!(d,SecurityResultDomain::Model{field:f} if matches(f,field,ctx)) }
pub(crate) struct Budget { work:usize,bytes:usize }
impl Budget {
 pub(crate) fn new()->Self{Self{work:1_000_000,bytes:16_000_000}}
 pub(crate) fn charge(&mut self,n:usize)->Result<()> { if self.work==0||n>self.bytes{return Err(fail());} self.work-=1;self.bytes-=n;Ok(()) }
 pub(crate) fn text(&mut self,s:&str)->Result<()> { if s.is_empty()||s.contains('\0')||s.chars().count()>4096{return Err(fail());}self.charge(s.len()) }
 fn identity(&mut self,i:&SecurityIdentity)->Result<()> { for s in [&i.document_id,&i.revision,&i.module,&i.element]{self.text(s)?;}Ok(()) }
 pub(crate) fn normalized(&mut self,value:&Option<ScalarLiteral>)->Result<()> {
  let bytes=match value {None=>1,Some(ScalarLiteral::Boolean(_))=>1,Some(ScalarLiteral::String(s)|ScalarLiteral::Binary(s))=>s.len(),Some(ScalarLiteral::Number(n))=>usize::try_from(n.magnitude().bits().div_ceil(64)).ok().and_then(|words|words.checked_mul(8)).ok_or_else(fail)?};
  self.charge(bytes)
 }
 pub(crate) fn literal(&mut self,value:&Value)->Result<()> {
  // Bound caller-owned declarations before normalization or serialization.
  let mut stack=vec![(value,1usize)];
  while let Some((v,depth))=stack.pop(){self.charge(0)?;if depth>64{return Err(fail());}
   match v { Value::String(s)=>{if s.len()>4_000_000{return Err(fail());}self.charge(s.len())?;},Value::Array(a)=>{if a.len()>4096{return Err(fail());}for x in a{stack.push((x,depth+1));}},Value::Object(o)=>{if o.len()>4096{return Err(fail());}for (k,x) in o{self.charge(k.len())?;stack.push((x,depth+1));}},_=>{} }
  } Ok(())
 }
}
#[derive(PartialEq)]
struct Class<'a> { output:&'a SecurityRef,version:&'a str,domain:&'a Value,literal:Option<ScalarLiteral>,sources:BTreeSet<&'a str> }
pub(crate) fn check(ctx:&SecurityBackendContext<'_>,contract:&SecurityResultContract)->Result<()> {
 ctx.profiled_query().require_sources(ctx.logical_plan(),ctx.catalog(),ctx.binding_json(),ctx.backend_id(),ctx.backend_version(),ctx.target_profile())?;
 let requirements=ctx.requirements();
 if contract.version!="weft.security.result-contract/0.1.0"||contract.encoding!="weft.security.cells/0.1.0"||contract.columns.len()!=requirements.outputs().len()||contract.columns.is_empty()||contract.columns.len()>256{return Err(fail());}
 let ontology=SecurityOntologyClosure::read(ctx.logical_plan().source(),ctx.catalog())?;
 let mut budget=Budget{work:1_000_000,bytes:16_000_000};
 for (expected,column) in requirements.outputs().iter().zip(&contract.columns) {
  budget.charge(0)?;budget.text(&column.output_name)?;
  if column.position!=expected.position()||column.output_name!=expected.output().name||column.source_fields.len()!=1||column.outcomes.is_empty()||column.outcomes.len()>256{return Err(fail());}
  let (scan,identity)=match &expected.output().expression {Expression::Field{scan,identity}=>(scan,identity),_=>return Err(fail())};
  budget.identity(&column.source_fields[0])?;if !same(&column.source_fields[0],identity){return Err(fail());}
  let field=SecurityRef{document_id:identity.document_id.clone(),module_id:identity.module.clone(),element_id:identity.element.clone()};
  let scan_requirement=requirements.scans().iter().find(|s|s.inventory().scan()==scan).ok_or_else(fail)?;
  let target=scan_requirement.inventory().target();
  let action=scan_requirement.actions().iter().find(|a|a.inventory().action()==requirements.primary_action()).ok_or_else(fail)?;
  let classification=ontology.types.get(target).ok_or_else(fail)?.source["fields"].as_array().ok_or_else(fail)?.iter().find(|f|SecurityRef::read(&f["ref"]).as_ref().is_ok_and(|r|r==&field)).ok_or_else(fail)?;
  let mut original=classification["protection"]=="unprotected";let mut withheld=false;let mut classes:Vec<Class<'_>>=Vec::new();
  for rule in action.rules(){budget.charge(0)?;if rule.effect!=Effect::Permit{continue;}
   for (selected,disposition) in &rule.disclosure {budget.charge(0)?;if selected!=&field{continue;}
    match disposition {
     Disposition::Original=>original=true,Disposition::Withheld=>withheld=true,
     Disposition::Transformed{transform,version,output_field,domain:declared,literal}=>{
      if transform!="constant"||version!="0.1.0"{return Err(fail());}budget.literal(literal)?;
      let source=locate(ctx.catalog(),output_field).map_err(|_|fail())?;if &domain(source).map_err(|_|fail())?!=declared{return Err(fail());}
      let normalized=normalized_literal(source,literal).map_err(|_|fail())?;budget.normalized(&normalized)?;
      let found=classes.iter().position(|c|c.output==output_field&&c.version==version&&c.domain==declared&&c.literal==normalized);
      budget.text(&rule.id)?;
      if let Some(i)=found {classes[i].sources.insert(&rule.id);} else {classes.push(Class{output:output_field,version,domain:declared,literal:normalized,sources:BTreeSet::from([rule.id.as_str()])});}
     }
    }
   }
  }
  if original&&locate(ctx.catalog(),&field).map_err(|_|fail())?["nullability"]!="required"{return Err(fail());} // Original presence remains unresolved outside the required subset.
  let required=usize::from(original)+usize::from(withheld)+classes.len();
  if required==0||required>256||required!=column.outcomes.len(){return Err(fail());}
  let mut ids=BTreeSet::new();let mut seen_classes=BTreeSet::new();let mut seen_original=false;let mut seen_withheld=false;
  for outcome in &column.outcomes {budget.charge(0)?;
   let id=match outcome {SecurityResultOutcome::Original{id,..}|SecurityResultOutcome::Transformed{id,..}|SecurityResultOutcome::Withheld{id}|SecurityResultOutcome::Absent{id}=>id};
   budget.text(id)?;if !ids.insert(id){return Err(fail());}
   match outcome {
    SecurityResultOutcome::Original{domain,..}=>{if !original||seen_original||!model_domain(domain,&field,ctx){return Err(fail());}seen_original=true;},
    SecurityResultOutcome::Withheld{..}=>{if !withheld||seen_withheld{return Err(fail());}seen_withheld=true;},
    SecurityResultOutcome::Absent{..}=>return Err(fail()),
    SecurityResultOutcome::Transformed{transform,disposition_sources,domain:result_domain,..}=>{
     let SecurityTransform::Constant{version,output_field,literal}=transform;
     budget.text(version)?;budget.identity(output_field)?;budget.literal(literal)?;
     if disposition_sources.is_empty()||disposition_sources.len()>256{return Err(fail());}
     let reference=SecurityRef{document_id:output_field.document_id.clone(),module_id:output_field.module.clone(),element_id:output_field.element.clone()};
     if !matches(output_field,&reference,ctx)||!model_domain(result_domain,&reference,ctx){return Err(fail());}
     let source=locate(ctx.catalog(),&reference).map_err(|_|fail())?;let output_domain=domain(source).map_err(|_|fail())?;let normalized=normalized_literal(source,literal).map_err(|_|fail())?;budget.normalized(&normalized)?;
     let i=classes.iter().position(|c|c.output==&reference&&c.version==version&&c.domain==&output_domain&&c.literal==normalized).ok_or_else(fail)?;
     if !seen_classes.insert(i){return Err(fail());}
     let mut sources=BTreeSet::new();for s in disposition_sources {budget.text(&s.rule_id)?;budget.identity(&s.target)?;budget.identity(&s.field)?;
      if !matches(&s.target,target,ctx)||!matches(&s.field,&field,ctx)||!sources.insert(s.rule_id.as_str()){return Err(fail());}
     }
     if sources!=classes[i].sources{return Err(fail());}
    }
   }
  }
  if seen_original!=original||seen_withheld!=withheld||seen_classes.len()!=classes.len(){return Err(fail());}
 }
 Ok(())
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn normalized_numeric_payload_is_charged_independently_of_short_tokens() {
  let field=serde_json::json!({"id":"n","kind":"field","scalarType":"integer","cardinality":"one","nullability":"required"});
  let expanded=normalized_literal(&field,&serde_json::json!({"integerToken":"1e10000"})).unwrap();
  let mut short_token_budget=Budget{work:100,bytes:32};assert!(short_token_budget.normalized(&expanded).is_err());
  let number=Some(ScalarLiteral::Number(num_bigint::BigInt::from(1)<<4096));
  let mut insufficient=Budget{work:10,bytes:8};assert!(insufficient.normalized(&number).is_err());assert_eq!(insufficient.bytes,8);
  let mut exact=Budget{work:10,bytes:520};assert!(exact.normalized(&number).is_ok());assert_eq!(exact.bytes,0);
  assert!(exact.normalized(&Some(ScalarLiteral::String("x".into()))).is_err());
 }
}
