// SPDX-License-Identifier: MIT OR Apache-2.0
//! Explicit managed UC Spark target for the physical Ashlar .4 xxhash64 layout.
//! The lookup hash is storage metadata; this route preserves the graph query SQL.
use crate::{
    binding::Binding, paths::PathTargetPlan, paths_keys_having_spark42::CountStarHavingSpark42,
};
use weft_core::{
    backend::{Obligation, ObligationOwner},
    backend03::*,
    error::{Diagnostic, Result},
};

pub const ID: &str = "ashlar.databricks.managed-uc-spark-xxhash64";
pub const VERSION: &str = "0.2.0-managed-current-core-xxhash64-candidate";
pub const PROFILE: &str = "ashlar-managed-current-core/0.2-xxhash64";
pub struct ManagedCurrentCoreXxhash64;

pub(crate) fn refine_managed_publication(obligations: &mut [Obligation]) {
    for obligation in obligations
        .iter_mut()
        .filter(|item| item.id == "ashlar.candidate.publication")
    {
        obligation.parameters["nativeProfile"] = serde_json::json!({
         "targetProfile":PROFILE,"storageLayoutRevision":"ashlar-delta/0.4-xxhash64",
         "lookupHashProfile":"spark-xxhash64-typed-key/0.1","engine":"databricks-serverless-spark-jobs",
         "runtime":"host-observed","comparison":"UTF8_BINARY","arithmetic":"ANSI exact-or-error",
         "requirements":[
          "require host-observed selected serverless Spark Jobs runtime/build, authenticated principal, UC namespace, table UUIDs and Delta protocol",
          "require physical ashlar-delta/0.4-xxhash64 layout and spark-xxhash64-typed-key/0.1 BIGINT lookup hash on applicable current tables",
          "resolve and verify the complete pinned manifest vector for exactly object_current, edge_current, tombstone and whole_source_history",
          "keep the six-role installation inventory separate from the four-role manifest data vector",
          "bind publication opening and closing checks to the same observed UC tables, versions, schema and retained data files",
          "treat lookup_hash only as a candidate selector; preserve exact typed graph identity and query semantics"
         ]
        });
    }
}
fn obligation() -> Obligation {
    Obligation {
        id: "ashlar.nativeProfile".into(),
        owner: ObligationOwner::Host,
        failure_code: "WFT-OBLIGATION".into(),
        parameters: serde_json::json!({
         "targetProfile":PROFILE,"storageLayoutRevision":"ashlar-delta/0.4-xxhash64","lookupHashProfile":"spark-xxhash64-typed-key/0.1",
        "requirements":["verify the selected Databricks serverless Spark Jobs runtime/build, authenticated principal, Unity Catalog catalog/schema/table UUIDs and Delta protocol before SQL","verify the .4-xxhash64 layout and typed BIGINT lookup-hash profile from the selected publication","bind exactly the four graph-role tables and complete pinned publication vector; retain six-role installation inventory separately","retain original source and publication authority over opening, execution and closing checks","preserve exact UTF8_BINARY comparison, typed results and original compiler SQL/parameters; hash is a physical candidate index only","recheck runtime, principal, table identity, protocol, retention, source and publication custody before releasing buffered results"],
         "qualification":"managed current-core xxhash64 candidate; host must fill values from selected observed runtime and publication"
        }),
    }
}
impl Backend for ManagedCurrentCoreXxhash64 {
    type Mapping = Binding;
    type TargetPlan = PathTargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let mut m = CountStarHavingSpark42.describe()?;
        m.backend_id = ID.into();
        m.backend_version = VERSION.into();
        m.binding_profile = "ashlar-managed-current-core/0.1".into();
        let p = &mut m.target_profiles[0];
        p.id = PROFILE.into();
        p.engine = "databricks-serverless-spark-jobs".into();
        p.engine_version = "host-observed".into();
        p.session_settings = serde_json::json!({"comparison":"UTF8_BINARY","runtime":"host-observed","lookupHashProfile":"spark-xxhash64-typed-key/0.1"});
        p.storage_layout_revision = "ashlar-delta/0.4-xxhash64".into();
        p.publication_revision = "ashlar-managed-current-core/0.2-xxhash64".into();
        for c in &mut m.capabilities {
            c.target_profiles = vec![PROFILE.into()];
            c.obligations.retain(|o| o.id != "ashlar.nativeProfile");
            c.obligations.push(obligation());
            c.constraints.push("Managed .4 physical layout and publication custody are host obligations; lookup_hash does not change graph identity or query semantics".into());
        }
        Ok(m)
    }
    fn validate_binding(&self, c: &Context<'_>) -> Result<Validated<Binding>> {
        if c.plan.ir_version() != "weft-ir/0.4.1" {
            return Err(Diagnostic::new(
                "WFT-CAPABILITY",
                "capability",
                "Current-core grouped Backend03 requires weft-ir/0.4.1",
            ));
        }
        let mut v =
            crate::paths_keys::PathsKeys.validate_binding_managed_current_xxhash64(c, PROFILE)?;
        refine_managed_publication(&mut v.obligations);
        Ok(v)
    }
    fn assess(&self, c: &Context<'_>, b: &Binding) -> Result<Vec<Assessment>> {
        CountStarHavingSpark42.assess(c, b)
    }
    fn lower(&self, c: &Context<'_>, b: &Binding) -> Result<PathTargetPlan> {
        CountStarHavingSpark42.lower(c, b)
    }
    fn emit(&self, c: &Context<'_>, p: &PathTargetPlan) -> Result<Emission> {
        let mut e = CountStarHavingSpark42.emit(c, p)?;
        e.obligations.retain(|o| o.id != "ashlar.nativeProfile");
        refine_managed_publication(&mut e.obligations);
        e.obligations.push(obligation());
        Ok(e)
    }
}
