// SPDX-License-Identifier: MIT OR Apache-2.0
//! Resolved UMF identities and logical scalar types.
//!
//! A [`LogicalType`] is parameterized by the family set a profile admits:
//!
//! - [`Family`] is the closed set every existing weft dialect profile admits
//!   (boolean, string, integer, decimal). `LogicalType` defaults to it, so weft's
//!   IR and backends cannot represent anything wider.
//! - [`ScalarFamily`] adds UMF `timestamp` and `float`. New profiles and other
//!   compilers (Fireweed) select it explicitly, as [`ScalarType`].
//!
//! The split is enforced by the type system, not by refusal checks: an old
//! profile has no value that could carry a timestamp or float.
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
/// The four UMF scalar families every weft dialect profile admits.
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
/// UMF scalar families available to opt-in profiles: [`Family`] plus `timestamp`
/// and `float`. Wire spellings are the UMF `scalarType` names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScalarFamily {
    Boolean,
    String,
    Integer,
    Decimal,
    /// An exact UTC instant: an RFC 3339 token with any fractional precision,
    /// ordered and compared through [`crate::exact_instant::key`].
    Timestamp,
    /// A finite IEEE 754 binary64 value written as a UMF float token. The
    /// original token is retained.
    Float,
}
impl ScalarFamily {
    /// Read a UMF `scalarType` name. Unknown and unsupported families are `None`.
    pub fn from_umf(name: &str) -> Option<Self> {
        Some(match name {
            "boolean" => Self::Boolean,
            "string" => Self::String,
            "integer" => Self::Integer,
            "decimal" => Self::Decimal,
            "timestamp" => Self::Timestamp,
            "float" => Self::Float,
            _ => return None,
        })
    }
    /// The UMF `scalarType` spelling.
    pub fn umf_name(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::String => "string",
            Self::Integer => "integer",
            Self::Decimal => "decimal",
            Self::Timestamp => "timestamp",
            Self::Float => "float",
        }
    }
}
impl From<Family> for ScalarFamily {
    fn from(family: Family) -> Self {
        match family {
            Family::Boolean => Self::Boolean,
            Family::String => Self::String,
            Family::Integer => Self::Integer,
            Family::Decimal => Self::Decimal,
        }
    }
}
impl TryFrom<ScalarFamily> for Family {
    type Error = ScalarFamily;
    fn try_from(family: ScalarFamily) -> std::result::Result<Self, ScalarFamily> {
        Ok(match family {
            ScalarFamily::Boolean => Self::Boolean,
            ScalarFamily::String => Self::String,
            ScalarFamily::Integer => Self::Integer,
            ScalarFamily::Decimal => Self::Decimal,
            other => return Err(other),
        })
    }
}
/// A set of scalar families a profile admits. Implemented by [`Family`] and
/// [`ScalarFamily`]; typing functions are generic over it.
pub trait FamilySet: Clone + PartialEq + std::fmt::Debug {
    /// Admit `family`, or `None` when it is outside this set.
    fn admit(family: ScalarFamily) -> Option<Self>;
    /// This family as a [`ScalarFamily`].
    fn widen(&self) -> ScalarFamily;
}
impl FamilySet for Family {
    fn admit(family: ScalarFamily) -> Option<Self> { Self::try_from(family).ok() }
    fn widen(&self) -> ScalarFamily { self.clone().into() }
}
impl FamilySet for ScalarFamily {
    fn admit(family: ScalarFamily) -> Option<Self> { Some(family) }
    fn widen(&self) -> ScalarFamily { *self }
}
/// A logical scalar type: family, UMF facets and nullability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalType<F = Family> {
    pub family: F,
    pub facets: Value,
    pub nullable: bool,
}
/// A logical type over the opt-in family set.
pub type ScalarType = LogicalType<ScalarFamily>;
impl<F: FamilySet> LogicalType<F> {
    /// A facetless type of `family`.
    pub fn plain(family: F, nullable: bool) -> Self {
        Self { family, facets: Value::Object(Default::default()), nullable }
    }
    /// The same type over the opt-in family set.
    pub fn widen(&self) -> ScalarType {
        LogicalType { family: self.family.widen(), facets: self.facets.clone(), nullable: self.nullable }
    }
    /// Narrow to another family set, or `None` when the family is outside it.
    pub fn narrow<G: FamilySet>(&self) -> Option<LogicalType<G>> {
        Some(LogicalType { family: G::admit(self.family.widen())?, facets: self.facets.clone(), nullable: self.nullable })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn core_family_wire_format_is_unchanged_and_refuses_opt_in_names() {
        assert_eq!(serde_json::to_value(Family::Decimal).unwrap(), "decimal");
        for name in ["timestamp", "float"] {
            assert!(serde_json::from_value::<Family>(serde_json::json!(name)).is_err());
            assert!(serde_json::from_value::<LogicalType>(serde_json::json!({"family":name,"facets":{},"nullable":false})).is_err());
        }
    }
    #[test]
    fn opt_in_families_use_umf_spellings_and_narrow_explicitly() {
        for name in ["boolean", "string", "integer", "decimal", "timestamp", "float"] {
            let family = ScalarFamily::from_umf(name).unwrap();
            assert_eq!(family.umf_name(), name);
            assert_eq!(serde_json::to_value(family).unwrap(), name);
        }
        assert_eq!(ScalarFamily::from_umf("time"), None);
        let timestamp = ScalarType::plain(ScalarFamily::Timestamp, true);
        assert_eq!(timestamp.narrow::<Family>(), None);
        let decimal = ScalarType::plain(ScalarFamily::Decimal, false);
        assert_eq!(decimal.narrow::<Family>(), Some(LogicalType::plain(Family::Decimal, false)));
        assert_eq!(LogicalType::plain(Family::Integer, true).widen(), ScalarType::plain(ScalarFamily::Integer, true));
    }
}
