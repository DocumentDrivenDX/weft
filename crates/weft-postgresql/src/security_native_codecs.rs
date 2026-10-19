// SPDX-License-Identifier: MIT OR Apache-2.0
//! PostgreSQL18.6 UTF8/raw-cast value correspondence. Never authenticates a native session or source.
use crate::{
    security_graph_binding::GraphSource,
    security_graph_parameters::GraphPolicyParameterCodec,
    security_relational_parameters::{
        prepare_relational_graph_parameters, RelationalQueryParameterCodec,
    },
};
use serde_json::Value;
use weft_core::{
    application_ir::Value as QueryValue,
    error::{Diagnostic, Result},
    json::checked_json,
    security_backend::SecurityBackendContext,
};
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-NATIVE-CODEC",
        "capability",
        "PostgreSQL value codec correspondence refused",
    )
}
fn text(s: &str) -> Result<()> {
    if s.contains('\0') {
        Err(fail())
    } else {
        Ok(())
    }
}
fn numeric(token: &str) -> Result<()> {
    if token.is_empty() || token.len() > 4_000_000 {
        return Err(fail());
    }
    let body = token.strip_prefix('-').unwrap_or(token);
    let (mantissa, exp) = match body.find(['e', 'E']) {
        Some(i) => (&body[..i], &body[i + 1..]),
        None => (body, "0"),
    };
    let digits = exp.strip_prefix(['+', '-']).unwrap_or(exp);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(fail());
    }
    let mut magnitude = 0i64;
    for b in digits.bytes() {
        magnitude = magnitude
            .checked_mul(10)
            .and_then(|n| n.checked_add((b - b'0') as i64))
            .ok_or_else(fail)?;
        if magnitude > i32::MAX as i64 / 2 {
            return Err(fail());
        }
    }
    let exp = if exp.starts_with('-') {
        -magnitude
    } else {
        magnitude
    };
    let (whole, fraction) = match mantissa.split_once('.') {
        Some((w, f)) => {
            if f.is_empty() {
                return Err(fail());
            }
            (w, f)
        }
        None => (mantissa, ""),
    };
    if whole.is_empty()
        || (whole.len() > 1 && whole.starts_with('0'))
        || !whole
            .bytes()
            .chain(fraction.bytes())
            .all(|b| b.is_ascii_digit())
    {
        return Err(fail());
    }
    let first = whole
        .bytes()
        .chain(fraction.bytes())
        .position(|b| b != b'0');
    let integer_digits = first
        .map(|i| (whole.len() as i64 + exp - i as i64).max(0))
        .unwrap_or(0);
    let scale = (fraction.len() as i64 - exp).max(0);
    if integer_digits > 131_072 || scale > 16_383 {
        return Err(fail());
    }
    Ok(())
}
fn jsonb(v: &Value) -> Result<()> {
    let mut stack = vec![v];
    let mut work = 1_000_000usize;
    while let Some(v) = stack.pop() {
        if work == 0 {
            return Err(fail());
        }
        work -= 1;
        match v {
            Value::String(s) => text(s)?,
            Value::Number(n) => numeric(n.as_str())?,
            Value::Array(a) => stack.extend(a),
            Value::Object(o) => {
                for (k, v) in o {
                    text(k)?;
                    stack.push(v)
                }
            }
            _ => {}
        }
    }
    Ok(())
}
/// Correspondence receipt only; actual PostgreSQL18.6 session/settings, native
/// source/caller/authority, capture completeness, arithmetic and release remain required.
/// ```compile_fail
/// use weft_postgresql::security_native_codecs::CheckedPostgresqlCodecs;
/// fn forge(p:&mut CheckedPostgresqlCodecs){p.parameters=0;}
/// ```
pub struct CheckedPostgresqlCodecs {
    context_sha256: String,
    parameters: usize,
    operands: usize,
}
impl CheckedPostgresqlCodecs {
    pub fn context_sha256(&self) -> &str {
        &self.context_sha256
    }
    pub fn parameter_count(&self) -> usize {
        self.parameters
    }
    pub fn operand_count(&self) -> usize {
        self.operands
    }
}
/// Run before ANY SQL, including type anchors and operand captures. Subject is
/// deliberately unvalued here; its actual int8 locator must be issued by the guarded native host.
pub fn check_postgresql_parameter_codecs(
    ctx: &SecurityBackendContext<'_>,
    source: &GraphSource,
) -> Result<CheckedPostgresqlCodecs> {
    let bindings = prepare_relational_graph_parameters(ctx, source)?;
    for (p, value) in bindings
        .graph()
        .policy()
        .iter()
        .zip(bindings.policy_values())
    {
        match p.codec() {
            GraphPolicyParameterCodec::Boolean => {
                if !value.is_null() && !value.is_boolean() {
                    return Err(fail());
                }
            }
            GraphPolicyParameterCodec::Text => {
                if !value.is_null() {
                    text(value.as_str().ok_or_else(fail)?)?;
                }
            }
            GraphPolicyParameterCodec::CoreLiteralJson => jsonb(value)?,
        }
    }
    for p in bindings.query() {
        let value = match p.value() {
            QueryValue::Literal { value, .. } | QueryValue::Parameter { value, .. } => value,
            _ => return Err(fail()),
        };
        match p.codec() {
            RelationalQueryParameterCodec::Text => text(value)?,
            RelationalQueryParameterCodec::Boolean => {
                if !matches!(value.as_str(), "true" | "false") {
                    return Err(fail());
                }
            }
            RelationalQueryParameterCodec::Numeric => numeric(value)?,
        }
    }
    Ok(CheckedPostgresqlCodecs {
        context_sha256: ctx.execution_query_context_sha256(),
        parameters: bindings.graph().policy().len() + bindings.query().len(),
        operands: 0,
    })
}
/// Original domain/presence check followed by native representation checks.
/// Does not prove that the supplied rows came from the database or were selected correctly.
pub fn check_postgresql_operand_codecs(
    ctx: &SecurityBackendContext<'_>,
    source: &GraphSource,
    raw: &str,
) -> Result<CheckedPostgresqlCodecs> {
    let mut receipt = check_postgresql_parameter_codecs(ctx, source)?;
    weft_core::security_operand_values::check_operand_values(ctx, raw)?;
    let v = checked_json(raw).map_err(|_| fail())?;
    for scan in v["scans"].as_array().ok_or_else(fail)? {
        for row in scan["rows"].as_array().ok_or_else(fail)? {
            for operand in row["operands"].as_array().ok_or_else(fail)? {
                let cell = &operand["cell"];
                if let Some(value) = cell.get("value") {
                    if !value.is_null() {
                        let o = value.as_object().ok_or_else(fail)?;
                        if let Some(v) = o.get("string") {
                            text(v.as_str().ok_or_else(fail)?)?;
                        } else if let Some(v) =
                            o.get("integerToken").or_else(|| o.get("decimalToken"))
                        {
                            numeric(v.as_str().ok_or_else(fail)?)?;
                        } else if !o.contains_key("boolean") {
                            return Err(fail());
                        }
                    }
                }
                receipt.operands += 1;
            }
        }
    }
    Ok(receipt)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numeric_raw_cast_boundaries_match_postgresql_storage_format() {
        for value in [
            "1e131071",
            "1e-16383",
            "0e1000000",
            "-9223372036854775809",
            "0.00",
            "1.2300e3",
        ] {
            numeric(value).unwrap();
        }
        for value in [
            "1e131072",
            "1e-16384",
            "0e-16384",
            "0e1073741824",
            "NaN",
            "Infinity",
            "+1",
            "01",
            "1.",
            "1e",
        ] {
            assert!(numeric(value).is_err(), "{value}");
        }
    }
    #[test]
    fn utf8_text_and_jsonb_refuse_nul_without_sanitizing_value() {
        text("雪'").unwrap();
        assert!(text("x\0y").is_err());
        assert!(jsonb(&serde_json::json!({"string":"x\0y"})).is_err());
    }
}
