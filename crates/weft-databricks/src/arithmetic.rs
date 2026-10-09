//! Explicit finite native representation; original mathematical domains remain unchanged.
use crate::{
    binding::Binding,
    candidate::{Candidate, TargetPlan},
};
use serde_json::json;
use weft_core::{
    backend::*,
    error::{Diagnostic, Result},
};
pub const PROFILE: &str = "spark4-delta4-arithmetic-candidate";
pub struct Arithmetic;
impl Backend for Arithmetic {
    type Mapping = Binding;
    type TargetPlan = TargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let mut manifest = Candidate.describe()?;
        manifest.backend_version = "0.3.0-arithmetic-candidate".into();
        manifest.language_profiles = vec![LanguageProfile {
            dialect_profile: "weft-sql/0.3.0".into(),
            ir_version: "weft-ir/0.3.0".into(),
        }];
        for profile in &mut manifest.target_profiles {
            profile.id = PROFILE.into();
            profile.engine = "spark-sql".into();
            profile.engine_version = "4.0.1".into();
            profile.session_settings = json!({"comparison":"UTF8_BINARY","spark.sql.ansi.enabled":true,"spark.sql.session.timeZone":"UTC","numericRepresentation":"DECIMAL(38,0) coefficients; exact-or-capability-failure","maxDerivedScale":18,"sourceValidity":"public UMF independently; native capacity is separate"});
        }
        manifest.capabilities.retain(|c| {
            [
                "scan",
                "project",
                "filter",
                "innerJoin",
                "equal",
                "and",
                "parameter.named",
                "compare.lexicographicGreater",
                "order.asc",
                "limit",
                "key.uniqueStable",
                "type.string",
                "type.boolean",
                "type.integer",
                "type.decimal",
                "value.presence",
            ]
            .contains(&c.id.as_str())
        });
        for capability in &mut manifest.capabilities {
            capability.target_profiles = vec![PROFILE.into()];
            capability.language_profiles = manifest.language_profiles.clone();
            capability.logical_domain = if capability.id=="value.presence" {json!({"subset":"selected optional scalar envelope with explicit original native-null property encoding; missing keys refuse physical profile"})}else{json!({"subset":"required scalar nonaggregate row queries; original Field facets preserved"})};
            capability.result_domain =
                json!({"carrier":"exact text; original logical domains unchanged"});
            capability.constraints = vec!["All consumed original source values and every unfiltered intermediate require exact native guards before buffered result release".into(),"Finite coefficient capacity is not source validity or general arbitrary precision support".into()];
        }
        manifest.capabilities.push(Capability {id:"project.positionedOutputs".into(),target_profiles:vec![PROFILE.into()],language_profiles:manifest.language_profiles.clone(),logical_domain:json!({"subset":"repeated implicit scalar Field labels; logical positions/names/Field scan lineage retained; explicit/computed duplicate labels refuse"}),result_domain:json!({"carrier":"unique physical names separate from repeated logical labels; ordered exact row arrays"}),constraints:vec!["Host must explicitly admit carrierName and decode by position without dictionary collapse".into()],obligations:vec![],status:Status::Candidate,evidence:vec![]});
        manifest.capabilities.push(Capability {id:"project.distinct".into(),target_profiles:vec![PROFILE.into()],language_profiles:manifest.language_profiles.clone(),logical_domain:json!({"subset":"final required exact String direct-Field tuple distinct; no input bag or aggregate dedup"}),result_domain:json!({"equality":"UTF8_BINARY; all original projection positions retained"}),constraints:vec!["All consumed source guards precede user SQL; hidden ordering Fields and page profile refuse".into()],obligations:vec![],status:Status::Candidate,evidence:vec![]});
        for id in ["predicate.nativeNull","compare.nullAwareStringEqual","value.nativeNull"] {
            manifest.capabilities.push(Capability{id:id.into(),target_profiles:vec![PROFILE.into()],language_profiles:manifest.language_profiles.clone(),logical_domain:json!({"subset":"selected scalar optional envelopes; explicit present-null or exact nonnull value; missing keys refuse finite physical representation; optional arithmetic refuses"}),result_domain:json!({"carrier":"tagged state:null or state:value exact-text/Boolean; ideal scalar domain unchanged"}),constraints:vec!["Exact selected property home and original source/type/revision guards required before query".into()],obligations:vec![],status:Status::Candidate,evidence:vec![]});
        }
        for id in ["compare.less", "compare.lessEqual", "compare.greaterEqual", "compare.notEqual", "compare.scalarJoin"] {
            manifest.capabilities.push(Capability { id:id.into(),target_profiles:vec![PROFILE.into()],language_profiles:manifest.language_profiles.clone(),logical_domain:json!({"subset":"new 0.3 scalar operators; same exact source family; string UTF8_BINARY; Boolean only not-equal; explicit new-operator ON admission"}),result_domain:json!({"comparison":"exact coefficients or original scalar meaning; no null widening"}),constraints:vec!["All numeric scale alignment guards cover complete pre-ON/pre-WHERE candidate bags".into()],obligations:vec![],status:Status::Candidate,evidence:vec![] });
        }
        for id in [
            "arithmetic.exact.integer",
            "arithmetic.exact.decimal",
            "arithmetic.+",
            "arithmetic.-",
            "arithmetic.*",
            "arithmetic.negate",
            "arithmetic.compareExact",
            "type.integer.unbounded",
        ] {
            manifest.capabilities.push(Capability {id:id.into(),target_profiles:vec![PROFILE.into()],language_profiles:manifest.language_profiles.clone(),logical_domain:json!({"numericDomain":"original mathematical integer or derived decimal scale; no native width/precision synthesized"}),result_domain:json!({"nativeRepresentation":"DECIMAL(38,0) coefficients","maxScale":18,"carrier":"exact integer or scale-preserving decimal text"}),constraints:vec!["Exact-or-capability-error; all intermediate and scale-alignment products are guarded before filtering/cancellation".into()],obligations:vec![],status:Status::Candidate,evidence:vec![]});
        }
        Ok(manifest)
    }
    fn validate_binding(&self, context: &Context<'_>) -> Result<Validated<Binding>> {
        if !matches!(context.plan, Plan::V03(_)) {
            return Err(Diagnostic::new(
                "WFT-BACKEND-VERSION",
                "bind",
                "Arithmetic requires the explicit 0.3 plan",
            ));
        }
        Candidate.validate_binding(context)
    }
    fn assess(&self, context: &Context<'_>, binding: &Binding) -> Result<Vec<Assessment>> {
        Candidate.assess(context, binding)
    }
    fn lower(&self, context: &Context<'_>, binding: &Binding) -> Result<TargetPlan> {
        let Plan::V03(plan) = context.plan else {
            return Err(Diagnostic::new(
                "WFT-BACKEND-VERSION",
                "lower",
                "Arithmetic requires 0.3",
            ));
        };
        crate::candidate::lower_arithmetic(context, binding, plan)
    }
    fn emit(&self, context: &Context<'_>, plan: &TargetPlan) -> Result<Emission> {
        Candidate.emit(context, plan)
    }
}
