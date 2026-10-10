use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Value, value::RawValue};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt};
type RawResult<T> = std::result::Result<T, &'static str>;
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
struct RawObject(Vec<(String, Box<RawValue>)>);
impl<'de> Deserialize<'de> for RawObject {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct ObjectVisitor;
        impl<'de> Visitor<'de> for ObjectVisitor {
            type Value = RawObject;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an object")
            }
            fn visit_map<M: MapAccess<'de>>(
                self,
                mut map: M,
            ) -> std::result::Result<Self::Value, M::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, Box<RawValue>>()? {
                    entries.push(entry);
                }
                Ok(RawObject(entries))
            }
        }
        d.deserialize_map(ObjectVisitor)
    }
}
// Traverse raw nested values so even arbitrary-size unknown numbers are never f64.
fn inspect_json(
    raw: &str,
    depth: usize,
    count: &mut usize,
    max_depth: usize,
    max_nodes: usize,
) -> RawResult<()> {
    if depth > max_depth || *count >= max_nodes {
        return Err("WFT-LIMIT");
    }
    *count += 1;
    match raw.trim_start().as_bytes().first() {
        Some(b'{') => {
            let object: RawObject = serde_json::from_str(raw).map_err(|_| "WFT-INPUT")?;
            let mut keys = BTreeSet::new();
            for (key, value) in object.0 {
                if !keys.insert(key) {
                    return Err("WFT-JSON-DUPLICATE");
                }
                inspect_json(value.get(), depth + 1, count, max_depth, max_nodes)?;
            }
        }
        Some(b'[') => {
            let values: Vec<Box<RawValue>> = serde_json::from_str(raw).map_err(|_| "WFT-INPUT")?;
            for value in values {
                inspect_json(value.get(), depth + 1, count, max_depth, max_nodes)?;
            }
        }
        _ => {
            let _: Value = serde_json::from_str(raw).map_err(|_| "WFT-INPUT")?;
        }
    }
    Ok(())
}
// Value's arbitrary-precision/raw-value visitor reserves ordinary object keys.
// Construct objects ourselves so unknown JSON members remain ordinary data.
fn faithful_tree(raw: &str) -> RawResult<Value> {
    match raw.trim_start().as_bytes().first() {
        Some(b'{') => {
            let object: RawObject = serde_json::from_str(raw).map_err(|_| "WFT-INPUT")?;
            let mut out = serde_json::Map::new();
            for (key, value) in object.0 {
                out.insert(key, faithful_tree(value.get())?);
            }
            Ok(Value::Object(out))
        }
        Some(b'[') => {
            let values: Vec<Box<RawValue>> = serde_json::from_str(raw).map_err(|_| "WFT-INPUT")?;
            values
                .iter()
                .map(|v| faithful_tree(v.get()))
                .collect::<RawResult<Vec<_>>>()
                .map(Value::Array)
        }
        _ => serde_json::from_str(raw).map_err(|_| "WFT-INPUT"),
    }
}
pub fn checked_json(raw: &str) -> RawResult<Value> {
    inspect_json(raw, 0, &mut 0, 128, 100_000)?;
    faithful_tree(raw)
}

/// Bounded transport reader; root depth is one and each value counts once.
/// Parsing is custody only and grants no result release authority.
#[allow(dead_code)] // Pending runtime-cell consumer.
pub(crate) fn checked_json_bounded(
    raw: &str,
    max_bytes: usize,
    max_depth: usize,
    max_nodes: usize,
) -> RawResult<Value> {
    if raw.len() > max_bytes {
        return Err("WFT-LIMIT");
    }
    inspect_json(raw, 1, &mut 0, max_depth, max_nodes)?;
    faithful_tree(raw)
}
/// Same bounded reader with parse visits consumed from the caller's work ledger.
/// Raw inspection temporaries are separately bounded by input bytes and depth;
/// this is not a process-memory bound.
pub(crate) fn checked_json_bounded_charged(
    raw: &str,
    max_bytes: usize,
    max_depth: usize,
    remaining_work: &mut usize,
) -> RawResult<Value> {
    if raw.len() > max_bytes {
        return Err("WFT-LIMIT");
    }
    let mut count = 0;
    inspect_json(raw, 1, &mut count, max_depth, *remaining_work)?;
    *remaining_work = remaining_work.checked_sub(count).ok_or("WFT-LIMIT")?;
    faithful_tree(raw)
}
#[cfg(test)]
mod bounded_transport_tests {
    use super::checked_json_bounded_charged;
    #[test]
    fn charged_reader_uses_the_callers_actual_remaining_work() {
        let raw = "{\"a\":[1,2]}";
        let mut work = 4;
        assert!(checked_json_bounded_charged(raw, raw.len(), 3, &mut work).is_ok());
        assert_eq!(work, 0);
        let mut work = 3;
        assert!(checked_json_bounded_charged(raw, raw.len(), 3, &mut work).is_err());
        assert_eq!(work, 3);
    }
    use super::*;
    #[test]
    fn ordinary_reserved_looking_objects_survive_every_reader() {
        for key in [
            "$serde_json::private::Number",
            "$serde_json::private::RawValue",
        ] {
            let raw = format!("{{\"{key}\":\"not JSON\",\"sibling\":[1.00e0,true]}}");
            let mut map = serde_json::Map::new();
            map.insert(key.into(), Value::String("not JSON".into()));
            map.insert(
                "sibling".into(),
                Value::Array(vec![
                    serde_json::from_str("1.00e0").unwrap(),
                    Value::Bool(true),
                ]),
            );
            let expected = Value::Object(map);
            assert_eq!(checked_json(&raw).unwrap(), expected);
            assert_eq!(
                checked_json_bounded(&raw, raw.len(), 3, 5).unwrap(),
                expected
            );
            let mut work = 5;
            assert_eq!(
                checked_json_bounded_charged(&raw, raw.len(), 3, &mut work).unwrap(),
                expected
            );
            assert_eq!(work, 0);
        }
    }
    #[test]
    fn byte_limits_count_utf8_and_whitespace() {
        let raw = " \"é\" ";
        assert!(checked_json_bounded(raw, raw.len(), 1, 1).is_ok());
        assert_eq!(
            checked_json_bounded(raw, raw.len() - 1, 1, 1),
            Err("WFT-LIMIT")
        );
    }
    #[test]
    fn depth_counts_root_and_nodes_count_values_not_names() {
        let raw = r#"{"rows":[[null,true]]}"#;
        assert!(checked_json_bounded(raw, raw.len(), 4, 5).is_ok());
        assert_eq!(checked_json_bounded(raw, raw.len(), 3, 5), Err("WFT-LIMIT"));
        assert_eq!(checked_json_bounded(raw, raw.len(), 4, 4), Err("WFT-LIMIT"));
        assert_eq!(checked_json_bounded("null", 4, 0, 1), Err("WFT-LIMIT"));
        assert_eq!(checked_json_bounded("null", 4, 1, 0), Err("WFT-LIMIT"));
    }
    #[test]
    fn duplicate_members_and_malformed_json_refuse() {
        for raw in [r#"{"x":1,"x":2}"#, r#"[{"x":1,"\u0078":2}]"#] {
            assert_eq!(
                checked_json_bounded(raw, raw.len(), 64, 1_000_000),
                Err("WFT-JSON-DUPLICATE")
            );
        }
        assert_eq!(
            checked_json_bounded("[null,]", 100, 64, 100),
            Err("WFT-INPUT")
        );
    }
}
