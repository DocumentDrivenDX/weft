//! Compiler-owned direct projection from the original resolved query and graph source.
use serde_json::{Value,json};
use weft_core::{error::{Diagnostic,Result},security_backend::SecurityBackendContext,security_lowering::SecurityResultContract,security_ontology::{SecurityRef,SecurityOntologyClosure}};
use crate::security_graph_binding::{GraphBinding,GraphSource};
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-GRAPH-QUERY","capability","Original direct graph projection required")}
pub(crate) fn column(input:&serde_json::Value,output:&serde_json::Value,probe:&serde_json::Value,position:usize)->Result<weft_core::security_lowering::SecurityResultColumn> {
 use weft_core::security_lowering::*;
 let object=probe.as_object().ok_or_else(fail)?;
 if object.len()!=3||object.keys().any(|k|!matches!(k.as_str(),"target"|"field"|"outputName")){return Err(fail());}
 fn identity(input:&serde_json::Value,r:&serde_json::Value)->Result<SecurityIdentity>{
  let document=r["documentId"].as_str().ok_or_else(fail)?;
  let pins=input["modules"].as_array().ok_or_else(fail)?;
  let selected=pins.iter().filter(|m|m["pin"]["documentId"]==document).collect::<Vec<_>>();if selected.len()!=1{return Err(fail());}
  Ok(SecurityIdentity{document_id:document.into(),revision:selected[0]["pin"]["revision"].as_str().ok_or_else(fail)?.into(),module:r["moduleId"].as_str().ok_or_else(fail)?.into(),element:r["elementId"].as_str().ok_or_else(fail)?.into()})
 }
 let profile=weft_core::json::checked_json(input["queryProfileJson"].as_str().ok_or_else(fail)?).map_err(|_|fail())?;
 let groups=output["eligibility"].as_array().ok_or_else(fail)?.iter().filter(|g|g["target"]==probe["target"]&&g["action"]==profile["action"]).collect::<Vec<_>>();if groups.len()!=1{return Err(fail());}
 let fields=groups[0]["disclosure"].as_array().ok_or_else(fail)?.iter().filter(|f|f["field"]==probe["field"]).collect::<Vec<_>>();if fields.len()!=1{return Err(fail());}
 let field=identity(input,&probe["field"])?;let target=identity(input,&probe["target"])?;
 let policy=weft_core::json::checked_json(input["policyJson"].as_str().ok_or_else(fail)?).map_err(|_|fail())?;
 let ontology=weft_core::json::checked_json(input["ontologyJson"].as_str().ok_or_else(fail)?).map_err(|_|fail())?;
 let classification=ontology["entities"].as_array().ok_or_else(fail)?.iter().chain(ontology["associations"].as_array().ok_or_else(fail)?).find(|e|e["type"]==probe["target"]).ok_or_else(fail)?["fields"].as_array().ok_or_else(fail)?.iter().find(|f|f["ref"]==probe["field"]).ok_or_else(fail)?;
 let mut original=classification["protection"]=="unprotected";let mut withheld=false;
 for rule in policy["rules"].as_array().ok_or_else(fail)? {if rule["effect"]!="permit"||!rule["target"].as_array().ok_or_else(fail)?.contains(&probe["target"])||!rule["actions"].as_array().ok_or_else(fail)?.contains(&profile["action"]){continue;}
  if let Some(disclosures)=rule["disclosure"].as_array(){for d in disclosures {if d["field"]==probe["field"]{original|=d["disposition"]["kind"]=="original";withheld|=d["disposition"]["kind"]=="withheld";}}}
 }
 let mut outcomes=Vec::new();if original{outcomes.push(SecurityResultOutcome::Original{id:"original".into(),domain:SecurityResultDomain::Model{field:field.clone()}});
  let doc=weft_core::json::checked_json(input["modules"].as_array().ok_or_else(fail)?.iter().find(|m|m["pin"]["documentId"]==field.document_id).ok_or_else(fail)?["documentJson"].as_str().ok_or_else(fail)?).map_err(|_|fail())?;
  let source=doc["modules"].as_array().ok_or_else(fail)?.iter().find(|m|m["id"]==field.module).ok_or_else(fail)?["elements"].as_array().ok_or_else(fail)?.iter().find(|e|e["id"]==field.element).ok_or_else(fail)?;
  if source["nullability"]=="absent-allowed"{outcomes.push(SecurityResultOutcome::Absent{id:"absent".into()});}
 }
 if withheld{outcomes.push(SecurityResultOutcome::Withheld{id:"withheld".into()});}
 let mut classes:std::collections::BTreeMap<String,(SecurityTransform,SecurityResultDomain,Vec<SecurityDispositionSource>)>=std::collections::BTreeMap::new();
 for t in fields[0]["transformations"].as_array().ok_or_else(fail)? {
  let d=&t["disposition"]["transformed"];let output_field=identity(input,&d["outputField"])?;
  let entry=classes.entry(t["outcomeId"].as_str().ok_or_else(fail)?.into()).or_insert((SecurityTransform::Constant{version:d["version"].as_str().ok_or_else(fail)?.into(),output_field:output_field.clone(),literal:d["literal"].clone()},SecurityResultDomain::Model{field:output_field},vec![]));
  entry.2.push(SecurityDispositionSource{rule_id:t["ruleId"].as_str().ok_or_else(fail)?.into(),target:target.clone(),field:field.clone()});
 }
 for (id,(transform,domain,disposition_sources)) in classes{outcomes.push(SecurityResultOutcome::Transformed{id,transform,domain,disposition_sources});}
 Ok(SecurityResultColumn{position,output_name:probe["outputName"].as_str().ok_or_else(fail)?.into(),source_fields:vec![field],outcomes})
}
/// Original owner-checked graph query declaration. No constructor/deserializer
/// or native release authority is exposed.
pub struct GraphQueryCandidate {value:Value,contract:SecurityResultContract,parameter_bindings:crate::security_graph_parameters::GraphParameterBindings}
impl GraphQueryCandidate {
 pub fn value(&self)->&Value {&self.value}
 pub fn result_contract(&self)->&SecurityResultContract {&self.contract}
 pub fn parameter_bindings(&self)->&crate::security_graph_parameters::GraphParameterBindings {&self.parameter_bindings}
}
pub fn prepare_graph_query(ctx:&SecurityBackendContext<'_>,source:&GraphSource)->Result<Value>{Ok(prepare_graph_query_checked(ctx,source)?.value)}
/// Source-only candidate. Subject locator values must come from the authenticated
/// guarded host; this function creates no caller or native execution credential.
pub fn prepare_graph_query_checked(ctx:&SecurityBackendContext<'_>,source:&GraphSource)->Result<GraphQueryCandidate>{
 let binding_value=weft_core::json::checked_json(ctx.binding_json()).map_err(|_|fail())?;
 if binding_value!=serde_json::to_value(source).map_err(|_|fail())?{return Err(fail());}
 let app=ctx.query().application_plan();
 if !app.joins.is_empty()||!app.filters.is_empty()||!app.groups.is_empty()||app.aggregate||!app.order.is_empty()||app.limit.is_some()||app.outputs.is_empty()||app.outputs.len()>64{return Err(fail());}
 let target=SecurityRef{document_id:app.source.record.document_id.clone(),module_id:app.source.record.module.clone(),element_id:app.source.record.element.clone()};
 let graph_binding=GraphBinding::read(ctx.logical_plan(),ctx.catalog(),source)?;
 let rendered=crate::security_predicate_spike::render_graph_with_sources(ctx.logical_plan(),ctx.catalog(),source)?;let graph=rendered.value;
 let ontology=SecurityOntologyClosure::read(ctx.logical_plan().source(),ctx.catalog())?;
 let subject=ontology.subject();let subject_tid=graph_binding.type_id(subject)?;let target_tid=graph_binding.type_id(&target)?;
 let rules=graph["rules"].as_array().ok_or_else(fail)?;
 let selected:Vec<_>=rules.iter().filter(|r|r["target"]==json!(target)&&r["actions"].as_array().is_some_and(|a|a.contains(&json!(ctx.profiled_query().primary_action())))).collect();
 if selected.is_empty(){return Err(fail());}
 let parameters=graph["parameters"].as_array().ok_or_else(fail)?;let parameter_bindings=crate::security_graph_parameters::issue(ctx,&rendered.policy_source_paths,parameters,graph["parameterTypes"].as_array().ok_or_else(fail)?,subject_tid)?;let subject_slot=parameter_bindings.subject().ordinal();
 let truth_columns=selected.iter().map(|r|Ok(format!("({}) AS {}",r["expression"].as_str().ok_or_else(fail)?,r["truthColumn"].as_str().ok_or_else(fail)?))).collect::<Result<Vec<_>>>()?;
 let unknown=selected.iter().map(|r|format!("{} IS NULL",r["truthColumn"].as_str().unwrap())).collect::<Vec<_>>().join(" OR ");
 let groups=graph["eligibility"].as_array().ok_or_else(fail)?;let matches:Vec<_>=groups.iter().filter(|g|g["target"]==json!(target)&&g["action"]==ctx.profiled_query().primary_action()).collect();if matches.len()!=1{return Err(fail());}let group=matches[0];
 let input=json!({"modules":ctx.catalog().inputs(),"policyJson":ctx.logical_plan().source().policy_json(),"ontologyJson":ctx.logical_plan().source().ontology_json(),"queryProfileJson":json!({"action":ctx.profiled_query().primary_action()}).to_string()});
 let mut columns=Vec::new();let mut cells=Vec::new();
 for (i,output) in app.outputs.iter().enumerate(){
  let weft_core::application_ir::Expression::Field{scan,identity}=&output.expression else{return Err(fail());};
  if scan!=&app.source.occurrence{return Err(fail());}
  let field=SecurityRef{document_id:identity.document_id.clone(),module_id:identity.module.clone(),element_id:identity.element.clone()};
  let entries:Vec<_>=group["disclosure"].as_array().ok_or_else(fail)?.iter().filter(|d|d["field"]==json!(field)).collect();if entries.len()!=1{return Err(fail());}
  cells.push(entries[0]["cellExpression"].as_str().ok_or_else(fail)?.to_string());
  columns.push(column(&input,&graph,&json!({"target":target,"field":field,"outputName":output.name}),i+1)?);
 }
 let contract=SecurityResultContract{version:"weft.security.result-contract/0.1.0".into(),encoding:"weft.security.cells/0.1.0".into(),columns};ctx.check_result_declaration(&contract)?;
 let contract_json=serde_json::to_string(&contract).map_err(|_|fail())?;
 let cte=format!("WITH rule_truths AS MATERIALIZED (SELECT r.id,r.props,{} FROM truss.object r CROSS JOIN truss.object s WHERE r.type_id={target_tid} AND s.type_id={subject_tid} AND s.id=${subject_slot}::pg_catalog.int8)",truth_columns.join(","));
 let sql=format!("{cte} SELECT pg_catalog.jsonb_build_array({})::pg_catalog.text AS cells FROM rule_truths r WHERE ({})='permit'",cells.join(","),group["decisionExpression"].as_str().ok_or_else(fail)?);
 let truth_guard=format!("{cte} SELECT pg_catalog.count(*)::pg_catalog.text AS violations FROM rule_truths r WHERE {unknown}");
 let subject_guard=format!("SELECT CASE WHEN pg_catalog.count(*)=1 THEN '0' ELSE '1' END::pg_catalog.text AS violations FROM truss.object s WHERE s.type_id={subject_tid} AND s.id=${subject_slot}::pg_catalog.int8");
 let result=json!({"version":"weft.truss.security.graph-query/0.1-candidate","securityReady":false,"contextSha256":ctx.execution_query_context_sha256(),"graphSourceSha256":graph_binding.source_sha256(),"execution":{"sql":sql,"policyParameters":parameters,"policyParameterTypes":graph["parameterTypes"],"subjectSlot":{"ordinal":subject_slot,"physicalType":"int8","nativeTypeId":subject_tid,"logicalSubject":subject,"supplier":"original_authenticated_guarded_host_only"}},"integrityChecks":graph_binding.integrity_checks(),"subjectCheck":{"sql":subject_guard,"failureCode":"WFT-SECURITY-SUBJECT"},"truthCheck":{"sql":truth_guard,"failureCode":"WFT-SECURITY-INDETERMINATE"},"resultContract":contract,"resultContractJson":contract_json,"resultContractSha256":weft_core::json::sha256(contract_json.as_bytes()),"nativeAdmission":"required"});
 if serde_json::to_vec(&result).map_err(|_|fail())?.len()>16_000_000{return Err(fail());}Ok(GraphQueryCandidate{value:result,contract,parameter_bindings})
}
