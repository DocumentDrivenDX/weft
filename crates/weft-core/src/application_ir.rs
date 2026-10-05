//! Relational stages execute in field order: scans/joins, filters, aggregate,
//! projection, ordering and limit. Arrays are bags except explicit order stages.
use crate::{
    application_model::{Descriptor, RelationshipRead},
    ir::{Identity, LogicalType, ModelPin, Span},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Subset {
    EntityPage,
    CountSummary,
    RelatedEntityPage,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadProfile {
    pub version: String,
    pub subset: Subset,
}
#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub scan: String,
    pub identity: Identity,
    #[serde(rename = "type")]
    pub logical_type: LogicalType,
    pub span: Span,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Value {
    Literal {
        value: String,
        #[serde(rename = "type")]
        logical_type: LogicalType,
        span: Span,
    },
    Parameter {
        name: String,
        value: String,
        #[serde(rename = "type")]
        logical_type: LogicalType,
        span: Span,
    },
    Field {
        field: Field,
    },
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Predicate {
    Equal {
        left: Field,
        right: Value,
    },
    LexicographicGreater {
        columns: Vec<Field>,
        values: Vec<Value>,
    },
    HasRelated {
        scan: String,
        relationship: RelationshipRead,
        key: Vec<Value>,
    },
}
#[derive(Debug, Clone, Serialize)]
pub struct Scan {
    pub occurrence: String,
    pub record: Identity,
    pub pin: ModelPin,
}
#[derive(Debug, Clone, Serialize)]
pub struct Join {
    pub right: Scan,
    pub on: Vec<Predicate>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Expression {
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
