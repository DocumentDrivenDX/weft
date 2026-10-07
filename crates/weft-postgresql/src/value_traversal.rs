//! Bounded traversal of runtime JSONB values under original finite UMF topology.
//! Leaf and member presence procedures remain explicitly selected callbacks.
use crate::value_definition::{Layout, LayoutShape};
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
/// Normalize a complete recursive value body, preserving literal stored names.
/// Every runtime occurrence is visited even when metadata nodes are shared/cyclic.
/// Scalar callbacks own original codec/type/facet/null semantics and allocation
/// bounds. Presence callbacks decide each missing record member from original
/// field identity, presence bytes and UMF descriptor; no default is inferred.
pub fn decode(
    layout: &Layout<'_>,
    value: &Value,
    budget: &mut Budget,
    mut scalar: impl FnMut(&[u8], &str, &str, &Value) -> Result<Value>,
    mut absent: impl FnMut(&Value, &[u8]) -> Result<()>,
) -> Result<Value> {
    enum Frame<'a> {
        Enter(usize, &'a Value, usize),
        Array(usize),
        Object(Vec<&'a str>),
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
                        pending.push(Frame::Object(
                            children.iter().map(|(name, _, _)| *name).collect(),
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
