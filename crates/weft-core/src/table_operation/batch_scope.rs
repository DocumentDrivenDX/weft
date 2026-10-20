// SPDX-License-Identifier: MIT OR Apache-2.0
//! Source-derived safety selection for set-based inserts.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

type RecordRef = (String, String);
type RelationshipRef = (String, String);
type ForeignKeyEdge = (RecordRef, RecordRef);

fn record_ref(value: &Value) -> Option<RecordRef> {
    Some((
        value["module"].as_str()?.to_owned(),
        value["element"].as_str()?.to_owned(),
    ))
}

fn relationship_ref(value: &Value) -> Option<RelationshipRef> {
    Some((
        value["module"].as_str()?.to_owned(),
        value["id"].as_str()?.to_owned(),
    ))
}

fn visit_rule(
    rule: &Value,
    current: &RecordRef,
    tables: &BTreeMap<RecordRef, &Value>,
    foreign_keys: &BTreeMap<RelationshipRef, Vec<ForeignKeyEdge>>,
    edges: &mut BTreeMap<RecordRef, BTreeSet<RecordRef>>,
) -> bool {
    match rule["kind"].as_str() {
        Some("allow" | "deny" | "owner" | "memberOf" | "memberOfRole") => true,
        Some("any" | "all") => rule["rules"].as_array().is_some_and(|rules| {
            rules
                .iter()
                .all(|child| visit_rule(child, current, tables, foreign_keys, edges))
        }),
        Some("related") => {
            let Some(destination) = record_ref(&rule["record"]) else {
                return false;
            };
            if !tables.contains_key(&destination) {
                return false;
            }
            edges
                .entry(current.clone())
                .or_default()
                .insert(destination.clone());
            visit_rule(&rule["rule"], &destination, tables, foreign_keys, edges)
        }
        Some("via") => {
            let Some(path) = rule["path"].as_array() else {
                return false;
            };
            let mut destination = current.clone();
            for step in path {
                let Some(reference) = relationship_ref(step) else {
                    return false;
                };
                let Some(matches) = foreign_keys.get(&reference) else {
                    return false;
                };
                if matches.len() != 1 {
                    return false;
                }
                let (source, target) = &matches[0];
                destination = match step["direction"].as_str() {
                    Some("forward") if &destination == source => target.clone(),
                    Some("inverse") if &destination == target => source.clone(),
                    _ => return false,
                };
                if !tables.contains_key(&destination) {
                    return false;
                }
                edges
                    .entry(current.clone())
                    .or_default()
                    .insert(destination.clone());
            }
            visit_rule(&rule["rule"], &destination, tables, foreign_keys, edges)
        }
        _ => false,
    }
}

/// Reject self-FKs and any admitted policy dependency path that returns to the
/// inserted Record. Unknown policy steps are conservatively ineligible.
pub(crate) fn independent_insert_scopes(payload: &Value) -> BTreeMap<String, bool> {
    let Some(declarations) = payload["tables"].as_array() else {
        return BTreeMap::new();
    };
    let mut tables = BTreeMap::new();
    let mut table_records = BTreeMap::new();
    let mut self_foreign_keys = BTreeSet::new();
    let mut foreign_keys: BTreeMap<RelationshipRef, Vec<ForeignKeyEdge>> = BTreeMap::new();
    let mut valid = true;
    for table in declarations {
        let Some(record) = record_ref(&table["record"]) else {
            valid = false;
            continue;
        };
        if tables.insert(record.clone(), table).is_some() {
            valid = false;
        }
        if let Some(id) = table["id"].as_str() { table_records.entry(id.to_owned()).or_insert(record.clone()); }
        let Some(keys) = table["foreignKeys"].as_array() else {
            valid = false;
            continue;
        };
        for key in keys {
            let Some(target) = record_ref(&key["target"]["record"]) else {
                valid = false;
                continue;
            };
            if record == target {
                self_foreign_keys.insert(record.clone());
            }
            if let Some(reference) = key.get("relationship") {
                let Some(reference) = relationship_ref(reference) else {
                    valid = false;
                    continue;
                };
                foreign_keys
                    .entry(reference)
                    .or_default()
                    .push((record.clone(), target));
            }
        }
    }
    let mut edges: BTreeMap<RecordRef, BTreeSet<RecordRef>> = BTreeMap::new();
    for (record, table) in &tables {
        for audience in ["person", "batch"] {
            for action in ["select", "insert", "update", "delete"] {
                if !visit_rule(
                    &table["policy"][audience][action],
                    record,
                    &tables,
                    &foreign_keys,
                    &mut edges,
                ) {
                    valid = false;
                }
            }
        }
    }

    let mut reverse = BTreeMap::<RecordRef, BTreeSet<RecordRef>>::new();
    for record in tables.keys() { reverse.entry(record.clone()).or_default(); }
    for (from, destinations) in &edges {
        for destination in destinations { reverse.entry(destination.clone()).or_default().insert(from.clone()); }
    }
    // Iterative Kosaraju pass avoids recursion on source-controlled graph size.
    let mut visited = BTreeSet::new();
    let mut finish = Vec::with_capacity(tables.len());
    for root in tables.keys() {
        if visited.contains(root) { continue; }
        let mut stack = vec![(root.clone(), false)];
        while let Some((node, exiting)) = stack.pop() {
            if exiting { finish.push(node); continue; }
            if !visited.insert(node.clone()) { continue; }
            stack.push((node.clone(), true));
            if let Some(next) = edges.get(&node) {
                for destination in next.iter().rev() {
                    if !visited.contains(destination) { stack.push((destination.clone(), false)); }
                }
            }
        }
    }
    let mut component = BTreeMap::<RecordRef, usize>::new();
    let mut component_sizes = Vec::new();
    for root in finish.into_iter().rev() {
        if component.contains_key(&root) { continue; }
        let component_id = component_sizes.len();
        let mut size = 0;
        let mut stack = vec![root.clone()];
        component.insert(root, component_id);
        while let Some(node) = stack.pop() {
            size += 1;
            if let Some(next) = reverse.get(&node) {
                for predecessor in next {
                    if !component.contains_key(predecessor) {
                        component.insert(predecessor.clone(), component_id);
                        stack.push(predecessor.clone());
                    }
                }
            }
        }
        component_sizes.push(size);
    }
    let cyclic: BTreeSet<_> = tables.keys().filter(|record| {
        let id = component.get(*record).copied().unwrap_or(usize::MAX);
        component_sizes.get(id).is_some_and(|size| *size > 1)
            || edges.get(*record).is_some_and(|next| next.contains(*record))
    }).cloned().collect();
    table_records.into_iter().map(|(id, record)| {
        let eligible = valid && !self_foreign_keys.contains(&record) && !cyclic.contains(&record);
        (id, eligible)
    }).collect()
}

#[cfg(test)]
fn independent_insert_scope(payload: &Value, selected_record_value: &Value, selected_table_id: &Value) -> bool {
    let Some(record) = record_ref(selected_record_value) else { return false; };
    independent_insert_scopes(payload).get(selected_table_id.as_str().unwrap_or_default()).copied().unwrap_or(false)
        && payload["tables"].as_array().is_some_and(|tables| tables.iter().any(|table| table["id"] == *selected_table_id && record_ref(&table["record"]).as_ref() == Some(&record)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy(rule: Value) -> Value {
        json!({"mode":"native-rls","person":{"select":rule,"insert":{"kind":"allow"},"update":{"kind":"allow"},"delete":{"kind":"allow"}},"batch":{"select":{"kind":"allow"},"insert":{"kind":"allow"},"update":{"kind":"allow"},"delete":{"kind":"allow"}}})
    }

    fn table(id: &str, record: &str, policy: Value, foreign_keys: Value) -> Value {
        json!({"id":id,"record":{"module":"m","element":record},"foreignKeys":foreign_keys,"policy":policy})
    }

    fn eligible(tables: Value) -> bool {
        let payload = json!({"tables":tables});
        independent_insert_scope(
            &payload,
            &json!({"module":"m","element":"a"}),
            &json!("a-table"),
        )
    }

    #[test]
    fn only_dependency_cycles_and_selected_self_foreign_keys_disable_set_inserts() {
        let plain = vec![
            table(
                "a-table",
                "a",
                policy(json!({"kind":"owner","field":{"module":"m","element":"owner"}})),
                json!([]),
            ),
            table(
                "b-table",
                "b",
                policy(json!({"kind":"owner","field":{"module":"m","element":"owner"}})),
                json!([]),
            ),
        ];
        assert!(eligible(json!(plain)));

        let self_fk = json!([{"id":"a-table","record":{"module":"m","element":"a"},"foreignKeys":[{"target":{"record":{"module":"m","element":"a"}}}],"policy":policy(json!({"kind":"allow"}))}]);
        assert!(!eligible(self_fk));

        let cycle = vec![
            table(
                "a-table",
                "a",
                policy(
                    json!({"kind":"related","record":{"module":"m","element":"b"},"matches":[{"source":{"module":"m","element":"id"},"target":{"module":"m","element":"id"}}],"rule":{"kind":"allow"}}),
                ),
                json!([]),
            ),
            table(
                "b-table",
                "b",
                policy(
                    json!({"kind":"related","record":{"module":"m","element":"a"},"matches":[{"source":{"module":"m","element":"id"},"target":{"module":"m","element":"id"}}],"rule":{"kind":"allow"}}),
                ),
                json!([]),
            ),
        ];
        assert!(!eligible(json!(cycle)));
    }
}
