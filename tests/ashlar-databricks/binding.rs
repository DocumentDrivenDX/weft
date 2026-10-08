// @covers US-004-AC2 @covers US-004-AC4 (binding component, not native acceptance)
use serde_json::{json, Value};
use weft_core::{
    ir::ModelPin,
    json::sha256,
    model::{Catalog, ModuleInput},
};
use weft_databricks::binding::{admit, Binding};

fn catalog() -> Catalog {
    let raw = include_str!("../../docs/helix/03-test/fixtures/sales.umf.json");
    Catalog::prepare(vec![ModuleInput {
        document_json: raw.into(),
        pin: ModelPin {
            document_id: "sales-fixture".into(),
            revision: "1".into(),
            umf_version: "0.7.0".into(),
            sha256: sha256(raw.as_bytes()),
        },
        selected_module_ids: vec!["sales".into()],
    }])
    .unwrap()
}
fn identity(element: &str) -> Value {
    json!({"documentId":"sales-fixture","revision":"1","module":"sales","element":element})
}
fn fixture() -> Value {
    json!({
        "profile":"ashlar-databricks-candidate/0.1.0",
        "layoutRevision":"ashlar-delta/0.3",
        "layoutSha256":"ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e",
        "modelPins":catalog().pins(),
        "publication":{"id":"fixture-publication-1","manifestUuid":"manifest-fixture-uuid",
            "tables":[{"name":["fixture_catalog","fixture_schema","node_type_a"],"uuid":"object-fixture-uuid","version":0}]},
        "records":[{"logical":identity("customer"),"table":0,"kind":"nodeProjection","sourceSystem":"source-a","typeId":"1","schemaRevision":"accepted-fixture-1",
            "properties":[{"logical":identity("customer-id"),"home":{"kind":"props","propertyId":"23"}},
                {"logical":identity("customer-name"),"home":{"kind":"column","value":"group_value","present":"group_present","nativeType":"STRING"}}]}]
    })
}
fn check(v: Value) -> Result<Binding, weft_core::error::Diagnostic> {
    admit(&catalog(), &v)
}

#[test]
fn explicit_property_homes_and_snapshot_zero_are_admitted() {
    let binding = check(fixture()).unwrap();
    assert_eq!(binding.records.len(), 1);
    assert_eq!(binding.publication.tables[0].version, 0);
}
#[test]
fn unsafe_and_ambiguous_mapping_content_refuses() {
    let mut cases = Vec::new();
    for (path, value) in [
        ("/layoutRevision", json!("ashlar-delta/unknown")),
        ("/layoutSha256", json!("0".repeat(64))),
        ("/publication/id", json!("")),
        ("/publication/manifestUuid", json!("")),
        ("/publication/tables/0/version", json!(-1)),
        ("/publication/tables/0/version", json!("0")),
        ("/publication/tables/0/version", json!(true)),
        ("/publication/tables/0/uuid", json!("")),
        ("/publication/tables/0/name", json!(["catalog", "schema"])),
        ("/records/0/table", json!(1)),
        ("/records/0/typeId", json!("01")),
        ("/records/0/typeId", json!("18446744073709551615")),
        ("/records/0/schemaRevision", json!("")),
        (
            "/records/0/properties/0/home/propertyId",
            json!("23; DROP TABLE x"),
        ),
        ("/records/0/properties/1/home/nativeType", json!("DOUBLE")),
        ("/records/0/properties/1/home/nativeType", json!("BIGINT")),
        (
            "/records/0/properties/1/home/value",
            json!("invented_value"),
        ),
        (
            "/records/0/properties/1/home/present",
            json!("different_present"),
        ),
        ("/records/0/properties/1/logical", identity("order-total")),
    ] {
        let mut v = fixture();
        *v.pointer_mut(path).unwrap() = value;
        cases.push(v);
    }
    let mut v = fixture();
    v["records"][0]["sql"] = json!("SELECT 1");
    cases.push(v);
    let mut v = fixture();
    v["records"][0]["logical"]["nativeMeaning"] = json!("invented");
    cases.push(v);
    let mut v = fixture();
    v["records"][0]["properties"][0]["logical"]["plugin"] = json!("external");
    cases.push(v);
    let mut v = fixture();
    v["records"][0]["properties"][0]["home"]["decode"] = json!("plugin.js");
    cases.push(v);
    let mut v = fixture();
    v["modelPins"][0]["revision"] = json!("stale");
    cases.push(v);
    let mut v = fixture();
    let p = v["records"][0]["properties"][0].clone();
    v["records"][0]["properties"]
        .as_array_mut()
        .unwrap()
        .push(p);
    cases.push(v);
    let mut v = fixture();
    let r = v["records"][0].clone();
    v["records"].as_array_mut().unwrap().push(r);
    cases.push(v);
    let mut v = fixture();
    let t = v["publication"]["tables"][0].clone();
    v["publication"]["tables"].as_array_mut().unwrap().push(t);
    cases.push(v);
    for (i, v) in cases.into_iter().enumerate() {
        assert!(check(v).is_err(), "refusal case {i}");
    }
}
#[test]
fn unmapped_unknown_model_content_is_retained() {
    let c = catalog();
    assert!(c.inputs[0].document_json.contains("future.vendor"));
    admit(&c, &fixture()).unwrap();
    assert!(c.inputs[0].document_json.contains("opaque"));
}

#[test]
fn bigint_admits_unsigned_subdomains_and_refuses_uint64() {
    for bits in [1, 8, 32, 63, 64] {
        let original = catalog();
        let mut input = original.inputs[0].clone();
        let mut document: Value = serde_json::from_str(&input.document_json).unwrap();
        document["modules"][0]["elements"][2]["facets"]["integerWidth"]["bits"] = json!(bits);
        input.document_json = document.to_string();
        input.pin.sha256 = sha256(input.document_json.as_bytes());
        let c = Catalog::prepare(vec![input]).unwrap();
        let mut binding = fixture();
        binding["modelPins"] = json!(c.pins());
        binding["records"][0]["properties"][0]["home"] = json!({"kind":"column","value":"rank_value","present":"rank_present","nativeType":"BIGINT"});
        assert_eq!(
            admit(&c, &binding).is_ok(),
            bits < 64,
            "unsigned width {bits}"
        );
    }
}
#[test]
fn physical_identifiers_are_quoted_and_values_never_become_code() {
    let mut v = fixture();
    v["publication"]["tables"][0]["name"] =
        json!(["odd.catalog", "schema`name", "table; DROP TABLE x"]);
    let b = check(v).unwrap();
    assert_eq!(
        b.publication.tables[0].sql(),
        "`odd.catalog`.`schema``name`.`table; DROP TABLE x` VERSION AS OF 0"
    );
}
