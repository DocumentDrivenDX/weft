// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exact literal checks live in `weft_model::exact`; this module builds the
//! weft-sql 0.1 IR literal from a checked token.
use crate::{
    error::Result,
    ir::{Expression, LogicalType},
    syntax::Literal,
};
pub use weft_model::exact::validate_binary_token;
pub(crate) use weft_model::exact::{validate_date_token, validate_mathematical_decimal};
pub fn literal(input: &Literal, expected: &LogicalType) -> Result<Expression> {
    weft_model::exact::check_literal(input, expected)?;
    Ok(Expression::Literal {
        value: input.value.clone(),
        logical_type: expected.clone(),
        span: input.span.clone(),
    })
}
