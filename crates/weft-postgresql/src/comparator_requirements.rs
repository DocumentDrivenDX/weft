//! Operation requirements derived from resolved query uses, independent of homes.
use crate::native_comparator_definition::{Definition, Operation};
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    application_ir as app,
    application_model::{AuthoredKey, RelationshipRead},
    backend::Plan,
    error::{Diagnostic, Result},
    ir::{Expression, Identity, LogicalType, Node},
};
#[derive(Debug)]
pub struct Requirement {
    pub identity: Identity,
    pub logical_type: LogicalType,
    pub operations: BTreeSet<Operation>,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "lower", message)
}
fn add(
    all: &mut BTreeMap<String, Requirement>,
    identity: &Identity,
    logical: &LogicalType,
    operation: Operation,
) -> Result<()> {
    let key = serde_json::to_string(identity)
        .map_err(|_| fail("Comparator identity could not be encoded"))?;
    let entry = all.entry(key).or_insert_with(|| Requirement {
        identity: identity.clone(),
        logical_type: logical.clone(),
        operations: BTreeSet::new(),
    });
    if &entry.logical_type != logical {
        return Err(fail(
            "Repeated comparator identity has inconsistent resolved types",
        ));
    }
    entry.operations.insert(operation);
    Ok(())
}
fn key(
    all: &mut BTreeMap<String, Requirement>,
    key: &AuthoredKey,
    operations: &[Operation],
) -> Result<()> {
    if key.fields.len() != key.types.len() {
        return Err(fail("Resolved key type arity differs"));
    }
    for (identity, logical) in key.fields.iter().zip(&key.types) {
        for operation in operations {
            add(all, identity, logical, *operation)?;
        }
    }
    Ok(())
}
fn relationship(
    all: &mut BTreeMap<String, Requirement>,
    r: &RelationshipRead,
    ordered_target: bool,
) -> Result<()> {
    key(all, &r.source_key, &[Operation::Key, Operation::Equality])?;
    key(all, &r.target_key, &[Operation::Key, Operation::Equality])?;
    if ordered_target {
        key(all, &r.target_key, &[Operation::Ordering])?;
    }
    Ok(())
}
fn predicate(all: &mut BTreeMap<String, Requirement>, predicate: &app::Predicate) -> Result<()> {
    match predicate {
        app::Predicate::Equal { left, right } => {
            add(all, &left.identity, &left.logical_type, Operation::Equality)?;
            if let app::Value::Field { field } = right {
                add(
                    all,
                    &field.identity,
                    &field.logical_type,
                    Operation::Equality,
                )?;
            }
        }
        app::Predicate::LexicographicGreater { columns, .. } => {
            for field in columns {
                for operation in [Operation::Equality, Operation::Ordering] {
                    add(all, &field.identity, &field.logical_type, operation)?;
                }
            }
        }
        app::Predicate::HasRelated {
            relationship: r, ..
        } => relationship(all, r, false)?,
    }
    Ok(())
}
pub fn collect(plan: Plan<'_>) -> Result<Vec<Requirement>> {
    let mut all = BTreeMap::new();
    match plan {
        Plan::V01(plan) => {
            let mut nodes = vec![&plan.root];
            let mut expressions = vec![];
            while let Some(node) = nodes.pop() {
                match node {
                    Node::Scan { .. } => {}
                    Node::InnerJoin { left, right, on } => {
                        nodes.extend([left.as_ref(), right.as_ref()]);
                        expressions.push((on, None));
                    }
                    Node::Filter { input, predicate } => {
                        nodes.push(input);
                        expressions.push((predicate, None));
                    }
                    Node::Aggregate {
                        input,
                        groups,
                        aggregates,
                    } => {
                        nodes.push(input);
                        for group in groups {
                            expressions.push((group, Some(Operation::Equality)));
                        }
                        for aggregate in aggregates {
                            expressions.push((aggregate, None));
                        }
                    }
                    Node::Project { input, outputs } => {
                        nodes.push(input);
                        for output in outputs {
                            expressions.push((&output.expression, None));
                        }
                    }
                }
            }
            while let Some((expression, operation)) = expressions.pop() {
                match expression {
                    Expression::Field {
                        identity,
                        logical_type,
                        ..
                    } => {
                        if let Some(operation) = operation {
                            add(&mut all, identity, logical_type, operation)?;
                        }
                    }
                    Expression::Literal { .. } => {}
                    Expression::Equal { left, right, .. } => expressions.extend([
                        (left.as_ref(), Some(Operation::Equality)),
                        (right.as_ref(), Some(Operation::Equality)),
                    ]),
                    Expression::And { left, right, .. } => expressions
                        .extend([(left.as_ref(), operation), (right.as_ref(), operation)]),
                    Expression::Sum { argument, .. } => {
                        expressions.push((argument, Some(Operation::Sum)))
                    }
                }
            }
        }
        Plan::V02(plan) => {
            for join in &plan.joins {
                for p in &join.on {
                    predicate(&mut all, p)?;
                }
            }
            for p in &plan.filters {
                predicate(&mut all, p)?;
            }
            for field in &plan.groups {
                add(
                    &mut all,
                    &field.identity,
                    &field.logical_type,
                    Operation::Equality,
                )?;
            }
            for field in &plan.order {
                add(
                    &mut all,
                    &field.identity,
                    &field.logical_type,
                    Operation::Ordering,
                )?;
            }
            if let Some(page_key) = &plan.page_key {
                key(&mut all, page_key, &[Operation::Key, Operation::Ordering])?;
            }
            for output in &plan.outputs {
                match &output.expression {
                    app::Expression::Sum { argument, .. } => add(
                        &mut all,
                        &argument.identity,
                        &argument.logical_type,
                        Operation::Sum,
                    )?,
                    app::Expression::RelatedKeys {
                        relationship: r, ..
                    } => relationship(&mut all, r, true)?,
                    app::Expression::Field { .. } | app::Expression::Count { .. } => {}
                }
            }
        }
    }
    Ok(all.into_values().collect())
}
/// Called before SQL lowering with a trusted backend's selected definitions.
pub fn admit(
    requirements: &[Requirement],
    definitions: &BTreeMap<String, Definition>,
) -> Result<()> {
    for requirement in requirements {
        let key = serde_json::to_string(&requirement.identity)
            .map_err(|_| fail("Comparator identity could not be encoded"))?;
        let definition = definitions.get(&key).ok_or_else(|| {
            Diagnostic::new(
                "WFT-CAPABILITY",
                "lower",
                "Requested field operation has no selected comparator",
            )
        })?;
        definition.require_type(&requirement.logical_type)?;
        for operation in &requirement.operations {
            definition.require(*operation)?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use weft_core::{
        application_resolve, application_syntax,
        model::{Catalog, ModuleInput},
    };
    #[test]
    fn actual_application_corpus_resolves_home_independent_requirements() {
        let cases: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/application-cases.json"
        ))
        .unwrap();
        let mut ordered = 0;
        let mut sums = 0;
        let mut keys = 0;
        for case in cases {
            let request = &case["request"];
            let inputs: Vec<ModuleInput> =
                serde_json::from_value(request["modules"].clone()).unwrap();
            let catalog = Catalog::prepare(inputs).unwrap();
            let parameters = serde_json::from_value(
                request
                    .get("parameters")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({})),
            )
            .unwrap();
            let profile = serde_json::from_value(request["readProfile"].clone()).unwrap();
            let plan = application_resolve::resolve(
                &catalog,
                application_syntax::parse(request["sql"].as_str().unwrap()).unwrap(),
                parameters,
                profile,
            )
            .unwrap();
            let requirements = collect(Plan::V02(&plan)).unwrap();
            for requirement in &requirements {
                ordered += usize::from(requirement.operations.contains(&Operation::Ordering));
                sums += usize::from(requirement.operations.contains(&Operation::Sum));
                keys += usize::from(requirement.operations.contains(&Operation::Key));
            }
            if !requirements.is_empty() {
                assert!(admit(&requirements, &BTreeMap::new()).is_err());
            }
        }
        assert!(ordered > 0 && sums > 0 && keys > 0);
    }
    #[test]
    fn plain_projection_does_not_invent_comparison_and_grouping_requires_equality() {
        let cases: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/compiler-cases.json"
        ))
        .unwrap();
        let inputs: Vec<ModuleInput> =
            serde_json::from_value(cases[0]["request"]["modules"].clone()).unwrap();
        let catalog = Catalog::prepare(inputs).unwrap();
        let query_text = "SELECT c.name FROM Customer c";
        let (_, plan) =
            weft_core::prepare_and_resolve(&query_text, catalog.inputs.clone()).unwrap();
        assert!(collect(Plan::V01(&plan)).unwrap().is_empty());
        let query_text = "SELECT c.name, SUM(o.total) AS total FROM Customer c JOIN Orders o ON o.customer_id = c.id GROUP BY c.name";
        let (_, plan) =
            weft_core::prepare_and_resolve(&query_text, catalog.inputs.clone()).unwrap();
        let requirements = collect(Plan::V01(&plan)).unwrap();
        assert_eq!(
            requirements
                .iter()
                .filter(|r| r.operations.contains(&Operation::Equality))
                .count(),
            3
        );
        assert_eq!(
            requirements
                .iter()
                .filter(|r| r.operations.contains(&Operation::Sum))
                .count(),
            1
        );
    }
    #[test]
    fn selected_field_comparator_requires_exact_type_and_every_requested_operation() {
        use crate::{
            leaf_codec_definition::OriginalArtifact, native_comparator_definition::Selection,
        };
        use base64::{engine::general_purpose::STANDARD, Engine};
        use serde_json::json;
        use weft_core::{ir::Family, json::sha256};
        let pin = json!({"identity":"fixture","version":"0.1.0","sha256":sha256(b"{}")});
        let artifact = json!({"identity":"fixture","bytesBase64":STANDARD.encode(b"{}"),"sha256":sha256(b"{}")});
        let value = json!({"interfaceVersion":"truss-native-comparator/0.1.0","profile":pin,"valueDefinition":artifact,"sourceDomainDefinition":artifact,"nativeDomainProfile":pin,"nativeDomainDefinition":artifact,"operatorInventory":artifact,"strategy":{"kind":"unicode-text-C","nativeType":"pg_catalog.text","encoding":"UTF8","collation":"pg_catalog.C","normalization":"none"},"castOutcome":"exact-or-error","nullOperands":"refuse","absentOperands":"refuse","qualification":artifact});
        let originals = [
            "valueDefinition",
            "sourceDomainDefinition",
            "nativeDomainDefinition",
            "operatorInventory",
            "qualification",
        ]
        .into_iter()
        .map(|key| {
            (
                key.into(),
                OriginalArtifact {
                    identity: "fixture".into(),
                    bytes: b"{}".to_vec(),
                },
            )
        })
        .collect();
        let logical = LogicalType {
            family: Family::String,
            facets: json!({}),
            nullable: false,
        };
        let definition = Definition::parse(
            &value.to_string(),
            Selection {
                profile: &pin,
                native_profile: &pin,
                original_artifacts: &originals,
                operations: &BTreeSet::from([Operation::Equality]),
            },
            &logical,
        )
        .unwrap();
        let identity = Identity {
            document_id: "d".into(),
            revision: "1".into(),
            module: "m".into(),
            element: "field".into(),
        };
        let definitions = BTreeMap::from([(serde_json::to_string(&identity).unwrap(), definition)]);
        let mut requirements = vec![Requirement {
            identity,
            logical_type: logical,
            operations: BTreeSet::from([Operation::Equality]),
        }];
        admit(&requirements, &definitions).unwrap();
        requirements[0].operations.insert(Operation::Ordering);
        assert!(admit(&requirements, &definitions).is_err());
        requirements[0].operations.remove(&Operation::Ordering);
        requirements[0].logical_type.family = Family::Boolean;
        assert!(admit(&requirements, &definitions).is_err());
    }
}
