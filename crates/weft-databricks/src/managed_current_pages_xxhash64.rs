// SPDX-License-Identifier: MIT OR Apache-2.0
//! Managed UC Spark Backend02 typed pages for the .4 xxhash64 physical layout.
use crate::candidate::Candidate;
use weft_core::{backend::*, error::Result};
pub const ID: &str = "ashlar.databricks.managed-uc-spark-xxhash64";
pub const VERSION: &str = "0.2.0-managed-current-core-xxhash64-pages-candidate";
pub const PROFILE: &str = "ashlar-managed-current-core/0.2-xxhash64";
pub struct ManagedCurrentPagesXxhash64;
fn native() -> Obligation {
    Obligation {
        id: "ashlar.nativeProfile".into(),
        owner: ObligationOwner::Host,
        failure_code: "WFT-OBLIGATION".into(),
        parameters: serde_json::json!({
         "targetProfile":PROFILE,"storageLayoutRevision":"ashlar-delta/0.4-xxhash64","lookupHashProfile":"spark-xxhash64-typed-key/0.1",
         "requirements":["verify the selected Databricks serverless Spark Jobs runtime/build, authenticated principal, Unity Catalog catalog/schema/table UUIDs and Delta protocol before SQL","verify physical .4-xxhash64 layout and typed BIGINT lookup-hash profile from the selected publication","bind exactly four graph-role tables and the complete pinned publication vector; keep six-role installation inventory separate","retain source/publication authority through each SDK page and continuation","preserve original typed request, selected key, continuation identity, UTF8 ordering and exact values; lookup hash is candidate selection only","recheck runtime, principal, UC identities, protocol, retention, source and publication before releasing buffered pages"],
         "qualification":"managed current-core .4 xxhash64 typed page candidate; host fills values from selected observed runtime/publication"
        }),
    }
}
impl Backend for ManagedCurrentPagesXxhash64 {
    type Mapping = <Candidate as Backend>::Mapping;
    type TargetPlan = <Candidate as Backend>::TargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let mut m = Candidate.describe()?;
        m.backend_id = ID.into();
        m.backend_version = VERSION.into();
        m.binding_profile = "ashlar-managed-current-core/0.1".into();
        m.target_profiles[0].id = PROFILE.into();
        m.target_profiles[0].engine = "databricks-serverless-spark-jobs".into();
        m.target_profiles[0].engine_version = "host-observed".into();
        m.target_profiles[0].session_settings = serde_json::json!({"comparison":"UTF8_BINARY","runtime":"host-observed","lookupHashProfile":"spark-xxhash64-typed-key/0.1"});
        m.target_profiles[0].storage_layout_revision = "ashlar-delta/0.4-xxhash64".into();
        m.target_profiles[0].publication_revision =
            "ashlar-managed-current-core/0.2-xxhash64".into();
        for c in &mut m.capabilities {
            c.target_profiles = vec![PROFILE.into()];
            c.obligations.retain(|o| o.id != "ashlar.nativeProfile");
            c.obligations.push(native());
        }
        Ok(m)
    }
    fn validate_binding(&self, c: &Context<'_>) -> Result<Validated<Self::Mapping>> {
        if c.target.id != PROFILE {
            return Err(weft_core::error::Diagnostic::new(
                "WFT-BINDING",
                "lower",
                "Exact managed current-core xxhash64 page target required",
            ));
        }
        let mut v = Candidate.validate_binding(c)?;
        crate::managed_current_core_xxhash64::refine_managed_publication(&mut v.obligations);
        Ok(v)
    }
    fn assess(&self, c: &Context<'_>, m: &Self::Mapping) -> Result<Vec<Assessment>> {
        Candidate.assess(c, m)
    }
    fn lower(&self, c: &Context<'_>, m: &Self::Mapping) -> Result<Self::TargetPlan> {
        Candidate.lower(c, m)
    }
    fn emit(&self, c: &Context<'_>, p: &Self::TargetPlan) -> Result<Emission> {
        let mut e = Candidate.emit(c, p)?;
        e.obligations.retain(|o| o.id != "ashlar.nativeProfile");
        crate::managed_current_core_xxhash64::refine_managed_publication(&mut e.obligations);
        e.obligations.push(native());
        Ok(e)
    }
}
