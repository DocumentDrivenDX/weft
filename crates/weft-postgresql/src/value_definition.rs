//! Structural original value graph admission. Codec/source qualification follows.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    error::{Diagnostic, Result},
    json::{checked_json, sha256},
};
#[jsonschema::validator(
    path = "../../tests/truss-postgresql/upstream/value-definition-schema-bundle.json"
)]
struct Shape;
#[derive(Debug)]
pub struct Graph {
    pub original_json: String,
    pub value: Value,
    pub artifacts: BTreeMap<String, Vec<u8>>,
    pub root: usize,
    pub references: Vec<Vec<usize>>,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
impl Graph {
    /// Verify every authored node against the original pinned model, retaining
    /// lexical artifacts separately. This does not qualify shape or codec meaning.
    pub fn verify_model_sources(
        &self,
        catalog: &weft_core::model::Catalog,
        expected_root: &weft_core::ir::Identity,
    ) -> Result<()> {
        let nodes = self.value["nodes"]
            .as_array()
            .ok_or_else(|| fail("Missing graph nodes"))?;
        let root = nodes
            .get(self.root)
            .ok_or_else(|| fail("Missing graph root"))?;
        if root["authoredIdentity"] != serde_json::json!(expected_root) {
            return Err(fail("Graph root differs from selected authored identity"));
        }
        for (index, node) in nodes.iter().enumerate() {
            let identity: weft_core::ir::Identity =
                serde_json::from_value(node["authoredIdentity"].clone())
                    .map_err(|_| fail("Invalid authored node identity"))?;
            let input = catalog
                .inputs
                .iter()
                .find(|input| {
                    input.pin.document_id == identity.document_id
                        && input.pin.revision == identity.revision
                        && input.selected_module_ids.contains(&identity.module)
                })
                .ok_or_else(|| fail("Graph node does not belong to an original selected model"))?;
            let document = checked_json(&input.document_json)
                .map_err(|_| fail("Original model JSON refused"))?;
            let element = document["modules"]
                .as_array()
                .and_then(|modules| modules.iter().find(|m| m["id"] == identity.module))
                .and_then(|module| module["elements"].as_array())
                .and_then(|elements| elements.iter().find(|e| e["id"] == identity.element))
                .ok_or_else(|| fail("Graph authored element is absent from original model"))?;
            let decode = |path: &str| -> Result<Value> {
                let bytes = self
                    .artifacts
                    .get(path)
                    .ok_or_else(|| fail("Missing original graph artifact"))?;
                let text = std::str::from_utf8(bytes)
                    .map_err(|_| fail("Authored definition is not UTF-8"))?;
                checked_json(text).map_err(|_| fail("Authored definition JSON refused"))
            };
            if decode(&format!("/nodes/{index}/authoredDefinition"))? != *element {
                return Err(fail(
                    "Graph authored definition differs from original model",
                ));
            }
            if index == self.root && decode("/acceptedDefinition")? != *element {
                return Err(fail("Graph accepted definition differs from original root"));
            }
        }
        Ok(())
    }
    /// Match storage graph topology to the frontend's resolved finite type graph.
    /// Literal storage names and representations still require selected codec admission.
    pub fn verify_descriptors(
        &self,
        descriptors: &[weft_core::application_model::Descriptor],
        root_identity: &weft_core::ir::Identity,
    ) -> Result<()> {
        use weft_core::application_model::Shape;
        let nodes = self.value["nodes"]
            .as_array()
            .ok_or_else(|| fail("Missing graph nodes"))?;
        if nodes.len() != descriptors.len()
            || nodes
                .get(self.root)
                .is_none_or(|n| n["authoredIdentity"] != serde_json::json!(root_identity))
        {
            return Err(fail(
                "Storage graph closure or root differs from resolved type graph",
            ));
        }
        let by_id: BTreeMap<_, _> = nodes
            .iter()
            .filter_map(|n| n["nodeId"].as_str().map(|id| (id, n)))
            .collect();
        let referred_identity = |id: &Value| -> Result<&Value> {
            by_id
                .get(id.as_str().ok_or_else(|| fail("Invalid graph reference"))?)
                .map(|n| &n["authoredIdentity"])
                .ok_or_else(|| fail("Unresolved graph reference"))
        };
        let mut seen = BTreeSet::new();
        for descriptor in descriptors {
            let identity = serde_json::json!(descriptor.identity);
            if !seen.insert(identity.to_string()) {
                return Err(fail("Repeated resolved descriptor identity"));
            }
            let node = nodes
                .iter()
                .find(|n| n["authoredIdentity"] == identity)
                .ok_or_else(|| fail("Resolved identity is missing from storage graph"))?;
            let shape = &node["shape"];
            let matches = match &descriptor.shape {
                Shape::Scalar { logical_type } => {
                    shape["kind"] == "scalar"
                        && shape["family"] == serde_json::json!(logical_type.family)
                }
                Shape::Sequence { item } => {
                    shape["kind"] == "sequence"
                        && *referred_identity(&shape["itemNodeId"])? == serde_json::json!(item)
                }
                Shape::Map { item } => {
                    shape["kind"] == "map"
                        && *referred_identity(&shape["itemNodeId"])? == serde_json::json!(item)
                }
                Shape::Structured { record } => {
                    shape["kind"] == "structured"
                        && *referred_identity(&shape["recordNodeId"])? == serde_json::json!(record)
                }
                Shape::Record { members } => {
                    shape["kind"] == "record"
                        && shape["members"].as_array().is_some_and(|bound| {
                            bound.len() == members.len()
                                && bound.iter().zip(members).all(|(binding, member)| {
                                    binding["fieldIdentity"] == serde_json::json!(member.identity)
                                })
                        })
                }
            };
            if !matches {
                return Err(fail(
                    "Storage shape differs from resolved authored type topology",
                ));
            }
        }
        Ok(())
    }
    /// Bind each scalar to its separately admitted original leaf codec. Compound
    /// codecs and operation capabilities remain independent admissions.
    pub fn verify_leaf_codecs(
        &self,
        codecs: &BTreeMap<String, crate::leaf_codec_definition::Definition>,
    ) -> Result<()> {
        let nodes = self.value["nodes"]
            .as_array()
            .ok_or_else(|| fail("Missing graph nodes"))?;
        let scalar_count = nodes
            .iter()
            .filter(|node| node["shape"]["kind"] == "scalar")
            .count();
        if scalar_count != codecs.len() {
            return Err(fail(
                "Leaf codec selection differs from graph scalar closure",
            ));
        }
        for (index, node) in nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node["shape"]["kind"] == "scalar")
        {
            let codec = codecs
                .get(
                    node["nodeId"]
                        .as_str()
                        .ok_or_else(|| fail("Invalid scalar node ID"))?,
                )
                .ok_or_else(|| fail("Scalar node has no selected leaf codec"))?;
            let codec_value = checked_json(&codec.original_json)
                .map_err(|_| fail("Original leaf codec JSON refused"))?;
            if self
                .artifacts
                .get(&format!("/nodes/{index}/codecDefinition"))
                .is_none_or(|bytes| bytes != codec.original_json.as_bytes())
                || node["codecProfile"] != codec_value["profile"]
                || node["authoredDefinition"] != codec_value["authoredDefinition"]
                || node["shape"]["family"] != codec_value["rule"]["family"]
                || node["shape"]["storageRepresentation"]
                    != codec_value["rule"]["storageRepresentation"]
            {
                return Err(fail(
                    "Scalar node differs from original selected leaf codec",
                ));
            }
        }
        Ok(())
    }
    /// Match every record-member presence artifact to its separately admitted
    /// original definition and authored Field. Keys are original artifact pointers.
    pub fn verify_record_presence(
        &self,
        definitions: &BTreeMap<String, crate::presence_definition::Definition>,
    ) -> Result<()> {
        let nodes = self.value["nodes"]
            .as_array()
            .ok_or_else(|| fail("Missing graph nodes"))?;
        let by_id: BTreeMap<_, _> = nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| node["nodeId"].as_str().map(|id| (id, (index, node))))
            .collect();
        let mut used = BTreeSet::new();
        for (index, node) in nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node["shape"]["kind"] == "record")
        {
            let members = node["shape"]["members"]
                .as_array()
                .ok_or_else(|| fail("Missing record members"))?;
            for (member_index, member) in members.iter().enumerate() {
                let path =
                    format!("/nodes/{index}/shape/members/{member_index}/presenceDefinition");
                let definition = definitions.get(&path).ok_or_else(|| {
                    fail("Record member lacks selected original presence meaning")
                })?;
                let (child_index, child) = by_id
                    .get(
                        member["valueNodeId"]
                            .as_str()
                            .ok_or_else(|| fail("Invalid member reference"))?,
                    )
                    .ok_or_else(|| fail("Missing record member value node"))?;
                let presence = checked_json(&definition.original_json)
                    .map_err(|_| fail("Original presence JSON refused"))?;
                if self
                    .artifacts
                    .get(&path)
                    .is_none_or(|bytes| bytes != definition.original_json.as_bytes())
                    || presence["acceptedDefinition"] != child["authoredDefinition"]
                    || self
                        .artifacts
                        .get(&format!("/nodes/{child_index}/authoredDefinition"))
                        .is_none_or(|bytes| bytes != &definition.accepted_definition)
                {
                    return Err(fail(
                        "Record member presence differs from original authored field",
                    ));
                }
                used.insert(path);
            }
        }
        if used.len() != definitions.len() {
            return Err(fail(
                "Unrelated record presence definitions cannot enter graph admission",
            ));
        }
        Ok(())
    }
    pub fn parse(raw: &str, profile: &Value, accepted: &[u8]) -> Result<Self> {
        if raw.len() > 4 * 1024 * 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "binding",
                "Value graph exceeds candidate source bound",
            ));
        }
        let value =
            checked_json(raw).map_err(|_| fail("Malformed or duplicate-member value graph"))?;
        if !Shape::is_valid(&value) || &value["profile"] != profile {
            return Err(fail(
                "Value graph grammar/profile differs from registered selection",
            ));
        }
        let mut artifacts = BTreeMap::new();
        let mut stack = vec![(String::new(), &value)];
        let mut total = 0usize;
        while let Some((path, v)) = stack.pop() {
            match v {
                Value::Object(object) => {
                    if let Some(encoded) = object.get("bytesBase64") {
                        let encoded = encoded
                            .as_str()
                            .ok_or_else(|| fail("Artifact base64 is not text"))?;
                        let bytes = STANDARD
                            .decode(encoded)
                            .map_err(|_| fail("Artifact bytes are invalid base64"))?;
                        if STANDARD.encode(&bytes) != encoded || v["sha256"] != sha256(&bytes) {
                            return Err(fail("Value graph artifact custody is invalid"));
                        }
                        total = total
                            .checked_add(bytes.len())
                            .ok_or_else(|| fail("Artifact accounting overflow"))?;
                        if total > 4 * 1024 * 1024 {
                            return Err(Diagnostic::new(
                                "WFT-LIMIT",
                                "binding",
                                "Value graph decoded artifact bound exceeded",
                            ));
                        }
                        artifacts.insert(path.clone(), bytes);
                    }
                    for (key, child) in object {
                        stack.push((
                            format!("{path}/{}", key.replace('~', "~0").replace('/', "~1")),
                            child,
                        ));
                    }
                }
                Value::Array(array) => {
                    for (i, child) in array.iter().enumerate() {
                        stack.push((format!("{path}/{i}"), child));
                    }
                }
                _ => {}
            }
        }
        if artifacts
            .get("/acceptedDefinition")
            .is_none_or(|bytes| bytes != accepted)
        {
            return Err(fail(
                "Value graph accepted definition differs from original authored bytes",
            ));
        }
        let nodes = value["nodes"].as_array().unwrap();
        let mut indexes = BTreeMap::new();
        let mut identities = BTreeSet::new();
        for (i, node) in nodes.iter().enumerate() {
            if indexes
                .insert(node["nodeId"].as_str().unwrap(), i)
                .is_some()
                || !identities.insert(node["authoredIdentity"].to_string())
            {
                return Err(fail(
                    "Duplicate node or authored identity needs an admitted alias rule",
                ));
            }
        }
        let resolve = |id: &Value| {
            indexes
                .get(id.as_str().unwrap())
                .copied()
                .ok_or_else(|| fail("Value graph reference is unresolved"))
        };
        let root = resolve(&value["rootNodeId"])?;
        let mut references = vec![];
        let mut edges = 0usize;
        for node in nodes {
            let shape = &node["shape"];
            let mut children = vec![];
            match shape["kind"].as_str().unwrap() {
                "sequence" | "map" => children.push(resolve(&shape["itemNodeId"])?),
                "structured" => {
                    let index = resolve(&shape["recordNodeId"])?;
                    if nodes[index]["shape"]["kind"] != "record" {
                        return Err(fail("Structured value does not reference a record"));
                    }
                    children.push(index);
                }
                "record" => {
                    let mut names = BTreeSet::new();
                    let mut fields = BTreeSet::new();
                    for member in shape["members"].as_array().unwrap() {
                        let index = resolve(&member["valueNodeId"])?;
                        if member["fieldIdentity"] != nodes[index]["authoredIdentity"]
                            || !names.insert(member["storedMemberName"].as_str().unwrap())
                            || !fields.insert(member["fieldIdentity"].to_string())
                        {
                            return Err(fail("Record field identity or literal name is ambiguous"));
                        }
                        children.push(index);
                    }
                }
                "scalar" => {}
                _ => return Err(fail("Unsupported graph shape")),
            }
            edges += children.len();
            if edges > 100000 {
                return Err(Diagnostic::new(
                    "WFT-LIMIT",
                    "binding",
                    "Value graph reference work bound exceeded",
                ));
            }
            references.push(children);
        }
        let mut pending = vec![root];
        let mut seen = BTreeSet::new();
        while let Some(index) = pending.pop() {
            if seen.insert(index) {
                pending.extend(references[index].iter().copied());
            }
        }
        if seen.len() != nodes.len() {
            return Err(fail("Value graph contains unreachable authored nodes"));
        }
        Ok(Self {
            original_json: raw.into(),
            value,
            artifacts,
            root,
            references,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn authored_definitions_must_match_selected_original_models() {
        use weft_core::{
            ir::Identity,
            model::{Catalog, ModuleInput},
        };
        let cases: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/compiler-cases.json"
        ))
        .unwrap();
        let inputs: Vec<ModuleInput> =
            serde_json::from_value(cases[0]["request"]["modules"].clone()).unwrap();
        let catalog = Catalog::prepare(inputs).unwrap();
        let input = &catalog.inputs[0];
        let document = checked_json(&input.document_json).unwrap();
        let module = &document["modules"][0];
        let element = &module["elements"][0];
        let identity = Identity {
            document_id: input.pin.document_id.clone(),
            revision: input.pin.revision.clone(),
            module: module["id"].as_str().unwrap().into(),
            element: element["id"].as_str().unwrap().into(),
        };
        let bytes = element.to_string().into_bytes();
        let artifact = json!({"identity":"original-node","bytesBase64":STANDARD.encode(&bytes),"sha256":sha256(&bytes)});
        let mut value = fixture();
        value["nodes"] = json!([{"nodeId":"root","authoredIdentity":identity,"authoredDefinition":artifact,"codecProfile":value["profile"],"codecDefinition":artifact,"shape":{"kind":"scalar","family":"fixture","storageRepresentation":"opaque-archive"}}]);
        value["acceptedDefinition"] = artifact;
        let graph = Graph::parse(&value.to_string(), &value["profile"], &bytes).unwrap();
        graph.verify_model_sources(&catalog, &identity).unwrap();
        let mut wrong = identity.clone();
        wrong.revision.push('x');
        assert!(graph.verify_model_sources(&catalog, &wrong).is_err());
        let mut altered = value.clone();
        altered["nodes"][0]["authoredDefinition"] = json!({"identity":"altered","bytesBase64":STANDARD.encode(b"{}"),"sha256":sha256(b"{}")});
        let graph = Graph::parse(&altered.to_string(), &value["profile"], &bytes).unwrap();
        assert!(graph.verify_model_sources(&catalog, &identity).is_err());
        let mut foreign = value.clone();
        foreign["nodes"][0]["authoredIdentity"]["module"] = json!("unselected");
        let graph = Graph::parse(&foreign.to_string(), &value["profile"], &bytes).unwrap();
        let mut foreign_identity = identity;
        foreign_identity.module = "unselected".into();
        assert!(graph
            .verify_model_sources(&catalog, &foreign_identity)
            .is_err());
    }
    #[test]
    fn resolved_topology_cannot_be_replaced_by_another_valid_graph() {
        use weft_core::{
            application_model::{Descriptor, Member, Shape},
            ir::Identity,
        };
        let value = fixture();
        let identity = |index: usize| -> Identity {
            serde_json::from_value(value["nodes"][index]["authoredIdentity"].clone()).unwrap()
        };
        let root = identity(0);
        let record = identity(1);
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
                    members: vec![Member {
                        name: "next".into(),
                        identity: root.clone(),
                    }],
                },
            },
        ];
        parse(&value)
            .unwrap()
            .verify_descriptors(&descriptors, &root)
            .unwrap();
        let mut wrong = descriptors.clone();
        wrong[0].shape = Shape::Map { item: root.clone() };
        assert!(parse(&value)
            .unwrap()
            .verify_descriptors(&wrong, &root)
            .is_err());
        let mut wrong = descriptors.clone();
        if let Shape::Record { members } = &mut wrong[1].shape {
            members.clear();
        }
        assert!(parse(&value)
            .unwrap()
            .verify_descriptors(&wrong, &root)
            .is_err());
        assert!(parse(&value)
            .unwrap()
            .verify_descriptors(&descriptors[..1], &root)
            .is_err());
        let mut wrong = descriptors;
        wrong[1].identity = root.clone();
        assert!(parse(&value)
            .unwrap()
            .verify_descriptors(&wrong, &root)
            .is_err());
    }
    #[test]
    fn record_member_order_and_scalar_family_are_semantic() {
        use weft_core::{
            application_model::{Descriptor, Member, Shape},
            ir::{Family, Identity, LogicalType},
        };
        let mut value = fixture();
        let root: Identity =
            serde_json::from_value(value["nodes"][0]["authoredIdentity"].clone()).unwrap();
        let record: Identity =
            serde_json::from_value(value["nodes"][1]["authoredIdentity"].clone()).unwrap();
        let mut scalar = root.clone();
        scalar.element = "leaf".into();
        let mut node = value["nodes"][0].clone();
        node["nodeId"] = json!("leaf");
        node["authoredIdentity"] = json!(scalar);
        node["shape"] =
            json!({"kind":"scalar","family":"string","storageRepresentation":"json-string"});
        value["nodes"].as_array_mut().unwrap().push(node);
        let presence = value["acceptedDefinition"].clone();
        value["nodes"][1]["shape"]["members"].as_array_mut().unwrap().push(json!({"fieldIdentity":scalar,"storedMemberName":"leaf","valueNodeId":"leaf","presenceDefinition":presence}));
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
                    members: vec![
                        Member {
                            name: "next".into(),
                            identity: root.clone(),
                        },
                        Member {
                            name: "leaf".into(),
                            identity: scalar.clone(),
                        },
                    ],
                },
            },
            Descriptor {
                identity: scalar,
                availability: Some("required".into()),
                shape: Shape::Scalar {
                    logical_type: LogicalType {
                        family: Family::String,
                        facets: json!({}),
                        nullable: false,
                    },
                },
            },
        ];
        parse(&value)
            .unwrap()
            .verify_descriptors(&descriptors, &root)
            .unwrap();
        let mut swapped = value.clone();
        swapped["nodes"][1]["shape"]["members"]
            .as_array_mut()
            .unwrap()
            .swap(0, 1);
        assert!(parse(&swapped)
            .unwrap()
            .verify_descriptors(&descriptors, &root)
            .is_err());
        value["nodes"][2]["shape"]["family"] = json!("integer");
        assert!(parse(&value)
            .unwrap()
            .verify_descriptors(&descriptors, &root)
            .is_err());
    }
    #[test]
    fn graph_scalar_requires_its_exact_admitted_leaf_codec() {
        use crate::leaf_codec_definition::{Definition, OriginalArtifact, Selection};
        let pin = fixture()["profile"].clone();
        let authored = fixture()["acceptedDefinition"].clone();
        let leaf = json!({"interfaceVersion":"truss-jsonb-leaf-codec/0.1.0","profile":pin,"authoredDefinition":authored,"sourceInterpretationProfile":pin,"sourceInterpretationDefinition":authored,"nativeDomainProfile":pin,"nativeDomainDefinition":authored,"rule":{"family":"string","storageRepresentation":"json-string","encoding":"preserve-unicode-scalars","decodedCarrierKind":"string"},"coercion":"none","readDefault":"none","invalidStoredValue":"complete-result-refusal"});
        let originals: BTreeMap<String, OriginalArtifact> = [
            "authoredDefinition",
            "sourceInterpretationDefinition",
            "nativeDomainDefinition",
        ]
        .into_iter()
        .map(|key| {
            (
                key.into(),
                OriginalArtifact {
                    identity: "fixture".into(),
                    bytes: b"{}".to_vec(),
                },
            )
        })
        .collect();
        let codec = Definition::parse(
            &leaf.to_string(),
            Selection {
                profile: &pin,
                source_profile: &pin,
                native_profile: &pin,
                original_artifacts: &originals,
            },
        )
        .unwrap();
        let mut value = fixture();
        value["nodes"].as_array_mut().unwrap().truncate(1);
        value["nodes"][0]["shape"] =
            json!({"kind":"scalar","family":"string","storageRepresentation":"json-string"});
        let bytes = codec.original_json.as_bytes();
        value["nodes"][0]["codecDefinition"] = json!({"identity":"selected-leaf","bytesBase64":STANDARD.encode(bytes),"sha256":sha256(bytes)});
        let codecs = BTreeMap::from([("root".into(), codec)]);
        parse(&value).unwrap().verify_leaf_codecs(&codecs).unwrap();
        assert!(parse(&value)
            .unwrap()
            .verify_leaf_codecs(&BTreeMap::new())
            .is_err());
        let mut wrong = value.clone();
        wrong["nodes"][0]["shape"]["storageRepresentation"] = json!("json-number");
        assert!(parse(&wrong).unwrap().verify_leaf_codecs(&codecs).is_err());
        let mut wrong = value.clone();
        wrong["nodes"][0]["codecProfile"]["identity"] = json!("foreign");
        assert!(parse(&wrong).unwrap().verify_leaf_codecs(&codecs).is_err());
        let mut wrong = value.clone();
        wrong["nodes"][0]["authoredDefinition"]["identity"] = json!("foreign");
        assert!(parse(&wrong).unwrap().verify_leaf_codecs(&codecs).is_err());
        let mut wrong = value;
        wrong["nodes"][0]["codecDefinition"] = authored;
        assert!(parse(&wrong).unwrap().verify_leaf_codecs(&codecs).is_err());
    }
    #[test]
    fn record_presence_is_bound_to_exact_original_member_and_field() {
        use crate::presence_definition::Definition;
        let mut value = fixture();
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
        presence["profile"] = value["profile"].clone();
        presence["acceptedDefinition"] = value["nodes"][0]["authoredDefinition"].clone();
        let definition =
            Definition::parse(&presence.to_string(), &presence["profile"], b"{}").unwrap();
        let bytes = definition.original_json.as_bytes();
        value["nodes"][1]["shape"]["members"][0]["presenceDefinition"] = json!({"identity":"selected-presence","bytesBase64":STANDARD.encode(bytes),"sha256":sha256(bytes)});
        let path = "/nodes/1/shape/members/0/presenceDefinition";
        let definitions = BTreeMap::from([(path.into(), definition)]);
        parse(&value)
            .unwrap()
            .verify_record_presence(&definitions)
            .unwrap();
        assert!(parse(&value)
            .unwrap()
            .verify_record_presence(&BTreeMap::new())
            .is_err());
        let mut wrong = value.clone();
        wrong["nodes"][0]["authoredDefinition"]["identity"] = json!("foreign-field");
        assert!(parse(&wrong)
            .unwrap()
            .verify_record_presence(&definitions)
            .is_err());
        let mut wrong = value.clone();
        wrong["nodes"][1]["shape"]["members"][0]["presenceDefinition"] =
            wrong["acceptedDefinition"].clone();
        assert!(parse(&wrong)
            .unwrap()
            .verify_record_presence(&definitions)
            .is_err());
        let mut misplaced = BTreeMap::new();
        let definition =
            Definition::parse(&presence.to_string(), &presence["profile"], b"{}").unwrap();
        misplaced.insert("/unrelated/presence".into(), definition);
        assert!(parse(&value)
            .unwrap()
            .verify_record_presence(&misplaced)
            .is_err());
    }
    fn fixture() -> Value {
        let artifact = json!({"identity":"fixture","bytesBase64":STANDARD.encode(b"{}"),"sha256":sha256(b"{}")});
        let pin = json!({"identity":"graph-fixture","version":"0.1.0","sha256":sha256(b"{}")});
        let node = |id: &str, element: &str, shape: Value| json!({"nodeId":id,"authoredIdentity":{"documentId":"d","revision":"1","module":"m","element":element},"authoredDefinition":artifact,"codecProfile":pin,"codecDefinition":artifact,"shape":shape});
        json!({"interfaceVersion":"truss-value-definition/0.1.0","profile":pin,"rootNodeId":"root","acceptedDefinition":artifact,"nodes":[node("root","structured",json!({"kind":"structured","recordNodeId":"record"})),node("record","record",json!({"kind":"record","members":[{"fieldIdentity":{"documentId":"d","revision":"1","module":"m","element":"structured"},"storedMemberName":"next","valueNodeId":"root","presenceDefinition":artifact}]}))]})
    }
    fn parse(value: &Value) -> Result<Graph> {
        Graph::parse(&value.to_string(), &fixture()["profile"], b"{}")
    }
    #[test]
    fn cyclic_type_graph_is_finite_and_preserves_original_artifact_bytes() {
        let v = fixture();
        let g = parse(&v).unwrap();
        assert_eq!(g.references, vec![vec![1], vec![0]]);
        assert_eq!(g.root, 0);
        assert_eq!(g.artifacts["/acceptedDefinition"], b"{}");
        assert_eq!(g.value, v);
    }
    #[test]
    fn dangling_unreachable_duplicate_and_corrupt_graphs_refuse() {
        let base = fixture();
        let mut v = base.clone();
        v["nodes"][0]["shape"]["recordNodeId"] = json!("missing");
        assert!(parse(&v).is_err());
        let mut v = base.clone();
        v["nodes"][1]["nodeId"] = json!("root");
        assert!(parse(&v).is_err());
        let mut v = base.clone();
        v["nodes"][1]["authoredIdentity"] = v["nodes"][0]["authoredIdentity"].clone();
        assert!(parse(&v).is_err());
        let mut v = base.clone();
        let mut extra = v["nodes"][0].clone();
        extra["nodeId"] = json!("unused");
        extra["authoredIdentity"]["element"] = json!("unused");
        v["nodes"].as_array_mut().unwrap().push(extra);
        assert!(parse(&v).is_err());
        let mut v = base.clone();
        v["nodes"][0]["codecDefinition"]["sha256"] = json!("0".repeat(64));
        assert!(parse(&v).is_err());
    }
}
