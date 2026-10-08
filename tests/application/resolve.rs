// @covers US-007-AC1 @covers US-007-AC2 @covers US-007-AC3
// @covers US-007-AC4 @covers US-007-AC5 @covers US-007-AC6
use serde_json::{json, Value};
use weft_core::frontend_json;
fn corpus() -> Value {
    serde_json::from_str(include_str!("fixtures/cases.json")).unwrap()
}
fn run(r: &Value) -> Value {
    serde_json::from_str(&frontend_json(&r.to_string())).unwrap()
}
#[test]
fn application_acceptance_and_refusals() {
    for c in corpus().as_array().unwrap() {
        let r = run(&c["request"]);
        assert_eq!(r["status"], c["expected"]["status"], "{}: {}", c["id"], r);
        if r["status"] == "blocked" {
            assert_eq!(
                r["diagnostics"][0]["code"], c["expected"]["code"],
                "{}: {}",
                c["id"], r
            );
            assert!(r.get("logicalPlan").is_none());
        }
    }
}
#[test]
fn plans_keep_domains_presence_join_bags_and_cursors() {
    let cases = corpus();
    let find = |id: &str| {
        cases
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == id)
            .unwrap()
    };
    let r = run(&find("whole-entity")["request"]);
    let p = &r["logicalPlan"];
    assert_eq!(
        p["outputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["id", "name", "active", "nickname", "tags", "address"]
    );
    assert!(p["requiredCapabilities"]
        .as_array()
        .unwrap()
        .contains(&json!("value.presence")));
    assert!(p["requiredCapabilities"]
        .as_array()
        .unwrap()
        .contains(&json!("value.sequence")));
    assert_eq!(p["typeGraph"][4]["availability"], "absent-allowed");
    let r = run(&find("composite-cursor")["request"]);
    assert_eq!(
        r["logicalPlan"]["filters"][0]["columns"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(r["logicalPlan"]["filters"][0]["op"], "lexicographicGreater");
    let r = run(&find("join-count")["request"]);
    assert_eq!(r["logicalPlan"]["joins"].as_array().unwrap().len(), 1);
    assert_eq!(r["logicalPlan"]["outputs"][0]["expression"]["op"], "count");
    assert_eq!(
        r["logicalPlan"]["outputs"][0]["expression"]["type"]["facets"],
        json!({})
    );
}
#[test]
fn parameters_intersect_all_use_domains() {
    let mut r = corpus()[0]["request"].clone();
    r["sql"] =
        json!("SELECT c.id FROM Customer c WHERE c.id=:id AND c.id=:id ORDER BY c.id LIMIT 10");
    r["parameters"] = json!({"id":{"family":"integer","value":"18446744073709551615"}});
    assert_eq!(run(&r)["status"], "resolved");
    let mut d: Value =
        serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    d["modules"][0]["elements"][4]["scalarType"] = json!("integer");
    d["modules"][0]["elements"][4]["facets"] = json!({"integerWidth":{"bits":8,"signed":false}});
    let text = d.to_string();
    r["modules"][0]["documentJson"] = json!(text);
    r["modules"][0]["pin"]["sha256"] = json!(weft_core::json::sha256(text.as_bytes()));
    r["sql"] =
        json!("SELECT c.id FROM Customer c WHERE c.id=:id AND c.active=:id ORDER BY c.id LIMIT 10");
    assert_eq!(run(&r)["diagnostics"][0]["code"], "WFT-NUMERIC-DOMAIN");
    r["parameters"]["id"]["value"] = json!("255");
    assert_eq!(run(&r)["status"], "resolved");
}
#[test]
fn original_valid_queries_remain_valid_in_explicit_02() {
    let cases: Value =
        serde_json::from_str(include_str!("../../docs/helix/03-test/fixtures/cases.json")).unwrap();
    for c in cases
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["expected"]["status"] == "compiled")
    {
        let mut r = c["request"].clone();
        r["dialect"] = json!("weft-sql/0.2.0");
        assert_eq!(run(&r)["status"], "resolved", "{}", c["id"]);
    }
}
#[test]
fn explicit_profile_version_and_parameter_map_bounds() {
    let mut r = corpus()[0]["request"].clone();
    r["readProfile"]["version"] = json!("weft-application-read/0.3.0");
    assert_eq!(run(&r)["diagnostics"][0]["code"], "WFT-VERSION");
    r["readProfile"]["version"] = json!("weft-application-read/0.2.0");
    let parameters = (0..1025)
        .map(|n| (format!("p{n}"), json!({"family":"integer","value":"1"})))
        .collect::<serde_json::Map<_, _>>();
    r["parameters"] = json!(parameters);
    assert_eq!(run(&r)["diagnostics"][0]["code"], "WFT-LIMIT");
}

// @covers US-007-AC4 @covers US-007-AC6 @covers US-006-AC3 @covers US-006-AC4
#[test]
fn direct_parameter_boundaries_and_quoted_parameter_guard() {
    use weft_core::{
        application_resolve as resolver, application_syntax as syntax, model::Catalog,
    };
    let request = corpus()[0]["request"].clone();
    let catalog =
        Catalog::prepare(serde_json::from_value(request["modules"].clone()).unwrap()).unwrap();
    let original = syntax::parse("SELECT c.id FROM Customer c WHERE c.id=:p").unwrap();
    let mut at = original.clone();
    let mut parameters = resolver::Parameters::new();
    for i in 0..1024 {
        let name = format!("p{i}");
        parameters.insert(
            name.clone(),
            resolver::Parameter {
                family: weft_core::ir::Family::Integer,
                value: "1".into(),
            },
        );
        let mut predicate = original.predicates[0].clone();
        let syntax::Predicate::Compare { ref mut values, .. } = predicate else {
            panic!("comparison required")
        };
        let syntax::Value::Parameter(parameter) = &mut values[0] else {
            panic!("parameter required")
        };
        parameter.value = name;
        if i == 0 {
            at.predicates.clear();
        }
        at.predicates.push(predicate);
    }
    let plan = resolver::resolve(&catalog, at.clone(), parameters.clone(), None).unwrap();
    assert_eq!(plan.filters.len(), 1024);
    assert!(plan
        .required_capabilities
        .contains(&"parameter.named".into()));
    assert!(plan.required_capabilities.contains(&"and".into()));
    parameters.insert(
        "extra".into(),
        resolver::Parameter {
            family: weft_core::ir::Family::Integer,
            value: "1".into(),
        },
    );
    assert_eq!(
        resolver::resolve(&catalog, at, parameters, None)
            .unwrap_err()
            .message,
        "Source parameter count exceeds 1024"
    );
    let mut quoted = original;
    let syntax::Predicate::Compare { ref mut values, .. } = quoted.predicates[0] else {
        panic!("comparison required")
    };
    let syntax::Value::Parameter(name) = &mut values[0] else {
        panic!("parameter required")
    };
    name.quoted = true;
    let parameters = [(
        "p".into(),
        resolver::Parameter {
            family: weft_core::ir::Family::Integer,
            value: "1".into(),
        },
    )]
    .into_iter()
    .collect();
    let error = resolver::resolve(&catalog, quoted, parameters, None).unwrap_err();
    assert_eq!(error.code, "WFT-PARAMETER");
    assert_eq!(
        error.message,
        "Source parameter names must be unquoted ASCII identifiers"
    );
}

#[test]
fn application_parameter_and_read_recognizer_refusals_are_explicit() {
    let base = corpus()[0]["request"].clone();
    let mut request = base.clone();
    request["readProfile"] = Value::Null;
    for (sql, parameters, message) in [
        (
            "SELECT c.name FROM Customer c WHERE c.name=:p",
            json!({"p":{"family":"string","value":"a\u{0}b"}}),
            "Source parameter value violates the exact lexical grammar",
        ),
        (
            "SELECT c.active FROM Customer c WHERE c.active=:p",
            json!({"p":{"family":"boolean","value":"TRUE"}}),
            "Source parameter value violates the exact lexical grammar",
        ),
        (
            "SELECT c.id FROM Customer c",
            json!({"bad-name":{"family":"integer","value":"1"}}),
            "Parameter names must be distinct ASCII identifiers after case folding",
        ),
    ] {
        request["sql"] = json!(sql);
        request["parameters"] = parameters;
        let result = run(&request);
        assert_eq!(
            result["diagnostics"][0]["code"], "WFT-PARAMETER",
            "{result}"
        );
        assert_eq!(result["diagnostics"][0]["message"], message);
        assert!(result.get("logicalPlan").is_none());
    }
    for (sql, subset, message) in [
        (
            "SELECT COUNT(*) FROM Customer c LIMIT 1",
            "count-summary",
            "Global count-summary has one row and excludes ORDER BY and LIMIT",
        ),
        (
            "SELECT COUNT(*) FROM Customer c WHERE c.id>1",
            "count-summary",
            "count-summary excludes cursor and relationship predicates",
        ),
        (
            "SELECT COUNT(*) FROM Customer c WHERE HAS_RELATED(c.orders,KEY(1))",
            "count-summary",
            "count-summary excludes cursor and relationship predicates",
        ),
        (
            "SELECT c.id FROM Customer c JOIN Orders o ON o.customer_id=c.id ORDER BY c.id LIMIT 1",
            "entity-page",
            "Entity-page profiles require a single nonaggregated source",
        ),
        (
            "SELECT c.id FROM Customer c WHERE c.id>c.id ORDER BY c.id LIMIT 1",
            "entity-page",
            "Cursor must compare the complete ordered key to literal or parameter tuple",
        ),
        (
            "SELECT c.id FROM Customer c",
            "count-summary",
            "count-summary requires COUNT(*) and optional grouped scalar projections",
        ),
    ] {
        let mut request = base.clone();
        request["sql"] = json!(sql);
        request["readProfile"]["subset"] = json!(subset);
        let result = run(&request);
        assert_eq!(result["diagnostics"][0]["code"], "WFT-PROFILE", "{result}");
        assert_eq!(result["diagnostics"][0]["message"], message);
        assert!(result.get("logicalPlan").is_none());
    }
}

#[test]
fn entity_expansion_boundary_and_empty_typed_projection() {
    use weft_core::{application_resolve, application_syntax, model::Catalog};
    let base = corpus()[0]["request"].clone();
    for total in [256, 257] {
        let mut request = base.clone();
        request["readProfile"] = Value::Null;
        let mut document: Value =
            serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
        let elements = document["modules"][0]["elements"].as_array_mut().unwrap();
        let record = elements
            .iter()
            .position(|e| e["name"] == "Customer")
            .unwrap();
        let count = elements[record]["members"].as_array().unwrap().len();
        for n in count..total {
            let id = format!("expanded-{n}");
            elements[record]["members"]
                .as_array_mut()
                .unwrap()
                .push(json!({"module":"sales","element":id}));
            elements.push(json!({"id":id,"name":format!("expanded_{n}"),"kind":"field","scalarType":"string","cardinality":"one","nullability":"required","extensions":{}}));
        }
        let raw = document.to_string();
        request["modules"][0]["documentJson"] = json!(raw);
        request["modules"][0]["pin"]["sha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
        let result = run(&request);
        if total == 256 {
            assert_eq!(result["status"], "resolved", "{result}");
            assert_eq!(
                result["logicalPlan"]["outputs"].as_array().unwrap().len(),
                256
            );
        } else {
            assert_eq!(result["diagnostics"][0]["code"], "WFT-LIMIT", "{result}");
            assert_eq!(
                result["diagnostics"][0]["message"],
                "Expanded output count exceeds 256"
            );
            assert!(result.get("logicalPlan").is_none());
        }
    }
    let catalog =
        Catalog::prepare(serde_json::from_value(base["modules"].clone()).unwrap()).unwrap();
    let mut query = application_syntax::parse("SELECT c.id FROM Customer c").unwrap();
    query.outputs.clear();
    let error =
        application_resolve::resolve(&catalog, query, Default::default(), None).unwrap_err();
    assert_eq!(error.code, "WFT-OUTPUT-NAME");
    assert_eq!(error.message, "Projection expands to no output fields");
}

#[test]
fn application_predicate_scan_occurrence_and_numeric_result_branches() {
    let mut request = corpus()[0]["request"].clone();
    request["readProfile"] = Value::Null;
    for (sql, code, message) in [
        (
            "SELECT c.id FROM Customer c JOIN Orders c ON c.id=c.customer_id",
            "WFT-NAME-AMBIGUOUS",
            "Repeated source alias",
        ),
        (
            "SELECT c.id FROM Customer c WHERE c.id=c.name",
            "WFT-TYPE",
            "Compared fields require the same exact family",
        ),
        (
            "SELECT c.id FROM Customer c JOIN Orders o ON HAS_RELATED(c.orders,KEY(1))",
            "WFT-UNSUPPORTED",
            "HAS_RELATED is a WHERE predicate",
        ),
        (
            "SELECT c.id FROM Customer c JOIN Orders o ON c.id>o.customer_id",
            "WFT-UNSUPPORTED",
            "JOIN ON requires field equality",
        ),
        (
            "SELECT c.id FROM Customer c WHERE HAS_RELATED(c.orders,KEY(c.id))",
            "WFT-UNSUPPORTED",
            "Related key operands require literals or parameters",
        ),
        (
            "SELECT a.id,COUNT(*) FROM Customer a JOIN Customer b ON a.id=b.id GROUP BY b.id",
            "WFT-GROUPING",
            "Projected field is not grouped",
        ),
    ] {
        request["sql"] = json!(sql);
        let result = run(&request);
        assert_eq!(result["diagnostics"][0]["code"], code, "{result}");
        assert_eq!(result["diagnostics"][0]["message"], message);
        assert!(result.get("logicalPlan").is_none());
    }
    request["sql"]=json!("SELECT a.id AS a_id,b.id AS b_id,COUNT(*) FROM Customer a JOIN Customer b ON a.id=b.id GROUP BY a.id,b.id");
    let result = run(&request);
    assert_eq!(result["status"], "resolved", "{result}");
    let outputs = result["logicalPlan"]["outputs"].as_array().unwrap();
    assert_ne!(
        outputs[0]["expression"]["scan"],
        outputs[1]["expression"]["scan"]
    );
    assert_eq!(
        outputs[0]["expression"]["identity"],
        outputs[1]["expression"]["identity"]
    );
    request["sql"] = json!("SELECT o.id FROM Orders o WHERE HAS_RELATED(o.customer,KEY(1))");
    let result = run(&request);
    assert_eq!(result["status"], "resolved", "{result}");
    for capability in ["relationship.exists", "relationship.inverse"] {
        assert!(result["logicalPlan"]["requiredCapabilities"]
            .as_array()
            .unwrap()
            .contains(&json!(capability)));
    }
    for value in ["true", "false"] {
        request["sql"] = json!("SELECT c.active FROM Customer c WHERE c.active=:p");
        request["parameters"] = json!({"p":{"family":"boolean","value":value}});
        let result = run(&request);
        assert_eq!(result["status"], "resolved", "{result}");
        assert_eq!(result["logicalPlan"]["filters"][0]["right"]["value"], value);
    }
    request["parameters"] = json!({});
    for (sql, family, facets, nullable) in [
        (
            "SELECT SUM(c.id) FROM Customer c",
            "integer",
            json!({}),
            true,
        ),
        (
            "SELECT SUM(c.id) FROM Customer c GROUP BY c.name",
            "integer",
            json!({}),
            false,
        ),
        (
            "SELECT SUM(o.total) FROM Orders o",
            "decimal",
            json!({"scale":2}),
            true,
        ),
        (
            "SELECT SUM(o.total) FROM Orders o GROUP BY o.id",
            "decimal",
            json!({"scale":2}),
            false,
        ),
    ] {
        request["sql"] = json!(sql);
        let result = run(&request);
        assert_eq!(result["status"], "resolved", "{result}");
        assert_eq!(
            result["logicalPlan"]["outputs"][0]["expression"]["type"],
            json!({"family":family,"facets":facets,"nullable":nullable})
        );
    }
    for absent in [true, false] {
        let mut request = corpus()[0]["request"].clone();
        let mut doc: Value =
            serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
        let customer = doc["modules"][0]["elements"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["name"] == "Customer")
            .unwrap();
        if absent {
            customer.as_object_mut().unwrap().remove("keys");
        } else {
            let mut key = customer["keys"][0].clone();
            key["id"] = json!("alternative-key");
            customer["keys"].as_array_mut().unwrap().push(key);
        }
        let raw = doc.to_string();
        request["modules"][0]["documentJson"] = json!(raw);
        request["modules"][0]["pin"]["sha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
        let result = run(&request);
        assert_eq!(result["diagnostics"][0]["code"], "WFT-PROFILE", "{result}");
        assert_eq!(
            result["diagnostics"][0]["message"],
            if absent {
                "Entity page needs an authored key"
            } else {
                "Entity page requires unambiguous complete authored key order and LIMIT"
            }
        );
    }
}
