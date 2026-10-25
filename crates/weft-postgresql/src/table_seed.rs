// SPDX-License-Identifier: MIT OR Apache-2.0
//! Administrative source-selected seed SQL. Hosts retain transaction/authority.
use serde_json::{Value,json};
use weft_core::{raw_read::Carrier,table_seed::{AdmittedSeed,SeedRow,admit_json}};
type Result<T> = std::result::Result<T,&'static str>;
fn q(value:&str)->Result<String>{crate::Identifier::new(value).map(|v|v.sql()).map_err(|_|"seed identifier outside native profile")}
fn scalar(index:usize,carrier:&Carrier)->String {
 let text=format!("(i.v ->> {index})");
 match carrier {
  Carrier::Bits128=>format!("pg_catalog.decode({text},'hex')"),
  Carrier::Date=>format!("{text}::pg_catalog.date"),
  Carrier::Timestamp=>format!("{text}::pg_catalog.timestamptz"),
  Carrier::LocalTimestamp=>format!("{text}::pg_catalog.timestamp"),
  Carrier::Uuid=>format!("{text}::pg_catalog.uuid"),
  Carrier::Boolean=>format!("{text}::pg_catalog.bool"),
  Carrier::Integer|Carrier::Decimal=>format!("{text}::pg_catalog.numeric"),
  Carrier::String=>format!("({text} COLLATE pg_catalog.\"C\")"),
 }
}
fn same_shape(seed:&AdmittedSeed,a:&SeedRow,b:&SeedRow)->bool {
 seed.tables[a.table].independent && a.table==b.table && a.ignore==b.ignore && a.set_index==b.set_index && !a.fields.is_empty() && a.fields==b.fields
}
fn compile(raw:&str)->Result<Value> {
 let seed=admit_json(raw)?;
 let schema=seed.request.binding.schema.as_deref().ok_or("seed native schema absent")?;
 let mut steps=vec![];let mut offset=0;
 while offset<seed.rows.len() {
  let first=&seed.rows[offset];let table=&seed.tables[first.table];let mut end=offset+1;
  while end<seed.rows.len() && end-offset<1000 && same_shape(&seed,first,&seed.rows[end]) { end+=1; }
  let relation=format!("{}.{}",q(schema)?,q(&table.table.native)?);
  let insert=if first.fields.is_empty() { format!("INSERT INTO {relation} DEFAULT VALUES") } else {
   let columns=first.fields.iter().map(|i|q(&table.fields[*i].native)).collect::<Result<Vec<_>>>()?.join(", ");
   let expressions=first.fields.iter().enumerate().map(|(i,f)|scalar(i,&table.fields[*f].carrier)).collect::<Vec<_>>().join(", ");
   format!("INSERT INTO {relation} ({columns}) SELECT {expressions} FROM pg_catalog.jsonb_array_elements($1::pg_catalog.jsonb) WITH ORDINALITY AS i(v,ordinal) ORDER BY i.ordinal")
  };
  let conflict=if first.ignore { format!(" ON CONFLICT ({}) DO NOTHING",table.conflict_key.iter().map(|v|q(v)).collect::<Result<Vec<_>>>()?.join(", ")) } else {String::new()};
  let sql=format!("WITH umf_seed_effects AS ({insert}{conflict} RETURNING 1) SELECT pg_catalog.count(*)::pg_catalog.text AS umf_seed_count FROM umf_seed_effects");
  let mut step=json!({"sql":sql,"parameterOrder":if first.fields.is_empty(){vec![]}else{vec!["nativeRows"]},"nativeRows":seed.rows[offset..end].iter().map(|r|&r.values).collect::<Vec<_>>(),"sourceRows":{"start":offset,"count":end-offset,"setIndex":first.set_index},"table":table.table.native,"maxEffects":end-offset});
  crate::dbapi::attach(&mut step)?;steps.push(step);offset=end;
 }
 let mut program=json!({"version":"weft.table-seed/0.1","status":"compiled","requestJson":raw,"requestSha256":weft_core::json::sha256(raw.as_bytes()),"module":seed.request.module,"seedSets":seed.request.seed_sets,"schema":schema,"sourceDigest":seed.request.binding.model_sha256,"nativeLifecycle":seed.request.binding.native_lifecycle,"steps":steps,"sourceRowCount":seed.rows.len(),"transport":{"profile":"dbapi-percent/0.1","bindEmptyParameterTuple":true},"obligations":{"authority":"approved-source-selected-administrative-seed","transaction":"entire-migration","rowOrder":"authored","rls":"installer-owner-scoped-disable-and-restore","automaticRetry":false,"nativeExecutionQualified":false}});
 program["structuralDigest"]=json!(weft_core::json::sha256(weft_core::mutation::canonical(&program).as_bytes()));Ok(program)
}
fn blocked(reason:&str)->String {json!({"version":"weft.table-seed/0.1","status":"blocked","diagnostics":[{"code":reason}]}).to_string()}
pub fn compile_json(raw:&str)->String {match compile(raw){Ok(v)=>{let text=v.to_string();if text.len()>33_554_432 {blocked("seed program capacity exceeded")}else{text}},Err(e)=>blocked(e)}}
pub fn check_json(raw:&str)->String {
 if raw.len()>33_554_432 {return blocked("seed program capacity exceeded");}
 let value=match weft_core::json::checked_json(raw){Ok(v)=>v,Err(_)=>return blocked("seed program JSON invalid")};
 let Some(original)=value["requestJson"].as_str() else{return blocked("seed original request absent")};
 match compile(original){Ok(expected) if expected==value=>json!({"version":"weft.table-seed/0.1","status":"checked","nativeExecutionQualified":false}).to_string(),_=>blocked("seed program differs from original source compilation")}
}
