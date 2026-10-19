// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{json, Value};
use weft_core::{json::sha256, security_mapping_transport::with_security_mapping_context};
const REQUEST: &str =
    include_str!("../../../docs/helix/03-test/fixtures/security-graph-source/mapping-request.json");
#[test]
fn different_queries_retain_source_identity_but_distinct_query_lineage() {
    with_security_mapping_context(REQUEST, |first| {
        let source = first.source_context()?;
        let query = first.execution_query_context_sha256();
        for sql in [
            "SELECT r.resourceId AS first FROM Resource r",
            "SELECT r.salary, r.resourceId FROM Resource r",
            "SELECT r.resourceId FROM Resource r WHERE r.resourceId='RA'",
            "SELECT COUNT(*) FROM Resource r",
        ] {
            let mut request: Value = serde_json::from_str(REQUEST).unwrap();
            request["sql"] = json!(sql);
            with_security_mapping_context(&request.to_string(), |other| {
                source.require_sources(other)?;
                let next = other.source_context()?;
                assert_eq!(source.json(), next.json());
                assert_eq!(source.sha256(), next.sha256());
                // Query lineage changes while exact source components remain fixed.
                assert_ne!(query, other.execution_query_context_sha256());
                Ok(())
            })?;
        }
        Ok(())
    })
    .unwrap();
}
#[test]
fn capsule_preserves_exact_opaque_source_bytes_and_contains_no_query_or_credential() {
    with_security_mapping_context(REQUEST, |ctx| {
        let source = ctx.source_context()?;
        assert_eq!(source.sha256(), sha256(source.json().as_bytes()));
        let value: Value = serde_json::from_str(source.json()).unwrap();
        assert_eq!(
            value["version"],
            "weft.security.source-context/0.1-candidate"
        );
        assert_eq!(value["securityReady"], false);
        assert_eq!(value.as_object().unwrap().len(), 10);
        assert_eq!(
            value["policyJson"],
            ctx.logical_plan().source().policy_json()
        );
        assert_eq!(
            value["ontologyJson"],
            ctx.logical_plan().source().ontology_json()
        );
        assert_eq!(value["bindingJson"], ctx.binding_json());
        assert_eq!(
            value["queryProfileJson"],
            serde_json::from_str::<Value>(REQUEST).unwrap()["queryProfileJson"]
        );
        assert_eq!(
            value["modules"],
            serde_json::to_value(ctx.catalog().inputs()).unwrap()
        );
        for excluded in [
            "sql",
            "parameters",
            "subject",
            "principal",
            "epoch",
            "permit",
            "resultContract",
        ] {
            assert!(value.get(excluded).is_none());
        }
        Ok(())
    })
    .unwrap();
}
#[test]
fn changed_source_bytes_are_not_the_same_capsule_even_with_equal_parsed_meaning() {
    with_security_mapping_context(REQUEST, |original| {
        let source = original.source_context()?;
        for field in [
            "policyJson",
            "ontologyJson",
            "bindingJson",
            "queryProfileJson",
        ] {
            let mut request: Value = serde_json::from_str(REQUEST).unwrap();
            request[field] = json!(format!("{} ", request[field].as_str().unwrap()));
            let mut profile: Value =
                serde_json::from_str(request["queryProfileJson"].as_str().unwrap()).unwrap();
            match field {
                "policyJson" => {
                    profile["policySha256"] =
                        json!(sha256(request[field].as_str().unwrap().as_bytes()))
                }
                "ontologyJson" => {
                    profile["ontologySha256"] =
                        json!(sha256(request[field].as_str().unwrap().as_bytes()))
                }
                "bindingJson" => {
                    profile["binding"]["sha256"] =
                        json!(sha256(request[field].as_str().unwrap().as_bytes()))
                }
                _ => {}
            }
            if field != "queryProfileJson" {
                request["queryProfileJson"] = json!(profile.to_string());
            }
            with_security_mapping_context(&request.to_string(), |other| {
                assert_eq!(
                    source.require_sources(other).unwrap_err().code,
                    "WFT-SECURITY-SOURCE-CONTEXT"
                );
                assert_ne!(source.sha256(), other.source_context()?.sha256());
                Ok(())
            })?;
        }
        Ok(())
    })
    .unwrap();
}
#[test]
fn parameter_value_changes_only_query_context_and_cannot_authorize_compilation() {
    let mut first: Value = serde_json::from_str(REQUEST).unwrap();
    first["sql"] = json!("SELECT r.resourceId FROM Resource r WHERE r.resourceId=:id");
    first["parameters"] = json!({"id":{"family":"string","value":"RA"}});
    with_security_mapping_context(&first.to_string(), |original| {
        let source = original.source_context()?;
        let query = original.execution_query_context_sha256();
        let mut changed = first.clone();
        changed["parameters"]["id"]["value"] = json!("RB");
        with_security_mapping_context(&changed.to_string(), |other| {
            source.require_sources(other)?;
            assert_ne!(query, other.execution_query_context_sha256());
            Ok(())
        })?;
        let refusal: Value = serde_json::from_str(
            &weft_core::compile::Compiler::default().compile_json(source.json()),
        )
        .unwrap();
        assert_eq!(refusal["status"], "blocked");
        assert!(
            weft_core::security_backend::validate_security_manifest_json(source.json()).is_err()
        );
        assert!(weft_core::backend::validate_manifest_json(source.json()).is_err());
        assert!(refusal["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["severity"] == "error"));
        for excluded in ["sql", "parameters", "logicalPlan", "security"] {
            assert!(refusal.get(excluded).is_none());
        }
        Ok(())
    })
    .unwrap();
}
#[test]
fn changed_model_byte_pin_is_a_distinct_source_even_when_model_meaning_is_equal() {
    with_security_mapping_context(REQUEST, |original| {
        let source = original.source_context()?;
        let mut changed: Value = serde_json::from_str(REQUEST).unwrap();
        let bytes = format!(
            "{} ",
            changed["modules"][0]["documentJson"].as_str().unwrap()
        );
        changed["modules"][0]["documentJson"] = json!(bytes);
        changed["modules"][0]["pin"]["sha256"] = json!(sha256(bytes.as_bytes()));
        let mut profile: Value =
            serde_json::from_str(changed["queryProfileJson"].as_str().unwrap()).unwrap();
        profile["modelPins"][0] = changed["modules"][0]["pin"].clone();
        changed["queryProfileJson"] = json!(profile.to_string());
        with_security_mapping_context(&changed.to_string(), |other| {
            assert!(source.require_sources(other).is_err());
            assert_ne!(source.sha256(), other.source_context()?.sha256());
            Ok(())
        })?;
        Ok(())
    })
    .unwrap();
}
