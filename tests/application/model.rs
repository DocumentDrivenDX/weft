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
fn catalog(doc: Value) -> Catalog {
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
    .unwrap()
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
