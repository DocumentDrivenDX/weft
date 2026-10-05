use crate::{
    error::{Diagnostic, Result},
    ir::{Expression, Family, LogicalType},
    syntax::{Literal, LiteralKind},
};
pub fn literal(input: &Literal, expected: &LogicalType) -> Result<Expression> {
    let error = |code, message| Diagnostic::new(code, "type", message).at(&input.span);
    match (&input.kind, &expected.family) {
        (LiteralKind::String, Family::String) | (LiteralKind::Boolean, Family::Boolean) => {}
        (LiteralKind::Number, Family::Integer) => {
            if input.value.contains('.') {
                return Err(error(
                    "WFT-NUMERIC-DOMAIN",
                    "Integer literal must be integral base-ten text",
                ));
            }
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
        (LiteralKind::Number, Family::Decimal) => {
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
