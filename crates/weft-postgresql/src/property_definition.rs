//! Compose original value and home admissions. Operation/native qualification follows.
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
    pub definition_artifact: Value,
    pub graph: Graph,
    pub presence: presence_definition::Definition,
    pub descriptors: Vec<Descriptor>,
}
pub struct PhysicalSelection<'a> {
    pub profile: &'a Value,
    pub inventory: &'a leaf_codec_definition::OriginalArtifact,
    pub relations: &'a BTreeMap<String, String>,
    pub columns: &'a BTreeMap<String, crate::row_join_definition::Column>,
    pub row_join: Option<&'a crate::row_join_definition::Definition>,
    pub obligations: &'a BTreeSet<String>,
    pub edge_association: Option<(&'a Value, &'a leaf_codec_definition::OriginalArtifact)>,
}
#[derive(Debug)]
pub enum HomeAdmission {
    Props {
        member: String,
        record_kind: crate::row_join_definition::RecordKind,
    },
    Row {
        access: String,
        record_kind: crate::row_join_definition::RecordKind,
    },
}
#[derive(Debug)]
pub struct PropertyAdmission {
    pub owner: Identity,
    pub identity: Identity,
    pub value: ValueAdmission,
    pub home: HomeAdmission,
}
/// Registered metadata must already correspond to original inventory bytes.
/// This function checks selectors against that metadata; it does not create it.
pub fn admit_home(
    binding: &Admission,
    index: usize,
    selected: PhysicalSelection<'_>,
) -> Result<HomeAdmission> {
    use crate::row_join_definition::RecordKind;
    let home = binding.original_home_definition(index)?;
    let property = &binding.value["properties"][index];
    let inventory = &binding.value["basis"]["layoutInventory"];
    if &property["homeProfile"] != selected.profile
        || inventory["identity"] != selected.inventory.identity
        || binding
            .artifacts
            .get("/basis/layoutInventory")
            .is_none_or(|bytes| bytes != &selected.inventory.bytes)
    {
        return Err(fail(
            "Home profile or original inventory differs from registration",
        ));
    }
    let edge = home["recordKind"] == "edge";
    let record_kind = if edge {
        RecordKind::Edge
    } else {
        RecordKind::Object
    };
    if edge {
        use base64::{engine::general_purpose::STANDARD, Engine};
        let (profile, original) = selected
            .edge_association
            .ok_or_else(|| fail("Edge home lacks registered association meaning"))?;
        let artifact = &home["edgeAssociationDefinition"];
        let encoded = artifact["bytesBase64"].as_str().unwrap();
        let bytes = STANDARD
            .decode(encoded)
            .map_err(|_| fail("Edge association artifact base64 refused"))?;
        if &home["edgeAssociationProfile"] != profile
            || artifact["identity"] != original.identity
            || bytes != original.bytes
            || STANDARD.encode(&bytes) != encoded
            || artifact["sha256"] != weft_core::json::sha256(&bytes)
        {
            return Err(fail(
                "Edge home association differs from original registered selection",
            ));
        }
    } else if selected.edge_association.is_some() {
        return Err(fail(
            "Object home cannot consume edge association selection",
        ));
    }
    if property["home"] == "row" {
        let join = selected
            .row_join
            .ok_or_else(|| fail("Native row home has no admitted original join"))?;
        join.verify_home(
            binding,
            index,
            selected.obligations,
            selected.edge_association.map(|(profile, _)| profile),
        )?;
        return Ok(HomeAdmission::Row {
            access: home["access"].as_str().unwrap().into(),
            record_kind,
        });
    }
    if selected.row_join.is_some() {
        return Err(fail("Props home cannot consume native row join selection"));
    }
    let relation = home["relationPhysicalIdentity"].as_str().unwrap();
    let relation_name = if edge { "edge" } else { "object" };
    let discriminator = if edge { "rel_type_id" } else { "type_id" };
    if home["relationName"] != relation_name
        || home["discriminatorColumnName"] != discriminator
        || selected
            .relations
            .get(relation)
            .is_none_or(|name| name != relation_name)
        || home["memberName"] != property["propertyId"]
        || home["valueProfile"] != property["valueProfile"]
        || home["presenceProfile"] != property["presenceProfile"]
    {
        return Err(fail(
            "Props owner/member/value/presence differs from registered home",
        ));
    }
    let props_id = home["propsColumnPhysicalIdentity"].as_str().unwrap();
    let discriminator_id = home["discriminatorColumnPhysicalIdentity"]
        .as_str()
        .unwrap();
    if props_id == discriminator_id {
        return Err(fail(
            "Props and discriminator cannot alias one physical column",
        ));
    }
    for (id, name) in [(props_id, "props"), (discriminator_id, discriminator)] {
        if selected
            .columns
            .get(id)
            .is_none_or(|column| column.relation_identity != relation || column.name != name)
        {
            return Err(fail(
                "Props column differs from original physical association",
            ));
        }
    }
    Ok(HomeAdmission::Props {
        member: home["memberName"].as_str().unwrap().into(),
        record_kind,
    })
}
pub fn admit_property(
    binding: &Admission,
    index: usize,
    catalog: &Catalog,
    descriptors: &[Descriptor],
    value: Selection<'_>,
    physical: PhysicalSelection<'_>,
) -> Result<PropertyAdmission> {
    let property = binding.value["properties"]
        .get(index)
        .ok_or_else(|| fail("Missing selected property"))?;
    let identity: Identity = serde_json::from_value(property["logical"].clone())
        .map_err(|_| fail("Invalid property identity"))?;
    let (entity_index, entity) = binding.value["entities"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .find(|(_, entity)| entity["typeId"] == property["ownerTypeId"])
        .ok_or_else(|| fail("Property owner has no entity mapping"))?;
    let owner: Identity = serde_json::from_value(entity["logical"].clone())
        .map_err(|_| fail("Invalid property owner identity"))?;
    if owner.document_id != identity.document_id || owner.revision != identity.revision {
        return Err(fail(
            "Property owner and Field are not in the same original source cut",
        ));
    }
    let input = catalog
        .inputs
        .iter()
        .find(|input| {
            input.pin.document_id == owner.document_id
                && input.pin.revision == owner.revision
                && input.selected_module_ids.contains(&owner.module)
        })
        .ok_or_else(|| fail("Property owner is outside original selected model"))?;
    let document =
        checked_json(&input.document_json).map_err(|_| fail("Original owner document refused"))?;
    let record = document["modules"]
        .as_array()
        .and_then(|modules| modules.iter().find(|module| module["id"] == owner.module))
        .and_then(|module| module["elements"].as_array())
        .and_then(|elements| {
            elements
                .iter()
                .find(|element| element["id"] == owner.element)
        })
        .ok_or_else(|| fail("Original property owner record is missing"))?;
    if record["kind"] != "record"
        || record["members"].as_array().is_none_or(|members| {
            !members.iter().any(|member| {
                member["module"] == identity.module && member["element"] == identity.element
            })
        })
    {
        return Err(fail("Original Record does not declare the mapped Field"));
    }
    if binding.decoded_json(&format!("/entities/{entity_index}/source"))? != document
        || binding.decoded_json(&format!("/entities/{entity_index}/acceptedDefinition"))? != *record
    {
        return Err(fail(
            "Mapped owner source/definition differs from original Record",
        ));
    }
    let value = admit_value(binding, index, catalog, descriptors, value)?;
    let home = admit_home(binding, index, physical)?;
    if matches!(&home,HomeAdmission::Row{access,..} if access=="scalar-root")
        && value.graph.value["nodes"][value.graph.root]["shape"]["kind"] != "scalar"
    {
        return Err(Diagnostic::new(
            "WFT-CAPABILITY",
            "lower",
            "Compound value cannot use scalar-root native storage",
        ));
    }
    Ok(PropertyAdmission {
        owner,
        identity,
        value,
        home,
    })
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
        definition_artifact: property["valueDefinition"].clone(),
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
        presence["acceptedDefinition"] = authored.clone();
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
        let inventory = leaf_codec_definition::OriginalArtifact {
            identity: binding["basis"]["layoutInventory"]["identity"]
                .as_str()
                .unwrap()
                .into(),
            bytes: b"{}".to_vec(),
        };
        let relations = BTreeMap::from([("object-table".into(), "object".into())]);
        let columns = BTreeMap::from([
            (
                "object-props".into(),
                crate::row_join_definition::Column {
                    relation_identity: "object-table".into(),
                    name: "props".into(),
                },
            ),
            (
                "object-type".into(),
                crate::row_join_definition::Column {
                    relation_identity: "object-table".into(),
                    name: "type_id".into(),
                },
            ),
        ]);
        let obligations = BTreeSet::new();
        let physical = || PhysicalSelection {
            profile: &pin,
            inventory: &inventory,
            relations: &relations,
            columns: &columns,
            row_join: None,
            obligations: &obligations,
            edge_association: None,
        };
        let property = admit_property(
            &admitted,
            index,
            &catalog,
            &descriptors,
            select(),
            physical(),
        )
        .unwrap();
        assert!(matches!(property.home, HomeAdmission::Props { .. }));
        assert_eq!(property.owner, record.identity);
        assert_eq!(property.identity, member.identity);
        let mut foreign = binding.clone();
        let other = foreign["entities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entity| entity["logical"] != json!(record.identity))
            .unwrap()["typeId"]
            .clone();
        foreign["properties"][index]["ownerTypeId"] = other;
        let foreign = Admission::parse(
            &foreign.to_string(),
            binding["bindingProfileId"].as_str().unwrap(),
        )
        .unwrap();
        assert!(admit_property(
            &foreign,
            index,
            &catalog,
            &descriptors,
            select(),
            physical()
        )
        .is_err());
        let mut foreign = binding.clone();
        let owner_index = foreign["entities"]
            .as_array()
            .unwrap()
            .iter()
            .position(|entity| entity["logical"] == json!(record.identity))
            .unwrap();
        foreign["entities"][owner_index]["acceptedDefinition"] = artifact("replaced-record", b"{}");
        let foreign = Admission::parse(
            &foreign.to_string(),
            binding["bindingProfileId"].as_str().unwrap(),
        )
        .unwrap();
        assert!(admit_property(
            &foreign,
            index,
            &catalog,
            &descriptors,
            select(),
            physical()
        )
        .is_err());

        use crate::{
            comparator_requirements::{admit_properties, registration_key, Requirement},
            native_comparator_definition::{
                Definition as Comparator, Operation, Selection as ComparatorSelection,
            },
        };
        let logical = if let Shape::Scalar { logical_type } = &descriptors[0].shape {
            logical_type.clone()
        } else {
            panic!("fixture scalar")
        };
        let value_artifact = property.value.definition_artifact.clone();
        let graph_bytes = property.value.graph.original_json.as_bytes().to_vec();
        let make_comparator = |value_artifact: Value,
                               graph_bytes: &[u8],
                               native_profile: &Value| {
            let comparator = json!({"interfaceVersion":"truss-native-comparator/0.1.0","profile":pin,"valueDefinition":value_artifact,"sourceDomainDefinition":authored,"nativeDomainProfile":native_profile,"nativeDomainDefinition":empty,"operatorInventory":empty,"strategy":{"kind":"unicode-text-C","nativeType":"pg_catalog.text","encoding":"UTF8","collation":"pg_catalog.C","normalization":"none"},"castOutcome":"exact-or-error","nullOperands":"refuse","absentOperands":"refuse","qualification":empty});
            let originals = BTreeMap::from([
                (
                    "valueDefinition".into(),
                    OriginalArtifact {
                        identity: value_artifact["identity"].as_str().unwrap().into(),
                        bytes: graph_bytes.to_vec(),
                    },
                ),
                (
                    "sourceDomainDefinition".into(),
                    OriginalArtifact {
                        identity: authored["identity"].as_str().unwrap().into(),
                        bytes: bytes.clone(),
                    },
                ),
                (
                    "nativeDomainDefinition".into(),
                    OriginalArtifact {
                        identity: "fixture".into(),
                        bytes: b"{}".to_vec(),
                    },
                ),
                (
                    "operatorInventory".into(),
                    OriginalArtifact {
                        identity: "fixture".into(),
                        bytes: b"{}".to_vec(),
                    },
                ),
                (
                    "qualification".into(),
                    OriginalArtifact {
                        identity: "fixture".into(),
                        bytes: b"{}".to_vec(),
                    },
                ),
            ]);
            Comparator::parse(
                &comparator.to_string(),
                ComparatorSelection {
                    profile: &pin,
                    native_profile,
                    original_artifacts: &originals,
                    operations: &BTreeSet::from([Operation::Equality]),
                },
                &logical,
            )
            .unwrap()
        };
        let registration = registration_key(&record.identity, &member.identity);
        let requirements = vec![Requirement {
            owner: record.identity.clone(),
            identity: member.identity.clone(),
            logical_type: logical.clone(),
            operations: BTreeSet::from([Operation::Equality]),
        }];
        let comparisons = BTreeMap::from([(
            registration.clone(),
            make_comparator(value_artifact.clone(), &graph_bytes, &pin),
        )]);
        let properties = BTreeMap::from([(registration.clone(), property)]);
        admit_properties(&requirements, &properties, &comparisons).unwrap();
        assert!(admit_properties(&requirements, &BTreeMap::new(), &comparisons).is_err());
        let mut wrong_artifact = value_artifact.clone();
        wrong_artifact["identity"] = json!("another-original-graph");
        let wrong = BTreeMap::from([(
            registration.clone(),
            make_comparator(wrong_artifact, &graph_bytes, &pin),
        )]);
        assert!(admit_properties(&requirements, &properties, &wrong).is_err());
        let mut native_profile = pin.clone();
        native_profile["identity"] = json!("another-native-domain");
        let wrong = BTreeMap::from([(
            registration,
            make_comparator(value_artifact, &graph_bytes, &native_profile),
        )]);
        assert!(admit_properties(&requirements, &properties, &wrong).is_err());
        let empty_columns = BTreeMap::new();
        let mut missing = physical();
        missing.columns = &empty_columns;
        assert!(admit_home(&admitted, index, missing).is_err());
        let foreign_inventory = leaf_codec_definition::OriginalArtifact {
            identity: inventory.identity.clone(),
            bytes: b"changed".to_vec(),
        };
        let mut missing = physical();
        missing.inventory = &foreign_inventory;
        assert!(admit_home(&admitted, index, missing).is_err());

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
    #[test]
    fn native_home_cannot_enter_props_path_without_original_join() {
        let binding: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/binding-row.json"
        ))
        .unwrap();
        let admitted = Admission::parse(
            &binding.to_string(),
            binding["bindingProfileId"].as_str().unwrap(),
        )
        .unwrap();
        let inventory = leaf_codec_definition::OriginalArtifact {
            identity: binding["basis"]["layoutInventory"]["identity"]
                .as_str()
                .unwrap()
                .into(),
            bytes: b"{}".to_vec(),
        };
        assert!(admit_home(
            &admitted,
            0,
            PhysicalSelection {
                profile: &binding["properties"][0]["homeProfile"],
                inventory: &inventory,
                relations: &BTreeMap::new(),
                columns: &BTreeMap::new(),
                row_join: None,
                obligations: &BTreeSet::new(),
                edge_association: None
            }
        )
        .is_err());
    }
}
