//! Shared logical payload accounting for security simulation/composition.
use crate::{
    error::{Diagnostic, Result},
    security_literals::{normalized_literal, ScalarLiteral},
};
use serde::Serialize;
use serde_json::Value;
use std::io::{self, Write};

pub(crate) const NORMALIZED_LIMIT: usize = 4_000_000;
pub(crate) const COPY_LIMIT: usize = 16_000_000;
pub(crate) struct PayloadBudget {
    normalized: usize,
    copied: usize,
    code: &'static str,
}
impl PayloadBudget {
    pub(crate) fn new(code: &'static str) -> Self {
        Self {
            normalized: 0,
            copied: 0,
            code,
        }
    }
    fn fail(&self) -> Diagnostic {
        Diagnostic::new(self.code, "model", "Security payload budget exhausted")
    }
    pub(crate) fn scalar_cost(value: &ScalarLiteral) -> usize {
        match value {
            ScalarLiteral::String(v) | ScalarLiteral::Binary(v) => v.len(),
            ScalarLiteral::Number(v) => v.magnitude().bits().div_ceil(8) as usize,
            _ => 1,
        }
    }
    pub(crate) fn copy_scalar(&mut self, value: &ScalarLiteral) -> Result<()> {
        self.copy_bytes(Self::scalar_cost(value))
    }
    fn copy_bytes(&mut self, cost: usize) -> Result<()> {
        let next = self.copied.checked_add(cost).ok_or_else(|| self.fail())?;
        if next > COPY_LIMIT {
            return Err(self.fail());
        }
        self.copied = next;
        Ok(())
    }
    /// Count encoded JSON before cloning, without allocating a serialized buffer.
    /// Includes domain/ref/string metadata, so this is conservative scalar-copy accounting.
    pub(crate) fn copy_json<T: Serialize>(&mut self, value: &T) -> Result<()> {
        struct Counter(usize);
        impl Write for Counter {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                self.0 = self
                    .0
                    .checked_add(bytes.len())
                    .ok_or_else(|| io::Error::other("payload overflow"))?;
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut counter = Counter(0);
        serde_json::to_writer(&mut counter, value).map_err(|_| self.fail())?;
        self.copy_bytes(counter.0)
    }
    pub(crate) fn literal(
        &mut self,
        field: &Value,
        value: &Value,
    ) -> Result<Option<ScalarLiteral>> {
        // Reserve a proven upper bound before normalization allocates; reconcile
        // to the actual scalar bytes afterwards. Admitted numeric tokens may expand.
        let upper = literal_upper_bound(field, value).ok_or_else(|| self.fail())?;
        if self
            .normalized
            .checked_add(upper)
            .is_none_or(|n| n > NORMALIZED_LIMIT)
        {
            return Err(self.fail());
        }
        let result = normalized_literal(field, value)?;
        let cost = result.as_ref().map(Self::scalar_cost).unwrap_or(1);
        if cost > upper {
            return Err(self.fail());
        }
        self.normalized = self
            .normalized
            .checked_add(cost)
            .ok_or_else(|| self.fail())?;
        Ok(result)
    }
}
fn literal_upper_bound(field: &Value, value: &Value) -> Option<usize> {
    if value.is_null() {
        return Some(1);
    }
    match field["scalarType"].as_str()? {
        "string" => Some(value["string"].as_str()?.len()),
        "binary" => Some(value["binaryHex"].as_str()?.len()),
        "boolean" => Some(1),
        "integer" | "decimal" => {
            let token = value[if field["scalarType"] == "integer" {
                "integerToken"
            } else {
                "decimalToken"
            }]
            .as_str()?;
            let body = token.strip_prefix('-').unwrap_or(token);
            let (mantissa, exponent) = body.split_once(['e', 'E']).unwrap_or((body, "0"));
            // Zero coefficients never expand, even with a large authored exponent.
            // Full syntax/refinement validation still runs in normalized_literal.
            if mantissa.bytes().all(|b| b == b'0' || b == b'.') {
                return Some(0);
            }
            let exponent: i128 = exponent.parse().ok()?;
            let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
            let scale = if field["scalarType"] == "decimal" {
                crate::security_literals::normalized_facets(field).ok()?["scale"].as_u64()? as i128
            } else {
                0
            };
            let shift = exponent
                .checked_sub(fraction.len() as i128)?
                .checked_add(scale)?;
            // Each decimal digit needs < 4 bits. This bounds the BigInt byte
            // payload including rounding, without allocating its coefficient.
            let digits = (whole.len() as i128)
                .checked_add(fraction.len() as i128)?
                .checked_add(shift.max(0))?;
            usize::try_from(digits.checked_add(1)? / 2)
                .ok()
                .map(|n| n.max(1))
        }
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    // @covers US-008-AC2
    #[test]
    fn normalized_and_copy_budget_boundaries_are_independent() {
        let field = json!({"kind":"field","cardinality":"one","scalarType":"string","nullability":"required"});
        for bytes in [NORMALIZED_LIMIT - 1, NORMALIZED_LIMIT, NORMALIZED_LIMIT + 1] {
            let mut budget = PayloadBudget::new("WFT-SECURITY-EVALUATION");
            let value = json!({"string":"x".repeat(bytes)});
            assert_eq!(
                budget.literal(&field, &value).is_ok(),
                bytes <= NORMALIZED_LIMIT
            );
        }
        for bytes in [COPY_LIMIT - 1, COPY_LIMIT, COPY_LIMIT + 1] {
            let mut budget = PayloadBudget::new("WFT-SECURITY-EVALUATION");
            assert_eq!(budget.copy_bytes(bytes).is_ok(), bytes <= COPY_LIMIT);
        }
        let mut budget = PayloadBudget::new("WFT-SECURITY-EVALUATION");
        budget.copy_bytes(COPY_LIMIT).unwrap();
        assert!(budget.copy_scalar(&ScalarLiteral::Boolean(true)).is_err());
    }
    #[test]
    fn retained_json_copies_charge_before_cloning_at_exact_boundary() {
        for bytes in [COPY_LIMIT - 1, COPY_LIMIT, COPY_LIMIT + 1] {
            let mut budget = PayloadBudget::new("WFT-SECURITY-EVALUATION");
            // Encoded JSON string has two quotes, with no escaping for this input.
            let value = Value::String("x".repeat(bytes - 2));
            assert_eq!(budget.copy_json(&value).is_ok(), bytes <= COPY_LIMIT);
        }
        let mut budget = PayloadBudget::new("WFT-SECURITY-EVALUATION");
        let value = Value::String("x".repeat(999_998));
        for _ in 0..16 {
            budget.copy_json(&value).unwrap();
        }
        assert!(budget.copy_json(&Value::String(String::new())).is_err());
    }
    #[test]
    fn short_numeric_exponent_expansion_is_reserved_before_normalization() {
        let field = json!({"kind":"field","cardinality":"one","scalarType":"integer","nullability":"required"});
        let value = json!({"integerToken":"1e1000000"});
        let upper = literal_upper_bound(&field, &value).unwrap();
        assert!(upper > 500_000);
        let mut budget = PayloadBudget::new("WFT-SECURITY-EVALUATION");
        budget.normalized = NORMALIZED_LIMIT - upper + 1;
        assert!(budget.literal(&field, &value).is_err());
        budget.normalized = 0;
        let scalar = budget
            .literal(&field, &json!({"integerToken":"1e100"}))
            .unwrap()
            .unwrap();
        assert!(
            PayloadBudget::scalar_cost(&scalar)
                <= literal_upper_bound(&field, &json!({"integerToken":"1e100"})).unwrap()
        );
        budget.normalized = NORMALIZED_LIMIT;
        assert_eq!(
            budget
                .literal(
                    &field,
                    &json!({"integerToken":"0e99999999999999999999999999999999999999999"})
                )
                .unwrap(),
            Some(ScalarLiteral::Number(0.into()))
        );
    }
}
