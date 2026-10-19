// SPDX-License-Identifier: MIT OR Apache-2.0
//! Original filtered entity-set navigation; no runtime query construction.
use serde::Deserialize;
use serde_json::{json,Value as Json};
use sqlparser::ast::*;
use std::collections::BTreeSet;
use weft_core::raw_read::{AdmittedRead,Carrier,Table};
type Result<T>=std::result::Result<T,&'static str>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Relationship {module:String,id:String,direction:String}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {version:String,source:Json,target:Json,relationship:Relationship}
fn ident(value:&str)->Result<String>{if value.is_empty()||value.len()>63||value.contains('\0'){return Err("related native identifier outside profile")};Ok(format!("\"{}\"",value.replace('"',"\"\"")))}
fn name(value:&Ident)->String{if value.quote_style.is_some(){value.value.clone()}else{value.value.to_ascii_lowercase()}}
fn root(a:&AdmittedRead)->Result<(&Table,String)>{
 let SetExpr::Select(s)=a.query().body.as_ref() else{return Err("related single SELECT required")};
 let Some(first)=s.from.first()else{return Err("related source root absent")};
 let TableFactor::Table{name:n,alias:Some(alias),..}=&first.relation else{return Err("related original aliased Record root required")};
 let path=n.0.iter().map(|part|match part{ObjectNamePart::Identifier(i)=>Ok(name(i)),_=>Err("related dynamic root refused")}).collect::<Result<Vec<_>>>()?;
 let table=a.request().binding.tables.iter().find(|t|t.logical==path).ok_or("related Record binding absent")?;
 Ok((table,name(&alias.name)))
}
fn merge_array(target:&mut Json,source:&Json,path:&str){
 let Some(values)=source.pointer(path).and_then(Json::as_array)else{return};
 let current=target.pointer_mut(path).expect("merge path prepared").as_array_mut().unwrap();
 for value in values{if !current.contains(value){current.push(value.clone())}}
}
fn expanded_source(a:&AdmittedRead,table:&Table,alias:&str)->Result<Query>{
 let mut query=a.query().clone();let SetExpr::Select(selected)=query.body.as_mut()else{return Err("related single SELECT required")};let mut projection=vec![];
 for item in &selected.projection {match item{
  SelectItem::QualifiedWildcard(SelectItemQualifiedWildcardKind::ObjectName(path),options)=>{
   if path.0.len()!=1||!matches!(&path.0[0],ObjectNamePart::Identifier(i) if name(i)==alias)||options.opt_ilike.is_some()||options.opt_exclude.is_some()||options.opt_except.is_some()||options.opt_replace.is_some()||options.opt_rename.is_some()||options.opt_alias.is_some(){return Err("related wildcard requires unchanged original root Record")}
   let record=a.original_record(&table.logical)?;for reference in record.value["members"].as_array().ok_or("related original Record members absent")?{
    let module=a.original_document()["modules"].as_array().unwrap().iter().find(|m|m["id"]==reference["module"]).ok_or("related original Field module absent")?;
    let field=module["elements"].as_array().unwrap().iter().find(|f|f["id"]==reference["element"]).ok_or("related original Field absent")?;if field["kind"]!="field"{return Err("related wildcard member not a Field")};let logical=field["name"].as_str().ok_or("related original Field name absent")?;
    projection.push(SelectItem::ExprWithAlias{expr:Expr::CompoundIdentifier(vec![Ident::with_quote('"',alias),Ident::with_quote('"',logical)]),alias:Ident::with_quote('"',logical)});
   }
  },SelectItem::ExprWithAlias{..}|SelectItem::UnnamedExpr(_)=>projection.push(item.clone()),_=>return Err("related explicit root wildcard or direct projection required")
 }}
 selected.projection=projection;Ok(query)
}
fn lower(raw:&str)->Result<Json>{
 let original=weft_core::json::checked_json(raw).map_err(|_|"related duplicate or invalid JSON")?;
 let request:Request=serde_json::from_value(original.clone()).map_err(|_|"closed related request required")?;
 if request.version!="weft.related-read/0.1"||!matches!(request.relationship.direction.as_str(),"forward"|"inverse"){return Err("related request version/direction unsupported")}
 if request.source["binding"]!=request.target["binding"]||request.source["parameters"]!=request.target["parameters"]||request.source["target"]!=request.target["target"]{return Err("related requires identical original binding/target/parameter inventory")}
 if !matches!(request.source["target"].as_str(),Some("postgresql18"|"lakebase18")){return Err("related native target not qualified")}
 let source=weft_core::raw_read::admit_json(&request.source.to_string())?;let original_target=weft_core::raw_read::admit_json(&request.target.to_string())?;
 if source.request().verification_profile!=original_target.request().verification_profile{return Err("related source verification profiles differ")}
 let(target_record,target_root_alias)=root(&original_target)?;let mut target_request=request.target.clone();target_request["sql"]=json!(expanded_source(&original_target,target_record,&target_root_alias)?.to_string());let target=weft_core::raw_read::admit_json(&target_request.to_string())?;
 if source.request().keyset||source.request().stream_page_size||source.request().snapshot||target.request().snapshot{return Err("related source cannot be a page or snapshot")}
 let (source_table,source_alias)=root(&source)?;let(target_table,target_alias)=root(&target)?;
 let relations=source.request().binding.relationships.iter().filter(|r|r.id==request.relationship.id&&r.module.as_ref().is_none_or(|m|*m==request.relationship.module)).collect::<Vec<_>>();
 if relations.len()!=1{return Err("related original relation ambiguous or absent")};let relation=relations[0];
 let original_relations=source.original_document()["modules"].as_array().unwrap().iter().filter(|m|m["id"]==request.relationship.module).flat_map(|m|m["relationships"].as_array().into_iter().flatten()).filter(|r|r["id"]==request.relationship.id).collect::<Vec<_>>();
 if original_relations.len()!=1{return Err("related canonical original module/id absent")}
 let forward=request.relationship.direction=="forward";
 if source_table.logical!=if forward{relation.source.clone()}else{relation.target.clone()}||target_table.logical!=if forward{relation.target.clone()}else{relation.source.clone()}{return Err("related endpoint Records differ from original relation direction")}
 let mut query=expanded_source(&source,source_table,&source_alias)?;let SetExpr::Select(selected)=query.body.as_ref()else{unreachable!()};
 if !matches!(&selected.group_by,GroupByExpr::Expressions(v,modifiers) if v.is_empty()&&modifiers.is_empty())||selected.having.is_some(){return Err("related source grouped/aggregate identity unqualified")}
 for item in &selected.projection{let expr=match item{SelectItem::ExprWithAlias{expr,..}|SelectItem::UnnamedExpr(expr)=>expr,_=>return Err("related source explicit direct projections required")};if !matches!(expr,Expr::CompoundIdentifier(_)){return Err("related source aggregate/computed projections unqualified")}}
 let target_base=crate::raw_read::lower_related_fragment(&target,true,&[],None)?;
 let prefix=target_base["parameterOrder"].as_array().unwrap().iter().map(|v|v.as_str().unwrap().to_string()).collect::<Vec<_>>();
 let SetExpr::Select(selection)=query.body.as_mut()else{unreachable!()};let mut aliases=BTreeSet::new();
 for item in &selection.projection{match item{SelectItem::ExprWithAlias{alias,..}=>{aliases.insert(name(alias));},SelectItem::UnnamedExpr(Expr::CompoundIdentifier(p))=>{aliases.insert(name(p.last().unwrap()));},_=>unreachable!()}}
 let mut derived_alias="__umf_related_source".to_string();while derived_alias==target_alias {derived_alias.push('_');}
 let mut equalities=vec![];
 for (i,pair) in relation.pairs.iter().enumerate(){
  let from=&pair[if forward{0}else{1}];let to=&pair[if forward{1}else{0}];
  let source_column=source_table.columns.iter().find(|c|c.logical==*from).ok_or("related source Key component unbound")?;let target_column=target_table.columns.iter().find(|c|c.logical==*to).ok_or("related target component unbound")?;
  if source_column.carrier!=target_column.carrier{return Err("related component carrier differs")}
  let mut hidden=format!("__umf_related_source_{i}");while !aliases.insert(hidden.clone()){hidden.push('_');if hidden.len()>63{return Err("related hidden alias budget")}}
  selection.projection.push(SelectItem::ExprWithAlias{expr:Expr::CompoundIdentifier(vec![Ident::with_quote('"',&source_alias),Ident::with_quote('"',from)]),alias:Ident::with_quote('"',&hidden)});
  equalities.push(format!("{}.{} = {}.{}{}",ident(&derived_alias)?,ident(&hidden)?,ident(&target_alias)?,ident(&target_column.native)?,if target_column.carrier==Carrier::String{" COLLATE pg_catalog.\"C\""}else{""}));
 }
 if equalities.is_empty(){return Err("related complete relation key required")}
 let mut source_request=request.source.clone();source_request["sql"]=json!(query.to_string());let source_fragment=weft_core::raw_read::admit_json(&source_request.to_string())?;
 let source_program=crate::raw_read::lower_related_fragment(&source_fragment,false,&prefix,None)?;
 let exists=format!("EXISTS(SELECT 1 FROM ({}) AS {} WHERE {})",source_program["sql"].as_str().unwrap(),ident(&derived_alias)?,equalities.join(" AND "));
 let mut result=crate::raw_read::lower_related_fragment(&target,true,&[],Some(&exists))?;
 result["parameterOrder"]=source_program["parameterOrder"].clone();
 let used=result["parameterOrder"].as_array().unwrap().iter().filter_map(Json::as_str).collect::<BTreeSet<_>>();
 for parameter in target.request().parameters.iter(){if !used.contains(parameter.name.as_str()){return Err("unused related global parameter")}}
 for key in ["lockSql","referencedTables"]{merge_array(&mut result,&source_program,&format!("/{key}"));}
 for (kind,array) in [("nativeProjection","columns"),("nativeRelationshipKeys","keys"),("nativeRelationships","relationships"),("termFilter","indexes")]{
  if source_program["obligations"].get(kind).is_some(){if result["obligations"].get(kind).is_none(){result["obligations"][kind]=source_program["obligations"][kind].clone()}else{merge_array(&mut result,&source_program,&format!("/obligations/{kind}/{array}"));}}
 }
 let fk=crate::raw_read::relationship_metadata(&source,relation)?;
 if result["obligations"].get("nativeRelationships").is_none(){result["obligations"]["nativeRelationships"]=json!({"version":"umf.native-relationships/0.1","nativeCatalogBeforeAfter":true,"relationships":[]});}
 if fk["completeNativeSnapshotRequired"]==true{result["obligations"]["readIsolation"]=json!("repeatable-read")}
 let fks=result["obligations"]["nativeRelationships"]["relationships"].as_array_mut().unwrap();if !fks.contains(&fk){fks.push(fk)}
 if source_program["obligations"]["readIsolation"]=="repeatable-read"{result["obligations"]["readIsolation"]=json!("repeatable-read")}
 let original_target=source.request().binding.tables.iter().find(|t|t.logical==relation.target).ok_or("related declared target binding absent")?;let fields=relation.pairs.iter().map(|p|p[1].clone()).collect::<Vec<_>>();
 if fields!=original_target.key{let key=crate::raw_read::relationship_key_metadata(original_target,&fields,&target.request().binding.schema)?;
  if result["obligations"].get("nativeRelationshipKeys").is_none(){result["obligations"]["nativeRelationshipKeys"]=json!({"version":"umf.native-relationship-keys/0.1","nativeCatalogBeforeAfter":true,"keys":[]});}
  let keys=result["obligations"]["nativeRelationshipKeys"]["keys"].as_array_mut().unwrap();if !keys.contains(&key){keys.push(key)}
 }
 for key in ["tables","functions","operators","indexObjects","semanticFields"]{merge_array(&mut result,&source_program,&format!("/privilegeInventory/{key}"));}
 result["relatedRequest"]=original;result["rootLogicalTable"]=json!(target_table.logical);result["relatedSourceSql"]=json!(source_program["sql"]);result["sourceSql"]=request.target["sql"].clone();
 crate::native_catalog_fence::attach(&target.request().binding,&mut result["obligations"])?;crate::native_catalog_fence::seal_version(&mut result)?;
 result.as_object_mut().unwrap().remove("structuralDigest");result["structuralDigest"]=json!(weft_core::json::sha256(result.to_string().as_bytes()));Ok(result)
}
pub fn compile_json(raw:&str)->String{lower(raw).unwrap_or_else(|reason|json!({"version":"weft.raw-read/0.1","status":"blocked","diagnostics":[{"code":"WFT-RELATED-READ","message":reason}]})).to_string()}
#[cfg(test)]mod tests{
 use super::*;
 fn request()->Json{
  let mut source:Json=serde_json::from_str(include_str!("../../../tests/raw-postgresql/read-request.json")).unwrap();source["parameters"]=json!([]);source["sql"]=json!("SELECT o.id AS id FROM commerce.\"Order\" o WHERE o.status = 'open' ORDER BY o.id DESC");
  let mut target=source.clone();target["sql"]=json!("SELECT c.id AS id, c.name AS name FROM commerce.\"Customer\" c ORDER BY c.id LIMIT 10");
  json!({"version":"weft.related-read/0.1","source":source,"target":target,"relationship":{"module":"commerce","id":"customer","direction":"forward"}})
 }
 fn compile(r:&Json)->Json{serde_json::from_str(&compile_json(&r.to_string())).unwrap()}
 #[test]fn source_limit_direction_and_original_global_parameters_are_retained(){
  let mut r=request();let p=compile(&r);assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["relatedRequest"],r);assert_eq!(p["sourceSql"],r["target"]["sql"]);assert!(!p["relatedSourceSql"].as_str().unwrap().contains("LIMIT"));assert!(p["sql"].as_str().unwrap().contains("EXISTS(SELECT 1 FROM"));
  r["source"]["sql"]=json!("SELECT o.id AS id FROM commerce.\"Order\" o WHERE o.status=:status ORDER BY o.id DESC LIMIT 2");r["target"]["sql"]=json!("SELECT c.id AS id FROM commerce.\"Customer\" c WHERE c.id>:id ORDER BY c.id LIMIT 10");let params=json!([{"name":"status","carrier":"string","nullable":false},{"name":"id","carrier":"integer","nullable":false}]);r["source"]["parameters"]=params.clone();r["target"]["parameters"]=params;
  let p=compile(&r);assert_eq!(p["status"],"compiled","{p}");assert!(p["relatedSourceSql"].as_str().unwrap().ends_with("LIMIT 2"));assert_eq!(p["parameterOrder"],json!(["id","status"]));assert!(p["sql"].as_str().unwrap().contains("$2::pg_catalog.text"));
  let source=r["source"].clone();r["source"]=r["target"].clone();r["target"]=source;r["relationship"]["direction"]=json!("inverse");let p=compile(&r);assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["rootLogicalTable"],json!(["commerce","Order"]));
 }
 #[test]fn original_root_record_wildcard_is_expanded_by_owner_without_source_cap(){
  let mut r=request();r["source"]["sql"]=json!("SELECT o.* FROM commerce.\"Order\" o WHERE o.status='open' ORDER BY o.id DESC");r["target"]["sql"]=json!("SELECT c.* FROM commerce.\"Customer\" c LIMIT 10");let p=compile(&r);assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["relatedRequest"],r);assert!(!p["relatedSourceSql"].as_str().unwrap().contains("LIMIT"));assert!(p["relatedSourceSql"].as_str().unwrap().contains("\"o\".\"status\" COLLATE pg_catalog.\"C\" AS \"status\""));
  for sql in ["SELECT * FROM commerce.\"Order\" o","SELECT wrong.* FROM commerce.\"Order\" o","SELECT o.* EXCEPT(status) FROM commerce.\"Order\" o"]{r["source"]["sql"]=json!(sql);assert_eq!(compile(&r)["status"],"blocked");}
 }
 #[test]fn generated_scope_never_shadows_original_target_alias_and_literals_are_not_rewritten(){
  let mut r=request();r["source"]["sql"]=json!("SELECT o.id AS __umf_related_source_0 FROM commerce.\"Order\" o WHERE o.status='$1 literal LIMIT 1000' ORDER BY o.id DESC LIMIT 2");r["target"]["sql"]=json!("SELECT \"__umf_related_source\".id AS id FROM commerce.\"Customer\" AS \"__umf_related_source\" LIMIT 10");
  let p=compile(&r);assert_eq!(p["status"],"compiled","{p}");let sql=p["sql"].as_str().unwrap();assert!(sql.contains("E'$1 literal LIMIT 1000'"));assert!(sql.contains("AS \"__umf_related_source_\" WHERE \"__umf_related_source_\".\"__umf_related_source_0_\" = \"__umf_related_source\".\"id\""),"{sql}");
 }
 #[test]fn wrong_original_relation_grouped_source_and_unused_global_parameters_refuse(){
  let r=request();for module in ["wrong",""]{let mut q=r.clone();q["relationship"]["module"]=json!(module);assert_eq!(compile(&q)["status"],"blocked")}
  let mut q=r.clone();q["source"]["sql"]=json!("SELECT COUNT(*) AS n FROM commerce.\"Order\" o");assert_eq!(compile(&q)["status"],"blocked");
  let mut q=r.clone();let params=json!([{"name":"unused","carrier":"string","nullable":false}]);q["source"]["parameters"]=params.clone();q["target"]["parameters"]=params;assert_eq!(compile(&q)["status"],"blocked");
  let mut q=r.clone();q["target"]["binding"]["schema"]=json!("other");assert_eq!(compile(&q)["status"],"blocked");
 }
}
