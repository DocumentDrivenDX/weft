//! Explicit required-root String-key profile. Candidate metadata is not native evidence.
use crate::{
    binding::Binding,
    paths::{PathTargetPlan, Paths},
};
use serde_json::json;
use weft_core::{
    arithmetic_plan::Expression,
    backend03::*,
    error::{Diagnostic, Result},
    ir::Family,
};

pub const ID: &str = "ashlar.databricks.paths-keys";
pub const VERSION: &str = "0.4.0-paths-keys-candidate";
pub const PROFILE: &str = "spark4-delta4-paths-keys-candidate";
pub struct PathsKeys;
fn refusal(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-CAPABILITY", "capability", message)
}
impl Backend for PathsKeys {
    type Mapping = Binding;
    type TargetPlan = PathTargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let mut m = Paths.describe()?;
        m.backend_id = ID.into();
        m.backend_version = VERSION.into();
        m.target_profiles[0].id = PROFILE.into();
        m.target_profiles[0].session_settings["collectionOrdinalRepresentation"] =
            json!("decimal38 full-population TRY_SUM prefix");
        for cap in &mut m.capabilities {
            cap.target_profiles = vec![PROFILE.into()];
        }
        let mut cap = m.capabilities[0].clone();
        cap.id = "relationship.boundedKeys".into();
        cap.logical_domain = json!({"subset":"required-root nonaggregate authored forward/inverse edges; complete required String keys; no LEFT root or expansion mix"});
        cap.result_domain = json!({"carrier":"inherited closed relatedKeys items/truncated; parallel bag ordered complete String key then signed edge ID; full DECIMAL38 prefix or refusal"});
        m.capabilities.push(cap);
        Ok(m)
    }
    fn validate_binding(&self, c: &Context<'_>) -> Result<Validated<Binding>> {
        if c.target.id != PROFILE {
            return Err(refusal("Explicit paths-keys profile required"));
        }
        for o in c.plan.outputs() {
            if let ExpressionView::Legacy(Expression::RelatedKeys {
                scan, relationship, ..
            }) = o.expression
            {
                if c.plan.aggregate()
                    || !c.plan.groups().is_empty()
                    || c.plan.having().next().is_some()
                    || c.plan.expansion().is_some()
                    || c.plan.outer_join_scans().contains(scan)
                {
                    return Err(refusal(
                        "RelatedKeys requires nonaggregate required root without expansion",
                    ));
                }
                for (record_id, key) in [
                    (&relationship.from, &relationship.source_key),
                    (&relationship.to, &relationship.target_key),
                ] {
                    let record = c.catalog.record_by_identity(record_id)?;
                    for (id, ty) in key.fields.iter().zip(&key.types) {
                        let (_, graph) = c.catalog.member_descriptor_by_identity(&record, id)?;
                        if ty.family != Family::String
                            || ty.nullable
                            || ty.facets != json!({})
                            || !graph.iter().any(|d| {
                                &d.identity == id && d.availability.as_deref() == Some("required")
                            })
                        {
                            return Err(refusal(
                                "RelatedKeys requires complete required exact String keys",
                            ));
                        }
                    }
                }
            }
        }
        crate::paths::validate_sources(c)
    }
    fn assess(&self, c: &Context<'_>, b: &Binding) -> Result<Vec<Assessment>> {
        Paths.assess(c, b)
    }
    fn lower(&self, c: &Context<'_>, b: &Binding) -> Result<PathTargetPlan> {
        crate::candidate::path_lowering::lower_keys(c, b)
    }
    fn emit(&self, c: &Context<'_>, p: &PathTargetPlan) -> Result<Emission> {
        Paths.emit(c, p)
    }
}
