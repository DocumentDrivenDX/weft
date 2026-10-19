// SPDX-License-Identifier: MIT OR Apache-2.0
//! Original relational operand domain/presence correspondence. Not source, truth or authority admission.
use crate::{
    error::{Diagnostic, Result},
    json::{checked_json_bounded, sha256},
    security_backend::SecurityBackendContext,
    security_ir::{Disposition, Effect},
    security_ontology::{locate, SecurityOntologyClosure},
    security_requirements::SecurityOperatorMode,
    security_result_check::Budget,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
pub const VERSION: &str = "weft.security.operand-values/0.1-candidate";
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-OPERAND-VALUES",
        "result",
        "Original operand value correspondence refused",
    )
}
fn keys(v: &Value, names: &[&str]) -> Result<()> {
    let o = v.as_object().ok_or_else(fail)?;
    if o.len() != names.len() || names.iter().any(|n| !o.contains_key(*n)) {
        return Err(fail());
    }
    Ok(())
}
/// Private immutable pure check receipt, never a native execution credential.
/// ```compile_fail
/// use weft_core::security_operand_values::CheckedOperandValues;
/// fn forge(c:&mut CheckedOperandValues){c.counts.clear();}
/// ```
pub struct CheckedOperandValues {
    context_sha256: String,
    batch_sha256: String,
    counts: BTreeMap<String, usize>,
}
impl CheckedOperandValues {
    pub fn context_sha256(&self) -> &str {
        &self.context_sha256
    }
    pub fn batch_sha256(&self) -> &str {
        &self.batch_sha256
    }
    pub fn counts(&self) -> &BTreeMap<String, usize> {
        &self.counts
    }
}
/// Check every supplied operand at every original scan occurrence, against actual
/// original fields or constant-transform sources. Complete native population,
/// policy truth/selection, caller, source, cuts, codecs and release remain host obligations.
pub fn check_operand_values(
    ctx: &SecurityBackendContext<'_>,
    raw: &str,
) -> Result<CheckedOperandValues> {
    if raw.len() > 4_000_000 {
        return Err(fail());
    }
    ctx.profiled_query().require_sources(
        ctx.logical_plan(),
        ctx.catalog(),
        ctx.binding_json(),
        ctx.backend_id(),
        ctx.backend_version(),
        ctx.target_profile(),
    )?;
    let batch = checked_json_bounded(raw, 4_000_000, 64, 1_000_000).map_err(|_| fail())?;
    keys(&batch, &["version", "contextSha256", "scans"])?;
    let context = ctx.execution_query_context_sha256();
    if batch["version"] != VERSION || batch["contextSha256"] != context {
        return Err(fail());
    }
    let scans = batch["scans"].as_array().ok_or_else(fail)?;
    if scans.len() != ctx.requirements().scans().len() {
        return Err(fail());
    }
    let ontology = SecurityOntologyClosure::read(ctx.logical_plan().source(), ctx.catalog())?;
    let mut counts = BTreeMap::new();
    let mut budget = Budget::new();
    for scan in scans {
        keys(scan, &["scan", "rows"])?;
        let name = scan["scan"].as_str().ok_or_else(fail)?;
        budget.text(name)?;
        let original = ctx
            .requirements()
            .scans()
            .iter()
            .find(|s| s.inventory().scan() == name)
            .ok_or_else(fail)?;
        let expected: Vec<_> = ctx
            .requirements()
            .operators()
            .iter()
            .enumerate()
            .filter(|(_, o)| o.usage().scan == name)
            .collect();
        let rows = scan["rows"].as_array().ok_or_else(fail)?;
        if rows.len() > 4096 || counts.insert(name.into(), rows.len()).is_some() {
            return Err(fail());
        }
        let mut row_ids = BTreeSet::new();
        for row in rows {
            budget.charge(0)?;
            keys(row, &["nativeRowId", "operands"])?;
            let id = row["nativeRowId"].as_str().ok_or_else(fail)?;
            if id.is_empty()
                || id.len() > 19
                || id.starts_with('0')
                || !id.bytes().all(|c| c.is_ascii_digit())
                || id.parse::<i64>().is_err()
                || !row_ids.insert(id)
            {
                return Err(fail());
            }
            let values = row["operands"].as_array().ok_or_else(fail)?;
            if values.len() != expected.len() {
                return Err(fail());
            }
            for (provided, (index, requirement)) in values.iter().zip(&expected) {
                budget.charge(0)?;
                keys(provided, &["index", "cell"])?;
                if provided["index"].as_u64() != Some(*index as u64) {
                    return Err(fail());
                }
                let cell = &provided["cell"];
                let field = &requirement.usage().field;
                let source = locate(ctx.catalog(), field).map_err(|_| fail())?;
                let mut original_possible = matches!(
                    requirement.mode(),
                    SecurityOperatorMode::OriginalAuthorized(_)
                );
                if !original_possible {
                    let classification = ontology
                        .types
                        .get(original.inventory().target())
                        .ok_or_else(fail)?
                        .source["fields"]
                        .as_array()
                        .ok_or_else(fail)?
                        .iter()
                        .find(|f| f["ref"] == serde_json::to_value(field).unwrap())
                        .ok_or_else(fail)?;
                    original_possible = classification["protection"] == "unprotected"
                        || ctx.logical_plan().rules().iter().any(|r| {
                            r.target == *original.inventory().target()
                                && r.effect == Effect::Permit
                                && r.actions
                                    .iter()
                                    .any(|a| a == ctx.requirements().primary_action())
                                && r.disclosure
                                    .iter()
                                    .any(|(f, d)| f == field && matches!(d, Disposition::Original))
                        });
                }
                match cell["disposition"].as_str() {
                    Some("original") => {
                        keys(cell, &["disposition", "value"])?;
                        if !original_possible {
                            return Err(fail());
                        }
                        budget.literal(&cell["value"])?;
                        budget.normalize(source, &cell["value"])?;
                    }
                    Some("absent") => {
                        keys(cell, &["disposition"])?;
                        if !original_possible || source["nullability"] != "absent-allowed" {
                            return Err(fail());
                        }
                    }
                    Some("transformed") => {
                        if !matches!(requirement.mode(), SecurityOperatorMode::Disclosed) {
                            return Err(fail());
                        }
                        keys(cell, &["disposition", "value", "ruleId"])?;
                        let rule_id = cell["ruleId"].as_str().ok_or_else(fail)?;
                        budget.text(rule_id)?;
                        let rule = ctx
                            .logical_plan()
                            .rules()
                            .iter()
                            .find(|r| {
                                r.id == rule_id
                                    && r.target == *original.inventory().target()
                                    && r.effect == Effect::Permit
                                    && r.actions
                                        .iter()
                                        .any(|a| a == ctx.requirements().primary_action())
                            })
                            .ok_or_else(fail)?;
                        let disposition = rule
                            .disclosure
                            .iter()
                            .find(|(f, _)| f == field)
                            .ok_or_else(fail)?;
                        let Disposition::Transformed {
                            transform,
                            version,
                            output_field,
                            literal,
                            ..
                        } = &disposition.1
                        else {
                            return Err(fail());
                        };
                        if transform != "constant" || version != "0.1.0" {
                            return Err(fail());
                        }
                        let output = locate(ctx.catalog(), output_field).map_err(|_| fail())?;
                        budget.literal(&cell["value"])?;
                        budget.literal(literal)?;
                        if budget.normalize(output, &cell["value"])?
                            != budget.normalize(output, literal)?
                        {
                            return Err(fail());
                        }
                    }
                    _ => return Err(fail()),
                }
            }
        }
    }
    Ok(CheckedOperandValues {
        context_sha256: context,
        batch_sha256: sha256(raw.as_bytes()),
        counts,
    })
}
