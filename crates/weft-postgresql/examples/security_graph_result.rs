// SPDX-License-Identifier: MIT OR Apache-2.0
//! Original producer/context-bound result-cell correspondence; no release permit.
use std::io::{Read,Write};
use serde::Deserialize;
use serde_json::json;
use weft_core::{error::{Diagnostic,Result},json::{checked_json,sha256},security_mapping_transport::with_security_mapping_context};
use weft_postgresql::{security_graph_binding::GraphSource,security_graph_query::prepare_graph_query_checked,security_predicate_spike::render_graph};
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Input {version:String,source_request_json:String,source_output_json:String,batch_json:String}
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-GRAPH-RESULT","host","Original graph result correspondence refused")}
fn run()->Result<String>{
 let mut bytes=Vec::new();std::io::stdin().take(32_000_001).read_to_end(&mut bytes).map_err(|_|fail())?;
 if bytes.len()>32_000_000{return Err(fail());}let raw=std::str::from_utf8(&bytes).map_err(|_|fail())?;
 let input:Input=serde_json::from_value(checked_json(raw).map_err(|_|fail())?).map_err(|_|fail())?;
 if input.version!="weft.truss.security.graph-result/0.1-candidate"||input.source_output_json.len()>16_000_001||input.batch_json.len()>4_000_000{return Err(fail());}
 let request=checked_json(&input.source_request_json).map_err(|_|fail())?;
 if request["interfaceVersion"]!="weft-security-mapping/0.1.0"{return Err(fail());}
 let binding=request["bindingJson"].as_str().ok_or_else(fail)?;
 let source:GraphSource=serde_json::from_value(checked_json(binding).map_err(|_|fail())?).map_err(|_|fail())?;
 with_security_mapping_context(&input.source_request_json,|ctx|{
  let graph=render_graph(ctx.logical_plan(),ctx.catalog(),&source)?;let query=prepare_graph_query_checked(ctx,&source)?;
  let original=json!({"version":"weft.truss.security.graph-lowering/0.1-candidate","securityReady":false,"sourceRequestSha256":sha256(input.source_request_json.as_bytes()),"contextSha256":ctx.execution_query_context_sha256(),"bindingSha256":sha256(binding.as_bytes()),"graph":graph,"query":query.value()});
  let exact=format!("{}\n",original);if input.source_output_json!=exact{return Err(fail());}
  ctx.check_result_cells(query.result_contract(),original["query"]["resultContractJson"].as_str().ok_or_else(fail)?,original["query"]["resultContractSha256"].as_str().ok_or_else(fail)?,&input.batch_json)?;
  Ok(())
 })?;
 Ok(json!({"version":"weft.truss.security.graph-result/0.1-candidate","status":"inspected","securityReady":false,"sourceRequestSha256":sha256(input.source_request_json.as_bytes()),"sourceOutputSha256":sha256(input.source_output_json.as_bytes()),"batchSha256":sha256(input.batch_json.as_bytes())}).to_string())
}
fn main(){match run(){Ok(output)=>{if writeln!(std::io::stdout(),"{output}").is_err(){std::process::exit(1)}},Err(d)=>{let _=writeln!(std::io::stderr(),"{}",serde_json::to_string(&d).unwrap());std::process::exit(1)}}}
