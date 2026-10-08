//! Shared expression traversal. Registered backends own every native meaning.
use crate::Parameters;
use weft_core::{error::Result, ir::Expression};
/// The callback receives the original typed node and ordered rendered operands.
/// It owns field access, literal conversion and qualified operator selection.
/// No source/model content loads a callback. Parameters commit only on success.
pub fn render(
    root: &Expression,
    parameters: &mut Parameters,
    mut native: impl FnMut(&Expression, &[String], &mut Parameters) -> Result<String>,
) -> Result<String> {
    let mut staged = parameters.clone();
    let mut pending = vec![(root, false)];
    let mut values = Vec::new();
    while let Some((node, ready)) = pending.pop() {
        if !ready {
            pending.push((node, true));
            match node {
                Expression::Equal { left, right, .. } | Expression::And { left, right, .. } => {
                    pending.push((right, false));
                    pending.push((left, false));
                }
                Expression::Sum { argument, .. } => pending.push((argument, false)),
                Expression::Field { .. } | Expression::Literal { .. } => {}
            }
        } else {
            let count = match node {
                Expression::Equal { .. } | Expression::And { .. } => 2,
                Expression::Sum { .. } => 1,
                _ => 0,
            };
            let operands = values.split_off(values.len() - count);
            values.push(native(node, &operands, &mut staged)?);
        }
    }
    *parameters = staged;
    Ok(values.pop().expect("root produces one expression"))
}
#[cfg(test)]
mod tests {
    use super::*;
    use weft_core::{error::Diagnostic, ir::Node};
    #[test]
    fn typed_callback_order_and_late_refusal_preserve_atomic_parameters() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/compiler-cases.json"
        ))
        .unwrap();
        let inputs = serde_json::from_value(cases[0]["request"]["modules"].clone()).unwrap();
        let (_, plan) = weft_core::prepare_and_resolve(
            "SELECT c.name FROM Customer c WHERE c.name = 'selected'",
            inputs,
        )
        .unwrap();
        let expression = match &plan.root {
            Node::Project { input, .. } => match input.as_ref() {
                Node::Filter { predicate, .. } => predicate,
                _ => panic!("fixture filter"),
            },
            _ => panic!("fixture project"),
        };
        let mut parameters = Parameters::default();
        let mut trace = Vec::new();
        let result = render(expression, &mut parameters, |node, operands, _| {
            Ok(match node {
                Expression::Field { .. } => {
                    trace.push("field");
                    assert!(operands.is_empty());
                    "left_native".into()
                }
                Expression::Literal { value, .. } => {
                    trace.push("literal");
                    assert_eq!(value, "selected");
                    "right_native".into()
                }
                Expression::Equal { .. } => {
                    trace.push("equal");
                    assert_eq!(operands, ["left_native", "right_native"]);
                    format!("({} = {})", operands[0], operands[1])
                }
                _ => panic!("fixture node"),
            })
        })
        .unwrap();
        assert_eq!(trace, vec!["field", "literal", "equal"]);
        assert_eq!(result, "(left_native = right_native)");
        assert!(render(
            expression,
            &mut parameters,
            |node, _, parameters| match node {
                Expression::Field { logical_type, .. } => parameters.push(
                    logical_type.clone(),
                    "member".into(),
                    serde_json::json!({"use":"test-field-member"})
                ),
                _ => Err(Diagnostic::new(
                    "WFT-CAPABILITY",
                    "lower",
                    "Missing selected native meaning"
                )),
            }
        )
        .is_err());
        assert!(parameters.into_slots().is_empty());
    }
}
