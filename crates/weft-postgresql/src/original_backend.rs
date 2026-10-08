//! Trusted registration of original UMF/storage definitions through Backend.
use crate::{registered_access::Access, Parameters};
use std::collections::BTreeMap;
use weft_core::{
    backend::*,
    error::{Diagnostic, Result},
    ir::Expression,
};
pub type Native =
    for<'a> fn(&Expression, &[String], Option<&Access<'a>>, &mut Parameters) -> Result<String>;
pub struct OriginalBackend {
    manifest: Manifest,
    binding_sha256: String,
    records: BTreeMap<String, crate::record_definition::RecordAdmission>,
    properties: BTreeMap<String, crate::property_definition::PropertyAdmission>,
    comparators: BTreeMap<String, crate::native_comparator_definition::Definition>,
    native: Native,
    relationships: BTreeMap<String, crate::relationship_definition::RelationshipAdmission>,
}
pub struct Mapping {
    binding_sha256: String,
}
pub struct TargetPlan {
    emission: Emission,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
impl OriginalBackend {
    /// Definitions are statically registered executable-host inputs, never
    /// loaded from model content. Every compilation rechecks original custody.
    pub fn new(
        binding: &BindingInput,
        records: BTreeMap<String, crate::record_definition::RecordAdmission>,
        properties: BTreeMap<String, crate::property_definition::PropertyAdmission>,
        comparators: BTreeMap<String, crate::native_comparator_definition::Definition>,
        native: Native,
    ) -> Result<Self> {
        crate::binding::Admission::parse(&binding.json, &binding.profile)?;
        if weft_core::json::sha256(binding.json.as_bytes()) != binding.sha256 {
            return Err(fail("Registered original binding digest differs"));
        }
        let mut manifest = crate::candidate::Candidate.describe()?;
        manifest.backend_id = "truss.postgresql.original".into();
        manifest.binding_profile = binding.profile.clone();
        for capability in &mut manifest.capabilities {
            let application = matches!(
                capability.id.as_str(),
                "scan"
                    | "project"
                    | "filter"
                    | "innerJoin"
                    | "equal"
                    | "group"
                    | "type.string"
                    | "type.boolean"
                    | "type.integer"
                    | "type.decimal"
                    | "aggregate"
                    | "aggregate.count"
                    | "sum"
                    | "parameter.named"
                    | "compare.lexicographicGreater"
                    | "key.uniqueStable"
                    | "order.asc"
                    | "limit"
                    | "value.presence"
                    | "value.sequence"
                    | "value.map"
                    | "value.structured"
            );
            if !application {
                capability
                    .language_profiles
                    .retain(|profile| profile.ir_version == "weft-ir/0.1.0");
            }
            capability.logical_domain = serde_json::json!({"subset":"original-definition scalar relational plans; V02 equality joins/filters, direct scalar-root and recursive props sequence/structured projection, COUNT/SUM and string grouping, ordering and bounded LIMIT"});
        }
        Ok(Self {
            manifest,
            binding_sha256: binding.sha256.clone(),
            records,
            properties,
            comparators,
            native,
            relationships: BTreeMap::new(),
        })
    }
    pub fn with_relationships(
        mut self,
        relationships: BTreeMap<String, crate::relationship_definition::RelationshipAdmission>,
    ) -> Result<Self> {
        for (key, admission) in &relationships {
            admission.verify_registration(&self.binding_sha256, key)?;
        }
        if !relationships.is_empty() {
            let candidate = crate::candidate::Candidate.describe()?;
            for capability in &mut self.manifest.capabilities {
                if capability.id.starts_with("relationship.") {
                    capability.language_profiles = candidate
                        .capabilities
                        .iter()
                        .find(|c| c.id == capability.id)
                        .unwrap()
                        .language_profiles
                        .clone();
                    capability.logical_domain = serde_json::json!({"subset":"original admitted directed relationship subqueries; selected scalar authored target keys"});
                }
            }
        }
        self.relationships = relationships;
        Ok(self)
    }
    pub(crate) fn admit_owned_relationships(
        self,
        catalog: &weft_core::model::Catalog,
        binding: &BindingInput,
        selections: &[crate::original_admission::OwnedRelationshipSelection],
    ) -> Result<Self> {
        let admitted = crate::binding::Admission::parse(&binding.json, &binding.profile)?;
        let mut relationships = BTreeMap::new();
        for selected in selections {
            let mapped = admitted.value["relationships"]
                .get(selected.index)
                .ok_or_else(|| fail("Owned relationship mapping missing"))?;
            let role = if selected.inverse {
                "targetTypeId"
            } else {
                "sourceTypeId"
            };
            let entity = admitted.value["entities"]
                .as_array()
                .and_then(|es| es.iter().find(|e| e["typeId"] == mapped[role]))
                .ok_or_else(|| fail("Owned relationship source Record missing"))?;
            let identity = serde_json::from_value(entity["logical"].clone())
                .map_err(|_| fail("Owned relationship source identity invalid"))?;
            let record = catalog.record_by_identity(&identity)?;
            let original = admitted.decoded_json(&format!(
                "/relationships/{}/acceptedDefinition",
                selected.index
            ))?;
            let name = original[if selected.inverse { "inverse" } else { "name" }]
                .as_str()
                .ok_or_else(|| fail("Owned traversal name missing"))?;
            let read = catalog.relationship_read(
                &record,
                &weft_core::syntax::Name {
                    value: name.into(),
                    quoted: true,
                    span: weft_core::ir::Span { start: 0, end: 0 },
                },
            )?;
            let relationship = crate::relationship_definition::RelationshipAdmission::admit(
                &admitted,
                selected.index,
                catalog,
                &read,
                &self.records,
                crate::relationship_definition::Selection {
                    profile: &selected.profile,
                    inventory: &selected.inventory,
                    relation_identity: &selected.relation_identity,
                    relations: &selected.relations,
                    columns: &selected.columns,
                    relationship_type: &selected.relationship_type,
                    source_id: &selected.source_id,
                    source_type: &selected.source_type,
                    target_id: &selected.target_id,
                    target_type: &selected.target_type,
                },
            )?;
            if relationships
                .insert(
                    crate::relationship_definition::registration_key(&read),
                    relationship,
                )
                .is_some()
            {
                return Err(fail("Repeated owned relationship direction"));
            }
        }
        self.with_relationships(relationships)
    }
    fn verify(&self, context: &Context<'_>) -> Result<()> {
        if context.binding.sha256 != self.binding_sha256 {
            return Err(fail("Registered original backend binding cut differs"));
        }
        crate::comparator_requirements::admit_context(context, &self.properties, &self.comparators)
    }
}
impl Backend for OriginalBackend {
    type Mapping = Mapping;
    type TargetPlan = TargetPlan;
    fn describe(&self) -> Result<Manifest> {
        Ok(self.manifest.clone())
    }
    fn validate_binding(&self, context: &Context<'_>) -> Result<Validated<Mapping>> {
        self.verify(context)?;
        let mut parameters = Parameters::default();
        crate::registered_access::prepare(
            context,
            &self.records,
            &self.properties,
            &self.comparators,
            &mut parameters,
        )?;
        Ok(Validated {
            mapping: Mapping {
                binding_sha256: self.binding_sha256.clone(),
            },
            additional_capabilities: vec![],
            coverage: context.selection.clone(),
            obligations: vec![],
        })
    }
    fn assess(&self, context: &Context<'_>, mapping: &Mapping) -> Result<Vec<Assessment>> {
        self.verify(context)?;
        if mapping.binding_sha256 != self.binding_sha256 {
            return Err(fail("Original mapping cut differs"));
        }
        Ok(context
            .plan
            .capabilities()
            .iter()
            .map(|id| Assessment {
                id: id.clone(),
                status: Status::Candidate,
                evidence: vec![],
                obligations: vec![],
            })
            .collect())
    }
    fn lower(&self, context: &Context<'_>, mapping: &Mapping) -> Result<TargetPlan> {
        self.verify(context)?;
        if mapping.binding_sha256 != self.binding_sha256 {
            return Err(fail("Original mapping cut differs"));
        }
        let compilation = crate::select_definition::compile_with_relationships(
            context,
            &self.records,
            &self.properties,
            &self.comparators,
            &self.relationships,
            self.native,
        )?;
        let mut obligations = vec![Obligation {
            id: "truss.original.complete-read-context".into(),
            parameters: serde_json::json!({"bindingSha256":self.binding_sha256,"completeOwnerVisibility":true,"sameTransactionAndAuthorization":true,"beforePublication":true}),
            owner: ObligationOwner::Host,
            failure_code: "WFT-OBLIGATION".into(),
        }];
        for (index, sql) in compilation.select.structural_checks.iter().enumerate() {
            obligations.push(Obligation {id:format!("truss.original.owner-structural-{index}"),parameters:serde_json::json!({"sql":sql,"parameters":compilation.parameters,"expectedViolations":"0","beforeQuery":true}),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into()});
        }
        for (index, check) in compilation.select.payload_checks.iter().enumerate() {
            use base64::{engine::general_purpose::STANDARD, Engine};
            obligations.push(Obligation {id:format!("truss.original.owner-payload-{index}"),parameters:serde_json::json!({"sql":check.sql,"parameters":compilation.parameters,"expectedViolations":"0","beforeQuery":true,"scan":check.scan,"field":check.field,"codecBytesBase64":STANDARD.encode(&check.codec_bytes),"presenceBytesBase64":STANDARD.encode(&check.presence_bytes)}),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into()});
        }
        Ok(TargetPlan {
            emission: Emission {
                sql: compilation.select.sql,
                columns: compilation.select.columns,
                parameters: compilation.parameters,
                obligations,
            },
        })
    }
    fn emit(&self, context: &Context<'_>, plan: &TargetPlan) -> Result<Emission> {
        self.verify(context)?;
        Ok(plan.emission.clone())
    }
}
