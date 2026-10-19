// SPDX-License-Identifier: MIT OR Apache-2.0
//! Source-bound direct projection facts. No operand Carrier or SQL casts added.
use serde_json::{json,Value as Json};
use std::collections::BTreeMap;
use weft_core::{column_custody as custody,raw_read::{AdmittedRead,Table,Carrier}};
type Result<T>=std::result::Result<T,&'static str>;
pub(crate) struct ProjectionColumn {pub(crate) native:String,pub(crate) metadata:Json,pub(crate) generation:Json}
pub(crate) fn columns(a:&AdmittedRead,table:&Table,expected_core_version:&str)->Result<BTreeMap<String,ProjectionColumn>>{
 let doc=a.original_document();
 let exact=a.request().verification_profile.as_deref()==Some("source-exact/0.1");
 columns_for(&a.request().binding,&a.request().target,doc,a.original_record(&table.logical)?,table,exact,expected_core_version)
}
pub(crate) fn columns_for(binding:&weft_core::raw_read::Binding,target_name:&str,doc:&Json,record:&weft_core::model::Record,table:&Table,exact:bool,expected_core_version:&str)->Result<BTreeMap<String,ProjectionColumn>>{
 let raw=binding.native_columns.as_ref().ok_or("native ColumnSet envelope absent")?;
 let packet:custody::NativeColumnsEnvelope=serde_json::from_value(raw.clone()).map_err(|_|"closed native ColumnSet envelope required")?;
 let target=json!({"platform":if target_name=="postgresql18"{"postgresql"}else{"lakebase"},"majorVersion":"18"});
 if packet.target!=target||packet.sets.is_empty()||packet.sets.len()>256{return Err("native ColumnSet target/inventory differs")}
 let reference=json!({"module":record.identity.module,"element":record.identity.element});
 let mut facts=BTreeMap::new();let mut owners=BTreeMap::new();let members=record.value["members"].as_array().ok_or("Record members absent")?;
 for set in &packet.sets {
  let original=custody::original_column_set(&binding.model_sha256,doc,&target,set,expected_core_version)?;
  if !original["columns"].as_array().unwrap().iter().any(|c|members.contains(&c["field"])){continue}
  for (field,shape) in if exact {custody::admit_columns_with_collation(&binding.model_sha256,doc,&target,set,table,&reference,expected_core_version)?}else{custody::admit_columns(&binding.model_sha256,doc,&target,set,table,&reference,expected_core_version)?} {
   if facts.insert(field.clone(),shape).is_some(){return Err("duplicate original native column declaration")};owners.insert(field,set);
  }
 }
 if facts.is_empty(){return Err("original native ColumnSet for Record required")}
 let mut physical=std::collections::BTreeSet::new();for (native,_,_)in facts.values(){if !physical.insert(native){return Err("duplicate native physical column")}}
 let mut result=BTreeMap::new();
 for member in record.value["members"].as_array().ok_or("Record members absent")? {
  let Some((native,ty,_))=facts.get(&custody::field_key(member)) else {return Err("complete original Record native ColumnSet inventory required")};
  let owner=doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==member["module"]).ok_or("Field module absent")?;
  let field=owner["elements"].as_array().unwrap().iter().find(|f|f["id"]==member["element"]).ok_or("Field absent")?;
  let logical=field["name"].as_str().ok_or("Field name absent")?;
  if field["kind"]!="field"||!matches!(field["cardinality"].as_str(),Some("one"|"array"))||field.get("allowedValues").is_some()||field["extensions"].as_object().is_some_and(|v|!v.is_empty()){return Err("native projection Field constraints unsupported")}
  let facets=field.get("facets").cloned().unwrap_or(json!({}));
  let mut meta=json!({"carrier":field["scalarType"],"nullable":custody::row_null(doc,&reference,member,&facts)?,"nativeType":ty});
  let vector=field["cardinality"]=="array";
  if vector {meta["carrier"]=json!("vector");}
  match if vector {Some("vector")}else{field["scalarType"].as_str()}{
   Some("string") if matches!(ty.as_str(),"text")&&facets.as_object().is_some_and(|m|m.is_empty())=>{},
   Some("boolean") if matches!(ty.as_str(),"bool"|"boolean")&&facets.as_object().is_some_and(|m|m.is_empty())=>{},
   Some("integer")=>{let w=&facets["integerWidth"];let expected=match w["bits"].as_u64(){Some(16)=>"smallint",Some(32)=>"integer",Some(64)=>"bigint",_=>return Err("PG signed native width required")};if w["signed"]!=true||w.as_object().is_none_or(|m|m.len()!=2)||facets.as_object().is_none_or(|m|m.len()!=1)||ty!=expected{return Err("PG Integer native shape differs")};meta["integerWidth"]=w.clone();},
   Some("decimal")=>{let p=facets["precision"].as_u64().filter(|v|(1..=1000).contains(v)).ok_or("PG numeric precision required")?;let s=facets["scale"].as_u64().filter(|v|*v<=p).ok_or("PG numeric scale required")?;if facets.as_object().is_none_or(|m|m.len()!=2)||ty!=&format!("numeric({p},{s})"){return Err("PG Decimal native shape differs")};meta["precision"]=json!(p);meta["scale"]=json!(s);},
   Some("binary")=>{if ty!="bytea"||facets.as_object().is_none_or(|m|!m.is_empty()){return Err("PG Binary native shape differs")};meta["decoder"]=json!({"profile":"pg-bytea/0.1"});},
   Some("vector")=>{let declaration=custody::vector_declaration(doc,&reference,member)?;let n=declaration["dimensions"].as_u64().filter(|v|(1..=16000).contains(v)).ok_or("PG vector dimensions required")?;
    if ty!=&format!("vector({n})"){return Err("PG Vector native shape differs")}
    let ext=packet.vector_extension.as_ref().ok_or("PG vector extension pin required")?;if !matches!(ext.version.as_str(),"0.8.6"|"0.8.7")||ext.schema.is_empty(){return Err("PG vector extension pin unsupported")};
    meta["dimensions"]=json!(n);meta["decoder"]=json!({"profile":format!("pgvector-text-float32/{}",ext.version),"dimensions":n});meta["extensionSchema"]=json!(ext.schema);meta["extensionVersion"]=json!(ext.version);
   },
   _=>return Err("native projection source carrier/shape unsupported")
  }
  if let Some(c)=table.columns.iter().find(|c|c.logical==logical){if c.native!=*native||serde_json::to_value(&c.carrier).unwrap()!=meta["carrier"]||json!(c.nullable)!=meta["nullable"]{return Err("original native column differs from scalar binding")}}else if !vector&&field["scalarType"]!="binary"{return Err("native scalar column missing scalar binding")}
  if exact {let collation=custody::collation(doc,owners[&custody::field_key(member)],member)?;if (meta["carrier"]=="string")==collation.is_null(){return Err("native collation fact contradicts supported physical type")};meta["collation"]=collation;}
  result.insert(logical.to_string(),ProjectionColumn{native:native.clone(),metadata:meta,generation:custody::generation(doc,owners[&custody::field_key(member)],member)?});
 }
 for c in &table.columns {if !result.contains_key(&c.logical){return Err("bound scalar column absent from native custody")};if matches!(c.carrier,Carrier::Integer|Carrier::Decimal)&&result[&c.logical].metadata["carrier"]!=serde_json::to_value(&c.carrier).unwrap(){return Err("numeric custody differs")}}
 Ok(result)
}
pub(crate) fn admission(c:&ProjectionColumn,table:&Table,schema:&Option<String>)->Json{admission_source(c,table,schema,false)}
pub(crate) fn action_admission(c:&ProjectionColumn,table:&Table,schema:&Option<String>)->Json{admission_source(c,table,schema,true)}
fn admission_source(c:&ProjectionColumn,table:&Table,schema:&Option<String>,action:bool)->Json{
 let literal=|v:&str|json!({"source":{"literal":{"string":v}},"carrier":"string"});
 let mut parameters=vec![match schema{Some(s)=>literal(s),None=>json!({"source":{"appSchema":true},"carrier":"string"})},literal(&table.native),literal(&c.native),literal(c.metadata["nativeType"].as_str().unwrap())];
 let generation=match c.generation.as_str(){None=>"a.attidentity='' AND a.attgenerated='' AND NOT a.atthasdef",Some("identity")=>"a.attidentity IN ('a','d') AND a.attgenerated=''",Some("stored-expression")=>"a.attidentity='' AND a.attgenerated='s'",Some("default")=>"a.attidentity='' AND a.attgenerated='' AND a.atthasdef",_=>unreachable!()};
 let vector=c.metadata["carrier"]=="vector";
 if vector {parameters.push(literal(c.metadata["extensionSchema"].as_str().unwrap()));parameters.push(literal(c.metadata["extensionVersion"].as_str().unwrap()));}
 let collation_start=parameters.len()+1;
 if let Some(c)=c.metadata.get("collation").filter(|v|!v.is_null()){parameters.push(literal(c["schema"].as_str().unwrap()));parameters.push(literal(c["name"].as_str().unwrap()));}
 for(i,p)in parameters.iter_mut().enumerate(){p["name"]=json!(format!("__umf_native_{}",i+1));}
 let current_schema=action&&schema.is_none();if current_schema{parameters.remove(0);}
 let render=|dbapi:bool|{let slot=|n:usize|if current_schema&&n==1{unreachable!("schema slot omitted for original search-path relation resolution")}else if dbapi{format!("%(__umf_native_{n})s::pg_catalog.text")}else{format!("${}::pg_catalog.text",if current_schema{n-1}else{n})};
 let extension_membership=if c.metadata.get("collation").is_some(){" AND EXISTS(SELECT 1 FROM pg_catalog.pg_depend dep WHERE dep.classid='pg_catalog.pg_type'::pg_catalog.regclass AND dep.objid=ty.oid AND dep.objsubid=0 AND dep.refclassid='pg_catalog.pg_extension'::pg_catalog.regclass AND dep.refobjid=e.oid AND dep.refobjsubid=0 AND dep.deptype='e')"}else{""};
 let typecheck=if vector {format!("ty.typname='vector' AND tn.nspname={} AND a.atttypmod={} AND ('vector('||a.atttypmod::pg_catalog.text||')')={} AND EXISTS(SELECT 1 FROM pg_catalog.pg_extension AS e WHERE e.extname='vector' AND e.extnamespace=tn.oid AND e.extversion={}{extension_membership})",slot(5),c.metadata["dimensions"],slot(4),slot(6))}else{format!("tn.nspname='pg_catalog' AND pg_catalog.format_type(a.atttypid,a.atttypmod)=CASE {} WHEN 'bool' THEN 'boolean' ELSE {} END",slot(4),slot(4))};
 let collation=match c.metadata.get("collation"){None=>String::new(),Some(v)if v.is_null()=>" AND a.attcollation=0".to_string(),Some(_)=>format!(" AND EXISTS(SELECT 1 FROM pg_catalog.pg_collation co JOIN pg_catalog.pg_namespace cn ON cn.oid=co.collnamespace WHERE co.oid=a.attcollation AND cn.nspname={} AND co.collname={})",slot(collation_start),slot(collation_start+1))};
 let schema_predicate=if current_schema{format!("t.oid=pg_catalog.to_regclass(pg_catalog.quote_ident({}))",slot(2))}else{format!("ns.nspname={}",slot(1))};
 format!("SELECT (NOT a.attisdropped AND a.attnum>0 AND a.attnotnull={} AND {generation} AND {typecheck}{collation}) AS correspondence FROM pg_catalog.pg_class AS t JOIN pg_catalog.pg_namespace AS ns ON ns.oid=t.relnamespace JOIN pg_catalog.pg_attribute AS a ON a.attrelid=t.oid JOIN pg_catalog.pg_type AS ty ON ty.oid=a.atttypid JOIN pg_catalog.pg_namespace AS tn ON tn.oid=ty.typnamespace WHERE t.relkind='r' AND {schema_predicate} AND t.relname={} AND a.attname={}",if c.metadata["nullable"]==false{"TRUE"}else{"FALSE"},slot(2),slot(3))};
 json!({"dbapiSql":render(true),"sql":render(false),"parameters":parameters,"dbapiParameterOrder":parameters.iter().map(|p|p["name"].clone()).collect::<Vec<_>>()})
}

pub(crate) fn table_admission(table:&Table,schema:&Option<String>,columns:&[String])->Json{table_admission_source(table,schema,columns,false)}
pub(crate) fn action_table_admission(table:&Table,schema:&Option<String>,columns:&[String])->Json{table_admission_source(table,schema,columns,true)}
fn table_admission_source(table:&Table,schema:&Option<String>,columns:&[String],action:bool)->Json{
 let literal=|v:&str|json!({"source":{"literal":{"string":v}},"carrier":"string"});
 let mut names=columns.to_vec();names.sort();names.dedup();
 let mut parameters=vec![match schema{Some(s)=>literal(s),None=>json!({"source":{"appSchema":true},"carrier":"string"})},literal(&table.native),literal(&serde_json::to_string(&names).unwrap())];
 for(i,p)in parameters.iter_mut().enumerate(){p["name"]=json!(format!("__umf_inventory_{}",i+1));}
 let current_schema=action&&schema.is_none();if current_schema{parameters.remove(0);}
 let render=|dbapi:bool|{let slot=|n:usize|if current_schema&&n==1{unreachable!("schema slot omitted for original search-path relation resolution")}else if dbapi{format!("%(__umf_inventory_{n})s::pg_catalog.text")}else{format!("${}::pg_catalog.text",if current_schema{n-1}else{n})};
  let schema_predicate=if current_schema{format!("c.oid=pg_catalog.to_regclass(pg_catalog.quote_ident({}))",slot(2))}else{format!("n.nspname={}",slot(1))};
  format!("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE {schema_predicate} AND c.relname={} AND c.relkind='r' AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_inherits inh WHERE inh.inhparent=c.oid OR inh.inhrelid=c.oid) AND (SELECT pg_catalog.array_agg(a.attname::pg_catalog.text ORDER BY a.attname COLLATE pg_catalog.\"C\") FROM pg_catalog.pg_attribute a WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped)=ARRAY(SELECT pg_catalog.json_array_elements_text({}::pg_catalog.json)))",slot(2),slot(3))};
 json!({"sql":render(false),"dbapiSql":render(true),"parameters":parameters,"dbapiParameterOrder":parameters.iter().map(|p|p["name"].clone()).collect::<Vec<_>>()})
}
