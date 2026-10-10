//! Direct typed payload dispatch for template0.2. No facet-value qualification,
//! physical capability selection, authenticated profile or native authorization.
use crate::{error::{Diagnostic,Result},security_obligation_sources::{OwnerSourceDemands,FieldChannel,KeyPayload,QueryPayload,FieldUse},security_requirements::SecurityOperatorMode,security_ir::QueryOperator};
use serde_json::Value;
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-LOWERING-UNSUPPORTED","capability","Typed payload applicability refused")}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) enum Scalar{Boolean,String,Integer,Decimal,Binary}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) enum Nullability{Required,AbsentAllowed,Unspecified}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) enum Role{Stored,Context,KeyMember,Projection,QueryField,Operator,FieldOperand,ContextOperand,ConstantOperand,TransformOutput}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) enum Facet{Length,Precision,Scale,IntegerWidth,Range,CollectionSize,AllowedValues}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) enum Output{Field,Count,Sum}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) enum Availability{Required,AbsentAllowed}
fn availability(value:Option<&str>)->Result<Availability>{match value{Some("required")=>Ok(Availability::Required),Some("absent-allowed")=>Ok(Availability::AbsentAllowed),_=>Err(fail())}}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) enum Selector{
 Role(Role),Scalar(Scalar),Nullability(Nullability),Facet(Facet),Protection(bool),
 KeyPrimary(Option<bool>),OrderedKeyMember,Operator(QueryOperator,bool),
 Output(Output),OutputScalar(Scalar),OutputNullable(bool),OutputAvailability(Availability),OutputFacet(Facet),
 AssociationEndpoint,AssociationEndpointMember,Literal(Scalar,bool),ConstantTransformV1,
}
pub(crate) trait Sink{
 fn charge(&mut self,bytes:usize)->Result<()>;
 fn duty(&mut self,source:&str,selector:Selector,address:&[&str])->Result<()>;
}
fn facet(name:&str)->Result<Facet>{Ok(match name{"length"=>Facet::Length,"precision"=>Facet::Precision,"scale"=>Facet::Scale,"integerWidth"=>Facet::IntegerWidth,"range"=>Facet::Range,"collectionSize"=>Facet::CollectionSize,_=>return Err(fail())})}
fn facets(source:&str,value:Option<&Value>,output:bool,sink:&mut impl Sink)->Result<()> {
 if let Some(value)=value{let object=value.as_object().ok_or_else(fail)?;if object.len()>6{return Err(fail());}
  for name in object.keys(){sink.charge(name.len())?;let f=facet(name)?;sink.duty(source,if output{Selector::OutputFacet(f)}else{Selector::Facet(f)},&["payload",if output{"output-facet"}else{"facet"},name])?;}
 }Ok(())
}
fn scalar(value:&Value)->Result<Scalar>{Ok(match value.as_str(){Some("boolean")=>Scalar::Boolean,Some("string")=>Scalar::String,Some("integer")=>Scalar::Integer,Some("decimal")=>Scalar::Decimal,Some("binary")=>Scalar::Binary,_=>return Err(fail())})}
fn domain(source:&str,field:&Value,role:Role,sink:&mut impl Sink)->Result<()> {
 sink.charge(0)?;if field["cardinality"]!="one"{return Err(fail());}
 let scalar=scalar(&field["scalarType"])?;let nullable=match field["nullability"].as_str(){Some("required")=>Nullability::Required,Some("absent-allowed")=>Nullability::AbsentAllowed,Some("unspecified")=>Nullability::Unspecified,_=>return Err(fail())};
 sink.duty(source,Selector::Role(role),&["payload","role"])?;
 sink.duty(source,Selector::Scalar(scalar),&["payload","scalar"])?;
 sink.duty(source,Selector::Nullability(nullable),&["payload","nullability"])?;
 facets(source,field.get("facets"),false,sink)?;
 if let Some(allowed)=field.get("allowedValues"){if !allowed.is_null(){if !allowed.is_array(){return Err(fail());}sink.duty(source,Selector::Facet(Facet::AllowedValues),&["payload","allowed-values"])?;}}
 Ok(())
}
fn protection(source:&str,classification:&Value,sink:&mut impl Sink)->Result<()> {
 sink.charge(0)?;let protected=match classification["protection"].as_str(){Some("protected")=>true,Some("unprotected")=>false,_=>return Err(fail())};
 sink.duty(source,Selector::Protection(protected),&["payload","protection"])
}
pub(crate) fn output(source:&str,selected:&crate::application_ir::Output,plan:&crate::application_ir::Plan,sink:&mut impl Sink)->Result<()> {
   use crate::{application_ir::Expression,ir::Family};
   let (kind,t)=match &selected.expression{Expression::Field{identity,..}=>{
    let mut selected=None;for d in &plan.type_graph{sink.charge(0)?;if d.identity==*identity{if selected.is_some(){return Err(fail());}selected=Some(d);}}
    let d=selected.ok_or_else(fail)?;let crate::application_model::Shape::Scalar{logical_type}=&d.shape else{return Err(fail());};
    sink.duty(source,Selector::OutputAvailability(availability(d.availability.as_deref())?),&["payload","output-availability"])?;
    (Output::Field,logical_type)
   },Expression::Count{logical_type}=>(Output::Count,logical_type),Expression::Sum{logical_type,..}=>(Output::Sum,logical_type),Expression::RelatedKeys{..}=>return Err(fail())};
   sink.duty(source,Selector::Output(kind),&["payload","output-kind"])?;
   let family=match t.family{Family::Boolean=>Scalar::Boolean,Family::String=>Scalar::String,Family::Integer=>Scalar::Integer,Family::Decimal=>Scalar::Decimal};sink.duty(source,Selector::OutputScalar(family),&["payload","output-scalar"])?;sink.duty(source,Selector::OutputNullable(t.nullable),&["payload","output-nullable"])?;facets(source,Some(&t.facets),true,sink)?;
 Ok(())
}

/// New selectors are reserved for the separate rule-payload template0.3 experiment.
pub(crate) fn rule_only(selector:Selector)->bool{matches!(selector,Selector::Role(Role::FieldOperand|Role::ContextOperand|Role::ConstantOperand|Role::TransformOutput)|Selector::Literal(..)|Selector::ConstantTransformV1)}
struct At<'a,S>{address:&'a str,sink:&'a mut S}
impl<S:Sink> Sink for At<'_,S>{
 fn charge(&mut self,bytes:usize)->Result<()>{self.sink.charge(bytes)}
 fn duty(&mut self,source:&str,selector:Selector,address:&[&str])->Result<()>{
  if address.len()>3{return Err(fail());}let mut tokens=["";5];tokens[0]="rule-payload";tokens[1]=self.address;tokens[2..2+address.len()].copy_from_slice(address);
  self.sink.duty(source,selector,&tokens[..2+address.len()])
 }
}
fn literal(source:&str,domain:&Value,value:&Value,sink:&mut impl Sink)->Result<()> {
 sink.charge(0)?;let family=scalar(&domain["scalarType"])?;
 if value.is_null(){if domain["nullability"]!="absent-allowed"{return Err(fail());}return sink.duty(source,Selector::Literal(family,true),&["payload","literal"]);}
 let object=value.as_object().ok_or_else(fail)?;if object.len()!=1{return Err(fail());}
 let (name,value)=object.iter().next().ok_or_else(fail)?;sink.charge(name.len())?;
 let wrapper=match family{Scalar::Boolean=>"boolean",Scalar::String=>"string",Scalar::Integer=>"integerToken",Scalar::Decimal=>"decimalToken",Scalar::Binary=>"binaryHex"};
 if name!=wrapper{return Err(fail());}
 if family==Scalar::Boolean{if !value.is_boolean(){return Err(fail());}}else{sink.charge(value.as_str().ok_or_else(fail)?.len())?;}
 // Exact literal interpretation/refinements were checked by logical source admission.
 // This visitor does not renormalize or assert native representation compatibility.
 sink.duty(source,Selector::Literal(family,false),&["payload","literal"])
}
pub(crate) fn rule(source:&str,payload:&crate::security_rule_occurrences::RulePayload<'_>,address:&str,sink:&mut impl Sink)->Result<()> {
 use crate::{security_rule_occurrences::RulePayload as R,security_ir::{Term,Disposition}};
 let mut at=At{address,sink};at.charge(0)?;
 match payload{
  R::Operand(Term::Field{domain:d,..})=>domain(source,d,Role::FieldOperand,&mut at)?,
  R::Operand(Term::Context{domain:d,..})=>domain(source,d,Role::ContextOperand,&mut at)?,
  R::Operand(Term::Constant{domain:d,literal:v,..})=>{domain(source,d,Role::ConstantOperand,&mut at)?;literal(source,d,v,&mut at)?;},
  R::Disclosure{disposition:Disposition::Transformed{transform,version,domain:d,literal:v,..},..}=>{
   at.charge(transform.len())?;at.charge(version.len())?;if transform!="constant"||version!="0.1.0"{return Err(fail());}
   domain(source,d,Role::TransformOutput,&mut at)?;literal(source,d,v,&mut at)?;at.duty(source,Selector::ConstantTransformV1,&["payload","transform-revision"])?;
  },_=>{}
 }Ok(())
}
/// Sources/borrowed payloads come exclusively from the actual owner. Facet values,
/// action names, ordered members, transforms and full types remain there intact.
/// Dispatch never parses canonical source names or clones/normalizes JSON values.
pub(crate) fn visit(owner:&OwnerSourceDemands<'_, '_, '_, '_>,sink:&mut impl Sink)->Result<()> {
 for (source,e) in owner.field_events(){
  sink.charge(source.len())?;let role=match e.channel{FieldChannel::Stored{..}=>Role::Stored,FieldChannel::Context=>Role::Context};domain(source,e.carrier,role,sink)?;
  match (e.channel,e.classification){(FieldChannel::Stored{..},Some(c))=>protection(source,c,sink)?,(FieldChannel::Context,None)=>{},_=>return Err(fail())}
 }
 for (source,e) in owner.key_events(){sink.charge(source.len())?;match e{
  KeyPayload::Key(key)=>{let primary=match key.definition.get("primary"){None=>None,Some(v)=>Some(v.as_bool().ok_or_else(fail)?)};sink.duty(source,Selector::KeyPrimary(primary),&["payload","key-primary"])?;},
  KeyPayload::Member{carrier,..}=>{domain(source,carrier,Role::KeyMember,sink)?;sink.duty(source,Selector::OrderedKeyMember,&["payload","key-member"])?;}
 }}
 for (source,e) in owner.query_events(){sink.charge(source.len())?;match e{
  QueryPayload::Field{kind,carrier,classification,..}=>{domain(source,carrier,match kind{FieldUse::Projection=>Role::Projection,FieldUse::QueryField=>Role::QueryField},sink)?;protection(source,classification,sink)?;},
  QueryPayload::Operator{requirement,carrier,classification,..}=>{domain(source,carrier,Role::Operator,sink)?;protection(source,classification,sink)?;sink.duty(source,Selector::Operator(requirement.usage().operator,matches!(requirement.mode(),SecurityOperatorMode::OriginalAuthorized(_))),&["payload","operator-mode"])?;},
  QueryPayload::Output{requirement,plan}=>{
   output(source,requirement.output(),plan,sink)?;
  }
 }}
 for (source,e) in owner.association_events(){sink.charge(source.len())?;let endpoints=e.declaration["endpoints"].as_array().ok_or_else(fail)?;if endpoints.len()>4096{return Err(fail());}
  for (i,endpoint) in endpoints.iter().enumerate(){sink.charge(20)?;let i=i.to_string();sink.duty(source,Selector::AssociationEndpoint,&["payload","endpoint",&i])?;let members=endpoint["fields"].as_array().ok_or_else(fail)?;if members.len()>4096{return Err(fail());}
   for (j,_) in members.iter().enumerate(){sink.charge(20)?;let j=j.to_string();sink.duty(source,Selector::AssociationEndpointMember,&["payload","endpoint-member",&i,&j])?;}
  }
 }
 Ok(())
}
#[cfg(test)]
mod tests{
 use super::*;use serde_json::json;
 #[derive(Default)]struct Collected{duties:Vec<(Selector,Vec<String>)>,visits:usize,text:usize}
 impl Sink for Collected{fn charge(&mut self,bytes:usize)->Result<()>{self.visits+=1;self.text+=bytes;Ok(())}fn duty(&mut self,_:&str,s:Selector,a:&[&str])->Result<()>{self.duties.push((s,a.iter().map(|s|s.to_string()).collect()));Ok(())}}
 #[test]
 fn domain_dispatch_keeps_every_scalar_nullability_facet_and_protection_kind(){
  for (name,s) in [("boolean",Scalar::Boolean),("string",Scalar::String),("integer",Scalar::Integer),("decimal",Scalar::Decimal),("binary",Scalar::Binary)]{for (name_n,n) in [("required",Nullability::Required),("absent-allowed",Nullability::AbsentAllowed),("unspecified",Nullability::Unspecified)]{
   let mut out=Collected::default();domain("s",&json!({"scalarType":name,"nullability":name_n,"cardinality":"one"}),Role::Stored,&mut out).unwrap();assert_eq!(out.duties.iter().map(|(s,_)|*s).collect::<Vec<_>>(),vec![Selector::Role(Role::Stored),Selector::Scalar(s),Selector::Nullability(n)]);
  }}
  let mut out=Collected::default();domain("s",&json!({"scalarType":"integer","nullability":"required","cardinality":"one","facets":{"integerWidth":{"bits":128,"signed":false},"range":{"min":{"integerToken":"9007199254740993"}}},"allowedValues":[]}),Role::Context,&mut out).unwrap();
  assert_eq!(out.duties.iter().map(|(s,_)|*s).collect::<Vec<_>>(),vec![Selector::Role(Role::Context),Selector::Scalar(Scalar::Integer),Selector::Nullability(Nullability::Required),Selector::Facet(Facet::IntegerWidth),Selector::Facet(Facet::Range),Selector::Facet(Facet::AllowedValues)]);
  for (name,p) in [("protected",true),("unprotected",false)]{let mut out=Collected::default();protection("s",&json!({"protection":name}),&mut out).unwrap();assert_eq!(out.duties[0].0,Selector::Protection(p));}
 }
 #[test]
 fn availability_preserves_required_and_absent_allowed_and_refuses_unestablished(){
  assert_eq!(availability(Some("required")).unwrap(),Availability::Required);assert_eq!(availability(Some("absent-allowed")).unwrap(),Availability::AbsentAllowed);
  for v in [None,Some("unspecified"),Some("nullable")]{assert!(availability(v).is_err());}
 }
 #[test]
 fn unknown_selected_domain_and_classification_refuse(){
  for v in [json!({"scalarType":"floating","nullability":"required","cardinality":"one"}),json!({"scalarType":"string","nullability":"nullable","cardinality":"one"}),json!({"scalarType":"integer","nullability":"required","cardinality":"one","facets":{"nativeGuess":true}}),json!({"scalarType":"integer","nullability":"required","cardinality":"one","allowedValues":{}})]{assert!(domain("s",&v,Role::Stored,&mut Collected::default()).is_err());}
  assert!(protection("s",&json!({"protection":"inferred"}),&mut Collected::default()).is_err());
 }
 #[test]
 fn rule_literals_keep_all_wrappers_typed_absence_and_independent_small_ledger(){
  use crate::{security_ir::Term,security_rule_occurrences::RulePayload as R,security_ontology::SecurityRef};
  let field=SecurityRef{document_id:"d".into(),module_id:"m".into(),element_id:"f".into()};
  for (name,family,value) in [("boolean",Scalar::Boolean,json!({"boolean":false})),("string",Scalar::String,json!({"string":"𝄞"})),("integer",Scalar::Integer,json!({"integerToken":"9007199254740993"})),("decimal",Scalar::Decimal,json!({"decimalToken":"1.2300e2"})),("binary",Scalar::Binary,json!({"binaryHex":"00Ab"}))]{
   let d=json!({"cardinality":"one","scalarType":name,"nullability":"required","facets":{},"allowedValues":null});let t=Term::Constant{field:field.clone(),domain:d,literal:value};let mut out=Collected::default();rule("s",&R::Operand(&t),"operand-address",&mut out).unwrap();
   assert_eq!(out.duties.iter().map(|(s,_)|*s).collect::<Vec<_>>(),vec![Selector::Role(Role::ConstantOperand),Selector::Scalar(family),Selector::Nullability(Nullability::Required),Selector::Literal(family,false)]);
   assert!(out.duties.iter().all(|(_,a)|a[0]=="rule-payload"&&a[1]=="operand-address"));
   if family==Scalar::Boolean{assert_eq!((out.visits,out.text),(4,7));}
   let t=Term::Constant{field:field.clone(),domain:json!({"cardinality":"one","scalarType":name,"nullability":"absent-allowed","facets":{},"allowedValues":null}),literal:Value::Null};let mut out=Collected::default();rule("s",&R::Operand(&t),"operand-address",&mut out).unwrap();assert_eq!(out.duties.last().unwrap().0,Selector::Literal(family,true));assert_eq!((out.visits,out.text),(3,0));
  }
 }
 #[test]
 fn rule_literals_refuse_wrong_wrappers_required_absence_and_unknown_transform_revision(){
  use crate::{security_ir::Disposition,security_rule_occurrences::RulePayload as R,security_ontology::SecurityRef};
  let d=json!({"cardinality":"one","scalarType":"integer","nullability":"required","facets":{},"allowedValues":null});
  for value in [Value::Null,json!({"string":"1"}),json!({"integerToken":1}),json!({"integerToken":"1","other":true})]{assert!(literal("s",&d,&value,&mut Collected::default()).is_err());}
  let f=SecurityRef{document_id:"d".into(),module_id:"m".into(),element_id:"f".into()};
  for (transform,version) in [("redact","0.1.0"),("constant","0.2.0")]{let v=Disposition::Transformed{transform:transform.into(),version:version.into(),output_field:f.clone(),domain:d.clone(),literal:json!({"integerToken":"1"})};let mut out=Collected::default();assert!(rule("s",&R::Disclosure{field:&f,disposition:&v},"disclosure-address",&mut out).is_err());assert!(out.duties.is_empty());}
 }
 #[test]
 fn rule_occurrence_frames_keep_equal_values_at_distinct_disclosures_separate(){
  use crate::{security_ir::Disposition,security_rule_occurrences::RulePayload as R,security_ontology::SecurityRef};
  let f=SecurityRef{document_id:"d".into(),module_id:"m".into(),element_id:"f".into()};let v=Disposition::Transformed{transform:"constant".into(),version:"0.1.0".into(),output_field:f.clone(),domain:json!({"cardinality":"one","scalarType":"boolean","nullability":"required","facets":{},"allowedValues":null}),literal:json!({"boolean":false})};
  let mut out=Collected::default();for path in ["[\"disclosure\",\"0\"]","[\"disclosure\",\"1\"]"]{rule("s",&R::Disclosure{field:&f,disposition:&v},path,&mut out).unwrap();}
  let addresses=out.duties.iter().map(|(_,a)|a.clone()).collect::<std::collections::BTreeSet<_>>();assert_eq!(addresses.len(),10);assert_eq!(out.duties.iter().filter(|(p,_)|*p==Selector::ConstantTransformV1).count(),2);
 }

}
