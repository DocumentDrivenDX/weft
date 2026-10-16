//! Raw Spark/Delta lowering from the same core-admitted read AST.
//! Administrative local profile; no Unity Catalog service qualification.
use serde_json::{json, Value as J};
use sqlparser::ast::*;
use std::collections::BTreeSet;
use weft_core::raw_read::{AdmittedRead, Carrier, Request, Table};
type Result<T> = std::result::Result<T, &'static str>;
fn ident(s: &str) -> Result<String> {
    if s.is_empty() || s.len() > 255 || s.contains('\0') {
        return Err("bounded native identifier required");
    };
    Ok(format!("`{}`", s.replace('`', "``")))
}
fn name(i: &Ident) -> String {
    if i.quote_style.is_some() {
        i.value.clone()
    } else {
        i.value.to_ascii_lowercase()
    }
}
struct Lower<'a> {
    request: &'a Request,
    table: &'a Table,
    alias: String,
    slots: Vec<String>,
    used: BTreeSet<String>,
}
impl Lower<'_> {
    fn column(&self, e: &Expr) -> Result<(String, Carrier, bool)> {
        let Expr::CompoundIdentifier(parts) = e else {
            return Err("qualified scalar Field required");
        };
        if parts.len() != 2 || name(&parts[0]) != self.alias {
            return Err("Field owner differs");
        };
        let field = name(&parts[1]);
        let c = self
            .table
            .columns
            .iter()
            .find(|c| c.logical == field)
            .ok_or("Field binding absent")?;
        Ok((
            format!("{}.{}", ident(&self.alias)?, ident(&c.native)?),
            c.carrier.clone(),
            c.nullable,
        ))
    }
    fn scalar(&mut self, e: &Expr) -> Result<(String, Carrier, bool)> {
        match e {
            Expr::Nested(e) => self.scalar(e),
            Expr::CompoundIdentifier(_) => self.column(e),
            Expr::Value(v) => match &v.value {
                Value::Placeholder(p) => {
                    let key = p.strip_prefix(':').ok_or("named parameter required")?;
                    let param = self
                        .request
                        .parameters
                        .iter()
                        .find(|p| p.name == key)
                        .ok_or("parameter absent")?;
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
                        s.clone(),
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
            _ => Err("scalar operation unsupported"),
        }
    }
    fn predicate(&mut self, e: &Expr) -> Result<String> {
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
                Ok(format!("({} {op} {})", a.0, b.0))
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
                if list.is_empty() {
                    return Ok(if *negated { "TRUE" } else { "FALSE" }.into());
                };
                let a = self.scalar(expr)?;
                let mut values = vec![];
                for e in list {
                    let b = self.scalar(e)?;
                    if a.1 != b.1 {
                        return Err("membership carrier differs");
                    };
                    values.push(b.0)
                }
                Ok(format!(
                    "({} {}IN ({}))",
                    a.0,
                    if *negated { "NOT " } else { "" },
                    values.join(", ")
                ))
            }
            _ => Err("predicate outside raw Spark profile"),
        }
    }
}
fn lower(a: &AdmittedRead) -> Result<J> {
    let r = a.request();
    let q = a.query();
    if r.version != "weft.raw-read/0.1"
        || r.target != "spark-delta4"
        || r.binding.schema.is_none()
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
    if !group.is_empty() || !modifiers.is_empty() {
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
    if !with_hints.is_empty() || !partitions.is_empty() || !index_hints.is_empty() {
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
    if table
        .columns
        .iter()
        .any(|c| matches!(c.carrier, Carrier::Integer | Carrier::Decimal))
    {
        return Err("numeric raw Spark binding requires separately qualified shape profile");
    };
    let snapshots = r
        .snapshots
        .as_ref()
        .ok_or("explicit compiled snapshot versions required")?;
    if snapshots.len() != 1 {
        return Err("exact snapshot inventory required");
    };
    let version = snapshots
        .get(&table.native)
        .filter(|v| **v <= i64::MAX as u64)
        .ok_or("exact native snapshot version required")?;
    let mut l = Lower {
        request: r,
        table,
        alias: name(&alias.name),
        slots: vec![],
        used: BTreeSet::new(),
    };
    let mut outputs = vec![];
    let mut projection = vec![];
    let mut names = BTreeSet::new();
    if s.projection.is_empty() || s.projection.len() > 256 {
        return Err("projection budget");
    };
    for item in &s.projection {
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
        let c = l.column(e)?;
        projection.push(format!("{} AS {}", c.0, ident(&out)?));
        outputs.push(json!({"name":out,"carrier":c.1,"nullable":c.2}));
    }
    let mut sql = format!(
        "SELECT {} FROM {}.{} VERSION AS OF {version} AS {}",
        projection.join(", "),
        ident(r.binding.schema.as_ref().unwrap())?,
        ident(&table.native)?,
        ident(&l.alias)?
    );
    if let Some(e) = &s.selection {
        sql += &format!(" WHERE {}", l.predicate(e)?)
    };
    if let Some(order) = &q.order_by {
        let OrderByKind::Expressions(items) = &order.kind else {
            return Err("static ordering required");
        };
        let mut parts = vec![];
        for i in items {
            if i.with_fill.is_some() {
                return Err("ordering modifier unsupported");
            };
            let c = l.column(&i.expr)?;
            parts.push(format!(
                "{} {} NULLS {}",
                c.0,
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
    let native = format!(
        "{}.{} VERSION AS OF {version}",
        ident(r.binding.schema.as_ref().unwrap())?,
        ident(&table.native)?
    );
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
    let mut out = json!({"version":"weft.raw-read/0.1","status":"compiled","target":r.target,"sql":sql,"parameters":r.parameters,"parameterOrder":l.slots,"outputs":outputs,"binding":r.binding,"sourceSql":r.sql,"nativeChecks":[{"sql":check,"expected":0}],"snapshotTables":[{"table":table.native,"version":version}],"lockSql":[],"cursorParameters":[],"cursorOutputs":[],"cursorOrdering":[],"referencedTables":[{"logical":table.logical,"native":table.native,"columns":table.columns}],"obligations":{"transaction":"snapshot-read","principal":"administrative-single-tenant","ansiEnabled":true,"sparkVersion":"4.0.1","deltaVersion":"4.0.0","binaryStringCollation":true,"escapedStringLiterals":false,"rowCap":limit,"timeout":"one-call-deadline","tableIdentity":"uuid-and-version"}});
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
                2 => r["keyset"] = json!(true),
                3 => r["sql"] = json!("DELETE FROM sales.order"),
                _ => r["target"] = json!("unity-catalog"),
            };
            let a = output(r);
            assert_eq!(a["status"], "blocked");
            assert!(a.get("sql").is_none());
        }
    }
}
