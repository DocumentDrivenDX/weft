//! Typed internal 0.3 relational plan; not yet admitted by public transport.
pub(crate) use crate::application_ir::{Field, ReadProfile, Scan};
use crate::{
    application_model::{Descriptor, RelationshipRead},
    ir::{Identity, LogicalType, ModelPin},
};
use serde::Serialize;
/// Additive 0.3 scalar comparison operators; old =/> variants retain their wire shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ComparisonOperator { Less, LessEqual, GreaterEqual, NotEqual }
impl ComparisonOperator {
    pub fn sql(self) -> &'static str { match self { Self::Less => "<", Self::LessEqual => "<=", Self::GreaterEqual => ">=", Self::NotEqual => "<>" } }
    pub fn capability(self) -> &'static str { match self { Self::Less => "compare.less", Self::LessEqual => "compare.lessEqual", Self::GreaterEqual => "compare.greaterEqual", Self::NotEqual => "compare.notEqual" } }
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Predicate {
    NullTest { field: Field, negated: bool },
    NullableStringEqual { left: Field, right: Field },
    ScalarCompare { left: Field, right: crate::application_ir::Value, operator: ComparisonOperator },
    ArithmeticCompareExtended { left: crate::arithmetic_resolve::Expression, right: crate::arithmetic_resolve::Expression, operator: ComparisonOperator },
    Legacy {
        predicate: crate::application_ir::Predicate,
    },
    ArithmeticCompare {
        left: crate::arithmetic_resolve::Expression,
        right: crate::arithmetic_resolve::Expression,
        greater: bool,
    },
}
#[derive(Debug, Clone, Serialize)]
pub struct Join {
    pub right: Scan,
    pub on: Vec<Predicate>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Expression {
    Arithmetic {
        expression: crate::arithmetic_resolve::Expression,
    },
    Field {
        scan: String,
        identity: Identity,
    },
    Count {
        #[serde(rename = "type")]
        logical_type: LogicalType,
    },
    Sum {
        argument: Field,
        #[serde(rename = "type")]
        logical_type: LogicalType,
    },
    RelatedKeys {
        scan: String,
        relationship: RelationshipRead,
        bound: u16,
    },
}
#[derive(Debug, Clone, Serialize)]
pub struct Output {
    pub name: String,
    pub expression: Expression,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub ir_version: String,
    pub module_pins: Vec<ModelPin>,
    pub read_profile: Option<ReadProfile>,
    pub required_capabilities: Vec<String>,
    pub type_graph: Vec<Descriptor>,
    pub source: Scan,
    pub page_key: Option<crate::application_model::AuthoredKey>,
    pub joins: Vec<Join>,
    pub filters: Vec<Predicate>,
    pub groups: Vec<Field>,
    pub aggregate: bool,
    pub outputs: Vec<Output>,
    pub order: Vec<Field>,
    pub limit: Option<u16>,
}
