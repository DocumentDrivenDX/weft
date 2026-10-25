// SPDX-License-Identifier: MIT OR Apache-2.0
//! Parameterized PG18 programs for original declared operations. No execution.
use serde_json::{json, Value};
use std::collections::BTreeMap;
use weft_core::raw_read::Carrier;
use weft_core::table_operation::{self, AdmittedOperation, OperationCall, OperationField};
mod insert_batch;
type Result<T> = std::result::Result<T, &'static str>;
fn q(value: &str) -> Result<String> {
    crate::Identifier::new(value)
        .map(|v| v.sql())
        .map_err(|_| "operation identifier outside PG18 profile")
}
struct Lower<'a> {
    operation: &'a AdmittedOperation,
    slots: Vec<Value>,
    identities: BTreeMap<String, usize>,
    source_scope: bool,
    related_scope: Option<(String, String)>,
    next_alias: usize,
}
impl<'a> Lower<'a> {
    fn field(&self, reference: &Value) -> Result<&'a OperationField> {
        if let Some((id, _)) = &self.related_scope {
            self.operation.related_field(id, reference)
        } else if self.source_scope {
            self.operation.source_field(reference)
        } else {
            self.operation.field(reference)
        }
    }
    fn column(&self, reference: &Value, alias: bool) -> Result<String> {
        Ok(format!(
            "{}{}",
            if !alias {
                String::new()
            } else if let Some((_, name)) = &self.related_scope {
                format!("{name}.")
            } else if self.source_scope {
                "s.".to_owned()
            } else {
                "t.".to_owned()
            },
            q(&self.field(reference)?.native)?
        ))
    }
    fn slot(&mut self, source: Value, field: &OperationField) -> String {
        let identity = weft_core::mutation::canonical(&json!([source, field.reference]));
        let index = if let Some(index) = self.identities.get(&identity) {
            *index
        } else {
            let index = self.slots.len();
            self.slots.push(json!({"id":format!("slot{index}"),"source":source,"field":field.reference,"carrier":field.carrier,"nullable":field.nullable}));
            self.identities.insert(identity, index);
            index
        };
        let slot = format!("${}", index + 1);
        match field.carrier {
            Carrier::Bits128 => format!("pg_catalog.decode({slot}::pg_catalog.text,'hex')"),
            Carrier::Date => format!("{slot}::pg_catalog.date"),
            Carrier::Timestamp => format!("{slot}::pg_catalog.timestamptz"),
            Carrier::LocalTimestamp => format!("{slot}::pg_catalog.timestamp"),
            Carrier::Uuid => format!("{slot}::pg_catalog.uuid"),
            Carrier::Boolean => format!("{slot}::pg_catalog.bool"),
            Carrier::Integer | Carrier::Decimal => format!("{slot}::pg_catalog.numeric"),
            Carrier::String => format!("({slot}::pg_catalog.text COLLATE pg_catalog.\"C\")"),
        }
    }
    fn value(&mut self, value: &Value, reference: &Value) -> Result<String> {
        let field = self.field(reference)?;
        if value["kind"] == "currentPrincipal" {
            return Ok("(CURRENT_USER::pg_catalog.text COLLATE pg_catalog.\"C\")".into());
        }
        if value["kind"] == "sourceField" {
            return Ok(format!(
                "s.{}",
                q(&self.operation.source_field(&value["field"])?.native)?
            ));
        }
        if value["kind"] == "currentTimestamp" {
            return Ok("CURRENT_TIMESTAMP".into());
        }
        Ok(self.slot(
            if value["kind"] == "parameter" {
                json!({"kind":"input","parameter":value["parameter"]})
            } else {
                json!({"kind":"literal","value":value["value"],"nativeValue":if let Some((id,_))=&self.related_scope { self.operation.native_related_value(id,reference,&value["value"])? } else if self.source_scope { self.operation.native_source_value(reference,&value["value"])? } else { self.operation.native_value(reference,&value["value"])? }})
            },
            field,
        ))
    }
    fn predicate(&mut self, node: &Value) -> Result<String> {
        let op = node["op"].as_str().unwrap();
        Ok(match op {
            "all" => "TRUE".into(),
            "and" | "or" => {
                let parts = node["args"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| self.predicate(n))
                    .collect::<Result<Vec<_>>>()?;
                format!(
                    "({})",
                    parts.join(if op == "and" { " AND " } else { " OR " })
                )
            }
            "not" => format!("(NOT {})", self.predicate(&node["arg"])?),
            "related" => {
                let id = node["table"]
                    .as_str()
                    .ok_or("related table identity absent")?;
                let table = self.operation.related_table(id)?;
                let alias = format!("umf_related_{}", self.next_alias);
                self.next_alias += 1;
                let relation = format!(
                    "{}.{}",
                    q(self
                        .operation
                        .request()
                        .binding
                        .schema
                        .as_deref()
                        .ok_or("native schema absent")?)?,
                    q(&table.native)?
                );
                let mut parts = vec![];
                for pair in node["matches"].as_array().ok_or("related matches absent")? {
                    let source = self.column(&pair["source"], true)?;
                    let target = self.operation.related_field(id, &pair["target"])?;
                    let target = format!("{alias}.{}", q(&target.native)?);
                    parts.push(format!("{source} = {target}"));
                }
                let previous = self.related_scope.replace((id.to_owned(), alias.clone()));
                let inner = self.predicate(&node["where"]);
                self.related_scope = previous;
                parts.push(inner?);
                format!(
                    "EXISTS (SELECT 1 FROM {relation} AS {alias} WHERE {})",
                    parts.join(" AND ")
                )
            }

            "isNull" => format!("({} IS NULL)", self.column(&node["field"], true)?),
            "in" => {
                let column = self.column(&node["field"], true)?;
                let values = node["values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| self.value(v, &node["field"]))
                    .collect::<Result<Vec<_>>>()?;
                format!("({column} IN ({}))", values.join(", "))
            }
            _ => {
                let column = self.column(&node["field"], true)?;
                let value = self.value(&node["value"], &node["field"])?;
                if op == "startsWith" {
                    format!("pg_catalog.starts_with({column}, {value})")
                } else {
                    let operator = match op {
                        "eq" => "IS NOT DISTINCT FROM",
                        "ne" => "IS DISTINCT FROM",
                        "lt" => "<",
                        "le" => "<=",
                        "gt" => ">",
                        "ge" => ">=",
                        _ => return Err("operation comparison unsupported"),
                    };
                    format!("({column} {operator} {value})")
                }
            }
        })
    }
    fn row_expression(&mut self, node: &Value, literal_field: &Value) -> Result<String> {
        let op = node["op"]
            .as_str()
            .ok_or("row expression operator absent")?;
        Ok(match op {
            "field" => {
                let field = self.operation.field(&node["field"])?;
                let column = self.column(&node["field"], true)?;
                match field.carrier {
                    Carrier::Integer | Carrier::Decimal => {
                        format!("({column})::pg_catalog.numeric")
                    }
                    Carrier::String => format!("({column} COLLATE pg_catalog.\"C\")"),
                    _ => column,
                }
            }
            "literal" => {
                let field = self.operation.field(literal_field)?;
                let value = &node["value"];
                let native = weft_core::table_operation::row_expression::native_literal(value)?;
                self.slot(
                    json!({"kind":"literal","value":value,"nativeValue":native}),
                    field,
                )
            }
            "coalesce" => {
                let args = node["args"]
                    .as_array()
                    .ok_or("row expression arguments absent")?
                    .iter()
                    .map(|n| self.row_expression(n, literal_field))
                    .collect::<Result<Vec<_>>>()?;
                format!("COALESCE({})", args.join(", "))
            }
            "concat" => {
                let args = node["args"]
                    .as_array()
                    .ok_or("row expression arguments absent")?
                    .iter()
                    .map(|n| self.row_expression(n, literal_field))
                    .collect::<Result<Vec<_>>>()?;
                let mut it = args.into_iter();
                let mut result = it.next().ok_or("row expression arguments absent")?;
                for rhs in it {
                    result = format!("({result} OPERATOR(pg_catalog.||) {rhs})");
                }
                result
            }
            "add" | "subtract" | "multiply" => {
                let left = self.row_expression(&node["left"], literal_field)?;
                let right = self.row_expression(&node["right"], literal_field)?;
                let operator = match op {
                    "add" => "+",
                    "subtract" => "-",
                    _ => "*",
                };
                format!("({left} OPERATOR(pg_catalog.{operator}) {right})")
            }
            "conditional" => {
                let condition = self.row_predicate(&node["when"])?;
                let then = self.row_expression(&node["then"], literal_field)?;
                let otherwise = self.row_expression(&node["else"], literal_field)?;
                format!("CASE WHEN ({condition}) IS TRUE THEN {then} ELSE {otherwise} END")
            }
            _ => return Err("row expression operator unsupported"),
        })
    }
    fn row_predicate_value(&mut self, value: &Value, reference: &Value) -> Result<String> {
        if value["kind"] != "literal" {
            return Err("row expression predicate value is not a literal");
        }
        let field = self.field(reference)?;
        let native = weft_core::table_operation::row_expression::native_literal(&value["value"])?;
        Ok(self.slot(
            json!({"kind":"literal","value":value["value"],"nativeValue":native}),
            field,
        ))
    }
    fn row_predicate(&mut self, node: &Value) -> Result<String> {
        let op = node["op"]
            .as_str()
            .ok_or("row expression condition operator absent")?;
        Ok(match op {
            "all" => "TRUE".into(),
            "and" | "or" => {
                let terms = node["args"]
                    .as_array()
                    .ok_or("row expression condition args absent")?
                    .iter()
                    .map(|n| self.row_predicate(n))
                    .collect::<Result<Vec<_>>>()?;
                format!(
                    "({})",
                    terms.join(if op == "and" { " AND " } else { " OR " })
                )
            }
            "not" => format!("(NOT {})", self.row_predicate(&node["arg"])?),
            "isNull" => format!("({} IS NULL)", self.column(&node["field"], true)?),
            "in" => {
                let column = self.column(&node["field"], true)?;
                let vals = node["values"]
                    .as_array()
                    .ok_or("row expression condition values absent")?
                    .iter()
                    .map(|v| self.row_predicate_value(v, &node["field"]))
                    .collect::<Result<Vec<_>>>()?;
                format!(
                    "({})",
                    vals.into_iter()
                        .map(|value| format!("({column} OPERATOR(pg_catalog.=) {value})"))
                        .collect::<Vec<_>>()
                        .join(" OR ")
                )
            }
            "startsWith" => {
                let column = self.column(&node["field"], true)?;
                let value = self.row_predicate_value(&node["value"], &node["field"])?;
                format!("pg_catalog.starts_with(({column} COLLATE pg_catalog.\"C\"), {value})")
            }
            "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {
                let mut column = self.column(&node["field"], true)?;
                if self.field(&node["field"])?.carrier == Carrier::String {
                    column = format!("({column} COLLATE pg_catalog.\"C\")");
                }
                let value = self.row_predicate_value(&node["value"], &node["field"])?;
                let operator = match op {
                    "eq" => "=",
                    "ne" => "<>",
                    "lt" => "<",
                    "le" => "<=",
                    "gt" => ">",
                    _ => ">=",
                };
                if op == "eq" || op == "ne" {
                    let equal = format!(
                        "(({column} OPERATOR(pg_catalog.=) {value}) IS TRUE OR ({column} IS NULL AND {value} IS NULL))"
                    );
                    if op == "eq" {
                        equal
                    } else {
                        format!("(NOT {equal})")
                    }
                } else {
                    format!("({column} OPERATOR(pg_catalog.{operator}) {value})")
                }
            }
            _ => return Err("row expression condition operator unsupported"),
        })
    }
    fn projection(&self, reference: &Value, index: usize) -> Result<(String, Value)> {
        let field = self.operation.field(reference)?;
        let column = self.column(reference, true)?;
        let expression = match field.carrier {
            Carrier::Date => format!("pg_catalog.to_char({column}::pg_catalog.timestamp, 'YYYY-MM-DD')"),
            Carrier::Timestamp => format!("pg_catalog.to_char({column} AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"')"),
            Carrier::LocalTimestamp => format!("pg_catalog.to_char({column}, 'YYYY-MM-DD\"T\"HH24:MI:SS.US')"),
            Carrier::Boolean => column,
            Carrier::Bits128 => format!("pg_catalog.encode({column},'hex')"),
            _ => format!("({column})::pg_catalog.text"),
        };
        let label = format!("umf_result_{index}");
        Ok((
            format!("{expression} AS {}", q(&label)?),
            json!({"name":label,"field":reference,"carrier":field.carrier,"nullable":field.nullable}),
        ))
    }
}
fn compile(raw: &str) -> Result<Value> {
    let operation = table_operation::admit_json(raw)?;
    let mut program = compile_admitted(&operation, true)?;
    program["requestJson"] = json!(raw);
    program["requestSha256"] = json!(weft_core::json::sha256(raw.as_bytes()));
    program["binding"] = json!(operation.request().binding.as_ref());
    program["declaration"] = operation.declaration().clone();
    program["structuralDigest"] = json!(weft_core::json::sha256(
        weft_core::mutation::canonical(&program).as_bytes()
    ));
    Ok(program)
}
pub fn compile_admitted(
    operation: &table_operation::AdmittedOperation,
    compact: bool,
) -> Result<Value> {
    let request = operation.request();
    let body = &operation.declaration()["body"];
    let kind = body["kind"].as_str().unwrap();
    let read = kind == "read";
    let relation = format!(
        "{}.{}",
        q(request
            .binding
            .schema
            .as_deref()
            .ok_or("operation private schema absent")?)?,
        q(&operation.table().native)?
    );
    let mut lower = Lower {
        operation,
        slots: vec![],
        identities: BTreeMap::new(),
        source_scope: false,
        related_scope: None,
        next_alias: 0,
    };
    let mut outputs = vec![];
    let mut projections = vec![];
    let references = body[if read { "fields" } else { "returning" }]
        .as_array()
        .unwrap();
    for (index, reference) in references.iter().enumerate() {
        let (sql, output) = lower.projection(reference, index)?;
        projections.push(sql);
        outputs.push(output);
    }
    if !read {
        projections.push("1 AS umf_affected".into());
    }
    let mut cursor = Value::Null;
    let mut frame_order = vec![];
    let sql = if read {
        let result = body["result"].as_str().unwrap();
        let mut predicate = lower.predicate(&body["where"])?;
        if result == "page" {
            let order = body["orderBy"].as_array().unwrap();
            let fields = order.iter().map(|o| o["field"].clone()).collect::<Vec<_>>();
            cursor = json!({"key":body["cursorKey"],"fields":fields,"ordering":"declared-lexicographic-order","directions":order.iter().map(|o|o["direction"].clone()).collect::<Vec<_>>(),"requiresSourceAndPrincipalBinding":true});
            if matches!(
                operation.call(),
                OperationCall::Single { after: Some(_), .. }
            ) {
                let columns = fields
                    .iter()
                    .map(|f| lower.column(f, true))
                    .collect::<Result<Vec<_>>>()?;
                let values = fields
                    .iter()
                    .enumerate()
                    .map(|(index, reference)| {
                        Ok(lower.slot(
                            json!({"kind":"after","index":index}),
                            operation.field(reference)?,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let ascending = order.iter().all(|o| o["direction"] == "asc");
                let descending = order.iter().all(|o| o["direction"] == "desc");
                let compare = if ascending || descending {
                    let operator = if ascending { ">" } else { "<" };
                    if fields.len() == 1 {
                        format!("{} {operator} {}", columns[0], values[0])
                    } else {
                        format!(
                            "({}) {operator} ({})",
                            columns.join(", "),
                            values.join(", ")
                        )
                    }
                } else {
                    // Mixed directions require a lexicographic disjunction. Bind
                    // each scalar once; no sign flips, casts or precision loss.
                    (0..fields.len())
                        .map(|i| {
                            let mut terms = (0..i)
                                .map(|j| format!("{} = {}", columns[j], values[j]))
                                .collect::<Vec<_>>();
                            terms.push(format!(
                                "{} {} {}",
                                columns[i],
                                if order[i]["direction"] == "asc" {
                                    ">"
                                } else {
                                    "<"
                                },
                                values[i]
                            ));
                            format!("({})", terms.join(" AND "))
                        })
                        .collect::<Vec<_>>()
                        .join(" OR ")
                };
                predicate = format!("({predicate}) AND ({compare})");
            }
        }
        if result == "count" {
            outputs = vec![json!({"name":"umf_count","carrier":"integer","nullable":false})];
            format!("SELECT (COUNT(*))::pg_catalog.text AS umf_count FROM {relation} AS t WHERE {predicate}")
        } else {
            let order = body["orderBy"]
                .as_array()
                .unwrap()
                .iter()
                .map(|o| {
                    Ok(format!(
                        "{} {}",
                        lower.column(&o["field"], true)?,
                        if o["direction"] == "asc" {
                            "ASC"
                        } else {
                            "DESC"
                        }
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            for (index, ordering) in body["orderBy"].as_array().unwrap().iter().enumerate() {
                let label = format!("umf_sort_{index}");
                projections.push(format!(
                    "{} AS {}",
                    lower.column(&ordering["field"], true)?,
                    q(&label)?
                ));
                frame_order.push(format!(
                    "r.{} {}",
                    q(&label)?,
                    if ordering["direction"] == "asc" {
                        "ASC"
                    } else {
                        "DESC"
                    }
                ));
            }
            format!(
                "SELECT {} FROM {relation} AS t WHERE {predicate}{} LIMIT {}",
                projections.join(", "),
                if order.is_empty() {
                    String::new()
                } else {
                    format!(" ORDER BY {}", order.join(", "))
                },
                operation.max_rows() + 1
            )
        }
    } else {
        let returning = projections.join(", ");
        match kind {
            "insert" | "upsert" => {
                let mut columns = vec![];
                let mut values = vec![];
                for assignment in body["values"].as_array().unwrap() {
                    columns.push(lower.column(&assignment["field"], false)?);
                    values.push(lower.value(&assignment["value"], &assignment["field"])?);
                }
                let conflict = if kind == "upsert" {
                    let keys = operation
                        .key(body["conflictKey"].as_str().unwrap())?
                        .iter()
                        .map(|r| lower.column(r, false))
                        .collect::<Result<Vec<_>>>()?;
                    if body["onConflict"] == "ignore" {
                        format!(" ON CONFLICT ({}) DO NOTHING", keys.join(", "))
                    } else {
                        let updates = body["updateFields"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|r| {
                                let column = lower.column(r, false)?;
                                Ok(format!("{column} = EXCLUDED.{column}"))
                            })
                            .collect::<Result<Vec<_>>>()?;
                        format!(
                            " ON CONFLICT ({}) DO UPDATE SET {}",
                            keys.join(", "),
                            updates.join(", ")
                        )
                    }
                } else {
                    String::new()
                };
                format!(
                    "INSERT INTO {relation} AS t ({}) VALUES ({}){conflict} RETURNING {returning}",
                    columns.join(", "),
                    values.join(", ")
                )
            }
            "insertFrom" => {
                let mut columns = vec![];
                let mut values = vec![];
                for assignment in body["values"].as_array().unwrap() {
                    columns.push(lower.column(&assignment["field"], false)?);
                    values.push(lower.value(&assignment["value"], &assignment["field"])?);
                }
                lower.source_scope = true;
                let predicate = lower.predicate(&body["source"]["where"])?;
                lower.source_scope = false;
                let source = operation.source_table().ok_or("source relation absent")?;
                let source_relation = format!(
                    "{}.{}",
                    q(request
                        .binding
                        .schema
                        .as_deref()
                        .ok_or("source schema absent")?)?,
                    q(&source.native)?
                );
                format!("INSERT INTO {relation} AS t ({}) SELECT {} FROM {source_relation} AS s WHERE {predicate} RETURNING {returning}",columns.join(", "),values.join(", "))
            }
            "update" => {
                let assignments = body["values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|a| {
                        if a["value"]["kind"] == "rowExpression" {
                            let column = lower.column(&a["field"], false)?;
                            let expression =
                                lower.row_expression(&a["value"]["expression"], &a["field"])?;
                            return Ok(format!("{column} = {expression}"));
                        }
                        Ok(format!(
                            "{} = {}",
                            lower.column(&a["field"], false)?,
                            lower.value(&a["value"], &a["field"])?
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let predicate = lower.predicate(&body["where"])?;
                if operation.request().version == "weft.table-operation/0.2" {
                    let target = &body["values"][0]["field"];
                    let target_col = lower.column(target, true)?;
                    let exact =
                        lower.row_expression(&body["values"][0]["value"]["expression"], target)?;
                    let finite=weft_core::table_operation::row_expression::numeric_input_fields(operation)?.iter().map(|reference| {
                        let field=operation.field(reference)?;
                        if field.native_type["kind"]!="numeric" { return Ok("TRUE".to_owned()); }
                        let col=lower.column(reference,true)?;
                        Ok(format!("({col} IS NULL OR ({col} OPERATOR(pg_catalog.<>) 'NaN'::pg_catalog.numeric AND {col} OPERATOR(pg_catalog.<>) 'Infinity'::pg_catalog.numeric AND {col} OPERATOR(pg_catalog.<>) '-Infinity'::pg_catalog.numeric))"))
                    }).collect::<Result<Vec<_>>>()?;
                    let finite = if finite.is_empty() {
                        "TRUE".to_owned()
                    } else {
                        finite.join(" AND ")
                    };
                    let target_type = operation.field(target)?;
                    let stored = match target_type.carrier {
                        Carrier::Integer | Carrier::Decimal => {
                            format!("({target_col})::pg_catalog.numeric")
                        }
                        Carrier::String => format!("({target_col} COLLATE pg_catalog.\"C\")"),
                        _ => target_col.clone(),
                    };
                    let output_finite = if matches!(
                        target_type.carrier,
                        Carrier::Integer | Carrier::Decimal
                    ) && target_type.native_type["kind"] == "numeric"
                    {
                        format!(" AND ({target_col} OPERATOR(pg_catalog.<>) 'NaN'::pg_catalog.numeric AND {target_col} OPERATOR(pg_catalog.<>) 'Infinity'::pg_catalog.numeric AND {target_col} OPERATOR(pg_catalog.<>) '-Infinity'::pg_catalog.numeric)")
                    } else {
                        String::new()
                    };
                    format!("UPDATE {relation} AS t SET {} WHERE {predicate} RETURNING 1 AS umf_affected, (({stored} OPERATOR(pg_catalog.=) {exact}) IS TRUE) AS umf_native_exact, ({target_col} IS NOT NULL) AS umf_native_nonnull, (({finite}){output_finite}) AS umf_native_finite",assignments.join(", "))
                } else {
                    format!(
                        "UPDATE {relation} AS t SET {} WHERE {predicate} RETURNING {returning}",
                        assignments.join(", ")
                    )
                }
            }
            "delete" => {
                let predicate = lower.predicate(&body["where"])?;
                format!("DELETE FROM {relation} AS t WHERE {predicate} RETURNING {returning}")
            }
            _ => return Err("operation mutation unsupported"),
        }
    };
    // A data-modifying CTE always completes its effects. The aggregate gate
    // raises a runtime SQL error on excess effects, so PG rolls back the entire
    // statement rather than clipping a write or returning an unbounded frame.
    let sql = if read {
        let visible = outputs
            .iter()
            .map(|o| {
                Ok(format!(
                    "r.{}",
                    q(o["name"].as_str().ok_or("read output identity absent")?)?
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        // octet_length measures uncompressed data even when PostgreSQL keeps a
        // small TOAST pointer/compressed datum; pg_column_size is insufficient.
        let sizes = visible
            .iter()
            .map(|name| {
                format!(
                    "COALESCE(pg_catalog.octet_length({name}::pg_catalog.text)::pg_catalog.int8,0)"
                )
            })
            .collect::<Vec<_>>();
        format!("WITH umf_read AS MATERIALIZED ({sql}), umf_frame AS MATERIALIZED (SELECT CASE WHEN COALESCE(pg_catalog.sum({}),0) <= 8388608 THEN 1::pg_catalog.int8 ELSE 1::pg_catalog.int8 / (pg_catalog.count(*) - pg_catalog.count(*)) END AS ok FROM umf_read AS r) SELECT {} FROM umf_frame AS b CROSS JOIN umf_read AS r WHERE b.ok=1{}",sizes.join(" + "),visible.join(", "),if frame_order.is_empty(){String::new()}else{format!(" ORDER BY {}",frame_order.join(", "))})
    } else {
        let mut sizes = outputs
            .iter()
            .map(|output| {
                Ok(format!(
                    "COALESCE(pg_catalog.octet_length(e.{}::pg_catalog.text)::pg_catalog.int8,0)",
                    q(output["name"]
                        .as_str()
                        .ok_or("write output identity absent")?)?
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        sizes.push(
            "pg_catalog.octet_length(e.umf_affected::pg_catalog.text)::pg_catalog.int8".into(),
        );
        if operation.request().version == "weft.table-operation/0.2" {
            format!("WITH umf_effects AS ({sql}), umf_bound AS MATERIALIZED (SELECT CASE WHEN pg_catalog.count(*) OPERATOR(pg_catalog.<=) {} AND COALESCE(pg_catalog.sum({}),0) OPERATOR(pg_catalog.<=) 8388608 AND COALESCE(pg_catalog.bool_and(e.umf_native_exact AND e.umf_native_nonnull AND e.umf_native_finite),TRUE) THEN 1::pg_catalog.int8 ELSE 1::pg_catalog.int8 OPERATOR(pg_catalog./) (pg_catalog.count(*) OPERATOR(pg_catalog.-) pg_catalog.count(*)) END AS ok FROM umf_effects AS e) SELECT e.umf_affected FROM umf_bound AS b CROSS JOIN umf_effects AS e WHERE b.ok OPERATOR(pg_catalog.=) 1",operation.max_rows(),sizes.join(" OPERATOR(pg_catalog.+) "))
        } else {
            format!("WITH umf_effects AS ({sql}), umf_bound AS MATERIALIZED (SELECT CASE WHEN pg_catalog.count(*) <= {} AND COALESCE(pg_catalog.sum({}),0) <= 8388608 THEN 1::pg_catalog.int8 ELSE 1::pg_catalog.int8 / (pg_catalog.count(*) - pg_catalog.count(*)) END AS ok FROM umf_effects AS e) SELECT e.* FROM umf_bound AS b CROSS JOIN umf_effects AS e WHERE b.ok = 1",operation.max_rows(),sizes.join(" + "))
        }
    };
    let native_inputs = operation.native_inputs()?;
    let native_batch =
        insert_batch::compile(operation, &lower, &outputs, &projections, &native_inputs)?;
    let batch = matches!(operation.call(), OperationCall::Batch { .. });
    let mut program = json!({"version":request.version,"status":"compiled","target":request.target,
  "binding":{"modelSha256":request.binding.model_sha256},
  "module":request.module,"operation":request.operation,"callKind":if batch {"batch"} else {"single"},"idempotencyKey":operation.idempotency_key(),
  "nativeInputs":native_inputs,"nativeAfter":operation.native_after()?,
  "declarationDigest":operation.declaration_digest(),
  "sql":sql,"transport":{"profile":"dbapi-percent/0.1","bindEmptyParameterTuple":true},"parameterOrder":lower.slots.iter().map(|s|s["id"].clone()).collect::<Vec<_>>(),"parameterSources":lower.slots,"outputs":outputs,
  "result":{"kind":if read{body["result"].clone()}else{json!("mutation")},"maxRows":operation.max_rows(),"declaredMaxRows":body["maxRows"],"countScope":"direct-target-rows","referentialActions":"source-declared-native-FK-actions-not-counted-by-this-bound","affectedMarker":if read{Value::Null}else{json!("umf_affected")},"nativeReadGuard":if read{json!({"profile":"pg18-read-frame/0.1","maxNativeBytes":8388608,"sqlstate":"22012","includesLookahead":true,"partialFrames":false})}else{Value::Null},"nativeWriteFrameGuard":if read{Value::Null}else{json!({"profile":"pg18-write-frame/0.1","maxNativeBytes":8388608,"sqlstate":"22012","includesLookahead":false,"partialFrames":false})},"nativeBoundGuard":if read{Value::Null}else{json!({"profile":"pg18-effect-count/0.1","sqlstate":"22012","excessEffects":"statement-error-and-transaction-rollback","writeLimitClipping":false})},"lookahead":read&&body["result"]!="count","cursor":cursor},
  "obligations":{"tableLifecycle":operation.obligations(),"nativeExecutionQualified":false,
   "nativeFunctionAndOperatorClosure":"required-before-execution","consistency":operation.declaration()["consistency"],
   "transaction":{"batch":batch,"atomic":"entire-invocation","commitResponsibility":"host-caller-contract","onBoundExceeded":"rollback-entire-operation-or-batch","writeLimitClipping":false},
   "replay":{"mode":operation.declaration()["replay"],"durableOutcomeRequired":!read,"automaticRetry":false,"scope":["module","operation","source","deployment","ledger-head","verified-principal","ordered-exact-inputs","caller-key"]},
   "cursorScope":["operation","exact-inputs","source","deployment","ledger-head","principal","role-policy-scope"]}});
    if request.version == "weft.table-operation/0.2" {
        program["result"]["nativeExactValueGuard"] = json!({"profile":"pg18-row-expression-exact/0.1","sqlstate":"22012","scope":"all-direct-target-rows","nullResult":"statement-error-and-transaction-rollback","nonfiniteInput":"statement-error-and-transaction-rollback","lossyAssignment":"statement-error-and-transaction-rollback","privateResults":true});
    }
    crate::dbapi::attach(&mut program)?;
    if let Some(native_batch) = native_batch {
        program["nativeBatch"] = native_batch;
    }
    if !compact {
        program["binding"] = json!(request.binding.as_ref());
        program["declaration"] = operation.declaration().clone();
        program["structuralDigest"] = json!(weft_core::json::sha256(
            weft_core::mutation::canonical(&program).as_bytes()
        ));
    }
    Ok(program)
}
pub struct PreparedOperationModule {
    inner: weft_core::table_operation::PreparedOperationModule,
    query_module: std::sync::OnceLock<
        std::result::Result<super::table_query::PreparedQueryModule, &'static str>,
    >,
}
pub struct PreparedOperation {
    inner: weft_core::table_operation::PreparedOperation,
}
impl PreparedOperationModule {
    pub fn prepare(source: &str) -> Result<Self> {
        weft_core::table_operation::prepare_module_json(source).map(|inner| Self {
            inner,
            query_module: std::sync::OnceLock::new(),
        })
    }
    pub fn prepare_operation(&self, operation: &str) -> Result<PreparedOperation> {
        self.inner
            .prepare_operation(operation)
            .map(|inner| PreparedOperation { inner })
    }
    pub fn prepare_query(&self, operation: &str) -> Result<super::table_query::PreparedQuery> {
        let module = self.query_module.get_or_init(|| {
            self.inner
                .prepare_query_module()
                .map(super::table_query::PreparedQueryModule::from_admitted)
        });
        let module = module.as_ref().map_err(|error| *error)?;
        module.prepare_operation(operation)
    }
}
impl PreparedOperation {
    pub fn bind_json(&self, call: &str) -> String {
        match self
            .inner
            .bind_json(call)
            .and_then(|operation| compile_admitted(&operation, true))
        {
            Ok(program) => {
                let output = program.to_string();
                if output.len() > 33_554_432 {
                    blocked("operation program capacity exceeded")
                } else {
                    output
                }
            }
            Err(reason) => blocked(reason),
        }
    }
}
pub fn compile_json(request: &str) -> String {
    match compile(request) {
        Ok(program) => {
            let output = program.to_string();
            if output.len() > 33_554_432 {
                blocked("operation program capacity exceeded")
            } else {
                output
            }
        }
        Err(reason) => blocked(reason),
    }
}
fn blocked(reason: &str) -> String {
    json!({"version":"weft.table-operation/0.1","status":"blocked","diagnostics":[{"code":reason}]})
        .to_string()
}
pub fn check_json(raw: &str) -> String {
    if raw.len() > 33_554_432 {
        return blocked("operation program capacity exceeded");
    }
    let program = match weft_core::json::checked_json(raw) {
        Ok(value) => value,
        Err(_) => return blocked("operation program JSON invalid"),
    };
    let Some(original) = program["requestJson"].as_str() else {
        return blocked("operation original request absent");
    };
    match compile(original){Ok(expected)if expected==program=>json!({"version":"weft.table-operation-check/0.1","status":"checked","structuralDigest":program["structuralDigest"],"nativeExecutionQualified":false}).to_string(),_=>blocked("operation source recompilation differs")}
}
