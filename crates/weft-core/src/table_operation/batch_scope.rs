// SPDX-License-Identifier: MIT OR Apache-2.0
//! Source-derived safety selection for set-based inserts.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

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
pub(crate) fn independent_insert_scope(
    payload: &Value,
    selected_record_value: &Value,
    selected_table_id: &Value,
) -> bool {
    let Some(selected) = record_ref(selected_record_value) else {
        return false;
    };
    let Some(declarations) = payload["tables"].as_array() else {
        return false;
    };
    let mut tables = BTreeMap::new();
    let mut foreign_keys: BTreeMap<RelationshipRef, Vec<ForeignKeyEdge>> = BTreeMap::new();
    for table in declarations {
        let Some(record) = record_ref(&table["record"]) else {
            return false;
        };
        if tables.insert(record.clone(), table).is_some() {
            return false;
        }
        let Some(keys) = table["foreignKeys"].as_array() else {
            return false;
        };
        for key in keys {
            let Some(target) = record_ref(&key["target"]["record"]) else {
                return false;
            };
            if record == selected && target == selected {
                return false;
            }
            if let Some(reference) = key.get("relationship") {
                let Some(reference) = relationship_ref(reference) else {
                    return false;
                };
                foreign_keys
                    .entry(reference)
                    .or_default()
                    .push((record.clone(), target));
            }
        }
    }
    if !tables.contains_key(&selected) {
        return false;
    }
    // The ID ties this check to the exact selected declaration, not a logical
    // name supplied independently by the request.
    if !tables
        .get(&selected)
        .is_some_and(|table| table["id"] == *selected_table_id)
    {
        return false;
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
                    return false;
                }
            }
        }
    }

    let mut pending = VecDeque::from([selected.clone()]);
    let mut visited = BTreeSet::new();
    while let Some(current) = pending.pop_front() {
        if !visited.insert(current.clone()) {
            continue;
        }
        if let Some(next) = edges.get(&current) {
            for destination in next {
                if destination == &selected {
                    return false;
                }
                pending.push_back(destination.clone());
            }
        }
    }
    true
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
