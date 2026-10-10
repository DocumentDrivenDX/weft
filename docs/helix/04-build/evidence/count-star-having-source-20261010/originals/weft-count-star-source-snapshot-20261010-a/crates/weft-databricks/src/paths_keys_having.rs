//! Explicit 0.4.1 grouped row-count HAVING realization; old PathsKeys stays strict.
use crate::{binding::Binding, paths::PathTargetPlan, paths_keys::PathsKeys};
use weft_core::{
    backend03::*,
    error::{Diagnostic, Result},
};
pub const VERSION: &str = "0.4.1-count-star-having-candidate";
pub struct CountStarHaving;
impl Backend for CountStarHaving {
    type Mapping = Binding;
    type TargetPlan = PathTargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let mut m = PathsKeys.describe()?;
        m.backend_version = VERSION.into();
        let language = LanguageProfile {
            dialect_profile: "weft-sql/0.4.1".into(),
            ir_version: "weft-ir/0.4.1".into(),
        };
        m.language_profiles = vec![language.clone()];
        for c in &mut m.capabilities {
            c.language_profiles = vec![language.clone()];
        }
        let mut c = m
            .capabilities
            .iter()
            .find(|c| c.id == "aggregate.havingCountDistinctGreater")
            .expect("owned manifest capability")
            .clone();
        c.id = "aggregate.havingCountStarGreater".into();
        c.logical_domain = serde_json::json!({"subset":"explicitly projected COUNT(*) with nonempty required exact String groups; one HAVING > original nonnegative Integer literal"});
        c.result_domain = serde_json::json!({"count":"mathematicalInteger; complete pre-HAVING bag signed64 capacity or refusal","threshold":"exact nonnegative signed64 capacity; no logical width"});
        m.capabilities.push(c);
        Ok(m)
    }
    fn validate_binding(&self, c: &Context<'_>) -> Result<Validated<Binding>> {
        if c.plan.ir_version() != "weft-ir/0.4.1" {
            return Err(Diagnostic::new(
                "WFT-CAPABILITY",
                "capability",
                "Exact 0.4.1 row-count realization required",
            ));
        }
        PathsKeys.validate_binding(c)
    }
    fn assess(&self, c: &Context<'_>, b: &Binding) -> Result<Vec<Assessment>> {
        PathsKeys.assess(c, b)
    }
    fn lower(&self, c: &Context<'_>, b: &Binding) -> Result<PathTargetPlan> {
        PathsKeys.lower(c, b)
    }
    fn emit(&self, c: &Context<'_>, p: &PathTargetPlan) -> Result<Emission> {
        PathsKeys.emit(c, p)
    }
}
