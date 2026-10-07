pub mod access;
pub mod binding;
pub mod candidate;
pub mod collection;
mod json_codec;
pub mod presence_definition;
pub mod read_context_definition;
pub mod recursive_observation;
mod row_codec;
pub mod row_tree_mapping;
pub mod row_value_traversal;
pub mod structured;
mod tree;
pub mod value_definition;
// PostgreSQL emission primitives. Storage mappings and engine support are separate.
use weft_core::{
    backend::ParameterSlot,
    error::{Diagnostic, Result},
    ir::LogicalType,
};

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
            logical_type,
            value,
            origin,
        });
        Ok(format!("${position}"))
    }
    pub fn into_slots(self) -> Vec<ParameterSlot> {
        self.0
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
                family: weft_core::ir::Family::Integer,
                facets: serde_json::json!({"integerWidth":{"bits":bits,"signed":true}}),
                nullable: false,
            },
            text.into(),
            origin,
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use weft_core::ir::Family;
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
            slots[0].logical_type.facets,
            json!({"integerWidth":{"bits":32,"signed":true}})
        );
        assert_eq!(
            slots[3].logical_type.facets,
            json!({"integerWidth":{"bits":16,"signed":true}})
        );
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

pub mod leaf_codec_definition;

pub mod native_comparator_definition;

pub mod comparator_requirements;

pub mod row_join_definition;

pub mod property_definition;

pub mod numeric_correspondence;

pub mod registered_access;

pub mod expression;

pub mod relational;

pub mod record_definition;

pub mod result_definition;

pub mod select_definition;

pub mod row_custody;

pub mod value_traversal;

pub mod original_backend;
