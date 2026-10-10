use super::*;
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
#[jsonschema::validator(
    path = "../../docs/helix/02-design/contracts/backend-manifest-v0.3.schema.json"
)]
struct Schema;
pub fn validate_manifest_json(raw: &str) -> Result<Manifest> {
    if raw.len() > 1024 * 1024 {
        return Err(fail("WFT-LIMIT", "Manifest exceeds one MiB"));
    }
    let v = crate::json::checked_json(raw)
        .map_err(|_| fail("WFT-BACKEND-VERSION", "Malformed or duplicate-key manifest"))?;
    if !Schema::is_valid(&v) {
        return Err(fail(
            "WFT-BACKEND-VERSION",
            "Manifest violates closed backend03 schema",
        ));
    }
    let m: Manifest = serde_json::from_value(v)
        .map_err(|_| fail("WFT-BACKEND-VERSION", "Invalid backend03 manifest"))?;
    validate(&m)?;
    Ok(m)
}
pub(super) fn validate(m: &Manifest) -> Result<()> {
    let value = bounded_json(m, 1024 * 1024)?;
    if !Schema::is_valid(&value) || !unique(&m.evidence) {
        return Err(fail(
            "WFT-BACKEND-VERSION",
            "Invalid bounded backend03 manifest",
        ));
    }
    if [&m.backend_id, &m.backend_version, &m.binding_profile]
        .iter()
        .any(|s| s.is_empty() || s.contains('\0'))
        || m.target_profiles.iter().any(|t| {
            [
                &t.id,
                &t.engine,
                &t.engine_version,
                &t.storage_layout_revision,
                &t.publication_revision,
            ]
            .iter()
            .any(|s| s.is_empty() || s.contains('\0'))
        })
    {
        return Err(fail(
            "WFT-BACKEND-VERSION",
            "Original manifest identifier rules require nonempty NUL-free names",
        ));
    }
    let targets: std::collections::BTreeSet<_> = m.target_profiles.iter().map(|t| &t.id).collect();
    if targets.len() != m.target_profiles.len() {
        return Err(fail("WFT-BACKEND-VERSION", "Duplicate target identity"));
    }
    let mut caps = std::collections::BTreeSet::new();
    for c in &m.capabilities {
        if c.id.is_empty()
            || c.id.contains('\0')
            || !caps.insert(&c.id)
            || !unique(&c.target_profiles)
            || c.target_profiles.iter().any(|p| !targets.contains(p))
            || !unique(&c.constraints)
            || !unique(&c.evidence)
            || c.evidence.iter().any(|e| !m.evidence.contains(e))
            || (c.status == Status::Supported && c.evidence.is_empty())
        {
            return Err(fail(
                "WFT-CAPABILITY",
                "Capability identity/profile/evidence invalid",
            ));
        }
        let mut ids = std::collections::BTreeSet::new();
        for o in &c.obligations {
            if !valid_obligation(o) || !ids.insert(&o.id) {
                return Err(fail("WFT-OBLIGATION", "Duplicate declared obligation"));
            }
        }
    }
    Ok(())
}
