// SPDX-License-Identifier: MIT OR Apache-2.0
//! Source-selected administrative seeds; no synthesized source commands.
use crate::raw_read::{Binding,Table,Carrier};
use crate::table_operation::OperationField;
use serde::{Deserialize,Serialize};
use serde_json::{Value,json};
use std::collections::{BTreeMap,BTreeSet};
use std::sync::Arc;
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
struct SeedTableIndexes {field_positions:BTreeMap<String,usize>,required_fields:BTreeSet<usize>,generated_always:BTreeSet<usize>}
fn query_name(value:&Value)->Result<&str>{value["name"].as_str().or_else(||value["id"].as_str()).ok_or("seed logical name absent")}
fn prepare_table(table:&Value,module:&Value,binding:&Binding,bound_tables:&BTreeMap<Vec<String>,usize>,elements:&BTreeMap<&str,&Value>)->Result<(SeedTable,SeedTableIndexes)>{
 let record=*elements.get(table["record"]["element"].as_str().ok_or("seed record identity absent")?).ok_or("seed record absent")?;
 let path=vec![module["namespace"].as_str().unwrap().to_owned(),query_name(record)?.to_owned()];
 let bound_index=*bound_tables.get(&path).ok_or("seed native table mapping absent")?;
 let bound=binding.tables.get(bound_index).ok_or("seed native table mapping absent")?;
 let native_columns=bound.columns.iter().map(|c|(c.native.as_str(),c)).collect::<BTreeMap<_,_>>();
 let mut fields=vec![];
 for column in table["columns"].as_array().unwrap(){
  let field=*elements.get(column["field"]["element"].as_str().ok_or("seed Field identity absent")?).ok_or("seed Field absent")?;
  let native=*native_columns.get(column["name"].as_str().unwrap()).ok_or("seed native column absent")?;
  if field["semanticTypes"].as_array().is_some_and(|v|!v.is_empty()){return Err("seed semantic type interpreter unavailable");}
  let prepared_literal=if matches!(native.carrier,Carrier::Date|Carrier::Timestamp|Carrier::Uuid|Carrier::Bits128){
   if field["facets"].as_object().is_some_and(|v|!v.is_empty())||field["allowedValues"].as_array().is_some_and(|v|!v.is_empty()){return Err("seed exact carrier qualifiers unsupported");}
   None
  }else{Some(Arc::new(crate::security_literals::PreparedLiteral::new(field).map_err(|_|"seed Field qualifiers unsupported")?))};
  let mut native_type=column["nativeType"].clone();
  for key in ["length","precision","scale"]{if let Some(v)=native_type.get(key){native_type[key]=json!(crate::security_literals::unsigned(v).map_err(|_|"seed native capacity invalid")?);}}
  let resolved=OperationField{reference:column["field"].clone(),native:native.native.clone(),carrier:native.carrier.clone(),nullable:native.nullable,declaration:field.clone(),native_type,prepared_literal};
  if column["default"]["kind"]=="literal"{crate::table_operation::native_seed_value(&resolved,&column["default"]["value"])?;}
  fields.push(resolved);
 }
 let mut field_positions=BTreeMap::new();let mut required_fields=BTreeSet::new();let mut generated_always=BTreeSet::new();
 for (index,(field,column)) in fields.iter().zip(table["columns"].as_array().unwrap()).enumerate(){
  field_positions.entry(field.reference.to_string()).or_insert(index);
  if !field.nullable&&column.get("default").is_none()&&column.get("generated").is_none(){required_fields.insert(index);}
  if column["generated"]["generation"]=="always"{generated_always.insert(index);}
 }
 let mut conflict_key=vec![];
 if let Some(key)=record["keys"].as_array().unwrap().iter().find(|k|k["primary"]==true){for reference in key["fields"].as_array().unwrap(){let index=*field_positions.get(&reference.to_string()).ok_or("seed primary field unmapped")?;conflict_key.push(fields[index].native.clone());}}
 let independent=!table["foreignKeys"].as_array().unwrap().iter().any(|fk|fk["target"]["record"]==table["record"]);
 Ok((SeedTable{table:bound.clone(),fields,conflict_key,independent},SeedTableIndexes{field_positions,required_fields,generated_always}))
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
 let mut identities=BTreeSet::new();let mut set_positions=BTreeMap::new();for (index,set) in sets.iter().enumerate(){let id=set["id"].as_str().ok_or("seed identity absent")?;if !identities.insert(id){return Err("duplicate seed set identity");}set_positions.insert(id.to_owned(),index);}
 let mut bound_tables=BTreeMap::new();for (index,bound) in request.binding.tables.iter().enumerate(){bound_tables.entry(bound.logical.clone()).or_insert(index);}
 let mut selected=BTreeSet::new();let mut last=None;let mut ordered=vec![];
 for id in &request.seed_sets{
  if !selected.insert(id){return Err("duplicate seed selection");}
  let index=*set_positions.get(id.as_str()).ok_or("seed set not declared")?;
  if last.is_some_and(|n|index<=n){return Err("seed selection must retain authored set order");}last=Some(index);ordered.push(&sets[index]);
 }
 let mut tables=vec![];let mut table_indexes=vec![];let mut prepared=BTreeMap::new();let mut rows=vec![];let mut budget=8_388_608usize;
 for (set_index,set) in ordered.iter().enumerate(){
  let authored=set["rows"].as_array().ok_or("seed rows absent")?;if authored.len()>10000{return Err("seed set row capacity exceeded");}
  for row in authored{
   if rows.len()>=10000{return Err("seed invocation row capacity exceeded");}
   let id=row["table"].as_str().ok_or("seed table identity absent")?;
   let declaration=*declarations.get(id).ok_or("seed table outside module")?;
   let index=if let Some(index)=prepared.get(id){*index}else{let n=tables.len();let (table,indexes)=prepare_table(declaration,module,&request.binding,&bound_tables,&elements)?;tables.push(table);table_indexes.push(indexes);prepared.insert(id,n);n};
   let table=&tables[index];
   let mut supplied=BTreeMap::new();for entry in row["values"].as_array().ok_or("seed values absent")?{if supplied.insert(entry["field"].to_string(),&entry["value"]).is_some(){return Err("duplicate seed field");}}
   let indexes=&table_indexes[index];
   let mut selected=vec![];let mut has_unknown=false;
   for (reference,value) in supplied {
    if let Some(field_index)=indexes.field_positions.get(&reference).copied(){selected.push((field_index,value));}else{has_unknown=true;}
   }
   selected.sort_by_key(|(field_index,_)|*field_index);
   let present=selected.iter().map(|(field_index,_)|*field_index).collect::<BTreeSet<_>>();
   let first_missing=if present.iter().filter(|field|indexes.required_fields.contains(field)).count()==indexes.required_fields.len(){None}else{indexes.required_fields.iter().find(|field|!present.contains(field)).copied()};
   let mut converted=vec![];
   for (field_index,value) in selected {
    if first_missing.is_some_and(|missing|missing<field_index){return Err("seed omits required create field");}
    if indexes.generated_always.contains(&field_index){return Err("seed cannot override generated-always field");}
    let value=crate::table_operation::native_seed_value(&table.fields[field_index],value)?;
    budget=budget.checked_sub(value.to_string().len()+8).ok_or("seed native value expansion capacity exceeded")?;
    converted.push((field_index,value));
   }
   if first_missing.is_some(){return Err("seed omits required create field");}
   if has_unknown{return Err("seed field outside selected table");}
   let (fields,values)=converted.into_iter().unzip();
   let ignore=set["conflict"]=="ignore";if ignore&&table.conflict_key.is_empty(){return Err("seed ignore needs original primary Key");}
   rows.push(SeedRow{table:index,fields,values,ignore,set_index});
  }
 }
 Ok(AdmittedSeed{request,tables,rows})
}
