//! Original owner parameter provenance. Inventories are declarations, never credentials.
use crate::{
    application_ir::{ExactValue, Predicate, Value as QueryValue},
    error::{Diagnostic, Result},
    security_backend::SecurityBackendContext,
    security_ir::{Disposition, Expression, Term},
    security_ontology::{authored_key_fields, SecurityOntologyClosure, SecurityRef},
};
use serde_json::Value;

fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-PARAMETER-SOURCE",
        "capability",
        "Original parameter source inventory refused",
    )
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityPolicyParameterKind {
    ConditionConstant,
    Context,
    TransformationConstant,
}
/// Borrows the actual admitted policy operand. No constructor or deserializer.
pub struct SecurityPolicyParameterSource<'a> {
    path: String,
    kind: SecurityPolicyParameterKind,
    field: &'a SecurityRef,
    domain: &'a Value,
    literal: Option<&'a Value>,
}
impl SecurityPolicyParameterSource<'_> {
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn kind(&self) -> SecurityPolicyParameterKind {
        self.kind
    }
    pub fn field(&self) -> &SecurityRef {
        self.field
    }
    pub fn domain(&self) -> &Value {
        self.domain
    }
    /// Context operands have no compiler-supplied value. Null constants remain Some(Null).
    pub fn literal(&self) -> Option<&Value> {
        self.literal
    }
}
/// The actual original SQL value family is retained, including exact scalar values.
pub enum SecurityQueryParameterValue<'a> {
    Scalar(&'a QueryValue),
    Exact(&'a ExactValue),
}
pub struct SecurityQueryParameterSource<'a> {
    path: String,
    value: SecurityQueryParameterValue<'a>,
}
impl<'a> SecurityQueryParameterSource<'a> {
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn value(&self) -> &SecurityQueryParameterValue<'a> {
        &self.value
    }
}
/// Logical subject/key demand only. It contains no principal or native locator value.
pub struct SecuritySubjectParameterSource {
    subject: SecurityRef,
    key_id: String,
    key_fields: Vec<SecurityRef>,
}
impl SecuritySubjectParameterSource {
    pub fn subject(&self) -> &SecurityRef {
        &self.subject
    }
    pub fn key_id(&self) -> &str {
        &self.key_id
    }
    pub fn key_fields(&self) -> &[SecurityRef] {
        &self.key_fields
    }
}
/// Inventory tied to actual owner objects, rather than caller source labels.
/// It does not grant permission to evaluate a policy or execute/release a query.
/// ```compile_fail
/// use weft_core::security_parameter_sources::SecurityParameterSources;
/// fn forge(s: &mut SecurityParameterSources<'_, '_>) { s.policy.clear(); }
/// ```
pub struct SecurityParameterSources<'a, 'owner> {
    context: &'a SecurityBackendContext<'owner>,
    query: Vec<SecurityQueryParameterSource<'a>>,
    policy: Vec<SecurityPolicyParameterSource<'a>>,
    subject: SecuritySubjectParameterSource,
}
impl<'a, 'owner> SecurityParameterSources<'a, 'owner> {
    pub fn query_values(&self) -> &[SecurityQueryParameterSource<'a>] {
        &self.query
    }
    pub fn policy_operands(&self) -> &[SecurityPolicyParameterSource<'a>] {
        &self.policy
    }
    pub fn subject(&self) -> &SecuritySubjectParameterSource {
        &self.subject
    }
    /// Original object correspondence only; no digest is used as a credential.
    pub fn require_context(&self, context: &SecurityBackendContext<'_>) -> Result<()> {
        if !std::ptr::eq(self.context.catalog(), context.catalog())
            || !std::ptr::eq(self.context.logical_plan(), context.logical_plan())
            || !std::ptr::eq(self.context.query(), context.query())
            || !std::ptr::eq(self.context.profiled_query(), context.profiled_query())
            || self.context.binding_json() != context.binding_json()
            || self.context.backend_id() != context.backend_id()
            || self.context.backend_version() != context.backend_version()
            || self.context.target_profile() != context.target_profile()
        {
            return Err(fail());
        }
        context.profiled_query().require_sources(
            context.logical_plan(),
            context.catalog(),
            context.binding_json(),
            context.backend_id(),
            context.backend_version(),
            context.target_profile(),
        )
    }
}
struct Budget {
    work: usize,
    text: usize,
    sources: usize,
}
impl Budget {
    fn step(&mut self, path: &str) -> Result<()> {
        if self.work == 0 || path.len() > 4096 || path.len() > self.text {
            return Err(fail());
        }
        self.work -= 1;
        self.text -= path.len();
        Ok(())
    }
    fn source(&mut self) -> Result<()> {
        if self.sources == 0 {
            return Err(fail());
        }
        self.sources -= 1;
        Ok(())
    }
}
pub(crate) fn issue<'a, 'owner>(
    context: &'a SecurityBackendContext<'owner>,
) -> Result<SecurityParameterSources<'a, 'owner>> {
    issue_with_budget(
        context,
        Budget {
            work: 65536,
            text: 1048576,
            sources: 4096,
        },
    )
}
fn issue_with_budget<'a, 'owner>(
    context: &'a SecurityBackendContext<'owner>,
    mut budget: Budget,
) -> Result<SecurityParameterSources<'a, 'owner>> {
    context.profiled_query().require_sources(
        context.logical_plan(),
        context.catalog(),
        context.binding_json(),
        context.backend_id(),
        context.backend_version(),
        context.target_profile(),
    )?;
    let mut policy = Vec::new();
    fn operand<'a>(
        term: &'a Term,
        path: String,
        policy: &mut Vec<SecurityPolicyParameterSource<'a>>,
        budget: &mut Budget,
    ) -> Result<()> {
        budget.step(&path)?;
        let (kind, field, domain, literal) = match term {
            Term::Constant {
                field,
                domain,
                literal,
            } => (
                SecurityPolicyParameterKind::ConditionConstant,
                field,
                domain,
                Some(literal),
            ),
            Term::Context { field, domain } => {
                (SecurityPolicyParameterKind::Context, field, domain, None)
            }
            _ => return Ok(()),
        };
        budget.source()?;
        policy.push(SecurityPolicyParameterSource {
            path,
            kind,
            field,
            domain,
            literal,
        });
        Ok(())
    }
    for (i, rule) in context.logical_plan().rules().iter().enumerate() {
        fn push<'b>(
            stack: &mut Vec<(String, &'b Expression)>,
            path: String,
            expression: &'b Expression,
            budget: &mut Budget,
        ) -> Result<()> {
            budget.step(&path)?;
            stack.push((path, expression));
            Ok(())
        }
        let mut stack = Vec::new();
        push(
            &mut stack,
            format!("logicalRules/{i}/condition"),
            &rule.condition,
            &mut budget,
        )?;
        while let Some((path, expression)) = stack.pop() {
            match expression {
                Expression::Equal(left, right) => {
                    operand(left, format!("{path}/left"), &mut policy, &mut budget)?;
                    operand(right, format!("{path}/right"), &mut policy, &mut budget)?;
                }
                Expression::And(args) | Expression::Or(args) => {
                    if args.len() > budget.work {
                        return Err(fail());
                    }
                    for (j, arg) in args.iter().enumerate().rev() {
                        push(&mut stack, format!("{path}/args/{j}"), arg, &mut budget)?;
                    }
                }
                Expression::Not(arg) => push(&mut stack, format!("{path}/arg"), arg, &mut budget)?,
                Expression::Exists { condition, .. } => {
                    push(&mut stack, format!("{path}/where"), condition, &mut budget)?
                }
                Expression::Literal(_) => {}
            }
        }
        for (j, (_, disposition)) in rule.disclosure.iter().enumerate() {
            let path = format!("logicalRules/{i}/disclosure/{j}/transformed");
            budget.step(&path)?;
            if let Disposition::Transformed {
                output_field,
                domain,
                literal,
                ..
            } = disposition
            {
                budget.source()?;
                policy.push(SecurityPolicyParameterSource {
                    path,
                    kind: SecurityPolicyParameterKind::TransformationConstant,
                    field: output_field,
                    domain,
                    literal: Some(literal),
                });
            }
        }
    }
    let mut query = Vec::new();
    fn scalar<'a>(
        value: &'a QueryValue,
        path: String,
        query: &mut Vec<SecurityQueryParameterSource<'a>>,
        budget: &mut Budget,
    ) -> Result<()> {
        budget.step(&path)?;
        if !matches!(value, QueryValue::Field { .. }) {
            budget.source()?;
            query.push(SecurityQueryParameterSource {
                path,
                value: SecurityQueryParameterValue::Scalar(value),
            });
        }
        Ok(())
    }
    fn predicate<'a>(
        p: &'a Predicate,
        path: String,
        query: &mut Vec<SecurityQueryParameterSource<'a>>,
        budget: &mut Budget,
    ) -> Result<()> {
        budget.step(&path)?;
        match p {
            Predicate::Boolean{..}=>return Err(fail()),
            Predicate::ExactEqual { right, .. } => {
                let path = format!("{path}/right");
                budget.step(&path)?;
                if !matches!(right, ExactValue::Field { .. }) {
                    budget.source()?;
                    query.push(SecurityQueryParameterSource {
                        path,
                        value: SecurityQueryParameterValue::Exact(right),
                    });
                }
            }
            Predicate::Equal { right, .. } => {
                scalar(right, format!("{path}/right"), query, budget)?
            }
            Predicate::LexicographicGreater { values, .. } => {
                for (j, value) in values.iter().enumerate() {
                    scalar(value, format!("{path}/values/{j}"), query, budget)?;
                }
            }
            Predicate::HasRelated { key, .. } => {
                for (j, value) in key.iter().enumerate() {
                    scalar(value, format!("{path}/key/{j}"), query, budget)?;
                }
            }
        }
        Ok(())
    }
    let app = context.query().application_plan();
    for (i, join) in app.joins.iter().enumerate() {
        for (j, p) in join.on.iter().enumerate() {
            predicate(p, format!("joins/{i}/on/{j}"), &mut query, &mut budget)?;
        }
    }
    for (i, p) in app.filters.iter().enumerate() {
        predicate(p, format!("filters/{i}"), &mut query, &mut budget)?;
    }
    let ontology =
        SecurityOntologyClosure::read(context.logical_plan().source(), context.catalog())?;
    let subject = ontology.subject().clone();
    let key_id = ontology.types.get(&subject).ok_or_else(fail)?.source["keyId"]
        .as_str()
        .ok_or_else(fail)?
        .to_owned();
    let key_fields = authored_key_fields(context.catalog(), &subject, &key_id)?;
    if key_fields.len() > 64 {
        return Err(fail());
    }
    Ok(SecurityParameterSources {
        context,
        policy,
        query,
        subject: SecuritySubjectParameterSource {
            subject,
            key_id,
            key_fields,
        },
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_inventory_refuses_exhausted_ledgers_without_returning_partial_sources() {
        let raw = include_str!(
            "../../../docs/helix/03-test/fixtures/security-graph-source/mapping-request.json"
        );
        crate::security_mapping_transport::with_security_mapping_context(raw, |context| {
            for budget in [
                Budget {
                    work: 0,
                    text: 1048576,
                    sources: 4096,
                },
                Budget {
                    work: 65536,
                    text: 0,
                    sources: 4096,
                },
                Budget {
                    work: 65536,
                    text: 1048576,
                    sources: 0,
                },
            ] {
                assert!(issue_with_budget(context, budget).is_err());
            }
            assert!(issue_with_budget(
                context,
                Budget {
                    work: 65536,
                    text: 1048576,
                    sources: 1
                }
            )
            .is_ok());
            Ok(())
        })
        .unwrap();
    }
    #[test]
    fn budget_refuses_before_underflow() {
        let mut b = Budget {
            work: 1,
            text: 2,
            sources: 1,
        };
        assert!(b.step("abc").is_err());
        assert_eq!((b.work, b.text), (1, 2));
        b.step("ab").unwrap();
        assert!(b.step("").is_err());
        b.source().unwrap();
        assert!(b.source().is_err());
    }
}
