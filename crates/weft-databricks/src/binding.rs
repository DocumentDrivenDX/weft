//! Explicit candidate mapping admission against pinned Ashlar-owned layout.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use weft_core::{
    error::{Diagnostic, Result},
    ir::{Family, Identity, ModelPin, Span},
    model::Catalog,
    syntax::Name,
};

pub const PROFILE: &str = "ashlar-databricks-candidate/0.1.0";
pub const LAYOUT_SHA256: &str = "ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Binding {
    pub profile: String,
    pub layout_revision: String,
    pub layout_sha256: String,
    pub model_pins: Vec<ModelPin>,
    pub publication: Publication,
    pub records: Vec<Record>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Publication {
    pub id: String,
    pub manifest_uuid: String,
    pub tables: Vec<Table>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Table {
    pub name: [String; 3],
    pub uuid: String,
    pub version: i64,
}
impl Table {
    pub fn sql(&self) -> String {
        format!(
            "{} VERSION AS OF {}",
            self.name
                .iter()
                .map(|p| quote(p))
                .collect::<Vec<_>>()
                .join("."),
            self.version
        )
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Record {
    pub logical: Identity,
    pub table: usize,
    pub kind: RecordKind,
    pub source_system: String,
    pub type_id: String,
    pub schema_revision: String,
    pub properties: Vec<Property>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordKind {
    Object,
    Edge,
    NodeProjection,
    EdgeProjection,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Property {
    pub logical: Identity,
    pub home: Home,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Home {
    Props {
        #[serde(rename = "propertyId")]
        property_id: String,
    },
    Column {
        value: String,
        present: Option<String>,
        #[serde(rename = "nativeType")]
        native_type: String,
    },
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
fn text(s: &str) -> bool {
    !s.is_empty() && !s.contains('\0') && s.len() <= 4096
}
pub fn quote(s: &str) -> String {
    format!("`{}`", s.replace('`', "``"))
}
fn signed(s: &str) -> bool {
    s.parse::<i64>().is_ok_and(|v| v.to_string() == s)
}
fn identity_key(id: &Identity) -> String {
    serde_json::to_string(id).unwrap()
}
fn native_column(kind: RecordKind, column: &str) -> Option<&'static str> {
    match (kind, column) {
        (_, "source_system") => Some("STRING"),
        (_, "id") => Some("BIGINT"),
        (RecordKind::Object | RecordKind::NodeProjection, "type_id") => Some("BIGINT"),
        (RecordKind::Edge | RecordKind::EdgeProjection, "rel_type_id") => Some("BIGINT"),
        (RecordKind::Object | RecordKind::Edge, "entity_version") => Some("BIGINT"),
        (RecordKind::NodeProjection, "group_value") => Some("STRING"),
        (RecordKind::NodeProjection, "rank_value") => Some("BIGINT"),
        // edge_ab.score is DOUBLE: it cannot implement exact Weft decimal.
        _ => None,
    }
}
fn presence(kind: RecordKind, column: &str) -> Option<&'static str> {
    match (kind, column) {
        (RecordKind::NodeProjection, "group_value") => Some("group_present"),
        (RecordKind::NodeProjection, "rank_value") => Some("rank_present"),
        _ => None,
    }
}

/// Admission proves mapping structure/model agreement, never database custody.
/// Runtime policy, publication, schema and exact field correspondence are host obligations.
pub fn admit(catalog: &Catalog, value: &Value) -> Result<Binding> {
    // Core Identity intentionally retains a wider compatibility surface. A
    // backend mapping cannot silently ignore extra members in that identity.
    if let Some(records) = value["records"].as_array() {
        for record in records {
            let ids = std::iter::once(&record["logical"]).chain(
                record["properties"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|p| &p["logical"]),
            );
            for id in ids {
                if id.as_object().is_none_or(|o| {
                    o.len() != 4
                        || o.keys().any(|k| {
                            !matches!(k.as_str(), "documentId" | "revision" | "module" | "element")
                        })
                }) {
                    return Err(fail("Unknown original identity mapping member"));
                }
            }
        }
    }
    let binding: Binding = serde_json::from_value(value.clone())
        .map_err(|_| fail("Unknown or malformed Ashlar mapping member"))?;
    if binding.profile != PROFILE
        || binding.layout_revision != "ashlar-delta/0.3"
        || binding.layout_sha256 != LAYOUT_SHA256
    {
        return Err(fail("Unregistered Ashlar layout or binding profile"));
    }
    if binding.model_pins != catalog.pins() {
        return Err(fail(
            "Binding model pins differ from the original module bundle",
        ));
    }
    if !text(&binding.publication.id)
        || !text(&binding.publication.manifest_uuid)
        || binding.publication.tables.is_empty()
        || binding.publication.tables.len() > 128
    {
        return Err(fail(
            "Publication identity/vector is missing or exceeds bounds",
        ));
    }
    let mut names = BTreeSet::new();
    let mut uuids = BTreeSet::new();
    for table in &binding.publication.tables {
        if table.version < 0
            || !text(&table.uuid)
            || table
                .name
                .iter()
                .any(|s| !text(s) || s.chars().count() > 255)
            || !names.insert(table.name.clone())
            || !uuids.insert(&table.uuid)
        {
            return Err(fail("Invalid or ambiguous pinned Delta table"));
        }
    }
    if binding.records.is_empty() || binding.records.len() > 4096 {
        return Err(fail("Missing or excessive record mappings"));
    }
    let mut records = BTreeSet::new();
    for record in &binding.records {
        if !records.insert(identity_key(&record.logical))
            || record.table >= binding.publication.tables.len()
            || !text(&record.source_system)
            || !signed(&record.type_id)
            || !text(&record.schema_revision)
            || record.properties.len() > 4096
        {
            return Err(fail("Invalid or duplicate original record mapping"));
        }
        let authored = catalog
            .record_by_identity(&record.logical)
            .map_err(|_| fail("Mapped record is not in the supplied original module"))?;
        let mut fields = BTreeSet::new();
        for property in &record.properties {
            if !fields.insert(identity_key(&property.logical))
                || property.logical.document_id != record.logical.document_id
                || property.logical.revision != record.logical.revision
                || property.logical.module != record.logical.module
            {
                return Err(fail("Duplicate or foreign property mapping"));
            }
            let input = catalog
                .inputs
                .iter()
                .find(|i| i.pin == authored.pin)
                .unwrap();
            let document = weft_core::json::checked_json(&input.document_json).unwrap();
            let field = document["modules"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["id"] == property.logical.module)
                .and_then(|m| m["elements"].as_array())
                .and_then(|es| es.iter().find(|e| e["id"] == property.logical.element))
                .ok_or_else(|| fail("Original field is missing"))?;
            let name = field["name"]
                .as_str()
                .ok_or_else(|| fail("Mapped field has no original name"))?;
            let (identity, ty, _) = catalog
                .field(
                    &authored,
                    &Name {
                        value: name.into(),
                        quoted: true,
                        span: Span { start: 0, end: 0 },
                    },
                )
                .map_err(|_| fail("Property is not an admitted scalar member of this record"))?;
            if identity != property.logical
                || field["extensions"]
                    .as_object()
                    .is_none_or(|e| !e.is_empty())
            {
                return Err(fail("Selected field extension semantics require separately registered interpretation"));
            }
            match &property.home {
                Home::Props { property_id } => {
                    if !signed(property_id) {
                        return Err(fail("Property catalog ID is not canonical signed64 text"));
                    }
                }
                Home::Column {
                    value,
                    present,
                    native_type,
                } => {
                    if native_column(record.kind, value) != Some(native_type.as_str())
                        || present.as_deref() != presence(record.kind, value)
                    {
                        return Err(fail("Column home differs from the pinned owner layout"));
                    }
                    let agrees = match (&ty.family, native_type.as_str()) {
                        (Family::String, "STRING") => true,
                        (Family::Integer, "BIGINT") => {
                            ty.facets["integerWidth"]["signed"] == true
                                && ty.facets["integerWidth"]["bits"]
                                    .as_u64()
                                    .is_some_and(|bits| bits <= 64)
                        }
                        _ => false,
                    };
                    if !agrees {
                        return Err(fail("Native column carrier cannot establish the original logical scalar domain"));
                    }
                }
            }
        }
    }
    Ok(binding)
}
