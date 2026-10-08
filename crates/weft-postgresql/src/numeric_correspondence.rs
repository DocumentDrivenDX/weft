//! Truss NX compact mathematical witness after separately admitted source/native parsing.
//! No token grammar, facets, native representability or operation capability is granted.
use weft_core::error::{Diagnostic, Result};
pub struct Parts<'a> {
    pub negative: bool,
    pub digits: &'a str,
    pub fraction_digits: u64,
    pub exponent: i64,
}
pub struct Budget {
    pub remaining_steps: usize,
    pub max_digits: usize,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Witness {
    pub negative: bool,
    pub digits: String,
    pub exponent: i64,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-LIMIT", "decode", message)
}
/// Reserve a conservative four steps per digit plus fixed bookkeeping on every
/// occurrence. The shared invocation budget is not reset by equal inputs/hashes.
pub fn witness(parts: Parts<'_>, budget: &mut Budget) -> Result<Witness> {
    let size = parts.digits.len();
    if size == 0 || size > budget.max_digits {
        return Err(fail("Numeric witness digit bound refused"));
    }
    let steps = size
        .checked_mul(4)
        .and_then(|n| n.checked_add(8))
        .ok_or_else(|| fail("Numeric witness work accounting overflow"))?;
    budget.remaining_steps = budget
        .remaining_steps
        .checked_sub(steps)
        .ok_or_else(|| fail("Numeric witness work budget exhausted"))?;
    if !parts.digits.bytes().all(|b| b.is_ascii_digit()) || parts.fraction_digits > size as u64 {
        return Err(Diagnostic::new(
            "WFT-BINDING",
            "decode",
            "Numeric parsed parts are inconsistent",
        ));
    }
    let fraction = i64::try_from(parts.fraction_digits)
        .map_err(|_| fail("Numeric fraction count overflow"))?;
    // Validate arithmetic even for zero; zero must not bypass counters/admission.
    let exponent = parts
        .exponent
        .checked_sub(fraction)
        .ok_or_else(|| fail("Numeric exponent arithmetic overflow"))?;
    let nonzero = parts.digits.trim_start_matches('0');
    if nonzero.is_empty() {
        return Ok(Witness {
            negative: false,
            digits: "0".into(),
            exponent: 0,
        });
    }
    let digits = nonzero.trim_end_matches('0');
    let removed = i64::try_from(nonzero.len() - digits.len())
        .map_err(|_| fail("Numeric trailing digit count overflow"))?;
    let exponent = exponent
        .checked_add(removed)
        .ok_or_else(|| fail("Numeric exponent arithmetic overflow"))?;
    Ok(Witness {
        negative: parts.negative,
        digits: digits.into(),
        exponent,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_mathematical_vectors_and_pairs_without_exponent_expansion() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/upstream/numeric-correspondence-vectors.json"
        ))
        .unwrap();
        let pin: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/upstream/numeric-correspondence-source-pin.json"
        ))
        .unwrap();
        assert_eq!(
            weft_core::json::sha256(include_bytes!(
                "../../../tests/truss-postgresql/upstream/numeric-correspondence-vectors.json"
            )),
            pin["sha256"].as_str().unwrap()
        );
        let mut budget = Budget {
            remaining_steps: 10000,
            max_digits: 1000,
        };
        let mut results = Vec::new();
        for vector in fixture["vectors"].as_array().unwrap() {
            // Fixture-only splitting; production source/native grammars remain separate.
            let token = vector["token"].as_str().unwrap();
            let (mantissa, exponent) = token.split_once('e').unwrap_or((token, "0"));
            let negative = mantissa.starts_with('-');
            let mantissa = mantissa.trim_start_matches('-');
            let fraction = mantissa.split_once('.').map_or(0, |(_, frac)| frac.len());
            let digits = mantissa.replace('.', "");
            let result = witness(
                Parts {
                    negative,
                    digits: &digits,
                    fraction_digits: fraction as u64,
                    exponent: exponent.parse().unwrap(),
                },
                &mut budget,
            )
            .unwrap();
            assert_eq!(result.negative, vector["negative"].as_bool().unwrap());
            assert_eq!(result.digits, vector["digits"].as_str().unwrap());
            assert_eq!(
                result.exponent.to_string(),
                vector["exponent"].as_str().unwrap()
            );
            results.push(result);
        }
        for pair in fixture["pairs"].as_array().unwrap() {
            assert_eq!(
                results[pair["left"].as_u64().unwrap() as usize]
                    == results[pair["right"].as_u64().unwrap() as usize],
                pair["mathematicallyEqual"].as_bool().unwrap()
            );
        }
    }
    #[test]
    fn zero_overflow_malformed_parts_and_repeated_budget_are_not_bypassed() {
        let mut budget = Budget {
            remaining_steps: 100,
            max_digits: 10,
        };
        assert!(witness(
            Parts {
                negative: true,
                digits: "00",
                fraction_digits: 1,
                exponent: i64::MIN
            },
            &mut budget
        )
        .is_err());
        assert!(witness(
            Parts {
                negative: false,
                digits: "1x",
                fraction_digits: 0,
                exponent: 0
            },
            &mut budget
        )
        .is_err());
        assert!(witness(
            Parts {
                negative: false,
                digits: "1",
                fraction_digits: 2,
                exponent: 0
            },
            &mut budget
        )
        .is_err());
        let mut budget = Budget {
            remaining_steps: 12,
            max_digits: 10,
        };
        witness(
            Parts {
                negative: false,
                digits: "1",
                fraction_digits: 0,
                exponent: 1000000,
            },
            &mut budget,
        )
        .unwrap();
        assert!(witness(
            Parts {
                negative: false,
                digits: "1",
                fraction_digits: 0,
                exponent: 1000000
            },
            &mut budget
        )
        .is_err());
    }
}
