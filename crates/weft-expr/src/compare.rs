// SPDX-License-Identifier: MIT OR Apache-2.0
//! Comparison operators and their typing rule.
use serde::{Deserialize, Serialize};
use weft_model::{
    diagnostic::{Diagnostic, Result},
    types::{FamilySet, LogicalType, ScalarFamily, Span},
};

/// The six scalar comparisons. Wire spelling is the `weft.update-selection/1`
/// spelling: `eq`, `ne`, `lt`, `le`, `gt`, `ge`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
impl CompareOp {
    pub const ALL: [CompareOp; 6] = [Self::Eq, Self::Ne, Self::Lt, Self::Le, Self::Gt, Self::Ge];
    /// Read the selection spelling.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|op| op.name() == name)
    }
    /// The selection spelling.
    pub fn name(self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::Ne => "ne",
            Self::Lt => "lt",
            Self::Le => "le",
            Self::Gt => "gt",
            Self::Ge => "ge",
        }
    }
    /// Standard SQL operator text.
    pub fn sql(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "<>",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
        }
    }
    /// True for `lt`, `le`, `gt`, `ge`.
    pub fn orders(self) -> bool {
        !matches!(self, Self::Eq | Self::Ne)
    }
    /// Whether this comparison is defined on `family`: equality on every family,
    /// ordering on every family except boolean.
    pub fn admits(self, family: ScalarFamily) -> bool {
        !(self.orders() && family == ScalarFamily::Boolean)
    }
}
impl From<ComparisonOperator> for CompareOp {
    fn from(op: ComparisonOperator) -> Self {
        match op {
            ComparisonOperator::Less => Self::Lt,
            ComparisonOperator::LessEqual => Self::Le,
            ComparisonOperator::GreaterEqual => Self::Ge,
            ComparisonOperator::NotEqual => Self::Ne,
        }
    }
}

/// Type a comparison of two operands: same family and facets, and the operator
/// defined on that family. Nullability may differ; a null operand makes the
/// comparison unknown, which is the evaluator's concern.
pub fn check<F: FamilySet>(op: CompareOp, left: &LogicalType<F>, right: &LogicalType<F>, span: &Span) -> Result<()> {
    let fail = |message| Diagnostic::new("WFT-TYPE", "type", message).at(span);
    if left.family != right.family || left.facets != right.facets {
        return Err(fail("Comparison operands have different value domains"));
    }
    if !op.admits(left.family.widen()) {
        return Err(fail("Comparison is not defined on this value family"));
    }
    Ok(())
}

/// weft-sql 0.3 additive scalar comparison operators. The original `=` and `>`
/// keep their own plan shapes, so this wire enum has four members.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ComparisonOperator { Less, LessEqual, GreaterEqual, NotEqual }
impl ComparisonOperator {
    pub fn sql(self) -> &'static str { CompareOp::from(self).sql() }
    pub fn capability(self) -> &'static str { match self { Self::Less => "compare.less", Self::LessEqual => "compare.lessEqual", Self::GreaterEqual => "compare.greaterEqual", Self::NotEqual => "compare.notEqual" } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use weft_model::types::{Family, ScalarType};
    const SPAN: Span = Span { start: 0, end: 0 };
    #[test]
    fn spellings_round_trip_and_match_legacy_sql() {
        for op in CompareOp::ALL {
            assert_eq!(CompareOp::parse(op.name()), Some(op));
            assert_eq!(serde_json::to_value(op).unwrap(), op.name());
        }
        assert_eq!(CompareOp::parse("EQ"), None);
        for (legacy, sql) in [(ComparisonOperator::Less, "<"), (ComparisonOperator::LessEqual, "<="), (ComparisonOperator::GreaterEqual, ">="), (ComparisonOperator::NotEqual, "<>")] {
            assert_eq!(legacy.sql(), sql);
        }
        assert_eq!(serde_json::to_value(ComparisonOperator::LessEqual).unwrap(), "lessEqual");
    }
    #[test]
    fn ordering_is_refused_only_on_booleans() {
        let boolean = LogicalType::plain(Family::Boolean, false);
        assert!(check(CompareOp::Eq, &boolean, &boolean, &SPAN).is_ok());
        assert_eq!(check(CompareOp::Lt, &boolean, &boolean, &SPAN).unwrap_err().code, "WFT-TYPE");
        let instant = ScalarType::plain(ScalarFamily::Timestamp, true);
        let float = ScalarType::plain(ScalarFamily::Float, false);
        assert!(check(CompareOp::Ge, &instant, &ScalarType::plain(ScalarFamily::Timestamp, false), &SPAN).is_ok());
        assert!(check(CompareOp::Lt, &float, &float, &SPAN).is_ok());
        assert!(check(CompareOp::Eq, &instant, &float, &SPAN).is_err());
        let narrow = LogicalType { family: Family::Decimal, facets: serde_json::json!({"precision":4,"scale":2}), nullable: false };
        let wide = LogicalType { facets: serde_json::json!({"precision":6,"scale":2}), ..narrow.clone() };
        assert!(check(CompareOp::Eq, &narrow, &wide, &SPAN).is_err());
    }
}
