//! Closed Spark HLL++ projection ownership; no deterministic error bound claim.
use super::raw_read::{ident, name, Lower};
use serde_json::{json, Value};
use sqlparser::ast::*;
use std::collections::{BTreeMap, BTreeSet};
use weft_core::raw_read::Carrier;
type Result<T> = std::result::Result<T, &'static str>;
pub(crate) struct Projection {
    pub sql: Vec<String>,
    pub outputs: Vec<Value>,
    pub groups: Vec<String>,
    pub order: BTreeMap<String, String>,
}
pub(crate) fn requested(s: &Select) -> bool {
    s.projection.iter().any(|item| {
        matches!(
            item,
            SelectItem::ExprWithAlias {
                expr: Expr::Function(_),
                ..
            } | SelectItem::UnnamedExpr(Expr::Function(_))
        )
    })
}
pub(crate) fn lower(l: &Lower<'_>, s: &Select, original_shapes: bool) -> Result<Projection> {
    let GroupByExpr::Expressions(groups, modifiers) = &s.group_by else {
        return Err("explicit aggregate grouping required");
    };
    if !modifiers.is_empty() {
        return Err("aggregate grouping modifiers refused");
    }
    let mut grouped = BTreeSet::new();
    let mut group_sql = vec![];
    for e in groups {
        let c = l.column(e)?;
        if !grouped.insert(c.0.clone()) {
            return Err("duplicate grouping Field");
        };
        group_sql.push(c.0);
    }
    let mut out = Projection {
        sql: vec![],
        outputs: vec![],
        groups: group_sql,
        order: BTreeMap::new(),
    };
    let mut names = BTreeSet::new();
    let mut seen = false;
    for item in &s.projection {
        let SelectItem::ExprWithAlias { expr, alias } = item else {
            return Err("aggregate projections require explicit aliases");
        };
        let alias = name(alias);
        if !names.insert(alias.clone()) {
            return Err("duplicate aggregate output");
        }
        if let Expr::Function(f) = expr {
            if f.name.0.len() != 1
                || !matches!(&f.name.0[0],ObjectNamePart::Identifier(i) if name(i)=="approx_count_distinct")
                || f.uses_odbc_syntax
                || !matches!(f.parameters, FunctionArguments::None)
                || !f.within_group.is_empty()
                || f.filter.is_some()
                || f.null_treatment.is_some()
                || f.over.is_some()
            {
                return Err("closed APPROX_COUNT_DISTINCT Field function required");
            }
            let FunctionArguments::List(args) = &f.args else {
                return Err("single aggregate Field required");
            };
            if args.args.len() != 1
                || args.duplicate_treatment.is_some()
                || !args.clauses.is_empty()
            {
                return Err("single unmodified aggregate Field required");
            }
            let FunctionArg::Unnamed(FunctionArgExpr::Expr(e)) = &args.args[0] else {
                return Err("aggregate Field required");
            };
            let c = l.column(e)?;
            if matches!(c.1, Carrier::Integer | Carrier::Decimal) && !original_shapes {
                return Err(
                    "numeric approximate aggregate requires original native ColumnSet custody",
                );
            }
            let sql = format!("approx_count_distinct({}, 0.05)", c.0);
            out.sql.push(format!("{sql} AS {}", ident(&alias)?));
            out.order.insert(alias.clone(), sql);
            seen = true;
            out.outputs.push(json!({"name":alias,"carrier":"integer","nullable":false,"approximate":true,"approximation":{"algorithm":"HyperLogLog++","backend":"Apache Spark","backendVersion":"4.0.1","error":{"kind":"relative-standard-deviation","value":"0.05","hardBound":false},"documentation":"https://spark.apache.org/docs/4.0.1/api/sql/index.html#approx_count_distinct"}}));
        } else {
            let c = l.column(expr)?;
            if !grouped.contains(&c.0) {
                return Err("projected nonaggregate Field must be grouped");
            };
            out.sql.push(format!("{} AS {}", c.0, ident(&alias)?));
            out.order.insert(alias.clone(), c.0);
            out.outputs
                .push(json!({"name":alias,"carrier":c.1,"nullable":c.2}));
        }
    }
    if !seen {
        return Err("approximate aggregate required");
    };
    Ok(out)
}
