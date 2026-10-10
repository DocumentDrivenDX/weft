use super::*;
use std::collections::{BTreeMap, BTreeSet};
trait Registered: Send + Sync {
    fn manifest(&self) -> &Manifest;
    fn compile(
        &self,
        catalog: &Catalog,
        plan: Plan04View<'_>,
        target: &Target,
        binding: &BindingInput,
    ) -> Result<Compilation>;
}
struct Adapter<B: Backend> {
    backend: B,
    manifest: Manifest,
}
#[derive(Default)]
pub struct Registry {
    backends: BTreeMap<String, Box<dyn Registered>>,
}
impl Registry {
    pub fn register<B: Backend>(&mut self, backend: B) -> Result<()> {
        let manifest = backend.describe()?;
        manifest::validate(&manifest)?;
        if self.backends.contains_key(&manifest.backend_id) {
            return Err(fail(
                "WFT-BACKEND-VERSION",
                "Backend03 identity already registered",
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
        catalog: &Catalog,
        plan: Plan04View<'_>,
        target: &Target,
        binding: &BindingInput,
    ) -> Result<Compilation> {
        let b = self
            .backends
            .get(&target.backend_id)
            .ok_or_else(|| fail("WFT-BACKEND-MISSING", "Backend03 not registered"))?;
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            b.compile(catalog, plan, target, binding)
        }))
        .map_err(|_| fail("WFT-BACKEND-FAILURE", "Backend03 panicked"))?
    }
}
pub(super) fn merge(into: &mut Vec<Obligation>, from: &[Obligation]) -> Result<()> {
    for o in from {
        if !valid_obligation(o) {
            return Err(fail("WFT-OBLIGATION", "Malformed obligation"));
        }
        if let Some(existing) = into.iter().find(|e| e.id == o.id) {
            if existing != o {
                return Err(fail("WFT-OBLIGATION", "Conflicting obligation"));
            }
        } else {
            into.push(o.clone())
        }
    }
    into.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(())
}
fn coverage(have: &Selection, need: &Selection) -> bool {
    need.records.iter().all(|v| have.records.contains(v))
        && need.fields.iter().all(|v| have.fields.contains(v))
        && need.types.iter().all(|v| have.types.contains(v))
        && need
            .relationships
            .iter()
            .all(|v| have.relationships.contains(v))
}
impl<B: Backend> Registered for Adapter<B> {
    fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    fn compile(
        &self,
        catalog: &Catalog,
        plan: Plan04View<'_>,
        target: &Target,
        binding: &BindingInput,
    ) -> Result<Compilation> {
        if target.backend_version != self.manifest.backend_version {
            return Err(fail("WFT-BACKEND-VERSION", "Backend03 version mismatch"));
        }
        let profile = self
            .manifest
            .target_profiles
            .iter()
            .find(|p| p.id == target.profile_id)
            .ok_or_else(|| fail("WFT-BACKEND-VERSION", "Unknown target profile"))?;
        if plan.pins() != catalog.pins().as_slice() {
            return Err(fail("WFT-PIN", "Original plan pins differ from Catalog"));
        }
        if binding.profile != self.manifest.binding_profile {
            return Err(fail("WFT-BINDING", "Binding profile mismatch"));
        }
        if binding.json.len() > 4 * 1024 * 1024 {
            return Err(fail("WFT-LIMIT", "Binding exceeds four MiB"));
        }
        if crate::json::sha256(binding.json.as_bytes()) != binding.sha256 {
            return Err(fail("WFT-PIN", "Binding digest mismatch"));
        }
        let value = crate::json::checked_json(&binding.json)
            .map_err(|_| fail("WFT-BINDING", "Invalid binding JSON"))?;
        if !value.is_object() {
            return Err(fail("WFT-BINDING", "Binding must be an object"));
        }
        let selection = emission::selection(plan)?;
        let language = LanguageProfile {
            dialect_profile: if plan.ir_version() == "weft-ir/0.4.1" {
                "weft-sql/0.4.1"
            } else {
                "weft-sql/0.4.0"
            }
            .into(),
            ir_version: plan.ir_version().into(),
        };
        let admitted = |id: &str| {
            self.manifest.capabilities.iter().find(|c| {
                c.id == id
                    && c.target_profiles.contains(&target.profile_id)
                    && c.language_profiles.contains(&language)
                    && (c.status == Status::Supported
                        || (c.status == Status::Candidate && target.allow_candidate))
            })
        };
        if !unique(plan.capabilities()) || plan.capabilities().len() > 4096 {
            return Err(fail("WFT-CAPABILITY", "Invalid selected capabilities"));
        }
        for id in plan.capabilities() {
            if admitted(id).is_none() {
                return Err(fail(
                    "WFT-CAPABILITY",
                    "Selected04 capability refused before binding",
                ));
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
        if !coverage(&validated.coverage, &selection) {
            return Err(fail("WFT-BINDING", "Incomplete original identity coverage"));
        }
        if !unique(&validated.additional_capabilities)
            || validated.additional_capabilities.len() > 4096
        {
            return Err(fail("WFT-CAPABILITY", "Invalid binding capabilities"));
        }
        let record_sources = validated.record_sources.clone();
        let left: Vec<_> = plan
            .joins()
            .iter()
            .filter(|j| j.kind == Some(crate::arithmetic_plan::JoinKind::Left))
            .collect();
        if record_sources.len() != left.len() {
            return Err(fail("WFT-BINDING", "Native LEFT source inventory mismatch"));
        }
        for (record, join) in record_sources.iter().zip(left) {
            if record.scan != join.right.occurrence
                || record.record != join.right.record
                || !valid_table(&record.table)
                || !valid_name(&record.identity_column)
                || !valid_name(&record.native_type)
            {
                return Err(fail("WFT-BINDING","Native LEFT declaration changes original scan/Record or physical identity role"));
            }
        }
        let edge_sources = validated.edge_sources.clone();
        if edge_sources.len() != selection.relationships.len() {
            return Err(fail("WFT-BINDING", "Native edge source coverage mismatch"));
        }
        let mut edges = Vec::new();
        for e in &edge_sources {
            if edges.contains(&e.relationship)
                || !selection.relationships.contains(&e.relationship)
                || e.table.version < 0
                || e.table
                    .name
                    .iter()
                    .any(|n| n.is_empty() || n.len() > 128 || n.contains('\0'))
                || e.table.uuid.is_empty()
                || e.table.uuid.len() > 128
                || e.table.uuid.contains('\0')
                || e.identity_column != "id"
                || e.native_type != "BIGINT"
            {
                return Err(fail(
                    "WFT-BINDING",
                    "Invalid or conflicting native edge source declaration",
                ));
            }
            edges.push(e.relationship.clone())
        }
        let required: BTreeSet<_> = plan
            .capabilities()
            .iter()
            .chain(&validated.additional_capabilities)
            .cloned()
            .collect();
        for id in &required {
            if admitted(id).is_none() {
                return Err(fail("WFT-CAPABILITY", "Binding capability refused"));
            }
        }
        let assessments = self.backend.assess(&context, &validated.mapping)?;
        let mut seen = BTreeSet::new();
        let mut qualifications = vec![];
        let mut obligations = vec![];
        merge(&mut obligations, &validated.obligations)?;
        for a in &assessments {
            if !seen.insert(a.id.clone()) || !required.contains(&a.id) {
                return Err(fail(
                    "WFT-CAPABILITY",
                    "Duplicate or unrequested assessment",
                ));
            }
            let c = admitted(&a.id).unwrap();
            if a.status == Status::Unsupported
                || (a.status == Status::Candidate && !target.allow_candidate)
                || !unique(&a.evidence)
                || (a.status == Status::Supported && a.evidence.is_empty())
                || a.evidence
                    .iter()
                    .any(|e| !self.manifest.evidence.contains(e))
            {
                return Err(fail("WFT-CAPABILITY", "Invalid assessment qualification"));
            }
            merge(&mut obligations, &a.obligations)?;
            merge(&mut obligations, &c.obligations)?;
            qualifications.push(crate::backend::Qualification {
                assessment: a.clone(),
                declaration: c.clone(),
            });
        }
        if seen != required {
            return Err(fail("WFT-CAPABILITY", "Assessment omits operation"));
        }
        let lowered = self.backend.lower(&context, &validated.mapping)?;
        let mut emitted = self.backend.emit(&context, &lowered)?;
        merge(&mut obligations, &emitted.obligations)?;
        emitted.obligations = obligations;
        emission::validate(
            plan,
            &emitted,
            &selection,
            &qualifications,
            profile,
            &self.manifest,
            binding,
            &edge_sources,
            &record_sources,
        )?;
        Ok(Compilation {
            backend_id: self.manifest.backend_id.clone(),
            backend_version: self.manifest.backend_version.clone(),
            target_profile: profile.clone(),
            binding_profile: binding.profile.clone(),
            binding_sha256: binding.sha256.clone(),
            qualifications,
            emission: emitted,
        })
    }
}

fn valid_table(table: &PinnedTable) -> bool {
    table.version >= 0
        && table
            .name
            .iter()
            .all(|n| !n.is_empty() && n.len() <= 128 && !n.contains('\0'))
        && !table.uuid.is_empty()
        && table.uuid.len() <= 128
        && !table.uuid.contains('\0')
}

fn valid_name(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && !value.contains('\0')
}
