// SPDX-License-Identifier: MIT OR Apache-2.0
//! Source-selected administrative seeds; no synthesized source commands.
use crate::raw_read::{Binding,Table,Carrier};
use crate::table_operation::OperationField;
use serde::{Deserialize,Serialize};
use serde_json::{Value,json};
use std::collections::{BTreeMap,BTreeSet};
type Result<T> = std::result::Result<T,&'static str>;
#[derive(Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct SeedRequest {pub version:String,pub target:String,pub module:String,pub seed_sets:Vec<String>,pub binding:Binding}
#[derive(Debug)]
pub struct SeedTable {pub table:Table,pub fields:Vec<OperationField>,pub conflict_key:Vec<String>,pub independent:bool}
#[derive(Debug)]
pub struct SeedRow {pub table:usize,pub fields:Vec<usize>,pub values:Vec<Value>,pub ignore:bool,pub set_index:usize}
#[derive(Debug)]
pub struct AdmittedSeed {pub request:SeedRequest,pub tables:Vec<SeedTable>,pub rows:Vec<SeedRow>}
fn query_name(value:&Value)->Result<&str>{value["name"].as_str().or_else(||value["id"].as_str()).ok_or("seed logical name absent")}
fn prepare_table(table:&Value,module:&Value,binding:&Binding,elements:&BTreeMap<&str,&Value>)->Result<SeedTable>{
 let record=*elements.get(table["record"]["element"].as_str().ok_or("seed record identity absent")?).ok_or("seed record absent")?;
 let path=vec![module["namespace"].as_str().unwrap().to_owned(),query_name(record)?.to_owned()];
 let bound=binding.tables.iter().find(|b|b.logical==path).ok_or("seed native table mapping absent")?;
 let native_columns=bound.columns.iter().map(|c|(c.native.as_str(),c)).collect::<BTreeMap<_,_>>();
 let mut fields=vec![];
 for column in table["columns"].as_array().unwrap(){
  let field=*elements.get(column["field"]["element"].as_str().ok_or("seed Field identity absent")?).ok_or("seed Field absent")?;
  let native=*native_columns.get(column["name"].as_str().unwrap()).ok_or("seed native column absent")?;
  if field["semanticTypes"].as_array().is_some_and(|v|!v.is_empty()){return Err("seed semantic type interpreter unavailable");}
  if matches!(native.carrier,Carrier::Date|Carrier::Timestamp|Carrier::Uuid|Carrier::Bits128){
   if field["facets"].as_object().is_some_and(|v|!v.is_empty())||field["allowedValues"].as_array().is_some_and(|v|!v.is_empty()){return Err("seed exact carrier qualifiers unsupported");}
  }else{crate::security_literals::validate_field(field).map_err(|_|"seed Field qualifiers unsupported")?;}
  let mut native_type=column["nativeType"].clone();
  for key in ["length","precision","scale"]{if let Some(v)=native_type.get(key){native_type[key]=json!(crate::security_literals::unsigned(v).map_err(|_|"seed native capacity invalid")?);}}
  let resolved=OperationField{reference:column["field"].clone(),native:native.native.clone(),carrier:native.carrier.clone(),nullable:native.nullable,declaration:field.clone(),native_type};
  if column["default"]["kind"]=="literal"{crate::table_operation::native_seed_value(&resolved,&column["default"]["value"])?;}
  fields.push(resolved);
 }
 let mut conflict_key=vec![];
 if let Some(key)=record["keys"].as_array().unwrap().iter().find(|k|k["primary"]==true){for reference in key["fields"].as_array().unwrap(){conflict_key.push(fields.iter().find(|f|f.reference==*reference).ok_or("seed primary field unmapped")?.native.clone());}}
 let independent=!table["foreignKeys"].as_array().unwrap().iter().any(|fk|fk["target"]["record"]==table["record"]);
 Ok(SeedTable{table:bound.clone(),fields,conflict_key,independent})
}
pub fn admit_json(raw:&str)->Result<AdmittedSeed>{
 if raw.len()>16_777_216{return Err("seed request capacity exceeded");}
 let request:SeedRequest=serde_json::from_value(crate::json::checked_json(raw).map_err(|_|"seed JSON invalid or duplicate")?).map_err(|_|"seed request shape differs")?;
 if request.version!="weft.table-seed/0.1"||!matches!(request.target.as_str(),"postgresql18"|"lakebase18")||request.binding.native_lifecycle.as_ref().is_none_or(|c|c.module!=request.module){return Err("seed target/module differs");}
 if request.seed_sets.is_empty()||request.seed_sets.len()>128||request.seed_sets.iter().any(|s|s.is_empty()||s.len()>256){return Err("seed selection capacity exceeded");}
 let model=crate::raw_read::prepare_seed_model(&request.binding)?;
 if model.lifecycle.is_none(){return Err("seed native lifecycle absent");}
 let module=model.document["modules"].as_array().unwrap().iter().find(|m|m["id"]==request.module).ok_or("seed module absent")?;
 let payload=&module["extensions"]["umf.tables"];
 let elements=module["elements"].as_array().unwrap().iter().map(|e|Ok((e["id"].as_str().ok_or("seed element identity absent")?,e))).collect::<Result<BTreeMap<_,_>>>()?;
 let declarations=payload["tables"].as_array().unwrap().iter().map(|t|Ok((t["id"].as_str().ok_or("seed table identity absent")?,t))).collect::<Result<BTreeMap<_,_>>>()?;
 let sets=payload["seedSets"].as_array().ok_or("seed inventory absent")?;
 if sets.iter().map(|s|s["rows"].as_array().map_or(0,Vec::len)).sum::<usize>()>10000{return Err("module seed row capacity exceeded");}
 let mut identities=BTreeSet::new();for set in sets{if !identities.insert(set["id"].as_str().ok_or("seed identity absent")?){return Err("duplicate seed set identity");}}
 let mut selected=BTreeSet::new();let mut last=None;let mut ordered=vec![];
 for id in &request.seed_sets{
  if !selected.insert(id){return Err("duplicate seed selection");}
  let index=sets.iter().position(|s|s["id"]==*id).ok_or("seed set not declared")?;
  if last.is_some_and(|n|index<=n){return Err("seed selection must retain authored set order");}last=Some(index);ordered.push(&sets[index]);
 }
 let mut tables=vec![];let mut prepared=BTreeMap::new();let mut rows=vec![];let mut budget=8_388_608usize;
 for (set_index,set) in ordered.iter().enumerate(){
  let authored=set["rows"].as_array().ok_or("seed rows absent")?;if authored.len()>10000{return Err("seed set row capacity exceeded");}
  for row in authored{
   if rows.len()>=10000{return Err("seed invocation row capacity exceeded");}
   let id=row["table"].as_str().ok_or("seed table identity absent")?;
   let declaration=*declarations.get(id).ok_or("seed table outside module")?;
   let index=if let Some(index)=prepared.get(id){*index}else{let n=tables.len();tables.push(prepare_table(declaration,module,&request.binding,&elements)?);prepared.insert(id,n);n};
   let table=&tables[index];let columns=declaration["columns"].as_array().unwrap();
   let mut supplied=BTreeMap::new();for entry in row["values"].as_array().ok_or("seed values absent")?{if supplied.insert(entry["field"].to_string(),&entry["value"]).is_some(){return Err("duplicate seed field");}}
   let mut fields=vec![];let mut values=vec![];
   for (field_index,field) in table.fields.iter().enumerate(){
    let column=&columns[field_index];let value=supplied.remove(&field.reference.to_string());
    let Some(value)=value else{if !field.nullable&&column.get("default").is_none()&&column.get("generated").is_none(){return Err("seed omits required create field");}continue};
    if column["generated"]["generation"]=="always"{return Err("seed cannot override generated-always field");}
    let value=crate::table_operation::native_seed_value(field,value)?;
    budget=budget.checked_sub(value.to_string().len()+8).ok_or("seed native value expansion capacity exceeded")?;
    fields.push(field_index);values.push(value);
   }
   if !supplied.is_empty(){return Err("seed field outside selected table");}
   let ignore=set["conflict"]=="ignore";if ignore&&table.conflict_key.is_empty(){return Err("seed ignore needs original primary Key");}
   rows.push(SeedRow{table:index,fields,values,ignore,set_index});
  }
 }
 Ok(AdmittedSeed{request,tables,rows})
}
