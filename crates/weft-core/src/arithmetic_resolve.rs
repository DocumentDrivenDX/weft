//! Exact expression domains. Field admission remains owned by the model resolver.
use crate::{
    application_ir::Field,
    application_resolve::Parameter,
    arithmetic_syntax::{Expression as Syntax, Kind as SyntaxKind},
    error::{Diagnostic, Result},
    ir::{Family, Span},
    syntax::{Column, Name},
};
use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "family", rename_all = "camelCase")]
pub(crate) enum Domain {
    Integer,
    Decimal { scale: u64 },
}
impl Domain {
    fn scale(&self) -> u64 {
        match self {
            Self::Integer => 0,
            Self::Decimal { scale } => *scale,
        }
    }
    fn combine(&self, right: &Self, operator: char, span: &Span) -> Result<Self> {
        if matches!((self, right), (Self::Integer, Self::Integer)) {
            return Ok(Self::Integer);
        }
        let scale = if operator == '*' {
            self.scale().checked_add(right.scale()).ok_or_else(|| {
                fail(
                    "WFT-LIMIT",
                    "Derived decimal scale exceeds representable metadata",
                    span,
                )
            })?
        } else {
            self.scale().max(right.scale())
        };
        Ok(Self::Decimal { scale })
    }
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Expression {
    pub domain: Domain,
    pub span: Span,
    pub kind: Kind,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub(crate) enum Kind {
    Field {
        field: Field,
    },
    Literal {
        value: String,
    },
    Parameter {
        name: String,
        value: String,
    },
    Negate {
        operand: Box<Expression>,
    },
    Binary {
        operator: char,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}
fn fail(code: &str, message: &str, span: &Span) -> Diagnostic {
    Diagnostic::new(code, "resolve", message).at(span)
}
fn token_domain(token: &str, family: Option<&Family>, span: &Span) -> Result<Domain> {
    if token.len() > crate::arithmetic_syntax::MAX_LITERAL_BYTES {
        return Err(fail(
            "WFT-LIMIT",
            "Numeric token exceeds expression limit",
            span,
        ));
    }
    let unsigned = token.strip_prefix('-').unwrap_or(token);
    let mut parts = unsigned.split('.');
    let whole = parts.next().unwrap_or("");
    let fraction = parts.next();
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || parts.next().is_some()
        || fraction.is_some_and(|f| f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(fail(
            "WFT-NUMERIC-DOMAIN",
            "Exact numeric token requires base-ten digits",
            span,
        ));
    }
    match family {
        Some(Family::Integer) if fraction.is_none() => Ok(Domain::Integer),
        Some(Family::Integer) => Err(fail(
            "WFT-NUMERIC-DOMAIN",
            "Integer parameter has a fractional token",
            span,
        )),
        Some(Family::Decimal) => Ok(Domain::Decimal {
            scale: fraction.map_or(0, |f| f.len() as u64),
        }),
        Some(_) => Err(fail(
            "WFT-TYPE",
            "Arithmetic parameter must be numeric",
            span,
        )),
        None => Ok(fraction.map_or(Domain::Integer, |f| Domain::Decimal {
            scale: f.len() as u64,
        })),
    }
}
pub(crate) fn resolve<F, P>(syntax: &Syntax, field: &mut F, parameter: &mut P) -> Result<Expression>
where
    F: FnMut(&Column) -> Result<Field>,
    P: FnMut(&Name) -> Result<Parameter>,
{
    let span = syntax.span.clone();
    let (domain, kind) = match &syntax.kind {
        SyntaxKind::Field(column) => {
            let value = field(column)?;
            if value.logical_type.nullable {
                return Err(fail(
                    "WFT-TYPE",
                    "Arithmetic requires an admitted required scalar",
                    &span,
                ));
            }
            let domain = match value.logical_type.family {
                Family::Integer => Domain::Integer,
                Family::Decimal => Domain::Decimal {
                    scale: value.logical_type.facets["scale"].as_u64().ok_or_else(|| {
                        fail("WFT-TYPE", "Authored decimal scale is unavailable", &span)
                    })?,
                },
                _ => {
                    return Err(fail(
                        "WFT-TYPE",
                        "Arithmetic requires numeric fields",
                        &span,
                    ))
                }
            };
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
                token_domain(&value.value, Some(&value.family), &span)?,
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
        ir::{Identity, LogicalType},
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
