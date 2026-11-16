// SPDX-License-Identifier: MIT OR Apache-2.0
//! The SQL emission carrier: one [`crate::boundary::TargetArtifact`]. Statement
//! text, typed parameter slots in position order, and result columns with their
//! carriers and exact decoders. Target-neutral: PostgreSQL and Databricks
//! backends both emit it.
use crate::{boundary::{Frontend, TargetArtifact}, manifest::{failure, Obligation}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use weft_model::diagnostic::Result;

/// Explicit backend-owned facts, never synthetic authored UMF Field identities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ParameterType {
    Atomic(weft_model::types::LogicalType),
    Exact { kind: ExactParameterKind, #[serde(rename="scalarType")] scalar_type:weft_model::descriptor::ExactScalarType },
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ExactParameterKind { #[serde(rename="exactScalar")] ExactScalar }
impl From<weft_model::types::LogicalType> for ParameterType {
    fn from(value:weft_model::types::LogicalType)->Self { Self::Atomic(value) }
}
impl ParameterType {
    pub fn atomic(&self)->Result<&weft_model::types::LogicalType> {
        match self { Self::Atomic(value)=>Ok(value), _=>Err(failure("WFT-EMISSION","Atomic parameter required by this backend")) }
    }
}
impl PartialEq<weft_model::types::LogicalType> for ParameterType {
    fn eq(&self,other:&weft_model::types::LogicalType)->bool { matches!(self,Self::Atomic(value) if value==other) }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterSlot {
    pub position: usize,
    pub logical_type: ParameterType,
    pub value: String,
    pub origin: Value,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScalarCarrier {
    Text,
    Boolean,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScalarDecoder {
    Text,
    Boolean,
    ExactInteger,
    ExactDecimal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OuterJoin {
    pub scan: String,
    pub record: weft_model::types::Identity,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Representation {
    NativeMetadata {
        metadata: NativeMetadata,
        #[serde(rename = "logicalType")]
        logical_type: weft_model::types::LogicalType,
        carrier: ScalarCarrier,
        decoder: ScalarDecoder,
    },
    Scalar {
        #[serde(rename = "logicalType")]
        logical_type: weft_model::types::LogicalType,
        carrier: ScalarCarrier,
        decoder: ScalarDecoder,
    },
    Value {
        descriptor: weft_model::types::Identity,
        #[serde(rename = "nativeNull")]
        native_null: bool,
        #[serde(rename = "outerJoin", skip_serializing_if = "Option::is_none")]
        outer_join: Option<OuterJoin>,
    },
    CompleteRelatedKeys {
        relationship: weft_model::descriptor::RelationshipIdentity,
        key: weft_model::descriptor::AuthoredKey,
        bound: u16,
    },
    RelatedKeys {
        relationship: weft_model::descriptor::RelationshipIdentity,
        key: weft_model::descriptor::AuthoredKey,
        bound: u16,
    },
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    pub position: usize,
    pub output_name: String,
    /// Physical result label; only explicitly admitted 0.3 positional outputs may carry it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carrier_name: Option<String>,
    pub representation: Representation,
    pub source_identities: Vec<weft_model::types::Identity>,
    pub nullable: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct Emission {
    pub sql: String,
    pub parameters: Vec<ParameterSlot>,
    pub columns: Vec<Column>,
    pub obligations: Vec<Obligation>,
}

impl<F: Frontend> TargetArtifact<F> for Emission
where
    F: SqlChecks,
{
    fn obligations_mut(&mut self) -> &mut Vec<Obligation> {
        &mut self.obligations
    }
    fn check(&self, program: F::Program<'_>, selection: &crate::boundary::Selection, assessments: &[crate::boundary::Assessment]) -> Result<()> {
        F::check_emission(program, self, selection, assessments)
    }
}
/// A frontend's checks on an emitted SQL artifact against its program
/// (output order, decoders, parameter domains). Default: none.
pub trait SqlChecks: Frontend {
    fn check_emission(_program: Self::Program<'_>, _emission: &Emission, _selection: &crate::boundary::Selection, _assessments: &[crate::boundary::Assessment]) -> Result<()> {
        Ok(())
    }
}
