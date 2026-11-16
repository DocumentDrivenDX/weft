// SPDX-License-Identifier: MIT OR Apache-2.0
//! Typed operands: a Field read through a scan occurrence, and a literal,
//! parameter or Field value. Wire shapes are weft's logical-plan shapes.
use serde::{Deserialize, Serialize};
use weft_model::types::{Family, Identity, LogicalType, Span};

/// A selected Field read through scan occurrence `scan`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Field<F = Family> {
    pub scan: String,
    pub identity: Identity,
    #[serde(rename = "type")]
    pub logical_type: LogicalType<F>,
    pub span: Span,
}
/// A comparison operand: an exact literal token, a typed parameter, or a Field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Value<F = Family> {
    Literal {
        value: String,
        #[serde(rename = "type")]
        logical_type: LogicalType<F>,
        span: Span,
    },
    Parameter {
        name: String,
        value: String,
        #[serde(rename = "type")]
        logical_type: LogicalType<F>,
        span: Span,
    },
    Field {
        field: Field<F>,
    },
}
impl<F> Value<F> {
    pub fn logical_type(&self) -> &LogicalType<F> {
        match self {
            Self::Literal { logical_type, .. } | Self::Parameter { logical_type, .. } => logical_type,
            Self::Field { field } => &field.logical_type,
        }
    }
}
