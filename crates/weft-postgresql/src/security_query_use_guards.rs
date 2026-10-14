//! Original per-scan query-use authorization/disclosure guards. Not relational lowering or native admission.
use crate::security_graph_binding::GraphSource;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    error::{Diagnostic, Result},
    security_backend::SecurityBackendContext,
    security_ir::QueryOperator,
    security_ontology::SecurityRef,
    security_requirements::SecurityOperatorMode,
};
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-QUERY-USE-GUARD",
        "capability",
        "Original query-use guard correspondence refused",
    )
}
/// Rule expressions must be evaluated in this scan's private relation before
/// using its decision/field guard; q aliases are local to that scan occurrence.
pub struct QueryUseRuleTruth {
    column: String,
    expression: String,
}
impl QueryUseRuleTruth {
    pub fn column(&self) -> &str {
        &self.column
    }
    pub fn expression(&self) -> &str {
        &self.expression
    }
}
pub struct QueryUseOccurrence {
    field: SecurityRef,
    operator: QueryOperator,
    original_action: Option<String>,
}
impl QueryUseOccurrence {
    pub fn field(&self) -> &SecurityRef {
        &self.field
    }
    pub fn operator(&self) -> QueryOperator {
        self.operator
    }
    pub fn original_action(&self) -> Option<&str> {
        self.original_action.as_deref()
    }
}
/// All actual operators on one original scan, before filters/order/group/join/
/// aggregates/limit. This guard does not validate native scalar values or cuts.
pub struct ScanQueryUseGuard {
    scan: String,
    target: SecurityRef,
    actions: Vec<String>,
    truths: Vec<QueryUseRuleTruth>,
    refusal_expression: String,
    uses: Vec<QueryUseOccurrence>,
}
impl ScanQueryUseGuard {
    pub fn scan(&self) -> &str {
        &self.scan
    }
    pub fn target(&self) -> &SecurityRef {
        &self.target
    }
    pub fn actions(&self) -> &[String] {
        &self.actions
    }
    pub fn rule_truths(&self) -> &[QueryUseRuleTruth] {
        &self.truths
    }
    pub fn refusal_expression(&self) -> &str {
        &self.refusal_expression
    }
    pub fn uses(&self) -> &[QueryUseOccurrence] {
        &self.uses
    }
}
/// Private-constructor owner guards, not caller annotations or credentials.
/// ```compile_fail
/// use weft_postgresql::security_query_use_guards::GraphQueryUseGuards;
/// fn omit(g: &mut GraphQueryUseGuards) { g.scans.clear(); }
/// ```
pub struct GraphQueryUseGuards {
    context_sha256: String,
    scans: Vec<ScanQueryUseGuard>,
}
impl GraphQueryUseGuards {
    pub fn context_sha256(&self) -> &str {
        &self.context_sha256
    }
    pub fn scans(&self) -> &[ScanQueryUseGuard] {
        &self.scans
    }
}
struct Budget {
    work: usize,
    text: usize,
}
impl Budget {
    fn step(&mut self) -> Result<()> {
        if self.work == 0 {
            return Err(fail());
        }
        self.work -= 1;
        Ok(())
    }
    fn text(&mut self, s: &str) -> Result<()> {
        self.step()?;
        if s.len() > self.text {
            return Err(fail());
        }
        self.text -= s.len();
        Ok(())
    }
}
/// Derive guards for every actual owner scan/operator, including query-only
/// fields and designated original-action groups. This does not admit query SQL.
pub fn prepare_graph_query_use_guards(
    ctx: &SecurityBackendContext<'_>,
    source: &GraphSource,
) -> Result<GraphQueryUseGuards> {
    let binding = weft_core::json::checked_json(ctx.binding_json()).map_err(|_| fail())?;
    if binding != serde_json::to_value(source).map_err(|_| fail())? {
        return Err(fail());
    }
    let graph =
        crate::security_predicate_spike::render_graph_context(ctx,source)?.value;
    derive(
        ctx,
        &graph,
        Budget {
            work: 100_000,
            text: 1_048_576,
        },
    )
}
fn derive(
    ctx: &SecurityBackendContext<'_>,
    graph: &Value,
    mut budget: Budget,
) -> Result<GraphQueryUseGuards> {
    ctx.profiled_query().require_sources(
        ctx.logical_plan(),
        ctx.catalog(),
        ctx.binding_json(),
        ctx.backend_id(),
        ctx.backend_version(),
        ctx.target_profile(),
    )?;
    let mut groups = BTreeMap::new();
    for group in graph["eligibility"].as_array().ok_or_else(fail)? {
        budget.step()?;
        let target: SecurityRef =
            serde_json::from_value(group["target"].clone()).map_err(|_| fail())?;
        let action = group["action"].as_str().ok_or_else(fail)?;
        if groups.insert((target, action), group).is_some() {
            return Err(fail());
        }
    }
    let rules = graph["rules"].as_array().ok_or_else(fail)?;
    let mut scans = Vec::new();
    for scan in ctx.requirements().scans() {
        budget.step()?;
        let inventory = scan.inventory();
        let primary = groups
            .get(&(
                inventory.target().clone(),
                ctx.requirements().primary_action(),
            ))
            .ok_or_else(fail)?;
        let primary_decision = primary["decisionExpression"].as_str().ok_or_else(fail)?;
        let mut actions = Vec::new();
        for action in scan.actions() {
            budget.text(action.inventory().action())?;
            actions.push(action.inventory().action().to_owned());
        }
        for name in [
            &inventory.target().document_id,
            &inventory.target().module_id,
            &inventory.target().element_id,
        ] {
            budget.text(name)?;
        }
        let rule_ids: BTreeSet<_> = scan
            .actions()
            .iter()
            .flat_map(|a| a.inventory().rule_ids().iter().map(String::as_str))
            .collect();
        let mut truths = Vec::new();
        let mut found = BTreeSet::new();
        for rule in rules {
            budget.step()?;
            let id = rule["id"].as_str().ok_or_else(fail)?;
            if !rule_ids.contains(id)
                || rule["target"] != serde_json::to_value(inventory.target()).map_err(|_| fail())?
            {
                continue;
            }
            let column = rule["truthColumn"].as_str().ok_or_else(fail)?;
            let expression = rule["expression"].as_str().ok_or_else(fail)?;
            budget.text(column)?;
            budget.text(expression)?;
            truths.push(QueryUseRuleTruth {
                column: column.into(),
                expression: expression.into(),
            });
            found.insert(id);
        }
        if found.len() != rule_ids.len() {
            return Err(fail());
        }
        let mut refused = Vec::new();
        let mut uses = Vec::new();
        for requirement in ctx.requirements().operators() {
            budget.step()?;
            let usage = requirement.usage();
            if usage.scan != inventory.scan() {
                continue;
            }
            let original_action = match requirement.mode() {
                SecurityOperatorMode::OriginalAuthorized(action) => {
                    let denial = match groups.get(&(inventory.target().clone(), action)) {
                        Some(group) => format!(
                            "({}) IS DISTINCT FROM 'permit'",
                            group["decisionExpression"].as_str().ok_or_else(fail)?
                        ),
                        None => "TRUE".into(),
                    };
                    budget.text(&denial)?;
                    refused.push(denial);
                    budget.text(action)?;
                    Some(action.into())
                }
                SecurityOperatorMode::Disclosed => {
                    let mut matching = None;
                    for field in primary["disclosure"].as_array().ok_or_else(fail)? {
                        budget.step()?;
                        if field["field"]
                            == serde_json::to_value(&usage.field).map_err(|_| fail())?
                        {
                            if matching.replace(field).is_some() {
                                return Err(fail());
                            }
                        }
                    }
                    let cell = matching.ok_or_else(fail)?["cellExpression"]
                        .as_str()
                        .ok_or_else(fail)?;
                    // The original composed disclosed operand may be an original value,
                    // constant replacement, or known absence. Withholding/conflict/refusal is
                    // never translated into a predicate-false row or ordered NULL value.
                    let denial = format!(
                        "((( {}) ->> 'disposition') IN ('original','transformed','absent')) IS NOT TRUE",
                        cell
                    );
                    budget.text(&denial)?;
                    refused.push(denial);
                    None
                }
            };
            for name in [
                &usage.field.document_id,
                &usage.field.module_id,
                &usage.field.element_id,
            ] {
                budget.text(name)?;
            }
            uses.push(QueryUseOccurrence {
                field: usage.field.clone(),
                operator: usage.operator,
                original_action,
            });
        }
        let operand_denial = if refused.is_empty() {
            "FALSE".into()
        } else {
            refused.join(" OR ")
        };
        let refusal_expression=format!("CASE WHEN ({primary_decision})='indeterminate' THEN TRUE WHEN ({primary_decision})='permit' THEN ({operand_denial}) ELSE FALSE END");
        budget.text(&refusal_expression)?;
        budget.text(inventory.scan())?;
        scans.push(ScanQueryUseGuard {
            scan: inventory.scan().into(),
            target: inventory.target().clone(),
            actions,
            truths,
            refusal_expression,
            uses,
        });
    }
    Ok(GraphQueryUseGuards {
        context_sha256: ctx.execution_query_context_sha256(),
        scans,
    })
}
