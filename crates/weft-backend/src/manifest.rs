// SPDX-License-Identifier: MIT OR Apache-2.0
//! Backend manifests (CONTRACT-002): identity, language and target profiles,
//! capabilities with evidence and obligations, and their structural validation.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use weft_model::{
    diagnostic::{Diagnostic, Result},
    json::checked_json,
};
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
/// A capability-phase refusal.
pub fn failure(code: &str, message: &str) -> Diagnostic {
    Diagnostic::new(code, "capability", message)
}
/// Non-empty text without NUL.
pub fn nonempty(s: &str) -> bool {
    !s.is_empty() && !s.contains('\0')
}
/// Distinct non-empty texts.
pub fn unique_strings(v: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    v.iter().all(|s| nonempty(s) && seen.insert(s))
}
/// Which manifest interface versions and language/IR pairs a frontend accepts.
pub trait ManifestRules {
    /// Whether `interface` (a manifest `interfaceVersion`) is accepted.
    fn admits_interface(interface: &str) -> bool;
    /// Whether `profile` is a valid language/IR pair under `interface`.
    fn admits_language(interface: &str, profile: &LanguageProfile) -> bool;
}
fn valid_languages<R: ManifestRules>(profiles: &[LanguageProfile], interface: &str) -> bool {
    let mut seen = BTreeSet::new();
    !profiles.is_empty()
        && profiles.iter().all(|p| R::admits_language(interface, p))
        && profiles.iter().all(|p| seen.insert((&p.dialect_profile, &p.ir_version)))
}
/// Opaque declaration members, held aside while the typed manifest is read.
pub struct OpaqueManifest { settings:Vec<Value>, capabilities:Vec<(Value,Value,Vec<Value>)> }
fn take_opaque(value:&mut Value,key:&str)->Result<Value>{
 let member=value.as_object_mut().and_then(|o|o.get_mut(key)).ok_or_else(||failure("WFT-BACKEND-VERSION","Missing opaque declaration member"))?;
 Ok(std::mem::replace(member,Value::Object(Default::default())))
}
pub fn take_manifest_opaque(value:&mut Value)->Result<OpaqueManifest>{
 if !value.is_object(){return Err(failure("WFT-BACKEND-VERSION","Manifest must be an object"));}
 let mut settings=Vec::new();let mut capabilities=Vec::new();
 for target in value["targetProfiles"].as_array_mut().ok_or_else(||failure("WFT-BACKEND-VERSION","Invalid targets"))? {settings.push(take_opaque(target,"sessionSettings")?);}
 for capability in value["capabilities"].as_array_mut().ok_or_else(||failure("WFT-BACKEND-VERSION","Invalid capabilities"))? {
  if !capability["status"].as_str().is_some_and(|s|matches!(s,"supported"|"candidate"|"unsupported")){return Err(failure("WFT-BACKEND-VERSION","Status must be a declared string token"));}
  let logical=take_opaque(capability,"logicalDomain")?;let result=take_opaque(capability,"resultDomain")?;let mut parameters=Vec::new();
  for obligation in capability["obligations"].as_array_mut().ok_or_else(||failure("WFT-BACKEND-VERSION","Invalid obligations"))? {
   if !obligation["owner"].as_str().is_some_and(|s|matches!(s,"host"|"backend")){return Err(failure("WFT-BACKEND-VERSION","Obligation owner must be a declared string token"));}
   parameters.push(take_opaque(obligation,"parameters")?);
  }
  capabilities.push((logical,result,parameters));
 }Ok(OpaqueManifest{settings,capabilities})
}
pub fn restore_manifest_opaque(targets:&mut [TargetProfile],capabilities:&mut [Capability],opaque:OpaqueManifest)->Result<()>{
 if targets.len()!=opaque.settings.len()||capabilities.len()!=opaque.capabilities.len(){return Err(failure("WFT-BACKEND-VERSION","Declaration correspondence changed"));}
 for (target,setting) in targets.iter_mut().zip(opaque.settings){target.session_settings=setting;}
 for (capability,(logical,result,parameters)) in capabilities.iter_mut().zip(opaque.capabilities){
  if capability.obligations.len()!=parameters.len(){return Err(failure("WFT-BACKEND-VERSION","Obligation correspondence changed"));}
  capability.logical_domain=logical;capability.result_domain=result;
  for (obligation,parameter) in capability.obligations.iter_mut().zip(parameters){obligation.parameters=parameter;}
 }Ok(())
}
/// Read and validate a manifest JSON text under frontend rules `R`.
pub fn validate_manifest_json<R: ManifestRules>(raw: &str) -> Result<Manifest> {
    if raw.len() > 1024 * 1024 {
        return Err(failure("WFT-LIMIT", "Backend manifest exceeds one MiB"));
    }
    let mut value = checked_json(raw).map_err(|_| {
        failure(
            "WFT-BACKEND-VERSION",
            "Malformed or duplicate-key backend manifest JSON",
        )
    })?;
    let opaque=take_manifest_opaque(&mut value)?;
    let mut manifest: Manifest = serde_json::from_value(value).map_err(|_| {
        failure(
            "WFT-BACKEND-VERSION",
            "Backend manifest has unknown or malformed members",
        )
    })?;
    restore_manifest_opaque(&mut manifest.target_profiles,&mut manifest.capabilities,opaque)?;
    validate_manifest::<R>(&manifest)?;
    Ok(manifest)
}
/// Validate a manifest's structure under frontend rules `R`.
pub fn validate_manifest<R: ManifestRules>(m: &Manifest) -> Result<()> {
    if !R::admits_interface(&m.interface_version) || !valid_languages::<R>(&m.language_profiles, &m.interface_version) {
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
            || !valid_languages::<R>(&c.language_profiles, &m.interface_version)
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

