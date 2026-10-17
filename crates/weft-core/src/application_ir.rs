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
    CompleteEntity,
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
pub struct ExactField {
    pub scan: String,
    pub identity: Identity,
    #[serde(rename="scalarType")]
    pub scalar_type: crate::application_model::ExactScalarType,
    pub span: Span,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag="kind",rename_all="camelCase")]
pub enum ExactValue {
    Literal { value:String, #[serde(rename="scalarType")] scalar_type:crate::application_model::ExactScalarType, span:Span },
    Parameter { name:String, value:String, #[serde(rename="scalarType")] scalar_type:crate::application_model::ExactScalarType, span:Span },
    Field { field:ExactField },
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Predicate {
    Boolean { value:bool },
    ExactEqual { left:ExactField, right:ExactValue },
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
    NativeMetadata {
        scan: String,
        record: Identity,
        metadata: NativeMetadata,
        #[serde(rename = "type")]
        logical_type: LogicalType,
    },
    Field {
        scan: String,
        identity: Identity,
    },
    CountDistinct { argument:Field, #[serde(rename="type")] logical_type:LogicalType },
    Count {
        #[serde(rename = "type")]
        logical_type: LogicalType,
    },
    Sum {
        argument: Field,
        #[serde(rename = "type")]
        logical_type: LogicalType,
    },
    CompleteRelatedKeys {
        scan: String,
        relationship: RelationshipRead,
        bound: u16,
    },
    RelatedKeys {
        scan: String,
        relationship: RelationshipRead,
        bound: u16,
    },
}
/// Explicit backend-owned facts, never synthetic authored UMF Field identities.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NativeMetadata {
    Version,
    CreatedBy,
    CreatedAt,
    UpdatedBy,
    UpdatedAt,
    OriginalAuthor,
    OriginalAt,
    LoadId,
}
#[derive(Debug, Clone, Serialize)]
pub struct Output {
    pub name: String,
    pub expression: Expression,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    /// Original compiler-owned source entity selection. Ordinary plans omit it.
    #[serde(skip_serializing_if="Option::is_none")]
    pub navigation: Option<Box<Navigation>>,
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

/// Prepared by Core from original source SQL and canonical relationship identity.
/// Private members prevent a host from fabricating an admitted source selection.
#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Navigation {
    version:String,
    source:Box<Plan>,
    relationship:RelationshipRead,
}
impl Navigation {
    pub fn source(&self)->&Plan{&self.source}
    pub(crate) fn source_mut(&mut self)->&mut Plan{&mut self.source}
    pub fn relationship(&self)->&RelationshipRead{&self.relationship}
    pub(crate) fn new(source:Plan,relationship:RelationshipRead)->Self{Self{version:"weft.engine-navigation/0.1".into(),source:Box::new(source),relationship}}
}
