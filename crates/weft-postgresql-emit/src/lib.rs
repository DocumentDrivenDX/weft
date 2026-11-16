// SPDX-License-Identifier: MIT OR Apache-2.0
//! PostgreSQL emission primitives shared by weft's PostgreSQL backends and other
//! compilers' PostgreSQL bindings: quoted identifiers, numbered parameter slots,
//! string literals, typed casts with the `C` collation, and result decoders.
//! Storage layouts (Truss, Fireweed) and engine support are separate bindings
//! over these primitives. See `README.md` for the versioned surface.
use weft_backend::sql::{ExactParameterKind, ParameterSlot, ParameterType, ScalarDecoder};
use weft_model::{
    descriptor::ExactScalarType,
    diagnostic::{Diagnostic, Result},
    types::{Family, FamilySet, LogicalType, ScalarFamily},
};

/// This crate's contract version (`README.md`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Exact quoted identifier in the proposed standard NAMEDATALEN=64 profile.
/// Nonstandard server builds require a separately qualified identifier profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(String);
impl Identifier {
    pub fn new(value: &str) -> Result<Self> {
        if value.is_empty() || value.len() > 63 || value.contains('\0') {
            return Err(Diagnostic::new(
                "WFT-BINDING",
                "binding",
                "PostgreSQL identifier is empty, contains NUL or exceeds 63 UTF-8 bytes",
            ));
        }
        Ok(Self(value.to_owned()))
    }
    pub fn sql(&self) -> String {
        format!("\"{}\"", self.0.replace('"', "\"\""))
    }
}
/// Namespace and object are separately quoted; dots in either remain literal.
pub fn qualified(namespace: &Identifier, object: &Identifier) -> String {
    format!("{}.{}", namespace.sql(), object.sql())
}
/// Per-query slots: values never enter emitted SQL. Common emission validation
/// remains responsible for exact lexical/domain checking before any artifact.
#[derive(Debug, Default, Clone)]
pub struct Parameters(Vec<ParameterSlot>);
impl Parameters {
    pub fn push(
        &mut self,
        logical_type: LogicalType,
        value: String,
        origin: serde_json::Value,
    ) -> Result<String> {
        if self.0.len() >= 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "emit",
                "PostgreSQL parameter count exceeds 1024",
            ));
        }
        let position = self.0.len() + 1;
        self.0.push(ParameterSlot {
            position,
            logical_type: logical_type.into(),
            value,
            origin,
        });
        Ok(format!("${position}"))
    }
    pub fn push_binary(&mut self, value: String, origin: serde_json::Value) -> Result<String> {
        use base64::Engine;
        if value.len() > 1048576 || self.0.len() >= 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "emit",
                "Binary parameter bounds exceeded",
            ));
        }
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&value)
            .map_err(|_| {
                Diagnostic::new("WFT-BINARY-DOMAIN", "emit", "Canonical base64 required")
            })?;
        if base64::engine::general_purpose::STANDARD.encode(bytes) != value {
            return Err(Diagnostic::new(
                "WFT-BINARY-DOMAIN",
                "emit",
                "Canonical base64 required",
            ));
        }
        let position = self.0.len() + 1;
        self.0.push(ParameterSlot {
            position,
            logical_type: ParameterType::Exact {
                kind: ExactParameterKind::ExactScalar,
                scalar_type: ExactScalarType::Binary,
            },
            value,
            origin,
        });
        Ok(format!("${position}"))
    }
    pub fn push_timestamp(&mut self, value: String, origin: serde_json::Value) -> Result<String> {
        if value.len() > 1048576 || self.0.len() >= 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "emit",
                "Timestamp parameter bounds exceeded",
            ));
        }
        let position = self.0.len() + 1;
        self.0.push(ParameterSlot {
            position,
            logical_type: ParameterType::Exact {
                kind: ExactParameterKind::ExactScalar,
                scalar_type: ExactScalarType::Timestamp,
            },
            value,
            origin,
        });
        Ok(format!("${position}"))
    }
    pub fn push_date(&mut self, value: String, origin: serde_json::Value) -> Result<String> {
        if value.len() != 10 || self.0.len() >= 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "emit",
                "Date parameter bounds exceeded",
            ));
        }
        let position = self.0.len() + 1;
        self.0.push(ParameterSlot {
            position,
            logical_type: ParameterType::Exact {
                kind: ExactParameterKind::ExactScalar,
                scalar_type: ExactScalarType::Date,
            },
            value,
            origin,
        });
        Ok(format!("${position}"))
    }
    pub fn into_slots(self) -> Vec<ParameterSlot> {
        self.0
    }
    /// Slots pushed so far, in position order.
    pub fn slots(&self) -> &[ParameterSlot] {
        &self.0
    }
    /// Replace the recorded origin of slot `index` (0-based). Values, types and
    /// positions cannot be changed after a push.
    pub fn set_origin(&mut self, index: usize, origin: serde_json::Value) -> bool {
        self.0.get_mut(index).map(|slot| slot.origin = origin).is_some()
    }
}
/// Native catalog discriminator domain, independent of logical business keys.
/// Historical nonpositive IDs are retained; allocation policy belongs to Truss.
#[derive(Debug, Clone, Copy)]
pub enum CatalogDomain {
    Int,
    SmallInt,
}
impl Parameters {
    pub fn catalog(
        &mut self,
        domain: CatalogDomain,
        text: &str,
        origin: serde_json::Value,
    ) -> Result<String> {
        let number = text.parse::<i64>().ok();
        let bits = match domain {
            CatalogDomain::Int => 32,
            CatalogDomain::SmallInt => 16,
        };
        let valid = number.is_some_and(|value| {
            value.to_string() == text
                && match domain {
                    CatalogDomain::Int => i32::try_from(value).is_ok(),
                    CatalogDomain::SmallInt => i16::try_from(value).is_ok(),
                }
        });
        if !valid {
            return Err(Diagnostic::new(
                "WFT-BINDING",
                "binding",
                "Catalog discriminator is not canonical text in its signed native domain",
            ));
        }
        self.push(
            LogicalType {
                family: Family::Integer,
                facets: serde_json::json!({"integerWidth":{"bits":bits,"signed":true}}),
                nullable: false,
            },
            text.into(),
            origin,
        )
    }
}
/// The bytewise collation every emitted text comparison and ordering uses.
pub const COLLATE_C: &str = "COLLATE pg_catalog.\"C\"";
/// `expression COLLATE pg_catalog."C"`.
pub fn collate_c(expression: &str) -> String {
    format!("{expression} {COLLATE_C}")
}
/// A standard-conforming string literal `'...'`, or `None` for text containing
/// NUL, which PostgreSQL text cannot represent.
pub fn string_literal(text: &str) -> Option<String> {
    (!text.contains('\0')).then(|| format!("'{}'", text.replace('\'', "''")))
}
/// An escape string literal `E'...'` (backslashes doubled), or `None` for NUL.
pub fn escape_string_literal(text: &str) -> Option<String> {
    (!text.contains('\0')).then(|| format!("E'{}'", text.replace('\\', "\\\\").replace('\'', "''")))
}
/// The `pg_catalog` type a value of `family` is carried as: `text`, `bool`,
/// `numeric` (integers and decimals, exact), and for opt-in families
/// `timestamptz` and `float8`.
pub fn cast_type<F: FamilySet>(family: &F) -> &'static str {
    match family.widen() {
        ScalarFamily::String => "text",
        ScalarFamily::Boolean => "bool",
        ScalarFamily::Integer | ScalarFamily::Decimal => "numeric",
        ScalarFamily::Timestamp => "timestamptz",
        ScalarFamily::Float => "float8",
    }
}
/// `expression::pg_catalog.<type>`, collated `C` when the family is string.
pub fn typed<F: FamilySet>(expression: &str, family: &F) -> String {
    let value = format!("{expression}::pg_catalog.{}", cast_type(family));
    if family.widen() == ScalarFamily::String { collate_c(&value) } else { value }
}
/// The exact result decoder for a column of a core family.
pub fn decoder(family: &Family) -> ScalarDecoder {
    match family {
        Family::String => ScalarDecoder::Text,
        Family::Boolean => ScalarDecoder::Boolean,
        Family::Integer => ScalarDecoder::ExactInteger,
        Family::Decimal => ScalarDecoder::ExactDecimal,
    }
}
/// The decoder for any family set; `None` where the SQL carrier has no exact
/// decoder yet (timestamp and float columns).
pub fn decoder_for<F: FamilySet>(family: &F) -> Option<ScalarDecoder> {
    Family::try_from(family.widen()).ok().map(|family| decoder(&family))
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use weft_model::types::Family;
    #[test]
    fn identifiers_quote_every_component_and_never_truncate() {
        let namespace = Identifier::new("sales; DROP SCHEMA x;--").unwrap();
        let table = Identifier::new("odd.\"object").unwrap();
        assert_eq!(
            qualified(&namespace, &table),
            "\"sales; DROP SCHEMA x;--\".\"odd.\"\"object\""
        );
        assert_eq!(Identifier::new("MiXeD").unwrap().sql(), "\"MiXeD\"");
        assert!(Identifier::new(&"é".repeat(31)).is_ok());
        assert!(Identifier::new(&"é".repeat(32)).is_err());
        assert!(Identifier::new(&"a".repeat(63)).is_ok());
        for invalid in ["", "a\0b", &"a".repeat(64)] {
            assert!(Identifier::new(invalid).is_err());
        }
    }
    #[test]
    fn catalog_slots_keep_signed_native_ids_and_refuse_overflow() {
        let mut parameters = Parameters::default();
        for text in ["-2147483648", "0", "2147483647"] {
            parameters
                .catalog(CatalogDomain::Int, text, json!({"catalog":"type"}))
                .unwrap();
        }
        for text in ["-32768", "0", "32767"] {
            parameters
                .catalog(CatalogDomain::SmallInt, text, json!({"catalog":"key"}))
                .unwrap();
        }
        for (domain, text) in [
            (CatalogDomain::Int, "2147483648"),
            (CatalogDomain::Int, "-2147483649"),
            (CatalogDomain::SmallInt, "32768"),
            (CatalogDomain::SmallInt, "-32769"),
            (CatalogDomain::Int, "-0"),
            (CatalogDomain::Int, "01"),
            (CatalogDomain::Int, "+1"),
            (CatalogDomain::Int, "1.0"),
            (CatalogDomain::Int, "1;DROP TABLE x"),
        ] {
            assert_eq!(
                parameters
                    .catalog(domain, text, json!({}))
                    .unwrap_err()
                    .code,
                "WFT-BINDING"
            );
        }
        let slots = parameters.into_slots();
        assert_eq!(slots.len(), 6);
        assert_eq!(slots[0].value, "-2147483648");
        assert_eq!(
            slots[0].logical_type.atomic().unwrap().facets,
            json!({"integerWidth":{"bits":32,"signed":true}})
        );
        assert_eq!(
            slots[3].logical_type.atomic().unwrap().facets,
            json!({"integerWidth":{"bits":16,"signed":true}})
        );
    }
    #[test]
    fn literals_casts_collation_and_decoders() {
        assert_eq!(string_literal("it's").unwrap(), "'it''s'");
        assert_eq!(escape_string_literal(r"a\b'c").unwrap(), r"E'a\\b''c'");
        assert!(string_literal("a\0").is_none() && escape_string_literal("a\0").is_none());
        assert_eq!(typed("$1", &Family::String), "$1::pg_catalog.text COLLATE pg_catalog.\"C\"");
        assert_eq!(typed("$2", &Family::Decimal), "$2::pg_catalog.numeric");
        assert_eq!(typed("$3", &ScalarFamily::Timestamp), "$3::pg_catalog.timestamptz");
        assert_eq!(cast_type(&ScalarFamily::Float), "float8");
        assert!(matches!(decoder(&Family::Integer), ScalarDecoder::ExactInteger));
        assert!(decoder_for(&ScalarFamily::Float).is_none());
        assert!(matches!(decoder_for(&ScalarFamily::Boolean), Some(ScalarDecoder::Boolean)));
    }
    #[test]
    fn slots_preserve_exact_values_and_origins_without_deduplicating() {
        let ty = LogicalType {
            family: Family::Integer,
            facets: json!({"integerWidth":{"bits":64,"signed":false}}),
            nullable: false,
        };
        let mut slots = Parameters::default();
        for i in 0..1024 {
            assert_eq!(
                slots
                    .push(
                        ty.clone(),
                        "18446744073709551615".into(),
                        json!({"literal":i})
                    )
                    .unwrap(),
                format!("${}", i + 1)
            );
        }
        assert_eq!(
            slots.push(ty, "1".into(), json!({})).unwrap_err().code,
            "WFT-LIMIT"
        );
        let slots = slots.into_slots();
        assert_eq!(slots.len(), 1024);
        assert_eq!(slots[1023].value, "18446744073709551615");
        assert_eq!(slots[1023].origin, json!({"literal":1023}));
    }
}

