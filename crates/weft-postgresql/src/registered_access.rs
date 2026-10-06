//! Context-admitted physical access planning; codec/result and relational emission follow.
use crate::{
    comparator_requirements::{admit_context, registration_key},
    native_comparator_definition::Definition,
    property_definition::{HomeAdmission, PropertyAdmission, PropsLocation},
    row_join_definition::RootLocation,
    Identifier, Parameters,
};
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    backend::{Context, Plan},
    error::{Diagnostic, Result},
    ir::{Identity, Node},
};
#[derive(Debug, Clone)]
pub struct Request {
    pub scan: String,
    pub field: Identity,
}
#[derive(Debug)]
pub enum Location {
    Props(PropsLocation),
    Row(RootLocation),
}
#[derive(Debug)]
pub struct Access<'a> {
    pub scan: String,
    pub field: Identity,
    pub owner_alias: Identifier,
    pub location: Location,
    pub scalar_storage: Option<crate::property_definition::ScalarStorage>,
    pub value_layout: std::sync::Arc<crate::value_definition::Layout<'a>>,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "lower", message)
}
/// Exact outer-scan accesses, preserving occurrence identity in self-joins.
/// Relationship target subqueries need their separately scoped access plan.
pub fn requests(plan: Plan<'_>) -> Result<Vec<Request>> {
    use weft_core::{application_ir as app, ir::Expression};
    let mut fields: Vec<(String, Identity)> = Vec::new();
    match plan {
        Plan::V01(plan) => {
            let mut nodes = vec![&plan.root];
            let mut expressions = Vec::new();
            while let Some(node) = nodes.pop() {
                match node {
                    Node::Scan { .. } => {}
                    Node::InnerJoin { left, right, on } => {
                        nodes.extend([left.as_ref(), right.as_ref()]);
                        expressions.push(on);
                    }
                    Node::Filter { input, predicate } => {
                        nodes.push(input);
                        expressions.push(predicate);
                    }
                    Node::Aggregate {
                        input,
                        groups,
                        aggregates,
                    } => {
                        nodes.push(input);
                        expressions.extend(groups);
                        expressions.extend(aggregates);
                    }
                    Node::Project { input, outputs } => {
                        nodes.push(input);
                        expressions.extend(outputs.iter().map(|output| &output.expression));
                    }
                }
            }
            while let Some(expression) = expressions.pop() {
                match expression {
                    Expression::Field { scan, identity, .. } => {
                        fields.push((scan.clone(), identity.clone()))
                    }
                    Expression::Equal { left, right, .. } | Expression::And { left, right, .. } => {
                        expressions.extend([left.as_ref(), right.as_ref()])
                    }
                    Expression::Sum { argument, .. } => expressions.push(argument),
                    Expression::Literal { .. } => {}
                }
            }
        }
        Plan::V02(plan) => {
            let mut add_field =
                |field: &app::Field| fields.push((field.scan.clone(), field.identity.clone()));
            let predicates = plan
                .filters
                .iter()
                .chain(plan.joins.iter().flat_map(|join| &join.on));
            for predicate in predicates {
                match predicate {
                    app::Predicate::Equal { left, right } => {
                        add_field(left);
                        if let app::Value::Field { field } = right {
                            add_field(field);
                        }
                    }
                    app::Predicate::LexicographicGreater { columns, values } => {
                        for field in columns {
                            add_field(field);
                        }
                        for value in values {
                            if let app::Value::Field { field } = value {
                                add_field(field);
                            }
                        }
                    }
                    app::Predicate::HasRelated { .. } => {}
                }
            }
            for field in plan.groups.iter().chain(&plan.order) {
                add_field(field);
            }
            for output in &plan.outputs {
                if let app::Expression::Sum { argument, .. } = &output.expression {
                    add_field(argument);
                }
            }
            for output in &plan.outputs {
                match &output.expression {
                    app::Expression::Field { scan, identity } => {
                        fields.push((scan.clone(), identity.clone()))
                    }
                    app::Expression::RelatedKeys {
                        scan, relationship, ..
                    } => {
                        for field in &relationship.source_key.fields {
                            fields.push((scan.clone(), field.clone()));
                        }
                    }
                    _ => {}
                }
            }
            for predicate in plan
                .filters
                .iter()
                .chain(plan.joins.iter().flat_map(|join| &join.on))
            {
                if let app::Predicate::HasRelated {
                    scan, relationship, ..
                } = predicate
                {
                    for field in &relationship.source_key.fields {
                        fields.push((scan.clone(), field.clone()));
                    }
                }
            }
            if let Some(key) = &plan.page_key {
                for field in &key.fields {
                    fields.push((plan.source.occurrence.clone(), field.clone()));
                }
            }
        }
    }
    let mut unique = BTreeMap::new();
    for (scan, field) in fields {
        unique.insert(
            (
                scan.clone(),
                serde_json::to_string(&field)
                    .map_err(|_| fail("Access identity encoding refused"))?,
            ),
            Request { scan, field },
        );
    }
    Ok(unique.into_values().collect())
}
pub fn lower_plan<'a>(
    context: &Context<'_>,
    properties: &'a BTreeMap<String, PropertyAdmission>,
    comparators: &BTreeMap<String, Definition>,
    parameters: &mut Parameters,
) -> Result<Vec<Access<'a>>> {
    let selected = requests(context.plan)?;
    lower(context, properties, comparators, &selected, parameters)
}
/// Requests are backend-owned relational access choices, never SQL/model plugins.
/// All property/context/comparator gates complete before parameters are committed.
pub fn lower<'a>(
    context: &Context<'_>,
    properties: &'a BTreeMap<String, PropertyAdmission>,
    comparators: &BTreeMap<String, Definition>,
    requests: &[Request],
    parameters: &mut Parameters,
) -> Result<Vec<Access<'a>>> {
    admit_context(context, properties, comparators)?;
    let reads = self::requests(context.plan)?;
    let mut scans = BTreeMap::new();
    match context.plan {
        Plan::V01(plan) => {
            let mut nodes = vec![&plan.root];
            while let Some(node) = nodes.pop() {
                match node {
                    Node::Scan {
                        occurrence, record, ..
                    } => {
                        scans.insert(occurrence.clone(), record.clone());
                    }
                    Node::InnerJoin { left, right, .. } => {
                        nodes.extend([left.as_ref(), right.as_ref()])
                    }
                    Node::Filter { input, .. }
                    | Node::Aggregate { input, .. }
                    | Node::Project { input, .. } => nodes.push(input),
                }
            }
        }
        Plan::V02(plan) => {
            scans.insert(plan.source.occurrence.clone(), plan.source.record.clone());
            for join in &plan.joins {
                scans.insert(join.right.occurrence.clone(), join.right.record.clone());
            }
        }
    }
    let aliases: BTreeMap<_, _> = scans
        .keys()
        .enumerate()
        .map(|(index, scan)| {
            Ok((
                scan.clone(),
                Identifier::new(&format!("weft_scan_{index}"))?,
            ))
        })
        .collect::<Result<_>>()?;
    let namespace = Identifier::new(
        context.binding_value["basis"]["namespace"]
            .as_str()
            .ok_or_else(|| fail("Original binding namespace is missing"))?,
    )?;
    let mut staged = parameters.clone();
    let mut accessed = BTreeSet::new();
    let mut result = Vec::new();
    let mut layouts = BTreeMap::new();
    for (index, request) in requests.iter().enumerate() {
        let owner = scans
            .get(&request.scan)
            .ok_or_else(|| fail("Access request has no original scan"))?;
        if !reads
            .iter()
            .any(|read| read.scan == request.scan && read.field == request.field)
        {
            return Err(fail("Access request is outside original selected reads"));
        }
        let key = registration_key(owner, &request.field);
        if !accessed.insert((request.scan.clone(), key.clone())) {
            return Err(fail("Duplicate physical access request"));
        }
        let property = properties
            .get(&key)
            .ok_or_else(|| fail("Access request lacks original owned property"))?;
        let alias = &aliases[&request.scan];
        let location = match &property.home {
            HomeAdmission::Props { .. } => {
                Location::Props(property.home.props_location(alias, &mut staged)?)
            }
            HomeAdmission::Row { .. } => {
                Location::Row(property.row_root_location(&namespace, alias, index, &mut staged)?)
            }
        };
        let scalar_storage = match &location {
            Location::Props(location) => property.value.props_scalar_storage(location)?,
            Location::Row(_) => None, // Row source/native codec correspondence is separate.
        };
        let value_layout = if let Some(layout) = layouts.get(&key) {
            std::sync::Arc::clone(layout)
        } else {
            let layout = std::sync::Arc::new(property.value.graph.layout()?);
            layouts.insert(key.clone(), std::sync::Arc::clone(&layout));
            layout
        };
        result.push(Access {
            scan: request.scan.clone(),
            field: request.field.clone(),
            owner_alias: alias.clone(),
            location,
            scalar_storage,
            value_layout,
        });
    }
    *parameters = staged;
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn self_join_requests_cannot_move_a_field_to_another_occurrence() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/compiler-cases.json"
        ))
        .unwrap();
        let inputs = serde_json::from_value(cases[0]["request"]["modules"].clone()).unwrap();
        let (_,plan)=weft_core::prepare_and_resolve("SELECT c.name AS customer_name, d.id AS other_id FROM Customer c JOIN Customer d ON c.id = d.id",inputs).unwrap();
        let (name_scan, name_identity, other_scan) = match &plan.root {
            Node::Project { outputs, .. } => match (&outputs[0].expression, &outputs[1].expression)
            {
                (
                    weft_core::ir::Expression::Field { scan, identity, .. },
                    weft_core::ir::Expression::Field { scan: other, .. },
                ) => (scan, identity, other),
                _ => panic!("fixture fields"),
            },
            _ => panic!("fixture project"),
        };
        let requests = requests(Plan::V01(&plan)).unwrap();
        assert_eq!(requests.len(), 3);
        assert!(requests
            .iter()
            .any(|request| &request.scan == name_scan && &request.field == name_identity));
        assert!(!requests
            .iter()
            .any(|request| &request.scan == other_scan && &request.field == name_identity));
    }
}
