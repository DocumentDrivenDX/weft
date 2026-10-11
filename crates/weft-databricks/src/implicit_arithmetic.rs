//! Explicit 0.4.2 source-expression labels; existing profiles stay strict.
use crate::{
    binding::Binding, paths::PathTargetPlan, paths_keys::PathsKeys,
    paths_keys_having::CountStarHaving,
};
use weft_core::{
    backend03::*,
    error::{Diagnostic, Result},
};
pub const VERSION: &str = "0.4.2-implicit-arithmetic-labels-candidate";
pub struct ImplicitArithmeticLabels;
impl Backend for ImplicitArithmeticLabels {
    type Mapping = Binding;
    type TargetPlan = PathTargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let mut m = CountStarHaving.describe()?;
        m.backend_version = VERSION.into();
        let language = LanguageProfile {
            dialect_profile: "weft-sql/0.4.2".into(),
            ir_version: "weft-ir/0.4.2".into(),
        };
        m.language_profiles = vec![language.clone()];
        for c in &mut m.capabilities {
            c.language_profiles = vec![language.clone()];
        }
        let mut c = m
            .capabilities
            .iter()
            .find(|c| c.id == "project")
            .ok_or_else(|| {
                Diagnostic::new(
                    "WFT-BACKEND-VERSION",
                    "capability",
                    "Owned projection capability missing",
                )
            })?
            .clone();
        c.id = "project.implicitArithmeticLabels".into();
        c.logical_domain = serde_json::json!({"subset":"unnamed exact arithmetic expression source-span labels; existing collision and positioned output rules"});
        c.result_domain = serde_json::json!({"naming":"logical source substring; distinct exact ordered physical carrier mapping"});
        m.capabilities.push(c);
        Ok(m)
    }
    fn validate_binding(&self, c: &Context<'_>) -> Result<Validated<Binding>> {
        if c.plan.ir_version() != "weft-ir/0.4.2" {
            return Err(Diagnostic::new(
                "WFT-CAPABILITY",
                "capability",
                "Exact 0.4.2 implicit arithmetic label profile required",
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
