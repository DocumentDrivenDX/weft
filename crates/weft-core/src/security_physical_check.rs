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
