// @covers US-011-AC3 @covers US-011-AC4 @covers US-011-AC5
// Independent test consumers, not Truss/Ashlar or native qualification.
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use weft_core::{
    mutation::{Artifact, MResult, MutationDiagnostic, MutationRegistry},
    mutation_backend::*,
};
fn artifact(v: Value) -> Artifact {
    let utf8 = v.to_string();
    Artifact {
        media_type: "application/json".into(),
        sha256: weft_core::json::sha256(utf8.as_bytes()),
        utf8,
    }
}
fn inputs() -> (MutationRegistry, Artifact, Artifact, Artifact) {
    let registry = MutationRegistry::from_json(include_str!("fixtures/registry.json")).unwrap();
    let request: Value = serde_json::from_str(include_str!("fixtures/request.json")).unwrap();
    let response: Value =
        serde_json::from_str(&registry.compile_json(&request.to_string())).unwrap();
    let plan: Value = serde_json::from_str(response["plan"]["utf8"].as_str().unwrap()).unwrap();
    (
        registry,
        serde_json::from_value(plan["selection"]["artifact"].clone()).unwrap(),
        serde_json::from_value(request["model"].clone()).unwrap(),
        artifact(json!({"testConsumer":"original fixture only"})),
    )
}
struct Consumer {
    mode: &'static str,
    status: SelectionStatus,
    events: Arc<Mutex<Vec<String>>>,
}
impl SelectionBackend for Consumer {
    fn describe(&self) -> MResult<SelectionCapabilities> {
        if self.mode == "describe-error" {
            return Err(bad_diagnostic());
        }
        self.events.lock().unwrap().push("describe".into());
        Ok(SelectionCapabilities {
            profile: SelectionProfile {
                id: "weft.update-selection".into(),
                version: "1".into(),
            },
            claims: ["selection.interpretation", "selection.completeRead"]
                .iter()
                .map(|id| SelectionClaim {
                    id: (*id).into(),
                    status: self.status.clone(),
                    evidence: vec!["fixture://independent-test-consumer-only".into()],
                })
                .collect(),
        })
    }
    fn assess_selection(
        &self,
        _: &Artifact,
        _: &Artifact,
        _: &Artifact,
    ) -> MResult<SelectionAssessment> {
        if self.mode == "assess-error" {
            return Err(bad_diagnostic());
        }
        self.events.lock().unwrap().push("assess".into());
        Ok(SelectionAssessment {
            status: if self.mode == "unknown" {
                SelectionStatus::Unknown
            } else {
                SelectionStatus::Supported
            },
            diagnostics: if self.mode == "supported-diagnostics" {
                vec![MutationDiagnostic {
                    code: "FIXTURE".into(),
                    phase: "action".into(),
                    path: "".into(),
                    message: "Meaning must not disappear".into(),
                }]
            } else {
                vec![]
            },
        })
    }
    fn lower_selection(
        &self,
        selection: &Artifact,
        model: &Artifact,
        _: &Artifact,
        max_items: u32,
    ) -> MResult<SelectionLowering> {
        self.events.lock().unwrap().push("lower".into());
        if self.mode == "lower-error" {
            return Err(bad_diagnostic());
        }
        // An independent small-row consumer checks the unchanged carrier, with hand-authored expected IDs.
        let source: Value = serde_json::from_str(&selection.utf8).unwrap();
        let c = &source["predicate"]["compare"];
        assert_eq!(c["field"], json!({"module":"sales","element":"last-seen"}));
        assert_eq!(c["op"], "lt");
        let cutoff = c["value"]["integerToken"]
            .as_str()
            .unwrap()
            .parse::<i128>()
            .unwrap();
        let selected: Vec<i128> = [(2, 1), (10, 2), (11, 99), (12, 101)]
            .into_iter()
            .filter(|(_, last)| *last < cutoff)
            .map(|(id, _)| id)
            .collect();
        assert_eq!(selected, vec![2, 10, 11]);
        let key = json!({"module":"sales","element":"customer-id"});
        let columns = vec![
            SelectionColumn {
                name: "identity".into(),
                kind: "identity".into(),
                field: None,
            },
            SelectionColumn {
                name: "version".into(),
                kind: "version".into(),
                field: None,
            },
            SelectionColumn {
                name: "key0".into(),
                kind: "key".into(),
                field: Some(if self.mode == "key" {
                    json!({"module":"sales","element":"active"})
                } else {
                    key
                }),
            },
        ];
        let sql = match self.mode {
            "sql-exact" => "é".repeat(32768),
            "sql-overflow" => format!("{}x", "é".repeat(32768)),
            _ => "fixture-only-program".into(),
        };
        let parameters = if self.mode.starts_with("literal-") {
            vec![SelectionParameter {
                name: "string-key".into(),
                field: json!({"module":"sales","element":"customer-id"}),
                value: json!({"string":"x".repeat(1048576-13+usize::from(self.mode=="literal-overflow"))}),
            }]
        } else {
            vec![]
        };
        let mut result = SelectionLowering::Lowered {
            sql,
            parameters,
            columns,
            preflight: if self.mode == "preflight" {
                vec![]
            } else {
                vec![SelectionPreflight {
                    sql: "fixture-only-domain-preflight".into(),
                    parameters: vec![],
                }]
            },
            ordering: SelectionOrdering {
                kind: if self.mode == "order" {
                    "text-identity"
                } else {
                    "native-integer-identity"
                }
                .into(),
                direction: "asc".into(),
            },
            bounds: SelectionBounds {
                max_items,
                fetch_limit: if self.mode == "overflow" {
                    max_items
                } else {
                    max_items + 1
                },
            },
            obligations: vec![
                "fixture-only; native/source/current-authority still unqualified".into(),
            ],
        };
        if self.mode.starts_with("aggregate-") {
            if let SelectionLowering::Lowered { parameters, .. } = &mut result {
                *parameters = (0..16)
                    .map(|i| SelectionParameter {
                        name: format!("p{i}"),
                        field: json!({"module":"sales","element":"customer-id"}),
                        value: json!({"string":"x".repeat(1048576-13)}),
                    })
                    .collect();
            }
            let size = serde_json::to_vec(&result).unwrap().len();
            let remove = size - 16777216 - usize::from(self.mode == "aggregate-overflow");
            if let SelectionLowering::Lowered { parameters, .. } = &mut result {
                parameters[15].value = json!({"string":"x".repeat(1048576-13-remove)});
            }
        }
        if let Some(count) = self
            .mode
            .strip_prefix("preflight-")
            .and_then(|v| v.parse::<usize>().ok())
        {
            if let SelectionLowering::Lowered { preflight, .. } = &mut result {
                *preflight = (0..count)
                    .map(|_| SelectionPreflight {
                        sql: "fixture-only".into(),
                        parameters: vec![],
                    })
                    .collect();
            }
        }
        if let Some(count) = self
            .mode
            .strip_prefix("parameters-")
            .and_then(|v| v.parse::<usize>().ok())
        {
            if let SelectionLowering::Lowered { parameters, .. } = &mut result {
                *parameters = (0..count)
                    .map(|i| SelectionParameter {
                        name: format!("p{i}"),
                        field: json!({"module":"sales","element":"active"}),
                        value: json!({"boolean":false}),
                    })
                    .collect();
            }
        }
        if let Some(count) = self
            .mode
            .strip_prefix("obligations-")
            .and_then(|v| v.parse::<usize>().ok())
        {
            if let SelectionLowering::Lowered { obligations, .. } = &mut result {
                *obligations = vec!["fixture-only".into(); count];
            }
        }
        if let Some(count) = self
            .mode
            .strip_prefix("columns-name-")
            .and_then(|v| v.parse::<usize>().ok())
        {
            if let SelectionLowering::Lowered { columns, .. } = &mut result {
                columns[0].name = "x".repeat(count);
            }
        }
        if let Some(count) = self
            .mode
            .strip_prefix("diagnostics-")
            .and_then(|v| v.parse::<usize>().ok())
        {
            result = SelectionLowering::Blocked {
                diagnostics: vec![
                    MutationDiagnostic {
                        code: "FIXTURE".into(),
                        phase: "action".into(),
                        path: "".into(),
                        message: "fixture unsupported".into()
                    };
                    count
                ],
            };
        }
        if self.mode == "many-key" {
            let doc = model.read().unwrap();
            let fields = doc["modules"][0]["elements"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["id"] == "customer")
                .unwrap()["keys"][0]["fields"]
                .as_array()
                .unwrap();
            if let SelectionLowering::Lowered { columns, .. } = &mut result {
                columns.truncate(2);
                columns.extend(fields.iter().enumerate().map(|(i, field)| SelectionColumn {
                    name: format!("key{i}"),
                    kind: "key".into(),
                    field: Some(field.clone()),
                }));
            }
        }
        Ok(result)
    }
}
fn consumer(
    mode: &'static str,
    status: SelectionStatus,
) -> (Box<dyn SelectionBackend>, Arc<Mutex<Vec<String>>>) {
    let events = Arc::new(Mutex::new(vec![]));
    (
        Box::new(Consumer {
            mode,
            status,
            events: events.clone(),
        }),
        events,
    )
}
#[test]
fn independent_executor_registration_and_specific_refusal() {
    let (mut registry, selection, model, binding) = inputs();
    let (first, events) = consumer("normal", SelectionStatus::Supported);
    let good = registry.install_selection_backend(first).unwrap();
    let (second, other) = consumer("normal", SelectionStatus::Unsupported);
    let missing = registry.install_selection_backend(second).unwrap();
    assert!(matches!(
        registry
            .lower_selection(good, &selection, &model, &binding, 10000)
            .unwrap(),
        SelectionLowering::Lowered { .. }
    ));
    assert!(matches!(
        registry
            .lower_selection(missing, &selection, &model, &binding, 10000)
            .unwrap(),
        SelectionLowering::Blocked { .. }
    ));
    assert_eq!(*events.lock().unwrap(), vec!["describe", "assess", "lower"]);
    assert_eq!(*other.lock().unwrap(), vec!["describe"]);
    let mut caps = registry.selection_capabilities(good).unwrap();
    caps.claims.clear();
    assert_eq!(
        registry.selection_capabilities(good).unwrap().claims.len(),
        2
    );
    let value = serde_json::to_value(registry.selection_capabilities(good).unwrap()).unwrap();
    assert!(value.get("executionVerified").is_none());
    assert_eq!(
        value["claims"][0]["evidence"][0],
        "fixture://independent-test-consumer-only"
    );
}
#[test]
fn incomplete_lowering_and_assessment_never_fall_back() {
    for mode in ["order", "overflow", "key", "preflight", "unknown"] {
        let (mut registry, selection, model, binding) = inputs();
        let (backend, events) = consumer(mode, SelectionStatus::Supported);
        let handle = registry.install_selection_backend(backend).unwrap();
        let result = registry.lower_selection(handle, &selection, &model, &binding, 10000);
        if mode == "unknown" {
            assert!(matches!(result.unwrap(), SelectionLowering::Blocked { .. }));
            assert_eq!(*events.lock().unwrap(), vec!["describe", "assess"]);
        } else {
            assert!(result.is_err(), "{mode}");
        }
    }
}
#[test]
fn original_transport_profile_and_bounds_refuse_before_dispatch() {
    let (mut registry, selection, model, binding) = inputs();
    let (backend, events) = consumer("normal", SelectionStatus::Supported);
    let handle = registry.install_selection_backend(backend).unwrap();
    for max in [0, 10001] {
        assert!(registry
            .lower_selection(handle, &selection, &model, &binding, max)
            .is_err());
    }
    let mut p = selection.read().unwrap();
    p["protocol"] = json!("weft.update-selection/99");
    assert!(registry
        .lower_selection(handle, &artifact(p), &model, &binding, 100)
        .is_err());
    let mut bad = binding.clone();
    bad.sha256 = "0".repeat(64);
    assert!(registry
        .lower_selection(handle, &selection, &model, &bad, 100)
        .is_err());
    assert!(registry
        .lower_selection(
            handle,
            &selection,
            &artifact(json!({"umf":"0.8.0"})),
            &binding,
            100
        )
        .is_err());
    assert_eq!(*events.lock().unwrap(), vec!["describe"]);
}
#[test]
fn old_read_interface_and_no_implicit_mutation_upgrade() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../docs/helix/03-test/fixtures/cases.json")).unwrap();
    let read =
        weft_core::compile::Compiler::default().compile_json(&fixture[0]["request"].to_string());
    let result: Value = serde_json::from_str(&read).unwrap();
    assert_eq!(result["interfaceVersion"], "weft-compile/0.1.0");
    assert_eq!(result["diagnostics"][0]["code"], "WFT-BACKEND-MISSING");
    let (mut registry, _, _, _) = inputs();
    let (backend, _) = consumer("normal", SelectionStatus::Supported);
    registry.install_selection_backend(backend).unwrap();
    assert_eq!(
        weft_core::compile::Compiler::default().compile_json(&fixture[0]["request"].to_string()),
        read
    );
    let mut r: Value = serde_json::from_str(include_str!("fixtures/request.json")).unwrap();
    r["dialect"] = json!("weft-mutation/99");
    let rejected: Value = serde_json::from_str(&registry.compile_json(&r.to_string())).unwrap();
    assert_eq!(rejected["status"], "blocked");
    assert!(rejected.get("plan").is_none());
}

fn bad_diagnostic() -> MutationDiagnostic {
    MutationDiagnostic {
        code: "BAD".into(),
        phase: "native-uncontracted".into(),
        path: "".into(),
        message: "x".repeat(4097),
    }
}
#[test]
fn callback_error_diagnostics_and_supported_messages_are_not_lost() {
    for mode in [
        "describe-error",
        "assess-error",
        "lower-error",
        "supported-diagnostics",
    ] {
        let (mut registry, selection, model, binding) = inputs();
        let (backend, _) = consumer(mode, SelectionStatus::Supported);
        let result = registry
            .install_selection_backend(backend)
            .and_then(|handle| registry.lower_selection(handle, &selection, &model, &binding, 10));
        let error = result.unwrap_err();
        assert_eq!(error.phase, "action");
        assert!(error.message.len() < 4096);
        assert_eq!(error.code, "WFT-MUTATION-ACTION");
    }
}
#[test]
fn exact_lowering_utf8_literal_and_aggregate_byte_bounds() {
    for (mode, allowed) in [
        ("sql-exact", true),
        ("sql-overflow", false),
        ("literal-exact", true),
        ("literal-overflow", false),
        ("aggregate-exact", true),
        ("aggregate-overflow", false),
    ] {
        let (mut registry, selection, model, binding) = inputs();
        let (backend, _) = consumer(mode, SelectionStatus::Supported);
        let handle = registry.install_selection_backend(backend).unwrap();
        let outcome = registry.lower_selection(handle, &selection, &model, &binding, 10);
        assert_eq!(outcome.is_ok(), allowed, "{mode}");
        if !allowed {
            assert_eq!(outcome.unwrap_err().code, "WFT-MUTATION-LIMIT", "{mode}");
        }
    }
}
#[test]
fn owner_preparation_target_closure_and_original_key_domains() {
    for (kind, text) in [
        ("binary", include_str!("fixtures/selection-binary.json")),
        ("decimal", include_str!("fixtures/selection-decimal.json")),
        ("other", include_str!("fixtures/selection-other.json")),
    ] {
        let fixture: Value = serde_json::from_str(text).unwrap();
        let mut registry =
            MutationRegistry::from_json(&fixture["configuration"].to_string()).unwrap();
        let response: Value =
            serde_json::from_str(&registry.compile_json(&fixture["request"].to_string())).unwrap();
        assert_eq!(response["status"], "compiled", "{kind}: {response}");
        let plan: Value = serde_json::from_str(response["plan"]["utf8"].as_str().unwrap()).unwrap();
        let selection: Artifact =
            serde_json::from_value(plan["selection"]["artifact"].clone()).unwrap();
        let model: Artifact = serde_json::from_value(fixture["request"]["model"].clone()).unwrap();
        let binding = artifact(json!({"fixture":true}));
        let (backend, events) = consumer("normal", SelectionStatus::Supported);
        let handle = registry.install_selection_backend(backend).unwrap();
        assert!(registry
            .lower_selection(handle, &selection, &model, &binding, 10)
            .is_ok());
        if kind == "other" {
            let mut program = selection.read().unwrap();
            program["record"] = json!({"module":"sales","element":"other"});
            program["key"] = json!({"module":"sales","element":"other","key":"other-pk"});
            program["predicate"] = json!({"compare":{"field":{"module":"sales","element":"other-id"},"op":"eq","value":{"string":"bar"}}});
            let before = events.lock().unwrap().len();
            assert!(registry
                .lower_selection(handle, &artifact(program), &model, &binding, 10)
                .is_err());
            assert_eq!(events.lock().unwrap().len(), before);
        }
    }
}

#[test]
fn selection_collection_metadata_and_registration_boundaries() {
    for (mode, allowed) in [
        ("preflight-64", true),
        ("preflight-65", false),
        ("parameters-1024", true),
        ("parameters-1025", false),
        ("obligations-128", true),
        ("obligations-129", false),
        ("columns-name-256", true),
        ("columns-name-257", false),
        ("diagnostics-128", true),
        ("diagnostics-129", false),
    ] {
        let (mut registry, selection, model, binding) = inputs();
        let (backend, _) = consumer(mode, SelectionStatus::Supported);
        let handle = registry.install_selection_backend(backend).unwrap();
        let outcome = registry.lower_selection(handle, &selection, &model, &binding, 10);
        assert_eq!(outcome.is_ok(), allowed, "{mode}");
        if !allowed {
            assert_eq!(outcome.unwrap_err().code, "WFT-MUTATION-LIMIT", "{mode}");
        }
    }
    let (mut registry, _, _, _) = inputs();
    for _ in 0..64 {
        let (backend, _) = consumer("normal", SelectionStatus::Unknown);
        registry.install_selection_backend(backend).unwrap();
    }
    let (backend, _) = consumer("normal", SelectionStatus::Unknown);
    assert_eq!(
        registry
            .install_selection_backend(backend)
            .unwrap_err()
            .code,
        "WFT-MUTATION-LIMIT"
    );
}

#[test]
fn binding_and_direct_selection_framing_limits_have_limit_codes() {
    for (kind, maximum) in [
        ("bytes", 4194304usize),
        ("depth", 64),
        ("nodes", 100000),
        ("operands", 64),
    ] {
        for extra in [0, 1] {
            let (mut registry, selection, model, mut binding) = inputs();
            let (backend, _) = consumer("normal", SelectionStatus::Supported);
            let handle = registry.install_selection_backend(backend).unwrap();
            let mut chosen = selection.clone();
            match kind {
                "bytes" => binding = artifact(json!({"x":"x".repeat(maximum+extra-8)})),
                "depth" => {
                    let mut value = json!(0);
                    for _ in 0..maximum + extra - 1 {
                        value = json!([value]);
                    }
                    binding = artifact(json!({"x":value}));
                }
                "nodes" => binding = artifact(json!({"x":vec![Value::Null;maximum+extra-2]})),
                "operands" => {
                    let mut program = selection.read().unwrap();
                    program["predicate"] =
                        json!({"and":vec![program["predicate"].clone();maximum+extra]});
                    chosen = artifact(program);
                }
                _ => unreachable!(),
            }
            if kind == "operands" && extra == 0 {
                // Shape is admitted; fixture consumer only implements a single comparison.
                // Use an unsupported declaration to witness input admission without calling fixture semantics.
                let (mut r, _, _, _) = inputs();
                let (b, _) = consumer("normal", SelectionStatus::Unsupported);
                let h = r.install_selection_backend(b).unwrap();
                assert!(matches!(
                    r.lower_selection(h, &chosen, &model, &binding, 10).unwrap(),
                    SelectionLowering::Blocked { .. }
                ));
            } else {
                let result = registry.lower_selection(handle, &chosen, &model, &binding, 10);
                if extra == 0 {
                    assert!(result.is_ok(), "{kind}: {result:?}");
                } else {
                    assert_eq!(result.unwrap_err().code, "WFT-MUTATION-LIMIT", "{kind}");
                }
            }
        }
    }
}
#[test]
fn original_owner_key_component_transport_limit() {
    for (allowed, text) in [
        (true, include_str!("fixtures/selection-key64.json")),
        (false, include_str!("fixtures/selection-key65.json")),
    ] {
        let f: Value = serde_json::from_str(text).unwrap();
        let mut registry = MutationRegistry::from_json(&f["configuration"].to_string()).unwrap();
        let response: Value =
            serde_json::from_str(&registry.compile_json(&f["request"].to_string())).unwrap();
        assert_eq!(response["status"], "compiled");
        let p: Value = serde_json::from_str(response["plan"]["utf8"].as_str().unwrap()).unwrap();
        let selection: Artifact =
            serde_json::from_value(p["selection"]["artifact"].clone()).unwrap();
        let model: Artifact = serde_json::from_value(f["request"]["model"].clone()).unwrap();
        let (backend, _) = consumer("many-key", SelectionStatus::Supported);
        let handle = registry.install_selection_backend(backend).unwrap();
        let result = registry.lower_selection(
            handle,
            &selection,
            &model,
            &artifact(json!({"fixture":true})),
            10,
        );
        assert_eq!(result.is_ok(), allowed);
        if !allowed {
            assert_eq!(result.unwrap_err().code, "WFT-MUTATION-LIMIT");
        }
    }
}
