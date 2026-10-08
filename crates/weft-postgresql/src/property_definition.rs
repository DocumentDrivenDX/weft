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
    pub(crate) admitted_record_presence: BTreeMap<String, presence_definition::Definition>,
    admitted_leaf_codecs: BTreeMap<String, leaf_codec_definition::Definition>,
    pub definition_artifact: Value,
    pub graph: Graph,
    pub presence: presence_definition::Definition,
    pub(crate) descriptors: Vec<Descriptor>,
}
impl ValueAdmission {
    pub(crate) fn verify_leaf_codec_custody(&self) -> Result<()> {
        let nodes = self.graph.value["nodes"]
            .as_array()
            .ok_or_else(|| fail("Original codec graph nodes missing"))?;
        for (index, node) in nodes.iter().enumerate() {
            if node["shape"]["kind"] != "scalar" {
                continue;
            }
            let codec = self
                .admitted_leaf_codecs
                .get(
                    node["nodeId"]
                        .as_str()
                        .ok_or_else(|| fail("Original scalar node identity missing"))?,
                )
                .ok_or_else(|| fail("Original scalar lacks captured codec"))?;
            if self
                .graph
                .artifacts
                .get(&format!("/nodes/{index}/codecDefinition"))
                .map(Vec::as_slice)
                != Some(codec.original_json.as_bytes())
            {
                return Err(fail(
                    "Original scalar codec custody differs from captured meaning",
                ));
            }
        }
        Ok(())
    }
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
    pub(crate) native_tree: Option<crate::row_tree_mapping::Procedures>,
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
    pub fn with_native_tree(
        mut self,
        procedures: crate::row_tree_mapping::Procedures,
    ) -> Result<Self> {
        if !matches!(&self.home,HomeAdmission::Row {access,..} if access=="complete-value-tree") {
            return Err(fail(
                "Native tree procedures require admitted complete-value-tree home",
            ));
        }
        crate::result_definition::property_column(&self, 1, "native_tree_selection")?;
        self.value
            .graph
            .verify_record_presence(&self.value.admitted_record_presence)?;
        self.value.verify_leaf_codec_custody()?;
        self.native_tree = Some(procedures);
        Ok(self)
    }

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
        native_tree: None,
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
        admitted_record_presence: selected.record_presence.clone(),
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
    fn original_decimal_property_compiles_comparator_owned_sum() {
        original_numeric_property_fixture(
            "Orders",
            "total",
            "decimal",
            json!({"kind":"finite-decimal","nativeType":"pg_catalog.numeric","scaleCoercion":"forbidden","nonfinite":"refuse"}),
            "WEFT_ORIGINAL_NUMERIC_CAPTURE",
        );
    }
    #[test]
    fn original_unsigned_property_compiles_comparator_owned_sum() {
        original_numeric_property_fixture(
            "Customer",
            "id",
            "integer",
            json!({"kind":"unsigned-integer","nativeType":"pg_catalog.numeric","integrality":"validate-before-cast","range":"original-authored-unsigned-facets"}),
            "WEFT_ORIGINAL_INTEGER_CAPTURE",
        );
    }
    fn original_numeric_property_fixture(
        record_name: &str,
        member_name: &str,
        family: &str,
        strategy: Value,
        capture: &str,
    ) {
        use base64::{engine::general_purpose::STANDARD, Engine};
        use leaf_codec_definition::{
            Definition as Leaf, OriginalArtifact, Selection as LeafSelection,
        };
        use weft_core::json::sha256;
        let cases: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/application-cases.json"
        ))
        .unwrap();
        let inputs = serde_json::from_value(cases[0]["request"]["modules"].clone()).unwrap();
        let catalog = Catalog::prepare(inputs).unwrap();
        let name = |value: &str| Name {
            value: value.to_ascii_lowercase(),
            quoted: false,
            span: Span { start: 0, end: 0 },
        };
        let record = catalog.record(None, &name(record_name)).unwrap();
        let (member, descriptors) = catalog
            .member_descriptor(&record, &name(member_name))
            .unwrap();
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
        let leaf = json!({"interfaceVersion":"truss-jsonb-leaf-codec/0.1.0","profile":pin,"authoredDefinition":authored,"sourceInterpretationProfile":pin,"sourceInterpretationDefinition":empty,"nativeDomainProfile":pin,"nativeDomainDefinition":empty,"rule":{"family":family,"storageRepresentation":"json-string","encoding":"preserve-admitted-source-token","decodedCarrierKind":family,"numericAdoptionEvidence":empty},"coercion":"none","readDefault":"none","invalidStoredValue":"complete-result-refusal"});
        let mut originals = BTreeMap::from([
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
        originals.insert(
            "rule/numericAdoptionEvidence".into(),
            OriginalArtifact {
                identity: "fixture".into(),
                bytes: b"{}".to_vec(),
            },
        );
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
        let graph = json!({"interfaceVersion":"truss-value-definition/0.1.0","profile":pin,"rootNodeId":"root","acceptedDefinition":authored,"nodes":[{"nodeId":"root","authoredIdentity":member.identity,"authoredDefinition":authored,"codecProfile":pin,"codecDefinition":artifact("selected-leaf",leaf.original_json.as_bytes()),"shape":{"kind":"scalar","family":family,"storageRepresentation":"json-string"}}]});
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
            let comparator = json!({"interfaceVersion":"truss-native-comparator/0.1.0","profile":pin,"valueDefinition":value_artifact,"sourceDomainDefinition":authored,"nativeDomainProfile":native_profile,"nativeDomainDefinition":empty,"operatorInventory":empty,"strategy":strategy,"castOutcome":"exact-or-error","nullOperands":"refuse","absentOperands":"refuse","qualification":empty});
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
                    operations: &BTreeSet::from([
                        Operation::Sum,
                        Operation::Equality,
                        Operation::Ordering,
                        Operation::Key,
                    ]),
                },
                &logical,
            )
            .unwrap()
        };
        let registration = registration_key(&record.identity, &member.identity);
        let comparisons = BTreeMap::from([(
            registration.clone(),
            make_comparator(value_artifact, &graph_bytes, &pin),
        )]);
        let properties = BTreeMap::from([(registration.clone(), property)]);
        let (_, plan) = weft_core::prepare_and_resolve(
            &format!("SELECT SUM(o.{member_name}) AS total FROM {record_name} o"),
            catalog.inputs.clone(),
        )
        .unwrap();
        let manifest = <crate::candidate::Candidate as weft_core::backend::Backend>::describe(
            &crate::candidate::Candidate,
        )
        .unwrap();
        let input = weft_core::backend::BindingInput {
            profile: binding["bindingProfileId"].as_str().unwrap().into(),
            json: admitted.original_json.clone(),
            sha256: sha256(admitted.original_json.as_bytes()),
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
        let context = weft_core::backend::Context {
            catalog: &catalog,
            plan: weft_core::backend::Plan::V01(&plan),
            target: &manifest.target_profiles[0],
            binding: &input,
            binding_value: &admitted.value,
            selection: &weft_core::backend::Selection::default(),
        };
        let compiled = crate::select_definition::compile_with_registry(
            &context,
            &record_registry,
            &properties,
            &comparisons,
            |node, _, access, _| match node {
                weft_core::ir::Expression::Field { .. } => Ok(access
                    .unwrap()
                    .scalar_storage
                    .as_ref()
                    .unwrap()
                    .carrier
                    .clone()),
                _ => panic!("SUM must be comparator-owned"),
            },
        )
        .unwrap();
        assert!(compiled.select.sql.contains("pg_catalog.sum"));
        let (scan, result_type) = match &plan.root {
            weft_core::ir::Node::Project { input, outputs } => {
                let scan = match input.as_ref() {
                    weft_core::ir::Node::Aggregate { input, .. } => match input.as_ref() {
                        weft_core::ir::Node::Scan {
                            occurrence,
                            record,
                            pin,
                        } => weft_core::application_ir::Scan {
                            occurrence: occurrence.clone(),
                            record: record.clone(),
                            pin: pin.clone(),
                        },
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                };
                (scan, outputs[0].expression.logical_type().clone())
            }
            _ => unreachable!(),
        };
        let application = weft_core::application_ir::Plan {
            ir_version: "weft-ir/0.2.0".into(),
            module_pins: plan.module_pins.clone(),
            read_profile: None,
            required_capabilities: vec!["scan".into(), "project".into(), "sum".into()],
            type_graph: descriptors.clone(),
            outputs: vec![weft_core::application_ir::Output {
                name: "total".into(),
                expression: weft_core::application_ir::Expression::Sum {
                    argument: weft_core::application_ir::Field {
                        scan: scan.occurrence.clone(),
                        identity: member.identity.clone(),
                        logical_type: logical.clone(),
                        span: Span { start: 0, end: 0 },
                    },
                    logical_type: result_type,
                },
            }],
            source: scan,
            page_key: None,
            joins: vec![],
            filters: vec![],
            groups: vec![],
            aggregate: true,
            order: vec![],
            limit: None,
        };
        let context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&application),
            ..context
        };
        let application_compiled = crate::select_definition::compile_with_registry(
            &context,
            &record_registry,
            &properties,
            &comparisons,
            |node, _, access, _| match node {
                weft_core::ir::Expression::Field { .. } => Ok(access
                    .unwrap()
                    .scalar_storage
                    .as_ref()
                    .unwrap()
                    .carrier
                    .clone()),
                _ => panic!("V02 SUM must be comparator-owned"),
            },
        )
        .unwrap();
        assert_eq!(compiled.select.sql, application_compiled.select.sql);
        assert_eq!(
            serde_json::to_value(&compiled.select.columns).unwrap(),
            serde_json::to_value(&application_compiled.select.columns).unwrap()
        );
        fn scalar_native(
            node: &weft_core::ir::Expression,
            _: &[String],
            access: Option<&crate::registered_access::Access<'_>>,
            parameters: &mut crate::Parameters,
        ) -> weft_core::error::Result<String> {
            match node {
                weft_core::ir::Expression::Field { .. } => Ok(format!(
                    "({})::pg_catalog.numeric",
                    access.unwrap().scalar_storage.as_ref().unwrap().carrier
                )),
                weft_core::ir::Expression::Literal {
                    value,
                    logical_type,
                    span,
                } => Ok(format!(
                    "{}::pg_catalog.numeric",
                    parameters.push(
                        logical_type.clone(),
                        value.clone(),
                        json!({"literalSpan":span})
                    )?
                )),
                _ => panic!("Public SUM must be comparator-owned"),
            }
        }
        if family == "integer" {
            let page = weft_core::application_resolve::resolve(
                &catalog,
                weft_core::application_syntax::parse(
                    "SELECT c.id FROM Customer c ORDER BY c.id LIMIT 2",
                )
                .unwrap(),
                BTreeMap::new(),
                Some(weft_core::application_ir::ReadProfile {
                    version: "weft-application-read/0.2.0".into(),
                    subset: weft_core::application_ir::Subset::EntityPage,
                }),
            )
            .unwrap();
            let page_context = weft_core::backend::Context {
                plan: weft_core::backend::Plan::V02(&page),
                ..context
            };
            let page_compiled = crate::select_definition::compile_with_registry(
                &page_context,
                &record_registry,
                &properties,
                &comparisons,
                |node, _, access, _| match node {
                    weft_core::ir::Expression::Field { .. } => Ok(format!(
                        "({})::pg_catalog.numeric",
                        access.unwrap().scalar_storage.as_ref().unwrap().carrier
                    )),
                    _ => panic!("Page fixture field operation"),
                },
            )
            .unwrap();
            assert!(page.page_key.is_some());
            if let Ok(path) = std::env::var("WEFT_ORIGINAL_PAGE_CAPTURE") {
                std::fs::write(path,serde_json::to_vec_pretty(&json!({"sql":page_compiled.select.sql,"columns":page_compiled.select.columns,"checks":page_compiled.select.structural_checks.iter().cloned().chain(page_compiled.select.payload_checks.iter().map(|check|check.sql.clone())).collect::<Vec<_>>(),"parameters":page_compiled.parameters})).unwrap()).unwrap();
            }
        }
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
        let row_record = crate::record_definition::RecordAdmission::admit(
            &row_admission,
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
        row_record.verify_property(&row_property).unwrap();
        let row_records = BTreeMap::from([(
            serde_json::to_string(row_record.identity()).unwrap(),
            row_record,
        )]);
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

        fn row_numeric_native(
            node: &weft_core::ir::Expression,
            _: &[String],
            access: Option<&crate::registered_access::Access<'_>>,
            parameters: &mut crate::Parameters,
        ) -> weft_core::error::Result<String> {
            if let weft_core::ir::Expression::Literal {
                value,
                logical_type,
                span,
            } = node
            {
                return Ok(format!(
                    "{}::pg_catalog.numeric",
                    parameters.push(
                        logical_type.clone(),
                        value.clone(),
                        json!({"literalSpan":span})
                    )?
                ));
            }
            let weft_core::ir::Expression::Field { .. } = node else {
                return Err(weft_core::error::Diagnostic::new(
                    "WFT-CAPABILITY",
                    "lower",
                    "Numeric row fixture only selects field operands",
                ));
            };
            let crate::registered_access::Location::Row(location) = &access.unwrap().location
            else {
                unreachable!()
            };
            Ok(format!(
                "({})::pg_catalog.numeric",
                location.scalar_observation().native_numeric_text
            ))
        }
        let row_sum = crate::select_definition::compile_with_registry(
            &row_context,
            &row_records,
            &row_properties,
            &comparisons,
            row_numeric_native,
        )
        .unwrap();
        assert!(row_sum.select.sql.contains("pg_catalog.sum"));
        assert_eq!(
            serde_json::to_value(&row_sum.select.columns).unwrap(),
            serde_json::to_value(&application_compiled.select.columns).unwrap()
        );
        for scope in [0, 1] {
            let mut scoped_parameters = crate::Parameters::default();
            let target = crate::registered_access::prepare_relationship_target(
                &row_context,
                &row_records,
                &row_properties,
                &comparisons,
                &mut scoped_parameters,
                scope,
            )
            .unwrap();
            let access = &target.accesses[0];
            assert_eq!(
                access.owner_alias.sql(),
                format!("\"weft_related_{scope}_scan_0\"")
            );
            let crate::registered_access::Location::Row(location) = &access.location else {
                unreachable!()
            };
            let physical = &target.scans[&application.source.occurrence];
            assert!(!physical.source.sql.contains("\"weft_scan_0\""));
            assert!(physical.source.sql.contains(&format!(
                "\"weft_state_{}\"",
                (usize::from(scope) + 1) * 1024
            )));
            target
                .verify_context(&row_context, &scoped_parameters)
                .unwrap();
            if let Ok(path) = std::env::var(format!("{capture}_SCOPED_{scope}")) {
                let alias = access.owner_alias.sql();
                let sql=format!("SELECT o.id::pg_catalog.text AS owner, {} AS value FROM pg_temp.object o JOIN {} ON {alias}.id=o.id AND {alias}.type_id=o.type_id WHERE {} ORDER BY o.id",location.scalar_observation().native_numeric_text,physical.source.sql,physical.source.filters.join(" AND "));
                std::fs::write(path,serde_json::to_vec_pretty(&json!({"sql":sql,"checks":physical.structural_check_sql,"parameters":scoped_parameters.into_slots(),"codecHex":leaves["root"].original_json.as_bytes().iter().map(|b|format!("{b:02x}")).collect::<String>()})).unwrap()).unwrap();
            }
        }
        if family == "integer" {
            let name = Name {
                value: "Orders".into(),
                quoted: true,
                span: Span { start: 0, end: 0 },
            };
            let from = catalog.record(None, &name).unwrap();
            let read = catalog
                .relationship_read(
                    &from,
                    &Name {
                        value: "customer".into(),
                        quoted: true,
                        span: Span { start: 0, end: 0 },
                    },
                )
                .unwrap();
            let endpoint_records: BTreeMap<_, _> = (0..2)
                .map(|i| {
                    let r = crate::record_definition::RecordAdmission::admit(
                        &row_admission,
                        i,
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
                    (serde_json::to_string(r.identity()).unwrap(), r)
                })
                .collect();
            let edge_relations = BTreeMap::from([("edge".into(), "edge".into())]);
            let edge_columns: BTreeMap<_, _> = [
                "rel_type_id",
                "source_id",
                "source_type",
                "target_id",
                "target_type",
            ]
            .into_iter()
            .map(|n| {
                (
                    n.into(),
                    crate::row_join_definition::Column {
                        relation_identity: "edge".into(),
                        name: n.into(),
                    },
                )
            })
            .collect();
            let relationship = crate::relationship_definition::RelationshipAdmission::admit(
                &row_admission,
                0,
                &catalog,
                &read,
                &endpoint_records,
                crate::relationship_definition::Selection {
                    profile: &row_binding["relationships"][0]["relationshipProfile"],
                    inventory: &inventory,
                    relation_identity: "edge",
                    relations: &edge_relations,
                    columns: &edge_columns,
                    relationship_type: "rel_type_id",
                    source_id: "source_id",
                    source_type: "source_type",
                    target_id: "target_id",
                    target_type: "target_type",
                },
            )
            .unwrap();
            let mut parameters = crate::Parameters::default();
            let target = relationship
                .prepare_target(
                    &row_context,
                    &read,
                    &endpoint_records,
                    &row_properties,
                    &comparisons,
                    &mut parameters,
                    2,
                )
                .unwrap();
            assert_eq!(target.plan.source.record, read.to);
            assert_eq!(target.plan.outputs.len(), read.target_key.fields.len());
            assert_eq!(
                target.prepared.accesses[0].owner_alias.sql(),
                "\"weft_related_2_scan_0\""
            );
            let target_context = weft_core::backend::Context {
                plan: weft_core::backend::Plan::V02(&target.plan),
                ..row_context
            };
            target
                .prepared
                .verify_context(&target_context, &parameters)
                .unwrap();
            let key = vec![weft_core::application_ir::Value::Parameter {
                name: "customer_id".into(),
                value: "9007199254740993".into(),
                logical_type: logical.clone(),
                span: Span { start: 0, end: 0 },
            }];
            let mut captured = Vec::new();
            for (kind, operation) in [
                ("has", crate::relationship_lowering::Read::HasRelated(&key)),
                ("keys", crate::relationship_lowering::Read::RelatedKeys(2)),
            ] {
                let mut params = crate::Parameters::default();
                let lowered = crate::relationship_lowering::lower(
                    &relationship,
                    &row_context,
                    &read,
                    &crate::Identifier::new("o").unwrap(),
                    operation,
                    &endpoint_records,
                    &row_properties,
                    &comparisons,
                    4,
                    &mut params,
                    row_numeric_native,
                )
                .unwrap();
                assert!(!lowered.structural_checks.is_empty());
                let checks: Vec<_> = lowered
                    .structural_checks
                    .iter()
                    .cloned()
                    .chain(lowered.payload_checks.iter().map(|c| c.sql.clone()))
                    .collect();
                captured.push(json!({"kind":kind,"sql":format!("SELECT o.id::pg_catalog.text AS owner, {} AS related FROM pg_temp.object o WHERE o.type_id=-2 ORDER BY o.id",lowered.expression),"checks":checks,"parameters":params.into_slots(),"codecHex":leaves["root"].original_json.as_bytes().iter().map(|b|format!("{b:02x}")).collect::<String>()}));
            }
            if let Ok(path) = std::env::var("WEFT_ORIGINAL_RELATIONSHIP_LOWERING_CAPTURE") {
                std::fs::write(path, serde_json::to_vec_pretty(&captured).unwrap()).unwrap();
            }
            let mut failed = crate::Parameters::default();
            assert!(crate::relationship_lowering::lower(
                &relationship,
                &row_context,
                &read,
                &crate::Identifier::new("o").unwrap(),
                crate::relationship_lowering::Read::RelatedKeys(0),
                &endpoint_records,
                &row_properties,
                &comparisons,
                4,
                &mut failed,
                row_numeric_native
            )
            .is_err());
            assert!(failed.clone().into_slots().is_empty());
            assert!(crate::relationship_lowering::lower(
                &relationship,
                &row_context,
                &read,
                &crate::Identifier::new("o").unwrap(),
                crate::relationship_lowering::Read::HasRelated(&[]),
                &endpoint_records,
                &row_properties,
                &comparisons,
                4,
                &mut failed,
                row_numeric_native
            )
            .is_err());
            assert!(failed.into_slots().is_empty());
            // Admit both endpoint key properties at the final binding cut.
            let source_identity = read.source_key.fields[0].clone();
            let source_index = row_binding["properties"]
                .as_array()
                .unwrap()
                .iter()
                .position(|p| p["logical"] == json!(source_identity))
                .unwrap();
            let source_authored =
                row_binding["properties"][source_index]["acceptedDefinition"].clone();
            let source_bytes = STANDARD
                .decode(source_authored["bytesBase64"].as_str().unwrap())
                .unwrap();
            let mut source_leaf_json: Value =
                serde_json::from_str(&leaves["root"].original_json).unwrap();
            source_leaf_json["authoredDefinition"] = source_authored.clone();
            let mut source_originals = originals.clone();
            source_originals.insert(
                "authoredDefinition".into(),
                OriginalArtifact {
                    identity: source_authored["identity"].as_str().unwrap().into(),
                    bytes: source_bytes.clone(),
                },
            );
            let source_leaf = Leaf::parse(
                &source_leaf_json.to_string(),
                LeafSelection {
                    profile: &pin,
                    source_profile: &pin,
                    native_profile: &pin,
                    original_artifacts: &source_originals,
                },
            )
            .unwrap();
            let mut source_graph = graph.clone();
            source_graph["acceptedDefinition"] = source_authored.clone();
            source_graph["nodes"][0]["authoredIdentity"] = json!(source_identity);
            source_graph["nodes"][0]["authoredDefinition"] = source_authored.clone();
            source_graph["nodes"][0]["codecDefinition"] =
                artifact("selected-source-leaf", source_leaf.original_json.as_bytes());
            let mut source_presence = presence.clone();
            source_presence["acceptedDefinition"] = source_authored.clone();
            let mut final_binding = row_binding.clone();
            final_binding["properties"][source_index]["valueDefinition"] =
                artifact("original-source-graph", source_graph.to_string().as_bytes());
            final_binding["properties"][source_index]["presenceDefinition"] = artifact(
                "original-source-presence",
                source_presence.to_string().as_bytes(),
            );
            let mut source_home = row_home.clone();
            source_home["ownerCatalogId"] =
                final_binding["properties"][source_index]["ownerTypeId"].clone();
            source_home["propertyCatalogId"] =
                final_binding["properties"][source_index]["propertyId"].clone();
            source_home["valueDefinition"] =
                final_binding["properties"][source_index]["valueDefinition"].clone();
            source_home["presenceDefinition"] =
                final_binding["properties"][source_index]["presenceDefinition"].clone();
            final_binding["properties"][source_index]["home"] = json!("row");
            final_binding["properties"][source_index]["homeDefinition"] =
                artifact("original-source-home", source_home.to_string().as_bytes());
            let final_admission =
                Admission::parse(&final_binding.to_string(), &row_input.profile).unwrap();
            let final_records: BTreeMap<_, _> = (0..2)
                .map(|i| {
                    let r = crate::record_definition::RecordAdmission::admit(
                        &final_admission,
                        i,
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
                    (serde_json::to_string(r.identity()).unwrap(), r)
                })
                .collect();
            let target_property = admit_property(
                &final_admission,
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
            let source_leaves = BTreeMap::from([("root".into(), source_leaf)]);
            let (_, source_descriptors) = catalog
                .member_descriptor_by_identity(&from, &source_identity)
                .unwrap();
            let source_property = admit_property(
                &final_admission,
                source_index,
                &catalog,
                &source_descriptors,
                Selection {
                    value_profile: &pin,
                    presence_profile: &pin,
                    leaf_codecs: &source_leaves,
                    record_presence: &records,
                },
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
            let source_registration = registration_key(&from.identity, &source_identity);
            let mut source_cmp_json: Value =
                serde_json::from_str(&comparisons[&registration].original_json).unwrap();
            source_cmp_json["valueDefinition"] =
                final_binding["properties"][source_index]["valueDefinition"].clone();
            source_cmp_json["sourceDomainDefinition"] = source_authored.clone();
            let source_cmp_originals: BTreeMap<_, _> = [
                "valueDefinition",
                "sourceDomainDefinition",
                "nativeDomainDefinition",
                "operatorInventory",
                "qualification",
            ]
            .into_iter()
            .map(|role| {
                let bytes = STANDARD
                    .decode(source_cmp_json[role]["bytesBase64"].as_str().unwrap())
                    .unwrap();
                (
                    role.into(),
                    OriginalArtifact {
                        identity: source_cmp_json[role]["identity"].as_str().unwrap().into(),
                        bytes,
                    },
                )
            })
            .collect();
            let source_comparator = Comparator::parse(
                &source_cmp_json.to_string(),
                ComparatorSelection {
                    profile: &pin,
                    native_profile: &pin,
                    original_artifacts: &source_cmp_originals,
                    operations: &BTreeSet::from([
                        Operation::Equality,
                        Operation::Key,
                        Operation::Ordering,
                        Operation::Sum,
                    ]),
                },
                &logical,
            )
            .unwrap();
            let mut final_comparators = comparisons.clone();
            final_comparators.insert(source_registration.clone(), source_comparator);
            let final_properties = BTreeMap::from([
                (registration.clone(), target_property),
                (source_registration, source_property),
            ]);
            let final_relationship = crate::relationship_definition::RelationshipAdmission::admit(
                &final_admission,
                0,
                &catalog,
                &read,
                &final_records,
                crate::relationship_definition::Selection {
                    profile: &final_binding["relationships"][0]["relationshipProfile"],
                    inventory: &inventory,
                    relation_identity: "edge",
                    relations: &edge_relations,
                    columns: &edge_columns,
                    relationship_type: "rel_type_id",
                    source_id: "source_id",
                    source_type: "source_type",
                    target_id: "target_id",
                    target_type: "target_type",
                },
            )
            .unwrap();
            let customer = catalog
                .record(
                    None,
                    &Name {
                        value: "Customer".into(),
                        quoted: true,
                        span: Span { start: 0, end: 0 },
                    },
                )
                .unwrap();
            let forward_read = catalog
                .relationship_read(
                    &customer,
                    &Name {
                        value: "orders".into(),
                        quoted: true,
                        span: Span { start: 0, end: 0 },
                    },
                )
                .unwrap();
            let forward_relationship =
                crate::relationship_definition::RelationshipAdmission::admit(
                    &final_admission,
                    0,
                    &catalog,
                    &forward_read,
                    &final_records,
                    crate::relationship_definition::Selection {
                        profile: &final_binding["relationships"][0]["relationshipProfile"],
                        inventory: &inventory,
                        relation_identity: "edge",
                        relations: &edge_relations,
                        columns: &edge_columns,
                        relationship_type: "rel_type_id",
                        source_id: "source_id",
                        source_type: "source_type",
                        target_id: "target_id",
                        target_type: "target_type",
                    },
                )
                .unwrap();
            let final_input = weft_core::backend::BindingInput {
                profile: row_input.profile.clone(),
                json: final_admission.original_json.clone(),
                sha256: sha256(final_admission.original_json.as_bytes()),
            };
            let backend = crate::original_backend::OriginalBackend::new(
                &final_input,
                final_records,
                final_properties,
                final_comparators,
                row_numeric_native,
            )
            .unwrap()
            .with_relationships(BTreeMap::from([
                (
                    crate::relationship_definition::registration_key(&read),
                    final_relationship,
                ),
                (
                    crate::relationship_definition::registration_key(&forward_read),
                    forward_relationship,
                ),
            ]))
            .unwrap();
            let mut registry = weft_core::backend::Registry::default();
            registry.register(backend).unwrap();
            let target = weft_core::backend::Target {
                backend_id: "truss.postgresql.original".into(),
                backend_version: "0.1.0-candidate".into(),
                profile_id: "pg17.9-candidate".into(),
                allow_candidate: true,
            };
            let compiler = weft_core::compile::Compiler { registry };
            let mut transport_captures = Vec::new();
            let mut public_captures = Vec::new();
            let mut forward_captures = Vec::new();
            for (direction,kind,sql) in [("inverse","has","SELECT COUNT(*) AS n FROM Orders o WHERE HAS_RELATED(o.customer, KEY(:customer_id))"),("inverse","keys","SELECT o.id, RELATED_KEYS(o.customer,2) AS customers FROM Orders o ORDER BY o.id LIMIT 10"),("forward","has","SELECT COUNT(*) AS n FROM Customer c WHERE HAS_RELATED(c.orders, KEY(:order_id))"),("forward","keys","SELECT c.id, RELATED_KEYS(c.orders,2) AS orders FROM Customer c ORDER BY c.id LIMIT 10")] {
                let parameter_name=if direction=="forward" {"order_id"} else {"customer_id"};
                let parameter_value=if direction=="forward" {"100"} else {"9007199254740993"};
                let params=if kind=="has" {BTreeMap::from([(parameter_name.into(),weft_core::application_resolve::Parameter {family:weft_core::ir::Family::Integer,value:parameter_value.into()})])} else {BTreeMap::new()};
                let plan=weft_core::application_resolve::resolve(&catalog,weft_core::application_syntax::parse(sql).unwrap(),params,None).unwrap();
                let compiled=compiler.registry.compile(&catalog,weft_core::backend::Plan::V02(&plan),&target,&final_input).unwrap();
                let checks:Vec<_>=compiled.emission.obligations.iter().filter_map(|o|o.parameters.get("sql").and_then(|v|v.as_str())).collect();
                assert!(checks.len()>=6);
                let mut request=cases[0]["request"].clone();request.as_object_mut().unwrap().remove("readProfile");
                request["sql"]=json!(sql);request["target"]["backendId"]=json!("truss.postgresql.original");request["target"]["backendVersion"]=json!("0.1.0-candidate");
                request["target"]["bindingJson"]=json!(final_input.json);request["target"]["bindingSha256"]=json!(final_input.sha256);
                if kind=="has" {request["parameters"]=json!({parameter_name:{"family":"integer","value":parameter_value}});}
                let response:Value=serde_json::from_str(&compiler.compile_json(&request.to_string())).unwrap();
                assert_eq!(response["status"],"compiled", "{response}");
                assert_eq!(response["sql"],compiled.emission.sql);
                assert_eq!(response["parameters"],serde_json::to_value(&compiled.emission.parameters).unwrap());
                assert_eq!(response["columns"],serde_json::to_value(&compiled.emission.columns).unwrap());
                assert_eq!(response["obligations"],serde_json::to_value(&compiled.emission.obligations).unwrap());
                transport_captures.push(json!({"request":request,"response":response}));

                let captures=if direction=="forward" {&mut forward_captures} else {&mut public_captures};
                captures.push(json!({"kind":kind,"sql":compiled.emission.sql,"columns":compiled.emission.columns,"parameters":compiled.emission.parameters,"checks":checks,"targetCodecHex":leaves["root"].original_json.as_bytes().iter().map(|b|format!("{b:02x}")).collect::<String>(),"sourceCodecHex":source_leaves["root"].original_json.as_bytes().iter().map(|b|format!("{b:02x}")).collect::<String>()}));
            }
            if let Ok(path) = std::env::var("WEFT_PUBLIC_ORIGINAL_FORWARD_CAPTURE") {
                std::fs::write(path, serde_json::to_vec_pretty(&forward_captures).unwrap())
                    .unwrap();
            }
            if let Ok(path) = std::env::var("WEFT_PUBLIC_ORIGINAL_RELATIONSHIP_CAPTURE") {
                std::fs::write(path, serde_json::to_vec_pretty(&public_captures).unwrap()).unwrap();
            }
            if let Ok(path) = std::env::var("WEFT_PUBLIC_ORIGINAL_RELATIONSHIP_TRANSPORT_CAPTURE") {
                std::fs::write(
                    path,
                    serde_json::to_vec_pretty(&transport_captures).unwrap(),
                )
                .unwrap();
            }
            let mut wrong = read.clone();
            wrong.target_key.id = "wrong".into();
            let before = serde_json::to_value(parameters.clone().into_slots()).unwrap();
            assert!(relationship
                .prepare_target(
                    &row_context,
                    &wrong,
                    &endpoint_records,
                    &row_properties,
                    &comparisons,
                    &mut parameters,
                    3
                )
                .is_err());
            assert_eq!(
                serde_json::to_value(parameters.into_slots()).unwrap(),
                before
            );
        }
        let mut scope_parameters = crate::Parameters::default();
        assert!(crate::registered_access::prepare_relationship_target(
            &row_context,
            &row_records,
            &row_properties,
            &comparisons,
            &mut scope_parameters,
            1024
        )
        .is_err());
        assert!(scope_parameters.into_slots().is_empty());
        let row_backend = crate::original_backend::OriginalBackend::new(
            &row_input,
            row_records,
            row_properties,
            comparisons.clone(),
            row_numeric_native,
        )
        .unwrap();
        let mut row_registry = weft_core::backend::Registry::default();
        row_registry.register(row_backend).unwrap();
        let row_target = weft_core::backend::Target {
            backend_id: "truss.postgresql.original".into(),
            backend_version: "0.1.0-candidate".into(),
            profile_id: "pg17.9-candidate".into(),
            allow_candidate: true,
        };
        let row_public = row_registry
            .compile(
                &catalog,
                weft_core::backend::Plan::V02(&application),
                &row_target,
                &row_input,
            )
            .unwrap();
        assert_eq!(row_public.emission.sql, row_sum.select.sql);
        assert_eq!(
            serde_json::to_value(&row_public.emission.parameters).unwrap(),
            serde_json::to_value(&row_sum.parameters).unwrap()
        );
        let capture_row = |suffix: &str, emission: &weft_core::backend::Emission| {
            if let Ok(path) = std::env::var(format!("{capture}_{suffix}")) {
                let checks: Vec<_> = emission
                    .obligations
                    .iter()
                    .filter_map(|o| o.parameters.get("sql").and_then(|v| v.as_str()))
                    .collect();
                std::fs::write(path, serde_json::to_vec_pretty(&json!({"sql":emission.sql,"columns":emission.columns,"checks":checks,"parameters":emission.parameters,"codecHex":leaves["root"].original_json.as_bytes().iter().map(|b|format!("{b:02x}")).collect::<String>()})).unwrap()).unwrap();
            }
        };
        capture_row("ROW_SUM", &row_public.emission);
        if family == "integer" {
            let page = weft_core::application_resolve::resolve(
                &catalog,
                weft_core::application_syntax::parse(
                    "SELECT c.id FROM Customer c ORDER BY c.id LIMIT 2",
                )
                .unwrap(),
                BTreeMap::new(),
                Some(weft_core::application_ir::ReadProfile {
                    version: "weft-application-read/0.2.0".into(),
                    subset: weft_core::application_ir::Subset::EntityPage,
                }),
            )
            .unwrap();
            let row_page = row_registry
                .compile(
                    &catalog,
                    weft_core::backend::Plan::V02(&page),
                    &row_target,
                    &row_input,
                )
                .unwrap();
            assert!(row_page.emission.sql.contains("ORDER BY"));
            assert!(
                row_page
                    .emission
                    .obligations
                    .iter()
                    .filter(|o| o.parameters.get("sql").is_some())
                    .count()
                    > row_public
                        .emission
                        .obligations
                        .iter()
                        .filter(|o| o.parameters.get("sql").is_some())
                        .count()
            );
            capture_row("ROW_PAGE", &row_page.emission);
        }
        let backend = crate::original_backend::OriginalBackend::new(
            &input,
            record_registry,
            properties,
            comparisons,
            scalar_native,
        )
        .unwrap();
        let mut registry = weft_core::backend::Registry::default();
        registry.register(backend).unwrap();
        let mut target = weft_core::backend::Target {
            backend_id: "truss.postgresql.original".into(),
            backend_version: "0.1.0-candidate".into(),
            profile_id: "pg17.9-candidate".into(),
            allow_candidate: true,
        };
        let public = registry
            .compile(
                &catalog,
                weft_core::backend::Plan::V02(&application),
                &target,
                &input,
            )
            .unwrap();
        assert!(public.emission.sql.contains("pg_catalog.sum"));
        assert_eq!(
            serde_json::to_value(&public.emission.parameters).unwrap(),
            serde_json::to_value(&compiled.parameters).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&public.emission.columns).unwrap(),
            serde_json::to_value(&compiled.select.columns).unwrap()
        );
        let public_checks: Vec<_> = public
            .emission
            .obligations
            .iter()
            .filter(|obligation| obligation.id.starts_with("truss.original.owner-"))
            .map(|obligation| obligation.parameters["sql"].as_str().unwrap())
            .collect();
        assert_eq!(public_checks.len(), 3);
        if let Ok(path) = std::env::var(format!("{capture}_PUBLIC")) {
            std::fs::write(path,serde_json::to_vec_pretty(&json!({"sql":public.emission.sql,"columns":public.emission.columns,"checks":public_checks,"parameters":public.emission.parameters})).unwrap()).unwrap();
        }
        if family == "integer" {
            let page = weft_core::application_resolve::resolve(
                &catalog,
                weft_core::application_syntax::parse(
                    "SELECT c.id FROM Customer c ORDER BY c.id LIMIT 2",
                )
                .unwrap(),
                BTreeMap::new(),
                Some(weft_core::application_ir::ReadProfile {
                    version: "weft-application-read/0.2.0".into(),
                    subset: weft_core::application_ir::Subset::EntityPage,
                }),
            )
            .unwrap();
            let public_page = registry
                .compile(
                    &catalog,
                    weft_core::backend::Plan::V02(&page),
                    &target,
                    &input,
                )
                .unwrap();
            let checks: Vec<_> = public_page
                .emission
                .obligations
                .iter()
                .filter_map(|obligation| {
                    obligation
                        .parameters
                        .get("sql")
                        .and_then(|sql| sql.as_str())
                })
                .collect();
            assert_eq!(checks.len(), 4);
            if let Ok(path) = std::env::var("WEFT_PUBLIC_PAGE_CAPTURE") {
                std::fs::write(path,serde_json::to_vec_pretty(&json!({"sql":public_page.emission.sql,"columns":public_page.emission.columns,"checks":checks,"parameters":public_page.emission.parameters})).unwrap()).unwrap();
            }
        }
        if family == "integer" {
            let cursor = weft_core::application_resolve::resolve(
                &catalog,
                weft_core::application_syntax::parse(
                    "SELECT c.id FROM Customer c WHERE c.id > :cursor ORDER BY c.id LIMIT 2",
                )
                .unwrap(),
                BTreeMap::from([(
                    "cursor".into(),
                    weft_core::application_resolve::Parameter {
                        family: weft_core::ir::Family::Integer,
                        value: "2".into(),
                    },
                )]),
                Some(weft_core::application_ir::ReadProfile {
                    version: "weft-application-read/0.2.0".into(),
                    subset: weft_core::application_ir::Subset::EntityPage,
                }),
            )
            .unwrap();
            let public_cursor = registry
                .compile(
                    &catalog,
                    weft_core::backend::Plan::V02(&cursor),
                    &target,
                    &input,
                )
                .unwrap();
            assert_eq!(public_cursor.emission.parameters.len(), 3);
            assert_eq!(
                public_cursor.emission.parameters[2].origin["parameter"],
                "cursor"
            );
            let checks: Vec<_> = public_cursor
                .emission
                .obligations
                .iter()
                .filter_map(|obligation| {
                    obligation
                        .parameters
                        .get("sql")
                        .and_then(|sql| sql.as_str())
                })
                .collect();
            assert_eq!(checks.len(), 4);
            if let Ok(path) = std::env::var("WEFT_PUBLIC_CURSOR_CAPTURE") {
                std::fs::write(path, serde_json::to_vec_pretty(&json!({"sql":public_cursor.emission.sql,"columns":public_cursor.emission.columns,"checks":checks,"parameters":public_cursor.emission.parameters})).unwrap()).unwrap();
            }
        }
        target.allow_candidate = false;
        assert!(registry
            .compile(
                &catalog,
                weft_core::backend::Plan::V02(&application),
                &target,
                &input
            )
            .is_err());
        if let Ok(path) = std::env::var(capture) {
            std::fs::write(path, serde_json::to_vec_pretty(&json!({"sql":compiled.select.sql,"columns":compiled.select.columns,"checks":compiled.select.structural_checks.iter().cloned().chain(compiled.select.payload_checks.iter().map(|check|check.sql.clone())).collect::<Vec<_>>(),"parameters":compiled.parameters})).unwrap()).unwrap();
        }
    }
    fn synthetic_native_tree(
        layout: &crate::value_definition::Layout<'_>,
        body: &Value,
    ) -> Vec<[Option<String>; 23]> {
        use crate::value_definition::LayoutShape;
        fn hex(bytes: &[u8]) -> String {
            bytes.iter().map(|b| format!("{b:02x}")).collect()
        }
        fn visit(
            layout: &crate::value_definition::Layout<'_>,
            index: usize,
            body: &Value,
            parent: Option<usize>,
            slot: &str,
            key: Option<String>,
            rows: &mut Vec<[Option<String>; 23]>,
        ) {
            let id = rows.len() + 1;
            let node = &layout.nodes[index];
            let mut row: [Option<String>; 23] = std::array::from_fn(|_| None);
            row[0] = Some("1".into());
            row[1] = Some(id.to_string());
            row[2] = parent.map(|p| p.to_string());
            row[4] = Some(slot.into());
            if let Some(key) = key {
                row[match slot {
                    "sequence" => 5,
                    "map" => 6,
                    "record" => 7,
                    _ => panic!(),
                }] = Some(key);
            }
            row[8] = Some(hex(node.codec_bytes));
            row[9] = Some(String::new());
            let shape = if let LayoutShape::Structured { record } = node.shape {
                &layout.nodes[record].shape
            } else {
                &node.shape
            };
            row[3] = Some(
                match node.shape {
                    LayoutShape::Structured { .. } => "structured",
                    LayoutShape::Sequence { .. } => "sequence",
                    LayoutShape::Map { .. } => "map",
                    LayoutShape::Record { .. } => "record",
                    LayoutShape::Scalar { .. } => "scalar",
                }
                .into(),
            );
            if let LayoutShape::Scalar { family, .. } = shape {
                row[10] = Some("1".into());
                row[11] = Some(id.to_string());
                row[12] = Some(
                    if *family == "integer" {
                        "integer"
                    } else {
                        "text"
                    }
                    .into(),
                );
                if *family == "integer" {
                    row[15] = Some(body.as_str().unwrap().into());
                    row[16] = row[15].clone();
                } else {
                    row[13] = Some(body.as_str().unwrap().into());
                }
                row[21] = Some(hex(node.codec_bytes));
                row[22] = Some(String::new());
            }
            rows.push(row);
            match shape {
                LayoutShape::Sequence { item } => {
                    for (i, v) in body.as_array().unwrap().iter().enumerate() {
                        visit(
                            layout,
                            *item,
                            v,
                            Some(id),
                            "sequence",
                            Some(i.to_string()),
                            rows,
                        )
                    }
                }
                LayoutShape::Map { item } => {
                    for (k, v) in body.as_object().unwrap() {
                        visit(layout, *item, v, Some(id), "map", Some(k.clone()), rows)
                    }
                }
                LayoutShape::Record { members } => {
                    for member in members {
                        if let Some(v) = body.get(member.stored_name) {
                            visit(
                                layout,
                                member.value_node,
                                v,
                                Some(id),
                                "record",
                                Some(hex(&serde_json::to_vec(member.field_identity).unwrap())),
                                rows,
                            )
                        }
                    }
                }
                LayoutShape::Scalar { .. } => {}
                _ => unreachable!(),
            }
        }
        let mut rows = Vec::new();
        visit(layout, layout.root, body, None, "root", None, &mut rows);
        rows
    }
    fn fixture_native_leaf(
        _: usize,
        node: &crate::value_definition::LayoutNode<'_>,
    ) -> weft_core::error::Result<crate::row_tree_mapping::NativeLeaf> {
        let (value, kind, allowed) = match node.shape {
            crate::value_definition::LayoutShape::Scalar {
                family: "integer", ..
            } => ("to_jsonb(w.r->>16)", "integer", vec![15, 16]),
            crate::value_definition::LayoutShape::Scalar {
                family: "string", ..
            } => ("to_jsonb(w.r->>13)", "text", vec![13]),
            _ => panic!("fixture scalar procedure unsupported"),
        };
        let codec: String = node
            .codec_bytes
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let extra = (13..=20)
            .filter(|i| !allowed.contains(i))
            .map(|i| format!("w.r->>{i} IS NULL"))
            .collect::<Vec<_>>()
            .join(" AND ");
        let numeric = if kind == "integer" {
            " AND CASE WHEN w.r->>16 ~ '^(0|[1-9][0-9]*)$' AND pg_input_is_valid(w.r->>16,'numeric') THEN (w.r->>15)::numeric=(w.r->>16)::numeric ELSE FALSE END"
        } else {
            ""
        };
        Ok(crate::row_tree_mapping::NativeLeaf {value_sql:value.into(),integrity_sql:format!("w.r->>3='scalar' AND w.r->>12='{kind}' AND w.r->>21='{codec}' AND w.r->>22='' AND {extra}{numeric}")})
    }
    fn fixture_native_source(
        _: usize,
        node: &crate::value_definition::LayoutNode<'_>,
    ) -> weft_core::error::Result<String> {
        let codec: String = node
            .codec_bytes
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Ok(format!("w.r->>8='{codec}' AND w.r->>9=''"))
    }
    #[test]
    fn original_compound_properties_decode_storage_slots_into_logical_members() {
        use base64::{engine::general_purpose::STANDARD, Engine};
        use leaf_codec_definition::{
            Definition as Leaf, OriginalArtifact, Selection as LeafSelection,
        };
        use weft_core::json::sha256;
        let cases: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/application-cases.json"
        ))
        .unwrap();
        for (fixture_name, member_name, case_index) in [
            ("tags", "tags", 0),
            ("address", "address", 0),
            ("map", "tags", 58),
            ("cyclic", "address", 64),
            ("nested-sequence", "tags", 66),
            ("numeric-map", "tags", 62),
            ("numeric-address", "address", 56),
        ] {
            let catalog = Catalog::prepare(
                serde_json::from_value(cases[case_index]["request"]["modules"].clone()).unwrap(),
            )
            .unwrap();
            let name = |value: &str| Name {
                value: value.into(),
                quoted: false,
                span: Span { start: 0, end: 0 },
            };
            let record = catalog.record(None, &name("customer")).unwrap();
            let (member, descriptors) = catalog
                .member_descriptor(&record, &name(member_name))
                .unwrap();
            let mut binding: Value = serde_json::from_str(
                cases[case_index]["request"]["target"]["bindingJson"]
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
            let pin = binding["properties"][index]["valueProfile"].clone();
            let artifact = |identity: &str, bytes: &[u8]| json!({"identity":identity,"bytesBase64":STANDARD.encode(bytes),"sha256":sha256(bytes)});
            let empty = artifact("fixture", b"{}");
            let document: Value = serde_json::from_str(&catalog.inputs[0].document_json).unwrap();
            let authored: Vec<_> = descriptors
                .iter()
                .map(|d| {
                    let element = document["modules"][0]["elements"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|e| e["id"] == d.identity.element)
                        .unwrap();
                    artifact(&d.identity.element, element.to_string().as_bytes())
                })
                .collect();
            let node_id = |identity: &Identity| {
                descriptors
                    .iter()
                    .position(|d| &d.identity == identity)
                    .unwrap()
                    .to_string()
            };
            let presence_for = |accepted: &Value| {
                let schema: Value = serde_json::from_str(include_str!(
                    "../../../tests/truss-postgresql/upstream/presence-definition.schema.json"
                ))
                .unwrap();
                let mut value = json!({});
                for (key, rule) in schema["properties"].as_object().unwrap() {
                    if let Some(c) = rule.get("const") {
                        value[key] = c.clone();
                    }
                }
                value["profile"] = pin.clone();
                value["acceptedDefinition"] = accepted.clone();
                presence_definition::Definition::parse(
                    &value.to_string(),
                    &pin,
                    &STANDARD
                        .decode(accepted["bytesBase64"].as_str().unwrap())
                        .unwrap(),
                )
                .unwrap()
            };
            let mut leaves = BTreeMap::new();
            let mut records = BTreeMap::new();
            let mut nodes = Vec::new();
            for (i, d) in descriptors.iter().enumerate() {
                let mut codec = empty.clone();
                let shape = match &d.shape {
                    Shape::Scalar { logical_type } => {
                        let numeric = logical_type.family == weft_core::ir::Family::Integer;
                        assert!(numeric || logical_type.family == weft_core::ir::Family::String);
                        let mut raw = json!({"interfaceVersion":"truss-jsonb-leaf-codec/0.1.0","profile":pin,"authoredDefinition":authored[i],"sourceInterpretationProfile":pin,"sourceInterpretationDefinition":empty,"nativeDomainProfile":pin,"nativeDomainDefinition":empty,"rule":{"family":"string","storageRepresentation":"json-string","encoding":"preserve-unicode-scalars","decodedCarrierKind":"string"},"coercion":"none","readDefault":"none","invalidStoredValue":"complete-result-refusal"});
                        if numeric {
                            raw["rule"]["family"] = json!("integer");
                            raw["rule"]["decodedCarrierKind"] = json!("integer");
                            raw["rule"]["encoding"] = json!("preserve-admitted-source-token");
                            raw["rule"]["numericAdoptionEvidence"] = empty.clone();
                        }
                        let mut originals = BTreeMap::from([
                            (
                                "authoredDefinition".into(),
                                OriginalArtifact {
                                    identity: authored[i]["identity"].as_str().unwrap().into(),
                                    bytes: STANDARD
                                        .decode(authored[i]["bytesBase64"].as_str().unwrap())
                                        .unwrap(),
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
                        if numeric {
                            originals.insert(
                                "rule/numericAdoptionEvidence".into(),
                                OriginalArtifact {
                                    identity: "fixture".into(),
                                    bytes: b"{}".to_vec(),
                                },
                            );
                        }
                        let leaf = Leaf::parse(
                            &raw.to_string(),
                            LeafSelection {
                                profile: &pin,
                                source_profile: &pin,
                                native_profile: &pin,
                                original_artifacts: &originals,
                            },
                        )
                        .unwrap();
                        codec = artifact("selected-leaf", leaf.original_json.as_bytes());
                        leaves.insert(i.to_string(), leaf);
                        json!({"kind":"scalar","family":logical_type.family,"storageRepresentation":"json-string"})
                    }
                    Shape::Sequence { item } => {
                        json!({"kind":"sequence","itemNodeId":node_id(item)})
                    }
                    Shape::Map { item } => json!({"kind":"map","itemNodeId":node_id(item)}),
                    Shape::Structured { record } => {
                        json!({"kind":"structured","recordNodeId":node_id(record)})
                    }
                    Shape::Record { members } => {
                        let members: Vec<_>=members.iter().enumerate().map(|(j,m)| {
                            let child=descriptors.iter().position(|d|d.identity==m.identity).unwrap();
                            let presence=presence_for(&authored[child]);
                            let artifact=artifact("selected-presence",presence.original_json.as_bytes());
                            records.insert(format!("/nodes/{i}/shape/members/{j}/presenceDefinition"),presence);
                            json!({"fieldIdentity":m.identity,"storedMemberName":format!("slot.{j}"),"valueNodeId":child.to_string(),"presenceDefinition":artifact})
                        }).collect();
                        json!({"kind":"record","members":members})
                    }
                };
                nodes.push(json!({"nodeId":i.to_string(),"authoredIdentity":d.identity,"authoredDefinition":authored[i],"codecProfile":pin,"codecDefinition":codec,"shape":shape}));
            }
            let root = descriptors
                .iter()
                .position(|d| d.identity == member.identity)
                .unwrap();
            let graph = json!({"interfaceVersion":"truss-value-definition/0.1.0","profile":pin,"rootNodeId":root.to_string(),"acceptedDefinition":authored[root],"nodes":nodes});
            binding["properties"][index]["acceptedDefinition"] = authored[root].clone();
            binding["properties"][index]["valueDefinition"] =
                artifact("original-value-graph", graph.to_string().as_bytes());
            binding["properties"][index]["presenceDefinition"] = artifact(
                "original-presence",
                presence_for(&authored[root]).original_json.as_bytes(),
            );
            let admitted = Admission::parse(
                &binding.to_string(),
                binding["bindingProfileId"].as_str().unwrap(),
            )
            .unwrap();
            let value = admit_value(
                &admitted,
                index,
                &catalog,
                &descriptors,
                Selection {
                    value_profile: &pin,
                    presence_profile: &pin,
                    leaf_codecs: &leaves,
                    record_presence: &records,
                },
            )
            .unwrap();
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
            let mut property = admit_property(
                &admitted,
                index,
                &catalog,
                &descriptors,
                Selection {
                    value_profile: &pin,
                    presence_profile: &pin,
                    leaf_codecs: &leaves,
                    record_presence: &records,
                },
                PhysicalSelection {
                    profile: &pin,
                    inventory: &inventory,
                    relations: &relations,
                    columns: &columns,
                    row_join: None,
                    obligations: &obligations,
                    edge_association: None,
                },
            )
            .unwrap();
            assert_eq!(
                value.descriptors().len(),
                property.value.descriptors().len()
            );
            let mut owner_parameters = crate::Parameters::default();
            let owner_check = crate::recursive_observation::props_owner(
                &property,
                &crate::Identifier::new("pg_temp").unwrap(),
                &crate::Identifier::new("owner").unwrap(),
                &mut owner_parameters,
            )
            .unwrap();
            if let Ok(directory) = std::env::var("WEFT_ORIGINAL_COMPOUND_CAPTURE") {
                std::fs::write(
                    std::path::Path::new(&directory)
                        .join(format!("original-{fixture_name}-owner-observation.json")),
                    serde_json::to_vec_pretty(
                        &json!({"sql":owner_check,"parameters":owner_parameters.into_slots()}),
                    )
                    .unwrap(),
                )
                .unwrap();
            }
            let mut observation_parameters = crate::Parameters::default();
            let observation = crate::recursive_observation::encode(
                &property,
                "fixture.v",
                &mut observation_parameters,
            )
            .unwrap();
            assert_eq!(observation_parameters.clone().into_slots().len(), 1);
            if let Ok(directory) = std::env::var("WEFT_ORIGINAL_COMPOUND_CAPTURE") {
                std::fs::write(std::path::Path::new(&directory).join(format!("original-{fixture_name}-observation.json")),serde_json::to_vec_pretty(&json!({"sql":observation.integrity,"body":observation.body,"parameters":observation_parameters.into_slots()})).unwrap()).unwrap();
            }
            let decode = |input: &Value| {
                let mut budget = crate::value_traversal::Budget {
                    remaining_nodes: 100,
                    remaining_key_bytes: 1000,
                    remaining_members: 100,
                    max_depth: 32,
                };
                crate::value_traversal::decode_admitted_logical_property(
                    &property,
                    input,
                    &mut budget,
                    |codec, family, representation, v| {
                        assert!(family == "string" || family == "integer");
                        assert_eq!(representation, "json-string");
                        assert!(property
                            .value
                            .admitted_leaf_codecs
                            .values()
                            .any(|leaf| leaf.original_json.as_bytes() == codec));
                        if !v.is_string() {
                            return Err(weft_core::error::Diagnostic::new(
                                "WFT-DECODE",
                                "decode",
                                "Original string codec refuses input",
                            ));
                        }
                        if family == "integer" {
                            v.as_str().unwrap().parse::<u64>().map_err(|_| {
                                weft_core::error::Diagnostic::new(
                                    "WFT-DECODE",
                                    "decode",
                                    "Fixture original uint64 token refuses input",
                                )
                            })?;
                        }
                        Ok(v.clone())
                    },
                )
            };
            if fixture_name == "numeric-map" {
                assert_eq!(
                    decode(&json!({"9.a":"18446744073709551615","":"9007199254740993"})).unwrap(),
                    json!({"9.a":"18446744073709551615","":"9007199254740993"})
                );
                assert!(decode(&json!({"x":"-1"})).is_err());
            } else if fixture_name == "numeric-address" {
                assert_eq!(
                    decode(&json!({"slot.0":"é  ","slot.1":"18446744073709551615"})).unwrap(),
                    json!({"street":"é  ","zip":{"state":"value","value":"18446744073709551615"}})
                );
                assert!(decode(&json!({"slot.0":"x","slot.1":"18446744073709551616"})).is_err());
            } else if fixture_name == "nested-sequence" {
                assert_eq!(
                    decode(&json!([
                        ["9007199254740993", "18446744073709551615"],
                        [],
                        ["0"]
                    ]))
                    .unwrap(),
                    json!([["9007199254740993", "18446744073709551615"], [], ["0"]])
                );
                assert!(decode(&json!(["x"])).is_err());
                assert!(decode(&json!([[1]])).is_err());
            } else if fixture_name == "cyclic" {
                assert_eq!(
                    decode(&json!({"slot.0":"outer","slot.2":{"slot.0":"inner"}})).unwrap(),
                    json!({"street":"outer","zip":{"state":"absent"},"next":{"state":"value","value":{"street":"inner","zip":{"state":"absent"},"next":{"state":"absent"}}}})
                );
                assert!(decode(&json!({"slot.0":"outer","slot.2":{}})).is_err());
            } else if fixture_name == "map" {
                assert_eq!(
                    decode(&json!({"1.a[0]":"é  ","":"","雪":"x"})).unwrap(),
                    json!({"1.a[0]":"é  ","":"","雪":"x"})
                );
                assert_eq!(decode(&json!({})).unwrap(), json!({}));
                assert!(decode(&json!({"x":1})).is_err());
                assert!(decode(&json!([])).is_err());
            } else if member_name == "tags" {
                assert_eq!(
                    decode(&json!(["é  ", "1.a[0]", ""])).unwrap(),
                    json!(["é  ", "1.a[0]", ""])
                );
                assert_eq!(decode(&json!([])).unwrap(), json!([]));
                assert!(decode(&json!([1])).is_err());
            } else {
                assert_eq!(
                    decode(&json!({"slot.0":"é  "})).unwrap(),
                    json!({"street":"é  ","zip":{"state":"absent"}})
                );
                assert_eq!(
                    decode(&json!({"slot.0":"","slot.1":"00123"})).unwrap(),
                    json!({"street":"","zip":{"state":"value","value":"00123"}})
                );
                for bad in [
                    json!({}),
                    json!({"slot.0":null}),
                    json!({"street":"x"}),
                    json!({"slot.0":"x","unknown":"x"}),
                ] {
                    assert!(decode(&bad).is_err());
                }
                property.value.admitted_record_presence.clear();
                let mut budget = crate::value_traversal::Budget {
                    remaining_nodes: 100,
                    remaining_key_bytes: 1000,
                    remaining_members: 100,
                    max_depth: 32,
                };
                assert!(crate::value_traversal::decode_admitted_logical_property(
                    &property,
                    &json!({"slot.0":"x"}),
                    &mut budget,
                    |_, _, _, _| panic!(
                        "Missing original presence must refuse before leaf decoding"
                    )
                )
                .is_err());
            }
            property.value.admitted_record_presence = records.clone();
            let plan = weft_core::application_resolve::resolve(
                &catalog,
                weft_core::application_syntax::parse(&format!(
                    "SELECT c.{member_name} FROM Customer c"
                ))
                .unwrap(),
                BTreeMap::new(),
                None,
            )
            .unwrap();
            let input = weft_core::backend::BindingInput {
                profile: binding["bindingProfileId"].as_str().unwrap().into(),
                json: admitted.original_json.clone(),
                sha256: sha256(admitted.original_json.as_bytes()),
            };
            let manifest = <crate::candidate::Candidate as weft_core::backend::Backend>::describe(
                &crate::candidate::Candidate,
            )
            .unwrap();
            let selection = weft_core::backend::Selection {
                fields: vec![member.identity.clone()],
                records: vec![record.identity.clone()],
                ..Default::default()
            };
            let context = weft_core::backend::Context {
                catalog: &catalog,
                plan: weft_core::backend::Plan::V02(&plan),
                target: &manifest.target_profiles[0],
                binding: &input,
                binding_value: &admitted.value,
                selection: &selection,
            };
            // Same original UMF graph, independently admitted complete native home.
            let mut row_fixture = crate::row_join_definition::tests::fixture(false);
            row_fixture.value["layoutInventory"] = binding["basis"]["layoutInventory"].clone();
            row_fixture
                .artifacts
                .get_mut("layoutInventory")
                .unwrap()
                .identity = inventory.identity.clone();
            let row_join = std::sync::Arc::new(
                crate::row_join_definition::tests::parse(&row_fixture.value, &row_fixture).unwrap(),
            );
            let template: Value = serde_json::from_str(include_str!(
                "../../../tests/truss-postgresql/fixtures/binding-row.json"
            ))
            .unwrap();
            let mut home: Value = serde_json::from_slice(
                &STANDARD
                    .decode(
                        template["properties"][0]["homeDefinition"]["bytesBase64"]
                            .as_str()
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .unwrap();
            home["access"] = json!("complete-value-tree");
            home["layoutInventory"] = binding["basis"]["layoutInventory"].clone();
            for (key, source) in [
                ("ownerCatalogId", "ownerTypeId"),
                ("propertyCatalogId", "propertyId"),
                ("valueDefinition", "valueDefinition"),
                ("presenceDefinition", "presenceDefinition"),
            ] {
                home[key] = binding["properties"][index][source].clone();
            }
            home["joinProfile"] = row_fixture.value["profile"].clone();
            home["joinDefinition"] =
                artifact("selected-row-join", row_join.original_json.as_bytes());
            for (role, key) in [
                ("state", "stateRelationPhysicalIdentity"),
                ("node", "nodeRelationPhysicalIdentity"),
                ("scalar", "scalarRelationPhysicalIdentity"),
            ] {
                home[key] = row_fixture.value[role]["relationPhysicalIdentity"].clone();
            }
            let mut row_binding = binding.clone();
            row_binding["properties"][index]["home"] = json!("row");
            row_binding["properties"][index]["homeDefinition"] =
                artifact("selected-row-home", home.to_string().as_bytes());
            let row_admission = Admission::parse(&row_binding.to_string(), &input.profile).unwrap();
            let row_obligations =
                BTreeSet::from([home["storedDomainObligation"].as_str().unwrap().to_string()]);
            let row_property = admit_property(
                &row_admission,
                index,
                &catalog,
                &descriptors,
                Selection {
                    value_profile: &pin,
                    presence_profile: &pin,
                    leaf_codecs: &leaves,
                    record_presence: &records,
                },
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
            let row_input = weft_core::backend::BindingInput {
                profile: input.profile.clone(),
                sha256: sha256(row_admission.original_json.as_bytes()),
                json: row_admission.original_json.clone(),
            };
            let row_context = weft_core::backend::Context {
                binding: &row_input,
                binding_value: &row_binding,
                ..context
            };
            let row_properties = BTreeMap::from([(
                crate::comparator_requirements::registration_key(
                    &record.identity,
                    &member.identity,
                ),
                row_property,
            )]);

            let row_record_index = row_admission.value["entities"]
                .as_array()
                .unwrap()
                .iter()
                .position(|e| e["logical"] == json!(record.identity))
                .unwrap();
            let row_record = crate::record_definition::RecordAdmission::admit(
                &row_admission,
                row_record_index,
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
            let row_records = BTreeMap::from([(
                serde_json::to_string(row_record.identity()).unwrap(),
                row_record,
            )]);
            let mut unselected_parameters = crate::Parameters::default();
            let unselected_accesses = crate::registered_access::lower_plan(
                &row_context,
                &row_properties,
                &BTreeMap::new(),
                &mut unselected_parameters,
            )
            .unwrap();
            let unselected_before =
                serde_json::to_value(unselected_parameters.clone().into_slots()).unwrap();
            let unselected = crate::result_definition::property_projection_with_parameters(
                row_properties.values().next().unwrap(),
                &unselected_accesses[0],
                1,
                member_name,
                &mut unselected_parameters,
            )
            .unwrap_err();
            assert_eq!(unselected.code, "WFT-CAPABILITY");
            assert_eq!(
                serde_json::to_value(unselected_parameters.clone().into_slots()).unwrap(),
                unselected_before
            );
            let mut observation_parameters = crate::Parameters::default();
            let unselected_prepared = crate::registered_access::prepare(
                &row_context,
                &row_records,
                &row_properties,
                &BTreeMap::new(),
                &mut observation_parameters,
            )
            .unwrap();
            let observation_before =
                serde_json::to_value(observation_parameters.clone().into_slots()).unwrap();
            assert!(
                crate::result_definition::read_payload_observations_with_parameters(
                    &unselected_prepared,
                    &row_properties,
                    &mut observation_parameters
                )
                .is_err()
            );
            assert_eq!(
                serde_json::to_value(observation_parameters.into_slots()).unwrap(),
                observation_before
            );
            drop(unselected_prepared);
            let missing_selection = crate::select_definition::compile_with_registry(
                &row_context,
                &row_records,
                &row_properties,
                &BTreeMap::new(),
                |_, _, _, _| {
                    panic!("Missing native procedures must refuse before operand rendering")
                },
            )
            .err()
            .unwrap();
            assert_eq!(missing_selection.code, "WFT-CAPABILITY");
            drop(unselected_accesses);
            let row_properties = row_properties
                .into_iter()
                .map(|(key, property)| {
                    (key,property.with_native_tree(crate::row_tree_mapping::Procedures {
                    field_identity:|identity|Ok(serde_json::to_vec(identity).unwrap()),
                    scalar:fixture_native_leaf,node_source:fixture_native_source,
                }).unwrap())
                })
                .collect::<BTreeMap<_, _>>();
            let mut native_mapping_parameters = crate::Parameters::default();
            let row_accesses = crate::registered_access::lower_plan(
                &row_context,
                &row_properties,
                &BTreeMap::new(),
                &mut native_mapping_parameters,
            )
            .unwrap();
            let row_property = row_properties.values().next().unwrap();
            let stored = match fixture_name {
                "tags" => json!(["é  ", "1.a[0]", ""]),
                "address" => json!({"slot.0":"é  "}),
                "map" => json!({"1.a[0]":"é  ","":"","雪":"x"}),
                "cyclic" => json!({"slot.0":"outer","slot.2":{"slot.0":"inner"}}),
                "nested-sequence" => {
                    json!([["9007199254740993", "18446744073709551615"], [], ["0"]])
                }
                "numeric-map" => json!({"9.a":"18446744073709551615","":"9007199254740993"}),
                "numeric-address" => json!({"slot.0":"é  ","slot.1":"18446744073709551615"}),
                _ => unreachable!(),
            };
            let expected = crate::value_traversal::decode_admitted_logical_property(
                &property,
                &stored,
                &mut crate::value_traversal::Budget {
                    remaining_nodes: 1000,
                    remaining_key_bytes: 10000,
                    remaining_members: 1000,
                    max_depth: 32,
                },
                |_, _, _, v| Ok(v.clone()),
            )
            .unwrap();
            let layout = row_property.value.graph.layout().unwrap();
            let native_mapping = crate::row_tree_mapping::encode(
                row_property,
                &row_accesses[0],
                &mut native_mapping_parameters,
                |identity| Ok(serde_json::to_vec(identity).unwrap()),
            )
            .unwrap();
            assert_eq!(native_mapping.topology_parameter, "$4");
            assert!(!native_mapping.custody_sql.is_empty());
            assert_eq!(native_mapping_parameters.clone().into_slots().len(), 4);
            let mut body_parameters = native_mapping_parameters.clone();
            let native_body = crate::row_tree_mapping::body(
                row_property,
                &row_accesses[0],
                &mut body_parameters,
                |identity| Ok(serde_json::to_vec(identity).unwrap()),
                |_, node| match node.shape {
                    crate::value_definition::LayoutShape::Scalar {
                        family: "integer", ..
                    } => Ok("to_jsonb(w.r->>16)".into()),
                    crate::value_definition::LayoutShape::Scalar {
                        family: "string", ..
                    } => Ok("to_jsonb(w.r->>13)".into()),
                    _ => panic!("fixture native leaf procedure unsupported"),
                },
            )
            .unwrap();
            assert_eq!(body_parameters.clone().into_slots().len(), 6);
            let mut guarded_parameters = native_mapping_parameters.clone();
            let guarded = crate::row_tree_mapping::guarded_body(
                row_property,
                &row_accesses[0],
                &mut guarded_parameters,
                |identity| Ok(serde_json::to_vec(identity).unwrap()),
                fixture_native_leaf,
                fixture_native_source,
            )
            .unwrap();
            assert_eq!(guarded_parameters.clone().into_slots().len(), 6);
            let mut projection_parameters = native_mapping_parameters.clone();
            let projection = crate::result_definition::native_tree_projection(
                row_property,
                &row_accesses[0],
                1,
                member_name,
                &mut projection_parameters,
                |identity| Ok(serde_json::to_vec(identity).unwrap()),
                fixture_native_leaf,
                fixture_native_source,
            )
            .unwrap();
            assert_eq!(projection_parameters.clone().into_slots().len(), 6);
            if let Ok(directory) = std::env::var("WEFT_NATIVE_TREE_PROJECTION_CAPTURE") {
                let crate::registered_access::Location::Row(location) = &row_accesses[0].location
                else {
                    unreachable!()
                };
                std::fs::write(std::path::Path::new(&directory).join(format!("original-{fixture_name}-native-projection.json")),serde_json::to_vec_pretty(&json!({"fixture":fixture_name,"sql":format!("SELECT {} FROM {} {} WHERE {} AND {}.\"id\"=9007199254740993 LIMIT 1",projection.sql,row_accesses[0].owner_source.sql,location.joins.join(" "),row_accesses[0].owner_source.discriminator,row_accesses[0].owner_alias.sql()),"check":projection.payload_check_sql,"parameters":projection_parameters.into_slots(),"columns":[projection.column],"propertyId":row_property.property_catalog_id,"ownerTypeId":row_property.owner_catalog_id,"required":row_property.value.descriptors().iter().find(|d|d.identity==row_property.identity).unwrap().availability.as_deref()==Some("required")})).unwrap()).unwrap();
            }

            if let Ok(directory) = std::env::var("WEFT_NATIVE_TREE_GUARDED_CAPTURE") {
                let crate::registered_access::Location::Row(location) = &row_accesses[0].location
                else {
                    unreachable!()
                };
                std::fs::write(std::path::Path::new(&directory).join(format!("original-{fixture_name}-native-guarded-body.json")),serde_json::to_vec_pretty(&json!({"fixture":fixture_name,"sql":format!("SELECT {} AS stored,{} AS logical,{} AS integrity,{} AS structure,{} AS native FROM {} {} WHERE {}",guarded.body.storage_body,guarded.body.logical_body,guarded.body.logical_integrity,guarded.body.structural_integrity,guarded.native_integrity,row_accesses[0].owner_source.sql,location.joins.join(" "),row_accesses[0].owner_source.discriminator),"parameters":guarded_parameters.clone().into_slots(),"propertyId":row_property.property_catalog_id,"ownerTypeId":row_property.owner_catalog_id,"procedure":"fixture original codec bytes, empty source bytes, closed text/canonical uint64 slots; not Truss adopted encoding"})).unwrap()).unwrap();
            }
            let guarded_before =
                serde_json::to_value(guarded_parameters.clone().into_slots()).unwrap();
            assert!(crate::row_tree_mapping::guarded_body(
                row_property,
                &row_accesses[0],
                &mut guarded_parameters,
                |identity| Ok(serde_json::to_vec(identity).unwrap()),
                |_, _| Ok(crate::row_tree_mapping::NativeLeaf {
                    value_sql: "NULL::jsonb".into(),
                    integrity_sql: "TRUE".into()
                }),
                |_, _| Err(weft_core::error::Diagnostic::new(
                    "WFT-CAPABILITY",
                    "emit",
                    "fixture source procedure refused"
                ))
            )
            .is_err());
            assert_eq!(
                serde_json::to_value(guarded_parameters.into_slots()).unwrap(),
                guarded_before
            );
            let before_body = serde_json::to_value(body_parameters.clone().into_slots()).unwrap();
            assert!(crate::row_tree_mapping::body(
                row_property,
                &row_accesses[0],
                &mut body_parameters,
                |identity| Ok(serde_json::to_vec(identity).unwrap()),
                |_, _| Err(weft_core::error::Diagnostic::new(
                    "WFT-CAPABILITY",
                    "emit",
                    "fixture refuses native leaf lowering"
                ))
            )
            .is_err());
            assert_eq!(
                serde_json::to_value(body_parameters.clone().into_slots()).unwrap(),
                before_body
            );

            if let Ok(directory) = std::env::var("WEFT_NATIVE_TREE_BODY_CAPTURE") {
                let crate::registered_access::Location::Row(location) = &row_accesses[0].location
                else {
                    unreachable!()
                };
                std::fs::write(std::path::Path::new(&directory).join(format!("original-{fixture_name}-native-structure-body.json")),serde_json::to_vec_pretty(&json!({"fixture":fixture_name,"sql":format!("SELECT {} AS stored,{} AS logical,{} AS integrity,{} AS structure FROM {} {} WHERE {}",native_body.storage_body,native_body.logical_body,native_body.logical_integrity,native_body.structural_integrity,row_accesses[0].owner_source.sql,location.joins.join(" "),row_accesses[0].owner_source.discriminator),"parameters":body_parameters.into_slots(),"propertyId":row_property.property_catalog_id,"ownerTypeId":row_property.owner_catalog_id})).unwrap()).unwrap();
            }

            if let Ok(directory) = std::env::var("WEFT_NATIVE_TREE_WALK_CAPTURE") {
                let crate::registered_access::Location::Row(location) = &row_accesses[0].location
                else {
                    unreachable!()
                };
                std::fs::write(std::path::Path::new(&directory).join(format!("original-{fixture_name}-native-walk.json")),serde_json::to_vec_pretty(&json!({"fixture":fixture_name,"sql":format!("SELECT {} AS mapping FROM {} {} WHERE {}",native_mapping.walk_sql,row_accesses[0].owner_source.sql,location.joins.join(" "),row_accesses[0].owner_source.discriminator),"parameters":native_mapping_parameters.clone().into_slots(),"propertyId":row_property.property_catalog_id,"ownerTypeId":row_property.owner_catalog_id})).unwrap()).unwrap();
            }

            if fixture_name == "address"
                || fixture_name == "cyclic"
                || fixture_name == "numeric-address"
            {
                let before =
                    serde_json::to_value(native_mapping_parameters.clone().into_slots()).unwrap();
                assert!(crate::row_tree_mapping::encode(
                    row_property,
                    &row_accesses[0],
                    &mut native_mapping_parameters,
                    |_| Ok(vec![])
                )
                .is_err());
                assert_eq!(
                    serde_json::to_value(native_mapping_parameters.clone().into_slots()).unwrap(),
                    before
                );
            }
            let before =
                serde_json::to_value(native_mapping_parameters.clone().into_slots()).unwrap();
            assert!(crate::row_tree_mapping::encode(
                &property,
                &row_accesses[0],
                &mut native_mapping_parameters,
                |_| panic!("substituted property must refuse before native identity encoding")
            )
            .is_err());
            assert_eq!(
                serde_json::to_value(native_mapping_parameters.into_slots()).unwrap(),
                before
            );

            let synthetic_cells = synthetic_native_tree(&layout, &stored);
            let native_receipt: Value = serde_json::from_str(include_str!(
                "../../../docs/helix/04-build/evidence/B-005-original-recursive-row-native.json"
            ))
            .unwrap();
            let native_cases: Vec<_> = native_receipt["results"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|case| case["fixture"] == fixture_name)
                .collect();
            assert_eq!(native_cases.len(), 2);
            for case in &native_cases {
                assert_eq!(case["cells"], json!(synthetic_cells));
            }
            let cells: Vec<[Option<String>; 23]> =
                serde_json::from_value(native_cases[0]["cells"].clone()).unwrap();
            let decode_native =
                |cells: &[[Option<String>; 23]]| -> weft_core::error::Result<Value> {
                    let refuse = |message: &str| {
                        weft_core::error::Diagnostic::new("WFT-DECODE", "decode", message)
                    };
                    let borrowed = cells
                        .iter()
                        .map(|row| row.iter().map(Option::as_deref).collect())
                        .collect::<Vec<Vec<_>>>();
                    let native = crate::row_custody::admit_tree_rows(
                        &borrowed,
                        &mut crate::row_custody::Budget {
                            remaining_cells: 10000,
                            remaining_bytes: 1000000,
                        },
                    )?;
                    let tree = crate::row_custody::index_tree(
                        &native,
                        "1",
                        "1",
                        &mut crate::row_custody::TreeBudget {
                            remaining_nodes: 1000,
                            max_depth: 32,
                        },
                    )?;
                    crate::row_value_traversal::decode_logical_property(
                        row_property,
                        &row_accesses[0],
                        &tree,
                        &mut crate::value_traversal::Budget {
                            remaining_nodes: 2000,
                            remaining_key_bytes: 20000,
                            remaining_members: 2000,
                            max_depth: 32,
                        },
                        |node, row| {
                            if row.bytes[&8] != node.codec_bytes || !row.bytes[&9].is_empty() {
                                return Err(refuse(
                                    "Fixture native definition/source custody differs",
                                ));
                            }
                            Ok(())
                        },
                        |node, row| {
                            if let crate::value_definition::LayoutShape::Scalar { family, .. } =
                                node.shape
                            {
                                let token = if family == "integer" {
                                    let token = row.cells[16]
                                        .as_deref()
                                        .ok_or_else(|| refuse("Fixture numeric token missing"))?;
                                    token
                                        .parse::<u64>()
                                        .map_err(|_| refuse("Fixture uint64 domain refused"))?;
                                    token
                                } else {
                                    row.cells[13]
                                        .as_deref()
                                        .ok_or_else(|| refuse("Fixture text missing"))?
                                };
                                Ok(json!(token))
                            } else {
                                Err(refuse("Fixture leaf shape differs"))
                            }
                        },
                        |bytes, identity| Ok(bytes == serde_json::to_vec(identity).unwrap()),
                    )
                };
            let logical = decode_native(&cells).unwrap();
            assert_eq!(logical, expected, "native fixture {fixture_name}");
            let mut wrong_definition = cells.clone();
            wrong_definition[0][8] = Some("ff".into());
            assert!(decode_native(&wrong_definition).is_err());
            let mut wrong_source = cells.clone();
            wrong_source[0][9] = Some("ff".into());
            assert!(decode_native(&wrong_source).is_err());
            if let Some(index) = cells.iter().position(|row| row[16].is_some()) {
                let mut overflow = cells.clone();
                overflow[index][15] = Some("18446744073709551616".into());
                overflow[index][16] = overflow[index][15].clone();
                assert!(decode_native(&overflow).is_err());
            }
            if fixture_name == "address"
                || fixture_name == "numeric-address"
                || fixture_name == "cyclic"
            {
                let mut unknown = cells.clone();
                unknown[1][7] = Some("ff".into());
                assert!(decode_native(&unknown).is_err());
            }
            if fixture_name == "address" {
                assert!(
                    decode_native(&cells[..1]).is_err(),
                    "missing required street must refuse"
                );
            }
            if let Ok(directory) = std::env::var("WEFT_ORIGINAL_NATIVE_TREE_CAPTURE") {
                std::fs::write(std::path::Path::new(&directory).join(format!("original-{fixture_name}-native-tree.json")),
                    serde_json::to_vec_pretty(&json!({"fixture":fixture_name,"binding":row_binding,"cells":cells,"stored":stored,"logical":logical,"identityProcedure":"synthetic exact JSON bytes; not a Truss adopted encoding"})).unwrap()).unwrap();
            }

            let row_compiled = crate::select_definition::compile_with_registry(
                &row_context,
                &row_records,
                &row_properties,
                &BTreeMap::new(),
                |_, _, _, _| panic!("Native compound projection must use selected tree procedures"),
            )
            .unwrap();
            assert_eq!(row_compiled.select.payload_checks.len(), 1);

            let mut wrong_record = record.identity.clone();
            wrong_record.revision = "unselected-revision".into();
            assert!(catalog.record_by_identity(&wrong_record).is_err());
            let mut wrong_member = member.identity.clone();
            wrong_member.document_id = "other-document".into();
            assert!(catalog
                .member_descriptor_by_identity(&record, &wrong_member)
                .is_err());
            let mut bad_pin = row_input.clone();
            bad_pin.sha256 = "0".repeat(64);
            assert!(crate::original_admission::backend(
                &catalog,
                &bad_pin,
                vec![],
                vec![],
                BTreeMap::new(),
                compound_native
            )
            .is_err());
            let record_selection = || crate::original_admission::RecordSelection {
                index: row_record_index,
                physical: crate::record_definition::Selection {
                    inventory: &inventory,
                    relation_identity: "object-table",
                    discriminator_identity: "object-type",
                    relations: &relations,
                    columns: &columns,
                },
            };
            let property_selection = || crate::original_admission::PropertySelection {
                index,
                value: Selection {
                    value_profile: &pin,
                    presence_profile: &pin,
                    leaf_codecs: &leaves,
                    record_presence: &records,
                },
                physical: PhysicalSelection {
                    profile: &pin,
                    inventory: &inventory,
                    relations: &row_fixture.relations,
                    columns: &row_fixture.columns,
                    row_join: Some(&row_join),
                    obligations: &row_obligations,
                    edge_association: None,
                },
                native_tree: Some(crate::row_tree_mapping::Procedures {
                    field_identity: |identity| Ok(serde_json::to_vec(identity).unwrap()),
                    scalar: fixture_native_leaf,
                    node_source: fixture_native_source,
                }),
            };
            assert!(crate::original_admission::backend(
                &catalog,
                &row_input,
                vec![record_selection(), record_selection()],
                vec![],
                BTreeMap::new(),
                compound_native
            )
            .is_err());
            assert!(crate::original_admission::backend(
                &catalog,
                &row_input,
                vec![record_selection()],
                vec![property_selection(), property_selection()],
                BTreeMap::new(),
                compound_native
            )
            .is_err());
            assert!(crate::original_admission::backend(
                &catalog,
                &row_input,
                vec![],
                vec![property_selection()],
                BTreeMap::new(),
                compound_native
            )
            .is_err());
            let row_backend = crate::original_admission::backend(
                &catalog,
                &row_input,
                vec![record_selection()],
                vec![property_selection()],
                BTreeMap::new(),
                compound_native,
            )
            .unwrap();
            let mut row_registry = weft_core::backend::Registry::default();
            row_registry.register(row_backend).unwrap();
            let row_target = weft_core::backend::Target {
                backend_id: "truss.postgresql.original".into(),
                backend_version: "0.1.0-candidate".into(),
                profile_id: "pg17.9-candidate".into(),
                allow_candidate: true,
            };
            let row_public = row_registry
                .compile(
                    &catalog,
                    weft_core::backend::Plan::V02(&plan),
                    &row_target,
                    &row_input,
                )
                .unwrap();
            assert_eq!(row_public.emission.sql, row_compiled.select.sql);
            assert_eq!(
                serde_json::to_value(&row_public.emission.parameters).unwrap(),
                serde_json::to_value(&row_compiled.parameters).unwrap()
            );

            let serialized_request = json!({"interfaceVersion":"weft-compile/0.2.0","dialect":"weft-sql/0.2.0","sql":format!("SELECT c.{member_name} FROM Customer c"),"modules":catalog.inputs,"target":{"backendId":row_target.backend_id,"backendVersion":row_target.backend_version,"targetProfile":row_target.profile_id,"bindingJson":row_input.json,"bindingSha256":row_input.sha256},"options":{"allowCandidate":true}});
            let owned_configuration = crate::original_admission::Configuration {
                binding_profile: row_input.profile.clone(),
                native: compound_native,
                comparators: BTreeMap::new(),
                records: vec![crate::original_admission::OwnedRecordSelection {
                    index: row_record_index,
                    inventory: inventory.clone(),
                    relation_identity: "object-table".into(),
                    discriminator_identity: "object-type".into(),
                    relations: relations.clone(),
                    columns: columns.clone(),
                }],
                properties: vec![crate::original_admission::OwnedPropertySelection {
                    index,
                    value_profile: pin.clone(),
                    presence_profile: pin.clone(),
                    leaf_codecs: leaves.clone(),
                    record_presence: records.clone(),
                    physical_profile: pin.clone(),
                    inventory: inventory.clone(),
                    relations: row_fixture.relations.clone(),
                    columns: row_fixture.columns.clone(),
                    row_join: Some(row_join.clone()),
                    obligations: row_obligations.clone(),
                    edge_association: None,
                    native_tree: Some(crate::row_tree_mapping::Procedures {
                        field_identity: |identity| Ok(serde_json::to_vec(identity).unwrap()),
                        scalar: fixture_native_leaf,
                        node_source: fixture_native_source,
                    }),
                }],
            };
            let mut calls = 0;
            let mut factory =
                |resolved_catalog: &Catalog,
                 resolved_plan: weft_core::backend::Plan<'_>,
                 selected: weft_core::compile::CompositionInput<'_>| {
                    calls += 1;
                    assert_eq!(selected.binding_json, row_input.json);
                    assert_eq!(selected.binding_sha256, row_input.sha256);
                    assert_eq!(selected.backend_id, "truss.postgresql.original");
                    assert!(selected.allow_candidate);
                    assert_eq!(resolved_catalog.inputs.len(), catalog.inputs.len());
                    assert!(matches!(resolved_plan, weft_core::backend::Plan::V02(_)));
                    let request_binding = weft_core::backend::BindingInput {
                        profile: row_input.profile.clone(),
                        json: selected.binding_json.into(),
                        sha256: selected.binding_sha256.into(),
                    };
                    let admitted_backend = crate::original_admission::backend(
                        resolved_catalog,
                        &request_binding,
                        vec![record_selection()],
                        vec![property_selection()],
                        BTreeMap::new(),
                        compound_native,
                    )?;
                    let mut composed = weft_core::backend::Registry::default();
                    composed.register(admitted_backend)?;
                    Ok(composed)
                };
            for invalid in [
                {
                    let mut v = serialized_request.clone();
                    v["target"]["bindingSha256"] = json!("0".repeat(64));
                    v
                },
                {
                    let mut v = serialized_request.clone();
                    v["sql"] = json!("SELECT c.missing FROM Customer c");
                    v
                },
                {
                    let mut v = serialized_request.clone();
                    v["sql"] = json!("SELECT FROM");
                    v
                },
            ] {
                let response: Value = serde_json::from_str(
                    &weft_core::compile::Compiler::default()
                        .compile_json_with_factory(&invalid.to_string(), &mut factory),
                )
                .unwrap();
                assert_eq!(response["status"], "blocked");
                assert!(
                    response.get("sql").is_none()
                        && response.get("parameters").is_none()
                        && response.get("columns").is_none()
                );
            }
            let transported: Value = serde_json::from_str(
                &weft_core::compile::Compiler::default()
                    .compile_json_with_factory(&serialized_request.to_string(), &mut factory),
            )
            .unwrap();
            let repeated: Value = serde_json::from_str(
                &weft_core::compile::Compiler::default()
                    .compile_json_with_factory(&serialized_request.to_string(), &mut factory),
            )
            .unwrap();
            assert_eq!(repeated, transported);
            drop(factory);
            assert_eq!(calls, 2);
            assert_eq!(transported["status"], "compiled");
            let mut refused_calls = 0;
            let mut refused_factory =
                |_: &Catalog,
                 _: weft_core::backend::Plan<'_>,
                 _: weft_core::compile::CompositionInput<'_>| {
                    refused_calls += 1;
                    Err(weft_core::error::Diagnostic::new(
                        "WFT-BACKEND-FAILURE",
                        "admit",
                        "fixture composition refused",
                    ))
                };
            let refused: Value = serde_json::from_str(
                &weft_core::compile::Compiler::default().compile_json_with_factory(
                    &serialized_request.to_string(),
                    &mut refused_factory,
                ),
            )
            .unwrap();
            drop(refused_factory);
            assert_eq!(refused_calls, 1);
            assert_eq!(refused["status"], "blocked");
            assert_eq!(refused["diagnostics"][0]["code"], "WFT-BACKEND-FAILURE");
            for member in ["sql", "parameters", "columns", "obligations", "logicalPlan"] {
                assert!(refused.get(member).is_none());
            }

            assert_eq!(transported["sql"], json!(row_public.emission.sql));
            assert_eq!(
                transported["parameters"],
                serde_json::to_value(&row_public.emission.parameters).unwrap()
            );
            assert_eq!(
                transported["columns"],
                serde_json::to_value(&row_public.emission.columns).unwrap()
            );
            assert_eq!(
                transported["obligations"],
                serde_json::to_value(&row_public.emission.obligations).unwrap()
            );
            assert_eq!(
                transported["logicalPlan"],
                serde_json::to_value(&plan).unwrap()
            );
            assert_eq!(transported["bindingSha256"], row_input.sha256);
            assert_eq!(transported["qualification"]["status"], "candidate");
            let owned_response: Value = serde_json::from_str(
                &owned_configuration.compile_json(&serialized_request.to_string()),
            )
            .unwrap();
            assert_eq!(owned_response, transported);
            if let Ok(directory) = std::env::var("WEFT_ORIGINAL_CONFIGURATION_CAPTURE") {
                let original = |a: &crate::leaf_codec_definition::OriginalArtifact| json!({"identity":a.identity,"bytesBase64":STANDARD.encode(&a.bytes),"sha256":sha256(&a.bytes)});
                let columns = |columns: &BTreeMap<String, crate::row_join_definition::Column>| {
                    columns
                        .iter()
                        .map(|(id, c)| {
                            (
                                id.clone(),
                                json!({"relationIdentity":c.relation_identity,"name":c.name}),
                            )
                        })
                        .collect::<serde_json::Map<String, Value>>()
                };
                let configuration = json!({"interfaceVersion":"weft-original-conformance-composition/0.1.0","bindingProfile":owned_configuration.binding_profile,
                    "records":owned_configuration.records.iter().map(|r|json!({"index":r.index,"inventory":original(&r.inventory),"relationIdentity":r.relation_identity,"discriminatorIdentity":r.discriminator_identity,"relations":r.relations,"columns":columns(&r.columns)})).collect::<Vec<_>>(),
                    "properties":owned_configuration.properties.iter().map(|p|json!({"index":p.index,"valueProfile":p.value_profile,"presenceProfile":p.presence_profile,"leafCodecs":p.leaf_codecs.iter().map(|(id,d)|(id.clone(),json!({"originalJson":d.original_json,"originalArtifacts":d.original_artifacts.iter().map(|(path,bytes)|(path.clone(),STANDARD.encode(bytes))).collect::<BTreeMap<_,_>>()}))).collect::<BTreeMap<_,_>>(),"recordPresence":p.record_presence.iter().map(|(path,d)|(path.clone(),json!({"originalJson":d.original_json,"acceptedDefinitionBase64":STANDARD.encode(&d.accepted_definition)}))).collect::<BTreeMap<_,_>>(),"physicalProfile":p.physical_profile,"inventory":original(&p.inventory),"relations":p.relations,"columns":columns(&p.columns),"rowJoin":p.row_join.as_ref().map(|j|json!({"originalJson":j.original_json,"originalArtifacts":j.original_artifacts.iter().map(|(path,bytes)|(path.clone(),STANDARD.encode(bytes))).collect::<BTreeMap<_,_>>() })),"obligations":p.obligations})).collect::<Vec<_>>(),
                    "procedureScope":"Compiled fixture-only exact JSON field identities, original codec bytes, empty source bytes and closed text/uint64 slots; no executable names in metadata"});
                std::fs::write(
                    std::path::Path::new(&directory)
                        .join(format!("original-{fixture_name}-composition.json")),
                    serde_json::to_vec_pretty(&configuration).unwrap(),
                )
                .unwrap();
                std::fs::write(
                    std::path::Path::new(&directory)
                        .join(format!("original-{fixture_name}-compile-transport.json")),
                    serde_json::to_vec_pretty(
                        &json!({"request":serialized_request,"response":owned_response}),
                    )
                    .unwrap(),
                )
                .unwrap();
            }

            assert_eq!(
                owned_configuration.compile_json(&serialized_request.to_string()),
                owned_configuration.compile_json(&serialized_request.to_string())
            );
            let mut other_backend = serialized_request.clone();
            other_backend["target"]["backendId"] = json!("other.backend");
            let refusal: Value =
                serde_json::from_str(&owned_configuration.compile_json(&other_backend.to_string()))
                    .unwrap();
            assert_eq!(refusal["status"], "blocked");
            assert!(refusal.get("sql").is_none());

            if let Ok(directory) = std::env::var("WEFT_NATIVE_TREE_PUBLIC_CAPTURE") {
                let checks: Vec<_> = row_public
                    .emission
                    .obligations
                    .iter()
                    .filter_map(|o| o.parameters.get("sql").and_then(Value::as_str))
                    .collect();
                std::fs::write(std::path::Path::new(&directory).join(format!("original-{fixture_name}-native-public.json")),serde_json::to_vec_pretty(&json!({"fixture":fixture_name,"sql":row_public.emission.sql,"checks":checks,"parameters":row_public.emission.parameters,"columns":row_public.emission.columns,"propertyId":row_binding["properties"][index]["propertyId"],"ownerTypeId":row_binding["properties"][index]["ownerTypeId"]})).unwrap()).unwrap();
            }
            let properties = BTreeMap::from([(
                crate::comparator_requirements::registration_key(
                    &record.identity,
                    &member.identity,
                ),
                property,
            )]);
            let mut parameters = crate::Parameters::default();
            let accesses = crate::registered_access::lower_plan(
                &context,
                &properties,
                &BTreeMap::new(),
                &mut parameters,
            )
            .unwrap();
            assert_eq!(accesses.len(), 1);
            let property = properties.values().next().unwrap();
            let projection = crate::result_definition::property_projection_with_parameters(
                property,
                &accesses[0],
                1,
                member_name,
                &mut parameters,
            )
            .unwrap();
            assert!(matches!(
                projection.column.representation,
                weft_core::backend::Representation::Value {
                    native_null: false,
                    ..
                }
            ));
            if let Ok(directory) = std::env::var("WEFT_ORIGINAL_COMPOUND_CAPTURE") {
                std::fs::write(std::path::Path::new(&directory).join(format!("original-{fixture_name}-projection.json")),serde_json::to_vec_pretty(&json!({"sql":format!("SELECT {} FROM {} WHERE {}",projection.sql,accesses[0].owner_source.sql,accesses[0].owner_source.discriminator),"check":projection.payload_check_sql,"columns":[projection.column],"parameters":parameters.into_slots()})).unwrap()).unwrap();
            }
            let record_index = admitted.value["entities"]
                .as_array()
                .unwrap()
                .iter()
                .position(|e| e["logical"] == json!(record.identity))
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
            let record_registry = BTreeMap::from([(
                serde_json::to_string(record_source.identity()).unwrap(),
                record_source,
            )]);
            let compiled = crate::select_definition::compile_with_registry(
                &context,
                &record_registry,
                &properties,
                &BTreeMap::new(),
                |_, _, _, _| panic!("Compound projection must use original recursive codec"),
            )
            .unwrap();
            assert_eq!(compiled.select.payload_checks.len(), 1);
            assert_eq!(compiled.parameters.len(), 4);
            if let Ok(directory) = std::env::var("WEFT_ORIGINAL_COMPOUND_CAPTURE") {
                std::fs::write(std::path::Path::new(&directory).join(format!("original-{fixture_name}-select.json")),serde_json::to_vec_pretty(&json!({"sql":compiled.select.sql,"checks":compiled.select.structural_checks.iter().cloned().chain(compiled.select.payload_checks.iter().map(|p|p.sql.clone())).collect::<Vec<_>>(),"columns":compiled.select.columns,"parameters":compiled.parameters})).unwrap()).unwrap();
            }
            fn compound_native(
                _: &weft_core::ir::Expression,
                _: &[String],
                _: Option<&crate::registered_access::Access<'_>>,
                _: &mut crate::Parameters,
            ) -> weft_core::error::Result<String> {
                panic!("Compound projection must use original recursive codec");
            }
            let backend = crate::original_backend::OriginalBackend::new(
                &input,
                record_registry,
                properties,
                BTreeMap::new(),
                compound_native,
            )
            .unwrap();
            let mut registry = weft_core::backend::Registry::default();
            registry.register(backend).unwrap();
            let target = weft_core::backend::Target {
                backend_id: "truss.postgresql.original".into(),
                backend_version: "0.1.0-candidate".into(),
                profile_id: "pg17.9-candidate".into(),
                allow_candidate: true,
            };
            let public = registry
                .compile(
                    &catalog,
                    weft_core::backend::Plan::V02(&plan),
                    &target,
                    &input,
                )
                .unwrap();
            assert_eq!(public.emission.sql, compiled.select.sql);
            assert_eq!(
                serde_json::to_value(&public.emission.parameters).unwrap(),
                serde_json::to_value(&compiled.parameters).unwrap()
            );
            if let Ok(directory) = std::env::var("WEFT_ORIGINAL_COMPOUND_CAPTURE") {
                let checks: Vec<_> = public
                    .emission
                    .obligations
                    .iter()
                    .filter_map(|o| o.parameters.get("sql").and_then(|v| v.as_str()))
                    .collect();
                std::fs::write(std::path::Path::new(&directory).join(format!("public-{fixture_name}-select.json")),serde_json::to_vec_pretty(&json!({"sql":public.emission.sql,"checks":checks,"columns":public.emission.columns,"parameters":public.emission.parameters})).unwrap()).unwrap();
            }
        }
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
        let mut recursive_budget = crate::value_traversal::Budget {
            remaining_nodes: 10,
            remaining_key_bytes: 100,
            remaining_members: 10,
            max_depth: 8,
        };
        assert_eq!(
            crate::value_traversal::decode_property(
                &property,
                &json!("é  "),
                &mut recursive_budget,
                |codec, family, representation, value| {
                    assert_eq!(codec, leaves["root"].original_json.as_bytes());
                    assert_eq!(family, "string");
                    assert_eq!(representation, "json-string");
                    value.as_str().ok_or_else(|| {
                        weft_core::error::Diagnostic::new(
                            "WFT-DECODE",
                            "decode",
                            "Original string codec refused input",
                        )
                    })?;
                    Ok(value.clone())
                },
                |_, _| panic!("scalar has no member absence"),
            )
            .unwrap(),
            json!("é  ")
        );
        assert_eq!(recursive_budget.remaining_nodes, 9);
        assert_eq!(
            crate::value_traversal::decode_admitted_logical_property(
                &property,
                &json!("é  "),
                &mut recursive_budget,
                |codec, family, representation, value| {
                    assert_eq!(codec, leaves["root"].original_json.as_bytes());
                    assert_eq!(family, "string");
                    assert_eq!(representation, "json-string");
                    Ok(value.clone())
                }
            )
            .unwrap(),
            json!("é  ")
        );

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
                    operations: &BTreeSet::from([Operation::Equality, Operation::Ordering]),
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
        let scan = match &plan.root {
            weft_core::ir::Node::Project { input, .. } => match input.as_ref() {
                weft_core::ir::Node::Scan {
                    occurrence,
                    record,
                    pin,
                } => weft_core::application_ir::Scan {
                    occurrence: occurrence.clone(),
                    record: record.clone(),
                    pin: pin.clone(),
                },
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
        let mut application_plan = weft_core::application_ir::Plan {
            ir_version: "weft-ir/0.2.0".into(),
            module_pins: plan.module_pins.clone(),
            read_profile: None,
            required_capabilities: vec![],
            type_graph: descriptors.clone(),
            source: scan.clone(),
            page_key: None,
            joins: vec![],
            filters: vec![],
            groups: vec![],
            aggregate: false,
            outputs: vec![weft_core::application_ir::Output {
                name: "name".into(),
                expression: weft_core::application_ir::Expression::Field {
                    scan: scan.occurrence,
                    identity: member.identity.clone(),
                },
            }],
            order: vec![],
            limit: None,
        };
        let application_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&application_plan),
            ..context
        };
        let application_compiled = crate::select_definition::compile_with_registry(
            &application_context,
            &record_registry,
            &properties,
            &comparisons,
            |_, _, _, _| panic!("Direct application projection called native operator"),
        )
        .unwrap();
        if let Ok(path) = std::env::var("WEFT_APPLICATION_SELECT_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&json!({
                "sql": application_compiled.select.sql, "columns": application_compiled.select.columns,
                "checks": application_compiled.select.structural_checks.iter().cloned().chain(application_compiled.select.payload_checks.iter().map(|check| check.sql.clone())).collect::<Vec<_>>(),
                "parameters": application_compiled.parameters,
            })).unwrap()).unwrap();
        }
        assert_eq!(application_compiled.select.columns.len(), 1);
        assert!(application_compiled.select.sql.contains("AS \"name\""));
        assert_eq!(application_compiled.parameters.len(), 2);
        assert_eq!(application_compiled.select.payload_checks.len(), 1);
        application_plan.limit = Some(0);
        let application_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&application_plan),
            ..context
        };
        assert!(crate::select_definition::compile_with_registry(
            &application_context,
            &record_registry,
            &properties,
            &comparisons,
            |_, _, _, _| panic!("Unimplemented stage called native operator"),
        )
        .is_err());
        application_plan.limit = Some(3);
        application_plan.order = vec![weft_core::application_ir::Field {
            scan: application_plan.source.occurrence.clone(),
            identity: member.identity.clone(),
            logical_type: match &descriptors[0].shape {
                Shape::Scalar { logical_type } => logical_type.clone(),
                _ => unreachable!(),
            },
            span: Span { start: 0, end: 0 },
        }];
        let ordered_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&application_plan),
            ..context
        };
        let ordered = crate::select_definition::compile_with_registry(
            &ordered_context,
            &record_registry,
            &properties,
            &comparisons,
            |_, _, access, _| {
                Ok(format!(
                    "({}) COLLATE \"C\"",
                    access.unwrap().scalar_storage.as_ref().unwrap().carrier
                ))
            },
        )
        .unwrap();
        assert!(ordered.select.sql.ends_with("LIMIT 3"));
        assert!(ordered.select.sql.contains("ORDER BY"));
        if let Ok(path) = std::env::var("WEFT_APPLICATION_ORDER_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&json!({"sql":ordered.select.sql,"columns":ordered.select.columns,"checks":ordered.select.structural_checks.iter().cloned().chain(ordered.select.payload_checks.iter().map(|check|check.sql.clone())).collect::<Vec<_>>(),"parameters":ordered.parameters})).unwrap()).unwrap();
        }
        application_plan.order.clear();
        application_plan.filters = vec![weft_core::application_ir::Predicate::Equal {
            left: application_plan.order.first().cloned().unwrap_or_else(|| {
                weft_core::application_ir::Field {
                    scan: application_plan.source.occurrence.clone(),
                    identity: member.identity.clone(),
                    logical_type: match &descriptors[0].shape {
                        Shape::Scalar { logical_type } => logical_type.clone(),
                        _ => unreachable!(),
                    },
                    span: Span { start: 0, end: 0 },
                }
            }),
            right: weft_core::application_ir::Value::Literal {
                value: "A".into(),
                logical_type: match &descriptors[0].shape {
                    Shape::Scalar { logical_type } => logical_type.clone(),
                    _ => unreachable!(),
                },
                span: Span { start: 0, end: 0 },
            },
        }];
        let filtered_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&application_plan),
            ..context
        };
        let filtered = crate::select_definition::compile_with_registry(
            &filtered_context,
            &record_registry,
            &properties,
            &comparisons,
            |node, operands, access, parameters| match node {
                weft_core::ir::Expression::Field { .. } => Ok(format!(
                    "({}) COLLATE \"C\"",
                    access.unwrap().scalar_storage.as_ref().unwrap().carrier
                )),
                weft_core::ir::Expression::Literal {
                    value,
                    logical_type,
                    ..
                } => Ok(format!(
                    "{}::pg_catalog.text",
                    parameters.push(
                        logical_type.clone(),
                        value.clone(),
                        json!({"use":"fixture-literal"})
                    )?
                )),
                weft_core::ir::Expression::Equal { .. } => {
                    Ok(format!("({} = {})", operands[0], operands[1]))
                }
                _ => panic!("Filtered fixture native operation"),
            },
        )
        .unwrap();
        assert_eq!(filtered.parameters.len(), 3);
        assert_eq!(filtered.parameters[2].value, "A");
        if let Ok(path) = std::env::var("WEFT_APPLICATION_FILTER_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&json!({"sql":filtered.select.sql,"columns":filtered.select.columns,"checks":filtered.select.structural_checks.iter().cloned().chain(filtered.select.payload_checks.iter().map(|check|check.sql.clone())).collect::<Vec<_>>(),"parameters":filtered.parameters})).unwrap()).unwrap();
        }
        if let weft_core::application_ir::Predicate::Equal { right, .. } =
            &mut application_plan.filters[0]
        {
            let weft_core::application_ir::Value::Literal {
                value,
                logical_type,
                span,
            } = right.clone()
            else {
                unreachable!()
            };
            *right = weft_core::application_ir::Value::Parameter {
                name: "selected_name".into(),
                value,
                logical_type,
                span,
            };
        }
        let named_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&application_plan),
            ..context
        };
        let named = crate::select_definition::compile_with_registry(
            &named_context,
            &record_registry,
            &properties,
            &comparisons,
            |node, operands, access, parameters| match node {
                weft_core::ir::Expression::Field { .. } => Ok(format!(
                    "({}) COLLATE \"C\"",
                    access.unwrap().scalar_storage.as_ref().unwrap().carrier
                )),
                weft_core::ir::Expression::Literal {
                    value,
                    logical_type,
                    ..
                } => Ok(format!(
                    "{}::pg_catalog.text",
                    parameters.push(
                        logical_type.clone(),
                        value.clone(),
                        json!({"fixture":true})
                    )?
                )),
                weft_core::ir::Expression::Equal { .. } => {
                    Ok(format!("({} = {})", operands[0], operands[1]))
                }
                _ => unreachable!(),
            },
        )
        .unwrap();
        assert_eq!(named.parameters[2].origin["parameter"], "selected_name");
        assert_eq!(named.select.sql, filtered.select.sql);
        assert!(crate::select_definition::compile_with_registry(
            &named_context,
            &record_registry,
            &properties,
            &comparisons,
            |node, operands, access, _| match node {
                weft_core::ir::Expression::Field { .. } => Ok(access
                    .unwrap()
                    .scalar_storage
                    .as_ref()
                    .unwrap()
                    .carrier
                    .clone()),
                weft_core::ir::Expression::Literal { .. } => Ok("'A'::pg_catalog.text".into()),
                weft_core::ir::Expression::Equal { .. } =>
                    Ok(format!("({} = {})", operands[0], operands[1])),
                _ => unreachable!(),
            }
        )
        .is_err());

        let mut cursor_plan = application_plan.clone();
        let weft_core::application_ir::Predicate::Equal { left, right } = &cursor_plan.filters[0]
        else {
            unreachable!()
        };
        cursor_plan.filters = vec![weft_core::application_ir::Predicate::LexicographicGreater {
            columns: vec![left.clone()],
            values: vec![right.clone()],
        }];
        let cursor_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&cursor_plan),
            ..context
        };
        let cursor = crate::select_definition::compile_with_registry(
            &cursor_context,
            &record_registry,
            &properties,
            &comparisons,
            |node, _, access, parameters| match node {
                weft_core::ir::Expression::Field { .. } => Ok(format!(
                    "({}) COLLATE \"C\"",
                    access.unwrap().scalar_storage.as_ref().unwrap().carrier
                )),
                weft_core::ir::Expression::Literal {
                    value,
                    logical_type,
                    ..
                } => Ok(format!(
                    "{}::pg_catalog.text",
                    parameters.push(
                        logical_type.clone(),
                        value.clone(),
                        json!({"fixture":true})
                    )?
                )),
                _ => panic!("Cursor uses selected tuple comparison"),
            },
        )
        .unwrap();
        assert_eq!(cursor.parameters[2].origin["parameter"], "selected_name");
        assert!(cursor.select.sql.contains("ROW("));
        if let Ok(path) = std::env::var("WEFT_CURSOR_BRIDGE_CAPTURE") {
            std::fs::write(path,serde_json::to_vec_pretty(&json!({"sql":cursor.select.sql,"columns":cursor.select.columns,"checks":cursor.select.structural_checks.iter().cloned().chain(cursor.select.payload_checks.iter().map(|check|check.sql.clone())).collect::<Vec<_>>(),"parameters":cursor.parameters})).unwrap()).unwrap();
        }
        let mut named_plan = application_plan.clone();
        application_plan.filters.clear();

        application_plan.limit = None;
        application_plan.aggregate = true;
        application_plan.outputs = vec![weft_core::application_ir::Output {
            name: "count".into(),
            expression: weft_core::application_ir::Expression::Count {
                logical_type: weft_core::ir::LogicalType {
                    family: weft_core::ir::Family::Integer,
                    facets: json!({}),
                    nullable: false,
                },
            },
        }];
        let count_selection = weft_core::backend::Selection {
            records: vec![record.identity.clone()],
            ..Default::default()
        };
        let count_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&application_plan),
            selection: &count_selection,
            ..context
        };
        let count_compiled = crate::select_definition::compile_with_registry(
            &count_context,
            &record_registry,
            &BTreeMap::new(),
            &BTreeMap::new(),
            |_, _, _, _| panic!("Fieldless COUNT called a native field operator"),
        )
        .unwrap();
        assert_eq!(count_compiled.parameters.len(), 1);
        assert!(count_compiled.select.payload_checks.is_empty());
        assert!(count_compiled.select.structural_checks.is_empty());
        assert!(count_compiled
            .select
            .sql
            .contains("pg_catalog.count(*)::pg_catalog.text"));
        if let Ok(path) = std::env::var("WEFT_APPLICATION_COUNT_CAPTURE") {
            std::fs::write(
                path,
                serde_json::to_vec_pretty(&json!({
                    "sql": count_compiled.select.sql, "columns": count_compiled.select.columns,
                    "checks": [], "parameters": count_compiled.parameters,
                }))
                .unwrap(),
            )
            .unwrap();
        }
        let group_type = match &descriptors[0].shape {
            Shape::Scalar { logical_type } => logical_type.clone(),
            _ => unreachable!(),
        };
        application_plan.groups = vec![weft_core::application_ir::Field {
            scan: application_plan.source.occurrence.clone(),
            identity: member.identity.clone(),
            logical_type: group_type,
            span: Span { start: 0, end: 0 },
        }];
        application_plan.outputs.insert(
            0,
            weft_core::application_ir::Output {
                name: "name".into(),
                expression: weft_core::application_ir::Expression::Field {
                    scan: application_plan.source.occurrence.clone(),
                    identity: member.identity.clone(),
                },
            },
        );
        let group_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&application_plan),
            ..context
        };
        let group_compiled = crate::select_definition::compile_with_registry(
            &group_context,
            &record_registry,
            &properties,
            &comparisons,
            |node, _, access, _| match node {
                weft_core::ir::Expression::Field { .. } => Ok(format!(
                    "({}) COLLATE \"C\"",
                    access.unwrap().scalar_storage.as_ref().unwrap().carrier
                )),
                _ => panic!("Grouped fixture native operation"),
            },
        )
        .unwrap();
        assert!(group_compiled.select.sql.contains("GROUP BY"));
        assert_eq!(group_compiled.select.payload_checks.len(), 1);
        if let Ok(path) = std::env::var("WEFT_APPLICATION_GROUP_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&json!({
                "sql": group_compiled.select.sql, "columns": group_compiled.select.columns,
                "checks": group_compiled.select.structural_checks.iter().cloned().chain(group_compiled.select.payload_checks.iter().map(|check|check.sql.clone())).collect::<Vec<_>>(),
                "parameters": group_compiled.parameters,
            })).unwrap()).unwrap();
        }
        assert!(crate::select_definition::compile_with_registry(
            &group_context,
            &record_registry,
            &properties,
            &BTreeMap::new(),
            |_, _, _, _| panic!("Missing comparator reached grouped native lowering"),
        )
        .is_err());
        application_plan.aggregate = false;
        let wrong_count_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&application_plan),
            selection: &count_selection,
            ..context
        };
        assert!(crate::select_definition::compile_with_registry(
            &wrong_count_context,
            &record_registry,
            &BTreeMap::new(),
            &BTreeMap::new(),
            |_, _, _, _| panic!("Invalid COUNT called native operator"),
        )
        .is_err());
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
        let crate::registered_access::Location::Props(presence_location) = &accesses[0].location
        else {
            panic!("fixture props")
        };
        let mut presence_cases = Vec::new();
        for required in [false, true] {
            for nullable in [false, true] {
                let (carrier, integrity) = crate::result_definition::props_presence_sql(
                    presence_location,
                    &captured_storage.carrier,
                    &captured_storage.storage_integrity,
                    required,
                    nullable,
                );
                assert!(carrier.contains("'state','null'"));
                assert!(integrity.contains(&presence_location.root_integrity));
                presence_cases.push(json!({"required":required,"nullable":nullable,"sql":format!("SELECT ({carrier})::text AS value FROM {} WHERE {}",accesses[0].owner_source.sql,accesses[0].owner_source.discriminator),"check":format!("SELECT count(*) AS violations FROM {} WHERE {} AND ({integrity}) IS DISTINCT FROM TRUE",accesses[0].owner_source.sql,accesses[0].owner_source.discriminator)}));
            }
        }
        if let Ok(path) = std::env::var("WEFT_SCALAR_PRESENCE_CAPTURE") {
            std::fs::write(path,serde_json::to_vec_pretty(&json!({"cases":presence_cases,"parameters":access_parameters.clone().into_slots()})).unwrap()).unwrap();
        }
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
        let (left_scan, right_scan) = match &self_plan.root {
            weft_core::ir::Node::Project { input, .. } => match input.as_ref() {
                weft_core::ir::Node::InnerJoin { left, right, .. } => {
                    (left.as_ref(), right.as_ref())
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
        let to_scan = |node: &weft_core::ir::Node| match node {
            weft_core::ir::Node::Scan {
                occurrence,
                record,
                pin,
            } => weft_core::application_ir::Scan {
                occurrence: occurrence.clone(),
                record: record.clone(),
                pin: pin.clone(),
            },
            _ => unreachable!(),
        };
        let mut join_plan = application_plan.clone();
        join_plan.aggregate = false;
        join_plan.groups.clear();
        join_plan.order.clear();
        join_plan.limit = None;
        join_plan.source = to_scan(left_scan);
        let right_scan = to_scan(right_scan);
        let field = |scan: &String| weft_core::application_ir::Field {
            scan: scan.clone(),
            identity: member.identity.clone(),
            logical_type: match &descriptors[0].shape {
                Shape::Scalar { logical_type } => logical_type.clone(),
                _ => unreachable!(),
            },
            span: Span { start: 0, end: 0 },
        };
        join_plan.outputs = vec![
            weft_core::application_ir::Output {
                name: "left_name".into(),
                expression: weft_core::application_ir::Expression::Field {
                    scan: join_plan.source.occurrence.clone(),
                    identity: member.identity.clone(),
                },
            },
            weft_core::application_ir::Output {
                name: "right_name".into(),
                expression: weft_core::application_ir::Expression::Field {
                    scan: right_scan.occurrence.clone(),
                    identity: member.identity.clone(),
                },
            },
        ];
        join_plan.joins = vec![weft_core::application_ir::Join {
            on: vec![weft_core::application_ir::Predicate::Equal {
                left: field(&join_plan.source.occurrence),
                right: weft_core::application_ir::Value::Field {
                    field: field(&right_scan.occurrence),
                },
            }],
            right: right_scan,
        }];
        let join_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&join_plan),
            ..context
        };
        let joined = crate::select_definition::compile_with_registry(
            &join_context,
            &record_registry,
            &properties,
            &comparisons,
            |node, operands, access, _| match node {
                weft_core::ir::Expression::Field { .. } => Ok(format!(
                    "({}) COLLATE \"C\"",
                    access.unwrap().scalar_storage.as_ref().unwrap().carrier
                )),
                weft_core::ir::Expression::Equal { .. } => {
                    Ok(format!("({} = {})", operands[0], operands[1]))
                }
                _ => unreachable!(),
            },
        )
        .unwrap();
        let mut tuple_plan = join_plan.clone();
        tuple_plan.joins[0].on = vec![weft_core::application_ir::Predicate::Equal {
            left: field(&tuple_plan.source.occurrence),
            right: weft_core::application_ir::Value::Field {
                field: field(&tuple_plan.source.occurrence),
            },
        }];
        let tuple_fields = vec![
            field(&tuple_plan.source.occurrence),
            field(&tuple_plan.joins[0].right.occurrence),
        ];
        tuple_plan.filters = vec![weft_core::application_ir::Predicate::LexicographicGreater {
            columns: tuple_fields.clone(),
            values: tuple_fields
                .iter()
                .enumerate()
                .map(
                    |(index, field)| weft_core::application_ir::Value::Parameter {
                        name: format!("cursor_{index}"),
                        value: "A".into(),
                        logical_type: field.logical_type.clone(),
                        span: field.span.clone(),
                    },
                )
                .collect(),
        }];
        let tuple_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&tuple_plan),
            ..context
        };
        let tuple = crate::select_definition::compile_with_registry(
            &tuple_context,
            &record_registry,
            &properties,
            &comparisons,
            |node, operands, access, parameters| match node {
                weft_core::ir::Expression::Field { .. } => Ok(format!(
                    "({}) COLLATE \"C\"",
                    access.unwrap().scalar_storage.as_ref().unwrap().carrier
                )),
                weft_core::ir::Expression::Literal {
                    value,
                    logical_type,
                    ..
                } => Ok(format!(
                    "{}::pg_catalog.text",
                    parameters.push(
                        logical_type.clone(),
                        value.clone(),
                        json!({"fixture":true})
                    )?
                )),
                weft_core::ir::Expression::Equal { .. } => {
                    Ok(format!("({} = {})", operands[0], operands[1]))
                }
                _ => unreachable!(),
            },
        )
        .unwrap();
        assert_eq!(tuple.parameters.len(), 6);
        assert_eq!(tuple.parameters[4].origin["parameter"], "cursor_0");
        assert_eq!(tuple.parameters[5].origin["parameter"], "cursor_1");
        if let Ok(path) = std::env::var("WEFT_COMPOSITE_CURSOR_CAPTURE") {
            std::fs::write(path,serde_json::to_vec_pretty(&json!({"sql":tuple.select.sql,"columns":tuple.select.columns,"checks":tuple.select.structural_checks.iter().cloned().chain(tuple.select.payload_checks.iter().map(|check|check.sql.clone())).collect::<Vec<_>>(),"parameters":tuple.parameters})).unwrap()).unwrap();
        }
        assert_eq!(joined.select.structural_checks.len(), 2);
        assert_eq!(joined.select.payload_checks.len(), 2);
        assert_eq!(joined.parameters.len(), 4);
        if let Ok(path) = std::env::var("WEFT_APPLICATION_JOIN_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&json!({"sql":joined.select.sql,"columns":joined.select.columns,"checks":joined.select.structural_checks.iter().cloned().chain(joined.select.payload_checks.iter().map(|check|check.sql.clone())).collect::<Vec<_>>(),"parameters":joined.parameters})).unwrap()).unwrap();
        }
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
        let compiled = crate::select_definition::compile_with_registry(
            &self_context,
            &record_registry,
            &properties,
            &comparisons,
            |node, operands, access, _| match node {
                weft_core::ir::Expression::Field { .. } => Ok(access
                    .unwrap()
                    .scalar_storage
                    .as_ref()
                    .unwrap()
                    .carrier
                    .clone()),
                weft_core::ir::Expression::Equal { .. } => {
                    Ok(format!("({} = {})", operands[0], operands[1]))
                }
                _ => panic!("fixture native operation"),
            },
        )
        .unwrap();
        if let Ok(path) = std::env::var("WEFT_REGISTRY_SELECT_CAPTURE") {
            std::fs::write(path,serde_json::to_vec_pretty(&json!({
                "sql":compiled.select.sql,"columns":compiled.select.columns,
                "checks":compiled.select.structural_checks.iter().cloned().chain(compiled.select.payload_checks.iter().map(|observation|observation.sql.clone())).collect::<Vec<_>>(),
                "parameters":compiled.parameters,
            })).unwrap()).unwrap();
        }
        assert_eq!(compiled.select.sql, selected.sql);
        assert_eq!(
            serde_json::to_value(&compiled.parameters).unwrap(),
            self_parameters_before
        );
        assert_eq!(
            compiled.select.structural_checks,
            selected.structural_checks
        );
        assert_eq!(
            compiled
                .select
                .payload_checks
                .iter()
                .map(|check| &check.sql)
                .collect::<Vec<_>>(),
            selected
                .payload_checks
                .iter()
                .map(|check| &check.sql)
                .collect::<Vec<_>>()
        );
        assert!(crate::select_definition::compile_with_registry(
            &self_context,
            &BTreeMap::new(),
            &properties,
            &comparisons,
            |_, _, _, _| panic!("Incomplete registry reached native callback"),
        )
        .is_err());
        assert!(crate::select_definition::compile_with_registry(
            &self_context,
            &record_registry,
            &properties,
            &comparisons,
            |_, _, _, _| Err(weft_core::error::Diagnostic::new(
                "WFT-CAPABILITY",
                "emit",
                "Native operation refused"
            )),
        )
        .is_err());
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
        let row_record = crate::record_definition::RecordAdmission::admit(
            &row_admission,
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
        row_record.verify_property(&row_property).unwrap();
        let row_records = BTreeMap::from([(
            serde_json::to_string(row_record.identity()).unwrap(),
            row_record,
        )]);
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
        let row_join_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&join_plan),
            ..row_context
        };
        let row_join_compiled = crate::select_definition::compile_with_registry(
            &row_join_context,
            &row_records,
            &row_properties,
            &comparisons,
            |node, operands, access, _| match node {
                weft_core::ir::Expression::Field { .. } => {
                    let crate::registered_access::Location::Row(location) =
                        &access.unwrap().location
                    else {
                        unreachable!()
                    };
                    Ok(format!(
                        "({}) COLLATE \"C\"",
                        location.scalar_observation().text
                    ))
                }
                weft_core::ir::Expression::Equal { .. } => {
                    Ok(format!("({} = {})", operands[0], operands[1]))
                }
                _ => unreachable!(),
            },
        )
        .unwrap();
        assert_eq!(row_join_compiled.select.payload_checks.len(), 2);
        assert_eq!(row_join_compiled.parameters.len(), 6);
        if let Ok(path) = std::env::var("WEFT_APPLICATION_ROW_JOIN_CAPTURE") {
            let codec_hex: String = row_properties[&registration].value.graph.artifacts
                ["/nodes/0/codecDefinition"]
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            std::fs::write(path, serde_json::to_vec_pretty(&json!({"codecHex":codec_hex,"sql":row_join_compiled.select.sql,"columns":row_join_compiled.select.columns,"checks":row_join_compiled.select.structural_checks.iter().cloned().chain(row_join_compiled.select.payload_checks.iter().map(|check|check.sql.clone())).collect::<Vec<_>>(),"parameters":row_join_compiled.parameters,"ownerTypeId":row_properties[&registration].owner_catalog_id,"propertyId":row_properties[&registration].property_catalog_id})).unwrap()).unwrap();
        }
        let mut row_parameters = crate::Parameters::default();
        let row_accesses = crate::registered_access::lower_plan(
            &row_context,
            &row_properties,
            &comparisons,
            &mut row_parameters,
        )
        .unwrap();
        assert_eq!(row_accesses.len(), 1);

        // Synthetic text custody exercises the admitted original-property gate;
        // source/codec interpretation is explicitly supplied by this fixture.
        let mut cells: [Option<String>; 23] = std::array::from_fn(|_| None);
        for (index, text) in [
            (0, "1"),
            (1, "10"),
            (3, "scalar"),
            (4, "root"),
            (8, "00"),
            (9, "00"),
            (10, "1"),
            (11, "10"),
            (12, "text"),
            (13, "é  "),
            (21, "00"),
            (22, "00"),
        ] {
            cells[index] = Some(text.into());
        }
        let raw: Vec<Vec<Option<&str>>> = vec![cells.iter().map(Option::as_deref).collect()];
        let native_rows = crate::row_custody::admit_tree_rows(
            &raw,
            &mut crate::row_custody::Budget {
                remaining_cells: 100,
                remaining_bytes: 10000,
            },
        )
        .unwrap();
        let native_tree = crate::row_custody::index_tree(
            &native_rows,
            "1",
            "10",
            &mut crate::row_custody::TreeBudget {
                remaining_nodes: 10,
                max_depth: 8,
            },
        )
        .unwrap();
        let make_budget = || crate::value_traversal::Budget {
            remaining_nodes: 10,
            remaining_key_bytes: 100,
            remaining_members: 10,
            max_depth: 8,
        };
        let native_body = crate::row_value_traversal::decode_property(
            &row_properties[&registration],
            &row_accesses[0],
            &native_tree,
            &mut make_budget(),
            |node, row| {
                assert_eq!(
                    node.codec_bytes,
                    row_properties[&registration].value.graph.artifacts["/nodes/0/codecDefinition"]
                );
                assert_eq!(row.bytes[&9], vec![0]);
                Ok(())
            },
            |node, row| {
                assert!(matches!(
                    node.shape,
                    crate::value_definition::LayoutShape::Scalar {
                        family: "string",
                        ..
                    }
                ));
                Ok(json!(row.cells[13].as_deref().unwrap()))
            },
            |_, _| panic!("scalar has no fields"),
            |_| panic!("scalar has no members"),
        )
        .unwrap();
        assert_eq!(native_body, json!("é  "));

        let decode_native_logical = |budget: &mut crate::value_traversal::Budget| {
            crate::row_value_traversal::decode_logical_property(
                &row_properties[&registration],
                &row_accesses[0],
                &native_tree,
                budget,
                |_, _| Ok(()),
                |_, row| Ok(json!(row.cells[13].as_deref().unwrap())),
                |_, _| panic!("scalar identity correspondence"),
            )
        };
        assert_eq!(
            decode_native_logical(&mut make_budget()).unwrap(),
            json!("é  ")
        );
        let mut shared_limit = make_budget();
        shared_limit.remaining_nodes = 1;
        assert!(decode_native_logical(&mut shared_limit).is_err());
        assert_eq!(shared_limit.remaining_nodes, 0);
        assert!(crate::row_value_traversal::decode_property(
            &properties[&registration],
            &row_accesses[0],
            &native_tree,
            &mut make_budget(),
            |_, _| panic!("substituted original property reached observer"),
            |_, _| panic!("substituted original property reached decoder"),
            |_, _| panic!(),
            |_| panic!(),
        )
        .is_err());
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
        let crate::registered_access::Location::Row(native_presence_location) =
            &row_accesses[0].location
        else {
            panic!("fixture row home")
        };
        let scan = crate::registered_access::scan_source(
            match &plan.root {
                weft_core::ir::Node::Project { input, .. } => input,
                _ => panic!("fixture projection"),
            },
            &row_accesses,
        )
        .unwrap();
        let observation = native_presence_location.scalar_observation();
        let scalar_integrity = format!(
            "({} AND {}='{}')",
            observation.payload_integrity(logical.family.clone()),
            observation.codec_bytes_hex,
            codec_hex
        );
        let mut native_presence_cases = Vec::new();
        for required in [false, true] {
            for nullable in [false, true] {
                let (carrier, integrity) = crate::result_definition::row_presence_sql(
                    native_presence_location,
                    &observation.text,
                    &scalar_integrity,
                    required,
                    nullable,
                );
                assert!(integrity.contains("value_kind"));
                native_presence_cases.push(json!({"required":required,"nullable":nullable,
                "sql":format!("SELECT ({carrier})::text AS value FROM {} WHERE {}",scan.source.sql,scan.source.filters.join(" AND ")),
                "check":format!("SELECT count(*) AS violations FROM {} WHERE {} AND ({integrity}) IS DISTINCT FROM TRUE",scan.source.sql,scan.source.filters.join(" AND ")),
                "structuralChecks":scan.structural_check_sql}));
            }
        }
        if let Ok(path) = std::env::var("WEFT_ROW_PRESENCE_CAPTURE") {
            std::fs::write(path,serde_json::to_vec_pretty(&json!({"cases":native_presence_cases,"parameters":row_parameters.clone().into_slots(),"codecHex":codec_hex,
                "ownerTypeId":row_properties[&registration].owner_catalog_id,"propertyId":row_properties[&registration].property_catalog_id})).unwrap()).unwrap();
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

        fn registered_native(
            node: &weft_core::ir::Expression,
            operands: &[String],
            access: Option<&crate::registered_access::Access<'_>>,
            parameters: &mut crate::Parameters,
        ) -> weft_core::error::Result<String> {
            match node {
                weft_core::ir::Expression::Field { .. } => Ok(access
                    .unwrap()
                    .scalar_storage
                    .as_ref()
                    .unwrap()
                    .carrier
                    .clone()),
                weft_core::ir::Expression::Equal { .. } => {
                    Ok(format!("({} = {})", operands[0], operands[1]))
                }
                weft_core::ir::Expression::Literal {
                    value,
                    logical_type,
                    span,
                } => Ok(format!(
                    "{}::pg_catalog.text",
                    parameters.push(
                        logical_type.clone(),
                        value.clone(),
                        json!({"literalSpan":span})
                    )?
                )),
                _ => panic!("registered fixture native operation"),
            }
        }
        let backend = crate::original_backend::OriginalBackend::new(
            self_context.binding,
            record_registry,
            properties,
            comparisons,
            registered_native,
        )
        .unwrap();
        let mut registry = weft_core::backend::Registry::default();
        registry.register(backend).unwrap();
        let mut target = weft_core::backend::Target {
            backend_id: "truss.postgresql.original".into(),
            backend_version: "0.1.0-candidate".into(),
            profile_id: "pg17.9-candidate".into(),
            allow_candidate: true,
        };
        let public = registry
            .compile(
                &catalog,
                weft_core::backend::Plan::V01(&self_plan),
                &target,
                self_context.binding,
            )
            .unwrap();
        assert_eq!(public.emission.sql, compiled.select.sql);
        assert_eq!(
            serde_json::to_value(&public.emission.parameters).unwrap(),
            serde_json::to_value(&compiled.parameters).unwrap()
        );
        assert!(public
            .emission
            .obligations
            .iter()
            .any(|o| o.id == "truss.original.complete-read-context"));
        let checks: Vec<_> = public
            .emission
            .obligations
            .iter()
            .filter(|o| o.id.starts_with("truss.original.owner-"))
            .collect();
        assert_eq!(
            checks.len(),
            compiled.select.structural_checks.len() + compiled.select.payload_checks.len()
        );
        for check in checks {
            assert_eq!(check.parameters["beforeQuery"], true);
            assert_eq!(check.parameters["expectedViolations"], "0");
            assert_eq!(
                check.parameters["parameters"],
                serde_json::to_value(&compiled.parameters).unwrap()
            );
        }
        join_plan.required_capabilities = vec![
            "scan".into(),
            "project".into(),
            "innerJoin".into(),
            "equal".into(),
            "type.string".into(),
        ];
        let public_application = registry
            .compile(
                &catalog,
                weft_core::backend::Plan::V02(&join_plan),
                &target,
                self_context.binding,
            )
            .unwrap();
        assert!(public_application.emission.sql.contains("INNER JOIN"));
        assert_eq!(
            serde_json::to_value(&public_application.emission.columns).unwrap(),
            serde_json::to_value(&joined.select.columns).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&public_application.emission.parameters).unwrap(),
            serde_json::to_value(&joined.parameters).unwrap()
        );
        if let Ok(path) = std::env::var("WEFT_PUBLIC_APPLICATION_CAPTURE") {
            let checks: Vec<_> = public_application
                .emission
                .obligations
                .iter()
                .filter_map(|obligation| {
                    obligation
                        .parameters
                        .get("sql")
                        .and_then(|sql| sql.as_str())
                })
                .collect();
            std::fs::write(path, serde_json::to_vec_pretty(&json!({"sql":public_application.emission.sql,"columns":public_application.emission.columns,"checks":checks,"parameters":public_application.emission.parameters})).unwrap()).unwrap();
        }
        assert_eq!(public_application.emission.parameters.len(), 4);
        assert!(public_application
            .emission
            .obligations
            .iter()
            .any(|o| o.id == "truss.original.complete-read-context"));
        let declaration = registry.manifest("truss.postgresql.original").unwrap();
        assert!(declaration
            .language_profiles
            .iter()
            .any(|profile| profile.ir_version == "weft-ir/0.2.0"));
        assert!(declaration
            .capabilities
            .iter()
            .find(|capability| capability.id == "relationship.exists")
            .unwrap()
            .language_profiles
            .iter()
            .all(|profile| profile.ir_version != "weft-ir/0.2.0"));
        join_plan
            .required_capabilities
            .push("relationship.exists".into());
        assert!(registry
            .compile(
                &catalog,
                weft_core::backend::Plan::V02(&join_plan),
                &target,
                self_context.binding
            )
            .is_err());
        named_plan.required_capabilities = vec![
            "scan".into(),
            "project".into(),
            "filter".into(),
            "equal".into(),
            "type.string".into(),
            "parameter.named".into(),
            "limit".into(),
        ];
        let public_named = registry
            .compile(
                &catalog,
                weft_core::backend::Plan::V02(&named_plan),
                &target,
                self_context.binding,
            )
            .unwrap();
        assert_eq!(
            public_named.emission.parameters[2].origin["parameter"],
            "selected_name"
        );
        assert_eq!(public_named.emission.parameters[2].value, "A");
        if let Ok(path) = std::env::var("WEFT_PUBLIC_NAMED_CAPTURE") {
            let checks: Vec<_> = public_named
                .emission
                .obligations
                .iter()
                .filter_map(|obligation| {
                    obligation
                        .parameters
                        .get("sql")
                        .and_then(|sql| sql.as_str())
                })
                .collect();
            std::fs::write(path,serde_json::to_vec_pretty(&json!({"sql":public_named.emission.sql,"columns":public_named.emission.columns,"checks":checks,"parameters":public_named.emission.parameters})).unwrap()).unwrap();
        }
        target.allow_candidate = false;
        assert!(registry
            .compile(
                &catalog,
                weft_core::backend::Plan::V01(&self_plan),
                &target,
                self_context.binding
            )
            .is_err());
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
