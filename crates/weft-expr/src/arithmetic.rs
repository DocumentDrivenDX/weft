// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exact arithmetic over integer and fixed-scale decimal domains.
//!
//! An [`Expression`] keeps every original token; its [`Domain`] is derived
//! exactly: integer op integer is integer, `*` adds decimal scales, `+` and `-`
//! take the larger scale. Wire shapes are weft-sql 0.3's plan shapes
//! (CONTRACT `exact-arithmetic-v0.3`). Division and float arithmetic are not
//! exact and are not defined here.
use crate::operand::Field;
use serde::Serialize;
use weft_model::{
    diagnostic::{Diagnostic, Result},
    types::{Family, FamilySet, LogicalType, ScalarFamily, Span},
};

/// Largest numeric token an arithmetic expression reads.
pub const MAX_LITERAL_BYTES: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "family", rename_all = "camelCase")]
pub enum Domain {
    Integer,
    Decimal { scale: u64 },
}
impl Domain {
    pub fn scale(&self) -> u64 {
        match self {
            Self::Integer => 0,
            Self::Decimal { scale } => *scale,
        }
    }
    /// The domain of `self operator right` for `operator` in `+`, `-`, `*`.
    pub fn combine(&self, right: &Self, operator: char, span: &Span) -> Result<Self> {
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
    /// The domain of a Field operand: a required integer, or a required decimal
    /// with an authored scale. Nullable, non-numeric and float Fields refuse.
    pub fn of_field<F: FamilySet>(logical_type: &LogicalType<F>, span: &Span) -> Result<Self> {
        if logical_type.nullable {
            return Err(fail("WFT-TYPE", "Arithmetic requires an admitted required scalar", span));
        }
        match logical_type.family.widen() {
            ScalarFamily::Integer => Ok(Self::Integer),
            ScalarFamily::Decimal => Ok(Self::Decimal {
                scale: logical_type.facets["scale"].as_u64().ok_or_else(|| {
                    fail("WFT-TYPE", "Authored decimal scale is unavailable", span)
                })?,
            }),
            _ => Err(fail("WFT-TYPE", "Arithmetic requires numeric fields", span)),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Expression<F = Family> {
    pub domain: Domain,
    pub span: Span,
    pub kind: Kind<F>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Kind<F = Family> {
    Field {
        field: Field<F>,
    },
    Literal {
        value: String,
    },
    Parameter {
        name: String,
        value: String,
    },
    Negate {
        operand: Box<Expression<F>>,
    },
    Binary {
        operator: char,
        left: Box<Expression<F>>,
        right: Box<Expression<F>>,
    },
}
fn fail(code: &str, message: &str, span: &Span) -> Diagnostic {
    Diagnostic::new(code, "resolve", message).at(span)
}
/// The domain of an exact base-ten token. `family` is the declared family of a
/// typed parameter; `None` derives the domain from the token's own digits.
pub fn token_domain(token: &str, family: Option<ScalarFamily>, span: &Span) -> Result<Domain> {
    if token.len() > MAX_LITERAL_BYTES {
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
        Some(ScalarFamily::Integer) if fraction.is_none() => Ok(Domain::Integer),
        Some(ScalarFamily::Integer) => Err(fail(
            "WFT-NUMERIC-DOMAIN",
            "Integer parameter has a fractional token",
            span,
        )),
        Some(ScalarFamily::Decimal) => Ok(Domain::Decimal {
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

#[cfg(test)]
mod tests {
    use super::*;
    use weft_model::types::ScalarType;
    const SPAN: Span = Span { start: 0, end: 0 };
    #[test]
    fn exact_scale_rules() {
        let d = |scale| Domain::Decimal { scale };
        assert_eq!(Domain::Integer.combine(&Domain::Integer, '*', &SPAN).unwrap(), Domain::Integer);
        assert_eq!(d(2).combine(&d(4), '*', &SPAN).unwrap(), d(6));
        assert_eq!(d(2).combine(&d(4), '+', &SPAN).unwrap(), d(4));
        assert_eq!(Domain::Integer.combine(&d(3), '-', &SPAN).unwrap(), d(3));
        assert_eq!(d(u64::MAX).combine(&d(1), '*', &SPAN).unwrap_err().code, "WFT-LIMIT");
    }
    #[test]
    fn tokens_and_fields() {
        assert_eq!(token_domain("12.500", None, &SPAN).unwrap(), Domain::Decimal { scale: 3 });
        assert_eq!(token_domain("-7", Some(ScalarFamily::Integer), &SPAN).unwrap(), Domain::Integer);
        for bad in ["1e2", "+1", "1.", ".5", ""] {
            assert_eq!(token_domain(bad, None, &SPAN).unwrap_err().code, "WFT-NUMERIC-DOMAIN", "{bad}");
        }
        assert_eq!(token_domain("1.5", Some(ScalarFamily::Float), &SPAN).unwrap_err().code, "WFT-TYPE");
        assert_eq!(token_domain(&"1".repeat(MAX_LITERAL_BYTES + 1), None, &SPAN).unwrap_err().code, "WFT-LIMIT");
        let decimal = LogicalType { family: Family::Decimal, facets: serde_json::json!({"precision":10,"scale":2}), nullable: false };
        assert_eq!(Domain::of_field(&decimal, &SPAN).unwrap(), Domain::Decimal { scale: 2 });
        assert!(Domain::of_field(&LogicalType { nullable: true, ..decimal }, &SPAN).is_err());
        assert!(Domain::of_field(&ScalarType::plain(ScalarFamily::Float, false), &SPAN).is_err());
        assert!(Domain::of_field(&ScalarType::plain(ScalarFamily::Timestamp, false), &SPAN).is_err());
    }
}
