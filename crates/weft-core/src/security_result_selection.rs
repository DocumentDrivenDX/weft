//! Conditional supplied-truth selection checks. Never authenticates truth or releases data.
use crate::{application_ir::Expression,error::{Diagnostic,Result},security_backend::{SecurityBackendContext,SecuritySimulatedRowTruths},security_composition::{compose_with_budget,Decision},security_ir::Disposition,security_ontology::{SecurityRef,locate},security_lowering::{SecurityResultContract,SecurityResultOutcome,SecurityTransform}};
use std::collections::{BTreeMap,BTreeSet};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-RESULT-SELECTION","result","Simulated result selection refused")}
fn limit()->Diagnostic{Diagnostic::new("WFT-LIMIT","result","Simulated selection resource limit exceeded")}
pub(crate) fn check(ctx:&SecurityBackendContext<'_>,contract:&SecurityResultContract,contract_json:&str,hash:&str,batch_json:&str,truth_rows:&[SecuritySimulatedRowTruths])->Result<()> {
 ctx.check_result_cells(contract,contract_json,hash,batch_json)?;
 let batch=crate::security_result_cells::parse(batch_json)?;
 let rows=batch["rows"].as_array().ok_or_else(fail)?;
 if truth_rows.len()!=rows.len(){return Err(fail());}
 let requirements=ctx.requirements();let mut budget=crate::security_result_check::Budget::new();
 let mut composition_budget=crate::security_budget::PayloadBudget::result_phase("WFT-LIMIT");
 let mut fields:BTreeMap<&str,BTreeSet<SecurityRef>>=BTreeMap::new();
 for output in requirements.outputs(){
  let Expression::Field{scan,identity}=&output.output().expression else{return Err(fail());};
  budget.charge(identity.document_id.len()+identity.module.len()+identity.element.len()).map_err(|_|limit())?;
  fields.entry(scan.as_str()).or_default().insert(SecurityRef{document_id:identity.document_id.clone(),module_id:identity.module.clone(),element_id:identity.element.clone()});
 }
 for (row,truths) in rows.iter().zip(truth_rows){
  if truths.len()!=requirements.scans().len(){return Err(fail());}
  let mut selected=BTreeMap::new();
  for scan in requirements.scans(){
   budget.charge(0).map_err(|_|limit())?;
   let actions=truths.get(scan.inventory().scan()).ok_or_else(fail)?;
   if actions.len()!=scan.actions().len(){return Err(fail());}
   for action in scan.actions(){
    let supplied=actions.get(action.inventory().action()).ok_or_else(fail)?;
    if supplied.len()!=action.rules().len(){return Err(fail());}
    for id in supplied.keys(){budget.text(id).map_err(|_|limit())?;}
    // compose reconstructs source classification; charge inspected source bytes
    // before that work. This ledger does not bound every allocation/instruction.
    budget.charge(ctx.logical_plan().source().ontology_json().len()).map_err(|_|limit())?;
    let primary=action.inventory().action()==requirements.primary_action();
    let output:Vec<_>=if primary{fields.get(scan.inventory().scan()).into_iter().flat_map(|f|f.iter().cloned()).collect()}else{Vec::new()};
    let composition=compose_with_budget(ctx.logical_plan(),ctx.catalog(),scan.inventory().target(),action.inventory().action(),&output,supplied,&mut composition_budget).map_err(|d|if d.code=="WFT-LIMIT"{limit()}else{fail()})?;
    if composition.decision!=Decision::Permit{return Err(fail());}
    if primary {for (field,disposition) in composition.disclosure{selected.insert((scan.inventory().scan(),field),disposition);}}
   }
  }
  let cells=row.as_array().ok_or_else(fail)?;
  for ((output,column),cell) in requirements.outputs().iter().zip(&contract.columns).zip(cells){
   budget.charge(0).map_err(|_|limit())?;
   let Expression::Field{scan,identity}=&output.output().expression else{return Err(fail());};
   let field=SecurityRef{document_id:identity.document_id.clone(),module_id:identity.module.clone(),element_id:identity.element.clone()};
   let disposition=selected.get(&(scan.as_str(),field)).ok_or_else(fail)?;
   let id=cell["outcomeId"].as_str().ok_or_else(fail)?;
   let outcome=column.outcomes.iter().find(|o|match o{SecurityResultOutcome::Original{id:i,..}|SecurityResultOutcome::Transformed{id:i,..}|SecurityResultOutcome::Withheld{id:i}|SecurityResultOutcome::Absent{id:i}=>i==id}).ok_or_else(fail)?;
   match (disposition,outcome){
    (Disposition::Original,SecurityResultOutcome::Original{..})|(Disposition::Withheld,SecurityResultOutcome::Withheld{..})=>{},
    (Disposition::Transformed{transform,version,output_field,literal,..},SecurityResultOutcome::Transformed{transform:SecurityTransform::Constant{version:v,output_field:f,literal:l},..})=>{
     if transform!="constant"||version!=v||output_field.document_id!=f.document_id||output_field.module_id!=f.module||output_field.element_id!=f.element{return Err(fail());}
     let source=locate(ctx.catalog(),output_field).map_err(|_|fail())?;
     budget.literal(literal).map_err(|_|limit())?;budget.literal(l).map_err(|_|limit())?;
     let a=budget.normalize(source,literal).map_err(|d|if d.code=="WFT-LIMIT"{limit()}else{fail()})?;let b=budget.normalize(source,l).map_err(|d|if d.code=="WFT-LIMIT"{limit()}else{fail()})?;
     if a!=b{return Err(fail());}
    },_=>return Err(fail())
   }
  }
 }
 Ok(())
}
