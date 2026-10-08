//! Pure build-time composition. Hosts opt into trusted candidate adapters.
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
