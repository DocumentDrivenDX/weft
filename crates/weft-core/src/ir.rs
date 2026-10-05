use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub document_id: String,
    pub revision: String,
    pub module: String,
    pub element: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelPin {
    pub document_id: String,
    pub revision: String,
    pub umf_version: String,
    pub sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Family {
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "decimal")]
    Decimal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalType {
    pub family: Family,
    pub facets: Value,
    pub nullable: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Expression {
    Field {
        scan: String,
        identity: Identity,
        #[serde(rename = "type")]
        logical_type: LogicalType,
        span: Span,
    },
    Literal {
        value: String,
        #[serde(rename = "type")]
        logical_type: LogicalType,
        span: Span,
    },
    Equal {
        left: Box<Expression>,
        right: Box<Expression>,
        #[serde(rename = "type")]
        logical_type: LogicalType,
        span: Span,
    },
    And {
        left: Box<Expression>,
        right: Box<Expression>,
        #[serde(rename = "type")]
        logical_type: LogicalType,
        span: Span,
    },
    Sum {
        argument: Box<Expression>,
        #[serde(rename = "type")]
        logical_type: LogicalType,
        span: Span,
    },
}
impl Expression {
    pub fn logical_type(&self) -> &LogicalType {
        match self {
            Self::Field { logical_type, .. }
            | Self::Literal { logical_type, .. }
            | Self::Equal { logical_type, .. }
            | Self::And { logical_type, .. }
            | Self::Sum { logical_type, .. } => logical_type,
        }
    }
    pub fn span(&self) -> &Span {
        match self {
            Self::Field { span, .. }
            | Self::Literal { span, .. }
            | Self::Equal { span, .. }
            | Self::And { span, .. }
            | Self::Sum { span, .. } => span,
        }
    }
    pub fn same_field(&self, other: &Self) -> bool {
        matches!((self,other),(Self::Field{scan:a,identity:i,..},Self::Field{scan:b,identity:j,..}) if a==b&&i==j)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output {
    pub name: String,
    pub expression: Expression,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Node {
    Scan {
        occurrence: String,
        record: Identity,
        pin: ModelPin,
    },
    InnerJoin {
        left: Box<Node>,
        right: Box<Node>,
        on: Expression,
    },
    Filter {
        input: Box<Node>,
        predicate: Expression,
    },
    Aggregate {
        input: Box<Node>,
        groups: Vec<Expression>,
        aggregates: Vec<Expression>,
    },
    Project {
        input: Box<Node>,
        outputs: Vec<Output>,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogicalPlan {
    pub ir_version: String,
    pub module_pins: Vec<ModelPin>,
    pub required_capabilities: Vec<String>,
    pub root: Node,
}
