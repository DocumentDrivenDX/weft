// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
pub(crate) fn known_declaration(value: &Value) -> bool {
    static VALIDATOR: std::sync::OnceLock<Option<jsonschema::Validator>> =
        std::sync::OnceLock::new();
    VALIDATOR
        .get_or_init(|| {
            let base: Value = serde_json::from_str(include_str!(
                "../../../../spec/upstream/umf-tables-0.1.0.schema.json"
            ))
            .ok()?;
            let mut schema: Value = serde_json::from_str(include_str!(
                "../../../../spec/upstream/umf-table-operation-0.1.schema.json"
            ))
            .ok()?;
            for (key, value) in base["$defs"].as_object()? {
                if schema["$defs"].get(key).is_some() {
                    return None;
                }
                schema["$defs"][key] = value.clone();
            }
            fn localize(value: &mut Value) {
                match value {
                    Value::Object(object) => {
                        if let Some(Value::String(reference)) = object.get_mut("$ref") {
                            if let Some(suffix) = reference.strip_prefix("urn:umf:tables:0.1.0#/") {
                                *reference = format!("#/{suffix}");
                            }
                        }
                        for child in object.values_mut() {
                            localize(child)
                        }
                    }
                    Value::Array(items) => {
                        for child in items {
                            localize(child)
                        }
                    }
                    _ => {}
                }
            }
            localize(&mut schema);
            crate::table_lifecycle::close_declared_objects(&mut schema);
            jsonschema::options().build(&schema).ok()
        })
        .as_ref()
        .is_some_and(|v| v.is_valid(value))
}
