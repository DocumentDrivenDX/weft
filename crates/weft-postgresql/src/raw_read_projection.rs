//! Source-bound direct projection facts. No operand Carrier or SQL casts added.
use serde_json::{json,Value as Json};
use std::collections::BTreeMap;
use weft_core::{column_custody as custody,raw_read::{AdmittedRead,Table,Carrier}};
type Result<T>=std::result::Result<T,&'static str>;
pub(crate) struct ProjectionColumn {pub(crate) native:String,pub(crate) metadata:Json,pub(crate) generation:Json}
pub(crate) fn columns(a:&AdmittedRead,table:&Table)->Result<BTreeMap<String,ProjectionColumn>>{
 let raw=a.request().binding.native_columns.as_ref().ok_or("native ColumnSet envelope absent")?;
 let packet:custody::NativeColumnsEnvelope=serde_json::from_value(raw.clone()).map_err(|_|"closed native ColumnSet envelope required")?;
 let target=json!({"platform":if a.request().target=="postgresql18"{"postgresql"}else{"lakebase"},"majorVersion":"18"});
 if packet.target!=target||packet.sets.is_empty()||packet.sets.len()>256{return Err("native ColumnSet target/inventory differs")}
 let doc=a.original_document();let record=a.original_record(&table.logical)?;
 let reference=json!({"module":record.identity.module,"element":record.identity.element});
 let mut admitted=vec![];
 for set in &packet.sets {if let Ok(facts)=custody::admit_columns(&a.request().binding.model_sha256,doc,&target,set,table,&reference){admitted.push((set,facts))}}
 if admitted.len()!=1{return Err("unique original native ColumnSet for Record required")}
 let(set,facts)=admitted.remove(0);let mut result=BTreeMap::new();
 for member in record.value["members"].as_array().ok_or("Record members absent")? {
  let Some((native,ty,_))=facts.get(&custody::field_key(member)) else {continue};
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
   Some("vector")=>{let size=&facets["collectionSize"];let n=size["min"].as_u64().filter(|v|(1..=16000).contains(v)).ok_or("PG vector dimensions required")?;
    let leaf=doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==field["itemType"]["module"]).and_then(|m|m["elements"].as_array().unwrap().iter().find(|v|v["id"]==field["itemType"]["element"])).ok_or("PG Vector leaf absent")?;
    let indexed=doc["modules"].as_array().unwrap().iter().flat_map(|m|m["extensions"]["umf.access"]["search"].as_array().into_iter().flatten()).any(|i|i["kind"]=="vector"&&i["record"]==reference&&i["vector"]["field"]==*member&&i["vector"]["dimensions"]==n);
    if !indexed||facets.as_object().is_none_or(|m|m.len()!=1)||size!=&json!({"min":n,"max":n})||leaf["scalarType"]!="float"||leaf["cardinality"]!="one"||leaf["nullability"]!="required"||leaf.get("allowedValues").is_some()||leaf["extensions"].as_object().is_none_or(|m|!m.is_empty())||leaf.get("facets").is_some_and(|m|m.as_object().is_none_or(|m|!m.is_empty()))||ty!=&format!("vector({n})"){return Err("PG Vector native shape differs")}
    let ext=packet.vector_extension.as_ref().ok_or("PG vector extension pin required")?;if !matches!(ext.version.as_str(),"0.8.6"|"0.8.7")||ext.schema.is_empty(){return Err("PG vector extension pin unsupported")};
    meta["dimensions"]=json!(n);meta["decoder"]=json!({"profile":format!("pgvector-text-float32/{}",ext.version),"dimensions":n});meta["extensionSchema"]=json!(ext.schema);meta["extensionVersion"]=json!(ext.version);
   },
   _=>return Err("native projection source carrier/shape unsupported")
  }
  if let Some(c)=table.columns.iter().find(|c|c.logical==logical){if c.native!=*native||serde_json::to_value(&c.carrier).unwrap()!=meta["carrier"]||json!(c.nullable)!=meta["nullable"]{return Err("original native column differs from scalar binding")}}else if !vector&&field["scalarType"]!="binary"{return Err("native scalar column missing scalar binding")}
  result.insert(logical.to_string(),ProjectionColumn{native:native.clone(),metadata:meta,generation:custody::generation(doc,set,member)?});
 }
 for c in &table.columns {if !result.contains_key(&c.logical){return Err("bound scalar column absent from native custody")};if matches!(c.carrier,Carrier::Integer|Carrier::Decimal)&&result[&c.logical].metadata["carrier"]!=serde_json::to_value(&c.carrier).unwrap(){return Err("numeric custody differs")}}
 Ok(result)
}
pub(crate) fn admission(c:&ProjectionColumn,table:&Table,schema:&Option<String>)->Json{
 let literal=|v:&str|json!({"source":{"literal":{"string":v}},"carrier":"string"});
 let mut parameters=vec![match schema{Some(s)=>literal(s),None=>json!({"source":{"appSchema":true},"carrier":"string"})},literal(&table.native),literal(&c.native),literal(c.metadata["nativeType"].as_str().unwrap())];
 let generation=match c.generation.as_str(){None=>"a.attidentity='' AND a.attgenerated='' AND NOT a.atthasdef",Some("identity")=>"a.attidentity IN ('a','d') AND a.attgenerated=''",Some("stored-expression")=>"a.attidentity='' AND a.attgenerated='s'",Some("default")=>"a.attidentity='' AND a.attgenerated='' AND a.atthasdef",_=>unreachable!()};
 let vector=c.metadata["carrier"]=="vector";
 if vector {parameters.push(literal(c.metadata["extensionSchema"].as_str().unwrap()));parameters.push(literal(c.metadata["extensionVersion"].as_str().unwrap()));}
 for(i,p)in parameters.iter_mut().enumerate(){p["name"]=json!(format!("__umf_native_{}",i+1));}
 let render=|dbapi:bool|{let slot=|n:usize|if dbapi{format!("%(__umf_native_{n})s::pg_catalog.text")}else{format!("${n}::pg_catalog.text")};
 let typecheck=if vector {format!("ty.typname='vector' AND tn.nspname={} AND a.atttypmod={} AND ('vector('||a.atttypmod::pg_catalog.text||')')={} AND EXISTS(SELECT 1 FROM pg_catalog.pg_extension AS e WHERE e.extname='vector' AND e.extnamespace=tn.oid AND e.extversion={})",slot(5),c.metadata["dimensions"],slot(4),slot(6))}else{format!("tn.nspname='pg_catalog' AND pg_catalog.format_type(a.atttypid,a.atttypmod)=CASE {} WHEN 'bool' THEN 'boolean' ELSE {} END",slot(4),slot(4))};
 format!("SELECT (NOT a.attisdropped AND a.attnum>0 AND a.attnotnull={} AND {generation} AND {typecheck}) AS correspondence FROM pg_catalog.pg_class AS t JOIN pg_catalog.pg_namespace AS ns ON ns.oid=t.relnamespace JOIN pg_catalog.pg_attribute AS a ON a.attrelid=t.oid JOIN pg_catalog.pg_type AS ty ON ty.oid=a.atttypid JOIN pg_catalog.pg_namespace AS tn ON tn.oid=ty.typnamespace WHERE t.relkind='r' AND ns.nspname={} AND t.relname={} AND a.attname={}",if c.metadata["nullable"]==false{"TRUE"}else{"FALSE"},slot(1),slot(2),slot(3))};
 json!({"dbapiSql":render(true),"sql":render(false),"parameters":parameters,"dbapiParameterOrder":parameters.iter().map(|p|p["name"].clone()).collect::<Vec<_>>()})
}
