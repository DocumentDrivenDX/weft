// SPDX-License-Identifier: MIT OR Apache-2.0
//! The checked backend pipeline: a frontend's typed program, a trusted backend
//! that validates its binding, assesses every required operation, lowers and
//! emits a target artifact, and the registry that runs them in a fixed order.
use crate::manifest::{
    failure, nonempty, unique_strings, validate_manifest, Capability, LanguageProfile, Manifest, ManifestRules, Obligation,
    Status, TargetProfile,
};
use serde::Serialize;
use serde_json::Value;
use std::{collections::BTreeSet, marker::PhantomData};
use weft_model::{
    catalog::Catalog,
    diagnostic::Result,
    json::{checked_json, sha256},
    types::ModelPin,
};

/// A compiler frontend: its typed program and the rules the shared pipeline
/// applies to it. weft's dialect plans are one frontend; Fireweed's is another.
pub trait Frontend: ManifestRules + Send + Sync + 'static {
    /// A borrowed, typed program (logical plan).
    type Program<'a>: Copy;
    /// Shorten a program borrow. Projections of `Program` are invariant in their
    /// lifetime, so implementations write `program` here.
    fn reborrow<'s, 'l: 's>(program: Self::Program<'l>) -> Self::Program<'s>;
    /// The program's dialect/IR pair.
    fn language(program: Self::Program<'_>) -> LanguageProfile;
    /// The IR version the program itself carries; must equal `language().ir_version`.
    fn ir_version(program: Self::Program<'_>) -> String;
    /// Operation identities the program requires.
    fn capabilities(program: Self::Program<'_>) -> Vec<String>;
    /// Model pins the program was resolved against.
    fn pins(program: Self::Program<'_>) -> Vec<ModelPin>;
    /// Every model identity the program reads; a binding must cover all of them.
    fn selection(program: Self::Program<'_>) -> Selection;
    /// Parse binding JSON. Default: bounded, duplicate-refusing [`checked_json`].
    fn read_binding(_target: &Target, _ir_version: &str, json: &str) -> std::result::Result<Value, ()> {
        checked_json(json).map_err(|_| ())
    }
    /// Refusals that must happen before any backend callback. Default: none.
    fn preflight(_manifest: &Manifest, _program: Self::Program<'_>, _target: &Target, _language: &LanguageProfile) -> Result<()> {
        Ok(())
    }
    /// Refuse catalogs this pipeline must not compile. Default: none.
    fn admit_catalog(_catalog: &Catalog) -> Result<()> {
        Ok(())
    }
}
/// What a backend's `emit` returns. The pipeline merges its obligations and lets
/// it check itself against the program.
pub trait TargetArtifact<F: Frontend>: Send {
    fn obligations_mut(&mut self) -> &mut Vec<Obligation>;
    fn check(&self, _program: F::Program<'_>, _selection: &Selection, _assessments: &[Assessment]) -> Result<()> {
        Ok(())
    }
}
/// The artifact of a backend that emits nothing (for example an in-memory
/// reference interpreter): only obligations.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct NoArtifact {
    pub obligations: Vec<Obligation>,
}
impl<F: Frontend> TargetArtifact<F> for NoArtifact {
    fn obligations_mut(&mut self) -> &mut Vec<Obligation> {
        &mut self.obligations
    }
}

/// Backend-owned mapping and target types remain statically checked inside this trait.
/// Only an explicitly registered host object implements it; manifests never load code.
pub trait Backend<F: Frontend>: Send + Sync + 'static {
    type Mapping: Send + Sync;
    type TargetPlan: Send + Sync;
    type Artifact: TargetArtifact<F>;
    fn describe(&self) -> Result<Manifest>;
    fn validate_binding(&self, context: &Context<'_, F>) -> Result<Validated<Self::Mapping>>;
    fn assess(&self, context: &Context<'_, F>, mapping: &Self::Mapping) -> Result<Vec<Assessment>>;
    fn lower(&self, context: &Context<'_, F>, mapping: &Self::Mapping) -> Result<Self::TargetPlan>;
    fn emit(&self, context: &Context<'_, F>, plan: &Self::TargetPlan) -> Result<Self::Artifact>;
}
#[derive(Debug, Clone)]
pub struct BindingInput {
    pub profile: String,
    pub json: String,
    pub sha256: String,
}
#[derive(Debug, Clone)]
pub struct Target {
    pub backend_id: String,
    pub backend_version: String,
    pub profile_id: String,
    pub allow_candidate: bool,
}
/// Every selected identity must receive an explicit binding/codec assessment.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Selection {
    pub records: Vec<weft_model::types::Identity>,
    pub fields: Vec<weft_model::types::Identity>,
    pub types: Vec<weft_model::types::Identity>,
    pub relationships: Vec<weft_model::descriptor::RelationshipIdentity>,
}
pub struct Context<'a, F: Frontend> {
    pub catalog: &'a Catalog,
    pub plan: F::Program<'a>,
    pub target: &'a TargetProfile,
    pub binding: &'a BindingInput,
    pub binding_value: &'a Value,
    pub selection: &'a Selection,
}
pub struct Validated<T> {
    pub mapping: T,
    pub additional_capabilities: Vec<String>,
    pub coverage: Selection,
    pub obligations: Vec<Obligation>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Assessment {
    pub id: String,
    pub status: Status,
    pub evidence: Vec<String>,
    pub obligations: Vec<Obligation>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Qualification {
    pub assessment: Assessment,
    pub declaration: Capability,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Compilation<A> {
    pub backend_id: String,
    pub backend_version: String,
    pub target_profile: TargetProfile,
    pub binding_profile: String,
    pub binding_sha256: String,
    pub qualifications: Vec<Qualification>,
    /// The backend's target artifact (wire name kept from the SQL-only boundary).
    pub emission: A,
}
trait Registered<F: Frontend, A>: Send + Sync {
    fn manifest(&self) -> &Manifest;
    fn compile(
        &self,
        catalog: &Catalog,
        plan: F::Program<'_>,
        target: &Target,
        binding: &BindingInput,
    ) -> Result<Compilation<A>>;
}
struct Adapter<F, B> {
    backend: B,
    manifest: Manifest,
    frontend: PhantomData<fn() -> F>,
}
impl<F: Frontend, B: Backend<F>> Registered<F, B::Artifact> for Adapter<F, B> {
    fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    fn compile(
        &self,
        catalog: &Catalog,
        plan: F::Program<'_>,
        target: &Target,
        binding: &BindingInput,
    ) -> Result<Compilation<B::Artifact>> {
        if target.backend_version != self.manifest.backend_version {
            return Err(failure(
                "WFT-BACKEND-VERSION",
                "Selected backend version is not registered",
            ));
        }
        let language = F::language(plan);
        let actual_ir = F::ir_version(plan);
        if actual_ir != language.ir_version || !unique_strings(&F::capabilities(plan)) {
            return Err(failure(
                "WFT-BACKEND-VERSION",
                "Typed plan version or operation identities are invalid",
            ));
        }
        if !self.manifest.language_profiles.contains(&language) {
            return Err(failure(
                "WFT-BACKEND-VERSION",
                "Registered backend does not accept the selected dialect/IR pair",
            ));
        }
        let profile = self
            .manifest
            .target_profiles
            .iter()
            .find(|p| p.id == target.profile_id)
            .ok_or_else(|| {
                failure(
                    "WFT-BACKEND-VERSION",
                    "Selected target profile is not registered",
                )
            })?;
        if binding.profile != self.manifest.binding_profile {
            return Err(failure(
                "WFT-BINDING",
                "Binding profile does not match the selected registered backend",
            ));
        }
        if F::pins(plan) != catalog.pins() {
            return Err(failure(
                "WFT-PIN",
                "Plan model pins do not match the supplied catalog",
            ));
        }
        if binding.json.len() > 4 * 1024 * 1024 {
            return Err(failure("WFT-LIMIT", "Binding exceeds four MiB"));
        }
        if sha256(binding.json.as_bytes()) != binding.sha256 {
            return Err(failure("WFT-PIN", "Binding byte digest mismatch"));
        }
        let value = F::read_binding(target, &actual_ir, &binding.json).map_err(|_| {
            failure(
                "WFT-BINDING",
                "Binding JSON is malformed or repeats members",
            )
        })?;
        if !value.is_object() {
            return Err(failure("WFT-BINDING", "Binding root must be an object"));
        }
        let selection = F::selection(plan);
        F::preflight(&self.manifest, plan, target, &language)?;
        let context = Context {
            catalog,
            plan: F::reborrow(plan),
            target: profile,
            binding,
            binding_value: &value,
            selection: &selection,
        };
        let validated = self.backend.validate_binding(&context)?;
        if !coverage_contains(&validated.coverage, &selection) {
            return Err(failure(
                "WFT-BINDING",
                "Backend mapping omits a selected record, field, type or relationship identity",
            ));
        }
        if !unique_strings(&validated.additional_capabilities)
            || validated.additional_capabilities.len() > 4096
        {
            return Err(failure(
                "WFT-CAPABILITY",
                "Binding-derived capabilities must be distinct and bounded",
            ));
        }
        let required: BTreeSet<String> = F::capabilities(plan)
            .into_iter()
            .chain(validated.additional_capabilities.iter().cloned())
            .collect();
        let assessments = self.backend.assess(&context, &validated.mapping)?;
        let mut qualifications = Vec::new();
        let mut seen = BTreeSet::new();
        let mut obligations = Vec::new();
        merge_obligations(&mut obligations, &validated.obligations)?;
        for a in &assessments {
            if !seen.insert(a.id.as_str()) || !required.contains(&a.id) {
                return Err(failure(
                    "WFT-CAPABILITY",
                    "Assessment has duplicate or unrequested operations",
                ));
            }
            let declaration = self
                .manifest
                .capabilities
                .iter()
                .find(|c| {
                    c.id == a.id
                        && c.target_profiles.contains(&target.profile_id)
                        && c.language_profiles.contains(&language)
                })
                .ok_or_else(|| {
                    failure(
                        "WFT-CAPABILITY",
                        "Operation is not declared for the selected target/language profile",
                    )
                })?;
            if declaration.status == Status::Unsupported || a.status == Status::Unsupported {
                return Err(failure(
                    "WFT-CAPABILITY",
                    "Selected operation is unsupported",
                ));
            }
            if declaration.status == Status::Candidate && a.status == Status::Supported {
                return Err(failure(
                    "WFT-CAPABILITY",
                    "Candidate declaration cannot be upgraded by assessment",
                ));
            }
            if a.status == Status::Candidate && !target.allow_candidate {
                return Err(failure(
                    "WFT-CAPABILITY",
                    "Candidate operation requires explicit allowCandidate",
                ));
            }
            if !unique_strings(&a.evidence)
                || a.evidence.iter().any(|e| !declaration.evidence.contains(e))
                || (a.status == Status::Supported && a.evidence.is_empty())
            {
                return Err(failure(
                    "WFT-CAPABILITY",
                    "Assessment has missing or undeclared qualification evidence",
                ));
            }
            merge_obligations(&mut obligations, &declaration.obligations)?;
            merge_obligations(&mut obligations, &a.obligations)?;
            qualifications.push(Qualification {
                assessment: a.clone(),
                declaration: declaration.clone(),
            });
        }
        if seen.len() != required.len() {
            return Err(failure(
                "WFT-CAPABILITY",
                "Assessment omits a required operation",
            ));
        }
        let lowered = self.backend.lower(&context, &validated.mapping)?;
        let mut emission = self.backend.emit(&context, &lowered)?;
        emission.check(plan, &selection, &assessments)?;
        let emitted_obligations = std::mem::take(emission.obligations_mut());
        merge_obligations(emission.obligations_mut(), &emitted_obligations)?;
        merge_obligations(emission.obligations_mut(), &obligations)?;
        Ok(Compilation {
            backend_id: self.manifest.backend_id.clone(),
            backend_version: self.manifest.backend_version.clone(),
            target_profile: profile.clone(),
            binding_profile: binding.profile.clone(),
            binding_sha256: binding.sha256.clone(),
            qualifications,
            emission,
        })
    }
}
/// Merge `from` into `into`: validate, de-duplicate by ID, refuse conflicting
/// duplicates, keep ID order.
pub fn merge_obligations(into: &mut Vec<Obligation>, from: &[Obligation]) -> Result<()> {
    for o in from {
        if !nonempty(&o.id)
            || !o.parameters.is_object()
            || !o.failure_code.starts_with("WFT-")
            || o.failure_code.len() <= 4
            || !o.failure_code[4..]
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(failure(
                "WFT-OBLIGATION",
                "Backend returned a malformed obligation",
            ));
        }
        if let Some(existing) = into.iter().find(|e| e.id == o.id) {
            if existing != o {
                return Err(failure(
                    "WFT-OBLIGATION",
                    "Obligation ID has conflicting requirements",
                ));
            }
        } else {
            into.push(o.clone());
        }
    }
    into.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(())
}
/// Whether `coverage` contains every identity in `selected`.
pub fn coverage_contains(coverage: &Selection, selected: &Selection) -> bool {
    selected
        .records
        .iter()
        .all(|i| coverage.records.contains(i))
        && selected.fields.iter().all(|i| coverage.fields.contains(i))
        && selected.types.iter().all(|i| coverage.types.contains(i))
        && selected
            .relationships
            .iter()
            .all(|i| coverage.relationships.contains(i))
}
/// Registered backends by ID, all emitting artifact type `A` for frontend `F`.
pub struct Registry<F: Frontend, A> {
    backends: std::collections::BTreeMap<String, Box<dyn Registered<F, A>>>,
}
impl<F: Frontend, A> Default for Registry<F, A> {
    fn default() -> Self {
        Self { backends: Default::default() }
    }
}
impl<F: Frontend, A: TargetArtifact<F> + 'static> Registry<F, A> {
    pub fn register<B: Backend<F, Artifact = A>>(&mut self, backend: B) -> Result<()> {
        let manifest =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| backend.describe()))
                .map_err(|_| {
                    failure(
                        "WFT-BACKEND-FAILURE",
                        "Registered backend panicked while describing itself",
                    )
                })??;
        validate_manifest::<F>(&manifest)?;
        if self.backends.contains_key(&manifest.backend_id) {
            return Err(failure(
                "WFT-BACKEND-VERSION",
                "Backend identity is already registered",
            ));
        }
        self.backends.insert(
            manifest.backend_id.clone(),
            Box::new(Adapter { backend, manifest, frontend: PhantomData }),
        );
        Ok(())
    }
    pub fn manifest(&self, id: &str) -> Option<&Manifest> {
        self.backends.get(id).map(|b| b.manifest())
    }
    pub fn compile(
        &self,
        catalog: &Catalog,
        plan: F::Program<'_>,
        target: &Target,
        binding: &BindingInput,
    ) -> Result<Compilation<A>> {
        F::admit_catalog(catalog)?;
        let backend = self.backends.get(&target.backend_id).ok_or_else(|| {
            failure(
                "WFT-BACKEND-MISSING",
                "Selected backend identity is not registered",
            )
        })?;
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            backend.compile(catalog, plan, target, binding)
        }))
        .map_err(|_| {
            failure(
                "WFT-BACKEND-FAILURE",
                "Registered backend panicked during compilation",
            )
        })?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use weft_model::{catalog::ModuleInput, types::Identity};

    /// A toy frontend: a program is a list of required operations over one Record.
    struct Toy;
    #[derive(Clone, Copy)]
    struct ToyProgram<'a> {
        capabilities: &'a [String],
        pins: &'a [ModelPin],
        record: &'a Identity,
    }
    impl ManifestRules for Toy {
        fn admits_interface(interface: &str) -> bool {
            interface == "toy-backend/1"
        }
        fn admits_language(_: &str, p: &LanguageProfile) -> bool {
            p.dialect_profile == "toy/1" && p.ir_version == "toy-ir/1"
        }
    }
    impl Frontend for Toy {
        type Program<'a> = ToyProgram<'a>;
        fn reborrow<'s, 'l: 's>(program: ToyProgram<'l>) -> ToyProgram<'s> {
            program
        }
        fn language(_: ToyProgram<'_>) -> LanguageProfile {
            LanguageProfile { dialect_profile: "toy/1".into(), ir_version: "toy-ir/1".into() }
        }
        fn ir_version(_: ToyProgram<'_>) -> String {
            "toy-ir/1".into()
        }
        fn capabilities(program: ToyProgram<'_>) -> Vec<String> {
            program.capabilities.to_vec()
        }
        fn pins(program: ToyProgram<'_>) -> Vec<ModelPin> {
            program.pins.to_vec()
        }
        fn selection(program: ToyProgram<'_>) -> Selection {
            Selection { records: vec![program.record.clone()], ..Selection::default() }
        }
    }
    /// An in-memory backend: emits nothing, carries one host obligation.
    struct Memory {
        cover: bool,
    }
    impl Backend<Toy> for Memory {
        type Mapping = ();
        type TargetPlan = ();
        type Artifact = NoArtifact;
        fn describe(&self) -> Result<Manifest> {
            Ok(serde_json::from_value(json!({"backendId":"toy.memory","backendVersion":"1","interfaceVersion":"toy-backend/1",
                "languageProfiles":[{"dialectProfile":"toy/1","irVersion":"toy-ir/1"}],"bindingProfile":"toy.memory/1",
                "targetProfiles":[{"id":"memory","engine":"memory","engineVersion":"1","sessionSettings":{},"storageLayoutRevision":"1","publicationRevision":"1"}],
                "capabilities":[{"id":"scan","targetProfiles":["memory"],"languageProfiles":[{"dialectProfile":"toy/1","irVersion":"toy-ir/1"}],
                    "logicalDomain":{"rows":true},"resultDomain":{"rows":true},"constraints":[],"obligations":[],"status":"supported","evidence":["unit"]}],
                "evidence":["unit"]})).unwrap())
        }
        fn validate_binding(&self, context: &Context<'_, Toy>) -> Result<Validated<()>> {
            let coverage = if self.cover { context.selection.clone() } else { Selection::default() };
            Ok(Validated { mapping: (), additional_capabilities: vec![], coverage, obligations: vec![] })
        }
        fn assess(&self, _: &Context<'_, Toy>, _: &()) -> Result<Vec<Assessment>> {
            Ok(vec![Assessment { id: "scan".into(), status: Status::Supported, evidence: vec!["unit".into()], obligations: vec![] }])
        }
        fn lower(&self, _: &Context<'_, Toy>, _: &()) -> Result<()> {
            Ok(())
        }
        fn emit(&self, _: &Context<'_, Toy>, _: &()) -> Result<NoArtifact> {
            Ok(NoArtifact { obligations: vec![Obligation { id: "snapshot".into(), parameters: json!({}), owner: crate::manifest::ObligationOwner::Host, failure_code: "WFT-SNAPSHOT".into() }] })
        }
    }
    fn catalog() -> Catalog {
        let text = include_str!("../../../tests/raw-postgresql/table-lifecycle-model.json").to_owned();
        let doc: Value = serde_json::from_str(&text).unwrap();
        let pin = ModelPin { document_id: doc["id"].as_str().unwrap().into(), revision: "1".into(), umf_version: "0.10.0".into(), sha256: sha256(text.as_bytes()) };
        Catalog::prepare(vec![ModuleInput { document_json: text, pin, selected_module_ids: vec!["demo".into()] }]).unwrap()
    }
    fn run(cover: bool, binding_json: &str) -> Result<Compilation<NoArtifact>> {
        let catalog = catalog();
        let mut registry: Registry<Toy, NoArtifact> = Registry::default();
        registry.register(Memory { cover }).unwrap();
        assert_eq!(registry.register(Memory { cover }).unwrap_err().code, "WFT-BACKEND-VERSION");
        let capabilities = vec!["scan".to_string()];
        let pins = catalog.pins();
        let record = Identity { document_id: pins[0].document_id.clone(), revision: "1".into(), module: "demo".into(), element: "Note".into() };
        let program = ToyProgram { capabilities: &capabilities, pins: &pins, record: &record };
        let target = Target { backend_id: "toy.memory".into(), backend_version: "1".into(), profile_id: "memory".into(), allow_candidate: false };
        let binding = BindingInput { profile: "toy.memory/1".into(), json: binding_json.into(), sha256: sha256(binding_json.as_bytes()) };
        registry.compile(&catalog, program, &target, &binding)
    }
    #[test]
    fn a_frontend_and_an_artifactless_backend_run_the_checked_pipeline() {
        let compiled = run(true, "{}").unwrap();
        assert_eq!(compiled.qualifications.len(), 1);
        assert_eq!(compiled.emission.obligations[0].id, "snapshot");
        assert_eq!(serde_json::to_value(&compiled).unwrap()["emission"], json!({"obligations":[{"id":"snapshot","parameters":{},"owner":"host","failureCode":"WFT-SNAPSHOT"}]}));
    }
    #[test]
    fn pipeline_refusals_come_from_the_shared_boundary() {
        assert_eq!(run(false, "{}").unwrap_err().code, "WFT-BINDING");
        assert_eq!(run(true, "[]").unwrap_err().code, "WFT-BINDING");
        assert_eq!(run(true, r#"{"a":1,"a":2}"#).unwrap_err().code, "WFT-BINDING");
    }
    #[test]
    fn manifest_rules_belong_to_the_frontend() {
        let mut manifest = Memory { cover: true }.describe().unwrap();
        crate::manifest::validate_manifest::<Toy>(&manifest).unwrap();
        manifest.interface_version = "weft-backend/0.2.0".into();
        assert_eq!(crate::manifest::validate_manifest::<Toy>(&manifest).unwrap_err().code, "WFT-BACKEND-VERSION");
    }
}
