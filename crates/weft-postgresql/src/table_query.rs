// SPDX-License-Identifier: MIT OR Apache-2.0
//! Fixed source-pinned PostgreSQL query templates.
use serde_json::{json, Value as JsonValue};
use sqlparser::ast::{VisitMut, *};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::ops::ControlFlow;
use weft_core::table_query::{self, AdmittedQuery};
use sqlparser::{dialect::PostgreSqlDialect, parser::Parser};
type Result<T> = std::result::Result<T, &'static str>;
fn ident(s: &str) -> Result<Ident> {
    if s.is_empty() || s.len() > 63 || s.contains('\0') {
        return Err("query identifier outside PostgreSQL profile");
    }
    Ok(Ident::with_quote('"', s.to_owned()))
}
fn canon(i: &Ident) -> String {
    if i.quote_style.is_some() {
        i.value.clone()
    } else {
        i.value.to_ascii_lowercase()
    }
}
fn definitely_nonnull(expr: &Expr) -> bool {
    match expr {
        Expr::Value(v) => !matches!(v.value, Value::Null | Value::Placeholder(_)),
        Expr::Nested(inner) | Expr::Cast { expr: inner, .. } => definitely_nonnull(inner),
        Expr::Function(f) => {
            let name=f.name.to_string().to_ascii_lowercase();
            let name=name.rsplit('.').next().unwrap_or(&name);
            if name == "count" { return true; }
            if name == "coalesce" {
                if let FunctionArguments::List(args)=&f.args {
                    return args.args.iter().any(|arg| match arg {
                        FunctionArg::Unnamed(FunctionArgExpr::Expr(e)) => definitely_nonnull(e),
                        FunctionArg::Named { arg: FunctionArgExpr::Expr(e), .. } => definitely_nonnull(e),
                        _ => false,
                    });
                }
            }
            false
        }
        _ => false,
    }
}
fn result_nullable(expr: &Expr, declared_nullable: bool) -> Result<bool> {
    if !declared_nullable && !definitely_nonnull(expr) {
        return Err("query expression may be NULL but its output Field is required");
    }
    Ok(declared_nullable)
}
fn qualify_function(f: &mut Function) -> Result<()> {
    let n=f.name.to_string().to_ascii_lowercase();
    let ok=["count","sum","avg","min","max","coalesce","lower","left","length","strpos","date_trunc","to_char","lag","lead","nullif","abs","round","greatest","least","encode","float8send"];
    if !ok.contains(&n.as_str()) || f.name.0.len()!=1 { return Err("query function outside closed profile"); }
    if !matches!(n.as_str(),"coalesce"|"nullif"|"greatest"|"least") {
        f.name=ObjectName(vec![ObjectNamePart::Identifier(Ident::new("pg_catalog")),ObjectNamePart::Identifier(Ident::new(n))]);
    }
    Ok(())
}
fn validate_typed_string(value:&TypedString)->Result<()> {
    if value.uses_odbc_syntax { return Err("ODBC typed string syntax outside profile"); }
    let ty=value.data_type.to_string().to_ascii_lowercase();
    if ["date","time","time without time zone","timestamp","timestamp without time zone","timestamp with time zone","timestamptz"].contains(&ty.as_str()) { Ok(()) }
    else { Err("typed string datatype outside fixed PostgreSQL profile") }
}
fn qualify_collation(collation:&mut ObjectName)->Result<()> {
    let valid=match collation.0.as_slice() {
        [ObjectNamePart::Identifier(name)]=>name.value.eq_ignore_ascii_case("C"),
        [ObjectNamePart::Identifier(schema),ObjectNamePart::Identifier(name)]=>schema.value.eq_ignore_ascii_case("pg_catalog")&&name.value.eq_ignore_ascii_case("C"),
        _=>false,
    };
    if !valid { return Err("collation outside fixed PostgreSQL C profile"); }
    collation.0=vec![ObjectNamePart::Identifier(Ident::new("pg_catalog")),ObjectNamePart::Identifier(Ident::with_quote('"',"C"))];
    Ok(())
}
fn unused_sort_alias(index: usize, referenced: &BTreeSet<String>, outputs: &BTreeMap<String,&Expr>) -> Result<String> {
    let mut suffix=0usize;
    loop {
        let candidate=format!("__weft_sort_{index}_{suffix}");
        if !outputs.contains_key(&candidate) && !referenced.contains(&candidate) { return Ok(candidate); }
        suffix=suffix.checked_add(1).ok_or("query sort alias space exhausted")?;
    }
}
fn projection_name(item: &SelectItem) -> Result<String> {
    match item {
        SelectItem::ExprWithAlias { alias, .. } => Ok(canon(alias)),
        SelectItem::UnnamedExpr(Expr::Identifier(i)) => Ok(canon(i)),
        SelectItem::UnnamedExpr(Expr::CompoundIdentifier(parts)) => parts.last().map(canon).ok_or("query output identifier absent"),
        _ => Err("query projection needs an explicit output name"),
    }
}
fn exact_projected_expressions(projected: &[(String, Expr)]) -> HashSet<&Expr> {
    projected.iter().map(|(_, expr)| expr).collect()
}
fn owned_projection_query(schema:&str, native_table:&str, columns:&[(String,String)]) -> Result<Query> {
    let mut pieces=Vec::with_capacity(columns.len());
    for (logical,native_column) in columns {
        let (Ok(nc),Ok(lc))=(ident(native_column),ident(logical)) else { return Err("invalid owned column identifier"); };
        pieces.push(format!("\"__weft_source\".{nc} AS {lc}"));
    }
    if pieces.is_empty() { return Err("owned table has no pinned columns"); }
    let (Ok(schema_name),Ok(native_name))=(ident(schema),ident(native_table)) else { return Err("invalid native relation identifier"); };
    let sql=format!("SELECT {} FROM {schema_name}.{native_name} AS \"__weft_source\"",pieces.join(", "));
    let mut parsed=Parser::parse_sql(&PostgreSqlDialect{},&sql).map_err(|_|"generated owned-table projection invalid")?;
    if parsed.len()!=1 { return Err("generated owned-table projection invalid"); }
    let Statement::Query(query)=parsed.remove(0) else { return Err("generated owned-table projection invalid"); };
    Ok(*query)
}
struct Lower<'a> {
    native_tables: &'a BTreeMap<String, String>,
    table_columns: &'a BTreeMap<String, Vec<(String, String)>>,
    cte_scopes: table_query::CteScopes,
    schema: &'a str,
    params: &'a BTreeMap<String, usize>,
    carriers: &'a BTreeMap<String, String>,
    referenced_names: BTreeSet<String>,
    error: Option<&'static str>,
}
impl VisitorMut for Lower<'_> {
    type Break = ();
    fn pre_visit_query(&mut self, query: &mut Query) -> ControlFlow<()> {
        self.cte_scopes.enter(query);
        ControlFlow::Continue(())
    }
    fn post_visit_query(&mut self, _query: &mut Query) -> ControlFlow<()> { self.cte_scopes.leave(); ControlFlow::Continue(()) }
    fn pre_visit_select(&mut self, select:&mut Select)->ControlFlow<()> {
        if select.into.is_some() { self.error=Some("SELECT INTO mutation outside read profile"); }
        ControlFlow::Continue(())
    }
    fn pre_visit_table_factor(&mut self, f: &mut TableFactor) -> ControlFlow<()> {
        if let TableFactor::Table { name, alias, args, with_hints, version, with_ordinality, partitions, json_path, sample, index_hints } = f {
            if args.is_some() || !with_hints.is_empty() || version.is_some() || *with_ordinality
                || !partitions.is_empty() || json_path.is_some() || sample.is_some() || !index_hints.is_empty() {
                self.error=Some("owned relation modifier outside fixed profile");
                return ControlFlow::Continue(());
            }
            let rel = name.0.last().and_then(|p| {
                if let ObjectNamePart::Identifier(i) = p {
                    Some(canon(i))
                } else {
                    None
                }
            });
            if let Some(rel) = rel {
                let is_cte=name.0.len()==1 && self.cte_scopes.contains(&rel);
                if !is_cte {
                    // The generated inner source is recognized by its exact schema/table
                    // pin and marker alias; it is not a user-authored relation escape.
                    if name.0.len()==2 && name.0[0].as_ident().is_some_and(|i|i.value==self.schema)
                        && alias.as_ref().is_some_and(|a|a.name.value=="__weft_source") {
                        return ControlFlow::Continue(());
                    }
                    let Some(native)=self.native_tables.get(&rel) else { self.error=Some("relation outside pinned module"); return ControlFlow::Continue(()); };
                    let Some(columns)=self.table_columns.get(&rel) else { self.error=Some("owned table columns absent"); return ControlFlow::Continue(()); };
                    // Identity column mappings need no projection. Keeping the base
                    // relation preserves PostgreSQL primary-key functional dependencies.
                    if columns.iter().all(|(logical,physical)|logical==physical)
                        && alias.as_ref().is_none_or(|a|a.columns.is_empty()) {
                        let (Ok(schema),Ok(table),Ok(logical_alias))=(ident(self.schema),ident(native),ident(&rel)) else {
                            self.error=Some("invalid native relation identifier");
                            return ControlFlow::Continue(());
                        };
                        *name=ObjectName(vec![ObjectNamePart::Identifier(schema),ObjectNamePart::Identifier(table)]);
                        if alias.is_none() {
                            *alias=Some(TableAlias{explicit:false,name:logical_alias,columns:vec![],at:None});
                        }
                        return ControlFlow::Continue(());
                    }
                    let subquery=match owned_projection_query(self.schema,native,columns) { Ok(q)=>Box::new(q),Err(reason)=>{self.error=Some(reason);return ControlFlow::Continue(())} };
                    let Some(default_name)=ident(&rel).ok() else { self.error=Some("invalid logical relation alias"); return ControlFlow::Continue(()); };
                    let outer_alias=alias.clone().unwrap_or_else(||TableAlias{explicit:false,name:default_name,columns:vec![],at:None});
                    // Preserve caller column aliases; otherwise expose only logical names.
                    *f=TableFactor::Derived{lateral:false,subquery,alias:Some(outer_alias),sample:None};
                }
            }
        }
        ControlFlow::Continue(())
    }
    fn pre_visit_expr(&mut self, e: &mut Expr) -> ControlFlow<()> {
        match e {
            Expr::Identifier(i) => { self.referenced_names.insert(canon(i)); }
            Expr::CompoundIdentifier(parts) => {
                self.referenced_names.extend(parts.iter().map(canon));
            }
            _ => {}
        }
        match e {
            Expr::Wildcard(_) | Expr::QualifiedWildcard(_, _) => self.error=Some("whole-row/system-column query expression refused"),
            Expr::CompoundIdentifier(parts) if parts.last().is_some_and(|i|matches!(canon(i).as_str(),"ctid"|"xmin"|"xmax"|"cmin"|"cmax"|"tableoid"|"oid")) => self.error=Some("system-column query expression refused"),
            Expr::Identifier(i) if matches!(canon(i).as_str(),"ctid"|"xmin"|"xmax"|"cmin"|"cmax"|"tableoid"|"oid") => self.error=Some("system-column query expression refused"),
            Expr::Value(v) => {
                if let sqlparser::ast::Value::Placeholder(p) = &mut v.value {
                    if let Some(id) = p.strip_prefix(':') {
                        if let Some(i) = self.params.get(id) {
                            let ty = match self.carriers.get(id).map(String::as_str).unwrap_or("") {
                                "boolean" => "bool",
                                "integer" | "decimal" => "numeric",
                                "float" => "float8",
                                "string" => "text",
                                "date" => "date",
                                "time" => "time",
                                "timestamp" => "timestamptz",
                                "uuid" => "uuid",
                                "bits128" => "text",
                                _ => {
                                    self.error = Some("unsupported query parameter carrier");
                                    return ControlFlow::Continue(());
                                }
                            };
                            let slot = "$".to_owned() + &(i + 1).to_string();
                            *p = if self.carriers.get(id).is_some_and(|c|c=="bits128") {
                                format!("pg_catalog.decode({slot}::pg_catalog.text,'hex')")
                            } else { format!("{slot}::pg_catalog.{ty}") };
                        } else {
                            self.error = Some("undeclared query parameter")
                        }
                    }
                }
            }
            Expr::Function(f) => {
                if let Err(reason)=qualify_function(f) { self.error=Some(reason); }
            }
            Expr::TypedString(typed) => if let Err(reason)=validate_typed_string(typed) { self.error=Some(reason); },
            Expr::Collate { collation, .. } => if let Err(reason)=qualify_collation(collation) { self.error=Some(reason); },
            Expr::Interval(_) => self.error=Some("interval syntax outside fixed PostgreSQL profile"),
            Expr::Cast { data_type, .. } => {
                let target = data_type.to_string().to_ascii_lowercase();
                let allowed = ["int", "int2", "int4", "int8", "integer", "bigint", "smallint", "numeric", "decimal", "float8", "double precision", "real", "text", "varchar", "character varying", "bool", "boolean", "date", "time", "timestamp", "timestamp with time zone", "timestamptz", "uuid", "bytea"];
                if !allowed.contains(&target.as_str()) { self.error = Some("query cast target outside closed PostgreSQL profile"); }
            }
            Expr::CompoundIdentifier(p) if p.len() > 2 => self.error = Some("query column path outside alias-qualified profile"),
            Expr::BinaryOp { op, .. } if !matches!(op.to_string().to_ascii_lowercase().as_str(),"+"|"-"|"*"|"/"|"%"|"="|"<>"|"!="|">"|"<"|">="|"<="|"and"|"or"|"||"|"like"|"not like"|"ilike"|"not ilike"|"is distinct from"|"is not distinct from") => self.error = Some("query operator outside fixed PostgreSQL closure"),
            _ => {}
        }
        ControlFlow::Continue(())
    }
}
fn template(a: &AdmittedQuery) -> Result<serde_json::Value> {
    let mut query = a.query().clone();
    // Give every public root projection its declared, stable alias before
    // resolving source identifiers to native column names. In particular,
    // `SELECT logical_name` may lower to `SELECT t.native_name`; without an
    // explicit alias PostgreSQL would expose the native spelling to the
    // outer wire frame.
    if let SetExpr::Select(select) = query.body.as_mut() {
        if select.projection.len() != a.outputs().len() { return Err("query projection/output arity differs"); }
        for (item, output) in select.projection.iter_mut().zip(a.outputs()) {
            match item {
                SelectItem::ExprWithAlias { alias, .. } if canon(alias) == output.id => {}
                SelectItem::ExprWithAlias { .. } => return Err("query output aliases/order differ from declared Field outputs"),
                SelectItem::UnnamedExpr(expr) => *item = SelectItem::ExprWithAlias { expr: expr.clone(), alias: ident(&output.id)? },
                _ => return Err("query wildcard projection refused"),
            }
        }
    }
    let schema = a
        .binding()
        .schema
        .as_deref()
        .ok_or("query schema absent")?;
    let params = a
        .parameter_order()
        .iter()
        .enumerate()
        .map(|(i, n)| (n.clone(), i))
        .collect::<BTreeMap<_, _>>();
    let carriers = a
        .parameters()
        .iter()
        .map(|(i, f)| (i.clone(), f.carrier.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut table_columns=BTreeMap::<String,Vec<(String,String)>>::new();
    for (relation,native) in a.tables() {
        let table=a.bound_table(relation).filter(|table|table.native==*native).ok_or("owned relation binding absent")?;
        table_columns.insert(relation.clone(),table.columns.iter().map(|c|(c.logical.clone(),c.native.clone())).collect());
    }
    let mut l = Lower {
        native_tables:a.tables(),
        table_columns:&table_columns,
        cte_scopes:table_query::CteScopes::default(),
        schema,
        params: &params,
        carriers: &carriers,
        referenced_names: BTreeSet::new(),
        error: None,
    };
    let _ = VisitMut::visit(&mut query, &mut l);
    if let Some(e) = l.error {
        return Err(e);
    }
    if query.fetch.is_some() || !query.locks.is_empty() || query.for_clause.is_some() {
        return Err("query fetch/locking/for-clause outside read profile");
    }
    let (result_nullable, sort_order) = {
        let SetExpr::Select(select) = query.body.as_mut() else { return Err("query root must be SELECT") };
        if select.projection.len()!=a.outputs().len(){return Err("query projection/output arity differs")}
        let projected = select.projection.iter().map(|item| match item {
            SelectItem::ExprWithAlias { expr, alias } => Ok((canon(alias), expr.clone())),
            SelectItem::UnnamedExpr(expr) => Ok((projection_name(item)?, expr.clone())),
            _ => Err("query wildcard projection refused"),
        }).collect::<Result<Vec<_>>>()?;
        let aliases=projected.iter().map(|(name,expr)|(name.clone(),expr)).collect::<BTreeMap<_,_>>();
        let distinct_projection=select.distinct.as_ref().map(|_|exact_projected_expressions(&projected));
        let mut nullability=Vec::with_capacity(projected.len());
        for ((_,expr),output) in projected.iter().zip(a.outputs()) {
            nullability.push(result_nullable(expr,output.nullable)?);
        }
        let mut sort_order=Vec::new();
        if let Some(order_by)=&mut query.order_by {
            let OrderByKind::Expressions(order_items)=&mut order_by.kind else { return Err("query ORDER BY ALL unsupported") };
            for (index,item) in order_items.iter().enumerate() {
                if item.with_fill.is_some() || matches!(item.options.sort,Some(OrderBySort::Using(_))) {
                    return Err("query ORDER BY extension/operator outside profile");
                }
                let sort_expr=match &item.expr {
                    Expr::Identifier(alias) => aliases.get(&canon(alias)).map(|expr|(*expr).clone()).unwrap_or_else(||item.expr.clone()),
                    Expr::Value(v) => if let Value::Number(n,false)=&v.value {
                        if let Ok(position)=n.parse::<usize>() {
                            if position==0 {return Err("query ORDER BY ordinal is one-based")}
                            projected.get(position-1).ok_or("query ORDER BY ordinal exceeds projection")?.1.clone()
                        } else { item.expr.clone() }
                    } else { item.expr.clone() },
                    _ => item.expr.clone(),
                };
                if distinct_projection.as_ref().is_some_and(|expressions|!expressions.contains(&&sort_expr)) {
                    return Err("DISTINCT query ORDER BY must use an existing projection");
                }
                let alias=unused_sort_alias(index,&l.referenced_names,&aliases)?;
                select.projection.push(SelectItem::ExprWithAlias { expr: sort_expr, alias: ident(&alias)? });
                sort_order.push(OrderByExpr { expr: Expr::Identifier(ident(&alias)?), options:item.options.clone(), with_fill:None });
            }
        }
        (nullability,sort_order)
    };
    let outputs = a.outputs().iter().zip(result_nullable).map(|(o,nullable)| {
        if o.carrier=="float" { json!({"name":o.id,"field":o.field,"carrier":o.carrier,"nullable":nullable,"nativeEncoding":"ieee754-binary64-hex"}) }
        else { json!({"name":o.id,"field":o.field,"carrier":o.carrier,"nullable":nullable}) }
    }).collect::<Vec<_>>();
    let source_sql=query.to_string();
    let native_outputs=outputs.iter().map(|o|json!({"name":o["name"],"carrier":o["carrier"]})).collect::<Vec<_>>();
    let describe_sql=crate::query_wire::describe_sql(&source_sql,&native_outputs)?;
    let sql=crate::query_wire::bound_frame_sql(&source_sql,&native_outputs,&sort_order,a.max_rows())?;
    let inputs = a
        .parameters()
        .iter()
        .map(|(id, f)| json!({"id":id,"carrier":f.carrier,"nullable":f.nullable}))
        .collect::<Vec<_>>();
    let declaration=a.declaration();
    let obligations=json!({
        "tableLifecycle":a.obligations(),
        "nativeExecutionQualified":false,
        "nativeFunctionAndOperatorClosure":"required-before-execution",
        "consistency":declaration["consistency"],
        "transaction":{"batch":false,"atomic":"entire-invocation","commitResponsibility":"host-caller-contract","onBoundExceeded":"rollback-entire-operation","writeLimitClipping":false},
        "replay":{"mode":declaration["replay"],"durableOutcomeRequired":false,"automaticRetry":false,"scope":["module","operation","source","deployment","ledger-head","verified-principal","ordered-exact-inputs","caller-key"]},
        "cursorScope":["operation","exact-inputs","source","deployment","ledger-head","principal","role-policy-scope"]
    });
    Ok(
        json!({"version":"weft.table-query-template/0.1","module":a.request().module,"operation":a.request().operation,"modelSha256":a.binding().model_sha256,"sql":sql,"nativeDescriptionSql":describe_sql,"parameterOrder":a.parameter_order(),"inputs":inputs,"outputs":outputs,"maxRows":a.max_rows(),"obligations":obligations}),
    )
}
pub fn prepare_json(s: &str) -> String {
    match PreparedQuery::prepare(s) {
        Ok(v) => v.template.to_string(),
        Err(e) => json!({"status":"error","error":e}).to_string(),
    }
}
pub struct PreparedQuery { admitted: AdmittedQuery, template: JsonValue, program: JsonValue }
impl PreparedQuery {
    pub fn prepare(source: &str) -> Result<Self> {
        let admitted = table_query::admit_json(source)?;
        Self::from_admitted(admitted)
    }
    fn from_admitted(admitted: AdmittedQuery) -> Result<Self> {
        let template = template(&admitted)?;
        let slots = template["parameterOrder"].clone();
        let mut attached = json!({"sql":template["sql"],"parameterOrder":slots});
        crate::query_wire::attach_dbapi(&mut attached)?;
        let mut description = json!({"sql":template["nativeDescriptionSql"],"parameterOrder":template["parameterOrder"]});
        crate::query_wire::attach_dbapi(&mut description)?;
        let sources=admitted.parameter_order().iter().map(|id|json!({"id":id,"source":{"kind":"input","parameter":id}})).collect::<Vec<_>>();
        let program=json!({"status":"compiled","version":"weft.table-operation/0.1","module":template["module"],"operation":template["operation"],"binding":{"modelSha256":template["modelSha256"]},"modelSha256":template["modelSha256"],"transport":{"profile":"dbapi-percent/0.1","bindEmptyParameterTuple":true},"dbapiSql":attached["dbapiSql"],"nativeDescriptionSql":description["dbapiSql"],"dbapiParameterOrder":attached["dbapiParameterOrder"],"parameterSources":sources,"outputs":template["outputs"],"result":{"kind":"many","maxRows":template["maxRows"],"nativeReadGuard":{"profile":"pg18-read-frame/0.1","maxNativeBytes":8388608,"sqlstate":"22012","includesLookahead":true,"partialFrames":false}},"obligations":template["obligations"]});
        Ok(Self { admitted, template, program })
    }
    pub fn program_json(&self) -> String { self.program.to_string() }
    pub fn bind_json(&self, call: &str) -> String { match bind(&self.template,&self.admitted,call) { Ok(v)=>v.to_string(),Err(e)=>json!({"status":"error","error":e}).to_string() } }
}
pub struct PreparedQueryModule { inner: table_query::AdmittedQueryModule }
impl PreparedQueryModule {
    pub fn prepare(source: &str) -> Result<Self> {
        Ok(Self { inner: table_query::prepare_module_json(source)? })
    }
    pub fn from_admitted(inner: table_query::AdmittedQueryModule) -> Self { Self { inner } }
    pub fn prepare_operation(&self, operation: &str) -> Result<PreparedQuery> {
        PreparedQuery::from_admitted(self.inner.prepare(operation)?)
    }
}
pub fn bind_json(_prepared: &str, _call: &str) -> String {
    json!({"status":"error","error":"an opaque prepared query object is required"}).to_string()
}
fn bind(p: &JsonValue, admitted: &AdmittedQuery, c: &str) -> Result<JsonValue> {
    let c: JsonValue = weft_core::json::checked_json(c).map_err(|_| "query call invalid")?;
    if c.as_object().is_none_or(|m|m.len()!=1||!m.contains_key("inputs")){return Err("query call shape differs")}
    let vals = c["inputs"].as_object().ok_or("query inputs absent")?;
    if vals.len()!=admitted.parameters().len()||vals.keys().any(|id|!admitted.parameters().contains_key(id)){return Err("query input IDs differ")}
    let input_map = vals.iter().map(|(id, value)| (id.clone(), value.clone())).collect::<BTreeMap<_,_>>();
    let normalized = admitted.bind_inputs(&input_map)?;
    let normalized_by_id = admitted.parameters().keys().cloned().zip(normalized).collect::<BTreeMap<_,_>>();
    let mut native = serde_json::Map::new();
    for idv in p["parameterOrder"]
        .as_array()
        .ok_or("parameter order absent")?
    {
        let id = idv.as_str().ok_or("parameter ID invalid")?;
        let _ = vals.get(id).ok_or("query parameter value absent")?;
        let x = normalized_by_id.get(id).cloned().ok_or("normalized input absent")?;
        native.insert(id.to_owned(), x);
    }
    Ok(json!({"nativeInputs":[native]}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlparser::{dialect::PostgreSqlDialect, parser::Parser};

    struct FunctionLower;
    impl VisitorMut for FunctionLower {
        type Break=();
        fn pre_visit_expr(&mut self, expr:&mut Expr)->ControlFlow<()> {
            if let Expr::Function(function)=expr { qualify_function(function).unwrap(); }
            ControlFlow::Continue(())
        }
    }
    fn query(sql:&str)->Query {
        let mut statements=Parser::parse_sql(&PostgreSqlDialect{},sql).unwrap();
        let Statement::Query(query)=statements.remove(0) else { panic!("expected query") };
        *query
    }

    #[test]
    fn postgresql_special_forms_remain_unqualified() {
        let mut query=query("SELECT COALESCE(x, 1), NULLIF(x, 0), GREATEST(x, 1), LEAST(x, 9), COUNT(*) FROM t");
        let _=VisitMut::visit(&mut query,&mut FunctionLower);
        let sql=query.to_string().to_ascii_lowercase();
        for special in ["coalesce(","nullif(","greatest(","least("] {
            assert!(sql.contains(special),"missing {special}: {sql}");
            assert!(!sql.contains(&format!("pg_catalog.{special}")),"invalid qualified special form: {sql}");
        }
        assert!(sql.contains("pg_catalog.count(*)"),"ordinary function was not schema-qualified: {sql}");
    }

    #[test]
    fn left_uses_the_native_catalog_string_function() {
        let mut query=query("SELECT LEFT(body, 160) AS preview FROM notes");
        let _=VisitMut::visit(&mut query,&mut FunctionLower);
        assert!(query.to_string().contains("pg_catalog.left(body, 160)"));
    }

    #[test]
    fn required_outputs_refuse_nullable_aggregates_and_inputs() {
        let query=query("SELECT COUNT(*), MAX(x), :maybe");
        let SetExpr::Select(select)=query.body.as_ref() else { panic!("expected select") };
        let count=match &select.projection[0]{SelectItem::UnnamedExpr(expr)=>expr,_=>panic!("expr")};
        let maximum=match &select.projection[1]{SelectItem::UnnamedExpr(expr)=>expr,_=>panic!("expr")};
        let placeholder=match &select.projection[2]{SelectItem::UnnamedExpr(expr)=>expr,_=>panic!("expr")};
        assert_eq!(result_nullable(count,false),Ok(false));
        assert!(result_nullable(maximum,false).is_err());
        assert!(result_nullable(placeholder,false).is_err());
        assert_eq!(result_nullable(maximum,true),Ok(true));
    }

    #[test]
    fn cte_name_scope_does_not_capture_aliases_or_qualified_base_relations() {
        let mut sql_query=query("WITH a AS (SELECT note_id FROM notes), b AS (SELECT note_id FROM a), c AS (SELECT note_id FROM b) SELECT n.note_id FROM c AS notes JOIN notes AS n ON true");
        let native_tables=[("notes".to_owned(),"notes_native".to_owned())].into_iter().collect();
        let table_columns=[("notes".to_owned(),vec![("note_id".to_owned(),"id_physical".to_owned())])].into_iter().collect();
        let params=BTreeMap::new();
        let carriers=BTreeMap::new();
        let mut lower=Lower{native_tables:&native_tables,table_columns:&table_columns,cte_scopes:table_query::CteScopes::default(),schema:"analytics",params:&params,carriers:&carriers,referenced_names:BTreeSet::new(),error:None};
        let _=VisitMut::visit(&mut sql_query,&mut lower);
        assert_eq!(lower.error,None);
        let rendered=sql_query.to_string();
        assert_eq!(rendered.matches("analytics\".\"notes_native").count(),2,"{rendered}");
        assert!(rendered.contains("FROM c AS notes JOIN (SELECT"),"table alias was treated as a CTE name: {rendered}");
        let mut qualified=query("WITH notes AS (SELECT note_id FROM notes) SELECT n.note_id FROM umf.notes AS n");
        let mut lower=Lower{native_tables:&native_tables,table_columns:&table_columns,cte_scopes:table_query::CteScopes::default(),schema:"analytics",params:&params,carriers:&carriers,referenced_names:BTreeSet::new(),error:None};
        let _=VisitMut::visit(&mut qualified,&mut lower);
        assert_eq!(lower.error,None);
        assert!(qualified.to_string().contains("FROM (SELECT"),"qualified module relation was misclassified as a CTE");
    }

    #[test]
    fn sort_alias_is_deterministically_advanced_on_identifier_and_output_collisions() {
        let mut source=query("SELECT body AS __weft_sort_0_1 FROM notes WHERE __weft_sort_0_0 = '__weft_sort_0_2' ORDER BY __weft_sort_0_3");
        let native_tables=[("notes".to_owned(),"notes".to_owned())].into_iter().collect();
        let table_columns=[("notes".to_owned(),vec![("body".to_owned(),"body".to_owned())])].into_iter().collect();
        let params=BTreeMap::new();let carriers=BTreeMap::new();
        let mut lower=Lower{native_tables:&native_tables,table_columns:&table_columns,cte_scopes:table_query::CteScopes::default(),schema:"analytics",params:&params,carriers:&carriers,referenced_names:BTreeSet::new(),error:None};
        let _=VisitMut::visit(&mut source,&mut lower);
        assert_eq!(lower.error,None);
        assert!(lower.referenced_names.contains("__weft_sort_0_0"));
        assert!(lower.referenced_names.contains("__weft_sort_0_3"));
        assert!(!lower.referenced_names.contains("__weft_sort_0_2"),"string-literal text does not reserve an alias");
        let output_expr=Expr::Identifier(Ident::new("body"));
        let outputs=[("__weft_sort_0_1".to_owned(),&output_expr)].into_iter().collect();
        assert_eq!(unused_sort_alias(0,&lower.referenced_names,&outputs).unwrap(),"__weft_sort_0_2");
        assert_eq!(unused_sort_alias(1,&lower.referenced_names,&outputs).unwrap(),"__weft_sort_1_0");
    }

    #[test]
    fn distinct_sort_membership_uses_exact_expression_identity() {
        let source=query("SELECT n.amount + 1 AS total FROM notes n");
        let SetExpr::Select(select)=source.body.as_ref() else {panic!("expected select")};
        let SelectItem::ExprWithAlias{expr,..}=&select.projection[0] else {panic!("expected aliased expression")};
        let projected=[("total".into(),expr.clone())];
        let index=exact_projected_expressions(&projected);
        assert!(index.contains(&expr));
        let other=query("SELECT n.amount + 2 FROM notes n");
        let SetExpr::Select(other_select)=other.body.as_ref() else {panic!("expected select")};
        let SelectItem::UnnamedExpr(other_expr)=&other_select.projection[0] else {panic!("expected expression")};
        assert!(!index.contains(&other_expr),"DISTINCT sort membership must retain exact AST equality");
    }

    #[test]
    fn identity_columns_preserve_base_relation_for_primary_key_grouping() {
        let mut sql_query=query("SELECT n.note_id, n.body, COUNT(*) AS count FROM notes AS n GROUP BY n.note_id");
        let native_tables=[("notes".to_owned(),"notes_native".to_owned())].into_iter().collect();
        let table_columns=[("notes".to_owned(),vec![("note_id".to_owned(),"note_id".to_owned()),("body".to_owned(),"body".to_owned())])].into_iter().collect();
        let params=BTreeMap::new();let carriers=BTreeMap::new();
        let mut lower=Lower{native_tables:&native_tables,table_columns:&table_columns,cte_scopes:table_query::CteScopes::default(),schema:"analytics",params:&params,carriers:&carriers,referenced_names:BTreeSet::new(),error:None};
        let _=VisitMut::visit(&mut sql_query,&mut lower);
        assert_eq!(lower.error,None);
        let rendered=sql_query.to_string();
        assert!(rendered.contains("FROM \"analytics\".\"notes_native\" AS n"),"{rendered}");
        assert!(!rendered.contains("FROM (SELECT"),"{rendered}");
        assert!(rendered.ends_with("GROUP BY n.note_id"),"{rendered}");
    }

    #[test]
    fn owned_projection_exposes_only_logical_column_names() {
        let query=owned_projection_query("analytics%", "notes_native", &[("note_id".into(),"id_physical".into()),("owner".into(),"owner_physical".into())]).unwrap();
        let sql=query.to_string();
        assert!(sql.contains("\"analytics%\".\"notes_native\""),"{sql}");
        assert!(sql.contains("\"__weft_source\".\"id_physical\" AS \"note_id\""),"{sql}");
        assert!(sql.contains("\"__weft_source\".\"owner_physical\" AS \"owner\""),"{sql}");
        assert!(!sql.contains(" AS \"id_physical\""),"native spelling leaked into derived output: {sql}");
    }

    #[test]
    fn typed_literals_and_collations_have_a_closed_profile() {
        let date=query("SELECT DATE '2025-01-01'");
        let SetExpr::Select(select)=date.body.as_ref() else {panic!("expected select")};
        let SelectItem::UnnamedExpr(Expr::TypedString(typed))=&select.projection[0] else {panic!("expected typed string")};
        assert!(validate_typed_string(typed).is_ok());
        let mut statements=Parser::parse_sql(&PostgreSqlDialect{},"SELECT x COLLATE \"C\"").unwrap();
        let Statement::Query(q)=statements.remove(0) else {panic!("expected query")};
        let SetExpr::Select(select)=q.body.as_ref() else {panic!("expected select")};
        let SelectItem::UnnamedExpr(Expr::Collate{collation,..})=&select.projection[0] else {panic!("expected collate")};
        let mut collation=collation.clone();
        assert!(qualify_collation(&mut collation).is_ok());
        assert_eq!(collation.to_string(),"pg_catalog.\"C\"");
        let mut statements=Parser::parse_sql(&PostgreSqlDialect{},"SELECT x COLLATE private_custom").unwrap();
        let Statement::Query(q)=statements.remove(0) else {panic!("expected query")};
        let SetExpr::Select(select)=q.body.as_ref() else {panic!("expected select")};
        let SelectItem::UnnamedExpr(Expr::Collate{collation,..})=&select.projection[0] else {panic!("expected collate")};
        let mut collation=collation.clone();
        assert!(qualify_collation(&mut collation).is_err());
    }

}
