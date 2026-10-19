// SPDX-License-Identifier: MIT OR Apache-2.0
//! Parameterized PG18 programs for original declared operations. No execution.
use serde_json::{json, Value};
use std::collections::BTreeMap;
use weft_core::raw_read::Carrier;
use weft_core::table_operation::{self, AdmittedOperation, OperationCall, OperationField};
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
}
impl Lower<'_> {
    fn column(&self, reference: &Value, alias: bool) -> Result<String> {
        Ok(format!(
            "{}{}",
            if alias { "t." } else { "" },
            q(&self.operation.field(reference)?.native)?
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
            Carrier::Uuid => format!("{slot}::pg_catalog.uuid"),
            Carrier::Boolean => format!("{slot}::pg_catalog.bool"),
            Carrier::Integer | Carrier::Decimal => format!("{slot}::pg_catalog.numeric"),
            Carrier::String => format!("({slot}::pg_catalog.text COLLATE pg_catalog.\"C\")"),
        }
    }
    fn value(&mut self, value: &Value, reference: &Value) -> Result<String> {
        let field = self.operation.field(reference)?;
        Ok(self.slot(
            if value["kind"] == "parameter" {
                json!({"kind":"input","parameter":value["parameter"]})
            } else {
                json!({"kind":"literal","value":value["value"],"nativeValue":self.operation.native_value(reference,&value["value"])?})
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
    fn projection(&self, reference: &Value, index: usize) -> Result<(String, Value)> {
        let field = self.operation.field(reference)?;
        let column = self.column(reference, true)?;
        let expression = match field.carrier {
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
        operation: &operation,
        slots: vec![],
        identities: BTreeMap::new(),
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
    let sql = if read {
        let result = body["result"].as_str().unwrap();
        let mut predicate = lower.predicate(&body["where"])?;
        if result == "page" {
            let fields = operation.key(body["cursorKey"].as_str().unwrap())?;
            cursor = json!({"key":body["cursorKey"],"fields":fields,"ordering":"ascending-complete-original-key","requiresSourceAndPrincipalBinding":true});
            if matches!(&request.call, OperationCall::Single { after: Some(_), .. }) {
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
                let compare = if fields.len() == 1 {
                    format!("{} > {}", columns[0], values[0])
                } else {
                    format!("({}) > ({})", columns.join(", "), values.join(", "))
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
            "update" => {
                let assignments = body["values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|a| {
                        Ok(format!(
                            "{} = {}",
                            lower.column(&a["field"], false)?,
                            lower.value(&a["value"], &a["field"])?
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let predicate = lower.predicate(&body["where"])?;
                format!(
                    "UPDATE {relation} AS t SET {} WHERE {predicate} RETURNING {returning}",
                    assignments.join(", ")
                )
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
        sql
    } else {
        format!("WITH umf_effects AS ({sql}), umf_bound AS MATERIALIZED (SELECT CASE WHEN pg_catalog.count(*) <= {} THEN 1::pg_catalog.int8 ELSE 1::pg_catalog.int8 / (pg_catalog.count(*) - pg_catalog.count(*)) END AS ok FROM umf_effects) SELECT e.* FROM umf_bound AS b CROSS JOIN umf_effects AS e WHERE b.ok = 1",operation.max_rows())
    };
    let batch = matches!(request.call, OperationCall::Batch { .. });
    let mut program = json!({"version":"weft.table-operation/0.1","status":"compiled","target":request.target,
  "requestJson":raw,"requestSha256":weft_core::json::sha256(raw.as_bytes()),"binding":request.binding,
  "module":request.module,"operation":request.operation,"declaration":operation.declaration(),
  "nativeInputs":operation.native_inputs()?,"nativeAfter":operation.native_after()?,
  "declarationDigest":weft_core::json::sha256(weft_core::mutation::canonical(operation.declaration()).as_bytes()),
  "sql":sql,"transport":{"profile":"dbapi-percent/0.1","bindEmptyParameterTuple":true},"parameterOrder":lower.slots.iter().map(|s|s["id"].clone()).collect::<Vec<_>>(),"parameterSources":lower.slots,"outputs":outputs,
  "result":{"kind":if read{body["result"].clone()}else{json!("mutation")},"maxRows":operation.max_rows(),"declaredMaxRows":body["maxRows"],"countScope":"direct-target-rows","referentialActions":"source-declared-native-FK-actions-not-counted-by-this-bound","affectedMarker":if read{Value::Null}else{json!("umf_affected")},"nativeBoundGuard":if read{Value::Null}else{json!({"profile":"pg18-effect-count/0.1","sqlstate":"22012","excessEffects":"statement-error-and-transaction-rollback","writeLimitClipping":false})},"lookahead":read&&body["result"]!="count","cursor":cursor},
  "obligations":{"tableLifecycle":operation.obligations(),"nativeExecutionQualified":false,
   "nativeFunctionAndOperatorClosure":"required-before-execution","consistency":operation.declaration()["consistency"],
   "transaction":{"batch":batch,"atomic":"entire-invocation","commitResponsibility":"host-caller-contract","onBoundExceeded":"rollback-entire-operation-or-batch","writeLimitClipping":false},
   "replay":{"mode":operation.declaration()["replay"],"durableOutcomeRequired":!read,"automaticRetry":false,"scope":["module","operation","source","deployment","ledger-head","verified-principal","ordered-exact-inputs","caller-key"]},
   "cursorScope":["operation","exact-inputs","source","deployment","ledger-head","principal","role-policy-scope"]}});
    crate::dbapi::attach(&mut program)?;
    program["structuralDigest"] = json!(weft_core::json::sha256(
        weft_core::mutation::canonical(&program).as_bytes()
    ));
    Ok(program)
}
pub fn compile_json(request: &str) -> String {
    match compile(request) {
        Ok(program) => {
            let output = program.to_string();
            if output.len() > 33_554_432 || weft_core::json::checked_json(&output).is_err() {
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
