// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exact literal tokens and their value-domain checks. Tokens are never converted
//! through native floats; the original text is what a compiler retains.
use crate::{
    diagnostic::{Diagnostic, Result},
    types::{FamilySet, LogicalType, ScalarFamily, Span},
};
use num_bigint::BigInt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralKind {
    Number,
    String,
    Boolean,
}
/// A source literal token: base-ten number text, string text, or `true`/`false`.
#[derive(Debug, Clone)]
pub struct Literal {
    pub value: String,
    pub kind: LiteralKind,
    pub span: Span,
}
/// Largest token any exact check reads.
pub const MAX_TOKEN: usize = 4_000_000;

/// Check that `input` is a value of `expected`'s family and facets.
///
/// Core families keep their weft-sql 0.1 rules. `Timestamp` takes a string token
/// in the RFC 3339 exact-instant profile ([`crate::exact_instant`]); `Float`
/// takes a number token whose nearest binary64 value is finite. Those two
/// families exist only in [`ScalarFamily`]; weft's core [`crate::types::Family`]
/// cannot name them.
pub fn check_literal<F: FamilySet>(input: &Literal, expected: &LogicalType<F>) -> Result<()> {
    let error = |code, message| Diagnostic::new(code, "type", message).at(&input.span);
    match (&input.kind, expected.family.widen()) {
        (LiteralKind::String, ScalarFamily::String) => {
            crate::string_domain::validate_facets(&expected.facets, &input.value)
                .map_err(|_| error("WFT-STRING-DOMAIN", "String literal exceeds selected domain"))?;
        }
        (LiteralKind::Boolean, ScalarFamily::Boolean) => {}
        (LiteralKind::Number, ScalarFamily::Integer) => {
            if input.value.contains('.') {
                return Err(error(
                    "WFT-NUMERIC-DOMAIN",
                    "Integer literal must be integral base-ten text",
                ));
            }
            let digits = input.value.strip_prefix('-').unwrap_or(&input.value);
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) { return Err(error("WFT-NUMERIC-DOMAIN", "Integer literal must retain exact base-ten digits")); }
            if expected.facets != serde_json::json!({}) {
            let n = input.value.parse::<i128>().map_err(|_| {
                error(
                    "WFT-NUMERIC-DOMAIN",
                    "Integer literal exceeds selected domain",
                )
            })?;
            let width = &expected.facets["integerWidth"];
            let bits = width["bits"].as_u64().unwrap();
            let (min, max) = if width["signed"] == true {
                (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
            } else {
                (0, (1i128 << bits) - 1)
            };
            if n < min || n > max {
                return Err(error(
                    "WFT-NUMERIC-DOMAIN",
                    "Integer literal exceeds selected domain",
                ));
            }
            }
        }
        (LiteralKind::Number, ScalarFamily::Decimal) => {
            if expected.facets == serde_json::json!({}) {
                validate_mathematical_decimal(&input.value).map_err(|_|error("WFT-NUMERIC-DOMAIN","Decimal literal must be exact finite base-ten text"))?;
                return Ok(());
            }
            let scale = expected.facets["scale"].as_u64().unwrap() as usize;
            let precision = expected.facets["precision"].as_u64().unwrap() as usize;
            let unsigned = input.value.strip_prefix('-').unwrap_or(&input.value);
            let mut parts = unsigned.split('.');
            let whole = parts.next().unwrap();
            let frac = parts.next().unwrap_or("").trim_end_matches('0');
            if frac.len() > scale {
                return Err(error(
                    "WFT-NUMERIC-DOMAIN",
                    "Decimal scale exceeds selected domain",
                ));
            }
            let coefficient = format!("{whole}{frac}{}", "0".repeat(scale - frac.len()));
            if coefficient.trim_start_matches('0').len() > precision {
                return Err(error(
                    "WFT-NUMERIC-DOMAIN",
                    "Decimal precision exceeds selected domain",
                ));
            }
        }
        (LiteralKind::String, ScalarFamily::Timestamp) => {
            if expected.facets != serde_json::json!({}) {
                return Err(error("WFT-TYPE", "Timestamp facets are unsupported"));
            }
            crate::exact_instant::key(&input.value).map_err(|d| error(
                if d.code == "WFT-LIMIT" { "WFT-LIMIT" } else { "WFT-TIMESTAMP-DOMAIN" },
                "Timestamp literal must be an RFC 3339 exact instant",
            ))?;
        }
        (LiteralKind::Number, ScalarFamily::Float) => {
            if expected.facets != serde_json::json!({}) {
                return Err(error("WFT-TYPE", "Float facets are unsupported"));
            }
            validate_float_token(&input.value)
                .map_err(|_| error("WFT-NUMERIC-DOMAIN", "Float literal must be finite base-ten text"))?;
        }
        _ => return Err(error("WFT-TYPE", "Literal family does not match the field")),
    }
    Ok(())
}

/// UMF `floatToken` grammar (`-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`) whose
/// nearest IEEE 754 binary64 value is finite. The token itself stays the value.
pub fn validate_float_token(value: &str) -> Result<()> {
    let fail = || Diagnostic::new("WFT-NUMERIC-DOMAIN", "type", "Finite float token required");
    if value.len() > MAX_TOKEN { return Err(fail()); }
    let body = value.strip_prefix('-').unwrap_or(value);
    let (mantissa, exponent) = match body.find(['e', 'E']) {
        Some(at) => (&body[..at], Some(&body[at + 1..])),
        None => (body, None),
    };
    let (whole, fraction) = match mantissa.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (mantissa, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(whole) || (whole.len() > 1 && whole.starts_with('0'))
        || fraction.is_some_and(|f| !digits(f))
        || exponent.is_some_and(|e| !digits(e.strip_prefix(['+', '-']).unwrap_or(e)))
    {
        return Err(fail());
    }
    match value.parse::<f64>() {
        Ok(v) if v.is_finite() => Ok(()),
        _ => Err(fail()),
    }
}

/// Exact decimal arithmetic: the integer coefficient `c` with `token == c * 10^-scale`.
///
/// Accepts UMF number tokens with an optional exponent. Returns `None` when the
/// token is malformed, needs more than `scale` fractional digits, or has more than
/// `precision` coefficient digits. Callers attach their own diagnostic.
pub fn coefficient(token: &str, scale: u64, precision: Option<u64>) -> Option<BigInt> {
    if token.len() > MAX_TOKEN { return None; }
    let (negative, body) = if let Some(v) = token.strip_prefix('-') { (true, v) } else { (false, token) };
    let at = body.find(['e', 'E']);
    let (mantissa, exponent) = if let Some(at) = at { (&body[..at], &body[at + 1..]) } else { (body, "0") };
    let exponent_digits = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
    if exponent_digits.is_empty() || !exponent_digits.bytes().all(|b| b.is_ascii_digit()) { return None; }
    let (whole, fraction) = if let Some((whole, fraction)) = mantissa.split_once('.') {
        if fraction.is_empty() || !fraction.bytes().all(|b| b.is_ascii_digit()) { return None; }
        (whole, fraction)
    } else { (mantissa, "") };
    if whole.is_empty() || !whole.bytes().all(|b| b.is_ascii_digit()) || (whole.len() > 1 && whole.starts_with('0')) { return None; }
    let joined = format!("{whole}{fraction}");
    let mut digits = joined.trim_start_matches('0').to_string();
    if digits.is_empty() { return Some(BigInt::from(0)); }
    if exponent.len() > 32 { return None; }
    let exponent: i128 = exponent.parse().ok()?;
    let shift = exponent - fraction.len() as i128 + scale as i128;
    if shift < 0 {
        let cut = -shift;
        if cut > digits.len() as i128 { return None; }
        let at = digits.len() - cut as usize;
        if !digits[at..].bytes().all(|b| b == b'0') { return None; }
        digits.truncate(at);
    } else {
        let bound = precision.unwrap_or(MAX_TOKEN as u64).min(MAX_TOKEN as u64);
        if digits.len() as i128 + shift > bound as i128 { return None; }
        digits.extend(std::iter::repeat_n('0', shift as usize));
    }
    if precision.is_some_and(|p| digits.len() as u64 > p) { return None; }
    if digits.is_empty() { return Some(BigInt::from(0)); }
    if negative { digits.insert(0, '-'); }
    BigInt::parse_bytes(digits.as_bytes(), 10)
}

/// Fixed-point finite token grammar; no conversion through native floats.
pub fn validate_mathematical_decimal(value:&str)->Result<()> {
    let unsigned=value.strip_prefix('-').unwrap_or(value);let mut parts=unsigned.split('.');
    let whole=parts.next().unwrap_or("");let fraction=parts.next();
    if value.len()>1048576 || whole.is_empty() || !whole.bytes().all(|b|b.is_ascii_digit())
      || fraction.is_some_and(|f|f.is_empty()||!f.bytes().all(|b|b.is_ascii_digit())) || parts.next().is_some(){
      return Err(Diagnostic::new("WFT-NUMERIC-DOMAIN","type","Finite fixed-point decimal token required"));
    }Ok(())
}

/// Byte values use bounded canonical standard base64; original tokens are retained.
pub fn validate_binary_token(value:&str)->Result<()> {
    use base64::Engine;
    if value.len()>1048576 {return Err(Diagnostic::new("WFT-LIMIT","type","Binary token exceeds one MiB"));}
    let bytes=base64::engine::general_purpose::STANDARD.decode(value).map_err(|_|Diagnostic::new("WFT-BINARY-DOMAIN","type","Canonical base64 required"))?;
    if base64::engine::general_purpose::STANDARD.encode(bytes)!=value {return Err(Diagnostic::new("WFT-BINARY-DOMAIN","type","Canonical base64 required"));}
    Ok(())
}

/// Explicit Truss canonical date subset: Gregorian YYYY-MM-DD, years 0001..9999.
/// UMF core declares the family; temporal refinements are not inferred here.
pub fn validate_date_token(value:&str)->Result<()> {
    let fail=||Diagnostic::new("WFT-DATE-DOMAIN","type","Canonical Gregorian YYYY-MM-DD required");
    if value.len()!=10 || !value.is_ascii() {return Err(fail());}
    crate::exact_instant::key(&format!("{value}T00:00:00Z")).map_err(|_|fail())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Family;
    use serde_json::json;
    fn lit(value: &str, kind: LiteralKind) -> Literal {
        Literal { value: value.into(), kind, span: Span { start: 0, end: 0 } }
    }
    #[test]
    fn timestamp_literals_use_the_exact_instant_profile() {
        let t = LogicalType::plain(ScalarFamily::Timestamp, false);
        for ok in ["2024-01-01T00:00:00Z", "2024-01-01T00:00:00.123Z", "2023-12-31T19:00:00.1234567890-05:00"] {
            check_literal(&lit(ok, LiteralKind::String), &t).unwrap();
        }
        for bad in ["2024-01-01", "2024-01-01T00:00:00", "1900-02-29T00:00:00Z", "2016-12-31T23:59:60Z"] {
            assert_eq!(check_literal(&lit(bad, LiteralKind::String), &t).unwrap_err().code, "WFT-TIMESTAMP-DOMAIN", "{bad}");
        }
        assert_eq!(check_literal(&lit("1", LiteralKind::Number), &t).unwrap_err().code, "WFT-TYPE");
        let faceted = LogicalType { facets: json!({"precision": 3}), ..t };
        assert_eq!(check_literal(&lit("2024-01-01T00:00:00Z", LiteralKind::String), &faceted).unwrap_err().code, "WFT-TYPE");
    }
    #[test]
    fn float_literals_are_finite_umf_float_tokens() {
        let f = LogicalType::plain(ScalarFamily::Float, true);
        for ok in ["0", "-0", "1.5", "1e308", "-2.5E-3", "123456789012345678901234567890"] {
            check_literal(&lit(ok, LiteralKind::Number), &f).unwrap();
        }
        for bad in ["1e309", "01", ".5", "1.", "NaN", "inf", "1e", "+1", " 1"] {
            assert_eq!(check_literal(&lit(bad, LiteralKind::Number), &f).unwrap_err().code, "WFT-NUMERIC-DOMAIN", "{bad}");
        }
        assert_eq!(check_literal(&lit("1", LiteralKind::String), &f).unwrap_err().code, "WFT-TYPE");
    }
    #[test]
    fn core_families_refuse_opt_in_tokens_unchanged() {
        let decimal: LogicalType = LogicalType { family: Family::Decimal, facets: json!({"precision": 4, "scale": 2}), nullable: false };
        assert_eq!(check_literal(&lit("1e2", LiteralKind::Number), &decimal).unwrap_err().code, "WFT-NUMERIC-DOMAIN");
        let string = LogicalType::plain(Family::String, false);
        // The same tokens refuse the core families that cannot carry them.
        let integer = LogicalType::plain(Family::Integer, false);
        assert_eq!(check_literal(&lit("1.5", LiteralKind::Number), &integer).unwrap_err().code, "WFT-NUMERIC-DOMAIN");
        check_literal(&lit("2024-01-01T00:00:00Z", LiteralKind::String), &string).unwrap();
    }
    #[test]
    fn coefficients_are_exact() {
        assert_eq!(coefficient("1.25", 2, Some(3)), Some(BigInt::from(125)));
        assert_eq!(coefficient("-1.5e1", 0, None), Some(BigInt::from(-15)));
        assert_eq!(coefficient("1.255", 2, None), None);
        assert_eq!(coefficient("1000", 0, Some(3)), None);
        assert_eq!(coefficient("0.000", 0, None), Some(BigInt::from(0)));
    }
}
