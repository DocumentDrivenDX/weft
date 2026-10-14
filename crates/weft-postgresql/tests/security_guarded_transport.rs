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
