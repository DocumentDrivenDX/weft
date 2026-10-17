//! Explicit administrative Spark keyset pages; native snapshot SQL is owner-emitted.
//! Current UC and mutation transaction lowerers do not route through this profile.
use crate::raw_read::{ident,name,Lower};
use serde_json::{json,Value};
use sqlparser::ast::*;
use sqlparser::ast::Value as SqlValue;
use std::collections::{BTreeMap,BTreeSet};
use weft_core::raw_read::{AdmittedRead,Carrier,Request,Table};
type Result<T>=std::result::Result<T,&'static str>;
struct Scan<'a>{table:&'a Table,nullable:bool,native:String,version:Option<u64>}
fn field(e:&Expr)->Result<(String,String)>{
 let Expr::CompoundIdentifier(parts)=e else{return Err("direct qualified page Field required")};
 if parts.len()!=2{return Err("direct qualified page Field required")};Ok((name(&parts[0]),name(&parts[1])))
}
fn scan<'a>(r:&'a Request,factor:&TableFactor,nullable:bool,snapshot:bool)->Result<(String,Scan<'a>)>{
 let TableFactor::Table{name:object,alias:Some(alias),args:None,with_hints,version:None,with_ordinality:false,partitions,json_path:None,sample:None,index_hints}=factor else{return Err("ordinary bound page table alias required")};
 if !with_hints.is_empty()||!partitions.is_empty()||!index_hints.is_empty()||!alias.columns.is_empty(){return Err("page table modifiers unsupported")};
 let logical=object.0.iter().map(|p|match p{ObjectNamePart::Identifier(i)=>Ok(name(i)),_=>Err("static page table required")}).collect::<Result<Vec<_>>>()?;
 let table=r.binding.tables.iter().find(|t|t.logical==logical).ok_or("page source binding absent")?;
 if r.binding.native_columns.is_none()&&table.columns.iter().any(|c|!matches!(c.carrier,Carrier::String|Carrier::Boolean)){return Err("Spark page numeric domains need separate qualification")};
 let base=format!("{}.{}",ident(r.binding.schema.as_ref().ok_or("explicit page schema required")?)?,ident(&table.native)?);
 let version=if snapshot{Some(*r.snapshots.as_ref().and_then(|versions|versions.get(&table.native)).filter(|v|**v<=i64::MAX as u64).ok_or("original page snapshot version absent")?)}else{None};
 let native=if let Some(version)=version{format!("{base} VERSION AS OF {version}")}else{base};
 Ok((name(&alias.name),Scan{table,nullable,native,version}))
}
fn pairs(e:&Expr,new:&str,old:Option<String>,out:&mut BTreeSet<[String;2]>)->Result<String>{
 match e{
  Expr::Nested(e)=>pairs(e,new,old,out),
  Expr::BinaryOp{left,op:BinaryOperator::And,right}=>{let old=pairs(left,new,old,out)?;pairs(right,new,Some(old),out)},
  Expr::BinaryOp{left,op:BinaryOperator::Eq,right}=>{
   let a=field(left)?;let b=field(right)?;
   let (source,target)=if a.0==new&&b.0!=new{(b,a)}else if b.0==new&&a.0!=new{(a,b)}else{return Err("page join must link original and new aliases")};
   if old.as_ref().is_some_and(|old|old!=&source.0)||!out.insert([source.1,target.1]){return Err("page join pair inventory differs")};Ok(source.0)
  },_=>Err("page join requires declared equality pairs")
 }
}
pub(crate) fn lower(a:&AdmittedRead)->Result<Value>{lower_mode(a,a.request().snapshot)}
fn lower_mode(a:&AdmittedRead,snapshot:bool)->Result<Value>{
 let r=a.request();let q=a.query();
 if matches!(&r.binding.schema_binding,Some(weft_core::raw_read::SchemaBinding::SearchPath)){return Err("Spark pages require explicit qualified schema binding")};
 if r.target!="spark-delta4"||!r.keyset||r.binding.schema.is_none()||r.version!="weft.raw-read/0.1"{return Err("explicit local Spark page target required")};
 if !(1..=1000).contains(&r.row_cap)||r.parameters.len()>64||r.sql.len()>65536{return Err("page budget")};
 let mut parameter_names=BTreeSet::new();
 for parameter in &r.parameters{if parameter.name.starts_with("__umf_")||!parameter_names.insert(&parameter.name){return Err("duplicate/reserved page parameter")}}
 if q.with.is_some()||q.fetch.is_some()||!q.locks.is_empty()||q.for_clause.is_some()||q.settings.is_some()||q.format_clause.is_some()||!q.pipe_operators.is_empty(){return Err("page query modifiers unsupported")};
 let SetExpr::Select(s)=q.body.as_ref()else{return Err("page SELECT required")};
 if s.from.len()!=1||s.from[0].joins.len()>3||s.projection.is_empty()||s.projection.len()>256||!s.optimizer_hints.is_empty()||s.select_modifiers.is_some()||s.distinct.is_some()||s.top.is_some()||s.exclude.is_some()||s.into.is_some()||!s.lateral_views.is_empty()||s.prewhere.is_some()||!s.connect_by.is_empty()||!s.cluster_by.is_empty()||!s.distribute_by.is_empty()||!s.sort_by.is_empty()||!s.named_window.is_empty()||s.qualify.is_some()||s.value_table_mode.is_some()||s.having.is_some(){return Err("bounded scalar page SELECT required")};
 let GroupByExpr::Expressions(group,modifiers)=&s.group_by else{return Err("page grouping unsupported")};if !group.is_empty()||!modifiers.is_empty(){return Err("page grouping unsupported")};
 let (root_alias,root)=scan(r,&s.from[0].relation,false,snapshot)?;
 let root_table=root.table;let mut from=format!("{} AS {}",root.native,ident(&root_alias)?);
 let mut scans=BTreeMap::from([(root_alias.clone(),root)]);
 // Prepare the complete native inventory before lowering predicates/projections.
 for join in &s.from[0].joins{
  if join.global{return Err("GLOBAL page joins unsupported")};
  let(left,on)=match &join.join_operator{JoinOperator::Inner(JoinConstraint::On(on))|JoinOperator::Join(JoinConstraint::On(on))=>(false,on),JoinOperator::LeftOuter(JoinConstraint::On(on))|JoinOperator::Left(JoinConstraint::On(on))=>(true,on),_=>return Err("page joins require declared inner/left ON relationships")};
  let(alias,next)=scan(r,&join.relation,left,snapshot)?;if scans.contains_key(&alias){return Err("duplicate page table alias")};
  let mut observed=BTreeSet::new();let old=pairs(on,&alias,None,&mut observed)?;
  let source=scans.get(&old).ok_or("page join source not admitted")?.table;
  let relationship=r.binding.relationships.iter().find(|rel|rel.to_one&&rel.source==source.logical&&rel.target==next.table.logical&&rel.pairs.iter().cloned().collect::<BTreeSet<_>>()==observed).ok_or("page join is not an original declared to-one relationship")?;
  if relationship.id.is_empty()||relationship.pairs.iter().map(|p|p[1].clone()).collect::<Vec<_>>()!=next.table.key{return Err("page relationship must cover target unique Key")};
  let native=next.native.clone();scans.insert(alias.clone(),next);
  let mut predicate=Lower{request:r,table:root_table,alias:root_alias.clone(),scans:Some(scans.iter().map(|(alias,scan)|(alias.clone(),(scan.table,scan.nullable))).collect()),slots:vec![],used:BTreeSet::new()};
  let condition=predicate.predicate(on)?;if !predicate.slots.is_empty(){return Err("join parameters not admitted")};
  from+=&format!(" {} JOIN {} AS {} ON {}",if left{"LEFT"}else{"INNER"},native,ident(&alias)?,condition);
 }
 let mut relations=BTreeMap::new();for scan in scans.values(){if let Some(previous)=relations.insert(scan.table.native.clone(),scan){if previous.table.logical!=scan.table.logical{return Err("ambiguous page native source alias")}}}
 if let Some(versions)=&r.snapshots{if versions.len()!=relations.len()||relations.keys().any(|name|!versions.contains_key(name)){return Err("exact complete page snapshot inventory required")}}
 let shapes=if r.binding.native_columns.is_some(){Some(scans.iter().map(|(alias,scan)|Ok((alias.clone(),crate::spark_shapes::columns(a,scan.table)?))).collect::<Result<BTreeMap<_,_>>>()?)}else{None};
 let mut l=Lower{request:r,table:root_table,alias:root_alias.clone(),scans:Some(scans.iter().map(|(alias,scan)|(alias.clone(),(scan.table,scan.nullable))).collect()),slots:vec![],used:BTreeSet::new()};
 let mut outputs=vec![];let mut projection=vec![];let mut names=BTreeSet::new();
 for item in &s.projection{
  let(expr,alias)=match item{SelectItem::ExprWithAlias{expr,alias}=>(expr,name(alias)),SelectItem::UnnamedExpr(expr)=>{let(_,name)=field(expr)?;(expr,name)},_=>return Err("explicit scalar page projection required")};
  if alias.starts_with("__umf_order_")||!names.insert(alias.clone()){return Err("duplicate/reserved page output")};
  if let Some(shapes)=&shapes {
   if let Ok((owner,logical))=field(expr){
    let shape=shapes.get(&owner).and_then(|columns|columns.get(&logical)).ok_or("projected Field outside original Spark ColumnSet")?;
    let mut meta=shape.metadata.clone();meta["name"]=json!(alias);if scans[&owner].nullable{meta["nullable"]=json!(true)};
    projection.push(format!("{}.{} AS {}",ident(&owner)?,ident(&shape.native)?,ident(&alias)?));outputs.push(meta);continue;
   }
  }
  let(sql,carrier,nullable)=match expr{Expr::IsNull(_)|Expr::IsNotNull(_)=>(l.predicate(expr)?,Carrier::Boolean,false),_=>l.column(expr)?};
  projection.push(format!("{sql} AS {}",ident(&alias)?));outputs.push(json!({"name":alias,"carrier":carrier,"nullable":nullable}));
 }
 let selection=s.selection.as_ref().map(|e|l.predicate(e)).transpose()?;
 let order=q.order_by.as_ref().ok_or("page complete ordering required")?;if order.interpolate.is_some(){return Err("page order interpolation unsupported")};let OrderByKind::Expressions(items)=&order.kind else{return Err("static page ordering required")};if items.is_empty()||items.len()>16{return Err("page ordering bound")};
 let mut cursor_parameters=vec![];let mut cursor_outputs=vec![];let mut cursor_ordering=vec![];let mut seen=BTreeSet::new();let mut equal=vec![];let mut after=vec![];let mut ordering=vec![];
 l.slots.push("__umf_has_after".into());let has_after=format!(":p{}",l.slots.len());cursor_parameters.push(json!({"name":"__umf_has_after","carrier":"boolean","nullable":false}));
 for(i,item)in items.iter().enumerate(){
  if item.with_fill.is_some(){return Err("page ordering fill unsupported")};
  let key=field(&item.expr)?;if !seen.insert(key.clone()){return Err("duplicate page ordering Field")};let(sql,carrier,nullable)=l.column(&item.expr)?;
  let direction=match item.options.sort{None|Some(OrderBySort::Asc)=>"ASC",Some(OrderBySort::Desc)=>"DESC",_=>return Err("page ordering operator unsupported")};let nulls_first=item.options.nulls_first.unwrap_or(false);let nulls=if nulls_first{"FIRST"}else{"LAST"};
  let table=scans.get(&key.0).ok_or("page ordering source absent")?.table;
  cursor_ordering.push(json!({"table":table.logical,"field":key.1,"direction":direction,"nulls":nulls}));
  let parameter=format!("__umf_after_{i}");l.slots.push(parameter.clone());let slot=format!(":p{}",l.slots.len());cursor_parameters.push(json!({"name":parameter,"carrier":carrier,"nullable":true}));
  let output=format!("__umf_order_{i}");projection.push(format!("{sql} AS {}",ident(&output)?));if let Some(shapes)=&shapes{let mut meta=shapes.get(&key.0).and_then(|columns|columns.get(&key.1)).ok_or("ordered Field original shape absent")?.metadata.clone();meta["name"]=json!(output);meta["nullable"]=json!(nullable);cursor_outputs.push(meta)}else{cursor_outputs.push(json!({"name":output,"carrier":carrier,"nullable":nullable}));}
  let comparison=if direction=="ASC"{">"}else{"<"};let special=if nulls_first{format!("({slot} IS NULL AND {sql} IS NOT NULL)")}else{format!("({slot} IS NOT NULL AND {sql} IS NULL)")};
  let strict=format!("(({slot} IS NOT NULL AND {sql} {comparison} {slot}) OR {special})");let prefix=if equal.is_empty(){String::new()}else{format!("{} AND ",equal.join(" AND "))};after.push(format!("({prefix}{strict})"));equal.push(format!("({sql} <=> {slot})"));ordering.push(format!("{sql} {direction} NULLS {nulls}"));
 }
 if root_table.key.iter().any(|key|!seen.contains(&(root_alias.clone(),key.clone()))){return Err("page ordering requires complete original root unique Key")};
 if l.used.len()!=r.parameters.len(){return Err("unused/duplicate page parameter inventory")};
 let limit=match &q.limit_clause{None=>r.row_cap,Some(LimitClause::LimitOffset{limit:Some(Expr::Value(value)),offset:None,limit_by})if limit_by.is_empty()=>match &value.value{SqlValue::Number(value,false)=>value.parse::<u16>().ok().filter(|value|*value>0&&*value<=r.row_cap).ok_or("bounded page limit required")?,_=>return Err("literal page limit required")},_=>return Err("page offsets unsupported")};
 let seek=format!("({has_after}=FALSE OR ({}))",after.join(" OR "));let predicate=if let Some(selection)=selection{format!("({selection}) AND {seek}")}else{seek};
 let limit_sql=if r.stream_page_size {
  let reserved="__umf_page_size";if l.slots.iter().any(|p|p==reserved){return Err("reserved stream page size collision")};
  l.slots.push(reserved.into());cursor_parameters.push(json!({"name":reserved,"carrier":"integer","nullable":false}));format!("CAST(:p{} AS INT)",l.slots.len())
 }else{limit.to_string()};
 let sql=format!("SELECT {} FROM {from} WHERE {predicate} ORDER BY {} LIMIT {limit_sql}",projection.join(", "),ordering.join(", "));
 let mut native_checks=vec![];let mut snapshot_tables=vec![];let mut referenced=vec![];
 for scan in relations.values(){
  let table=scan.table;let keys=table.key.iter().map(|key|table.columns.iter().find(|c|&c.logical==key).ok_or("complete page Key binding required").and_then(|c|ident(&c.native))).collect::<Result<Vec<_>>>()?;if keys.is_empty(){return Err("complete page Key required")};let nulls=keys.iter().map(|key|format!("{key} IS NULL")).collect::<Vec<_>>().join(" OR ");let key_list=keys.join(", ");
  native_checks.push(json!({"sql":format!("SELECT COUNT(*) AS violations FROM (SELECT {key_list} FROM {} GROUP BY {key_list} HAVING COUNT(*) <> 1 OR {nulls}) AS invalid_keys",scan.native),"expected":0}));
  if let Some(version)=scan.version{snapshot_tables.push(json!({"table":table.native,"version":version}));}
  let columns=if let Some(shapes)=&shapes {
   let selected=shapes.iter().find(|(alias,_)|scans[*alias].table.logical==table.logical).map(|(_,v)|v).ok_or("original Spark shape source absent")?;
   selected.iter().map(|(logical,shape)|{let mut c=shape.metadata.clone();c["logical"]=json!(logical);c["native"]=json!(shape.native);c}).collect::<Vec<_>>()
  }else{serde_json::to_value(&table.columns).unwrap().as_array().unwrap().clone()};
  referenced.push(json!({"logical":table.logical,"native":table.native,"columns":columns}));
 }
 let mut out=json!({"version":"weft.raw-read/0.1","status":"compiled","target":r.target,"sql":sql,"sourceSql":r.sql,"binding":r.binding,"parameters":r.parameters,"parameterOrder":l.slots,"outputs":outputs,"nativeChecks":native_checks,"snapshotTables":snapshot_tables,"lockSql":[],"cursorParameters":cursor_parameters,"cursorOutputs":cursor_outputs,"cursorOrdering":cursor_ordering,"referencedTables":referenced,"obligations":{"transaction":if snapshot{"snapshot-read"}else{"live-read"},"snapshot":snapshot,"principal":"administrative-single-tenant","ansiEnabled":true,"sparkVersion":"4.0.1","deltaVersion":"4.0.0","binaryStringCollation":true,"escapedStringLiterals":false,"rowCap":limit,"timeout":"one-page-deadline","tableIdentity":if snapshot{"uuid-and-version"}else{"uuid"}}});
 if let Some(parameters)=crate::raw_numeric::parameters(r){out["obligations"]["numericPredicateParameters"]=parameters;}
 if let Some(parameters)=crate::raw_read::list_parameters(r){out["obligations"]["listPredicateParameters"]=parameters;}
 if !snapshot&&r.snapshots.is_some(){out["snapshotProgram"]=lower_mode(a,true)?;}
 if r.stream_page_size{out["obligations"]["streamPageSize"]=json!({"parameter":"__umf_page_size","minimum":1,"maximum":limit});}
 out["structuralDigest"]=json!(weft_core::json::sha256(weft_core::mutation::canonical(&out).as_bytes()));Ok(out)
}
#[cfg(test)]
mod tests {
 use super::*;
 fn request()->Value{serde_json::from_str(include_str!("../../../tests/spark-pages/read-request.json")).unwrap()}
 fn output(r:Value)->Value{serde_json::from_str(&crate::raw_read::compile_json(&r.to_string())).unwrap()}
 #[test]
 fn live_and_snapshot_are_distinct_original_programs(){
  let r=request();let live=output(r.clone());assert_eq!(live["status"],"compiled","{live}");assert_eq!(live,output(r.clone()));
  assert!(!live["sql"].as_str().unwrap().contains("VERSION AS OF"));assert_eq!(live["obligations"]["snapshot"],false);
  let child=&live["snapshotProgram"];assert_eq!(child["obligations"]["snapshot"],true);assert_eq!(child["snapshotTables"],json!([{"table":"customers","version":2},{"table":"orders","version":1}]));
  assert!(child["sql"].as_str().unwrap().contains("`customers` VERSION AS OF 2"));assert!(child["sql"].as_str().unwrap().contains("`orders` VERSION AS OF 1"));
  let mut snapshot=r;snapshot["snapshot"]=json!(true);assert_eq!(*child,output(snapshot));
  assert_eq!(live["cursorOutputs"][0]["nullable"],true);assert_eq!(live["cursorOutputs"][1]["nullable"],false);
  assert!(live["sql"].as_str().unwrap().contains("`c`.`name` ASC NULLS LAST, `o`.`id` DESC NULLS FIRST"));
  assert_eq!(live["nativeChecks"].as_array().unwrap().len(),2);
 }
 #[test]
 fn live_page_does_not_require_snapshot_configuration(){let mut r=request();r.as_object_mut().unwrap().remove("snapshots");let a=output(r);assert_eq!(a["status"],"compiled","{a}");assert!(a.get("snapshotProgram").is_none());}
 #[test]
 fn refuse_missing_snapshot_key_join_or_reserved_parameters(){
  for mode in 0..9{let mut r=request();match mode{
   0=>{r["snapshot"]=json!(true);r.as_object_mut().unwrap().remove("snapshots");},
   1=>{r["snapshots"].as_object_mut().unwrap().remove("customers");},
   2=>r["sql"]=json!("SELECT o.id FROM commerce.Order AS o ORDER BY o.status ASC"),
   3=>r["binding"]["relationships"][0]["toOne"]=json!(false),
   4=>r["parameters"]=json!([{"name":"__umf_has_after","carrier":"boolean","nullable":false}]),
   5=>r["snapshots"]["foreign"]=json!(1),
   6=>{r["target"]=json!("postgresql18");r["snapshot"]=json!(true);},
   7=>{r["sql"]=json!("SELECT o.id FROM commerce.Order AS o(renamed,customer_id,status) ORDER BY o.id ASC");r["snapshots"]=json!({"orders":1});},
   _=>r["sql"]=json!(r["sql"].as_str().unwrap().replace("LEFT JOIN","GLOBAL LEFT JOIN")),
  };let a=output(r);assert_eq!(a["status"],"blocked","mode {mode}: {a}");assert!(a.get("sql").is_none());}
 }
 #[test]
 fn source_bound_spark_shapes_preserve_exact_columns_and_refuse_wrong_custody(){
  let mut r=request();let doc:Value=serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();
  let physical=include_str!("../../../tests/spark-pages/binding.json");
  r["binding"]["nativeColumns"]=json!({"target":{"platform":"spark-delta","majorVersion":"spark4-delta4"},"sets":[{"columnSet":{"documentId":doc["id"],"coreVersion":"0.8.0","documentSha256":r["binding"]["modelSha256"],"module":"commerce","pointer":"/modules/0/extensions/umf.access/columns/0"},"binding":{"sourceId":"physical","documentJson":physical,"documentSha256":weft_core::json::sha256(physical.as_bytes())}}]});
  let compiled=output(r.clone());assert_eq!(compiled["status"],"compiled","{compiled}");
  assert_eq!(compiled["outputs"][0]["nativeType"],"string");assert_eq!(compiled["outputs"][1]["nullable"],true);
  for fault in 0..4{let mut bad=r.clone();match fault {
   0=>bad["binding"]["nativeColumns"]["target"]["majorVersion"]=json!("other"),
   1=>bad["binding"]["nativeColumns"]["sets"][0]["columnSet"]["documentSha256"]=json!("0".repeat(64)),
   2=>bad["binding"]["nativeColumns"]["sets"][0]["binding"]["documentSha256"]=json!("0".repeat(64)),
   _=>bad["binding"]["nativeColumns"]["vectorExtension"]=json!({"schema":"public","version":"0.8.7"}),
  };let result=output(bad);assert_eq!(result["status"],"blocked","fault {fault}: {result}");assert!(result.get("sql").is_none());}
 }

 #[test]
 fn original_spark_numeric_binary_projections_keep_physical_shapes(){
  let mut r=request();let mut doc:Value=serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();
  let mut binding:Value=serde_json::from_str(include_str!("../../../tests/spark-pages/binding.json")).unwrap();
  for (id,family,facets,ty) in [("small","integer",json!({"integerWidth":{"bits":16,"signed":true}}),"SMALLINT"),("amount","decimal",json!({"precision":18,"scale":4}),"DECIMAL(18,4)"),("payload","binary",json!({}),"BINARY")] {
   let reference=json!({"module":"commerce","element":id});
   doc["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":id,"name":id,"kind":"field","scalarType":family,"cardinality":"one","nullability":"required","facets":facets,"extensions":{}}));
   doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="Order").unwrap()["members"].as_array_mut().unwrap().push(reference.clone());
   doc["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"].as_array_mut().unwrap().push(json!({"field":reference,"row":{"missing":false,"null":false},"create":{"missing":false,"null":false},"mutable":true,"generated":false}));
   doc["modules"][0]["extensions"]["umf.access"]["columns"][0]["columns"].as_array_mut().unwrap().push(json!({"field":reference,"type":ty,"notNull":true,"generation":null}));
   binding["extensions"]["umf.binding"]["fields"].as_array_mut().unwrap().push(json!({"module":"commerce","element":id,"storage":"column","column":id}));
   if family!="binary" {r["binding"]["tables"][0]["columns"].as_array_mut().unwrap().push(json!({"logical":id,"native":id,"carrier":family,"nullable":false}));}
  }
  let vector=json!({"module":"commerce","element":"embedding"});
  doc["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":"component","name":"component","kind":"field","scalarType":"float","cardinality":"one","nullability":"required","extensions":{}}));
  doc["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":"embedding","name":"embedding","kind":"field","cardinality":"array","nullability":"required","extensions":{},"itemType":{"module":"commerce","element":"component"},"facets":{"collectionSize":{"min":3,"max":3}}}));
  doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="Order").unwrap()["members"].as_array_mut().unwrap().push(vector.clone());
  doc["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"].as_array_mut().unwrap().push(json!({"field":vector,"row":{"missing":false,"null":false},"create":{"missing":false,"null":false},"mutable":true,"generated":false}));
  doc["modules"][0]["extensions"]["umf.access"]["columns"][0]["columns"].as_array_mut().unwrap().push(json!({"field":vector,"type":"ARRAY<FLOAT>","notNull":true,"generation":null}));
  doc["modules"][0]["extensions"]["umf.access"]["search"].as_array_mut().unwrap().push(json!({"id":"semantic","record":{"module":"commerce","element":"Order"},"kind":"vector","exactness":"approximate","filterable":[],"vector":{"field":vector,"source":{"module":"commerce","element":"Order_status"},"model":{"id":"spark-fixture","version":"1"},"dimensions":3,"metric":"l2","computedBy":"host"}}));
  binding["extensions"]["umf.binding"]["fields"].as_array_mut().unwrap().push(json!({"module":"commerce","element":"embedding","storage":"column","column":"embedding"}));
  let source=serde_json::to_string(&doc).unwrap();let physical=binding.to_string();let sha=weft_core::json::sha256(source.as_bytes());
  r["binding"]["modelJson"]=json!(source);r["binding"]["modelSha256"]=json!(sha);
  r["binding"]["nativeColumns"]=json!({"target":{"platform":"spark-delta","majorVersion":"spark4-delta4"},"sets":[{"columnSet":{"documentId":doc["id"],"coreVersion":"0.8.0","documentSha256":sha,"module":"commerce","pointer":"/modules/0/extensions/umf.access/columns/0"},"binding":{"sourceId":"physical","documentJson":physical,"documentSha256":weft_core::json::sha256(physical.as_bytes())}}]});
  r["sql"]=json!(r#"SELECT o.small, o.amount, o.payload, o.embedding FROM "commerce"."Order" AS o ORDER BY o.id ASC"#);r["binding"]["relationships"]=json!([]);r["binding"]["tables"].as_array_mut().unwrap().truncate(1);r["snapshots"]=json!({"orders":1});
  let p=output(r);assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["outputs"][0]["nativeType"],"smallint");assert_eq!(p["outputs"][0]["integerWidth"]["bits"],16);assert_eq!(p["outputs"][1]["precision"],18);assert_eq!(p["outputs"][1]["scale"],4);assert_eq!(p["outputs"][2]["carrier"],"binary");assert_eq!(p["outputs"][3]["carrier"],"vector");assert_eq!(p["outputs"][3]["dimensions"],3);
 }

 #[test]
 fn stream_page_size_is_an_original_bounded_parameter(){
  let mut r=request();r["streamPageSize"]=json!(true);let p=output(r.clone());
  assert_eq!(p["status"],"compiled","{p}");
  assert_eq!(p["obligations"]["streamPageSize"],json!({"parameter":"__umf_page_size","minimum":1,"maximum":50}));
  assert_eq!(p["obligations"]["rowCap"],50);
  assert_eq!(p["parameterOrder"].as_array().unwrap().last().unwrap(),"__umf_page_size");
  assert!(p["sql"].as_str().unwrap().contains("LIMIT CAST(:p"));
  assert_eq!(p["snapshotProgram"]["obligations"]["streamPageSize"],p["obligations"]["streamPageSize"]);
  r["binding"]["schemaBinding"]=json!("search-path");let refused=output(r);assert_eq!(refused["status"],"blocked");assert!(refused.get("sql").is_none());
 }

 #[test]
 fn separate_record_columnsets_use_only_their_original_record_bindings(){
  let mut r=request();let mut doc:Value=serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();let physical:Value=serde_json::from_str(include_str!("../../../tests/spark-pages/binding.json")).unwrap();let original=doc["modules"][0]["extensions"]["umf.access"]["columns"][0].clone();let mut sets=vec![];let mut packets=vec![];
  for (index,record) in ["Order","Customer"].iter().enumerate(){
   let members=doc["modules"][0]["elements"].as_array().unwrap().iter().find(|e|e["id"]==*record).unwrap()["members"].as_array().unwrap();let id=format!("physical-{record}");
   let mut set=original.clone();set["binding"]["sourceId"]=json!(id);set["columns"].as_array_mut().unwrap().retain(|c|members.contains(&c["field"]));sets.push(set);
   let mut binding=physical.clone();binding["id"]=json!(id);binding["extensions"]["umf.binding"]["elements"].as_array_mut().unwrap().retain(|e|e["element"]==*record);binding["extensions"]["umf.binding"]["fields"].as_array_mut().unwrap().retain(|f|members.contains(&json!({"module":f["module"],"element":f["element"]})));let source=binding.to_string();
   packets.push(json!({"columnSet":{"documentId":doc["id"],"coreVersion":"0.8.0","documentSha256":"pending","module":"commerce","pointer":format!("/modules/0/extensions/umf.access/columns/{index}")},"binding":{"sourceId":id,"documentJson":source,"documentSha256":weft_core::json::sha256(source.as_bytes())}}));
  }
  doc["modules"][0]["extensions"]["umf.access"]["columns"]=json!(sets);let source=doc.to_string();let sha=weft_core::json::sha256(source.as_bytes());for packet in &mut packets{packet["columnSet"]["documentSha256"]=json!(sha)}
  r["binding"]["modelJson"]=json!(source);r["binding"]["modelSha256"]=json!(sha);r["binding"]["nativeColumns"]=json!({"target":{"platform":"spark-delta","majorVersion":"spark4-delta4"},"sets":packets});
  let compiled=output(r.clone());assert_eq!(compiled["status"],"compiled","{compiled}");assert_eq!(compiled["outputs"][1]["nativeType"],"string");
  r["binding"]["nativeColumns"]["sets"][0]["binding"]["documentSha256"]=json!("0".repeat(64));let refused=output(r);assert_eq!(refused["status"],"blocked");assert!(refused.get("sql").is_none());
 }

}
