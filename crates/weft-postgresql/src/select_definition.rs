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
        return Err(fail("Selected assembly requires the V01 plan bridge"));
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
        if !matches!(column.representation, Representation::Scalar { .. }) {
            return Err(fail(
                "SELECT value requires its selected recursive result bridge",
            ));
        }
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
