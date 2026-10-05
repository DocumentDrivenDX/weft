use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{value::RawValue, Value};
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
fn inspect_json(raw: &str, depth: usize, count: &mut usize) -> RawResult<()> {
    if depth > 128 || *count >= 100_000 {
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
                inspect_json(value.get(), depth + 1, count)?;
            }
        }
        Some(b'[') => {
            let values: Vec<Box<RawValue>> = serde_json::from_str(raw).map_err(|_| "WFT-INPUT")?;
            for value in values {
                inspect_json(value.get(), depth + 1, count)?;
            }
        }
        _ => {
            let _: Value = serde_json::from_str(raw).map_err(|_| "WFT-INPUT")?;
        }
    }
    Ok(())
}
pub fn checked_json(raw: &str) -> RawResult<Value> {
    inspect_json(raw, 0, &mut 0)?;
    serde_json::from_str(raw).map_err(|_| "WFT-INPUT")
}
