// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
pub(crate) fn known_declaration(value: &Value) -> bool {
    fn validator(schema_source: &str, schema_id: &str) -> Option<jsonschema::Validator> {
        let base: Value = serde_json::from_str(include_str!(
            "../../../../spec/upstream/umf-tables-0.1.0.schema.json"
        ))
        .ok()?;
        let mut schema: Value = serde_json::from_str(schema_source).ok()?;
        for (key, value) in base["$defs"].as_object()? {
            if schema["$defs"].get(key).is_none() {
                schema["$defs"][key] = value.clone();
            }
        }
        fn localize(value: &mut Value, schema_id: &str) {
            match value {
                Value::Object(object) => {
                    if let Some(Value::String(reference)) = object.get_mut("$ref") {
                        if let Some(suffix) = reference.strip_prefix("urn:umf:tables:0.1.0#/") {
                            *reference = format!("#/{suffix}");
                        } else if let Some(suffix) =
                            reference.strip_prefix(&format!("{schema_id}#/"))
                        {
                            *reference = format!("#/{suffix}");
                        }
                    }
                    for child in object.values_mut() {
                        localize(child, schema_id)
                    }
                }
                Value::Array(items) => {
                    for child in items {
                        localize(child, schema_id)
                    }
                }
                _ => {}
            }
        }
        localize(&mut schema, schema_id);
        crate::table_lifecycle::close_declared_objects(&mut schema);
        jsonschema::options().build(&schema).ok()
    }
    static V01: std::sync::OnceLock<Option<jsonschema::Validator>> = std::sync::OnceLock::new();
    static V02: std::sync::OnceLock<Option<jsonschema::Validator>> = std::sync::OnceLock::new();
    let validator = if value["version"] == "umf.table-operation/0.2" {
        V02.get_or_init(|| {
            validator(
                include_str!("../../../../spec/upstream/umf-table-operation-0.2.schema.json"),
                "urn:umf:table-operation:0.2",
            )
        })
    } else {
        V01.get_or_init(|| {
            validator(
                include_str!("../../../../spec/upstream/umf-table-operation-0.1.schema.json"),
                "urn:umf:table-operation:0.1",
            )
        })
    };
    validator.as_ref().is_some_and(|v| v.is_valid(value))
}
