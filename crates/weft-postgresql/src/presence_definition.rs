//! Original Truss presence-definition interpretation; not execution authority.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use weft_core::{
    error::{Diagnostic, Result},
    json::{checked_json, sha256},
};
#[derive(Debug)]
pub struct Definition {
    pub original_json: String,
    pub accepted_definition: Vec<u8>,
}
#[derive(Debug, PartialEq)]
pub enum Presence<'a> {
    Absent,
    Present(&'a Value),
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
impl Definition {
    pub fn parse(raw: &str, profile: &Value, accepted: &[u8]) -> Result<Self> {
        if raw.len() > 4 * 1024 * 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "binding",
                "Presence definition exceeds candidate byte bound",
            ));
        }
        let value = checked_json(raw)
            .map_err(|_| fail("Malformed or duplicate-member presence definition"))?;
        let schema: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/upstream/presence-definition.schema.json"
        ))
        .map_err(|_| fail("Pinned presence grammar is unavailable"))?;
        let object = value
            .as_object()
            .ok_or_else(|| fail("Presence definition must be an object"))?;
        let properties = schema["properties"].as_object().unwrap();
        if object.len() != properties.len()
            || object.keys().any(|key| !properties.contains_key(key))
        {
            return Err(fail("Presence definition has missing or unknown members"));
        }
        for (key, rule) in properties {
            if let Some(constant) = rule.get("const") {
                if &value[key] != constant {
                    return Err(fail("Unsupported original presence interpretation"));
                }
            }
        }
        if &value["profile"] != profile {
            return Err(fail("Presence profile differs from registered selection"));
        }
        let pin = value["profile"]
            .as_object()
            .ok_or_else(|| fail("Presence profile is malformed"))?;
        if pin.len() != 3
            || ["identity", "version", "sha256"].iter().any(|key| {
                pin.get(*key)
                    .and_then(Value::as_str)
                    .is_none_or(str::is_empty)
            })
        {
            return Err(fail("Presence profile pin is malformed"));
        }
        let digest = pin["sha256"].as_str().unwrap();
        if digest.len() != 64
            || !digest
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err(fail("Presence profile digest is malformed"));
        }
        let artifact = value["acceptedDefinition"]
            .as_object()
            .ok_or_else(|| fail("Original accepted definition is malformed"))?;
        if artifact.len() != 3
            || artifact
                .get("identity")
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
        {
            return Err(fail("Original accepted definition fields differ"));
        }
        let encoded = artifact
            .get("bytesBase64")
            .and_then(Value::as_str)
            .ok_or_else(|| fail("Original accepted bytes are missing"))?;
        let bytes = STANDARD
            .decode(encoded)
            .map_err(|_| fail("Original accepted bytes are invalid base64"))?;
        if STANDARD.encode(&bytes) != encoded
            || artifact.get("sha256") != Some(&Value::String(sha256(&bytes)))
            || bytes != accepted
        {
            return Err(fail("Original accepted definition custody differs"));
        }
        Ok(Self {
            original_json: raw.into(),
            accepted_definition: bytes,
        })
    }
    pub fn observe<'a>(
        &self,
        root: Option<&'a Value>,
        member: &str,
        authored_nullable: bool,
    ) -> Result<Presence<'a>> {
        let object = root.and_then(Value::as_object).ok_or_else(|| {
            Diagnostic::new(
                "WFT-OBLIGATION",
                "decode",
                "Storage root is not a nonnull object",
            )
        })?;
        match object.get(member) {
            None => Ok(Presence::Absent),
            Some(v) if v.is_null() => Err(Diagnostic::new(
                if authored_nullable {
                    "WFT-CAPABILITY"
                } else {
                    "WFT-OBLIGATION"
                },
                "decode",
                "JSON null requires separately qualified authored/native-null semantics",
            )),
            Some(v) => Ok(Presence::Present(v)),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn fixture() -> Value {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/upstream/presence-definition.schema.json"
        ))
        .unwrap();
        let mut value = json!({});
        for (key, rule) in schema["properties"].as_object().unwrap() {
            if let Some(constant) = rule.get("const") {
                value[key] = constant.clone();
            }
        }
        value["profile"] =
            json!({"identity":"presence-fixture","version":"0.1.0","sha256":sha256(b"{}")});
        value["acceptedDefinition"] = json!({"identity":"authored-field","bytesBase64":STANDARD.encode(b"{}"),"sha256":sha256(b"{}")});
        value
    }
    #[test]
    fn original_definition_drives_presence_without_coercion() {
        let v = fixture();
        let raw = format!(" {} ", v);
        let d = Definition::parse(&raw, &v["profile"], b"{}").unwrap();
        assert_eq!(d.original_json, raw);
        assert_eq!(d.accepted_definition, b"{}");
        let root = json!({"":"","0":[],"record":{},"null":null,"boolean":false});
        assert_eq!(
            d.observe(Some(&root), "missing", false).unwrap(),
            Presence::Absent
        );
        for key in ["", "0", "record", "boolean"] {
            assert_eq!(
                d.observe(Some(&root), key, false).unwrap(),
                Presence::Present(&root[key])
            );
        }
        assert_eq!(
            d.observe(Some(&root), "null", false).unwrap_err().code,
            "WFT-OBLIGATION"
        );
        assert_eq!(
            d.observe(Some(&root), "null", true).unwrap_err().code,
            "WFT-CAPABILITY"
        );
        for invalid in [None, Some(&Value::Null), Some(&json!([]))] {
            assert_eq!(
                d.observe(invalid, "missing", false).unwrap_err().code,
                "WFT-OBLIGATION"
            );
        }
    }
    #[test]
    fn altered_meaning_profile_or_original_source_refuses() {
        let base = fixture();
        for field in [
            "defaultApplication",
            "sqlNullRoot",
            "absentMember",
            "nativeNullValue",
        ] {
            let mut v = base.clone();
            v[field] = json!("coerce");
            assert!(Definition::parse(&v.to_string(), &base["profile"], b"{}").is_err());
        }
        assert!(Definition::parse(&base.to_string(), &json!({}), b"{}").is_err());
        assert!(Definition::parse(&base.to_string(), &base["profile"], b"[]").is_err());
    }
}
