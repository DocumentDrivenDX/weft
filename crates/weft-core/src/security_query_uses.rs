//! Extract actual field/operator uses from the owner-resolved SQL tree.
//! This is compiler lineage, not physical lowering or native authorization.
use crate::{error::{Diagnostic,Result},model::{Catalog,ModuleInput},ir::Identity,application_ir as app,security_ir::{SecurityLogicalPlan,QueryOperator},security_ontology::{SecurityRef,SecurityOntologyClosure}};
use std::collections::{BTreeMap,BTreeSet};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-QUERY-USES","resolve","Security query-use extraction refused")}
fn unsupported()->Diagnostic{Diagnostic::new("WFT-SECURITY-QUERY-UNSUPPORTED","capability","Relationship query-use lowering is not admitted")}
#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub struct QueryUse{pub scan:String,pub target:SecurityRef,pub field:SecurityRef,pub operator:QueryOperator}
#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub struct Projection{pub scan:String,pub target:SecurityRef,pub field:SecurityRef}
#[derive(Debug)]
pub struct SecurityResolvedQuery{sql:String,query:app::Plan,model_inputs:Vec<ModuleInput>,policy_json:String,ontology_json:String,scans:BTreeMap<String,SecurityRef>,uses:Vec<QueryUse>,projections:Vec<Projection>}
impl SecurityResolvedQuery{
 pub fn sql(&self)->&str{&self.sql}
 pub fn application_plan(&self)->&app::Plan{&self.query}
 pub fn scans(&self)->&BTreeMap<String,SecurityRef>{&self.scans}
 pub fn uses(&self)->&[QueryUse]{&self.uses}
 pub fn projections(&self)->&[Projection]{&self.projections}
 pub fn require_sources(&self,security:&SecurityLogicalPlan,catalog:&Catalog)->Result<()>{security.require_catalog(catalog)?;if self.model_inputs!=catalog.inputs()||self.policy_json!=security.source().policy_json()||self.ontology_json!=security.source().ontology_json(){return Err(fail());}Ok(())}
 pub fn resolve(sql:&str,catalog:&Catalog,security:&SecurityLogicalPlan,parameters:crate::application_resolve::Parameters,profile:Option<app::ReadProfile>)->Result<Self>{
  security.require_catalog(catalog)?;if sql.len()>4_000_000{return Err(fail());}
  let query=crate::application_resolve::resolve(catalog,crate::application_syntax::parse(sql)?,parameters,profile)?;
  let closure=SecurityOntologyClosure::read(security.source(),catalog)?;
  fn reference(id:&Identity,catalog:&Catalog)->Result<SecurityRef>{if !catalog.inputs().iter().any(|m|m.pin.document_id==id.document_id&&m.pin.revision==id.revision){return Err(fail());}Ok(SecurityRef{document_id:id.document_id.clone(),module_id:id.module.clone(),element_id:id.element.clone()})}
  let mut scans=BTreeMap::new();for scan in std::iter::once(&query.source).chain(query.joins.iter().map(|j|&j.right)){
   let target=reference(&scan.record,catalog)?;if !closure.types.contains_key(&target)||scans.insert(scan.occurrence.clone(),target).is_some(){return Err(fail());}
  }
  fn field(scan:&str,id:&Identity,scans:&BTreeMap<String,SecurityRef>,closure:&SecurityOntologyClosure,catalog:&Catalog)->Result<Projection>{
   let target=scans.get(scan).ok_or_else(fail)?;let field=reference(id,catalog)?;if !closure.types[target].fields.contains(&field){return Err(fail());}Ok(Projection{scan:scan.into(),target:target.clone(),field})
  }
  fn add(f:&app::Field,operator:QueryOperator,uses:&mut BTreeSet<QueryUse>,scans:&BTreeMap<String,SecurityRef>,closure:&SecurityOntologyClosure,catalog:&Catalog)->Result<()>{let p=field(&f.scan,&f.identity,scans,closure,catalog)?;uses.insert(QueryUse{scan:p.scan,target:p.target,field:p.field,operator});if uses.len()>4096{return Err(fail());}Ok(())}
  fn value(v:&app::Value,operator:QueryOperator,uses:&mut BTreeSet<QueryUse>,scans:&BTreeMap<String,SecurityRef>,closure:&SecurityOntologyClosure,catalog:&Catalog)->Result<()>{if let app::Value::Field{field}=v{add(field,operator,uses,scans,closure,catalog)?;}Ok(())}
  fn predicate(p:&app::Predicate,operator:QueryOperator,uses:&mut BTreeSet<QueryUse>,scans:&BTreeMap<String,SecurityRef>,closure:&SecurityOntologyClosure,catalog:&Catalog)->Result<()>{match p{
   app::Predicate::Equal{left,right}=>{add(left,operator,uses,scans,closure,catalog)?;value(right,operator,uses,scans,closure,catalog)?;},
   app::Predicate::LexicographicGreater{columns,values}=>{for f in columns{add(f,operator,uses,scans,closure,catalog)?;}for v in values{value(v,operator,uses,scans,closure,catalog)?;}},
   app::Predicate::HasRelated{..}=>return Err(unsupported())
  }Ok(())}
  let mut uses=BTreeSet::new();let mut projections=BTreeSet::new();
  for join in &query.joins{for p in &join.on{predicate(p,QueryOperator::Join,&mut uses,&scans,&closure,catalog)?;}}
  for p in &query.filters{predicate(p,QueryOperator::Predicate,&mut uses,&scans,&closure,catalog)?;}
  for f in &query.groups{add(f,QueryOperator::Group,&mut uses,&scans,&closure,catalog)?;}
  for f in &query.order{add(f,QueryOperator::Order,&mut uses,&scans,&closure,catalog)?;}
  for output in &query.outputs{match &output.expression{
   app::Expression::Field{scan,identity}=>{projections.insert(field(scan,identity,&scans,&closure,catalog)?);},
   app::Expression::Sum{argument,..}=>add(argument,QueryOperator::Aggregate,&mut uses,&scans,&closure,catalog)?,
   app::Expression::Count{..}=>{}, // Field-free counts still retain every eligible scan.
   app::Expression::RelatedKeys{..}=>return Err(unsupported())
  }}
  if projections.len()>4096||scans.len()>256{return Err(fail());}
  Ok(Self{sql:sql.into(),query,model_inputs:catalog.inputs().to_vec(),policy_json:security.source().policy_json().into(),ontology_json:security.source().ontology_json().into(),scans,uses:uses.into_iter().collect(),projections:projections.into_iter().collect()})
 }
}

#[cfg(test)]
mod tests{
 use super::*;
 use serde_json::{json,Value};
 fn fixture()->(Catalog,SecurityLogicalPlan){
  let f:Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();let mut doc=f["resolution"]["documents"][0]["document"].clone();for e in doc["modules"][0]["elements"].as_array_mut().unwrap(){e["name"]=e["id"].clone();if e["scalarType"]=="integer"{e["facets"]=json!({"integerWidth":{"bits":64,"signed":true}});}}let text=doc.to_string();
  let inputs=serde_json::from_value(json!([{"documentJson":text,"pin":{"documentId":"domain","revision":"schema-1","umfVersion":"0.8.0","sha256":crate::json::sha256(text.as_bytes())},"selectedModuleIds":["m"]}])).unwrap();let catalog=Catalog::prepare_security(inputs).unwrap();let packet=crate::security_source::SecuritySourcePacket::read(&f["policy"].to_string(),&f["resolution"]["ontology"].to_string(),&catalog).unwrap();let security=SecurityLogicalPlan::read(packet,&catalog).unwrap();(catalog,security)
 }
 #[test]
 fn extraction_covers_every_operator_and_both_join_occurrences(){
  let (catalog,security)=fixture();let sql="SELECT r.salary AS amount,SUM(r.salary) AS total FROM Resource r JOIN Resource s ON r.salary=s.salary WHERE r.salary=100 GROUP BY r.salary ORDER BY r.salary";
  let query=SecurityResolvedQuery::resolve(sql,&catalog,&security,BTreeMap::new(),None).unwrap();assert_eq!(query.sql(),sql);assert_eq!(query.scans().len(),2);assert_eq!(query.uses().len(),6);assert_eq!(query.projections().len(),1);
  let ops=query.uses().iter().filter(|u|u.scan=="s0").map(|u|u.operator).collect::<BTreeSet<_>>();assert_eq!(ops,BTreeSet::from([QueryOperator::Predicate,QueryOperator::Order,QueryOperator::Group,QueryOperator::Join,QueryOperator::Aggregate]));
  assert!(query.uses().iter().any(|u|u.scan=="s1"&&u.operator==QueryOperator::Join));assert!(query.uses().iter().all(|u|u.field.element_id=="salary"&&u.target.element_id=="Resource"));
 }
 #[test]
 fn field_free_counts_preserve_scan_and_keyset_fields_are_predicate_uses(){
  let (catalog,security)=fixture();let count=SecurityResolvedQuery::resolve("SELECT COUNT(*) FROM Resource r",&catalog,&security,BTreeMap::new(),None).unwrap();assert!(count.uses().is_empty());assert!(count.projections().is_empty());assert_eq!(count.scans().len(),1);
  let page=SecurityResolvedQuery::resolve("SELECT r.resourceId FROM Resource r WHERE (r.resourceId)>('r1') ORDER BY r.resourceId LIMIT 1",&catalog,&security,BTreeMap::new(),None).unwrap();assert_eq!(page.uses().iter().map(|u|u.operator).collect::<BTreeSet<_>>(),BTreeSet::from([QueryOperator::Predicate,QueryOperator::Order]));
 }
 #[test]
 fn ordered_alias_outputs_survive_dependency_deduplication_and_stale_models_refuse(){
  let (catalog,security)=fixture();let query=SecurityResolvedQuery::resolve("SELECT r.salary AS first_value,r.salary AS second_value FROM Resource r",&catalog,&security,BTreeMap::new(),None).unwrap();assert_eq!(query.projections().len(),1);assert_eq!(query.application_plan().outputs.len(),2);assert_eq!(query.application_plan().outputs[0].name,"first_value");assert_eq!(query.application_plan().outputs[1].name,"second_value");
  let mut changed_inputs=catalog.inputs().to_vec();changed_inputs[0].selected_module_ids.clear();assert!(Catalog::prepare_security(changed_inputs).is_err());let mut changed_inputs=catalog.inputs().to_vec();changed_inputs[0].pin.revision.push_str("-changed");let changed=Catalog::prepare_security(changed_inputs).unwrap();assert!(query.require_sources(&security,&changed).is_err());
 }
}
