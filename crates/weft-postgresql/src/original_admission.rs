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
