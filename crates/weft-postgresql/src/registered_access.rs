//! Context-admitted physical access planning; codec/result and relational emission follow.
use crate::{
    comparator_requirements::{admit_context, collect_reads, registration_key},
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
pub struct Access {
    pub scan: String,
    pub field: Identity,
    pub owner_alias: Identifier,
    pub location: Location,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "lower", message)
}
/// Requests are backend-owned relational access choices, never SQL/model plugins.
/// All property/context/comparator gates complete before parameters are committed.
pub fn lower(
    context: &Context<'_>,
    properties: &BTreeMap<String, PropertyAdmission>,
    comparators: &BTreeMap<String, Definition>,
    requests: &[Request],
    parameters: &mut Parameters,
) -> Result<Vec<Access>> {
    admit_context(context, properties, comparators)?;
    let reads = collect_reads(context.plan)?;
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
    for (index, request) in requests.iter().enumerate() {
        let owner = scans
            .get(&request.scan)
            .ok_or_else(|| fail("Access request has no original scan"))?;
        if !reads.contains(&(owner.clone(), request.field.clone())) {
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
        result.push(Access {
            scan: request.scan.clone(),
            field: request.field.clone(),
            owner_alias: alias.clone(),
            location,
        });
    }
    *parameters = staged;
    Ok(result)
}
