//! Relational source staging over backend-owned scan and expression meanings.
use crate::Parameters;
use weft_core::{
    error::{Diagnostic, Result},
    ir::{Expression, Node},
};
#[derive(Debug)]
pub struct Source {
    pub sql: String,
    pub filters: Vec<String>,
    pub groups: Vec<String>,
    pub aggregated: bool,
}
fn unsupported(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-CAPABILITY", "lower", message)
}
/// Source callbacks own physical mapping. Integrity prerequisites stay outside
/// filters. Native expression callbacks own qualified operator/literal meaning.
pub fn assemble(
    root: &Node,
    parameters: &mut Parameters,
    mut scan: impl FnMut(&Node, &mut Parameters) -> Result<String>,
    mut expression: impl FnMut(&Expression, &mut Parameters) -> Result<String>,
) -> Result<Source> {
    let mut staged = parameters.clone();
    let mut pending = vec![(root, false)];
    let mut sources = Vec::new();
    while let Some((node, ready)) = pending.pop() {
        if !ready {
            pending.push((node, true));
            match node {
                Node::Scan { .. } => {}
                Node::InnerJoin { left, right, .. } => {
                    pending.push((right, false));
                    pending.push((left, false));
                }
                Node::Filter { input, .. } | Node::Aggregate { input, .. } => {
                    pending.push((input, false))
                }
                Node::Project { .. } => {
                    return Err(unsupported("Nested project requires another target stage"))
                }
            }
        } else {
            let source = match node {
                Node::Scan { .. } => Source {
                    sql: scan(node, &mut staged)?,
                    filters: vec![],
                    groups: vec![],
                    aggregated: false,
                },
                Node::InnerJoin { on, .. } => {
                    let right: Source = sources.pop().expect("right source");
                    let mut left: Source = sources.pop().expect("left source");
                    if left.aggregated || right.aggregated {
                        return Err(unsupported(
                            "Join over aggregate requires another target stage",
                        ));
                    }
                    left.filters.extend(right.filters);
                    Source {
                        sql: format!(
                            "({} INNER JOIN {} ON {})",
                            left.sql,
                            right.sql,
                            expression(on, &mut staged)?
                        ),
                        filters: left.filters,
                        groups: vec![],
                        aggregated: false,
                    }
                }
                Node::Filter { predicate, .. } => {
                    let mut source: Source = sources.pop().expect("input source");
                    if source.aggregated {
                        return Err(unsupported(
                            "Filter over aggregate requires another target stage",
                        ));
                    }
                    source.filters.push(expression(predicate, &mut staged)?);
                    source
                }
                Node::Aggregate { groups, .. } => {
                    let mut source: Source = sources.pop().expect("input source");
                    if source.aggregated {
                        return Err(unsupported(
                            "Repeated aggregate requires another target stage",
                        ));
                    }
                    source.groups = groups
                        .iter()
                        .map(|group| expression(group, &mut staged))
                        .collect::<Result<_>>()?;
                    source.aggregated = true;
                    source
                }
                Node::Project { .. } => unreachable!("refused before descendants"),
            };
            sources.push(source);
        }
    }
    *parameters = staged;
    Ok(sources.pop().expect("root source"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source(sql: &str) -> Node {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/compiler-cases.json"
        ))
        .unwrap();
        let inputs = serde_json::from_value(cases[0]["request"]["modules"].clone()).unwrap();
        let (_, plan) = weft_core::prepare_and_resolve(sql, inputs).unwrap();
        match plan.root {
            Node::Project { input, .. } => *input,
            _ => panic!("project"),
        }
    }
    const JOIN: &str = "SELECT c.name, SUM(o.total) AS total FROM Customer c JOIN Orders o ON o.customer_id = c.id GROUP BY c.name";
    #[test]
    fn grouped_join_preserves_order_and_backend_meanings() {
        let node = source(JOIN);
        let mut parameters = Parameters::default();
        let scans = std::cell::RefCell::new(Vec::new());
        let result = assemble(
            &node,
            &mut parameters,
            |node, _| {
                if let Node::Scan { record, .. } = node {
                    scans.borrow_mut().push(record.element.clone());
                    Ok(format!("{}_native", record.element))
                } else {
                    panic!("scan")
                }
            },
            |expression, _| {
                Ok(match expression {
                    Expression::Equal { .. } => "join_native".into(),
                    Expression::Field { .. } => "group_native".into(),
                    _ => panic!("expression"),
                })
            },
        )
        .unwrap();
        assert_eq!(*scans.borrow(), ["customer", "orders"]);
        assert_eq!(
            result.sql,
            "(customer_native INNER JOIN orders_native ON join_native)"
        );
        assert_eq!(result.groups, ["group_native"]);
        assert!(result.filters.is_empty());
        assert!(result.aggregated);
    }
    #[test]
    fn late_join_refusal_discards_all_source_parameters() {
        let node = source(JOIN);
        let mut parameters = Parameters::default();
        let mut scans = 0;
        let result = assemble(
            &node,
            &mut parameters,
            |_, parameters| {
                scans += 1;
                let logical_type = weft_core::ir::LogicalType {
                    family: weft_core::ir::Family::String,
                    facets: serde_json::json!({}),
                    nullable: false,
                };
                parameters.push(
                    logical_type,
                    "selected".into(),
                    serde_json::json!({"use":"test-source"}),
                )?;
                Ok("native_source".into())
            },
            |_, _| Err(unsupported("Unregistered native join")),
        );
        assert_eq!(scans, 2);
        assert!(result.is_err());
        assert!(parameters.into_slots().is_empty());
    }
    #[test]
    fn global_aggregate_cannot_be_flattened_into_join_or_filter() {
        let aggregate = source("SELECT SUM(o.total) AS total FROM Orders o");
        let joined = source(JOIN);
        let (right, on) = match joined {
            Node::Aggregate { input, .. } => match *input {
                Node::InnerJoin { right, on, .. } => (right, on),
                _ => panic!("join"),
            },
            _ => panic!("aggregate"),
        };
        let join = Node::InnerJoin {
            left: Box::new(aggregate.clone()),
            right,
            on: on.clone(),
        };
        let filter = Node::Filter {
            input: Box::new(aggregate),
            predicate: on,
        };
        for node in [join, filter] {
            let mut parameters = Parameters::default();
            assert!(assemble(
                &node,
                &mut parameters,
                |_, _| Ok("source".into()),
                |_, _| Ok("predicate".into())
            )
            .is_err());
            assert!(parameters.into_slots().is_empty());
        }
    }
}
