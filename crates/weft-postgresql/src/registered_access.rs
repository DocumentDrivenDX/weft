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
    pub owner: Identity,
    pub owner_alias: Identifier,
    pub owner_source: std::sync::Arc<crate::property_definition::OwnerSource>,
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
    let mut owner_sources: BTreeMap<
        String,
        (
            crate::property_definition::OwnerMapping,
            String,
            std::sync::Arc<crate::property_definition::OwnerSource>,
        ),
    > = BTreeMap::new();
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
        let mapping = property.home.owner_mapping();
        let owner_source = if let Some((original_mapping, original_id, source)) =
            owner_sources.get(&request.scan)
        {
            if original_mapping != &mapping || original_id != &property.owner_catalog_id {
                return Err(fail(
                    "Property homes disagree on the original scan owner mapping",
                ));
            }
            std::sync::Arc::clone(source)
        } else {
            let source = std::sync::Arc::new(mapping.source(
                &namespace,
                alias,
                &property.owner_catalog_id,
                &mut staged,
            )?);
            owner_sources.insert(
                request.scan.clone(),
                (
                    mapping,
                    property.owner_catalog_id.clone(),
                    std::sync::Arc::clone(&source),
                ),
            );
            source
        };
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
            owner: owner.clone(),
            owner_alias: alias.clone(),
            owner_source,
            location,
            scalar_storage,
            value_layout,
        });
    }
    *parameters = staged;
    Ok(result)
}
/// Physical scan assembly, with integrity prerequisites deliberately separate.
#[derive(Debug)]
pub struct PhysicalScan {
    pub source: crate::relational::Source,
    pub structural_integrity: Vec<String>,
    /// Must observe zero violations before query execution/publication under the
    /// same admitted complete visibility and transaction context. These inspect
    /// all selected owners independently of query joins/filters/limits.
    pub structural_check_sql: Vec<String>,
}
pub fn scan_source(node: &Node, accesses: &[Access<'_>]) -> Result<PhysicalScan> {
    let Node::Scan {
        occurrence, record, ..
    } = node
    else {
        return Err(fail("Physical scan requires an original scan node"));
    };
    let selected: Vec<_> = accesses
        .iter()
        .filter(|access| &access.scan == occurrence)
        .collect();
    let first = *selected.first().ok_or_else(|| {
        Diagnostic::new(
            "WFT-CAPABILITY",
            "lower",
            "Fieldless scan needs independent original record mapping",
        )
    })?;
    let mut fields = BTreeSet::new();
    let mut joins = Vec::new();
    let mut integrity = BTreeSet::new();
    for access in selected {
        if &access.owner != record
            || access.owner_alias != first.owner_alias
            || access.owner_source.sql != first.owner_source.sql
            || access.owner_source.discriminator != first.owner_source.discriminator
        {
            return Err(fail(
                "Selected accesses disagree on original scan ownership",
            ));
        }
        if !fields.insert(
            serde_json::to_string(&access.field)
                .map_err(|_| fail("Access identity encoding refused"))?,
        ) {
            return Err(fail("Duplicate scan field access"));
        }
        match &access.location {
            Location::Props(location) => {
                integrity.insert(location.root_integrity.clone());
            }
            Location::Row(location) => {
                joins.extend(location.joins.iter().cloned());
                integrity.insert(location.structural_integrity.clone());
            }
        }
    }
    let sql = if joins.is_empty() {
        first.owner_source.sql.clone()
    } else {
        format!("({} {})", first.owner_source.sql, joins.join(" "))
    };
    let structural_integrity: Vec<_> = integrity.into_iter().collect();
    let structural_check_sql = structural_integrity
        .iter()
        .map(|check| {
            format!(
        "SELECT count(*) AS violations FROM {sql} WHERE {} AND ({check}) IS DISTINCT FROM TRUE",
        first.owner_source.discriminator,
    )
        })
        .collect();
    Ok(PhysicalScan {
        source: crate::relational::Source {
            sql,
            filters: vec![first.owner_source.discriminator.clone()],
            groups: vec![],
            aggregated: false,
        },
        structural_integrity,
        structural_check_sql,
    })
}
/// Render a typed expression with exact occurrence-qualified physical accesses.
/// The trusted native callback still owns codecs, literals and operations. A
/// field cannot use another self-join occurrence's location or an unselected
/// access. This does not certify stored domains or result publication.
pub fn render_expression<'a>(
    expression: &weft_core::ir::Expression,
    accesses: &[Access<'a>],
    parameters: &mut Parameters,
    mut native: impl FnMut(
        &weft_core::ir::Expression,
        &[String],
        Option<&Access<'a>>,
        &mut Parameters,
    ) -> Result<String>,
) -> Result<String> {
    let mut indexed = BTreeMap::new();
    for access in accesses {
        let key = (
            access.scan.clone(),
            serde_json::to_string(&access.field)
                .map_err(|_| fail("Access identity encoding refused"))?,
        );
        if indexed.insert(key, access).is_some() {
            return Err(fail("Duplicate expression physical access"));
        }
    }
    crate::expression::render(expression, parameters, |node, operands, parameters| {
        let access =
            if let weft_core::ir::Expression::Field { scan, identity, .. } = node {
                let key = (
                    scan.clone(),
                    serde_json::to_string(identity)
                        .map_err(|_| fail("Expression identity encoding refused"))?,
                );
                Some(*indexed.get(&key).ok_or_else(|| {
                    fail("Expression field has no exact selected occurrence access")
                })?)
            } else {
                None
            };
        native(node, operands, access, parameters)
    })
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
