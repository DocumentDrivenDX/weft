// SPDX-License-Identifier: MIT OR Apache-2.0
//! The SQL emission carrier: one [`crate::boundary::TargetArtifact`]. Statement
//! text, typed parameter slots in position order, and result columns with their
//! carriers and exact decoders. Target-neutral: PostgreSQL and Databricks
//! backends both emit it.
use crate::{boundary::{Frontend, TargetArtifact}, manifest::{failure, Obligation}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use weft_model::{
    diagnostic::Result,
    types::{Family, FamilySet, ScalarFamily},
};

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
/// How a host decodes one scalar result value from its carrier. Every decoder
/// is exact: it reproduces the value without rounding or session dependence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScalarDecoder {
    /// The text value as is.
    Text,
    /// A boolean.
    Boolean,
    /// A canonical decimal integer token.
    ExactInteger,
    /// A canonical decimal token.
    ExactDecimal,
    /// `utc-instant`: a UTC instant as text `YYYY-MM-DDTHH:MM:SS.ffffffZ`, a
    /// four-digit year and exactly six fractional digits (microseconds). Hosts
    /// refuse any other text. For the opt-in `timestamp` family.
    UtcInstant,
    /// `ieee754-binary64-hex`: the IEEE 754 binary64 bit pattern as 16
    /// lowercase hexadecimal digits, most significant byte first. Hosts refuse
    /// other text and non-finite values (UMF floats are finite). For the opt-in
    /// `float` family.
    Ieee754Binary64Hex,
}
impl ScalarDecoder {
    /// The exact decoder for a column of `family`, in any family set.
    pub fn exact<T: FamilySet>(family: &T) -> Self {
        match family.widen() {
            ScalarFamily::String => Self::Text,
            ScalarFamily::Boolean => Self::Boolean,
            ScalarFamily::Integer => Self::ExactInteger,
            ScalarFamily::Decimal => Self::ExactDecimal,
            ScalarFamily::Timestamp => Self::UtcInstant,
            ScalarFamily::Float => Self::Ieee754Binary64Hex,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OuterJoin {
    pub scan: String,
    pub record: weft_model::types::Identity,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Representation<T = Family> {
    NativeMetadata {
        metadata: NativeMetadata,
        #[serde(rename = "logicalType")]
        logical_type: weft_model::types::LogicalType,
        carrier: ScalarCarrier,
        decoder: ScalarDecoder,
    },
    Scalar {
        #[serde(rename = "logicalType")]
        logical_type: weft_model::types::LogicalType<T>,
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
pub struct Column<T = Family> {
    pub position: usize,
    pub output_name: String,
    /// Physical result label; only explicitly admitted 0.3 positional outputs may carry it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carrier_name: Option<String>,
    pub representation: Representation<T>,
    pub source_identities: Vec<weft_model::types::Identity>,
    pub nullable: bool,
}
/// A SQL artifact. `T` is the family set of its scalar result columns: weft's
/// backends emit [`Family`] columns only; a compiler with timestamp and float
/// columns emits `Emission<ScalarFamily>`.
#[derive(Debug, Clone, Serialize)]
pub struct Emission<T = Family> {
    pub sql: String,
    pub parameters: Vec<ParameterSlot>,
    pub columns: Vec<Column<T>>,
    pub obligations: Vec<Obligation>,
}

impl<F, T> TargetArtifact<F> for Emission<T>
where
    F: SqlChecks<T>,
    T: Send,
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
pub trait SqlChecks<T = Family>: Frontend {
    fn check_emission(_program: Self::Program<'_>, _emission: &Emission<T>, _selection: &crate::boundary::Selection, _assessments: &[crate::boundary::Assessment]) -> Result<()> {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use weft_model::types::{Identity, LogicalType, ScalarType};
    #[test]
    fn opt_in_columns_carry_timestamp_and_float_decoders() {
        let column = |position: usize, family: ScalarFamily| Column {
            position,
            output_name: family.umf_name().into(),
            carrier_name: None,
            representation: Representation::Scalar {
                logical_type: ScalarType::plain(family, false),
                carrier: ScalarCarrier::Text,
                decoder: ScalarDecoder::exact(&family),
            },
            source_identities: Vec::<Identity>::new(),
            nullable: false,
        };
        let emission = Emission::<ScalarFamily> {
            sql: "SELECT".into(),
            parameters: vec![],
            columns: vec![column(1, ScalarFamily::Timestamp), column(2, ScalarFamily::Float)],
            obligations: vec![],
        };
        let value = serde_json::to_value(&emission).unwrap();
        assert_eq!(value["columns"][0]["representation"]["decoder"], json!("utc-instant"));
        assert_eq!(value["columns"][0]["representation"]["logicalType"]["family"], json!("timestamp"));
        assert_eq!(value["columns"][1]["representation"]["decoder"], json!("ieee754-binary64-hex"));
        // weft's core families keep their decoders.
        for (family, decoder) in [
            (Family::String, "text"),
            (Family::Boolean, "boolean"),
            (Family::Integer, "exact-integer"),
            (Family::Decimal, "exact-decimal"),
        ] {
            assert_eq!(serde_json::to_value(ScalarDecoder::exact(&family)).unwrap(), json!(decoder));
            let _: LogicalType = LogicalType::plain(family, true);
        }
    }
}
