//! Explicit trusted security registration, distinct from ordinary SQL emitters.
use crate::{backend::{Capability, LanguageProfile, Manifest, TargetProfile}, error::{Diagnostic, Result}, json::checked_json, model::Catalog, security_ir::SecurityLogicalPlan, security_query_profile::SecurityProfiledQuery, security_query_uses::SecurityResolvedQuery};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
pub use crate::security_lowering::SecurityLowering;
fn fail(message: &str) -> Diagnostic { Diagnostic::new("WFT-SECURITY-BACKEND-VERSION", "capability", message) }
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecuritySourceProfile {
    pub dialect: String,
    pub application_ir: String,
    pub policy: String,
    pub ontology: String,
    pub security_ir: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityManifest {
    pub interface_version: String,
    pub backend_id: String,
    pub backend_version: String,
    pub source_profiles: Vec<SecuritySourceProfile>,
    pub binding_profile: String,
    pub target_profiles: Vec<TargetProfile>,
    pub capabilities: Vec<Capability>,
    pub evidence: Vec<String>,
}
fn identity(s: &str) -> bool { !s.is_empty() && !s.contains('\0') && s.chars().count() <= 4096 }
fn identities(values: &[String]) -> bool { values.len() <= 4096 && values.iter().all(|v| identity(v)) && values.iter().collect::<BTreeSet<_>>().len() == values.len() }
// Serde accepts positional sequences for structs; selected declarations are objects.
// Guard only typed declaration carriers, leaving opaque domain/parameter JSON intact.
fn object_entries(value: &serde_json::Value) -> bool {
    value.as_array().is_some_and(|entries| entries.iter().all(serde_json::Value::is_object))
}
fn object_manifest(value: &serde_json::Value) -> bool {
    value.is_object() && object_entries(&value["sourceProfiles"])
        && object_entries(&value["targetProfiles"]) && object_entries(&value["capabilities"])
        && value["capabilities"].as_array().is_some_and(|entries| entries.iter().all(|entry|
            object_entries(&entry["languageProfiles"]) && object_entries(&entry["obligations"])))
}
fn faithful_manifest(mut value: serde_json::Value) -> Result<SecurityManifest> {
    let opaque=crate::backend::take_manifest_opaque(&mut value).map_err(|_|fail("Malformed opaque declaration members"))?;
    let mut manifest:SecurityManifest=serde_json::from_value(value).map_err(|_|fail("Unknown or malformed security manifest members"))?;
    crate::backend::restore_manifest_opaque(&mut manifest.target_profiles,&mut manifest.capabilities,opaque).map_err(|_|fail("Declaration correspondence changed"))?;
    Ok(manifest)
}
pub fn validate_security_manifest_json(raw: &str) -> Result<SecurityManifest> {
    if raw.len() > 1024*1024 { return Err(fail("Security manifest exceeds one MiB")); }
    let value = checked_json(raw).map_err(|_| fail("Invalid or duplicate-key security manifest"))?;
    if !object_manifest(&value) { return Err(fail("Security declaration carriers must be objects")); }
    let manifest=faithful_manifest(value)?;
    if manifest.interface_version != "weft-security-backend/0.1.0" || manifest.source_profiles.len() != 1 {
        return Err(fail("Unsupported security interface or source tuple count"));
    }
    let profile=&manifest.source_profiles[0];
    if (&*profile.dialect,&*profile.application_ir,&*profile.policy,&*profile.ontology,&*profile.security_ir) != ("weft-sql/0.2.0","weft-ir/0.2.0","0.1.0","0.1.0","weft.security.logical-ir/0.1.0") {
        return Err(fail("Unsupported exact security source tuple"));
    }
    let language=LanguageProfile { dialect_profile:profile.dialect.clone(), ir_version:profile.application_ir.clone() };
    if ![&manifest.backend_id,&manifest.backend_version,&manifest.binding_profile].iter().all(|s| identity(s)) || !identities(&manifest.evidence) {
        return Err(fail("Invalid bounded security identity or evidence"));
    }
    for t in &manifest.target_profiles {
        if ![&t.id,&t.engine,&t.engine_version,&t.storage_layout_revision,&t.publication_revision].iter().all(|s| identity(s)) { return Err(fail("Invalid bounded target identity")); }
    }
    for c in &manifest.capabilities {
        if !identity(&c.id) || c.language_profiles != [language.clone()] || !identities(&c.target_profiles) || !identities(&c.constraints) || !identities(&c.evidence) || c.obligations.len()>4096 {
            return Err(fail("Security capability bounds or exact language selection refused"));
        }
        for o in &c.obligations { if !identity(&o.id) || !identity(&o.failure_code) { return Err(fail("Invalid bounded obligation identity")); } }
    }
    // Reuse validation of the shared nested types only. This does not register
    // an ordinary emitter or grant security capability/semantic admission.
    crate::backend::validate_manifest(&Manifest {
        backend_id:manifest.backend_id.clone(), backend_version:manifest.backend_version.clone(), interface_version:"weft-backend/0.2.0".into(), language_profiles:vec![language], binding_profile:manifest.binding_profile.clone(), target_profiles:manifest.target_profiles.clone(), capabilities:manifest.capabilities.clone(), evidence:manifest.evidence.clone(),
    }).map_err(|_| fail("Invalid security target/capability references or declarations"))?;
    Ok(manifest)
}
/// No public constructor or deserializer. These references are actual owner
/// objects that an external host cannot substitute through public fields.
/// ```compile_fail
/// use weft_core::security_backend::SecurityBackendContext;
/// fn substitute(ctx: &mut SecurityBackendContext<'_>) { ctx.binding_json = "{}"; }
/// ```
/// These references are actual owner
/// objects with source/profile reuse checked immediately before dispatch.
pub struct SecurityBackendContext<'a> {
    catalog: &'a Catalog,
    logical_plan: &'a SecurityLogicalPlan,
    query: &'a SecurityResolvedQuery,
    profiled_query: &'a SecurityProfiledQuery<'a>,
    binding_json: &'a str,
    backend_id: &'a str,
    backend_version: &'a str,
    target_profile: &'a str,
    requirements: crate::security_requirements::SecurityRequirements<'a>,
}
/// Per-row scan/action/rule truths are conditional simulation inputs, never credentials.
pub type SecuritySimulatedRowTruths = BTreeMap<String,BTreeMap<String,BTreeMap<String,crate::security_composition::Truth>>>;
impl<'a> SecurityBackendContext<'a> {
    pub(crate) fn new(catalog: &'a Catalog, logical_plan: &'a SecurityLogicalPlan, query: &'a SecurityResolvedQuery, profiled_query: &'a SecurityProfiledQuery<'a>, binding_json: &'a str, backend_id: &'a str, backend_version: &'a str, target_profile: &'a str) -> Result<Self> {
        profiled_query.require_sources(logical_plan,catalog,binding_json,backend_id,backend_version,target_profile)?;
        if !std::ptr::eq(profiled_query.query(),query) { return Err(fail("Mismatched actual query context")); }
        let requirements = crate::security_requirements::SecurityRequirements::derive(profiled_query, logical_plan)?;
        Ok(Self { catalog,logical_plan,query,profiled_query,binding_json,backend_id,backend_version,target_profile,requirements })
    }
    /// Pure declaration correspondence only; never permission to select/release a cell.
    pub fn check_result_declaration(&self, contract: &crate::security_lowering::SecurityResultContract) -> Result<()> { crate::security_result_check::check(self, contract) }
    /// Pure contract/cell correspondence; no permission to select or release results.
    pub fn check_result_cells(&self, contract: &crate::security_lowering::SecurityResultContract, contract_json: &str, contract_sha256: &str, batch_json: &str) -> Result<()> { crate::security_result_cells::check(self,contract,contract_json,contract_sha256,batch_json) }
    /// Conditional complete-rule fold only; supplied truths do not grant authority.
    pub fn check_simulated_result_selection(&self, contract: &crate::security_lowering::SecurityResultContract, contract_json: &str, hash: &str, batch_json: &str, truth_rows: &[SecuritySimulatedRowTruths]) -> Result<()> { crate::security_result_selection::check(self,contract,contract_json,hash,batch_json,truth_rows) }
    /// Conditional evaluated facts and Original values; caller facts are not credentials.
    pub fn check_simulated_fact_selection(&self, contract: &crate::security_lowering::SecurityResultContract, contract_json: &str, hash: &str, batch_json: &str, cut_json: &str, rows_json: &str) -> Result<()> { crate::security_evaluation::check_owner_rows(self,contract,contract_json,hash,batch_json,cut_json,rows_json) }
    pub fn requirements(&self) -> &crate::security_requirements::SecurityRequirements<'a> { &self.requirements }
    pub fn catalog(&self) -> &Catalog { self.catalog }
    pub fn logical_plan(&self) -> &SecurityLogicalPlan { self.logical_plan }
    pub fn query(&self) -> &SecurityResolvedQuery { self.query }
    pub fn profiled_query(&self) -> &SecurityProfiledQuery<'a> { self.profiled_query }
    pub fn binding_json(&self) -> &str { self.binding_json }
    pub fn backend_id(&self) -> &str { self.backend_id }
    pub fn backend_version(&self) -> &str { self.backend_version }
    pub fn target_profile(&self) -> &str { self.target_profile }
}
/// Pure, audited trusted host code. Rust callbacks are not an IO sandbox.
pub trait SecurityBackend: Send + Sync + 'static {
    fn manifest_json(&self) -> &str;
    fn lower(&self, context: &SecurityBackendContext<'_>) -> Result<SecurityLowering>;
}
struct Registration { manifest_json: String, manifest: SecurityManifest, backend: Box<dyn SecurityBackend> }
/// Immutable original registry selection. Custody only; not semantic admission.
#[derive(Clone, Copy)]
pub(crate) struct RegisteredSecurityDeclaration<'a> {
    registration: &'a Registration,
    target: &'a TargetProfile,
}
impl<'a> RegisteredSecurityDeclaration<'a> {
    pub(crate) fn manifest_json(&self) -> &'a str { &self.registration.manifest_json }
    pub(crate) fn manifest(&self) -> &'a SecurityManifest { &self.registration.manifest }
    pub(crate) fn target(&self) -> &'a TargetProfile { self.target }
}
#[derive(Default)]
pub struct SecurityRegistry { entries: BTreeMap<(String,String),Registration> }
impl SecurityRegistry {
    pub fn register<B: SecurityBackend>(&mut self, backend: B) -> Result<()> {
        let raw=backend.manifest_json();
        if raw.len()>1024*1024 { return Err(fail("Security manifest exceeds one MiB")); }
        let manifest=validate_security_manifest_json(raw)?;
        let manifest_json=raw.to_owned();
        let key=(manifest.backend_id.clone(),manifest.backend_version.clone());
        if self.entries.contains_key(&key) { return Err(fail("Duplicate security registration")); }
        self.entries.insert(key,Registration {manifest_json,manifest,backend:Box::new(backend)});
        Ok(())
    }
    pub(crate) fn select_declaration(&self, context: &SecurityBackendContext<'_>) -> Result<RegisteredSecurityDeclaration<'_>> {
        self.declaration_for(context.backend_id,context.backend_version,context.target_profile)
    }
    pub(crate) fn declaration_for(&self, backend_id: &str, backend_version: &str, target_id: &str) -> Result<RegisteredSecurityDeclaration<'_>> {
        let registration=self.entries.get(&(backend_id.into(),backend_version.into())).ok_or_else(|| Diagnostic::new("WFT-SECURITY-BACKEND-REQUIRED","capability","Exact security backend registration is required"))?;
        let target=registration.manifest.target_profiles.iter().find(|p|p.id==target_id).ok_or_else(|| fail("Exact requested target profile is not registered"))?;
        Ok(RegisteredSecurityDeclaration {registration,target})
    }
    pub(crate) fn select(&self, context: &SecurityBackendContext<'_>) -> Result<(&dyn SecurityBackend,&SecurityManifest)> {
        let declaration=self.select_declaration(context)?;
        Ok((&*declaration.registration.backend,declaration.manifest()))
    }
}
pub type SecurityRegistryFactory<'host> = dyn for<'context> FnMut(&SecurityBackendContext<'context>) -> Result<SecurityRegistry> + 'host;

/// Host callbacks can construct mutable diagnostics. Refusals must remain
/// bounded error-shaped output even if a trusted callback supplies bad fields.
pub(crate) fn callback_error(d: Diagnostic, sql_bytes: usize) -> Diagnostic {
    let code=d.code.strip_prefix("WFT-").is_some_and(|tail| !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b==b'-'));
    let phase=matches!(d.phase.as_str(),"input"|"model"|"parse"|"resolve"|"type"|"capability"|"lower"|"emit"|"host");
    let span=d.source_span.as_ref().is_none_or(|s|s.start<=s.end && s.end<=sql_bytes);
    if d.severity=="error" && code && d.code.len()<=4096 && phase && !d.message.is_empty() && d.message.len()<=4096 && !d.message.contains('\0') && span { d }
    else { fail("Security callback returned an invalid or unbounded error diagnostic") }
}
