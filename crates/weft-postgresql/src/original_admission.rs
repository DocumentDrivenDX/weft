//! Pure original-byte admission batch for trusted embedding composition.
use crate::{
    binding::Admission,
    original_backend::{Native, OriginalBackend},
    property_definition, record_definition,
};
use std::collections::BTreeMap;
use weft_core::{
    backend::BindingInput,
    error::{Diagnostic, Result},
    ir::Identity,
    model::Catalog,
};
pub struct RecordSelection<'a> {
    pub index: usize,
    pub physical: record_definition::Selection<'a>,
}
pub struct PropertySelection<'a> {
    pub index: usize,
    pub value: property_definition::Selection<'a>,
    pub physical: property_definition::PhysicalSelection<'a>,
    pub native_tree: Option<crate::row_tree_mapping::Procedures>,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "admit", message)
}
/// Reconstruct private admissions from original bytes and exact UMF identities.
/// Selections are trusted Rust interpretations, not binding-authored executable
/// plugins or SQL. Nothing is registered until the entire batch succeeds.
pub fn backend(
    catalog: &Catalog,
    binding: &BindingInput,
    records: Vec<RecordSelection<'_>>,
    properties: Vec<PropertySelection<'_>>,
    comparators: BTreeMap<String, crate::native_comparator_definition::Definition>,
    native: Native,
) -> Result<OriginalBackend> {
    if weft_core::json::sha256(binding.json.as_bytes()) != binding.sha256 {
        return Err(fail("Original admission binding digest differs"));
    }
    let admission = Admission::parse(&binding.json, &binding.profile)?;
    let mut admitted_records = BTreeMap::new();
    for selected in records {
        let record = record_definition::RecordAdmission::admit(
            &admission,
            selected.index,
            catalog,
            selected.physical,
        )?;
        let key = serde_json::to_string(record.identity())
            .map_err(|_| fail("Original Record identity cannot serialize"))?;
        if admitted_records.insert(key, record).is_some() {
            return Err(fail("Repeated original Record selection"));
        }
    }
    let mut admitted_properties = BTreeMap::new();
    for selected in properties {
        let mapped = admission.value["properties"]
            .get(selected.index)
            .ok_or_else(|| fail("Selected property index missing"))?;
        let identity: Identity = serde_json::from_value(mapped["logical"].clone())
            .map_err(|_| fail("Original property identity invalid"))?;
        let owner = admission.value["entities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["typeId"] == mapped["ownerTypeId"])
            .ok_or_else(|| fail("Original property owner missing"))?;
        let owner_identity: Identity = serde_json::from_value(owner["logical"].clone())
            .map_err(|_| fail("Original owner identity invalid"))?;
        let record = catalog.record_by_identity(&owner_identity)?;
        let (_, descriptors) = catalog.member_descriptor_by_identity(&record, &identity)?;
        let mut property = property_definition::admit_property(
            &admission,
            selected.index,
            catalog,
            &descriptors,
            selected.value,
            selected.physical,
        )?;
        if let Some(procedures) = selected.native_tree {
            property = property.with_native_tree(procedures)?;
        }
        let owner_key = serde_json::to_string(&owner_identity)
            .map_err(|_| fail("Owner identity cannot serialize"))?;
        admitted_records
            .get(&owner_key)
            .ok_or_else(|| fail("Selected property lacks independent original Record admission"))?
            .verify_property(&property)?;
        let key =
            crate::comparator_requirements::registration_key(&property.owner, &property.identity);
        if admitted_properties.insert(key, property).is_some() {
            return Err(fail("Repeated original property selection"));
        }
    }
    OriginalBackend::new(
        binding,
        admitted_records,
        admitted_properties,
        comparators,
        native,
    )
}

/// Owned, statically selected meanings usable by native and browser hosts.
/// No serde implementation intentionally: procedure selection stays Rust code.
pub struct Configuration {
    pub binding_profile: String,
    pub records: Vec<OwnedRecordSelection>,
    pub properties: Vec<OwnedPropertySelection>,
    pub comparators: BTreeMap<String, crate::native_comparator_definition::Definition>,
    pub native: Native,
    pub relationships: Vec<OwnedRelationshipSelection>,
}
pub struct OwnedRecordSelection {
    pub index: usize,
    pub inventory: crate::leaf_codec_definition::OriginalArtifact,
    pub relation_identity: String,
    pub discriminator_identity: String,
    pub relations: BTreeMap<String, String>,
    pub columns: BTreeMap<String, crate::row_join_definition::Column>,
}
pub struct OwnedPropertySelection {
    pub index: usize,
    pub value_profile: serde_json::Value,
    pub presence_profile: serde_json::Value,
    pub leaf_codecs: BTreeMap<String, crate::leaf_codec_definition::Definition>,
    pub record_presence: BTreeMap<String, crate::presence_definition::Definition>,
    pub physical_profile: serde_json::Value,
    pub inventory: crate::leaf_codec_definition::OriginalArtifact,
    pub relations: BTreeMap<String, String>,
    pub columns: BTreeMap<String, crate::row_join_definition::Column>,
    pub row_join: Option<std::sync::Arc<crate::row_join_definition::Definition>>,
    pub obligations: std::collections::BTreeSet<String>,
    pub edge_association: Option<(
        serde_json::Value,
        crate::leaf_codec_definition::OriginalArtifact,
    )>,
    pub native_tree: Option<crate::row_tree_mapping::Procedures>,
}
#[derive(Clone)]
pub struct OwnedRelationshipSelection {
    pub index: usize,
    pub inverse: bool,
    pub profile: serde_json::Value,
    pub inventory: crate::leaf_codec_definition::OriginalArtifact,
    pub relation_identity: String,
    pub relations: BTreeMap<String, String>,
    pub columns: BTreeMap<String, crate::row_join_definition::Column>,
    pub relationship_type: String,
    pub source_id: String,
    pub source_type: String,
    pub target_id: String,
    pub target_type: String,
}
impl Configuration {
    /// Re-admit a request's original bytes; no admitted backend is cached.
    pub fn backend(&self, catalog: &Catalog, binding: &BindingInput) -> Result<OriginalBackend> {
        if binding.profile != self.binding_profile {
            return Err(fail("Original configuration binding profile differs"));
        }
        let records = self
            .records
            .iter()
            .map(|r| RecordSelection {
                index: r.index,
                physical: record_definition::Selection {
                    inventory: &r.inventory,
                    relation_identity: &r.relation_identity,
                    discriminator_identity: &r.discriminator_identity,
                    relations: &r.relations,
                    columns: &r.columns,
                },
            })
            .collect();
        let properties = self
            .properties
            .iter()
            .map(|p| PropertySelection {
                index: p.index,
                value: property_definition::Selection {
                    value_profile: &p.value_profile,
                    presence_profile: &p.presence_profile,
                    leaf_codecs: &p.leaf_codecs,
                    record_presence: &p.record_presence,
                },
                physical: property_definition::PhysicalSelection {
                    profile: &p.physical_profile,
                    inventory: &p.inventory,
                    relations: &p.relations,
                    columns: &p.columns,
                    row_join: p.row_join.as_deref(),
                    obligations: &p.obligations,
                    edge_association: p
                        .edge_association
                        .as_ref()
                        .map(|(profile, artifact)| (profile, artifact)),
                },
                native_tree: p.native_tree,
            })
            .collect();
        backend(
            catalog,
            binding,
            records,
            properties,
            self.comparators.clone(),
            self.native,
        )?
        .admit_owned_relationships(catalog, binding, &self.relationships)
    }
    /// Shared serialized compiler transport for an explicitly selected host.
    pub fn compile_json(&self, request: &str) -> String {
        let mut factory =
            |catalog: &Catalog,
             _: weft_core::backend::Plan<'_>,
             target: weft_core::compile::CompositionInput<'_>| {
                if target.backend_id != "truss.postgresql.original" {
                    return Err(fail(
                        "Original configuration cannot substitute selected backend",
                    ));
                }
                let binding = BindingInput {
                    profile: self.binding_profile.clone(),
                    json: target.binding_json.into(),
                    sha256: target.binding_sha256.into(),
                };
                let mut registry = weft_core::backend::Registry::default();
                registry.register(self.backend(catalog, &binding)?)?;
                Ok(registry)
            };
        weft_core::compile::Compiler::default().compile_json_with_factory(request, &mut factory)
    }
}
/// Test data only: preserve original inputs for independently checked embedding
/// conformance. Procedure pointers remain in trusted Rust; this is not a plugin
/// registration or executable serialization protocol.
#[cfg(test)]
impl Configuration {
    pub(crate) fn conformance_capture(&self) -> serde_json::Value {
        use base64::{engine::general_purpose::STANDARD, Engine};
        use serde_json::json;
        let artifact = |a: &crate::leaf_codec_definition::OriginalArtifact| json!({"identity":a.identity,"bytesBase64":STANDARD.encode(&a.bytes),"sha256":weft_core::json::sha256(&a.bytes)});
        let columns = |cs: &BTreeMap<String, crate::row_join_definition::Column>| {
            cs.iter()
                .map(|(id, c)| {
                    (
                        id.clone(),
                        json!({"relationIdentity":c.relation_identity,"name":c.name}),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };
        let originals = |items: &BTreeMap<String, Vec<u8>>| {
            items
                .iter()
                .map(|(path, bytes)| (path.clone(), STANDARD.encode(bytes)))
                .collect::<BTreeMap<_, _>>()
        };
        json!({
            "interfaceVersion":"weft-original-conformance-composition/0.1.0","bindingProfile":self.binding_profile,
            "records":self.records.iter().map(|r|json!({"index":r.index,"inventory":artifact(&r.inventory),"relationIdentity":r.relation_identity,"discriminatorIdentity":r.discriminator_identity,"relations":r.relations,"columns":columns(&r.columns)})).collect::<Vec<_>>(),
            "properties":self.properties.iter().map(|p|json!({"index":p.index,"valueProfile":p.value_profile,"presenceProfile":p.presence_profile,"leafCodecs":p.leaf_codecs.iter().map(|(id,d)|(id.clone(),json!({"originalJson":d.original_json,"originalArtifacts":originals(&d.original_artifacts)}))).collect::<BTreeMap<_,_>>(),"recordPresence":p.record_presence.iter().map(|(path,d)|(path.clone(),json!({"originalJson":d.original_json,"acceptedDefinitionBase64":STANDARD.encode(&d.accepted_definition)}))).collect::<BTreeMap<_,_>>(),"physicalProfile":p.physical_profile,"inventory":artifact(&p.inventory),"relations":p.relations,"columns":columns(&p.columns),"rowJoin":p.row_join.as_ref().map(|j|json!({"originalJson":j.original_json,"originalArtifacts":originals(&j.original_artifacts)})),"obligations":p.obligations,"nativeTree":p.native_tree.is_some()})).collect::<Vec<_>>(),
            "comparators":self.comparators.iter().map(|(key,c)|(key.clone(),json!({"originalJson":c.original_json,"originalArtifacts":originals(&c.original_artifacts)}))).collect::<BTreeMap<_,_>>(),
            "relationships":self.relationships.iter().map(|r|json!({"index":r.index,"inverse":r.inverse,"profile":r.profile,"inventory":artifact(&r.inventory),"relationIdentity":r.relation_identity,"relations":r.relations,"columns":columns(&r.columns),"relationshipType":r.relationship_type,"sourceId":r.source_id,"sourceType":r.source_type,"targetId":r.target_id,"targetType":r.target_type})).collect::<Vec<_>>(),
            "procedureScope":"Conformance data only; trusted Rust selects all native/operator/source interpretation, never executable metadata"
        })
    }
}
