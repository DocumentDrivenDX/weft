// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
pub(super) fn domain(field: &OperationField) -> String {
    crate::mutation::canonical(&json!([
        field.declaration["scalarType"],
        field.declaration["cardinality"],
        field.declaration["nullability"],
        crate::security_literals::normalized_facets(&field.declaration).ok(),
        field.declaration.get("allowedValues"),
        field.native_type
    ]))
}
pub(super) fn literal(field: &OperationField, value: &Value) -> Result<()> {
    if value.to_string().len() > 1_048_576 {
        return Err("operation value capacity exceeded");
    }
    if matches!(field.carrier, Carrier::Date | Carrier::Timestamp) {
        if value.is_null() { return if field.nullable { Ok(()) } else { Err("required operation value is null") }; }
        let key = if field.carrier == Carrier::Date { "date" } else { "timestamp" };
        if value.as_object().is_none_or(|m| m.len() != 1) { return Err("exact temporal wrapper required"); }
        let text = value[key].as_str().ok_or("temporal token absent")?;
        if field.carrier == Carrier::Date { temporal::date(text)?; } else { temporal::instant(text)?; }
        return Ok(());
    }
    if matches!(field.carrier, Carrier::Uuid | Carrier::Bits128) {
        if value.is_null() {
            return if field.nullable {
                Ok(())
            } else {
                Err("required operation value is null")
            };
        }
        if value.as_object().is_none_or(|m| m.len() != 1) {
            return Err("exact 128-bit wrapper required");
        }
        let key = if field.carrier == Carrier::Uuid {
            "uuid"
        } else {
            "bits128Hex"
        };
        let token = value[key].as_str().ok_or("128-bit value absent")?;
        let bytes = token.as_bytes();
        let valid = if key == "uuid" {
            bytes.len() == 36
                && bytes.iter().enumerate().all(|(i, b)| {
                    if [8, 13, 18, 23].contains(&i) {
                        *b == b'-'
                    } else {
                        b.is_ascii_hexdigit()
                    }
                })
        } else {
            bytes.len() == 32 && bytes.iter().all(u8::is_ascii_hexdigit)
        };
        return if valid {
            Ok(())
        } else {
            Err("128-bit lexical value differs")
        };
    }
    if let Some(prepared)=&field.prepared_literal {
        prepared.check(value).map_err(|_| "operation literal violates original Field")?;
    } else {
        crate::security_literals::check_literal(&field.declaration, value)
            .map_err(|_| "operation literal violates original Field")?;
    }
    if value.is_null() {
        return Ok(());
    }
    let native = &field.native_type;
    match native["kind"].as_str() {
        Some("text" | "varchar") => {
            let text = value["string"]
                .as_str()
                .ok_or("operation text wrapper differs")?;
            if text.contains('\0')
                || native["kind"] == "varchar"
                    && text.chars().count() as u64
                        > crate::security_literals::unsigned(&native["length"])
                            .map_err(|_| "native text length absent")?
            {
                return Err("operation text exceeds native capacity");
            }
        }
        Some("int2" | "int4" | "int8") => {
            let token = value["integerToken"]
                .as_str()
                .ok_or("operation integer wrapper differs")?;
            let n = crate::security_literals::coefficient(token, 0, None)
                .map_err(|_| "operation integer is not exact")?;
            let bits = if native["kind"] == "int2" {
                16
            } else if native["kind"] == "int4" {
                32
            } else {
                64
            };
            let bound = num_bigint::BigInt::from(1) << (bits - 1);
            if n < -&bound || n >= bound {
                return Err("operation integer exceeds native capacity");
            }
        }
        Some("numeric") => {
            crate::security_literals::coefficient(
                value["decimalToken"]
                    .as_str()
                    .ok_or("operation decimal wrapper differs")?,
                crate::security_literals::unsigned(&native["scale"])
                    .map_err(|_| "native scale absent")?,
                Some(
                    crate::security_literals::unsigned(&native["precision"])
                        .map_err(|_| "native precision absent")?,
                ),
            )
            .map_err(|_| "operation decimal would round or exceed native capacity")?;
        }
        Some("boolean") => {}
        _ => return Err("operation native value codec unsupported"),
    }
    Ok(())
}

/// Convert a value that has already passed source-aware call admission into a driver value.
/// Original CoreLiteral tokens remain in the original request/source for custody.
pub(super) fn native_value(field: &OperationField, value: &Value) -> Result<Value> {
    if value.is_null() {
        return Ok(Value::Null);
    }
    Ok(match field.carrier {
        Carrier::Date => json!(temporal::date(value["date"].as_str().unwrap())?),
        Carrier::Timestamp => json!(temporal::instant(value["timestamp"].as_str().unwrap())?),
        Carrier::LocalTimestamp => return Err("local timestamp value codec unsupported"),
        Carrier::String => value["string"].clone(),
        Carrier::Boolean => value["boolean"].clone(),
        Carrier::Uuid => json!(value["uuid"].as_str().unwrap().to_ascii_lowercase()),
        Carrier::Bits128 => json!(value["bits128Hex"].as_str().unwrap().to_ascii_lowercase()),
        Carrier::Integer => json!(crate::security_literals::coefficient(
            value["integerToken"].as_str().unwrap(),
            0,
            None
        )
        .map_err(|_| "native integer normalization failed")?
        .to_string()),
        Carrier::Decimal => {
            let scale = field.native_type["scale"]
                .as_u64()
                .ok_or("native decimal scale invalid")?;
            let coefficient = crate::security_literals::coefficient(
                value["decimalToken"].as_str().unwrap(),
                scale,
                field.native_type["precision"].as_u64(),
            )
            .map_err(|_| "native decimal normalization failed")?;
            let mut digits = coefficient.to_string();
            if digits == "0" {
                json!("0")
            } else {
                let zeros = digits.bytes().rev().take_while(|b| *b == b'0').count();
                digits.truncate(digits.len() - zeros);
                let exponent = zeros as i64 - scale as i64;
                json!(if exponent == 0 {
                    digits
                } else {
                    format!("{digits}e{exponent}")
                })
            }
        }
    })
}
