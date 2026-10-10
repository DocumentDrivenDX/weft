//! Qualified association namespaces for the graph source/IR migration.
//! Reference parsing only: no ontology closure, fact authority or admitted plan.
use crate::{error::{Diagnostic,Result},security_ontology::SecurityRef};
use serde_json::Value;
#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord,serde::Serialize,serde::Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct SecurityRelationshipRef{pub document_id:String,pub module_id:String,pub relationship_id:String}
/// JSON keeps original qualified Record reference shape; graph references use
/// relationshipId. The Rust variant preserves the distinct nominal namespace.
#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord,serde::Serialize)]
#[serde(untagged)]
pub enum SecurityAssociationRef{Record(SecurityRef),Relationship(SecurityRelationshipRef)}
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-ASSOCIATION-REF","model","Qualified security association reference refused")}
impl SecurityAssociationRef{
 /// Record-only consumers must explicitly refuse the Relationship variant.
 pub fn record(&self)->Option<&SecurityRef>{match self{Self::Record(r)=>Some(r),Self::Relationship(_)=>None}}
 pub fn read(value:&Value)->Result<Self>{
  if !value.is_object(){return Err(fail());}
  let result=if value.get("relationshipId").is_some(){
   Self::Relationship(serde_json::from_value(value.clone()).map_err(|_|fail())?)
  }else{Self::Record(serde_json::from_value(value.clone()).map_err(|_|fail())?)};
  let (document,module,identifier)=match &result{
   Self::Record(r)=>(&r.document_id,&r.module_id,&r.element_id),
   Self::Relationship(r)=>(&r.document_id,&r.module_id,&r.relationship_id),
  };
  if document.is_empty()||module.is_empty()||identifier.is_empty(){return Err(fail());}
  Ok(result)
 }
}
#[cfg(test)]
mod tests{
 use super::*;
 use serde_json::json;
 use std::collections::BTreeSet;
 #[test]
 fn reference_shapes_roundtrip_without_record_graph_namespace_erasure(){
  let record=json!({"documentId":"d","moduleId":"m","elementId":"same"});
  let graph=json!({"documentId":"d","moduleId":"m","relationshipId":"same"});
  let r=SecurityAssociationRef::read(&record).unwrap();let g=SecurityAssociationRef::read(&graph).unwrap();
  assert_ne!(r,g);assert_eq!(serde_json::to_value(&r).unwrap(),record);assert_eq!(serde_json::to_value(&g).unwrap(),graph);
  assert!(matches!(r,SecurityAssociationRef::Record(_)));assert!(matches!(g,SecurityAssociationRef::Relationship(_)));
 }
 #[test]
 fn malformed_or_ambiguous_selectors_refuse(){
  for value in [Value::Null,json!([]),json!(["d","m","r"]),json!(["d","m",1]),json!([["d"],"m","r"]),json!({}),json!({"documentId":"d","moduleId":"m"}),json!({"documentId":"d","moduleId":"m","elementId":"r","relationshipId":"r"}),json!({"documentId":"d","moduleId":"m","relationshipId":1}),json!({"documentId":"d","moduleId":"m","relationshipId":"r","future":true})]{
   assert_eq!(SecurityAssociationRef::read(&value).unwrap_err().code,"WFT-SECURITY-ASSOCIATION-REF");
  }
  for identifier in ["elementId","relationshipId"]{for field in ["documentId","moduleId",identifier]{let mut value=json!({"documentId":"d","moduleId":"m"});value[identifier]=json!("r");value[field]=json!("");assert!(SecurityAssociationRef::read(&value).is_err());}}
 }
 #[test]
 fn all_qualified_components_and_unicode_spelling_remain_distinct(){
  let mut refs=BTreeSet::new();
  for document in ["d","other"]{for module in ["m","other"]{for spelling in ["é","e\u{301}"]{for field in ["elementId","relationshipId"]{
   let mut value=json!({"documentId":document,"moduleId":module});value[field]=json!(spelling);assert!(refs.insert(SecurityAssociationRef::read(&value).unwrap()));
  }}}}assert_eq!(refs.len(),16);
 }
}
