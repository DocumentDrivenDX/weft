// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exact expression domains. Field admission remains owned by the model resolver.
use crate::{
    application_ir::Field,
    application_resolve::Parameter,
    arithmetic_syntax::{Expression as Syntax, Kind as SyntaxKind},
    error::Result,
    syntax::{Column, Name},
};
pub use weft_expr::arithmetic::{Domain, Expression, Kind};
pub(crate) use weft_expr::arithmetic::token_domain;
pub(crate) fn resolve<F, P>(syntax: &Syntax, field: &mut F, parameter: &mut P) -> Result<Expression>
where
    F: FnMut(&Column) -> Result<Field>,
    P: FnMut(&Name) -> Result<Parameter>,
{
    let span = syntax.span.clone();
    let (domain, kind) = match &syntax.kind {
        SyntaxKind::Field(column) => {
            let value = field(column)?;
            let domain = Domain::of_field(&value.logical_type, &span)?;
            (domain, Kind::Field { field: value })
        }
        SyntaxKind::Literal(value) => (
            token_domain(&value.value, None, &span)?,
            Kind::Literal {
                value: value.value.clone(),
            },
        ),
        SyntaxKind::Parameter(name) => {
            let value = parameter(name)?;
            (
                token_domain(&value.value, Some(value.family.clone().into()), &span)?,
                Kind::Parameter {
                    name: name.value.clone(),
                    value: value.value,
                },
            )
        }
        SyntaxKind::Negate(child) => {
            let operand = resolve(child, field, parameter)?;
            (
                operand.domain.clone(),
                Kind::Negate {
                    operand: Box::new(operand),
                },
            )
        }
        SyntaxKind::Binary {
            operator,
            left,
            right,
        } => {
            let left = resolve(left, field, parameter)?;
            let right = resolve(right, field, parameter)?;
            let domain = left.domain.combine(&right.domain, *operator, &span)?;
            (
                domain,
                Kind::Binary {
                    operator: *operator,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            )
        }
    };
    Ok(Expression { domain, kind, span })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        arithmetic_syntax::{self, Budget},
        ir::{Family, Identity, LogicalType},
        syntax::Parser,
    };
    use serde_json::json;
    fn compile(
        raw: &str,
        family: Family,
        facets: serde_json::Value,
        nullable: bool,
        parameter: Parameter,
    ) -> Result<Expression> {
        let mut p = Parser::new(raw)?;
        let syntax = arithmetic_syntax::parse(&mut p, &mut Budget::new())?;
        p.finish()?;
        resolve(
            &syntax,
            &mut |c| {
                Ok(Field {
                    scan: "s0".into(),
                    identity: Identity {
                        document_id: "original".into(),
                        revision: "0.8".into(),
                        module: "commerce".into(),
                        element: c.field.value.clone(),
                    },
                    logical_type: LogicalType {
                        family: family.clone(),
                        facets: facets.clone(),
                        nullable,
                    },
                    span: c.span.clone(),
                })
            },
            &mut |_| Ok(parameter.clone()),
        )
    }
    fn integer() -> Parameter {
        Parameter {
            family: Family::Integer,
            value: "2".into(),
        }
    }
    #[test]
    fn mathematical_integer_and_original_field_facets() {
        let e = compile(
            "l.quantity*999999999999999999999999999999999999",
            Family::Integer,
            json!({"integerWidth":{"bits":64,"signed":true}}),
            false,
            integer(),
        )
        .unwrap();
        assert_eq!(e.domain, Domain::Integer);
        let Kind::Binary { left, right, .. } = e.kind else {
            panic!()
        };
        let Kind::Field { field } = left.kind else {
            panic!()
        };
        assert_eq!(field.logical_type.facets["integerWidth"]["bits"], 64);
        assert_eq!(field.identity.revision, "0.8");
        assert!(matches!(right.kind, Kind::Literal { .. }));
    }
    #[test]
    fn exact_scale_and_token_retention() {
        let e = compile(
            "l.price*12.5000+0.010",
            Family::Decimal,
            json!({"precision":10,"scale":2}),
            false,
            integer(),
        )
        .unwrap();
        assert_eq!(e.domain, Domain::Decimal { scale: 6 });
        let encoded = serde_json::to_string(&e).unwrap();
        assert!(encoded.contains("12.5000"));
        assert!(encoded.contains("0.010"));
        assert_eq!(
            compile(
                "l.quantity*:rate",
                Family::Integer,
                json!({}),
                false,
                Parameter {
                    family: Family::Decimal,
                    value: "-1.2300".into()
                }
            )
            .unwrap()
            .domain,
            Domain::Decimal { scale: 4 }
        );
    }
    #[test]
    fn invalid_and_unavailable_numeric_meaning_refuses() {
        for token in ["1e2", "NaN", "+1", "1.", ".2", "1..2", ""] {
            assert!(
                compile(
                    "l.quantity+:value",
                    Family::Integer,
                    json!({}),
                    false,
                    Parameter {
                        family: Family::Decimal,
                        value: token.into()
                    }
                )
                .is_err(),
                "{token}"
            );
        }
        assert!(compile("l.quantity+1", Family::String, json!({}), false, integer()).is_err());
        assert!(compile("l.quantity+1", Family::Integer, json!({}), true, integer()).is_err());
        assert!(compile("l.price+1", Family::Decimal, json!({}), false, integer()).is_err());
        assert!(compile(
            "l.quantity+:n",
            Family::Integer,
            json!({}),
            false,
            Parameter {
                family: Family::Integer,
                value: "1.0".into()
            }
        )
        .is_err());
    }
}
