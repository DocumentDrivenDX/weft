//! Raw Spark/Delta lowering from the same core-admitted read AST.
//! Administrative local profile; no Unity Catalog service qualification.
use serde_json::{json, Value as J};
use sqlparser::ast::*;
use std::collections::{BTreeMap,BTreeSet};
use weft_core::raw_read::{AdmittedRead, Carrier, Request, Table};
type Result<T> = std::result::Result<T, &'static str>;
pub(crate) fn list_parameters(request:&Request)->Option<J>{
    let parameters:Vec<_>=request.parameters.iter().filter(|p|p.container.is_some()).map(|p|json!({"name":p.name,"carrier":p.carrier,"nullable":false,"elementNullable":p.element_nullable,"maxItems":p.max_items})).collect();
    if parameters.is_empty(){None}else{Some(json!({"version":"weft.spark-list-json/0.1","parameters":parameters,"maximumElementBytes":10000,"maximumEncodedBytes":1000000,"format":"typed-json-array"}))}
}
pub(crate) fn ident(s: &str) -> Result<String> {
    if s.is_empty() || s.len() > 255 || s.contains('\0') {
        return Err("bounded native identifier required");
    };
    Ok(format!("`{}`", s.replace('`', "``")))
}
pub(crate) fn name(i: &Ident) -> String {
    if i.quote_style.is_some() {
        i.value.clone()
    } else {
        i.value.to_ascii_lowercase()
    }
}
pub(crate) struct Lower<'a> {
    pub(crate) request: &'a Request,
    pub(crate) table: &'a Table,
    pub(crate) alias: String,
    pub(crate) scans: Option<BTreeMap<String,(&'a Table,bool)>>,
    pub(crate) slots: Vec<String>,
    pub(crate) used: BTreeSet<String>,
}
impl Lower<'_> {
    pub(crate) fn column(&self, e: &Expr) -> Result<(String, Carrier, bool)> {
        let Expr::CompoundIdentifier(parts)=e else{return Err("qualified scalar Field required")};
        if parts.len()!=2{return Err("qualified scalar Field required")};
        let alias=name(&parts[0]);let field=name(&parts[1]);
        let (table,nullable)=if let Some(scans)=&self.scans {*scans.get(&alias).ok_or("Field owner differs")?}else{if alias!=self.alias{return Err("Field owner differs")};(self.table,false)};
        let column=table.columns.iter().find(|c|c.logical==field).ok_or("Field binding absent")?;
        Ok((format!("{}.{}",ident(&alias)?,ident(&column.native)?),column.carrier.clone(),column.nullable||nullable))
    }
    pub(crate) fn scalar(&mut self, e: &Expr) -> Result<(String, Carrier, bool)> {
        match e {
            Expr::Nested(e) => self.scalar(e),
            Expr::CompoundIdentifier(_) => {
                let (sql,carrier,nullable)=self.column(e)?;
                Ok((if self.exact_numeric(&carrier){format!("CAST({sql} AS STRING)")}else{sql},carrier,nullable))
            },
            Expr::Value(v) => match &v.value {
                Value::Placeholder(p) => {
                    let key = p.strip_prefix(':').ok_or("named parameter required")?;
                    let param = self
                        .request
                        .parameters
                        .iter()
                        .find(|p| p.name == key)
                        .ok_or("parameter absent")?;
                    if param.container.is_some(){return Err("list parameter is only a complete IN/NOT IN membership operand")}
                    self.used.insert(key.into());
                    self.slots.push(key.into());
                    Ok((
                        format!(":p{}", self.slots.len()),
                        param.carrier.clone(),
                        param.nullable,
                    ))
                }
                Value::SingleQuotedString(s) => {
                    if s.contains('\0') {
                        return Err("literal NUL");
                    };
                    Ok((
                        format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'")),
                        Carrier::String,
                        false,
                    ))
                }
                Value::Boolean(b) => Ok((
                    if *b { "TRUE" } else { "FALSE" }.into(),
                    Carrier::Boolean,
                    false,
                )),
                Value::Number(s, false) => {
                    if !s
                        .bytes()
                        .all(|b| b.is_ascii_digit() || matches!(b, b'-' | b'+' | b'.'))
                    {
                        return Err("exact numeric literal required");
                    };
                    Ok((
                        if self.request.binding.native_columns.is_some(){format!("'{s}'")}else{s.clone()},
                        if s.contains('.') {
                            Carrier::Decimal
                        } else {
                            Carrier::Integer
                        },
                        false,
                    ))
                }
                _ => Err("literal carrier unsupported"),
            },
            Expr::UnaryOp {op:UnaryOperator::Minus,expr} if self.request.binding.native_columns.is_some()=>{
                let Expr::Value(v)=expr.as_ref() else{return Err("numeric unary operation requires literal")};
                let Value::Number(s,false)=&v.value else{return Err("numeric unary operation requires literal")};
                if s.is_empty()||!s.bytes().all(|b|b.is_ascii_digit()||b==b'.'){return Err("plain numeric literal required")}
                Ok((format!("'-{s}'"),if s.contains('.') {Carrier::Decimal}else{Carrier::Integer},false))
            },
            _ => Err("scalar operation unsupported"),
        }
    }
    fn exact_numeric(&self, carrier:&Carrier)->bool {
        self.request.binding.native_columns.is_some()&&matches!(carrier,Carrier::Integer|Carrier::Decimal)
    }
    fn comparison(&self,a:&(String,Carrier,bool),op:&str,b:&(String,Carrier,bool))->String {
        if self.exact_numeric(&a.1){crate::raw_numeric::comparison(&a.0,op,&b.0)}
        else{format!("({} {op} {})",a.0,b.0)}
    }
    pub(crate) fn predicate(&mut self, e: &Expr) -> Result<String> {
        match e {
            Expr::Nested(e) => Ok(format!("({})", self.predicate(e)?)),
            Expr::BinaryOp { left, op, right }
                if matches!(op, BinaryOperator::And | BinaryOperator::Or) =>
            {
                Ok(format!(
                    "({} {op} {})",
                    self.predicate(left)?,
                    self.predicate(right)?
                ))
            }
            Expr::BinaryOp { left, op, right }
                if matches!(
                    op,
                    BinaryOperator::Eq
                        | BinaryOperator::NotEq
                        | BinaryOperator::Gt
                        | BinaryOperator::GtEq
                        | BinaryOperator::Lt
                        | BinaryOperator::LtEq
                ) =>
            {
                let a = self.scalar(left)?;
                let b = self.scalar(right)?;
                if a.1 != b.1 {
                    return Err("comparison carrier differs");
                };
                Ok(self.comparison(&a,&op.to_string(),&b))
            }
            Expr::IsNull(e) => Ok(format!("({} IS NULL)", self.scalar(e)?.0)),
            Expr::IsNotNull(e) => Ok(format!("({} IS NOT NULL)", self.scalar(e)?.0)),
            Expr::UnaryOp {
                op: UnaryOperator::Not,
                expr,
            } => Ok(format!("NOT ({})", self.predicate(expr)?)),
            Expr::Between {
                expr,
                negated,
                low,
                high,
            } => {
                let a = self.scalar(expr)?;
                let b = self.scalar(low)?;
                let c = self.scalar(high)?;
                if a.1 != b.1 || a.1 != c.1 {
                    return Err("range carrier differs");
                };
                if self.exact_numeric(&a.1){
                    let range=format!("({} AND {})",self.comparison(&a,">=",&b),self.comparison(&a,"<=",&c));
                    return Ok(if *negated{format!("NOT ({range})")}else{range})
                }
                Ok(format!(
                    "({} {}BETWEEN {} AND {})",
                    a.0,
                    if *negated { "NOT " } else { "" },
                    b.0,
                    c.0
                ))
            }
            Expr::InList {
                expr,
                list,
                negated,
            } => {
                if list.len() > 1000 {
                    return Err("membership limit");
                };
                let a = self.scalar(expr)?;
                if let [Expr::Value(value)] = list.as_slice() {
                    if let Value::Placeholder(placeholder) = &value.value {
                        let key=placeholder.strip_prefix(':').ok_or("named list parameter required")?;
                        let parameter=self.request.parameters.iter().find(|p|p.name==key).ok_or("list parameter absent")?;
                        if parameter.container.is_some() {
                            if self.request.target!="spark-delta4"||parameter.carrier!=a.1{return Err("list membership carrier or target differs")}
                            if matches!(a.1,Carrier::Integer|Carrier::Decimal)&&!self.exact_numeric(&a.1){return Err("exact numeric list requires original native ColumnSet custody")}
                            self.used.insert(key.to_owned());self.slots.push(key.to_owned());
                            let native_type=if parameter.carrier==Carrier::Boolean{"ARRAY<BOOLEAN>"}else{"ARRAY<STRING>"};
                            let items=format!("from_json(:p{},'{native_type}',map('mode','FAILFAST'))",self.slots.len());
                            let comparison=if self.exact_numeric(&a.1){crate::raw_numeric::comparison("__umf_list.lhs","=","__umf_item")}else if a.1==Carrier::String{"((__umf_list.lhs COLLATE UTF8_BINARY) = (__umf_item COLLATE UTF8_BINARY))".to_owned()}else{"(__umf_list.lhs = __umf_item)".to_owned()};
                            let membership=format!("element_at(transform(array(named_struct('lhs',{},'items',{items})),__umf_list -> CASE WHEN size(__umf_list.items)=0 THEN FALSE WHEN __umf_list.lhs IS NULL THEN CAST(NULL AS BOOLEAN) ELSE exists(__umf_list.items,__umf_item -> __umf_item IS NOT NULL AND {comparison}) END),1)",a.0);
                            return Ok(if *negated{format!("NOT ({membership})")}else{membership});
                        }
                    }
                }
                if list.is_empty() {
                    return Ok(if *negated { "TRUE" } else { "FALSE" }.into());
                };
                let mut values = vec![];
                for e in list {
                    if matches!(e,Expr::Value(value) if matches!(value.value,Value::Null)){continue}
                    let b = self.scalar(e)?;
                    if a.1 != b.1 {
                        return Err("membership carrier differs");
                    };
                    values.push(format!("({} IS NOT NULL AND {})",b.0,self.comparison(&a,"=",&b)))
                }
                let membership=format!("(CASE WHEN {} IS NULL THEN CAST(NULL AS BOOLEAN) ELSE (FALSE{}) END)",a.0,values.iter().map(|value|format!(" OR {value}")).collect::<String>());
                Ok(if *negated{format!("NOT ({membership})")}else{membership})
            }
            Expr::Function(f) if f.name.0.len()==1&&matches!(&f.name.0[0],ObjectNamePart::Identifier(i) if name(i)=="in_empty")=>{
                if f.uses_odbc_syntax||!matches!(f.parameters,FunctionArguments::None)||!f.within_group.is_empty()||f.filter.is_some()||f.null_treatment.is_some()||f.over.is_some(){return Err("IN_EMPTY modifiers refused")}
                let FunctionArguments::List(args)=&f.args else{return Err("IN_EMPTY field argument required")};
                if args.duplicate_treatment.is_some()||!args.clauses.is_empty()||args.args.len()!=1{return Err("IN_EMPTY requires exactly one Field")}
                let FunctionArg::Unnamed(FunctionArgExpr::Expr(field))=&args.args[0] else{return Err("IN_EMPTY Field required")};
                self.column(field)?;
                Ok("FALSE".into())
            }
            _ => Err("predicate outside raw Spark profile"),
        }
    }
}
fn lower(a: &AdmittedRead) -> Result<J> {if a.request().keyset {crate::raw_pages::lower(a)}else{lower_inner(a,false,None,None,false,a.request().binding.native_columns.is_some())}}
pub(crate) fn lower_transaction(a:&AdmittedRead)->Result<J>{ lower_inner(a,true,None,None,false,false) }
pub(crate) fn lower_current(a:&AdmittedRead,catalog:&str)->Result<J>{lower_inner(a,false,Some(catalog),None,false,false)}
// Only sibling compiler admission may supply these already validated projections.
pub(crate) fn lower_current_projected(a:&AdmittedRead,catalog:&str,projection:&[(String,J)])->Result<J>{
    lower_inner(a,false,Some(catalog),Some(projection),true,false)
}
pub(crate) fn lower_current_shaped(a:&AdmittedRead,catalog:&str)->Result<J>{lower_inner(a,false,Some(catalog),None,true,false)}
pub(crate) fn lower_managed_transaction(a:&AdmittedRead,catalog:&str)->Result<J>{lower_inner(a,true,Some(catalog),None,false,false)}
fn lower_inner(a: &AdmittedRead, transaction:bool, current_catalog:Option<&str>, current_projection:Option<&[(String,J)]>, current_shapes:bool,spark_shapes:bool) -> Result<J> {
    let r = a.request();
    if matches!(&r.binding.schema_binding,Some(weft_core::raw_read::SchemaBinding::SearchPath)) {return Err("search-path schema binding belongs to PostgreSQL profile")}
    if matches!(&r.binding.schema_binding,Some(weft_core::raw_read::SchemaBinding::Qualified)) && r.binding.schema.is_none(){return Err("qualified schema binding requires explicit schema")}
    if r.binding.native_columns.is_some()&&!spark_shapes{return Err("native ColumnSet envelope not admitted by this Databricks profile")}
    if r.stream_page_size {return Err("stream page size requires explicit snapshot page profile")}

    let q = a.query();
    if r.version != "weft.raw-read/0.1"
        || r.target != if current_catalog.is_some(){if transaction{"databricks-managed-delta"}else{"unity-catalog-current"}}else{"spark-delta4"}
        || (!transaction && r.binding.schema.is_none())
        || r.keyset
    {
        return Err("explicit raw Spark4 qualified-schema non-page profile required");
    };
    if r.row_cap < 1 || r.row_cap > 1000 || r.sql.len() > 65536 || r.parameters.len() > 64 {
        return Err("read budget");
    };
    if q.with.is_some()
        || q.fetch.is_some()
        || !q.locks.is_empty()
        || q.for_clause.is_some()
        || q.settings.is_some()
        || q.format_clause.is_some()
        || !q.pipe_operators.is_empty()
    {
        return Err("query modifier unsupported");
    };
    let SetExpr::Select(s) = q.body.as_ref() else {
        return Err("single SELECT required");
    };
    if s.from.len() != 1
        || !s.from[0].joins.is_empty()
        || s.distinct.is_some()
        || s.top.is_some()
        || s.exclude.is_some()
        || s.into.is_some()
        || !s.lateral_views.is_empty()
        || s.prewhere.is_some()
        || !s.connect_by.is_empty()
        || !s.cluster_by.is_empty()
        || !s.distribute_by.is_empty()
        || !s.sort_by.is_empty()
        || !s.named_window.is_empty()
        || s.qualify.is_some()
        || s.value_table_mode.is_some()
        || s.having.is_some()
    {
        return Err("raw Spark SELECT shape unsupported");
    };
    let GroupByExpr::Expressions(group, modifiers) = &s.group_by else {
        return Err("grouping unsupported");
    };
    let aggregate=crate::raw_aggregates::requested(s);
    if aggregate&&(current_catalog.is_some()||transaction){return Err("Spark approximate aggregate belongs to original snapshot read profile")}
    if (!group.is_empty()&&!aggregate) || !modifiers.is_empty() {
        return Err("grouping unsupported");
    };
    let TableFactor::Table {
        name: object,
        alias: Some(alias),
        args: None,
        with_hints,
        version: None,
        with_ordinality: false,
        partitions,
        json_path: None,
        sample: None,
        index_hints,
    } = &s.from[0].relation
    else {
        return Err("ordinary aliased table required");
    };
    if !alias.columns.is_empty() || !with_hints.is_empty() || !partitions.is_empty() || !index_hints.is_empty() {
        return Err("table modifiers unsupported");
    };
    let logical = object
        .0
        .iter()
        .map(|p| match p {
            ObjectNamePart::Identifier(i) => Ok(name(i)),
            _ => Err("static logical names required"),
        })
        .collect::<Result<Vec<_>>>()?;
    let table = r
        .binding
        .tables
        .iter()
        .find(|t| t.logical == logical)
        .ok_or("bound source table absent")?;
    // Spark LONG has a finite native domain. Unbounded core integers must never
    // be silently narrowed by this backend. Decimal shapes require own profile.
    if !(current_catalog.is_some()&&current_shapes||spark_shapes) && table
        .columns
        .iter()
        .any(|c| matches!(c.carrier, Carrier::Integer | Carrier::Decimal))
    {
        return Err("numeric raw Spark binding requires separately qualified shape profile");
    };
    let (version,native)=if let Some(catalog)=current_catalog {
        if r.snapshots.is_some(){return Err("current managed selection cannot carry historical snapshots")}
        (None,format!("{}.{}.{}",ident(catalog)?,ident(r.binding.schema.as_ref().unwrap())?,ident(&table.native)?))
    }else if transaction {
        if r.snapshots.is_some(){return Err("transaction selection must use its original native transaction snapshot")}
        (None,"`__umf_delta_stage`".to_string())
    }else{
        let snapshots=r.snapshots.as_ref().ok_or("explicit compiled snapshot versions required")?;
        if snapshots.len()!=1{return Err("exact snapshot inventory required")}
        let version=*snapshots.get(&table.native).filter(|v|**v<=i64::MAX as u64).ok_or("exact native snapshot version required")?;
        (Some(version),format!("{}.{} VERSION AS OF {version}",ident(r.binding.schema.as_ref().unwrap())?,ident(&table.native)?))
    };
    let mut l = Lower {
        request: r,
        table,
        alias: name(&alias.name),
        scans: None,
        slots: vec![],
        used: BTreeSet::new(),
    };
    let shapes=if spark_shapes{Some(crate::spark_shapes::columns(a,table)?)}else{None};
    let mut outputs = vec![];
    let mut projection = vec![];
    let mut names = BTreeSet::new();
    if s.projection.is_empty() || s.projection.len() > 256 {
        return Err("projection budget");
    };
    let aggregation=if aggregate{Some(crate::raw_aggregates::lower(&l,s,shapes.is_some())?)}else{None};
    if let Some(a)=&aggregation{projection=a.sql.clone();outputs=a.outputs.clone();}
    else { for item in &s.projection {
        let (e, out) = match item {
            SelectItem::ExprWithAlias { expr, alias } => (expr, name(alias)),
            SelectItem::UnnamedExpr(e) => {
                let Expr::CompoundIdentifier(parts) = e else {
                    return Err("explicit projection alias required");
                };
                (e, name(parts.last().ok_or("empty Field")?))
            }
            _ => return Err("explicit scalar projection required"),
        };
        if !names.insert(out.clone()) {
            return Err("duplicate output");
        };
        if let Some(shapes)=&shapes {
            let Expr::CompoundIdentifier(parts)=e else{return Err("original direct Spark projection required")};
            if parts.len()!=2||name(&parts[0])!=l.alias{return Err("original Spark projection owner differs")};
            let shape=shapes.get(&name(&parts[1])).ok_or("projected Field outside original Spark ColumnSet")?;
            let mut meta=shape.metadata.clone();meta["name"]=json!(out);
            projection.push(format!("{}.{} AS {}",ident(&l.alias)?,ident(&shape.native)?,ident(&out)?));outputs.push(meta);continue;
        }
        let c = l.column(e)?;
        projection.push(format!("{} AS {}", c.0, ident(&out)?));
        outputs.push(json!({"name":out,"carrier":c.1,"nullable":c.2}));
    }
    }
    if let Some(selected)=current_projection {
        if current_catalog.is_none()||selected.is_empty()||selected.len()>256{return Err("bounded current projection required")}
        projection.clear();outputs.clear();let mut selected_names=BTreeSet::new();
        for (native,output) in selected {
            let out=output["name"].as_str().ok_or("named current projection required")?;
            if !selected_names.insert(out){return Err("duplicate current output")}
            projection.push(format!("{}.{} AS {}",ident(&l.alias)?,ident(native)?,ident(out)?));
            outputs.push(output.clone());
        }
    }
    let mut sql=format!("SELECT {} FROM {native} AS {}",projection.join(", "),ident(&l.alias)?);
    if let Some(e) = &s.selection {
        sql += &format!(" WHERE {}", l.predicate(e)?)
    };
    if let Some(a)=&aggregation{if !a.groups.is_empty(){sql+=&format!(" GROUP BY {}",a.groups.join(", "));}}
    if let Some(order) = &q.order_by {
        let OrderByKind::Expressions(items) = &order.kind else {
            return Err("static ordering required");
        };
        let mut parts = vec![];
        for i in items {
            if i.with_fill.is_some() {
                return Err("ordering modifier unsupported");
            };
            let order_sql=if let Some(a)=&aggregation{match &i.expr{Expr::Identifier(id)=>a.order.get(&name(id)).cloned().ok_or("aggregate ordering must name an original projected output")?,e=>{let c=l.column(e)?;if !a.groups.contains(&c.0){return Err("aggregate ordering Field must be grouped")};c.0}}}else{l.column(&i.expr)?.0};
            parts.push(format!(
                "{} {} NULLS {}",
                order_sql,
                match i.options.sort {
                    None | Some(OrderBySort::Asc) => "ASC",
                    Some(OrderBySort::Desc) => "DESC",
                    _ => return Err("sort operator unsupported"),
                },
                if i.options.nulls_first.unwrap_or(false) {
                    "FIRST"
                } else {
                    "LAST"
                }
            ));
        }
        if !parts.is_empty() {
            sql += &format!(" ORDER BY {}", parts.join(", "))
        }
    }
    let limit = match &q.limit_clause {
        None => r.row_cap,
        Some(LimitClause::LimitOffset {
            limit: Some(Expr::Value(v)),
            offset: None,
            limit_by,
        }) if limit_by.is_empty() => match &v.value {
            Value::Number(n, false) => n
                .parse::<u16>()
                .ok()
                .filter(|n| *n > 0 && *n <= r.row_cap)
                .ok_or("bounded limit required")?,
            _ => return Err("literal limit required"),
        },
        _ => return Err("offset unsupported"),
    };
    sql += &format!(" LIMIT {limit}");
    if l.used.len() != r.parameters.len() {
        return Err("unused or duplicate parameter");
    };
    let keys = table
        .key
        .iter()
        .map(|logical| {
            table
                .columns
                .iter()
                .find(|c| &c.logical == logical)
                .ok_or("complete bound Key required")
                .and_then(|c| ident(&c.native))
        })
        .collect::<Result<Vec<_>>>()?;
    if keys.is_empty() {
        return Err("complete Key required");
    };
    let nulls = keys
        .iter()
        .map(|k| format!("{k} IS NULL"))
        .collect::<Vec<_>>()
        .join(" OR ");
    let check=format!("SELECT COUNT(*) AS violations FROM (SELECT {} FROM {native} GROUP BY {} HAVING COUNT(*) <> 1 OR {nulls}) AS invalid_keys",keys.join(", "),keys.join(", "));
    let mut out = json!({"version":"weft.raw-read/0.1","status":"compiled","target":r.target,"sql":sql,"parameters":r.parameters,"parameterOrder":l.slots,"outputs":outputs,"binding":r.binding,"sourceSql":r.sql,"nativeChecks":[{"sql":check,"expected":0}],"snapshotTables":version.map(|v|vec![json!({"table":table.native,"version":v})]).unwrap_or_default(),"lockSql":[],"cursorParameters":[],"cursorOutputs":[],"cursorOrdering":[],"referencedTables":[{"logical":table.logical,"native":table.native,"columns":table.columns}],"obligations":{"transaction":"snapshot-read","principal":"administrative-single-tenant","ansiEnabled":true,"sparkVersion":"4.0.1","deltaVersion":"4.0.0","binaryStringCollation":true,"escapedStringLiterals":false,"rowCap":limit,"timeout":"one-call-deadline","tableIdentity":"uuid-and-version"}});
    if let Some(shapes)=&shapes {out["referencedTables"][0]["columns"]=json!(shapes.iter().map(|(logical,shape)|{let mut c=shape.metadata.clone();c["logical"]=json!(logical);c["native"]=json!(shape.native);c}).collect::<Vec<_>>());}
    if current_catalog.is_some(){
        out["version"]=json!("weft.unity-current-selection/0.1");
        out["obligations"]=json!({"transaction":"current-read","principal":"native-managed-current-principal","ansiEnabled":true,"binaryStringCollation":true,"escapedStringLiterals":false,"rowCap":limit,"timeout":"one-call-deadline","tableIdentity":"native-current-uuid","nativeAdmissionEachOperation":true,"temporalQualification":"held"});
    }
    if let Some(parameters)=crate::raw_numeric::parameters(r){out["obligations"]["numericPredicateParameters"]=parameters;}
    if let Some(parameters)=list_parameters(r){out["obligations"]["listPredicateParameters"]=parameters;}
    if transaction {out["version"]=json!("weft.delta-selection/0.1");out["obligations"]["transaction"]=json!("original-Delta4-optimistic-Serializable-whole-table");out["obligations"]["tableIdentity"]=json!("original-native-transaction-snapshot-view");}
    out["structuralDigest"] = json!(weft_core::json::sha256(out.to_string().as_bytes()));
    Ok(out)
}
pub fn compile_json(raw: &str) -> String {
    weft_core::raw_read::admit_json(raw).and_then(|a|lower(&a)).unwrap_or_else(|reason|json!({"version":"weft.raw-read/0.1","status":"blocked","diagnostics":[{"code":"WFT-RAW-SPARK","message":reason}]})).to_string()
}
#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> J {
        let original: J = serde_json::from_str(include_str!(
            "../../../tests/native-actions/action-request.json"
        ))
        .unwrap();
        let mut binding = original["binding"].clone();
        binding["schema"] = json!("sdk_spark");
        json!({"version":"weft.raw-read/0.1","target":"spark-delta4","sql":"SELECT o.id AS id, o.status AS status FROM sales.order AS o WHERE o.status = :status ORDER BY o.id ASC","parameters":[{"name":"status","carrier":"string","nullable":false}],"rowCap":10,"keyset":false,"binding":binding,"snapshots":{"orders":7}})
    }
    fn output(r: J) -> J {
        serde_json::from_str(&compile_json(&r.to_string())).unwrap()
    }
    #[test]
    fn original_hll_projection_preserves_algorithm_guarantee_and_group_scope(){
        for sql in ["SELECT APPROX_COUNT_DISTINCT(o.status) AS estimated FROM sales.order o", "SELECT o.status AS status, APPROX_COUNT_DISTINCT(o.id) AS estimated FROM sales.order o GROUP BY o.status ORDER BY estimated DESC LIMIT 3"]{
            let mut r=request();r["parameters"]=json!([]);r["sql"]=json!(sql);let a=output(r.clone());assert_eq!(a["status"],"compiled","{a}");assert_eq!(a["sourceSql"],r["sql"]);let estimated=a["outputs"].as_array().unwrap().last().unwrap();assert_eq!(estimated["carrier"],"integer");assert_eq!(estimated["nullable"],false);assert_eq!(estimated["approximation"]["algorithm"],"HyperLogLog++");assert_eq!(estimated["approximation"]["error"],json!({"kind":"relative-standard-deviation","value":"0.05","hardBound":false}));assert!(a["sql"].as_str().unwrap().contains("approx_count_distinct("));assert!(!a["sql"].as_str().unwrap().contains("FROM (SELECT"));
        }
        for sql in ["SELECT o.id AS id, APPROX_COUNT_DISTINCT(o.status) AS estimated FROM sales.order o", "SELECT APPROX_COUNT_DISTINCT(DISTINCT o.id) AS estimated FROM sales.order o", "SELECT APPROX_COUNT_DISTINCT(o.id,0.01) AS estimated FROM sales.order o", "SELECT APPROX_COUNT_DISTINCT(o.id) FILTER(WHERE o.id IS NOT NULL) AS estimated FROM sales.order o", "SELECT APPROX_COUNT_DISTINCT(o.id) OVER() AS estimated FROM sales.order o", "SELECT APPROX_COUNT_DISTINCT(other.id) AS estimated FROM sales.order o", "SELECT APPROX_COUNT_DISTINCT(o.id) AS estimated FROM sales.order o ORDER BY o.status"]{let mut r=request();r["parameters"]=json!([]);r["sql"]=json!(sql);assert_ne!(output(r)["status"],"compiled","{sql}");}
    }
    #[test]
    fn snapshot_is_compiler_owned_and_arguments_stay_bound() {
        let r = request();
        let a = output(r.clone());
        assert_eq!(a, output(r));
        assert_eq!(a["status"], "compiled");
        assert!(a["sql"].as_str().unwrap().contains("VERSION AS OF 7"));
        assert!(a["sql"].as_str().unwrap().contains(":p1"));
        assert_eq!(a["parameterOrder"], json!(["status"]));
        assert_eq!(a["snapshotTables"], json!([{"table":"orders","version":7}]));
        assert!(a["nativeChecks"][0]["sql"]
            .as_str()
            .unwrap()
            .contains("GROUP BY `id` HAVING COUNT(*) <> 1"));
    }
    #[test]
    fn unsupported_snapshot_and_query_meaning_refuse() {
        for mode in 0..5 {
            let mut r = request();
            match mode {
                0 => {
                    r.as_object_mut().unwrap().remove("snapshots");
                }
                1 => r["snapshots"]["orders"] = json!(u64::MAX),
                2 => r["sql"] = json!("SELECT o.id AS id FROM sales.order o ORDER BY o.id LIMIT 10 OFFSET 1"),
                3 => r["sql"] = json!("DELETE FROM sales.order"),
                _ => r["target"] = json!("unity-catalog"),
            };
            let a = output(r);
            assert_eq!(a["status"], "blocked");
            assert!(a.get("sql").is_none());
        }
    }
}
