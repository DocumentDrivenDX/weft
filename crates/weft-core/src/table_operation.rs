// SPDX-License-Identifier: MIT OR Apache-2.0
//! Original declared-operation admission. No SQL, connection or authority issuer.
use crate::raw_read::{Binding, Carrier, Table};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;
use std::sync::Arc;
mod schema;
mod validation;
mod batch_scope;
mod values;
mod temporal;
pub(crate) use schema::known_declaration;
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
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OperationModuleRequest {
    pub version: String,
    pub target: String,
    pub binding: Binding,
    pub module: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct OperationBindRequest {
    call: OperationCall,
    #[serde(default)]
    idempotency_key: Option<String>,
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
    pub prepared_literal: Option<Arc<crate::security_literals::PreparedLiteral>>,
}
#[derive(Debug)]
struct OperationSource {
    target: String,
    module: String,
    binding: Arc<Binding>,
    model: Arc<crate::raw_read::AdmittedModel>,
    metadata: Arc<ModuleMetadata>,
}
#[derive(Debug)]
struct ResolvedTable {
    table: Arc<Table>,
    fields: Arc<BTreeMap<String, Arc<OperationField>>>,
    keys: Arc<BTreeMap<String, Vec<Value>>>,
    batch_insert_scope_eligible: bool,
}
#[derive(Debug)]
struct ModuleMetadata {
    module_index: usize,
    commands: BTreeMap<String, usize>,
    tables: BTreeMap<String, usize>,
    elements: BTreeMap<String, usize>,
    resolved_tables: BTreeMap<String, Arc<ResolvedTable>>,
    parameter_fields: Arc<BTreeMap<String, Arc<OperationField>>>,
    related_tables: Arc<BTreeMap<String, Arc<Table>>>,
    related_fields: Arc<BTreeMap<String, BTreeSet<String>>>,
}
#[derive(Debug)]
pub struct PreparedOperationModule {
    source: Arc<OperationSource>,
}
#[derive(Debug)]
pub struct PreparedOperation {
    identity: OperationIdentity,
    static_operation: Arc<StaticOperation>,
}
#[derive(Debug)]
pub struct OperationIdentity {
    pub version: String,
    pub target: String,
    pub binding: Arc<Binding>,
    pub module: String,
    pub operation: String,
}
#[derive(Debug)]
pub struct StaticOperation {
    declared_max_rows: u64,
    declaration: Value,
    declaration_digest: String,
    table: Arc<Table>,
    fields: Arc<BTreeMap<String, Arc<OperationField>>>,
    source_table: Option<Arc<Table>>,
    source_fields: Arc<BTreeMap<String, Arc<OperationField>>>,
    related_tables: Arc<BTreeMap<String, Arc<Table>>>,
    related_fields: Arc<BTreeMap<String, BTreeSet<String>>>,
    parameter_fields: Arc<BTreeMap<String, Arc<OperationField>>>,
    keys: Arc<BTreeMap<String, Vec<Value>>>,
    obligations: Value,
    batch_insert_scope_eligible: bool,
    cursor_fields: Vec<OperationField>,
}
#[derive(Debug)]
pub struct AdmittedOperation {
    identity: OperationIdentity,
    static_operation: Arc<StaticOperation>,
    call: OperationCall,
    idempotency_key: Option<String>,
    max_rows: u64,
}
impl PreparedOperationModule {
    pub fn prepare_operation(&self, operation: &str) -> Result<PreparedOperation> {
        prepare_static_operation(Arc::clone(&self.source), operation)
    }
    pub fn prepare_query_module(&self) -> Result<crate::table_query::AdmittedQueryModule> {
        crate::table_query::prepare_module_shared(self.source.target.clone(), self.source.module.clone(), Arc::clone(&self.source.binding), Arc::clone(&self.source.model))
    }
}
impl PreparedOperation {
    pub fn bind(&self, call: OperationCall, idempotency_key: Option<String>) -> Result<AdmittedOperation> {
        let declaration_max = self.static_operation.declared_max_rows;
        let max_rows = match (&call, self.static_operation.declaration["body"]["kind"].as_str(), self.static_operation.declaration["body"]["result"].as_str()) {
            (OperationCall::Single { size: Some(size), .. }, Some("read"), Some("many" | "page")) if *size > 0 && *size <= declaration_max => *size,
            (OperationCall::Single { size: Some(_), .. }, ..) => return Err("requested read size exceeds declared result bounds"),
            _ => declaration_max,
        };
        let bound = AdmittedOperation {
            identity: OperationIdentity { version: "weft.table-operation/0.1".into(), target: self.identity.target.clone(), binding: Arc::clone(&self.identity.binding), module: self.identity.module.clone(), operation: self.identity.operation.clone() },
            static_operation: Arc::clone(&self.static_operation),
            call,
            idempotency_key,
            max_rows,
        };
        validation::validate_call(&bound)?;
        Ok(bound)
    }
    pub fn bind_json(&self, raw: &str) -> Result<AdmittedOperation> {
        if raw.len() > 4_194_304 { return Err("operation invocation capacity exceeded"); }
        let request: OperationBindRequest = serde_json::from_value(crate::json::checked_json(raw).map_err(|_| "operation invocation JSON invalid or duplicate")?).map_err(|_| "operation invocation shape differs")?;
        self.bind(request.call, request.idempotency_key)
    }
    pub fn declaration(&self) -> &Value { &self.static_operation.declaration }
    pub fn declaration_digest(&self) -> &str { &self.static_operation.declaration_digest }
    pub fn identity(&self) -> &OperationIdentity { &self.identity }
}
impl Deref for AdmittedOperation {
    type Target = StaticOperation;
    fn deref(&self) -> &Self::Target { &self.static_operation }
}
impl AdmittedOperation {
    pub fn declaration_digest(&self) -> &str { &self.static_operation.declaration_digest }
    /// Convert a value already admitted by `bind` / `bind_json`; this is not input admission.
    pub fn native_value(&self, reference: &Value, value: &Value) -> Result<Value> {
        values::native_value(self.field(reference)?, value)
    }
    pub fn native_inputs(&self) -> Result<Vec<BTreeMap<String, Value>>> {
        let rows: Vec<_> = match &self.call {
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
        match &self.call {
            OperationCall::Single {
                after: Some(after), ..
            } => {
                let order = self.declaration["body"]["orderBy"].as_array().ok_or("cursor ordering absent")?;
                Ok(Some(
                    order.iter()
                        .zip(after)
                        .map(|(item, value)| self.native_value(&item["field"], value))
                        .collect::<Result<_>>()?,
                ))
            }
            _ => Ok(None),
        }
    }
    pub fn max_rows(&self) -> u64 {
        self.max_rows
    }
    pub fn request(&self) -> &OperationIdentity {
        &self.identity
    }
    pub fn call(&self) -> &OperationCall { &self.call }
    pub fn idempotency_key(&self) -> Option<&str> { self.idempotency_key.as_deref() }
    pub fn declaration(&self) -> &Value {
        &self.declaration
    }
    pub fn table(&self) -> &Table {
        &self.table
    }
    pub fn field(&self, reference: &Value) -> Result<&OperationField> {
        self.fields
            .get(&reference_id(reference)?)
            .map(Arc::as_ref)
            .ok_or("operation Field is outside its table")
    }
    pub fn source_table(&self) -> Option<&Table> { self.source_table.as_deref() }
    pub fn source_field(&self, reference: &Value) -> Result<&OperationField> {
        self.source_fields.get(&reference_id(reference)?).map(Arc::as_ref).ok_or("Field is outside the declared source table")
    }
    /// Convert an already admitted source value; this does not revalidate authored input.
    pub fn native_source_value(&self, reference: &Value, value: &Value) -> Result<Value> {
        values::native_value(self.source_field(reference)?, value)
    }
    pub fn related_table(&self, id: &str) -> Result<&Table> { self.related_tables.get(id).map(Arc::as_ref).ok_or("related table is outside the module") }
    pub fn related_field(&self, id: &str, reference: &Value) -> Result<&OperationField> { { let key=reference_id(reference)?; if !self.related_fields.get(id).is_some_and(|fields|fields.contains(&key)) { return Err("related Field is outside its table"); } self.parameter_fields.get(&key).map(Arc::as_ref).ok_or("related Field absent") } }
    /// Convert an already admitted related-table value; this does not revalidate authored input.
    pub fn native_related_value(&self, id: &str, reference: &Value, value: &Value) -> Result<Value> { values::native_value(self.related_field(id,reference)?,value) }
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
        self.batch_insert_scope_eligible && matches!(self.call, OperationCall::Batch { .. })
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
pub fn prepare_module_json(raw: &str) -> Result<PreparedOperationModule> {
    if raw.len() > 8_388_608 { return Err("operation module request capacity exceeded"); }
    let request: OperationModuleRequest = serde_json::from_value(
        crate::json::checked_json(raw).map_err(|_| "operation module JSON invalid or duplicate")?,
    ).map_err(|_| "operation module request shape differs")?;
    if request.version != "weft.table-operation-module/0.1" { return Err("operation module version differs"); }
    let source = prepare_source(request.target, request.module, request.binding)?;
    Ok(PreparedOperationModule { source })
}

fn prepare_source(target: String, module_id: String, binding: Binding) -> Result<Arc<OperationSource>> {
    if !matches!(target.as_str(), "postgresql18" | "lakebase18")
        || binding.native_lifecycle.as_ref().is_none_or(|lifecycle| lifecycle.module != module_id)
    { return Err("operation target/module profile differs"); }
    let source_json = crate::json::checked_json(&binding.model_json).map_err(|_| "model JSON refused")?;
    if crate::json::sha256(binding.model_json.as_bytes()) != binding.model_sha256 { return Err("model source pin mismatch"); }
    let module_index = source_json["modules"].as_array().ok_or("model modules absent")?.iter().position(|m| m["id"] == module_id).ok_or("operation module absent")?;
    let module = &source_json["modules"][module_index];
    let first_operation = module["extensions"]["umf.tables"]["commands"].as_array().ok_or("operation command inventory absent")?.iter()
        .find(|command| known_declaration(command)).and_then(|command| command["id"].as_str()).ok_or("operation module has no supported command")?.to_owned();
    let binding = Arc::new(binding);
    let model = Arc::new(crate::raw_read::prepare_operation_document(&binding, source_json, Some(&first_operation))?);
    let metadata = Arc::new(prepare_module_metadata(&binding, &model, module_index)?);
    Ok(Arc::new(OperationSource { target, module: module_id, binding, model, metadata }))
}

fn prepare_module_metadata(binding: &Binding, model: &crate::raw_read::AdmittedModel, module_index: usize) -> Result<ModuleMetadata> {
    let module = &model.document["modules"][module_index];
    let payload = &module["extensions"]["umf.tables"];
    let mut commands = BTreeMap::new();
    for (index, command) in payload["commands"].as_array().ok_or("operation command inventory absent")?.iter().enumerate() {
        if let Some(id) = command["id"].as_str() {
            if commands.insert(id.to_owned(), index).is_some() { return Err("duplicate operation command identity"); }
        }
    }
    let mut tables = BTreeMap::new();
    for (index, table) in payload["tables"].as_array().ok_or("operation table inventory absent")?.iter().enumerate() {
        if let Some(id) = table["id"].as_str() { tables.entry(id.to_owned()).or_insert(index); }
    }
    let mut elements = BTreeMap::new();
    for (index, element) in module["elements"].as_array().ok_or("operation module elements absent")?.iter().enumerate() {
        if let Some(id) = element["id"].as_str() { elements.entry(id.to_owned()).or_insert(index); }
    }

    let mut binding_by_native = BTreeMap::new();
    let mut binding_by_logical = BTreeMap::new();
    for (index, table) in binding.tables.iter().enumerate() {
        binding_by_native.entry(table.native.clone()).or_insert(index);
        binding_by_logical.entry(table.logical.clone()).or_insert(index);
    }
    let mut resolved_tables = BTreeMap::new();
    let mut parameter_fields = BTreeMap::<String, Arc<OperationField>>::new();
    let mut related_tables = BTreeMap::new();
    let mut related_fields = BTreeMap::<String, BTreeSet<String>>::new();
    let batch_scope_by_table = batch_scope::independent_insert_scopes(payload);
    for current_table in payload["tables"].as_array().ok_or("operation table inventory absent")? {
        let table_id = current_table["id"].as_str().ok_or("table identity absent")?.to_owned();
        let native_name = current_table["name"].as_str().ok_or("table name absent")?;
        let bound_index = *binding_by_native.get(native_name).ok_or("parameter table mapping absent")?;
        let bound_table = &binding.tables[bound_index];
        let mut bound_columns = BTreeMap::new();
        for (index, column) in bound_table.columns.iter().enumerate() { bound_columns.entry(column.native.clone()).or_insert(index); }
        let mut fields = BTreeMap::new();
        for column in current_table["columns"].as_array().ok_or("table columns absent")? {
            let field_id = column["field"]["element"].as_str().ok_or("Field identity absent")?;
            let field_index = *elements.get(field_id).ok_or("original operation Field absent")?;
            let field = &module["elements"][field_index];
            let native_column = column["name"].as_str().ok_or("operation column name absent")?;
            let bound_column_index = *bound_columns.get(native_column).ok_or("operation column mapping absent")?;
            let bound = &bound_table.columns[bound_column_index];
            if field["semanticTypes"].as_array().is_some_and(|v| !v.is_empty()) { return Err("operation semantic type interpreter unavailable"); }
            if matches!(bound.carrier, Carrier::Uuid | Carrier::Bits128 | Carrier::Date | Carrier::Timestamp) {
                if field["facets"].as_object().is_some_and(|v| !v.is_empty()) || field["allowedValues"].as_array().is_some_and(|v| !v.is_empty()) {
                    return Err("operation exact carrier qualifiers unsupported");
                }
            }
            let prepared_literal = if matches!(bound.carrier, Carrier::Uuid | Carrier::Bits128 | Carrier::Date | Carrier::Timestamp) {
                None
            } else {
                Some(Arc::new(crate::security_literals::PreparedLiteral::new(field).map_err(|_| "operation Field facets unsupported")?))
            };
            let reference = reference_id(&column["field"])?;
            let mut native_type = column["nativeType"].clone();
            for key in ["length", "precision", "scale"] {
                if let Some(value) = native_type.get(key) {
                    let n = crate::security_literals::unsigned(value).map_err(|_| "native operation capacity invalid")?;
                    native_type[key] = json!(n);
                }
            }
            let resolved = Arc::new(OperationField { reference: column["field"].clone(), native: bound.native.clone(), carrier: bound.carrier.clone(), nullable: bound.nullable, declaration: field.clone(), native_type, prepared_literal });
            fields.insert(reference.clone(), Arc::clone(&resolved));
            if parameter_fields.insert(reference.clone(), Arc::clone(&resolved)).is_some() { return Err("ambiguous original parameter Field binding"); }
            related_fields.entry(table_id.clone()).or_default().insert(reference);
        }
        let record_id = current_table["record"]["element"].as_str().ok_or("table Record identity absent")?;
        let record_index = *elements.get(record_id).ok_or("operation Record absent")?;
        let record = &module["elements"][record_index];
        let logical = vec![module["namespace"].as_str().ok_or("module namespace absent")?.to_owned(), record["name"].as_str().or_else(|| record["id"].as_str()).ok_or("Record name absent")?.to_owned()];
        let logical_index = *binding_by_logical.get(&logical).ok_or("operation native mapping absent")?;
        let table = Arc::new(binding.tables[logical_index].clone());
        let keys = record["keys"].as_array().ok_or("Record keys absent")?.iter().map(|key| {
            Ok((key["id"].as_str().ok_or("Key ID absent")?.to_owned(), key["fields"].as_array().ok_or("Key Fields absent")?.clone()))
        }).collect::<Result<BTreeMap<_, _>>>()?;
        let batch_insert_scope_eligible = batch_scope_by_table.get(&table_id).copied().unwrap_or(false);
        related_tables.insert(table_id.clone(), Arc::clone(&table));
        resolved_tables.insert(table_id, Arc::new(ResolvedTable { table, fields: Arc::new(fields), keys: Arc::new(keys), batch_insert_scope_eligible }));
    }
    Ok(ModuleMetadata { module_index, commands, tables, elements, resolved_tables, parameter_fields: Arc::new(parameter_fields), related_tables: Arc::new(related_tables), related_fields: Arc::new(related_fields) })
}

pub fn admit_json(raw: &str) -> Result<AdmittedOperation> {
    if raw.len() > 4_194_304 { return Err("operation request capacity exceeded"); }
    let request: OperationRequest = serde_json::from_value(crate::json::checked_json(raw).map_err(|_| "operation JSON invalid or duplicate")?)
        .map_err(|_| "operation request shape differs")?;
    if request.version != "weft.table-operation/0.1" { return Err("operation request version differs"); }
    let source = prepare_source(request.target, request.module, request.binding)?;
    let module = PreparedOperationModule { source };
    module.prepare_operation(&request.operation)?.bind(request.call, request.idempotency_key)
}

fn prepare_static_operation(source: Arc<OperationSource>, operation: &str) -> Result<PreparedOperation> {
    if operation.is_empty() || operation.len() > 256 { return Err("operation identity invalid"); }
    let model = &source.model;
    let metadata = &source.metadata;
    let request = OperationIdentity { version: "weft.table-operation/0.1".into(), target: source.target.clone(), binding: Arc::clone(&source.binding), module: source.module.clone(), operation: operation.to_owned() };
    let lifecycle = model
        .lifecycle
        .as_ref()
        .ok_or("native lifecycle context absent")?;
    let module = &model.document["modules"][metadata.module_index];
    let payload = &module["extensions"]["umf.tables"];
    let command_index = *metadata.commands.get(operation).ok_or("operation not declared")?;
    let mut declaration = payload["commands"][command_index].clone();
    // Known integer controls become typed plan integers; original source bytes
    // remain unchanged in the binding. JSONB/JSON consumers must not turn an
    // authored 1e2 bound into a floating-point plan control.
    let max_rows = crate::security_literals::unsigned(&declaration["body"]["maxRows"])
        .map_err(|_| "operation result bound invalid")?;
    declaration["body"]["maxRows"] = json!(max_rows);
    if declaration.get("batch").is_some() {
        let limit = crate::security_literals::unsigned(&declaration["batch"]["maxItems"])
            .map_err(|_| "operation batch bound invalid")?;
        declaration["batch"]["maxItems"] = json!(limit);
    }
    let table_id = declaration["body"]["table"].as_str().ok_or("operation table identity absent")?;
    let table_index = *metadata.tables.get(table_id).ok_or("operation table not declared")?;
    let table_declaration = &payload["tables"][table_index];
    let resolved_table = metadata.resolved_tables.get(table_id).ok_or("operation native mapping absent")?;
    let record_id = table_declaration["record"]["element"].as_str().ok_or("operation Record identity absent")?;
    let record_index = *metadata.elements.get(record_id).ok_or("operation Record absent")?;
    let record = &module["elements"][record_index];
    let path = resolved_table.table.logical.clone();
    let table = Arc::clone(&resolved_table.table);
    let source_table_id = if declaration["body"]["kind"] == "insertFrom" {
        Some(declaration["body"]["source"]["table"].as_str().ok_or("source table identity absent")?)
    } else { None };
    let source_table = source_table_id.map(|id| metadata.resolved_tables.get(id).map(|table| Arc::clone(&table.table)).ok_or("source table is outside the module")).transpose()?;
    let source_fields = source_table_id.map(|id| metadata.resolved_tables.get(id).map(|table| Arc::clone(&table.fields)).ok_or("source table is outside the module")).transpose()?.unwrap_or_else(|| Arc::new(BTreeMap::new()));
    let batch_insert_scope_eligible = declaration["body"]["kind"] == "insert" && resolved_table.batch_insert_scope_eligible;
    let fields = Arc::clone(&resolved_table.fields);
    let related_tables = Arc::clone(&metadata.related_tables);
    let related_fields = Arc::clone(&metadata.related_fields);
    let parameter_fields = Arc::clone(&metadata.parameter_fields);
    let keys = Arc::clone(&resolved_table.keys);
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
        if predicate.get("field").is_some() || predicate.get("matches").is_some() {
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
    fn related_paths(node: &Value, tables: &BTreeMap<String,Arc<Table>>, paths: &mut BTreeSet<Vec<String>>) -> Result<()> {
        if node["op"] == "related" {
            let table=tables.get(node["table"].as_str().ok_or("related table identity absent")?).ok_or("related table is outside the module")?;
            paths.insert(table.logical.clone());related_paths(&node["where"],tables,paths)?;
        }
        if let Some(args)=node["args"].as_array() { for child in args { related_paths(child,tables,paths)?; } }
        if let Some(child)=node.get("arg") { related_paths(child,tables,paths)?; }
        Ok(())
    }
    related_paths(&declaration["body"]["where"],&related_tables,&mut paths)?;
    related_paths(&declaration["body"]["source"]["where"],&related_tables,&mut paths)?;
    let mut obligations = lifecycle.obligation(&paths)?;
    for item in obligations["tables"].as_array_mut().unwrap() {
        item.as_object_mut().unwrap().remove("command");
        item["commands"] = if item["id"] == table_declaration["id"] { json!(commands) } else { json!(["select"]) };
    }
    let cursor_fields = if declaration["body"]["result"] == "page" {
        declaration["body"]["orderBy"].as_array().into_iter().flatten()
            .filter_map(|order| reference_id(&order["field"]).ok().and_then(|key| fields.get(&key).map(|field| (**field).clone()))).collect()
    } else { vec![] };
    let static_operation = Arc::new(StaticOperation {
        declared_max_rows: crate::security_literals::unsigned(&declaration["body"]["maxRows"])
            .map_err(|_| "operation result bound invalid")?,
        declaration_digest: crate::json::sha256(crate::mutation::canonical(&declaration).as_bytes()),
        declaration,
        table,
        fields,
        source_table,
        source_fields,
        related_tables,
        related_fields,
        parameter_fields,
        keys,
        obligations,
        batch_insert_scope_eligible,
        cursor_fields,
    });
    let checked = validation::validate_static(AdmittedOperation {
        identity: request,
        static_operation,
        call: OperationCall::Single { inputs: BTreeMap::new(), size: None, after: None },
        idempotency_key: None,
        max_rows,
    }, record, table_declaration)?;
    Ok(PreparedOperation { identity: checked.identity, static_operation: checked.static_operation })
}

pub(crate) fn native_seed_value(field:&OperationField,value:&Value)->Result<Value>{
    values::literal(field,value)?;
    values::native_value(field,value)
}
