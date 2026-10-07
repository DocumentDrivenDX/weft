//! Original Record source admission, independent of selected property accesses.
use crate::{
    binding::Admission,
    leaf_codec_definition::OriginalArtifact,
    property_definition::{OwnerMapping, OwnerSource},
    row_join_definition::{Column, RecordKind},
    Identifier, Parameters,
};
use std::collections::BTreeMap;
use weft_core::{
    backend::{Context, Plan},
    error::{Diagnostic, Result},
    ir::{Identity, ModelPin, Node},
    json::{checked_json, sha256},
    model::Catalog,
};
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
/// Physical selectors are a trusted registry interpretation of this exact
/// inventory. No binding-authored SQL or name-only layout inference is admitted.
pub struct Selection<'a> {
    pub inventory: &'a OriginalArtifact,
    pub relation_identity: &'a str,
    pub discriminator_identity: &'a str,
    pub relations: &'a BTreeMap<String, String>,
    pub columns: &'a BTreeMap<String, Column>,
}
#[derive(Debug)]
pub struct RecordAdmission {
    owner: Identity,
    model_pin: ModelPin,
    binding_sha256: String,
    catalog_id: String,
    mapping: OwnerMapping,
    original_record: weft_core::model::Record,
}
impl RecordAdmission {
    pub fn admit(
        binding: &Admission,
        index: usize,
        catalog: &Catalog,
        selected: Selection<'_>,
    ) -> Result<Self> {
        let entity = binding.value["entities"]
            .get(index)
            .ok_or_else(|| fail("Missing original entity mapping"))?;
        let owner: Identity = serde_json::from_value(entity["logical"].clone())
            .map_err(|_| fail("Malformed entity identity"))?;
        let input = catalog
            .inputs
            .iter()
            .find(|input| {
                input.pin.document_id == owner.document_id
                    && input.pin.revision == owner.revision
                    && input.selected_module_ids.contains(&owner.module)
            })
            .ok_or_else(|| fail("Entity is outside the original selected model"))?;
        if sha256(input.document_json.as_bytes()) != input.pin.sha256 {
            return Err(fail("Record model bytes differ from original pinned input"));
        }
        let document = checked_json(&input.document_json)
            .map_err(|_| fail("Original record document refused"))?;
        let record = document["modules"]
            .as_array()
            .and_then(|modules| modules.iter().find(|module| module["id"] == owner.module))
            .and_then(|module| module["elements"].as_array())
            .and_then(|elements| {
                elements
                    .iter()
                    .find(|element| element["id"] == owner.element)
            })
            .ok_or_else(|| fail("Original Record is missing"))?;
        if record["kind"] != "record"
            || binding.decoded_json(&format!("/entities/{index}/source"))? != document
            || binding.decoded_json(&format!("/entities/{index}/acceptedDefinition"))? != *record
        {
            return Err(fail(
                "Entity source or accepted definition differs from original Record",
            ));
        }
        if binding.value["basis"]["layoutInventory"]["identity"] != selected.inventory.identity
            || binding
                .artifacts
                .get("/basis/layoutInventory")
                .is_none_or(|bytes| bytes != &selected.inventory.bytes)
            || selected
                .relations
                .get(selected.relation_identity)
                .is_none_or(|name| name != "object")
            || selected
                .columns
                .get(selected.discriminator_identity)
                .is_none_or(|column| {
                    column.relation_identity != selected.relation_identity
                        || column.name != "type_id"
                })
        {
            return Err(fail(
                "Record physical owner differs from registered original inventory",
            ));
        }
        // Entity Record mappings select object owners. Logical edge scans require
        // their separate association contract and cannot enter by discriminator.
        Ok(Self {
            original_record: weft_core::model::Record {
                identity: owner.clone(),
                pin: input.pin.clone(),
                value: record.clone(),
                document: catalog
                    .inputs
                    .iter()
                    .position(|candidate| candidate.pin == input.pin)
                    .ok_or_else(|| fail("Record document index missing"))?,
            },
            owner,
            model_pin: input.pin.clone(),
            binding_sha256: sha256(binding.original_json.as_bytes()),
            catalog_id: entity["typeId"]
                .as_str()
                .ok_or_else(|| fail("Missing entity catalog ID"))?
                .into(),
            mapping: OwnerMapping {
                record_kind: RecordKind::Object,
                relation: Identifier::new("object")?,
                discriminator_column: Identifier::new("type_id")?,
            },
        })
    }
    /// Proves logical/original physical key correspondence only. Selected key
    /// encoding/comparison procedures and uniqueness enforcement are separate.
    pub fn verify_key_mapping(
        &self,
        binding: &Admission,
        catalog: &Catalog,
        key: &weft_core::application_model::AuthoredKey,
    ) -> Result<()> {
        if sha256(binding.original_json.as_bytes()) != self.binding_sha256 {
            return Err(fail("Key binding cut differs from admitted Record"));
        }
        let input = catalog
            .inputs
            .get(self.original_record.document)
            .ok_or_else(|| fail("Key original model input missing"))?;
        if input.pin != self.model_pin
            || sha256(input.document_json.as_bytes()) != self.model_pin.sha256
        {
            return Err(fail("Key model cut differs from admitted Record"));
        }
        let original = catalog.authored_key(&self.original_record, &key.id)?;
        if serde_json::to_value(&original).map_err(|_| fail("Original key encoding refused"))?
            != serde_json::to_value(key).map_err(|_| fail("Resolved key encoding refused"))?
        {
            return Err(fail(
                "Resolved key order or types differ from original Record",
            ));
        }
        let mappings: Vec<_> = binding.value["keys"]
            .as_array()
            .ok_or_else(|| fail("Key mappings missing"))?
            .iter()
            .enumerate()
            .filter(|(_, mapping)| {
                mapping["ownerTypeId"] == self.catalog_id && mapping["keyId"] == key.id
            })
            .collect();
        if mappings.len() != 1 {
            return Err(fail("Original key mapping missing or ambiguous"));
        }
        let (index, mapping) = mappings[0];
        let authored = self.original_record.value["keys"]
            .as_array()
            .and_then(|keys| keys.iter().find(|value| value["id"] == key.id))
            .ok_or_else(|| fail("Original key definition missing"))?;
        if binding.decoded_json(&format!("/keys/{index}/acceptedDefinition"))? != *authored {
            return Err(fail("Mapped key definition differs from original Record"));
        }
        let mut ids = Vec::new();
        for field in &key.fields {
            let properties: Vec<_> = binding.value["properties"]
                .as_array()
                .ok_or_else(|| fail("Key property mappings missing"))?
                .iter()
                .filter(|property| {
                    property["ownerTypeId"] == self.catalog_id
                        && property["logical"] == serde_json::json!(field)
                })
                .collect();
            if properties.len() != 1 {
                return Err(fail("Key property mapping missing or ambiguous"));
            }
            ids.push(properties[0]["propertyId"].clone());
        }
        if mapping["orderedPropertyIds"] != serde_json::json!(ids) {
            return Err(fail(
                "Physical key property order differs from original Record",
            ));
        }
        Ok(())
    }
    pub fn identity(&self) -> &Identity {
        &self.owner
    }
    pub fn verify_property(
        &self,
        property: &crate::property_definition::PropertyAdmission,
    ) -> Result<()> {
        property.verify_binding_basis(&self.binding_sha256)?;
        if property.owner != self.owner
            || property.owner_catalog_id != self.catalog_id
            || property.home.owner_mapping() != self.mapping
        {
            return Err(fail(
                "Property home differs from independently admitted Record source",
            ));
        }
        Ok(())
    }
}
/// Prepare every scan, including fieldless counts. Property/value/comparator
/// and host obligations remain separately admitted. Parameters commit atomically.
pub fn lower(
    context: &Context<'_>,
    records: &BTreeMap<String, RecordAdmission>,
    parameters: &mut Parameters,
) -> Result<BTreeMap<String, OwnerSource>> {
    if context.binding_value["bindingProfileId"] != context.binding.profile
        || sha256(context.binding.json.as_bytes()) != context.binding.sha256
        || checked_json(&context.binding.json).map_err(|_| fail("Context binding JSON refused"))?
            != *context.binding_value
    {
        return Err(fail("Record context differs from original binding bytes"));
    }
    let mut scans = BTreeMap::new();
    let mut add = |occurrence: &String, owner: &Identity, pin: &ModelPin| -> Result<()> {
        if scans
            .insert(occurrence.clone(), (owner.clone(), pin.clone()))
            .is_some()
        {
            return Err(fail("Duplicate original scan occurrence"));
        }
        Ok(())
    };
    match context.plan {
        Plan::V01(plan) => {
            let mut nodes = vec![&plan.root];
            while let Some(node) = nodes.pop() {
                match node {
                    Node::Scan {
                        occurrence,
                        record,
                        pin,
                    } => add(occurrence, record, pin)?,
                    Node::InnerJoin { left, right, .. } => {
                        nodes.extend([left.as_ref(), right.as_ref()])
                    }
                    Node::Filter { input, .. }
                    | Node::Aggregate { input, .. }
                    | Node::Project { input, .. } => nodes.push(input),
                }
            }
        }
        Plan::V02(plan) => {
            add(
                &plan.source.occurrence,
                &plan.source.record,
                &plan.source.pin,
            )?;
            for join in &plan.joins {
                add(&join.right.occurrence, &join.right.record, &join.right.pin)?;
            }
        }
    }
    let namespace = Identifier::new(
        context.binding_value["basis"]["namespace"]
            .as_str()
            .ok_or_else(|| fail("Missing original namespace"))?,
    )?;
    let mut staged = parameters.clone();
    let mut sources = BTreeMap::new();
    for (index, (occurrence, (owner, pin))) in scans.into_iter().enumerate() {
        let key =
            serde_json::to_string(&owner).map_err(|_| fail("Record identity encoding refused"))?;
        let record = records
            .get(&key)
            .ok_or_else(|| fail("Original scan lacks independent Record admission"))?;
        if record.owner != owner
            || record.model_pin != pin
            || record.binding_sha256 != context.binding.sha256
            || !context.catalog.inputs.iter().any(|input| {
                input.pin == pin
                    && sha256(input.document_json.as_bytes()) == pin.sha256
                    && input.selected_module_ids.contains(&owner.module)
            })
        {
            return Err(fail(
                "Record source substitutes original owner, model or binding cut",
            ));
        }
        sources.insert(
            occurrence,
            record.mapping.source(
                &namespace,
                &Identifier::new(&format!("weft_scan_{index}"))?,
                &record.catalog_id,
                &mut staged,
            )?,
        );
    }
    *parameters = staged;
    Ok(sources)
}
#[cfg(test)]
mod tests {
    use super::*;
    use weft_core::{
        application_ir as app, application_resolve, application_syntax,
        backend::{Backend, BindingInput, Selection as QuerySelection},
    };
    #[test]
    fn original_record_key_mapping_retains_authored_identity_order_and_types() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/application-cases.json"
        ))
        .unwrap();
        let catalog = Catalog::prepare(
            serde_json::from_value(cases[0]["request"]["modules"].clone()).unwrap(),
        )
        .unwrap();
        let raw = cases[0]["request"]["target"]["bindingJson"]
            .as_str()
            .unwrap();
        let value = checked_json(raw).unwrap();
        let admitted = Admission::parse(raw, value["bindingProfileId"].as_str().unwrap()).unwrap();
        let home = admitted.original_home_definition(0).unwrap();
        let relation = home["relationPhysicalIdentity"].as_str().unwrap();
        let column = home["discriminatorColumnPhysicalIdentity"]
            .as_str()
            .unwrap();
        let relations = BTreeMap::from([(relation.into(), "object".into())]);
        let columns = BTreeMap::from([(
            column.into(),
            Column {
                relation_identity: relation.into(),
                name: "type_id".into(),
            },
        )]);
        let inventory = OriginalArtifact {
            identity: value["basis"]["layoutInventory"]["identity"]
                .as_str()
                .unwrap()
                .into(),
            bytes: admitted.artifacts["/basis/layoutInventory"].clone(),
        };
        let mut checked = 0;
        for index in 0..admitted.value["entities"].as_array().unwrap().len() {
            let record = RecordAdmission::admit(
                &admitted,
                index,
                &catalog,
                Selection {
                    inventory: &inventory,
                    relation_identity: relation,
                    discriminator_identity: column,
                    relations: &relations,
                    columns: &columns,
                },
            )
            .unwrap();
            if let Some(keys) = record.original_record.value["keys"].as_array() {
                for original in keys {
                    let key = catalog
                        .authored_key(&record.original_record, original["id"].as_str().unwrap())
                        .unwrap();
                    record
                        .verify_key_mapping(&admitted, &catalog, &key)
                        .unwrap();
                    let mut wrong = key.clone();
                    wrong.fields[0].element = "foreign-field".into();
                    assert!(record
                        .verify_key_mapping(&admitted, &catalog, &wrong)
                        .is_err());
                    let mut wrong = key;
                    wrong.types[0].nullable = true;
                    assert!(record
                        .verify_key_mapping(&admitted, &catalog, &wrong)
                        .is_err());
                    checked += 1;
                }
            }
        }
        assert!(checked > 0);
    }
    #[test]
    fn fieldless_count_uses_original_record_and_refuses_late_missing_owner_atomically() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/compiler-cases.json"
        ))
        .unwrap();
        let catalog = Catalog::prepare(
            serde_json::from_value(cases[0]["request"]["modules"].clone()).unwrap(),
        )
        .unwrap();
        let raw = cases[0]["request"]["target"]["bindingJson"]
            .as_str()
            .unwrap();
        let value = checked_json(raw).unwrap();
        let admitted = Admission::parse(raw, value["bindingProfileId"].as_str().unwrap()).unwrap();
        let home = admitted.original_home_definition(0).unwrap();
        let relation = home["relationPhysicalIdentity"].as_str().unwrap();
        let column = home["discriminatorColumnPhysicalIdentity"]
            .as_str()
            .unwrap();
        let relations = BTreeMap::from([(relation.into(), "object".into())]);
        let columns = BTreeMap::from([(
            column.into(),
            Column {
                relation_identity: relation.into(),
                name: "type_id".into(),
            },
        )]);
        let inventory = OriginalArtifact {
            identity: value["basis"]["layoutInventory"]["identity"]
                .as_str()
                .unwrap()
                .into(),
            bytes: admitted.artifacts["/basis/layoutInventory"].clone(),
        };
        let select = || Selection {
            inventory: &inventory,
            relation_identity: relation,
            discriminator_identity: column,
            relations: &relations,
            columns: &columns,
        };
        let records: BTreeMap<_, _> = (0..value["entities"].as_array().unwrap().len())
            .map(|index| {
                let record = RecordAdmission::admit(&admitted, index, &catalog, select()).unwrap();
                (serde_json::to_string(record.identity()).unwrap(), record)
            })
            .collect();
        let resolve = |sql| {
            application_resolve::resolve(
                &catalog,
                application_syntax::parse(sql).unwrap(),
                BTreeMap::new(),
                Some(app::ReadProfile {
                    version: "weft-application-read/0.2.0".into(),
                    subset: app::Subset::CountSummary,
                }),
            )
            .unwrap()
        };
        let plan = resolve("SELECT COUNT(*) AS count FROM Customer c");
        assert!(
            crate::comparator_requirements::collect_reads(Plan::V02(&plan))
                .unwrap()
                .is_empty()
        );
        let manifest = crate::candidate::Candidate.describe().unwrap();
        let input = BindingInput {
            profile: value["bindingProfileId"].as_str().unwrap().into(),
            json: raw.into(),
            sha256: sha256(raw.as_bytes()),
        };
        let query_selection = QuerySelection {
            records: vec![plan.source.record.clone()],
            ..Default::default()
        };
        let context = Context {
            catalog: &catalog,
            plan: Plan::V02(&plan),
            target: &manifest.target_profiles[0],
            binding: &input,
            binding_value: &value,
            selection: &query_selection,
        };
        let mut parameters = Parameters::default();
        let sources = lower(&context, &records, &mut parameters).unwrap();
        assert_eq!(sources.len(), 1);
        let mut fieldless_parameters = Parameters::default();
        let empty_properties = BTreeMap::new();
        let prepared = crate::registered_access::prepare(
            &context,
            &records,
            &empty_properties,
            &BTreeMap::new(),
            &mut fieldless_parameters,
        )
        .unwrap();
        assert_eq!(prepared.scans.len(), 1);
        assert!(prepared.accesses.is_empty());
        let result_columns = crate::result_definition::projection_columns(
            &context,
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .unwrap();
        assert_eq!(result_columns.len(), 1);
        assert_eq!(
            result_columns[0].source_identities,
            [plan.source.record.clone()]
        );
        assert!(matches!(
            result_columns[0].representation,
            weft_core::backend::Representation::Scalar {
                decoder: weft_core::backend::ScalarDecoder::ExactInteger,
                ..
            }
        ));

        assert!(prepared
            .scans
            .values()
            .next()
            .unwrap()
            .structural_check_sql
            .is_empty());
        assert_eq!(fieldless_parameters.into_slots().len(), 1);

        let source = &sources[&plan.source.occurrence];
        assert!(source.sql.ends_with(".\"object\" AS \"weft_scan_0\""));
        assert!(!source.sql.contains("props"));
        assert_eq!(parameters.clone().into_slots().len(), 1);
        if let Ok(path) = std::env::var("WEFT_RECORD_COUNT_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({"sql":format!("SELECT count(*) AS count FROM {} WHERE {}",source.sql,source.discriminator),"parameters":parameters.clone().into_slots(),"namespace":value["basis"]["namespace"]})).unwrap()).unwrap();
        }
        let joined = resolve(
            "SELECT COUNT(*) AS count FROM Customer c JOIN Orders o ON c.id = o.customer_id",
        );
        let joined_context = Context {
            plan: Plan::V02(&joined),
            ..context
        };
        let customer_only: BTreeMap<_, _> = records
            .into_iter()
            .filter(|(_, record)| record.owner == plan.source.record)
            .collect();
        let mut atomic = Parameters::default();
        assert!(lower(&joined_context, &customer_only, &mut atomic).is_err());
        assert!(atomic.into_slots().is_empty());
        let mut changed_value = value.clone();
        changed_value["basis"]["namespace"] = serde_json::json!("later_schema");
        let changed_raw = changed_value.to_string();
        let changed_input = BindingInput {
            profile: input.profile.clone(),
            sha256: sha256(changed_raw.as_bytes()),
            json: changed_raw,
        };
        let changed_context = Context {
            binding: &changed_input,
            binding_value: &changed_value,
            ..context
        };
        assert!(lower(&changed_context, &customer_only, &mut parameters).is_err());
        assert_eq!(parameters.clone().into_slots().len(), 1);
        let mut wrong_plan = plan.clone();
        wrong_plan.source.pin.sha256 = sha256(b"different-model");
        let wrong_context = Context {
            plan: Plan::V02(&wrong_plan),
            ..context
        };
        assert!(lower(&wrong_context, &customer_only, &mut parameters).is_err());
        assert_eq!(parameters.into_slots().len(), 1);
        let mut redirected = columns.clone();
        redirected.get_mut(column).unwrap().name = "rel_type_id".into();
        assert!(RecordAdmission::admit(
            &admitted,
            0,
            &catalog,
            Selection {
                inventory: &inventory,
                relation_identity: relation,
                discriminator_identity: column,
                relations: &relations,
                columns: &redirected
            }
        )
        .is_err());
        let mut wrong_source = value.clone();
        wrong_source["entities"][0]["source"] = value["entities"][0]["acceptedDefinition"].clone();
        let wrong_admission = Admission::parse(&wrong_source.to_string(), &input.profile).unwrap();
        assert!(RecordAdmission::admit(
            &wrong_admission,
            0,
            &catalog,
            Selection {
                inventory: &inventory,
                relation_identity: relation,
                discriminator_identity: column,
                relations: &relations,
                columns: &columns
            }
        )
        .is_err());
    }
}
