// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded physical declaration shape only; not SQL meaning or execution admission.
use std::collections::BTreeSet;
use crate::{error::{Diagnostic,Result},security_backend::SecurityBackendContext,security_lowering::{SecurityPhysicalPlan,SecurityOperation},security_result_check::Budget};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-PHYSICAL","capability","Physical security declaration shape refused")}
pub(crate) fn check(ctx:&SecurityBackendContext<'_>,plan:&SecurityPhysicalPlan)->Result<()> {
 ctx.profiled_query().require_sources(ctx.logical_plan(),ctx.catalog(),ctx.binding_json(),ctx.backend_id(),ctx.backend_version(),ctx.target_profile())?;
 if plan.version!="weft.security.physical-plan/0.1.0"||plan.installation.len()>512{return Err(fail());}
 let mut budget=Budget::new();let mut ids=BTreeSet::new();let mut slots=0usize;
 for operation in plan.installation.iter().chain(std::iter::once(&plan.execution)) {
  operation_shape(operation,&mut budget)?;
  if !ids.insert(operation.id.as_str()){return Err(fail());}
  slots=slots.checked_add(operation.parameters.len()).ok_or_else(fail)?;
  if slots>16384{return Err(fail());}
 }
 Ok(())
}
fn operation_shape(operation:&SecurityOperation,budget:&mut Budget)->Result<()> {
 if operation.id.len()>16384{return Err(fail());}
 budget.text(&operation.id).map_err(|_|fail())?;
 if operation.sql.len()>1024*1024||operation.sql.contains('\0')||operation.sql.trim().is_empty()||operation.parameters.len()>1024{return Err(fail());}
 budget.charge(operation.sql.len()).map_err(|_|fail())?;
 for (index,p) in operation.parameters.iter().enumerate(){
  let logical_type=p.logical_type.atomic().map_err(|_|fail())?;
  if p.position!=index+1||!p.origin.is_object()||logical_type.nullable||p.value.len()>1024*1024{return Err(fail());}
  budget.charge(p.value.len()).map_err(|_|fail())?;
  budget.literal(&logical_type.facets).map_err(|_|fail())?;
  budget.literal(&p.origin).map_err(|_|fail())?;
 }
 Ok(())
}

/// Reuse exact ordinary compiler value semantics; capability/source binding is separate.
pub(crate) fn check_domains(ctx:&SecurityBackendContext<'_>,plan:&SecurityPhysicalPlan)->Result<()> {
 check(ctx,plan)?;
 for operation in plan.installation.iter().chain(std::iter::once(&plan.execution)) {
  for p in &operation.parameters {
   let logical_type=p.logical_type.atomic().map_err(|_|fail())?;
   let facets=logical_type.facets.as_object().ok_or_else(fail)?;
   match logical_type.family {
    crate::ir::Family::String|crate::ir::Family::Boolean=>{if !facets.is_empty(){return Err(fail());}},
    crate::ir::Family::Integer=>{
     if facets.len()!=1{return Err(fail());}
     let width=facets.get("integerWidth").and_then(serde_json::Value::as_object).ok_or_else(fail)?;
     if width.len()!=2||!width.contains_key("bits")||!width.contains_key("signed"){return Err(fail());}
    },
    crate::ir::Family::Decimal=>{if facets.len()!=2||!facets.contains_key("precision")||!facets.contains_key("scale"){return Err(fail());}}
   }
   crate::backend_emission::validate_parameter_domain(&p.value,logical_type,false).map_err(|_|fail())?;
  }
 }
 Ok(())
}

/// Exact per-occurrence execution slots for the explicitly parameterized query profile.
/// Installation values and SQL placeholder semantics remain separate obligations.
pub(crate) fn check_query_slots(ctx:&SecurityBackendContext<'_>,plan:&SecurityPhysicalPlan)->Result<()> {
 check_domains(ctx,plan)?;
 use crate::application_ir::{Predicate,Value};
 let mut originals:Vec<(String,&Value)>=Vec::new();
 fn collect<'a>(p:&'a Predicate,path:String,out:&mut Vec<(String,&'a Value)>)->Result<()>{
  fn push<'a>(path:String,v:&'a Value,out:&mut Vec<(String,&'a Value)>)->Result<()>{if matches!(v,Value::Field{..}){return Ok(());}if out.len()>=1024{return Err(fail());}out.push((path,v));Ok(())}
  match p {
   Predicate::Boolean{..}|Predicate::ExactEqual{..}=>return Err(fail()),
   Predicate::Equal{right,..}=>push(format!("{path}/right"),right,out)?,
   Predicate::LexicographicGreater{values,..}=>for(i,v)in values.iter().enumerate(){push(format!("{path}/values/{i}"),v,out)?;},
   Predicate::HasRelated{key,..}=>for(i,v)in key.iter().enumerate(){push(format!("{path}/key/{i}"),v,out)?;}
  }
  Ok(())
 }
 let context_hash=context_fingerprint(ctx);
 let query=ctx.query().application_plan();
 for(i,j)in query.joins.iter().enumerate(){for(k,p)in j.on.iter().enumerate(){collect(p,format!("joins/{i}/on/{k}"),&mut originals)?;}}
 for(i,p)in query.filters.iter().enumerate(){collect(p,format!("filters/{i}"),&mut originals)?;}
 originals.retain(|(_,v)|!matches!(v,Value::Field{..}));
 if originals.len()!=plan.execution.parameters.len(){return Err(fail());}
 for((path,original),slot)in originals.iter().zip(&plan.execution.parameters){
  let (value,logical_type)=match original{Value::Literal{value,logical_type,..}|Value::Parameter{value,logical_type,..}=>(value,logical_type),_=>return Err(fail())};
  if slot.value.as_str()!=value.as_str()||&slot.logical_type!=logical_type||slot.origin!=serde_json::json!({"kind":"queryValue","sourcePath":path,"sourceContextSha256":context_hash}){return Err(fail());}
 }
 Ok(())
}

/// PostgreSQL ordinal placeholder coverage only, not statement/query semantics.
pub(crate) fn check_postgres_placeholders(ctx:&SecurityBackendContext<'_>,plan:&SecurityPhysicalPlan)->Result<()> {
 check_query_slots(ctx,plan)?;
 use sqlparser::{dialect::PostgreSqlDialect,tokenizer::{Tokenizer,Token}};
 let dialect=PostgreSqlDialect{};
 let tokens=Tokenizer::new(&dialect,&plan.execution.sql).tokenize_with_location().map_err(|_|fail())?;
 let mut used=BTreeSet::new();let mut escaped_string=false;
 for (index,located) in tokens.iter().enumerate() {
  let token=&located.token;
  if matches!(token,Token::Whitespace(_)){continue;}
  // sqlparser0.63 does not retain PostgreSQL escape mode across continued
  // string fragments. Refuse that form before any apparent ordinal can count.
  if escaped_string&&matches!(token,Token::SingleQuotedString(_)){return Err(fail());}
  escaped_string=matches!(token,Token::EscapedStringLiteral(_));
  match token {
   // PostgreSQL ends an operator before either comment opener; the pinned
   // tokenizer can absorb the opener and expose comment text as placeholders.
   Token::CustomBinaryOperator(op) if op.contains("--")||op.contains("/*")=>return Err(fail()),
   // The selected component retains ASCII dollar tags only. Numeric-leading
   // tags are not native dollar-quote delimiters, despite tokenizer acceptance.
   Token::DollarQuotedString(s) if s.tag.as_ref().is_some_and(|tag|{
    !tag.as_bytes().first().is_some_and(|c|c.is_ascii_alphabetic()||*c==b'_')||!tag.bytes().all(|c|c.is_ascii_alphanumeric()||c==b'_')
   })=>return Err(fail()),
   // UESCAPE-aware Unicode decoding is not implemented by the pinned tokenizer.
   Token::UnicodeStringLiteral(_)=>return Err(fail()),
   _=>{}
  }
  if let Token::Placeholder(name)=token {
   // PostgreSQL treats any identifier-start byte (including non-ASCII) after
   // an ordinal as parameter junk; sqlparser can split it into a separate word.
   if tokens.get(index+1).is_some_and(|next|located.span.end==next.span.start&&matches!(&next.token,Token::Word(w) if w.quote_style.is_none())){return Err(fail());}
   let digits=name.strip_prefix('$').ok_or_else(fail)?;
   let position=digits.parse::<usize>().map_err(|_|fail())?;
   if position==0||position>plan.execution.parameters.len()||digits!=position.to_string(){return Err(fail());}
   used.insert(position);
  }
 }
 if used.len()!=plan.execution.parameters.len(){return Err(fail());}
 Ok(())
}


/// Framed exact source correspondence, not issuer authentication or authority.
pub(crate) fn context_fingerprint(ctx:&SecurityBackendContext<'_>)->String {
 use sha2::{Digest,Sha256};
 fn feed(h:&mut Sha256,bytes:&[u8]){h.update((bytes.len() as u64).to_be_bytes());h.update(bytes);}
 let mut h=Sha256::new();
 for value in ["weft.security.query-slot-context/0.1.0",ctx.query().sql(),ctx.binding_json(),ctx.backend_id(),ctx.backend_version(),ctx.target_profile(),ctx.logical_plan().source().policy_json(),ctx.logical_plan().source().ontology_json(),ctx.profiled_query().profile_source_json()] {feed(&mut h,value.as_bytes());}
 h.update((ctx.catalog().inputs().len() as u64).to_be_bytes());
 for input in ctx.catalog().inputs(){
  for value in [&input.document_json,&input.pin.document_id,&input.pin.revision,&input.pin.umf_version,&input.pin.sha256]{feed(&mut h,value.as_bytes());}
  h.update((input.selected_module_ids.len() as u64).to_be_bytes());for id in &input.selected_module_ids{feed(&mut h,id.as_bytes());}
 }
 struct Sink(Sha256);
 impl std::io::Write for Sink{fn write(&mut self,b:&[u8])->std::io::Result<usize>{self.0.update(b);Ok(b.len())}fn flush(&mut self)->std::io::Result<()>{Ok(())}}
 let mut resolved=Sink(Sha256::new());serde_json::to_writer(&mut resolved,ctx.query().application_plan()).expect("owned relational plan serialization");feed(&mut h,&resolved.0.finalize());
 format!("{:x}",h.finalize())
}

/// Declaration graph only. Source coverage, evidence and authority are separate.
pub(crate) fn check_obligations(ctx:&SecurityBackendContext<'_>,items:&[crate::security_lowering::SecurityAdmissionObligation])->Result<()> {
 use std::collections::{BTreeMap,VecDeque};
 let refused=||Diagnostic::new("WFT-SECURITY-OBLIGATIONS","capability","Security obligation declaration graph refused");
 ctx.profiled_query().require_sources(ctx.logical_plan(),ctx.catalog(),ctx.binding_json(),ctx.backend_id(),ctx.backend_version(),ctx.target_profile())?;
 if items.len()>4096{return Err(refused());}
 let mut budget=Budget::new();let mut indices=BTreeMap::new();let mut edges=0usize;
 fn text(budget:&mut Budget,s:&str)->Result<()>{if s.len()>16384{return Err(fail());}budget.text(s)}
 for (i,item) in items.iter().enumerate(){
  text(&mut budget,&item.id).map_err(|_|refused())?;
  if indices.insert(item.id.as_str(),i).is_some()||item.semantic_sources.is_empty()||item.evidence_case_ids.is_empty(){return Err(refused());}
  text(&mut budget,&item.failure_code).map_err(|_|refused())?;
  if !item.failure_code.strip_prefix("WFT-").is_some_and(|s|!s.is_empty()&&s.bytes().all(|b|b.is_ascii_uppercase()||b.is_ascii_digit()||b==b'-')){return Err(refused());}
  for list in [&item.semantic_sources,&item.prerequisites,&item.evidence_case_ids]{
   if list.len()>4096{return Err(refused());}
   let mut seen=BTreeSet::new();
   for value in list{text(&mut budget,value).map_err(|_|refused())?;if !seen.insert(value.as_str()){return Err(refused());}}
  }
  edges=edges.checked_add(item.prerequisites.len()).ok_or_else(refused)?;
  if edges>32768{return Err(refused());}
 }
 let mut pending=vec![0usize;items.len()];let mut consumers=vec![Vec::new();items.len()];
 for (i,item) in items.iter().enumerate(){for name in &item.prerequisites{
  let prerequisite=*indices.get(name.as_str()).ok_or_else(refused)?;
  if prerequisite==i{return Err(refused());}
  pending[i]+=1;consumers[prerequisite].push(i);
 }}
 let mut ready:VecDeque<_>=pending.iter().enumerate().filter_map(|(i,n)|(*n==0).then_some(i)).collect();let mut settled=0;
 while let Some(i)=ready.pop_front(){settled+=1;for &consumer in &consumers[i]{pending[consumer]-=1;if pending[consumer]==0{ready.push_back(consumer);}}}
 if settled!=items.len(){return Err(refused());}Ok(())
}

/// Exact context-bound event-source coverage only; no profile or evidence admission.
pub(crate) fn check_source_coverage(ctx:&SecurityBackendContext<'_>,items:&[crate::security_lowering::SecurityAdmissionObligation])->Result<()> {
 check_obligations(ctx,items)?;
 let refused=||Diagnostic::new("WFT-SECURITY-SOURCE-COVERAGE","capability","Security semantic source coverage refused");
 let required=crate::security_obligation_sources::derive_context_bound(ctx)?;
 let mut seen=BTreeSet::new();
 for item in items {for source in &item.semantic_sources{
  if !required.contains(source){return Err(refused());}seen.insert(source.as_str());
 }}
 if seen.len()!=required.len(){return Err(refused());}Ok(())
}
