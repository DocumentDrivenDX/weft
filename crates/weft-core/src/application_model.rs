//! Identity-based application descriptors; native encodings remain backend obligations.
use crate::{
    error::{Diagnostic, Result},
    ir::{Identity, LogicalType, Span},
    model::{scalar_type, Catalog, Record},
    syntax::Name,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub name: String,
    pub identity: Identity,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Shape {
    Scalar {
        #[serde(rename = "type")]
        logical_type: LogicalType,
    },
    Sequence {
        item: Identity,
    },
    Map {
        item: Identity,
    },
    Structured {
        record: Identity,
    },
    Record {
        members: Vec<Member>,
    },
}
#[derive(Debug, Clone, Serialize)]
pub struct Descriptor {
    pub identity: Identity,
    pub availability: Option<String>,
    #[serde(flatten)]
    pub shape: Shape,
}
#[derive(Debug, Clone, Serialize)]
pub struct EntityDescriptor {
    pub members: Vec<Member>,
    pub graph: Vec<Descriptor>,
}
#[derive(Debug, Clone, Serialize)]
pub struct AuthoredKey {
    pub id: String,
    pub fields: Vec<Identity>,
    pub types: Vec<LogicalType>,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-TYPE", "model", message)
}
impl Catalog {
    pub(crate) fn local_element(
        &self,
        record: &Record,
        reference: &Value,
    ) -> Result<(Identity, Value)> {
        let mid = reference["module"]
            .as_str()
            .ok_or_else(|| fail("Missing local module reference"))?;
        let eid = reference["element"]
            .as_str()
            .ok_or_else(|| fail("Missing local element reference"))?;
        let element = self.documents[record.document]["modules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["id"] == mid)
            .and_then(|m| {
                m["elements"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| e["id"] == eid)
            })
            .ok_or_else(|| fail("Unresolved local type dependency"))?;
        let mut identity = record.identity.clone();
        identity.module = mid.into();
        identity.element = eid.into();
        Ok((identity, element.clone()))
    }
    fn members(&self, record: &Record, value: &Value) -> Result<Vec<Member>> {
        let refs = value["members"]
            .as_array()
            .ok_or_else(|| fail("Record needs explicit ordered members"))?;
        let mut ids = BTreeSet::new();
        let mut members = Vec::new();
        for r in refs {
            let (identity, field) = self.local_element(record, r)?;
            if field["kind"] != "field" {
                return Err(fail("Record member must reference a Field"));
            }
            if !ids.insert((identity.module.clone(), identity.element.clone())) {
                return Err(fail("Repeated member identity"));
            }
            let name = field["name"]
                .as_str()
                .ok_or_else(|| fail("Selected Field needs an authored name"))?
                .into();
            members.push(Member { name, identity });
        }
        Ok(members)
    }
    pub fn entity_descriptor(&self, record: &Record) -> Result<EntityDescriptor> {
        Ok(EntityDescriptor {
            members: self.members(record, &record.value)?,
            graph: self.type_graph(record, record.identity.clone(), record.value.clone())?,
        })
    }
    pub fn member_descriptor(
        &self,
        record: &Record,
        name: &Name,
    ) -> Result<(Member, Vec<Descriptor>)> {
        let matches: Vec<_> = self
            .members(record, &record.value)?
            .into_iter()
            .filter(|m| name.matches(&m.name))
            .collect();
        if matches.len() != 1 {
            return Err(Diagnostic::new(
                if matches.is_empty() {
                    "WFT-NAME-MISSING"
                } else {
                    "WFT-NAME-AMBIGUOUS"
                },
                "resolve",
                "Selected member name missing or ambiguous",
            )
            .at(&name.span));
        }
        let member = matches.into_iter().next().unwrap();
        let (_, v) = self.local_element(
            record,
            &serde_json::json!({"module":member.identity.module,"element":member.identity.element}),
        )?;
        let graph = self.type_graph(record, member.identity.clone(), v)?;
        Ok((member, graph))
    }
    fn type_graph(
        &self,
        record: &Record,
        identity: Identity,
        value: Value,
    ) -> Result<Vec<Descriptor>> {
        fn visit(
            c: &Catalog,
            r: &Record,
            id: Identity,
            v: Value,
            seen: &mut BTreeSet<(String, String)>,
            graph: &mut Vec<Descriptor>,
            depth: usize,
        ) -> Result<()> {
            if !seen.insert((id.module.clone(), id.element.clone())) {
                return Ok(());
            }
            if depth > 128 {
                return Err(Diagnostic::new(
                    "WFT-LIMIT",
                    "model",
                    "Selected type graph exceeds depth 128",
                ));
            }
            if seen.len() > 4096 {
                return Err(Diagnostic::new(
                    "WFT-LIMIT",
                    "model",
                    "Selected type graph exceeds 4096 identities",
                ));
            }
            let mut dependencies = Vec::new();
            let (availability, shape) = if v["kind"] == "record" {
                let members = c.members(r, &v)?;
                for m in &members {
                    dependencies.push(c.local_element(r,&serde_json::json!({"module":m.identity.module,"element":m.identity.element}))?);
                }
                (None, Shape::Record { members })
            } else if v["kind"] == "field" {
                let availability = v["nullability"]
                    .as_str()
                    .filter(|s| matches!(*s, "required" | "absent-allowed"))
                    .ok_or_else(|| fail("Selected Field availability is not established"))?
                    .to_string();
                let references = v
                    .get("references")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let shape = match v["cardinality"].as_str() {
                    Some("array" | "map") => {
                        if !references.is_empty() || v.get("scalarType").is_some() {
                            return Err(fail(
                                "Container cannot imply scalar or structured meaning",
                            ));
                        }
                        let dep = c.local_element(r, &v["itemType"])?;
                        if dep.1["kind"] != "field" {
                            return Err(fail("Container itemType must reference a Field"));
                        }
                        let item = dep.0.clone();
                        dependencies.push(dep);
                        if v["cardinality"] == "array" {
                            Shape::Sequence { item }
                        } else {
                            Shape::Map { item }
                        }
                    }
                    Some("one")
                        if references.len() == 1 && references[0]["role"] == "record-type" =>
                    {
                        let dep = c.local_element(r, &references[0])?;
                        if dep.1["kind"] != "record" || v.get("scalarType").is_some() {
                            return Err(fail(
                                "Structured Field needs a sole record-type reference",
                            ));
                        }
                        let record = dep.0.clone();
                        dependencies.push(dep);
                        Shape::Structured { record }
                    }
                    Some("one") => {
                        let mut required = v.clone();
                        required["nullability"] = Value::String("required".into());
                        let name = Name {
                            value: v["name"].as_str().unwrap_or("").into(),
                            quoted: true,
                            span: Span { start: 0, end: 0 },
                        };
                        Shape::Scalar {
                            logical_type: scalar_type(&required, &name)?,
                        }
                    }
                    _ => return Err(fail("Selected cardinality is unsupported")),
                };
                if !matches!(shape, Shape::Scalar { .. })
                    && v.get("facets")
                        .and_then(Value::as_object)
                        .is_some_and(|f| !f.is_empty())
                {
                    return Err(fail(
                        "Selected container or structured facets are unsupported",
                    ));
                }
                (Some(availability), shape)
            } else {
                return Err(fail(
                    "Selected type graph contains an unsupported element role",
                ));
            };
            graph.push(Descriptor {
                identity: id,
                availability,
                shape,
            });
            for (id, v) in dependencies {
                visit(c, r, id, v, seen, graph, depth + 1)?;
            }
            Ok(())
        }
        let mut graph = Vec::new();
        visit(
            self,
            record,
            identity,
            value,
            &mut BTreeSet::new(),
            &mut graph,
            0,
        )?;
        Ok(graph)
    }
    pub fn authored_key(&self, record: &Record, key_id: &str) -> Result<AuthoredKey> {
        let keys = record.value["keys"]
            .as_array()
            .ok_or_else(|| fail("Record has no authored unique key"))?;
        let matched: Vec<_> = keys.iter().filter(|k| k["id"] == key_id).collect();
        if matched.len() != 1 {
            return Err(fail("Authored key identity is missing or ambiguous"));
        }
        let members = self.members(record, &record.value)?;
        let mut fields = Vec::new();
        let mut types = Vec::new();
        let mut seen = BTreeSet::new();
        for reference in matched[0]["fields"].as_array().unwrap() {
            let (id, field) = self.local_element(record, reference)?;
            if !members.iter().any(|m| m.identity == id)
                || !seen.insert((id.module.clone(), id.element.clone()))
            {
                return Err(fail("Key must contain distinct declared member Fields"));
            }
            let name = Name {
                value: field["name"].as_str().unwrap_or("").into(),
                quoted: true,
                span: Span { start: 0, end: 0 },
            };
            types.push(scalar_type(&field, &name)?);
            fields.push(id);
        }
        Ok(AuthoredKey {
            id: key_id.into(),
            fields,
            types,
        })
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipIdentity {
    pub document_id: String,
    pub revision: String,
    pub module: String,
    pub relationship: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipRead {
    pub identity: RelationshipIdentity,
    pub inverse: bool,
    pub from: Identity,
    pub to: Identity,
    pub source_key: AuthoredKey,
    pub target_key: AuthoredKey,
    pub source_multiplicity: Value,
    pub target_multiplicity: Value,
    pub target_lifecycle: String,
}
impl Catalog {
    pub fn relationship_read(&self, source: &Record, name: &Name) -> Result<RelationshipRead> {
        let mut matches = Vec::new();
        for module in self.documents[source.document]["modules"]
            .as_array()
            .unwrap()
        {
            if !self.inputs[source.document]
                .selected_module_ids
                .iter()
                .any(|id| module["id"] == id.as_str())
            {
                continue;
            }
            let Some(relationships) = module["relationships"].as_array() else {
                continue;
            };
            let mut ids = BTreeSet::new();
            for rel in relationships {
                if !ids.insert(rel["id"].as_str().unwrap()) {
                    return Err(fail("Duplicate relationship identity"));
                }
                let endpoint_matches = |v: &Value| {
                    v["module"] == source.identity.module && v["element"] == source.identity.element
                };
                let forward = name.matches(rel["name"].as_str().unwrap())
                    && rel["source"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(endpoint_matches);
                let inverse = rel["inverse"].as_str().is_some_and(|n| name.matches(n))
                    && rel["target"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(endpoint_matches);
                if forward {
                    matches.push((module, rel, false));
                }
                if inverse {
                    matches.push((module, rel, true));
                }
            }
        }
        if matches.len() != 1 {
            return Err(Diagnostic::new(
                if matches.is_empty() {
                    "WFT-NAME-MISSING"
                } else {
                    "WFT-NAME-AMBIGUOUS"
                },
                "resolve",
                "Relationship name is missing or ambiguous",
            )
            .at(&name.span));
        }
        let (module, rel, inverse) = matches[0];
        if rel["source"].as_array().unwrap().len() != 1
            || rel["target"].as_array().unwrap().len() != 1
            || rel["directed"] != true
            || rel.get("associationRecord").is_some()
        {
            return Err(fail("Relationship requires a supported monomorphic directed traversal without an association Record"));
        }
        let lifecycle = rel["targetLifecycle"]
            .as_str()
            .filter(|v| matches!(*v, "owned" | "independent"))
            .ok_or_else(|| fail("Relationship lifecycle meaning is unsupported"))?;
        for side in ["sourceMultiplicity", "targetMultiplicity"] {
            let m = &rel[side];
            let min = m["min"].as_u64().unwrap();
            if m["max"] != "*" && m["max"].as_u64().is_none_or(|max| max < min) {
                return Err(fail("Relationship multiplicity is inconsistent"));
            }
        }
        // Validate the authored target key even for inverse traversal; neither direction invents a key.
        let (target_id, target_value) = self.local_element(source, &rel["target"][0])?;
        if target_value["kind"] != "record" {
            return Err(fail("Relationship target must be a Record"));
        }
        let target_record = Record {
            identity: target_id.clone(),
            pin: source.pin.clone(),
            value: target_value,
            document: source.document,
        };
        let forward_key =
            self.authored_key(&target_record, rel["target"][0]["key"].as_str().unwrap())?;
        let (source_id, source_value) = self.local_element(source, &rel["source"][0])?;
        if source_value["kind"] != "record" {
            return Err(fail("Relationship source must be a Record"));
        }
        let from_record = Record {
            identity: source_id.clone(),
            pin: source.pin.clone(),
            value: source_value,
            document: source.document,
        };
        let keys = from_record.value["keys"]
            .as_array()
            .ok_or_else(|| fail("Relationship source endpoint needs an authored key"))?;
        let primary: Vec<_> = keys.iter().filter(|k| k["primary"] == true).collect();
        let key = if primary.len() == 1 {
            primary[0]
        } else if primary.is_empty() && keys.len() == 1 {
            &keys[0]
        } else {
            return Err(fail("Relationship source endpoint key is ambiguous"));
        };
        let source_key = self.authored_key(&from_record, key["id"].as_str().unwrap())?;
        let (to, query_source_key, target_key) = if inverse {
            (source_id.clone(), forward_key, source_key)
        } else {
            (target_id.clone(), source_key, forward_key)
        };
        Ok(RelationshipRead {
            identity: RelationshipIdentity {
                document_id: source.identity.document_id.clone(),
                revision: source.identity.revision.clone(),
                module: module["id"].as_str().unwrap().into(),
                relationship: rel["id"].as_str().unwrap().into(),
            },
            inverse,
            from: if inverse { target_id } else { source_id },
            to,
            source_key: query_source_key,
            target_key,
            source_multiplicity: rel["sourceMultiplicity"].clone(),
            target_multiplicity: rel["targetMultiplicity"].clone(),
            target_lifecycle: lifecycle.into(),
        })
    }
}
