//! Static original read-context requirements. Never a live authority or handle.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::collections::BTreeSet;
use weft_core::{
    error::{Diagnostic, Result},
    json::{checked_json, sha256},
};
#[jsonschema::validator(
    path = "../../tests/truss-postgresql/upstream/read-context-schema-bundle.json"
)]
struct Shape;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Consistency {
    Live,
    HeldSnapshot,
}
#[derive(Debug)]
pub struct Definition {
    pub original_json: String,
    pub resource_bytes: Vec<u8>,
    allowed: Vec<Consistency>,
    obligations: Vec<String>,
}
#[derive(Debug)]
pub struct Requirements<'a> {
    pub original_binding_and_layout: bool,
    pub current_authority: bool,
    pub live_affine_transaction: bool,
    pub snapshot_custody: bool,
    pub publication_check: bool,
    pub obligations: &'a [String],
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
impl Definition {
    pub fn parse(
        raw: &str,
        profile: &Value,
        resource_profile: &Value,
        resource_bytes: &[u8],
        registry: &BTreeSet<String>,
    ) -> Result<Self> {
        if raw.len() > 4 * 1024 * 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "binding",
                "Read-context definition exceeds candidate byte bound",
            ));
        }
        let value = checked_json(raw)
            .map_err(|_| fail("Malformed or duplicate-member read-context definition"))?;
        if !Shape::is_valid(&value) {
            return Err(fail(
                "Read-context definition differs from the pinned original grammar",
            ));
        }
        if &value["profile"] != profile || &value["resourceProfile"] != resource_profile {
            return Err(fail(
                "Read-context or resource profile differs from registered selection",
            ));
        }
        let artifact = &value["resourceDefinition"];
        let encoded = artifact["bytesBase64"].as_str().unwrap();
        let bytes = STANDARD
            .decode(encoded)
            .map_err(|_| fail("Read-context resource bytes are invalid base64"))?;
        if STANDARD.encode(&bytes) != encoded
            || artifact["sha256"] != sha256(&bytes)
            || bytes != resource_bytes
        {
            return Err(fail(
                "Read-context resource custody differs from registered original",
            ));
        }
        let obligations: Vec<String> = value["requiredHostObligations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id.as_str().unwrap().to_owned())
            .collect();
        if obligations.iter().any(|id| !registry.contains(id)) {
            return Err(fail("Read-context requires an unknown host obligation"));
        }
        let allowed = value["allowedConsistency"]
            .as_array()
            .unwrap()
            .iter()
            .map(|mode| {
                if mode == "live" {
                    Consistency::Live
                } else {
                    Consistency::HeldSnapshot
                }
            })
            .collect();
        Ok(Self {
            original_json: raw.into(),
            resource_bytes: bytes,
            allowed,
            obligations,
        })
    }
    pub fn requirements(&self, mode: Consistency) -> Result<Requirements<'_>> {
        if !self.allowed.contains(&mode) {
            return Err(Diagnostic::new(
                "WFT-CAPABILITY",
                "binding",
                "Consistency mode is not admitted by the original context",
            ));
        }
        Ok(Requirements {
            original_binding_and_layout: true,
            current_authority: true,
            live_affine_transaction: true,
            snapshot_custody: mode == Consistency::HeldSnapshot,
            publication_check: true,
            obligations: &self.obligations,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn fixture() -> Value {
        let bundle: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/upstream/read-context-schema-bundle.json"
        ))
        .unwrap();
        let schema = bundle["$defs"]
            .as_object()
            .unwrap()
            .values()
            .find(|v| {
                v["properties"]["interfaceVersion"]["const"]
                    == "truss-read-context-definition/0.1.0"
            })
            .unwrap();
        let mut value = json!({});
        for (key, rule) in schema["properties"].as_object().unwrap() {
            if let Some(c) = rule.get("const") {
                value[key] = c.clone();
            }
        }
        let pin = json!({"identity":"context-fixture","version":"0.1.0","sha256":sha256(b"{}")});
        value["profile"] = pin.clone();
        value["resourceProfile"] = pin;
        value["resourceDefinition"] = json!({"identity":"resource-fixture","bytesBase64":STANDARD.encode(b"{}"),"sha256":sha256(b"{}")});
        value["allowedConsistency"] = json!(["live", "held_snapshot"]);
        value["requiredHostObligations"] = json!(["owner.visibility"]);
        value
    }
    fn parse(value: &Value) -> Result<Definition> {
        Definition::parse(
            &value.to_string(),
            &fixture()["profile"],
            &fixture()["resourceProfile"],
            b"{}",
            &BTreeSet::from(["owner.visibility".into()]),
        )
    }
    #[test]
    fn snapshot_custody_is_conditional_but_authority_is_current_in_both_modes() {
        let v = fixture();
        let d = parse(&v).unwrap();
        assert_eq!(d.resource_bytes, b"{}");
        for mode in [Consistency::Live, Consistency::HeldSnapshot] {
            let r = d.requirements(mode).unwrap();
            assert!(
                r.original_binding_and_layout
                    && r.current_authority
                    && r.live_affine_transaction
                    && r.publication_check
            );
            assert_eq!(r.snapshot_custody, mode == Consistency::HeldSnapshot);
            assert_eq!(r.obligations, ["owner.visibility"]);
        }
        let mut live = v;
        live["allowedConsistency"] = json!(["live"]);
        assert_eq!(
            parse(&live)
                .unwrap()
                .requirements(Consistency::HeldSnapshot)
                .unwrap_err()
                .code,
            "WFT-CAPABILITY"
        );
    }
    #[test]
    fn unknown_obligations_resources_and_serialized_authority_refuse() {
        let v = fixture();
        for (field, meaning) in [
            ("requiredHostObligations", json!(["unknown"])),
            ("allowedConsistency", json!(["live", "live"])),
            ("onMismatch", json!("recompile")),
            ("transactionHandle", json!({"trusted":true})),
        ] {
            let mut altered = v.clone();
            altered[field] = meaning;
            assert!(parse(&altered).is_err(), "{field}");
        }
        assert!(Definition::parse(
            &v.to_string(),
            &v["profile"],
            &v["resourceProfile"],
            b"[]",
            &BTreeSet::from(["owner.visibility".into()])
        )
        .is_err());
    }
}
