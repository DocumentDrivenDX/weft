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
    source_lower.scalar_presence_profile=lower.scalar_presence_profile;
    // Original parameter origins share one ordered native slot inventory.
    source_lower.parameters = std::mem::take(&mut lower.parameters);
    application::collect_plan(&source_context, &mut source_lower, source)?;
    source_lower
        .physical_ids
        .insert(source.source.occurrence.clone());
    let mut source_relations = relationships::Traversals::collect(&mut source_lower, source)?;
    #[cfg(feature="sdk-ordered-page-candidate")]
    if let Some(slots)=navigation.source_order(){
        for slot in slots{let id=(source.source.occurrence.clone(),serde_json::to_string(slot.identity()).unwrap());if slot.missing(){source_lower.optional.insert(id.clone());}if slot.logical_type().nullable{source_lower.native_null.insert(id);}}
    }
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
    let mut order = source
        .order
        .iter()
        .map(|field| source_lower.expression(&application::legacy(field)))
        .collect::<Result<Vec<_>>>()?;
    #[cfg(feature="sdk-ordered-page-candidate")]
    if let Some(slots)=navigation.source_order(){order=slots.iter().map(|slot|{
        let field=app::Field{scan:source.source.occurrence.clone(),identity:slot.identity().clone(),logical_type:slot.logical_type().clone(),span:weft_core::ir::Span{start:0,end:0}};
        source_lower.expression(&application::legacy(&field)).map(|value|format!("{value} {} NULLS {}",if slot.descending(){"DESC"}else{"ASC"},if slot.nulls_first(){"FIRST"}else{"LAST"}))
    }).collect::<Result<Vec<_>>>()?;}
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
    #[cfg(feature="sdk-ordered-page-candidate")]
    let source_key_obligation=if let Some(selected)=navigation.source_selected_key(){
        let key=source.page_key.as_ref().ok_or_else(||fail("WFT-BINDING","Original source selected Key required"))?;
        let fields=key.fields.iter().zip(&key.types).map(|(identity,ty)|source_lower.expression(&Expression::Field{scan:source.source.occurrence.clone(),identity:identity.clone(),logical_type:ty.clone(),span:weft_core::ir::Span{start:0,end:0}})).collect::<Result<Vec<_>>>()?;
        let fields=fields.join(", ");let count=count_sql();let sql=format!("WITH {} SELECT CAST({count} AS STRING) AS violations FROM (SELECT {fields} FROM {} GROUP BY {fields} HAVING {count}>1) source_duplicate_keys",source_lower.ctes.join(", "),binding::quote(&source.source.occurrence));
        Some(Obligation{id:"ashlar.candidate.sourceKeyIntegrity".into(),parameters:json!({"version":"ashlar.navigation-source-key/0.1","phase":"before-user-query","sourceRecord":source.source.record,"selectedKey":selected,"key":key,"checks":[{"sql":sql,"failureCode":"WFT-BINDING"}],"success":"one exact STRING count equal to 0","samePublicationRequired":true,"authority":"complete authorized original source Record and hidden ordering Fields"}),owner:ObligationOwner::Host,failure_code:"WFT-BINDING".into()})
    }else{None};
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
    let mut obligations=admitted.obligations;
    #[cfg(feature="sdk-ordered-page-candidate")]
    if let Some(obligation)=source_key_obligation{obligations.push(obligation);}
    Ok(Some((membership, obligations)))
}

/// A repeated identical whole-publication obligation is one obligation. A
/// distinct source meaning must never disappear into an ID-indexed host map.
pub(super) fn merge_source_obligations(
    target: &mut Vec<Obligation>,
    source: Vec<Obligation>,
) -> Result<()> {
    for obligation in source {
        if let Some(existing) = target.iter().find(|item| item.id == obligation.id) {
            if existing != &obligation {
                return Err(fail(
                    "WFT-OBLIGATION",
                    "Distinct source and target obligations share an unsupported ID",
                ));
            }
        } else {
            target.push(obligation);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn original() -> Obligation {
        Obligation {
            id: "ashlar.candidate.publication".into(),
            parameters: json!({"publication":"original","modelPins":[{"revision":"owned"}]}),
            owner: ObligationOwner::Host,
            failure_code: "WFT-OBLIGATION".into(),
        }
    }
    #[test]
    fn identical_source_meaning_deduplicates_but_distinct_meaning_refuses() {
        let first = original();
        let mut target = vec![first.clone()];
        merge_source_obligations(&mut target, vec![first.clone()]).unwrap();
        assert_eq!(target, vec![first.clone()]);
        for mode in 0..3 {
            let mut different = first.clone();
            match mode {
                0 => different.parameters["publication"] = json!("other"),
                1 => different.owner = ObligationOwner::Backend,
                _ => different.failure_code = "OTHER".into(),
            };
            assert!(merge_source_obligations(&mut target, vec![different]).is_err());
            assert_eq!(target, vec![first.clone()]);
        }
        let mut additional = first.clone();
        additional.id = "source-specific-native-proof".into();
        merge_source_obligations(&mut target, vec![additional.clone()]).unwrap();
        assert_eq!(target, vec![first, additional]);
    }
}
