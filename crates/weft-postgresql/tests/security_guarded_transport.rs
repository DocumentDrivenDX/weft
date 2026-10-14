#[jsonschema::validator(path="../../docs/helix/02-design/contracts/security-guarded-candidate-v0.1-candidate.schema.json")]
struct CandidateSchema;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/security-guarded-result-output-v0.1-candidate.schema.json")]
struct ResultSchema;
use serde_json::{json, Value};
use weft_core::{ir::ModelPin, json::sha256, model::ModuleInput};
use weft_postgresql::{
    security_graph_binding::GraphSource,
    security_guarded_transport::{inspect_guarded_graph_result, prepare_guarded_graph_candidate},
};
const FIXTURE: &str =
    include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/mapping-request.json");
fn request(sql: &str) -> String {
    let mut request: Value = serde_json::from_str(FIXTURE).unwrap();
    let physical =
        include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/adoc.json");
    let source=GraphSource{version:"weft.truss.security.graph-source/0.1-candidate".into(),native_schema_revision:"1".into(),physical_module:ModuleInput{document_json:physical.into(),pin:ModelPin{document_id:"adoc".into(),revision:"r1".into(),umf_version:"0.8.0".into(),sha256:sha256(physical.as_bytes())},selected_module_ids:vec!["m".into()]},inventory_json:include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/native-inventory.json").into(),correspondence_json:include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/binding-correspondence.json").into(),logical_resolution_json:include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/resolution.json").into(),logical_transport_json:include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/transport.json").into()};
    let binding = serde_json::to_string(&source).unwrap();
    request["bindingJson"] = json!(binding);
    request["sql"] = json!(sql);
    let mut profile: Value =
        serde_json::from_str(request["queryProfileJson"].as_str().unwrap()).unwrap();
    profile["binding"]["sha256"] = json!(sha256(binding.as_bytes()));
    request["queryProfileJson"] = json!(profile.to_string());
    request.to_string()
}
fn inspection(raw: &str, output: &str, batch: Value) -> Value {
    json!({"version":"weft.truss.security.guarded-result/0.1-candidate","sourceRequestJson":raw,"sourceOutputJson":output,"batchJson":batch.to_string()})
}
#[test]
fn guarded_handoff_keeps_source_query_parameter_and_subject_channels_distinct() {
    let raw = request("SELECT r.resourceId, r.salary FROM Resource r");
    let candidate = prepare_guarded_graph_candidate(&raw).unwrap();
    let wire: Value = serde_json::from_str(candidate.json()).unwrap();
    assert!(CandidateSchema::is_valid(&wire));
    assert!(candidate.json().ends_with('\n'));
    assert_eq!(
        wire["version"],
        "weft.truss.security.guarded-candidate/0.1-candidate"
    );
    assert_eq!(wire["securityReady"], false);
    assert_eq!(wire["nativeAdmission"], "required");
    assert_eq!(wire["sourceRequestSha256"], sha256(raw.as_bytes()));
    assert_eq!(wire["sourceContextSha256"], candidate.source().sha256());
    assert_eq!(wire["sourceContextJson"], candidate.source().json());
    assert_eq!(wire["queryContextSha256"], candidate.query_context_sha256());
    assert_eq!(wire["policyParameterBindings"][0]["ordinal"], 1);
    assert_eq!(
        wire["policyParameterBindings"][0]["physicalCodec"],
        "boolean"
    );
    assert_eq!(wire["subjectParameterBinding"]["ordinal"], 2);
    assert_eq!(
        wire["subjectParameterBinding"]["logicalKeyFields"][0]["elementId"],
        "staffId"
    );
    for forbidden in ["value", "principal", "epoch", "permit"] {
        assert!(wire["subjectParameterBinding"].get(forbidden).is_none());
    }
    let other =
        prepare_guarded_graph_candidate(&request("SELECT r.salary, r.resourceId FROM Resource r"))
            .unwrap();
    assert_eq!(candidate.source().json(), other.source().json());
    assert_ne!(
        candidate.query_context_sha256(),
        other.query_context_sha256()
    );
    assert_ne!(candidate.json(), other.json());
}
#[test]
fn original_result_batch_passes_correspondence_without_granting_a_release_permit() {
    let raw = request("SELECT r.resourceId, r.salary FROM Resource r");
    let candidate = prepare_guarded_graph_candidate(&raw).unwrap();
    let batch = json!({"version":"weft.security.cells/0.1.0","resultContractSha256":candidate.query().value()["resultContractSha256"],"rows":[[{"outcomeId":"original","disposition":"original","value":{"string":"RA"}},{"outcomeId":"withheld","disposition":"withheld"}]]});
    let checked: Value = serde_json::from_str(
        &inspect_guarded_graph_result(
            &inspection(&raw, candidate.json(), batch.clone()).to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(ResultSchema::is_valid(&checked));
    assert_eq!(checked["status"], "inspected");
    assert_eq!(checked["securityReady"], false);
    assert_eq!(
        checked["sourceOutputSha256"],
        sha256(candidate.json().as_bytes())
    );
    assert_eq!(checked["batchSha256"], sha256(batch.to_string().as_bytes()));
    let mut empty = batch;
    empty["rows"] = json!([]);
    assert!(
        inspect_guarded_graph_result(&inspection(&raw, candidate.json(), empty).to_string())
            .is_ok()
    );
}
#[test]
fn complete_guarded_result_refuses_changed_producer_source_or_later_withheld_value() {
    let raw = request("SELECT r.resourceId, r.salary FROM Resource r");
    let candidate = prepare_guarded_graph_candidate(&raw).unwrap();
    let batch = json!({"version":"weft.security.cells/0.1.0","resultContractSha256":candidate.query().value()["resultContractSha256"],"rows":[[{"outcomeId":"original","disposition":"original","value":{"string":"RA"}},{"outcomeId":"withheld","disposition":"withheld"}]]});
    let positional=json!(["weft.truss.security.guarded-result/0.1-candidate",raw,candidate.json(),batch.to_string()]);
    assert!(inspect_guarded_graph_result(&positional.to_string()).is_err());
    for mutation in 0..9 {
        let mut input = inspection(&raw, candidate.json(), batch.clone());
        match mutation {
            0 => input["sourceOutputJson"] = json!(candidate.json().trim_end()),
            1 => {
                let mut changed: Value = serde_json::from_str(candidate.json()).unwrap();
                changed["query"]["execution"]["sql"] = json!("SELECT 1");
                input["sourceOutputJson"] = json!(format!("{changed}\n"));
            }
            2 => {
                let mut changed: Value = serde_json::from_str(candidate.json()).unwrap();
                changed["policyParameterBindings"][0]["ordinal"] = json!(2);
                input["sourceOutputJson"] = json!(format!("{changed}\n"));
            }
            3 => {
                let mut changed: Value = serde_json::from_str(candidate.json()).unwrap();
                changed["sourceContextJson"] = json!("{}");
                input["sourceOutputJson"] = json!(format!("{changed}\n"));
            }
            4 => input["sourceRequestJson"] = json!(format!("{raw} ")),
            5 => {
                let mut corrupt = batch.clone();
                let mut row = corrupt["rows"][0].clone();
                row[1]["value"] = json!({"integerToken":"100"});
                corrupt["rows"].as_array_mut().unwrap().push(row);
                input["batchJson"] = json!(corrupt.to_string());
            }
            6 => input["permit"] = json!(true),
            7 => input["version"] = json!("weft.truss.security.graph-result/0.1-candidate"),
            _ => {
                let mut changed: Value = serde_json::from_str(candidate.json()).unwrap();
                changed["subjectParameterBinding"]["value"] = json!(1);
                input["sourceOutputJson"] = json!(format!("{changed}\n"));
            }
        };
        assert!(
            inspect_guarded_graph_result(&input.to_string()).is_err(),
            "mutation {mutation}"
        );
    }
}
#[test]
fn unsupported_queries_and_injected_authority_inputs_cannot_prepare_handoff() {
    for sql in [
        "SELECT COUNT(*) FROM Resource r",
        "SELECT r.resourceId FROM Resource r WHERE r.resourceId='RA'",
        "SELECT r.resourceId FROM Resource r ORDER BY r.resourceId",
    ] {
        assert!(prepare_guarded_graph_candidate(&request(sql)).is_err());
    }
    for injected in ["subject", "epoch", "permit", "nativeAdmission"] {
        let mut raw: Value =
            serde_json::from_str(&request("SELECT r.resourceId FROM Resource r")).unwrap();
        raw[injected] = json!(true);
        assert!(prepare_guarded_graph_candidate(&raw.to_string()).is_err());
    }
    let candidate =
        prepare_guarded_graph_candidate(&request("SELECT r.resourceId FROM Resource r")).unwrap();
    assert!(
        weft_core::security_backend::validate_security_manifest_json(candidate.json()).is_err()
    );
    assert!(weft_core::backend::validate_manifest_json(candidate.json()).is_err());
    let refused: Value = serde_json::from_str(
        &weft_core::compile::Compiler::default().compile_json(candidate.json()),
    )
    .unwrap();
    assert_eq!(refused["status"], "blocked");
    assert!(refused.get("sql").is_none());
}
fn query_use_request(sql:&str,mode:Option<&str>,grant:bool)->String {
 let mut request:Value=serde_json::from_str(&request(sql)).unwrap();
 if let Some(mode)=mode {
  let mut ontology:Value=serde_json::from_str(request["ontologyJson"].as_str().unwrap()).unwrap();let resource=ontology["entities"].as_array_mut().unwrap().iter_mut().find(|e|e["type"]["elementId"]=="Resource").unwrap();let salary=resource["fields"].as_array_mut().unwrap().iter_mut().find(|f|f["ref"]["elementId"]=="salary").unwrap();
  salary["queryUse"]=json!({"predicate":mode,"order":mode,"group":mode,"join":mode,"aggregate":mode});
  let mut policy:Value=serde_json::from_str(request["policyJson"].as_str().unwrap()).unwrap();
  let mut profile:Value=serde_json::from_str(request["queryProfileJson"].as_str().unwrap()).unwrap();
  if mode=="original-authorized" {
   ontology["actions"].as_array_mut().unwrap().push(json!("query-original"));
   if grant {policy["rules"].as_array_mut().unwrap().push(json!({"id":"original-access","effect":"permit","actions":["query-original"],"target":[{"documentId":"domain","moduleId":"m","elementId":"Resource"}],"condition":{"op":"literal","value":true}}));}
   profile["bindings"]=json!(["predicate","order","group","join","aggregate"].into_iter().map(|op|json!({"target":{"documentId":"domain","moduleId":"m","elementId":"Resource"},"field":{"documentId":"domain","moduleId":"m","elementId":"salary"},"operator":op,"originalAction":"query-original"})).collect::<Vec<_>>());
  }
  let mut source:GraphSource=serde_json::from_str(request["bindingJson"].as_str().unwrap()).unwrap();let mut resolution:Value=serde_json::from_str(&source.logical_resolution_json).unwrap();resolution["ontology"]=ontology.clone();source.logical_resolution_json=resolution.to_string();
  let ontology_bytes=ontology.to_string();let policy_bytes=policy.to_string();request["ontologyJson"]=json!(ontology_bytes);request["policyJson"]=json!(policy_bytes);
  let mut correspondence:Value=serde_json::from_str(&source.correspondence_json).unwrap();correspondence["logicalSourceDigests"]["resolution"]=json!(sha256(source.logical_resolution_json.as_bytes()));correspondence["logicalSourceDigests"]["policy"]=json!(sha256(policy_bytes.as_bytes()));source.correspondence_json=correspondence.to_string();let binding=serde_json::to_string(&source).unwrap();request["bindingJson"]=json!(binding);
  profile["ontologySha256"]=json!(sha256(ontology_bytes.as_bytes()));profile["policySha256"]=json!(sha256(policy_bytes.as_bytes()));profile["binding"]["sha256"]=json!(sha256(binding.as_bytes()));request["queryProfileJson"]=json!(profile.to_string());
 }
 request.to_string()
}
#[test]
fn query_use_guards_keep_all_five_operators_and_both_join_scan_occurrences() {
 use weft_core::{security_mapping_transport::with_security_mapping_context,security_ir::QueryOperator};
 use weft_postgresql::security_query_use_guards::prepare_graph_query_use_guards;
 let raw=query_use_request("SELECT r.salary, SUM(s.salary) FROM Resource r JOIN Resource s ON r.salary=s.salary WHERE r.salary=100 GROUP BY r.salary ORDER BY r.salary",Some("disclosed"),false);
 with_security_mapping_context(&raw,|ctx|{
  let source:GraphSource=serde_json::from_str(ctx.binding_json()).unwrap();let guards=prepare_graph_query_use_guards(ctx,&source)?;assert_eq!(guards.scans().len(),2);assert_eq!(guards.context_sha256(),ctx.execution_query_context_sha256());
  let ops=guards.scans().iter().flat_map(|s|s.uses()).map(|u|u.operator()).collect::<std::collections::BTreeSet<_>>();assert_eq!(ops,std::collections::BTreeSet::from([QueryOperator::Predicate,QueryOperator::Order,QueryOperator::Group,QueryOperator::Join,QueryOperator::Aggregate]));
  for scan in guards.scans(){assert!(scan.uses().iter().any(|u|u.operator()==QueryOperator::Join));assert!(scan.refusal_expression().contains("IN ('original','transformed','absent')) IS NOT TRUE"));assert!(scan.refusal_expression().contains("WHEN"));assert_eq!(scan.rule_truths().len(),2);}
  assert_ne!(guards.scans()[0].scan(),guards.scans()[1].scan());Ok(())
 }).unwrap();
}
#[test]
fn original_query_action_is_exact_and_missing_grant_refuses_before_query_filters() {
 use weft_core::security_mapping_transport::with_security_mapping_context;
 use weft_postgresql::security_query_use_guards::prepare_graph_query_use_guards;
 for granted in [false,true] {
  let raw=query_use_request("SELECT r.resourceId FROM Resource r WHERE r.salary=100",Some("original-authorized"),granted);
  with_security_mapping_context(&raw,|ctx|{
   let source:GraphSource=serde_json::from_str(ctx.binding_json()).unwrap();let guards=prepare_graph_query_use_guards(ctx,&source)?;let scan=&guards.scans()[0];assert!(scan.actions().iter().any(|a|a=="query-original"));assert_eq!(scan.uses()[0].original_action(),Some("query-original"));
   if granted {assert_eq!(scan.rule_truths().len(),3);assert!(scan.refusal_expression().contains("IS DISTINCT FROM 'permit'"));}else{assert_eq!(scan.rule_truths().len(),2);assert!(scan.refusal_expression().contains("THEN (TRUE)"));}
   // Guard derivation cannot activate the still-unsupported relational candidate.
   assert!(prepare_guarded_graph_candidate(&raw).is_err());Ok(())
  }).unwrap();
 }
}
#[test]
fn prohibited_and_foreign_sources_never_produce_query_use_guards() {
 use weft_core::security_mapping_transport::with_security_mapping_context;
 use weft_postgresql::security_query_use_guards::prepare_graph_query_use_guards;
 let prohibited=query_use_request("SELECT r.resourceId FROM Resource r WHERE r.salary=100",None,false);assert!(with_security_mapping_context(&prohibited,|_|Ok(())).is_err());
 let raw=query_use_request("SELECT r.resourceId FROM Resource r WHERE r.resourceId='RA'",None,false);
 with_security_mapping_context(&raw,|ctx|{let mut source:GraphSource=serde_json::from_str(ctx.binding_json()).unwrap();source.native_schema_revision="2".into();assert!(prepare_graph_query_use_guards(ctx,&source).is_err());Ok(())}).unwrap();
}
fn aggregate_contract(ctx:&weft_core::security_backend::SecurityBackendContext<'_>)->weft_core::security_lowering::SecurityResultContract {
 use weft_core::{application_ir::Expression,security_lowering::*};
 SecurityResultContract{version:weft_core::security_aggregate_results::VERSION.into(),encoding:"weft.security.cells/0.2-candidate".into(),columns:ctx.requirements().outputs().iter().map(|output|{
 if let Expression::Field{identity,..}=&output.output().expression {
 let identity=SecurityIdentity{document_id:identity.document_id.clone(),revision:identity.revision.clone(),module:identity.module.clone(),element:identity.element.clone()};
 return SecurityResultColumn{position:output.position(),output_name:output.output().name.clone(),source_fields:vec![identity.clone()],outcomes:vec![SecurityResultOutcome::Original{id:"original".into(),domain:SecurityResultDomain::Model{field:identity}}]};
 }
 let (t,source)=match &output.output().expression{Expression::Count{logical_type}=>(logical_type,None),Expression::Sum{argument,logical_type}=>(logical_type,Some(&argument.identity)),_=>panic!("aggregate fixture")};
 SecurityResultColumn{position:output.position(),output_name:output.output().name.clone(),source_fields:source.into_iter().map(|i|SecurityIdentity{document_id:i.document_id.clone(),revision:i.revision.clone(),module:i.module.clone(),element:i.element.clone()}).collect(),outcomes:vec![SecurityResultOutcome::Computed{id:"computed".into(),domain:SecurityResultDomain::Scalar{r#type:t.clone()}}]}
 }).collect()}
}
#[test]
fn aggregate_result_candidate_preserves_derived_domains_and_old_contract_refusals(){
 use weft_core::{security_mapping_transport::with_security_mapping_context,security_aggregate_results::*,security_lowering::*};
 let raw=query_use_request("SELECT COUNT(*), SUM(r.salary) FROM Resource r",Some("original-authorized"),true);
 with_security_mapping_context(&raw,|ctx|{
 let mut contract=aggregate_contract(ctx);check_aggregate_result_declaration(ctx,&contract)?;
 assert!(ctx.check_result_declaration(&contract).is_err());
 let text=serde_json::to_string(&contract).unwrap();let hash=sha256(text.as_bytes());
 let batch=|count:Value,sum:Value|json!({"version":"weft.security.cells/0.2-candidate","resultContractSha256":hash,"rows":[[{"outcomeId":"computed","disposition":"computed","value":count},{"outcomeId":"computed","disposition":"computed","value":sum}]]}).to_string();
 for (count,sum) in [(json!({"integerToken":"0"}),Value::Null),(json!({"integerToken":"123456789012345678901234567890"}),json!({"integerToken":"-100000000000000000000000000000"}))]{let batch=batch(count,sum);check_aggregate_result_cells(ctx,&contract,&text,&hash,&batch)?;assert!(ctx.check_result_cells(&contract,&text,&hash,&batch).is_err());}
 for (count,sum) in [(json!({"integerToken":"-1"}),Value::Null),(Value::Null,Value::Null),(json!({"integerToken":"1.5"}),Value::Null),(json!({"integerToken":"1"}),json!({"decimalToken":"1"})),(json!({"integerToken":"1"}),json!({"integerToken":"1","extra":true}))]{assert!(check_aggregate_result_cells(ctx,&contract,&text,&hash,&batch(count,sum)).is_err());}
 contract.version="weft.security.result-contract/0.1.0".into();assert!(check_aggregate_result_declaration(ctx,&contract).is_err());assert!(ctx.check_result_declaration(&contract).is_err());
 contract.version=VERSION.into();if let SecurityResultOutcome::Computed{domain:SecurityResultDomain::Scalar{r#type:t},..}=&mut contract.columns[1].outcomes[0]{t.facets=json!({"range":{"min":{"integerToken":"0"}}});}assert!(check_aggregate_result_declaration(ctx,&contract).is_err());Ok(())
 }).unwrap();
}
#[test]
fn grouped_sum_rejects_null_and_changed_input_lineage(){
 use weft_core::{security_mapping_transport::with_security_mapping_context,security_aggregate_results::*};
 let raw=query_use_request("SELECT SUM(r.salary) FROM Resource r GROUP BY r.resourceId",Some("original-authorized"),true);
 with_security_mapping_context(&raw,|ctx|{
 let mut contract=aggregate_contract(ctx);check_aggregate_result_declaration(ctx,&contract)?;let text=serde_json::to_string(&contract).unwrap();let hash=sha256(text.as_bytes());
 let batch=json!({"version":"weft.security.cells/0.2-candidate","resultContractSha256":hash,"rows":[[{"outcomeId":"computed","disposition":"computed","value":null}]]}).to_string();assert!(check_aggregate_result_cells(ctx,&contract,&text,&hash,&batch).is_err());
 contract.columns[0].source_fields[0].element="resourceId".into();assert!(check_aggregate_result_declaration(ctx,&contract).is_err());Ok(())
 }).unwrap();
}

#[test]
fn mixed_aggregate_contract_keeps_direct_disclosure_and_computed_outputs_distinct(){
 use weft_core::{security_mapping_transport::with_security_mapping_context,security_aggregate_results::*};
 let raw=query_use_request("SELECT r.resourceId, COUNT(*) FROM Resource r GROUP BY r.resourceId",None,false);
 with_security_mapping_context(&raw,|ctx|{
 let contract=aggregate_contract(ctx);check_aggregate_result_declaration(ctx,&contract)?;let text=serde_json::to_string(&contract).unwrap();let hash=sha256(text.as_bytes());
 let mut batch=json!({"version":ENCODING,"resultContractSha256":hash,"rows":[[{"outcomeId":"original","disposition":"original","value":{"string":"RA"}},{"outcomeId":"computed","disposition":"computed","value":{"integerToken":"1"}}]]});
 check_aggregate_result_cells(ctx,&contract,&text,&hash,&batch.to_string())?;
 batch["rows"][0][0]["disposition"]=json!("computed");assert!(check_aggregate_result_cells(ctx,&contract,&text,&hash,&batch.to_string()).is_err());
 batch["rows"][0][0]["disposition"]=json!("original");batch["rows"][0][1]["outcomeId"]=json!("original");assert!(check_aggregate_result_cells(ctx,&contract,&text,&hash,&batch.to_string()).is_err());Ok(())
 }).unwrap();
}
#[test]
fn relational_parameters_keep_repeated_named_values_distinct_from_policy_and_subject(){
 use weft_core::{security_mapping_transport::with_security_mapping_context,application_ir::Value as QueryValue};
 use weft_postgresql::security_relational_parameters::*;
 let mut raw:Value=serde_json::from_str(&query_use_request("SELECT r.resourceId FROM Resource r WHERE r.resourceId=:id AND r.resourceId=:id",None,false)).unwrap();
 let hostile="RA'; SELECT secret FROM private_table; --";
 raw["parameters"]=json!({"id":{"family":"string","value":hostile}});
 with_security_mapping_context(&raw.to_string(),|ctx|{
 let source:GraphSource=serde_json::from_str(ctx.binding_json()).unwrap();let bindings=prepare_relational_graph_parameters(ctx,&source)?;bindings.require_context(ctx)?;
 assert_eq!(bindings.graph().policy().len(),1);assert_eq!(bindings.policy_values(),&[json!(true)]);assert_eq!(bindings.graph().subject().ordinal(),2);assert_eq!(bindings.query().len(),2);
 for (i,p) in bindings.query().iter().enumerate(){assert_eq!(p.ordinal(),3+i);assert_eq!(p.codec(),RelationalQueryParameterCodec::Text);assert_eq!(p.source_path(),format!("filters/{i}/right"));assert!(matches!(p.value(),QueryValue::Parameter{name,value,..} if name=="id"&&value==hostile));}
 // Old direct graph producer still refuses query slots, not reclassifying them.
 assert!(prepare_guarded_graph_candidate(&raw.to_string()).is_err());
 with_security_mapping_context(&raw.to_string(),|other|{assert!(bindings.require_context(other).is_err());Ok(())})?;
 Ok(())
 }).unwrap();
}
#[test]
fn relational_parameters_preserve_numeric_literals_keyset_order_and_join_occurrences(){
 use weft_core::{security_mapping_transport::with_security_mapping_context,application_ir::Value as QueryValue};use weft_postgresql::security_relational_parameters::*;
 let raw=query_use_request("SELECT r.resourceId FROM Resource r JOIN Resource s ON r.salary=s.salary WHERE r.salary=100 AND s.salary=200",Some("original-authorized"),true);
 with_security_mapping_context(&raw,|ctx|{
 let source:GraphSource=serde_json::from_str(ctx.binding_json()).unwrap();let p=prepare_relational_graph_parameters(ctx,&source)?;assert_eq!(p.query().len(),2);
 for (binding,path,value) in [( &p.query()[0],"filters/0/right","100"),(&p.query()[1],"filters/1/right","200")]{assert_eq!(binding.codec(),RelationalQueryParameterCodec::Numeric);assert_eq!(binding.source_path(),path);assert!(matches!(binding.value(),QueryValue::Literal{value:v,..} if v==value));}Ok(())
 }).unwrap();
 let raw=query_use_request("SELECT r.resourceId FROM Resource r WHERE (r.resourceId)>('RA') ORDER BY r.resourceId LIMIT 1",None,false);
 with_security_mapping_context(&raw,|ctx|{let source:GraphSource=serde_json::from_str(ctx.binding_json()).unwrap();let p=prepare_relational_graph_parameters(ctx,&source)?;assert_eq!(p.query()[0].source_path(),"filters/0/values/0");assert_eq!(p.query()[0].ordinal(),3);Ok(())}).unwrap();
}
#[test]
fn relational_parameters_refuse_foreign_graph_sources_without_binding_native_subject(){
 use weft_core::security_mapping_transport::with_security_mapping_context;use weft_postgresql::security_relational_parameters::*;
 let raw=query_use_request("SELECT COUNT(*) FROM Resource r",None,false);
 with_security_mapping_context(&raw,|ctx|{let mut source:GraphSource=serde_json::from_str(ctx.binding_json()).unwrap();let p=prepare_relational_graph_parameters(ctx,&source)?;assert!(p.query().is_empty());assert_eq!(p.graph().subject().ordinal(),2);source.native_schema_revision="2".into();assert!(prepare_relational_graph_parameters(ctx,&source).is_err());Ok(())}).unwrap();
}
#[test]
fn relational_sql_emits_all_owner_stages_with_pre_query_guards_and_computed_lineage(){
 use weft_core::{security_mapping_transport::with_security_mapping_context,security_ir::QueryOperator};use weft_postgresql::security_relational_sql::prepare_relational_graph_sql;
 let raw=query_use_request("SELECT r.salary, SUM(s.salary), COUNT(*) FROM Resource r JOIN Resource s ON r.salary=s.salary WHERE r.salary=100 GROUP BY r.salary ORDER BY r.salary LIMIT 1",Some("original-authorized"),true);
 with_security_mapping_context(&raw,|ctx|{
 let source:GraphSource=serde_json::from_str(ctx.binding_json()).unwrap();let plan=prepare_relational_graph_sql(ctx,&source)?;
 for fragment in ["AS MATERIALIZED","JOIN a1 ON","WHERE (","GROUP BY","pg_catalog.sum(","pg_catalog.count(*)","ORDER BY","LIMIT 1","'computed'"]{assert!(plan.sql().contains(fragment),"{fragment}");}
 assert!(plan.checks().iter().filter(|c|c["failureCode"]=="WFT-SECURITY-QUERY-USE").count()==2);
 assert!(plan.checks().iter().any(|c|c["failureCode"]=="WFT-SECURITY-GROUP-DISCLOSURE"));
 let ops=plan.operand_demands().iter().map(|o|serde_json::from_value::<QueryOperator>(o["operator"].clone()).unwrap()).collect::<std::collections::BTreeSet<_>>();assert_eq!(ops.len(),5);
 assert_eq!(plan.result_contract().version,weft_core::security_aggregate_results::VERSION);assert_eq!(plan.result_contract().columns.len(),3);
 assert_eq!(plan.parameters().query().len(),1);assert_eq!(plan.parameters().query()[0].ordinal(),3);assert!(plan.sql().contains("$3::pg_catalog.numeric"));
 assert!(ctx.check_result_declaration(plan.result_contract()).is_err());assert!(prepare_guarded_graph_candidate(&raw).is_err());Ok(())
 }).unwrap();
}
#[test]
fn relational_sql_keeps_named_input_out_of_sql_and_retains_query_only_operand_guards(){
 use weft_core::security_mapping_transport::with_security_mapping_context;use weft_postgresql::security_relational_sql::prepare_relational_graph_sql;
 let mut raw:Value=serde_json::from_str(&query_use_request("SELECT r.resourceId FROM Resource r WHERE r.resourceId=:id ORDER BY r.resourceId LIMIT 2",None,false)).unwrap();let hostile="RA';SELECT private_password;--";raw["parameters"]=json!({"id":{"family":"string","value":hostile}});
 with_security_mapping_context(&raw.to_string(),|ctx|{let source:GraphSource=serde_json::from_str(ctx.binding_json()).unwrap();let plan=prepare_relational_graph_sql(ctx,&source)?;assert!(!plan.sql().contains(hostile));assert!(plan.sql().contains("$3::pg_catalog.text"));assert_eq!(plan.result_contract().columns.len(),1);assert_eq!(plan.operand_demands().len(),2);assert_eq!(plan.result_contract().version,"weft.security.result-contract/0.1.0");ctx.check_result_declaration(plan.result_contract())?;assert!(plan.checks().iter().all(|c|c["sql"].as_str().unwrap().contains("parameter_types AS")));Ok(())}).unwrap();
}
#[test]
fn relational_sql_count_preserves_subject_and_rule_checks_without_fabricated_field_operands(){
 use weft_core::security_mapping_transport::with_security_mapping_context;use weft_postgresql::security_relational_sql::prepare_relational_graph_sql;
 let raw=query_use_request("SELECT COUNT(*) FROM Resource r",None,false);
 with_security_mapping_context(&raw,|ctx|{let source:GraphSource=serde_json::from_str(ctx.binding_json()).unwrap();let plan=prepare_relational_graph_sql(ctx,&source)?;assert!(plan.operand_demands().is_empty());assert!(plan.parameters().query().is_empty());assert!(plan.result_contract().columns[0].source_fields.is_empty());assert!(plan.checks().iter().any(|c|c["failureCode"]=="WFT-SECURITY-SUBJECT"));assert!(plan.checks().iter().any(|c|c["failureCode"]=="WFT-SECURITY-QUERY-USE"));assert!(!plan.sql().contains("GROUP BY"));Ok(())}).unwrap();
}
#[test]
fn relational_boolean_group_order_uses_bool_or_and_keeps_absence_key(){
 use weft_core::security_mapping_transport::with_security_mapping_context;use weft_postgresql::security_relational_sql::prepare_relational_graph_sql;
 let mut raw:Value=serde_json::from_str(&query_use_request("SELECT a.active,COUNT(*) FROM Assignment a GROUP BY a.active ORDER BY a.active",None,false)).unwrap();
 let assignment=json!({"documentId":"domain","moduleId":"m","elementId":"Assignment"});let mut policy:Value=serde_json::from_str(raw["policyJson"].as_str().unwrap()).unwrap();policy["rules"]=json!([{"id":"read-assignment","effect":"permit","actions":["read"],"target":[assignment],"condition":{"op":"literal","value":true}}]);raw["policyJson"]=json!(policy.to_string());
 let mut source:GraphSource=serde_json::from_str(raw["bindingJson"].as_str().unwrap()).unwrap();let mut correspondence:Value=serde_json::from_str(&source.correspondence_json).unwrap();correspondence["logicalSourceDigests"]["policy"]=json!(sha256(policy.to_string().as_bytes()));source.correspondence_json=correspondence.to_string();raw["bindingJson"]=json!(serde_json::to_string(&source).unwrap());
 let mut profile:Value=serde_json::from_str(raw["queryProfileJson"].as_str().unwrap()).unwrap();profile["targets"]=json!([assignment]);profile["policySha256"]=json!(sha256(policy.to_string().as_bytes()));profile["binding"]["sha256"]=json!(sha256(raw["bindingJson"].as_str().unwrap().as_bytes()));raw["queryProfileJson"]=json!(profile.to_string());
 with_security_mapping_context(&raw.to_string(),|ctx|{let source:GraphSource=serde_json::from_str(ctx.binding_json()).unwrap();let p=prepare_relational_graph_sql(ctx,&source)?;assert!(p.sql().contains("ROW(("));assert!(p.sql().contains("ORDER BY pg_catalog.bool_or("));assert!(!p.sql().contains("pg_catalog.min("));assert!(p.checks().iter().find(|c|c["failureCode"]=="WFT-SECURITY-GROUP-DISCLOSURE").unwrap()["sql"].as_str().unwrap().contains("IS NULL"));Ok(())}).unwrap();
}
#[test]
fn relational_disclosed_cross_family_transform_refuses_before_silent_null_conversion(){
 use weft_core::security_mapping_transport::with_security_mapping_context;use weft_postgresql::security_relational_sql::prepare_relational_graph_sql;
 let mut raw:Value=serde_json::from_str(&query_use_request("SELECT COUNT(*) FROM Resource r WHERE r.salary=100",Some("disclosed"),false)).unwrap();let mut policy:Value=serde_json::from_str(raw["policyJson"].as_str().unwrap()).unwrap();policy["rules"][0]["disclosure"][0]["disposition"]=json!({"kind":"transformed","transform":"constant","version":"0.1.0","field":{"documentId":"domain","moduleId":"m","elementId":"resourceId"},"value":{"string":"redacted"}});raw["policyJson"]=json!(policy.to_string());
 let mut source:GraphSource=serde_json::from_str(raw["bindingJson"].as_str().unwrap()).unwrap();let mut correspondence:Value=serde_json::from_str(&source.correspondence_json).unwrap();correspondence["logicalSourceDigests"]["policy"]=json!(sha256(policy.to_string().as_bytes()));source.correspondence_json=correspondence.to_string();raw["bindingJson"]=json!(serde_json::to_string(&source).unwrap());
 let mut profile:Value=serde_json::from_str(raw["queryProfileJson"].as_str().unwrap()).unwrap();profile["policySha256"]=json!(sha256(policy.to_string().as_bytes()));profile["binding"]["sha256"]=json!(sha256(raw["bindingJson"].as_str().unwrap().as_bytes()));raw["queryProfileJson"]=json!(profile.to_string());
 with_security_mapping_context(&raw.to_string(),|ctx|{let source:GraphSource=serde_json::from_str(ctx.binding_json()).unwrap();assert!(prepare_relational_graph_sql(ctx,&source).is_err());Ok(())}).unwrap();
}
