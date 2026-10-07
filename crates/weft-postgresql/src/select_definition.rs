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
    native: impl FnMut(&Expression, &[String], Option<&Access<'a>>, &mut Parameters) -> Result<String>,
) -> Result<Compilation> {
    let mut parameters = Parameters::default();
    let prepared = crate::registered_access::prepare(
        context,
        records,
        properties,
        comparators,
        &mut parameters,
    )?;
    let select = assemble(
        context,
        &prepared,
        properties,
        comparators,
        &mut parameters,
        native,
    )?;
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
        return assemble_application(context, prepared, properties, comparators, parameters);
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

/// Original-definition V02 field projections. Other relational stages require
/// their selected lowering procedures and must never be silently discarded.
fn assemble_application(
    context: &Context<'_>,
    prepared: &Prepared<'_>,
    properties: &BTreeMap<String, crate::property_definition::PropertyAdmission>,
    comparators: &BTreeMap<String, crate::native_comparator_definition::Definition>,
    parameters: &mut Parameters,
) -> Result<Select> {
    use weft_core::application_ir as app;
    let fail = |message: &str| Diagnostic::new("WFT-CAPABILITY", "emit", message);
    let Plan::V02(plan) = context.plan else {
        unreachable!()
    };
    if !plan.joins.is_empty()
        || !plan.filters.is_empty()
        || !plan.groups.is_empty()
        || plan.aggregate
        || !plan.order.is_empty()
        || plan.limit.is_some()
    {
        return Err(fail(
            "Application relational stages need original-definition lowering",
        ));
    }
    if prepared.scans.len() != 1 {
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
    let mut projections = Vec::new();
    for (output, column) in plan.outputs.iter().zip(&columns) {
        let app::Expression::Field { scan, identity } = &output.expression else {
            return Err(fail(
                "Application computed result needs original-definition lowering",
            ));
        };
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
    let mut sql = format!(
        "SELECT {} FROM {}",
        projections.join(", "),
        source.source.sql
    );
    if !source.source.filters.is_empty() {
        sql.push_str(&format!(" WHERE {}", source.source.filters.join(" AND ")));
    }
    // This branch allocates no parameters; prepared custody was verified above.
    let _ = parameters;
    Ok(Select {
        sql,
        columns,
        payload_checks,
        structural_checks: source.structural_check_sql.clone(),
    })
}
