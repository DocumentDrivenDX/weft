//! Original association correspondence data; no native or semantic admission.
use crate::{current_presence::CurrentPresencePreparation,current_presence_model_requirements::{self,CurrentPresenceModelRequirements},error::{Diagnostic,Result},ir::Identity,json::{checked_json,checked_json_bounded,sha256},model::Catalog,security_association_ref::SecurityAssociationRef,security_budget::PayloadBudget,security_ontology::{authored_key_fields,SecurityRef}};
use serde::{Serialize,Serializer};
use serde_json::Value;
use std::{collections::{BTreeMap,BTreeSet},sync::Arc};
const LIMIT:usize=16_000_000;
const CODE:&str="WFT-CURRENT-PRESENCE-GRAPH-REQUIREMENTS";
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/current-presence-graph-requirements-v0.1-candidate.schema.json")]
struct ReportSchema;
fn fail()->Diagnostic{Diagnostic::new(CODE,"model","Original graph requirements refused")}
fn text(v:&Value)->Result<&str>{v.as_str().filter(|s|!s.is_empty()).ok_or_else(fail)}
fn array(v:&Value,min:usize,max:usize)->Result<&[Value]>{let a=v.as_array().ok_or_else(fail)?;if a.len()<min||a.len()>max{return Err(fail())}Ok(a)}
/// Actual typed model requirements remain owned, borrowed from the same context.
/// ```compile_fail
/// use weft_core::current_presence_graph_requirements::CurrentPresenceGraphRequirements;
/// fn imported(s:&str)->CurrentPresenceGraphRequirements<'_, '_>{serde_json::from_str(s).unwrap()}
/// ```
/// No report can replace the actual context or extend its lifetime.
/// ```compile_fail
/// use weft_core::{current_presence::CurrentPresencePreparation,current_presence_graph_requirements::{derive,CurrentPresenceGraphRequirements}};
/// fn detached(c:&CurrentPresencePreparation<'_>)->CurrentPresenceGraphRequirements<'static,'static>{derive(c).unwrap()}
/// ```
pub struct CurrentPresenceGraphRequirements<'context,'parent>{model:CurrentPresenceModelRequirements<'context,'parent>,json:String}
impl<'context,'parent> CurrentPresenceGraphRequirements<'context,'parent>{
 pub fn context(&self)->&'context CurrentPresencePreparation<'parent>{self.model.context()}
 pub fn model_requirements(&self)->&CurrentPresenceModelRequirements<'context,'parent>{&self.model}
 pub fn json(&self)->&str{&self.json}
}
#[derive(Serialize)]
struct Key{record:Identity,id:String,fields:Vec<Identity>}
#[derive(Clone)]
struct KeyHandle(Arc<Key>);
impl Serialize for KeyHandle{fn serialize<S:Serializer>(&self,s:S)->std::result::Result<S::Ok,S::Error>{self.0.serialize(s)}}
#[derive(Serialize)]
struct Header<'a>{ordinal:usize,kind:&'static str,reference:SecurityAssociationRef,revision:String,selection:&'a Value}
#[derive(Serialize)]
struct Member<'a>{field:&'a Value,descriptors:&'a Value}
#[derive(Serialize)]
struct RecordEndpoint<'a>{ordinal:usize,role:&'a str,target:Identity,key:KeyHandle,members:Vec<Member<'a>>}
#[derive(Serialize)]
struct RecordAssociation<'a>{#[serde(flatten)]header:Header<'a>,owner:Identity,key:KeyHandle,endpoints:Vec<RecordEndpoint<'a>>}
#[derive(Serialize)]
struct SourceDeclaration<'a>{ordinal:usize,record:Identity,metadata:&'a Value}
#[derive(Serialize)]
struct TargetDeclaration<'a>{ordinal:usize,record:Identity,key:KeyHandle,metadata:&'a Value}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct GraphEndpoint<'a>{ordinal:usize,role:&'a str,side:&'a str,target:Identity,key:KeyHandle,declaration_ordinals:Vec<usize>}
#[derive(Serialize)]
#[serde(tag="kind",rename_all="kebab-case")]
enum Witness{OpaqueExistential,RecordKey{record:Identity,key:KeyHandle}}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct GraphAssociation<'a>{#[serde(flatten)]header:Header<'a>,authored:&'a Value,sources:Vec<SourceDeclaration<'a>>,targets:Vec<TargetDeclaration<'a>>,association_record:Option<Identity>,endpoints:Vec<GraphEndpoint<'a>>,witness:Witness}
#[derive(Serialize)]
#[serde(untagged)]
enum Association<'a>{Record(RecordAssociation<'a>),Graph(GraphAssociation<'a>)}
struct Resolver<'a>{catalog:&'a Catalog,records:&'a BTreeMap<SecurityRef,Value>,keys:BTreeMap<(SecurityRef,String),KeyHandle>,key_budget:PayloadBudget,detail_budget:PayloadBudget}
impl Resolver<'_>{
 fn identity(&self,r:&SecurityRef)->Result<Identity>{
  let pins=self.catalog.pins();let mut found=pins.iter().filter(|p|p.document_id==r.document_id);let pin=found.next().ok_or_else(fail)?;if found.next().is_some(){return Err(fail())}
  Ok(Identity{document_id:r.document_id.clone(),revision:pin.revision.clone(),module:r.module_id.clone(),element:r.element_id.clone()})
 }
 fn record(&self,r:&SecurityRef)->Result<Identity>{let id=self.identity(r)?;self.catalog.record_by_identity(&id)?;Ok(id)}
 fn local(document:&str,v:&Value)->Result<SecurityRef>{Ok(SecurityRef{document_id:document.into(),module_id:text(&v["module"])?.into(),element_id:text(&v["element"])?.into()})}
 fn key(&mut self,r:&SecurityRef,id:&str)->Result<KeyHandle>{
  let cache=(r.clone(),id.to_owned());if let Some(k)=self.keys.get(&cache){return Ok(k.clone())}
  let record=self.record(r)?;let fields=authored_key_fields(self.catalog,r,id)?.iter().map(|f|self.identity(f)).collect::<Result<_>>()?;
  let key=Key{record,id:id.into(),fields};self.key_budget.copy_json(&key).map_err(|_|fail())?;
  let key=KeyHandle(Arc::new(key));self.keys.insert(cache,key.clone());Ok(key)
 }
 fn selected(&mut self,r:&SecurityRef,requested:Option<&str>)->Result<KeyHandle>{
  let record=self.records.get(r).ok_or_else(fail)?;let id=text(&record["selectedKey"]["id"])?;
  if requested.is_some_and(|v|v!=id){return Err(fail())}self.key(r,id)
 }
 fn member<'a>(&self,records:&'a BTreeMap<SecurityRef,Value>,owner:&SecurityRef,field:&SecurityRef)->Result<Member<'a>>{
  let id=serde_json::to_value(self.identity(field)?).map_err(|_|fail())?;let members=records.get(owner).ok_or_else(fail)?["fields"].as_array().ok_or_else(fail)?;
  let mut matches=members.iter().filter(|m|m["field"]==id);let m=matches.next().ok_or_else(fail)?;if matches.next().is_some(){return Err(fail())}
  Ok(Member{field:&m["field"],descriptors:&m["descriptors"]})
 }
 fn charge<T:Serialize>(&mut self,v:&T)->Result<()>{self.detail_budget.copy_json(v).map_err(|_|fail())}
 fn matches(&mut self,output:&mut Vec<usize>,ordinal:usize)->Result<()>{self.charge(&ordinal)?;output.push(ordinal);Ok(())}
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct Report<'a>{version:&'static str,scope:&'static str,complete:bool,authority:bool,security_ready:bool,native_admission:&'static str,source_request_json:&'a str,source_request_sha256:String,preparation_sha256:String,model_requirements_sha256:String,model_pins:Vec<crate::ir::ModelPin>,associations:Vec<Association<'a>>,unresolved:Vec<&'static str>}
// Charge the encoded payload before retaining an entry, independently of the
// final report encoder. Failure leaves already retained entries unchanged.
fn retain<T:Serialize>(budget:&mut PayloadBudget,entries:&mut Vec<T>,entry:T)->Result<()>{
 budget.copy_json(&entry).map_err(|_|fail())?;entries.push(entry);Ok(())
}
fn encode<T:Serialize>(v:&T)->Result<String>{
 struct Buffer{bytes:Vec<u8>,left:usize}
 impl std::io::Write for Buffer{fn write(&mut self,b:&[u8])->std::io::Result<usize>{self.left=self.left.checked_sub(b.len()).ok_or_else(||std::io::Error::other("budget"))?;self.bytes.extend_from_slice(b);Ok(b.len())}fn flush(&mut self)->std::io::Result<()>{Ok(())}}
 let mut out=Buffer{bytes:Vec::new(),left:LIMIT};serde_json::to_writer(&mut out,v).map_err(|_|fail())?;String::from_utf8(out.bytes).map_err(|_|fail())
}
/// Resolves metadata only. Unknown annotations survive; the original owner must
/// decide semantic validity/required unsupported meaning before native admission.
pub fn derive<'context,'parent>(context:&'context CurrentPresencePreparation<'parent>)->Result<CurrentPresenceGraphRequirements<'context,'parent>>{
 let model=current_presence_model_requirements::derive(context)?;
 let json={
  let parent=context.preparation();let catalog=parent.catalog();let records=model.record_requirements();
  let request=checked_json(parent.source_request_json()).map_err(|_|fail())?;let mut sources=array(&request["opaqueSources"],1,4096)?.iter().filter(|s|s["role"]=="ontology");let source=sources.next().ok_or_else(fail)?;if sources.next().is_some(){return Err(fail())}
  let ontology=checked_json(text(&source["utf8"])?).map_err(|_|fail())?;
  let mut resolver=Resolver{catalog,records,keys:BTreeMap::new(),key_budget:PayloadBudget::result_phase(CODE),detail_budget:PayloadBudget::result_phase(CODE)};
  let mut entries=Vec::new();let mut output_budget=PayloadBudget::result_phase(CODE);output_budget.copy_json(&parent.source_request_json()).map_err(|_|fail())?;output_budget.copy_json(&catalog.pins()).map_err(|_|fail())?;
  for (ordinal,selection) in array(&ontology["associations"],0,4096)?.iter().enumerate(){
   let entry=match text(&selection["kind"])?{
    "record-members"=>{
     let r=SecurityRef::read(&selection["type"])?;let owner=resolver.record(&r)?;let key=resolver.selected(&r,Some(text(&selection["keyId"])?))?;let mut endpoints=Vec::new();
     for (ordinal,e) in array(&selection["endpoints"],1,256)?.iter().enumerate(){
      let target_ref=SecurityRef::read(&e["target"])?;let target=resolver.record(&target_ref)?;let target_key=resolver.selected(&target_ref,None)?;let mut members=Vec::new();
      for field in array(&e["fields"],1,256)?{let f=SecurityRef::read(field)?;let member=resolver.member(records,&r,&f)?;resolver.charge(&member)?;members.push(member)}
      if members.len()!=target_key.0.fields.len(){return Err(fail())}
      let endpoint=RecordEndpoint{ordinal,role:text(&e["role"])?,target,key:target_key,members};resolver.charge(&endpoint.role)?;endpoints.push(endpoint)
     }
     Association::Record(RecordAssociation{header:Header{ordinal,kind:"record-members",reference:SecurityAssociationRef::Record(r),revision:owner.revision.clone(),selection},owner,key,endpoints})
    },
    "core-relationship"=>{
     let reference=SecurityAssociationRef::read(&selection["relationship"])?;let SecurityAssociationRef::Relationship(r)=&reference else{return Err(fail())};
     let pins=catalog.pins();let pin=pins.iter().find(|p|p.document_id==r.document_id).ok_or_else(fail)?;let revision=pin.revision.clone();
     let doc=catalog.documents.iter().find(|d|d["id"]==r.document_id).ok_or_else(fail)?;let module=doc["modules"].as_array().ok_or_else(fail)?.iter().find(|m|m["id"]==r.module_id).ok_or_else(fail)?;
     let mut matches=module["relationships"].as_array().ok_or_else(fail)?.iter().filter(|a|a["id"]==r.relationship_id);let authored=matches.next().ok_or_else(fail)?;if matches.next().is_some(){return Err(fail())}
     // Keep original metadata, including annotations and exact parsed values.
     // These required declarations enable correspondence, not semantic admission.
     if !authored["directed"].is_boolean()||!authored["sourceMultiplicity"].is_object()||!authored["targetMultiplicity"].is_object(){return Err(fail())}text(&authored["targetLifecycle"])?;
     let mut declared_sources=Vec::new();let mut declared_targets=Vec::new();
     for (ordinal,metadata) in array(&authored["source"],1,4096)?.iter().enumerate(){let reference=Resolver::local(&r.document_id,metadata)?;let record=resolver.record(&reference)?;let declaration=SourceDeclaration{ordinal,record,metadata};resolver.charge(&declaration)?;declared_sources.push(declaration)}
     for (ordinal,metadata) in array(&authored["target"],1,4096)?.iter().enumerate(){let reference=Resolver::local(&r.document_id,metadata)?;let record=resolver.record(&reference)?;let key=resolver.key(&reference,text(&metadata["key"])?)?;let declaration=TargetDeclaration{ordinal,record,key,metadata};resolver.charge(&declaration)?;declared_targets.push(declaration)}
     let association_record=match authored.get("associationRecord"){None=>None,Some(value)=>Some(resolver.record(&Resolver::local(&r.document_id,value)?)?)};
     let mut endpoints=Vec::new();let mut sides=BTreeSet::new();
     for (ordinal,e) in array(&selection["endpoints"],2,2)?.iter().enumerate(){
      let side=text(&e["side"])?;if !matches!(side,"source"|"target")||!sides.insert(side){return Err(fail())}
      let target_ref=SecurityRef::read(&e["target"])?;let target=resolver.record(&target_ref)?;let key=resolver.selected(&target_ref,Some(text(&e["keyId"])?))?;let mut declaration_ordinals=Vec::new();
      if side=="source"{for d in &declared_sources{if d.record==target{resolver.matches(&mut declaration_ordinals,d.ordinal)?}}}
      else{for d in &declared_targets{if d.record==target&&d.key.0.id==key.0.id{resolver.matches(&mut declaration_ordinals,d.ordinal)?}}}
      if declaration_ordinals.is_empty(){return Err(fail())}
      let endpoint=GraphEndpoint{ordinal,role:text(&e["role"])?,side,target,key,declaration_ordinals};resolver.charge(&endpoint.role)?;endpoints.push(endpoint)
     }
     let w=&selection["witness"];let witness=match text(&w["kind"])?{
      "opaque-existential"=>{if association_record.is_some(){return Err(fail())}Witness::OpaqueExistential},
      "record-key"=>{let r=SecurityRef::read(&w["type"])?;let record=resolver.record(&r)?;if association_record.as_ref()!=Some(&record){return Err(fail())}let key=resolver.selected(&r,Some(text(&w["keyId"])?))?;Witness::RecordKey{record,key}},
      _=>return Err(fail()),
     };
     Association::Graph(GraphAssociation{header:Header{ordinal,kind:"core-relationship",reference,revision,selection},authored,sources:declared_sources,targets:declared_targets,association_record,endpoints,witness})
    },
    _=>return Err(fail()),
   };
   retain(&mut output_budget,&mut entries,entry)?
  }
  let report=Report{version:"weft.current-presence.graph-requirements/0.1-candidate",scope:"original-association-endpoint-requirements-only",complete:false,authority:false,security_ready:false,native_admission:"required",source_request_json:parent.source_request_json(),source_request_sha256:sha256(parent.source_request_json().as_bytes()),preparation_sha256:sha256(context.json().as_bytes()),model_requirements_sha256:sha256(model.json().as_bytes()),model_pins:catalog.pins(),associations:entries,unresolved:vec!["original UMF whole-source semantics and required-meaning verdict","native graph interpretation, codec, complete occurrence populations and context correspondence","actual held source/caller/current-authority witness","closing guards and guarded release"]};
  let json=encode(&report)?;
  // Issued-output validation only: no report importer or narrower input scanner.
  // Encoded bytes bound value count; wrapper depth stays below twice input depth.
  let value=checked_json_bounded(&json,LIMIT,256,LIMIT).map_err(|_|fail())?;if !ReportSchema::is_valid(&value){return Err(fail())}json
 };
 Ok(CurrentPresenceGraphRequirements{model,json})
}

#[cfg(test)]
#[path="current_presence_graph_requirements_tests.rs"]
mod tests;
