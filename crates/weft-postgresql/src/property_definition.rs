//! Compose original value admissions. Physical and operation admission follow separately.
use crate::{
    binding::Admission, leaf_codec_definition, presence_definition, value_definition::Graph,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    application_model::{Descriptor, Shape},
    error::{Diagnostic, Result},
    ir::Identity,
    json::checked_json,
    model::Catalog,
};
pub struct Selection<'a> {
    pub value_profile: &'a Value,
    pub presence_profile: &'a Value,
    pub leaf_codecs: &'a BTreeMap<String, leaf_codec_definition::Definition>,
    pub record_presence: &'a BTreeMap<String, presence_definition::Definition>,
}
#[derive(Debug)]
pub struct ValueAdmission {
    pub graph: Graph,
    pub presence: presence_definition::Definition,
    pub descriptors: Vec<Descriptor>,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
/// Compute closure from the frontend descriptors, never from binding-selected
/// nodes. Thus an omitted authored dependency cannot make a smaller graph valid.
fn closure(descriptors: &[Descriptor], root: &Identity) -> Result<Vec<Descriptor>> {
    let id = |identity: &Identity| {
        serde_json::to_string(identity).map_err(|_| fail("Identity encoding refused"))
    };
    let mut indexed = BTreeMap::new();
    for descriptor in descriptors {
        if indexed
            .insert(id(&descriptor.identity)?, descriptor)
            .is_some()
        {
            return Err(fail("Duplicate frontend descriptor identity"));
        }
    }
    let mut pending = vec![id(root)?];
    let mut seen = BTreeSet::new();
    while let Some(identity) = pending.pop() {
        if !seen.insert(identity.clone()) {
            continue;
        }
        let descriptor = indexed
            .get(&identity)
            .ok_or_else(|| fail("Original frontend descriptor closure is incomplete"))?;
        match &descriptor.shape {
            Shape::Sequence { item } | Shape::Map { item } => pending.push(id(item)?),
            Shape::Structured { record } => pending.push(id(record)?),
            Shape::Record { members } => {
                for member in members {
                    pending.push(id(&member.identity)?);
                }
            }
            Shape::Scalar { .. } => {}
        }
    }
    descriptors
        .iter()
        .filter_map(|descriptor| match id(&descriptor.identity) {
            Ok(key) if seen.contains(&key) => Some(Ok(descriptor.clone())),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}
/// This gate establishes original value correspondence. It does not grant SQL
/// operations, physical inventory interpretation or host execution authority.
pub fn admit_value(
    binding: &Admission,
    index: usize,
    catalog: &Catalog,
    descriptors: &[Descriptor],
    selected: Selection<'_>,
) -> Result<ValueAdmission> {
    let property = binding.value["properties"]
        .get(index)
        .ok_or_else(|| fail("Missing selected property"))?;
    if &property["valueProfile"] != selected.value_profile
        || &property["presenceProfile"] != selected.presence_profile
    {
        return Err(fail(
            "Selected property value/presence profiles differ from registration",
        ));
    }
    let root: Identity = serde_json::from_value(property["logical"].clone())
        .map_err(|_| fail("Invalid property identity"))?;
    let input = catalog
        .inputs
        .iter()
        .find(|input| {
            input.pin.document_id == root.document_id
                && input.pin.revision == root.revision
                && input.selected_module_ids.contains(&root.module)
        })
        .ok_or_else(|| fail("Property source is outside selected original model"))?;
    if binding.decoded_json(&format!("/properties/{index}/source"))?
        != checked_json(&input.document_json)
            .map_err(|_| fail("Original property source JSON refused"))?
    {
        return Err(fail(
            "Property source differs from original selected model document",
        ));
    }
    let descriptors = closure(descriptors, &root)?;
    let accepted = binding
        .artifacts
        .get(&format!("/properties/{index}/acceptedDefinition"))
        .ok_or_else(|| fail("Missing original accepted property"))?;
    let text = |path: &str| -> Result<&str> {
        let bytes = binding
            .artifacts
            .get(path)
            .ok_or_else(|| fail("Missing original property definition"))?;
        std::str::from_utf8(bytes).map_err(|_| fail("Original property definition is not UTF-8"))
    };
    let graph = Graph::parse(
        text(&format!("/properties/{index}/valueDefinition"))?,
        selected.value_profile,
        accepted,
    )?;
    graph.verify_model_sources(catalog, &root)?;
    graph.verify_descriptors(&descriptors, &root)?;
    graph.verify_leaf_codecs(selected.leaf_codecs)?;
    graph.verify_record_presence(selected.record_presence)?;
    let raw = text(&format!("/properties/{index}/presenceDefinition"))?;
    let presence =
        presence_definition::Definition::parse(raw, selected.presence_profile, accepted)?;
    if checked_json(raw).map_err(|_| fail("Original presence JSON refused"))?["acceptedDefinition"]
        != property["acceptedDefinition"]
    {
        return Err(fail(
            "Property presence accepted artifact differs from original binding",
        ));
    }
    Ok(ValueAdmission {
        graph,
        presence,
        descriptors,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use weft_core::{ir::Span, syntax::Name};
    #[test]
    fn closure_is_authored_and_finite_without_binding_influence() {
        let identity = |element: &str| Identity {
            document_id: "d".into(),
            revision: "1".into(),
            module: "m".into(),
            element: element.into(),
        };
        let root = identity("root");
        let record = identity("record");
        let descriptors = vec![
            Descriptor {
                identity: root.clone(),
                availability: Some("required".into()),
                shape: Shape::Structured {
                    record: record.clone(),
                },
            },
            Descriptor {
                identity: record,
                availability: None,
                shape: Shape::Record {
                    members: vec![weft_core::application_model::Member {
                        name: "next".into(),
                        identity: root.clone(),
                    }],
                },
            },
        ];
        assert_eq!(closure(&descriptors, &root).unwrap().len(), 2);
        assert!(closure(&descriptors[..1], &root).is_err());
    }
    #[test]
    fn real_authored_field_composes_graph_presence_and_leaf_correspondence() {
        use base64::{engine::general_purpose::STANDARD, Engine};
        use leaf_codec_definition::{
            Definition as Leaf, OriginalArtifact, Selection as LeafSelection,
        };
        use weft_core::json::sha256;
        let cases: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/compiler-cases.json"
        ))
        .unwrap();
        let inputs = serde_json::from_value(cases[0]["request"]["modules"].clone()).unwrap();
        let catalog = Catalog::prepare(inputs).unwrap();
        let name = |value: &str| Name {
            value: value.to_ascii_lowercase(),
            quoted: false,
            span: Span { start: 0, end: 0 },
        };
        let record = catalog.record(None, &name("Customer")).unwrap();
        let (member, descriptors) = catalog.member_descriptor(&record, &name("name")).unwrap();
        let mut binding: Value = serde_json::from_str(
            cases[0]["request"]["target"]["bindingJson"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        let index = binding["properties"]
            .as_array()
            .unwrap()
            .iter()
            .position(|p| p["logical"] == json!(member.identity))
            .unwrap();
        let property = &binding["properties"][index];
        let pin = property["valueProfile"].clone();
        let authored = property["acceptedDefinition"].clone();
        let bytes = STANDARD
            .decode(authored["bytesBase64"].as_str().unwrap())
            .unwrap();
        let artifact = |identity: &str, bytes: &[u8]| json!({"identity":identity,"bytesBase64":STANDARD.encode(bytes),"sha256":sha256(bytes)});
        let empty = artifact("fixture", b"{}");
        let leaf = json!({"interfaceVersion":"truss-jsonb-leaf-codec/0.1.0","profile":pin,"authoredDefinition":authored,"sourceInterpretationProfile":pin,"sourceInterpretationDefinition":empty,"nativeDomainProfile":pin,"nativeDomainDefinition":empty,"rule":{"family":"string","storageRepresentation":"json-string","encoding":"preserve-unicode-scalars","decodedCarrierKind":"string"},"coercion":"none","readDefault":"none","invalidStoredValue":"complete-result-refusal"});
        let originals = BTreeMap::from([
            (
                "authoredDefinition".into(),
                OriginalArtifact {
                    identity: authored["identity"].as_str().unwrap().into(),
                    bytes: bytes.clone(),
                },
            ),
            (
                "sourceInterpretationDefinition".into(),
                OriginalArtifact {
                    identity: "fixture".into(),
                    bytes: b"{}".to_vec(),
                },
            ),
            (
                "nativeDomainDefinition".into(),
                OriginalArtifact {
                    identity: "fixture".into(),
                    bytes: b"{}".to_vec(),
                },
            ),
        ]);
        let leaf = Leaf::parse(
            &leaf.to_string(),
            LeafSelection {
                profile: &pin,
                source_profile: &pin,
                native_profile: &pin,
                original_artifacts: &originals,
            },
        )
        .unwrap();
        let graph = json!({"interfaceVersion":"truss-value-definition/0.1.0","profile":pin,"rootNodeId":"root","acceptedDefinition":authored,"nodes":[{"nodeId":"root","authoredIdentity":member.identity,"authoredDefinition":authored,"codecProfile":pin,"codecDefinition":artifact("selected-leaf",leaf.original_json.as_bytes()),"shape":{"kind":"scalar","family":"string","storageRepresentation":"json-string"}}]});
        let schema: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/upstream/presence-definition.schema.json"
        ))
        .unwrap();
        let mut presence = json!({});
        for (key, rule) in schema["properties"].as_object().unwrap() {
            if let Some(constant) = rule.get("const") {
                presence[key] = constant.clone();
            }
        }
        presence["profile"] = pin.clone();
        presence["acceptedDefinition"] = authored;
        binding["properties"][index]["valueDefinition"] =
            artifact("original-value-graph", graph.to_string().as_bytes());
        binding["properties"][index]["presenceDefinition"] =
            artifact("original-presence", presence.to_string().as_bytes());
        let admitted = Admission::parse(
            &binding.to_string(),
            binding["bindingProfileId"].as_str().unwrap(),
        )
        .unwrap();
        let leaves = BTreeMap::from([("root".into(), leaf)]);
        let records = BTreeMap::new();
        let select = || Selection {
            value_profile: &pin,
            presence_profile: &pin,
            leaf_codecs: &leaves,
            record_presence: &records,
        };
        let value = admit_value(&admitted, index, &catalog, &descriptors, select()).unwrap();
        assert_eq!(value.descriptors.len(), 1);
        assert_eq!(value.presence.accepted_definition, bytes);
        assert!(admit_value(&admitted, index, &catalog, &[], select()).is_err());
        let mut wrong_binding = binding.clone();
        wrong_binding["properties"][index]["source"] = artifact("foreign-source", b"{}");
        let wrong_binding = Admission::parse(
            &wrong_binding.to_string(),
            binding["bindingProfileId"].as_str().unwrap(),
        )
        .unwrap();
        assert!(admit_value(&wrong_binding, index, &catalog, &descriptors, select()).is_err());
        let mut wrong = descriptors.clone();
        wrong[0].shape = Shape::Sequence {
            item: member.identity,
        };
        assert!(admit_value(&admitted, index, &catalog, &wrong, select()).is_err());
    }
}
