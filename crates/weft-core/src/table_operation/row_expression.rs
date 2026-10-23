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
        return true;
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
    if token.is_empty() || token.len() > 1_048_576 {
        return Err("row expression decimal token unsupported");
    }
    let unsigned = token
        .strip_prefix('-')
        .or_else(|| token.strip_prefix('+'))
        .unwrap_or(token);
    let mut exponent_parts = unsigned.split(['e', 'E']);
    let mantissa = exponent_parts.next().unwrap_or("");
    let exponent_token = exponent_parts.next();
    if exponent_parts.next().is_some() {
        return Err("row expression exponent outside bounds");
    }
    if let Some(exponent) = exponent_token {
        let digits = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("row expression exponent invalid");
        }
    }
    let mut p = mantissa.split('.');
    let whole = p.next().unwrap_or("");
    let frac = p.next().unwrap_or("");
    if p.next().is_some()
        || whole.is_empty() && frac.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !frac.bytes().all(|b| b.is_ascii_digit())
    {
        return Err("row expression decimal token invalid");
    }
    let mut digits = format!("{whole}{frac}");
    if digits.bytes().all(|byte| byte == b'0') {
        return Ok((1, 0));
    }
    let exponent = exponent_token
        .map(|s| {
            s.parse::<i32>()
                .map_err(|_| "row expression exponent invalid")
        })
        .transpose()?
        .unwrap_or(0);
    if exponent.unsigned_abs() > 2000 {
        return Err("row expression exponent outside bounds");
    }
    let scale = frac.len() as i32 - exponent;
    let first = digits.find(|c: char| c != '0');
    if let Some(first) = first {
        digits.drain(..first);
    } else {
        return Ok((1, 0));
    }
    let (mut integral, mut fractional) = if scale <= 0 {
        (((digits.len() as i32 - scale).max(1) as u32), 0)
    } else {
        (((digits.len() as i32 - scale).max(0) as u32), scale as u32)
    };
    if fractional > 0 {
        let fractional_start = digits.len().saturating_sub(fractional as usize);
        let trailing = digits[fractional_start..]
            .bytes()
            .rev()
            .take_while(|b| *b == b'0')
            .count() as u32;
        fractional -= trailing;
    }
    if integral == 0 && fractional == 0 {
        integral = 1;
    }
    Ok((integral, fractional))
}

fn canonical_numeric_token(token: &str) -> Result<String> {
    let unsigned = token.strip_prefix('-').unwrap_or(token);
    let mut parts = unsigned.split(['e', 'E']);
    let mantissa = parts
        .next()
        .ok_or("row expression numeric literal invalid")?;
    let exponent_token = parts.next();
    if parts.next().is_some() {
        return Err("row expression exponent outside bounds");
    }
    if let Some(exponent) = exponent_token {
        let exponent_digits = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        if exponent_digits.is_empty() || !exponent_digits.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err("row expression exponent invalid");
        }
    }
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let mut digits = format!("{whole}{fraction}");
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("row expression numeric literal invalid");
    }
    if digits.bytes().all(|byte| byte == b'0') {
        return Ok("0".into());
    }
    let exponent = exponent_token
        .map(|value| {
            value
                .parse::<i32>()
                .map_err(|_| "row expression exponent invalid")
        })
        .transpose()?
        .unwrap_or(0);
    if exponent.unsigned_abs() > 2000 {
        return Err("row expression exponent outside bounds");
    }
    let point = whole.len() as i32 + exponent;
    let (integer, fractional) = if point <= 0 {
        (
            "0".to_owned(),
            format!("{}{}", "0".repeat((-point) as usize), digits),
        )
    } else if point as usize >= digits.len() {
        digits.push_str(&"0".repeat(point as usize - digits.len()));
        (digits, String::new())
    } else {
        let remainder = digits.split_off(point as usize);
        (digits, remainder)
    };
    let integer = integer.trim_start_matches('0');
    let integer = if integer.is_empty() { "0" } else { integer };
    let fractional = fractional.trim_end_matches('0');
    let sign = if token.starts_with('-') { "-" } else { "" };
    Ok(if fractional.is_empty() {
        format!("{sign}{integer}")
    } else {
        format!("{sign}{integer}.{fractional}")
    })
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
                if op == "concat" && !matches!(acc, Type::Text | Type::Null) {
                    return Err("row expression concat requires text");
                }
                acc
            }
            "add" | "subtract" | "multiply" => {
                let left = self.expression(&node["left"], depth + 1)?;
                let right = self.expression(&node["right"], depth + 1)?;
                if left == Type::Null || right == Type::Null {
                    if matches!(left, Type::Numeric { .. } | Type::Null)
                        && matches!(right, Type::Numeric { .. } | Type::Null)
                    {
                        Type::Null
                    } else {
                        return Err("row expression arithmetic requires numeric operands");
                    }
                } else {
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
    use super::literal_payload_bytes;
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
}
