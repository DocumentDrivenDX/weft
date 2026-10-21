// SPDX-License-Identifier: MIT OR Apache-2.0
//! Separate exact Spark 4.2/Delta 4.4.1 local candidate. No old-profile upgrade.
use crate::candidate::Candidate;
use serde_json::json;
use weft_core::{backend::*, error::Result};

pub const VERSION: &str = "0.1.0-local-spark4.2-candidate";
pub const PROFILE: &str = "spark4.2.0-delta4.4.1-candidate";
pub const DELTA_SPARK_SHA256: &str = "2b1d2cd411bf0bbced64b96fe621f8ffce7bb6e394c8064ab783fc6edce449be";
pub const DELTA_STORAGE_SHA256: &str = "f585818d646cb3625b305270ec24f3e6f0b6ea63ed9bac1ae8f340f8faaeae8f";

pub struct LocalSpark42;

fn execution_profile() -> Obligation {
    Obligation {
        id: "ashlar.nativeProfile".into(),
        parameters: json!({
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
                "retain the same admitted catalog/publication context across checks and buffered result publication",
                "refuse mismatched or unavailable profile evidence; never silently choose a newer engine or another profile"
            ],
            "qualification": "local development candidate; native publication, authorization and complete pin custody remain mandatory"
        }),
        owner: ObligationOwner::Host,
        failure_code: "WFT-OBLIGATION".into(),
    }
}

impl Backend for LocalSpark42 {
    type Mapping = <Candidate as Backend>::Mapping;
    type TargetPlan = <Candidate as Backend>::TargetPlan;

    fn describe(&self) -> Result<Manifest> {
        let mut manifest = Candidate.describe()?;
        manifest.backend_version = VERSION.into();
        let target = &mut manifest.target_profiles[0];
        target.id = PROFILE.into();
        target.engine = "spark-sql".into();
        target.engine_version = "4.2.0".into();
        target.session_settings = json!({
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
        target.storage_layout_revision = "ashlar-delta/0.3".into();
        target.publication_revision = "host-verified-local-spark4.2/0.1".into();
        for capability in &mut manifest.capabilities {
            capability.target_profiles = vec![PROFILE.into()];
            capability.obligations.push(execution_profile());
            capability.constraints.push(
                "Exact Spark 4.2.0, Delta 4.4.1, JDK 21 and pinned JARs required; candidate opt-in remains mandatory".into(),
            );
        }
        Ok(manifest)
    }

    fn validate_binding(&self, context: &Context<'_>) -> Result<Validated<Self::Mapping>> {
        let mut validated = Candidate.validate_binding(context)?;
        validated.obligations.push(execution_profile());
        Ok(validated)
    }

    fn assess(&self, context: &Context<'_>, mapping: &Self::Mapping) -> Result<Vec<Assessment>> {
        Candidate.assess(context, mapping)
    }

    fn lower(&self, context: &Context<'_>, mapping: &Self::Mapping) -> Result<Self::TargetPlan> {
        Candidate.lower(context, mapping)
    }

    fn emit(&self, context: &Context<'_>, plan: &Self::TargetPlan) -> Result<Emission> {
        Candidate.emit(context, plan)
    }
}
