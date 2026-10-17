//! Original source-Plan identity selection; never parse a previously emitted SQL string.
use super::*;
use weft_core::application_ir as app;

pub(super) fn prepare(
    context: &Context<'_>,
    binding: &Binding,
    plan: &app::Plan,
    lower: &mut Lower<'_>,
    relations: &relationships::Traversals,
    relationship_checks: &mut Vec<Value>,
) -> Result<Option<(String, Vec<Obligation>)>> {
    let Some(navigation) = &plan.navigation else {
        return Ok(None);
    };
    let source = navigation.source();
    if source.navigation.is_some()
        || !source.groups.is_empty()
        || source.outputs.iter().any(|o| {
            matches!(
                o.expression,
                app::Expression::Count { .. }
                    | app::Expression::CountDistinct { .. }
                    | app::Expression::Sum { .. }
            )
        })
    {
        return Err(fail(
            "WFT-PROFILE",
            "Original nonaggregated single-level navigation source required",
        ));
    }
    let selection = weft_core::backend::selection(Plan::V02(source));
    let source_context = Context {
        catalog: context.catalog,
        plan: Plan::V02(source),
        target: context.target,
        binding: context.binding,
        binding_value: context.binding_value,
        selection: &selection,
    };
    // This is a fresh source admission, not a claim that target admission alone
    // covers the source's physical homes, Fields or projection dependencies.
    let admitted = validate_mapping(&source_context, false)?;
    let mut source_lower = Lower::new(&admitted.mapping);
    source_lower.mathematical_profile = lower.mathematical_profile;
    // Original parameter origins share one ordered native slot inventory.
    source_lower.parameters = std::mem::take(&mut lower.parameters);
    application::collect_plan(&source_context, &mut source_lower, source)?;
    source_lower
        .physical_ids
        .insert(source.source.occurrence.clone());
    let mut source_relations = relationships::Traversals::collect(&mut source_lower, source)?;
    source_lower.prepare()?;
    let source_relationship_checks = source_relations.prepare(&mut source_lower)?;
    let mut from = binding::quote(&source.source.occurrence);
    for join in &source.joins {
        let on = join
            .on
            .iter()
            .map(|p| application::predicate(&mut source_lower, &source_relations, p))
            .collect::<Result<Vec<_>>>()?
            .join(" AND ");
        from = format!(
            "({from} INNER JOIN {} ON {on})",
            binding::quote(&join.right.occurrence)
        );
    }
    let filters = source
        .filters
        .iter()
        .map(|p| application::predicate(&mut source_lower, &source_relations, p))
        .collect::<Result<Vec<_>>>()?;
    let order = source
        .order
        .iter()
        .map(|field| source_lower.expression(&application::legacy(field)))
        .collect::<Result<Vec<_>>>()?;
    // Projection is deliberately the original source object identity. Original
    // output dependencies were still collected/admitted above; arbitrary source
    // projection values never become graph identities.
    let mut selection_sql = format!(
        "WITH{} {} SELECT {}.__id AS __id FROM {from}",
        if source_lower.compounds.is_empty() {
            ""
        } else {
            " RECURSIVE"
        },
        source_lower.ctes.join(", "),
        binding::quote(&source.source.occurrence)
    );
    if !filters.is_empty() {
        selection_sql.push_str(&format!(" WHERE {}", filters.join(" AND ")));
    }
    if !order.is_empty() {
        selection_sql.push_str(&format!(" ORDER BY {}", order.join(", ")));
    }
    if let Some(limit) = source.limit {
        selection_sql.push_str(&format!(" LIMIT {limit}"));
    }
    let mut index = 0;
    let selected = loop {
        let name = format!("__weft_navigation_source_{index}");
        index += 1;
        if !lower.scans.contains_key(&name) {
            break name;
        }
    };
    lower.ctes.push(format!(
        "{} AS ({selection_sql})",
        binding::quote(&selected)
    ));
    lower.parameters = source_lower.parameters;
    lower.checks.extend(source_lower.checks);
    lower.compound_checks.extend(source_lower.compound_checks);
    relationship_checks.extend(source_relationship_checks);
    let membership = relations.source_selection(
        &plan.source.occurrence,
        navigation.relationship(),
        &selected,
    )?;
    // EXISTS deduplicates edge/source occurrences only for target membership;
    // source LIMIT ran on its original ordered bag before this selection.
    let _ = binding; // Admission above independently consumed the same full binding.
    Ok(Some((membership, admitted.obligations)))
}
