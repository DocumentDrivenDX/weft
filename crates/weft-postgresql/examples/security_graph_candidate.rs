// SPDX-License-Identifier: MIT OR Apache-2.0
//! Source-only reified graph compiler handoff; no native execution permission.
use std::io::{Read,Write};
use serde_json::json;
use weft_core::{error::{Diagnostic,Result},json::{checked_json,sha256},security_mapping_transport::with_security_mapping_context};
use weft_postgresql::{security_graph_binding::GraphSource,security_predicate_spike::render_graph};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-GRAPH-INPUT","input","Original graph source mapping request refused")}
fn run()->Result<String>{
 let mut bytes=Vec::new();std::io::stdin().take(4_000_001).read_to_end(&mut bytes).map_err(|_|fail())?;
 if bytes.len()>4_000_000{return Err(fail());}let raw=std::str::from_utf8(&bytes).map_err(|_|fail())?;
 let request=checked_json(raw).map_err(|_|fail())?;
 if request["interfaceVersion"]!="weft-security-mapping/0.1.0"{return Err(fail());}
 let binding=request["bindingJson"].as_str().ok_or_else(fail)?;
 let value=checked_json(binding).map_err(|_|fail())?;
 let source:GraphSource=serde_json::from_value(value.clone()).map_err(|_|fail())?;
 let result=with_security_mapping_context(raw,|ctx|{
  let graph=render_graph(ctx.logical_plan(),ctx.catalog(),&source)?;
  let query=weft_postgresql::security_graph_query::prepare_graph_query(ctx,&source)?;
  Ok(json!({"version":"weft.truss.security.graph-lowering/0.1-candidate","securityReady":false,"sourceRequestSha256":sha256(raw.as_bytes()),"contextSha256":ctx.execution_query_context_sha256(),"bindingSha256":sha256(binding.as_bytes()),"graph":graph,"query":query}))
 })?;
 let output=result.to_string();if output.len()>16_000_000{return Err(fail());}Ok(output)
}
fn main(){match run(){Ok(output)=>{if writeln!(std::io::stdout(),"{output}").is_err(){std::process::exit(1)}},Err(d)=>{let _=writeln!(std::io::stderr(),"{}",serde_json::to_string(&d).unwrap());std::process::exit(1)}}}
