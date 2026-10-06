//! Candidate owner-artifact integrity admission. This is not production qualification.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    error::{Diagnostic, Result},
    json::{checked_json, sha256},
};
#[jsonschema::validator(path = "../../tests/truss-postgresql/upstream/binding-schema-bundle.json")]
struct OwnerShape;
#[derive(Debug)]
pub struct Admission {
    pub value: Value,
    pub original_json: String,
    pub artifacts: BTreeMap<String, Vec<u8>>,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
fn native(text: &Value, bits: u8) -> Result<()> {
    let s = text
        .as_str()
        .ok_or_else(|| fail("Catalog ID must be exact text"))?;
    let n = s
        .parse::<i64>()
        .map_err(|_| fail("Catalog ID is outside native domain"))?;
    if n.to_string() != s
        || (bits == 32 && i32::try_from(n).is_err())
        || (bits == 16 && i16::try_from(n).is_err())
    {
        return Err(fail("Catalog ID is not canonical signed native text"));
    }
    Ok(())
}
impl Admission {
    /// Verify the frozen proposal shape, exact artifact bytes and mapping uniqueness.
    /// Qualified source/model correspondence and codec interpretation follow separately.
    pub fn parse(raw: &str, profile_id: &str) -> Result<Self> {
        if raw.len() > 4 * 1024 * 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "binding",
                "Binding exceeds four MiB",
            ));
        }
        let value =
            checked_json(raw).map_err(|_| fail("Malformed or duplicate-key binding JSON"))?;
        if value["interfaceVersion"] != "truss-postgresql-binding/0.1.0"
            || !OwnerShape::is_valid(&value)
        {
            return Err(fail(
                "Binding does not match the pinned candidate owner grammar",
            ));
        }
        if value["bindingProfileId"] != profile_id {
            return Err(fail(
                "Binding profile string differs from registered selection",
            ));
        }
        let mut artifacts = BTreeMap::new();
        let mut decoded = 0;
        fn visit(
            value: &Value,
            path: &str,
            out: &mut BTreeMap<String, Vec<u8>>,
            total: &mut usize,
        ) -> Result<()> {
            match value {
                Value::Object(m) => {
                    if let Some(encoded) = m.get("bytesBase64") {
                        let encoded = encoded
                            .as_str()
                            .ok_or_else(|| fail("Artifact encoding must be text"))?;
                        let bytes = STANDARD
                            .decode(encoded)
                            .map_err(|_| fail("Malformed artifact base64"))?;
                        if STANDARD.encode(&bytes) != encoded
                            || m.get("sha256").and_then(Value::as_str)
                                != Some(sha256(&bytes).as_str())
                        {
                            return Err(fail(
                                "Artifact digest or canonical encoding differs from original bytes",
                            ));
                        }
                        *total = total
                            .checked_add(bytes.len())
                            .ok_or_else(|| fail("Artifact accounting overflow"))?;
                        if *total > 4 * 1024 * 1024 {
                            return Err(Diagnostic::new(
                                "WFT-LIMIT",
                                "binding",
                                "Decoded artifact closure exceeds four MiB",
                            ));
                        }
                        out.insert(path.to_owned(), bytes);
                    }
                    for (k, v) in m {
                        visit(
                            v,
                            &format!("{path}/{}", k.replace('~', "~0").replace('/', "~1")),
                            out,
                            total,
                        )?;
                    }
                }
                Value::Array(a) => {
                    for (i, v) in a.iter().enumerate() {
                        visit(v, &format!("{path}/{i}"), out, total)?;
                    }
                }
                _ => {}
            }
            Ok(())
        }
        visit(&value, "", &mut artifacts, &mut decoded)?;
        native(&value["basis"]["catalogRevision"], 32)?;
        let mut identities = BTreeSet::new();
        let mut type_ids = BTreeSet::new();
        for entity in value["entities"].as_array().unwrap() {
            native(&entity["typeId"], 32)?;
            if !identities.insert(entity["logical"].to_string())
                || !type_ids.insert(entity["typeId"].to_string())
            {
                return Err(fail("Duplicate entity identity or native type mapping"));
            }
        }
        let mut identities = BTreeSet::new();
        let mut property_ids = BTreeSet::new();
        for property in value["properties"].as_array().unwrap() {
            native(&property["ownerTypeId"], 32)?;
            native(&property["propertyId"], 32)?;
            if !identities.insert((
                property["ownerTypeId"].to_string(),
                property["logical"].to_string(),
            )) || !property_ids.insert(property["propertyId"].to_string())
            {
                return Err(fail("Duplicate owner/field or native property mapping"));
            }
        }
        let mut keys = BTreeSet::new();
        let mut numbers = BTreeSet::new();
        for key in value["keys"].as_array().unwrap() {
            native(&key["ownerTypeId"], 32)?;
            native(&key["keyNumber"], 16)?;
            if !keys.insert((key["ownerTypeId"].to_string(), key["keyId"].to_string()))
                || !numbers.insert((key["ownerTypeId"].to_string(), key["keyNumber"].to_string()))
            {
                return Err(fail("Duplicate authored key or native key number"));
            }
            let mut components = BTreeSet::new();
            for id in key["orderedPropertyIds"].as_array().unwrap() {
                native(id, 32)?;
                if !components.insert(id.to_string()) {
                    return Err(fail("Repeated key component"));
                }
            }
        }
        let mut identities = BTreeSet::new();
        let mut ids = BTreeSet::new();
        for rel in value["relationships"].as_array().unwrap() {
            for k in ["relationshipId", "sourceTypeId", "targetTypeId"] {
                native(&rel[k], 32)?;
            }
            if !identities.insert(rel["logical"].to_string())
                || !ids.insert(rel["relationshipId"].to_string())
            {
                return Err(fail("Duplicate logical/native relationship mapping"));
            }
            for side in ["sourceOrderedPropertyIds", "targetOrderedPropertyIds"] {
                let mut seen = BTreeSet::new();
                for id in rel[side].as_array().unwrap() {
                    native(id, 32)?;
                    if !seen.insert(id.to_string()) {
                        return Err(fail("Repeated relationship endpoint key component"));
                    }
                }
            }
        }
        Ok(Self {
            value,
            original_json: raw.into(),
            artifacts,
        })
    }
    pub fn decoded_json(&self, path: &str) -> Result<Value> {
        let bytes = self
            .artifacts
            .get(path)
            .ok_or_else(|| fail("Missing original artifact"))?;
        let raw = std::str::from_utf8(bytes)
            .map_err(|_| fail("Selected definition is not UTF-8 JSON"))?;
        checked_json(raw).map_err(|_| fail("Selected definition has malformed or duplicate JSON"))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PropertyHome {
    Props { member: String },
    Row { access: String },
}
impl Admission {
    /// Decode explicit location meaning. Inventory/codec/guard qualification is
    /// intentionally a separate backend assessment, not established by this tag.
    /// Read original home correspondence without applying a fixed candidate's
    /// physical IDs. A registered backend must resolve the returned selectors.
    pub fn original_home_definition(&self, index: usize) -> Result<Value> {
        let property = self.value["properties"]
            .get(index)
            .ok_or_else(|| fail("Missing property mapping"))?;
        let home = self.decoded_json(&format!("/properties/{index}/homeDefinition"))?;
        if !OwnerShape::is_valid(&home)
            || home["ownerCatalogId"] != property["ownerTypeId"]
            || home["propertyCatalogId"] != property["propertyId"]
            || home["layoutInventory"] != self.value["basis"]["layoutInventory"]
        {
            return Err(fail(
                "Original home grammar or owner/property/inventory differs",
            ));
        }
        native(&home["ownerCatalogId"], 32)?;
        native(&home["propertyCatalogId"], 32)?;
        let matches = match property["home"].as_str() {
            Some("props") => home["interfaceVersion"] == "truss-property-home/0.1.0",
            Some("row") => home["interfaceVersion"] == "truss-row-home/0.1.0",
            _ => false,
        };
        if !matches {
            return Err(fail("Declared home and original definition disagree"));
        }
        Ok(home)
    }
    pub fn property_home(&self, index: usize) -> Result<PropertyHome> {
        let property = self.value["properties"]
            .get(index)
            .ok_or_else(|| fail("Missing property mapping"))?;
        let home = self.decoded_json(&format!("/properties/{index}/homeDefinition"))?;
        if !OwnerShape::is_valid(&home) {
            return Err(fail("Home definition does not match pinned owner grammar"));
        }
        if home["ownerCatalogId"] != property["ownerTypeId"]
            || home["propertyCatalogId"] != property["propertyId"]
            || home["layoutInventory"] != self.value["basis"]["layoutInventory"]
        {
            return Err(fail(
                "Home owner/property/inventory differs from enclosing binding",
            ));
        }
        if home["recordKind"] != "object" {
            return Err(fail(
                "Candidate property lowering only addresses object records",
            ));
        }
        native(&home["ownerCatalogId"], 32)?;
        native(&home["propertyCatalogId"], 32)?;
        match property["home"].as_str() {
            Some("props") if home["interfaceVersion"] == "truss-property-home/0.1.0" => {
                if home["relationName"] != "object"
                    || home["propsColumnName"] != "props"
                    || home["discriminatorColumnName"] != "type_id"
                    || home["relationPhysicalIdentity"] != "object-table"
                    || home["propsColumnPhysicalIdentity"] != "object-props"
                    || home["discriminatorColumnPhysicalIdentity"] != "object-type"
                    || home["accessor"] != "jsonb-top-level-member"
                {
                    return Err(fail(
                        "Physical home differs from the fixed candidate SQL profile",
                    ));
                }
                if home["memberName"] != property["propertyId"]
                    || home["valueProfile"] != property["valueProfile"]
                    || home["presenceProfile"] != property["presenceProfile"]
                {
                    return Err(fail("JSONB member or codec/presence profile differs"));
                }
                Ok(PropertyHome::Props {
                    member: home["memberName"].as_str().unwrap().into(),
                })
            }
            Some("row") if home["interfaceVersion"] == "truss-row-home/0.1.0" => {
                if home["stateRelationPhysicalIdentity"] != "state"
                    || home["nodeRelationPhysicalIdentity"] != "node"
                    || home["scalarRelationPhysicalIdentity"] != "scalar"
                {
                    return Err(fail(
                        "Native row selectors differ from the fixed candidate SQL profile",
                    ));
                }
                if home["valueDefinition"] != property["valueDefinition"]
                    || home["presenceDefinition"] != property["presenceDefinition"]
                {
                    return Err(fail("Row value/presence definition differs"));
                }
                Ok(PropertyHome::Row {
                    access: home["access"].as_str().unwrap().into(),
                })
            }
            _ => Err(fail("Declared property home and definition do not agree")),
        }
    }
}
