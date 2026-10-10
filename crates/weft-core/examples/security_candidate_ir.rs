//! Private original draft IR inspection; no execution or authority.
use std::io::{self,Read};
use serde::Deserialize;
use weft_core::{model::{Catalog,ModuleInput},security_source::SecurityCandidateSourcePacket,security_candidate_ir::SecurityCandidateLogicalPlan};
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Input{modules:Vec<ModuleInput>,policy_json:String,ontology_json:String}
fn main(){let mut raw=String::new();io::stdin().take(4_000_001).read_to_string(&mut raw).unwrap();assert!(raw.len()<=4_000_000);let input:Input=serde_json::from_value(weft_core::json::checked_json(&raw).unwrap()).unwrap();let catalog=Catalog::prepare_security(input.modules).unwrap();let packet=SecurityCandidateSourcePacket::read(&input.policy_json,&input.ontology_json,&catalog).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();println!("{}",serde_json::to_string(plan.rules()).unwrap());}
