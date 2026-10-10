//! Pure build-time composition. Hosts explicitly select trusted adapter versions.
#[cfg(any(
    all(feature="ashlar-databricks-left-join", any(feature="ashlar-databricks-count-having", feature="ashlar-databricks-count-distinct",feature="ashlar-databricks-arithmetic",feature="ashlar-databricks-candidate",feature="ashlar-databricks-qualified",feature="ashlar-databricks-mathematical-integer")),
    all(
        feature = "truss-postgresql-candidate",
        feature = "truss-postgresql-qualified"
    ),
    all(
        feature = "ashlar-databricks-candidate",
        feature = "ashlar-databricks-qualified"
    ),
    all(feature = "ashlar-databricks-arithmetic", feature = "ashlar-databricks-candidate"),
    all(feature = "ashlar-databricks-arithmetic", feature = "ashlar-databricks-qualified"),
    all(feature = "ashlar-databricks-arithmetic", feature = "ashlar-databricks-mathematical-integer"),
    all(feature = "ashlar-databricks-count-having", any(feature = "ashlar-databricks-count-distinct",feature = "ashlar-databricks-arithmetic",feature = "ashlar-databricks-candidate",feature = "ashlar-databricks-qualified",feature = "ashlar-databricks-mathematical-integer")),
    all(feature = "ashlar-databricks-count-distinct", any(feature = "ashlar-databricks-arithmetic",feature = "ashlar-databricks-candidate",feature = "ashlar-databricks-qualified",feature = "ashlar-databricks-mathematical-integer"))
))]
compile_error!("Choose either candidate or qualified registration for each backend; versions never silently override or fall back");
pub fn compile_json(request: &str) -> String {
    #[cfg(feature = "test-third")]
    let registry = weft_backend_probe::compile_fixture_registry();
    #[cfg(not(feature = "test-third"))]
    let registry = weft_core::backend::Registry::default();
    #[cfg(feature = "truss-postgresql-candidate")]
    let registry = {
        let mut registry = registry;
        registry
            .register(weft_postgresql::candidate::Candidate)
            .expect("build-time candidate backend registration must be unique");
        registry
    };
    #[cfg(feature = "ashlar-databricks-candidate")]
    let registry = {
        let mut registry = registry;
        registry
            .register(weft_databricks::candidate::Candidate)
            .expect("build-time candidate backend registration must be unique");
        registry
    };
    #[cfg(feature = "truss-postgresql-qualified")]
    let registry = {
        let mut registry = registry;
        registry
            .register(weft_postgresql::qualified_profile::Qualified)
            .expect("build-time qualified registration must be unique");
        registry
    };
    #[cfg(feature = "ashlar-databricks-qualified")]
    let registry = {
        let mut registry = registry;
        registry
            .register(weft_databricks::qualified_profile::Qualified)
            .expect("build-time qualified registration must be unique");
        registry
    };
    #[cfg(feature = "ashlar-databricks-mathematical-integer")]
    let registry = {
        let mut registry = registry;
        registry.register(weft_databricks::mathematical_integer::MathematicalInteger)
            .expect("explicit mathematical integer backend registration must be unique");
        registry
    };
    #[cfg(feature = "ashlar-databricks-arithmetic")]
    let registry = {
        let mut registry = registry;
        registry.register(weft_databricks::arithmetic::Arithmetic)
            .expect("explicit arithmetic backend registration must be unique");
        registry
    };
    #[cfg(feature = "ashlar-databricks-count-distinct")]
    let registry = {
        let mut registry=registry;
        registry.register(weft_databricks::arithmetic::Arithmetic).expect("prior explicit arithmetic version remains unique");
        registry.register(weft_databricks::count_distinct::CountDistinct).expect("explicit distinct-count version remains unique");
        registry
    };

    #[cfg(feature = "ashlar-databricks-count-having")]
    let registry = {
        let mut registry=registry;
        registry.register(weft_databricks::arithmetic::Arithmetic).expect("prior explicit arithmetic version remains unique");
        registry.register(weft_databricks::count_distinct::CountDistinct)
            .expect("explicit required-count backend registration must be unique");
        registry
            .register(weft_databricks::count_having::CountHaving).expect("explicit distinct-count version remains unique");
        registry
    };
    #[cfg(feature = "ashlar-databricks-left-join")]
    let registry = {
        let mut registry=registry;
        registry.register(weft_databricks::arithmetic::Arithmetic).expect("old arithmetic identity unique");
        registry.register(weft_databricks::count_distinct::CountDistinct).expect("old count identity unique");
        registry.register(weft_databricks::count_having::CountHaving).expect("old optional-count identity unique");
        registry.register(weft_databricks::left_join::LeftJoin).expect("explicit LEFT identity unique");
        registry
    };
    #[cfg(feature = "test-original")]
    {
        let mut fallback = Some(registry);
        let mut factory =
            |catalog: &weft_core::model::Catalog,
             plan: weft_core::backend::Plan<'_>,
             target: weft_core::compile::CompositionInput<'_>| {
                if target.backend_id == "truss.postgresql.original" {
                    weft_postgresql::conformance_original::registry(catalog, plan, target)
                } else {
                    Ok(fallback.take().expect("request-local factory called once"))
                }
            };
        return weft_core::compile::Compiler::default()
            .compile_json_with_factory(request, &mut factory);
    }
    #[cfg(not(feature = "test-original"))]
    weft_core::compile::Compiler { registry }.compile_json(request)
}

#[cfg(all(test, feature = "test-original"))]
mod original_tests {
    #[test]
    fn native_compound_runtime_composes_every_request_and_refuses_unsupported_cuts() {
        let cases=[
include_str!("../../../tests/truss-postgresql/fixtures/original-address-compile-transport.json"),
include_str!("../../../tests/truss-postgresql/fixtures/original-cyclic-compile-transport.json"),
include_str!("../../../tests/truss-postgresql/fixtures/original-map-compile-transport.json"),
include_str!("../../../tests/truss-postgresql/fixtures/original-nested-sequence-compile-transport.json"),
include_str!("../../../tests/truss-postgresql/fixtures/original-numeric-address-compile-transport.json"),
include_str!("../../../tests/truss-postgresql/fixtures/original-numeric-map-compile-transport.json"),
include_str!("../../../tests/truss-postgresql/fixtures/original-tags-compile-transport.json"),
 ];
        for raw in cases {
            let fixture: serde_json::Value = serde_json::from_str(raw).unwrap();
            let request = fixture["request"].to_string();
            let compiled: serde_json::Value =
                serde_json::from_str(&super::compile_json(&request)).unwrap();
            assert_eq!(compiled, fixture["response"]);
            assert_eq!(super::compile_json(&request), super::compile_json(&request));
            let mut unsupported = fixture["request"].clone();
            let binding = unsupported["target"]["bindingJson"]
                .as_str()
                .unwrap()
                .to_string()
                + " ";
            unsupported["target"]["bindingSha256"] =
                serde_json::json!(weft_core::json::sha256(binding.as_bytes()));
            unsupported["target"]["bindingJson"] = serde_json::json!(binding);
            let refused: serde_json::Value =
                serde_json::from_str(&super::compile_json(&unsupported.to_string())).unwrap();
            assert_eq!(refused["status"], "blocked");
            assert!(refused.get("sql").is_none());
            let mut disabled = fixture["request"].clone();
            disabled["options"]["allowCandidate"] = serde_json::json!(false);
            let refused: serde_json::Value =
                serde_json::from_str(&super::compile_json(&disabled.to_string())).unwrap();
            assert_eq!(refused["status"], "blocked");
            assert!(refused.get("sql").is_none());
        }
    }
}

#[cfg(all(test, feature = "test-original"))]
mod relationship_tests {
    #[test]
    fn original_relationship_runtime_matches_all_public_responses() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-relationship-public-transport.json"
        ))
        .unwrap();
        for case in cases.as_array().unwrap() {
            let request = case["request"].to_string();
            let response = super::compile_json(&request);
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&response).unwrap(),
                case["response"]
            );
            assert_eq!(super::compile_json(&request), response);
        }
    }
}

#[cfg(all(test, feature = "test-original"))]
mod optional_tests {
    #[test]
    fn original_optional_entity_runtime_matches_all_home_responses() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-optional-public-transport.json"
        ))
        .unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 6);
        for case in cases.as_array().unwrap() {
            let request = case["request"].to_string();
            let response = super::compile_json(&request);
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&response).unwrap(),
                case["response"]
            );
            assert_eq!(super::compile_json(&request), response);
        }
    }
}

#[cfg(all(test, feature = "test-original"))]
mod recursive_entity_tests {
    #[test]
    fn original_complete_recursive_entities_match_all_public_responses() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-entity-public-transport.json"
        ))
        .unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 28);
        for case in cases.as_array().unwrap() {
            let request = case["request"].to_string();
            let response = super::compile_json(&request);
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&response).unwrap(),
                case["response"]
            );
            assert_eq!(super::compile_json(&request), response);
        }
    }
}

#[cfg(all(test, feature = "test-original"))]
mod signed_property_tests {
    #[test]
    fn original_signed_properties_match_complete_public_responses() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-signed-public-transport.json"
        ))
        .unwrap();
        for case in cases.as_array().unwrap() {
            let request = case["request"].to_string();
            let raw = super::compile_json(&request);
            let response: serde_json::Value = serde_json::from_str(&raw).unwrap();
            assert_eq!(response, case["response"]);
            assert_eq!(super::compile_json(&request), raw);
        }
    }
}

#[cfg(all(test, feature = "test-original"))]
mod boolean_property_tests {
    #[test]
    fn original_boolean_properties_match_full_public_responses_and_refuse_sum() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-boolean-public-transport.json"
        ))
        .unwrap();
        for case in cases.as_array().unwrap() {
            let request = case["request"].to_string();
            let raw = super::compile_json(&request);
            let response: serde_json::Value = serde_json::from_str(&raw).unwrap();
            assert_eq!(response, case["response"]);
            assert_eq!(super::compile_json(&request), raw);
            let mut invalid = case["request"].clone();
            invalid["sql"] = serde_json::json!("SELECT SUM(c.id) AS total FROM Customer c");
            invalid.as_object_mut().unwrap().remove("readProfile");
            let refused: serde_json::Value =
                serde_json::from_str(&super::compile_json(&invalid.to_string())).unwrap();
            assert_eq!(refused["status"], "blocked");
            assert!(refused.get("sql").is_none());
        }
    }
}

#[cfg(all(test, feature = "test-original"))]
mod boolean_sequence_tests {
    #[test]
    fn original_boolean_sequence_matches_complete_public_response() {
        let case:serde_json::Value=serde_json::from_str(include_str!("../../../tests/truss-postgresql/fixtures/original-boolean-sequence-compile-transport.json")).unwrap();
        let request = case["request"].to_string();
        let raw = super::compile_json(&request);
        let response: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(response, case["response"]);
        assert_eq!(super::compile_json(&request), raw);
    }
}

#[cfg(all(test, feature = "test-original"))]
mod multi_recursive_entity_tests {
    #[test]
    fn original_multi_recursive_entities_match_complete_public_responses() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-multi-recursive-entity-public.json"
        ))
        .unwrap();
        for case in cases.as_array().unwrap() {
            let request = case["request"].to_string();
            let raw = super::compile_json(&request);
            let response: serde_json::Value = serde_json::from_str(&raw).unwrap();
            assert_eq!(response, case["response"]);
            assert_eq!(super::compile_json(&request), raw);
        }
    }
}

/// Explicit conformance instrumentation, available only in test-original builds.
#[cfg(feature = "test-original")]
pub fn compile_json_with_conformance_configuration(request: &str, configuration: &str) -> String {
    let mut factory = |catalog: &weft_core::model::Catalog,
                       _: weft_core::backend::Plan<'_>,
                       target: weft_core::compile::CompositionInput<'_>| {
        weft_postgresql::conformance_original::registry_with_configuration(
            catalog,
            target,
            configuration,
        )
    };
    weft_core::compile::Compiler::default().compile_json_with_factory(request, &mut factory)
}

#[cfg(all(test, feature = "test-original"))]
mod supplied_configuration_tests {
    #[test]
    fn explicit_host_configuration_admits_unlisted_binding_and_preserves_refusals() {
        use serde_json::{json, Value};
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-address-compile-transport.json"
        ))
        .unwrap();
        let configuration = include_str!(
            "../../../tests/truss-postgresql/fixtures/original-address-composition.json"
        );
        let mut request = fixture["request"].clone();
        let binding = request["target"]["bindingJson"]
            .as_str()
            .unwrap()
            .to_owned()
            + " ";
        request["target"]["bindingJson"] = json!(binding);
        request["target"]["bindingSha256"] = json!(weft_core::json::sha256(binding.as_bytes()));
        let fixed: Value =
            serde_json::from_str(&super::compile_json(&request.to_string())).unwrap();
        assert_eq!(fixed["status"], "blocked");
        let result =
            super::compile_json_with_conformance_configuration(&request.to_string(), configuration);
        let compiled: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(compiled["status"], "compiled", "{compiled}");
        assert_eq!(compiled["sql"], fixture["response"]["sql"]);
        assert_eq!(compiled["parameters"], fixture["response"]["parameters"]);
        assert_eq!(
            result,
            super::compile_json_with_conformance_configuration(&request.to_string(), configuration)
        );
        for invalid in [
            "{}".to_owned(),
            configuration.to_owned() + "{}",
            " ".repeat(4 * 1024 * 1024 + 1),
        ] {
            let refused: Value = serde_json::from_str(
                &super::compile_json_with_conformance_configuration(&request.to_string(), &invalid),
            )
            .unwrap();
            assert_eq!(refused["status"], "blocked");
            assert!(refused.get("sql").is_none());
        }
        request["options"]["allowCandidate"] = json!(false);
        let refused: Value =
            serde_json::from_str(&super::compile_json_with_conformance_configuration(
                &request.to_string(),
                configuration,
            ))
            .unwrap();
        assert_eq!(refused["status"], "blocked");
        assert!(refused.get("sql").is_none());
    }
}

#[cfg(test)]
mod ashlar_tests {
    #[test]
    fn ashlar_feature_controls_registration_and_retains_public_artifact() {
        let case: serde_json::Value = serde_json::from_str(include_str!(
            "../../../docs/helix/04-build/evidence/B-006-cross-module-native/compile.json"
        ))
        .unwrap();
        let request = case["request"].to_string();
        let raw = super::compile_json(&request);
        let response: serde_json::Value = serde_json::from_str(&raw).unwrap();
        #[cfg(feature = "ashlar-databricks-candidate")]
        {
            // Retain the original native receipt. Only the explicitly obsolete
            // Spark observation may differ from the current compiler artifact.
            let mut expected = case["response"].clone();
            let publication = expected["obligations"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|o| o["id"] == "ashlar.candidate.publication")
                .unwrap();
            assert_eq!(
                publication["parameters"]["nativeProfile"]
                    .as_object_mut()
                    .unwrap()
                    .remove("versionReported"),
                Some(serde_json::json!("4.2.0 zero build hash"))
            );
            assert_eq!(response, expected);
        }
        #[cfg(not(feature = "ashlar-databricks-candidate"))]
        {
            assert_eq!(response["status"], "blocked");
            assert!(response.get("sql").is_none());
        }
        assert_eq!(raw, super::compile_json(&request));
        let mut disabled = case["request"].clone();
        disabled["options"]["allowCandidate"] = serde_json::json!(false);
        let refused: serde_json::Value =
            serde_json::from_str(&super::compile_json(&disabled.to_string())).unwrap();
        assert_eq!(refused["status"], "blocked");
        assert!(refused.get("sql").is_none());
    }
}

#[cfg(all(
    test,
    feature = "truss-postgresql-qualified",
    feature = "ashlar-databricks-qualified"
))]
mod qualified_tests {
    #[test]
    fn public_composition_requires_exact_versions_and_supports_without_candidate_opt_in() {
        let inputs = [
            (
                include_str!("../../../tests/truss-postgresql/fixtures/compiler-cases.json"),
                "pg17.9-qualified-fixtures",
            ),
            (
                include_str!(
                    "../../../docs/helix/04-build/evidence/B-006-cross-module-native/compile.json"
                ),
                "dbsql2026.39-qualified",
            ),
        ];
        for (raw, profile) in inputs {
            let cases: serde_json::Value = serde_json::from_str(raw).unwrap();
            let mut request = if cases.is_array() {
                cases[0]["request"].clone()
            } else {
                cases["request"].clone()
            };
            request["target"]["backendVersion"] = serde_json::json!("0.1.0-qualified");
            request["target"]["targetProfile"] = serde_json::json!(profile);
            request["options"]["allowCandidate"] = serde_json::json!(false);
            let response: serde_json::Value =
                serde_json::from_str(&super::compile_json(&request.to_string())).unwrap();
            assert_eq!(response["status"], "compiled", "{response}");
            assert_eq!(response["qualification"]["status"], "conformance-verified");
            assert!(response["qualification"]["operations"]
                .as_array()
                .unwrap()
                .iter()
                .all(|op| op["assessment"]["status"] == "supported"));
            request["target"]["backendVersion"] = serde_json::json!("0.1.0-candidate");
            let refused: serde_json::Value =
                serde_json::from_str(&super::compile_json(&request.to_string())).unwrap();
            assert_eq!(refused["status"], "blocked");
            assert_eq!(refused["diagnostics"][0]["code"], "WFT-BACKEND-VERSION");
            assert!(refused.get("sql").is_none());
        }
    }
}
