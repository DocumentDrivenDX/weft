//! Experimental inspection transport. No backend factory or data execution.
use std::io::{Read,Write};
use serde::Deserialize;
use weft_core::{error::{Diagnostic,Result},model::{Catalog,ModuleInput},security_source::SecuritySourcePacket,security_ir::SecurityLogicalPlan,security_query_profile::SecurityQueryProfile,security_query_uses::SecurityResolvedQuery};
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Input{version:String,modules:Vec<ModuleInput>,policy_json:String,ontology_json:String,query_profile_json:String,binding_json:String,backend_id:String,backend_version:String,target_profile:String,sql:String,#[serde(default)]parameters:weft_core::application_resolve::Parameters,read_profile:Option<weft_core::application_ir::ReadProfile>}
fn run()->Result<String>{
 let fail=||Diagnostic::new("WFT-SECURITY-MAPPING-INPUT","input","Mapping inspection input refused");
 let mut bytes=Vec::new();std::io::stdin().take(32_000_001).read_to_end(&mut bytes).map_err(|_|fail())?;
 if bytes.len()>32_000_000{return Err(fail());}
 let text=std::str::from_utf8(&bytes).map_err(|_|fail())?;
 let value=weft_core::json::checked_json(text).map_err(|_|fail())?;
 let input:Input=serde_json::from_value(value).map_err(|_|fail())?;
 if input.version!="weft.security.mapping-input/0.1.0"{return Err(fail());}
 let catalog=Catalog::prepare_security(input.modules)?;
 let packet=SecuritySourcePacket::read(&input.policy_json,&input.ontology_json,&catalog)?;
 let plan=SecurityLogicalPlan::read(packet,&catalog)?;
 let profile=SecurityQueryProfile::read(&input.query_profile_json,&plan,&catalog,&input.binding_json,&input.backend_id,&input.backend_version,&input.target_profile)?;
 let query=SecurityResolvedQuery::resolve(&input.sql,&catalog,&plan,input.parameters,input.read_profile)?;
 let admitted=profile.admit_resolved_query(&query,&plan,&catalog,&input.binding_json,&input.backend_id,&input.backend_version,&input.target_profile)?;
 admitted.mapping_handoff_json(&plan,&catalog,&input.binding_json,&input.backend_id,&input.backend_version,&input.target_profile)
}
fn main(){match run(){Ok(packet)=>{if writeln!(std::io::stdout(),"{packet}").is_err(){std::process::exit(1)}},Err(d)=>{let _=writeln!(std::io::stderr(),"{}",serde_json::to_string(&d).expect("diagnostic serialization"));std::process::exit(1)}}}
