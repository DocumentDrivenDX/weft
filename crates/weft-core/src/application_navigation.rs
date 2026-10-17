//! Original source entity-set navigation; native object/edge SQL belongs to backends.
use crate::{
    application_ir::{Navigation, Plan},
    application_resolve::{ExactParameters, Parameters},
    error::{Diagnostic, Result},
    model::Catalog,
};
#[derive(Debug, Clone)]
pub struct Input {
    pub source_sql: String,
    pub relationship_module: String,
    pub relationship_id: String,
    pub inverse: bool,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-ENGINE-NAVIGATION", "resolve", message)
}
pub(crate) fn prepare(
    catalog: &Catalog,
    input: &Input,
    parameters: Parameters,
    exact: ExactParameters,
    timestamp_equality: bool,
    exact_equality: bool,
    target: &Plan,
) -> Result<Navigation> {
    if input.source_sql.len() > 65536
        || input.relationship_module.is_empty()
        || input.relationship_id.is_empty()
    {
        return Err(fail(
            "Bounded original source SQL and canonical relationship identity required",
        ));
    }
    let query = crate::application_syntax::parse(&input.source_sql)?;
    let source = if exact_equality {
        crate::application_resolve::resolve_with_exact(
            catalog,
            query,
            parameters,
            exact,
            None,
            timestamp_equality,
        )?
    } else {
        if !exact.is_empty() {
            return Err(fail("Original source exact-parameter profile differs"));
        }
        crate::application_resolve::resolve(catalog, query, parameters, None)?
    };
    if source.aggregate || !source.groups.is_empty() || source.navigation.is_some() {
        return Err(fail(
            "Navigation source must retain ungrouped original entity identity",
        ));
    }
    if source.module_pins != target.module_pins {
        return Err(fail("Navigation source/target model pins differ"));
    }
    let record = catalog.record_by_identity(&source.source.record)?;
    let relationship = catalog.relationship_read_identity(
        &record,
        &input.relationship_module,
        &input.relationship_id,
        input.inverse,
    )?;
    if relationship.from != source.source.record || relationship.to != target.source.record {
        return Err(fail(
            "Original directed navigation endpoints differ from source/target Records",
        ));
    }
    Ok(Navigation::new(source, relationship))
}

pub(crate) struct Partition {
    pub source: Parameters,
    pub source_exact: ExactParameters,
    pub target: Parameters,
    pub target_exact: ExactParameters,
}
pub(crate) fn partition(
    source: &crate::application_syntax::Query,
    target: &crate::application_syntax::Query,
    parameters: Parameters,
    exact: ExactParameters,
) -> Result<Partition> {
    use crate::application_syntax::{Predicate, Query, Value};
    use std::collections::BTreeSet;
    fn names(query: &Query) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        for p in query
            .predicates
            .iter()
            .chain(query.joins.iter().flat_map(|(_, on)| on))
        {
            let values = match p {
                Predicate::Compare { values, .. } => values.as_slice(),
                Predicate::HasRelated { key, .. } => key.as_slice(),
                Predicate::Boolean { .. } => &[],
            };
            for value in values {
                if let Value::Parameter(name) = value {
                    names.insert(name.value.to_ascii_lowercase());
                }
            }
        }
        names
    }
    let source = names(source);
    let target = names(target);
    let used = source.union(&target).cloned().collect::<BTreeSet<_>>();
    let mut declared = BTreeSet::new();
    if parameters.len() + exact.len() > 1024 {
        return Err(fail("Combined navigation parameter count exceeds1024"));
    }
    for name in parameters.keys().chain(exact.keys()) {
        if !name
            .as_bytes()
            .first()
            .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || !declared.insert(name.to_ascii_lowercase())
        {
            return Err(fail(
                "Navigation parameters must retain distinct ASCII identities after case folding",
            ));
        }
    }
    if used != declared {
        return Err(fail(
            "Navigation source/target parameter union differs from complete original inventory",
        ));
    }
    Ok(Partition {
        source: parameters
            .iter()
            .filter(|(name, _)| source.contains(&name.to_ascii_lowercase()))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect(),
        source_exact: exact
            .iter()
            .filter(|(name, _)| source.contains(&name.to_ascii_lowercase()))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect(),
        target: parameters
            .iter()
            .filter(|(name, _)| target.contains(&name.to_ascii_lowercase()))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect(),
        target_exact: exact
            .iter()
            .filter(|(name, _)| target.contains(&name.to_ascii_lowercase()))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect(),
    })
}
