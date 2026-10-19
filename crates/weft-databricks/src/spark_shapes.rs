// SPDX-License-Identifier: MIT OR Apache-2.0
//! Spark-owned original ColumnSet projection facts. This grants no UC profile.
use serde_json::{json, Value};
use std::collections::BTreeMap;
use weft_core::{
    column_custody::{self, NativeColumnsEnvelope},
    raw_read::{AdmittedRead, Table},
};
type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone)]
pub(crate) struct Shape {
    pub native: String,
    pub metadata: Value,
}
pub(crate) fn columns(a: &AdmittedRead, table: &Table) -> Result<BTreeMap<String, Shape>> {
    let raw = a
        .request()
        .binding
        .native_columns
        .as_ref()
        .ok_or("original Spark ColumnSet custody required")?;
    let envelope: NativeColumnsEnvelope =
        serde_json::from_value(raw.clone()).map_err(|_| "closed nativeColumns custody required")?;
    if envelope.target != json!({"platform":"spark-delta","majorVersion":"spark4-delta4"})
        || envelope.vector_extension.is_some()
        || envelope.sets.is_empty()
        || envelope.sets.len() > 256
    {
        return Err("explicit Spark4 Delta4 native column profile required");
    }
    let doc = a.original_document();
    let record = a.original_record(&table.logical)?;
    let reference = json!({"module":record.identity.module,"element":record.identity.element});
    let mut physical = BTreeMap::new();
    for set in &envelope.sets {
        let original_set = column_custody::original_column_set(
            &a.request().binding.model_sha256,
            doc,
            &envelope.target,
            set,
            "0.8.0",
        )?;
        let members = record.value["members"]
            .as_array()
            .ok_or("original Record members absent")?;
        if !original_set["columns"]
            .as_array()
            .unwrap()
            .iter()
            .any(|column| members.contains(&column["field"]))
        {
            continue;
        }
        let current = column_custody::admit_columns(
            &a.request().binding.model_sha256,
            doc,
            &envelope.target,
            set,
            table,
            &reference,
            "0.8.0",
        )?;
        for (field, shape) in current {
            if physical.insert(field, shape).is_some() {
                return Err("duplicate original Spark column declaration");
            }
        }
    }
    let mut result = BTreeMap::new();
    for member in record.value["members"]
        .as_array()
        .ok_or("original Record members absent")?
    {
        let Some((native, ty, _)) = physical.get(&column_custody::field_key(member)) else {
            continue;
        };
        let field = locate(doc, member)?;
        let logical = field["name"]
            .as_str()
            .ok_or("original Field name required")?;
        if field["extensions"]
            .as_object()
            .is_none_or(|m| !m.is_empty())
            || field.get("allowedValues").is_some()
        {
            return Err("Spark projection Field constraints not qualified");
        }
        let nullable = column_custody::row_null(doc, &reference, member, &physical)?;
        let facets = field.get("facets").cloned().unwrap_or(json!({}));
        let mut metadata =
            json!({"carrier":field["scalarType"],"nullable":nullable,"nativeType":ty});
        if field["cardinality"] == "array" {
            let leaf = locate(doc, &field["itemType"])?;
            let vector = column_custody::vector_declaration(doc, &reference, member)?;
            let dimensions = vector["dimensions"]
                .as_u64()
                .filter(|n| (1..=16000).contains(n))
                .ok_or("bounded original Vector dimensions required")?;
            if ty != "array<float>"
                || leaf["scalarType"] != "float"
                || leaf["cardinality"] != "one"
                || leaf["nullability"] != "required"
                || leaf["extensions"].as_object().is_none_or(|m| !m.is_empty())
                || leaf.get("allowedValues").is_some()
                || leaf
                    .get("facets")
                    .is_some_and(|f| f.as_object().is_none_or(|m| !m.is_empty()))
                || facets != json!({"collectionSize":{"min":dimensions,"max":dimensions}})
            {
                return Err("exact fixed float32 Vector projection required");
            }
            metadata["carrier"] = json!("vector");
            metadata["dimensions"] = json!(dimensions);
        } else {
            if field["cardinality"] != "one" {
                return Err("Spark scalar projection cardinality refused");
            }
            match field["scalarType"].as_str() {
                Some("integer") => {
                    let width = &facets["integerWidth"];
                    let expected = match width["bits"].as_u64() {
                        Some(8) => "tinyint",
                        Some(16) => "smallint",
                        Some(32) => "int",
                        Some(64) => "bigint",
                        _ => return Err("signed integer width required"),
                    };
                    if width["signed"] != true
                        || width.as_object().is_none_or(|m| m.len() != 2)
                        || facets.as_object().is_none_or(|m| m.len() != 1)
                        || ty != expected
                    {
                        return Err("original Spark integer type differs");
                    };
                    metadata["integerWidth"] = width.clone();
                }
                Some("decimal") => {
                    let precision = facets["precision"]
                        .as_u64()
                        .filter(|n| (1..=38).contains(n))
                        .ok_or("exact Decimal precision required")?;
                    let scale = facets["scale"]
                        .as_u64()
                        .filter(|n| *n <= precision)
                        .ok_or("exact Decimal scale required")?;
                    if facets.as_object().is_none_or(|m| m.len() != 2)
                        || ty != &format!("decimal({precision},{scale})")
                    {
                        return Err("original Spark Decimal type differs");
                    };
                    metadata["precision"] = json!(precision);
                    metadata["scale"] = json!(scale);
                }
                Some("string" | "boolean" | "binary") => {
                    if facets.as_object().is_none_or(|m| !m.is_empty())
                        || field["scalarType"] != *ty
                    {
                        return Err("original Spark scalar type differs");
                    };
                    if ty == "string" {
                        metadata["collation"] = json!("UTF8_BINARY")
                    }
                }
                _ => return Err("Spark projection scalar family unqualified"),
            }
        }
        if let Some(bound) = table.columns.iter().find(|c| c.logical == logical) {
            if bound.native != *native
                || bound.nullable != nullable
                || serde_json::to_value(&bound.carrier).unwrap() != metadata["carrier"]
            {
                return Err("scalar binding differs from original Spark ColumnSet");
            }
        }
        if result
            .insert(
                logical.to_owned(),
                Shape {
                    native: native.clone(),
                    metadata,
                },
            )
            .is_some()
        {
            return Err("original Spark Field name ambiguous");
        }
    }
    if table
        .columns
        .iter()
        .any(|c| !result.contains_key(&c.logical))
    {
        return Err("all bound Spark columns require original custody");
    }
    Ok(result)
}
fn locate<'a>(doc: &'a Value, reference: &Value) -> Result<&'a Value> {
    doc["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == reference["module"])
        .and_then(|m| {
            m["elements"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["id"] == reference["element"])
        })
        .ok_or("original Field reference absent")
}
