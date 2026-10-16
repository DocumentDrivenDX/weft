use crate::{
    error::{Diagnostic, Result},
    ir::{Expression, Family, LogicalType},
    syntax::{Literal, LiteralKind},
};
pub fn literal(input: &Literal, expected: &LogicalType) -> Result<Expression> {
    let error = |code, message| Diagnostic::new(code, "type", message).at(&input.span);
    match (&input.kind, &expected.family) {
        (LiteralKind::String, Family::String) => {
            crate::current_string_domain::validate_facets(&expected.facets, &input.value)
                .map_err(|_| error("WFT-STRING-DOMAIN", "String literal exceeds selected domain"))?;
        }
        (LiteralKind::Boolean, Family::Boolean) => {}
        (LiteralKind::Number, Family::Integer) => {
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
        (LiteralKind::Number, Family::Decimal) => {
            if expected.facets == serde_json::json!({}) {
                validate_mathematical_decimal(&input.value).map_err(|_|error("WFT-NUMERIC-DOMAIN","Decimal literal must be exact finite base-ten text"))?;
                return Ok(Expression::Literal{value:input.value.clone(),logical_type:expected.clone(),span:input.span.clone()});
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
        _ => return Err(error("WFT-TYPE", "Literal family does not match the field")),
    }
    Ok(Expression::Literal {
        value: input.value.clone(),
        logical_type: expected.clone(),
        span: input.span.clone(),
    })
}

/// Fixed-point finite token grammar; no conversion through native floats.
pub(crate) fn validate_mathematical_decimal(value:&str)->Result<()> {
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
