// SPDX-License-Identifier: MIT OR Apache-2.0
//! Narrow set-based lowering for source-independent plain INSERT batches.
use super::{Lower, q};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use weft_core::{
    raw_read::Carrier,
    table_operation::{AdmittedOperation, OperationCall},
};
type Result<T> = std::result::Result<T, &'static str>;

fn cast_json_scalar(expression: &str, carrier: &Carrier) -> String {
    match carrier {
        Carrier::Bits128 => format!("pg_catalog.decode(({expression})::pg_catalog.text,'hex')"),
        Carrier::Date => format!("({expression})::pg_catalog.date"),
        Carrier::Timestamp => format!("({expression})::pg_catalog.timestamptz"),
        Carrier::Uuid => format!("({expression})::pg_catalog.uuid"),
        Carrier::Boolean => format!("({expression})::pg_catalog.bool"),
        Carrier::Integer | Carrier::Decimal => format!("({expression})::pg_catalog.numeric"),
        Carrier::String => format!("({expression})::pg_catalog.text COLLATE pg_catalog.\"C\""),
    }
}

fn slot_kind(slot: &Value) -> Option<&str> {
    slot["source"]["kind"].as_str()
}

fn encoded_size<'a>(values: impl IntoIterator<Item = &'a Value>) -> Result<usize> {
    let mut size = 2usize; // JSON array brackets.
    let mut first = true;
    for value in values {
        if !first {
            size = size
                .checked_add(1)
                .ok_or("batch payload capacity exceeded")?;
        }
        first = false;
        size = size
            .checked_add(value.to_string().len())
            .ok_or("batch payload capacity exceeded")?;
    }
    Ok(size)
}

fn matrix_size(
    rows: &[BTreeMap<String, Value>],
    input_parameters: &[String],
    literals: &[Value],
) -> Result<usize> {
    let mut size = 2usize;
    for (row_index, row) in rows.iter().enumerate() {
        if row_index != 0 {
            size = size
                .checked_add(1)
                .ok_or("batch payload capacity exceeded")?;
        }
        size = size
            .checked_add(2)
            .ok_or("batch payload capacity exceeded")?;
        for (column_index, parameter) in input_parameters.iter().enumerate() {
            if column_index != 0 {
                size = size
                    .checked_add(1)
                    .ok_or("batch payload capacity exceeded")?;
            }
            size = size
                .checked_add(
                    row.get(parameter)
                        .ok_or("batch native input parameter absent")?
                        .to_string()
                        .len(),
                )
                .ok_or("batch payload capacity exceeded")?;
        }
    }
    size.checked_add(encoded_size(literals.iter())?)
        .ok_or("batch payload capacity exceeded")
}

/// Return `None` when the source profile cannot use the independent INSERT
/// lowering; the caller then retains the established sequential SQL.
pub(super) fn compile(
    operation: &AdmittedOperation,
    lower: &Lower<'_>,
    outputs: &[Value],
    projections: &[String],
    native_inputs: &[BTreeMap<String, Value>],
) -> Result<Option<Value>> {
    let body = &operation.declaration()["body"];
    if !matches!(&operation.request().call, OperationCall::Batch { .. })
        || body["kind"] != "insert"
        || !operation.batch_insert_scope()
        || operation.max_rows() == 0
    {
        return Ok(None);
    }

    let assignments = body["values"].as_array().ok_or("insert values absent")?;
    let mut field_slots = BTreeMap::new();
    for (index, slot) in lower.slots.iter().enumerate() {
        let field = weft_core::mutation::canonical(&slot["field"]);
        if field_slots.insert(field, index).is_some() {
            return Ok(None);
        }
    }
    let mut root_to_input = BTreeMap::new();
    let mut root_to_literal = BTreeMap::new();
    let mut input_parameters = Vec::new();
    let mut literal_values = Vec::new();
    for (index, slot) in lower.slots.iter().enumerate() {
        match slot_kind(slot) {
            Some("input") => {
                let parameter = slot["source"]["parameter"]
                    .as_str()
                    .ok_or("batch input slot absent")?;
                root_to_input.insert(index, input_parameters.len());
                input_parameters.push(parameter.to_owned());
            }
            Some("literal") => {
                root_to_literal.insert(index, literal_values.len());
                literal_values.push(slot["source"]["nativeValue"].clone());
            }
            _ => return Ok(None),
        }
    }
    if matrix_size(native_inputs, &input_parameters, &literal_values)? > 8_388_608 {
        return Ok(None);
    }

    let mut ctes = Vec::new();
    let mut source_columns =
        vec!["(v.umf_ordinality - 1)::pg_catalog.int8 AS umf_input_ordinal".to_owned()];
    let mut literal_columns = Vec::new();
    let mut source_expressions = BTreeMap::new();
    let mut row_expressions = BTreeMap::new();
    for (index, slot) in lower.slots.iter().enumerate() {
        let field = operation.field(&slot["field"])?;
        let slot_name = q(slot["id"].as_str().ok_or("batch slot id absent")?)?;
        if let Some(matrix_index) = root_to_input.get(&index) {
            let value =
                cast_json_scalar(&format!("v.umf_value ->> {matrix_index}"), &field.carrier);
            source_columns.push(format!("{value} AS {slot_name}"));
            source_expressions.insert(index, format!("i.{slot_name}"));
            row_expressions.insert(index, format!("i.{slot_name}"));
        } else if let Some(literal_index) = root_to_literal.get(&index) {
            let value = cast_json_scalar(
                &format!("(($2::pg_catalog.jsonb -> {literal_index}) #>> '{{}}')"),
                &field.carrier,
            );
            literal_columns.push(format!("{value} AS {slot_name}"));
            source_expressions.insert(index, format!("l.{slot_name}"));
            row_expressions.insert(index, format!("i.{slot_name}"));
        }
    }
    // Both per-input CTEs have one consumer. Keep them inline so the USING
    // sort does not follow a redundant full-input tuplestore.
    ctes.push(format!(
        "umf_batch_source AS NOT MATERIALIZED (SELECT {} FROM pg_catalog.jsonb_array_elements($1::pg_catalog.jsonb) WITH ORDINALITY AS v(umf_value, umf_ordinality))",
        source_columns.join(", ")
    ));
    if !literal_columns.is_empty() {
        ctes.push(format!(
            "umf_batch_literals AS MATERIALIZED (SELECT {})",
            literal_columns.join(", ")
        ));
    }
    let row_source = if literal_columns.is_empty() {
        "umf_batch_source AS i".to_owned()
    } else {
        "umf_batch_source AS i CROSS JOIN umf_batch_literals AS l".to_owned()
    };
    let mut row_columns = vec!["i.umf_input_ordinal".to_owned()];
    row_columns.extend(
        lower
            .slots
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                Ok(format!(
                    "{} AS {}",
                    source_expressions
                        .get(&index)
                        .ok_or("batch source expression absent")?,
                    q(slot["id"].as_str().ok_or("batch slot id absent")?)?
                ))
            })
            .collect::<Result<Vec<_>>>()?,
    );
    ctes.push(format!(
        "umf_batch_rows AS NOT MATERIALIZED (SELECT {} FROM {row_source})",
        row_columns.join(", ")
    ));

    let relation = format!(
        "{}.{}",
        q(operation
            .request()
            .binding
            .schema
            .as_deref()
            .ok_or("operation private schema absent")?)?,
        q(&operation.table().native)?
    );
    let insert_columns = assignments
        .iter()
        .map(|assignment| Ok(q(&operation.field(&assignment["field"])?.native)?))
        .collect::<Result<Vec<_>>>()?;
    let values = assignments
        .iter()
        .map(|assignment| match assignment["value"]["kind"].as_str() {
            Some("currentTimestamp") => Ok("CURRENT_TIMESTAMP".to_owned()),
            Some("currentPrincipal") => Ok("(CURRENT_USER::pg_catalog.text COLLATE pg_catalog.\"C\")".to_owned()),
            Some("parameter") => {
                let parameter = assignment["value"]["parameter"]
                    .as_str()
                    .ok_or("batch insert parameter absent")?;
                let field = weft_core::mutation::canonical(&assignment["field"]);
                let slot_index = *field_slots.get(&field).ok_or("batch insert slot absent")?;
                let slot = &lower.slots[slot_index];
                if slot_kind(slot) != Some("input") || slot["source"]["parameter"] != parameter {
                    return Err("batch insert source differs from compiled slot");
                }
                row_expressions
                    .get(&slot_index)
                    .cloned()
                    .ok_or("batch insert expression absent")
            }
            Some("literal") => {
                let field = weft_core::mutation::canonical(&assignment["field"]);
                let slot_index = *field_slots
                    .get(&field)
                    .ok_or("batch insert literal slot absent")?;
                if slot_kind(&lower.slots[slot_index]) != Some("literal") {
                    return Err("batch literal source differs from compiled slot");
                }
                row_expressions
                    .get(&slot_index)
                    .cloned()
                    .ok_or("batch insert expression absent")
            }
            _ => Err("batch insert value source unsupported"),
        })
        .collect::<Result<Vec<_>>>()?;

    let mut returned = vec!["i.umf_input_ordinal AS umf_input_ordinal".to_owned()];
    returned.extend(
        projections
            .get(..outputs.len())
            .ok_or("batch output projections absent")?
            .iter()
            .cloned(),
    );
    returned.push("1::pg_catalog.int8 AS umf_affected".to_owned());
    ctes.push(format!(
        "umf_batch_results AS (MERGE INTO {relation} AS t USING (SELECT * FROM umf_batch_rows ORDER BY umf_input_ordinal) AS i ON FALSE WHEN NOT MATCHED THEN INSERT ({}) VALUES ({}) RETURNING {})",
        insert_columns.join(", "),
        values.join(", "),
        returned.join(", ")
    ));

    let output_names: Vec<_> = outputs
        .iter()
        .map(|output| q(output["name"].as_str().ok_or("batch output name absent")?))
        .collect::<Result<_>>()?;
    let size_terms = output_names
        .iter()
        .map(|name| {
            format!(
                "COALESCE(pg_catalog.octet_length(m.{name}::pg_catalog.text)::pg_catalog.int8,0)"
            )
        })
        .chain([
            "pg_catalog.octet_length(m.umf_affected::pg_catalog.text)::pg_catalog.int8".to_owned(),
            "pg_catalog.octet_length(m.umf_input_ordinal::pg_catalog.text)::pg_catalog.int8"
                .to_owned(),
        ])
        .collect::<Vec<_>>();
    let expected_items = match &operation.request().call {
        OperationCall::Batch { items } => items.len(),
        _ => return Ok(None),
    };
    ctes.push(format!(
        "umf_batch_counts AS MATERIALIZED (SELECT pg_catalog.count(*) AS effects, COALESCE(pg_catalog.sum({}),0) AS frame_bytes FROM umf_batch_results AS m), umf_batch_guard AS MATERIALIZED (SELECT CASE WHEN effects <= {expected_items} AND frame_bytes <= 8388608 THEN 1::pg_catalog.int8 ELSE 1::pg_catalog.int8 / (effects - effects) END AS ok FROM umf_batch_counts)",
        size_terms.join(" + ")
    ));
    let mut final_projection = vec!["m.umf_input_ordinal".to_owned()];
    final_projection.extend(output_names.iter().map(|name| format!("m.{name}")));
    final_projection.push("m.umf_affected".to_owned());
    let sql = format!(
        "WITH {} SELECT {} FROM umf_batch_guard AS g CROSS JOIN umf_batch_results AS m WHERE g.ok = 1 ORDER BY m.umf_input_ordinal",
        ctes.join(", "),
        final_projection.join(", ")
    );

    let mut parameter_order = vec![json!("nativeRows")];
    if !literal_columns.is_empty() {
        parameter_order.push(json!("nativeLiterals"));
    }
    let mut artifact = json!({
        "version":"weft.table-insert-batch/0.1",
        "sql":sql,
        "parameterOrder":parameter_order,
        "ordinalColumn":"umf_input_ordinal"
    });
    crate::dbapi::attach(&mut artifact)?;
    Ok(Some(artifact))
}
