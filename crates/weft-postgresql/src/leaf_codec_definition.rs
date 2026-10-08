//! Original JSONB leaf-codec selection. Static admission is not native qualification.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::collections::BTreeMap;
use weft_core::{
    error::{Diagnostic, Result},
    json::{checked_json, sha256},
};
#[jsonschema::validator(
    path = "../../tests/truss-postgresql/upstream/leaf-codec-schema-bundle.json"
)]
struct Shape;
/// Exact immutable registry selection supplied by the backend, not by an SQL caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OriginalArtifact {
    pub identity: String,
    pub bytes: Vec<u8>,
}
pub struct Selection<'a> {
    pub profile: &'a Value,
    pub source_profile: &'a Value,
    pub native_profile: &'a Value,
    pub original_artifacts: &'a BTreeMap<String, OriginalArtifact>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rule {
    UnicodeString,
    Boolean,
    IntegerToken,
    DecimalToken,
}
#[derive(Debug, Clone)]
pub struct Definition {
    pub original_json: String,
    pub original_artifacts: BTreeMap<String, Vec<u8>>,
    pub rule: Rule,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
impl Definition {
    /// Physical extraction only: callers must couple this codec to its admitted
    /// graph and qualify source/native domains before publishing the carrier.
    pub(crate) fn storage_expressions(
        &self,
        location: &crate::property_definition::PropsLocation,
    ) -> Result<(String, String)> {
        let original_codec = checked_json(&self.original_json)
            .map_err(|_| fail("Original extraction codec JSON refused"))?;
        let (kind, carrier) = match original_codec["rule"]["family"].as_str() {
            Some("boolean") => ("boolean", format!(
                "CASE WHEN pg_catalog.jsonb_typeof({})='boolean' THEN {}::pg_catalog.bool ELSE NULL END",
                location.leaf, location.text)),
            Some("string" | "integer" | "decimal") => ("string", location.text.clone()),
            _ => return Err(fail("Original extraction family is unknown")),
        };
        let storage_integrity = format!(
            "({} AND pg_catalog.jsonb_typeof({})='{kind}')",
            location.root_integrity, location.leaf
        );
        Ok((carrier, storage_integrity))
    }
    pub fn parse(raw: &str, selected: Selection<'_>) -> Result<Self> {
        if raw.len() > 4 * 1024 * 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "binding",
                "Leaf codec exceeds candidate byte bound",
            ));
        }
        let value =
            checked_json(raw).map_err(|_| fail("Malformed or duplicate-member leaf codec"))?;
        if !Shape::is_valid(&value)
            || &value["profile"] != selected.profile
            || &value["sourceInterpretationProfile"] != selected.source_profile
            || &value["nativeDomainProfile"] != selected.native_profile
        {
            return Err(fail(
                "Leaf codec grammar or registered profile selection differs",
            ));
        }
        let family = value["rule"]["family"].as_str().unwrap();
        if value["rule"]["decodedCarrierKind"] != family {
            return Err(fail("Leaf codec changes scalar carrier family"));
        }
        let rule = match family {
            "string" => Rule::UnicodeString,
            "boolean" => Rule::Boolean,
            "integer" => Rule::IntegerToken,
            "decimal" => Rule::DecimalToken,
            _ => return Err(fail("Unknown leaf family")),
        };
        let mut paths = vec![
            "authoredDefinition",
            "sourceInterpretationDefinition",
            "nativeDomainDefinition",
        ];
        if matches!(rule, Rule::IntegerToken | Rule::DecimalToken) {
            paths.push("rule/numericAdoptionEvidence");
        }
        if paths.len() != selected.original_artifacts.len() {
            return Err(fail(
                "Original leaf artifact selection is incomplete or contains unrelated artifacts",
            ));
        }
        let mut artifacts = BTreeMap::new();
        let mut total = 0usize;
        for path in paths {
            let mut artifact = &value;
            for component in path.split('/') {
                artifact = &artifact[component];
            }
            let encoded = artifact["bytesBase64"].as_str().unwrap();
            let bytes = STANDARD
                .decode(encoded)
                .map_err(|_| fail("Leaf artifact is invalid base64"))?;
            total = total
                .checked_add(bytes.len())
                .ok_or_else(|| fail("Leaf byte accounting overflow"))?;
            if total > 4 * 1024 * 1024 {
                return Err(Diagnostic::new(
                    "WFT-LIMIT",
                    "binding",
                    "Leaf artifact closure exceeds candidate byte bound",
                ));
            }
            if STANDARD.encode(&bytes) != encoded
                || artifact["sha256"] != sha256(&bytes)
                || selected
                    .original_artifacts
                    .get(path)
                    .is_none_or(|original| {
                        original.bytes != bytes || artifact["identity"] != original.identity
                    })
            {
                return Err(fail(
                    "Leaf artifact differs from original registered definition/evidence",
                ));
            }
            artifacts.insert(path.into(), bytes);
        }
        Ok(Self {
            original_json: raw.into(),
            original_artifacts: artifacts,
            rule,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn fixture(family: &str) -> (Value, BTreeMap<String, OriginalArtifact>) {
        let pin = json!({"identity":"fixture","version":"0.1.0","sha256":sha256(b"{}")});
        let artifact = json!({"identity":"fixture","bytesBase64":STANDARD.encode(b"{}"),"sha256":sha256(b"{}")});
        let mut rule = match family {
            "string" => {
                json!({"family":"string","storageRepresentation":"json-string","encoding":"preserve-unicode-scalars","decodedCarrierKind":"string"})
            }
            "boolean" => {
                json!({"family":"boolean","storageRepresentation":"json-boolean","encoding":"preserve-boolean","decodedCarrierKind":"boolean"})
            }
            _ => {
                json!({"family":family,"storageRepresentation":"json-string","encoding":"preserve-admitted-source-token","decodedCarrierKind":family})
            }
        };
        let mut originals: BTreeMap<String, OriginalArtifact> = [
            "authoredDefinition",
            "sourceInterpretationDefinition",
            "nativeDomainDefinition",
        ]
        .into_iter()
        .map(|key| {
            (
                key.into(),
                OriginalArtifact {
                    identity: "fixture".into(),
                    bytes: b"{}".to_vec(),
                },
            )
        })
        .collect();
        if matches!(family, "integer" | "decimal") {
            rule["numericAdoptionEvidence"] = artifact.clone();
            originals.insert(
                "rule/numericAdoptionEvidence".into(),
                OriginalArtifact {
                    identity: "fixture".into(),
                    bytes: b"{}".to_vec(),
                },
            );
        }
        (
            json!({"interfaceVersion":"truss-jsonb-leaf-codec/0.1.0","profile":pin,"authoredDefinition":artifact,"sourceInterpretationProfile":pin,"sourceInterpretationDefinition":artifact,"nativeDomainProfile":pin,"nativeDomainDefinition":artifact,"rule":rule,"coercion":"none","readDefault":"none","invalidStoredValue":"complete-result-refusal"}),
            originals,
        )
    }
    fn parse(value: &Value, originals: &BTreeMap<String, OriginalArtifact>) -> Result<Definition> {
        let pin = fixture("string").0["profile"].clone();
        Definition::parse(
            &value.to_string(),
            Selection {
                profile: &pin,
                source_profile: &pin,
                native_profile: &pin,
                original_artifacts: originals,
            },
        )
    }
    #[test]
    fn original_leaf_rules_retain_exact_registered_closure() {
        for (family, rule) in [
            ("string", Rule::UnicodeString),
            ("boolean", Rule::Boolean),
            ("integer", Rule::IntegerToken),
            ("decimal", Rule::DecimalToken),
        ] {
            let (value, originals) = fixture(family);
            let definition = parse(&value, &originals).unwrap();
            assert_eq!(definition.rule, rule);
            assert_eq!(
                definition.original_artifacts,
                originals
                    .iter()
                    .map(|(k, v)| (k.clone(), v.bytes.clone()))
                    .collect()
            );
            assert_eq!(definition.original_json, value.to_string());
        }
    }
    #[test]
    fn all_leaf_storage_rules_preserve_carriers_and_guard_boolean_casts() {
        let location = crate::property_definition::PropsLocation {
            root: "owner.props".into(),
            leaf: "(owner.props -> $1::text)".into(),
            text: "(owner.props ->> $1::text)".into(),
            present: "presence".into(),
            native_null: "native_null".into(),
            root_integrity: "root_ok".into(),
        };
        for family in ["string", "boolean", "integer", "decimal"] {
            let (value, originals) = fixture(family);
            let mut definition = parse(&value, &originals).unwrap();
            // Extraction uses captured original rules rather than this mutable projection.
            definition.rule = Rule::Boolean;
            let (carrier, integrity) = definition.storage_expressions(&location).unwrap();
            let kind = if family == "boolean" {
                "boolean"
            } else {
                "string"
            };
            assert_eq!(
                integrity,
                format!(
                    "(root_ok AND pg_catalog.jsonb_typeof({})='{kind}')",
                    location.leaf
                )
            );
            if family == "boolean" {
                assert_eq!(carrier, format!("CASE WHEN pg_catalog.jsonb_typeof({})='boolean' THEN {}::pg_catalog.bool ELSE NULL END", location.leaf, location.text));
            } else {
                assert_eq!(carrier, location.text);
                assert!(!carrier.contains("numeric"));
            }
        }
    }
    #[test]
    fn rehashed_source_unknown_profiles_and_unproven_numeric_selection_refuse() {
        let (base, originals) = fixture("integer");
        let mut value = base.clone();
        value["sourceInterpretationDefinition"] = json!({"identity":"replacement","bytesBase64":STANDARD.encode(b"changed"),"sha256":sha256(b"changed")});
        assert!(parse(&value, &originals).is_err());
        let mut value = base.clone();
        value["nativeDomainProfile"]["version"] = json!("other");
        assert!(parse(&value, &originals).is_err());
        let mut value = base.clone();
        value["rule"]["decodedCarrierKind"] = json!("decimal");
        assert!(parse(&value, &originals).is_err());
        let mut missing = originals.clone();
        missing.remove("rule/numericAdoptionEvidence");
        assert!(parse(&base, &missing).is_err());
        let mut value = base.clone();
        value["nativeDomainDefinition"]["identity"] = json!("other-owner");
        assert!(parse(&value, &originals).is_err());
        let mut value = base;
        value["coercion"] = json!("cast");
        assert!(parse(&value, &originals).is_err());
    }
}
