//! Selected native comparator meaning and explicit operation admission.
use crate::leaf_codec_definition::OriginalArtifact;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    error::{Diagnostic, Result},
    ir::{Family, LogicalType},
    json::{checked_json, sha256},
};
#[jsonschema::validator(
    path = "../../tests/truss-postgresql/upstream/native-comparator-schema-bundle.json"
)]
struct Shape;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operation {
    Equality,
    Ordering,
    Key,
    Sum,
}
/// Operation meanings are selected by the trusted backend registry. Merely
/// supplying qualification bytes cannot populate this set.
pub struct Selection<'a> {
    pub profile: &'a Value,
    pub native_profile: &'a Value,
    pub original_artifacts: &'a BTreeMap<String, OriginalArtifact>,
    pub operations: &'a BTreeSet<Operation>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Strategy {
    UnicodeText,
    Boolean,
    SignedInteger,
    UnsignedInteger,
    FiniteDecimal,
}
#[derive(Debug, Clone)]
pub struct Definition {
    pub original_json: String,
    pub original_artifacts: BTreeMap<String, Vec<u8>>,
    pub strategy: Strategy,
    pub native_type: String,
    operations: BTreeSet<Operation>,
    logical_type: LogicalType,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
impl Definition {
    pub fn parse(raw: &str, selected: Selection<'_>, logical: &LogicalType) -> Result<Self> {
        if raw.len() > 4 * 1024 * 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "binding",
                "Comparator exceeds candidate byte bound",
            ));
        }
        let value =
            checked_json(raw).map_err(|_| fail("Malformed or duplicate-member comparator"))?;
        if !Shape::is_valid(&value)
            || &value["profile"] != selected.profile
            || &value["nativeDomainProfile"] != selected.native_profile
        {
            return Err(fail("Comparator grammar or registered profile differs"));
        }
        if logical.nullable {
            return Err(fail("Comparator cannot admit nullable operands"));
        }
        let facets = logical
            .facets
            .as_object()
            .ok_or_else(|| fail("Original facets are not an object"))?;
        let complete = match logical.family {
            Family::String | Family::Boolean => facets.is_empty(),
            Family::Integer => {
                facets.len() == 1
                    && facets.contains_key("integerWidth")
                    && logical.facets["integerWidth"]
                        .as_object()
                        .is_some_and(|width| {
                            width.len() == 2
                                && width.contains_key("bits")
                                && width.contains_key("signed")
                        })
            }
            Family::Decimal => {
                facets.len() == 2
                    && facets.contains_key("precision")
                    && facets.contains_key("scale")
            }
        };
        if !complete {
            return Err(fail(
                "Comparator cannot discard unknown or incomplete authored facets",
            ));
        }
        let native = value["strategy"]["nativeType"].as_str().unwrap();
        let kind = value["strategy"]["kind"].as_str().unwrap();
        let strategy = match kind {
            "unicode-text-C" if logical.family == Family::String => Strategy::UnicodeText,
            "native-boolean" if logical.family == Family::Boolean => Strategy::Boolean,
            "signed-integer" | "unsigned-integer" if logical.family == Family::Integer => {
                let width = &logical.facets["integerWidth"];
                let bits = width["bits"]
                    .as_u64()
                    .filter(|n| (1..=64).contains(n))
                    .ok_or_else(|| fail("Original integer width is unavailable"))?;
                let signed = width["signed"]
                    .as_bool()
                    .ok_or_else(|| fail("Original integer signedness is unavailable"))?;
                if signed != (kind == "signed-integer") {
                    return Err(fail("Comparator signedness differs from authored integer"));
                }
                let capacity = match native {
                    "pg_catalog.int2" => 16,
                    "pg_catalog.int4" => 32,
                    "pg_catalog.int8" => 64,
                    "pg_catalog.numeric" => 64,
                    _ => return Err(fail("Unknown integer native domain")),
                };
                if bits > capacity {
                    return Err(fail(
                        "Comparator native integer domain narrows authored width",
                    ));
                }
                if signed {
                    Strategy::SignedInteger
                } else {
                    Strategy::UnsignedInteger
                }
            }
            "finite-decimal" if logical.family == Family::Decimal => {
                let precision = logical.facets["precision"]
                    .as_u64()
                    .filter(|p| (1..=28).contains(p))
                    .ok_or_else(|| fail("Original decimal precision is unavailable"))?;
                if logical.facets["scale"]
                    .as_u64()
                    .is_none_or(|s| s > precision)
                {
                    return Err(fail("Original decimal scale is unavailable"));
                }
                Strategy::FiniteDecimal
            }
            _ => {
                return Err(fail(
                    "Comparator strategy differs from authored scalar family",
                ))
            }
        };
        let paths = [
            "valueDefinition",
            "sourceDomainDefinition",
            "nativeDomainDefinition",
            "operatorInventory",
            "qualification",
        ];
        if paths.len() != selected.original_artifacts.len() {
            return Err(fail("Comparator original artifact closure differs"));
        }
        let mut artifacts = BTreeMap::new();
        let mut total = 0usize;
        for path in paths {
            let artifact = &value[path];
            let encoded = artifact["bytesBase64"].as_str().unwrap();
            let bytes = STANDARD
                .decode(encoded)
                .map_err(|_| fail("Comparator artifact base64 refused"))?;
            total = total
                .checked_add(bytes.len())
                .ok_or_else(|| fail("Comparator artifact accounting overflow"))?;
            if total > 4 * 1024 * 1024 {
                return Err(Diagnostic::new(
                    "WFT-LIMIT",
                    "binding",
                    "Comparator artifact bound exceeded",
                ));
            }
            if STANDARD.encode(&bytes) != encoded
                || artifact["sha256"] != sha256(&bytes)
                || selected
                    .original_artifacts
                    .get(path)
                    .is_none_or(|original| {
                        original.bytes != bytes || artifact["identity"] != original.identity
                    })
            {
                return Err(fail(
                    "Comparator artifact differs from original registered selection",
                ));
            }
            artifacts.insert(path.into(), bytes);
        }
        Ok(Self {
            original_json: raw.into(),
            original_artifacts: artifacts,
            strategy,
            native_type: native.into(),
            operations: selected.operations.clone(),
            logical_type: logical.clone(),
        })
    }
    /// Trusted scalar SQL is supplied by the admitted physical access plan.
    /// Domain/transport preflight must precede execution; this never validates
    /// source values by an unchecked PostgreSQL cast.
    /// Native domain check only; original source-token grammar remains a
    /// separate selected host/codec obligation.
    pub fn numeric_domain_sql(&self, carrier: &str) -> Result<String> {
        let _ = self.numeric_native_type()?;
        numeric_domain_for_type(&self.logical_type, carrier)
    }
    pub fn sum_sql(&self, carrier: &str) -> Result<String> {
        self.require(Operation::Sum)?;
        let native = self.numeric_native_type()?;
        Ok(format!("pg_catalog.sum(({carrier})::{native})"))
    }
    fn numeric_native_type(&self) -> Result<&str> {
        let original = checked_json(&self.original_json)
            .map_err(|_| fail("Original comparator JSON refused"))?;
        if original["strategy"]["nativeType"] != self.native_type {
            return Err(fail("SUM native type differs from original comparator"));
        }
        let kind = match self.strategy {
            Strategy::SignedInteger => "signed-integer",
            Strategy::UnsignedInteger => "unsigned-integer",
            Strategy::FiniteDecimal => "finite-decimal",
            _ => return Err(fail("SUM has no numeric comparator strategy")),
        };
        if original["strategy"]["kind"] != kind {
            return Err(fail("SUM strategy differs from original comparator"));
        }
        let native = match self.strategy {
            Strategy::SignedInteger
                if matches!(
                    self.native_type.as_str(),
                    "pg_catalog.int2"
                        | "pg_catalog.int4"
                        | "pg_catalog.int8"
                        | "pg_catalog.numeric"
                ) =>
            {
                self.native_type.as_str()
            }
            Strategy::UnsignedInteger | Strategy::FiniteDecimal
                if self.native_type == "pg_catalog.numeric" =>
            {
                "pg_catalog.numeric"
            }
            _ => return Err(fail("SUM lacks its closed exact numeric native strategy")),
        };
        Ok(native)
    }
    pub fn require_type(&self, logical: &LogicalType) -> Result<()> {
        if &self.logical_type != logical {
            return Err(fail(
                "Requested comparator type differs from admitted original domain",
            ));
        }
        Ok(())
    }
    pub fn require(&self, operation: Operation) -> Result<()> {
        if !self.operations.contains(&operation)
            || (operation == Operation::Sum
                && matches!(self.strategy, Strategy::UnicodeText | Strategy::Boolean))
        {
            return Err(Diagnostic::new(
                "WFT-CAPABILITY",
                "lower",
                "Comparator operation has no selected registered meaning",
            ));
        }
        Ok(())
    }
}
/// Exact PostgreSQL numeric-domain predicate for an already admitted UMF type.
/// This does not choose a comparator, source-token grammar or storage encoding.
pub(crate) fn numeric_domain_for_type(logical_type: &LogicalType, carrier: &str) -> Result<String> {
    let value = format!("({carrier})::pg_catalog.numeric");
    let facets = &logical_type.facets;
    let bounds = match logical_type.family {
        Family::Decimal => {
            let precision = facets["precision"]
                .as_u64()
                .ok_or_else(|| fail("Decimal precision missing"))?;
            let scale = facets["scale"]
                .as_u64()
                .ok_or_else(|| fail("Decimal scale missing"))?;
            let integral = precision
                .checked_sub(scale)
                .ok_or_else(|| fail("Decimal scale exceeds precision"))?;
            format!("{value}=pg_catalog.trunc({value},{scale}) AND pg_catalog.abs({value})<pg_catalog.power(10::pg_catalog.numeric,{integral})")
        }
        Family::Integer => {
            let bits = facets["integerWidth"]["bits"]
                .as_u64()
                .ok_or_else(|| fail("Integer width missing"))?;
            let signed = facets["integerWidth"]["signed"]
                .as_bool()
                .ok_or_else(|| fail("Integer signedness missing"))?;
            let exponent = if signed {
                bits.checked_sub(1)
                    .ok_or_else(|| fail("Integer width invalid"))?
            } else {
                bits
            };
            let lower = if signed {
                format!("-pg_catalog.power(2::pg_catalog.numeric,{exponent})")
            } else {
                "0".into()
            };
            format!("{value}=pg_catalog.trunc({value}) AND {value}>={lower} AND {value}<pg_catalog.power(2::pg_catalog.numeric,{exponent})")
        }
        _ => return Err(fail("SUM has no exact numeric domain")),
    };
    Ok(format!("CASE WHEN pg_catalog.pg_input_is_valid(({carrier})::pg_catalog.text,'pg_catalog.numeric') THEN ({value}::pg_catalog.text NOT IN ('NaN','Infinity','-Infinity') AND {bounds}) ELSE FALSE END"))
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn fixture(strategy: Value) -> (Value, BTreeMap<String, OriginalArtifact>) {
        let pin = json!({"identity":"fixture","version":"0.1.0","sha256":sha256(b"{}")});
        let artifact = json!({"identity":"fixture","bytesBase64":STANDARD.encode(b"{}"),"sha256":sha256(b"{}")});
        let originals = [
            "valueDefinition",
            "sourceDomainDefinition",
            "nativeDomainDefinition",
            "operatorInventory",
            "qualification",
        ]
        .into_iter()
        .map(|key| {
            (
                key.into(),
                OriginalArtifact {
                    identity: "fixture".into(),
                    bytes: b"{}".to_vec(),
                },
            )
        })
        .collect();
        (
            json!({"interfaceVersion":"truss-native-comparator/0.1.0","profile":pin,"valueDefinition":artifact,"sourceDomainDefinition":artifact,"nativeDomainProfile":pin,"nativeDomainDefinition":artifact,"operatorInventory":artifact,"strategy":strategy,"castOutcome":"exact-or-error","nullOperands":"refuse","absentOperands":"refuse","qualification":artifact}),
            originals,
        )
    }
    fn integer(signed: bool, bits: u64) -> LogicalType {
        LogicalType {
            family: Family::Integer,
            facets: json!({"integerWidth":{"signed":signed,"bits":bits}}),
            nullable: false,
        }
    }
    fn parse(
        value: &Value,
        originals: &BTreeMap<String, OriginalArtifact>,
        logical: &LogicalType,
        operations: &BTreeSet<Operation>,
    ) -> Result<Definition> {
        let pin = fixture(json!({})).0["profile"].clone();
        Definition::parse(
            &value.to_string(),
            Selection {
                profile: &pin,
                native_profile: &pin,
                original_artifacts: originals,
                operations,
            },
            logical,
        )
    }
    #[test]
    fn selected_numeric_sum_sql_preserves_exact_native_types() {
        let mut captures = Vec::new();
        for (name, strategy, logical) in [
            (
                "uint64",
                json!({"kind":"unsigned-integer","nativeType":"pg_catalog.numeric","integrality":"validate-before-cast","range":"original-authored-unsigned-facets"}),
                integer(false, 64),
            ),
            (
                "decimal",
                json!({"kind":"finite-decimal","nativeType":"pg_catalog.numeric","scaleCoercion":"forbidden","nonfinite":"refuse"}),
                LogicalType {
                    family: Family::Decimal,
                    facets: json!({"precision":28,"scale":9}),
                    nullable: false,
                },
            ),
        ] {
            let (value, originals) = fixture(strategy);
            let mut definition = parse(
                &value,
                &originals,
                &logical,
                &BTreeSet::from([Operation::Sum]),
            )
            .unwrap();
            let sql = format!(
                "SELECT ({})::pg_catalog.text AS total FROM values_table",
                definition.sum_sql("value").unwrap()
            );
            captures.push(json!({"name":name,"sql":sql}));
            let original_strategy =
                std::mem::replace(&mut definition.strategy, Strategy::SignedInteger);
            assert!(definition.sum_sql("value").is_err());
            definition.strategy = original_strategy;
            definition.native_type = "pg_catalog.float8".into();
            assert!(definition.sum_sql("value").is_err());
        }
        if let Ok(path) = std::env::var("WEFT_NUMERIC_SUM_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
        }
    }
    #[test]
    fn unsigned64_and_operation_registration_are_independent() {
        let (value, originals) = fixture(
            json!({"kind":"unsigned-integer","nativeType":"pg_catalog.numeric","integrality":"validate-before-cast","range":"original-authored-unsigned-facets"}),
        );
        let definition = parse(
            &value,
            &originals,
            &integer(false, 64),
            &BTreeSet::from([Operation::Equality]),
        )
        .unwrap();
        assert_eq!(definition.strategy, Strategy::UnsignedInteger);
        definition.require(Operation::Equality).unwrap();
        for operation in [Operation::Ordering, Operation::Key, Operation::Sum] {
            assert!(definition.require(operation).is_err());
        }
        assert!(parse(&value, &originals, &integer(true, 64), &BTreeSet::new()).is_err());
        let definition = parse(&value, &originals, &integer(false, 64), &BTreeSet::new()).unwrap();
        assert!(definition.require(Operation::Equality).is_err());
    }
    #[test]
    fn narrowing_and_rehashed_native_meaning_refuse() {
        let (value, originals) = fixture(
            json!({"kind":"signed-integer","nativeType":"pg_catalog.int2","integrality":"validate-before-cast"}),
        );
        parse(&value, &originals, &integer(true, 16), &BTreeSet::new()).unwrap();
        assert!(parse(&value, &originals, &integer(true, 17), &BTreeSet::new()).is_err());
        let mut wrong = value.clone();
        wrong["operatorInventory"] = json!({"identity":"fixture","bytesBase64":STANDARD.encode(b"replacement"),"sha256":sha256(b"replacement")});
        assert!(parse(&wrong, &originals, &integer(true, 16), &BTreeSet::new()).is_err());
        let mut wrong = value;
        wrong["nativeDomainProfile"]["version"] = json!("unknown");
        assert!(parse(&wrong, &originals, &integer(true, 16), &BTreeSet::new()).is_err());
    }
    #[test]
    fn boolean_cannot_gain_numeric_sum_from_registration() {
        let (value, originals) =
            fixture(json!({"kind":"native-boolean","nativeType":"pg_catalog.bool"}));
        let logical = LogicalType {
            family: Family::Boolean,
            facets: json!({}),
            nullable: false,
        };
        let definition = parse(
            &value,
            &originals,
            &logical,
            &BTreeSet::from([Operation::Ordering, Operation::Sum]),
        )
        .unwrap();
        definition.require(Operation::Ordering).unwrap();
        assert!(definition.require(Operation::Sum).is_err());
        let mut nullable = logical;
        nullable.nullable = true;
        assert!(parse(&value, &originals, &nullable, &BTreeSet::new()).is_err());
    }
    #[test]
    fn text_and_decimal_strategies_keep_independent_operations_and_facets() {
        let (text, originals) = fixture(
            json!({"kind":"unicode-text-C","nativeType":"pg_catalog.text","encoding":"UTF8","collation":"pg_catalog.C","normalization":"none"}),
        );
        let logical = LogicalType {
            family: Family::String,
            facets: json!({}),
            nullable: false,
        };
        let definition = parse(
            &text,
            &originals,
            &logical,
            &BTreeSet::from([Operation::Equality, Operation::Key]),
        )
        .unwrap();
        definition.require(Operation::Key).unwrap();
        assert!(definition.require(Operation::Ordering).is_err());
        let mut unknown = logical;
        unknown.facets = json!({"pattern":".*"});
        assert!(parse(&text, &originals, &unknown, &BTreeSet::new()).is_err());
        let (decimal, originals) = fixture(
            json!({"kind":"finite-decimal","nativeType":"pg_catalog.numeric","scaleCoercion":"forbidden","nonfinite":"refuse"}),
        );
        let mut logical = LogicalType {
            family: Family::Decimal,
            facets: json!({"precision":28,"scale":9}),
            nullable: false,
        };
        let definition = parse(
            &decimal,
            &originals,
            &logical,
            &BTreeSet::from([Operation::Sum]),
        )
        .unwrap();
        definition.require(Operation::Sum).unwrap();
        assert_eq!(
            definition.sum_sql("value").unwrap(),
            "pg_catalog.sum((value)::pg_catalog.numeric)"
        );
        assert!(definition.require(Operation::Equality).is_err());
        logical.facets["scale"] = json!(29);
        assert!(parse(&decimal, &originals, &logical, &BTreeSet::new()).is_err());
    }
}
