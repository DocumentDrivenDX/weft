//! Experimental string-cell inspection. No data connection or result release.
use std::io::{Read,Write};
use serde::Deserialize;
use serde_json::{json,Value};
use weft_core::{error::{Diagnostic,Result},compile::Compiler,security_backend::SecurityRegistry,security_lowering::*};
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Input {version:String,compile_request_json:String,native_rows_json:String}
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-CELL-INSPECTION","input","Cell inspection refused")}
fn run()->Result<String>{
 let mut bytes=Vec::new();std::io::stdin().take(32*1024*1024+1).read_to_end(&mut bytes).map_err(|_|fail())?;
 if bytes.len()>32*1024*1024{return Err(fail());}
 let root=weft_core::json::checked_json(std::str::from_utf8(&bytes).map_err(|_|fail())?).map_err(|_|fail())?;
 if !root.is_object(){return Err(fail());}
 let input:Input=serde_json::from_value(root).map_err(|_|fail())?;
 if input.version!="weft.security.cell-inspection/0.1.0"{return Err(fail());}
 let native=weft_core::json::checked_json(&input.native_rows_json).map_err(|_|fail())?;
 let rows=native.as_array().ok_or_else(fail)?;if rows.len()>4096{return Err(fail());}
 let mut calls=0;let mut report=None;let mut error=None;
 let response=Compiler::default().compile_json_with_security_factory(&input.compile_request_json,&mut |ctx|{
  calls+=1;
  let check=(||->Result<Value>{
   let columns=ctx.requirements().outputs().iter().map(|o|{
    let weft_core::application_ir::Expression::Field{identity,..}=&o.output().expression else{return Err(fail());};
    let field=SecurityIdentity{document_id:identity.document_id.clone(),revision:identity.revision.clone(),module:identity.module.clone(),element:identity.element.clone()};
    Ok(SecurityResultColumn{position:o.position(),output_name:o.output().name.clone(),source_fields:vec![field.clone()],outcomes:vec![SecurityResultOutcome::Original{id:"original".into(),domain:SecurityResultDomain::Model{field}}]})
   }).collect::<Result<Vec<_>>>()?;
   let contract=SecurityResultContract{version:"weft.security.result-contract/0.1.0".into(),encoding:"weft.security.cells/0.1.0".into(),columns};
   let mut cells=Vec::new();for row in rows {let row=row.as_array().ok_or_else(fail)?;if row.len()!=contract.columns.len(){return Err(fail());}
    let mut values=Vec::new();for value in row {let value=value.as_str().ok_or_else(fail)?;values.push(json!({"outcomeId":"original","disposition":"original","value":{"string":value}}));}cells.push(values);
   }
   let contract_json=serde_json::to_string(&contract).map_err(|_|fail())?;let hash=weft_core::json::sha256(contract_json.as_bytes());
   let batch=json!({"version":"weft.security.cells/0.1.0","resultContractSha256":hash,"rows":cells});
   ctx.check_result_cells(&contract,&contract_json,&hash,&batch.to_string())?;
   Ok(json!({"status":"correspondence-only","rowCount":rows.len(),"columnCount":contract.columns.len(),"nativeRowsSha256":weft_core::json::sha256(input.native_rows_json.as_bytes()),"resultContractSha256":hash,"releasedRows":0}))
  })();
  match check {Ok(value)=>report=Some(value),Err(d)=>{error=Some(d.clone());return Err(d);}}
  Ok(SecurityRegistry::default())
 });
 if let Some(d)=error{return Err(d);}
 let response:Value=serde_json::from_str(&response).map_err(|_|fail())?;
 if calls!=1||response["status"]!="blocked"||response["diagnostics"][0]["code"]!="WFT-SECURITY-BACKEND-REQUIRED"{return Err(fail());}
 report.map(|r|r.to_string()).ok_or_else(fail)
}
fn main(){match run(){Ok(report)=>{if writeln!(std::io::stdout(),"{report}").is_err(){std::process::exit(1)}},Err(d)=>{let _=writeln!(std::io::stderr(),"{}",serde_json::to_string(&d).expect("diagnostic"));std::process::exit(1)}}}
