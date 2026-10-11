//! Pure cell correspondence, not policy selection or release authorization.
use crate::{error::{Diagnostic,Result},json::{checked_json_bounded,sha256},security_backend::SecurityBackendContext,security_lowering::{SecurityResultContract,SecurityResultOutcome,SecurityResultDomain,SecurityTransform},security_ontology::{SecurityRef,locate}};
use serde_json::Value;
fn fail()->Diagnostic {Diagnostic::new("WFT-SECURITY-RESULT","result","Security cell correspondence refused")}
fn limit()->Diagnostic {Diagnostic::new("WFT-LIMIT","result","Security cell resource limit exceeded")}
pub(crate) fn parse(raw:&str)->Result<Value>{checked_json_bounded(raw,32*1024*1024,64,1_000_000).map_err(|code|Diagnostic::new(code,"result","Security result JSON refused"))}
pub(crate) fn check(ctx:&SecurityBackendContext<'_>,contract:&SecurityResultContract,contract_json:&str,contract_sha256:&str,batch_json:&str)->Result<()> {
 ctx.check_result_declaration(contract)?;
 // Hash the retained bytes before parsing; never substitute reserialization.
 if contract_json.len()>32*1024*1024{return Err(Diagnostic::new("WFT-LIMIT","result","Security result JSON refused"));}
 if sha256(contract_json.as_bytes())!=contract_sha256{return Err(fail());}
 let retained=parse(contract_json)?;
 if retained!=serde_json::to_value(contract).map_err(|_|fail())?{return Err(fail());}
 let batch=parse(batch_json)?;let object=batch.as_object().ok_or_else(fail)?;
 if object.len()!=3||batch["version"]!="weft.security.cells/0.1.0"||batch["resultContractSha256"]!=contract_sha256{return Err(fail());}
 let rows=batch["rows"].as_array().ok_or_else(fail)?;if rows.len()>4096{return Err(fail());}
 let mut budget=crate::security_result_check::Budget::new();
 for row in rows {let cells=row.as_array().ok_or_else(fail)?;if cells.len()!=contract.columns.len(){return Err(fail());}
  for (cell,column) in cells.iter().zip(&contract.columns) {let object=cell.as_object().ok_or_else(fail)?;
   let id=cell["outcomeId"].as_str().ok_or_else(fail)?;let disposition=cell["disposition"].as_str().ok_or_else(fail)?;
   let outcome=column.outcomes.iter().find(|o|match o{SecurityResultOutcome::Original{id:i,..}|SecurityResultOutcome::Transformed{id:i,..}|SecurityResultOutcome::Withheld{id:i}|SecurityResultOutcome::Absent{id:i}=>i==id}).ok_or_else(fail)?;
   let (expected,domain,literal)=match outcome {
    SecurityResultOutcome::Original{domain,..}=>("original",Some(domain),None),
    SecurityResultOutcome::Transformed{domain,transform:SecurityTransform::Constant{literal,..},..}=>("transformed",Some(domain),Some(literal)),
    SecurityResultOutcome::Withheld{..}=>("withheld",None,None),SecurityResultOutcome::Absent{..}=>("absent",None,None)
   };
   if disposition!=expected||object.len()!=if domain.is_some(){3}else{2}{return Err(fail());}
   // Exact keys: equal length is insufficient when required keys are replaced.
   if !object.contains_key("outcomeId")||!object.contains_key("disposition"){return Err(fail());}
   if let Some(domain)=domain {
    let value=object.get("value").ok_or_else(fail)?;
    let SecurityResultDomain::Model{field}=domain else{return Err(fail());};
    let reference=SecurityRef{document_id:field.document_id.clone(),module_id:field.module.clone(),element_id:field.element.clone()};
    let source=locate(ctx.catalog(),&reference).map_err(|_|fail())?;
    budget.literal(value).map_err(|_|limit())?;let normalized=budget.normalize(source,value).map_err(|d|if d.code=="WFT-LIMIT"{limit()}else{fail()})?;
    if let Some(literal)=literal {budget.literal(literal).map_err(|_|limit())?;let constant=budget.normalize(source,literal).map_err(|d|if d.code=="WFT-LIMIT"{limit()}else{fail()})?;if normalized!=constant{return Err(fail());}}
   }
  }
 }
 Ok(())
}
