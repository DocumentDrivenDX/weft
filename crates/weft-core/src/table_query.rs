// SPDX-License-Identifier: MIT OR Apache-2.0
//! Source-bound declarations for fixed, typed PostgreSQL analytics queries.
//! Query text belongs to the pinned UMF operation; invocation values never carry SQL.
use crate::raw_read::Binding;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlparser::{
    ast::{
        Expr, Query, Select, SelectItem, SetExpr, Statement, TableFactor, Value as SqlValue, ValueWithSpan,
        Visit, Visitor,
    },
    dialect::PostgreSqlDialect,
    parser::Parser,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    ops::ControlFlow,
    sync::Arc,
};

type Result<T> = std::result::Result<T, &'static str>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PrepareRequest {
    pub version: String,
    pub target: String,
    pub binding: Binding,
    pub module: String,
    pub operation: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PrepareModuleRequest {
    pub version: String,
    pub target: String,
    pub binding: Binding,
    pub module: String,
}

/// Source and validated UMF/table indexes shared by every lazily selected query
/// in one private reader. No database work or native authority is retained here.
pub struct AdmittedQueryModule {
    target: String,
    module: String,
    // Retain the trusted prepared-module admission profile with its owned
    // source. Standalone query-module preparation always uses the legacy path.
    _logical_defaults020: bool,
    binding: Arc<Binding>,
    model: Arc<crate::raw_read::AdmittedModel>,
    indexes: Arc<ModuleIndexes>,
}

impl AdmittedQueryModule {
    pub fn prepare(&self, operation: &str) -> Result<AdmittedQuery> {
        if operation.is_empty() || operation.len() > 256 {
            return Err("query operation identity invalid");
        }
        admit_request(
            QueryIdentity { target: self.target.clone(), module: self.module.clone(), operation: operation.to_owned() },
            Arc::clone(&self.binding),
            Arc::clone(&self.model),
            Arc::clone(&self.indexes),
        )
    }
}

pub fn prepare_module_json(raw: &str) -> Result<AdmittedQueryModule> {
    if raw.len() > 8_388_608 { return Err("query module request capacity exceeded"); }
    let request: PrepareModuleRequest = serde_json::from_value(
        crate::json::checked_json(raw).map_err(|_| "query module JSON invalid or duplicate")?,
    ).map_err(|_| "query module request shape differs")?;
    if request.version != "weft.table-query-module/0.1"
        || !matches!(request.target.as_str(), "postgresql18" | "lakebase18")
        || request.binding.native_lifecycle.as_ref().is_none_or(|l| l.module != request.module)
    { return Err("query module target/source profile differs"); }
    let source = crate::json::checked_json(&request.binding.model_json).map_err(|_| "model JSON refused")?;
    if crate::json::sha256(request.binding.model_json.as_bytes()) != request.binding.model_sha256 {
        return Err("model source pin mismatch");
    }
    let module = source["modules"].as_array().ok_or("model modules absent")?.iter()
        .find(|m| m["id"] == request.module).ok_or("query source module absent")?;
    let first_query = module["extensions"]["umf.tables"]["commands"].as_array()
        .ok_or("query command source absent")?.iter()
        .find(|c| c["version"] == "umf.table-operation/0.1" && c["body"]["kind"] == "query")
        .and_then(|c| c["id"].as_str()).ok_or("query source module has no fixed query operation")?.to_owned();
    // Parse/validate the source model once. Selected query ASTs are compiled lazily.
    let model = Arc::new(crate::raw_read::prepare_operation_document(&request.binding, source, Some(&first_query))?);
    prepare_module_shared(request.target, request.module, Arc::new(request.binding), model, false)
}

pub(crate) fn prepare_module_shared(target: String, module: String, binding: Arc<Binding>, model: Arc<crate::raw_read::AdmittedModel>, logical_defaults020: bool) -> Result<AdmittedQueryModule> {
    if !matches!(target.as_str(), "postgresql18" | "lakebase18") { return Err("query module target differs"); }
    let module_value = model.document["modules"].as_array().and_then(|modules|modules.iter().find(|m|m["id"]==module)).ok_or("query source module absent")?;
    if !module_value["extensions"]["umf.tables"]["commands"].as_array().is_some_and(|commands|commands.iter().any(|command|command["body"]["kind"]=="query")) { return Err("query source module has no fixed query operation"); }
    let indexes=Arc::new(ModuleIndexes::new(&model,&binding,&module)?);
    Ok(AdmittedQueryModule { target, module, _logical_defaults020: logical_defaults020, binding, model, indexes })
}

struct ModuleIndexes {
    module_position: usize,
    elements: BTreeMap<String, usize>,
    commands: BTreeMap<String, usize>,
    tables: BTreeMap<String, usize>,
    record_paths: BTreeMap<(String, String), Vec<String>>,
    bound_tables: BTreeMap<Vec<String>, usize>,
    namespace: String,
}
impl ModuleIndexes {
    fn new(model:&crate::raw_read::AdmittedModel,binding:&Binding,module_id:&str)->Result<Self>{
        let modules=model.document["modules"].as_array().ok_or("query source modules absent")?;
        let module_position=modules.iter().position(|m|m["id"]==module_id).ok_or("query source module absent")?;
        let module=&modules[module_position];
        let namespace=module["namespace"].as_str().ok_or("query module namespace absent")?.to_owned();
        let elements=module["elements"].as_array().ok_or("query source elements absent")?.iter().enumerate()
            .filter_map(|(i,e)|e["id"].as_str().map(|id|(id.to_owned(),i))).collect::<BTreeMap<_,_>>();
        let commands=module["extensions"]["umf.tables"]["commands"].as_array().ok_or("query command source absent")?.iter().enumerate()
            .filter_map(|(i,c)|c["id"].as_str().map(|id|(id.to_owned(),i))).collect::<BTreeMap<_,_>>();
        let mut tables=BTreeMap::new();
        for (i,t) in module["extensions"]["umf.tables"]["tables"].as_array().ok_or("query table source absent")?.iter().enumerate(){
            let id=t["record"]["element"].as_str().ok_or("query table Record identity absent")?;
            let pos=*elements.get(id).ok_or("query table Record missing")?;
            let record=&module["elements"][pos];
            let record_name=record["name"].as_str().or_else(||record["id"].as_str()).ok_or("query Record name absent")?;
            for name in [t["id"].as_str(),t["name"].as_str(),Some(record_name)].into_iter().flatten(){
                for candidate in [name.to_owned(),name.to_ascii_lowercase()] {
                    if tables.insert(candidate,i).is_some_and(|old|old!=i){return Err("ambiguous query table name")}
                }
            }
        }
        let mut record_paths=BTreeMap::new();
        for record in model.records.values() {
            let Some(name)=record.value["name"].as_str().or_else(||record.value["id"].as_str()) else { continue; };
            record_paths.entry((record.identity.module.clone(),record.identity.element.clone()))
                .or_insert_with(||vec![namespace.clone(),name.to_owned()]);
        }
        let mut bound_tables=BTreeMap::new();
        for (index,table) in binding.tables.iter().enumerate() {
            bound_tables.entry(table.logical.clone()).or_insert(index);
        }
        Ok(Self{module_position,elements,commands,tables,record_paths,bound_tables,namespace})
    }
}

#[derive(Debug, Clone)]
pub struct QueryField {
    pub id: String,
    pub field: Value,
    pub declaration: Value,
    pub carrier: String,
    pub nullable: bool,
    prepared_literal: Option<Arc<crate::security_literals::PreparedLiteral>>,
    float_allowed_values: Option<BTreeSet<u64>>,
}

#[derive(Debug)]
pub struct AdmittedQuery {
    request: QueryIdentity,
    binding: Arc<Binding>,
    _model: Arc<crate::raw_read::AdmittedModel>,
    declaration: Value,
    query: Query,
    parameters: BTreeMap<String, QueryField>,
    outputs: Vec<QueryField>,
    parameter_order: Vec<String>,
    tables: BTreeMap<String, String>,
    bound_table_positions: BTreeMap<String, usize>,
    cte_names: BTreeSet<String>,
    max_rows: u64,
    obligations: Value,
    original_source_digest: String,
}
#[derive(Debug)]
pub struct QueryIdentity { pub target: String, pub module: String, pub operation: String }

impl AdmittedQuery {
    pub fn request(&self) -> &QueryIdentity { &self.request }
    pub fn binding(&self) -> &Binding { &self.binding }
    pub fn declaration(&self) -> &Value {
        &self.declaration
    }
    pub fn query(&self) -> &Query {
        &self.query
    }
    pub fn parameters(&self) -> &BTreeMap<String, QueryField> {
        &self.parameters
    }
    pub fn outputs(&self) -> &[QueryField] {
        &self.outputs
    }
    pub fn parameter_order(&self) -> &[String] {
        &self.parameter_order
    }
    pub fn tables(&self) -> &BTreeMap<String, String> {
        &self.tables
    }
    pub fn bound_table(&self, relation: &str) -> Option<&crate::raw_read::Table> {
        self.bound_table_positions.get(relation).and_then(|index| self.binding.tables.get(*index))
    }
    pub fn cte_names(&self) -> &BTreeSet<String> {
        &self.cte_names
    }
    pub fn max_rows(&self) -> u64 {
        self.max_rows
    }
    pub fn obligations(&self) -> &Value {
        &self.obligations
    }
    pub fn original_source_digest(&self) -> &str {
        &self.original_source_digest
    }

    /// Validate the exact typed input map. Native conversion is performed by the backend codec.
    pub fn bind_inputs(&self, inputs: &BTreeMap<String, Value>) -> Result<Vec<Value>> {
        if inputs.len() != self.parameters.len()
            || inputs.keys().any(|k| !self.parameters.contains_key(k))
        {
            return Err("query input IDs differ from the source declaration");
        }
        let mut normalized = Vec::with_capacity(self.parameters.len());
        for (id, field) in &self.parameters {
            let value = inputs.get(id).ok_or("query input absent")?;
            check_value(field, value)?;
            normalized.push(native_value(field, value)?);
        }
        Ok(normalized)
    }
}

#[derive(Default)]
pub struct CteScopes {
    stack: Vec<Option<Arc<CteScope>>>,
    pending: HashMap<usize, Option<Arc<CteScope>>>,
}
struct CteScope {
    names: Arc<BTreeMap<String, usize>>,
    visible_prefix: usize,
    parent: Option<Arc<CteScope>>,
}
impl CteScope {
    fn contains(&self, name:&str) -> bool {
        if self.names.get(name).is_some_and(|index|*index<self.visible_prefix) { return true; }
        self.parent.as_ref().is_some_and(|parent|parent.contains(name))
    }
}
impl CteScopes {
    pub fn enter(&mut self, query:&Query) {
        let pointer=query as *const Query as usize;
        let parent=self.pending.remove(&pointer).or_else(||self.stack.last().cloned()).flatten();
        let scope=if let Some(with)=&query.with {
            let names=Arc::new(with.cte_tables.iter().enumerate().map(|(index,cte)|{
                let name=&cte.alias.name;
                (if name.quote_style.is_some(){name.value.clone()}else{name.value.to_ascii_lowercase()},index)
            }).collect::<BTreeMap<_,_>>());
            for (index,cte) in with.cte_tables.iter().enumerate() {
                self.pending.insert(&*cte.query as *const Query as usize,Some(Arc::new(CteScope{
                    names:Arc::clone(&names),visible_prefix:index,parent:parent.clone(),
                })));
            }
            Some(Arc::new(CteScope{visible_prefix:with.cte_tables.len(),names,parent}))
        } else { parent };
        self.stack.push(scope);
    }
    pub fn leave(&mut self) { self.stack.pop(); }
    pub fn contains(&self,name:&str)->bool {
        self.stack.last().and_then(Option::as_ref).is_some_and(|scope|scope.contains(name))
    }
}

struct ScanInventory {
    relations: Vec<(Vec<String>, Option<String>, bool)>,
    parameters: BTreeSet<String>,
    parameter_order: Vec<String>,
    cte_names: BTreeSet<String>,
    cte_scopes: CteScopes,
    unsupported: bool,
}
impl Visitor for ScanInventory {
    type Break = ();
    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<Self::Break> {
        if query.fetch.is_some() || !query.locks.is_empty() || query.for_clause.is_some()
            || query.with.as_ref().is_some_and(|w|w.recursive)
            || !matches!(query.body.as_ref(),SetExpr::Select(_))
        { self.unsupported=true; }
        self.cte_scopes.enter(query);
        if let Some(with) = &query.with {
            for cte in &with.cte_tables {
                let name = &cte.alias.name;
                let normalized=if name.quote_style.is_some() {
                    name.value.clone()
                } else {
                    name.value.to_ascii_lowercase()
                };
                self.cte_names.insert(normalized.clone());
            }
        }
        ControlFlow::Continue(())
    }
    fn post_visit_query(&mut self, _query: &Query) -> ControlFlow<Self::Break> { self.cte_scopes.leave(); ControlFlow::Continue(()) }
    fn pre_visit_select(&mut self, select:&Select)->ControlFlow<Self::Break> {
        if select.into.is_some() { self.unsupported=true; }
        ControlFlow::Continue(())
    }
    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<Self::Break> {
        if let TableFactor::Table {
            name,
            alias,
            args,
            with_hints,
            version,
            with_ordinality,
            partitions,
            json_path,
            sample,
            index_hints,
        } = factor
        {
            if args.is_some() || !with_hints.is_empty() || version.is_some() || *with_ordinality
                || !partitions.is_empty() || json_path.is_some() || sample.is_some() || !index_hints.is_empty()
            { self.unsupported=true; }
            let parts = name
                .0
                .iter()
                .map(|p| match p {
                    sqlparser::ast::ObjectNamePart::Identifier(i) => {
                        Ok(if i.quote_style.is_some() {
                            i.value.clone()
                        } else {
                            i.value.to_ascii_lowercase()
                        })
                    }
                    _ => Err(()),
                })
                .collect::<std::result::Result<Vec<_>, _>>();
            if let Ok(parts) = parts {
                let alias = alias.as_ref().map(|a| {
                    if a.name.quote_style.is_some() {
                        a.name.value.clone()
                    } else {
                        a.name.value.to_ascii_lowercase()
                    }
                });
                let is_cte=parts.len()==1&&self.cte_scopes.contains(&parts[0]);
                self.relations.push((parts, alias, is_cte));
            }
        } else if !matches!(factor,TableFactor::Derived{..}|TableFactor::NestedJoin{..}) {
            self.unsupported=true;
        }
        ControlFlow::Continue(())
    }
    fn pre_visit_value(&mut self, value: &ValueWithSpan) -> ControlFlow<Self::Break> {
        if let SqlValue::Placeholder(value) = &value.value {
            if let Some(name) = value.strip_prefix(':') {
                if self.parameters.insert(name.to_owned()) {
                    self.parameter_order.push(name.to_owned());
                }
            }
        }
        ControlFlow::Continue(())
    }
}

fn field_profile<'a>(doc: &'a Value, module_id: &str, indexes: &ModuleIndexes, reference: &Value) -> Result<&'a Value> {
    if reference["module"] != module_id {
        return Err("query Field must belong to its owning module");
    }
    let modules = doc["modules"].as_array().ok_or("query source modules absent")?;
    let module = modules.get(indexes.module_position).filter(|m|m["id"]==module_id).ok_or("query source module absent")?;
    let element_id=reference["element"].as_str().ok_or("query Field identity absent")?;
    let position=*indexes.elements.get(element_id).ok_or("query Field identity absent")?;
    let found = module["elements"].as_array().and_then(|e|e.get(position)).ok_or("query Field identity absent")?;
    const FIELD_KEYS: &[&str] = &[
        "id", "kind", "name", "description", "scalarType", "extensions", "references",
        "nullability", "cardinality", "itemType", "facets", "members", "keys", "title",
        "aliases", "examples", "allowedValues", "default", "semanticTypes",
    ];
    if found.as_object().is_none_or(|object| object.keys().any(|key| !FIELD_KEYS.contains(&key.as_str())))
        || found.get("itemType").is_some()
        || found.get("members").is_some()
        || found.get("keys").is_some()
        || found.get("references").is_some_and(|references| !references.as_array().is_some_and(Vec::is_empty))
    {
        return Err("query Field has unknown or structural meaning outside the scalar source profile");
    }
    if found["kind"] != "field"
        || found["cardinality"] != "one"
        || !matches!(
            found["nullability"].as_str(),
            Some("required" | "absent-allowed")
        )
        || found
            .get("semanticTypes")
            .and_then(Value::as_array)
            .is_some_and(|v| !v.is_empty())
        || found
            .get("extensions")
            .and_then(Value::as_object)
            .is_none_or(|v| !v.is_empty())
    {
        return Err("query Field is outside the scalar source profile");
    }
    let scalar = found["scalarType"]
        .as_str()
        .ok_or("query scalar type absent")?;
    if !matches!(
        scalar,
        "boolean"
            | "integer"
            | "decimal"
            | "float"
            | "string"
            | "date"
            | "time"
            | "timestamp"
            | "uuid"
            | "bits128"
    ) {
        return Err("query scalar family unsupported");
    }
    Ok(found)
}

/// Query inputs retain the shared, strict literal/refinement admission rules.
fn field<'a>(doc: &'a Value, module_id: &str, indexes: &ModuleIndexes, reference: &Value) -> Result<&'a Value> {
    let found = field_profile(doc, module_id, indexes, reference)?;
    if found["scalarType"] == "float" {
        let facets = found.get("facets").cloned().unwrap_or_else(|| json!({}));
        if !facets.as_object().is_some_and(|m| m.is_empty()) {
            return Err("query Float facets unsupported");
        }
    } else {
        crate::security_literals::validate_field(found)
            .map_err(|_| "query Field facets unsupported")?;
    }
    Ok(found)
}

/// Query output Fields are nullable DTO declarations, not input literals.
/// They admit supported core scalar families without asking the narrower
/// security-ontology literal validator to assign semantics to the result type.
fn output_field<'a>(doc: &'a Value, module_id: &str, indexes: &ModuleIndexes, reference: &Value) -> Result<&'a Value> {
    let found = field_profile(doc, module_id, indexes, reference)?;
    if found.get("facets").is_some_and(|v| v.as_object().is_none_or(|m| !m.is_empty()))
        || found.get("allowedValues").is_some()
    {
        return Err("query outputs require unrefined DTO Fields");
    }
    Ok(found)
}

fn field_key(reference: &Value) -> Result<String> {
    Ok(json!([
        reference["module"]
            .as_str()
            .ok_or("query Field module absent")?,
        reference["element"]
            .as_str()
            .ok_or("query Field element absent")?
    ])
    .to_string())
}
fn query_field(doc: &Value, module: &str, indexes:&ModuleIndexes, id: String, reference: &Value) -> Result<QueryField> {
    let declaration = field(doc, module, indexes, reference)?.clone();
    let carrier=declaration["scalarType"].as_str().ok_or("query Field type absent")?.to_owned();
    let prepared_literal=if carrier=="float" {None}else{Some(Arc::new(crate::security_literals::PreparedLiteral::from_validated(&declaration).map_err(|_|"query Field facets unsupported")?))};
    let float_allowed_values=if carrier=="float" {declaration.get("allowedValues").and_then(Value::as_array).map(|allowed|allowed.iter().filter_map(|candidate|candidate.get("floatToken").and_then(Value::as_str)).filter_map(|token|token.parse::<f64>().ok()).filter(|number|number.is_finite()).map(float_key).collect())}else{None};
    Ok(QueryField {
        id,
        field: reference.clone(),
        carrier,
        nullable: declaration["nullability"] == "absent-allowed",
        declaration,
        prepared_literal,
        float_allowed_values,
    })
}
fn query_output_field(doc: &Value, module: &str, indexes:&ModuleIndexes, id: String, reference: &Value) -> Result<QueryField> {
    let declaration = output_field(doc, module, indexes, reference)?.clone();
    Ok(QueryField {
        id,
        field: reference.clone(),
        carrier: declaration["scalarType"].as_str().ok_or("query Field type absent")?.to_owned(),
        nullable: declaration["nullability"] == "absent-allowed",
        declaration,
        prepared_literal:None,
        float_allowed_values:None,
    })
}
fn canonical_name(part: &sqlparser::ast::ObjectNamePart) -> Result<String> {
    match part {
        sqlparser::ast::ObjectNamePart::Identifier(i) => Ok(if i.quote_style.is_some() {
            i.value.clone()
        } else {
            i.value.to_ascii_lowercase()
        }),
        _ => Err("dynamic query relation name refused"),
    }
}
fn output_name(item: &SelectItem) -> Result<String> {
    let expression = match item {
        SelectItem::ExprWithAlias { alias, .. } => return Ok(alias.value.clone()),
        SelectItem::UnnamedExpr(expression) => expression,
        _ => return Err("query wildcard projections require explicit declared outputs"),
    };
    let last = match expression {
        Expr::Identifier(identifier) => identifier,
        Expr::CompoundIdentifier(parts) => parts.last().ok_or("query output identifier absent")?,
        _ => return Err("computed query expressions require an explicit output alias"),
    };
    Ok(last.value.clone())
}

pub fn admit_json(raw: &str) -> Result<AdmittedQuery> {
    if raw.len() > 8_388_608 {
        return Err("query preparation request capacity exceeded");
    }
    let request: PrepareRequest = serde_json::from_value(
        crate::json::checked_json(raw)
            .map_err(|_| "query preparation JSON invalid or duplicate")?,
    )
    .map_err(|_| "query preparation request shape differs")?;
    if request.version != "weft.table-query/0.1"
        || !matches!(request.target.as_str(), "postgresql18" | "lakebase18")
        || request
            .binding
            .native_lifecycle
            .as_ref()
            .is_none_or(|l| l.module != request.module)
    {
        return Err("query preparation target/module profile differs");
    }
    let model = Arc::new(crate::raw_read::prepare_operation_model(&request.binding, Some(&request.operation))?);
    let binding = Arc::new(request.binding);
    let indexes=Arc::new(ModuleIndexes::new(&model,&binding,&request.module)?);
    let identity = QueryIdentity { target: request.target, module: request.module, operation: request.operation };
    admit_request(identity, binding, model, indexes)
}

fn admit_request(request: QueryIdentity, binding: Arc<Binding>, model: Arc<crate::raw_read::AdmittedModel>, indexes:Arc<ModuleIndexes>) -> Result<AdmittedQuery> {
    let lifecycle = model.lifecycle.as_ref().ok_or("query lifecycle source absent")?;
    let module = model.document["modules"].as_array().and_then(|m|m.get(indexes.module_position)).filter(|m|m["id"]==request.module).ok_or("query source module absent")?;
    let payload = &module["extensions"]["umf.tables"];
    let command_index=*indexes.commands.get(&request.operation).ok_or("query operation not declared")?;
    let declaration = payload["commands"].as_array().and_then(|c|c.get(command_index)).filter(|c|c["id"]==request.operation).ok_or("query operation not declared")?.clone();
    if !crate::table_operation::known_declaration(&declaration) {
        return Err("query operation declaration schema refused");
    }
    if declaration["version"] != "umf.table-operation/0.1"
        || declaration["body"]["kind"] != "query"
        || declaration["body"]["language"] != "weft.pg-select/0.1"
        || declaration["consistency"] != "statement"
        || declaration["replay"] != "none"
        || declaration.get("batch").is_some()
    {
        return Err("query operation profile differs");
    }
    let sql = declaration["body"]["sql"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 1_048_576)
        .ok_or("query source text outside bounds")?;
    let mut statements =
        Parser::parse_sql(&PostgreSqlDialect {}, sql).map_err(|_| "query source syntax refused")?;
    if statements.len() != 1 {
        return Err("query source must contain one SELECT");
    }
    let Statement::Query(query) = statements.remove(0) else {
        return Err("query source must contain one SELECT");
    };
    let root_select = match query.body.as_ref() {
        SetExpr::Select(select) => select,
        _ => return Err("query 0.1 requires a SELECT root"),
    };

    let mut parameters = BTreeMap::new();
    for parameter in declaration["parameters"]
        .as_array()
        .ok_or("query parameter declaration absent")?
    {
        let id = parameter["id"]
            .as_str()
            .filter(|v| {
                !v.is_empty()
                    && v.len() <= 256
                    && v.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
            })
            .ok_or("query parameter identity invalid")?
            .to_owned();
        let resolved = query_field(
            &model.document,
            &request.module,
            &indexes,
            id.clone(),
            &parameter["field"],
        )?;
        if parameters.insert(id, resolved).is_some() {
            return Err("duplicate query parameter identity");
        }
    }
    let mut outputs = vec![];
    let mut output_names = BTreeSet::new();
    let mut output_fields = BTreeSet::new();
    for output in declaration["body"]["outputs"]
        .as_array()
        .ok_or("query outputs absent")?
    {
        let name = output["name"]
            .as_str()
            .filter(|n| !n.is_empty() && n.len() <= 256)
            .ok_or("query output alias invalid")?;
        if !output_names.insert(name.to_owned()) {
            return Err("duplicate query output alias");
        }
        if !output_fields.insert(field_key(&output["field"])?) {
            return Err("query output Field references must be unique");
        }
        outputs.push(query_output_field(
            &model.document,
            &request.module,
            &indexes,
            name.to_owned(),
            &output["field"],
        )?);
    }
    let selected_names = root_select
        .projection
        .iter()
        .map(output_name)
        .collect::<Result<Vec<_>>>()?;
    if selected_names != outputs.iter().map(|o| o.id.clone()).collect::<Vec<_>>() {
        return Err("query SELECT output aliases/order differ from the declared Field outputs");
    }
    let mut scan = ScanInventory {
        relations: vec![],
        parameters: BTreeSet::new(),
        parameter_order: vec![],
        cte_names: BTreeSet::new(),
        cte_scopes: CteScopes::default(),
        unsupported:false,
    };
    let _ = query.visit(&mut scan);
    if scan.unsupported { return Err("query relation/query modifier outside read profile"); }
    let declared_parameters = parameters.keys().cloned().collect::<BTreeSet<_>>();
    if scan.parameters != declared_parameters {
        return Err("query SQL parameters differ from declared typed inputs");
    }

    let table_declarations = payload["tables"].as_array().ok_or("query table source absent")?;
    let module_namespace = indexes.namespace.as_str();
    let mut selected_paths = BTreeSet::new();
    let mut selected_tables = BTreeMap::new();
    let mut bound_table_positions = BTreeMap::new();
    for (parts, alias, is_cte) in scan.relations {
        if is_cte {
            continue;
        }
        let (namespace, relation) = match parts.as_slice() {
            [relation] => (module_namespace, relation.as_str()),
            [namespace, relation] if namespace == module_namespace => {
                (namespace.as_str(), relation.as_str())
            }
            _ => return Err("query relation must resolve inside its owning module"),
        };
        let _ = namespace;
        let table_index=*indexes.tables.get(relation).ok_or("query relation is not an original declared table")?;
        let table_decl=table_declarations.get(table_index).ok_or("query table index absent")?;
        let record_ref = &table_decl["record"];
        let record_module=record_ref["module"].as_str().ok_or("query Record source absent")?;
        let record_element=record_ref["element"].as_str().ok_or("query Record source absent")?;
        let path=indexes.record_paths.get(&(record_module.to_owned(),record_element.to_owned()))
            .ok_or("query Record source absent")?.clone();
        let bound_index=*indexes.bound_tables.get(&path).ok_or("query relation native mapping absent")?;
        let bound=binding.tables.get(bound_index).ok_or("query relation native mapping absent")?;
        if bound.native != table_decl["name"] {
            return Err("query relation native mapping differs from original table source");
        }
        selected_paths.insert(path);
        selected_tables.entry(relation.to_owned()).or_insert_with(|| bound.native.clone());
        bound_table_positions.entry(relation.to_owned()).or_insert(bound_index);
        let _ = alias;
    }
    if selected_paths.is_empty() {
        return Err("query must select at least one declared table");
    }
    let max_rows = crate::security_literals::unsigned(&declaration["body"]["maxRows"])
        .map_err(|_| "query result bound invalid")?;
    if !(1..=1000).contains(&max_rows) {
        return Err("query result bound outside profile");
    }
    let obligations = lifecycle.obligation(&selected_paths)?;
    let original_source_digest = binding.model_sha256.clone();
    Ok(AdmittedQuery {
        request,
        binding,
        _model: model,
        declaration,
        query: *query,
        parameters,
        outputs,
        parameter_order: scan.parameter_order,
        tables: selected_tables,
        bound_table_positions,
        cte_names: scan.cte_names,
        max_rows,
        obligations,
        original_source_digest,
    })
}

fn check_value(field: &QueryField, value: &Value) -> Result<()> {
    if field.carrier == "float" {
        if value.is_null() {
            return if field.nullable {
                Ok(())
            } else {
                Err("required query Float input is null")
            };
        }
        if value.as_object().is_none_or(|o| o.len() != 1) {
            return Err("query Float input needs an exact floatToken wrapper");
        }
        let token = value["floatToken"]
            .as_str()
            .ok_or("query Float token absent")?;
        if token.is_empty() || token.len() > 128 || token.trim() != token {
            return Err("query Float token lexical form invalid");
        }
        let number: f64 = token
            .parse()
            .map_err(|_| "query Float token is outside binary64")?;
        if !number.is_finite() { return Err("query Float token must be finite binary64"); }
        if field.float_allowed_values.as_ref().is_some_and(|allowed|!allowed.contains(&float_key(number))) {
            return Err("query Float input is outside its original UMF allowedValues");
        }
        return Ok(());
    }
    field.prepared_literal.as_ref().ok_or("query input literal checker absent")?.check(value)
        .map_err(|_| "query input violates its original UMF Field")
}
fn float_key(number:f64)->u64{if number==0.0{0}else{number.to_bits()}}
fn native_value(field: &QueryField, value: &Value) -> Result<Value> {
    if value.is_null() {
        return Ok(Value::Null);
    }
    Ok(match field.carrier.as_str() {
        "boolean" => value["boolean"].clone(),
        "string" => value["string"].clone(),
        "float" => json!(value["floatToken"].as_str().ok_or("Float token absent")?),
        "uuid" => json!(value["uuid"].as_str().ok_or("UUID token absent")?.to_ascii_lowercase()),
        "date" => json!(value["date"].as_str().ok_or("date token absent")?),
        "time" => json!(value["time"].as_str().ok_or("time token absent")?),
        "timestamp" => json!(value["timestamp"].as_str().ok_or("timestamp token absent")?),
        "bits128" => json!(value["bits128Hex"].as_str().ok_or("Bits128 token absent")?.to_ascii_lowercase()),
        "integer" => json!(crate::security_literals::coefficient(
            value["integerToken"]
                .as_str()
                .ok_or("integer token absent")?,
            0,
            None
        )
        .map_err(|_| "integer token invalid")?
        .to_string()),
        "decimal" => {
            let scale = crate::security_literals::unsigned(&field.declaration["facets"]["scale"])
                .map_err(|_| "decimal scale absent")?;
            let precision =
                crate::security_literals::unsigned(&field.declaration["facets"]["precision"])
                    .map_err(|_| "decimal precision absent")?;
            let n = crate::security_literals::coefficient(
                value["decimalToken"]
                    .as_str()
                    .ok_or("decimal token absent")?,
                scale,
                Some(precision),
            )
            .map_err(|_| "decimal token invalid")?;
            let mut digits = n.to_string();
            if digits == "0" {
                json!("0")
            } else {
                let zeros = digits.bytes().rev().take_while(|b| *b == b'0').count();
                digits.truncate(digits.len() - zeros);
                let exp = zeros as i64 - scale as i64;
                json!(if exp == 0 {
                    digits
                } else {
                    format!("{digits}e{exp}")
                })
            }
        }
        _ => return Err("query input native codec is not yet admitted"),
    })
}

#[cfg(test)]
mod query_input_tests {
    use super::*;
    fn dto_field(scalar: &str, nullable: &str, facets: Option<Value>, allowed: Option<Value>) -> (Value, ModuleIndexes) {
        let mut field = json!({
            "id": "out", "kind": "field", "scalarType": scalar,
            "cardinality": "one", "nullability": nullable, "extensions": {}
        });
        if let Some(facets) = facets { field["facets"] = facets; }
        if let Some(allowed) = allowed { field["allowedValues"] = allowed; }
        let doc = json!({"modules":[{"id":"m","elements":[field]}]});
        let indexes = ModuleIndexes {
            module_position: 0,
            elements: BTreeMap::from([("out".to_owned(), 0)]),
            commands: BTreeMap::new(),
            tables: BTreeMap::new(),
            record_paths: BTreeMap::new(),
            bound_tables: BTreeMap::new(),
            namespace: "".into(),
        };
        (doc, indexes)
    }

    #[test]
    fn nullable_unrefined_decimal_and_temporal_dto_outputs_are_admitted() {
        for scalar in ["decimal", "date", "time", "timestamp"] {
            let (mut doc, indexes) = dto_field(scalar, "absent-allowed", None, None);
            doc["modules"][0]["elements"][0]["name"] = json!("value");
            doc["modules"][0]["elements"][0]["title"] = json!("Display value");
            doc["modules"][0]["elements"][0]["aliases"] = json!(["displayValue"]);
            let field = query_output_field(
                &doc, "m", &indexes, "value".into(), &json!({"module":"m","element":"out"}),
            ).expect("unrefined nullable output scalar should be admitted");
            assert_eq!(field.carrier, scalar);
            assert!(field.nullable);
            assert!(field.declaration.get("facets").is_none());
            assert_eq!(field.declaration["title"], "Display value");
            assert_eq!(field.declaration["aliases"], json!(["displayValue"]));
            assert!(crate::security_literals::validate_field(&field.declaration).is_err(),
                "query DTO admission must not broaden shared security literal semantics");
        }
    }

    #[test]
    fn query_output_fields_refuse_structural_and_unknown_umf_meaning() {
        let cases = [
            ("nonempty references", json!({"references":[{"role":"item","module":"m","element":"other"}]})),
            ("item type link", json!({"itemType":{"module":"m","element":"other"}})),
            ("semantic type reference", json!({"semanticTypes":[{"module":"m","element":"meaning"}]})),
            ("unknown singular semanticType key", json!({"semanticType":{"module":"m","element":"meaning"}})),
            ("unknown meaning key", json!({"storageMeaning":{"kind":"opaque"}})),
        ];
        for (label, addition) in cases {
            let (mut doc, indexes) = dto_field("decimal", "absent-allowed", None, None);
            for (key, value) in addition.as_object().unwrap() {
                doc["modules"][0]["elements"][0][key] = value.clone();
            }
            assert!(output_field(&doc, "m", &indexes, &json!({"module":"m","element":"out"})).is_err(),
                "query output must refuse {label}");
        }
    }

    #[test]
    fn constrained_decimal_inputs_keep_exact_original_validation() {
        let (doc, indexes) = dto_field(
            "decimal", "required", Some(json!({"precision":8,"scale":2})),
            Some(json!([{"decimalToken":"1.23"}])),
        );
        let declaration = field(&doc, "m", &indexes, &json!({"module":"m","element":"out"})).unwrap().clone();
        let input = QueryField {
            id: "amount".into(), field: json!({"module":"m","element":"out"}),
            carrier: "decimal".into(), nullable: false, declaration,
            prepared_literal:Some(Arc::new(crate::security_literals::PreparedLiteral::new(&json!({"kind":"field","cardinality":"one","nullability":"required","scalarType":"decimal","facets":{"precision":8,"scale":2},"allowedValues":[{"decimalToken":"1.23"}] })).unwrap())),
            float_allowed_values:None,
        };
        assert!(check_value(&input, &json!({"decimalToken":"1.23"})).is_ok());
        assert!(check_value(&input, &json!({"decimalToken":"1.24"})).is_err());
    }

    #[test]
    fn prepared_decimal_inputs_reuse_numeric_allowed_values_and_keep_nullability() {
        let (doc,indexes)=dto_field("decimal","absent-allowed",Some(json!({"precision":8,"scale":2})),Some(json!([{"decimalToken":"1.00"}])));
        let input=query_field(&doc,"m",&indexes,"amount".into(),&json!({"module":"m","element":"out"})).unwrap();
        for token in ["1","1.0","1.000"] { assert!(check_value(&input,&json!({"decimalToken":token})).is_ok(),"{token}"); }
        assert!(check_value(&input,&json!(null)).is_ok());
        assert!(check_value(&input,&json!({"decimalToken":"2"})).is_err());
    }

    #[test]
    fn float_inputs_enforce_source_allowed_values() {
        let field=QueryField {
            id:"score".into(), field:json!({}), carrier:"float".into(), nullable:false,
            declaration:json!({"allowedValues":[{"floatToken":"1.5"}]}),
            prepared_literal:None,
            float_allowed_values:Some([float_key(1.5)].into_iter().collect()),
        };
        assert!(check_value(&field,&json!({"floatToken":"1.5"})).is_ok());
        assert!(check_value(&field,&json!({"floatToken":"2.5"})).is_err());
    }

    #[test]
    fn float_allowed_values_use_binary64_equality_with_signed_zero_canonicalized() {
        let field=QueryField{id:"score".into(),field:json!({}),carrier:"float".into(),nullable:false,
            declaration:json!({"allowedValues":[{"floatToken":"-0.0"}]}),prepared_literal:None,
            float_allowed_values:Some([float_key(-0.0)].into_iter().collect())};
        assert!(check_value(&field,&json!({"floatToken":"0"})).is_ok());
        assert!(check_value(&field,&json!({"floatToken":"-0"})).is_ok());
        assert!(check_value(&field,&json!({"floatToken":"1e-300"})).is_err());
    }

    #[test]
    fn nonrecursive_cte_visibility_is_sequential() {
        use sqlparser::{ast::Visit,dialect::PostgreSqlDialect,parser::Parser};
        fn scan(sql:&str)->Vec<(Vec<String>,Option<String>,bool)> {
            let mut statements=Parser::parse_sql(&PostgreSqlDialect{},sql).unwrap();
            let Statement::Query(query)=statements.remove(0) else { panic!("expected query") };
            let mut inventory=ScanInventory{relations:vec![],parameters:BTreeSet::new(),parameter_order:vec![],cte_names:BTreeSet::new(),cte_scopes:CteScopes::default(),unsupported:false};
            let _=query.visit(&mut inventory);inventory.relations
        }
        let forward=scan("WITH first AS (SELECT id FROM later), later AS (SELECT id FROM source) SELECT id FROM first");
        assert_eq!(forward.iter().filter(|(r,_,is_cte)|r==&["later"]&&!is_cte).count(),1);
        assert_eq!(forward.iter().filter(|(r,_,is_cte)|r==&["first"]&&*is_cte).count(),1);
        let self_ref=scan("WITH first AS (SELECT id FROM first) SELECT id FROM first");
        assert_eq!(self_ref.iter().filter(|(r,_,is_cte)|r==&["first"]&&!*is_cte).count(),1);
        assert_eq!(self_ref.iter().filter(|(r,_,is_cte)|r==&["first"]&&*is_cte).count(),1);
        let earlier=scan("WITH first AS (SELECT id FROM source), later AS (SELECT id FROM first) SELECT id FROM later");
        assert_eq!(earlier.iter().filter(|(r,_,is_cte)|r==&["first"]&&*is_cte).count(),1);
        let chain=scan("WITH first AS (SELECT id FROM source), second AS (SELECT id FROM first), third AS (SELECT id FROM second) SELECT id FROM third");
        assert_eq!(chain.iter().filter(|(r,_,is_cte)|r==&["first"]&&*is_cte).count(),1);
        assert_eq!(chain.iter().filter(|(r,_,is_cte)|r==&["second"]&&*is_cte).count(),1);
        assert_eq!(chain.iter().filter(|(r,_,is_cte)|r==&["third"]&&*is_cte).count(),1);
    }
    #[test]
    fn select_into_is_not_a_read_query() {
        use sqlparser::{ast::Visit,dialect::PostgreSqlDialect,parser::Parser};
        let mut statements=Parser::parse_sql(&PostgreSqlDialect{},"SELECT id INTO created_table FROM source").unwrap();
        let Statement::Query(query)=statements.remove(0) else {panic!("expected query")};
        let mut inventory=ScanInventory{relations:vec![],parameters:BTreeSet::new(),parameter_order:vec![],cte_names:BTreeSet::new(),cte_scopes:CteScopes::default(),unsupported:false};
        let _=query.visit(&mut inventory);
        assert!(inventory.unsupported);
    }
}
