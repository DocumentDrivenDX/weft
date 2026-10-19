// SPDX-License-Identifier: MIT OR Apache-2.0
//! Original scalar domain binding and typed selection AST construction; no backend IO.
use crate::{
    model::{Catalog, Record},
    mutation::{fail, validate_ref, MResult},
    mutation_syntax::{Input, Predicate},
    security_literals::check_literal,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
pub(crate) struct Resolver<'a> {
    pub(crate) catalog: &'a Catalog,
    pub(crate) record: &'a Record,
    pub(crate) parameters: &'a serde_json::Map<String, Value>,
    pub(crate) used: BTreeMap<String, Value>,
    pub(crate) dependencies: Vec<Value>,
}
impl Resolver<'_> {
    pub(crate) fn field(&self, name: &crate::syntax::Name) -> MResult<(Value, Value)> {
        let mut matches = Vec::new();
        for member in self.record.value["members"]
            .as_array()
            .ok_or_else(|| fail("RESOLVE", "resolve", "Explicit Record members required"))?
        {
            let module = self.catalog.documents[self.record.document]["modules"]
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
    pub(crate) fn predicate(&mut self, p: &Predicate, depth: usize) -> MResult<Value> {
        if depth > 24 {
            return Err(fail("LIMIT", "limit", "Decoded predicate depth exceeded"));
        }
        match p {
            Predicate::Compare(name, op, input) => {
                let (reference, field) = self.field(name)?;
                if field["scalarType"] == "boolean" && !matches!(op.as_str(), "eq" | "ne") {
                    return Err(fail("TYPE", "type", "Boolean ordering refused"));
                }
                let value = self.value(input, &reference, &field)?;
                if !self.dependencies.contains(&reference) {
                    self.dependencies.push(reference.clone());
                }
                Ok(json!({"compare":{"field":reference,"op":op,"value":value}}))
            }
            Predicate::And(children) | Predicate::Or(children) => {
                let and = matches!(p, Predicate::And(_));
                let operator = if and { "and" } else { "or" };
                let mut out = Vec::new();
                for child in children {
                    let next = self.predicate(child, depth + 1)?;
                    if let Some(same) = next.get(operator).and_then(Value::as_array) {
                        out.extend(same.iter().cloned());
                    } else {
                        out.push(next);
                    }
                    if out.len() > 64 {
                        return Err(fail("LIMIT", "limit", "Predicate operand bound exceeded"));
                    }
                }
                Ok(json!({operator:out}))
            }
        }
    }
}
