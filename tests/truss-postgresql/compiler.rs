// @covers US-003-AC1 @covers US-003-AC2 @covers US-003-AC3 @covers US-003-AC4
// These are candidate component cases; native execution/production scope is separate.
use serde_json::{json, Value};
use weft_core::{backend::Registry, compile::Compiler};
use weft_postgresql::candidate::Candidate;
fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("fixtures/compiler-cases.json")).unwrap()
}
fn compiler() -> Compiler {
    let mut registry = Registry::default();
    registry.register(Candidate).unwrap();
    Compiler { registry }
}
fn run(c: &Compiler, r: &Value) -> Value {
    serde_json::from_str(&c.compile_json(&r.to_string())).unwrap()
}
#[test]
fn candidate_registry_compiles_both_homes_and_retains_exact_contracts() {
    let c = compiler();
    for case in cases() {
        let response = run(&c, &case["request"]);
        assert_eq!(
            response["status"], "compiled",
            "{}: {}",
            case["id"], response
        );
        assert_eq!(response["qualification"]["status"], "candidate");
        assert_eq!(
            response["logicalPlan"]["modulePins"],
            json!(case["request"]["modules"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| m["pin"].clone())
                .collect::<Vec<_>>())
        );
        assert!(response["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["id"] == "truss.candidate.scalarIntegrity"));
    }
}
#[test]
fn default_candidate_and_altered_bundle_refuse_without_sql() {
    let c = compiler();
    let mut r = cases()[0]["request"].clone();
    r["options"]["allowCandidate"] = json!(false);
    let response = run(&c, &r);
    assert_eq!(response["diagnostics"][0]["code"], "WFT-CAPABILITY");
    assert!(response.get("sql").is_none());
    let mut r = cases()[0]["request"].clone();
    let mut binding: Value =
        serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();
    binding["basis"]["modelBundle"] =
        json!({"identity":"changed","bytesBase64":"W10=","sha256":weft_core::json::sha256(b"[]")});
    let raw = binding.to_string();
    r["target"]["bindingJson"] = json!(raw);
    r["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
    let response = run(&c, &r);
    assert_eq!(response["diagnostics"][0]["code"], "WFT-BINDING");
    assert!(response.get("sql").is_none());
}

#[test]
fn application_relational_stages_compile_with_typed_result_contracts() {
    let compiler = compiler();
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    assert_eq!(cases.len(), 76);
    for case in cases {
        let response = run(&compiler, &case["request"]);
        assert_eq!(
            response["status"], "compiled",
            "{}: {}",
            case["id"], response
        );
        assert_eq!(response["qualification"]["status"], "candidate");
        let id = case["id"].as_str().unwrap();
        if case["request"]["sql"]
            .as_str()
            .unwrap()
            .contains("RELATED_KEYS")
        {
            let result = &response["columns"][1];
            assert_eq!(result["representation"]["kind"], "relatedKeys");
            assert_eq!(result["nullable"], false);
            assert!(!result["representation"]["key"]["fields"]
                .as_array()
                .unwrap()
                .is_empty());
            assert!(response["obligations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|o| o["id"] == "truss.candidate.relationshipIntegrity"));
        }
        if id.starts_with("optional-scalar") {
            assert_eq!(response["columns"][1]["representation"]["kind"], "value");
            assert_eq!(
                response["columns"][1]["representation"]["nativeNull"],
                false
            );
            assert_eq!(response["columns"][1]["nullable"], false);
        }
        if id.contains("count") {
            let columns = response["columns"].as_array().unwrap();
            let count = columns.last().unwrap();
            assert_eq!(
                count["representation"]["decoder"], "exact-integer",
                "{}",
                response
            );
            assert_eq!(count["nullable"], false);
            assert!(response["sql"].as_str().unwrap().contains("count(*)::text"));
        }
        if id.starts_with("injection-text") {
            let sql = response["sql"].as_str().unwrap();
            let params = response["parameters"].as_array().unwrap();
            let supplied = params
                .iter()
                .find(|p| p["origin"]["parameter"] == "name")
                .unwrap();
            assert!(!sql.contains(supplied["value"].as_str().unwrap()));
        }
    }
}

#[test]
fn authored_page_key_missing_wrong_owner_and_wrong_component_refuse_atomically() {
    let compiler = compiler();
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    for home in ["props", "row"] {
        let original = &cases
            .iter()
            .find(|c| c["id"] == format!("optional-scalar-{home}"))
            .unwrap()["request"];
        for alteration in ["missing", "owner", "component", "definition"] {
            let mut request = original.clone();
            let mut binding: Value =
                serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
            match alteration {
                "missing" => binding["keys"] = json!([]),
                "owner" => binding["keys"][0]["ownerTypeId"] = json!("-99"),
                "component" => binding["keys"][0]["orderedPropertyIds"] = json!(["1"]),
                "definition" => {
                    binding["keys"][0]["acceptedDefinition"] = json!({"identity":"wrong-key","bytesBase64":"e30=","sha256":weft_core::json::sha256(b"{}")})
                }
                _ => unreachable!(),
            }
            let raw = binding.to_string();
            request["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
            request["target"]["bindingJson"] = json!(raw);
            let response = run(&compiler, &request);
            assert_eq!(
                response["status"], "blocked",
                "{home}/{alteration}: {response}"
            );
            assert_eq!(response["diagnostics"][0]["code"], "WFT-BINDING");
            assert!(response.get("sql").is_none());
            assert!(response.get("parameters").is_none());
        }
    }
}

#[test]
fn rehashed_selected_definitions_do_not_override_the_original_umf() {
    let compiler = compiler();
    for case in cases().into_iter().take(2) {
        for section in ["entities", "properties"] {
            for artifact in ["source", "acceptedDefinition"] {
                let mut request = case["request"].clone();
                let mut binding: Value =
                    serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap())
                        .unwrap();
                binding[section][0][artifact] = json!({"identity":"changed-model-fragment","bytesBase64":"e30=","sha256":weft_core::json::sha256(b"{}")});
                let raw = binding.to_string();
                request["target"]["bindingJson"] = json!(raw);
                request["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
                let response = run(&compiler, &request);
                assert_eq!(
                    response["diagnostics"][0]["code"], "WFT-BINDING",
                    "{section}/{artifact}: {response}"
                );
                assert!(response.get("sql").is_none());
            }
        }
    }
}

#[test]
fn composite_key_order_is_not_an_unordered_property_set() {
    let compiler = compiler();
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    for case in cases
        .iter()
        .filter(|c| c["id"].as_str().unwrap().starts_with("composite-cursor"))
    {
        let mut request = case["request"].clone();
        let mut binding: Value =
            serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
        binding["keys"][0]["orderedPropertyIds"]
            .as_array_mut()
            .unwrap()
            .reverse();
        let raw = binding.to_string();
        request["target"]["bindingJson"] = json!(raw);
        request["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
        let response = run(&compiler, &request);
        assert_eq!(
            response["diagnostics"][0]["code"], "WFT-BINDING",
            "{response}"
        );
        assert!(response.get("sql").is_none());
    }
}

#[test]
fn relationship_roles_and_original_definitions_cannot_be_substituted() {
    let compiler = compiler();
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    for case in cases
        .iter()
        .filter(|c| c["id"].as_str().unwrap().contains("has-related"))
    {
        for alteration in [
            "missing",
            "sourceTypeId",
            "targetTypeId",
            "sourceKeyId",
            "targetKeyId",
            "sourceOrderedPropertyIds",
            "targetOrderedPropertyIds",
            "source",
            "acceptedDefinition",
        ] {
            let mut request = case["request"].clone();
            let mut binding: Value =
                serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
            match alteration {
                "missing" => binding["relationships"] = json!([]),
                "sourceTypeId" | "targetTypeId" => {
                    binding["relationships"][0][alteration] = json!("-99")
                }
                "sourceKeyId" | "targetKeyId" => {
                    binding["relationships"][0][alteration] = json!("wrong-key")
                }
                "sourceOrderedPropertyIds" | "targetOrderedPropertyIds" => {
                    binding["relationships"][0][alteration] = json!(["99"])
                }
                "source" | "acceptedDefinition" => {
                    binding["relationships"][0][alteration] = json!({"identity":"wrong-relationship","bytesBase64":"e30=","sha256":weft_core::json::sha256(b"{}")})
                }
                _ => unreachable!(),
            }
            let raw = binding.to_string();
            request["target"]["bindingJson"] = json!(raw);
            request["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
            let response = run(&compiler, &request);
            assert_eq!(
                response["diagnostics"][0]["code"], "WFT-BINDING",
                "{} / {alteration}: {response}",
                case["id"]
            );
            assert!(response.get("sql").is_none());
        }
    }
}

#[test]
fn sequence_cannot_fall_back_to_scalar_root_row_storage() {
    use base64::Engine;
    let compiler = compiler();
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    let mut request = cases
        .iter()
        .find(|c| c["id"] == "sequence-page-row")
        .unwrap()["request"]
        .clone();
    let mut binding: Value =
        serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
    let p = binding["properties"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|p| p["logical"]["element"] == "tags")
        .unwrap();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(p["homeDefinition"]["bytesBase64"].as_str().unwrap())
        .unwrap();
    let mut home: Value = serde_json::from_slice(&bytes).unwrap();
    home["access"] = json!("scalar-root");
    let bytes = home.to_string().into_bytes();
    p["homeDefinition"]["bytesBase64"] =
        json!(base64::engine::general_purpose::STANDARD.encode(&bytes));
    p["homeDefinition"]["sha256"] = json!(weft_core::json::sha256(&bytes));
    let raw = binding.to_string();
    request["target"]["bindingJson"] = json!(raw);
    request["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
    let response = run(&compiler, &request);
    assert_eq!(
        response["diagnostics"][0]["code"], "WFT-CAPABILITY",
        "{response}"
    );
    assert!(response.get("sql").is_none());
}

#[test]
fn structured_result_keeps_nested_identity_presence_and_exact_numeric_type() {
    let compiler = compiler();
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    for case in cases.iter().filter(|c| {
        c["id"]
            .as_str()
            .unwrap()
            .starts_with("numeric-structured-page")
    }) {
        let response = run(&compiler, &case["request"]);
        assert_eq!(response["status"], "compiled", "{response}");
        assert_eq!(response["columns"][1]["representation"]["kind"], "value");
        assert_eq!(
            response["columns"][1]["representation"]["descriptor"]["element"],
            "address"
        );
        assert_eq!(
            response["columns"][1]["representation"]["nativeNull"],
            false
        );
        let zip = response["logicalPlan"]["typeGraph"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["identity"]["element"] == "zip")
            .unwrap();
        assert_eq!(zip["availability"], "absent-allowed");
        assert_eq!(zip["type"]["family"], "integer");
        assert_eq!(
            zip["type"]["facets"]["integerWidth"],
            json!({"bits":64,"signed":false})
        );
    }
}

#[test]
fn candidate_context_cannot_delegate_hidden_state_to_presence_decoding() {
    let mut registry = weft_core::backend::Registry::default();
    registry
        .register(weft_postgresql::candidate::Candidate)
        .unwrap();
    let compiler = weft_core::compile::Compiler { registry };
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    for case in cases
        .iter()
        .filter(|c| c["id"].as_str().unwrap().starts_with("scalar-page"))
    {
        let response: Value =
            serde_json::from_str(&compiler.compile_json(&case["request"].to_string())).unwrap();
        assert_eq!(response["status"], "compiled");
        let context = response["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["id"] == "truss.candidate.context")
            .unwrap();
        assert_eq!(context["owner"], "host");
        assert_eq!(
            context["parameters"]["visibility"]["hiddenRowsAreNotAbsent"],
            true
        );
        assert_eq!(
            context["parameters"]["visibility"]["absenceRequiresCompleteStateView"],
            true
        );
        assert_eq!(
            context["parameters"]["visibility"]["compoundRequiresCompleteChildView"],
            true
        );
        assert_eq!(
            context["parameters"]["execution"]["pinsRecheckedPerExecution"],
            true
        );
        assert_eq!(
            context["parameters"]["execution"]["sameAffineTransactionForIntegrityAndData"],
            true
        );
        assert_eq!(
            context["parameters"]["execution"]["unknownObligationMeaning"],
            "refuse-before-sql"
        );
    }
}

#[test]
fn unknown_row_obligation_cannot_be_silently_replaced_by_candidate_context() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let compiler = compiler();
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    let mut request =
        cases.iter().find(|c| c["id"] == "global-sum-row").unwrap()["request"].clone();
    let mut binding: Value =
        serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
    let property = binding["properties"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|p| p["logical"]["element"] == "order-total")
        .unwrap();
    let artifact = &mut property["homeDefinition"];
    let bytes = STANDARD
        .decode(artifact["bytesBase64"].as_str().unwrap())
        .unwrap();
    let mut home: Value = serde_json::from_slice(&bytes).unwrap();
    home["storedDomainObligation"] = json!("unregistered.owner.obligation");
    let bytes = home.to_string().into_bytes();
    artifact["bytesBase64"] = json!(STANDARD.encode(&bytes));
    artifact["sha256"] = json!(weft_core::json::sha256(&bytes));
    let raw = binding.to_string();
    request["target"]["bindingJson"] = json!(raw);
    request["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
    let response = run(&compiler, &request);
    assert_eq!(response["status"], "blocked");
    assert_eq!(response["diagnostics"][0]["code"], "WFT-BINDING");
    assert!(response.get("sql").is_none());
}

#[test]
fn unknown_selected_codec_cannot_reuse_candidate_type_directed_decoding() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let compiler = compiler();
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    for home in ["props", "row"] {
        for profile in ["valueProfile", "presenceProfile"] {
            let id = format!("global-sum-{home}");
            let mut request = cases.iter().find(|c| c["id"] == id).unwrap()["request"].clone();
            let mut binding: Value =
                serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
            let property = binding["properties"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|p| p["logical"]["element"] == "order-total")
                .unwrap();
            property[profile]["identity"] = json!("unregistered.codec");
            if home == "props" {
                let mut definition: Value = serde_json::from_slice(
                    &STANDARD
                        .decode(property["homeDefinition"]["bytesBase64"].as_str().unwrap())
                        .unwrap(),
                )
                .unwrap();
                definition[profile] = property[profile].clone();
                let bytes = definition.to_string().into_bytes();
                property["homeDefinition"]["bytesBase64"] = json!(STANDARD.encode(&bytes));
                property["homeDefinition"]["sha256"] = json!(weft_core::json::sha256(&bytes));
            }
            let raw = binding.to_string();
            request["target"]["bindingJson"] = json!(raw);
            request["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
            let response = run(&compiler, &request);
            assert_eq!(response["status"], "blocked");
            assert_eq!(response["diagnostics"][0]["code"], "WFT-BINDING");
            assert!(response.get("sql").is_none());
        }
    }
}

#[test]
fn unknown_native_join_profile_cannot_select_fixed_row_joins() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let compiler = compiler();
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    let mut request =
        cases.iter().find(|c| c["id"] == "global-sum-row").unwrap()["request"].clone();
    let mut binding: Value =
        serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
    let property = binding["properties"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|p| p["logical"]["element"] == "order-total")
        .unwrap();
    let artifact = &mut property["homeDefinition"];
    let mut definition: Value = serde_json::from_slice(
        &STANDARD
            .decode(artifact["bytesBase64"].as_str().unwrap())
            .unwrap(),
    )
    .unwrap();
    definition["joinProfile"]["identity"] = json!("unregistered.row.join");
    let bytes = definition.to_string().into_bytes();
    artifact["bytesBase64"] = json!(STANDARD.encode(&bytes));
    artifact["sha256"] = json!(weft_core::json::sha256(&bytes));
    let raw = binding.to_string();
    request["target"]["bindingJson"] = json!(raw);
    request["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
    let response = run(&compiler, &request);
    assert_eq!(response["status"], "blocked");
    assert_eq!(response["diagnostics"][0]["code"], "WFT-BINDING");
    assert!(response.get("sql").is_none());
}

#[test]
fn unknown_execution_basis_profiles_refuse_before_sql() {
    let compiler = compiler();
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    for profile in [
        "readContextProfile",
        "layoutProfile",
        "identityProfile",
        "valueProfile",
        "keyProfile",
        "exporterProfile",
    ] {
        let mut request = cases
            .iter()
            .find(|c| c["id"] == "global-count-props")
            .unwrap()["request"]
            .clone();
        let mut binding: Value =
            serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
        binding["basis"][profile]["identity"] = json!("unregistered.execution.basis");
        let raw = binding.to_string();
        request["target"]["bindingJson"] = json!(raw);
        request["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
        let response = run(&compiler, &request);
        assert_eq!(response["status"], "blocked", "{profile}");
        assert_eq!(response["diagnostics"][0]["code"], "WFT-BINDING");
        assert!(response.get("sql").is_none());
    }
}

#[test]
fn selected_page_and_relationship_keys_cannot_replace_registered_meaning() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let compiler = compiler();
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    for case in cases
        .iter()
        .filter(|case| {
            case["id"].as_str().unwrap().ends_with("-props")
                && (case["request"]["sql"]
                    .as_str()
                    .unwrap()
                    .contains("RELATED_KEYS")
                    || case["id"] == "whole-entity-props")
        })
        .take(3)
    {
        let selected_keys = if case["id"] == "whole-entity-props" {
            vec![0]
        } else {
            vec![0, 1]
        };
        for key_index in selected_keys {
            for member in [
                "comparisonProfile",
                "encodingProfile",
                "comparisonDefinition",
                "encodingDefinition",
            ] {
                let mut request = case["request"].clone();
                let mut binding: Value =
                    serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap())
                        .unwrap();
                // Change endpoints independently so the page key cannot mask a target-key gap.
                {
                    let key = &mut binding["keys"][key_index];
                    if member.ends_with("Profile") {
                        key[member]["identity"] = json!("unregistered.key.meaning");
                    } else {
                        let bytes = b"{\"meaning\":\"replacement\"}";
                        key[member]["bytesBase64"] = json!(STANDARD.encode(bytes));
                        key[member]["sha256"] = json!(weft_core::json::sha256(bytes));
                    }
                }
                let raw = binding.to_string();
                request["target"]["bindingJson"] = json!(raw);
                request["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
                let response = run(&compiler, &request);
                assert_eq!(
                    response["status"], "blocked",
                    "{} {member}: {response}",
                    case["id"]
                );
                assert_eq!(response["diagnostics"][0]["code"], "WFT-BINDING");
                assert!(response.get("sql").is_none());
            }
        }
    }
}

#[test]
fn unknown_binding_home_and_execution_meanings_refuse_atomically() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let compiler = compiler();
    for case in cases().iter().take(2) {
        for kind in ["bindingProfile", "homeProfile", "executionObligations"] {
            let mut request = case["request"].clone();
            let mut binding: Value =
                serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
            match kind {
                "bindingProfile" => {
                    binding["bindingProfile"]["identity"] = json!("unknown.binding.meaning")
                }
                "homeProfile" => {
                    for property in binding["properties"].as_array_mut().unwrap() {
                        property["homeProfile"]["identity"] = json!("unknown.home.meaning");
                    }
                }
                _ => {
                    binding["executionObligations"] = json!([{"id":"unknown.host.procedure","profile":binding["bindingProfile"],"definition":{"identity":"unknown-obligation","bytesBase64":STANDARD.encode(b"{}"),"sha256":weft_core::json::sha256(b"{}")}}])
                }
            }
            let raw = binding.to_string();
            request["target"]["bindingJson"] = json!(raw);
            request["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
            let response = run(&compiler, &request);
            assert_eq!(
                response["status"], "blocked",
                "{} {kind}: {response}",
                case["id"]
            );
            assert_eq!(response["diagnostics"][0]["code"], "WFT-BINDING");
            assert!(response.get("sql").is_none());
        }
    }
}
#[test]
fn relationship_profile_cannot_change_fixed_traversal_meaning() {
    let compiler = compiler();
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/application-cases.json")).unwrap();
    for case in cases
        .iter()
        .filter(|case| case["id"] == "related-page-props" || case["id"] == "inverse-page-row")
    {
        let mut request = case["request"].clone();
        let mut binding: Value =
            serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
        for relationship in binding["relationships"].as_array_mut().unwrap() {
            relationship["relationshipProfile"]["identity"] = json!("unknown.relationship.meaning");
        }
        let raw = binding.to_string();
        request["target"]["bindingJson"] = json!(raw);
        request["target"]["bindingSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
        let response = run(&compiler, &request);
        assert_eq!(response["status"], "blocked", "{}: {response}", case["id"]);
        assert_eq!(response["diagnostics"][0]["code"], "WFT-BINDING");
        assert!(response.get("sql").is_none());
    }
}
