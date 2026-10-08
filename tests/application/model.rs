use serde_json::{json, Value};
use weft_core::{
    ir::{ModelPin, Span},
    json::sha256,
    model::{Catalog, ModuleInput},
    syntax::Name,
};
fn document() -> Value {
    serde_json::from_str(include_str!(
        "../../docs/helix/03-test/fixtures/sales.umf.json"
    ))
    .unwrap()
}
fn try_catalog(doc: Value) -> Result<Catalog, weft_core::error::Diagnostic> {
    let text = doc.to_string();
    Catalog::prepare(vec![ModuleInput {
        document_json: text.clone(),
        pin: ModelPin {
            document_id: "sales-fixture".into(),
            revision: "app-fixture".into(),
            umf_version: "0.7.0".into(),
            sha256: sha256(text.as_bytes()),
        },
        selected_module_ids: vec!["sales".into()],
    }])
}
fn catalog(doc: Value) -> Catalog {
    try_catalog(doc).unwrap()
}
fn customer(c: &Catalog) -> weft_core::model::Record {
    c.record(
        None,
        &Name {
            value: "customer".into(),
            quoted: false,
            span: Span { start: 0, end: 0 },
        },
    )
    .unwrap()
}
#[test]
fn ordered_members_and_optional_availability() {
    let mut d = document();
    d["modules"][0]["elements"][3]["nullability"] = json!("absent-allowed");
    let c = catalog(d);
    let r = customer(&c);
    let e = c.entity_descriptor(&r).unwrap();
    assert_eq!(
        e.members
            .iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>(),
        vec!["id", "name", "active"]
    );
    assert_eq!(e.graph[2].availability.as_deref(), Some("absent-allowed"));
    // Availability cannot become SQL nullable.
    let v = serde_json::to_value(e).unwrap();
    assert_eq!(v["graph"][2]["type"]["nullable"], false);
    assert!(c
        .field(
            &r,
            &Name {
                value: "name".into(),
                quoted: false,
                span: Span { start: 0, end: 0 }
            }
        )
        .is_err());
}
#[test]
fn recursive_structure_is_an_identity_graph() {
    let mut d = document();
    let elements = d["modules"][0]["elements"].as_array_mut().unwrap();
    elements[0]["members"]
        .as_array_mut()
        .unwrap()
        .push(json!({"module":"sales","element":"parent"}));
    elements.push(json!({"id":"parent","name":"parent","kind":"field","cardinality":"one","nullability":"absent-allowed","references":[{"role":"record-type","module":"sales","element":"customer"}],"extensions":{}}));
    let c = catalog(d);
    let e = c.entity_descriptor(&customer(&c)).unwrap();
    assert_eq!(e.graph.len(), 5);
    let v = serde_json::to_value(e).unwrap();
    assert_eq!(v["graph"][4]["record"]["element"], "customer");
}
#[test]
fn ordered_list_items_have_exact_leaf_types() {
    let mut d = document();
    let elements = d["modules"][0]["elements"].as_array_mut().unwrap();
    elements[0]["members"]
        .as_array_mut()
        .unwrap()
        .push(json!({"module":"sales","element":"totals"}));
    elements.push(json!({"id":"totals","name":"totals","kind":"field","cardinality":"array","nullability":"required","itemType":{"module":"sales","element":"order-total"},"extensions":{}}));
    let c = catalog(d);
    let e = c.entity_descriptor(&customer(&c)).unwrap();
    let v = serde_json::to_value(e).unwrap();
    assert_eq!(v["graph"][4]["kind"], "sequence");
    assert_eq!(v["graph"][5]["type"]["facets"]["precision"], 28);
}
#[test]
fn keys_require_required_member_scalars() {
    let mut d = document();
    d["modules"][0]["elements"][0]["keys"] = json!([{"id":"pk","name":"pk","fields":[{"module":"sales","element":"customer-id"}],"primary":true}]);
    let c = catalog(d.clone());
    let k = c.authored_key(&customer(&c), "pk").unwrap();
    assert_eq!(k.fields[0].element, "customer-id");
    assert!(c.authored_key(&customer(&c), "unknown").is_err());
    d["modules"][0]["elements"][2]["nullability"] = json!("absent-allowed");
    let c = catalog(d);
    assert!(c.authored_key(&customer(&c), "pk").is_err());
}
#[test]
fn unknown_selected_meaning_is_not_dropped() {
    for nullability in ["unspecified", "future"] {
        let mut d = document();
        d["modules"][0]["elements"][3]["nullability"] = json!(nullability);
        let c = catalog(d);
        assert!(c.entity_descriptor(&customer(&c)).is_err());
    }
    let mut d = document();
    d["modules"][0]["elements"][3]["scalarType"] = json!("future");
    let c = catalog(d);
    assert!(c.entity_descriptor(&customer(&c)).is_err());
    let mut d = document();
    d["modules"][0]["elements"][0]["members"][0]["element"] = json!("missing");
    let c = catalog(d);
    assert!(c.entity_descriptor(&customer(&c)).is_err());
}
fn relationship_document() -> Value {
    let mut d = document();
    d["modules"][0]["elements"][0]["keys"] = json!([{"id":"customer-pk","name":"pk","fields":[{"module":"sales","element":"customer-id"}],"primary":true}]);
    d["modules"][0]["elements"][1]["keys"] = json!([{"id":"order-pk","name":"pk","fields":[{"module":"sales","element":"order-customer"}]}]);
    d["modules"][0]["relationships"] = json!([{"id":"customer-orders","name":"orders","source":[{"module":"sales","element":"customer"}],"target":[{"module":"sales","element":"orders","key":"order-pk"}],"sourceMultiplicity":{"min":0,"max":1},"targetMultiplicity":{"min":0,"max":"*"},"targetLifecycle":"independent","directed":true,"inverse":"customer"}]);
    d
}
fn name(value: &str) -> Name {
    Name {
        value: value.into(),
        quoted: false,
        span: Span { start: 0, end: 0 },
    }
}
#[test]
fn authored_relationship_and_inverse_keep_identity_and_key() {
    let c = catalog(relationship_document());
    let forward = c.relationship_read(&customer(&c), &name("orders")).unwrap();
    assert_eq!(forward.identity.relationship, "customer-orders");
    assert_eq!(forward.target_key.id, "order-pk");
    assert!(!forward.inverse);
    let order = c.record(None, &name("orders")).unwrap();
    let inverse = c.relationship_read(&order, &name("customer")).unwrap();
    assert_eq!(inverse.identity.relationship, "customer-orders");
    assert_eq!(inverse.target_key.id, "customer-pk");
    assert!(inverse.inverse);
    assert_eq!(inverse.to.element, "customer");
}
#[test]
fn unsupported_relationship_meanings_refuse() {
    for change in [
        json!({"directed":false}),
        json!({"targetLifecycle":"future"}),
        json!({"targetLifecycle":"unspecified"}),
        json!({"associationRecord":{"module":"sales","element":"orders"}}),
        json!({"targetMultiplicity":{"min":2,"max":1}}),
    ] {
        let mut d = relationship_document();
        for (k, v) in change.as_object().unwrap() {
            d["modules"][0]["relationships"][0][k] = v.clone();
        }
        let c = catalog(d);
        assert!(c.relationship_read(&customer(&c), &name("orders")).is_err());
    }
    let mut d = relationship_document();
    d["modules"][0]["relationships"][0]["target"][0]["key"] = json!("missing");
    let c = catalog(d);
    assert!(c.relationship_read(&customer(&c), &name("orders")).is_err());
}
#[test]
fn deep_reference_graph_refuses_without_recursing_forever() {
    let mut d = document();
    d["modules"][0]["elements"][0]["members"] = json!([{"module":"sales","element":"chain-0"}]);
    for i in 0..130 {
        d["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":format!("chain-{i}"),"name":format!("chain-{i}"),"kind":"field","nullability":"required","cardinality":"array","itemType":{"module":"sales","element":if i==129 {"customer-id".into()}else{format!("chain-{}",i+1)}},"extensions":{}}));
    }
    let c = catalog(d);
    let err = c.entity_descriptor(&customer(&c)).unwrap_err();
    assert_eq!(err.code, "WFT-LIMIT");
}
#[test]
fn unrelated_unknown_member_does_not_block_explicit_known_projection() {
    let mut d = document();
    d["modules"][0]["elements"][3]["scalarType"] = json!("future");
    let c = catalog(d);
    let r = customer(&c);
    assert!(c.entity_descriptor(&r).is_err());
    assert!(c.member_descriptor(&r, &name("id")).is_ok());
}
#[test]
fn ambiguous_relationship_and_unkeyed_inverse_refuse() {
    let mut d = relationship_document();
    let mut second = d["modules"][0]["relationships"][0].clone();
    second["id"] = json!("second-id");
    d["modules"][0]["relationships"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let c = catalog(d);
    assert_eq!(
        c.relationship_read(&customer(&c), &name("orders"))
            .unwrap_err()
            .code,
        "WFT-NAME-AMBIGUOUS"
    );
    let mut d = relationship_document();
    d["modules"][0]["elements"][0]
        .as_object_mut()
        .unwrap()
        .remove("keys");
    let c = catalog(d);
    let r = c.record(None, &name("orders")).unwrap();
    assert!(c.relationship_read(&r, &name("customer")).is_err());
}

// @covers US-006-AC4
#[test]
fn selected_graph_depth_and_identity_boundaries_are_explicit() {
    for (length, accepted) in [(127, true), (128, false)] {
        let mut d = document();
        d["modules"][0]["elements"][0]["members"] =
            json!([{"module":"sales","element":"boundary-0"}]);
        for i in 0..length {
            let next = if i + 1 == length {
                "customer-id".to_string()
            } else {
                format!("boundary-{}", i + 1)
            };
            d["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":format!("boundary-{i}"),"name":format!("boundary-{i}"),"kind":"field","nullability":"required","cardinality":"array","itemType":{"module":"sales","element":next},"extensions":{}}));
        }
        let c = catalog(d);
        let result = c.entity_descriptor(&customer(&c));
        if accepted {
            assert_eq!(result.unwrap().graph.len(), 129);
        } else {
            assert_eq!(result.unwrap_err().code, "WFT-LIMIT");
        }
    }
    for (fields, accepted) in [(4095, true), (4096, false)] {
        let mut d = document();
        let template = d["modules"][0]["elements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["id"] == "customer-name")
            .unwrap()
            .clone();
        let mut members = Vec::new();
        for i in 0..fields {
            let id = format!("wide-{i}");
            let mut f = template.clone();
            f["id"] = json!(id);
            f["name"] = json!(id);
            members.push(json!({"module":"sales","element":id}));
            d["modules"][0]["elements"].as_array_mut().unwrap().push(f);
        }
        d["modules"][0]["elements"][0]["members"] = json!(members);
        let c = catalog(d);
        let result = c.entity_descriptor(&customer(&c));
        if accepted {
            assert_eq!(result.unwrap().graph.len(), 4096);
        } else {
            assert_eq!(result.unwrap_err().code, "WFT-LIMIT");
        }
    }
    println!("MODEL_BOUNDARY_REPORT depth=128/129 identities=4096/4097");
}

// @covers US-006-AC4
#[test]
fn owning_document_and_selection_limits_are_explicit() {
    fn input(id: &str, count: usize, bytes: Option<usize>) -> ModuleInput {
        let mut doc = document();
        doc["id"] = json!(id);
        let original = doc["modules"][0].clone();
        let mut selected = Vec::new();
        doc["modules"] = json!((0..count)
            .map(|i| {
                let mut module = original.clone();
                let name = format!("module-{i}");
                module["id"] = json!(name);
                selected.push(name);
                module
            })
            .collect::<Vec<_>>());
        let mut raw = doc.to_string();
        if let Some(size) = bytes {
            assert!(raw.len() <= size);
            raw.push_str(&" ".repeat(size - raw.len()));
        }
        ModuleInput {
            pin: ModelPin {
                document_id: id.into(),
                revision: "boundary".into(),
                umf_version: "0.7.0".into(),
                sha256: sha256(raw.as_bytes()),
            },
            document_json: raw,
            selected_module_ids: selected,
        }
    }
    assert_eq!(Catalog::prepare(vec![]).unwrap_err().code, "WFT-LIMIT");
    assert!(Catalog::prepare(
        (0..32)
            .map(|i| input(&format!("document-{i}"), 1, None))
            .collect()
    )
    .is_ok());
    assert_eq!(
        Catalog::prepare(
            (0..33)
                .map(|i| input(&format!("document-{i}"), 1, None))
                .collect()
        )
        .unwrap_err()
        .code,
        "WFT-LIMIT"
    );
    assert!(Catalog::prepare(vec![input("selected", 256, None)]).is_ok());
    assert_eq!(
        Catalog::prepare(vec![input("selected", 257, None)])
            .unwrap_err()
            .code,
        "WFT-LIMIT"
    );
    assert!(Catalog::prepare(vec![input("bytes", 1, Some(4 * 1024 * 1024))]).is_ok());
    assert_eq!(
        Catalog::prepare(vec![input("bytes", 1, Some(4 * 1024 * 1024 + 1))])
            .unwrap_err()
            .code,
        "WFT-LIMIT"
    );
}

// @covers US-006-AC2 @covers US-006-AC3
#[test]
fn catalog_admission_refusal_branches_are_explicit() {
    fn input(doc: Value) -> ModuleInput {
        let raw = doc.to_string();
        ModuleInput {
            pin: ModelPin {
                document_id: "sales-fixture".into(),
                revision: "branch".into(),
                umf_version: "0.7.0".into(),
                sha256: sha256(raw.as_bytes()),
            },
            document_json: raw,
            selected_module_ids: vec!["sales".into()],
        }
    }
    fn refused(inputs: Vec<ModuleInput>, code: &str, message: &str) {
        let error = Catalog::prepare(inputs).unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(error.message, message);
    }
    assert!(Catalog::prepare(vec![input(document())]).is_ok());
    let mut i = input(document());
    i.pin.sha256 = "0".repeat(64);
    refused(vec![i], "WFT-PIN", "Owning document digest mismatch");
    let mut i = input(document());
    i.document_json = "{".into();
    i.pin.sha256 = sha256(i.document_json.as_bytes());
    refused(vec![i], "WFT-INPUT", "Owning document JSON refused");
    let mut i = input(document());
    i.pin.document_id = "different".into();
    refused(vec![i], "WFT-PIN", "Owning document identity mismatch");
    let mut d = document();
    d["umf"] = json!("99.0.0");
    refused(
        vec![input(d)],
        "WFT-MODEL-VERSION",
        "Unsupported UMF profile",
    );
    let mut i = input(document());
    i.pin.umf_version = "99.0.0".into();
    refused(vec![i], "WFT-MODEL-VERSION", "Unsupported UMF profile");
    let mut i = input(document());
    i.pin.revision.clear();
    refused(
        vec![i],
        "WFT-MODEL",
        "Empty revision or repeated owning document",
    );
    refused(
        vec![input(document()), input(document())],
        "WFT-MODEL",
        "Empty revision or repeated owning document",
    );
    let mut d = document();
    d.as_object_mut().unwrap().remove("modules");
    refused(
        vec![input(d)],
        "WFT-MODEL",
        "Document violates the pinned UMF envelope",
    );
    let mut d = document();
    let mut m = d["modules"][0].clone();
    m["namespace"] = json!("other");
    d["modules"].as_array_mut().unwrap().push(m);
    refused(vec![input(d)], "WFT-MODEL", "Duplicate module identity");
    let mut d = document();
    let mut e = d["modules"][0]["elements"][0].clone();
    e["name"] = json!("Other");
    d["modules"][0]["elements"].as_array_mut().unwrap().push(e);
    refused(vec![input(d)], "WFT-MODEL", "Duplicate element identity");
    let mut i = input(document());
    i.selected_module_ids.clear();
    refused(vec![i], "WFT-MODEL", "No module selected");
    let mut i = input(document());
    i.selected_module_ids = vec!["sales".into(), "sales".into()];
    refused(vec![i], "WFT-MODEL", "Missing or repeated selected module");
    let mut i = input(document());
    i.selected_module_ids = vec!["missing".into()];
    refused(vec![i], "WFT-MODEL", "Missing or repeated selected module");
}

// @covers US-007-AC1 @covers US-007-AC6 @covers US-006-AC3
#[test]
fn selected_member_and_compound_meaning_guards_have_exact_diagnostics() {
    for (change, envelope_refusal, message) in [
        (
            json!({"members":null}),
            false,
            "Record needs explicit ordered members",
        ),
        (
            json!({"members":[{"module":"sales","element":"orders"}]}),
            false,
            "Record member must reference a Field",
        ),
        (
            json!({"members":[{"module":"sales","element":"customer-id"},{"module":"sales","element":"customer-id"}]}),
            true,
            "Repeated member identity",
        ),
    ] {
        let mut d = document();
        for (key, value) in change.as_object().unwrap() {
            if value.is_null() {
                d["modules"][0]["elements"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove(key);
            } else {
                d["modules"][0]["elements"][0][key] = value.clone();
            }
        }
        if envelope_refusal {
            let error = try_catalog(d).unwrap_err();
            assert_eq!(error.code, "WFT-MODEL");
            assert_eq!(error.message, "Document violates the pinned UMF envelope");
            continue;
        }
        let c = catalog(d);
        let error = c.entity_descriptor(&customer(&c)).unwrap_err();
        assert_eq!(error.code, "WFT-TYPE");
        assert_eq!(error.message, message);
    }
    let mut d = document();
    d["modules"][0]["elements"][3]
        .as_object_mut()
        .unwrap()
        .remove("name");
    let c = catalog(d);
    assert_eq!(
        c.entity_descriptor(&customer(&c)).unwrap_err().message,
        "Selected Field needs an authored name"
    );
    for (shape, envelope_refusal, message) in [
        (
            json!({"cardinality":"array","scalarType":"string","itemType":{"module":"sales","element":"customer-name"}}),
            true,
            "Container cannot imply scalar or structured meaning",
        ),
        (
            json!({"cardinality":"array","itemType":{"module":"sales","element":"customer"}}),
            false,
            "Container itemType must reference a Field",
        ),
        (
            json!({"cardinality":"array","itemType":{"module":"sales","element":"customer-name"},"facets":{"future":true}}),
            true,
            "Selected container or structured facets are unsupported",
        ),
        (
            json!({"cardinality":"one","references":[{"role":"record-type","module":"sales","element":"customer-name"}]}),
            false,
            "Structured Field needs a sole record-type reference",
        ),
        (
            json!({"cardinality":"one","scalarType":"string","references":[{"role":"record-type","module":"sales","element":"customer"}]}),
            false,
            "Structured Field needs a sole record-type reference",
        ),
    ] {
        let mut d = document();
        d["modules"][0]["elements"][0]["members"]
            .as_array_mut()
            .unwrap()
            .push(json!({"module":"sales","element":"selected-compound"}));
        let mut field = json!({"id":"selected-compound","name":"selected_compound","kind":"field","nullability":"required","extensions":{}});
        for (key, value) in shape.as_object().unwrap() {
            field[key] = value.clone();
        }
        d["modules"][0]["elements"]
            .as_array_mut()
            .unwrap()
            .push(field);
        if envelope_refusal {
            let error = try_catalog(d).unwrap_err();
            assert_eq!(error.code, "WFT-MODEL");
            assert_eq!(error.message, "Document violates the pinned UMF envelope");
            continue;
        }
        let c = catalog(d);
        let error = c.entity_descriptor(&customer(&c)).unwrap_err();
        assert_eq!(error.code, "WFT-TYPE");
        assert_eq!(error.message, message);
    }
    let c = catalog(document());
    let r = customer(&c);
    assert_eq!(
        c.member_descriptor(&r, &name("unknown")).unwrap_err().code,
        "WFT-NAME-MISSING"
    );
    let mut unowned = r.identity.clone();
    unowned.element = "order-total".into();
    assert_eq!(
        c.member_descriptor_by_identity(&r, &unowned)
            .unwrap_err()
            .message,
        "Original Record does not own selected member identity"
    );
    let (member, graph) = c.member_descriptor(&r, &name("id")).unwrap();
    let (by_id, id_graph) = c
        .member_descriptor_by_identity(&r, &member.identity)
        .unwrap();
    assert_eq!(member.identity, by_id.identity);
    assert_eq!(
        serde_json::to_value(graph).unwrap(),
        serde_json::to_value(id_graph).unwrap()
    );
}

#[test]
fn authored_key_and_relationship_endpoint_guards_are_explicit() {
    for (change,message) in [
        (json!({"source":[{"module":"sales","element":"customer"},{"module":"sales","element":"orders"}]}),"Relationship requires a supported monomorphic directed traversal without an association Record"),
        (json!({"target":[{"module":"sales","element":"order-total","key":"order-pk"}]}),"Relationship target must be a Record"),
        (json!({"sourceMultiplicity":{"min":2,"max":1}}),"Relationship multiplicity is inconsistent"),
    ] {
        let mut d=relationship_document();for (key,value) in change.as_object().unwrap(){d["modules"][0]["relationships"][0][key]=value.clone();}
        let c=catalog(d);let error=c.relationship_read(&customer(&c),&name("orders")).unwrap_err();
        assert_eq!(error.code,"WFT-TYPE");assert_eq!(error.message,message);
    }
    let mut d = relationship_document();
    d["modules"][0]["elements"][0]["keys"][0]
        .as_object_mut()
        .unwrap()
        .remove("primary");
    let c = catalog(d);
    assert_eq!(
        c.relationship_read(&customer(&c), &name("orders"))
            .unwrap()
            .source_key
            .id,
        "customer-pk"
    );
    let mut d = relationship_document();
    let mut alternative = d["modules"][0]["elements"][0]["keys"][0].clone();
    alternative["id"] = json!("alternative");
    alternative["name"] = json!("alternative");
    alternative["primary"] = json!(false);
    d["modules"][0]["elements"][0]["keys"][0]["primary"] = json!(false);
    d["modules"][0]["elements"][0]["keys"]
        .as_array_mut()
        .unwrap()
        .push(alternative);
    let c = catalog(d);
    assert_eq!(
        c.relationship_read(&customer(&c), &name("orders"))
            .unwrap_err()
            .message,
        "Relationship source endpoint key is ambiguous"
    );
    for references in [
        json!([{"module":"sales","element":"customer-id"},{"module":"sales","element":"customer-id"}]),
        json!([{"module":"sales","element":"order-total"}]),
    ] {
        let mut d = relationship_document();
        d["modules"][0]["elements"][0]["keys"][0]["fields"] = references.clone();
        if references.as_array().unwrap().len() == 2 {
            let error = try_catalog(d).unwrap_err();
            assert_eq!(error.code, "WFT-MODEL");
            assert_eq!(error.message, "Document violates the pinned UMF envelope");
        } else {
            let c = catalog(d);
            assert_eq!(
                c.authored_key(&customer(&c), "customer-pk")
                    .unwrap_err()
                    .message,
                "Key must contain distinct declared member Fields"
            );
        }
    }
}
