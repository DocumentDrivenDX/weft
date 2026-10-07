//! Bounded traversal of runtime JSONB values under original finite UMF topology.
//! Leaf and member presence procedures remain explicitly selected callbacks.
use crate::value_definition::{Layout, LayoutShape, MemberSlot};
use serde_json::{Map, Value};
use weft_core::error::{Diagnostic, Result};
pub struct Budget {
    pub remaining_nodes: usize,
    pub remaining_key_bytes: usize,
    pub remaining_members: usize,
    pub max_depth: usize,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-DECODE", "decode", message)
}
/// Couple traversal to the original property graph/descriptor custody before
/// invoking any leaf or member procedure. The returned body is intermediate;
/// public result-envelope decoding still owns logical member representation.
pub fn decode_property(
    property: &crate::property_definition::PropertyAdmission,
    value: &Value,
    budget: &mut Budget,
    scalar: impl FnMut(&[u8], &str, &str, &Value) -> Result<Value>,
    absent: impl FnMut(&Value, &[u8]) -> Result<()>,
) -> Result<Value> {
    crate::result_definition::property_column(property, 1, "weft_recursive_body")?;
    let layout = property.value.graph.layout()?;
    decode(&layout, value, budget, scalar, absent)
}
/// Original-property entry point for logical record assembly. Recheck graph and
/// descriptor custody before invoking any selected host procedure.
pub fn decode_property_with_records(
    property: &crate::property_definition::PropertyAdmission,
    value: &Value,
    budget: &mut Budget,
    scalar: impl FnMut(&[u8], &str, &str, &Value) -> Result<Value>,
    absent: impl FnMut(&Value, &[u8]) -> Result<()>,
    record: impl FnMut(Vec<(&MemberSlot<'_>, Option<Value>)>, &mut Budget) -> Result<Value>,
) -> Result<Value> {
    crate::result_definition::property_column(property, 1, "weft_recursive_body")?;
    let layout = property.value.graph.layout()?;
    decode_with_records(&layout, value, budget, scalar, absent, record)
}
/// Decode storage bodies with original UMF logical member names. Selected
/// member procedures own availability/null envelopes; this bridge only supplies
/// the exact descriptor, original presence bytes and explicit decoded presence.
pub fn decode_logical_property(
    property: &crate::property_definition::PropertyAdmission,
    value: &Value,
    budget: &mut Budget,
    scalar: impl FnMut(&[u8], &str, &str, &Value) -> Result<Value>,
    absent: impl FnMut(&Value, &[u8]) -> Result<()>,
    mut member: impl FnMut(
        &weft_core::application_model::Descriptor,
        &[u8],
        Option<Value>,
    ) -> Result<Value>,
) -> Result<Value> {
    decode_property_with_records(property, value, budget, scalar, absent, |slots, budget| {
        assemble_logical_record(property.value.descriptors(), slots, budget, &mut member)
    })
}
fn assemble_logical_record(
    descriptors: &[weft_core::application_model::Descriptor],
    slots: Vec<(&MemberSlot<'_>, Option<Value>)>,
    budget: &mut Budget,
    member: &mut impl FnMut(
        &weft_core::application_model::Descriptor,
        &[u8],
        Option<Value>,
    ) -> Result<Value>,
) -> Result<Value> {
    use weft_core::application_model::Shape;
    let mut selected = Vec::new();
    let mut names = std::collections::BTreeSet::new();
    let mut bytes = 0usize;
    for (slot, value) in slots {
        let descriptor = descriptors
            .iter()
            .find(|d| serde_json::json!(d.identity) == *slot.field_identity)
            .ok_or_else(|| fail("Logical member lacks original descriptor"))?;
        let mut name = None;
        for d in descriptors {
            if let Shape::Record { members } = &d.shape {
                for m in members.iter().filter(|m| m.identity == descriptor.identity) {
                    if name.is_some_and(|prior| prior != m.name.as_str()) {
                        return Err(fail("Logical member identity has ambiguous authored names"));
                    }
                    name = Some(m.name.as_str());
                }
            }
        }
        let name = name.ok_or_else(|| fail("Logical member lacks original Record membership"))?;
        if !names.insert(name) {
            return Err(fail("Logical Record has duplicate authored names"));
        }
        bytes = bytes
            .checked_add(name.len())
            .ok_or_else(|| fail("Logical member name size overflow"))?;
        selected.push((name, descriptor, slot.presence_bytes, value));
    }
    if bytes > budget.remaining_key_bytes {
        return Err(Diagnostic::new(
            "WFT-LIMIT",
            "decode",
            "Logical member name budget exhausted",
        ));
    }
    budget.remaining_key_bytes -= bytes;
    let mut object = Map::new();
    for (name, descriptor, presence, value) in selected {
        object.insert(name.into(), member(descriptor, presence, value)?);
    }
    Ok(Value::Object(object))
}
/// Normalize a complete recursive value body, preserving literal stored names.
/// Every runtime occurrence is visited even when metadata nodes are shared/cyclic.
/// Scalar callbacks own original codec/type/facet/null semantics and allocation
/// bounds. Presence callbacks decide each missing record member from original
/// field identity, presence bytes and UMF descriptor; no default is inferred.
pub fn decode(
    layout: &Layout<'_>,
    value: &Value,
    budget: &mut Budget,
    scalar: impl FnMut(&[u8], &str, &str, &Value) -> Result<Value>,
    absent: impl FnMut(&Value, &[u8]) -> Result<()>,
) -> Result<Value> {
    decode_with_records(layout, value, budget, scalar, absent, |members, _| {
        let mut object = Map::new();
        for (member, value) in members {
            if let Some(value) = value {
                if object.insert(member.stored_name.into(), value).is_some() {
                    return Err(fail("Repeated original record storage member"));
                }
            }
        }
        Ok(Value::Object(object))
    })
}
/// Assemble every original record slot after its children have decoded. Missing
/// slots have already passed the original presence callback and remain explicit
/// here. Hosts can map storage names to authored identities/names and construct
/// their selected member envelopes without revisiting or dropping slots.
pub fn decode_with_records(
    layout: &Layout<'_>,
    value: &Value,
    budget: &mut Budget,
    mut scalar: impl FnMut(&[u8], &str, &str, &Value) -> Result<Value>,
    mut absent: impl FnMut(&Value, &[u8]) -> Result<()>,
    mut record: impl FnMut(Vec<(&MemberSlot<'_>, Option<Value>)>, &mut Budget) -> Result<Value>,
) -> Result<Value> {
    enum Frame<'a> {
        Enter(usize, &'a Value, usize),
        Array(usize),
        Object(Vec<&'a str>),
        Record(Vec<&'a MemberSlot<'a>>, Vec<bool>),
    }
    reserve_nodes(1, budget)?;
    let mut pending = vec![Frame::Enter(layout.root, value, 0)];
    let mut results = Vec::new();
    while let Some(frame) = pending.pop() {
        match frame {
            Frame::Enter(index, value, depth) => {
                if depth > budget.max_depth {
                    return Err(Diagnostic::new(
                        "WFT-LIMIT",
                        "decode",
                        "Recursive value traversal budget exhausted",
                    ));
                }
                let node = layout
                    .nodes
                    .get(index)
                    .ok_or_else(|| fail("Runtime value references missing original node"))?;
                match &node.shape {
                    LayoutShape::Scalar {
                        family,
                        storage_representation,
                    } => {
                        let decoded =
                            scalar(node.codec_bytes, family, storage_representation, value)?;
                        if !matches!(decoded, Value::String(_) | Value::Bool(_) | Value::Null) {
                            return Err(fail("Scalar decoder returned a number/container outside exact carrier contract"));
                        }
                        results.push(decoded);
                    }
                    LayoutShape::Structured { record } => {
                        reserve_nodes(1, budget)?;
                        pending.push(Frame::Enter(*record, value, depth + 1))
                    }
                    LayoutShape::Sequence { item } => {
                        let values = value
                            .as_array()
                            .ok_or_else(|| fail("Sequence value is not an array"))?;
                        reserve_nodes(values.len(), budget)?;
                        pending.push(Frame::Array(values.len()));
                        for child in values.iter().rev() {
                            pending.push(Frame::Enter(*item, child, depth + 1));
                        }
                    }
                    LayoutShape::Map { item } => {
                        let values = value
                            .as_object()
                            .ok_or_else(|| fail("Map value is not an object"))?;
                        reserve(values.len(), values.keys().map(String::as_str), budget)?;
                        reserve_nodes(values.len(), budget)?;
                        pending.push(Frame::Object(values.keys().map(String::as_str).collect()));
                        for child in values.values().rev() {
                            pending.push(Frame::Enter(*item, child, depth + 1));
                        }
                    }
                    LayoutShape::Record { members } => {
                        let values = value
                            .as_object()
                            .ok_or_else(|| fail("Record value is not an object"))?;
                        if members.len() > budget.remaining_members {
                            return Err(Diagnostic::new(
                                "WFT-LIMIT",
                                "decode",
                                "Record member work budget exhausted",
                            ));
                        }
                        budget.remaining_members -= members.len();
                        let names: std::collections::BTreeSet<_> =
                            members.iter().map(|member| member.stored_name).collect();
                        if names.len() != members.len()
                            || values.keys().any(|key| !names.contains(key.as_str()))
                        {
                            return Err(fail(
                                "Record member layout is repeated or stored member is unknown",
                            ));
                        }
                        reserve(values.len(), values.keys().map(String::as_str), budget)?;
                        reserve_nodes(values.len(), budget)?;
                        let mut children = Vec::new();
                        for member in members {
                            if let Some(child) = values.get(member.stored_name) {
                                children.push((member.stored_name, member.value_node, child));
                            } else {
                                absent(member.field_identity, member.presence_bytes)?;
                            }
                        }
                        pending.push(Frame::Record(
                            members.iter().collect(),
                            members
                                .iter()
                                .map(|member| values.contains_key(member.stored_name))
                                .collect(),
                        ));
                        for (_, index, child) in children.into_iter().rev() {
                            pending.push(Frame::Enter(index, child, depth + 1));
                        }
                    }
                }
            }
            Frame::Array(count) => {
                let start = results
                    .len()
                    .checked_sub(count)
                    .ok_or_else(|| fail("Incomplete sequence decode"))?;
                let values = results.split_off(start);
                results.push(Value::Array(values));
            }
            Frame::Record(members, present) => {
                let count = present.iter().filter(|present| **present).count();
                let start = results
                    .len()
                    .checked_sub(count)
                    .ok_or_else(|| fail("Incomplete record decode"))?;
                let mut values = results.split_off(start).into_iter();
                let slots = members
                    .into_iter()
                    .zip(present)
                    .map(|(member, present)| (member, if present { values.next() } else { None }))
                    .collect();
                let assembled = record(slots, budget)?;
                if !assembled.is_object() {
                    return Err(fail(
                        "Record assembler returned a non-object logical representation",
                    ));
                }
                results.push(assembled);
            }
            Frame::Object(names) => {
                let start = results
                    .len()
                    .checked_sub(names.len())
                    .ok_or_else(|| fail("Incomplete object decode"))?;
                let values = results.split_off(start);
                let mut object = Map::new();
                for (name, value) in names.into_iter().zip(values) {
                    if object.insert(name.into(), value).is_some() {
                        return Err(fail("Repeated original record storage member"));
                    }
                }
                results.push(Value::Object(object));
            }
        }
    }
    if results.len() != 1 {
        return Err(fail("Recursive decode did not produce one complete value"));
    }
    Ok(results.pop().unwrap())
}
// Charge queued occurrences, rather than only visits. Shared/cyclic topology
// cannot accumulate uncharged pending work through nested wide containers.
fn reserve_nodes(count: usize, budget: &mut Budget) -> Result<()> {
    budget.remaining_nodes = budget.remaining_nodes.checked_sub(count).ok_or_else(|| {
        Diagnostic::new(
            "WFT-LIMIT",
            "decode",
            "Recursive child reservation exhausted",
        )
    })?;
    Ok(())
}
fn reserve<'a>(
    count: usize,
    mut keys: impl Iterator<Item = &'a str>,
    budget: &mut Budget,
) -> Result<()> {
    let bytes = keys
        .try_fold(0usize, |sum, key| sum.checked_add(key.len()))
        .ok_or_else(|| fail("Literal member size overflow"))?;
    if count > budget.remaining_nodes || bytes > budget.remaining_key_bytes {
        return Err(Diagnostic::new(
            "WFT-LIMIT",
            "decode",
            "Object traversal reservation exhausted",
        ));
    }
    budget.remaining_key_bytes -= bytes;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_definition::{LayoutNode, MemberSlot};
    use serde_json::json;
    fn budget() -> Budget {
        Budget {
            remaining_nodes: 1000,
            remaining_key_bytes: 1000,
            remaining_members: 1000,
            max_depth: 64,
        }
    }
    #[test]
    fn nested_exact_values_literal_keys_and_empty_containers_survive() {
        let layout = Layout {
            root: 0,
            nodes: vec![
                LayoutNode {
                    codec_bytes: b"map",
                    shape: LayoutShape::Map { item: 1 },
                },
                LayoutNode {
                    codec_bytes: b"sequence",
                    shape: LayoutShape::Sequence { item: 2 },
                },
                LayoutNode {
                    codec_bytes: b"decimal",
                    shape: LayoutShape::Scalar {
                        family: "decimal",
                        storage_representation: "json-string",
                    },
                },
            ],
        };
        let input = json!({"1.a[0]": ["-0.00","18446744073709551615"],"empty":[]});
        let mut calls = 0;
        let decoded = decode(
            &layout,
            &input,
            &mut budget(),
            |codec, family, representation, value| {
                assert_eq!(codec, b"decimal");
                assert_eq!(family, "decimal");
                assert_eq!(representation, "json-string");
                calls += 1;
                value
                    .as_str()
                    .ok_or_else(|| fail("Original string codec refuses numeric JSON"))?;
                Ok(value.clone())
            },
            |_, _| panic!("map has no record member absence"),
        )
        .unwrap();
        assert_eq!(decoded, input);
        assert_eq!(calls, 2);
        assert!(decode(
            &layout,
            &json!({"x":[42]}),
            &mut budget(),
            |_, _, _, v| Ok(v.clone()),
            |_, _| Ok(())
        )
        .is_err());
    }
    #[test]
    fn recursive_metadata_is_charged_per_runtime_occurrence_and_missing_members_are_explicit() {
        let field = json!({"documentId":"d","module":"m","element":"next"});
        let layout = Layout {
            root: 0,
            nodes: vec![
                LayoutNode {
                    codec_bytes: b"structured",
                    shape: LayoutShape::Structured { record: 1 },
                },
                LayoutNode {
                    codec_bytes: b"record",
                    shape: LayoutShape::Record {
                        members: vec![MemberSlot {
                            field_identity: &field,
                            stored_name: "0.next",
                            value_node: 0,
                            presence_bytes: b"optional-original",
                        }],
                    },
                },
            ],
        };
        let input = json!({"0.next":{"0.next":{}}});
        let mut work = budget();
        let mut absent_calls = 0;
        assert_eq!(
            decode(
                &layout,
                &input,
                &mut work,
                |_, _, _, _| panic!("no scalar"),
                |identity, presence| {
                    assert_eq!(identity, &field);
                    assert_eq!(presence, b"optional-original");
                    absent_calls += 1;
                    Ok(())
                }
            )
            .unwrap(),
            input
        );
        assert_eq!(work.remaining_nodes, 994);
        assert_eq!(work.remaining_members, 997);
        assert_eq!(absent_calls, 1);
        assert!(decode(
            &layout,
            &input,
            &mut budget(),
            |_, _, _, _| unreachable!(),
            |_, _| Err(fail("Required original member is absent"))
        )
        .is_err());
        assert!(decode(
            &layout,
            &json!({"unknown":{}}),
            &mut budget(),
            |_, _, _, _| unreachable!(),
            |_, _| Ok(())
        )
        .is_err());
        let mut limited = budget();
        limited.max_depth = 2;
        assert_eq!(
            decode(
                &layout,
                &input,
                &mut limited,
                |_, _, _, _| unreachable!(),
                |_, _| Ok(())
            )
            .unwrap_err()
            .code,
            "WFT-LIMIT"
        );
    }
    #[test]
    fn logical_record_names_and_presence_are_selected_from_original_descriptors() {
        use weft_core::{
            application_model::{Descriptor, Member, Shape},
            ir::{Family, Identity, LogicalType},
        };
        let identity = |element: &str| Identity {
            document_id: "d".into(),
            revision: "1".into(),
            module: "m".into(),
            element: element.into(),
        };
        let field = identity("field");
        let field_json = json!(field);
        let mut descriptors = vec![
            Descriptor {
                identity: identity("record"),
                availability: None,
                shape: Shape::Record {
                    members: vec![Member {
                        name: "authored.name".into(),
                        identity: field.clone(),
                    }],
                },
            },
            Descriptor {
                identity: field.clone(),
                availability: Some("absent-allowed".into()),
                shape: Shape::Scalar {
                    logical_type: LogicalType {
                        family: Family::String,
                        facets: json!({}),
                        nullable: false,
                    },
                },
            },
        ];
        let slot = MemberSlot {
            field_identity: &field_json,
            stored_name: "17",
            value_node: 0,
            presence_bytes: b"original-optional",
        };
        let mut procedure = |descriptor: &Descriptor, presence: &[u8], value: Option<Value>| {
            assert_eq!(descriptor.identity, field);
            assert_eq!(descriptor.availability.as_deref(), Some("absent-allowed"));
            assert_eq!(presence, b"original-optional");
            Ok(match value {
                Some(v) => json!({"state":"value","value":v}),
                None => json!({"state":"absent"}),
            })
        };
        assert_eq!(
            assemble_logical_record(
                &descriptors,
                vec![(&slot, None)],
                &mut budget(),
                &mut procedure
            )
            .unwrap(),
            json!({"authored.name":{"state":"absent"}})
        );
        assert_eq!(
            assemble_logical_record(
                &descriptors,
                vec![(&slot, Some(json!("é  ")))],
                &mut budget(),
                &mut procedure
            )
            .unwrap(),
            json!({"authored.name":{"state":"value","value":"é  "}})
        );
        let mut limited = budget();
        limited.remaining_key_bytes = 1;
        assert_eq!(
            assemble_logical_record(
                &descriptors,
                vec![(&slot, None)],
                &mut limited,
                &mut |_, _, _| panic!("budget must refuse before procedure")
            )
            .unwrap_err()
            .code,
            "WFT-LIMIT"
        );
        assert!(assemble_logical_record(
            &descriptors,
            vec![(&slot, None), (&slot, None)],
            &mut budget(),
            &mut |_, _, _| panic!("duplicate must refuse before procedure")
        )
        .is_err());
        descriptors.push(Descriptor {
            identity: identity("other-record"),
            availability: None,
            shape: Shape::Record {
                members: vec![Member {
                    name: "conflicting-name".into(),
                    identity: field,
                }],
            },
        });
        assert!(assemble_logical_record(
            &descriptors,
            vec![(&slot, None)],
            &mut budget(),
            &mut |_, _, _| panic!("ambiguous identity must refuse before procedure")
        )
        .is_err());
    }
    #[test]
    fn record_assembly_retains_original_order_identity_and_missing_slots() {
        let first = json!({"element":"authored-first"});
        let second = json!({"element":"authored-second"});
        let layout = Layout {
            root: 0,
            nodes: vec![
                LayoutNode {
                    codec_bytes: b"record",
                    shape: LayoutShape::Record {
                        members: vec![
                            MemberSlot {
                                field_identity: &first,
                                stored_name: "9.first",
                                value_node: 1,
                                presence_bytes: b"required-original",
                            },
                            MemberSlot {
                                field_identity: &second,
                                stored_name: "1.second",
                                value_node: 1,
                                presence_bytes: b"optional-original",
                            },
                        ],
                    },
                },
                LayoutNode {
                    codec_bytes: b"text",
                    shape: LayoutShape::Scalar {
                        family: "string",
                        storage_representation: "json-string",
                    },
                },
            ],
        };
        let decoded = decode_with_records(&layout, &json!({"9.first":"é  "}), &mut budget(),
            |_,_,_,value| Ok(value.clone()),
            |identity,presence| { assert_eq!(identity,&second); assert_eq!(presence,b"optional-original"); Ok(()) },
            |slots, _| {
                assert_eq!(slots.len(),2);
                assert_eq!(slots[0].0.field_identity,&first);
                assert_eq!(slots[0].0.presence_bytes,b"required-original");
                assert_eq!(slots[1].0.field_identity,&second);
                assert_eq!(slots[1].1,None);
                Ok(json!({"logical-first":slots[0].1.as_ref().unwrap(),"logical-second":{"state":"absent"}}))
            }).unwrap();
        assert_eq!(
            decoded,
            json!({"logical-first":"é  ","logical-second":{"state":"absent"}})
        );
        assert!(decode_with_records(
            &layout,
            &json!({"9.first":"x"}),
            &mut budget(),
            |_, _, _, v| Ok(v.clone()),
            |_, _| Ok(()),
            |_, _| Ok(json!([]))
        )
        .is_err());
    }
    #[test]
    fn nested_wide_containers_charge_pending_work_before_leaf_callbacks() {
        let layout = Layout {
            root: 0,
            nodes: vec![LayoutNode {
                codec_bytes: b"sequence",
                shape: LayoutShape::Sequence { item: 0 },
            }],
        };
        let value = json!([[[], [], []], [], []]);
        let mut limited = budget();
        limited.remaining_nodes = 6;
        let error = decode(
            &layout,
            &value,
            &mut limited,
            |_, _, _, _| panic!("no leaf"),
            |_, _| panic!("no record"),
        )
        .unwrap_err();
        assert_eq!(error.code, "WFT-LIMIT");
        // Root and its three queued children consume four slots. The first
        // nested three-child reservation refuses with only two slots left.
        assert_eq!(limited.remaining_nodes, 2);
        let mut exact = budget();
        exact.remaining_nodes = 7;
        assert_eq!(
            decode(
                &layout,
                &value,
                &mut exact,
                |_, _, _, _| panic!("no leaf"),
                |_, _| panic!("no record")
            )
            .unwrap(),
            value
        );
        assert_eq!(exact.remaining_nodes, 0);
    }
    #[test]
    fn wide_repeated_values_and_literal_key_work_refuse_without_partial_success() {
        let layout = Layout {
            root: 0,
            nodes: vec![
                LayoutNode {
                    codec_bytes: b"map",
                    shape: LayoutShape::Map { item: 1 },
                },
                LayoutNode {
                    codec_bytes: b"text",
                    shape: LayoutShape::Scalar {
                        family: "string",
                        storage_representation: "json-string",
                    },
                },
            ],
        };
        let input = json!({"long-literal-key":"","second":""});
        let mut limited = budget();
        limited.remaining_key_bytes = 1;
        assert_eq!(
            decode(
                &layout,
                &input,
                &mut limited,
                |_, _, _, v| Ok(v.clone()),
                |_, _| Ok(())
            )
            .unwrap_err()
            .code,
            "WFT-LIMIT"
        );
        let mut limited = budget();
        limited.remaining_nodes = 2;
        assert_eq!(
            decode(
                &layout,
                &input,
                &mut limited,
                |_, _, _, v| Ok(v.clone()),
                |_, _| Ok(())
            )
            .unwrap_err()
            .code,
            "WFT-LIMIT"
        );
    }
}
