// SPDX-License-Identifier: MIT OR Apache-2.0
//! Pure build-time composition. Hosts explicitly select trusted adapter versions.
/// Original ontology Subject/Record/Key source selection only; no authority.
pub fn inspect_security_mapping_subject_key_json(request: &str, selection: &str) -> String {
    weft_core::security_mapping_transport::inspect_security_mapping_subject_key_json(
        request, selection,
    )
}
/// Source-bound logical mapping input only; no backend registration or permission.
pub fn prepare_security_mapping_json(request: &str) -> String {
    weft_core::security_mapping_transport::prepare_security_mapping_json(request)
}
#[cfg(feature = "ashlar-databricks-paths")]
pub mod paths;
#[cfg(feature = "ashlar-databricks-paths-keys")]
pub mod paths_keys;
#[cfg(any(
    all(
        feature = "ashlar-databricks-left-join",
        any(
            feature = "ashlar-databricks-count-having",
            feature = "ashlar-databricks-count-distinct",
            feature = "ashlar-databricks-arithmetic",
            feature = "ashlar-databricks-candidate",
            feature = "ashlar-databricks-qualified",
            feature = "ashlar-databricks-mathematical-integer"
        )
    ),
    all(
        feature = "truss-postgresql-candidate",
        feature = "truss-postgresql-qualified"
    ),
    all(
        feature = "ashlar-databricks-compatible-revisions-candidate",
        any(
            feature = "ashlar-databricks-candidate",
            feature = "ashlar-databricks-qualified",
            feature = "ashlar-databricks-arithmetic",
            feature = "ashlar-databricks-count-having",
            feature = "ashlar-databricks-count-distinct",
            feature = "ashlar-databricks-left-join",
            feature = "ashlar-databricks-mathematical-integer",
            feature = "ashlar-databricks-paths",
            feature = "ashlar-databricks-paths-keys",
            feature = "ashlar-databricks-implicit-arithmetic"
        )
    ),
    all(
        feature = "ashlar-databricks-candidate",
        feature = "ashlar-databricks-qualified"
    ),
    all(
        feature = "ashlar-databricks-arithmetic",
        feature = "ashlar-databricks-candidate"
    ),
    all(
        feature = "ashlar-databricks-arithmetic",
        feature = "ashlar-databricks-qualified"
    ),
    all(
        feature = "ashlar-databricks-arithmetic",
        feature = "ashlar-databricks-mathematical-integer"
    ),
    all(
        feature = "ashlar-databricks-count-having",
        any(
            feature = "ashlar-databricks-count-distinct",
            feature = "ashlar-databricks-arithmetic",
            feature = "ashlar-databricks-candidate",
            feature = "ashlar-databricks-qualified",
            feature = "ashlar-databricks-mathematical-integer"
        )
    ),
    all(
        feature = "ashlar-databricks-count-distinct",
        any(
            feature = "ashlar-databricks-arithmetic",
            feature = "ashlar-databricks-candidate",
            feature = "ashlar-databricks-qualified",
            feature = "ashlar-databricks-mathematical-integer"
        )
    )
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
    #[cfg(feature = "ashlar-databricks-compatible-revisions-candidate")]
    let registry = {
        let mut registry = registry;
        registry
            .register(weft_databricks::compatible_profile::CompatibleRevisions)
            .expect("explicit compatible revision registration must be unique");
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
        registry
            .register(weft_databricks::mathematical_integer::MathematicalInteger)
            .expect("explicit mathematical integer backend registration must be unique");
        registry
    };
    #[cfg(feature = "ashlar-databricks-arithmetic")]
    let registry = {
        let mut registry = registry;
        registry
            .register(weft_databricks::arithmetic::Arithmetic)
            .expect("explicit arithmetic backend registration must be unique");
        registry
    };
    #[cfg(feature = "ashlar-databricks-count-distinct")]
    let registry = {
        let mut registry = registry;
        registry
            .register(weft_databricks::arithmetic::Arithmetic)
            .expect("prior explicit arithmetic version remains unique");
        registry
            .register(weft_databricks::count_distinct::CountDistinct)
            .expect("explicit distinct-count version remains unique");
        registry
    };

    #[cfg(feature = "ashlar-databricks-count-having")]
    let registry = {
        let mut registry = registry;
        registry
            .register(weft_databricks::arithmetic::Arithmetic)
            .expect("prior explicit arithmetic version remains unique");
        registry
            .register(weft_databricks::count_distinct::CountDistinct)
            .expect("explicit required-count backend registration must be unique");
        registry
            .register(weft_databricks::count_having::CountHaving)
            .expect("explicit distinct-count version remains unique");
        registry
    };
    #[cfg(feature = "ashlar-databricks-left-join")]
    let registry = {
        let mut registry = registry;
        registry
            .register(weft_databricks::arithmetic::Arithmetic)
            .expect("old arithmetic identity unique");
        registry
            .register(weft_databricks::count_distinct::CountDistinct)
            .expect("old count identity unique");
        registry
            .register(weft_databricks::count_having::CountHaving)
            .expect("old optional-count identity unique");
        registry
            .register(weft_databricks::left_join::LeftJoin)
            .expect("explicit LEFT identity unique");
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
            // Historical native evidence remains immutable. This source-owned
            // current-release baseline was produced by frozen R22; both R22 and
            // this successor reproduce the old golden's missing navigation profile.
            let current: serde_json::Value = serde_json::from_str(include_str!(
                "../../../tests/engine-catalog/B006-current-r22-baseline.json"
            ))
            .unwrap();
            assert_eq!(
                current["sourceCommit"],
                "5605d358650468d651f69ff29c40e986a0359a5b"
            );
            assert_eq!(
                current["requestSha256"],
                weft_core::json::sha256(
                    weft_core::mutation::canonical(&case["request"]).as_bytes()
                )
            );
            assert_eq!(response, current["response"]);
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
                    "../../../tests/ashlar-databricks/fixtures/current010-cross-module-qualified-request.json"
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
            // Keep the public compiler case on the current compiler contract; the
            // cross-module case is a current-UMF successor to frozen B-006 evidence.
            request["interfaceVersion"] = serde_json::json!("weft-compile/0.2.0");
            request["dialect"] = serde_json::json!("weft-sql/0.2.0");
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

#[cfg(feature = "ashlar-databricks-implicit-arithmetic")]
pub mod implicit_arithmetic;

/// Explicit production PG18 original-definition composition; generic transport stays unchanged.
#[cfg(feature = "truss-postgresql-pg18")]
pub fn compile_truss_postgresql_json(request: &str, composition: &str) -> String {
    weft_postgresql::pg18::compile_json(request, composition)
}

#[cfg(feature = "truss-postgresql-pg18")]
pub fn compile_truss_runtime_json(request: &str) -> String {
    weft_postgresql::truss_runtime::compile_json(request)
}

#[cfg(feature = "truss-postgresql-pg18")]
pub fn compile_truss_runtime_page_json(
    request: &str,
    page_size: u16,
    after: Option<Vec<String>>,
) -> String {
    weft_postgresql::truss_runtime::compile_page_json(request, page_size, after)
}

/// Host-configured mutation compilation; no execution authority.
pub use weft_core::mutation::MutationRegistry;

/// Install the current original Truss renderer into a host-owned mutation registry.
/// No request can choose executable SQL or load a renderer.
#[cfg(feature = "truss-postgresql-pg18")]
pub fn install_truss_mutation_backend(
    registry: &mut MutationRegistry,
    configuration: &str,
) -> weft_core::mutation::MResult<usize> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Configuration {
        model: weft_core::mutation::Artifact,
        model_pin: weft_core::ir::ModelPin,
        binding: weft_core::mutation::Artifact,
        evidence: Vec<String>,
    }
    let fail = || weft_core::mutation::MutationDiagnostic {
        code: "WFT-MUTATION-REGISTRATION".into(),
        phase: "input".into(),
        path: String::new(),
        message: "Bounded original Truss registration required".into(),
    };
    if configuration.len() > 4194304 {
        return Err(fail());
    }
    let value = weft_core::json::checked_json(configuration).map_err(|_| fail())?;
    let c: Configuration = serde_json::from_value(value).map_err(|_| fail())?;
    registry.install_selection_backend(Box::new(
        weft_postgresql::truss_mutation::CurrentTrussMutationBackend::new(
            c.model,
            c.binding,
            c.model_pin,
            c.evidence,
        )?,
    ))
}
/// Lower an original selection through a previously installed host registration.
pub fn lower_mutation_selection_json(
    registry: &MutationRegistry,
    handle: usize,
    request: &str,
) -> weft_core::mutation::MResult<String> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        selection: weft_core::mutation::Artifact,
        model: weft_core::mutation::Artifact,
        binding: weft_core::mutation::Artifact,
        max_items: u32,
    }
    let fail = || weft_core::mutation::MutationDiagnostic {
        code: "WFT-MUTATION-LOWERING".into(),
        phase: "input".into(),
        path: String::new(),
        message: "Bounded original selection request required".into(),
    };
    if request.len() > 8388608 {
        return Err(fail());
    }
    let value = weft_core::json::checked_json(request).map_err(|_| fail())?;
    let r: Request = serde_json::from_value(value).map_err(|_| fail())?;
    serde_json::to_string(&registry.lower_selection(
        handle,
        &r.selection,
        &r.model,
        &r.binding,
        r.max_items,
    )?)
    .map_err(|_| fail())
}

/// Separate pure relationship candidate transport. Never registers a backend or grants authority.
#[cfg(feature = "truss-security-relationship-candidate")]
pub fn prepare_truss_security_relationship_candidate_json(
    request: &str,
) -> weft_core::error::Result<String> {
    weft_postgresql::security_relational_transport::prepare_guarded_relationship_candidate(request)
        .map(|p| p.json().to_owned())
}
/// Reconstruct original producer bytes and inspect supplied cells. Native completeness and release remain host obligations.
#[cfg(feature = "truss-security-relationship-candidate")]
pub fn inspect_truss_security_relationship_result_json(
    request: &str,
) -> weft_core::error::Result<String> {
    weft_postgresql::security_relational_transport::inspect_guarded_relationship_result(request)
        .map(|v| v.to_string())
}

/// Opt-in original source read intents only. Never policy/native admission.
#[cfg(feature = "application-lineage-candidate")]
pub fn prepare_application_lineage_json(request: &str) -> weft_core::error::Result<String> {
    prepare_application_lineage(request).map(|p| p.json().to_owned())
}
/// Original in-process issued preparation. Possession is not native authority.
#[cfg(feature = "application-lineage-candidate")]
pub fn prepare_application_lineage(
    request: &str,
) -> weft_core::error::Result<ApplicationLineagePreparation> {
    weft_core::application_lineage::prepare(request).map(|inner| ApplicationLineagePreparation {
        inner: std::sync::Arc::new(inner),
    })
}
/// Issued only by the actual core producer. No caller report constructor or serde.
/// ```compile_fail
/// use weft_runtime::ApplicationLineagePreparation;
/// fn forge(inner: weft_core::application_lineage::ApplicationLineage) -> ApplicationLineagePreparation {
///     ApplicationLineagePreparation { inner }
/// }
/// ```
#[cfg(feature = "application-lineage-candidate")]
pub struct ApplicationLineagePreparation {
    inner: std::sync::Arc<weft_core::application_lineage::ApplicationLineage>,
}
#[cfg(feature = "application-lineage-candidate")]
impl ApplicationLineagePreparation {
    pub fn json(&self) -> &str {
        self.inner.json()
    }
    pub fn source_request_json(&self) -> &str {
        self.inner.source_request_json()
    }
    /// A separate issued presence object; .4 never enters the old designation path.
    #[cfg(feature = "current-presence-candidate")]
    pub fn prepare_current_presence(&self) -> weft_core::error::Result<CurrentPresencePreparation> {
        let prepared = weft_core::current_presence::prepare(&self.inner)?;
        Ok(CurrentPresencePreparation {
            original: std::sync::Arc::clone(&self.inner),
            preparation_json: prepared.json().to_owned(),
        })
    }
    pub fn current_query_designations_json(&self) -> weft_core::error::Result<String> {
        weft_core::current_query_designation::derive(&self.inner).map(|d| d.json().to_owned())
    }
    /// Derive both original declarations before issuing a child; no input overrides.
    pub fn prepare_current_query(&self) -> weft_core::error::Result<CurrentQueryPreparation> {
        let designation = weft_core::current_query_designation::derive(&self.inner)?;
        let operands = weft_core::current_query_operands::derive(&designation)?;
        Ok(CurrentQueryPreparation {
            original: std::sync::Arc::clone(&self.inner),
            designation_json: designation.json().to_owned(),
            operand_count: operands.operations().len(),
        })
    }
}
/// Same original immutable core context. Never a native permission or imported report.
/// ```compile_fail
/// use weft_runtime::CurrentQueryPreparation;
/// fn import(raw:&str)->CurrentQueryPreparation{serde_json::from_str(raw).unwrap()}
/// ```
/// Scoped operand borrows cannot escape the callback.
/// ```compile_fail
/// use weft_runtime::CurrentQueryPreparation;
/// use weft_core::current_query_operands::CurrentQueryOperands;
/// fn leak<'a>(p:&'a CurrentQueryPreparation)->&'a CurrentQueryOperands<'a>{
///   p.with_context(|_,_,operands|Ok(operands)).unwrap()
/// }
/// ```
#[cfg(feature = "application-lineage-candidate")]
pub struct CurrentQueryPreparation {
    original: std::sync::Arc<weft_core::application_lineage::ApplicationLineage>,
    designation_json: String,
    operand_count: usize,
}
#[cfg(feature = "application-lineage-candidate")]
impl CurrentQueryPreparation {
    pub fn designation_json(&self) -> &str {
        &self.designation_json
    }
    pub fn operand_count(&self) -> usize {
        self.operand_count
    }
    /// Derive computation inputs from the same actual original typed context.
    pub fn native_request_json(&self) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            weft_core::current_query_designation::native_request_json(operands)
        })
    }

    /// Original object identity only; identical source bytes are not object custody.
    pub fn is_from(&self, original: &ApplicationLineagePreparation) -> bool {
        std::sync::Arc::ptr_eq(&self.original, &original.inner)
    }
    /// Conditional compiler output only. No native issuer, permission or execution.
    #[cfg(feature = "current-postgresql-scalar-candidate")]
    pub fn lower_postgresql_scalar_candidate_json(
        &self,
        binding: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            weft_postgresql::current_scalar::lower(operands, binding)?.json_string()
        })
    }

    /// Conditional nonaggregate SQL with private source row contribution.
    #[cfg(feature = "current-postgresql-scalar-candidate")]
    pub fn lower_postgresql_current_rows_candidate_json(
        &self,
        binding: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            weft_postgresql::current_scalar::lower_rows(operands, binding)?.json_string()
        })
    }
    /// Conditional current SQL with complete private contributor bags.
    #[cfg(feature = "current-postgresql-scalar-candidate")]
    pub fn lower_postgresql_current_contributors_candidate_json(
        &self,
        binding: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            weft_postgresql::current_scalar::lower_contributors(operands, binding)?.json_string()
        })
    }
    /// Conditional SQL for the explicit live-original integer SUM capability.
    #[cfg(feature = "current-postgresql-scalar-candidate")]
    pub fn lower_postgresql_current_sum_candidate_json(
        &self,
        binding: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            weft_postgresql::current_scalar::lower_original_sum(operands, binding)?.json_string()
        })
    }
    /// Conditional Databricks current SQL with complete private contributor bags.
    #[cfg(feature = "current-databricks-scalar-candidate")]
    pub fn lower_databricks_current_contributors_candidate_json(
        &self,
        binding: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            weft_databricks::current_scalar::lower(operands, binding)?.json_string()
        })
    }
    /// Conditional Databricks SQL for the explicit original signed64 SUM context.
    #[cfg(feature = "current-databricks-scalar-candidate")]
    pub fn lower_databricks_current_sum_candidate_json(
        &self,
        binding: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            weft_databricks::current_scalar::lower_original_sum(operands, binding)?.json_string()
        })
    }
    /// Explicit local Spark contributors; no target inference or SUM fallback.
    #[cfg(feature = "current-spark-scalar-candidate")]
    pub fn lower_spark_current_contributors_candidate_json(
        &self,
        binding: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            weft_databricks::current_scalar::lower_spark_contributors(operands, binding)?
                .json_string()
        })
    }
    #[cfg(feature = "current-spark-scalar-candidate")]
    pub fn check_spark_current_contributors_candidate_json(
        &self,
        binding: &str,
        input: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            Ok(
                weft_databricks::current_scalar::check_spark_contributors(
                    operands, binding, input,
                )?
                .to_string(),
            )
        })
        .map_err(|_| {
            weft_core::error::Diagnostic::new(
                "WFT-CURRENT-QUERY-REFINEMENT",
                "capability",
                "Conditional current-query refinement refused",
            )
        })
    }
    /// Conditional result comparison under independently supplied native-host premises.
    #[cfg(feature = "current-query-refinement-candidate")]
    pub fn check_postgresql_current_rows_candidate_json(
        &self,
        binding: &str,
        input: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            Ok(weft_postgresql::current_scalar::check_rows(operands, binding, input)?.to_string())
        })
        .map_err(|_| {
            weft_core::error::Diagnostic::new(
                "WFT-CURRENT-QUERY-REFINEMENT",
                "capability",
                "Conditional current-query refinement refused",
            )
        })
    }
    /// Conditional result comparison under independently supplied native-host premises.
    #[cfg(feature = "current-query-refinement-candidate")]
    pub fn check_postgresql_current_contributors_candidate_json(
        &self,
        binding: &str,
        input: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            Ok(
                weft_postgresql::current_scalar::check_contributors(operands, binding, input)?
                    .to_string(),
            )
        })
        .map_err(|_| {
            weft_core::error::Diagnostic::new(
                "WFT-CURRENT-QUERY-REFINEMENT",
                "capability",
                "Conditional current-query refinement refused",
            )
        })
    }
    /// Conditional result comparison under independently supplied native-host premises.
    #[cfg(feature = "current-query-refinement-candidate")]
    pub fn check_postgresql_current_sum_candidate_json(
        &self,
        binding: &str,
        input: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            Ok(
                weft_postgresql::current_scalar::check_original_sum(operands, binding, input)?
                    .to_string(),
            )
        })
        .map_err(|_| {
            weft_core::error::Diagnostic::new(
                "WFT-CURRENT-QUERY-REFINEMENT",
                "capability",
                "Conditional current-query refinement refused",
            )
        })
    }
    /// Conditional result comparison under independently supplied native-host premises.
    #[cfg(feature = "current-query-refinement-candidate")]
    pub fn check_databricks_current_contributors_candidate_json(
        &self,
        binding: &str,
        input: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            Ok(
                weft_databricks::current_scalar::check_contributors(operands, binding, input)?
                    .to_string(),
            )
        })
        .map_err(|_| {
            weft_core::error::Diagnostic::new(
                "WFT-CURRENT-QUERY-REFINEMENT",
                "capability",
                "Conditional current-query refinement refused",
            )
        })
    }
    /// Conditional result comparison under independently supplied native-host premises.
    #[cfg(feature = "current-query-refinement-candidate")]
    pub fn check_databricks_current_sum_candidate_json(
        &self,
        binding: &str,
        input: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            Ok(
                weft_databricks::current_scalar::check_original_sum(operands, binding, input)?
                    .to_string(),
            )
        })
        .map_err(|_| {
            weft_core::error::Diagnostic::new(
                "WFT-CURRENT-QUERY-REFINEMENT",
                "capability",
                "Conditional current-query refinement refused",
            )
        })
    }
    #[cfg(feature = "bounded-current-string-candidate")]
    pub fn lower_postgresql_bounded_current_rows_candidate_json(
        &self,
        binding: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            weft_postgresql::current_scalar::lower_bounded_rows(operands, binding)?.json_string()
        })
    }
    #[cfg(feature = "bounded-current-string-candidate")]
    pub fn check_postgresql_bounded_current_rows_candidate_json(
        &self,
        binding: &str,
        input: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            Ok(
                weft_postgresql::current_scalar::check_bounded_rows(operands, binding, input)?
                    .to_string(),
            )
        })
        .map_err(|_| {
            weft_core::error::Diagnostic::new(
                "WFT-CURRENT-QUERY-REFINEMENT",
                "capability",
                "Conditional current-query refinement refused",
            )
        })
    }
    #[cfg(feature = "bounded-current-string-candidate")]
    pub fn lower_postgresql_bounded_current_contributors_candidate_json(
        &self,
        binding: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            weft_postgresql::current_scalar::lower_bounded_contributors(operands, binding)?
                .json_string()
        })
    }
    #[cfg(feature = "bounded-current-string-candidate")]
    pub fn check_postgresql_bounded_current_contributors_candidate_json(
        &self,
        binding: &str,
        input: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            Ok(weft_postgresql::current_scalar::check_bounded_contributors(
                operands, binding, input,
            )?
            .to_string())
        })
        .map_err(|_| {
            weft_core::error::Diagnostic::new(
                "WFT-CURRENT-QUERY-REFINEMENT",
                "capability",
                "Conditional current-query refinement refused",
            )
        })
    }
    #[cfg(feature = "bounded-current-string-candidate")]
    pub fn lower_databricks_bounded_current_contributors_candidate_json(
        &self,
        binding: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            weft_databricks::current_scalar::lower_bounded_contributors(operands, binding)?
                .json_string()
        })
    }
    #[cfg(feature = "bounded-current-string-candidate")]
    pub fn check_databricks_bounded_current_contributors_candidate_json(
        &self,
        binding: &str,
        input: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|_, _, operands| {
            Ok(weft_databricks::current_scalar::check_bounded_contributors(
                operands, binding, input,
            )?
            .to_string())
        })
        .map_err(|_| {
            weft_core::error::Diagnostic::new(
                "WFT-CURRENT-QUERY-REFINEMENT",
                "capability",
                "Conditional current-query refinement refused",
            )
        })
    }
    /// Trusted Rust consumer hook, not a Python callback or native admission API.
    /// Borrowed declarations are derived over the same original context on each
    /// scoped invocation; SQL/AST are not reparsed and caller reports are not imported.
    /// The original factory revalidates retained opaque profile/source JSON.
    /// A consumer still must admit native source/caller/current authority and all
    /// effective routes, action permissions, scalar/runtime scopes and release.
    pub fn with_context<T>(
        &self,
        consume: impl FnOnce(
            &weft_core::application_lineage::ApplicationLineage,
            &weft_core::current_query_designation::CurrentQueryDesignations<'_>,
            &weft_core::current_query_operands::CurrentQueryOperands<'_>,
        ) -> weft_core::error::Result<T>,
    ) -> weft_core::error::Result<T> {
        let designation = weft_core::current_query_designation::derive(&self.original)?;
        let operands = weft_core::current_query_operands::derive(&designation)?;
        if designation.json() != self.designation_json
            || operands.operations().len() != self.operand_count
        {
            return Err(weft_core::error::Diagnostic::new(
                "WFT-CURRENT-QUERY-CONTEXT",
                "capability",
                "Original current query context refused",
            ));
        }
        consume(&self.original, &designation, &operands)
    }
}
#[cfg(all(test, feature = "application-lineage-candidate"))]
mod application_lineage_bridge_tests {
    // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn bridge_returns_exact_original_core_bytes_and_refuses_report_override() {
        for raw in [
            include_str!("../../../tests/application-lineage/fixtures/token-equality.request.json"),
            include_str!(
                "../../../tests/lineage-v02/fixtures/distinct-selfjoin-false-zero.request.json"
            ),
            include_str!("../../../tests/lineage-v02/fixtures/field-free-false-zero.request.json"),
        ] {
            let actual = super::prepare_application_lineage_json(raw).unwrap();
            assert_eq!(
                actual,
                weft_core::application_lineage::prepare(raw).unwrap().json()
            );
            let r: serde_json::Value = serde_json::from_str(&actual).unwrap();
            assert_eq!(r["securityReady"], false);
            assert_eq!(r["authority"], false);
            let mut bad: serde_json::Value = serde_json::from_str(raw).unwrap();
            bad["readRequirements"] = serde_json::json!({});
            assert!(super::prepare_application_lineage_json(&bad.to_string()).is_err());
        }
        for bad in ["{}".to_owned(), " ".repeat(4_000_001)] {
            assert!(super::prepare_application_lineage_json(&bad).is_err());
        }
    }
}

/// Source-bound original-action designation only; no query mode or native permit.
#[cfg(feature = "application-lineage-candidate")]
pub fn prepare_current_query_designations_json(request: &str) -> weft_core::error::Result<String> {
    prepare_application_lineage(request)?.current_query_designations_json()
}

/// Pure browser-local catalog mapping; no store capability is accepted from JSON.
#[cfg(feature = "truss-browser-local")]
pub fn install_browser_mutation_backend(
    registry: &mut MutationRegistry,
    configuration: &str,
) -> weft_core::mutation::MResult<usize> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Configuration {
        model: weft_core::mutation::Artifact,
        model_pin: weft_core::ir::ModelPin,
        binding: weft_core::mutation::Artifact,
    }
    let fail = || weft_core::mutation::MutationDiagnostic {
        code: "WFT-MUTATION-BROWSER-REGISTRATION".into(),
        phase: "input".into(),
        path: String::new(),
        message: "Bounded original browser mapping required".into(),
    };
    if configuration.len() > 4194304 {
        return Err(fail());
    }
    let value = weft_core::json::checked_json(configuration).map_err(|_| fail())?;
    let c: Configuration = serde_json::from_value(value).map_err(|_| fail())?;
    registry.install_selection_backend(Box::new(
        weft_postgresql::truss_mutation::CurrentTrussMutationBackend::new_browser(
            c.model,
            c.binding,
            c.model_pin,
        )?,
    ))
}

#[cfg(all(test, feature = "application-lineage-candidate"))]
mod current_query_object_tests {
    use super::*;
    use serde_json::Value;
    fn raw() -> String {
        include_str!(
            "../../../tests/current-query-designation/fixtures/mixed-original-uses.request.json"
        )
        .into()
    }
    // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn scoped_consumer_uses_same_original_parent_and_owned_payload_without_report_import() {
        let raw = format!(" \n{}\t", raw());
        let parent = prepare_application_lineage(&raw).unwrap();
        let child = parent.prepare_current_query().unwrap();
        let same_bytes = prepare_application_lineage(&raw).unwrap();
        assert!(child.is_from(&parent));
        assert!(!child.is_from(&same_bytes));
        assert_eq!(
            child.designation_json(),
            parent.current_query_designations_json().unwrap()
        );
        assert_eq!(child.operand_count(), 2);
        child
            .with_context(|original, designation, operands| {
                assert!(std::ptr::eq(original, parent.inner.as_ref()));
                assert!(std::ptr::eq(original, designation.preparation()));
                assert!(std::ptr::eq(operands.designation(), designation));
                assert_eq!(original.source_request_json(), raw);
                assert_eq!(operands.operations().len(), child.operand_count());
                Ok(())
            })
            .unwrap();
        drop(parent);
        child
            .with_context(|original, _, _| {
                assert_eq!(original.source_request_json(), raw);
                Ok(())
            })
            .unwrap();
    }
    // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn cached_declaration_or_original_context_substitution_refuses_before_consumer() {
        for kind in 0..3 {
            let parent = prepare_application_lineage(&raw()).unwrap();
            let mut child = parent.prepare_current_query().unwrap();
            match kind {
                0 => child.operand_count += 1,
                1 => child.designation_json = "{}".into(),
                _ => {
                    let substituted = format!(" {}", raw());
                    let replacement = prepare_application_lineage(&substituted).unwrap();
                    let before: Value = serde_json::from_str(parent.json()).unwrap();
                    let after: Value = serde_json::from_str(replacement.json()).unwrap();
                    assert_eq!(before["applicationPlanJson"], after["applicationPlanJson"]);
                    child.original = replacement.inner;
                }
            }
            let called = std::cell::Cell::new(false);
            let result = child.with_context(|_, _, _| {
                called.set(true);
                Ok(())
            });
            assert_eq!(result.unwrap_err().code, "WFT-CURRENT-QUERY-CONTEXT");
            assert!(!called.get());
        }
    }
    // @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn unsupported_and_legacy_preparations_issue_no_child() {
        for raw in [include_str!("../../../tests/current-query-designation/fixtures/unsupported-salary-sum.request.json"),include_str!("../../../tests/application-lineage/fixtures/token-equality.request.json")]{let parent=prepare_application_lineage(raw).unwrap();let before=parent.json().to_owned();assert!(parent.prepare_current_query().is_err());assert_eq!(parent.json(),before);}
    }
}

#[cfg(feature = "raw-postgresql-reads")]
pub fn compile_raw_read_json(request: &str) -> String {
    let parsed = weft_core::json::checked_json(request).ok();
    let version = parsed
        .as_ref()
        .and_then(|r| r.get("version").and_then(serde_json::Value::as_str));
    if matches!(
        version,
        Some("weft.unity-current-read-request/0.4")
            | Some("weft.unity-current-read-request/0.5")
    ) {
        // These closed owner requests intentionally have no generic `target`
        // key. Dispatch by their exact request version before legacy routing.
        weft_databricks::unity_current_read::compile_json(request)
    } else if serde_json::from_str::<serde_json::Value>(request)
        .ok()
        .is_some_and(|r| r["target"] == "spark-delta4")
    {
        weft_databricks::raw_read::compile_json(request)
    } else {
        weft_postgresql::raw_read::compile_json(request)
    }
}

#[cfg(all(test, feature = "raw-postgresql-reads"))]
mod current_unity_raw_dispatch_tests {
    #[test]
    fn exact_sum_avg_versions_route_without_changing_legacy_target_routing() {
        for (version, response_version, code) in [
            ("weft.unity-current-read-request/0.4", "weft.unity-current-selection/0.5", "WFT-UNITY-CURRENT-SUM"),
            ("weft.unity-current-read-request/0.5", "weft.unity-current-selection/0.6", "WFT-UNITY-CURRENT-AVG"),
        ] {
            let response: serde_json::Value = serde_json::from_str(&super::compile_raw_read_json(
                &serde_json::json!({"version":version}).to_string(),
            )).unwrap();
            assert_eq!(response["version"], response_version);
            assert_eq!(response["status"], "blocked");
            assert_eq!(response["diagnostics"][0]["code"], code);
            let extra_target: serde_json::Value = serde_json::from_str(&super::compile_raw_read_json(
                &serde_json::json!({"version":version,"target":"spark-delta4"}).to_string(),
            )).unwrap();
            assert_eq!(extra_target["version"], response_version);
            assert_eq!(extra_target["status"], "blocked");
            assert_eq!(extra_target["diagnostics"][0]["code"], code);
        }
        let legacy_refusal: serde_json::Value = serde_json::from_str(&super::compile_raw_read_json(
            &serde_json::json!({"version":"weft.raw-read/0.1","target":"spark-delta4"}).to_string(),
        )).unwrap();
        assert_eq!(legacy_refusal["version"], "weft.raw-read/0.1");
        assert_eq!(legacy_refusal["diagnostics"][0]["code"], "WFT-RAW-SPARK");
        let duplicate_target: serde_json::Value = serde_json::from_str(&super::compile_raw_read_json(
            r#"{"version":"weft.raw-read/0.1","target":"postgresql","target":"spark-delta4"}"#,
        )).unwrap();
        assert_eq!(duplicate_target["diagnostics"][0]["code"], "WFT-RAW-SPARK");

        let valid_legacy = include_str!("../../../tests/spark-pages/read-request.json");
        let compiled: serde_json::Value =
            serde_json::from_str(&super::compile_raw_read_json(valid_legacy)).unwrap();
        assert_eq!(compiled["version"], "weft.raw-read/0.1");
        assert_eq!(compiled["status"], "compiled", "{compiled}");
        assert!(compiled["sql"].as_str().is_some_and(|sql| !sql.is_empty()));
        assert_eq!(compiled["sourceSql"], "SELECT o.id AS id, c.name AS customer FROM \"commerce\".\"Order\" AS o LEFT JOIN \"commerce\".\"Customer\" AS c ON o.customer_id = c.id ORDER BY c.name ASC NULLS LAST, o.id DESC NULLS FIRST LIMIT 50");
        assert_eq!(compiled["outputs"].as_array().map(Vec::len), Some(2));
    }
}

#[cfg(feature = "raw-postgresql-reads")]
pub fn compile_native_privilege_probes_json(request: &str) -> String {
    weft_postgresql::native_privilege_probes::compile_json(request)
}
#[cfg(feature = "raw-postgresql-reads")]
pub fn check_native_privilege_probes_json(program: &str) -> String {
    weft_postgresql::native_privilege_probes::check_json(program)
}

#[cfg(feature = "raw-postgresql-reads")]
pub fn compile_native_action_json(request: &str) -> String {
    if weft_core::json::checked_json(request)
        .ok()
        .is_some_and(|v| v["target"] == "spark-delta4")
    {
        weft_databricks::raw_action::compile_json(request)
    } else {
        weft_postgresql::raw_action::compile_json(request)
    }
}

#[cfg(feature = "raw-postgresql-reads")]
pub fn compile_native_bulk_json(request: &str) -> String {
    if weft_core::json::checked_json(request)
        .ok()
        .is_some_and(|v| v["action"]["target"] == "spark-delta4")
    {
        weft_databricks::raw_bulk::compile_json(request)
    } else {
        weft_postgresql::raw_bulk::compile_json(request)
    }
}

#[cfg(feature = "raw-postgresql-reads")]
pub fn compile_search_json(request: &str) -> String {
    weft_postgresql::search::compile_json(request)
}

#[cfg(feature = "raw-postgresql-reads")]
pub mod engine_read;
#[cfg(feature = "raw-postgresql-reads")]
pub mod engine_related_read;
/// Original-object presence custody, not JSON import or native permission.
/// ```compile_fail
/// use weft_runtime::CurrentPresencePreparation;
/// fn import(raw:&str)->CurrentPresencePreparation{serde_json::from_str(raw).unwrap()}
/// ```
#[cfg(feature = "current-presence-candidate")]
pub struct CurrentPresencePreparation {
    original: std::sync::Arc<weft_core::application_lineage::ApplicationLineage>,
    preparation_json: String,
}
#[cfg(feature = "current-presence-candidate")]
impl CurrentPresencePreparation {
    pub fn json(&self) -> &str {
        &self.preparation_json
    }
    pub fn is_from(&self, original: &ApplicationLineagePreparation) -> bool {
        std::sync::Arc::ptr_eq(&self.original, &original.inner)
    }
    pub fn native_request_json(&self) -> weft_core::error::Result<String> {
        self.with_context(|context| context.native_request_json())
    }
    /// Data-only requirements from the same actual retained original context.
    #[cfg(feature = "current-presence-model-requirements-candidate")]
    pub fn model_requirements_json(&self) -> weft_core::error::Result<String> {
        self.with_context(|context| {
            Ok(
                weft_core::current_presence_model_requirements::derive(context)?
                    .json()
                    .to_owned(),
            )
        })
    }
    /// Original association data from this object's retained original parent.
    #[cfg(feature = "current-presence-graph-requirements-candidate")]
    pub fn graph_requirements_json(&self) -> weft_core::error::Result<String> {
        self.with_context(|context| {
            Ok(
                weft_core::current_presence_graph_requirements::derive(context)?
                    .json()
                    .to_owned(),
            )
        })
    }
    pub fn lower_postgresql_current_presence_candidate_json(
        &self,
        binding: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|context| {
            Ok(weft_postgresql::current_presence::lower(context, binding)?
                .json()
                .to_owned())
        })
    }
    pub fn check_postgresql_current_presence_candidate_json(
        &self,
        binding: &str,
        input: &str,
    ) -> weft_core::error::Result<String> {
        self.with_context(|context| {
            weft_postgresql::current_presence::check(context, binding, input)
        })
    }
    // Re-derive only the scoped borrow over the actual retained immutable parent.
    // No source request/SQL/AST or success JSON is imported on any method call.
    fn with_context<T>(
        &self,
        consume: impl FnOnce(
            &weft_core::current_presence::CurrentPresencePreparation<'_>,
        ) -> weft_core::error::Result<T>,
    ) -> weft_core::error::Result<T> {
        let prepared = weft_core::current_presence::prepare(&self.original)?;
        if prepared.json() != self.preparation_json {
            return Err(weft_core::error::Diagnostic::new(
                "WFT-CURRENT-PRESENCE-BRIDGE",
                "capability",
                "Current presence bridge refused",
            ));
        }
        consume(&prepared)
    }
}

#[cfg(all(test, feature = "current-presence-candidate"))]
mod current_presence_bridge_tests {
    use super::*;
    use serde_json::{json, Value};
    #[test]
    fn actual_presence_bridge_preserves_issued_parent_and_all_producer_artifacts() {
        let corpus: Value = serde_json::from_str(include_str!(
            "../../../tests/current-query-presence/fixtures/actual-atomic-transport.json"
        ))
        .unwrap();
        let emitted: Value = serde_json::from_str(include_str!(
            "../../../tests/current-query-presence/fixtures/actual-atomic-transport-emitted.json"
        ))
        .unwrap();
        for record in emitted.as_array().unwrap() {
            let case = corpus["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == record["id"])
                .unwrap();
            let raw = case["request"].to_string();
            let original = prepare_application_lineage(&raw).unwrap();
            let child = original.prepare_current_presence().unwrap();
            assert!(child.is_from(&original));
            let other = prepare_application_lineage(&raw).unwrap();
            assert!(!child.is_from(&other));
            assert!(original.prepare_current_query().is_err());
            assert!(original.current_query_designations_json().is_err());
            let core = weft_core::application_lineage::prepare(&raw).unwrap();
            let context = weft_core::current_presence::prepare(&core).unwrap();
            assert_eq!(child.json(), context.json());
            assert_eq!(
                child.native_request_json().unwrap(),
                context.native_request_json().unwrap()
            );
            drop(original);
            let binding = record["binding"].to_string();
            let compiled: Value = serde_json::from_str(
                &child
                    .lower_postgresql_current_presence_candidate_json(&binding)
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(compiled, record["compiled"]);
            let checked: Value = serde_json::from_str(
                &child
                    .check_postgresql_current_presence_candidate_json(
                        &binding,
                        &record["frame"].to_string(),
                    )
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(checked, record["report"]);
            assert_eq!(checked["authority"], false);
        }
    }
    #[test]
    fn old_parent_binding_and_wrong_transport_cannot_enter_new_bridge() {
        let raw = include_str!(
            "../../../tests/current-query-presence/fixtures/unadmitted-context.request.json"
        );
        let original = prepare_application_lineage(raw).unwrap();
        let child = original.prepare_current_presence().unwrap();
        assert!(child
            .lower_postgresql_current_presence_candidate_json("{}")
            .is_err());
        assert!(child
            .check_postgresql_current_presence_candidate_json("{}", "{}")
            .is_err());
        for version in [
            "weft-application-lineage/0.1-candidate",
            "weft-application-lineage/0.2-candidate",
            "weft-application-lineage/0.3-candidate",
        ] {
            let mut old: Value = serde_json::from_str(raw).unwrap();
            old["interfaceVersion"] = json!(version);
            old["sql"] = json!("SELECT r.resourceId FROM Resource r");
            let parent = prepare_application_lineage(&old.to_string()).unwrap();
            assert!(parent.prepare_current_presence().is_err());
        }
    }
}

#[cfg(all(test, feature = "current-presence-model-requirements-candidate"))]
mod current_presence_model_requirements_bridge_tests {
    use super::*;
    #[test]
    fn requirements_use_the_same_original_arc_and_survive_parent_handle_drop() {
        let raw = include_str!(
            "../../../tests/current-query-presence/fixtures/unadmitted-context.request.json"
        );
        let original = prepare_application_lineage(raw).unwrap();
        let child = original.prepare_current_presence().unwrap();
        let other = prepare_application_lineage(raw).unwrap();
        assert!(child.is_from(&original));
        assert!(!child.is_from(&other));
        let core = weft_core::application_lineage::prepare(raw).unwrap();
        let context = weft_core::current_presence::prepare(&core).unwrap();
        let expected = weft_core::current_presence_model_requirements::derive(&context).unwrap();
        assert_eq!(child.model_requirements_json().unwrap(), expected.json());
        drop(original);
        assert_eq!(child.model_requirements_json().unwrap(), expected.json());
        let report: serde_json::Value = serde_json::from_str(expected.json()).unwrap();
        assert_eq!(report["sourceRequestJson"], raw);
        assert_eq!(report["complete"], false);
        assert_eq!(report["authority"], false);
    }
}

#[cfg(all(test, feature = "current-presence-graph-requirements-candidate"))]
mod current_presence_graph_requirements_bridge_tests {
    use super::*;
    #[test]
    fn graph_data_refuses_stored_context_mismatch_before_factory() {
        let raw = include_str!(
            "../../../tests/current-query-presence/fixtures/unadmitted-context.request.json"
        );
        let original = prepare_application_lineage(raw).unwrap();
        let mut child = original.prepare_current_presence().unwrap();
        child.preparation_json.push(' ');
        let failure = child.graph_requirements_json().unwrap_err();
        assert_eq!(failure.code, "WFT-CURRENT-PRESENCE-BRIDGE");
    }
    #[test]
    fn graph_data_preserves_original_arc_and_exact_core_report() {
        let raw = include_str!(
            "../../../tests/current-query-presence/fixtures/unadmitted-context.request.json"
        );
        let original = prepare_application_lineage(raw).unwrap();
        let child = original.prepare_current_presence().unwrap();
        let other = prepare_application_lineage(raw).unwrap();
        assert!(child.is_from(&original));
        assert!(!child.is_from(&other));
        let core = weft_core::application_lineage::prepare(raw).unwrap();
        let context = weft_core::current_presence::prepare(&core).unwrap();
        let expected = weft_core::current_presence_graph_requirements::derive(&context).unwrap();
        assert_eq!(child.graph_requirements_json().unwrap(), expected.json());
        let value: serde_json::Value = serde_json::from_str(expected.json()).unwrap();
        let model: serde_json::Value =
            serde_json::from_str(&child.model_requirements_json().unwrap()).unwrap();
        assert_eq!(value["sourceRequestJson"], raw);
        assert_eq!(value["sourceRequestJson"], model["sourceRequestJson"]);
        assert_eq!(value["associations"].as_array().unwrap().len(), 2);
        assert_eq!(value["authority"], false);
        assert_eq!(value["securityReady"], false);
        drop(original);
        assert_eq!(child.graph_requirements_json().unwrap(), expected.json());
    }
}

#[cfg(feature = "raw-postgresql-reads")]
pub mod engine_export;

#[cfg(feature = "raw-postgresql-reads")]
pub fn compile_unity_search_json(request: &str) -> String {
    weft_databricks::unity_search::compile_json(request)
}

#[cfg(feature = "managed-delta-mutation-candidate")]
pub fn compile_managed_action_json(request: &str) -> String {
    weft_databricks::raw_action::compile_managed_json(request)
}
#[cfg(feature = "managed-delta-mutation-candidate")]
pub fn compile_managed_bulk_json(request: &str) -> String {
    weft_databricks::raw_bulk::compile_managed_json(request)
}

#[cfg(feature = "raw-postgresql-reads")]
pub fn compile_related_read_json(request: &str) -> String {
    weft_postgresql::related_read::compile_json(request)
}

#[cfg(feature = "ashlar-sdk-ordered-page-candidate")]
pub mod engine_ordered_page;

#[cfg(feature = "raw-postgresql-reads")]
pub fn compile_table_operation_json(request: &str) -> String {
    weft_postgresql::table_operation::compile_json(request)
}
#[cfg(feature = "raw-postgresql-reads")]
pub struct PreparedTableOperationModule { inner: weft_postgresql::table_operation::PreparedOperationModule }
#[cfg(feature = "raw-postgresql-reads")]
impl PreparedTableOperationModule {
    pub fn prepare(source: &str) -> std::result::Result<Self, &'static str> {
        weft_postgresql::table_operation::PreparedOperationModule::prepare(source).map(|inner| Self { inner })
    }
    pub fn prepare_operation(&self, operation: &str) -> std::result::Result<PreparedTableOperation, &'static str> {
        self.inner.prepare_operation(operation).map(|inner| PreparedTableOperation { inner })
    }
    pub fn prepare_query(&self, operation: &str) -> std::result::Result<PreparedTableQuery, &'static str> {
        self.inner.prepare_query(operation).map(|inner| PreparedTableQuery { inner })
    }
}
#[cfg(feature = "raw-postgresql-reads")]
pub struct PreparedTableOperation { inner: weft_postgresql::table_operation::PreparedOperation }
#[cfg(feature = "raw-postgresql-reads")]
impl PreparedTableOperation {
    pub fn bind_json(&self, call: &str) -> String { self.inner.bind_json(call) }
}
#[cfg(feature = "raw-postgresql-reads")]
pub fn prepare_table_query_json(request: &str) -> String {
    weft_postgresql::table_query::prepare_json(request)
}
#[cfg(feature = "raw-postgresql-reads")]
pub fn bind_table_query_json(prepared: &str, call: &str) -> String {
    weft_postgresql::table_query::bind_json(prepared, call)
}
#[cfg(feature = "raw-postgresql-reads")]
pub struct PreparedTableQuery { inner: weft_postgresql::table_query::PreparedQuery }
#[cfg(feature = "raw-postgresql-reads")]
impl PreparedTableQuery {
    pub fn prepare(source: &str) -> std::result::Result<Self, &'static str> {
        weft_postgresql::table_query::PreparedQuery::prepare(source).map(|inner| Self { inner })
    }
    pub fn bind_json(&self, call: &str) -> String { self.inner.bind_json(call) }
    pub fn program_json(&self) -> String { self.inner.program_json() }
}
#[cfg(feature = "raw-postgresql-reads")]
pub struct PreparedTableQueryModule { inner: weft_postgresql::table_query::PreparedQueryModule }
#[cfg(feature = "raw-postgresql-reads")]
impl PreparedTableQueryModule {
    pub fn prepare(source: &str) -> std::result::Result<Self, &'static str> {
        weft_postgresql::table_query::PreparedQueryModule::prepare(source).map(|inner| Self { inner })
    }
    pub fn prepare_operation(&self, operation: &str) -> std::result::Result<PreparedTableQuery, &'static str> {
        self.inner.prepare_operation(operation).map(|inner| PreparedTableQuery { inner })
    }
}
#[cfg(feature = "raw-postgresql-reads")]
pub fn check_table_operation_json(program: &str) -> String {
    weft_postgresql::table_operation::check_json(program)
}

#[cfg(feature="raw-postgresql-reads")]
pub fn compile_table_seed_json(request:&str)->String{weft_postgresql::table_seed::compile_json(request)}
#[cfg(feature="raw-postgresql-reads")]
pub fn check_table_seed_json(program:&str)->String{weft_postgresql::table_seed::check_json(program)}
