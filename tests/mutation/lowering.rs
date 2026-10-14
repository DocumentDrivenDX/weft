// @covers US-010-AC6 original mapping composition; native effect/authority proof is separate.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use weft_core::{
    backend::BindingInput,
    model::Catalog,
    mutation::{Artifact, MutationRegistry},
};
use weft_postgresql::{
    conformance_original,
    leaf_codec_definition::OriginalArtifact,
    mutation::{NativeObjectColumns, OriginalMutationBackend},
    row_join_definition::Column,
};
fn setup() -> (MutationRegistry, Artifact, Value) {
    setup_mode(None, None)
}
fn setup_with(
    operations: Option<
        std::collections::BTreeSet<weft_postgresql::native_comparator_definition::Operation>,
    >,
) -> (MutationRegistry, Artifact, Value) {
    setup_mode(operations, None)
}
fn setup_mode(
    operations: Option<
        std::collections::BTreeSet<weft_postgresql::native_comparator_definition::Operation>,
    >,
    mode: Option<&str>,
) -> (MutationRegistry, Artifact, Value) {
    let input: Value =
        serde_json::from_str(include_str!("fixtures/postgres-original-input.json")).unwrap();
    let owner: Value =
        serde_json::from_str(include_str!("fixtures/postgres-owner-plan.json")).unwrap();
    let catalog =
        Catalog::prepare_security(serde_json::from_value(input["modules"].clone()).unwrap())
            .unwrap();
    let binding = BindingInput {
        json: input["binding"]["bindingJson"].as_str().unwrap().into(),
        sha256: input["binding"]["bindingSha256"].as_str().unwrap().into(),
        profile: input["composition"]["bindingProfile"]
            .as_str()
            .unwrap()
            .into(),
    };
    let mut configuration =
        conformance_original::configuration(&input["composition"].to_string(), &catalog).unwrap();
    if let Some(operations) = operations {
        use weft_postgresql::native_comparator_definition::{Definition, Selection};
        for (key, admitted) in &mut configuration.comparators {
            let identity: Value = serde_json::from_str(key).unwrap();
            let owner = serde_json::from_value(identity["owner"].clone()).unwrap();
            let field = serde_json::from_value(identity["field"].clone()).unwrap();
            let record = catalog.record_by_identity(&owner).unwrap();
            let (_, descriptors) = catalog
                .member_descriptor_by_identity(&record, &field)
                .unwrap();
            let logical = descriptors
                .iter()
                .find_map(|d| {
                    if d.identity == field {
                        if let weft_core::application_model::Shape::Scalar { logical_type } =
                            &d.shape
                        {
                            Some(logical_type)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap();
            let raw: Value = serde_json::from_str(&admitted.original_json).unwrap();
            let artifacts = admitted
                .original_artifacts
                .iter()
                .map(|(name, bytes)| {
                    (
                        name.clone(),
                        OriginalArtifact {
                            identity: raw[name]["identity"].as_str().unwrap().into(),
                            bytes: bytes.clone(),
                        },
                    )
                })
                .collect();
            *admitted = Definition::parse(
                &admitted.original_json,
                Selection {
                    profile: &raw["profile"],
                    native_profile: &raw["nativeDomainProfile"],
                    original_artifacts: &artifacts,
                    operations: &operations,
                },
                logical,
            )
            .unwrap();
        }
    }
    match mode {
        Some("missing") => configuration.comparators.clear(),
        Some("int8") => {
            for admitted in configuration.comparators.values_mut() {
                if admitted.native_type == "pg_catalog.numeric" {
                    admitted.native_type = "pg_catalog.int8".into();
                }
            }
        }
        _ => (),
    }
    let backend = configuration.backend(&catalog, &binding).unwrap();
    let binding_artifact = Artifact {
        media_type: "application/json".into(),
        utf8: binding.json,
        sha256: binding.sha256,
    };
    let n = &input["nativeColumns"];
    let columns = NativeObjectColumns {
        inventory: OriginalArtifact {
            identity: n["inventory"]["identity"].as_str().unwrap().into(),
            bytes: STANDARD
                .decode(n["inventory"]["bytesBase64"].as_str().unwrap())
                .unwrap(),
        },
        relation_identity: n["relationIdentity"].as_str().unwrap().into(),
        identity: Column {
            relation_identity: n["identity"]["relationIdentity"].as_str().unwrap().into(),
            name: n["identity"]["name"].as_str().unwrap().into(),
        },
        version: Column {
            relation_identity: n["version"]["relationIdentity"].as_str().unwrap().into(),
            name: n["version"]["name"].as_str().unwrap().into(),
        },
        evidence: vec![
            "fixture://original-codecs/truss-column-source/pg17.11-mutation-selection".into(),
        ],
    };
    let mut registry = MutationRegistry::from_json(&owner["registry"].to_string()).unwrap();
    let native =
        OriginalMutationBackend::new(backend, catalog, binding_artifact.clone(), columns).unwrap();
    assert_eq!(
        registry
            .install_selection_backend(Box::new(native))
            .unwrap(),
        0
    );
    (registry, binding_artifact, owner["request"].clone())
}
fn lower(sql: &str, parameters: Value, maximum: u32) -> Value {
    lower_with(sql, parameters, maximum, setup())
}
fn lower_with(
    sql: &str,
    parameters: Value,
    maximum: u32,
    prepared: (MutationRegistry, Artifact, Value),
) -> Value {
    let (registry, binding, mut request) = prepared;
    request["sql"] = json!(sql);
    request["parameters"] = parameters;
    let response: Value =
        serde_json::from_str(&registry.compile_json(&request.to_string())).unwrap();
    assert_eq!(response["status"], "compiled", "{response}");
    let plan: Value = serde_json::from_str(response["plan"]["utf8"].as_str().unwrap()).unwrap();
    let selection: Artifact =
        serde_json::from_value(plan["selection"]["artifact"].clone()).unwrap();
    let model: Artifact = serde_json::from_value(request["model"].clone()).unwrap();
    serde_json::to_value(
        registry
            .lower_selection(0, &selection, &model, &binding, maximum)
            .unwrap(),
    )
    .unwrap()
}
#[test]
fn original_selection_mapping_yields_preflight_numeric_order_and_overflow() {
    let result = lower(
        "UPDATE Customer SET active=FALSE WHERE id<:cutoff",
        json!({"cutoff":{"integerToken":"100"}}),
        2,
    );
    assert_eq!(result["status"], "lowered", "{result}");
    assert_eq!(result["bounds"], json!({"maxItems":2,"fetchLimit":3}));
    assert!(result["sql"]
        .as_str()
        .unwrap()
        .contains("ORDER BY \"weft_mutation\".\"id\" ASC LIMIT 3"));
    assert_eq!(result["columns"][2]["field"]["element"], "customer-part");
    assert_eq!(result["columns"][3]["field"]["element"], "customer-id");
    assert_eq!(result["preflight"].as_array().unwrap().len(), 3);
}
#[test]
fn constants_are_typed_parameters_and_complete_population_checks_ignore_where() {
    let value = "O'Reilly😀";
    let result = lower(
        "UPDATE Customer SET active=FALSE WHERE part=:name OR id<:cutoff",
        json!({"name":{"string":value},"cutoff":{"integerToken":"100"}}),
        100,
    );
    assert_eq!(result["status"], "lowered", "{result}");
    assert!(!result["sql"].as_str().unwrap().contains(value));
    assert_eq!(result["parameters"][0]["value"], json!({"string":value}));
    for check in result["preflight"].as_array().unwrap() {
        assert!(!check["sql"].as_str().unwrap().contains(value));
        assert!(check["parameters"].as_array().unwrap().is_empty());
    }
}
#[test]
fn missing_native_property_and_unrepresentable_string_refuse_without_fallback() {
    for (sql, parameters) in [
        (
            "UPDATE Customer SET active=FALSE WHERE active=TRUE",
            json!({}),
        ),
        (
            "UPDATE Customer SET active=FALSE WHERE part=:name",
            json!({"name":{"string":"a\u{0}b"}}),
        ),
    ] {
        let result = lower(sql, parameters, 10);
        assert_eq!(result["status"], "blocked", "{result}");
        assert!(result.get("sql").is_none());
    }
}
#[test]
fn capture_native_programs_from_current_compiler() {
    let cases = [
        (
            "integer",
            "UPDATE Customer SET active=FALSE WHERE id<:cutoff",
            json!({"cutoff":{"integerToken":"100"}}),
        ),
        (
            "mixed",
            "UPDATE Customer SET active=FALSE WHERE part='Foo😀' OR (id<100 AND part<>'excluded')",
            json!({}),
        ),
        (
            "string",
            "UPDATE Customer SET active=FALSE WHERE part>:name",
            json!({"name":{"string":"é"}}),
        ),
    ];
    let captures: Vec<_> = cases
        .into_iter()
        .map(|(id, sql, p)| json!({"id":id,"lowered":lower(sql,p,2)}))
        .collect();
    for c in &captures {
        assert_eq!(c["lowered"]["status"], "lowered", "{c}");
    }
    if let Ok(path) = std::env::var("WEFT_MUTATION_NATIVE_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
    }
}

#[test]
fn original_operations_are_required_per_predicate_not_invented_for_key_extraction() {
    use weft_postgresql::native_comparator_definition::Operation;
    for (operation, sql, status) in [
        (
            Operation::Ordering,
            "UPDATE Customer SET active=FALSE WHERE id=100",
            "blocked",
        ),
        (
            Operation::Ordering,
            "UPDATE Customer SET active=FALSE WHERE id<100",
            "lowered",
        ),
        (
            Operation::Equality,
            "UPDATE Customer SET active=FALSE WHERE id<100",
            "blocked",
        ),
        (
            Operation::Equality,
            "UPDATE Customer SET active=FALSE WHERE id=100",
            "lowered",
        ),
        (
            Operation::Ordering,
            "UPDATE Customer SET active=FALSE WHERE part='a'",
            "blocked",
        ),
        (
            Operation::Equality,
            "UPDATE Customer SET active=FALSE WHERE part>'a'",
            "blocked",
        ),
        (
            Operation::Equality,
            "UPDATE Customer SET active=FALSE WHERE part='a'",
            "lowered",
        ),
    ] {
        let result = lower_with(
            sql,
            json!({}),
            10,
            setup_with(Some(std::collections::BTreeSet::from([operation]))),
        );
        assert_eq!(result["status"], status, "{result}");
    }
}
#[test]
fn exact_domain_zero_and_long_fraction_refuse_unqualified_native_parameter_representations() {
    for token in [
        "0e999999999".to_string(),
        format!("1.{}", "0".repeat(17000)),
    ] {
        let result = lower(
            "UPDATE Customer SET active=FALSE WHERE id=:cutoff",
            json!({"cutoff":{"integerToken":token}}),
            10,
        );
        assert_eq!(result["status"], "blocked", "{result}");
        assert!(result.get("sql").is_none());
    }
}

#[test]
fn absent_comparators_and_non_numeric_integer_custody_refuse() {
    for (mode, sql) in [
        ("missing", "UPDATE Customer SET active=FALSE WHERE part='a'"),
        ("missing", "UPDATE Customer SET active=FALSE WHERE id=1"),
        ("int8", "UPDATE Customer SET active=FALSE WHERE id=1"),
    ] {
        let result = lower_with(sql, json!({}), 10, setup_mode(None, Some(mode)));
        assert_eq!(result["status"], "blocked", "{result}");
        assert!(result.get("sql").is_none());
    }
}
#[test]
fn native_integer_parameter_representation_edges() {
    for (token, status) in [
        (format!("1.{}", "0".repeat(254)), "lowered"),
        (format!("1.{}", "0".repeat(255)), "blocked"),
        ("0e100".into(), "lowered"),
        ("0e-100".into(), "lowered"),
        ("0e101".into(), "blocked"),
        ("0e-101".into(), "blocked"),
    ] {
        let result = lower(
            "UPDATE Customer SET active=FALSE WHERE id=:cutoff",
            json!({"cutoff":{"integerToken":token}}),
            10,
        );
        assert_eq!(result["status"], status, "{result}");
    }
}
