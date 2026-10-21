// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exact Spark 4.2/Delta 4.4.1 realization for 0.4.1 grouped COUNT(*) HAVING.
//! The existing Spark 4.0.1 path target remains independently registered.
use crate::{binding::Binding, paths::PathTargetPlan, paths_keys::PathsKeys};
use weft_core::{
    backend03::*, backend::ObligationOwner,
    error::{Diagnostic, Result},
};

pub const ID: &str = "ashlar.databricks.paths-keys-having";
pub const VERSION: &str = "0.4.1-count-star-having-spark4.2-candidate";
pub const PROFILE: &str = "spark4.2.0-delta4.4.1-paths-keys-having-candidate";
pub const DELTA_SPARK_SHA256: &str = "2b1d2cd411bf0bbced64b96fe621f8ffce7bb6e394c8064ab783fc6edce449be";
pub const DELTA_STORAGE_SHA256: &str = "f585818d646cb3625b305270ec24f3e6f0b6ea63ed9bac1ae8f340f8faaeae8f";

pub struct CountStarHavingSpark42;

fn native_profile() -> Obligation {
    Obligation {
        id: "ashlar.nativeProfile".into(),
        parameters: serde_json::json!({
            "targetProfile": PROFILE,
            "engineVersion": "4.2.0",
            "deltaVersion": "4.4.1",
            "jdkVersion": "21",
            "deltaJars": {
                "delta-spark_4.2_2.13-4.4.1.jar": DELTA_SPARK_SHA256,
                "delta-storage-4.4.1.jar": DELTA_STORAGE_SHA256
            },
            "settings": {
                "spark.sql.ansi.enabled": "true",
                "spark.sql.session.timeZone": "UTC",
                "spark.sql.legacy.parameterSubstitution.constantsOnly": "true",
                "comparison": "explicit UTF8_BINARY",
                "resultTransport": "exact native STRING"
            },
            "requirements": [
                "verify the actual Spark version, JVM version, selected JAR hashes and loaded Delta API class before any integrity or user SQL",
                "retain the same admitted publication context across checks and buffered result publication",
                "refuse mismatched or unavailable profile evidence; never choose a different engine or target profile"
            ],
            "qualification": "candidate target; native publication, authorization and complete pin custody remain mandatory"
        }),
        owner: ObligationOwner::Host,
        failure_code: "WFT-OBLIGATION".into(),
    }
}

impl Backend for CountStarHavingSpark42 {
    type Mapping = Binding;
    type TargetPlan = PathTargetPlan;

    fn describe(&self) -> Result<Manifest> {
        let mut manifest = PathsKeys.describe()?;
        manifest.backend_id = ID.into();
        manifest.backend_version = VERSION.into();
        let profile = &mut manifest.target_profiles[0];
        profile.id = PROFILE.into();
        profile.engine_version = "4.2.0".into();
        profile.session_settings = serde_json::json!({
            "spark.sql.ansi.enabled": "true",
            "spark.sql.session.timeZone": "UTC",
            "spark.sql.legacy.parameterSubstitution.constantsOnly": "true",
            "comparison": "explicit UTF8_BINARY",
            "resultTransport": "exact native STRING",
            "deltaVersion": "4.4.1",
            "jdkVersion": "21",
            "deltaJars": {
                "delta-spark_4.2_2.13-4.4.1.jar": DELTA_SPARK_SHA256,
                "delta-storage-4.4.1.jar": DELTA_STORAGE_SHA256
            }
        });
        profile.publication_revision = "host-verified-local-spark4.2/0.1".into();
        let native = native_profile();
        for capability in &mut manifest.capabilities {
            capability.target_profiles = vec![PROFILE.into()];
            capability.obligations.push(native.clone());
        }
        let language = LanguageProfile {
            dialect_profile: "weft-sql/0.4.1".into(),
            ir_version: "weft-ir/0.4.1".into(),
        };
        manifest.language_profiles = vec![language.clone()];
        for capability in &mut manifest.capabilities {
            capability.language_profiles = vec![language.clone()];
        }
        let mut count_star = manifest
            .capabilities
            .iter()
            .find(|capability| capability.id == "aggregate.havingCountDistinctGreater")
            .cloned()
            .ok_or_else(|| {
                Diagnostic::new(
                    "WFT-BACKEND-VERSION",
                    "capability",
                    "Owned row-count realization is missing its source capability",
                )
            })?;
        count_star.id = "aggregate.havingCountStarGreater".into();
        count_star.logical_domain = serde_json::json!({
            "subset": "explicitly projected COUNT(*) with nonempty required exact String groups; one HAVING > original nonnegative Integer literal"
        });
        count_star.result_domain = serde_json::json!({
            "count": "mathematicalInteger; complete pre-HAVING bag signed64 capacity or refusal",
            "threshold": "exact nonnegative signed64 capacity; no logical width"
        });
        manifest.capabilities.push(count_star);
        Ok(manifest)
    }

    fn validate_binding(&self, context: &Context<'_>) -> Result<Validated<Binding>> {
        if context.plan.ir_version() != "weft-ir/0.4.1" {
            return Err(Diagnostic::new(
                "WFT-CAPABILITY",
                "capability",
                "Exact 0.4.1 row-count realization required",
            ));
        }
        let mut validated = PathsKeys.validate_binding_for_profile(context, PROFILE)?;
        validated.obligations.push(native_profile());
        Ok(validated)
    }

    fn assess(&self, context: &Context<'_>, binding: &Binding) -> Result<Vec<Assessment>> {
        PathsKeys.assess(context, binding)
    }

    fn lower(&self, context: &Context<'_>, binding: &Binding) -> Result<PathTargetPlan> {
        PathsKeys.lower(context, binding)
    }

    fn emit(&self, context: &Context<'_>, plan: &PathTargetPlan) -> Result<Emission> {
        PathsKeys.emit(context, plan)
    }
}
