use weft_core::{ir::ModelPin, json::sha256, model::{Catalog, ModuleInput}};
#[test]
fn sales_fixture_current_envelope_is_valid() {
    let old = include_str!("../../../docs/helix/03-test/fixtures/sales.umf.json");
    let mut document: serde_json::Value = serde_json::from_str(old).unwrap();
    document["umf"] = serde_json::json!("0.10.0");
    let text = document.to_string();
    let input = ModuleInput {
        document_json: text.clone(),
        pin: ModelPin {
            document_id: document["id"].as_str().unwrap().into(),
            revision: "sales-fixture-010".into(),
            umf_version: "0.10.0".into(),
            sha256: sha256(text.as_bytes()),
        },
        selected_module_ids: document["modules"].as_array().unwrap().iter().map(|m|m["id"].as_str().unwrap().into()).collect(),
    };
    Catalog::prepare(vec![input]).unwrap();
}
