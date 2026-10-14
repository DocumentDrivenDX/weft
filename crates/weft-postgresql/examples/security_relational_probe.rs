//! Bounded pure compiler diagnostic for controlled SQL probes. Not guarded dispatch.
use std::io::{Read,Write};
use serde_json::json;
use weft_core::{error::{Diagnostic,Result},application_ir::Value as QueryValue};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-RELATIONAL-PROBE","input","Bounded original relational diagnostic refused")}
struct Counter(usize);
impl Write for Counter{fn write(&mut self,b:&[u8])->std::io::Result<usize>{if b.len()>self.0{return Err(std::io::Error::other("output limit"));}self.0-=b.len();Ok(b.len())}fn flush(&mut self)->std::io::Result<()>{Ok(())}}
fn run()->Result<Vec<u8>>{
 let mut input=Vec::new();std::io::stdin().take(4_000_001).read_to_end(&mut input).map_err(|_|fail())?;if input.len()>4_000_000{return Err(fail());}let raw=std::str::from_utf8(&input).map_err(|_|fail())?;
 weft_core::security_mapping_transport::with_security_mapping_context(raw,|ctx|{
  let source=serde_json::from_str(ctx.binding_json()).map_err(|_|fail())?;let program=weft_postgresql::security_relational_sql::prepare_relational_graph_sql(ctx,&source)?;
  let p=program.parameters();let mut parameters=Vec::new();for (binding,value) in p.graph().policy().iter().zip(p.policy_values()){let t=match binding.codec(){weft_postgresql::security_graph_parameters::GraphPolicyParameterCodec::Boolean=>"boolean",weft_postgresql::security_graph_parameters::GraphPolicyParameterCodec::Text=>"text",weft_postgresql::security_graph_parameters::GraphPolicyParameterCodec::CoreLiteralJson=>"jsonb"};parameters.push(json!({"ordinal":binding.ordinal(),"channel":"policy","physicalType":t,"sourcePath":binding.source_path(),"value":value}));}
  parameters.push(json!({"ordinal":p.graph().subject().ordinal(),"channel":"subject","physicalType":"bigint","supplier":"original_authenticated_guarded_host_only"}));
  for binding in p.query(){let value=match binding.value(){QueryValue::Literal{value,..}|QueryValue::Parameter{value,..}=>value,_=>return Err(fail())};let t=match binding.codec(){weft_postgresql::security_relational_parameters::RelationalQueryParameterCodec::Boolean=>"boolean",weft_postgresql::security_relational_parameters::RelationalQueryParameterCodec::Text=>"text",weft_postgresql::security_relational_parameters::RelationalQueryParameterCodec::Numeric=>"numeric"};parameters.push(json!({"ordinal":binding.ordinal(),"channel":"query","physicalType":t,"sourcePath":binding.source_path(),"value":value}));}
  let contract=serde_json::to_string(program.result_contract()).map_err(|_|fail())?;
  let output=json!({"version":"weft.truss.security.relational-probe/0.2-candidate","securityReady":false,"contextSha256":program.context_sha256(),"sql":program.sql(),"checks":program.checks(),"operandDemands":program.operand_demands(),"operandReads":program.operand_reads(),"parameters":parameters,"resultContract":program.result_contract(),"resultContractJson":contract,"resultContractSha256":weft_core::json::sha256(contract.as_bytes()),"qualification":"controlled_compiler_sql_probe_only_not_native_admission_or_guarded_dispatch"});
  serde_json::to_writer(Counter(16_000_000-1),&output).map_err(|_|fail())?;let mut encoded=serde_json::to_vec(&output).map_err(|_|fail())?;encoded.push(b'\n');Ok(encoded)
 })
}
fn main(){match run(){Ok(b)=>{if std::io::stdout().write_all(&b).is_err(){std::process::exit(1)}},Err(d)=>{let _=writeln!(std::io::stderr(),"{}",serde_json::to_string(&d).unwrap());std::process::exit(1)}}}
