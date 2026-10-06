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
