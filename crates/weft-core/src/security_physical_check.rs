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
  if p.position!=index+1||!p.origin.is_object()||p.logical_type.nullable||p.value.len()>1024*1024{return Err(fail());}
  budget.charge(p.value.len()).map_err(|_|fail())?;
  budget.literal(&p.logical_type.facets).map_err(|_|fail())?;
  budget.literal(&p.origin).map_err(|_|fail())?;
 }
 Ok(())
}

/// Reuse exact ordinary compiler value semantics; capability/source binding is separate.
pub(crate) fn check_domains(ctx:&SecurityBackendContext<'_>,plan:&SecurityPhysicalPlan)->Result<()> {
 check(ctx,plan)?;
 for operation in plan.installation.iter().chain(std::iter::once(&plan.execution)) {
  for p in &operation.parameters {
   let facets=p.logical_type.facets.as_object().ok_or_else(fail)?;
   match p.logical_type.family {
    crate::ir::Family::String|crate::ir::Family::Boolean=>{if !facets.is_empty(){return Err(fail());}},
    crate::ir::Family::Integer=>{
     if facets.len()!=1{return Err(fail());}
     let width=facets.get("integerWidth").and_then(serde_json::Value::as_object).ok_or_else(fail)?;
     if width.len()!=2||!width.contains_key("bits")||!width.contains_key("signed"){return Err(fail());}
    },
    crate::ir::Family::Decimal=>{if facets.len()!=2||!facets.contains_key("precision")||!facets.contains_key("scale"){return Err(fail());}}
   }
   crate::backend_emission::validate_parameter_domain(&p.value,&p.logical_type,false).map_err(|_|fail())?;
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
   Predicate::Equal{right,..}=>push(format!("{path}/right"),right,out)?,
   Predicate::LexicographicGreater{values,..}=>for(i,v)in values.iter().enumerate(){push(format!("{path}/values/{i}"),v,out)?;},
   Predicate::HasRelated{key,..}=>for(i,v)in key.iter().enumerate(){push(format!("{path}/key/{i}"),v,out)?;}
  }
  Ok(())
 }
 let query=ctx.query().application_plan();
 for(i,j)in query.joins.iter().enumerate(){for(k,p)in j.on.iter().enumerate(){collect(p,format!("joins/{i}/on/{k}"),&mut originals)?;}}
 for(i,p)in query.filters.iter().enumerate(){collect(p,format!("filters/{i}"),&mut originals)?;}
 originals.retain(|(_,v)|!matches!(v,Value::Field{..}));
 if originals.len()!=plan.execution.parameters.len(){return Err(fail());}
 for((path,original),slot)in originals.iter().zip(&plan.execution.parameters){
  let (value,logical_type)=match original{Value::Literal{value,logical_type,..}|Value::Parameter{value,logical_type,..}=>(value,logical_type),_=>return Err(fail())};
  if slot.value.as_str()!=value.as_str()||&slot.logical_type!=logical_type||slot.origin!=serde_json::json!({"kind":"queryValue","sourcePath":path}){return Err(fail());}
 }
 Ok(())
}

/// PostgreSQL ordinal placeholder coverage only, not statement/query semantics.
pub(crate) fn check_postgres_placeholders(ctx:&SecurityBackendContext<'_>,plan:&SecurityPhysicalPlan)->Result<()> {
 check_query_slots(ctx,plan)?;
 use sqlparser::{dialect::PostgreSqlDialect,tokenizer::{Tokenizer,Token}};
 let dialect=PostgreSqlDialect{};
 let tokens=Tokenizer::new(&dialect,&plan.execution.sql).tokenize().map_err(|_|fail())?;
 let mut used=BTreeSet::new();
 for token in tokens {
  if let Token::Placeholder(name)=token {
   let digits=name.strip_prefix('$').ok_or_else(fail)?;
   let position=digits.parse::<usize>().map_err(|_|fail())?;
   if position==0||position>plan.execution.parameters.len()||digits!=position.to_string(){return Err(fail());}
   used.insert(position);
  }
 }
 if used.len()!=plan.execution.parameters.len(){return Err(fail());}
 Ok(())
}
