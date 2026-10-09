//! Explicit finite exact representation of mathematical source integers.
//! Does not narrow the original logical domain or establish source validity.
use crate::{binding::Binding,candidate::{Candidate,TargetPlan}};
use serde_json::json;
use weft_core::{backend::*,error::{Diagnostic,Result},ir::{Family,LogicalType}};
pub const PROFILE:&str="dbsql-mathematical-integer-candidate";
pub struct MathematicalInteger;
pub fn unbounded(t:&LogicalType)->bool {t.family==Family::Integer&&t.facets==json!({})}
pub fn representable(token:&str)->bool {
 let digits=token.strip_prefix('-').unwrap_or(token);
 !digits.is_empty()&&digits.bytes().all(|b|b.is_ascii_digit())&&digits.trim_start_matches('0').len()<=38
}
impl Backend for MathematicalInteger {
 type Mapping=Binding;type TargetPlan=TargetPlan;
 fn describe(&self)->Result<Manifest> {
  let mut m=Candidate.describe()?;m.backend_id="ashlar.databricks.mathematical-integer".into();m.backend_version="0.1.0-candidate".into();
  for p in &mut m.target_profiles{p.id=PROFILE.into();p.session_settings["mathematicalIntegerRepresentation"]=json!({"logicalDomain":"original mathematical integers; no inferred width","nativeDomain":"DECIMAL(38,0), exact-or-capability-error","sourceValidity":"not established by representability checks"});}
  m.capabilities.retain(|c|!c.id.starts_with("value.")&&!c.id.starts_with("relationship.")&&c.id!="project.entity");
  for c in &mut m.capabilities{c.target_profiles=vec![PROFILE.into()];}
  m.capabilities.push(Capability{id:"type.integer.unbounded".into(),target_profiles:vec![PROFILE.into()],language_profiles:m.language_profiles.clone(),logical_domain:json!({"family":"integer","facets":{},"owningCore":"0.8.0"}),result_domain:json!({"carrier":"exact integer text","representation":"DECIMAL(38,0) or explicit capability failure"}),constraints:vec!["Required scalar relational reads only; no arbitrary precision engine claim".into(),"All consumed pinned sources, exact literals and aggregate results require representability guards".into()],obligations:vec![],status:Status::Candidate,evidence:vec![]});Ok(m)
 }
 fn validate_binding(&self,c:&Context<'_>)->Result<Validated<Binding>> {
  if let Plan::V02(p)=c.plan {
   if p.type_graph.iter().any(|d|!matches!(d.shape,weft_core::application_model::Shape::Scalar{..})||d.availability.as_deref()!=Some("required")) {
    return Err(Diagnostic::new("WFT-CAPABILITY","binding","Mathematical integer profile admits only required scalar descriptors"));
   }
  }
  Candidate.validate_binding(c)
 }
 fn assess(&self,c:&Context<'_>,m:&Binding)->Result<Vec<Assessment>> {Candidate.assess(c,m)}
 fn lower(&self,c:&Context<'_>,m:&Binding)->Result<TargetPlan> {
  let mut plan=Candidate.lower(c,m)?;let mut checks=vec![];let mut sources=vec![];
  for obligation in &mut plan.0.obligations {
   if obligation.id=="ashlar.candidate.scalarIntegrity" {
    let original=obligation.parameters["checks"].as_array_mut().unwrap();let mut scalar=vec![];
    for check in original.drain(..){if check["representabilityOnly"]==true{checks.push(check)}else if check["publicSourceOnly"]==true{sources.push(check)}else{scalar.push(check)}}*original=scalar;
   }
  }
  plan.0.obligations.retain(|o|o.id!="ashlar.candidate.scalarIntegrity"||!o.parameters["checks"].as_array().unwrap().is_empty());
  if !sources.is_empty(){plan.0.obligations.push(Obligation{id:"ashlar.mathematicalInteger.publicSourceValidity".into(),parameters:json!({"phase":"before-all-capacity-checks-and-user-query","checks":sources,"samePinnedSourceRequired":true,"publicValidator":"actual owning UMF validateCoreFieldValue with original integerToken","custody":"exact model pin, field identity, original raw JSON bytes/token/type and source/type/row/revision identity from original check SQL; every consumed occurrence including subsequently filtered rows","tokenFidelity":"exact original JSON numeric token must equal emitted extracted_token; no AST numeric value or float intermediate","unknownOrUnfulfilled":"refuse execution"}),owner:ObligationOwner::Host,failure_code:"WFT-NUMERIC-DOMAIN".into()});}
  if !checks.is_empty(){plan.0.obligations.push(Obligation{id:"ashlar.mathematicalInteger.representability".into(),parameters:json!({"phase":"before-user-query","checks":checks,"samePinnedSourceRequired":true,"success":"one exact STRING count equal to 0 per check","meaning":"backend representation capability, not original UMF source validity","nativeRepresentation":"DECIMAL(38,0)","noFloatOrRounding":true,"noPartialPublication":true}),owner:ObligationOwner::Host,failure_code:"WFT-CAPABILITY".into()});}
  plan.0.obligations.push(Obligation{id:"ashlar.mathematicalInteger.exactResult".into(),parameters:json!({"logicalDomain":"unchanged integer; no width synthesized","carrier":"exact base-ten integer text","representation":"DECIMAL(38,0)","arithmetic":"TRY_SUM overflow raises WFT-CAPABILITY; no rounded/null partial result","host":"buffer all rows; exact text decode; reject malformed/out-of-representation integer carriers; no partial publication"}),owner:ObligationOwner::Host,failure_code:"WFT-CAPABILITY".into()});Ok(plan)
 }
 fn emit(&self,c:&Context<'_>,p:&TargetPlan)->Result<Emission>{Candidate.emit(c,p)}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn exact_representation_boundary_does_not_bound_logical_type(){assert!(representable(&"9".repeat(38)));assert!(representable(&format!("-{}","9".repeat(38))));assert!(representable("-0000"));assert!(!representable(&format!("1{}","0".repeat(38))));for s in ["", "-", "1.0", "1e2", "1x", "+1"]{assert!(!representable(s));}}
 fn request(sql:&str,math:bool)->serde_json::Value{
  let text=include_str!("../../../tests/fixtures/original-commerce-0.8/ontology.json");let d:serde_json::Value=serde_json::from_str(text).unwrap();let pin=json!({"documentId":d["id"],"revision":"original-1","umfVersion":"0.8.0","sha256":weft_core::json::sha256(text.as_bytes())});let logical=|e:&str|json!({"documentId":d["id"],"revision":"original-1","module":"domain","element":e});
  // Explicit compiler fixture; UUID-shaped locator has no native/publication authority.
  let binding=json!({"profile":"ashlar-databricks-candidate/0.1.0","layoutRevision":"ashlar-delta/0.3","layoutSha256":"ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e","modelPins":[pin],"publication":{"id":"development-compiler-fixture","manifestUuid":"00000000-0000-4000-8000-000000000000","tables":[{"name":["spark_catalog","default","compiler_fixture"],"uuid":"00000000-0000-4000-8000-000000000000","version":1}]},"records":[{"logical":logical("order_lines"),"table":0,"kind":"object","sourceSystem":"development","typeId":"3","schemaRevision":"original-1","properties":[{"logical":logical("order_lines.quantity"),"home":{"kind":"props","propertyId":"7"}}]}]}).to_string();
  json!({"interfaceVersion":"weft-compile/0.2.0","dialect":"weft-sql/0.2.0","sql":sql,"modules":[{"documentJson":text,"pin":pin,"selectedModuleIds":["domain"]}],"target":{"backendId":if math{"ashlar.databricks.mathematical-integer"}else{"ashlar.databricks"},"backendVersion":"0.1.0-candidate","targetProfile":if math{PROFILE}else{"dbsql-candidate"},"bindingJson":binding,"bindingSha256":weft_core::json::sha256(binding.as_bytes())},"options":{"allowCandidate":true}})
 }
 fn compile(r:&serde_json::Value)->serde_json::Value{let mut registry=Registry::default();registry.register(MathematicalInteger).unwrap();registry.register(Candidate).unwrap();serde_json::from_str(&weft_core::compile::Compiler{registry}.compile_json(&r.to_string())).unwrap()}
 #[test]fn only_explicit_profile_admits_original_unbounded_fields_and_retains_tokens(){
  let old=compile(&request("SELECT o.quantity FROM order_lines o",false));assert_eq!(old["diagnostics"][0]["code"],"WFT-CAPABILITY");assert!(old.get("sql").is_none());
  let token="9007199254740993";let r=request(&format!("SELECT o.quantity FROM order_lines o WHERE o.quantity = {token}"),true);let out=compile(&r);assert_eq!(out["status"],"compiled","{out}");assert!(out["parameters"].as_array().unwrap().iter().any(|p|p["value"]==token&&p["logicalType"]["facets"]==json!({})));assert_eq!(out["columns"][0]["representation"]["logicalType"]["facets"],json!({}));assert_eq!(out["modelPins"][0]["umfVersion"],"0.8.0");let obligation=out["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.mathematicalInteger.representability").unwrap();assert_eq!(obligation["failureCode"],"WFT-CAPABILITY");
  let obligations=out["obligations"].as_array().unwrap();
  let source=obligations.iter().find(|o|o["id"]=="ashlar.mathematicalInteger.publicSourceValidity").unwrap();assert_eq!(source["failureCode"],"WFT-NUMERIC-DOMAIN");assert!(source["parameters"]["checks"][0]["sql"].as_str().unwrap().contains("r.props_json"));
  let integrity=obligations.iter().find(|o|o["id"]=="ashlar.candidate.scalarIntegrity").unwrap();let integrity_sql=integrity["parameters"]["checks"][0]["sql"].as_str().unwrap();assert!(integrity_sql.contains("schema_revision"));assert!(!integrity_sql.contains("try_cast(get_json_object"));
  let capacity_sql=obligation["parameters"]["checks"][0]["sql"].as_str().unwrap();assert!(capacity_sql.contains("try_cast(get_json_object"));assert!(!capacity_sql.contains("schema_revision"));
  let mut no_opt_in=request("SELECT o.quantity FROM order_lines o",true);no_opt_in["options"]["allowCandidate"]=json!(false);assert_eq!(compile(&no_opt_in)["diagnostics"][0]["code"],"WFT-CAPABILITY");
  let out=compile(&request(&format!("SELECT o.quantity FROM order_lines o WHERE o.quantity = 1{}","0".repeat(38)),true));assert_eq!(out["diagnostics"][0]["code"],"WFT-CAPABILITY");assert!(out.get("sql").is_none());
  let out=compile(&request("SELECT SUM(o.quantity) AS total FROM order_lines o",true));assert_eq!(out["status"],"compiled");assert!(out["sql"].as_str().unwrap().contains("TRY_SUM"));assert!(out["sql"].as_str().unwrap().contains("raise_error('WFT-CAPABILITY')"));
 }
 #[test]fn explicit_profile_classifies_bounded_integer_and_decimal_sum_capacity(){
  for (family,facets) in [("integer",json!({"integerWidth":{"bits":64,"signed":true}})),("decimal",json!({"precision":28,"scale":2}))] {
   for math in [false,true] {
    // Independent compiler control, not a replacement/relabel of commerce.
    let mut r=request("SELECT SUM(o.quantity) AS total FROM order_lines o",math);let mut source:serde_json::Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    let field=source["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="order_lines.quantity").unwrap();field["scalarType"]=json!(family);field["facets"]=facets.clone();let text=source.to_string();let sha=weft_core::json::sha256(text.as_bytes());r["modules"][0]["documentJson"]=json!(text);r["modules"][0]["pin"]["sha256"]=json!(sha.clone());let mut binding:serde_json::Value=serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();binding["modelPins"][0]["sha256"]=json!(sha);let bytes=binding.to_string();r["target"]["bindingSha256"]=json!(weft_core::json::sha256(bytes.as_bytes()));r["target"]["bindingJson"]=json!(bytes);
    let out=compile(&r);assert_eq!(out["status"],"compiled","{family}/{math}: {out}");let expected=if math{"WFT-CAPABILITY"}else{"WFT-NUMERIC-DOMAIN"};assert!(out["sql"].as_str().unwrap().contains(&format!("raise_error('{expected}')")));
   }
  }
 }
 struct BindingTrap;
 impl Backend for BindingTrap {
  type Mapping=Binding;type TargetPlan=TargetPlan;
  fn describe(&self)->Result<Manifest>{Candidate.describe()}
  fn validate_binding(&self,_:&Context<'_>)->Result<Validated<Binding>>{panic!("unsupported unbounded domain reached old binding code")}
  fn assess(&self,c:&Context<'_>,m:&Binding)->Result<Vec<Assessment>>{Candidate.assess(c,m)}
  fn lower(&self,c:&Context<'_>,m:&Binding)->Result<TargetPlan>{Candidate.lower(c,m)}
  fn emit(&self,c:&Context<'_>,p:&TargetPlan)->Result<Emission>{Candidate.emit(c,p)}
 }
 #[test]fn existing_manifest_refuses_before_binding_dispatch(){let mut registry=Registry::default();registry.register(BindingTrap).unwrap();let r=request("SELECT o.quantity FROM order_lines o",false);let out:serde_json::Value=serde_json::from_str(&weft_core::compile::Compiler{registry}.compile_json(&r.to_string())).unwrap();assert_eq!(out["diagnostics"][0]["code"],"WFT-CAPABILITY");assert!(out.get("sql").is_none());}
}
