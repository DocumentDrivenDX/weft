// SPDX-License-Identifier: MIT OR Apache-2.0
//! Original scalar domain binding and typed selection AST construction; no backend IO.
use crate::{
    model::{Catalog, Record},
    mutation::{fail, validate_ref, MResult},
    mutation_syntax::{Input, Predicate},
    security_literals::check_literal,
};
use serde_json::Value;
use std::collections::BTreeMap;
use weft_expr::selection;
use weft_model::types::ScalarFamily;
pub(crate) struct Resolver<'a> {
    pub(crate) catalog: &'a Catalog,
    pub(crate) record: &'a Record,
    pub(crate) parameters: &'a serde_json::Map<String, Value>,
    pub(crate) used: BTreeMap<String, Value>,
}
impl Resolver<'_> {
    pub(crate) fn field(&self, name: &crate::syntax::Name) -> MResult<(Value, Value)> {
        let mut matches = Vec::new();
        for member in self.record.value["members"]
            .as_array()
            .ok_or_else(|| fail("RESOLVE", "resolve", "Explicit Record members required"))?
        {
            let module = self.catalog.documents()[self.record.document]["modules"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["id"] == member["module"])
                .ok_or_else(|| fail("RESOLVE", "resolve", "Original member module missing"))?;
            let field = module["elements"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["id"] == member["element"])
                .ok_or_else(|| fail("RESOLVE", "resolve", "Original member Field missing"))?;
            if name.matches(field["name"].as_str().unwrap_or("")) {
                matches.push((member.clone(), field.clone()));
            }
        }
        if matches.len() != 1 {
            return Err(fail(
                "RESOLVE",
                "resolve",
                "Field name missing or ambiguous",
            ));
        }
        let (reference, value) = matches.pop().unwrap();
        validate_ref(&reference, false)?;
        if value["nullability"] != "required"
            || value["cardinality"] != "one"
            || !matches!(
                value["scalarType"].as_str(),
                Some("boolean" | "integer" | "string")
            )
        {
            return Err(fail(
                "TYPE",
                "type",
                "Required scalar mutation Field required",
            ));
        }
        crate::security_literals::validate_field(&value)
            .map_err(|_| fail("TYPE", "type", "Original Field domain unsupported"))?;
        Ok((reference, value))
    }
    pub(crate) fn value(
        &mut self,
        input: &Input,
        reference: &Value,
        field: &Value,
    ) -> MResult<Value> {
        let value = match input {
            Input::Literal(v) => v.clone(),
            Input::Marker(name) => {
                let value = self
                    .parameters
                    .get(name)
                    .ok_or_else(|| fail("TYPE", "type", "Missing typed parameter"))?
                    .clone();
                if let Some(previous) = self.used.insert(name.clone(), reference.clone()) {
                    if previous != *reference {
                        return Err(fail(
                            "TYPE",
                            "type",
                            "Marker reused across different Fields",
                        ));
                    }
                }
                value
            }
        };
        check_literal(field, &value)
            .map_err(|_| fail("TYPE", "type", "Literal violates original Field domain"))?;
        Ok(value)
    }
    pub(crate) fn predicate(&mut self, p: &Predicate, depth: usize) -> MResult<selection::Predicate> {
        if depth > selection::MAX_DEPTH {
            return Err(fail("LIMIT", "limit", "Decoded predicate depth exceeded"));
        }
        match p {
            Predicate::Compare(name, op, input) => {
                let (reference, field) = self.field(name)?;
                let family = field["scalarType"].as_str().and_then(ScalarFamily::from_umf);
                if family.is_some_and(|family| !op.admits(family)) {
                    return Err(fail("TYPE", "type", "Boolean ordering refused"));
                }
                let value = self.value(input, &reference, &field)?;
                Ok(selection::Predicate::Compare(selection::Compare { field: reference, op: *op, value }))
            }
            Predicate::And(children) | Predicate::Or(children) => {
                let and = matches!(p, Predicate::And(_));
                let mut out = Vec::new();
                for child in children {
                    let next = self.predicate(child, depth + 1)?;
                    selection::Predicate::splice(&mut out, and, next);
                    if out.len() > selection::MAX_OPERANDS {
                        return Err(fail("LIMIT", "limit", "Predicate operand bound exceeded"));
                    }
                }
                Ok(if and { selection::Predicate::And(out) } else { selection::Predicate::Or(out) })
            }
        }
    }
}
