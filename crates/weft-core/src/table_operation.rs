// SPDX-License-Identifier: MIT OR Apache-2.0
//! Original declared-operation admission. No SQL, connection or authority issuer.
use crate::raw_read::{Binding, Carrier, Table};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
mod schema;
mod validation;
mod batch_scope;
mod values;
mod temporal;
pub(crate) use schema::known_declaration;
use validation::validate;
use values::{domain, literal};
type Result<T> = std::result::Result<T, &'static str>;
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OperationRequest {
    pub version: String,
    pub target: String,
    pub binding: Binding,
    pub module: String,
    pub operation: String,
    pub call: OperationCall,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum OperationCall {
    Single {
        inputs: BTreeMap<String, Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        size: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        after: Option<Vec<Value>>,
    },
    Batch {
        items: Vec<BTreeMap<String, Value>>,
    },
}
#[derive(Debug, Clone)]
pub struct OperationField {
    pub reference: Value,
    pub native: String,
    pub carrier: Carrier,
    pub nullable: bool,
    pub declaration: Value,
    pub native_type: Value,
}
#[derive(Debug)]
pub struct AdmittedOperation {
    request: OperationRequest,
    max_rows: u64,
    declaration: Value,
    table: Table,
    fields: BTreeMap<String, OperationField>,
    source_table: Option<Table>,
    source_fields: BTreeMap<String, OperationField>,
    parameter_fields: BTreeMap<String, OperationField>,
    keys: BTreeMap<String, Vec<Value>>,
    obligations: Value,
    batch_insert_scope: bool,
}
impl AdmittedOperation {
    pub fn native_value(&self, reference: &Value, value: &Value) -> Result<Value> {
        values::native_value(self.field(reference)?, value)
    }
    pub fn native_inputs(&self) -> Result<Vec<BTreeMap<String, Value>>> {
        let rows: Vec<_> = match &self.request.call {
            OperationCall::Single { inputs, .. } => vec![inputs],
            OperationCall::Batch { items } => items.iter().collect(),
        };
        let mut budget = 8_388_608usize;
        let mut result = vec![];
        for row in rows {
            let mut bound = BTreeMap::new();
            for parameter in self.declaration["parameters"].as_array().unwrap() {
                let id = parameter["id"].as_str().unwrap();
                let field = self
                    .parameter_fields
                    .get(&reference_id(&parameter["field"])?)
                    .ok_or("parameter native Field absent")?;
                let value =
                    values::native_value(field, row.get(id).ok_or("parameter value absent")?)?;
                budget = budget
                    .checked_sub(value.to_string().len() + id.len() + 8)
                    .ok_or("native binding expansion capacity exceeded")?;
                bound.insert(id.to_owned(), value);
            }
            result.push(bound);
        }
        Ok(result)
    }
    pub fn native_after(&self) -> Result<Option<Vec<Value>>> {
        match &self.request.call {
            OperationCall::Single {
                after: Some(after), ..
            } => {
                let key = self.key(
                    self.declaration["body"]["cursorKey"]
                        .as_str()
                        .ok_or("cursor Key absent")?,
                )?;
                Ok(Some(
                    key.iter()
                        .zip(after)
                        .map(|(field, value)| self.native_value(field, value))
                        .collect::<Result<_>>()?,
                ))
            }
            _ => Ok(None),
        }
    }
    pub fn max_rows(&self) -> u64 {
        self.max_rows
    }
    pub fn request(&self) -> &OperationRequest {
        &self.request
    }
    pub fn declaration(&self) -> &Value {
        &self.declaration
    }
    pub fn table(&self) -> &Table {
        &self.table
    }
    pub fn field(&self, reference: &Value) -> Result<&OperationField> {
        self.fields
            .get(&reference_id(reference)?)
            .ok_or("operation Field is outside its table")
    }
    pub fn source_table(&self) -> Option<&Table> { self.source_table.as_ref() }
    pub fn source_field(&self, reference: &Value) -> Result<&OperationField> {
        self.source_fields.get(&reference_id(reference)?).ok_or("Field is outside the declared source table")
    }
    pub fn native_source_value(&self, reference: &Value, value: &Value) -> Result<Value> {
        values::native_value(self.source_field(reference)?, value)
    }
    pub fn key(&self, id: &str) -> Result<&[Value]> {
        self.keys
            .get(id)
            .map(Vec::as_slice)
            .ok_or("original operation Key absent")
    }
    pub fn obligations(&self) -> &Value {
        &self.obligations
    }
    /// Whether an original-source batch insert is independent of its own prior rows.
    pub fn batch_insert_scope(&self) -> bool {
        self.batch_insert_scope
    }
}
fn reference_id(reference: &Value) -> Result<String> {
    Ok(json!([
        reference["module"].as_str().ok_or("Field module absent")?,
        reference["element"]
            .as_str()
            .ok_or("Field identity absent")?
    ])
    .to_string())
}
pub fn admit_json(raw: &str) -> Result<AdmittedOperation> {
    if raw.len() > 4_194_304 {
        return Err("operation request capacity exceeded");
    }
    let request: OperationRequest = serde_json::from_value(
        crate::json::checked_json(raw).map_err(|_| "operation JSON invalid or duplicate")?,
    )
    .map_err(|_| "operation request shape differs")?;
    if request.version != "weft.table-operation/0.1"
        || !matches!(request.target.as_str(), "postgresql18" | "lakebase18")
        || request
            .binding
            .native_lifecycle
            .as_ref()
            .is_none_or(|c| c.module != request.module)
    {
        return Err("operation target/module profile differs");
    }
    let model =
        crate::raw_read::prepare_operation_model(&request.binding, Some(&request.operation))?;
    let lifecycle = model
        .lifecycle
        .as_ref()
        .ok_or("native lifecycle context absent")?;
    let module = model.document["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == request.module)
        .ok_or("operation module absent")?;
    let payload = &module["extensions"]["umf.tables"];
    let mut declaration = payload["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == request.operation)
        .ok_or("operation not declared")?
        .clone();
    // Known integer controls become typed plan integers; original source bytes
    // remain unchanged in the binding. JSONB/JSON consumers must not turn an
    // authored 1e2 bound into a floating-point plan control.
    let max_rows = crate::security_literals::unsigned(&declaration["body"]["maxRows"])
        .map_err(|_| "operation result bound invalid")?;
    declaration["body"]["maxRows"] = json!(max_rows);
    let max_rows = match &request.call {
        OperationCall::Single {
            size: Some(size), ..
        } => {
            if declaration["body"]["kind"] != "read"
                || !matches!(
                    declaration["body"]["result"].as_str(),
                    Some("many" | "page")
                )
                || *size == 0
                || *size > max_rows
            {
                return Err("requested read size exceeds declared result bounds");
            }
            *size
        }
        _ => max_rows,
    };
    if declaration.get("batch").is_some() {
        let limit = crate::security_literals::unsigned(&declaration["batch"]["maxItems"])
            .map_err(|_| "operation batch bound invalid")?;
        declaration["batch"]["maxItems"] = json!(limit);
    }
    let table_declaration = payload["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == declaration["body"]["table"])
        .ok_or("operation table not declared")?;
    let record = module["elements"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == table_declaration["record"]["element"])
        .ok_or("operation Record absent")?;
    let path = vec![
        module["namespace"].as_str().unwrap().to_owned(),
        record["name"]
            .as_str()
            .or_else(|| record["id"].as_str())
            .unwrap()
            .to_owned(),
    ];
    let table = request
        .binding
        .tables
        .iter()
        .find(|t| t.logical == path)
        .ok_or("operation native mapping absent")?
        .clone();
    let source_declaration = if declaration["body"]["kind"] == "insertFrom" {
        Some(payload["tables"].as_array().unwrap().iter().find(|t| t["id"] == declaration["body"]["source"]["table"]).ok_or("source table is outside the module")?)
    } else { None };
    let source_table = source_declaration.map(|source| request.binding.tables.iter().find(|t| t.native == source["name"].as_str().unwrap()).cloned().ok_or("source native mapping absent")).transpose()?;
    let mut source_fields = BTreeMap::new();
    let batch_insert_scope = matches!(&request.call, OperationCall::Batch { .. })
        && declaration["body"]["kind"] == "insert"
        && batch_scope::independent_insert_scope(
            payload,
            &table_declaration["record"],
            &table_declaration["id"],
        );
    let mut fields = BTreeMap::new();
    let mut parameter_fields = BTreeMap::new();
    for current_table in payload["tables"].as_array().unwrap() {
        let bound_table = request
            .binding
            .tables
            .iter()
            .find(|t| t.native == current_table["name"].as_str().unwrap())
            .ok_or("parameter table mapping absent")?;
        for column in current_table["columns"].as_array().unwrap() {
            let field = module["elements"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["id"] == column["field"]["element"])
                .ok_or("original operation Field absent")?;
            let bound = bound_table
                .columns
                .iter()
                .find(|c| c.native == column["name"].as_str().unwrap())
                .ok_or("operation column mapping absent")?;
            if field["semanticTypes"]
                .as_array()
                .is_some_and(|v| !v.is_empty())
            {
                return Err("operation semantic type interpreter unavailable");
            }
            if matches!(bound.carrier, Carrier::Uuid | Carrier::Bits128 | Carrier::Date | Carrier::Timestamp) {
                if field["facets"].as_object().is_some_and(|v| !v.is_empty())
                    || field["allowedValues"]
                        .as_array()
                        .is_some_and(|v| !v.is_empty())
                {
                    return Err("operation exact carrier qualifiers unsupported");
                }
            } else {
                crate::security_literals::validate_field(field)
                    .map_err(|_| "operation Field facets unsupported")?;
            }
            let reference = reference_id(&column["field"])?;
            let mut native_type = column["nativeType"].clone();
            for key in ["length", "precision", "scale"] {
                if let Some(value) = native_type.get(key) {
                    let n = crate::security_literals::unsigned(value)
                        .map_err(|_| "native operation capacity invalid")?;
                    native_type[key] = json!(n);
                }
            }
            let resolved = OperationField {
                reference: column["field"].clone(),
                native: bound.native.clone(),
                carrier: bound.carrier.clone(),
                nullable: bound.nullable,
                declaration: field.clone(),
                native_type,
            };
            if current_table["id"] == table_declaration["id"] {
                fields.insert(reference.clone(), resolved.clone());
            }
            if source_declaration.is_some_and(|source| source["id"] == current_table["id"]) { source_fields.insert(reference.clone(), resolved.clone()); }
            if parameter_fields.insert(reference, resolved).is_some() {
                return Err("ambiguous original parameter Field binding");
            }
        }
    }
    let kind = declaration["body"]["kind"].as_str().unwrap();
    let mut commands = match kind {
        "read" => vec!["select"],
        "insert" | "insertFrom" => vec!["insert"],
        "update" => vec!["update"],
        "delete" => vec!["delete"],
        "upsert" => {
            if declaration["body"]["onConflict"] == "update" {
                vec!["insert", "update", "select"]
            } else {
                vec!["insert", "select"]
            }
        }
        _ => return Err("operation kind unsupported"),
    };
    fn reads_column(predicate: &Value) -> bool {
        if predicate.get("field").is_some() {
            return true;
        }
        if let Some(args) = predicate["args"].as_array() {
            return args.iter().any(reads_column);
        }
        predicate.get("arg").is_some_and(reads_column)
    }
    if declaration["body"]["returning"]
        .as_array()
        .is_some_and(|v| !v.is_empty())
        || declaration["body"].get("where").is_some_and(reads_column)
    {
        if !commands.contains(&"select") {
            commands.push("select");
        }
    }
    let mut paths = BTreeSet::from([path.clone()]);
    if let Some(source) = &source_table {
        paths.insert(source.logical.clone());
        if source.logical == path && !commands.contains(&"select") { commands.push("select"); }
    }
    let mut obligations = lifecycle.obligation(&paths)?;
    for item in obligations["tables"].as_array_mut().unwrap() {
        item.as_object_mut().unwrap().remove("command");
        item["commands"] = if item["id"] == table_declaration["id"] { json!(commands) } else { json!(["select"]) };
    }
    let admitted = AdmittedOperation {
        request,
        max_rows,
        declaration,
        table,
        fields,
        source_table,
        source_fields,
        parameter_fields,
        keys: record["keys"]
            .as_array()
            .unwrap()
            .iter()
            .map(|k| {
                Ok((
                    k["id"].as_str().ok_or("Key ID absent")?.to_owned(),
                    k["fields"].as_array().ok_or("Key Fields absent")?.clone(),
                ))
            })
            .collect::<Result<_>>()?,
        obligations,
        batch_insert_scope,
    };
    validate(admitted, record, table_declaration)
}
