// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{json, Value};
use weft_core::{
    json::sha256,
    security_mapping_transport::with_security_mapping_context,
    security_parameter_sources::{SecurityPolicyParameterKind, SecurityQueryParameterValue},
};
const REQUEST: &str =
    include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/mapping-request.json");
fn request(sql: &str, expanded: bool) -> String {
    let mut r: Value = serde_json::from_str(REQUEST).unwrap();
    r["sql"] = json!(sql);
    if expanded {
        let active = json!({"documentId":"domain","moduleId":"m","elementId":"active"});
        let salary = json!({"documentId":"domain","moduleId":"m","elementId":"salary"});
        let mut policy: Value = serde_json::from_str(r["policyJson"].as_str().unwrap()).unwrap();
        policy["rules"][0]["condition"] = json!({"op":"and","args":[{"op":"or","args":[{"op":"eq","left":{"kind":"constant","field":active,"value":{"boolean":true}},"right":{"kind":"constant","field":active,"value":{"boolean":false}}},{"op":"not","arg":{"op":"eq","left":{"kind":"context","field":active},"right":{"kind":"constant","field":active,"value":{"boolean":true}}}}]},{"op":"literal","value":true}]});
        policy["rules"][0]["disclosure"][0]["disposition"] = json!({"kind":"transformed","transform":"constant","version":"0.1.0","field":salary,"value":{"integerToken":"1e2"}});
        r["policyJson"] = json!(policy.to_string());
        let mut ontology: Value =
            serde_json::from_str(r["ontologyJson"].as_str().unwrap()).unwrap();
        ontology["context"] = json!([active]);
        r["ontologyJson"] = json!(ontology.to_string());
        let mut profile: Value =
            serde_json::from_str(r["queryProfileJson"].as_str().unwrap()).unwrap();
        profile["policySha256"] = json!(sha256(r["policyJson"].as_str().unwrap().as_bytes()));
        profile["ontologySha256"] = json!(sha256(r["ontologyJson"].as_str().unwrap().as_bytes()));
        r["queryProfileJson"] = json!(profile.to_string());
    }
    r.to_string()
}
#[test]
fn inventory_borrows_actual_nested_constants_and_original_subject_key() {
    with_security_mapping_context(
        &request("SELECT r.resourceId, r.salary FROM Resource r", false),
        |ctx| {
            let inventory = ctx.parameter_sources()?;
            inventory.require_context(ctx)?;
            assert!(inventory.query_values().is_empty());
            assert_eq!(inventory.policy_operands().len(), 1);
            let constant = &inventory.policy_operands()[0];
            assert_eq!(
                constant.kind(),
                SecurityPolicyParameterKind::ConditionConstant
            );
            assert_eq!(
                constant.path(),
                "logicalRules/1/condition/where/args/1/where/args/2/right"
            );
            assert_eq!(constant.field().element_id, "active");
            assert_eq!(constant.literal(), Some(&json!({"boolean":true})));
            assert_eq!(constant.domain()["scalarType"], "boolean");
            assert_eq!(inventory.subject().subject().document_id, "domain");
            assert_eq!(inventory.subject().subject().element_id, "Staff");
            assert_eq!(inventory.subject().key_id(), "pk");
            assert_eq!(inventory.subject().key_fields().len(), 1);
            assert_eq!(inventory.subject().key_fields()[0].element_id, "staffId");
            Ok(())
        },
    )
    .unwrap();
}
#[test]
fn inventory_keeps_context_unvalued_and_transform_literal_exact() {
    with_security_mapping_context(
        &request("SELECT r.resourceId FROM Resource r", true),
        |ctx| {
            let inventory = ctx.parameter_sources()?;
            let operands = inventory.policy_operands();
            assert_eq!(operands.len(), 6);
            assert_eq!(
                operands[0].path(),
                "logicalRules/0/condition/args/0/args/0/left"
            );
            assert_eq!(operands[0].literal(), Some(&json!({"boolean":true})));
            assert_eq!(operands[1].literal(), Some(&json!({"boolean":false})));
            assert_eq!(operands[2].kind(), SecurityPolicyParameterKind::Context);
            assert_eq!(operands[2].literal(), None);
            assert_eq!(operands[2].field().element_id, "active");
            assert_eq!(
                operands[3].path(),
                "logicalRules/0/condition/args/0/args/1/arg/right"
            );
            assert_eq!(
                operands[4].kind(),
                SecurityPolicyParameterKind::TransformationConstant
            );
            assert_eq!(
                operands[4].path(),
                "logicalRules/0/disclosure/0/transformed"
            );
            assert_eq!(operands[4].literal(), Some(&json!({"integerToken":"1e2"})));
            assert_eq!(operands[4].field().element_id, "salary");
            assert_eq!(
                operands[5].kind(),
                SecurityPolicyParameterKind::ConditionConstant
            );
            Ok(())
        },
    )
    .unwrap();
}
#[test]
fn inventory_keeps_query_occurrences_separate_from_policy_and_host_subject() {
    with_security_mapping_context(
        &request(
            "SELECT r.resourceId FROM Resource r WHERE r.resourceId='RA'",
            false,
        ),
        |ctx| {
            let inventory = ctx.parameter_sources()?;
            assert_eq!(inventory.query_values().len(), 1);
            let value = &inventory.query_values()[0];
            assert_eq!(value.path(), "filters/0/right");
            match value.value() {
                SecurityQueryParameterValue::Scalar(
                    weft_core::application_ir::Value::Literal { value, .. },
                )
                | SecurityQueryParameterValue::Exact(
                    weft_core::application_ir::ExactValue::Literal { value, .. },
                ) => assert_eq!(value, "RA"),
                _ => panic!("Original literal occurrence required"),
            };
            assert_eq!(inventory.policy_operands().len(), 1);
            assert_eq!(inventory.subject().subject().element_id, "Staff");
            Ok(())
        },
    )
    .unwrap();
}
#[test]
fn matching_bytes_in_a_new_owner_context_cannot_substitute_inventory_objects() {
    let raw = request("SELECT r.resourceId FROM Resource r", false);
    with_security_mapping_context(&raw, |original| {
        let inventory = original.parameter_sources()?;
        with_security_mapping_context(&raw, |other| {
            assert_eq!(
                original.execution_query_context_sha256(),
                other.execution_query_context_sha256()
            );
            assert_eq!(
                inventory.require_context(other).unwrap_err().code,
                "WFT-SECURITY-PARAMETER-SOURCE"
            );
            Ok(())
        })?;
        inventory.require_context(original)?;
        Ok(())
    })
    .unwrap();
}
#[test]
fn repeated_query_values_keep_distinct_occurrence_paths() {
    with_security_mapping_context(
        &request(
            "SELECT r.resourceId FROM Resource r WHERE r.resourceId='RA' AND r.resourceId='RA'",
            false,
        ),
        |ctx| {
            let inventory = ctx.parameter_sources()?;
            assert_eq!(
                inventory
                    .query_values()
                    .iter()
                    .map(|v| v.path())
                    .collect::<Vec<_>>(),
                vec!["filters/0/right", "filters/1/right"]
            );
            assert_eq!(inventory.policy_operands().len(), 1);
            Ok(())
        },
    )
    .unwrap();
}
#[test]
fn explicit_null_policy_constant_is_distinct_from_an_unvalued_context() {
    let mut r: Value =
        serde_json::from_str(&request("SELECT r.resourceId FROM Resource r", false)).unwrap();
    let mut document: Value =
        serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    document["modules"][0]["elements"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|f| f["id"] == "active")
        .unwrap()["nullability"] = json!("absent-allowed");
    let doc = document.to_string();
    r["modules"][0]["documentJson"] = json!(doc);
    r["modules"][0]["pin"]["sha256"] = json!(sha256(doc.as_bytes()));
    let mut policy: Value = serde_json::from_str(r["policyJson"].as_str().unwrap()).unwrap();
    policy["rules"][1]["condition"]["where"]["args"][1]["where"]["args"][2]["right"]["value"] =
        Value::Null;
    r["policyJson"] = json!(policy.to_string());
    let mut profile: Value = serde_json::from_str(r["queryProfileJson"].as_str().unwrap()).unwrap();
    profile["modelPins"][0] = r["modules"][0]["pin"].clone();
    profile["policySha256"] = json!(sha256(policy.to_string().as_bytes()));
    r["queryProfileJson"] = json!(profile.to_string());
    with_security_mapping_context(&r.to_string(), |ctx| {
        let inventory = ctx.parameter_sources()?;
        assert_eq!(inventory.policy_operands().len(), 1);
        let source = &inventory.policy_operands()[0];
        assert_eq!(source.literal(), Some(&Value::Null));
        assert_eq!(source.domain()["nullability"], "absent-allowed");
        Ok(())
    })
    .unwrap();
}
