//! Selected graph incidence values; normalization is not fact authority.
use crate::{error::{Diagnostic,Result},model::Catalog,security_candidate_ir::{SecurityCandidateLogicalPlan,CandidateSide},security_candidate_ontology::SecurityCandidateOntologyClosure,security_association_ref::{SecurityAssociationRef,SecurityRelationshipRef},security_ontology::{SecurityRef,locate,domain},security_literals::{ScalarLiteral,normalized_literal}};
use serde_json::Value;
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-CANDIDATE-INCIDENCE","model","Graph incidence value refused")}
#[derive(Debug)]
pub struct CandidateIncidenceValue{relationship:SecurityRelationshipRef,role:String,side:CandidateSide,target:SecurityRef,key_id:String,components:Vec<ScalarLiteral>,component_domains:Vec<Value>,model_pins:Vec<crate::ir::ModelPin>}
impl CandidateIncidenceValue{
 pub fn role(&self)->&str{&self.role}
 pub fn side(&self)->&CandidateSide{&self.side}
 pub fn target(&self)->&SecurityRef{&self.target}
 pub fn key_id(&self)->&str{&self.key_id}
 pub fn components(&self)->&[ScalarLiteral]{&self.components}
 pub fn component_domains(&self)->&[Value]{&self.component_domains}
 pub fn require_catalog(&self,catalog:&Catalog)->Result<()>{if catalog.pins()!=self.model_pins||catalog.inputs().len()!=catalog.documents.len(){return Err(fail());}for (input,document) in catalog.inputs().iter().zip(&catalog.documents){if crate::json::sha256(input.document_json.as_bytes())!=input.pin.sha256||input.pin.document_id!=document["id"]||input.pin.umf_version!=document["umf"]||crate::json::checked_json(&input.document_json).map_err(|_|fail())?!=*document{return Err(fail());}}Ok(())}

 pub fn read(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,association:&SecurityAssociationRef,role:&str,side:CandidateSide,target:&SecurityRef,key_id:&str,components:&[Value])->Result<Self>{
  Self::read_bounded(plan,catalog,association,role,side,target,key_id,components,4_000_000)
 }
 fn read_bounded(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,association:&SecurityAssociationRef,role:&str,side:CandidateSide,target:&SecurityRef,key_id:&str,components:&[Value],mut normalized_remaining:usize)->Result<Self>{
  plan.require_catalog(catalog)?;let closure=SecurityCandidateOntologyClosure::read(plan.source(),catalog)?;
  let SecurityAssociationRef::Relationship(relationship)=association else{return Err(fail());};
  let selector=closure.associations.get(association).ok_or_else(fail)?;
  let endpoint=selector["endpoints"].as_array().ok_or_else(fail)?.iter().find(|e|e["role"]==role).ok_or_else(fail)?;
  let expected_side=match endpoint["side"].as_str(){Some("source")=>CandidateSide::Source,Some("target")=>CandidateSide::Target,_=>return Err(fail())};
  if expected_side!=side||SecurityRef::read(&endpoint["target"])?!=*target||endpoint["keyId"]!=key_id{return Err(fail());}
  let binding=closure.types.get(target).ok_or_else(fail)?;if binding.source["keyId"]!=key_id||components.len()!=binding.keys.len()||components.len()>256{return Err(fail());}
  // Exact carrier scalar text is bounded before scalar normalization; source metadata
  // is retained by the plan and no digest substitutes for identity components.
  let mut remaining=4_000_000usize;for value in components{let object=value.as_object().filter(|o|o.len()==1).ok_or_else(fail)?;let (key,value)=object.iter().next().ok_or_else(fail)?;let size=if let Some(text)=value.as_str(){text.len()}else if value.is_boolean(){1}else{return Err(fail());};remaining=remaining.checked_sub(key.len()+size).ok_or_else(fail)?;}
  let mut normalized=Vec::new();let mut domains=Vec::new();for (value,field) in components.iter().zip(&binding.keys){let source=locate(catalog,field)?;let scalar=normalized_literal(source,value)?.ok_or_else(fail)?;let size=match &scalar{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};normalized_remaining=normalized_remaining.checked_sub(size).ok_or_else(fail)?;normalized.push(scalar);domains.push(domain(source)?);}
  Ok(Self{relationship:relationship.clone(),role:role.into(),side,target:target.clone(),key_id:key_id.into(),components:normalized,component_domains:domains,model_pins:catalog.pins()})
 }
}

/// One caller-supplied endpoint bundle. No edge identity or population authority
/// is inferred from this grouping; the eventual fact issuer must establish both.
#[derive(Debug)]
pub struct CandidateIncidenceBundle { association:SecurityAssociationRef,policy_digest:String,ontology_digest:String,values:Vec<CandidateIncidenceValue> }
pub struct CandidateIncidenceInput<'a> {pub role:&'a str,pub side:CandidateSide,pub target:&'a SecurityRef,pub key_id:&'a str,pub components:&'a [Value]}
impl CandidateIncidenceBundle {
 pub fn values(&self)->&[CandidateIncidenceValue]{&self.values}
 pub fn association(&self)->&SecurityAssociationRef{&self.association}
 pub fn require_plan(&self,plan:&SecurityCandidateLogicalPlan,catalog:&Catalog)->Result<()>{plan.require_catalog(catalog)?;if crate::json::sha256(plan.source().policy_json().as_bytes())!=self.policy_digest||crate::json::sha256(plan.source().ontology_json().as_bytes())!=self.ontology_digest{return Err(fail());}for value in &self.values{value.require_catalog(catalog)?;}Ok(())}
 pub fn read(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,association:&SecurityAssociationRef,inputs:&[CandidateIncidenceInput<'_>])->Result<Self>{
  Self::read_bounded(plan,catalog,association,inputs,4_000_000)
 }
 fn read_bounded(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,association:&SecurityAssociationRef,inputs:&[CandidateIncidenceInput<'_>],mut remaining:usize)->Result<Self>{
  plan.require_catalog(catalog)?;let closure=SecurityCandidateOntologyClosure::read(plan.source(),catalog)?;
  if !matches!(association,SecurityAssociationRef::Relationship(_)){return Err(fail());}
  let selector=closure.associations.get(association).ok_or_else(fail)?;let endpoints=selector["endpoints"].as_array().ok_or_else(fail)?;
  if inputs.len()!=endpoints.len()||inputs.len()>2{return Err(fail());}
  let mut roles=std::collections::BTreeSet::new();let mut values=Vec::new();
  for input in inputs {
   if !roles.insert(input.role){return Err(fail());}
   let value=CandidateIncidenceValue::read_bounded(plan,catalog,association,input.role,input.side.clone(),input.target,input.key_id,input.components,remaining)?;
   for scalar in value.components(){let size=match scalar{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};remaining=remaining.checked_sub(size).ok_or_else(fail)?;}
   values.push(value);
  }
  Ok(Self{association:association.clone(),policy_digest:crate::json::sha256(plan.source().policy_json().as_bytes()),ontology_digest:crate::json::sha256(plan.source().ontology_json().as_bytes()),values})
 }
}

/// Caller-declared simulation coverage, never native completeness evidence.
#[derive(Debug,Clone,Copy)]
pub enum SimulatedIncidenceCoverage { Complete,Incomplete }
/// Bounded existential conjunction over endpoint constraints on each single
/// bundle. Does not evaluate a general policy or issue an authorization permit.
pub fn simulate_incidence_exists(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,association:&SecurityAssociationRef,bundles:&[CandidateIncidenceBundle],constraints:&[CandidateIncidenceInput<'_>],coverage:SimulatedIncidenceCoverage)->Result<crate::security_composition::Truth>{
 simulate_incidence_exists_bounded(plan,catalog,association,bundles,constraints,coverage,16_000_000)
}
fn simulate_incidence_exists_bounded(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,association:&SecurityAssociationRef,bundles:&[CandidateIncidenceBundle],constraints:&[CandidateIncidenceInput<'_>],coverage:SimulatedIncidenceCoverage,mut comparison_bytes:usize)->Result<crate::security_composition::Truth>{
 use crate::security_composition::Truth;
 if bundles.len()>4096||constraints.is_empty()||constraints.len()>2{return Err(fail());}
 let mut roles=std::collections::BTreeSet::new();let mut expected=Vec::new();let mut remaining=4_000_000usize;
 for input in constraints{if !roles.insert(input.role){return Err(fail());}let value=CandidateIncidenceValue::read_bounded(plan,catalog,association,input.role,input.side.clone(),input.target,input.key_id,input.components,remaining)?;for scalar in value.components(){let cost=match scalar{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};remaining=remaining.checked_sub(cost).ok_or_else(fail)?;}expected.push(value);}
 // Validate all supplied bundles before observing truth, including later rows.
 for bundle in bundles{bundle.require_plan(plan,catalog)?;if bundle.association()!=association{return Err(fail());}}
 let mut found=false;
 for bundle in bundles{let mut matches=true;for wanted in &expected{let actual=bundle.values().iter().find(|value|value.role()==wanted.role()).ok_or_else(fail)?;
  for scalar in wanted.components().iter().chain(actual.components()){let cost=match scalar{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};comparison_bytes=comparison_bytes.checked_sub(cost).ok_or_else(fail)?;}
  matches &= actual.relationship==wanted.relationship&&actual.side==wanted.side&&actual.target==wanted.target&&actual.key_id==wanted.key_id&&actual.component_domains==wanted.component_domains&&actual.components==wanted.components;
 }found|=matches;}
 Ok(if found{Truth::True}else{match coverage{SimulatedIncidenceCoverage::Complete=>Truth::False,SimulatedIncidenceCoverage::Incomplete=>Truth::Unknown}})
}


/// Source-bound logical Record identity shared by raw and graph bindings.
#[derive(Debug)]
pub struct CandidateEntityIdentity{target:SecurityRef,key_id:String,components:Vec<ScalarLiteral>,domains:Vec<Value>,pins:Vec<crate::ir::ModelPin>,policy_digest:String,ontology_digest:String}
impl CandidateEntityIdentity{
 pub fn target(&self)->&SecurityRef{&self.target}
 pub fn key_id(&self)->&str{&self.key_id}
 pub fn components(&self)->&[ScalarLiteral]{&self.components}
 pub fn component_domains(&self)->&[Value]{&self.domains}
 pub fn read(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,target:&SecurityRef,key_id:&str,components:&[Value])->Result<Self>{
  Self::read_bounded(plan,catalog,target,key_id,components,4_000_000)
 }
 fn read_bounded(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,target:&SecurityRef,key_id:&str,components:&[Value],mut remaining:usize)->Result<Self>{
  plan.require_catalog(catalog)?;let closure=SecurityCandidateOntologyClosure::read(plan.source(),catalog)?;let binding=closure.types.get(target).ok_or_else(fail)?;
  if binding.source["keyId"]!=key_id||components.len()!=binding.keys.len()||components.len()>256{return Err(fail());}
  let mut text=4_000_000usize;for value in components{let object=value.as_object().filter(|o|o.len()==1).ok_or_else(fail)?;let (key,value)=object.iter().next().ok_or_else(fail)?;let size=if let Some(v)=value.as_str(){v.len()}else if value.is_boolean(){1}else{return Err(fail());};text=text.checked_sub(key.len()+size).ok_or_else(fail)?;}
  let mut normalized=Vec::new();let mut domains=Vec::new();for (value,field) in components.iter().zip(&binding.keys){let source=locate(catalog,field)?;let scalar=normalized_literal(source,value)?.ok_or_else(fail)?;let size=match &scalar{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};remaining=remaining.checked_sub(size).ok_or_else(fail)?;normalized.push(scalar);domains.push(domain(source)?);}
  Ok(Self{target:target.clone(),key_id:key_id.into(),components:normalized,domains,pins:catalog.pins(),policy_digest:crate::json::sha256(plan.source().policy_json().as_bytes()),ontology_digest:crate::json::sha256(plan.source().ontology_json().as_bytes())})
 }
 pub fn require_plan(&self,plan:&SecurityCandidateLogicalPlan,catalog:&Catalog)->Result<()>{plan.require_catalog(catalog)?;if catalog.pins()!=self.pins||crate::json::sha256(plan.source().policy_json().as_bytes())!=self.policy_digest||crate::json::sha256(plan.source().ontology_json().as_bytes())!=self.ontology_digest{return Err(fail());}Ok(())}
}

/// Explicit selected field values; partial inventories are allowed at construction
/// but every used field must exist before interpretation. No fact authority.
#[derive(Debug)]
pub struct CandidateEntityFact{identity:CandidateEntityIdentity,fields:std::collections::BTreeMap<SecurityRef,ScalarLiteral>}
impl CandidateEntityFact{
 pub fn read(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,target:&SecurityRef,key_id:&str,components:&[Value],fields:&[(SecurityRef,Value)])->Result<Self>{
  Self::read_bounded(plan,catalog,target,key_id,components,fields,4_000_000)
 }
 fn read_bounded(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,target:&SecurityRef,key_id:&str,components:&[Value],fields:&[(SecurityRef,Value)],mut remaining:usize)->Result<Self>{
  if fields.len()>4096{return Err(fail());}let identity=CandidateEntityIdentity::read(plan,catalog,target,key_id,components)?;let closure=SecurityCandidateOntologyClosure::read(plan.source(),catalog)?;let binding=closure.types.get(target).ok_or_else(fail)?;let mut normalized=std::collections::BTreeMap::new();let mut text=4_000_000usize;
  for (field,value) in fields{if !binding.fields.contains(field)||normalized.contains_key(field){return Err(fail());}for part in [&field.document_id,&field.module_id,&field.element_id]{text=text.checked_sub(part.len()).ok_or_else(fail)?;}let object=value.as_object().filter(|o|o.len()==1).ok_or_else(fail)?;let (key,carrier)=object.iter().next().ok_or_else(fail)?;let size=if let Some(v)=carrier.as_str(){v.len()}else if carrier.is_boolean(){1}else{return Err(fail());};text=text.checked_sub(key.len()+size).ok_or_else(fail)?;let scalar=normalized_literal(locate(catalog,field)?,value)?.ok_or_else(fail)?;let cost=match &scalar{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};remaining=remaining.checked_sub(cost).ok_or_else(fail)?;if let Some(index)=binding.keys.iter().position(|r|r==field){if scalar!=identity.components[index]{return Err(fail());}}normalized.insert(field.clone(),scalar);}
  Ok(Self{identity,fields:normalized})
 }
}


/// One original raw association row and its complete declared endpoint projection.
/// Does not authenticate row provenance or association population completeness.
#[derive(Debug)]
pub struct CandidateRecordWitness{association:SecurityAssociationRef,fact:CandidateEntityFact,endpoints:std::collections::BTreeMap<String,CandidateEntityIdentity>}
impl CandidateRecordWitness{
 pub fn association(&self)->&SecurityAssociationRef{&self.association}
 pub fn identity(&self)->&CandidateEntityIdentity{&self.fact.identity}
 pub fn endpoint(&self,role:&str)->Option<&CandidateEntityIdentity>{self.endpoints.get(role)}
 pub fn require_plan(&self,plan:&SecurityCandidateLogicalPlan,catalog:&Catalog)->Result<()>{self.fact.identity.require_plan(plan,catalog)}
 pub fn read(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,association:&SecurityAssociationRef,fact:CandidateEntityFact)->Result<Self>{Self::read_bounded(plan,catalog,association,fact,4_000_000)}
 fn read_bounded(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,association:&SecurityAssociationRef,fact:CandidateEntityFact,mut remaining:usize)->Result<Self>{
  fact.identity.require_plan(plan,catalog)?;let closure=SecurityCandidateOntologyClosure::read(plan.source(),catalog)?;let owner=association.record().ok_or_else(fail)?;let selector=closure.associations.get(association).ok_or_else(fail)?;if selector["kind"]!="record-members"||&fact.identity.target!=owner||selector["keyId"]!=fact.identity.key_id{return Err(fail());}
  let declared=selector["endpoints"].as_array().ok_or_else(fail)?;if declared.len()>64{return Err(fail());}let mut endpoints=std::collections::BTreeMap::new();
  for endpoint in declared{let target=SecurityRef::read(&endpoint["target"])?;let binding=closure.types.get(&target).ok_or_else(fail)?;let fields=endpoint["fields"].as_array().ok_or_else(fail)?;if fields.len()!=binding.keys.len(){return Err(fail());}let mut components=Vec::new();let mut domains=Vec::new();for (field,key) in fields.iter().zip(&binding.keys){let field=SecurityRef::read(field)?;let value=fact.fields.get(&field).ok_or_else(fail)?;let cost=match value{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};remaining=remaining.checked_sub(cost).ok_or_else(fail)?;components.push(value.clone());domains.push(domain(locate(catalog,key)?)?);}
   let identity=CandidateEntityIdentity{target,key_id:binding.source["keyId"].as_str().ok_or_else(fail)?.into(),components,domains,pins:fact.identity.pins.clone(),policy_digest:fact.identity.policy_digest.clone(),ontology_digest:fact.identity.ontology_digest.clone()};if endpoints.insert(endpoint["role"].as_str().ok_or_else(fail)?.into(),identity).is_some(){return Err(fail());}
  }
  Ok(Self{association:association.clone(),fact,endpoints})
 }
}
/// One source-bound Record-backed edge; grouping is caller supplied, not authenticated.
#[derive(Debug)]
pub struct CandidateGraphRecordWitness{bundle:CandidateIncidenceBundle,fact:CandidateEntityFact}
impl CandidateGraphRecordWitness{
 pub fn association(&self)->&SecurityAssociationRef{self.bundle.association()}
 pub fn identity(&self)->&CandidateEntityIdentity{&self.fact.identity}
 pub fn require_plan(&self,plan:&SecurityCandidateLogicalPlan,catalog:&Catalog)->Result<()>{self.bundle.require_plan(plan,catalog)?;self.fact.identity.require_plan(plan,catalog)}
 pub fn read(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,bundle:CandidateIncidenceBundle,fact:CandidateEntityFact)->Result<Self>{
  bundle.require_plan(plan,catalog)?;fact.identity.require_plan(plan,catalog)?;
  let closure=SecurityCandidateOntologyClosure::read(plan.source(),catalog)?;
  let selector=closure.associations.get(bundle.association()).ok_or_else(fail)?;
  if selector["kind"]!="core-relationship"||selector["witness"]["kind"]!="record-key"||SecurityRef::read(&selector["witness"]["type"])?!=fact.identity.target||selector["witness"]["keyId"]!=fact.identity.key_id{return Err(fail());}
  Ok(Self{bundle,fact})
 }
}
pub struct CandidateGraphRecordPopulation<'a>{pub association:&'a SecurityAssociationRef,pub witnesses:&'a [CandidateGraphRecordWitness],pub coverage:SimulatedIncidenceCoverage}

/// Explicit simulation context; declared channels never substitute for stored facts.
#[derive(Debug)]
pub struct CandidateContextValues{fields:std::collections::BTreeMap<SecurityRef,ScalarLiteral>,pins:Vec<crate::ir::ModelPin>,policy_digest:String,ontology_digest:String}
impl CandidateContextValues{
 pub fn read(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,fields:&[(SecurityRef,Value)])->Result<Self>{Self::read_bounded(plan,catalog,fields,4_000_000)}
 fn read_bounded(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,fields:&[(SecurityRef,Value)],mut remaining:usize)->Result<Self>{
  plan.require_catalog(catalog)?;if fields.len()>256{return Err(fail());}let closure=SecurityCandidateOntologyClosure::read(plan.source(),catalog)?;let mut normalized=std::collections::BTreeMap::new();let mut text=4_000_000usize;
  for (field,value) in fields{if !closure.context.contains(field)||normalized.contains_key(field){return Err(fail());}for part in [&field.document_id,&field.module_id,&field.element_id]{text=text.checked_sub(part.len()).ok_or_else(fail)?;}let object=value.as_object().filter(|o|o.len()==1).ok_or_else(fail)?;let (key,carrier)=object.iter().next().ok_or_else(fail)?;let size=if let Some(v)=carrier.as_str(){v.len()}else if carrier.is_boolean(){1}else{return Err(fail());};text=text.checked_sub(key.len()+size).ok_or_else(fail)?;let scalar=normalized_literal(locate(catalog,field)?,value)?.ok_or_else(fail)?;let cost=match &scalar{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};remaining=remaining.checked_sub(cost).ok_or_else(fail)?;normalized.insert(field.clone(),scalar);}
  Ok(Self{fields:normalized,pins:catalog.pins(),policy_digest:crate::json::sha256(plan.source().policy_json().as_bytes()),ontology_digest:crate::json::sha256(plan.source().ontology_json().as_bytes())})
 }
 pub fn require_plan(&self,plan:&SecurityCandidateLogicalPlan,catalog:&Catalog)->Result<()>{plan.require_catalog(catalog)?;if catalog.pins()!=self.pins||crate::json::sha256(plan.source().policy_json().as_bytes())!=self.policy_digest||crate::json::sha256(plan.source().ontology_json().as_bytes())!=self.ontology_digest{return Err(fail());}Ok(())}
}

pub struct CandidateRecordPopulation<'a>{pub association:&'a SecurityAssociationRef,pub witnesses:&'a [CandidateRecordWitness],pub coverage:SimulatedIncidenceCoverage}
#[derive(Clone,Copy)]
enum BoundWitness<'a>{Graph(&'a CandidateIncidenceBundle),Record(&'a CandidateRecordWitness),GraphRecord(&'a CandidateGraphRecordWitness)}
struct RulePopulation<'a>{rows:Vec<BoundWitness<'a>>,coverage:SimulatedIncidenceCoverage,record:bool}
#[derive(Clone,Copy)]
struct GraphIdentities<'a>{subject:Option<&'a CandidateEntityIdentity>,resource:Option<&'a CandidateEntityIdentity>,subject_fact:Option<&'a CandidateEntityFact>,resource_fact:Option<&'a CandidateEntityFact>,context:Option<&'a CandidateContextValues>,records:Option<&'a [CandidateRecordPopulation<'a>]>,graph_records:Option<&'a [CandidateGraphRecordPopulation<'a>]>}
/// Simulation populations have no authenticated issuer or native completeness.
pub struct CandidateGraphPopulation<'a>{pub association:&'a SecurityAssociationRef,pub bundles:&'a [CandidateIncidenceBundle],pub coverage:SimulatedIncidenceCoverage}
/// Original draft IR interpreter for opaque graph endpoint terms. The default
/// entry point supplies no intrinsic identities; the explicit identity entry
/// point supports subject/resource identities, and the fact entry point adds
/// selected field values and constants; the context entry point keeps context
/// scalars separate from stored fields. Raw-member and Record-backed
/// witness expressions remain explicit refusals.
pub fn simulate_graph_rule(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,rule_index:usize,populations:&[CandidateGraphPopulation<'_>])->Result<crate::security_composition::Truth>{
 simulate_graph_rule_bounded(plan,catalog,rule_index,populations,1_000_000,16_000_000)
}
fn simulate_graph_rule_bounded(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,rule_index:usize,populations:&[CandidateGraphPopulation<'_>],steps:usize,bytes:usize)->Result<crate::security_composition::Truth>{
 simulate_graph_rule_with_identities_bounded(plan,catalog,rule_index,populations,GraphIdentities{subject:None,resource:None,subject_fact:None,resource_fact:None,context:None,records:None,graph_records:None},steps,bytes)
}
pub fn simulate_graph_rule_with_identities(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,rule_index:usize,populations:&[CandidateGraphPopulation<'_>],subject:Option<&CandidateEntityIdentity>,resource:Option<&CandidateEntityIdentity>)->Result<crate::security_composition::Truth>{
 simulate_graph_rule_with_identities_bounded(plan,catalog,rule_index,populations,GraphIdentities{subject,resource,subject_fact:None,resource_fact:None,context:None,records:None,graph_records:None},1_000_000,16_000_000)
}
pub fn simulate_graph_rule_with_facts(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,rule_index:usize,populations:&[CandidateGraphPopulation<'_>],subject:Option<&CandidateEntityFact>,resource:Option<&CandidateEntityFact>)->Result<crate::security_composition::Truth>{
 simulate_graph_rule_with_identities_bounded(plan,catalog,rule_index,populations,GraphIdentities{subject:subject.map(|f|&f.identity),resource:resource.map(|f|&f.identity),subject_fact:subject,resource_fact:resource,context:None,records:None,graph_records:None},1_000_000,16_000_000)
}
pub fn simulate_graph_rule_with_context(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,rule_index:usize,populations:&[CandidateGraphPopulation<'_>],subject:Option<&CandidateEntityFact>,resource:Option<&CandidateEntityFact>,context:Option<&CandidateContextValues>)->Result<crate::security_composition::Truth>{
 simulate_graph_rule_with_identities_bounded(plan,catalog,rule_index,populations,GraphIdentities{subject:subject.map(|f|&f.identity),resource:resource.map(|f|&f.identity),subject_fact:subject,resource_fact:resource,context,records:None,graph_records:None},1_000_000,16_000_000)
}
/// Mixed raw Record and opaque graph simulation; caller coverage is not native authority.
pub fn simulate_candidate_rule(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,rule_index:usize,graphs:&[CandidateGraphPopulation<'_>],records:&[CandidateRecordPopulation<'_>],subject:Option<&CandidateEntityFact>,resource:Option<&CandidateEntityFact>,context:Option<&CandidateContextValues>)->Result<crate::security_composition::Truth>{
 simulate_graph_rule_with_identities_bounded(plan,catalog,rule_index,graphs,GraphIdentities{subject:subject.map(|f|&f.identity),resource:resource.map(|f|&f.identity),subject_fact:subject,resource_fact:resource,context,records:Some(records),graph_records:None},1_000_000,16_000_000)
}
/// Mixed raw, opaque and Record-backed graph simulation; no native fact authority.
pub fn simulate_candidate_rule_with_graph_records(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,rule_index:usize,graphs:&[CandidateGraphPopulation<'_>],records:&[CandidateRecordPopulation<'_>],graph_records:&[CandidateGraphRecordPopulation<'_>],subject:Option<&CandidateEntityFact>,resource:Option<&CandidateEntityFact>,context:Option<&CandidateContextValues>)->Result<crate::security_composition::Truth>{
 simulate_graph_rule_with_identities_bounded(plan,catalog,rule_index,graphs,GraphIdentities{subject:subject.map(|f|&f.identity),resource:resource.map(|f|&f.identity),subject_fact:subject,resource_fact:resource,context,records:Some(records),graph_records:Some(graph_records)},1_000_000,16_000_000)
}
fn simulate_graph_rule_with_identities_bounded(plan:&SecurityCandidateLogicalPlan,catalog:&Catalog,rule_index:usize,populations:&[CandidateGraphPopulation<'_>],identities:GraphIdentities<'_>,mut steps:usize,mut bytes:usize)->Result<crate::security_composition::Truth>{
 use crate::security_candidate_ir::{CandidateExpression as E,CandidateTerm as T,CandidateWitness,CandidateEndpointCarrier};use crate::security_ir::Binding;use crate::security_composition::{Truth,and,or,not};use std::collections::BTreeMap;
 plan.require_catalog(catalog)?;let rule=plan.rules().get(rule_index).ok_or_else(fail)?;if populations.len()+identities.records.map_or(0,|r|r.len())+identities.graph_records.map_or(0,|r|r.len())>512{return Err(fail());}
 let closure=SecurityCandidateOntologyClosure::read(plan.source(),catalog)?;
 for (identity,target) in [(identities.subject,&closure.subject),(identities.resource,&rule.target)]{if let Some(identity)=identity{identity.require_plan(plan,catalog)?;if &identity.target!=target{return Err(fail());}}}
 if let Some(context)=identities.context{context.require_plan(plan,catalog)?;}
 let mut data=BTreeMap::new();let mut count=0usize;for population in populations{if !matches!(population.association,SecurityAssociationRef::Relationship(_))||!closure.associations.contains_key(population.association){return Err(fail());}if closure.associations[population.association]["witness"]["kind"]!="opaque-existential"{return Err(fail());}count=count.checked_add(population.bundles.len()).ok_or_else(fail)?;if count>4096||data.insert(population.association,RulePopulation{rows:population.bundles.iter().map(BoundWitness::Graph).collect(),coverage:population.coverage,record:false}).is_some(){return Err(fail());}for bundle in population.bundles{bundle.require_plan(plan,catalog)?;if bundle.association()!=population.association{return Err(fail());}}}

 if let Some(records)=identities.records{for population in records{if population.association.record().is_none()||closure.associations.get(population.association).is_none_or(|s|s["kind"]!="record-members"){return Err(fail());}count=count.checked_add(population.witnesses.len()).ok_or_else(fail)?;if count>4096{return Err(fail());}let mut keys:Vec<&CandidateEntityIdentity>=Vec::new();for row in population.witnesses{row.require_plan(plan,catalog)?;if row.association()!=population.association{return Err(fail());}let identity=&row.fact.identity;for previous in &keys{for scalar in previous.components.iter().chain(&identity.components){let cost=match scalar{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};bytes=bytes.checked_sub(cost.max(1)).ok_or_else(fail)?;}if previous.target==identity.target&&previous.key_id==identity.key_id&&previous.components==identity.components{return Err(fail());}}keys.push(identity);}if data.insert(population.association,RulePopulation{rows:population.witnesses.iter().map(BoundWitness::Record).collect(),coverage:population.coverage,record:true}).is_some(){return Err(fail());}}}

 if let Some(populations)=identities.graph_records{for population in populations{
  if !matches!(population.association,SecurityAssociationRef::Relationship(_))||closure.associations.get(population.association).is_none_or(|s|s["kind"]!="core-relationship"||s["witness"]["kind"]!="record-key"){return Err(fail());}
  count=count.checked_add(population.witnesses.len()).ok_or_else(fail)?;if count>4096{return Err(fail());}
  let mut keys:Vec<&CandidateEntityIdentity>=Vec::new();
  for row in population.witnesses{row.require_plan(plan,catalog)?;if row.association()!=population.association{return Err(fail());}let identity=row.identity();for previous in &keys{for scalar in previous.components.iter().chain(&identity.components){let cost=match scalar{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};bytes=bytes.checked_sub(cost.max(1)).ok_or_else(fail)?;}if previous.target==identity.target&&previous.key_id==identity.key_id&&previous.components==identity.components{return Err(fail());}}keys.push(identity);}
  if data.insert(population.association,RulePopulation{rows:population.witnesses.iter().map(BoundWitness::GraphRecord).collect(),coverage:population.coverage,record:true}).is_some(){return Err(fail());}
 }}

 fn preflight(e:&E,data:&BTreeMap<&SecurityAssociationRef,RulePopulation<'_>>,identities:GraphIdentities<'_>,vars:&BTreeMap<usize,&SecurityAssociationRef>)->Result<()>{match e{E::Literal(_)=>{},E::Equal(a,b)=>for term in [a,b]{match term{
 T::Endpoint{binding:Binding::Variable(slot),association,..} if vars.get(slot).is_some_and(|a|*a==association)=>{},
 T::Value(crate::security_ir::Term::Identity{binding:Binding::Subject,..}) if identities.subject.is_some()=>{},T::Value(crate::security_ir::Term::Identity{binding:Binding::Resource,..}) if identities.resource.is_some()=>{},T::Value(crate::security_ir::Term::Identity{binding:Binding::Variable(slot),..}) if vars.get(slot).is_some_and(|a|data[*a].record)=>{},
 T::Value(crate::security_ir::Term::Field{binding:Binding::Subject,field,..}) if identities.subject_fact.is_some_and(|f|f.fields.contains_key(field))=>{},T::Value(crate::security_ir::Term::Field{binding:Binding::Resource,field,..}) if identities.resource_fact.is_some_and(|f|f.fields.contains_key(field))=>{},T::Value(crate::security_ir::Term::Field{binding:Binding::Variable(slot),field,..})=>{let population=data.get(vars.get(slot).ok_or_else(fail)?).ok_or_else(fail)?;if !population.record||population.rows.iter().any(|r|!match r{BoundWitness::Record(w)=>w.fact.fields.contains_key(field),BoundWitness::GraphRecord(w)=>w.fact.fields.contains_key(field),_=>false}){return Err(fail());}},
 T::Value(crate::security_ir::Term::Context{field,..}) if identities.context.is_some_and(|c|c.fields.contains_key(field))=>{},T::Value(crate::security_ir::Term::Constant{..})=>{},_=>return Err(fail())}},E::Not(e)=>preflight(e,data,identities,vars)?,E::And(es)|E::Or(es)=>for e in es{preflight(e,data,identities,vars)?;},E::Exists{slot,association,witness,condition}=>{let population=data.get(association).ok_or_else(fail)?;if population.record!=matches!(witness,CandidateWitness::RecordKey{..}){return Err(fail());}let mut nested=vars.clone();if nested.insert(*slot,association).is_some(){return Err(fail());}preflight(condition,data,identities,&nested)?;}}Ok(())}
 preflight(&rule.condition,&data,identities,&BTreeMap::new())?;
 fn identity_view<'a>(term:&T,vars:&BTreeMap<usize,BoundWitness<'a>>,identities:GraphIdentities<'a>)->Result<(&'a SecurityRef,&'a str,&'a [Value],&'a [ScalarLiteral])>{
  if let T::Value(crate::security_ir::Term::Identity{binding,target,key_id})=term{let value=match binding{Binding::Subject=>identities.subject,Binding::Resource=>identities.resource,Binding::Variable(slot)=>match vars.get(slot){Some(BoundWitness::Record(w))=>Some(&w.fact.identity),Some(BoundWitness::GraphRecord(w))=>Some(&w.fact.identity),_=>None}}.ok_or_else(fail)?;if &value.target!=target||&value.key_id!=key_id{return Err(fail());}return Ok((&value.target,&value.key_id,&value.domains,&value.components));}
  let T::Endpoint{binding:Binding::Variable(slot),association,role,target,key_id,carrier}=term else{return Err(fail());};match vars.get(slot).ok_or_else(fail)?{
   BoundWitness::Graph(bundle)=>{let CandidateEndpointCarrier::Incidence{side}=carrier else{return Err(fail());};if bundle.association()!=association{return Err(fail());}let value=bundle.values().iter().find(|v|v.role()==role).ok_or_else(fail)?;if value.side()!=side||value.target()!=target||value.key_id()!=key_id{return Err(fail());}Ok((&value.target,&value.key_id,&value.component_domains,&value.components))},
   BoundWitness::GraphRecord(witness)=>{let bundle=&witness.bundle;let CandidateEndpointCarrier::Incidence{side}=carrier else{return Err(fail());};if bundle.association()!=association{return Err(fail());}let value=bundle.values().iter().find(|v|v.role()==role).ok_or_else(fail)?;if value.side()!=side||value.target()!=target||value.key_id()!=key_id{return Err(fail());}Ok((&value.target,&value.key_id,&value.component_domains,&value.components))},
   BoundWitness::Record(witness)=>{if !matches!(carrier,CandidateEndpointCarrier::Members{..})||witness.association()!=association{return Err(fail());}let value=witness.endpoint(role).ok_or_else(fail)?;if value.target()!=target||value.key_id()!=key_id{return Err(fail());}Ok((&value.target,&value.key_id,&value.domains,&value.components))}
  }
 }

 fn scalar_term(term:&T)->bool{matches!(term,T::Value(crate::security_ir::Term::Field{..}|crate::security_ir::Term::Constant{..}|crate::security_ir::Term::Context{..}))}
 fn scalar(term:&T,vars:&BTreeMap<usize,BoundWitness<'_>>,identities:GraphIdentities<'_>,catalog:&Catalog,bytes:&mut usize)->Result<ScalarLiteral>{
  let T::Value(value)=term else{return Err(fail());};let result=match value{crate::security_ir::Term::Field{binding,field,..}=>{let fact=match binding{Binding::Subject=>identities.subject_fact,Binding::Resource=>identities.resource_fact,Binding::Variable(slot)=>match vars.get(slot){Some(BoundWitness::Record(w))=>Some(&w.fact),Some(BoundWitness::GraphRecord(w))=>Some(&w.fact),_=>None}}.ok_or_else(fail)?;let value=fact.fields.get(field).ok_or_else(fail)?;let size=match value{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};*bytes=bytes.checked_sub(size).ok_or_else(fail)?;return Ok(value.clone());},crate::security_ir::Term::Context{field,..}=>{let value=identities.context.ok_or_else(fail)?.fields.get(field).ok_or_else(fail)?;let size=match value{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};*bytes=bytes.checked_sub(size).ok_or_else(fail)?;return Ok(value.clone());},crate::security_ir::Term::Constant{field,literal,..}=>normalized_literal(locate(catalog,field)?,literal)?.ok_or_else(fail)?,_=>return Err(fail())};let size=match &result{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};*bytes=bytes.checked_sub(size).ok_or_else(fail)?;Ok(result)
 }
 fn eval<'a>(e:&E,catalog:&Catalog,data:&BTreeMap<&SecurityAssociationRef,RulePopulation<'a>>,vars:&BTreeMap<usize,BoundWitness<'a>>,identities:GraphIdentities<'a>,steps:&mut usize,bytes:&mut usize)->Result<Truth>{*steps=steps.checked_sub(1).ok_or_else(fail)?;Ok(match e{
  E::Literal(v)=>if *v{Truth::True}else{Truth::False},E::Equal(a,b) if scalar_term(a)&&scalar_term(b)=>{let a=scalar(a,vars,identities,catalog,bytes)?;let b=scalar(b,vars,identities,catalog,bytes)?;if a==b{Truth::True}else{Truth::False}},E::Equal(a,b)=>{let a=identity_view(a,vars,identities)?;let b=identity_view(b,vars,identities)?;for scalar in a.3.iter().chain(b.3){let cost=match scalar{ScalarLiteral::String(v)|ScalarLiteral::Binary(v)=>v.len(),ScalarLiteral::Number(v)=>v.magnitude().bits().div_ceil(8) as usize,_=>1};*bytes=bytes.checked_sub(cost).ok_or_else(fail)?;}if a==b{Truth::True}else{Truth::False}},
  E::Not(e)=>not(eval(e,catalog,data,vars,identities,steps,bytes)?),E::And(es)|E::Or(es)=>{let values=es.iter().map(|e|eval(e,catalog,data,vars,identities,steps,bytes)).collect::<Result<Vec<_>>>()?;if matches!(e,E::And(_)){and(&values)?}else{or(&values)?}},
  E::Exists{slot,association,condition,..}=>{let population=data.get(association).ok_or_else(fail)?;let mut found=false;let mut unknown=matches!(population.coverage,SimulatedIncidenceCoverage::Incomplete);for bundle in &population.rows{let mut nested=vars.clone();if nested.insert(*slot,*bundle).is_some(){return Err(fail());}match eval(condition,catalog,data,&nested,identities,steps,bytes)?{Truth::True=>found=true,Truth::Unknown=>unknown=true,Truth::False=>{}}}if found{Truth::True}else if unknown{Truth::Unknown}else{Truth::False}}
 })}
 eval(&rule.condition,catalog,&data,&BTreeMap::new(),identities,&mut steps,&mut bytes)
}

#[cfg(test)]
mod tests {
 use super::*;
 use serde_json::json;
 use crate::{model::ModuleInput,security_source::SecurityCandidateSourcePacket};
 fn fixture(decimal:bool,composite:bool)->(Catalog,SecurityCandidateLogicalPlan,SecurityAssociationRef,SecurityRef){
  let fixture:Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();
  let mut doc=fixture["resolution"]["documents"][0]["document"].clone();let mut ontology=fixture["resolution"]["ontology"].clone();let mut policy=fixture["policy"].clone();
  policy["version"]=json!("0.2.0");policy["revision"]=json!("policy-draft2");policy["ontology"]["revision"]=json!("ontology-draft2");ontology["version"]=json!("0.2.0");ontology["revision"]=json!("ontology-draft2");
  let original=ontology["associations"].as_array().unwrap().clone();for a in &original{let mut e=a.clone();e.as_object_mut().unwrap().remove("endpoints");ontology["entities"].as_array_mut().unwrap().push(e);}
  ontology["associations"]=Value::Array(original.iter().map(|a|json!({"kind":"record-members","type":a["type"],"keyId":a["keyId"],"endpoints":a["endpoints"]})).collect());
  for rule in policy["rules"].as_array_mut().unwrap(){rule["condition"]=json!({"op":"literal","value":true});}
  let elements=doc["modules"][0]["elements"].as_array_mut().unwrap();let field=elements.iter_mut().find(|e|e["id"]=="staffId").unwrap();field["scalarType"]=json!(if decimal{"decimal"}else{"integer"});if decimal{field["facets"]=json!({"precision":10,"scale":1});}
  if composite{let staff=elements.iter_mut().find(|e|e["id"]=="Staff").unwrap();staff["members"].as_array_mut().unwrap().push(json!({"module":"m","element":"staffExtra"}));staff["keys"][0]["fields"].as_array_mut().unwrap().push(json!({"module":"m","element":"staffExtra"}));elements.push(json!({"id":"staffExtra","kind":"field","scalarType":"integer","nullability":"required","cardinality":"one","extensions":{}}));ontology["entities"][0]["fields"].as_array_mut().unwrap().push(json!({"ref":{"documentId":"domain","moduleId":"m","elementId":"staffExtra"},"protection":"unprotected"}));}
  doc["modules"][0]["relationships"]=json!([{"id":"WorksOn","name":"WorksOn","source":[{"module":"m","element":"Staff"}],"target":[{"module":"m","element":"Project","key":"pk"}],"sourceMultiplicity":{"min":0,"max":"*"},"targetMultiplicity":{"min":0,"max":"*"},"targetLifecycle":"independent","directed":true}]);
  let relationship=json!({"documentId":"domain","moduleId":"m","relationshipId":"WorksOn"});let target=json!({"documentId":"domain","moduleId":"m","elementId":"Staff"});ontology["associations"][1]=json!({"kind":"core-relationship","relationship":relationship,"witness":{"kind":"opaque-existential"},"endpoints":[{"role":"staff","side":"source","target":target,"keyId":"pk"},{"role":"project","side":"target","target":{"documentId":"domain","moduleId":"m","elementId":"Project"},"keyId":"pk"}]});
  let text=doc.to_string();let input:ModuleInput=serde_json::from_value(json!({"documentJson":text,"pin":{"documentId":"domain","revision":"schema-1","umfVersion":"0.8.0","sha256":crate::json::sha256(text.as_bytes())},"selectedModuleIds":["m"]})).unwrap();let catalog=Catalog::prepare_security(vec![input]).unwrap();let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&ontology.to_string(),&catalog).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&catalog).unwrap();(catalog,plan,SecurityAssociationRef::read(&relationship).unwrap(),SecurityRef::read(&target).unwrap())
 }
 #[test]
 fn aggregate_normalized_storage_is_bounded_before_retention(){
  let (c,p,a,t)=fixture(false,true);let values=[json!({"integerToken":"1"}),json!({"integerToken":"2"})];
  assert!(CandidateIncidenceValue::read_bounded(&p,&c,&a,"staff",CandidateSide::Source,&t,"pk",&values,1).is_err());
  assert!(CandidateIncidenceValue::read_bounded(&p,&c,&a,"staff",CandidateSide::Source,&t,"pk",&values,2).is_ok());
  let (c,p,a,t)=fixture(false,false);let expanded=[json!({"integerToken":"1e1000"})];assert!(CandidateIncidenceValue::read_bounded(&p,&c,&a,"staff",CandidateSide::Source,&t,"pk",&expanded,10).is_err());assert!(CandidateIncidenceValue::read(&p,&c,&a,"staff",CandidateSide::Source,&t,"pk",&expanded).is_ok());
 }
 #[test]
 fn equal_numeric_carriers_retain_distinct_domains_and_catalog_custody(){
  let (ic,ip,ia,it)=fixture(false,false);let (dc,dp,da,dt)=fixture(true,false);
  let integer=CandidateIncidenceValue::read(&ip,&ic,&ia,"staff",CandidateSide::Source,&it,"pk",&[json!({"integerToken":"1"})]).unwrap();
  let decimal=CandidateIncidenceValue::read(&dp,&dc,&da,"staff",CandidateSide::Source,&dt,"pk",&[json!({"decimalToken":"0.1"})]).unwrap();
  assert_eq!(integer.components(),decimal.components());assert_ne!(integer.component_domains(),decimal.component_domains());assert!(integer.require_catalog(&ic).is_ok());assert!(decimal.require_catalog(&dc).is_ok());assert!(integer.require_catalog(&dc).is_err());assert!(decimal.require_catalog(&ic).is_err());
 }
 #[test]
 fn bundle_requires_complete_distinct_role_specific_endpoints(){
  let (c,p,a,t)=fixture(false,false);let project=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"Project".into()};let staff_values=[json!({"integerToken":"1"})];let project_values=[json!({"string":"p1"})];
  let staff=||CandidateIncidenceInput{role:"staff",side:CandidateSide::Source,target:&t,key_id:"pk",components:&staff_values};let project_input=||CandidateIncidenceInput{role:"project",side:CandidateSide::Target,target:&project,key_id:"pk",components:&project_values};
  let bundle=CandidateIncidenceBundle::read(&p,&c,&a,&[staff(),project_input()]).unwrap();assert_eq!(bundle.association(),&a);assert!(bundle.require_plan(&p,&c).is_ok());let mut other_policy=p.source().policy().clone();other_policy["revision"]=json!("different-policy");let other_packet=SecurityCandidateSourcePacket::read(&other_policy.to_string(),p.source().ontology_json(),&c).unwrap();let other_plan=SecurityCandidateLogicalPlan::read(other_packet,&c).unwrap();assert!(bundle.require_plan(&other_plan,&c).is_err());let mut other_ontology=p.source().ontology().clone();other_ontology["associations"][1]["endpoints"][0]["role"]=json!("different-staff-role");let other_packet=SecurityCandidateSourcePacket::read(p.source().policy_json(),&other_ontology.to_string(),&c).unwrap();let other_plan=SecurityCandidateLogicalPlan::read(other_packet,&c).unwrap();assert!(bundle.require_plan(&other_plan,&c).is_err());assert_eq!(bundle.values()[0].role(),"staff");assert_eq!(bundle.values()[0].side(),&CandidateSide::Source);let mut stale_inputs=c.inputs().to_vec();stale_inputs[0].document_json.push(' ');stale_inputs[0].pin.sha256=crate::json::sha256(stale_inputs[0].document_json.as_bytes());let stale=Catalog::prepare_security(stale_inputs).unwrap();assert!(bundle.require_plan(&p,&stale).is_err());assert_eq!(bundle.values().len(),2);assert_eq!(bundle.values()[0].target(),&t);assert_eq!(bundle.values()[1].target(),&project);
  assert!(CandidateIncidenceBundle::read(&p,&c,&a,&[staff()]).is_err());assert!(CandidateIncidenceBundle::read(&p,&c,&a,&[staff(),staff()]).is_err());let mut reversed=project_input();reversed.side=CandidateSide::Source;assert!(CandidateIncidenceBundle::read(&p,&c,&a,&[staff(),reversed]).is_err());
  assert!(CandidateIncidenceBundle::read(&p,&c,&a,&[project_input(),staff()]).is_ok());
  for inputs in [[staff(),project_input()],[project_input(),staff()]]{assert!(CandidateIncidenceBundle::read_bounded(&p,&c,&a,&inputs,2).is_err());assert!(CandidateIncidenceBundle::read_bounded(&p,&c,&a,&inputs,3).is_ok());}

 }

 #[test]
 fn existential_simulation_keeps_both_endpoints_on_the_same_bundle(){
  use crate::security_composition::Truth;let (c,p,a,t)=fixture(false,false);let project=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"Project".into()};
  let s1=[json!({"integerToken":"1"})];let s2=[json!({"integerToken":"2"})];let p1=[json!({"string":"p1"})];let p2=[json!({"string":"p2"})];
  let staff=|v|CandidateIncidenceInput{role:"staff",side:CandidateSide::Source,target:&t,key_id:"pk",components:v};let proj=|v|CandidateIncidenceInput{role:"project",side:CandidateSide::Target,target:&project,key_id:"pk",components:v};
  let split=[CandidateIncidenceBundle::read(&p,&c,&a,&[staff(&s1),proj(&p2)]).unwrap(),CandidateIncidenceBundle::read(&p,&c,&a,&[staff(&s2),proj(&p1)]).unwrap()];let constraints=[staff(&s1),proj(&p1)];
  assert_eq!(simulate_incidence_exists(&p,&c,&a,&split,&constraints,SimulatedIncidenceCoverage::Complete).unwrap(),Truth::False);assert_eq!(simulate_incidence_exists(&p,&c,&a,&split,&constraints,SimulatedIncidenceCoverage::Incomplete).unwrap(),Truth::Unknown);
  let joined=[CandidateIncidenceBundle::read(&p,&c,&a,&[staff(&s1),proj(&p1)]).unwrap()];assert_eq!(simulate_incidence_exists(&p,&c,&a,&joined,&constraints,SimulatedIncidenceCoverage::Incomplete).unwrap(),Truth::True);
  assert_eq!(simulate_incidence_exists(&p,&c,&a,&[],&constraints,SimulatedIncidenceCoverage::Complete).unwrap(),Truth::False);assert_eq!(simulate_incidence_exists(&p,&c,&a,&[],&constraints,SimulatedIncidenceCoverage::Incomplete).unwrap(),Truth::Unknown);
  assert!(simulate_incidence_exists_bounded(&p,&c,&a,&joined,&constraints,SimulatedIncidenceCoverage::Complete,5).is_err());assert!(simulate_incidence_exists_bounded(&p,&c,&a,&joined,&constraints,SimulatedIncidenceCoverage::Complete,6).is_ok());let twice=[CandidateIncidenceBundle::read(&p,&c,&a,&[staff(&s1),proj(&p1)]).unwrap(),CandidateIncidenceBundle::read(&p,&c,&a,&[staff(&s1),proj(&p1)]).unwrap()];assert!(simulate_incidence_exists_bounded(&p,&c,&a,&twice,&constraints,SimulatedIncidenceCoverage::Complete,6).is_err());assert!(simulate_incidence_exists_bounded(&p,&c,&a,&twice,&constraints,SimulatedIncidenceCoverage::Complete,12).is_ok());
  assert!(simulate_incidence_exists(&p,&c,&a,&joined,&[staff(&s1),staff(&s1)],SimulatedIncidenceCoverage::Complete).is_err());
 }

 #[test]
 fn existential_simulation_matches_independent_finite_edge_oracle(){
  use crate::security_composition::Truth;let (c,p,a,t)=fixture(false,false);let project=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"Project".into()};let staff_values=[[json!({"integerToken":"1"})],[json!({"integerToken":"2"})]];let project_values=[[json!({"string":"p1"})],[json!({"string":"p2"})]];
  let staff=|i:usize|CandidateIncidenceInput{role:"staff",side:CandidateSide::Source,target:&t,key_id:"pk",components:&staff_values[i]};let proj=|i:usize|CandidateIncidenceInput{role:"project",side:CandidateSide::Target,target:&project,key_id:"pk",components:&project_values[i]};
  for population in 0..16 {let mut bundles=Vec::new();let mut oracle=Vec::new();for i in 0..2{for j in 0..2{if population&(1<<(i*2+j))!=0{oracle.push((i,j));bundles.push(CandidateIncidenceBundle::read(&p,&c,&a,&[staff(i),proj(j)]).unwrap());}}}for i in 0..2{for j in 0..2{let found=oracle.iter().any(|pair|*pair==(i,j));for (coverage,absent) in [(SimulatedIncidenceCoverage::Complete,Truth::False),(SimulatedIncidenceCoverage::Incomplete,Truth::Unknown)]{assert_eq!(simulate_incidence_exists(&p,&c,&a,&bundles,&[staff(i),proj(j)],coverage).unwrap(),if found{Truth::True}else{absent});}}}}
 }
 #[test]
 fn matching_first_bundle_cannot_hide_later_source_substitution(){
  let (c,p,a,t)=fixture(false,false);let project=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"Project".into()};let sv=[json!({"integerToken":"1"})];let pv=[json!({"string":"p1"})];let inputs=||[CandidateIncidenceInput{role:"staff",side:CandidateSide::Source,target:&t,key_id:"pk",components:&sv},CandidateIncidenceInput{role:"project",side:CandidateSide::Target,target:&project,key_id:"pk",components:&pv}];
  let mut policy=p.source().policy().clone();policy["revision"]=json!("other-source");let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),p.source().ontology_json(),&c).unwrap();let other=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let valid=CandidateIncidenceBundle::read(&p,&c,&a,&inputs()).unwrap();let substituted=CandidateIncidenceBundle::read(&other,&c,&a,&inputs()).unwrap();assert!(simulate_incidence_exists(&p,&c,&a,&[valid,substituted],&inputs(),SimulatedIncidenceCoverage::Complete).is_err());
 }

 #[test]
 fn original_graph_ir_preserves_nested_outer_witness_correlation(){
  use crate::security_composition::Truth;let (c,baseline,a,t)=fixture(false,false);let mut policy=baseline.source().policy().clone();let association=serde_json::to_value(&a).unwrap();let term=|name:&str,role:&str|json!({"kind":"variable","name":name,"endpoint":role});
  policy["rules"][0]["condition"]=json!({"op":"exists","association":association,"as":"outer","where":{"op":"exists","association":association,"as":"inner","where":{"op":"and","args":[{"op":"eq","left":term("outer","staff"),"right":term("inner","staff")},{"op":"not","arg":{"op":"eq","left":term("outer","project"),"right":term("inner","project")}}]}}});
  let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),baseline.source().ontology_json(),&c).unwrap();let p=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let project=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"Project".into()};let s1=[json!({"integerToken":"1"})];let s2=[json!({"integerToken":"2"})];let p1=[json!({"string":"p1"})];let p2=[json!({"string":"p2"})];let bundle=|sv:&[Value],pv:&[Value]|CandidateIncidenceBundle::read(&p,&c,&a,&[CandidateIncidenceInput{role:"staff",side:CandidateSide::Source,target:&t,key_id:"pk",components:sv},CandidateIncidenceInput{role:"project",side:CandidateSide::Target,target:&project,key_id:"pk",components:pv}]).unwrap();
  let correlated=[bundle(&s1,&p1),bundle(&s1,&p2)];let unrelated=[bundle(&s1,&p1),bundle(&s2,&p2)];let run=|bundles,coverage|simulate_graph_rule(&p,&c,0,&[CandidateGraphPopulation{association:&a,bundles,coverage}]);
  assert_eq!(run(&correlated,SimulatedIncidenceCoverage::Complete).unwrap(),Truth::True);assert_eq!(run(&unrelated,SimulatedIncidenceCoverage::Complete).unwrap(),Truth::False);assert_eq!(run(&unrelated,SimulatedIncidenceCoverage::Incomplete).unwrap(),Truth::Unknown);assert_eq!(run(&[],SimulatedIncidenceCoverage::Complete).unwrap(),Truth::False);assert_eq!(run(&[],SimulatedIncidenceCoverage::Incomplete).unwrap(),Truth::Unknown);assert!(simulate_graph_rule(&p,&c,0,&[]).is_err());let unknown=SecurityAssociationRef::Relationship(SecurityRelationshipRef{document_id:"domain".into(),module_id:"m".into(),relationship_id:"Unknown".into()});assert!(simulate_graph_rule(&p,&c,0,&[CandidateGraphPopulation{association:&unknown,bundles:&[],coverage:SimulatedIncidenceCoverage::Complete}]).is_err());
  let mut intrinsic=policy.clone();intrinsic["rules"][0]["condition"]=json!({"op":"eq","left":{"kind":"resource","identity":true},"right":{"kind":"resource","identity":true}});let packet=SecurityCandidateSourcePacket::read(&intrinsic.to_string(),baseline.source().ontology_json(),&c).unwrap();let intrinsic=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();assert!(simulate_graph_rule(&intrinsic,&c,0,&[]).is_err());
 }

 #[test]
 fn empty_unused_populations_require_declared_supported_relationships(){
  use crate::security_composition::Truth;let (c,p,a,_)=fixture(false,false);let run=|association|simulate_graph_rule(&p,&c,0,&[CandidateGraphPopulation{association,bundles:&[],coverage:SimulatedIncidenceCoverage::Complete}]);assert_eq!(run(&a).unwrap(),Truth::True);
  let unknown=SecurityAssociationRef::Relationship(SecurityRelationshipRef{document_id:"domain".into(),module_id:"m".into(),relationship_id:"Unknown".into()});assert!(run(&unknown).is_err());let record=SecurityAssociationRef::Record(SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"WorksOn".into()});assert!(run(&record).is_err());
 }

 #[test]
 fn empty_graph_quantifier_cannot_hide_unsupported_nested_profiles(){
  let (c,baseline,a,_)=fixture(false,false);let association=serde_json::to_value(&a).unwrap();let intrinsic=json!({"op":"eq","left":{"kind":"resource","identity":true},"right":{"kind":"resource","identity":true}});let raw=json!({"op":"exists","association":{"documentId":"domain","moduleId":"m","elementId":"Ownership"},"as":"raw","where":{"op":"literal","value":true}});
  for condition in [intrinsic,raw]{let mut policy=baseline.source().policy().clone();policy["rules"][0]["condition"]=json!({"op":"exists","association":association,"as":"outer","where":condition});let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),baseline.source().ontology_json(),&c).unwrap();let plan=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();assert!(simulate_graph_rule(&plan,&c,0,&[CandidateGraphPopulation{association:&a,bundles:&[],coverage:SimulatedIncidenceCoverage::Complete}]).is_err());}
 }

 #[test]
 fn graph_ir_budgets_are_charged_after_a_first_matching_witness(){
  use crate::security_composition::Truth;let (c,baseline,a,t)=fixture(false,false);let mut policy=baseline.source().policy().clone();let term=json!({"kind":"variable","name":"edge","endpoint":"staff"});policy["rules"][0]["condition"]=json!({"op":"exists","association":serde_json::to_value(&a).unwrap(),"as":"edge","where":{"op":"eq","left":term,"right":term}});let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),baseline.source().ontology_json(),&c).unwrap();let p=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let project=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"Project".into()};let sv=[json!({"integerToken":"1"})];let pv=[json!({"string":"p1"})];let bundle=||CandidateIncidenceBundle::read(&p,&c,&a,&[CandidateIncidenceInput{role:"staff",side:CandidateSide::Source,target:&t,key_id:"pk",components:&sv},CandidateIncidenceInput{role:"project",side:CandidateSide::Target,target:&project,key_id:"pk",components:&pv}]).unwrap();let bundles=[bundle(),bundle()];let populations=[CandidateGraphPopulation{association:&a,bundles:&bundles,coverage:SimulatedIncidenceCoverage::Complete}];assert!(simulate_graph_rule_bounded(&p,&c,0,&populations,2,4).is_err());assert!(simulate_graph_rule_bounded(&p,&c,0,&populations,3,3).is_err());assert_eq!(simulate_graph_rule_bounded(&p,&c,0,&populations,3,4).unwrap(),Truth::True);
 }

 #[test]
 fn graph_rule_matches_subject_and_resource_exact_logical_identities(){
  use crate::security_composition::Truth;let (c,baseline,a,staff)=fixture(false,false);let project=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"Project".into()};let mut policy=baseline.source().policy().clone();policy["rules"][0]["target"]=json!([serde_json::to_value(&project).unwrap()]);policy["rules"][0]["disclosure"]=json!([]);policy["rules"][0]["condition"]=json!({"op":"exists","association":serde_json::to_value(&a).unwrap(),"as":"edge","where":{"op":"and","args":[{"op":"eq","left":{"kind":"variable","name":"edge","endpoint":"staff"},"right":{"kind":"subject","identity":true}},{"op":"eq","left":{"kind":"variable","name":"edge","endpoint":"project"},"right":{"kind":"resource","identity":true}}]}});let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),baseline.source().ontology_json(),&c).unwrap();let p=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let s1=[json!({"integerToken":"1"})];let s2=[json!({"integerToken":"2"})];let p1=[json!({"string":"p1"})];let p2=[json!({"string":"p2"})];let subject=CandidateEntityIdentity::read(&p,&c,&staff,"pk",&s1).unwrap();let resource=CandidateEntityIdentity::read(&p,&c,&project,"pk",&p1).unwrap();let bundle=|sv:&[Value],pv:&[Value]|CandidateIncidenceBundle::read(&p,&c,&a,&[CandidateIncidenceInput{role:"staff",side:CandidateSide::Source,target:&staff,key_id:"pk",components:sv},CandidateIncidenceInput{role:"project",side:CandidateSide::Target,target:&project,key_id:"pk",components:pv}]).unwrap();let split=[bundle(&s1,&p2),bundle(&s2,&p1)];let joined=[bundle(&s1,&p1)];let run=|bundles,subject,resource|simulate_graph_rule_with_identities(&p,&c,0,&[CandidateGraphPopulation{association:&a,bundles,coverage:SimulatedIncidenceCoverage::Complete}],subject,resource);
  assert_eq!(run(&split,Some(&subject),Some(&resource)).unwrap(),Truth::False);assert_eq!(run(&joined,Some(&subject),Some(&resource)).unwrap(),Truth::True);assert!(run(&[],None,Some(&resource)).is_err());assert!(run(&[],Some(&subject),None).is_err());assert!(run(&joined,Some(&resource),Some(&resource)).is_err());assert!(run(&joined,Some(&subject),Some(&subject)).is_err());
  let other_subject=CandidateEntityIdentity::read(&baseline,&c,&staff,"pk",&s1).unwrap();assert!(run(&joined,Some(&other_subject),Some(&resource)).is_err());assert!(CandidateEntityIdentity::read(&p,&c,&staff,"other",&s1).is_err());assert!(CandidateEntityIdentity::read(&p,&c,&staff,"pk",&[]).is_err());assert!(CandidateEntityIdentity::read(&p,&c,&staff,"pk",&[json!({"string":"1"})]).is_err());
 }

 #[test]
 fn entity_identity_preserves_key_order_exact_numbers_and_domains(){
  let (c,p,_,t)=fixture(false,true);let one=CandidateEntityIdentity::read(&p,&c,&t,"pk",&[json!({"integerToken":"9007199254740993"}),json!({"integerToken":"2"})]).unwrap();let other=CandidateEntityIdentity::read(&p,&c,&t,"pk",&[json!({"integerToken":"9007199254740994"}),json!({"integerToken":"2"})]).unwrap();let reversed=CandidateEntityIdentity::read(&p,&c,&t,"pk",&[json!({"integerToken":"2"}),json!({"integerToken":"9007199254740993"})]).unwrap();assert_ne!(one.components,other.components);assert_ne!(one.components,reversed.components);assert!(one.require_plan(&p,&c).is_ok());
  let (ic,ip,_,it)=fixture(false,false);let (dc,dp,_,dt)=fixture(true,false);let integer=CandidateEntityIdentity::read(&ip,&ic,&it,"pk",&[json!({"integerToken":"1"})]).unwrap();let decimal=CandidateEntityIdentity::read(&dp,&dc,&dt,"pk",&[json!({"decimalToken":"0.1"})]).unwrap();assert_eq!(integer.components,decimal.components);assert_ne!(integer.domains,decimal.domains);assert!(integer.require_plan(&dp,&dc).is_err());assert!(decimal.require_plan(&ip,&ic).is_err());
 }

 #[test]
 fn identity_payload_budgets_and_unused_binding_checks_are_isolated(){
  use crate::security_composition::Truth;let (c,p,_,staff)=fixture(false,true);let values=[json!({"integerToken":"1"}),json!({"integerToken":"2"})];assert!(CandidateEntityIdentity::read_bounded(&p,&c,&staff,"pk",&values,1).is_err());assert!(CandidateEntityIdentity::read_bounded(&p,&c,&staff,"pk",&values,2).is_ok());
  let (c,p,_,staff)=fixture(false,false);let exponent=[json!({"integerToken":"1e1000"})];assert!(CandidateEntityIdentity::read_bounded(&p,&c,&staff,"pk",&exponent,10).is_err());assert!(CandidateEntityIdentity::read(&p,&c,&staff,"pk",&exponent).is_ok());let values=[json!({"integerToken":"1"})];let subject=CandidateEntityIdentity::read(&p,&c,&staff,"pk",&values).unwrap();assert_eq!(simulate_graph_rule_with_identities(&p,&c,0,&[],Some(&subject),None).unwrap(),Truth::True);
  let project=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"Project".into()};let wrong=CandidateEntityIdentity::read(&p,&c,&project,"pk",&[json!({"string":"p1"})]).unwrap();assert!(simulate_graph_rule_with_identities(&p,&c,0,&[],Some(&wrong),None).is_err());assert!(simulate_graph_rule_with_identities(&p,&c,0,&[],None,Some(&wrong)).is_err());
  let mut policy=p.source().policy().clone();policy["revision"]=json!("stale-policy");let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),p.source().ontology_json(),&c).unwrap();let other=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let stale=CandidateEntityIdentity::read(&other,&c,&staff,"pk",&values).unwrap();assert!(simulate_graph_rule_with_identities(&p,&c,0,&[],Some(&stale),None).is_err());
 }

 #[test]
 fn authored_abac_graph_rule_uses_exact_resource_fields_and_subject_identity(){
  use crate::security_composition::Truth;let (c,baseline,a,staff)=fixture(false,false);let resource_type=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"Resource".into()};let salary=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"salary".into()};let staff_id=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"staffId".into()};let project=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"Project".into()};let mut policy=baseline.source().policy().clone();policy["rules"][0]["condition"]=json!({"op":"exists","association":serde_json::to_value(&a).unwrap(),"as":"edge","where":{"op":"and","args":[{"op":"eq","left":{"kind":"variable","name":"edge","endpoint":"staff"},"right":{"kind":"subject","identity":true}},{"op":"eq","left":{"kind":"resource","field":serde_json::to_value(&salary).unwrap()},"right":{"kind":"constant","field":serde_json::to_value(&salary).unwrap(),"value":{"integerToken":"9007199254740993"}}}]}});policy["rules"][0]["condition"]["where"]["args"].as_array_mut().unwrap().push(json!({"op":"eq","left":{"kind":"subject","field":serde_json::to_value(&staff_id).unwrap()},"right":{"kind":"constant","field":serde_json::to_value(&staff_id).unwrap(),"value":{"integerToken":"1"}}}));let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),baseline.source().ontology_json(),&c).unwrap();let p=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let sv=[json!({"integerToken":"1"})];let rv=[json!({"string":"r1"})];let subject=CandidateEntityFact::read(&p,&c,&staff,"pk",&sv,&[(staff_id.clone(),sv[0].clone())]).unwrap();let resource=|value|CandidateEntityFact::read(&p,&c,&resource_type,"pk",&rv,&[(salary.clone(),json!({"integerToken":value}))]).unwrap();let good=resource("9007199254740993");let bad=resource("9007199254740994");let missing=CandidateEntityFact::read(&p,&c,&resource_type,"pk",&rv,&[]).unwrap();let bundles=[CandidateIncidenceBundle::read(&p,&c,&a,&[CandidateIncidenceInput{role:"staff",side:CandidateSide::Source,target:&staff,key_id:"pk",components:&sv},CandidateIncidenceInput{role:"project",side:CandidateSide::Target,target:&project,key_id:"pk",components:&[json!({"string":"p1"})]}]).unwrap()];let populations=[CandidateGraphPopulation{association:&a,bundles:&bundles,coverage:SimulatedIncidenceCoverage::Complete}];assert_eq!(simulate_graph_rule_with_facts(&p,&c,0,&populations,Some(&subject),Some(&good)).unwrap(),Truth::True);assert_eq!(simulate_graph_rule_with_facts(&p,&c,0,&populations,Some(&subject),Some(&bad)).unwrap(),Truth::False);assert!(simulate_graph_rule_with_facts(&p,&c,0,&[CandidateGraphPopulation{association:&a,bundles:&[],coverage:SimulatedIncidenceCoverage::Complete}],Some(&subject),Some(&missing)).is_err());
  let missing_subject=CandidateEntityFact::read(&p,&c,&staff,"pk",&sv,&[]).unwrap();assert!(simulate_graph_rule_with_facts(&p,&c,0,&[CandidateGraphPopulation{association:&a,bundles:&[],coverage:SimulatedIncidenceCoverage::Complete}],Some(&missing_subject),Some(&good)).is_err());

  let mut mismatch_policy=policy.clone();mismatch_policy["rules"][0]["condition"]["where"]["args"][2]["right"]["value"]=json!({"integerToken":"2"});let packet=SecurityCandidateSourcePacket::read(&mismatch_policy.to_string(),p.source().ontology_json(),&c).unwrap();let mismatch=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let mismatch_subject=CandidateEntityFact::read(&mismatch,&c,&staff,"pk",&sv,&[(staff_id.clone(),sv[0].clone())]).unwrap();let mismatch_resource=CandidateEntityFact::read(&mismatch,&c,&resource_type,"pk",&rv,&[(salary.clone(),json!({"integerToken":"9007199254740993"}))]).unwrap();let mismatch_bundles=[CandidateIncidenceBundle::read(&mismatch,&c,&a,&[CandidateIncidenceInput{role:"staff",side:CandidateSide::Source,target:&staff,key_id:"pk",components:&sv},CandidateIncidenceInput{role:"project",side:CandidateSide::Target,target:&project,key_id:"pk",components:&[json!({"string":"p1"})]}]).unwrap()];assert_eq!(simulate_graph_rule_with_facts(&mismatch,&c,0,&[CandidateGraphPopulation{association:&a,bundles:&mismatch_bundles,coverage:SimulatedIncidenceCoverage::Complete}],Some(&mismatch_subject),Some(&mismatch_resource)).unwrap(),Truth::False);
  let bindings=GraphIdentities{subject:Some(&subject.identity),resource:Some(&good.identity),subject_fact:Some(&subject),resource_fact:Some(&good),context:None,records:None,graph_records:None};assert!(simulate_graph_rule_with_identities_bounded(&p,&c,0,&populations,bindings,5,17).is_err());assert_eq!(simulate_graph_rule_with_identities_bounded(&p,&c,0,&populations,bindings,5,18).unwrap(),Truth::True);
  assert!(CandidateEntityFact::read(&p,&c,&staff,"pk",&sv,&[(salary.clone(),json!({"integerToken":"1"}))]).is_err());assert!(CandidateEntityFact::read(&p,&c,&staff,"pk",&sv,&[(staff_id.clone(),json!({"integerToken":"2"}))]).is_err());assert!(CandidateEntityFact::read(&p,&c,&staff,"pk",&sv,&[(staff_id.clone(),sv[0].clone()),(staff_id.clone(),sv[0].clone())]).is_err());assert!(CandidateEntityFact::read_bounded(&p,&c,&resource_type,"pk",&rv,&[(salary.clone(),json!({"integerToken":"9007199254740993"}))],6).is_err());assert!(CandidateEntityFact::read_bounded(&p,&c,&resource_type,"pk",&rv,&[(salary,json!({"integerToken":"9007199254740993"}))],7).is_ok());
 }

 #[test]
 fn fact_budget_accumulates_across_coherent_fields_in_both_orders(){
  let (c,p,_,_)=fixture(false,false);let target=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"Resource".into()};let resource_id=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"resourceId".into()};let salary=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"salary".into()};let key=[json!({"string":"r1"})];for fields in [[(resource_id.clone(),key[0].clone()),(salary.clone(),json!({"integerToken":"9007199254740993"}))],[(salary.clone(),json!({"integerToken":"9007199254740993"})),(resource_id.clone(),key[0].clone())]]{assert!(CandidateEntityFact::read_bounded(&p,&c,&target,"pk",&key,&fields,8).is_err());assert!(CandidateEntityFact::read_bounded(&p,&c,&target,"pk",&key,&fields,9).is_ok());}
 }

 #[test]
 fn stored_and_context_same_field_are_independent_channels(){
  use crate::security_composition::Truth;let (c,baseline,a,_)=fixture(false,false);let target=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"Resource".into()};let salary=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"salary".into()};let resource_id=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"resourceId".into()};let mut ontology=baseline.source().ontology().clone();ontology["context"]=json!([serde_json::to_value(&salary).unwrap(),serde_json::to_value(&resource_id).unwrap()]);let mut policy=baseline.source().policy().clone();let equality=|kind:&str,token:&str|json!({"op":"eq","left":{"kind":kind,"field":serde_json::to_value(&salary).unwrap()},"right":{"kind":"constant","field":serde_json::to_value(&salary).unwrap(),"value":{"integerToken":token}}});policy["rules"][0]["condition"]=json!({"op":"and","args":[equality("resource","1"),equality("context","2")]});let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&ontology.to_string(),&c).unwrap();let p=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let key=[json!({"string":"r1"})];let fact=|v:&str|CandidateEntityFact::read(&p,&c,&target,"pk",&key,&[(salary.clone(),json!({"integerToken":v}))]).unwrap();let context=|v:&str|CandidateContextValues::read(&p,&c,&[(salary.clone(),json!({"integerToken":v}))]).unwrap();let stored_one=fact("1");let stored_two=fact("2");let context_one=context("1");let context_two=context("2");let run=|stored,context|simulate_graph_rule_with_context(&p,&c,0,&[],None,Some(stored),Some(context));assert_eq!(run(&stored_one,&context_two).unwrap(),Truth::True);assert_eq!(run(&stored_two,&context_two).unwrap(),Truth::False);assert_eq!(run(&stored_one,&context_one).unwrap(),Truth::False);assert!(simulate_graph_rule_with_facts(&p,&c,0,&[],None,Some(&stored_one)).is_err());
  let bindings=GraphIdentities{subject:None,resource:Some(&stored_one.identity),subject_fact:None,resource_fact:Some(&stored_one),context:Some(&context_two),records:None,graph_records:None};assert!(simulate_graph_rule_with_identities_bounded(&p,&c,0,&[],bindings,3,3).is_err());assert_eq!(simulate_graph_rule_with_identities_bounded(&p,&c,0,&[],bindings,3,4).unwrap(),Truth::True);
  let missing=CandidateContextValues::read(&p,&c,&[]).unwrap();let mut nested_policy=policy.clone();nested_policy["rules"][0]["condition"]=json!({"op":"exists","association":serde_json::to_value(&a).unwrap(),"as":"edge","where":policy["rules"][0]["condition"]});let packet=SecurityCandidateSourcePacket::read(&nested_policy.to_string(),&ontology.to_string(),&c).unwrap();let nested=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let nested_fact=CandidateEntityFact::read(&nested,&c,&target,"pk",&key,&[(salary.clone(),json!({"integerToken":"1"}))]).unwrap();let missing_nested=CandidateContextValues::read(&nested,&c,&[]).unwrap();assert!(simulate_graph_rule_with_context(&nested,&c,0,&[CandidateGraphPopulation{association:&a,bundles:&[],coverage:SimulatedIncidenceCoverage::Complete}],None,Some(&nested_fact),Some(&missing_nested)).is_err());assert!(run(&stored_one,&missing).is_err());
  let fields=[(salary.clone(),json!({"integerToken":"2"})),(resource_id.clone(),json!({"string":"r1"}))];for fields in [fields.clone(),[fields[1].clone(),fields[0].clone()]]{assert!(CandidateContextValues::read_bounded(&p,&c,&fields,2).is_err());assert!(CandidateContextValues::read_bounded(&p,&c,&fields,3).is_ok());}assert!(CandidateContextValues::read(&p,&c,&[(salary.clone(),json!({"integerToken":"2"})),(salary.clone(),json!({"integerToken":"2"}))]).is_err());let foreign=SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:"staffId".into()};assert!(CandidateContextValues::read(&p,&c,&[(foreign,json!({"integerToken":"1"}))]).is_err());assert!(CandidateContextValues::read_bounded(&p,&c,&[(salary.clone(),json!({"integerToken":"1e1000"}))],10).is_err());assert!(CandidateContextValues::read(&p,&c,&[(salary.clone(),json!({"integerToken":"1e1000"}))]).is_ok());
  let mut literal_policy=policy.clone();literal_policy["rules"][0]["condition"]=json!({"op":"literal","value":true});let packet=SecurityCandidateSourcePacket::read(&literal_policy.to_string(),&ontology.to_string(),&c).unwrap();let literal=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let valid_unused=CandidateContextValues::read(&literal,&c,&[(salary,json!({"integerToken":"2"}))]).unwrap();assert_eq!(simulate_graph_rule_with_context(&literal,&c,0,&[],None,None,Some(&valid_unused)).unwrap(),Truth::True);assert!(simulate_graph_rule_with_context(&literal,&c,0,&[],None,None,Some(&context_two)).is_err());
 }

 #[test]
 fn raw_witness_projects_endpoints_from_one_source_bound_record(){
  let (c,p,_,_)=fixture(false,false);let reference=|id:&str|SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:id.into()};let owner=reference("Ownership");let association=SecurityAssociationRef::Record(owner.clone());let make=|fields:&[(SecurityRef,Value)]|CandidateEntityFact::read(&p,&c,&owner,"pk",&[json!({"string":"o1"})],fields).unwrap();let fields=[(reference("ownerResource"),json!({"string":"r1"})),(reference("ownerProject"),json!({"string":"p1"}))];let witness=CandidateRecordWitness::read(&p,&c,&association,make(&fields)).unwrap();assert_eq!(witness.association(),&association);assert!(witness.require_plan(&p,&c).is_ok());let resource=CandidateEntityIdentity::read(&p,&c,&reference("Resource"),"pk",&[json!({"string":"r1"})]).unwrap();let project=CandidateEntityIdentity::read(&p,&c,&reference("Project"),"pk",&[json!({"string":"p1"})]).unwrap();assert_eq!(witness.endpoint("resource").unwrap().target,resource.target);assert_eq!(witness.endpoint("resource").unwrap().components,resource.components);assert_eq!(witness.endpoint("project").unwrap().target,project.target);assert_eq!(witness.endpoint("project").unwrap().components,project.components);assert_eq!(witness.identity().target,owner);assert_ne!(witness.identity().components,witness.endpoint("resource").unwrap().components);assert!(witness.endpoint("missing").is_none());
  assert!(CandidateRecordWitness::read(&p,&c,&association,make(&fields[..1])).is_err());let wrong_owner=CandidateEntityFact::read(&p,&c,&reference("Resource"),"pk",&[json!({"string":"r1"})],&[]).unwrap();assert!(CandidateRecordWitness::read(&p,&c,&association,wrong_owner).is_err());let graph=SecurityAssociationRef::Relationship(SecurityRelationshipRef{document_id:"domain".into(),module_id:"m".into(),relationship_id:"Ownership".into()});assert!(CandidateRecordWitness::read(&p,&c,&graph,make(&fields)).is_err());assert!(CandidateRecordWitness::read_bounded(&p,&c,&association,make(&fields),3).is_err());assert!(CandidateRecordWitness::read_bounded(&p,&c,&association,make(&fields),4).is_ok());
 }

 #[test]
 fn raw_composite_endpoint_retains_order_domain_and_source_custody(){
  let (baseline_catalog,baseline,_,_)=fixture(false,false);let reference=|id:&str|SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:id.into()};let mut input=baseline_catalog.inputs()[0].clone();let mut doc:Value=serde_json::from_str(&input.document_json).unwrap();let elements=doc["modules"][0]["elements"].as_array_mut().unwrap();elements.iter_mut().find(|e|e["id"]=="Resource").unwrap()["keys"][0]["fields"].as_array_mut().unwrap().push(json!({"module":"m","element":"salary"}));elements.iter_mut().find(|e|e["id"]=="Ownership").unwrap()["members"].as_array_mut().unwrap().push(json!({"module":"m","element":"ownerSalary"}));elements.push(json!({"id":"ownerSalary","kind":"field","scalarType":"integer","nullability":"required","cardinality":"one","extensions":{}}));input.document_json=doc.to_string();input.pin.sha256=crate::json::sha256(input.document_json.as_bytes());input.pin.revision="schema-composite2".into();let c=Catalog::prepare_security(vec![input]).unwrap();let mut ontology=baseline.source().ontology().clone();ontology["documents"][0]["revision"]=json!("schema-composite2");ontology["revision"]=json!("ontology-composite2");ontology["entities"][3]["fields"].as_array_mut().unwrap().push(json!({"ref":serde_json::to_value(reference("ownerSalary")).unwrap(),"protection":"unprotected"}));ontology["associations"][0]["endpoints"][0]["fields"].as_array_mut().unwrap().push(serde_json::to_value(reference("ownerSalary")).unwrap());let mut policy=baseline.source().policy().clone();policy["revision"]=json!("policy-composite2");policy["ontology"]["revision"]=json!("ontology-composite2");let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&ontology.to_string(),&c).unwrap();let p=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let association=SecurityAssociationRef::Record(reference("Ownership"));let fact=CandidateEntityFact::read(&p,&c,&reference("Ownership"),"pk",&[json!({"string":"o1"})],&[(reference("ownerResource"),json!({"string":"r1"})),(reference("ownerSalary"),json!({"integerToken":"9007199254740993"})),(reference("ownerProject"),json!({"string":"p1"}))]).unwrap();let witness=CandidateRecordWitness::read(&p,&c,&association,fact).unwrap();let expected=CandidateEntityIdentity::read(&p,&c,&reference("Resource"),"pk",&[json!({"string":"r1"}),json!({"integerToken":"9007199254740993"})]).unwrap();let actual=witness.endpoint("resource").unwrap();assert_eq!(actual.target(),expected.target());assert_eq!(actual.key_id(),expected.key_id());assert_eq!(actual.components(),expected.components());assert_eq!(actual.component_domains(),expected.component_domains());
  let mut reversed=ontology.clone();reversed["associations"][0]["endpoints"][0]["fields"].as_array_mut().unwrap().swap(0,1);let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&reversed.to_string(),&c).unwrap();assert!(SecurityCandidateLogicalPlan::read(packet,&c).is_err());
  let mut other_policy=policy.clone();other_policy["revision"]=json!("other-policy");let packet=SecurityCandidateSourcePacket::read(&other_policy.to_string(),&ontology.to_string(),&c).unwrap();let other=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();assert!(witness.require_plan(&other,&c).is_err());let mut other_ontology=ontology.clone();other_ontology["associations"][0]["endpoints"][0]["role"]=json!("other-resource-role");let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&other_ontology.to_string(),&c).unwrap();let other=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();assert!(witness.require_plan(&other,&c).is_err());assert!(witness.require_plan(&p,&c).is_ok());
 }

 #[test]
 fn mixed_raw_ownership_and_graph_membership_keep_project_correlation(){
  use crate::security_composition::Truth;let (c,baseline,graph,staff)=fixture(false,false);let reference=|id:&str|SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:id.into()};let raw=SecurityAssociationRef::Record(reference("Ownership"));let term=|name:&str,role:&str|json!({"kind":"variable","name":name,"endpoint":role});let eq=|left:Value,right:Value|json!({"op":"eq","left":left,"right":right});let owner_field=json!({"kind":"variable","name":"owner","field":serde_json::to_value(reference("ownerId")).unwrap()});let mut policy=baseline.source().policy().clone();policy["rules"][0]["condition"]=json!({"op":"exists","association":serde_json::to_value(&raw).unwrap(),"as":"owner","where":{"op":"and","args":[eq(term("owner","resource"),json!({"kind":"resource","identity":true})),eq(owner_field,json!({"kind":"constant","field":serde_json::to_value(reference("ownerId")).unwrap(),"value":{"string":"o1"}})),eq(json!({"kind":"variable","name":"owner","identity":true}),json!({"kind":"variable","name":"owner","identity":true})),{"op":"exists","association":serde_json::to_value(&graph).unwrap(),"as":"assignment","where":{"op":"and","args":[eq(term("assignment","staff"),json!({"kind":"subject","identity":true})),eq(term("assignment","project"),term("owner","project"))]}}]}});let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),baseline.source().ontology_json(),&c).unwrap();let p=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let subject=CandidateEntityFact::read(&p,&c,&staff,"pk",&[json!({"integerToken":"1"})],&[]).unwrap();let resource=CandidateEntityFact::read(&p,&c,&reference("Resource"),"pk",&[json!({"string":"r1"})],&[]).unwrap();let row=|id:&str,r:&str,project:&str,include_id:bool|{let mut fields=vec![(reference("ownerResource"),json!({"string":r})),(reference("ownerProject"),json!({"string":project}))];if include_id{fields.push((reference("ownerId"),json!({"string":id})));}let fact=CandidateEntityFact::read(&p,&c,&reference("Ownership"),"pk",&[json!({"string":id})],&fields).unwrap();CandidateRecordWitness::read(&p,&c,&raw,fact).unwrap()};let edge=|staff_id:&str,project_id:&str|CandidateIncidenceBundle::read(&p,&c,&graph,&[CandidateIncidenceInput{role:"staff",side:CandidateSide::Source,target:&staff,key_id:"pk",components:&[json!({"integerToken":staff_id})]},CandidateIncidenceInput{role:"project",side:CandidateSide::Target,target:&reference("Project"),key_id:"pk",components:&[json!({"string":project_id})]}]).unwrap();let owners=[row("o1","r1","p1",true)];let joined=[edge("1","p1")];let split_edges=[edge("1","p2"),edge("2","p1")];let split_owners=[row("o1","r1","p2",true),row("o2","r2","p1",true)];let run=|owners,edges,coverage|simulate_candidate_rule(&p,&c,0,&[CandidateGraphPopulation{association:&graph,bundles:edges,coverage}],&[CandidateRecordPopulation{association:&raw,witnesses:owners,coverage:SimulatedIncidenceCoverage::Complete}],Some(&subject),Some(&resource),None);
  assert_eq!(run(&owners,&joined,SimulatedIncidenceCoverage::Complete).unwrap(),Truth::True);assert_eq!(run(&owners,&split_edges,SimulatedIncidenceCoverage::Complete).unwrap(),Truth::False);assert_eq!(run(&split_owners,&joined,SimulatedIncidenceCoverage::Complete).unwrap(),Truth::False);assert_eq!(run(&owners,&[],SimulatedIncidenceCoverage::Incomplete).unwrap(),Truth::Unknown);assert_eq!(run(&[],&joined,SimulatedIncidenceCoverage::Complete).unwrap(),Truth::False);let missing_rows=[row("o1","r1","p1",false)];let duplicate_rows=[row("o1","r1","p1",true),row("o1","r1","p1",true)];assert!(run(&missing_rows,&[],SimulatedIncidenceCoverage::Complete).is_err());assert!(run(&duplicate_rows,&joined,SimulatedIncidenceCoverage::Complete).is_err());
 }

 #[test]
 fn raw_population_admission_budgets_identity_and_all_sources(){
  use crate::security_composition::Truth;
  let (c,baseline,_,_)=fixture(false,false);
  let reference=|id:&str|SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:id.into()};
  let raw=SecurityAssociationRef::Record(reference("Ownership"));
  let make=|plan:&SecurityCandidateLogicalPlan,id:&str|{
   let fact=CandidateEntityFact::read(plan,&c,&reference("Ownership"),"pk",&[json!({"string":id})],&[(reference("ownerResource"),json!({"string":"r1"})),(reference("ownerProject"),json!({"string":"p1"}))]).unwrap();
   CandidateRecordWitness::read(plan,&c,&raw,fact).unwrap()
  };
  let rows=[make(&baseline,"o1"),make(&baseline,"o2")];
  let populations=[CandidateRecordPopulation{association:&raw,witnesses:&rows,coverage:SimulatedIncidenceCoverage::Complete}];
  let bounded=|budget|simulate_graph_rule_with_identities_bounded(&baseline,&c,0,&[],GraphIdentities{subject:None,resource:None,subject_fact:None,resource_fact:None,context:None,records:Some(&populations),graph_records:None},1,budget);
  assert!(bounded(3).is_err());assert_eq!(bounded(4).unwrap(),Truth::True);
  let duplicates=[make(&baseline,"o1"),make(&baseline,"o1")];
  assert!(simulate_candidate_rule(&baseline,&c,0,&[],&[CandidateRecordPopulation{association:&raw,witnesses:&duplicates,coverage:SimulatedIncidenceCoverage::Complete}],None,None,None).is_err());
  let mut policy=baseline.source().policy().clone();policy["revision"]=json!("different-source");
  let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),baseline.source().ontology_json(),&c).unwrap();let other=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();
  let stale=[make(&baseline,"o1"),make(&other,"o2")];
  assert!(simulate_candidate_rule(&baseline,&c,0,&[],&[CandidateRecordPopulation{association:&raw,witnesses:&stale,coverage:SimulatedIncidenceCoverage::Complete}],None,None,None).is_err());
  let unknown=SecurityAssociationRef::Record(reference("unknown"));
  let wrong_namespace=SecurityAssociationRef::Relationship(SecurityRelationshipRef{document_id:"domain".into(),module_id:"m".into(),relationship_id:"Ownership".into()});
  for association in [&unknown,&wrong_namespace]{assert!(simulate_candidate_rule(&baseline,&c,0,&[],&[CandidateRecordPopulation{association,witnesses:&[],coverage:SimulatedIncidenceCoverage::Complete}],None,None,None).is_err());}
  let identity=|name:&str|json!({"kind":"variable","name":name,"identity":true});
  policy=baseline.source().policy().clone();
  policy["rules"][0]["condition"]=json!({"op":"exists","association":serde_json::to_value(&raw).unwrap(),"as":"outer","where":{"op":"exists","association":serde_json::to_value(&raw).unwrap(),"as":"inner","where":{"op":"not","arg":{"op":"eq","left":identity("outer"),"right":identity("inner")}}}});
  let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),baseline.source().ontology_json(),&c).unwrap();let p=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();
  let distinct=[make(&p,"o1"),make(&p,"o2")];
  assert_eq!(simulate_candidate_rule(&p,&c,0,&[],&[CandidateRecordPopulation{association:&raw,witnesses:&distinct,coverage:SimulatedIncidenceCoverage::Complete}],None,None,None).unwrap(),Truth::True);
  assert_eq!(simulate_candidate_rule(&p,&c,0,&[],&[CandidateRecordPopulation{association:&raw,witnesses:&distinct[..1],coverage:SimulatedIncidenceCoverage::Complete}],None,None,None).unwrap(),Truth::False);
 }

 #[test]
 fn record_backed_graph_fields_stay_with_their_edge_endpoints(){
  use crate::security_composition::Truth;
  let (baseline_catalog,baseline,graph,staff)=fixture(false,false);
  let reference=|id:&str|SecurityRef{document_id:"domain".into(),module_id:"m".into(),element_id:id.into()};
  let mut input=baseline_catalog.inputs()[0].clone();let mut doc:Value=serde_json::from_str(&input.document_json).unwrap();
  doc["modules"][0]["relationships"][0]["associationRecord"]=json!({"module":"m","element":"Assignment"});
  input.document_json=doc.to_string();input.pin.sha256=crate::json::sha256(input.document_json.as_bytes());input.pin.revision="record-edge-schema".into();let c=Catalog::prepare_security(vec![input]).unwrap();
  let mut ontology=baseline.source().ontology().clone();ontology["documents"][0]["revision"]=json!("record-edge-schema");ontology["revision"]=json!("record-edge-ontology");ontology["associations"][1]["witness"]=json!({"kind":"record-key","type":serde_json::to_value(reference("Assignment")).unwrap(),"keyId":"pk"});
  let mut policy=baseline.source().policy().clone();policy["ontology"]["revision"]=json!("record-edge-ontology");
  policy["rules"][0]["condition"]=json!({"op":"exists","association":serde_json::to_value(&graph).unwrap(),"as":"edge","where":{"op":"and","args":[{"op":"eq","left":{"kind":"variable","name":"edge","endpoint":"staff"},"right":{"kind":"subject","identity":true}},{"op":"eq","left":{"kind":"variable","name":"edge","field":serde_json::to_value(reference("active")).unwrap()},"right":{"kind":"constant","field":serde_json::to_value(reference("active")).unwrap(),"value":{"boolean":true}}},{"op":"eq","left":{"kind":"variable","name":"edge","identity":true},"right":{"kind":"variable","name":"edge","identity":true}}]}});
  let packet=SecurityCandidateSourcePacket::read(&policy.to_string(),&ontology.to_string(),&c).unwrap();let p=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();
  let bundle=|plan:&SecurityCandidateLogicalPlan,id:&str|CandidateIncidenceBundle::read(plan,&c,&graph,&[CandidateIncidenceInput{role:"staff",side:CandidateSide::Source,target:&staff,key_id:"pk",components:&[json!({"integerToken":id})]},CandidateIncidenceInput{role:"project",side:CandidateSide::Target,target:&reference("Project"),key_id:"pk",components:&[json!({"string":"p1"})]}]).unwrap();
  let fact=|plan:&SecurityCandidateLogicalPlan,id:&str,active:Option<bool>|CandidateEntityFact::read(plan,&c,&reference("Assignment"),"pk",&[json!({"string":id})],&active.map(|v|vec![(reference("active"),json!({"boolean":v}))]).unwrap_or_default()).unwrap();
  let row=|plan:&SecurityCandidateLogicalPlan,id:&str,staff_id:&str,active:Option<bool>|CandidateGraphRecordWitness::read(plan,&c,bundle(plan,staff_id),fact(plan,id,active)).unwrap();
  let subject=CandidateEntityFact::read(&p,&c,&staff,"pk",&[json!({"integerToken":"1"})],&[]).unwrap();
  let joined=[row(&p,"e1","1",Some(true))];let split=[row(&p,"e1","1",Some(false)),row(&p,"e2","2",Some(true))];let missing=[row(&p,"e1","1",None)];let duplicates=[row(&p,"e1","1",Some(true)),row(&p,"e1","2",Some(false))];
  let run=|rows,coverage|simulate_candidate_rule_with_graph_records(&p,&c,0,&[],&[],&[CandidateGraphRecordPopulation{association:&graph,witnesses:rows,coverage}],Some(&subject),None,None);
  assert_eq!(run(&joined,SimulatedIncidenceCoverage::Complete).unwrap(),Truth::True);
  assert_eq!(run(&split,SimulatedIncidenceCoverage::Complete).unwrap(),Truth::False);
  assert_eq!(run(&[],SimulatedIncidenceCoverage::Complete).unwrap(),Truth::False);
  assert_eq!(run(&[],SimulatedIncidenceCoverage::Incomplete).unwrap(),Truth::Unknown);
  assert!(run(&missing,SimulatedIncidenceCoverage::Complete).is_err());assert!(run(&duplicates,SimulatedIncidenceCoverage::Complete).is_err());
  let wrong=CandidateEntityFact::read(&p,&c,&reference("Project"),"pk",&[json!({"string":"p1"})],&[]).unwrap();assert!(CandidateGraphRecordWitness::read(&p,&c,bundle(&p,"1"),wrong).is_err());
  let mut other_policy=policy.clone();other_policy["revision"]=json!("stale-record-edge-policy");let packet=SecurityCandidateSourcePacket::read(&other_policy.to_string(),&ontology.to_string(),&c).unwrap();let other=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();
  assert!(CandidateGraphRecordWitness::read(&p,&c,bundle(&p,"1"),fact(&other,"e2",Some(true))).is_err());
  let stale=[row(&p,"e1","1",Some(true)),row(&other,"e2","1",Some(true))];assert!(run(&stale,SimulatedIncidenceCoverage::Complete).is_err());
  assert!(simulate_graph_rule_with_facts(&p,&c,0,&[CandidateGraphPopulation{association:&graph,bundles:&[bundle(&p,"1")],coverage:SimulatedIncidenceCoverage::Complete}],Some(&subject),None).is_err());
  let mut literal_policy=policy.clone();literal_policy["rules"][0]["condition"]=json!({"op":"literal","value":true});let packet=SecurityCandidateSourcePacket::read(&literal_policy.to_string(),&ontology.to_string(),&c).unwrap();let literal=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();
  let literal_rows=[row(&literal,"e1","1",Some(true)),row(&literal,"e2","1",Some(true))];let populations=[CandidateGraphRecordPopulation{association:&graph,witnesses:&literal_rows,coverage:SimulatedIncidenceCoverage::Complete}];
  let bounded=|budget|simulate_graph_rule_with_identities_bounded(&literal,&c,0,&[],GraphIdentities{subject:None,resource:None,subject_fact:None,resource_fact:None,context:None,records:None,graph_records:Some(&populations)},1,budget);
  assert!(bounded(3).is_err());assert_eq!(bounded(4).unwrap(),Truth::True);
  let equal_rows=[row(&literal,"e1","1",Some(true)),row(&literal,"e1","1",Some(true))];assert!(simulate_candidate_rule_with_graph_records(&literal,&c,0,&[],&[],&[CandidateGraphRecordPopulation{association:&graph,witnesses:&equal_rows,coverage:SimulatedIncidenceCoverage::Complete}],None,None,None).is_err());
  let unknown=SecurityAssociationRef::Relationship(SecurityRelationshipRef{document_id:"domain".into(),module_id:"m".into(),relationship_id:"unknown".into()});let raw_namespace=SecurityAssociationRef::Record(reference("WorksOn"));
  for association in [&unknown,&raw_namespace]{assert!(simulate_candidate_rule_with_graph_records(&literal,&c,0,&[],&[],&[CandidateGraphRecordPopulation{association,witnesses:&[],coverage:SimulatedIncidenceCoverage::Complete}],None,None,None).is_err());}
  assert!(simulate_candidate_rule_with_graph_records(&baseline,&baseline_catalog,0,&[],&[],&[CandidateGraphRecordPopulation{association:&graph,witnesses:&[],coverage:SimulatedIncidenceCoverage::Complete}],None,None,None).is_err());
  let own_identity=|name:&str|json!({"kind":"variable","name":name,"identity":true});
  let mut identity_policy=policy.clone();identity_policy["rules"][0]["condition"]=json!({"op":"exists","association":serde_json::to_value(&graph).unwrap(),"as":"outer","where":{"op":"exists","association":serde_json::to_value(&graph).unwrap(),"as":"inner","where":{"op":"not","arg":{"op":"eq","left":own_identity("outer"),"right":own_identity("inner")}}}});
  let packet=SecurityCandidateSourcePacket::read(&identity_policy.to_string(),&ontology.to_string(),&c).unwrap();let identity_plan=SecurityCandidateLogicalPlan::read(packet,&c).unwrap();let identity_rows=[row(&identity_plan,"e1","1",Some(true)),row(&identity_plan,"e2","1",Some(true))];
  assert_eq!(simulate_candidate_rule_with_graph_records(&identity_plan,&c,0,&[],&[],&[CandidateGraphRecordPopulation{association:&graph,witnesses:&identity_rows,coverage:SimulatedIncidenceCoverage::Complete}],None,None,None).unwrap(),Truth::True);
  assert_eq!(simulate_candidate_rule_with_graph_records(&identity_plan,&c,0,&[],&[],&[CandidateGraphRecordPopulation{association:&graph,witnesses:&identity_rows[..1],coverage:SimulatedIncidenceCoverage::Complete}],None,None,None).unwrap(),Truth::False);

 }

}
