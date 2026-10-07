//! Original-profile SELECT assembly; native operator meaning stays backend-owned.
use crate::{
    registered_access::{Access, Prepared},
    Parameters,
};
use std::collections::BTreeMap;
use weft_core::{
    backend::{Column, Context, Plan, Representation},
    error::{Diagnostic, Result},
    ir::{Expression, Node},
};
#[derive(Debug)]
pub struct Select {
    pub sql: String,
    pub columns: Vec<Column>,
    pub structural_checks: Vec<String>,
    pub payload_checks: Vec<crate::result_definition::ReadPayloadObservation>,
}
/// Complete original-profile lowering output. Execution stays in the host.
#[derive(Debug)]
pub struct Compilation {
    pub select: Select,
    pub parameters: Vec<weft_core::backend::ParameterSlot>,
}
/// Assemble from exact registered original definitions in one operation. No
/// caller-owned parameter state or detached preparation can escape on failure.
pub fn compile_with_registry<'a>(
    context: &Context<'_>,
    records: &BTreeMap<String, crate::record_definition::RecordAdmission>,
    properties: &'a BTreeMap<String, crate::property_definition::PropertyAdmission>,
    comparators: &BTreeMap<String, crate::native_comparator_definition::Definition>,
    mut native: impl FnMut(
        &Expression,
        &[String],
        Option<&Access<'a>>,
        &mut Parameters,
    ) -> Result<String>,
) -> Result<Compilation> {
    let mut parameters = Parameters::default();
    let prepared = crate::registered_access::prepare(
        context,
        records,
        properties,
        comparators,
        &mut parameters,
    )?;
    let mut select = assemble(
        context,
        &prepared,
        properties,
        comparators,
        &mut parameters,
        |node, operands, access, parameters| {
            if let Expression::Sum { argument, .. } = node {
                let Expression::Field {
                    scan,
                    identity,
                    logical_type,
                    ..
                } = argument.as_ref()
                else {
                    return Err(Diagnostic::new(
                        "WFT-CAPABILITY",
                        "lower",
                        "SUM requires its admitted original field argument",
                    ));
                };
                let argument_access = prepared
                    .accesses
                    .iter()
                    .find(|access| &access.scan == scan && &access.field == identity)
                    .ok_or_else(|| {
                        Diagnostic::new(
                            "WFT-BINDING",
                            "lower",
                            "SUM argument lacks its exact prepared scan access",
                        )
                    })?;
                let key = crate::comparator_requirements::registration_key(
                    &argument_access.owner,
                    identity,
                );
                let property = properties.get(&key).ok_or_else(|| {
                    Diagnostic::new(
                        "WFT-BINDING",
                        "lower",
                        "SUM lacks its original owned property",
                    )
                })?;
                argument_access.verify_property(property)?;
                let comparator = comparators.get(&key).ok_or_else(|| {
                    Diagnostic::new(
                        "WFT-BINDING",
                        "lower",
                        "SUM lacks its selected original comparator",
                    )
                })?;
                comparator.require_type(logical_type)?;
                if operands.len() != 1 {
                    return Err(Diagnostic::new(
                        "WFT-BINDING",
                        "lower",
                        "SUM operand arity differs",
                    ));
                }
                comparator.sum_sql(&operands[0])
            } else {
                native(node, operands, access, parameters)
            }
        },
    )?;
    for requirement in crate::comparator_requirements::collect(context.plan)? {
        if !requirement
            .operations
            .contains(&crate::native_comparator_definition::Operation::Sum)
        {
            continue;
        }
        let key = crate::comparator_requirements::registration_key(
            &requirement.owner,
            &requirement.identity,
        );
        let comparator = comparators.get(&key).ok_or_else(|| {
            Diagnostic::new("WFT-BINDING", "lower", "SUM domain comparator missing")
        })?;
        for access in prepared.accesses.iter().filter(|access| {
            access.owner == requirement.owner && access.field == requirement.identity
        }) {
            let carrier = match &access.location {
                crate::registered_access::Location::Props(_) => access
                    .scalar_storage
                    .as_ref()
                    .ok_or_else(|| {
                        Diagnostic::new("WFT-BINDING", "lower", "SUM scalar carrier missing")
                    })?
                    .carrier
                    .clone(),
                crate::registered_access::Location::Row(location) => {
                    location.scalar_observation().native_numeric_text
                }
            };
            let integrity = comparator.numeric_domain_sql(&carrier)?;
            let scan = prepared
                .scans
                .get(&access.scan)
                .ok_or_else(|| Diagnostic::new("WFT-BINDING", "lower", "SUM owner scan missing"))?;
            let mut check = select
                .payload_checks
                .iter()
                .find(|check| check.scan == access.scan && check.field == access.field)
                .ok_or_else(|| {
                    Diagnostic::new(
                        "WFT-BINDING",
                        "lower",
                        "SUM original payload prerequisite missing",
                    )
                })?
                .clone();
            check.sql = format!("SELECT count(*) AS violations FROM {} WHERE {} AND ({integrity}) IS DISTINCT FROM TRUE",scan.source.sql,scan.source.filters.join(" AND "));
            select.payload_checks.push(check);
        }
    }
    Ok(Compilation {
        select,
        parameters: parameters.into_slots(),
    })
}
/// Parameters commit only after source, projection and prerequisite assembly.
/// Returned checks must run under the selected host contract before publication.
/// This does not qualify a native operator callback or a source/codec procedure.
pub fn assemble<'a>(
    context: &Context<'_>,
    prepared: &Prepared<'a>,
    properties: &BTreeMap<String, crate::property_definition::PropertyAdmission>,
    comparators: &BTreeMap<String, crate::native_comparator_definition::Definition>,
    parameters: &mut Parameters,
    mut native: impl FnMut(
        &Expression,
        &[String],
        Option<&Access<'a>>,
        &mut Parameters,
    ) -> Result<String>,
) -> Result<Select> {
    prepared.verify_context(context, parameters)?;
    let fail = |message: &str| Diagnostic::new("WFT-CAPABILITY", "emit", message);
    let Plan::V01(plan) = context.plan else {
        return assemble_application(
            context,
            prepared,
            properties,
            comparators,
            parameters,
            &mut native,
        );
    };
    let Node::Project { input, outputs, .. } = &plan.root else {
        return Err(fail("SELECT requires its outer projection"));
    };
    let columns = crate::result_definition::projection_columns(context, properties, comparators)?;
    let payload_checks = crate::result_definition::read_payload_observations(prepared, properties)?;
    let mut staged = parameters.clone();
    let mut used = std::collections::BTreeSet::new();
    let source = crate::relational::assemble_sources(
        input,
        &mut staged,
        |node, _| {
            let Node::Scan { occurrence, .. } = node else {
                unreachable!()
            };
            used.insert(occurrence.clone());
            prepared
                .scans
                .get(occurrence)
                .map(|scan| scan.source.clone())
                .ok_or_else(|| fail("SELECT scan lacks prepared original source"))
        },
        |expression, parameters| {
            crate::registered_access::render_expression(
                expression,
                &prepared.accesses,
                parameters,
                &mut native,
            )
        },
    )?;
    if used.len() != prepared.scans.len() {
        return Err(fail("Prepared sources do not match SELECT scan inventory"));
    }
    let mut projections = Vec::new();
    for (output, column) in outputs.iter().zip(&columns) {
        if let Expression::Field { scan, identity, .. } = &output.expression {
            let access = prepared
                .accesses
                .iter()
                .find(|access| &access.scan == scan && &access.field == identity)
                .ok_or_else(|| fail("Projected field lacks exact prepared access"))?;
            let key = crate::comparator_requirements::registration_key(&access.owner, identity);
            let property = properties
                .get(&key)
                .ok_or_else(|| fail("Projected field lacks original property admission"))?;
            let projection = crate::result_definition::property_projection(
                property,
                access,
                column.position,
                &column.output_name,
            )?;
            if serde_json::to_value(&projection.column)
                .map_err(|_| fail("Projection metadata encoding refused"))?
                != serde_json::to_value(column)
                    .map_err(|_| fail("Projection metadata encoding refused"))?
            {
                return Err(fail("Projected codec and output metadata differ"));
            }
            projections.push(projection.sql);
            continue;
        }
        if !matches!(column.representation, Representation::Scalar { .. }) {
            return Err(fail(
                "SELECT value requires its selected recursive result bridge",
            ));
        }
        let expression = crate::registered_access::render_expression(
            &output.expression,
            &prepared.accesses,
            &mut staged,
            &mut native,
        )?;
        projections.push(format!(
            "({expression})::pg_catalog.text AS {}",
            crate::Identifier::new(&column.output_name)?.sql()
        ));
    }
    if outputs.len() != columns.len() {
        return Err(fail("SELECT output metadata arity mismatch"));
    }
    let mut sql = format!("SELECT {} FROM {}", projections.join(", "), source.sql);
    if !source.filters.is_empty() {
        sql.push_str(&format!(" WHERE {}", source.filters.join(" AND ")));
    }
    if !source.groups.is_empty() {
        sql.push_str(&format!(" GROUP BY {}", source.groups.join(", ")));
    }
    let structural_checks = prepared
        .scans
        .values()
        .flat_map(|scan| scan.structural_check_sql.iter().cloned())
        .collect();
    *parameters = staged;
    Ok(Select {
        sql,
        columns,
        structural_checks,
        payload_checks,
    })
}

/// Original-definition V02 field projections and string-grouped COUNT. Other stages require
/// their selected lowering procedures and must never be silently discarded.
fn assemble_application<'a>(
    context: &Context<'_>,
    prepared: &Prepared<'a>,
    properties: &BTreeMap<String, crate::property_definition::PropertyAdmission>,
    comparators: &BTreeMap<String, crate::native_comparator_definition::Definition>,
    parameters: &mut Parameters,
    native: &mut impl FnMut(
        &Expression,
        &[String],
        Option<&Access<'a>>,
        &mut Parameters,
    ) -> Result<String>,
) -> Result<Select> {
    use weft_core::application_ir as app;
    let fail = |message: &str| Diagnostic::new("WFT-CAPABILITY", "emit", message);
    let Plan::V02(plan) = context.plan else {
        unreachable!()
    };
    if plan.limit.is_some_and(|limit| !(1..=1000).contains(&limit)) {
        return Err(fail("Application LIMIT is outside its resolved bounds"));
    }
    let has_aggregate = plan.outputs.iter().any(|output| {
        matches!(
            output.expression,
            app::Expression::Count { .. } | app::Expression::Sum { .. }
        )
    });
    if plan.aggregate != has_aggregate || (!plan.aggregate && !plan.groups.is_empty()) {
        return Err(fail("Application aggregate and projection stages differ"));
    }
    if prepared.scans.len() != 1 + plan.joins.len() {
        return Err(fail("Application projection scan inventory differs"));
    }
    let source = prepared
        .scans
        .get(&plan.source.occurrence)
        .ok_or_else(|| fail("Application source lacks original scan admission"))?;
    let columns = crate::result_definition::projection_columns(context, properties, comparators)?;
    if columns.len() != plan.outputs.len() || columns.is_empty() {
        return Err(fail("Application projection metadata arity differs"));
    }
    let payload_checks = crate::result_definition::read_payload_observations(prepared, properties)?;
    let mut staged = parameters.clone();
    let mut filters = source.source.filters.clone();
    let mut from = source.source.sql.clone();
    let mut visible = std::collections::BTreeSet::from([plan.source.occurrence.clone()]);
    for join in &plan.joins {
        if !visible.insert(join.right.occurrence.clone()) || join.on.is_empty() {
            return Err(fail("Join occurrence or condition inventory differs"));
        }
        let right = prepared
            .scans
            .get(&join.right.occurrence)
            .ok_or_else(|| fail("Join lacks admitted original right source"))?;
        filters.extend(right.source.filters.iter().cloned());
        let mut conditions = Vec::new();
        for predicate in &join.on {
            let expression = application_equality(predicate)?;
            let Expression::Equal { left, right, .. } = &expression else {
                unreachable!()
            };
            for operand in [left.as_ref(), right.as_ref()] {
                if let Expression::Field { scan, .. } = operand {
                    if !visible.contains(scan) {
                        return Err(fail("Join condition references a later scan"));
                    }
                }
            }
            conditions.push(render_application_equality(
                predicate,
                &prepared.accesses,
                &mut staged,
                &mut *native,
            )?);
        }
        let right_sql = if prepared.accesses.iter().any(|access| {
            access.scan == join.right.occurrence
                && matches!(access.location, crate::registered_access::Location::Row(_))
        }) {
            format!("({})", right.source.sql)
        } else {
            right.source.sql.clone()
        };
        from = format!(
            "({from} INNER JOIN {right_sql} ON {})",
            conditions.join(" AND ")
        );
    }

    for predicate in &plan.filters {
        filters.push(render_application_equality(
            predicate,
            &prepared.accesses,
            &mut staged,
            &mut *native,
        )?);
    }
    let mut groups = BTreeMap::new();
    for field in &plan.groups {
        // This selected bridge preserves string carriers; numeric grouping
        // needs its canonical result-correspondence procedure.
        if field.logical_type.family != weft_core::ir::Family::String || field.logical_type.nullable
        {
            return Err(fail(
                "Grouped output requires selected carrier correspondence",
            ));
        }
        let expression = Expression::Field {
            scan: field.scan.clone(),
            identity: field.identity.clone(),
            logical_type: field.logical_type.clone(),
            span: field.span.clone(),
        };
        let sql = crate::registered_access::render_expression(
            &expression,
            &prepared.accesses,
            &mut staged,
            &mut *native,
        )?;
        if groups
            .insert(
                serde_json::json!({"scan":field.scan,"field":field.identity}).to_string(),
                sql,
            )
            .is_some()
        {
            return Err(fail("Repeated original application group field"));
        }
    }
    let mut projections = Vec::new();
    for (output, column) in plan.outputs.iter().zip(&columns) {
        if let app::Expression::Sum {
            argument,
            logical_type,
        } = &output.expression
        {
            validate_sum_result(&argument.logical_type, logical_type, plan.groups.is_empty())?;
            let expression = Expression::Sum {
                argument: Box::new(Expression::Field {
                    scan: argument.scan.clone(),
                    identity: argument.identity.clone(),
                    logical_type: argument.logical_type.clone(),
                    span: argument.span.clone(),
                }),
                logical_type: logical_type.clone(),
                span: argument.span.clone(),
            };
            let sql = crate::registered_access::render_expression(
                &expression,
                &prepared.accesses,
                &mut staged,
                &mut *native,
            )?;
            projections.push(format!(
                "({sql})::pg_catalog.text AS {}",
                crate::Identifier::new(&column.output_name)?.sql()
            ));
            continue;
        }
        if let app::Expression::Count { logical_type } = &output.expression {
            if logical_type.family != weft_core::ir::Family::Integer
                || logical_type.nullable
                || logical_type.facets != serde_json::json!({})
            {
                return Err(fail(
                    "COUNT result differs from resolved exact integer contract",
                ));
            }
            projections.push(format!(
                "pg_catalog.count(*)::pg_catalog.text AS {}",
                crate::Identifier::new(&column.output_name)?.sql()
            ));
            continue;
        }
        let app::Expression::Field { scan, identity } = &output.expression else {
            return Err(fail(
                "Application computed result needs original-definition lowering",
            ));
        };
        if plan.aggregate {
            let expression = groups
                .get(&serde_json::json!({"scan":scan,"field":identity}).to_string())
                .ok_or_else(|| fail("Aggregate output is not an admitted group field"))?;
            projections.push(format!(
                "({expression})::pg_catalog.text AS {}",
                crate::Identifier::new(&column.output_name)?.sql()
            ));
            continue;
        }
        let access = prepared
            .accesses
            .iter()
            .find(|access| &access.scan == scan && &access.field == identity)
            .ok_or_else(|| fail("Application output lacks exact prepared access"))?;
        let property = properties
            .get(&crate::comparator_requirements::registration_key(
                &access.owner,
                identity,
            ))
            .ok_or_else(|| fail("Application output lacks original property"))?;
        let projection = crate::result_definition::property_projection(
            property,
            access,
            column.position,
            &column.output_name,
        )?;
        if serde_json::to_value(&projection.column)
            .map_err(|_| fail("Application metadata encoding refused"))?
            != serde_json::to_value(column)
                .map_err(|_| fail("Application metadata encoding refused"))?
        {
            return Err(fail(
                "Application original codec and result metadata differ",
            ));
        }
        projections.push(projection.sql);
    }
    let mut sql = format!("SELECT {} FROM {}", projections.join(", "), from);
    if !filters.is_empty() {
        sql.push_str(&format!(" WHERE {}", filters.join(" AND ")));
    }
    if !groups.is_empty() {
        sql.push_str(&format!(
            " GROUP BY {}",
            groups.values().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    let mut order = Vec::new();
    for field in &plan.order {
        let key = serde_json::json!({"scan":field.scan,"field":field.identity}).to_string();
        let expression = if plan.aggregate {
            groups
                .get(&key)
                .cloned()
                .ok_or_else(|| fail("Aggregate order field is not grouped"))?
        } else {
            let expression = Expression::Field {
                scan: field.scan.clone(),
                identity: field.identity.clone(),
                logical_type: field.logical_type.clone(),
                span: field.span.clone(),
            };
            crate::registered_access::render_expression(
                &expression,
                &prepared.accesses,
                &mut staged,
                &mut *native,
            )?
        };
        order.push(format!("({expression}) ASC"));
    }
    if !order.is_empty() {
        sql.push_str(&format!(" ORDER BY {}", order.join(", ")));
    }
    if let Some(limit) = plan.limit {
        sql.push_str(&format!(" LIMIT {limit}"));
    }
    *parameters = staged;
    Ok(Select {
        sql,
        columns,
        payload_checks,
        structural_checks: prepared
            .scans
            .values()
            .flat_map(|scan| scan.structural_check_sql.iter().cloned())
            .collect(),
    })
}

fn application_equality(predicate: &weft_core::application_ir::Predicate) -> Result<Expression> {
    use weft_core::application_ir as app;
    let fail = |message: &str| Diagnostic::new("WFT-CAPABILITY", "emit", message);
    let app::Predicate::Equal { left, right } = predicate else {
        return Err(fail(
            "Application predicate requires its selected lowering bridge",
        ));
    };
    let field = |field: &app::Field| Expression::Field {
        scan: field.scan.clone(),
        identity: field.identity.clone(),
        logical_type: field.logical_type.clone(),
        span: field.span.clone(),
    };
    let right = match right {
        app::Value::Field { field: value } => field(value),
        app::Value::Literal {
            value,
            logical_type,
            span,
        } => Expression::Literal {
            value: value.clone(),
            logical_type: logical_type.clone(),
            span: span.clone(),
        },
        app::Value::Parameter {
            value,
            logical_type,
            span,
            ..
        } => Expression::Literal {
            value: value.clone(),
            logical_type: logical_type.clone(),
            span: span.clone(),
        },
    };
    Ok(Expression::Equal {
        left: Box::new(field(left)),
        right: Box::new(right),
        logical_type: weft_core::ir::LogicalType {
            family: weft_core::ir::Family::Boolean,
            facets: serde_json::json!({}),
            nullable: false,
        },
        span: left.span.clone(),
    })
}

fn validate_sum_result(
    argument: &weft_core::ir::LogicalType,
    result: &weft_core::ir::LogicalType,
    ungrouped: bool,
) -> Result<()> {
    use weft_core::ir::Family;
    let facets = match argument.family {
        Family::Integer => serde_json::json!({}),
        Family::Decimal if argument.facets.get("scale").is_some() => {
            serde_json::json!({"scale":argument.facets["scale"]})
        }
        _ => {
            return Err(Diagnostic::new(
                "WFT-TYPE",
                "emit",
                "SUM requires its exact numeric argument contract",
            ))
        }
    };
    if argument.family != result.family || result.facets != facets || result.nullable != ungrouped {
        return Err(Diagnostic::new(
            "WFT-TYPE",
            "emit",
            "SUM result differs from its resolved numeric/nullability contract",
        ));
    }
    Ok(())
}
#[cfg(test)]
mod aggregate_tests {
    use super::*;
    use weft_core::ir::{Family, LogicalType};
    #[test]
    fn sum_preserves_exact_family_scale_and_empty_input_nullability() {
        for (family, input, output) in [
            (
                Family::Integer,
                serde_json::json!({"integerWidth":{"bits":64,"signed":false}}),
                serde_json::json!({}),
            ),
            (
                Family::Decimal,
                serde_json::json!({"precision":28,"scale":9}),
                serde_json::json!({"scale":9}),
            ),
        ] {
            let argument = LogicalType {
                family: family.clone(),
                facets: input,
                nullable: false,
            };
            for ungrouped in [false, true] {
                let result = LogicalType {
                    family: family.clone(),
                    facets: output.clone(),
                    nullable: ungrouped,
                };
                validate_sum_result(&argument, &result, ungrouped).unwrap();
                let mut wrong = result.clone();
                wrong.nullable = !ungrouped;
                assert!(validate_sum_result(&argument, &wrong, ungrouped).is_err());
                wrong = result.clone();
                wrong.facets = serde_json::json!({"scale":8});
                assert!(validate_sum_result(&argument, &wrong, ungrouped).is_err());
            }
        }
        let text = LogicalType {
            family: Family::String,
            facets: serde_json::json!({}),
            nullable: false,
        };
        assert!(validate_sum_result(&text, &text, false).is_err());
    }
}

fn render_application_equality<'a>(
    predicate: &weft_core::application_ir::Predicate,
    accesses: &[Access<'a>],
    parameters: &mut Parameters,
    native: &mut dyn FnMut(
        &Expression,
        &[String],
        Option<&Access<'a>>,
        &mut Parameters,
    ) -> Result<String>,
) -> Result<String> {
    use weft_core::application_ir as app;
    if let app::Predicate::LexicographicGreater { columns, values } = predicate {
        if columns.is_empty() || columns.len() != values.len() || columns.len() > 32 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "emit",
                "Cursor tuple requires 1 through 32 matching components",
            ));
        }
        let mut staged = parameters.clone();
        let mut left = Vec::new();
        let mut right = Vec::new();
        for (field, value) in columns.iter().zip(values) {
            let logical = match value {
                app::Value::Field { field } => &field.logical_type,
                app::Value::Literal { logical_type, .. }
                | app::Value::Parameter { logical_type, .. } => logical_type,
            };
            if logical != &field.logical_type || logical.nullable {
                return Err(Diagnostic::new(
                    "WFT-TYPE",
                    "emit",
                    "Cursor component type differs or permits native null",
                ));
            }
            let component = app::Predicate::Equal {
                left: field.clone(),
                right: value.clone(),
            };
            let mut operands = None;
            render_application_equality(
                &component,
                accesses,
                &mut staged,
                &mut |node, rendered, access, parameters| {
                    if matches!(node, Expression::Equal { .. }) {
                        operands = Some(rendered.to_vec());
                        Ok("TRUE".into())
                    } else {
                        native(node, rendered, access, parameters)
                    }
                },
            )?;
            let operands = operands.ok_or_else(|| {
                Diagnostic::new(
                    "WFT-BINDING",
                    "emit",
                    "Cursor component lowering incomplete",
                )
            })?;
            left.push(operands[0].clone());
            right.push(operands[1].clone());
        }
        *parameters = staged;
        return Ok(format!(
            "(ROW({}) > ROW({}))",
            left.join(", "),
            right.join(", ")
        ));
    }
    let expression = application_equality(predicate)?;
    crate::registered_access::render_expression(
        &expression,
        accesses,
        parameters,
        |node, operands, access, parameters| {
            if let (
                app::Predicate::Equal {
                    right:
                        app::Value::Parameter {
                            name,
                            value,
                            logical_type,
                            span,
                        },
                    ..
                },
                Expression::Literal { .. },
            ) = (predicate, node)
            {
                let before = parameters.0.len();
                let sql = native(node, operands, access, parameters)?;
                if parameters.0.len() != before + 1
                    || parameters.0[before].value != *value
                    || parameters.0[before].logical_type != *logical_type
                {
                    return Err(Diagnostic::new(
                        "WFT-BINDING",
                        "emit",
                        "Named parameter conversion must retain one exact typed value slot",
                    ));
                }
                parameters.0[before].origin = serde_json::json!({"parameter":name,"span":span});
                Ok(sql)
            } else {
                native(node, operands, access, parameters)
            }
        },
    )
}

#[cfg(test)]
mod cursor_bounds_tests {
    use super::*;
    #[test]
    fn tuple_bounds_refuse_before_callbacks_and_preserve_parameter_state() {
        use weft_core::{
            application_ir as app,
            ir::{Family, Identity, LogicalType, Span},
        };
        let field = app::Field {
            scan: "scan".into(),
            identity: Identity {
                document_id: "d".into(),
                revision: "1".into(),
                module: "m".into(),
                element: "f".into(),
            },
            logical_type: LogicalType {
                family: Family::String,
                facets: serde_json::json!({}),
                nullable: false,
            },
            span: Span { start: 0, end: 0 },
        };
        let value = app::Value::Literal {
            value: "A".into(),
            logical_type: field.logical_type.clone(),
            span: field.span.clone(),
        };
        for (columns, values) in [
            (vec![], vec![]),
            (vec![field.clone()], vec![]),
            (vec![field; 33], vec![value; 33]),
        ] {
            let mut parameters = Parameters::default();
            let result = render_application_equality(
                &app::Predicate::LexicographicGreater { columns, values },
                &[],
                &mut parameters,
                &mut |_, _, _, _| panic!("Invalid tuple reached native lowering"),
            );
            assert_eq!(result.unwrap_err().code, "WFT-LIMIT");
            assert!(parameters.into_slots().is_empty());
        }
    }
}
