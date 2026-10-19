// SPDX-License-Identifier: MIT OR Apache-2.0
//! Explicit reified graph correspondence. Source checks are not native authority.
use std::collections::BTreeMap;
use serde::{Serialize,Deserialize};
use serde_json::{Value,json};
use weft_core::{error::{Diagnostic,Result},model::{Catalog,ModuleInput},security_ir::SecurityLogicalPlan,security_ontology::{SecurityRef,authored_key_fields,check_mapped_scalar_fields},json::{sha256,checked_json}};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-GRAPH-BINDING","model","Original typed graph correspondence required")}
fn source_json(s:&str)->Result<Value>{if s.len()>4_000_000{return Err(fail());}checked_json(s).map_err(|_|fail())}
fn object(v:&Value,keys:&[&str])->Result<()>{let o=v.as_object().ok_or_else(fail)?;if o.len()!=keys.len()||keys.iter().any(|k|!o.contains_key(*k)){return Err(fail());}Ok(())}
fn id(s:&str)->Result<i64>{if s.is_empty()||s.len()>10||s.starts_with('0')||!s.bytes().all(|b|b.is_ascii_digit()){return Err(fail());}let n=s.parse::<i64>().map_err(|_|fail())?;if n>i32::MAX as i64{return Err(fail());}Ok(n)}
fn reference(v:&Value)->Result<SecurityRef>{serde_json::from_value(v.clone()).map_err(|_|fail())}
fn rows<'a>(v:&'a Value,k:&str)->Result<&'a [Value]>{let a=v[k].as_array().ok_or_else(fail)?;if a.len()>4096{return Err(fail());}Ok(a)}
fn one<'a>(a:&'a [Value],p:impl Fn(&Value)->bool)->Result<&'a Value>{let mut found=None;for v in a{if p(v){if found.is_some(){return Err(fail());}found=Some(v);}}found.ok_or_else(fail)}
fn element<'a>(catalog:&'a Catalog,r:&SecurityRef)->Result<Value>{let input=catalog.inputs().iter().find(|i|i.pin.document_id==r.document_id).ok_or_else(fail)?;let doc=source_json(&input.document_json)?;let module=one(rows(&doc,"modules")?,|m|m["id"]==r.module_id)?;Ok(one(rows(module,"elements")?,|e|e["id"]==r.element_id)?.clone())}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct GraphSource {
 pub version:String,
 pub native_schema_revision:String,
 pub physical_module:ModuleInput,
 pub inventory_json:String,
 pub correspondence_json:String,
 pub logical_resolution_json:String,
 pub logical_transport_json:String,
}
#[derive(Clone)]
struct Entity {native:SecurityRef,tid:i64,key_id:String,key_field:SecurityRef,prop:i64}
#[derive(Clone)]
enum Home {Property{i:i64},Endpoint{relation:i64,incoming:bool,source:i64,target:i64,key_prop:i64}}
#[derive(Clone)]
pub(crate) struct QueryRelationship {pub(crate) relation:i64,pub(crate) source:i64,pub(crate) target:i64}
pub struct GraphBinding {
 query_relationships:BTreeMap<(String,String,String),QueryRelationship>,
 entities:BTreeMap<SecurityRef,Entity>,homes:BTreeMap<(SecurityRef,SecurityRef),Home>,
 logical_keys:BTreeMap<(SecurityRef,String),Vec<SecurityRef>>,pub(crate) inventory:Value,
 source_sha256:String,guards:Vec<Value>,
}
impl GraphBinding {
 pub(crate) fn read_context(ctx:&weft_core::security_backend::SecurityBackendContext<'_>,source:&GraphSource)->Result<Self>{
  if ctx.query().relationship_filters().is_empty(){Self::read(ctx.logical_plan(),ctx.catalog(),source)}else{Self::read_relationship_candidate(ctx.logical_plan(),ctx.catalog(),source)}
 }
 pub(crate) fn query_relationship(&self,identity:&weft_core::application_model::RelationshipIdentity)->Result<&QueryRelationship>{self.query_relationships.get(&(identity.document_id.clone(),identity.module.clone(),identity.relationship.clone())).ok_or_else(fail)}
 pub fn source_sha256(&self)->&str{&self.source_sha256}
 pub fn integrity_checks(&self)->&[Value]{&self.guards}
 pub(crate) fn type_id(&self,r:&SecurityRef)->Result<i64>{Ok(self.entities.get(r).ok_or_else(fail)?.tid)}
 pub(crate) fn key(&self,r:&SecurityRef,key:&str)->Result<Vec<SecurityRef>>{self.logical_keys.get(&(r.clone(),key.into())).cloned().ok_or_else(fail)}
 pub(crate) fn raw_cell(&self,r:&SecurityRef,f:&SecurityRef,alias:&str)->Result<String>{
  if !matches!(alias,"r"|"s")&&!alias.strip_prefix('v').is_some_and(|n|n.len()<=6&&!n.is_empty()&&n.bytes().all(|b|b.is_ascii_digit())){return Err(fail());}
  match self.homes.get(&(r.clone(),f.clone())).ok_or_else(fail)? {
   Home::Property{i}=>Ok(format!("{alias}.props->'{i}'")),
   Home::Endpoint{relation,incoming,source,target,key_prop}=>{
    let (own,other,own_type,other_type)=if *incoming{("target","source",target,source)}else{("source","target",source,target)};
    Ok(format!("(SELECT CASE WHEN pg_catalog.count(*)=1 AND pg_catalog.bool_and(e.{own}_type={own_type} AND e.{other}_type={other_type} AND t.type_id={other_type} AND truss.valid_scalar_cell(t.props->'{key_prop}','string',FALSE) IS TRUE) THEN pg_catalog.jsonb_agg(t.props->'{key_prop}')->0 ELSE NULL::pg_catalog.jsonb END FROM truss.edge e LEFT JOIN truss.object t ON t.id=e.{other}_id AND t.type_id=e.{other}_type WHERE e.rel_type_id={relation} AND e.{own}_id={alias}.id)"))
   }
  }
 }
 pub fn read(plan:&SecurityLogicalPlan,logical:&Catalog,source:&GraphSource)->Result<Self>{Self::read_selected(plan,logical,source,false)}
 /// Original relationship correspondence admission; not a native authority or query permit.
 pub fn read_relationship_candidate(plan:&SecurityLogicalPlan,logical:&Catalog,source:&GraphSource)->Result<Self>{Self::read_selected(plan,logical,source,true)}
 fn read_selected(plan:&SecurityLogicalPlan,logical:&Catalog,source:&GraphSource,relationships:bool)->Result<Self>{
  plan.require_catalog(logical)?;
  if source.version!=if relationships{"weft.truss.security.graph-source/0.2-candidate"}else{"weft.truss.security.graph-source/0.1-candidate"}{return Err(fail());}
  let total=[source.inventory_json.len(),source.correspondence_json.len(),source.logical_resolution_json.len(),source.logical_transport_json.len(),source.physical_module.document_json.len()].into_iter().try_fold(0usize,|a,b|a.checked_add(b)).ok_or_else(fail)?;
  if total>16_000_000||logical.inputs().len()!=1{return Err(fail());}
  let parse=source_json;
  let c=parse(&source.correspondence_json)?;
  let mut keys=vec!["version","scope","logicalSourceDigests","physicalSourceSha256","inventorySha256","entities","endpoints","active","salary","identityMeaning"];if relationships{keys.push("queryRelationships");}object(&c,&keys)?;
  if c["version"]!=if relationships{"truss.reified-security-binding-fixture/0.2-candidate"}else{"truss.reified-security-binding-fixture/0.1-candidate"}||c["physicalSourceSha256"]!=sha256(source.physical_module.document_json.as_bytes())||c["inventorySha256"]!=sha256(source.inventory_json.as_bytes()){return Err(fail());}
  object(&c["logicalSourceDigests"],&["document","resolution","policy","transport"])?;
  for (name,bytes) in [("document",logical.inputs()[0].document_json.as_bytes()),("resolution",source.logical_resolution_json.as_bytes()),("policy",plan.source().policy_json().as_bytes()),("transport",source.logical_transport_json.as_bytes())]{if c["logicalSourceDigests"][name]!=sha256(bytes){return Err(fail());}}
  let resolution=parse(&source.logical_resolution_json)?;
  if resolution["ontology"]!=*plan.source().ontology()||rows(&resolution,"documents")?.len()!=1||resolution["documents"][0]["document"]!=parse(&logical.inputs()[0].document_json)?||resolution["documents"][0]["revision"]!=logical.inputs()[0].pin.revision{return Err(fail());}
  let physical=Catalog::prepare_security(vec![source.physical_module.clone()])?;
  let revision=source.native_schema_revision.parse::<i64>().map_err(|_|fail())?;
  if revision<=0||revision.to_string()!=source.native_schema_revision{return Err(fail());}
  let native_doc=parse(&source.physical_module.document_json)?;
  let wrapper=parse(&source.inventory_json)?;let inventory=wrapper["inventory"].clone();
  for key in ["types","properties","keys","relationships","endpoints"]{rows(&inventory,key)?;}
  let mut out=Self{query_relationships:BTreeMap::new(),entities:BTreeMap::new(),homes:BTreeMap::new(),logical_keys:BTreeMap::new(),inventory,source_sha256:sha256(serde_json::to_string(source).map_err(|_|fail())?.as_bytes()),guards:Vec::new()};
  let entities=rows(&c,"entities")?;if entities.is_empty()||entities.len()>32{return Err(fail());}
  for entry in entities {
   object(entry,&["logicalType","nativeType","logicalKeyId","nativeKeyId","nativeKeyNumber","logicalKeyField","nativeKeyField","nativePropertyId","scalarFamily"])?;
   object(&entry["nativeType"],&["documentId","moduleId","elementId","typeId"])?;
   let l=reference(&entry["logicalType"])?;
   let n=SecurityRef{document_id:entry["nativeType"]["documentId"].as_str().ok_or_else(fail)?.into(),module_id:entry["nativeType"]["moduleId"].as_str().ok_or_else(fail)?.into(),element_id:entry["nativeType"]["elementId"].as_str().ok_or_else(fail)?.into()};
   let tid=id(entry["nativeType"]["typeId"].as_str().ok_or_else(fail)?)?;let prop=id(entry["nativePropertyId"].as_str().ok_or_else(fail)?)?;
   let logical_id=entry["logicalKeyId"].as_str().ok_or_else(fail)?;let native_id=entry["nativeKeyId"].as_str().ok_or_else(fail)?;
   let keys=authored_key_fields(logical,&l,logical_id)?;let native_keys=authored_key_fields(&physical,&n,native_id)?;
   if keys.len()!=1||native_keys.len()!=1||keys[0].element_id!=entry["logicalKeyField"]||native_keys[0].element_id!=entry["nativeKeyField"]||entry["scalarFamily"]!="string"{return Err(fail());}
   check_mapped_scalar_fields(logical,&keys[0],&physical,&native_keys[0])?;
   if element(&physical,&native_keys[0])?["scalarType"]!="string"{return Err(fail());}
   one(rows(&out.inventory,"types")?,|v|v.get("retired_rev").is_some_and(Value::is_null)&&v["definition_rev"]==revision&&v["definition_source_kind"]=="accepted_document"&&v["type_id"]==tid&&v["document_id"]==n.document_id&&v["module"]==n.module_id&&v["element"]==n.element_id&&v["kind"]=="record")?;
   let p=one(rows(&out.inventory,"properties")?,|v|v.get("retired_rev").is_some_and(Value::is_null)&&v["definition_rev"]==revision&&v["definition_source_kind"]=="accepted_document"&&v["type_id"]==tid&&v["prop_id"]==prop)?;
   if p["definition_document_id"]!=n.document_id||p["declaration_module"]!=native_keys[0].module_id||p["element"]!=native_keys[0].element_id||p["home"]!="json"||p["scalar_type"]!="string"||p["cardinality"]!="one"||p["nullability"]!="required"{return Err(fail());}
   let key_number=id(entry["nativeKeyNumber"].as_str().ok_or_else(fail)?)?;
   one(rows(&out.inventory,"keys")?,|v|v.get("retired_rev").is_some_and(Value::is_null)&&v["definition_rev"]==revision&&v["definition_source_kind"]=="accepted_document"&&v["type_id"]==tid&&v["key_id"]==native_id&&v["key_num"]==key_number&&v["prop_ids"]==json!([prop]))?;
   if out.entities.values().any(|e|e.tid==tid)||out.entities.insert(l.clone(),Entity{native:n,tid,key_id:native_id.into(),key_field:native_keys[0].clone(),prop}).is_some(){return Err(fail());}
   out.logical_keys.insert((l.clone(),logical_id.into()),keys.clone());out.homes.insert((l,keys[0].clone()),Home::Property{i:prop});
  }
  let endpoints=rows(&c,"endpoints")?;if endpoints.len()>64{return Err(fail());}
  let ontology=plan.source().ontology();let mut expected=0;
  for association in rows(ontology,"associations")?{expected+=rows(association,"endpoints")?.len();}
  if endpoints.len()!=expected{return Err(fail());}
  for e in endpoints {
   object(e,&["logicalAssociation","role","logicalField","nativeRelationship","nativeRelationshipId","direction","sourceTypeId","targetTypeId","targetKeyType","targetKeyId","runtimeCardinality","schemaMultiplicityDoesNotProveRuntimeCardinality"])?;
   if e["runtimeCardinality"]!="exactly_one_for_every_association_node"||e["schemaMultiplicityDoesNotProveRuntimeCardinality"]!=true{return Err(fail());}
   let a=one(rows(ontology,"associations")?,|v|v["type"]["elementId"]==e["logicalAssociation"])?;let ar=reference(&a["type"])?;let owner=out.entities.get(&ar).ok_or_else(fail)?.clone();
   let ep=one(rows(a,"endpoints")?,|v|v["role"]==e["role"])?;let target=reference(&ep["target"])?;let target_native=out.entities.get(&target).ok_or_else(fail)?.clone();
   let fields=rows(ep,"fields")?;if fields.len()!=1||fields[0]["elementId"]!=e["logicalField"]||target.element_id!=e["targetKeyType"]||target_native.key_id!=e["targetKeyId"]{return Err(fail());}let f=reference(&fields[0])?;
   check_mapped_scalar_fields(logical,&f,&physical,&target_native.key_field)?;
   let relation=id(e["nativeRelationshipId"].as_str().ok_or_else(fail)?)?;let source_type=id(e["sourceTypeId"].as_str().ok_or_else(fail)?)?;let target_type=id(e["targetTypeId"].as_str().ok_or_else(fail)?)?;
   let incoming=match e["direction"].as_str(){Some("incoming")=>true,Some("outgoing")=>false,_=>return Err(fail())};
   let (src,dst)=if incoming{(&target_native,&owner)}else{(&owner,&target_native)};if src.tid!=source_type||dst.tid!=target_type||src.native.document_id!=dst.native.document_id||src.native.module_id!=dst.native.module_id{return Err(fail());}
   let module=one(rows(&native_doc,"modules")?,|v|v["id"]==src.native.module_id)?;let def=one(rows(module,"relationships")?,|v|v["id"]==e["nativeRelationship"])?;
   if def["source"]!=json!([{"module":src.native.module_id,"element":src.native.element_id}])||def["target"]!=json!([{"module":dst.native.module_id,"element":dst.native.element_id,"key":dst.key_id}])||def["directed"]!=true||def["targetLifecycle"]!="independent"{return Err(fail());}
   one(rows(&out.inventory,"relationships")?,|v|v.get("retired_rev").is_some_and(Value::is_null)&&v["definition_rev"]==revision&&v["definition_source_kind"]=="accepted_document"&&v["rel_type_id"]==relation&&v["document_id"]==src.native.document_id&&v["module"]==src.native.module_id&&v["rel_id"]==e["nativeRelationship"]&&v["directed"]==true&&v["composition"]==false&&v["lifecycle"]=="independent"&&v["target_key"]==dst.key_id)?;
   one(rows(&out.inventory,"endpoints")?,|v|v["rel_type_id"]==relation&&v["source_type"]==source_type&&v["target_type"]==target_type)?;
   if out.homes.insert((ar.clone(),f),Home::Endpoint{relation,incoming,source:source_type,target:target_type,key_prop:target_native.prop}).is_some(){return Err(fail());}
   let (own,other)=if incoming{("target","source")}else{("source","target")};let (own_type,other_type)=(owner.tid,target_native.tid);let pid=target_native.prop;
   out.guards.push(json!({"association":ar,"role":e["role"],"failureCode":"WFT-SECURITY-GRAPH-CARDINALITY","sql":format!("SELECT pg_catalog.count(*)::pg_catalog.text AS violations FROM truss.object r WHERE r.type_id={own_type} AND (SELECT CASE WHEN pg_catalog.count(*)=1 AND pg_catalog.bool_and(e.{own}_type={own_type} AND e.{other}_type={other_type} AND t.type_id={other_type} AND truss.valid_scalar_cell(t.props->'{pid}','string',FALSE) IS TRUE) THEN FALSE ELSE TRUE END FROM truss.edge e LEFT JOIN truss.object t ON t.id=e.{other}_id AND t.type_id=e.{other}_type WHERE e.rel_type_id={relation} AND e.{own}_id=r.id)")}));
  }
  // These named scalar bindings are part of the explicit fixture carrier version.
  for (member,record_name,field_name,family) in [("active","Assignment","active","boolean"),("salary","Resource","salary","integer")] {
   let entry=&c[member];if member=="active"{object(entry,&["nativeTypeId","nativePropertyId","family"])?;}else{object(entry,&["nativeTypeId","nativePropertyId","family","mode"])?;if entry["mode"]!="withheld_only_not_a_captured_policy_operand"{return Err(fail());}}
   let (logical_ref,native)=out.entities.iter().find(|(l,_)|l.element_id==record_name).ok_or_else(fail)?;
   let tid=id(entry["nativeTypeId"].as_str().ok_or_else(fail)?)?;let prop=id(entry["nativePropertyId"].as_str().ok_or_else(fail)?)?;if tid!=native.tid||entry["family"]!=family{return Err(fail());}
   let logical_field=SecurityRef{document_id:logical_ref.document_id.clone(),module_id:logical_ref.module_id.clone(),element_id:field_name.into()};
   let p=one(rows(&out.inventory,"properties")?,|v|v.get("retired_rev").is_some_and(Value::is_null)&&v["definition_rev"]==revision&&v["definition_source_kind"]=="accepted_document"&&v["type_id"]==tid&&v["prop_id"]==prop&&v["home"]=="json"&&v["scalar_type"]==family)?;
   let physical_field=SecurityRef{document_id:p["definition_document_id"].as_str().ok_or_else(fail)?.into(),module_id:p["declaration_module"].as_str().ok_or_else(fail)?.into(),element_id:p["element"].as_str().ok_or_else(fail)?.into()};
   if physical_field.document_id!=native.native.document_id||physical_field.module_id!=native.native.module_id||physical_field.element_id!=field_name{return Err(fail());}
   let declaration=element(&physical,&physical_field)?;
   if p["cardinality"]!=declaration["cardinality"]||p["nullability"]!=declaration["nullability"]{return Err(fail());}
   check_mapped_scalar_fields(logical,&logical_field,&physical,&physical_field)?;
   out.homes.insert((logical_ref.clone(),logical_field),Home::Property{i:prop});
  }
  if relationships {
   fn resolve(catalog:&Catalog,r:&Value)->Result<weft_core::application_model::RelationshipRead>{
    object(r,&["documentId","moduleId","relationshipId"])?;
    let input=catalog.inputs().iter().find(|i|i.pin.document_id==r["documentId"]).ok_or_else(fail)?;
    let module_id=r["moduleId"].as_str().ok_or_else(fail)?;if !input.selected_module_ids.iter().any(|m|m==module_id){return Err(fail());}
    let doc=source_json(&input.document_json)?;let module=one(rows(&doc,"modules")?,|m|m["id"]==module_id)?;
    let rel=one(rows(module,"relationships")?,|v|v["id"]==r["relationshipId"])?;
    let endpoint=&rel["source"][0];let record=catalog.record_by_identity(&weft_core::ir::Identity{document_id:input.pin.document_id.clone(),revision:input.pin.revision.clone(),module:endpoint["module"].as_str().ok_or_else(fail)?.into(),element:endpoint["element"].as_str().ok_or_else(fail)?.into()})?;
    let name=weft_core::syntax::Name{value:rel["name"].as_str().ok_or_else(fail)?.into(),quoted:true,span:weft_core::ir::Span{start:0,end:0}};
    let read=catalog.relationship_read(&record,&name)?;
    if read.inverse||read.target_lifecycle!="independent"||read.identity.module!=module_id||read.identity.relationship!=r["relationshipId"]{return Err(fail());}Ok(read)
   }
   fn reference_of(i:&weft_core::ir::Identity)->SecurityRef{SecurityRef{document_id:i.document_id.clone(),module_id:i.module.clone(),element_id:i.element.clone()}}
   let entries=rows(&c,"queryRelationships")?;if entries.is_empty()||entries.len()>256{return Err(fail());}
   for entry in entries {
    object(entry,&["logicalRelationship","nativeRelationship","nativeRelationshipId","sourceTypeId","targetTypeId"])?;
    let l=resolve(logical,&entry["logicalRelationship"])?;let n=resolve(&physical,&entry["nativeRelationship"])?;
    let src=out.entities.get(&reference_of(&l.from)).ok_or_else(fail)?;let dst=out.entities.get(&reference_of(&l.to)).ok_or_else(fail)?;
    let relation=id(entry["nativeRelationshipId"].as_str().ok_or_else(fail)?)?;let source_id=id(entry["sourceTypeId"].as_str().ok_or_else(fail)?)?;let target_id=id(entry["targetTypeId"].as_str().ok_or_else(fail)?)?;
    if source_id!=src.tid||target_id!=dst.tid||reference_of(&n.from)!=src.native||reference_of(&n.to)!=dst.native||l.source_multiplicity!=n.source_multiplicity||l.target_multiplicity!=n.target_multiplicity||n.source_key.id!=src.key_id||n.target_key.id!=dst.key_id{return Err(fail());}
    for (lk,nk,owner) in [(&l.source_key,&n.source_key,&l.from),(&l.target_key,&n.target_key,&l.to)] {
     let expected=out.logical_keys.get(&(reference_of(owner),lk.id.clone())).ok_or_else(fail)?;
     if lk.fields.iter().map(reference_of).collect::<Vec<_>>()!=*expected||lk.fields.len()!=nk.fields.len(){return Err(fail());}
     for (a,b) in lk.fields.iter().zip(&nk.fields){check_mapped_scalar_fields(logical,&reference_of(a),&physical,&reference_of(b))?;}
    }
    let row=one(rows(&out.inventory,"relationships")?,|v|v.get("retired_rev").is_some_and(Value::is_null)&&v["definition_rev"]==revision&&v["definition_source_kind"]=="accepted_document"&&v["rel_type_id"]==relation&&v["document_id"]==n.identity.document_id&&v["module"]==n.identity.module&&v["rel_id"]==n.identity.relationship)?;
    if row["directed"]!=true||row["composition"]!=false||row["lifecycle"]!="independent"||row["target_key"]!=n.target_key.id||!row["assoc_type_id"].is_null(){return Err(fail());}
    for (side,m) in [("source",&n.source_multiplicity),("target",&n.target_multiplicity)] {if row[format!("{side}_min")]!=m["min"]||row[format!("{side}_max")]!=if m["max"]=="*"{Value::Null}else{m["max"].clone()}{return Err(fail());}}
    one(rows(&out.inventory,"endpoints")?,|v|v["rel_type_id"]==relation&&v["source_type"]==source_id&&v["target_type"]==target_id)?;
    let sp=src.prop;let tp=dst.prop;
    out.guards.push(json!({"failureCode":"WFT-SECURITY-PUBLIC-RELATIONSHIP-INTEGRITY","sql":format!("SELECT pg_catalog.count(*)::pg_catalog.text AS violations FROM truss.edge e LEFT JOIN truss.object src ON src.id=e.source_id AND src.type_id=e.source_type LEFT JOIN truss.object dst ON dst.id=e.target_id AND dst.type_id=e.target_type WHERE e.rel_type_id={relation} AND (e.source_type IS DISTINCT FROM {source_id} OR e.target_type IS DISTINCT FROM {target_id} OR src.id IS NULL OR dst.id IS NULL OR truss.valid_scalar_cell(src.props->'{sp}','string',FALSE) IS NOT TRUE OR truss.valid_scalar_cell(dst.props->'{tp}','string',FALSE) IS NOT TRUE)")}));
    let key=(l.identity.document_id,l.identity.module,l.identity.relationship);if out.query_relationships.insert(key,QueryRelationship{relation,source:source_id,target:target_id}).is_some(){return Err(fail());}
   }
  }
  Ok(out)
 }
}
