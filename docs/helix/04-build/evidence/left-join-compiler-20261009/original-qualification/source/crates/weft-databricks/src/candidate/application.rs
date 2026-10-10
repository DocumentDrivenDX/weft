//! Application stages reuse the same exact scalar access and native slots.
use super::*;
use weft_core::{application_ir as app, application_model::Shape, ir::Span};

fn legacy(field: &app::Field) -> Expression {
    Expression::Field {
        scan: field.scan.clone(),
        identity: field.identity.clone(),
        logical_type: field.logical_type.clone(),
        span: field.span.clone(),
    }
}
fn field(lower: &mut Lower<'_>, f: &app::Field) {
    collect_expression(&legacy(f), &mut lower.fields);
}
fn values(lower: &mut Lower<'_>, v: &app::Value) {
    if let app::Value::Field { field: f } = v {
        field(lower, f);
    }
}
fn collect_predicate(lower: &mut Lower<'_>, p: &app::Predicate) -> Result<()> {
    match p {
        app::Predicate::Equal { left, right } => {
            field(lower, left);
            values(lower, right);
        }
        app::Predicate::LexicographicGreater { columns, values: v } => {
            for f in columns {
                field(lower, f);
            }
            for value in v {
                values(lower, value);
            }
        }
        app::Predicate::HasRelated { key, .. } => {
            for v in key {
                values(lower, v);
            }
        }
    }
    Ok(())
}
pub(super) fn value(lower: &mut Lower<'_>, v: &app::Value) -> Result<String> {
    match v {
        app::Value::Field { field } => lower.expression(&legacy(field)),
        app::Value::Literal {
            value,
            logical_type,
            span,
        } => lower.expression(&Expression::Literal {
            value: value.clone(),
            logical_type: logical_type.clone(),
            span: span.clone(),
        }),
        app::Value::Parameter {
            name,
            value,
            logical_type,
            span,
        } => {
            let slot = lower.slot(
                logical_type.clone(),
                value.clone(),
                json!({"kind":"namedParameter","name":name,"sourceSpan":span}),
            )?;
            Ok(typed(&slot, logical_type))
        }
    }
}
fn predicate(
    lower: &mut Lower<'_>,
    relations: &relationships::Traversals,
    p: &app::Predicate,
) -> Result<String> {
    match p {
        app::Predicate::Equal { left, right } => Ok(format!(
            "({} = {})",
            lower.expression(&legacy(left))?,
            value(lower, right)?
        )),
        app::Predicate::LexicographicGreater { columns, values } => {
            if columns.is_empty() || columns.len() != values.len() {
                return Err(fail("WFT-EMIT", "Invalid resolved cursor tuple"));
            }
            let pairs = columns
                .iter()
                .zip(values)
                .map(|(f, v)| Ok((lower.expression(&legacy(f))?, value(lower, v)?)))
                .collect::<Result<Vec<_>>>()?;
            let mut clauses = Vec::new();
            for i in 0..pairs.len() {
                let mut clause = pairs[..i]
                    .iter()
                    .map(|(a, b)| format!("({a} = {b})"))
                    .collect::<Vec<_>>();
                clause.push(format!("({} > {})", pairs[i].0, pairs[i].1));
                clauses.push(format!("({})", clause.join(" AND ")));
            }
            Ok(format!("({})", clauses.join(" OR ")))
        }
        app::Predicate::HasRelated {
            scan,
            relationship,
            key,
        } => relations.exists(lower, scan, relationship, key),
    }
}
fn descriptor<'a>(
    p: &'a app::Plan,
    id: &Identity,
) -> Result<&'a weft_core::application_model::Descriptor> {
    p.type_graph
        .iter()
        .find(|d| &d.identity == id)
        .ok_or_else(|| fail("WFT-BINDING", "Selected application descriptor is missing"))
}
fn representation(ty: &LogicalType) -> Representation {
    Representation::Scalar {
        logical_type: ty.clone(),
        carrier: ScalarCarrier::Text,
        decoder: match ty.family {
            Family::String => ScalarDecoder::Text,
            Family::Boolean => ScalarDecoder::Boolean,
            Family::Integer => ScalarDecoder::ExactInteger,
            Family::Decimal => ScalarDecoder::ExactDecimal,
        },
    }
}
pub(super) fn lower(
    context: &Context<'_>,
    binding: &Binding,
    plan: &app::Plan,
) -> Result<TargetPlan> {
    let mut lower = Lower::new(binding);
    lower.mathematical_profile=context.target.id==crate::mathematical_integer::PROFILE;
    lower
        .scans
        .insert(plan.source.occurrence.clone(), plan.source.record.clone());
    for join in &plan.joins {
        lower
            .scans
            .insert(join.right.occurrence.clone(), join.right.record.clone());
        for p in &join.on {
            collect_predicate(&mut lower, p)?;
        }
    }
    for p in &plan.filters {
        collect_predicate(&mut lower, p)?;
    }
    for f in plan.groups.iter().chain(&plan.order) {
        field(&mut lower, f);
    }
    for output in &plan.outputs {
        match &output.expression {
            app::Expression::Field { scan, identity } => {
                let d = descriptor(plan, identity)?;
                let Shape::Scalar { logical_type } = &d.shape else {
                    let record = context.catalog.record_by_identity(&lower.scans[scan])?;
                    let (_,graph) = context.catalog.member_descriptor_by_identity(&record, identity)?;
                    lower.compounds.insert((scan.clone(),serde_json::to_string(identity).unwrap()),graph);
                    continue;
                };
                if d.availability.as_deref() == Some("absent-allowed") {
                    lower
                        .optional
                        .insert((scan.clone(), serde_json::to_string(identity).unwrap()));
                }
                field(
                    &mut lower,
                    &app::Field {
                        scan: scan.clone(),
                        identity: identity.clone(),
                        logical_type: logical_type.clone(),
                        span: Span { start: 0, end: 0 },
                    },
                );
            }
            app::Expression::Sum { argument, .. } => field(&mut lower, argument),
            app::Expression::Count { .. } => {}
            app::Expression::RelatedKeys { .. } => {}
        }
    }
    let mut relations = relationships::Traversals::collect(&mut lower, plan)?;
    lower.prepare()?;
    let relationship_checks = relations.prepare(&mut lower)?;
    let mut from = binding::quote(&plan.source.occurrence);
    for join in &plan.joins {
        let on = join
            .on
            .iter()
            .map(|p| predicate(&mut lower, &relations, p))
            .collect::<Result<Vec<_>>>()?
            .join(" AND ");
        from = format!(
            "({from} INNER JOIN {} ON {on})",
            binding::quote(&join.right.occurrence)
        );
    }
    let filters = plan
        .filters
        .iter()
        .map(|p| predicate(&mut lower, &relations, p))
        .collect::<Result<Vec<_>>>()?;
    let groups = plan
        .groups
        .iter()
        .map(|f| lower.expression(&legacy(f)))
        .collect::<Result<Vec<_>>>()?;
    let order = plan
        .order
        .iter()
        .map(|f| lower.expression(&legacy(f)).map(|sql| format!("{sql} ASC")))
        .collect::<Result<Vec<_>>>()?;
    let mut outputs = Vec::new();
    let mut columns = Vec::new();
    let mut related_joins = Vec::new();
    for (index, output) in plan.outputs.iter().enumerate() {
        if let app::Expression::Field {scan,identity} = &output.expression {
            let key=(scan.clone(),serde_json::to_string(identity).unwrap());
            if lower.compounds.contains_key(&key) {
                let sql=lower.expressions.get(&key).unwrap();
                outputs.push(format!("{sql} AS {}",binding::quote(&output.name)));
                columns.push(Column{position:index+1,carrier_name: None, output_name:output.name.clone(),representation:Representation::Value{descriptor:identity.clone(),native_null:false,outer_join:None},source_identities:vec![identity.clone()],nullable:false});
                continue;
            }
        }
        if let app::Expression::RelatedKeys {
            scan,
            relationship,
            bound,
        } = &output.expression
        {
            let (value, join) = relations.related(&mut lower, scan, relationship, *bound)?;
            related_joins.push(join);
            outputs.push(format!("{value} AS {}", binding::quote(&output.name)));
            columns.push(Column {
                position: index + 1,
                carrier_name: None, output_name: output.name.clone(),
                representation: Representation::RelatedKeys {
                    relationship: relationship.identity.clone(),
                    key: relationship.target_key.clone(),
                    bound: *bound,
                },
                source_identities: std::iter::once(relationship.from.clone())
                    .chain(std::iter::once(relationship.to.clone()))
                    .chain(relationship.target_key.fields.clone())
                    .collect(),
                nullable: false,
            });
            continue;
        }
        let (sql, ty, ids) = match &output.expression {
            app::Expression::Field { scan, identity } => {
                let Shape::Scalar { logical_type } = &descriptor(plan, identity)?.shape else {
                    unreachable!()
                };
                (
                    lower.expression(&legacy(&app::Field {
                        scan: scan.clone(),
                        identity: identity.clone(),
                        logical_type: logical_type.clone(),
                        span: Span { start: 0, end: 0 },
                    }))?,
                    logical_type.clone(),
                    vec![identity.clone()],
                )
            }
            app::Expression::Sum {
                argument,
                logical_type,
            } => {
                let expression = Expression::Sum {
                    argument: Box::new(legacy(argument)),
                    logical_type: logical_type.clone(),
                    span: argument.span.clone(),
                };
                (
                    lower.expression(&expression)?,
                    logical_type.clone(),
                    vec![argument.identity.clone()],
                )
            }
            app::Expression::Count { logical_type } => (
                count_sql(),
                logical_type.clone(),
                std::iter::once(plan.source.record.clone())
                    .chain(plan.joins.iter().map(|j| j.right.record.clone()))
                    .collect(),
            ),
            app::Expression::RelatedKeys { .. } => unreachable!(),
        };
        let optional = match &output.expression {
            app::Expression::Field { scan, identity } => lower
                .presence
                .get(&(scan.clone(), serde_json::to_string(identity).unwrap()))
                .map(|p| (p.clone(), identity.clone())),
            _ => None,
        };
        let (sql, repr, nullable) = if let Some((present, identity)) = optional {
            let scalar = if ty.family == Family::Boolean {
                sql
            } else {
                format!("CAST({sql} AS STRING)")
            };
            (format!("CASE WHEN NOT ({present}) THEN to_json(named_struct('state', 'absent')) ELSE to_json(named_struct('state', 'value', 'value', {scalar})) END"),
             Representation::Value { descriptor: identity, native_null: false, outer_join: None }, false)
        } else {
            (
                format!("CAST({sql} AS STRING)"),
                representation(&ty),
                ty.nullable,
            )
        };
        outputs.push(format!("{sql} AS {}", binding::quote(&output.name)));
        columns.push(Column {
            position: index + 1,
            carrier_name: None, output_name: output.name.clone(),
            representation: repr,
            source_identities: ids,
            nullable,
        });
    }
    for join in related_joins {
        from.push_str(&format!(" {join}"));
    }
    let mut sql = format!(
        "WITH{} {} SELECT {} FROM {from}",
        if lower.compounds.is_empty() { "" } else { " RECURSIVE" },
        lower.ctes.join(", "),
        outputs.join(", ")
    );
    if !filters.is_empty() {
        sql.push_str(&format!(" WHERE {}", filters.join(" AND ")));
    }
    if !groups.is_empty() {
        sql.push_str(&format!(" GROUP BY {}", groups.join(", ")));
    }
    if !order.is_empty() {
        sql.push_str(&format!(" ORDER BY {}", order.join(", ")));
    }
    if let Some(limit) = plan.limit {
        sql.push_str(&format!(" LIMIT {limit}"));
    }
    let mut obligations = vec![
        publication(binding),
        Obligation {
            id: "ashlar.candidate.scalarIntegrity".into(),
            parameters: json!({"phase":"before-user-query","checks":lower.checks,"success":"one exact STRING count equal to 0 per check","parameters":"same emitted ordered slots; values never interpolated","samePublicationRequired":true,"noPartialPublication":true}),
            owner: ObligationOwner::Host,
            failure_code: "WFT-NUMERIC-DOMAIN".into(),
        },
    ];
    if !lower.compound_checks.is_empty() {
        obligations.push(Obligation{id:"ashlar.candidate.compoundIntegrity".into(),parameters:json!({"phase":"before-user-query","checks":lower.compound_checks,"success":"one exact STRING count equal to 0 per check","samePublicationRequired":true,"encoding":"ashlar-weft-json-value/0.1-candidate","nativeNull":false,"numericLeaves":"exact strings","limits":{"depthExclusive":128,"nodesPerValue":100000},"execution":"native recursion/resource errors refuse atomically; no partial values"}),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into()});
    }
    if !relationship_checks.is_empty() {
        obligations.push(Obligation{id:"ashlar.candidate.relationshipIntegrity".into(),parameters:json!({"phase":"before-user-query","checks":relationship_checks,"success":"one exact STRING count equal to 0 per check","samePublicationRequired":true,"policy":"complete authorized source and target inputs; inverse traversal cannot broaden authority","multiplicity":"parallel edges retained; min/max checked in authored orientation","lifecycle":"host verifies the authored lifecycle and projection coverage against original publication"}),owner:ObligationOwner::Host,failure_code:"WFT-BINDING".into()});
    }
    if let Some(key) = &plan.page_key {
        let record = context.catalog.record_by_identity(&plan.source.record)?;
        let authored = context.catalog.authored_key(&record, &key.id)?;
        let raw = record.value["keys"]
            .as_array()
            .unwrap()
            .iter()
            .find(|k| k["id"] == key.id)
            .unwrap();
        if raw
            .as_object()
            .unwrap()
            .keys()
            .any(|k| !matches!(k.as_str(), "id" | "name" | "fields" | "primary"))
            || authored.fields != key.fields
            || authored.types != key.types
        {
            return Err(fail(
                "WFT-BINDING",
                "Selected authored key meaning is not registered",
            ));
        }
        let fields = key
            .fields
            .iter()
            .zip(&key.types)
            .map(|(id, ty)| {
                lower.expression(&Expression::Field {
                    scan: plan.source.occurrence.clone(),
                    identity: id.clone(),
                    logical_type: ty.clone(),
                    span: Span { start: 0, end: 0 },
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let keys = fields.join(", ");
        let count = count_sql();
        let check=format!("WITH{} {} SELECT CAST({count} AS STRING) AS violations FROM (SELECT {keys} FROM {} GROUP BY {keys} HAVING {count} > 1) duplicate_keys",if lower.compounds.is_empty(){""}else{" RECURSIVE"},lower.ctes.join(", "),binding::quote(&plan.source.occurrence));
        obligations.push(Obligation{id:"ashlar.candidate.keyIntegrity".into(),parameters:json!({"phase":"before-user-query","key":key,"checks":[{"sql":check,"failureCode":"WFT-BINDING"}],"success":"one exact STRING count equal to 0","continuation":"same immutable publication, original binding and effective policy across pages; ORDER BY alone proves no continuity"}),owner:ObligationOwner::Host,failure_code:"WFT-BINDING".into()});
    }
    Ok(TargetPlan(Emission {
        sql,
        parameters: lower.parameters,
        columns,
        obligations,
    }))
}
