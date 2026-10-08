//! Original admitted relationship subqueries and independent target prerequisites.
use crate::{
    native_comparator_definition::{Definition, Operation},
    property_definition::PropertyAdmission,
    record_definition::RecordAdmission,
    registered_access::Location,
    relationship_definition::RelationshipAdmission,
    Identifier, Parameters,
};
use std::collections::BTreeMap;
use weft_core::{
    application_ir::Value,
    application_model::RelationshipRead,
    backend::Context,
    error::{Diagnostic, Result},
    ir::{Expression, Family},
};
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "lower", message)
}
pub enum Read<'a> {
    HasRelated(&'a [Value]),
    RelatedKeys(u16),
}
#[derive(Debug)]
pub struct Lowered {
    pub expression: String,
    pub structural_checks: Vec<String>,
    pub payload_checks: Vec<crate::result_definition::ReadPayloadObservation>,
}
/// Source alias is assigned by trusted outer-plan preparation, never model SQL.
/// Target properties/comparators are admitted independently before SQL escapes.
pub fn lower<'a>(
    admission: &RelationshipAdmission,
    context: &Context<'_>,
    relationship: &RelationshipRead,
    source_alias: &Identifier,
    read: Read<'_>,
    records: &BTreeMap<String, RecordAdmission>,
    properties: &'a BTreeMap<String, PropertyAdmission>,
    comparators: &BTreeMap<String, Definition>,
    scope: u16,
    parameters: &mut Parameters,
    mut native: impl FnMut(
        &Expression,
        &[String],
        Option<&crate::registered_access::Access<'a>>,
        &mut Parameters,
    ) -> Result<String>,
) -> Result<Lowered> {
    let mut staged = parameters.clone();
    let target = admission.prepare_target(
        context,
        relationship,
        records,
        properties,
        comparators,
        &mut staged,
        scope,
    )?;
    let physical = &target.prepared.scans[&target.plan.source.occurrence];
    let alias = target
        .prepared
        .accesses
        .first()
        .ok_or_else(|| fail("Target key has no physical access"))?
        .owner_alias
        .clone();
    let edge_alias = Identifier::new(&format!("weft_related_{scope}_edge"))?;
    let binding =
        crate::binding::Admission::parse(&context.binding.json, &context.binding.profile)?;
    let edge = admission.correlate(
        &binding,
        relationship,
        &edge_alias,
        source_alias,
        &Identifier::new("id")?,
        &Identifier::new("type_id")?,
        &mut staged,
    )?;
    let payload_checks = crate::result_definition::read_payload_observations_with_parameters(
        &target.prepared,
        properties,
        &mut staged,
    )?;
    let mut checks = physical.structural_check_sql.clone();
    checks.extend(admission.integrity_checks(&binding, relationship, &mut staged)?);
    let mut ordered = Vec::new();
    let mut values = Vec::new();
    for field in &target.plan.order {
        let access = target
            .prepared
            .accesses
            .iter()
            .find(|a| a.field == field.identity)
            .ok_or_else(|| fail("Target key access missing"))?;
        let key =
            crate::comparator_requirements::registration_key(&relationship.to, &field.identity);
        let comparator = comparators
            .get(&key)
            .ok_or_else(|| fail("Target key comparator missing"))?;
        comparator.require_type(&field.logical_type)?;
        comparator.require(Operation::Key)?;
        comparator.require(Operation::Equality)?;
        if matches!(read, Read::RelatedKeys(_)) {
            comparator.require(Operation::Ordering)?;
        }
        let expression = Expression::Field {
            scan: field.scan.clone(),
            identity: field.identity.clone(),
            logical_type: field.logical_type.clone(),
            span: field.span.clone(),
        };
        let sql = crate::registered_access::render_expression(
            &expression,
            &target.prepared.accesses,
            &mut staged,
            &mut native,
        )?;
        if matches!(field.logical_type.family, Family::Integer | Family::Decimal) {
            let carrier = match &access.location {
                Location::Props(_) => access
                    .scalar_storage
                    .as_ref()
                    .ok_or_else(|| fail("Target numeric carrier missing"))?
                    .carrier
                    .clone(),
                Location::Row(l) => l.scalar_observation().native_numeric_text,
            };
            let integrity = comparator.numeric_domain_sql(&carrier)?;
            checks.push(format!("SELECT count(*) AS violations FROM {} WHERE {} AND ({integrity}) IS DISTINCT FROM TRUE",physical.source.sql,physical.source.filters.join(" AND ")));
            values.push(format!("({sql})::pg_catalog.text"));
        } else {
            values.push(sql.clone());
        }
        ordered.push(sql);
    }
    checks.push(format!("SELECT count(*) AS violations FROM (SELECT {} FROM {} WHERE {} GROUP BY {} HAVING pg_catalog.count(*)>1) AS weft_related_duplicate_keys",ordered.join(", "),physical.source.sql,physical.source.filters.join(" AND "),ordered.join(", ")));
    let from = format!(
        "{} JOIN {} ON {}.id={} AND {}.type_id={}",
        edge.source,
        physical.source.sql,
        alias.sql(),
        edge.target_object_id,
        alias.sql(),
        edge.target_type
    );
    let mut predicates = edge.predicates.clone();
    predicates.extend(physical.source.filters.iter().cloned());
    let expression = match read {
        Read::HasRelated(key) => {
            if key.len() != target.plan.order.len() {
                return Err(fail(
                    "HAS_RELATED key arity differs from authored target key",
                ));
            }
            for ((value, field), left) in key.iter().zip(&target.plan.order).zip(&ordered) {
                let (token, logical, span) = match value {
                    Value::Literal {
                        value,
                        logical_type,
                        span,
                    }
                    | Value::Parameter {
                        value,
                        logical_type,
                        span,
                        ..
                    } => (value, logical_type, span),
                    Value::Field { .. } => {
                        return Err(Diagnostic::new(
                            "WFT-CAPABILITY",
                            "lower",
                            "Relationship field-valued key needs outer operand access",
                        ))
                    }
                };
                if logical != &field.logical_type || logical.nullable {
                    return Err(fail("HAS_RELATED component type differs from target key"));
                }
                let literal = Expression::Literal {
                    value: token.clone(),
                    logical_type: logical.clone(),
                    span: span.clone(),
                };
                let before = staged.0.len();
                let right = native(&literal, &[], None, &mut staged)?;
                if let Value::Parameter {
                    name,
                    value,
                    logical_type,
                    span,
                } = value
                {
                    if staged.0.len() != before + 1
                        || staged.0[before].value != *value
                        || staged.0[before].logical_type != *logical_type
                    {
                        return Err(fail(
                            "Relationship named parameter must retain one exact typed slot",
                        ));
                    }
                    staged.0[before].origin = serde_json::json!({"parameter":name,"span":span});
                }
                predicates.push(format!("({left})=({right})"));
            }
            format!(
                "EXISTS (SELECT 1 FROM {from} WHERE {})",
                predicates.join(" AND ")
            )
        }
        Read::RelatedKeys(bound) => {
            if !(1..=1000).contains(&bound) {
                return Err(Diagnostic::new(
                    "WFT-LIMIT",
                    "lower",
                    "Related key bound requires 1 through 1000",
                ));
            }
            let bound_type = weft_core::ir::LogicalType {
                family: Family::Integer,
                nullable: false,
                facets: serde_json::json!({"integerWidth":{"bits":16,"signed":false}}),
            };
            let bound_slot = staged.push(
                bound_type.clone(),
                bound.to_string(),
                serde_json::json!({"relatedBound":bound}),
            )?;
            let probe_slot = staged.push(
                bound_type,
                (u32::from(bound) + 1).to_string(),
                serde_json::json!({"relatedProbe":u32::from(bound)+1}),
            )?;
            let order = ordered.join(", ");
            let inner=format!("SELECT pg_catalog.jsonb_build_array({}) AS k, pg_catalog.row_number() OVER (ORDER BY {order}) AS ord FROM {from} WHERE {} ORDER BY {order} LIMIT {probe_slot}::pg_catalog.int4",values.join(", "),predicates.join(" AND "));
            format!("(SELECT pg_catalog.jsonb_build_object('items',COALESCE(pg_catalog.jsonb_agg(k ORDER BY ord) FILTER (WHERE ord<={bound_slot}::pg_catalog.int4),'[]'::pg_catalog.jsonb),'truncated',pg_catalog.count(*)>{bound_slot}::pg_catalog.int4)::pg_catalog.text FROM ({inner}) AS weft_related_{scope}_bounded)")
        }
    };
    *parameters = staged;
    Ok(Lowered {
        expression,
        structural_checks: checks,
        payload_checks,
    })
}
