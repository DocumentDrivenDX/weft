//! Bounded original-topology reconstruction from admitted native tree custody.
use crate::{
    row_custody::{TreeIndex, TreeRow},
    value_definition::{Layout, LayoutNode, LayoutShape, MemberSlot},
    value_traversal::Budget,
};
use serde_json::{Map, Value};
use weft_core::error::{Diagnostic, Result};
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-DECODE", "decode", message)
}
fn reserve(count: usize, budget: &mut Budget) -> Result<()> {
    budget.remaining_nodes = budget.remaining_nodes.checked_sub(count).ok_or_else(|| {
        Diagnostic::new(
            "WFT-LIMIT",
            "decode",
            "Native value child reservation exhausted",
        )
    })?;
    Ok(())
}
/// Reconstruct an intermediate storage body, never a public logical envelope.
/// Procedures receive original codec/presence/identity metadata and every native
/// definition/source byte. Field-identity encoding is explicitly selected.
pub fn decode(
    layout: &Layout<'_>,
    tree: &TreeIndex<'_>,
    budget: &mut Budget,
    mut observe: impl FnMut(&LayoutNode<'_>, &TreeRow) -> Result<()>,
    mut scalar: impl FnMut(&LayoutNode<'_>, &TreeRow) -> Result<Value>,
    mut field: impl FnMut(&[u8], &Value) -> Result<bool>,
    mut absent: impl FnMut(&MemberSlot<'_>) -> Result<()>,
) -> Result<Value> {
    enum Frame<'a> {
        Enter(usize, &'a TreeRow, usize),
        Array(usize),
        Object(Vec<&'a str>),
    }
    reserve(1, budget)?;
    let mut pending = vec![Frame::Enter(layout.root, tree.root, 0)];
    let mut values = Vec::new();
    while let Some(frame) = pending.pop() {
        match frame {
            Frame::Enter(index, row, depth) => {
                if depth > budget.max_depth {
                    return Err(Diagnostic::new(
                        "WFT-LIMIT",
                        "decode",
                        "Native value depth exhausted",
                    ));
                }
                let node = layout
                    .nodes
                    .get(index)
                    .ok_or_else(|| fail("Native value references missing original node"))?;
                observe(node, row)?;
                let kind = row.cells[3].as_deref();
                let children = tree
                    .children
                    .get(
                        row.cells[1]
                            .as_deref()
                            .ok_or_else(|| fail("Native node identity missing"))?,
                    )
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                if matches!(node.shape, LayoutShape::Scalar { .. }) {
                    if !matches!(kind, Some("scalar" | "null")) || !children.is_empty() {
                        return Err(fail("Native scalar differs from original shape"));
                    }
                    let value = scalar(node, row)?;
                    if !matches!(value, Value::String(_) | Value::Bool(_) | Value::Null) {
                        return Err(fail(
                            "Native leaf procedure returned an inexact number/container",
                        ));
                    }
                    values.push(value);
                    continue;
                }
                let shape = if let LayoutShape::Structured { record } = node.shape {
                    if kind != Some("structured") {
                        return Err(fail("Native structured kind differs"));
                    }
                    let record = layout
                        .nodes
                        .get(record)
                        .ok_or_else(|| fail("Native structured record missing"))?;
                    // Record interpretation is separately observed on the same
                    // physical container; metadata references add no native row.
                    observe(record, row)?;
                    &record.shape
                } else {
                    &node.shape
                };
                match shape {
                    LayoutShape::Sequence { item } => {
                        if kind != Some("sequence") {
                            return Err(fail("Native sequence kind differs"));
                        }
                        reserve(children.len(), budget)?;
                        pending.push(Frame::Array(children.len()));
                        for child in children.iter().rev() {
                            pending.push(Frame::Enter(*item, child, depth + 1));
                        }
                    }
                    LayoutShape::Map { item } => {
                        if kind != Some("map") {
                            return Err(fail("Native map kind differs"));
                        }
                        let names: Vec<_> = children
                            .iter()
                            .map(|row| {
                                row.cells[6]
                                    .as_deref()
                                    .ok_or_else(|| fail("Native map key missing"))
                            })
                            .collect::<Result<_>>()?;
                        reserve_keys(&names, budget)?;
                        reserve(children.len(), budget)?;
                        pending.push(Frame::Object(names));
                        for child in children.iter().rev() {
                            pending.push(Frame::Enter(*item, child, depth + 1));
                        }
                    }
                    LayoutShape::Record { members } => {
                        if !matches!(kind, Some("structured" | "record")) {
                            return Err(fail("Native record kind differs"));
                        }
                        let work = members
                            .len()
                            .checked_mul(children.len())
                            .and_then(|n| n.checked_add(members.len()))
                            .ok_or_else(|| fail("Native member work overflow"))?;
                        if work > budget.remaining_members {
                            return Err(Diagnostic::new(
                                "WFT-LIMIT",
                                "decode",
                                "Native member correspondence budget exhausted",
                            ));
                        }
                        budget.remaining_members -= work;
                        let mut selected = std::collections::BTreeMap::new();
                        for child in children {
                            let bytes = child
                                .bytes
                                .get(&7)
                                .ok_or_else(|| fail("Native record identity bytes missing"))?;
                            let mut matched = None;
                            for (i, member) in members.iter().enumerate() {
                                if field(bytes, member.field_identity)? {
                                    if matched.replace(i).is_some() {
                                        return Err(fail("Native field identity is ambiguous"));
                                    }
                                }
                            }
                            let i = matched.ok_or_else(|| {
                                fail("Native record field is unknown to original model")
                            })?;
                            if selected.insert(i, *child).is_some() {
                                return Err(fail(
                                    "Native slots decode to one repeated original field",
                                ));
                            }
                        }
                        let mut ordered = Vec::new();
                        for (i, member) in members.iter().enumerate() {
                            if let Some(child) = selected.get(&i) {
                                ordered.push((member.stored_name, member.value_node, *child));
                            } else {
                                absent(member)?;
                            }
                        }
                        let names: Vec<_> = ordered.iter().map(|(name, _, _)| *name).collect();
                        reserve_keys(&names, budget)?;
                        reserve(ordered.len(), budget)?;
                        pending.push(Frame::Object(names));
                        for (_, i, child) in ordered.into_iter().rev() {
                            pending.push(Frame::Enter(i, child, depth + 1));
                        }
                    }
                    _ => return Err(fail("Original native container topology unsupported")),
                }
            }
            Frame::Array(count) => {
                let start = values
                    .len()
                    .checked_sub(count)
                    .ok_or_else(|| fail("Incomplete native sequence"))?;
                let children = values.split_off(start);
                values.push(Value::Array(children));
            }
            Frame::Object(names) => {
                let start = values
                    .len()
                    .checked_sub(names.len())
                    .ok_or_else(|| fail("Incomplete native object"))?;
                let children = values.split_off(start);
                let mut object = Map::new();
                for (name, value) in names.into_iter().zip(children) {
                    if object.insert(name.into(), value).is_some() {
                        return Err(fail("Native object repeats a storage key"));
                    }
                }
                values.push(Value::Object(object));
            }
        }
    }
    if values.len() != 1 {
        return Err(fail("Native reconstruction is incomplete"));
    }
    Ok(values.pop().unwrap())
}
fn reserve_keys(names: &[&str], budget: &mut Budget) -> Result<()> {
    let bytes = names
        .iter()
        .try_fold(0usize, |n, key| n.checked_add(key.len()))
        .ok_or_else(|| fail("Native key size overflow"))?;
    if bytes > budget.remaining_key_bytes {
        return Err(Diagnostic::new(
            "WFT-LIMIT",
            "decode",
            "Native output key budget exhausted",
        ));
    }
    budget.remaining_key_bytes -= bytes;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row_custody::{admit_tree_rows, index_tree, TreeBudget};
    fn budget() -> Budget {
        Budget {
            remaining_nodes: 100,
            remaining_key_bytes: 100,
            remaining_members: 100,
            max_depth: 32,
        }
    }
    fn rows() -> Vec<TreeRow> {
        let receipt: Value = serde_json::from_str(include_str!(
            "../../../docs/helix/04-build/evidence/B-005-row-tree-custody-native.json"
        ))
        .unwrap();
        let case = receipt["results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case"] == "descendant")
            .unwrap();
        let cells: Vec<Vec<Option<&str>>> = case["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row.as_array().unwrap().iter().map(Value::as_str).collect())
            .collect();
        admit_tree_rows(
            &cells,
            &mut crate::row_custody::Budget {
                remaining_bytes: 100000,
                remaining_cells: 10000,
            },
        )
        .unwrap()
    }
    fn leaf() -> LayoutNode<'static> {
        LayoutNode {
            codec_bytes: b"original-leaf",
            shape: LayoutShape::Scalar {
                family: "integer",
                storage_representation: "native",
            },
        }
    }
    #[test]
    fn recorded_native_sequence_preserves_exact_token_and_source_observations() {
        let rows = rows();
        let tree = index_tree(
            &rows,
            "1",
            "10",
            &mut TreeBudget {
                remaining_nodes: 100,
                max_depth: 32,
            },
        )
        .unwrap();
        let layout = Layout {
            root: 0,
            nodes: vec![
                LayoutNode {
                    codec_bytes: b"original-sequence",
                    shape: LayoutShape::Sequence { item: 1 },
                },
                leaf(),
            ],
        };
        let mut seen = Vec::new();
        let value = decode(
            &layout,
            &tree,
            &mut budget(),
            |node, row| {
                seen.push((node.codec_bytes.to_vec(), row.bytes[&9].clone()));
                Ok(())
            },
            |_, row| Ok(Value::String(row.cells[16].clone().unwrap())),
            |_, _| panic!(),
            |_| panic!(),
        )
        .unwrap();
        assert_eq!(value, serde_json::json!(["18446744073709551615"]));
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].0, b"original-sequence");
        let mut limited = budget();
        limited.remaining_nodes = 1;
        assert!(decode(
            &layout,
            &tree,
            &mut limited,
            |_, _| Ok(()),
            |_, _| panic!("unreserved leaf executed"),
            |_, _| panic!(),
            |_| panic!()
        )
        .is_err());
        assert!(decode(
            &layout,
            &tree,
            &mut budget(),
            |_, _| Ok(()),
            |_, _| Ok(serde_json::json!(42)),
            |_, _| panic!(),
            |_| panic!()
        )
        .is_err());
    }
    #[test]
    fn native_record_uses_selected_identity_and_original_absence_procedures() {
        let mut rows = rows();
        rows[0].cells[3] = Some("structured".into());
        rows[1].cells[4] = Some("record".into());
        rows[1].cells[5] = None;
        rows[1].cells[7] = Some("61".into());
        rows[1].bytes.insert(7, b"a".to_vec());
        let tree = index_tree(
            &rows,
            "1",
            "10",
            &mut TreeBudget {
                remaining_nodes: 100,
                max_depth: 32,
            },
        )
        .unwrap();
        let a = serde_json::json!("field-a");
        let b = serde_json::json!("field-b");
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
                        members: vec![
                            MemberSlot {
                                field_identity: &a,
                                stored_name: "stored-a",
                                value_node: 2,
                                presence_bytes: b"required",
                            },
                            MemberSlot {
                                field_identity: &b,
                                stored_name: "stored-b",
                                value_node: 2,
                                presence_bytes: b"optional",
                            },
                        ],
                    },
                },
                leaf(),
            ],
        };
        let mut observed = 0;
        let mut missing = 0;
        let result = decode(
            &layout,
            &tree,
            &mut budget(),
            |_, _| {
                observed += 1;
                Ok(())
            },
            |_, row| Ok(Value::String(row.cells[16].clone().unwrap())),
            |bytes, id| Ok(bytes == b"a" && id == &a),
            |member| {
                assert_eq!(member.presence_bytes, b"optional");
                assert_eq!(member.field_identity, &b);
                missing += 1;
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(
            result,
            serde_json::json!({"stored-a":"18446744073709551615"})
        );
        assert_eq!((observed, missing), (3, 1));
        assert!(decode(
            &layout,
            &tree,
            &mut budget(),
            |_, _| Ok(()),
            |_, _| panic!(),
            |_, _| Ok(true),
            |_| Ok(())
        )
        .is_err());
        assert!(decode(
            &layout,
            &tree,
            &mut budget(),
            |_, _| Ok(()),
            |_, _| panic!(),
            |_, _| Ok(false),
            |_| Ok(())
        )
        .is_err());
        let mut limited = budget();
        limited.remaining_members = 3;
        assert!(decode(
            &layout,
            &tree,
            &mut limited,
            |_, _| Ok(()),
            |_, _| panic!(),
            |_, _| panic!("unreserved identity executed"),
            |_| Ok(())
        )
        .is_err());
    }
}
