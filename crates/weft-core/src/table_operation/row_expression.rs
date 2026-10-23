// SPDX-License-Identifier: MIT OR Apache-2.0
//! Closed G02 same-row expressions. This module performs admission only; SQL
//! rendering remains in the backend.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
    Null,
    Text,
    Numeric {
        integral: u32,
        fractional: u32,
        integral_only: bool,
    },
    Exact(String),
}

struct State<'a> {
    operation: &'a AdmittedOperation,
    target: String,
    nodes: usize,
    numeric_inputs: BTreeSet<String>,
}

fn merge(a: &Type, b: &Type) -> Result<Type> {
    match (a, b) {
        (Type::Null, other) | (other, Type::Null) => Ok(other.clone()),
        (Type::Text, Type::Text) => Ok(Type::Text),
        (
            Type::Numeric {
                integral: i1,
                fractional: s1,
                integral_only: x1,
            },
            Type::Numeric {
                integral: i2,
                fractional: s2,
                integral_only: x2,
            },
        ) => Ok(Type::Numeric {
            integral: (*i1).max(*i2),
            fractional: (*s1).max(*s2),
            integral_only: *x1 && *x2,
        }),
        (Type::Exact(a), Type::Exact(b)) if a == b => Ok(Type::Exact(a.clone())),
        _ => Err("row expression carrier types differ"),
    }
}

fn predicate_types_compatible(op: &str, field: &Type, literal: &Type) -> bool {
    if literal == &Type::Null {
        return match op {
            "startsWith" => matches!(field, Type::Text),
            "lt" | "le" | "gt" | "ge" => {
                matches!(field, Type::Text | Type::Numeric { .. })
                    || matches!(field,Type::Exact(t) if t=="date"||t=="timestamptz")
            }
            _ => true,
        };
    }
    let compatible = field == literal
        || matches!(
            (field, literal),
            (Type::Numeric { .. }, Type::Numeric { .. })
        );
    if !compatible {
        return false;
    }
    match op {
        "startsWith" => matches!(field, Type::Text),
        "lt" | "le" | "gt" | "ge" => {
            matches!(field, Type::Text | Type::Numeric { .. })
                || matches!(field,Type::Exact(t) if t=="date"||t=="timestamptz")
        }
        _ => true,
    }
}

fn field_type(field: &OperationField) -> Result<Type> {
    let native = field.native_type["kind"]
        .as_str()
        .ok_or("row expression native type absent")?;
    Ok(match field.carrier {
        Carrier::String if matches!(native, "text" | "varchar") => Type::Text,
        Carrier::Integer => {
            let digits = match native {
                "int2" | "smallint" => 5,
                "int4" | "integer" => 10,
                "int8" | "bigint" => 19,
                "numeric" => {
                    let p = crate::security_literals::unsigned(&field.native_type["precision"])
                        .map_err(|_| "row expression integer precision absent")?;
                    let s = crate::security_literals::unsigned(&field.native_type["scale"])
                        .map_err(|_| "row expression integer scale absent")?;
                    if p == 0 || p > 1000 || s != 0 {
                        return Err("row expression integer native numeric type unsupported");
                    }
                    p as u32
                }
                _ => return Err("row expression integer native type unsupported"),
            };
            Type::Numeric {
                integral: digits,
                fractional: 0,
                integral_only: true,
            }
        }
        Carrier::Decimal if native == "numeric" => {
            let p0 = crate::security_literals::unsigned(&field.native_type["precision"])
                .map_err(|_| "row expression decimal precision absent")?;
            let s0 = crate::security_literals::unsigned(&field.native_type["scale"])
                .map_err(|_| "row expression decimal scale absent")?;
            if p0 == 0 || p0 > 1000 || s0 > p0 {
                return Err("row expression decimal precision or scale unsupported");
            }
            let (p, s) = (p0 as u32, s0 as u32);
            Type::Numeric {
                integral: p - s,
                fractional: s,
                integral_only: s == 0,
            }
        }
        Carrier::Boolean if matches!(native, "bool" | "boolean") => Type::Exact("boolean".into()),
        Carrier::Uuid if native == "uuid" => Type::Exact("uuid".into()),
        Carrier::Date if native == "date" => Type::Exact("date".into()),
        Carrier::Timestamp if native == "timestamptz" => Type::Exact("timestamptz".into()),
        _ => return Err("row expression carrier/native type unsupported"),
    })
}

fn literal_type(value: &Value, target: &OperationField) -> Result<Type> {
    if value.is_null() {
        return Ok(Type::Null);
    }
    if literal_payload_bytes(value).is_none_or(|length| length > 1_048_576) {
        return Err("row expression literal capacity exceeded");
    }
    if value.as_object().is_none_or(|m| m.len() != 1) {
        return Err("row expression literal wrapper invalid");
    }
    Ok(
        match value.as_object().unwrap().keys().next().unwrap().as_str() {
            "string" if value["string"].is_string() => {
                if value["string"].as_str().unwrap().contains('\0') {
                    return Err("row expression string literal contains NUL");
                }
                Type::Text
            }
            "boolean" if value["boolean"].is_boolean() => Type::Exact("boolean".into()),
            "integerToken" => {
                let text = value["integerToken"]
                    .as_str()
                    .ok_or("row expression integer literal invalid")?;
                // CoreLiteral integerToken deliberately preserves the authored
                // number token. Admit any spelling whose exact value is an
                // integer (including 1.0 and 1e2), with a hard coefficient
                // budget before constructing arbitrary-precision output.
                let coefficient = crate::security_literals::coefficient(text, 0, Some(1000))
                    .map_err(|_| "row expression integer token is not a bounded integer")?;
                let coefficient_text = coefficient.to_string();
                let digits = if coefficient_text == "0" {
                    1
                } else {
                    coefficient_text.trim_start_matches('-').len() as u32
                };
                Type::Numeric {
                    integral: digits.max(1),
                    fractional: 0,
                    integral_only: true,
                }
            }
            "decimalToken" => {
                let token = value["decimalToken"]
                    .as_str()
                    .ok_or("row expression decimal token invalid")?;
                let (i, s) = decimal_budget(token)?;
                Type::Numeric {
                    integral: i,
                    fractional: s,
                    integral_only: s == 0,
                }
            }
            "uuid" => {
                let text = value["uuid"]
                    .as_str()
                    .ok_or("row expression UUID literal invalid")?;
                if text.len() != 36
                    || !text.bytes().enumerate().all(|(i, b)| {
                        if [8, 13, 18, 23].contains(&i) {
                            b == b'-'
                        } else {
                            b.is_ascii_hexdigit()
                        }
                    })
                {
                    return Err("row expression UUID literal invalid");
                }
                Type::Exact("uuid".into())
            }
            "date" => {
                let text = value["date"]
                    .as_str()
                    .ok_or("row expression date literal invalid")?;
                super::temporal::date(text)?;
                Type::Exact("date".into())
            }
            "timestamp" => {
                let text = value["timestamp"]
                    .as_str()
                    .ok_or("row expression timestamp literal invalid")?;
                super::temporal::instant(text)?;
                Type::Exact("timestamptz".into())
            }
            _ => {
                let _ = target;
                return Err("row expression literal carrier unsupported");
            }
        },
    )
}

// The source bound applies to the literal payload itself, not its JSON wrapper
// or the extra escaping introduced by serializing an already parsed value.
fn literal_payload_bytes(value: &Value) -> Option<usize> {
    let object = value.as_object()?;
    if object.len() != 1 {
        return None;
    }
    let (tag, payload) = object.iter().next()?;
    match tag.as_str() {
        "boolean" => payload.as_bool().map(|_| 0),
        "string" | "integerToken" | "decimalToken" | "uuid" | "date" | "timestamp" => {
            payload.as_str().map(str::len)
        }
        _ => None,
    }
}

fn decimal_budget(token: &str) -> Result<(u32, u32)> {
    let (integral, fractional, _) = effective_numeric(token)?;
    Ok((integral, fractional))
}

fn canonical_numeric_token(token: &str) -> Result<String> {
    let (integral, fractional, coefficient) = effective_numeric(token)?;
    if integral + fractional > 1000 {
        return Err("row expression numeric budget exceeded");
    }
    let mut digits = coefficient.to_string();
    let negative = digits.starts_with('-');
    if negative {
        digits.remove(0);
    }
    if fractional == 0 {
        return Ok(if negative {
            format!("-{digits}")
        } else {
            digits
        });
    }
    let scale = fractional as usize;
    let (integer, fraction) = if digits.len() <= scale {
        (
            "0".to_owned(),
            format!("{}{}", "0".repeat(scale - digits.len()), digits),
        )
    } else {
        let split = digits.len() - scale;
        (digits[..split].to_owned(), digits[split..].to_owned())
    };
    let sign = if negative { "-" } else { "" };
    Ok(format!("{sign}{integer}.{fraction}"))
}

fn effective_numeric(token: &str) -> Result<(u32, u32, num_bigint::BigInt)> {
    if token.is_empty() || token.len() > 1_048_576 {
        return Err("row expression decimal token unsupported");
    }
    let unsigned = token.strip_prefix('-').unwrap_or(token);
    let mut exponent_parts = unsigned.split(['e', 'E']);
    let mantissa = exponent_parts.next().unwrap_or("");
    let exponent_token = exponent_parts.next();
    if exponent_parts.next().is_some() {
        return Err("row expression exponent invalid");
    }
    if let Some(exponent) = exponent_token {
        let digits = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("row expression exponent invalid");
        }
    }
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if whole.is_empty()
        || mantissa.contains('.') && fraction.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        || whole.len() > 1 && whole.starts_with('0')
    {
        return Err("row expression decimal token invalid");
    }
    let mut digits = format!("{whole}{fraction}");
    let first_nonzero = digits.find(|byte: char| byte != '0');
    let Some(first_nonzero) = first_nonzero else {
        let coefficient = crate::security_literals::coefficient(token, 0, Some(1000))
            .map_err(|_| "row expression numeric literal invalid")?;
        return Ok((1, 0, coefficient));
    };
    digits.drain(..first_nonzero);
    if exponent_token.is_some_and(|exponent| exponent.len() > 32) {
        return Err("row expression exponent invalid");
    }
    let exponent = exponent_token
        .map(|value| {
            value
                .parse::<i128>()
                .map_err(|_| "row expression exponent invalid")
        })
        .transpose()?
        .unwrap_or(0);
    let mut shift = exponent
        .checked_sub(fraction.len() as i128)
        .ok_or("row expression exponent outside bounds")?;
    let significant_len = digits.trim_end_matches('0').len();
    let trailing_zeros = digits.len() - significant_len;
    digits.truncate(significant_len);
    shift = shift
        .checked_add(trailing_zeros as i128)
        .ok_or("row expression exponent outside bounds")?;
    let (integral, fractional) = if shift < 0 {
        let scale = shift
            .checked_neg()
            .ok_or("row expression exponent outside bounds")?;
        ((digits.len() as i128 + shift).max(0), scale)
    } else {
        ((digits.len() as i128 + shift).max(1), 0)
    };
    if integral + fractional > 1000 {
        return Err("row expression numeric budget exceeded");
    }
    let integral = u32::try_from(integral).map_err(|_| "row expression numeric budget exceeded")?;
    let fractional =
        u32::try_from(fractional).map_err(|_| "row expression numeric budget exceeded")?;
    let coefficient = crate::security_literals::coefficient(token, fractional as u64, Some(1000))
        .map_err(|_| "row expression numeric literal outside effective bounds")?;
    Ok((integral, fractional, coefficient))
}

impl State<'_> {
    fn predicate(&mut self, node: &Value, depth: usize) -> Result<()> {
        self.nodes += 1;
        if depth > 16 || self.nodes > 256 {
            return Err("row expression resource limit exceeded");
        }
        match node["op"]
            .as_str()
            .ok_or("row expression condition operator absent")?
        {
            "all" => Ok(()),
            "and" | "or" => {
                let args = node["args"]
                    .as_array()
                    .ok_or("row expression condition args absent")?;
                if args.len() > 32 {
                    return Err("row expression condition fanout exceeded");
                }
                for child in args {
                    self.predicate(child, depth + 1)?;
                }
                Ok(())
            }
            "not" => self.predicate(&node["arg"], depth + 1),
            "isNull" => {
                let f = self.operation.field(&node["field"])?;
                self.reject_target(&f.reference)?;
                self.note_numeric(f);
                Ok(())
            }
            op @ ("eq" | "ne" | "lt" | "le" | "gt" | "ge" | "startsWith") => {
                let f = self.operation.field(&node["field"])?;
                self.reject_target(&f.reference)?;
                self.note_numeric(f);
                if op == "startsWith" && f.carrier != Carrier::String {
                    return Err("row expression startsWith requires text");
                }
                let value = &node["value"];
                if value["kind"] != "literal" {
                    return Err("row expression condition requires literal operand");
                }
                if depth + 1 > 16 {
                    return Err("row expression resource limit exceeded");
                }
                self.nodes += 1;
                if self.nodes > 256 {
                    return Err("row expression resource limit exceeded");
                }
                let ty = literal_type(&value["value"], f)?;
                let ft = field_type(f)?;
                if !predicate_types_compatible(op, &ft, &ty) {
                    return Err("row expression condition operand type differs");
                }
                Ok(())
            }
            "in" => {
                let f = self.operation.field(&node["field"])?;
                self.reject_target(&f.reference)?;
                self.note_numeric(f);
                let values = node["values"]
                    .as_array()
                    .ok_or("row expression condition values absent")?;
                if values.is_empty() || values.len() > 32 {
                    return Err("row expression condition fanout exceeded");
                }
                let ft = field_type(f)?;
                for value in values {
                    if depth + 1 > 16 {
                        return Err("row expression resource limit exceeded");
                    }
                    self.nodes += 1;
                    if self.nodes > 256 {
                        return Err("row expression resource limit exceeded");
                    }
                    if value["kind"] != "literal" {
                        return Err("row expression condition requires literal operand");
                    }
                    let ty = literal_type(&value["value"], f)?;
                    if !predicate_types_compatible("eq", &ft, &ty) {
                        return Err("row expression condition operand type differs");
                    }
                }
                Ok(())
            }
            _ => Err("row expression condition operator unsupported"),
        }
    }
    fn note_numeric(&mut self, f: &OperationField) {
        if matches!(f.carrier, Carrier::Integer | Carrier::Decimal) {
            self.numeric_inputs
                .insert(crate::mutation::canonical(&f.reference));
        }
    }
    fn reject_target(&self, reference: &Value) -> Result<()> {
        if crate::mutation::canonical(reference) == self.target {
            Err("row expression cannot reference assigned target")
        } else {
            Ok(())
        }
    }
    fn expression(&mut self, node: &Value, depth: usize) -> Result<Type> {
        self.nodes += 1;
        if depth > 16 || self.nodes > 256 {
            return Err("row expression resource limit exceeded");
        }
        let op = node["op"]
            .as_str()
            .ok_or("row expression operator absent")?;
        let ty = match op {
            "field" => {
                let f = self.operation.field(&node["field"])?;
                self.reject_target(&f.reference)?;
                self.note_numeric(f);
                field_type(f)?
            }
            "literal" => {
                let target = self
                    .operation
                    .field(&self.operation.declaration()["body"]["values"][0]["field"])?;
                literal_type(&node["value"], target)?
            }
            "coalesce" | "concat" => {
                let args = node["args"]
                    .as_array()
                    .ok_or("row expression args absent")?;
                if !(2..=32).contains(&args.len()) {
                    return Err("row expression fanout outside bounds");
                }
                let mut it = args.iter();
                let first = self.expression(it.next().unwrap(), depth + 1)?;
                let mut acc = first;
                for child in it {
                    let next = self.expression(child, depth + 1)?;
                    acc = merge(&acc, &next)?;
                }
                if op == "concat" {
                    if !matches!(acc, Type::Text | Type::Null) {
                        return Err("row expression concat requires text");
                    }
                    if acc == Type::Null {
                        Type::Text
                    } else {
                        acc
                    }
                } else {
                    acc
                }
            }
            "add" | "subtract" | "multiply" => {
                let left = self.expression(&node["left"], depth + 1)?;
                let right = self.expression(&node["right"], depth + 1)?;
                let left_null = left == Type::Null;
                let right_null = right == Type::Null;
                if left_null && right_null {
                    return Ok(Type::Numeric {
                        integral: 0,
                        fractional: 0,
                        integral_only: true,
                    });
                }
                if left_null || right_null {
                    let other = if left_null { &right } else { &left };
                    if !matches!(other, Type::Numeric { .. }) {
                        return Err("row expression arithmetic requires numeric operands");
                    }
                    return Ok(other.clone());
                }
                let (i1, s1, x1) = match left {
                    Type::Numeric {
                        integral,
                        fractional,
                        integral_only,
                    } => (integral, fractional, integral_only),
                    _ => return Err("row expression arithmetic requires numeric operands"),
                };
                let (i2, s2, x2) = match right {
                    Type::Numeric {
                        integral,
                        fractional,
                        integral_only,
                    } => (integral, fractional, integral_only),
                    _ => return Err("row expression arithmetic requires numeric operands"),
                };
                {
                    let (i, s) = match op {
                        "add" | "subtract" => (i1.max(i2).saturating_add(1), s1.max(s2)),
                        _ => (i1.saturating_add(i2), s1.saturating_add(s2)),
                    };
                    if i + s > 1000 {
                        return Err("row expression numeric budget exceeded");
                    }
                    Type::Numeric {
                        integral: i,
                        fractional: s,
                        integral_only: x1 && x2 && s == 0,
                    }
                }
            }
            "conditional" => {
                self.predicate(&node["when"], depth + 1)?;
                merge(
                    &self.expression(&node["then"], depth + 1)?,
                    &self.expression(&node["else"], depth + 1)?,
                )?
            }
            _ => return Err("row expression operator unsupported"),
        };
        if let Type::Numeric {
            integral,
            fractional,
            ..
        } = ty
        {
            if integral + fractional > 1000 {
                return Err("row expression numeric budget exceeded");
            }
        }
        Ok(ty)
    }
}

pub(super) fn validate(admitted: &AdmittedOperation) -> Result<BTreeSet<String>> {
    let body = &admitted.declaration()["body"];
    if body["kind"] != "update"
        || body["values"].as_array().is_none_or(|a| a.len() != 1)
        || !body["returning"].as_array().is_some_and(|a| a.is_empty())
        || body.get("batch").is_some()
    {
        return Err("row expression operation shape unsupported");
    }
    let target = &body["values"][0]["field"];
    if body["where"] != json!({"op":"isNull","field":target}) {
        return Err("row expression UPDATE selector differs");
    }
    let target_id = crate::mutation::canonical(target);
    let field = admitted.field(target)?;
    let mut state = State {
        operation: admitted,
        target: target_id,
        nodes: 0,
        numeric_inputs: BTreeSet::new(),
    };
    let result = state.expression(&body["values"][0]["value"]["expression"], 0)?;
    let expected = field_type(field)?;
    match (&expected, &result) {
        (_, Type::Null) => {}
        (Type::Text, Type::Text) => {}
        (
            Type::Numeric {
                integral_only: true,
                ..
            },
            Type::Numeric {
                integral_only: false,
                ..
            },
        ) => return Err("row expression integer target requires statically integral result"),
        (Type::Numeric { .. }, Type::Numeric { .. }) => {}
        (Type::Exact(a), Type::Exact(b)) if a == b => {}
        _ => return Err("row expression result type differs from assigned target"),
    }
    Ok(state.numeric_inputs)
}

pub fn native_literal(value: &Value) -> Result<Value> {
    if value.is_null() {
        return Ok(Value::Null);
    }
    if value.as_object().is_none_or(|m| m.len() != 1) {
        return Err("row expression literal wrapper invalid");
    }
    Ok(
        match value.as_object().unwrap().keys().next().unwrap().as_str() {
            "string" => {
                let s = value["string"]
                    .as_str()
                    .ok_or("row expression string literal invalid")?;
                if s.contains('\0') {
                    return Err("row expression string literal contains NUL");
                }
                json!(s)
            }
            "boolean" => json!(value["boolean"]
                .as_bool()
                .ok_or("row expression boolean literal invalid")?),
            "integerToken" | "decimalToken" => {
                let token = value
                    .as_object()
                    .unwrap()
                    .values()
                    .next()
                    .unwrap()
                    .as_str()
                    .ok_or("row expression numeric literal invalid")?;
                json!(canonical_numeric_token(token)?)
            }
            "uuid" => json!(value["uuid"]
                .as_str()
                .ok_or("row expression UUID literal invalid")?
                .to_ascii_lowercase()),
            "date" => json!(super::temporal::date(
                value["date"]
                    .as_str()
                    .ok_or("row expression date literal invalid")?
            )?),
            "timestamp" => json!(super::temporal::instant(
                value["timestamp"]
                    .as_str()
                    .ok_or("row expression timestamp literal invalid")?
            )?),
            _ => return Err("row expression literal carrier unsupported"),
        },
    )
}

pub fn numeric_input_fields(operation: &AdmittedOperation) -> Result<Vec<Value>> {
    fn visit(
        operation: &AdmittedOperation,
        node: &Value,
        out: &mut BTreeMap<String, Value>,
    ) -> Result<()> {
        if node.is_object() {
            if node["op"] == "field"
                || node["op"] == "isNull"
                || node.get("field").is_some() && node.get("op").is_some()
            {
                let reference = &node["field"];
                if let Ok(field) = operation.field(reference) {
                    if matches!(field.carrier, Carrier::Integer | Carrier::Decimal) {
                        out.insert(crate::mutation::canonical(reference), reference.clone());
                    }
                }
            }
            if let Some(object) = node.as_object() {
                for child in object.values() {
                    visit(operation, child, out)?;
                }
            }
        } else if let Some(items) = node.as_array() {
            for child in items {
                visit(operation, child, out)?;
            }
        }
        Ok(())
    }
    let mut found = BTreeMap::new();
    visit(
        operation,
        &operation.declaration()["body"]["values"][0]["value"]["expression"],
        &mut found,
    )?;
    Ok(found.into_values().collect())
}

#[cfg(test)]
mod literal_budget_tests {
    use super::{
        canonical_numeric_token, decimal_budget, literal_payload_bytes, predicate_types_compatible,
        Type,
    };
    use serde_json::json;

    #[test]
    fn literal_budget_counts_decoded_payload_bytes_without_json_escape_inflation() {
        let escaped = "\\".repeat(524_288);
        let literal = json!({"string": escaped});
        assert!(literal.to_string().len() > 1_048_576);
        assert_eq!(literal_payload_bytes(&literal), Some(524_288));

        let supplementary = json!({"string": "🧪".repeat(262_144)});
        assert_eq!(literal_payload_bytes(&supplementary), Some(1_048_576));
        let too_large = json!({"string": "🧪".repeat(262_145)});
        assert_eq!(literal_payload_bytes(&too_large), Some(1_048_580));
        assert_eq!(
            literal_payload_bytes(&json!({"decimalToken": "1e2"})),
            Some(3)
        );
    }

    #[test]
    fn effective_decimal_budget_handles_exponent_cancellation_without_expansion() {
        let integral = format!("0.{}99999e2005", "0".repeat(2000));
        assert_eq!(decimal_budget(&integral).unwrap(), (5, 0));
        assert_eq!(canonical_numeric_token(&integral).unwrap(), "99999");

        let fractional = format!("0.{}99999e2004", "0".repeat(2000));
        assert_eq!(decimal_budget(&fractional).unwrap(), (4, 1));
        assert_eq!(canonical_numeric_token(&fractional).unwrap(), "9999.9");

        assert_eq!(decimal_budget("1e-1000").unwrap(), (0, 1000));
        assert_eq!(
            canonical_numeric_token("1e-1000").unwrap(),
            format!("0.{}1", "0".repeat(999))
        );

        let long_zero = "0e9999999999999999999999999999";
        assert_eq!(decimal_budget(long_zero).unwrap(), (1, 0));
        assert_eq!(canonical_numeric_token(long_zero).unwrap(), "0");
        let zero_with_unbounded_exponent = format!("0e{}", "9".repeat(64));
        assert_eq!(
            decimal_budget(&zero_with_unbounded_exponent).unwrap(),
            (1, 0)
        );
        assert_eq!(
            canonical_numeric_token(&zero_with_unbounded_exponent).unwrap(),
            "0"
        );
        assert!(decimal_budget("1e1001").is_err());
    }

    #[test]
    fn null_comparison_support_tracks_operator_type_support() {
        let null = Type::Null;
        assert!(predicate_types_compatible(
            "eq",
            &Type::Exact("uuid".into()),
            &null
        ));
        assert!(predicate_types_compatible(
            "ne",
            &Type::Exact("boolean".into()),
            &null
        ));
        assert!(!predicate_types_compatible(
            "lt",
            &Type::Exact("boolean".into()),
            &null
        ));
        assert!(!predicate_types_compatible(
            "startsWith",
            &Type::Exact("uuid".into()),
            &null
        ));
    }
}
