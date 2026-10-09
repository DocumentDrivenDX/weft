//! Trusted, explicitly registered backend declarations. No content-driven loading.
use crate::{
    error::{Diagnostic, Result},
    json::checked_json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LanguageProfile {
    pub dialect_profile: String,
    pub ir_version: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TargetProfile {
    pub id: String,
    pub engine: String,
    pub engine_version: String,
    pub session_settings: Value,
    pub storage_layout_revision: String,
    pub publication_revision: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Supported,
    Candidate,
    Unsupported,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObligationOwner {
    Host,
    Backend,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Obligation {
    pub id: String,
    pub parameters: Value,
    pub owner: ObligationOwner,
    pub failure_code: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Capability {
    pub id: String,
    pub target_profiles: Vec<String>,
    pub language_profiles: Vec<LanguageProfile>,
    pub logical_domain: Value,
    pub result_domain: Value,
    pub constraints: Vec<String>,
    pub obligations: Vec<Obligation>,
    pub status: Status,
    pub evidence: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub backend_id: String,
    pub backend_version: String,
    pub interface_version: String,
    pub language_profiles: Vec<LanguageProfile>,
    pub binding_profile: String,
    pub target_profiles: Vec<TargetProfile>,
    pub capabilities: Vec<Capability>,
    pub evidence: Vec<String>,
}
fn failure(code: &str, message: &str) -> Diagnostic {
    Diagnostic::new(code, "capability", message)
}
fn nonempty(s: &str) -> bool {
    !s.is_empty() && !s.contains('\0')
}
fn unique_strings(v: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    v.iter().all(|s| nonempty(s) && seen.insert(s))
}
fn profile_key(p: &LanguageProfile) -> (&str, &str) {
    (&p.dialect_profile, &p.ir_version)
}
fn valid_languages(profiles: &[LanguageProfile]) -> bool {
    let mut seen = BTreeSet::new();
    !profiles.is_empty()
        && profiles.iter().all(|p| {
            matches!(
                profile_key(p),
                ("weft-sql/0.1.0", "weft-ir/0.1.0") | ("weft-sql/0.2.0", "weft-ir/0.2.0") | ("weft-sql/0.3.0", "weft-ir/0.3.0")
            ) && seen.insert(profile_key(p))
        })
}
pub fn validate_manifest_json(raw: &str) -> Result<Manifest> {
    if raw.len() > 1024 * 1024 {
        return Err(failure("WFT-LIMIT", "Backend manifest exceeds one MiB"));
    }
    let value = checked_json(raw).map_err(|_| {
        failure(
            "WFT-BACKEND-VERSION",
            "Malformed or duplicate-key backend manifest JSON",
        )
    })?;
    let manifest: Manifest = serde_json::from_value(value).map_err(|_| {
        failure(
            "WFT-BACKEND-VERSION",
            "Backend manifest has unknown or malformed members",
        )
    })?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}
pub fn validate_manifest(m: &Manifest) -> Result<()> {
    if m.interface_version != "weft-backend/0.2.0" || !valid_languages(&m.language_profiles) {
        return Err(failure(
            "WFT-BACKEND-VERSION",
            "Unsupported backend interface or language/IR pair",
        ));
    }
    if ![&m.backend_id, &m.backend_version, &m.binding_profile]
        .iter()
        .all(|s| nonempty(s))
        || m.target_profiles.is_empty()
        || m.target_profiles.len() > 256
        || m.capabilities.is_empty()
        || m.capabilities.len() > 4096
        || !unique_strings(&m.evidence)
    {
        return Err(failure(
            "WFT-BACKEND-VERSION",
            "Backend manifest identity, collection bounds or evidence IDs are invalid",
        ));
    }
    let mut targets = BTreeSet::new();
    for t in &m.target_profiles {
        if ![
            &t.id,
            &t.engine,
            &t.engine_version,
            &t.storage_layout_revision,
            &t.publication_revision,
        ]
        .iter()
        .all(|s| nonempty(s))
            || !t.session_settings.is_object()
            || !targets.insert(t.id.as_str())
        {
            return Err(failure(
                "WFT-BACKEND-VERSION",
                "Target profiles must be distinct, pinned and structurally explicit",
            ));
        }
    }
    let mut caps = BTreeSet::new();
    for c in &m.capabilities {
        if !nonempty(&c.id)
            || !caps.insert(c.id.as_str())
            || c.target_profiles.is_empty()
            || !unique_strings(&c.target_profiles)
            || c.target_profiles
                .iter()
                .any(|p| !targets.contains(p.as_str()))
            || !valid_languages(&c.language_profiles)
            || c.language_profiles
                .iter()
                .any(|p| !m.language_profiles.contains(p))
        {
            return Err(failure(
                "WFT-BACKEND-VERSION",
                "Capabilities must bind distinct IDs to declared target and language profiles",
            ));
        }
        if !c.logical_domain.is_object()
            || c.logical_domain.as_object().unwrap().is_empty()
            || !c.result_domain.is_object()
            || c.result_domain.as_object().unwrap().is_empty()
            || !unique_strings(&c.constraints)
            || !unique_strings(&c.evidence)
            || c.evidence.iter().any(|e| !m.evidence.contains(e))
            || (c.status == Status::Supported && c.evidence.is_empty())
        {
            return Err(failure(
                "WFT-CAPABILITY",
                "Capability domains, constraints and qualified evidence must be explicit",
            ));
        }
        let mut obligations = BTreeSet::new();
        for o in &c.obligations {
            if !nonempty(&o.id)
                || !o.parameters.is_object()
                || (!o.failure_code.starts_with("WFT-") || o.failure_code.len() <= 4)
                || !o.failure_code[4..]
                    .bytes()
                    .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-')
                || !obligations.insert(o.id.as_str())
            {
                return Err(failure(
                    "WFT-OBLIGATION",
                    "Capability obligations are malformed or ambiguous",
                ));
            }
        }
    }
    Ok(())
}

/// Backend-owned mapping and target types remain statically checked inside this trait.
/// Only an explicitly registered host object implements it; manifests never load code.
pub trait Backend: Send + Sync + 'static {
    type Mapping: Send + Sync;
    type TargetPlan: Send + Sync;
    fn describe(&self) -> Result<Manifest>;
    fn validate_binding(&self, context: &Context<'_>) -> Result<Validated<Self::Mapping>>;
    fn assess(&self, context: &Context<'_>, mapping: &Self::Mapping) -> Result<Vec<Assessment>>;
    fn lower(&self, context: &Context<'_>, mapping: &Self::Mapping) -> Result<Self::TargetPlan>;
    fn emit(&self, context: &Context<'_>, plan: &Self::TargetPlan) -> Result<Emission>;
}
#[derive(Debug, Clone, Copy)]
pub enum Plan<'a> {
    V01(&'a crate::ir::LogicalPlan),
    V02(&'a crate::application_ir::Plan),
    V03(&'a crate::arithmetic_plan::Plan),
}
impl Plan<'_> {
    pub fn language(&self) -> LanguageProfile {
        let suffix = match self {
            Self::V01(_) => "0.1.0",
            Self::V02(_) => "0.2.0",
            Self::V03(_) => "0.3.0",
        };
        LanguageProfile {
            dialect_profile: format!("weft-sql/{suffix}"),
            ir_version: format!("weft-ir/{suffix}"),
        }
    }
    pub fn capabilities(&self) -> &[String] {
        match self {
            Self::V01(p) => &p.required_capabilities,
            Self::V02(p) => &p.required_capabilities,
            Self::V03(p) => &p.required_capabilities,
        }
    }
    pub fn pins(&self) -> &[crate::ir::ModelPin] {
        match self {
            Self::V01(p) => &p.module_pins,
            Self::V02(p) => &p.module_pins,
            Self::V03(p) => &p.module_pins,
        }
    }
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
    pub records: Vec<crate::ir::Identity>,
    pub fields: Vec<crate::ir::Identity>,
    pub types: Vec<crate::ir::Identity>,
    pub relationships: Vec<crate::application_model::RelationshipIdentity>,
}
pub struct Context<'a> {
    pub catalog: &'a crate::model::Catalog,
    pub plan: Plan<'a>,
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
pub struct ParameterSlot {
    pub position: usize,
    pub logical_type: crate::ir::LogicalType,
    pub value: String,
    pub origin: Value,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScalarCarrier {
    Text,
    Boolean,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScalarDecoder {
    Text,
    Boolean,
    ExactInteger,
    ExactDecimal,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Representation {
    Scalar {
        #[serde(rename = "logicalType")]
        logical_type: crate::ir::LogicalType,
        carrier: ScalarCarrier,
        decoder: ScalarDecoder,
    },
    Value {
        descriptor: crate::ir::Identity,
        #[serde(rename = "nativeNull")]
        native_null: bool,
    },
    RelatedKeys {
        relationship: crate::application_model::RelationshipIdentity,
        key: crate::application_model::AuthoredKey,
        bound: u16,
    },
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    pub position: usize,
    pub output_name: String,
    /// Physical result label; only explicitly admitted 0.3 positional outputs may carry it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carrier_name: Option<String>,
    pub representation: Representation,
    pub source_identities: Vec<crate::ir::Identity>,
    pub nullable: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct Emission {
    pub sql: String,
    pub parameters: Vec<ParameterSlot>,
    pub columns: Vec<Column>,
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
pub struct Compilation {
    pub backend_id: String,
    pub backend_version: String,
    pub target_profile: TargetProfile,
    pub binding_profile: String,
    pub binding_sha256: String,
    pub qualifications: Vec<Qualification>,
    pub emission: Emission,
}
trait Registered: Send + Sync {
    fn manifest(&self) -> &Manifest;
    fn compile(
        &self,
        catalog: &crate::model::Catalog,
        plan: Plan<'_>,
        target: &Target,
        binding: &BindingInput,
    ) -> Result<Compilation>;
}
struct Adapter<B: Backend> {
    backend: B,
    manifest: Manifest,
}
impl<B: Backend> Registered for Adapter<B> {
    fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    fn compile(
        &self,
        catalog: &crate::model::Catalog,
        plan: Plan<'_>,
        target: &Target,
        binding: &BindingInput,
    ) -> Result<Compilation> {
        if target.backend_version != self.manifest.backend_version {
            return Err(failure(
                "WFT-BACKEND-VERSION",
                "Selected backend version is not registered",
            ));
        }
        let language = plan.language();
        let actual_ir = match plan {
            Plan::V01(p) => &p.ir_version,
            Plan::V02(p) => &p.ir_version,
            Plan::V03(p) => &p.ir_version,
        };
        if actual_ir != &language.ir_version || !unique_strings(plan.capabilities()) {
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
        if plan.pins() != catalog.pins().as_slice() {
            return Err(failure(
                "WFT-PIN",
                "Plan model pins do not match the supplied catalog",
            ));
        }
        if binding.json.len() > 4 * 1024 * 1024 {
            return Err(failure("WFT-LIMIT", "Binding exceeds four MiB"));
        }
        if crate::json::sha256(binding.json.as_bytes()) != binding.sha256 {
            return Err(failure("WFT-PIN", "Binding byte digest mismatch"));
        }
        let value = checked_json(&binding.json).map_err(|_| {
            failure(
                "WFT-BINDING",
                "Binding JSON is malformed or repeats members",
            )
        })?;
        if !value.is_object() {
            return Err(failure("WFT-BINDING", "Binding root must be an object"));
        }
        let selection = selected(plan);
        // Unbounded source integers are a new explicit opt-in. Refuse before
        // unchanged backend binding code can assume an authored width.
        if plan.capabilities().iter().any(|c| c == "type.integer.unbounded")
            && !self.manifest.capabilities.iter().any(|c| c.id == "type.integer.unbounded"
                && (c.status == Status::Supported || (c.status == Status::Candidate && target.allow_candidate))
                && c.target_profiles.contains(&target.profile_id) && c.language_profiles.contains(&language)) {
            return Err(failure("WFT-CAPABILITY", "Selected backend profile does not admit mathematical integer source domains"));
        }
        // New 0.3 comparisons must be admitted before any backend binding callback.
        // Retain the ordering of older operations; do not retroactively preflight them.
        if matches!(plan, Plan::V03(_)) {
            for id in plan.capabilities().iter().filter(|id| ["compare.less", "compare.lessEqual", "compare.greaterEqual", "compare.notEqual", "compare.scalarJoin", "project.positionedOutputs", "project.distinct", "predicate.nativeNull", "compare.nullAwareStringEqual", "value.nativeNull"].contains(&id.as_str())) {
                if !self.manifest.capabilities.iter().any(|c| &c.id == id
                    && (c.status == Status::Supported || (c.status == Status::Candidate && target.allow_candidate))
                    && c.target_profiles.contains(&target.profile_id) && c.language_profiles.contains(&language)) {
                    return Err(failure("WFT-CAPABILITY", "Selected backend profile does not admit the new comparison operator"));
                }
            }
        }
        let context = Context {
            catalog,
            plan,
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
        let required: BTreeSet<String> = plan
            .capabilities()
            .iter()
            .chain(&validated.additional_capabilities)
            .cloned()
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
        crate::backend_emission::validate(plan, &emission, &selection, &assessments)?;
        let emitted_obligations = std::mem::take(&mut emission.obligations);
        merge_obligations(&mut emission.obligations, &emitted_obligations)?;
        merge_obligations(&mut emission.obligations, &obligations)?;
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
fn merge_obligations(into: &mut Vec<Obligation>, from: &[Obligation]) -> Result<()> {
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
fn coverage_contains(coverage: &Selection, selected: &Selection) -> bool {
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
#[derive(Default)]
pub struct Registry {
    backends: std::collections::BTreeMap<String, Box<dyn Registered>>,
}
impl Registry {
    pub fn register<B: Backend>(&mut self, backend: B) -> Result<()> {
        let manifest =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| backend.describe()))
                .map_err(|_| {
                    failure(
                        "WFT-BACKEND-FAILURE",
                        "Registered backend panicked while describing itself",
                    )
                })??;
        validate_manifest(&manifest)?;
        if self.backends.contains_key(&manifest.backend_id) {
            return Err(failure(
                "WFT-BACKEND-VERSION",
                "Backend identity is already registered",
            ));
        }
        self.backends.insert(
            manifest.backend_id.clone(),
            Box::new(Adapter { backend, manifest }),
        );
        Ok(())
    }
    pub fn manifest(&self, id: &str) -> Option<&Manifest> {
        self.backends.get(id).map(|b| b.manifest())
    }
    pub fn compile(
        &self,
        catalog: &crate::model::Catalog,
        plan: Plan<'_>,
        target: &Target,
        binding: &BindingInput,
    ) -> Result<Compilation> {
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
fn selected(plan: Plan<'_>) -> Selection {
    use crate::{
        application_ir::{Expression as AE, Predicate as AP, Value as AV},
        ir::{Expression as E, Node},
    };
    fn expression(e: &E, s: &mut Selection) {
        match e {
            E::Field { identity, .. } => s.fields.push(identity.clone()),
            E::Literal { .. } => {}
            E::Equal { left, right, .. } | E::And { left, right, .. } => {
                expression(left, s);
                expression(right, s);
            }
            E::Sum { argument, .. } => expression(argument, s),
        }
    }
    fn node(n: &Node, s: &mut Selection) {
        match n {
            Node::Scan { record, .. } => s.records.push(record.clone()),
            Node::InnerJoin { left, right, on } => {
                node(left, s);
                node(right, s);
                expression(on, s);
            }
            Node::Filter { input, predicate } => {
                node(input, s);
                expression(predicate, s);
            }
            Node::Aggregate {
                input,
                groups,
                aggregates,
            } => {
                node(input, s);
                for e in groups.iter().chain(aggregates) {
                    expression(e, s);
                }
            }
            Node::Project { input, outputs } => {
                node(input, s);
                for o in outputs {
                    expression(&o.expression, s);
                }
            }
        }
    }
    fn rel(r: &crate::application_model::RelationshipRead, s: &mut Selection) {
        s.relationships.push(r.identity.clone());
        s.records.extend([r.from.clone(), r.to.clone()]);
        s.fields.extend(r.source_key.fields.clone());
        s.fields.extend(r.target_key.fields.clone());
    }
    fn value(v: &AV, s: &mut Selection) {
        if let AV::Field { field } = v {
            s.fields.push(field.identity.clone());
        }
    }
    fn predicate(p: &AP, s: &mut Selection) {
        match p {
            AP::Equal { left, right } => {
                s.fields.push(left.identity.clone());
                value(right, s);
            }
            AP::LexicographicGreater { columns, values } => {
                s.fields.extend(columns.iter().map(|f| f.identity.clone()));
                for v in values {
                    value(v, s);
                }
            }
            AP::HasRelated {
                relationship, key, ..
            } => {
                rel(relationship, s);
                for v in key {
                    value(v, s);
                }
            }
        }
    }
    let mut s = Selection::default();
    match plan {
        Plan::V01(p) => node(&p.root, &mut s),
        Plan::V03(p) => {
            fn arithmetic(e:&crate::arithmetic_resolve::Expression,s:&mut Selection) {
                use crate::arithmetic_resolve::Kind;
                match &e.kind {
                    Kind::Field{field}=>s.fields.push(field.identity.clone()),
                    Kind::Negate{operand}=>arithmetic(operand,s),
                    Kind::Binary{left,right,..}=>{arithmetic(left,s);arithmetic(right,s)},
                    Kind::Literal{..}|Kind::Parameter{..}=>{}
                }
            }
            fn pred(p:&crate::arithmetic_plan::Predicate,s:&mut Selection) {
                match p {
                    crate::arithmetic_plan::Predicate::Legacy{predicate:p}=>predicate(p,s),
                    crate::arithmetic_plan::Predicate::ArithmeticCompare{left,right,..}|crate::arithmetic_plan::Predicate::ArithmeticCompareExtended{left,right,..}=>{arithmetic(left,s);arithmetic(right,s)},
                    crate::arithmetic_plan::Predicate::NullTest{field,..}=>s.fields.push(field.identity.clone()),
                    crate::arithmetic_plan::Predicate::NullableStringEqual{left,right}=>{s.fields.push(left.identity.clone());s.fields.push(right.identity.clone())},
                    crate::arithmetic_plan::Predicate::ScalarCompare{left,right,..}=>{s.fields.push(left.identity.clone());value(right,s)}
                }
            }
            s.records.push(p.source.record.clone());
            for j in &p.joins {s.records.push(j.right.record.clone());for p in &j.on {pred(p,&mut s)}}
            for p in &p.filters {pred(p,&mut s)}
            s.fields.extend(p.groups.iter().chain(&p.order).map(|f|f.identity.clone()));
            if let Some(key) = &p.page_key {
                s.fields.extend(key.fields.clone());
            }
            for output in &p.outputs {
                use crate::arithmetic_plan::Expression;
                match &output.expression {
                    Expression::Field{identity,..}=>s.fields.push(identity.clone()),
                    Expression::Sum{argument,..}=>s.fields.push(argument.identity.clone()),
                    Expression::RelatedKeys{relationship,..}=>rel(relationship,&mut s),
                    Expression::Arithmetic{expression}=>arithmetic(expression,&mut s),
                    Expression::Count{..}=>{}
                }
            }
            s.types.extend(p.type_graph.iter().map(|d|d.identity.clone()));
        }
        Plan::V02(p) => {
            s.records.push(p.source.record.clone());
            for j in &p.joins {
                s.records.push(j.right.record.clone());
                for pred in &j.on {
                    predicate(pred, &mut s);
                }
            }
            for pred in &p.filters {
                predicate(pred, &mut s);
            }
            s.fields
                .extend(p.groups.iter().chain(&p.order).map(|f| f.identity.clone()));
            if let Some(key) = &p.page_key {
                s.fields.extend(key.fields.clone());
            }
            for output in &p.outputs {
                match &output.expression {
                    AE::Field { identity, .. } => s.fields.push(identity.clone()),
                    AE::Sum { argument, .. } => s.fields.push(argument.identity.clone()),
                    AE::RelatedKeys { relationship, .. } => rel(relationship, &mut s),
                    AE::Count { .. } => {}
                }
            }
            s.types
                .extend(p.type_graph.iter().map(|d| d.identity.clone()));
        }
    }
    fn dedup(v: &mut Vec<crate::ir::Identity>) {
        v.sort_by(|a, b| {
            (&a.document_id, &a.revision, &a.module, &a.element).cmp(&(
                &b.document_id,
                &b.revision,
                &b.module,
                &b.element,
            ))
        });
        v.dedup();
    }
    dedup(&mut s.records);
    dedup(&mut s.fields);
    dedup(&mut s.types);
    s.relationships.sort_by(|a, b| {
        (&a.document_id, &a.revision, &a.module, &a.relationship).cmp(&(
            &b.document_id,
            &b.revision,
            &b.module,
            &b.relationship,
        ))
    });
    s.relationships.dedup();
    s
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn arithmetic_selection_retains_original_operands_and_versions() {
        let cases: Value = serde_json::from_str(include_str!("../../../tests/application/fixtures/cases.json")).unwrap();
        let req = &cases.as_array().unwrap().iter().find(|c| c["id"] == "join-count").unwrap()["request"];
        let catalog = crate::model::Catalog::prepare(serde_json::from_value(req["modules"].clone()).unwrap()).unwrap();
        let plan = crate::arithmetic_application_resolve::resolve(&catalog, crate::arithmetic_query::parse("SELECT c.id+1 AS next,o.total*12.5000 AS scaled FROM Customer c JOIN Orders o ON o.customer_id=c.id WHERE o.total*2>1").unwrap(), Default::default(), None).unwrap();
        let selected = selected(Plan::V03(&plan));
        assert_eq!(Plan::V03(&plan).language().ir_version, "weft-ir/0.3.0");
        assert_eq!(selected.records.len(), 2);
        assert_eq!(selected.fields.len(), 3);
        let mut with_key = plan.clone();
        let mut key_field = with_key.source.record.clone();
        key_field.element = "page-key-not-in-query".into();
        with_key.page_key = Some(crate::application_model::AuthoredKey {
            id:"original-page-key".into(), fields:vec![key_field.clone()],
            types:vec![crate::ir::LogicalType { family:crate::ir::Family::Integer, facets:json!({}), nullable:false }],
        });
        let keyed_selection = self::selected(Plan::V03(&with_key));
        assert_eq!(keyed_selection.fields.len(), 4);
        assert!(keyed_selection.fields.contains(&key_field));

        for output in &plan.outputs {
            let crate::arithmetic_plan::Expression::Arithmetic { expression } = &output.expression else { panic!("expected arithmetic"); };
            let crate::arithmetic_resolve::Kind::Binary { left, .. } = &expression.kind else { panic!("expected binary"); };
            let crate::arithmetic_resolve::Kind::Field { field } = &left.kind else { panic!("expected field"); };
            assert!(selected.fields.contains(&field.identity));
        }
    }
    #[test]
    fn obligations_are_retained_sorted_and_conflicts_refuse() {
        let obligation = |id: &str, parameters: Value| Obligation {
            id: id.into(),
            parameters,
            owner: ObligationOwner::Host,
            failure_code: "WFT-OBLIGATION".into(),
        };
        let mut merged = vec![];
        let a = obligation("a", json!({"snapshot":"pinned"}));
        let b = obligation("b", json!({"transport":"exact-text"}));
        merge_obligations(&mut merged, &[b.clone(), a.clone(), a.clone()]).unwrap();
        assert_eq!(merged, vec![a.clone(), b]);
        assert!(merge_obligations(
            &mut merged,
            &[obligation("a", json!({"snapshot":"different"}))]
        )
        .is_err());
        let mut malformed = a;
        malformed.failure_code = "WFT-".into();
        assert!(merge_obligations(&mut vec![], &[malformed]).is_err());
    }
}
