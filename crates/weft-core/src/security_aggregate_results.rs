//! Separate aggregate result candidate. Pure correspondence, not aggregate computation or authority.
use crate::{
    application_ir::{Expression, Output},
    error::{Diagnostic, Result},
    ir::{Family, LogicalType},
    security_backend::SecurityBackendContext,
    security_lowering::{
        SecurityResultColumn, SecurityResultContract, SecurityResultDomain, SecurityResultOutcome,
    },
    security_result_check::Budget,
};
use serde_json::{json, Value};
pub const ENCODING: &str = "weft.security.cells/0.2-candidate";
pub const VERSION: &str = "weft.security.result-contract/0.2-candidate";
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-AGGREGATE-RESULT",
        "result",
        "Original aggregate result correspondence refused",
    )
}
fn require(ctx: &SecurityBackendContext<'_>) -> Result<()> {
    ctx.profiled_query().require_sources(
        ctx.logical_plan(),
        ctx.catalog(),
        ctx.binding_json(),
        ctx.backend_id(),
        ctx.backend_version(),
        ctx.target_profile(),
    )?;
    if !ctx.query().application_plan().aggregate
        || !ctx.requirements().outputs().iter().any(|o| {
            matches!(
                o.output().expression,
                Expression::Count { .. } | Expression::Sum { .. }
            )
        })
    {
        return Err(fail());
    }
    Ok(())
}
/// This separate version is refused by the original result and registration APIs.
/// Success checks source/shape/domain correspondence, never row selection, input
/// authorization, aggregate arithmetic or native release.
pub fn check_aggregate_result_declaration(
    ctx: &SecurityBackendContext<'_>,
    contract: &SecurityResultContract,
) -> Result<()> {
    require(ctx)?;
    crate::security_result_check::check_version(ctx, contract, VERSION)
}
pub fn check_aggregate_result_cells(
    ctx: &SecurityBackendContext<'_>,
    contract: &SecurityResultContract,
    contract_json: &str,
    contract_sha256: &str,
    batch_json: &str,
) -> Result<()> {
    require(ctx)?;
    crate::security_result_cells::check_version(
        ctx,
        contract,
        contract_json,
        contract_sha256,
        batch_json,
        VERSION,
    )
}
/// Derive aggregate result nullability from the actual original query and its
/// selected operand mode. Nullable/absent inputs or null constant disclosures
/// can make grouped SUM null even when a group contains admitted rows.
pub fn aggregate_result_type(
    ctx: &SecurityBackendContext<'_>,
    output: &Output,
) -> Result<LogicalType> {
    require(ctx)?;
    if !ctx
        .requirements()
        .outputs()
        .iter()
        .any(|o| std::ptr::eq(o.output(), output))
    {
        return Err(fail());
    }
    match &output.expression {
        Expression::Count { logical_type } => Ok(logical_type.clone()),
        Expression::Sum {
            argument,
            logical_type,
        } => {
            let field = crate::security_ontology::SecurityRef {
                document_id: argument.identity.document_id.clone(),
                module_id: argument.identity.module.clone(),
                element_id: argument.identity.element.clone(),
            };
            let source =
                crate::security_ontology::locate(ctx.catalog(), &field).map_err(|_| fail())?;
            let mut result = logical_type.clone();
            result.nullable |= source["nullability"] == "absent-allowed";
            let requirement = ctx
                .requirements()
                .operators()
                .iter()
                .find(|o| {
                    o.usage().scan == argument.scan
                        && o.usage().field == field
                        && o.usage().operator == crate::security_ir::QueryOperator::Aggregate
                })
                .ok_or_else(fail)?;
            if matches!(
                requirement.mode(),
                crate::security_requirements::SecurityOperatorMode::Disclosed
            ) {
                let scan = ctx
                    .requirements()
                    .scans()
                    .iter()
                    .find(|s| s.inventory().scan() == argument.scan)
                    .ok_or_else(fail)?;
                result.nullable|=ctx.logical_plan().rules().iter().any(|r|r.target==*scan.inventory().target()&&r.effect==crate::security_ir::Effect::Permit&&r.actions.iter().any(|a|a==ctx.requirements().primary_action())&&r.disclosure.iter().any(|(f,d)|f==&field&&matches!(d,crate::security_ir::Disposition::Transformed{literal,..} if literal.is_null())));
            }
            Ok(result)
        }
        _ => Err(fail()),
    }
}
pub(crate) fn check_column(
    ctx: &SecurityBackendContext<'_>,
    output: &Output,
    column: &SecurityResultColumn,
    budget: &mut Budget,
) -> Result<()> {
    let (_t, source) = match &output.expression {
        Expression::Count { logical_type } => (logical_type, None),
        Expression::Sum {
            argument,
            logical_type,
        } => (logical_type, Some(&argument.identity)),
        _ => return Err(fail()),
    };
    if column.source_fields.len() != usize::from(source.is_some()) || column.outcomes.len() != 1 {
        return Err(fail());
    }
    if let Some(source) = source {
        let field = &column.source_fields[0];
        for text in [
            &field.document_id,
            &field.revision,
            &field.module,
            &field.element,
        ] {
            budget.text(text)?;
        }
        if field.document_id != source.document_id
            || field.revision != source.revision
            || field.module != source.module
            || field.element != source.element
        {
            return Err(fail());
        }
        if !ctx
            .catalog()
            .pins()
            .iter()
            .any(|p| p.document_id == field.document_id && p.revision == field.revision)
        {
            return Err(fail());
        }
    }
    let SecurityResultOutcome::Computed {
        id,
        domain: SecurityResultDomain::Scalar { r#type: declared },
    } = &column.outcomes[0]
    else {
        return Err(fail());
    };
    budget.text(id)?;
    let expected = aggregate_result_type(ctx, output)?;
    if id != "computed" || declared != &expected {
        return Err(fail());
    }
    scalar_source(&output.expression, &expected)?;
    Ok(())
}
pub(crate) fn scalar_source(expression: &Expression, t: &LogicalType) -> Result<Value> {
    let mut facets = t.facets.clone();
    let family = match t.family {
        Family::Integer => "integer",
        Family::Decimal => "decimal",
        _ => return Err(fail()),
    };
    match expression {
        Expression::Count { .. } => {
            if family != "integer" || t.nullable || facets != json!({}) {
                return Err(fail());
            }
            facets = json!({"range":{"min":{"integerToken":"0"},"minInclusive":true}});
        }
        Expression::Sum { .. } => {
            if family == "integer" {
                if facets != json!({}) {
                    return Err(fail());
                }
            } else {
                let o = facets.as_object_mut().ok_or_else(fail)?;
                if o.len() != 1 || !o.contains_key("scale") {
                    return Err(fail());
                }
                // Synthetic normalization ceiling is the existing resource ledger, not an
                // inherited model precision or a public semantic bound on SUM.
                let scale = facets["scale"].as_u64().ok_or_else(fail)?;
                if scale > 4_000_000 {
                    return Err(fail());
                }
                facets["precision"] = json!(4_000_000);
            }
        }
        _ => return Err(fail()),
    }
    Ok(
        json!({"kind":"field","scalarType":family,"cardinality":"one","nullability":if t.nullable{"absent-allowed"}else{"required"},"facets":facets}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sum_decimal_keeps_exact_scale_without_inheriting_input_precision() {
        let t = LogicalType {
            family: Family::Decimal,
            facets: json!({"scale":2}),
            nullable: true,
        };
        let source = scalar_source(
            &Expression::Count {
                logical_type: t.clone(),
            },
            &t,
        );
        assert!(source.is_err());
        let expression = Expression::Sum {
            argument: crate::application_ir::Field {
                scan: "r".into(),
                identity: crate::ir::Identity {
                    document_id: "d".into(),
                    revision: "r1".into(),
                    module: "m".into(),
                    element: "x".into(),
                },
                logical_type: LogicalType {
                    family: Family::Decimal,
                    facets: json!({"precision":3,"scale":2}),
                    nullable: false,
                },
                span: crate::ir::Span { start: 0, end: 1 },
            },
            logical_type: t.clone(),
        };
        let source = scalar_source(&expression, &t).unwrap();
        let mut budget = Budget::new();
        for value in [
            json!({"decimalToken":"12345678901234567890.12"}),
            json!({"decimalToken":"-1.23"}),
            Value::Null,
        ] {
            budget.normalize(&source, &value).unwrap();
        }
        for value in [
            json!({"decimalToken":"1.234"}),
            json!({"integerToken":"123"}),
            json!({"decimalToken":"1e999999999999"}),
        ] {
            assert!(budget.normalize(&source, &value).is_err());
        }
    }
}
