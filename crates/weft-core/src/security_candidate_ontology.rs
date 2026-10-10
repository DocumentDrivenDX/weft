//! Draft 0.2 selected ontology closure; no policy/IR or native authorization.
use crate::{error::{Diagnostic,Result},model::Catalog,security_source::SecurityCandidateSourcePacket,security_ontology::{SecurityRef,TypeBinding,close_entities,locate,domain},security_association_ref::SecurityAssociationRef};
use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-CANDIDATE-ONTOLOGY","model","Draft selected ontology closure refused")}
fn known(v:&Value,keys:&[&str])->Result<()>{if v.as_object().ok_or_else(fail)?.keys().any(|k|!keys.contains(&k.as_str())){return Err(fail());}Ok(())}
fn no_extensions(v:&Value)->Result<()>{if v.get("extensions").is_some_and(|x|x.as_object().is_none_or(|o|!o.is_empty())){return Err(fail());}Ok(())}
#[derive(Debug)]
pub struct SecurityCandidateOntologyClosure{pub(crate) types:BTreeMap<SecurityRef,TypeBinding>,pub(crate) subject:SecurityRef,pub(crate) associations:BTreeMap<SecurityAssociationRef,Value>,pub(crate) context:BTreeSet<SecurityRef>}
impl SecurityCandidateOntologyClosure{
 pub fn type_count(&self)->usize{self.types.len()}
 pub fn association_count(&self)->usize{self.associations.len()}
 pub fn subject(&self)->&SecurityRef{&self.subject}
 pub fn read(packet:&SecurityCandidateSourcePacket,catalog:&Catalog)->Result<Self>{
  packet.require_catalog(catalog)?;let o=packet.ontology();
  let (types,fields)=close_entities(o["entities"].as_array().ok_or_else(fail)?,catalog)?;
  // Unknown selected Record/Field/Key qualifiers must not disappear in a typed projection.
  let mut field_owners=BTreeSet::new();
  for (r,b) in &types{let record=locate(catalog,r)?;known(record,&["id","kind","name","description","title","aliases","extensions","members","keys"])?;no_extensions(record)?;
   let mut key_ids=BTreeSet::new();let mut key_names=BTreeSet::new();let mut key_sets=BTreeSet::new();
   for k in record["keys"].as_array().ok_or_else(fail)?{
    known(k,&["id","name","fields","primary"])?;
    if !key_ids.insert(k["id"].as_str().ok_or_else(fail)?)||!key_names.insert(k["name"].as_str().ok_or_else(fail)?){return Err(fail());}
    let mut components=BTreeSet::new();for f in k["fields"].as_array().ok_or_else(fail)?{
     known(f,&["module","element"])?;let field=SecurityRef{document_id:r.document_id.clone(),module_id:f["module"].as_str().ok_or_else(fail)?.into(),element_id:f["element"].as_str().ok_or_else(fail)?.into()};
     if !components.insert(field.clone())||!b.fields.contains(&field){return Err(fail());}
     let source=locate(catalog,&field)?;if source["nullability"]!="required"{return Err(fail());}domain(source)?;
    }
    if components.is_empty()||!key_sets.insert(components){return Err(fail());}
   }
   for f in &b.fields{if !field_owners.insert(f.clone()){return Err(fail());}no_extensions(fields.get(f).ok_or_else(fail)?)?;}
  }
  let subject=SecurityRef::read(&o["subject"])?;if !types.contains_key(&subject){return Err(fail());}
  let mut contexts=BTreeSet::new();for f in o["context"].as_array().ok_or_else(fail)?{let r=SecurityRef::read(f)?;if !contexts.insert(r.clone())||!fields.contains_key(&r){return Err(fail());}}
  let mut actions=BTreeSet::new();for a in o["actions"].as_array().ok_or_else(fail)?{if !actions.insert(a.as_str().ok_or_else(fail)?){return Err(fail());}}
  let mut associations=BTreeMap::new();
  for selector in o["associations"].as_array().ok_or_else(fail)?{
   let graph=selector["kind"]=="core-relationship";let reference=SecurityAssociationRef::read(&selector[if graph{"relationship"}else{"type"}])?;
   if associations.contains_key(&reference){return Err(fail());}let mut roles=BTreeSet::new();
   if !graph{
    let r=reference.record().ok_or_else(fail)?;let owner=types.get(r).ok_or_else(fail)?;if owner.source["keyId"]!=selector["keyId"]{return Err(fail());}
    for e in selector["endpoints"].as_array().ok_or_else(fail)?{
     if !roles.insert(e["role"].as_str().ok_or_else(fail)?){return Err(fail());}let target_ref=SecurityRef::read(&e["target"])?;if target_ref.document_id!=r.document_id{return Err(fail());}let target=types.get(&target_ref).ok_or_else(fail)?;
     let members=e["fields"].as_array().ok_or_else(fail)?;if members.len()!=target.keys.len(){return Err(fail());}let mut seen=BTreeSet::new();
     for (f,k) in members.iter().zip(&target.keys){let field=SecurityRef::read(f)?;if !owner.fields.contains(&field)||!seen.insert(field.clone())||domain(locate(catalog,&field)?)?!=domain(locate(catalog,k)?)?{return Err(fail());}}
    }
   }else{
    let SecurityAssociationRef::Relationship(r)=&reference else{return Err(fail());};
    let doc=catalog.documents.iter().find(|d|d["id"]==r.document_id).ok_or_else(fail)?;
    let module=doc["modules"].as_array().ok_or_else(fail)?.iter().find(|m|m["id"]==r.module_id).ok_or_else(fail)?;
    let relations:Vec<_>=module["relationships"].as_array().ok_or_else(fail)?.iter().filter(|x|x["id"]==r.relationship_id).collect();if relations.len()!=1{return Err(fail());}let relation=relations[0];
    known(relation,&["id","name","source","target","sourceMultiplicity","targetMultiplicity","targetLifecycle","directed","inverse","associationRecord"])?;
    if relation["directed"]!=true||!matches!(relation["targetLifecycle"].as_str(),Some("owned"|"independent"|"unspecified")){return Err(fail());}
    for side in ["source","target"]{if relation[side].as_array().ok_or_else(fail)?.len()!=1{return Err(fail());}}
    for m in ["sourceMultiplicity","targetMultiplicity"]{known(&relation[m],&["min","max"])?;let min=crate::security_literals::unsigned(&relation[m]["min"])?;if relation[m]["max"]!="*"&&crate::security_literals::unsigned(&relation[m]["max"])?<min{return Err(fail());}}
    let mut sides=BTreeSet::new();for e in selector["endpoints"].as_array().ok_or_else(fail)?{
     let side=e["side"].as_str().ok_or_else(fail)?;if !sides.insert(side)||!roles.insert(e["role"].as_str().ok_or_else(fail)?){return Err(fail());}
     let native=&relation[side][0];known(native,if side=="source"{&["module","element"]}else{&["module","element","key"]})?;
     let target=SecurityRef::read(&e["target"])?;let binding=types.get(&target).ok_or_else(fail)?;
     if target.document_id!=r.document_id||target.module_id!=native["module"]||target.element_id!=native["element"]||binding.source["keyId"]!=e["keyId"]||(side=="target"&&native["key"]!=e["keyId"]){return Err(fail());}
    }
    if sides.len()!=2{return Err(fail());}
    let w=&selector["witness"];if w["kind"]=="opaque-existential"{if relation.get("associationRecord").is_some(){return Err(fail());}}
    else{let native=relation.get("associationRecord").ok_or_else(fail)?;known(native,&["module","element"])?;let owner=SecurityRef::read(&w["type"])?;let binding=types.get(&owner).ok_or_else(fail)?;if owner.document_id!=r.document_id||owner.module_id!=native["module"]||owner.element_id!=native["element"]||binding.source["keyId"]!=w["keyId"]{return Err(fail());}}
   }
   associations.insert(reference,selector.clone());
  }
  Ok(Self{types,subject,associations,context:contexts})
 }
}
