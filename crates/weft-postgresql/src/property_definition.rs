//! Compose original value and home admissions. Operation/native qualification follows.
use crate::{
    binding::Admission, leaf_codec_definition, presence_definition, value_definition::Graph,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    application_model::{Descriptor, Shape},
    error::{Diagnostic, Result},
    ir::Identity,
    json::checked_json,
    model::Catalog,
};
pub struct Selection<'a> {
    pub value_profile: &'a Value,
    pub presence_profile: &'a Value,
    pub leaf_codecs: &'a BTreeMap<String, leaf_codec_definition::Definition>,
    pub record_presence: &'a BTreeMap<String, presence_definition::Definition>,
}
#[derive(Debug)]
pub struct ValueAdmission {
    admitted_leaf_codecs: BTreeMap<String, leaf_codec_definition::Definition>,
    pub definition_artifact: Value,
    pub graph: Graph,
    pub presence: presence_definition::Definition,
    pub(crate) descriptors: Vec<Descriptor>,
}
impl ValueAdmission {
    pub fn descriptors(&self) -> &[Descriptor] {
        &self.descriptors
    }
    /// Scalar extraction uses the captured registry meaning; compound roots
    /// return no scalar template and require recursive decoding.
    pub fn props_scalar_storage(&self, location: &PropsLocation) -> Result<Option<ScalarStorage>> {
        self.props_node_storage(self.graph.root, location)
    }
    /// The supplied location must correspond to this node in the recursive
    /// decoder; original field/slot/presence correspondence is checked separately.
    pub fn props_node_storage(
        &self,
        index: usize,
        location: &PropsLocation,
    ) -> Result<Option<ScalarStorage>> {
        let node = self.graph.value["nodes"]
            .get(index)
            .ok_or_else(|| fail("Selected codec node is outside original graph"))?;
        if node["shape"]["kind"] != "scalar" {
            return Ok(None);
        }
        let codec = self
            .admitted_leaf_codecs
            .get(
                node["nodeId"]
                    .as_str()
                    .ok_or_else(|| fail("Original codec node ID is missing"))?,
            )
            .ok_or_else(|| fail("Captured original node codec is missing"))?;
        let original = self
            .graph
            .artifacts
            .get(&format!("/nodes/{index}/codecDefinition"))
            .ok_or_else(|| fail("Original node codec artifact is missing"))?;
        if original != codec.original_json.as_bytes() {
            return Err(fail("Captured node codec differs from original graph"));
        }
        let (carrier, storage_integrity) = codec.storage_expressions(location)?;
        Ok(Some(ScalarStorage {
            carrier,
            storage_integrity,
        }))
    }
}
#[derive(Debug)]
pub struct ScalarStorage {
    pub carrier: String,
    pub storage_integrity: String,
}
pub struct PhysicalSelection<'a> {
    pub profile: &'a Value,
    pub inventory: &'a leaf_codec_definition::OriginalArtifact,
    pub relations: &'a BTreeMap<String, String>,
    pub columns: &'a BTreeMap<String, crate::row_join_definition::Column>,
    pub row_join: Option<&'a crate::row_join_definition::Definition>,
    pub obligations: &'a BTreeSet<String>,
    pub edge_association: Option<(&'a Value, &'a leaf_codec_definition::OriginalArtifact)>,
}
#[derive(Debug)]
pub enum HomeAdmission {
    Props {
        member: String,
        record_kind: crate::row_join_definition::RecordKind,
        relation: crate::Identifier,
        props_column: crate::Identifier,
        discriminator_column: crate::Identifier,
    },
    Row {
        access: String,
        record_kind: crate::row_join_definition::RecordKind,
        original_join_json: String,
        relation: crate::Identifier,
        discriminator_column: crate::Identifier,
    },
}
#[derive(Debug)]
pub struct PropertyAdmission {
    original_binding_sha256: String,
    pub owner: Identity,
    pub identity: Identity,
    pub owner_catalog_id: String,
    pub property_catalog_id: String,
    pub value: ValueAdmission,
    pub home: HomeAdmission,
}
/// Physical JSONB location only. Codec interpretation and operation admission
/// must be completed before using a leaf as a logical operand or result.
#[derive(Debug)]
pub struct PropsLocation {
    pub root: String,
    pub leaf: String,
    pub text: String,
    pub present: String,
    pub native_null: String,
    pub root_integrity: String,
}
/// Original admitted physical owner mapping, independent of property storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerMapping {
    pub record_kind: crate::row_join_definition::RecordKind,
    pub relation: crate::Identifier,
    pub discriminator_column: crate::Identifier,
}
#[derive(Debug)]
pub struct OwnerSource {
    pub sql: String,
    /// Selects the admitted owner type; never substitutes for integrity checks.
    pub discriminator: String,
}
impl HomeAdmission {
    pub fn owner_mapping(&self) -> OwnerMapping {
        let (record_kind, relation, discriminator_column) = match self {
            Self::Props {
                record_kind,
                relation,
                discriminator_column,
                ..
            }
            | Self::Row {
                record_kind,
                relation,
                discriminator_column,
                ..
            } => (record_kind, relation, discriminator_column),
        };
        OwnerMapping {
            record_kind: record_kind.clone(),
            relation: relation.clone(),
            discriminator_column: discriminator_column.clone(),
        }
    }
}
impl OwnerMapping {
    pub fn source(
        &self,
        namespace: &crate::Identifier,
        alias: &crate::Identifier,
        catalog_id: &str,
        parameters: &mut crate::Parameters,
    ) -> Result<OwnerSource> {
        let slot = parameters.catalog(
            crate::CatalogDomain::Int,
            catalog_id,
            serde_json::json!({"typeId":catalog_id,"use":"admitted-owner-scan"}),
        )?;
        Ok(OwnerSource {
            sql: format!(
                "{} AS {}",
                crate::qualified(namespace, &self.relation),
                alias.sql()
            ),
            discriminator: format!(
                "({}.{} = {slot}::pg_catalog.int4)",
                alias.sql(),
                self.discriminator_column.sql()
            ),
        })
    }
}
impl HomeAdmission {
    pub fn props_location(
        &self,
        owner_alias: &crate::Identifier,
        parameters: &mut crate::Parameters,
    ) -> Result<PropsLocation> {
        let Self::Props {
            member,
            props_column,
            ..
        } = self
        else {
            return Err(Diagnostic::new(
                "WFT-CAPABILITY",
                "lower",
                "Native row home cannot be lowered through JSONB props",
            ));
        };
        let slot = parameters.push(
            weft_core::ir::LogicalType {
                family: weft_core::ir::Family::String,
                facets: serde_json::json!({}),
                nullable: false,
            },
            member.clone(),
            serde_json::json!({"propertyId": member, "use": "admitted-jsonb-member"}),
        )?;
        let root = format!("{}.{}", owner_alias.sql(), props_column.sql());
        let leaf = format!("({root} -> {slot}::pg_catalog.text)");
        Ok(PropsLocation {
            text: format!("({root} ->> {slot}::pg_catalog.text)"),
            present: format!("(CASE WHEN pg_catalog.jsonb_typeof({root})='object' THEN {root} ? {slot}::pg_catalog.text ELSE NULL END)"),
            native_null: format!("(pg_catalog.jsonb_typeof({leaf})='null')"),
            root_integrity: format!("({root} IS NOT NULL AND pg_catalog.jsonb_typeof({root})='object')"),
            root,
            leaf,
        })
    }
}
/// Storage carrier extraction retains numeric tokens as text. Structural
/// integrity does not replace the selected source/native domain procedure.
#[derive(Debug)]
pub struct LeafStorage {
    pub location: PropsLocation,
    pub carrier: String,
    pub storage_integrity: String,
}
impl PropertyAdmission {
    /// Presence is observed under the original field availability, before codec
    /// decoding. An observed present value is not yet a publishable logical value.
    pub fn observe_props_presence<'a>(
        &self,
        root: Option<&'a Value>,
    ) -> Result<presence_definition::Presence<'a>> {
        let HomeAdmission::Props { member, .. } = &self.home else {
            return Err(Diagnostic::new(
                "WFT-CAPABILITY",
                "decode",
                "Native row presence requires its selected row procedure",
            ));
        };
        let descriptor = self
            .value
            .descriptors
            .iter()
            .find(|descriptor| descriptor.identity == self.identity)
            .ok_or_else(|| fail("Original property descriptor is missing"))?;
        let nullable = match &descriptor.shape {
            Shape::Scalar { logical_type } => logical_type.nullable,
            _ => false, // No compound native-null profile is admitted by this gate.
        };
        let observed = self.value.presence.observe(root, member, nullable)?;
        match (descriptor.availability.as_deref(), &observed) {
            (Some("required"), presence_definition::Presence::Absent) => Err(Diagnostic::new(
                "WFT-OBLIGATION",
                "decode",
                "Required original property is absent",
            )),
            (Some("required" | "absent-allowed"), _) => Ok(observed),
            _ => Err(fail("Original field availability is not established")),
        }
    }
    pub(crate) fn binding_sha256(&self) -> &str {
        &self.original_binding_sha256
    }
    pub fn verify_binding_basis(&self, original_binding_sha256: &str) -> Result<()> {
        if self.original_binding_sha256 != original_binding_sha256 {
            return Err(fail(
                "Admitted property belongs to a different original binding basis",
            ));
        }
        Ok(())
    }
    pub fn row_root_location(
        &self,
        namespace: &crate::Identifier,
        owner_alias: &crate::Identifier,
        occurrence: usize,
        parameters: &mut crate::Parameters,
    ) -> Result<crate::row_join_definition::RootLocation> {
        let HomeAdmission::Row {
            original_join_json, ..
        } = &self.home
        else {
            return Err(Diagnostic::new(
                "WFT-CAPABILITY",
                "lower",
                "Props home cannot use native row access",
            ));
        };
        crate::row_join_definition::root_location(
            original_join_json,
            namespace,
            owner_alias,
            &self.owner_catalog_id,
            &self.property_catalog_id,
            occurrence,
            parameters,
        )
    }
    pub fn props_leaf_storage(
        &self,
        codec: &leaf_codec_definition::Definition,
        owner_alias: &crate::Identifier,
        parameters: &mut crate::Parameters,
    ) -> Result<LeafStorage> {
        let node = &self.value.graph.value["nodes"][self.value.graph.root];
        if node["shape"]["kind"] != "scalar" {
            return Err(Diagnostic::new(
                "WFT-CAPABILITY",
                "lower",
                "Compound property requires recursive codec lowering",
            ));
        }
        let original = self
            .value
            .graph
            .artifacts
            .get(&format!("/nodes/{}/codecDefinition", self.value.graph.root))
            .ok_or_else(|| fail("Original root leaf codec is missing"))?;
        if original != codec.original_json.as_bytes() {
            return Err(fail("Selected extraction codec differs from admitted root"));
        }
        let location = self.home.props_location(owner_alias, parameters)?;
        let (carrier, storage_integrity) = codec.storage_expressions(&location)?;
        Ok(LeafStorage {
            location,
            carrier,
            storage_integrity,
        })
    }
}
/// Registered metadata must already correspond to original inventory bytes.
/// This function checks selectors against that metadata; it does not create it.
pub fn admit_home(
    binding: &Admission,
    index: usize,
    selected: PhysicalSelection<'_>,
) -> Result<HomeAdmission> {
    use crate::row_join_definition::RecordKind;
    let home = binding.original_home_definition(index)?;
    let property = &binding.value["properties"][index];
    let inventory = &binding.value["basis"]["layoutInventory"];
    if &property["homeProfile"] != selected.profile
        || inventory["identity"] != selected.inventory.identity
        || binding
            .artifacts
            .get("/basis/layoutInventory")
            .is_none_or(|bytes| bytes != &selected.inventory.bytes)
    {
        return Err(fail(
            "Home profile or original inventory differs from registration",
        ));
    }
    let edge = home["recordKind"] == "edge";
    let record_kind = if edge {
        RecordKind::Edge
    } else {
        RecordKind::Object
    };
    if edge {
        use base64::{engine::general_purpose::STANDARD, Engine};
        let (profile, original) = selected
            .edge_association
            .ok_or_else(|| fail("Edge home lacks registered association meaning"))?;
        let artifact = &home["edgeAssociationDefinition"];
        let encoded = artifact["bytesBase64"].as_str().unwrap();
        let bytes = STANDARD
            .decode(encoded)
            .map_err(|_| fail("Edge association artifact base64 refused"))?;
        if &home["edgeAssociationProfile"] != profile
            || artifact["identity"] != original.identity
            || bytes != original.bytes
            || STANDARD.encode(&bytes) != encoded
            || artifact["sha256"] != weft_core::json::sha256(&bytes)
        {
            return Err(fail(
                "Edge home association differs from original registered selection",
            ));
        }
    } else if selected.edge_association.is_some() {
        return Err(fail(
            "Object home cannot consume edge association selection",
        ));
    }
    if property["home"] == "row" {
        let join = selected
            .row_join
            .ok_or_else(|| fail("Native row home has no admitted original join"))?;
        join.verify_home(
            binding,
            index,
            selected.obligations,
            selected.edge_association.map(|(profile, _)| profile),
        )?;
        return Ok(HomeAdmission::Row {
            access: home["access"].as_str().unwrap().into(),
            record_kind,
            original_join_json: join.original_json.clone(),
            relation: crate::Identifier::new(if edge { "edge" } else { "object" })?,
            discriminator_column: crate::Identifier::new(if edge {
                "rel_type_id"
            } else {
                "type_id"
            })?,
        });
    }
    if selected.row_join.is_some() {
        return Err(fail("Props home cannot consume native row join selection"));
    }
    let relation = home["relationPhysicalIdentity"].as_str().unwrap();
    let relation_name = if edge { "edge" } else { "object" };
    let discriminator = if edge { "rel_type_id" } else { "type_id" };
    if home["relationName"] != relation_name
        || home["discriminatorColumnName"] != discriminator
        || selected
            .relations
            .get(relation)
            .is_none_or(|name| name != relation_name)
        || home["memberName"] != property["propertyId"]
        || home["valueProfile"] != property["valueProfile"]
        || home["presenceProfile"] != property["presenceProfile"]
    {
        return Err(fail(
            "Props owner/member/value/presence differs from registered home",
        ));
    }
    let props_id = home["propsColumnPhysicalIdentity"].as_str().unwrap();
    let discriminator_id = home["discriminatorColumnPhysicalIdentity"]
        .as_str()
        .unwrap();
    if props_id == discriminator_id {
        return Err(fail(
            "Props and discriminator cannot alias one physical column",
        ));
    }
    for (id, name) in [(props_id, "props"), (discriminator_id, discriminator)] {
        if selected
            .columns
            .get(id)
            .is_none_or(|column| column.relation_identity != relation || column.name != name)
        {
            return Err(fail(
                "Props column differs from original physical association",
            ));
        }
    }
    Ok(HomeAdmission::Props {
        member: home["memberName"].as_str().unwrap().into(),
        record_kind,
        relation: crate::Identifier::new(relation_name)?,
        props_column: crate::Identifier::new("props")?,
        discriminator_column: crate::Identifier::new(discriminator)?,
    })
}
pub fn admit_property(
    binding: &Admission,
    index: usize,
    catalog: &Catalog,
    descriptors: &[Descriptor],
    value: Selection<'_>,
    physical: PhysicalSelection<'_>,
) -> Result<PropertyAdmission> {
    let property = binding.value["properties"]
        .get(index)
        .ok_or_else(|| fail("Missing selected property"))?;
    let identity: Identity = serde_json::from_value(property["logical"].clone())
        .map_err(|_| fail("Invalid property identity"))?;
    let (entity_index, entity) = binding.value["entities"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .find(|(_, entity)| entity["typeId"] == property["ownerTypeId"])
        .ok_or_else(|| fail("Property owner has no entity mapping"))?;
    let owner: Identity = serde_json::from_value(entity["logical"].clone())
        .map_err(|_| fail("Invalid property owner identity"))?;
    if owner.document_id != identity.document_id || owner.revision != identity.revision {
        return Err(fail(
            "Property owner and Field are not in the same original source cut",
        ));
    }
    let input = catalog
        .inputs
        .iter()
        .find(|input| {
            input.pin.document_id == owner.document_id
                && input.pin.revision == owner.revision
                && input.selected_module_ids.contains(&owner.module)
        })
        .ok_or_else(|| fail("Property owner is outside original selected model"))?;
    let document =
        checked_json(&input.document_json).map_err(|_| fail("Original owner document refused"))?;
    let record = document["modules"]
        .as_array()
        .and_then(|modules| modules.iter().find(|module| module["id"] == owner.module))
        .and_then(|module| module["elements"].as_array())
        .and_then(|elements| {
            elements
                .iter()
                .find(|element| element["id"] == owner.element)
        })
        .ok_or_else(|| fail("Original property owner record is missing"))?;
    if record["kind"] != "record"
        || record["members"].as_array().is_none_or(|members| {
            !members.iter().any(|member| {
                member["module"] == identity.module && member["element"] == identity.element
            })
        })
    {
        return Err(fail("Original Record does not declare the mapped Field"));
    }
    if binding.decoded_json(&format!("/entities/{entity_index}/source"))? != document
        || binding.decoded_json(&format!("/entities/{entity_index}/acceptedDefinition"))? != *record
    {
        return Err(fail(
            "Mapped owner source/definition differs from original Record",
        ));
    }
    let value = admit_value(binding, index, catalog, descriptors, value)?;
    let home = admit_home(binding, index, physical)?;
    if matches!(&home,HomeAdmission::Row{access,..} if access=="scalar-root")
        && value.graph.value["nodes"][value.graph.root]["shape"]["kind"] != "scalar"
    {
        return Err(Diagnostic::new(
            "WFT-CAPABILITY",
            "lower",
            "Compound value cannot use scalar-root native storage",
        ));
    }
    Ok(PropertyAdmission {
        original_binding_sha256: weft_core::json::sha256(binding.original_json.as_bytes()),
        owner,
        identity,
        owner_catalog_id: property["ownerTypeId"].as_str().unwrap().into(),
        property_catalog_id: property["propertyId"].as_str().unwrap().into(),
        value,
        home,
    })
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
/// Compute closure from the frontend descriptors, never from binding-selected
/// nodes. Thus an omitted authored dependency cannot make a smaller graph valid.
fn closure(descriptors: &[Descriptor], root: &Identity) -> Result<Vec<Descriptor>> {
    let id = |identity: &Identity| {
        serde_json::to_string(identity).map_err(|_| fail("Identity encoding refused"))
    };
    let mut indexed = BTreeMap::new();
    for descriptor in descriptors {
        if indexed
            .insert(id(&descriptor.identity)?, descriptor)
            .is_some()
        {
            return Err(fail("Duplicate frontend descriptor identity"));
        }
    }
    let mut pending = vec![id(root)?];
    let mut seen = BTreeSet::new();
    while let Some(identity) = pending.pop() {
        if !seen.insert(identity.clone()) {
            continue;
        }
        let descriptor = indexed
            .get(&identity)
            .ok_or_else(|| fail("Original frontend descriptor closure is incomplete"))?;
        match &descriptor.shape {
            Shape::Sequence { item } | Shape::Map { item } => pending.push(id(item)?),
            Shape::Structured { record } => pending.push(id(record)?),
            Shape::Record { members } => {
                for member in members {
                    pending.push(id(&member.identity)?);
                }
            }
            Shape::Scalar { .. } => {}
        }
    }
    descriptors
        .iter()
        .filter_map(|descriptor| match id(&descriptor.identity) {
            Ok(key) if seen.contains(&key) => Some(Ok(descriptor.clone())),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}
/// This gate establishes original value correspondence. It does not grant SQL
/// operations, physical inventory interpretation or host execution authority.
pub fn admit_value(
    binding: &Admission,
    index: usize,
    catalog: &Catalog,
    descriptors: &[Descriptor],
    selected: Selection<'_>,
) -> Result<ValueAdmission> {
    let property = binding.value["properties"]
        .get(index)
        .ok_or_else(|| fail("Missing selected property"))?;
    if &property["valueProfile"] != selected.value_profile
        || &property["presenceProfile"] != selected.presence_profile
    {
        return Err(fail(
            "Selected property value/presence profiles differ from registration",
        ));
    }
    let root: Identity = serde_json::from_value(property["logical"].clone())
        .map_err(|_| fail("Invalid property identity"))?;
    let input = catalog
        .inputs
        .iter()
        .find(|input| {
            input.pin.document_id == root.document_id
                && input.pin.revision == root.revision
                && input.selected_module_ids.contains(&root.module)
        })
        .ok_or_else(|| fail("Property source is outside selected original model"))?;
    if binding.decoded_json(&format!("/properties/{index}/source"))?
        != checked_json(&input.document_json)
            .map_err(|_| fail("Original property source JSON refused"))?
    {
        return Err(fail(
            "Property source differs from original selected model document",
        ));
    }
    let descriptors = closure(descriptors, &root)?;
    let accepted = binding
        .artifacts
        .get(&format!("/properties/{index}/acceptedDefinition"))
        .ok_or_else(|| fail("Missing original accepted property"))?;
    let text = |path: &str| -> Result<&str> {
        let bytes = binding
            .artifacts
            .get(path)
            .ok_or_else(|| fail("Missing original property definition"))?;
        std::str::from_utf8(bytes).map_err(|_| fail("Original property definition is not UTF-8"))
    };
    let graph = Graph::parse(
        text(&format!("/properties/{index}/valueDefinition"))?,
        selected.value_profile,
        accepted,
    )?;
    graph.verify_model_sources(catalog, &root)?;
    graph.verify_descriptors(&descriptors, &root)?;
    graph.verify_leaf_codecs(selected.leaf_codecs)?;
    graph.verify_record_presence(selected.record_presence)?;
    let raw = text(&format!("/properties/{index}/presenceDefinition"))?;
    let presence =
        presence_definition::Definition::parse(raw, selected.presence_profile, accepted)?;
    if checked_json(raw).map_err(|_| fail("Original presence JSON refused"))?["acceptedDefinition"]
        != property["acceptedDefinition"]
    {
        return Err(fail(
            "Property presence accepted artifact differs from original binding",
        ));
    }
    Ok(ValueAdmission {
        admitted_leaf_codecs: selected.leaf_codecs.clone(),
        definition_artifact: property["valueDefinition"].clone(),
        graph,
        presence,
        descriptors,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use weft_core::{ir::Span, syntax::Name};
    #[test]
    fn closure_is_authored_and_finite_without_binding_influence() {
        let identity = |element: &str| Identity {
            document_id: "d".into(),
            revision: "1".into(),
            module: "m".into(),
            element: element.into(),
        };
        let root = identity("root");
        let record = identity("record");
        let descriptors = vec![
            Descriptor {
                identity: root.clone(),
                availability: Some("required".into()),
                shape: Shape::Structured {
                    record: record.clone(),
                },
            },
            Descriptor {
                identity: record,
                availability: None,
                shape: Shape::Record {
                    members: vec![weft_core::application_model::Member {
                        name: "next".into(),
                        identity: root.clone(),
                    }],
                },
            },
        ];
        assert_eq!(closure(&descriptors, &root).unwrap().len(), 2);
        assert!(closure(&descriptors[..1], &root).is_err());
    }
    #[test]
    fn real_authored_field_composes_graph_presence_and_leaf_correspondence() {
        use base64::{engine::general_purpose::STANDARD, Engine};
        use leaf_codec_definition::{
            Definition as Leaf, OriginalArtifact, Selection as LeafSelection,
        };
        use weft_core::json::sha256;
        let cases: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/compiler-cases.json"
        ))
        .unwrap();
        let inputs = serde_json::from_value(cases[0]["request"]["modules"].clone()).unwrap();
        let catalog = Catalog::prepare(inputs).unwrap();
        let name = |value: &str| Name {
            value: value.to_ascii_lowercase(),
            quoted: false,
            span: Span { start: 0, end: 0 },
        };
        let record = catalog.record(None, &name("Customer")).unwrap();
        let (member, descriptors) = catalog.member_descriptor(&record, &name("name")).unwrap();
        let mut binding: Value = serde_json::from_str(
            cases[0]["request"]["target"]["bindingJson"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        let index = binding["properties"]
            .as_array()
            .unwrap()
            .iter()
            .position(|p| p["logical"] == json!(member.identity))
            .unwrap();
        let property = &binding["properties"][index];
        let pin = property["valueProfile"].clone();
        let authored = property["acceptedDefinition"].clone();
        let bytes = STANDARD
            .decode(authored["bytesBase64"].as_str().unwrap())
            .unwrap();
        let artifact = |identity: &str, bytes: &[u8]| json!({"identity":identity,"bytesBase64":STANDARD.encode(bytes),"sha256":sha256(bytes)});
        let empty = artifact("fixture", b"{}");
        let leaf = json!({"interfaceVersion":"truss-jsonb-leaf-codec/0.1.0","profile":pin,"authoredDefinition":authored,"sourceInterpretationProfile":pin,"sourceInterpretationDefinition":empty,"nativeDomainProfile":pin,"nativeDomainDefinition":empty,"rule":{"family":"string","storageRepresentation":"json-string","encoding":"preserve-unicode-scalars","decodedCarrierKind":"string"},"coercion":"none","readDefault":"none","invalidStoredValue":"complete-result-refusal"});
        let originals = BTreeMap::from([
            (
                "authoredDefinition".into(),
                OriginalArtifact {
                    identity: authored["identity"].as_str().unwrap().into(),
                    bytes: bytes.clone(),
                },
            ),
            (
                "sourceInterpretationDefinition".into(),
                OriginalArtifact {
                    identity: "fixture".into(),
                    bytes: b"{}".to_vec(),
                },
            ),
            (
                "nativeDomainDefinition".into(),
                OriginalArtifact {
                    identity: "fixture".into(),
                    bytes: b"{}".to_vec(),
                },
            ),
        ]);
        let leaf = Leaf::parse(
            &leaf.to_string(),
            LeafSelection {
                profile: &pin,
                source_profile: &pin,
                native_profile: &pin,
                original_artifacts: &originals,
            },
        )
        .unwrap();
        let graph = json!({"interfaceVersion":"truss-value-definition/0.1.0","profile":pin,"rootNodeId":"root","acceptedDefinition":authored,"nodes":[{"nodeId":"root","authoredIdentity":member.identity,"authoredDefinition":authored,"codecProfile":pin,"codecDefinition":artifact("selected-leaf",leaf.original_json.as_bytes()),"shape":{"kind":"scalar","family":"string","storageRepresentation":"json-string"}}]});
        let schema: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/upstream/presence-definition.schema.json"
        ))
        .unwrap();
        let mut presence = json!({});
        for (key, rule) in schema["properties"].as_object().unwrap() {
            if let Some(constant) = rule.get("const") {
                presence[key] = constant.clone();
            }
        }
        presence["profile"] = pin.clone();
        presence["acceptedDefinition"] = authored.clone();
        binding["properties"][index]["valueDefinition"] =
            artifact("original-value-graph", graph.to_string().as_bytes());
        binding["properties"][index]["presenceDefinition"] =
            artifact("original-presence", presence.to_string().as_bytes());
        let admitted = Admission::parse(
            &binding.to_string(),
            binding["bindingProfileId"].as_str().unwrap(),
        )
        .unwrap();
        let leaves = BTreeMap::from([("root".into(), leaf)]);
        let records = BTreeMap::new();
        let select = || Selection {
            value_profile: &pin,
            presence_profile: &pin,
            leaf_codecs: &leaves,
            record_presence: &records,
        };
        let value = admit_value(&admitted, index, &catalog, &descriptors, select()).unwrap();
        assert_eq!(value.descriptors.len(), 1);
        assert_eq!(value.presence.accepted_definition, bytes);
        let inventory = leaf_codec_definition::OriginalArtifact {
            identity: binding["basis"]["layoutInventory"]["identity"]
                .as_str()
                .unwrap()
                .into(),
            bytes: b"{}".to_vec(),
        };
        let relations = BTreeMap::from([("object-table".into(), "object".into())]);
        let columns = BTreeMap::from([
            (
                "object-props".into(),
                crate::row_join_definition::Column {
                    relation_identity: "object-table".into(),
                    name: "props".into(),
                },
            ),
            (
                "object-type".into(),
                crate::row_join_definition::Column {
                    relation_identity: "object-table".into(),
                    name: "type_id".into(),
                },
            ),
        ]);
        let obligations = BTreeSet::new();
        let physical = || PhysicalSelection {
            profile: &pin,
            inventory: &inventory,
            relations: &relations,
            columns: &columns,
            row_join: None,
            obligations: &obligations,
            edge_association: None,
        };
        let property = admit_property(
            &admitted,
            index,
            &catalog,
            &descriptors,
            select(),
            physical(),
        )
        .unwrap();
        match &property.home {
            HomeAdmission::Props {
                relation,
                props_column,
                discriminator_column,
                member,
                ..
            } => {
                assert_eq!(relation.sql(), "\"object\"");
                assert_eq!(props_column.sql(), "\"props\"");
                assert_eq!(discriminator_column.sql(), "\"type_id\"");
                assert_eq!(
                    member,
                    binding["properties"][index]["propertyId"].as_str().unwrap()
                );
            }
            _ => panic!("fixture props home"),
        }
        let mut parameters = crate::Parameters::default();
        let location = property
            .home
            .props_location(
                &crate::Identifier::new("owner.\"alias").unwrap(),
                &mut parameters,
            )
            .unwrap();
        assert_eq!(location.root, "\"owner.\"\"alias\".\"props\"");
        assert_eq!(
            location.leaf,
            format!("({} -> $1::pg_catalog.text)", location.root)
        );
        assert!(location.present.contains("ELSE NULL"));
        assert!(!location.root_integrity.contains("$1"));
        let slots = parameters.into_slots();
        assert_eq!(slots.len(), 1);
        assert_eq!(
            slots[0].value,
            binding["properties"][index]["propertyId"].as_str().unwrap()
        );
        property
            .verify_binding_basis(&weft_core::json::sha256(admitted.original_json.as_bytes()))
            .unwrap();
        assert!(property
            .verify_binding_basis(&weft_core::json::sha256(b"another original binding"))
            .is_err());
        let member_name = binding["properties"][index]["propertyId"].as_str().unwrap();
        let present_root = json!({member_name:""});
        assert!(
            matches!(property.observe_props_presence(Some(&present_root)).unwrap(),presence_definition::Presence::Present(value) if value=="")
        );
        assert!(property.observe_props_presence(Some(&json!({}))).is_err());
        assert!(property
            .observe_props_presence(Some(&json!({member_name:null})))
            .is_err());
        assert!(property.observe_props_presence(Some(&json!([]))).is_err());
        assert!(property.observe_props_presence(None).is_err());
        assert_eq!(property.owner, record.identity);
        assert_eq!(property.identity, member.identity);
        let result =
            crate::result_definition::property_column(&property, 1, "customer_name").unwrap();
        assert_eq!(result.source_identities, [member.identity.clone()]);
        assert!(matches!(
            result.representation,
            weft_core::backend::Representation::Scalar {
                carrier: weft_core::backend::ScalarCarrier::Text,
                decoder: weft_core::backend::ScalarDecoder::Text,
                ..
            }
        ));

        let mut foreign = binding.clone();
        let other = foreign["entities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entity| entity["logical"] != json!(record.identity))
            .unwrap()["typeId"]
            .clone();
        foreign["properties"][index]["ownerTypeId"] = other;
        let foreign = Admission::parse(
            &foreign.to_string(),
            binding["bindingProfileId"].as_str().unwrap(),
        )
        .unwrap();
        assert!(admit_property(
            &foreign,
            index,
            &catalog,
            &descriptors,
            select(),
            physical()
        )
        .is_err());
        let mut foreign = binding.clone();
        let owner_index = foreign["entities"]
            .as_array()
            .unwrap()
            .iter()
            .position(|entity| entity["logical"] == json!(record.identity))
            .unwrap();
        foreign["entities"][owner_index]["acceptedDefinition"] = artifact("replaced-record", b"{}");
        let foreign = Admission::parse(
            &foreign.to_string(),
            binding["bindingProfileId"].as_str().unwrap(),
        )
        .unwrap();
        assert!(admit_property(
            &foreign,
            index,
            &catalog,
            &descriptors,
            select(),
            physical()
        )
        .is_err());

        let mut storage_parameters = crate::Parameters::default();
        let storage = property
            .props_leaf_storage(
                &leaves["root"],
                &crate::Identifier::new("customer").unwrap(),
                &mut storage_parameters,
            )
            .unwrap();
        assert_eq!(storage.carrier, storage.location.text);
        assert!(property
            .value
            .props_node_storage(usize::MAX, &storage.location)
            .is_err());
        assert_eq!(
            property
                .value
                .props_node_storage(property.value.graph.root, &storage.location)
                .unwrap()
                .unwrap()
                .carrier,
            storage.carrier
        );
        assert!(storage.storage_integrity.ends_with("='string')"));
        assert!(!storage.carrier.contains("numeric"));
        let mut substituted = Leaf::parse(
            &leaves["root"].original_json,
            LeafSelection {
                profile: &pin,
                source_profile: &pin,
                native_profile: &pin,
                original_artifacts: &originals,
            },
        )
        .unwrap();
        substituted.original_json.push(' ');
        let mut refused_parameters = crate::Parameters::default();
        assert!(property
            .props_leaf_storage(
                &substituted,
                &crate::Identifier::new("customer").unwrap(),
                &mut refused_parameters
            )
            .is_err());
        assert!(refused_parameters.into_slots().is_empty());
        use crate::{
            comparator_requirements::{admit_properties, registration_key, Requirement},
            native_comparator_definition::{
                Definition as Comparator, Operation, Selection as ComparatorSelection,
            },
        };
        let logical = if let Shape::Scalar { logical_type } = &descriptors[0].shape {
            logical_type.clone()
        } else {
            panic!("fixture scalar")
        };
        let value_artifact = property.value.definition_artifact.clone();
        let graph_bytes = property.value.graph.original_json.as_bytes().to_vec();
        let make_comparator = |value_artifact: Value,
                               graph_bytes: &[u8],
                               native_profile: &Value| {
            let comparator = json!({"interfaceVersion":"truss-native-comparator/0.1.0","profile":pin,"valueDefinition":value_artifact,"sourceDomainDefinition":authored,"nativeDomainProfile":native_profile,"nativeDomainDefinition":empty,"operatorInventory":empty,"strategy":{"kind":"unicode-text-C","nativeType":"pg_catalog.text","encoding":"UTF8","collation":"pg_catalog.C","normalization":"none"},"castOutcome":"exact-or-error","nullOperands":"refuse","absentOperands":"refuse","qualification":empty});
            let originals = BTreeMap::from([
                (
                    "valueDefinition".into(),
                    OriginalArtifact {
                        identity: value_artifact["identity"].as_str().unwrap().into(),
                        bytes: graph_bytes.to_vec(),
                    },
                ),
                (
                    "sourceDomainDefinition".into(),
                    OriginalArtifact {
                        identity: authored["identity"].as_str().unwrap().into(),
                        bytes: bytes.clone(),
                    },
                ),
                (
                    "nativeDomainDefinition".into(),
                    OriginalArtifact {
                        identity: "fixture".into(),
                        bytes: b"{}".to_vec(),
                    },
                ),
                (
                    "operatorInventory".into(),
                    OriginalArtifact {
                        identity: "fixture".into(),
                        bytes: b"{}".to_vec(),
                    },
                ),
                (
                    "qualification".into(),
                    OriginalArtifact {
                        identity: "fixture".into(),
                        bytes: b"{}".to_vec(),
                    },
                ),
            ]);
            Comparator::parse(
                &comparator.to_string(),
                ComparatorSelection {
                    profile: &pin,
                    native_profile,
                    original_artifacts: &originals,
                    operations: &BTreeSet::from([Operation::Equality]),
                },
                &logical,
            )
            .unwrap()
        };
        let registration = registration_key(&record.identity, &member.identity);
        let requirements = vec![Requirement {
            owner: record.identity.clone(),
            identity: member.identity.clone(),
            logical_type: logical.clone(),
            operations: BTreeSet::from([Operation::Equality]),
        }];
        let comparisons = BTreeMap::from([(
            registration.clone(),
            make_comparator(value_artifact.clone(), &graph_bytes, &pin),
        )]);
        let properties = BTreeMap::from([(registration.clone(), property)]);
        admit_properties(&requirements, &properties, &comparisons).unwrap();
        let (_, plan) =
            weft_core::prepare_and_resolve("SELECT c.name FROM Customer c", catalog.inputs.clone())
                .unwrap();
        let manifest = <crate::candidate::Candidate as weft_core::backend::Backend>::describe(
            &crate::candidate::Candidate,
        )
        .unwrap();
        let input = weft_core::backend::BindingInput {
            profile: binding["bindingProfileId"].as_str().unwrap().into(),
            json: admitted.original_json.clone(),
            sha256: weft_core::json::sha256(admitted.original_json.as_bytes()),
        };
        let selection = weft_core::backend::Selection {
            fields: vec![member.identity.clone()],
            records: vec![record.identity.clone()],
            ..Default::default()
        };
        let context = weft_core::backend::Context {
            catalog: &catalog,
            plan: weft_core::backend::Plan::V01(&plan),
            target: &manifest.target_profiles[0],
            binding: &input,
            binding_value: &admitted.value,
            selection: &selection,
        };
        let record_index = admitted.value["entities"]
            .as_array()
            .unwrap()
            .iter()
            .position(|entity| entity["logical"] == json!(record.identity))
            .unwrap();
        let record_source = crate::record_definition::RecordAdmission::admit(
            &admitted,
            record_index,
            &catalog,
            crate::record_definition::Selection {
                inventory: &inventory,
                relation_identity: "object-table",
                discriminator_identity: "object-type",
                relations: &relations,
                columns: &columns,
            },
        )
        .unwrap();
        record_source
            .verify_property(&properties[&registration])
            .unwrap();
        let record_registry = BTreeMap::from([(
            serde_json::to_string(record_source.identity()).unwrap(),
            record_source,
        )]);
        let mut combined_parameters = crate::Parameters::default();
        let combined = crate::registered_access::prepare(
            &context,
            &record_registry,
            &properties,
            &BTreeMap::new(),
            &mut combined_parameters,
        )
        .unwrap();
        assert_eq!(combined.scans.len(), 1);
        assert_eq!(combined.accesses.len(), 1);
        assert_eq!(combined_parameters.into_slots().len(), 2);
        assert_eq!(
            combined.scans.values().next().unwrap().source.sql,
            combined.accesses[0].owner_source.sql
        );
        assert_eq!(
            combined
                .scans
                .values()
                .next()
                .unwrap()
                .structural_check_sql
                .len(),
            1
        );
        let other_index = admitted.value["entities"]
            .as_array()
            .unwrap()
            .iter()
            .position(|entity| entity["logical"] != json!(record.identity))
            .unwrap();
        let other_record = crate::record_definition::RecordAdmission::admit(
            &admitted,
            other_index,
            &catalog,
            crate::record_definition::Selection {
                inventory: &inventory,
                relation_identity: "object-table",
                discriminator_identity: "object-type",
                relations: &relations,
                columns: &columns,
            },
        )
        .unwrap();
        assert!(other_record
            .verify_property(&properties[&registration])
            .is_err());
        crate::comparator_requirements::admit_context(&context, &properties, &comparisons).unwrap();
        crate::comparator_requirements::admit_context(&context, &properties, &BTreeMap::new())
            .unwrap();
        assert!(crate::comparator_requirements::admit_context(
            &context,
            &BTreeMap::new(),
            &comparisons
        )
        .is_err());
        let mut changed = binding.clone();
        changed["basis"]["namespace"] = json!("later_schema");
        // A valid namespace change still changes the exact original binding basis.
        let changed_json = changed.to_string();
        Admission::parse(&changed_json, &input.profile).unwrap();
        let changed_input = weft_core::backend::BindingInput {
            profile: input.profile.clone(),
            sha256: weft_core::json::sha256(changed_json.as_bytes()),
            json: changed_json,
        };
        let requested_scan = match &plan.root {
            weft_core::ir::Node::Project { input, .. } => match input.as_ref() {
                weft_core::ir::Node::Scan { occurrence, .. } => occurrence.clone(),
                _ => panic!("fixture scan"),
            },
            _ => panic!("fixture project"),
        };
        let request = crate::registered_access::Request {
            scan: requested_scan.clone(),
            field: member.identity.clone(),
        };
        let mut access_parameters = crate::Parameters::default();
        let accesses = crate::registered_access::lower(
            &context,
            &properties,
            &BTreeMap::new(),
            &[request],
            &mut access_parameters,
        )
        .unwrap();
        assert_eq!(accesses.len(), 1);
        assert_eq!(accesses[0].value_layout.nodes.len(), 1);
        assert_eq!(accesses[0].value_layout.root, 0);
        let mut row_budget = crate::row_custody::Budget {
            remaining_bytes: 4096,
            remaining_cells: 8,
        };
        assert!(crate::row_custody::admit_property(
            &properties[&registration],
            &accesses[0],
            &[Some("false"), None, None, None, None, None, None, None],
            &mut row_budget,
        )
        .is_err());
        assert_eq!(
            row_budget.remaining_cells, 8,
            "Wrong storage home cannot consume native decoding work"
        );
        assert_eq!(row_budget.remaining_bytes, 4096);
        let captured_storage = accesses[0].scalar_storage.as_ref().unwrap();
        assert!(captured_storage.storage_integrity.ends_with("='string')"));
        assert!(captured_storage
            .carrier
            .contains("\"weft_scan_0\".\"props\" ->> $2"));
        assert!(matches!(
            &accesses[0].location,
            crate::registered_access::Location::Props(_)
        ));
        assert_eq!(accesses[0].owner_alias.sql(), "\"weft_scan_0\"");
        assert!(accesses[0]
            .owner_source
            .sql
            .ends_with(".\"object\" AS \"weft_scan_0\""));
        assert_eq!(
            accesses[0].owner_source.discriminator,
            "(\"weft_scan_0\".\"type_id\" = $1::pg_catalog.int4)"
        );
        let mut atomic_parameters = crate::Parameters::default();
        let requests = [
            crate::registered_access::Request {
                scan: requested_scan.clone(),
                field: member.identity.clone(),
            },
            crate::registered_access::Request {
                scan: "foreign-scan".into(),
                field: member.identity.clone(),
            },
        ];
        assert!(crate::registered_access::lower(
            &context,
            &properties,
            &BTreeMap::new(),
            &requests,
            &mut atomic_parameters
        )
        .is_err());
        assert!(atomic_parameters.into_slots().is_empty());
        let (_, self_plan) = weft_core::prepare_and_resolve("SELECT c.name AS left_name, d.name AS right_name FROM Customer c JOIN Customer d ON c.name = d.name",catalog.inputs.clone()).unwrap();
        let self_requests = match &self_plan.root {
            weft_core::ir::Node::Project { outputs, .. } => outputs
                .iter()
                .map(|output| match &output.expression {
                    weft_core::ir::Expression::Field { scan, identity, .. } => {
                        crate::registered_access::Request {
                            scan: scan.clone(),
                            field: identity.clone(),
                        }
                    }
                    _ => panic!("fixture field"),
                })
                .collect::<Vec<_>>(),
            _ => panic!("fixture projection"),
        };
        let self_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V01(&self_plan),
            ..context
        };
        let mut combined_self_parameters = crate::Parameters::default();
        let combined_self = crate::registered_access::prepare(
            &self_context,
            &record_registry,
            &properties,
            &comparisons,
            &mut combined_self_parameters,
        )
        .unwrap();
        assert_eq!(combined_self.scans.len(), 2);
        assert_eq!(combined_self.accesses.len(), 2);
        let self_parameters_before =
            serde_json::to_value(combined_self_parameters.clone().into_slots()).unwrap();
        let mut native_fields = 0;
        let selected = crate::select_definition::assemble(
            &self_context,
            &combined_self,
            &properties,
            &comparisons,
            &mut combined_self_parameters,
            |node, operands, access, _| match node {
                weft_core::ir::Expression::Field { .. } => {
                    native_fields += 1;
                    Ok(access
                        .unwrap()
                        .scalar_storage
                        .as_ref()
                        .unwrap()
                        .carrier
                        .clone())
                }
                weft_core::ir::Expression::Equal { .. } => {
                    Ok(format!("({} = {})", operands[0], operands[1]))
                }
                _ => panic!("self-join fixture callback"),
            },
        )
        .unwrap();
        if let Ok(path) = std::env::var("WEFT_SELECT_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&json!({
                "sql":selected.sql,"columns":selected.columns,
                "checks":selected.structural_checks.iter().cloned().chain(selected.payload_checks.iter().map(|observation| observation.sql.clone())).collect::<Vec<_>>(),
                "parameters":combined_self_parameters.clone().into_slots(),
            })).unwrap()).unwrap();
        }
        assert_eq!(
            native_fields, 2,
            "Only join operands use the native callback"
        );
        assert!(selected.sql.contains("INNER JOIN"));
        assert!(selected.sql.contains(" WHERE "));
        assert_eq!(selected.columns.len(), 2);
        assert_eq!(selected.payload_checks.len(), 2);
        assert!(!selected.structural_checks.is_empty());
        assert_eq!(
            serde_json::to_value(combined_self_parameters.clone().into_slots()).unwrap(),
            self_parameters_before
        );
        assert!(crate::select_definition::assemble(
            &context,
            &combined_self,
            &properties,
            &comparisons,
            &mut combined_self_parameters,
            |_, _, _, _| panic!("stale context reached native callback"),
        )
        .is_err());
        assert!(crate::select_definition::assemble(
            &self_context,
            &combined_self,
            &properties,
            &comparisons,
            &mut combined_self_parameters,
            |node, _, _, parameters| {
                parameters.push(
                    node.logical_type().clone(),
                    "fixture".into(),
                    json!({"fixture":true}),
                )?;
                Err(weft_core::error::Diagnostic::new(
                    "WFT-CAPABILITY",
                    "emit",
                    "selected callback refusal",
                ))
            },
        )
        .is_err());
        assert_eq!(
            serde_json::to_value(combined_self_parameters.clone().into_slots()).unwrap(),
            self_parameters_before
        );

        let mut changed_parameters = combined_self_parameters.clone();
        changed_parameters
            .push(logical.clone(), "extra".into(), json!({"fixture":true}))
            .unwrap();
        assert!(crate::select_definition::assemble(
            &self_context,
            &combined_self,
            &properties,
            &comparisons,
            &mut changed_parameters,
            |_, _, _, _| panic!("changed parameters reached callback"),
        )
        .is_err());

        // Positive native-home coupling using the same original UMF field.
        let mut row_fixture = crate::row_join_definition::tests::fixture(false);
        row_fixture.value["layoutInventory"] = binding["basis"]["layoutInventory"].clone();
        row_fixture
            .artifacts
            .get_mut("layoutInventory")
            .unwrap()
            .identity = inventory.identity.clone();
        let row_join =
            crate::row_join_definition::tests::parse(&row_fixture.value, &row_fixture).unwrap();
        let row_template: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/binding-row.json"
        ))
        .unwrap();
        let mut row_home: Value = serde_json::from_slice(
            &STANDARD
                .decode(
                    row_template["properties"][0]["homeDefinition"]["bytesBase64"]
                        .as_str()
                        .unwrap(),
                )
                .unwrap(),
        )
        .unwrap();
        row_home["layoutInventory"] = binding["basis"]["layoutInventory"].clone();
        row_home["ownerCatalogId"] = binding["properties"][index]["ownerTypeId"].clone();
        row_home["propertyCatalogId"] = binding["properties"][index]["propertyId"].clone();
        row_home["valueDefinition"] = binding["properties"][index]["valueDefinition"].clone();
        row_home["presenceDefinition"] = binding["properties"][index]["presenceDefinition"].clone();
        row_home["joinProfile"] = row_fixture.value["profile"].clone();
        row_home["joinDefinition"] =
            artifact("selected-row-join", row_join.original_json.as_bytes());
        for (role, key) in [
            ("state", "stateRelationPhysicalIdentity"),
            ("node", "nodeRelationPhysicalIdentity"),
            ("scalar", "scalarRelationPhysicalIdentity"),
        ] {
            row_home[key] = row_fixture.value[role]["relationPhysicalIdentity"].clone();
        }
        let row_obligations = BTreeSet::from([row_home["storedDomainObligation"]
            .as_str()
            .unwrap()
            .to_string()]);
        let mut row_binding = binding.clone();
        row_binding["properties"][index]["home"] = json!("row");
        row_binding["properties"][index]["homeDefinition"] =
            artifact("selected-row-home", row_home.to_string().as_bytes());
        let row_json = row_binding.to_string();
        let row_admission = Admission::parse(&row_json, &input.profile).unwrap();
        let row_property = admit_property(
            &row_admission,
            index,
            &catalog,
            &descriptors,
            select(),
            PhysicalSelection {
                profile: &pin,
                inventory: &inventory,
                relations: &row_fixture.relations,
                columns: &row_fixture.columns,
                row_join: Some(&row_join),
                obligations: &row_obligations,
                edge_association: None,
            },
        )
        .unwrap();
        let row_properties = BTreeMap::from([(registration.clone(), row_property)]);
        let row_input = weft_core::backend::BindingInput {
            profile: input.profile.clone(),
            sha256: sha256(row_json.as_bytes()),
            json: row_json,
        };
        let row_context = weft_core::backend::Context {
            binding: &row_input,
            binding_value: &row_binding,
            ..context
        };
        let mut row_parameters = crate::Parameters::default();
        let row_accesses = crate::registered_access::lower_plan(
            &row_context,
            &row_properties,
            &comparisons,
            &mut row_parameters,
        )
        .unwrap();
        assert_eq!(row_accesses.len(), 1);
        let codec = row_properties[&registration]
            .value
            .graph
            .artifacts
            .get("/nodes/0/codecDefinition")
            .unwrap();
        let codec_hex: String = codec.iter().map(|byte| format!("{byte:02x}")).collect();
        if let Ok(path) = std::env::var("WEFT_ORIGINAL_ROW_CAPTURE") {
            let crate::registered_access::Location::Row(location) = &row_accesses[0].location
            else {
                panic!("native fixture home")
            };
            let scan = crate::registered_access::scan_source(
                match &plan.root {
                    weft_core::ir::Node::Project { input, .. } => input,
                    _ => panic!("fixture projection"),
                },
                &row_accesses,
            )
            .unwrap();
            let sql = format!(
                "SELECT {} FROM {} WHERE {}",
                location
                    .scalar_observation()
                    .custody_projection()
                    .join(", "),
                scan.source.sql,
                scan.source.filters.join(" AND ")
            );
            std::fs::write(
                path,
                serde_json::to_vec_pretty(&json!({
                    "sql":sql,"parameters":row_parameters.clone().into_slots(),"codecHex":codec_hex,
                    "ownerTypeId":row_properties[&registration].owner_catalog_id,
                    "propertyId":row_properties[&registration].property_catalog_id,
                }))
                .unwrap(),
            )
            .unwrap();
        }
        let row_projection = crate::result_definition::property_projection(
            &row_properties[&registration],
            &row_accesses[0],
            1,
            "name",
        )
        .unwrap();
        assert!(row_projection.sql.contains("text_value"));
        assert!(!row_projection.sql.contains("props"));
        assert!(row_projection.payload_check_sql.contains("LEFT JOIN"));
        assert!(row_projection.payload_check_sql.contains(&codec_hex));
        if let Ok(path) = std::env::var("WEFT_NATIVE_ROW_PROJECTION_CAPTURE") {
            let scan = crate::registered_access::scan_source(
                match &plan.root {
                    weft_core::ir::Node::Project { input, .. } => input,
                    _ => panic!("fixture projection"),
                },
                &row_accesses,
            )
            .unwrap();
            std::fs::write(path,serde_json::to_vec_pretty(&json!({
                "sql":format!("SELECT {} FROM {} WHERE {}",row_projection.sql,scan.source.sql,scan.source.filters.join(" AND ")),
                "check":row_projection.payload_check_sql,"parameters":row_parameters.clone().into_slots(),
                "codecHex":codec_hex,"ownerTypeId":row_properties[&registration].owner_catalog_id,
                "propertyId":row_properties[&registration].property_catalog_id,
            })).unwrap()).unwrap();
        }
        let cells = [
            Some("true"),
            Some("string"),
            Some("é  "),
            None,
            None,
            None,
            Some(codec_hex.as_str()),
            Some("fe00"),
        ];
        let mut native_budget = crate::row_custody::Budget {
            remaining_bytes: 16384,
            remaining_cells: 16,
        };
        let selected_native = crate::row_custody::admit_property(
            &row_properties[&registration],
            &row_accesses[0],
            &cells,
            &mut native_budget,
        )
        .unwrap();
        assert!(
            matches!(selected_native.observation,crate::row_custody::Observation::Scalar { payload:crate::row_custody::Payload::Text(ref text),ref source_bytes,.. } if text=="é  " && source_bytes==&[254,0])
        );
        assert_eq!(
            selected_native.presence_bytes,
            row_properties[&registration]
                .value
                .presence
                .original_json
                .as_bytes()
        );
        assert!(crate::row_custody::admit_property(
            &properties[&registration],
            &row_accesses[0],
            &cells,
            &mut native_budget
        )
        .is_err());

        let changed_admission = Admission::parse(&changed_input.json, &input.profile).unwrap();
        let changed_property = admit_property(
            &changed_admission,
            index,
            &catalog,
            &descriptors,
            select(),
            physical(),
        )
        .unwrap();
        assert!(crate::result_definition::property_projection(
            &changed_property,
            &combined_self.accesses[0],
            1,
            "changed_cut"
        )
        .is_err());

        let read_payloads =
            crate::result_definition::read_payload_observations(&combined_self, &properties)
                .unwrap();
        assert_eq!(read_payloads.len(), 2);
        assert_ne!(read_payloads[0].scan, read_payloads[1].scan);
        assert!(read_payloads
            .iter()
            .all(|observation| observation.sql.contains("jsonb_typeof")
                && !observation.codec_bytes.is_empty()
                && !observation.presence_bytes.is_empty()));
        assert!(crate::result_definition::read_payload_observations(
            &combined_self,
            &std::collections::BTreeMap::new(),
        )
        .is_err());

        let projections: Vec<_> = combined_self
            .accesses
            .iter()
            .enumerate()
            .map(|(index, access)| {
                crate::result_definition::property_projection(
                    &properties[&registration],
                    access,
                    index + 1,
                    if index == 0 {
                        "left_name"
                    } else {
                        "right_name"
                    },
                )
                .unwrap()
            })
            .collect();
        assert!(projections[0].payload_check_sql.contains("jsonb_typeof"));
        assert!(!projections[0].payload_check_sql.contains("INNER JOIN"));
        assert_eq!(projections[0].column.position, 1);
        assert_eq!(projections[1].column.position, 2);
        assert!(!projections[0].codec_bytes.is_empty());
        assert!(!projections[0].presence_bytes.is_empty());

        let output_columns =
            crate::result_definition::projection_columns(&self_context, &properties, &comparisons)
                .unwrap();
        assert_eq!(output_columns.len(), 2);
        assert_eq!(output_columns[0].position, 1);
        assert_eq!(output_columns[0].output_name, "left_name");
        assert_eq!(output_columns[1].position, 2);
        assert_eq!(output_columns[1].output_name, "right_name");
        assert_eq!(
            output_columns[0].source_identities,
            output_columns[1].source_identities
        );
        let mut altered_projection = self_plan.clone();
        if let weft_core::ir::Node::Project { outputs, .. } = &mut altered_projection.root {
            if let weft_core::ir::Expression::Field { logical_type, .. } =
                &mut outputs[0].expression
            {
                logical_type.family = weft_core::ir::Family::Boolean;
            }
        }
        let altered_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V01(&altered_projection),
            ..self_context
        };
        assert!(crate::result_definition::projection_columns(
            &altered_context,
            &properties,
            &comparisons
        )
        .is_err());
        assert!(crate::result_definition::projection_columns(
            &self_context,
            &properties,
            &BTreeMap::new()
        )
        .is_err());

        if let Ok(path) = std::env::var("WEFT_SCALAR_PROJECTION_CAPTURE") {
            let native_source = &combined_self.scans[&combined_self.accesses[0].scan].source;
            let sql = format!(
                "SELECT {} FROM {} WHERE {}",
                projections[0].sql,
                native_source.sql,
                native_source.filters.join(" AND ")
            );
            std::fs::write(path, serde_json::to_vec_pretty(&json!({"sql":sql,"check":projections[0].payload_check_sql,"parameters":combined_self_parameters.clone().into_slots(),"column":projections[0].column,"codecSha256":sha256(&projections[0].codec_bytes),"presenceSha256":sha256(&projections[0].presence_bytes)})).unwrap()).unwrap();
        }
        if let Ok(path) = std::env::var("WEFT_COMBINED_CAPTURE") {
            let input = match &self_plan.root {
                weft_core::ir::Node::Project { input, .. } => input.as_ref(),
                _ => panic!("fixture project"),
            };
            let assembled = crate::relational::assemble_sources(
                input,
                &mut combined_self_parameters,
                |node, _| {
                    if let weft_core::ir::Node::Scan { occurrence, .. } = node {
                        Ok(combined_self.scans[occurrence].source.clone())
                    } else {
                        panic!("scan")
                    }
                },
                |expression, parameters| {
                    crate::registered_access::render_expression(
                        expression,
                        &combined_self.accesses,
                        parameters,
                        |node, operands, access, _| {
                            Ok(match node {
                                weft_core::ir::Expression::Field { .. } => access
                                    .unwrap()
                                    .scalar_storage
                                    .as_ref()
                                    .unwrap()
                                    .carrier
                                    .clone(),
                                weft_core::ir::Expression::Equal { .. } => {
                                    format!("({} = {})", operands[0], operands[1])
                                }
                                _ => panic!("fixture expression"),
                            })
                        },
                    )
                },
            )
            .unwrap();
            let sql = format!(
                "SELECT {} AS left_name, {} AS right_name FROM {} WHERE {}",
                combined_self.accesses[0]
                    .scalar_storage
                    .as_ref()
                    .unwrap()
                    .carrier,
                combined_self.accesses[1]
                    .scalar_storage
                    .as_ref()
                    .unwrap()
                    .carrier,
                assembled.sql,
                assembled.filters.join(" AND ")
            );
            let checks: Vec<_> = combined_self
                .scans
                .values()
                .flat_map(|scan| scan.structural_check_sql.iter())
                .collect();
            std::fs::write(path, serde_json::to_vec_pretty(&json!({"sql":sql,"structuralChecks":checks,"columns":output_columns,"parameters":combined_self_parameters.clone().into_slots(),"namespace":admitted.value["basis"]["namespace"]})).unwrap()).unwrap();
        }

        assert_eq!(combined_self_parameters.into_slots().len(), 4);
        let mut refused_combined = crate::Parameters::default();
        assert!(crate::registered_access::prepare(
            &self_context,
            &record_registry,
            &properties,
            &BTreeMap::new(),
            &mut refused_combined
        )
        .is_err());
        assert!(refused_combined.into_slots().is_empty());
        let mut self_parameters = crate::Parameters::default();
        let self_accesses = crate::registered_access::lower(
            &self_context,
            &properties,
            &comparisons,
            &self_requests,
            &mut self_parameters,
        )
        .unwrap();
        let mut automatic_parameters = crate::Parameters::default();
        let automatic = crate::registered_access::lower_plan(
            &self_context,
            &properties,
            &comparisons,
            &mut automatic_parameters,
        )
        .unwrap();
        assert_eq!(automatic.len(), 2);
        let join_expression = match &self_plan.root {
            weft_core::ir::Node::Project { input, .. } => match input.as_ref() {
                weft_core::ir::Node::InnerJoin { on, .. } => on,
                _ => panic!("fixture join"),
            },
            _ => panic!("fixture project"),
        };
        let emitted = crate::registered_access::render_expression(
            join_expression,
            &automatic,
            &mut automatic_parameters,
            |node, operands, access, _| {
                Ok(match node {
                    weft_core::ir::Expression::Field { .. } => access
                        .unwrap()
                        .scalar_storage
                        .as_ref()
                        .unwrap()
                        .carrier
                        .clone(),
                    weft_core::ir::Expression::Equal { .. } => {
                        assert!(access.is_none());
                        format!("({} = {})", operands[0], operands[1])
                    }
                    _ => panic!("fixture expression"),
                })
            },
        )
        .unwrap();
        assert!(emitted.contains("\"weft_scan_0\".\"props\""));
        assert!(emitted.contains("\"weft_scan_1\".\"props\""));
        let input = match &self_plan.root {
            weft_core::ir::Node::Project { input, .. } => input.as_ref(),
            _ => panic!("fixture project"),
        };
        let mut integrity = Vec::new();
        let mut structural_checks = Vec::new();
        let assembled = crate::relational::assemble_sources(
            input,
            &mut automatic_parameters,
            |node, _| {
                let scan = crate::registered_access::scan_source(node, &automatic)?;
                assert_eq!(scan.source.filters.len(), 1);
                assert_eq!(scan.structural_integrity.len(), 1);
                assert_eq!(scan.structural_check_sql.len(), 1);
                assert!(scan.structural_check_sql[0].ends_with("IS DISTINCT FROM TRUE"));
                assert!(!scan.structural_check_sql[0].contains("INNER JOIN"));
                structural_checks.extend(scan.structural_check_sql);
                integrity.extend(scan.structural_integrity);
                Ok(scan.source)
            },
            |expression, parameters| {
                crate::registered_access::render_expression(
                    expression,
                    &automatic,
                    parameters,
                    |node, operands, access, _| {
                        Ok(match node {
                            weft_core::ir::Expression::Field { .. } => access
                                .unwrap()
                                .scalar_storage
                                .as_ref()
                                .unwrap()
                                .carrier
                                .clone(),
                            weft_core::ir::Expression::Equal { .. } => {
                                format!("({} = {})", operands[0], operands[1])
                            }
                            _ => panic!("fixture expression"),
                        })
                    },
                )
            },
        )
        .unwrap();
        assert_eq!(assembled.filters.len(), 2);
        assert_eq!(integrity.len(), 2);
        assert!(assembled
            .filters
            .iter()
            .all(|filter| filter.contains("\"type_id\"") && !filter.contains("jsonb_typeof")));
        assert!(assembled.sql.contains("INNER JOIN"));
        let first_scan = match input {
            weft_core::ir::Node::InnerJoin { left, .. } => left.as_ref(),
            _ => panic!("fixture join"),
        };
        assert!(crate::registered_access::scan_source(first_scan, &automatic[1..]).is_err());
        let mut changed_scan = first_scan.clone();
        if let weft_core::ir::Node::Scan { record, .. } = &mut changed_scan {
            record.element = "another-owner".into();
        }
        assert!(crate::registered_access::scan_source(&changed_scan, &automatic).is_err());
        if let Ok(path) = std::env::var("WEFT_OWNER_SCAN_CAPTURE") {
            let sql = format!(
                "SELECT {} AS left_name, {} AS right_name FROM {} WHERE {}",
                automatic[0].scalar_storage.as_ref().unwrap().carrier,
                automatic[1].scalar_storage.as_ref().unwrap().carrier,
                assembled.sql,
                assembled.filters.join(" AND ")
            );
            std::fs::write(path, serde_json::to_vec(&json!({"sql":sql,"parameters":automatic_parameters.clone().into_slots(),"namespace":admitted.value["basis"]["namespace"],"integrity":integrity,"structuralChecks":structural_checks})).unwrap()).unwrap();
        }

        let mut rejected_parameters = crate::Parameters::default();
        assert!(crate::registered_access::render_expression(
            join_expression,
            &automatic[..1],
            &mut rejected_parameters,
            |_, _, _, parameters| parameters.push(
                weft_core::ir::LogicalType {
                    family: weft_core::ir::Family::String,
                    facets: json!({}),
                    nullable: false
                },
                "first-location".into(),
                json!({"use":"test-native-access"}),
            ),
        )
        .is_err());
        assert!(rejected_parameters.into_slots().is_empty());
        assert_eq!(automatic_parameters.into_slots().len(), 4);
        assert_eq!(self_accesses.len(), 2);
        assert!(std::sync::Arc::ptr_eq(
            &self_accesses[0].value_layout,
            &self_accesses[1].value_layout
        ));
        assert_ne!(self_accesses[0].owner_alias, self_accesses[1].owner_alias);
        assert_eq!(self_parameters.into_slots().len(), 4);
        let changed_context = weft_core::backend::Context {
            binding: &changed_input,
            binding_value: &changed,
            ..context
        };
        assert!(crate::comparator_requirements::admit_context(
            &changed_context,
            &properties,
            &comparisons
        )
        .is_err());
        assert!(admit_properties(&requirements, &BTreeMap::new(), &comparisons).is_err());
        let mut wrong_artifact = value_artifact.clone();
        wrong_artifact["identity"] = json!("another-original-graph");
        let wrong = BTreeMap::from([(
            registration.clone(),
            make_comparator(wrong_artifact, &graph_bytes, &pin),
        )]);
        assert!(admit_properties(&requirements, &properties, &wrong).is_err());
        let mut native_profile = pin.clone();
        native_profile["identity"] = json!("another-native-domain");
        let wrong = BTreeMap::from([(
            registration,
            make_comparator(value_artifact, &graph_bytes, &native_profile),
        )]);
        assert!(admit_properties(&requirements, &properties, &wrong).is_err());
        let empty_columns = BTreeMap::new();
        let mut missing = physical();
        missing.columns = &empty_columns;
        assert!(admit_home(&admitted, index, missing).is_err());
        let foreign_inventory = leaf_codec_definition::OriginalArtifact {
            identity: inventory.identity.clone(),
            bytes: b"changed".to_vec(),
        };
        let mut missing = physical();
        missing.inventory = &foreign_inventory;
        assert!(admit_home(&admitted, index, missing).is_err());

        assert!(admit_value(&admitted, index, &catalog, &[], select()).is_err());
        let mut wrong_binding = binding.clone();
        wrong_binding["properties"][index]["source"] = artifact("foreign-source", b"{}");
        let wrong_binding = Admission::parse(
            &wrong_binding.to_string(),
            binding["bindingProfileId"].as_str().unwrap(),
        )
        .unwrap();
        assert!(admit_value(&wrong_binding, index, &catalog, &descriptors, select()).is_err());
        let mut wrong = descriptors.clone();
        wrong[0].shape = Shape::Sequence {
            item: member.identity,
        };
        assert!(admit_value(&admitted, index, &catalog, &wrong, select()).is_err());
    }
    #[test]
    fn native_home_cannot_enter_props_path_without_original_join() {
        let binding: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/binding-row.json"
        ))
        .unwrap();
        let admitted = Admission::parse(
            &binding.to_string(),
            binding["bindingProfileId"].as_str().unwrap(),
        )
        .unwrap();
        let inventory = leaf_codec_definition::OriginalArtifact {
            identity: binding["basis"]["layoutInventory"]["identity"]
                .as_str()
                .unwrap()
                .into(),
            bytes: b"{}".to_vec(),
        };
        assert!(admit_home(
            &admitted,
            0,
            PhysicalSelection {
                profile: &binding["properties"][0]["homeProfile"],
                inventory: &inventory,
                relations: &BTreeMap::new(),
                columns: &BTreeMap::new(),
                row_join: None,
                obligations: &BTreeSet::new(),
                edge_association: None
            }
        )
        .is_err());
    }
}
