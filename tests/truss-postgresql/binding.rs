use serde_json::{json, Value};
use weft_postgresql::binding::{Admission, PropertyHome};
const PROFILE: &str = "candidate-test-binding/0.1.0";
fn fixture(row: bool) -> Value {
    serde_json::from_str(if row {
        include_str!("fixtures/binding-row.json")
    } else {
        include_str!("fixtures/binding-props.json")
    })
    .unwrap()
}
#[test]
fn both_home_locations_admit_original_bytes_without_claiming_qualification() {
    for row in [false, true] {
        let raw = fixture(row).to_string();
        let admitted = Admission::parse(&raw, PROFILE).unwrap();
        assert_eq!(admitted.original_json, raw);
        assert!(admitted.artifacts.len() >= 10);
        assert_eq!(
            admitted.property_home(0).unwrap(),
            if row {
                PropertyHome::Row {
                    access: "scalar-root".into(),
                }
            } else {
                PropertyHome::Props { member: "0".into() }
            }
        );
        assert_eq!(admitted.value["qualification"], json!([]));
    }
}
#[test]
fn shape_hash_profile_and_native_mapping_errors_refuse() {
    let mut cases = Vec::new();
    let mut v = fixture(false);
    v["basis"]["modelBundle"]["sha256"] = json!("a".repeat(64));
    cases.push(v);
    let mut v = fixture(false);
    v["basis"]["modelBundle"]["bytesBase64"] = json!("!!!!");
    cases.push(v);
    let mut v = fixture(false);
    let entity = v["entities"][0].clone();
    v["entities"].as_array_mut().unwrap().push(entity);
    cases.push(v);
    let mut v = fixture(false);
    v["entities"][0]["typeId"] = json!("2147483648");
    cases.push(v);
    let mut v = fixture(false);
    v["properties"][0]["propertyId"] = json!(3);
    cases.push(v);
    let mut v = fixture(false);
    v["sql"] = json!("SELECT arbitrary");
    cases.push(v);
    for v in cases {
        assert_eq!(
            Admission::parse(&v.to_string(), PROFILE).unwrap_err().code,
            "WFT-BINDING"
        );
    }
    assert!(Admission::parse(&fixture(false).to_string(), "wrong-profile").is_err());
    assert!(Admission::parse("{\"interfaceVersion\":1,\"interfaceVersion\":2}", PROFILE).is_err());
}
#[test]
fn home_correspondence_refuses_silent_location_fallback() {
    let mut v = fixture(false);
    v["properties"][0]["home"] = json!("row");
    let admitted = Admission::parse(&v.to_string(), PROFILE).unwrap();
    assert!(admitted.property_home(0).is_err());
    let mut v = fixture(false);
    v["properties"][0]["propertyId"] = json!("1");
    assert!(Admission::parse(&v.to_string(), PROFILE)
        .unwrap()
        .property_home(0)
        .is_err());
}

#[test]
fn rehashed_physical_homes_cannot_redirect_fixed_candidate_templates() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    for (row, fields) in [
        (
            false,
            vec![
                "recordKind",
                "relationName",
                "propsColumnName",
                "discriminatorColumnName",
                "relationPhysicalIdentity",
                "propsColumnPhysicalIdentity",
                "discriminatorColumnPhysicalIdentity",
                "accessor",
            ],
        ),
        (
            true,
            vec![
                "recordKind",
                "stateRelationPhysicalIdentity",
                "nodeRelationPhysicalIdentity",
                "scalarRelationPhysicalIdentity",
            ],
        ),
    ] {
        for field in fields {
            let mut binding = fixture(row);
            let artifact = &mut binding["properties"][0]["homeDefinition"];
            let original = STANDARD
                .decode(artifact["bytesBase64"].as_str().unwrap())
                .unwrap();
            let mut home: Value = serde_json::from_slice(&original).unwrap();
            home[field] = json!(if field == "recordKind" {
                "edge"
            } else {
                "different-physical-target"
            });
            let bytes = home.to_string().into_bytes();
            artifact["bytesBase64"] = json!(STANDARD.encode(&bytes));
            artifact["sha256"] = json!(weft_core::json::sha256(&bytes));
            let admission = Admission::parse(&binding.to_string(), PROFILE).unwrap();
            assert!(admission.property_home(0).is_err(), "{field}");
        }
    }
}
