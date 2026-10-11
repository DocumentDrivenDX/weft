//! Explicit candidate mapping admission against pinned Ashlar-owned layout.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use weft_core::{
    application_model::Shape,
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
    #[serde(default)]
    pub relationships: Vec<Relationship>,
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
        #[serde(default)]
        encoding: Option<String>,
    },
    Column {
        value: String,
        present: Option<String>,
        #[serde(rename = "nativeType")]
        native_type: String,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Relationship {
    pub logical: Value,
    pub accepted_definition: Value,
    pub table: usize,
    pub kind: RecordKind,
    pub source_system: String,
    pub type_id: String,
    pub schema_revision: String,
    pub source: Identity,
    pub target: Identity,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "lower", message)
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

fn faithful_binding(value:&Value)->Result<Binding>{
    if !value.is_object(){return Err(fail("Ashlar mapping must be an object"));}
    if value["relationships"].as_array().is_some_and(|a|a.len()>4096){return Err(fail("Excessive relationship mappings"));}
    let records=value["records"].as_array().ok_or_else(||fail("Record mappings must be an array"))?;
    if records.len()>4096{return Err(fail("Excessive record mappings"));}
    for record in records {
        if !record.is_object()||!record["kind"].as_str().is_some_and(|s|matches!(s,"object"|"edge"|"nodeProjection"|"edgeProjection")){return Err(fail("Record kind must be a declared string token"));}
    }
    if let Some(relationships)=value["relationships"].as_array(){for relationship in relationships{
        if !relationship.is_object()||!relationship["kind"].as_str().is_some_and(|s|matches!(s,"object"|"edge"|"nodeProjection"|"edgeProjection")){return Err(fail("Relationship kind must be a declared string token"));}
    }}
    let mut skeleton=value.clone();let mut originals=Vec::new();
    if let Some(relationships)=skeleton.get_mut("relationships"){
        for relationship in relationships.as_array_mut().ok_or_else(||fail("Relationship mappings must be an array"))?{
            let object=relationship.as_object_mut().ok_or_else(||fail("Relationship mapping must be an object"))?;
            let logical=object.get_mut("logical").ok_or_else(||fail("Missing relationship identity"))?;
            let logical=std::mem::replace(logical,Value::Null);
            let accepted=object.get_mut("acceptedDefinition").ok_or_else(||fail("Missing accepted relationship definition"))?;
            let accepted=std::mem::replace(accepted,Value::Null);originals.push((logical,accepted));
        }
    }
    let mut binding:Binding=serde_json::from_value(skeleton).map_err(|_|fail("Unknown or malformed Ashlar mapping member"))?;
    if binding.relationships.len()!=originals.len(){return Err(fail("Relationship correspondence changed"));}
    for (relationship,(logical,accepted)) in binding.relationships.iter_mut().zip(originals){relationship.logical=logical;relationship.accepted_definition=accepted;}
    Ok(binding)
}

/// Admission proves mapping structure/model agreement, never database custody.
/// Runtime policy, publication, schema and exact field correspondence are host obligations.
pub const NATIVE_NULL_ENCODING:&str="ashlar-weft-json-native-null/0.1-candidate";
pub fn admit(catalog: &Catalog, value: &Value) -> Result<Binding> {admit_mode(catalog,value,false)}
pub(crate) fn admit03(catalog:&Catalog,value:&Value)->Result<Binding> {admit_mode(catalog,value,true)}
fn admit_mode(catalog:&Catalog,value:&Value,native_null03:bool)->Result<Binding> {
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
    if let Some(relationships) = value["relationships"].as_array() {
        for relationship in relationships {
            for id in [&relationship["source"], &relationship["target"]] {
                if id.as_object().is_none_or(|o| {
                    o.len() != 4
                        || o.keys().any(|k| {
                            !matches!(k.as_str(), "documentId" | "revision" | "module" | "element")
                        })
                }) {
                    return Err(fail(
                        "Unknown relationship endpoint identity mapping member",
                    ));
                }
            }
        }
    }
    let binding=faithful_binding(value)?;
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
            {
                return Err(fail("Duplicate or foreign property mapping"));
            }
            let input = catalog
                .inputs()
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
            let (_, graph) = catalog
                .member_descriptor_by_identity(&authored, &property.logical)
                .map_err(|_| fail("Property is not an admitted member of this record"))?;
            let descriptor = graph
                .iter()
                .find(|d| d.identity == property.logical)
                .unwrap();
            let ty = match &descriptor.shape {
                Shape::Scalar { logical_type } => Some(logical_type),
                _ => None,
            };
            if ty.is_none() {
                if !matches!(&property.home, Home::Props { encoding: Some(e), .. } if e == "ashlar-weft-json-value/0.1-candidate") {
                    return Err(fail("Compound property needs the registered exact JSON value encoding"));
                }
                for d in &graph {
                    let native = document["modules"].as_array().unwrap().iter().find(|m| m["id"] == d.identity.module)
                        .and_then(|m|m["elements"].as_array()).and_then(|es|es.iter().find(|e|e["id"] == d.identity.element)).unwrap();
                    if native["extensions"].as_object().is_none_or(|e|!e.is_empty()) {
                        return Err(fail("Selected recursive extension semantics are unregistered"));
                    }
                }
                crate::candidate::compound::pack(&graph)?;
            }
            if field["extensions"]
                .as_object()
                .is_none_or(|e| !e.is_empty())
            {
                return Err(fail("Selected field extension semantics require separately registered interpretation"));
            }
            match &property.home {
                Home::Props { property_id, encoding } => {
                    if encoding.as_deref()==Some(NATIVE_NULL_ENCODING) {
                        if !native_null03 || ty.is_none() || descriptor.availability.as_deref()!=Some("absent-allowed") {
                            return Err(fail("Explicit native-null encoding requires 0.3 original optional Scalar"));
                        }
                    } else if encoding.as_deref().is_some_and(|e| e != "ashlar-weft-json-value/0.1-candidate") {
                        return Err(fail("Unknown exact JSON value encoding"));
                    }
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
                    let ty = ty.ok_or_else(|| fail("Typed scalar column cannot represent a compound value"))?;
                    let agrees = match (&ty.family, native_type.as_str()) {
                        (Family::String, "STRING") => true,
                        (Family::Integer, "BIGINT") => ty.facets["integerWidth"]["bits"]
                            .as_u64()
                            .is_some_and(|bits| {
                                bits <= if ty.facets["integerWidth"]["signed"] == true {
                                    64
                                } else {
                                    63
                                }
                            }),
                        _ => false,
                    };
                    if !agrees {
                        return Err(fail("Native column carrier cannot establish the original logical scalar domain"));
                    }
                }
            }
        }
    }
    let mut relationships = BTreeSet::new();
    if binding.relationships.len() > 4096 {
        return Err(fail("Excessive relationship mappings"));
    }
    for physical in &binding.relationships {
        if physical.logical.as_object().is_none_or(|o| {
            o.len() != 4
                || o.keys().any(|k| {
                    !matches!(
                        k.as_str(),
                        "documentId" | "revision" | "module" | "relationship"
                    )
                })
        }) || !relationships.insert(physical.logical.to_string())
            || physical.table >= binding.publication.tables.len()
            || !matches!(physical.kind, RecordKind::Edge | RecordKind::EdgeProjection)
            || !text(&physical.source_system)
            || !signed(&physical.type_id)
            || !text(&physical.schema_revision)
        {
            return Err(fail("Invalid original relationship mapping"));
        }
        let endpoints = [&physical.source, &physical.target]
            .map(|id| binding.records.iter().find(|r| &r.logical == id));
        let [Some(source), Some(target)] = endpoints else {
            return Err(fail("Relationship endpoint mapping is missing"));
        };
        if [source, target].iter().any(|r| {
            r.source_system != physical.source_system
                || !matches!(r.kind, RecordKind::Object | RecordKind::NodeProjection)
        }) || (physical.kind == RecordKind::EdgeProjection
            && [source, target]
                .iter()
                .any(|r| r.kind != RecordKind::NodeProjection))
        {
            return Err(fail(
                "Relationship endpoint layout or source scope is incompatible",
            ));
        }
        let authored = catalog.record_by_identity(&physical.source)?;
        let input = catalog
            .inputs()
            .iter()
            .find(|i| i.pin == authored.pin)
            .unwrap();
        let document = weft_core::json::checked_json(&input.document_json).unwrap();
        let definition = document["modules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["id"] == physical.logical["module"])
            .and_then(|m| m["relationships"].as_array())
            .and_then(|rs| {
                rs.iter()
                    .find(|r| r["id"] == physical.logical["relationship"])
            })
            .ok_or_else(|| fail("Original relationship definition is missing"))?;
        if definition != &physical.accepted_definition
            || definition.as_object().unwrap().keys().any(|k| {
                !matches!(
                    k.as_str(),
                    "id" | "name"
                        | "inverse"
                        | "source"
                        | "target"
                        | "directed"
                        | "sourceMultiplicity"
                        | "targetMultiplicity"
                        | "targetLifecycle"
                        | "extensions"
                )
            })
            || definition
                .get("extensions")
                .is_some_and(|e| e.as_object().is_none_or(|o| !o.is_empty()))
        {
            return Err(fail(
                "Selected relationship meaning is changed or unregistered",
            ));
        }
        for side in ["source", "target"] {
            if definition[side].as_array().unwrap().iter().any(|r| {
                r.as_object()
                    .unwrap()
                    .keys()
                    .any(|k| !matches!(k.as_str(), "module" | "element" | "key"))
            }) {
                return Err(fail("Unregistered relationship endpoint meaning"));
            }
        }
        for side in ["sourceMultiplicity", "targetMultiplicity"] {
            if definition[side]
                .as_object()
                .unwrap()
                .keys()
                .any(|k| !matches!(k.as_str(), "min" | "max"))
            {
                return Err(fail("Unregistered relationship multiplicity meaning"));
            }
        }
        let resolved = catalog.relationship_read(
            &authored,
            &Name {
                value: definition["name"].as_str().unwrap().into(),
                quoted: true,
                span: Span { start: 0, end: 0 },
            },
        )?;
        if serde_json::to_value(&resolved.identity).unwrap() != physical.logical
            || resolved.from != physical.source
            || resolved.to != physical.target
            || resolved.inverse
        {
            return Err(fail("Relationship original endpoints or identity disagree"));
        }
        for (record, key) in [
            (source, &resolved.source_key),
            (target, &resolved.target_key),
        ] {
            let original = catalog.record_by_identity(&record.logical)?;
            let raw = original.value["keys"]
                .as_array()
                .unwrap()
                .iter()
                .find(|k| k["id"] == key.id)
                .unwrap();
            if raw
                .as_object()
                .unwrap()
                .keys()
                .any(|k| !matches!(k.as_str(), "id" | "name" | "fields" | "primary"))
                || key
                    .fields
                    .iter()
                    .any(|id| !record.properties.iter().any(|p| &p.logical == id))
            {
                return Err(fail(
                    "Relationship authored key has unknown meaning or missing homes",
                ));
            }
        }
    }
    Ok(binding)
}
