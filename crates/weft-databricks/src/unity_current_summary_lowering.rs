// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exact finite-current summary AST lowering; separate from Source11 HLL donor.
use super::raw_aggregates::Projection;
use super::raw_read::{ident, name, Lower};
use serde_json::{json, Value};
use sqlparser::ast::*;
use std::collections::{BTreeMap, BTreeSet};
use weft_core::raw_read::Carrier;
type Result<T> = std::result::Result<T, &'static str>;

/// Exact, finite-current aggregate projection. This deliberately has no
/// approximation, FILTER, window, arbitrary function or numeric input arm.
pub(crate) fn lower_current_exact(l: &Lower<'_>, s: &Select) -> Result<Projection> {
    let GroupByExpr::Expressions(groups, modifiers) = &s.group_by else {
        return Err("exact summary grouping expression required");
    };
    if !modifiers.is_empty() || groups.len() > 3 {
        return Err("exact summary group budget exceeded");
    }
    let mut group_fields = BTreeSet::new();
    let mut group_sql = Vec::new();
    let mut group_meta = BTreeMap::<String, (String, Carrier, bool)>::new();
    for group in groups {
        let (sql, carrier, nullable) = l.column(group)?;
        if !matches!(&carrier, Carrier::String | Carrier::Boolean) || !group_fields.insert(sql.clone()) {
            return Err("duplicate or unsupported exact group Field");
        }
        group_sql.push(if carrier == Carrier::String {
            format!("({sql} COLLATE UTF8_BINARY)")
        } else { sql.clone() });
        group_meta.insert(sql, (group_sql.last().unwrap().clone(), carrier, nullable));
    }
    let mut out = Projection { sql: Vec::new(), outputs: Vec::new(), groups: group_sql, order: BTreeMap::new() };
    let mut output_names = BTreeSet::new();
    let mut saw_measure = false;
    if s.projection.is_empty() || s.projection.len() > 8 { return Err("exact summary output bound required"); }
    for (ordinal, item) in s.projection.iter().enumerate() {
        let SelectItem::ExprWithAlias { expr, alias } = item else { return Err("generated summary aliases required"); };
        let alias = name(alias);
        if alias != format!("__weft_out_{ordinal}") || !output_names.insert(alias.clone()) {
            return Err("generated summary alias order differs");
        }
        match expr {
            Expr::Function(f) => {
                if f.name.0.len() != 1
                    || !matches!(&f.name.0[0], ObjectNamePart::Identifier(i) if name(i) == "count")
                    || f.uses_odbc_syntax || !matches!(f.parameters, FunctionArguments::None)
                    || !f.within_group.is_empty() || f.filter.is_some() || f.null_treatment.is_some() || f.over.is_some()
                { return Err("closed exact COUNT expression required"); }
                let FunctionArguments::List(args) = &f.args else { return Err("COUNT arguments required"); };
                if !args.clauses.is_empty() || args.args.len() != 1 {
                    return Err("modified or multi-argument COUNT refused");
                }
                let count_sql = match &args.args[0] {
                    FunctionArg::Unnamed(FunctionArgExpr::Wildcard) if args.duplicate_treatment.is_none() => "COUNT(*)".to_owned(),
                    FunctionArg::Unnamed(FunctionArgExpr::Expr(field)) => {
                        if args.duplicate_treatment.as_ref() != Some(&DuplicateTreatment::Distinct) {
                            return Err("exact Field count requires explicit DISTINCT");
                        }
                        let (native, carrier, _) = l.column(field)?;
                        if !matches!(&carrier, Carrier::String | Carrier::Boolean) { return Err("COUNT DISTINCT requires selected STRING/BOOLEAN Field"); }
                        let operand = if carrier == Carrier::String { format!("({native} COLLATE UTF8_BINARY)") } else { native };
                        format!("COUNT(DISTINCT {operand})")
                    }
                    _ => return Err("COUNT supports only row count or single Field distinct count"),
                };
                out.sql.push(format!("CAST({count_sql} AS STRING) AS {}", ident(&alias)?));
                out.order.insert(alias.clone(), count_sql);
                out.outputs.push(json!({"name":alias,"carrier":"integer","nullable":false,"decoder":{"kind":"canonical-nonnegative-integer-text","nativeAggregateType":"BIGINT"}}));
                saw_measure = true;
            }
            Expr::CompoundIdentifier(_) => {
                let (native, carrier, nullable) = l.column(expr)?;
                let Some((group_expr, _, _)) = group_meta.get(&native) else { return Err("projected nonaggregate Field must be grouped"); };
                out.sql.push(format!("{} AS {}", group_expr, ident(&alias)?));
                out.order.insert(alias.clone(), group_expr.clone());
                let mut meta = json!({"name":alias,"role":"group","carrier":carrier_name(&carrier),"nullable":nullable,"decoder":{"kind":if carrier == Carrier::String {"native-string"} else {"native-boolean"}}});
                if carrier == Carrier::String { meta["collation"] = json!("UTF8_BINARY"); }
                out.outputs.push(meta);
            }
            _ => return Err("exact summary projection outside group/count union"),
        }
    }
    if !saw_measure { return Err("aggregate summary requires a count measure"); }
    if group_fields.len() != groups.len() { return Err("duplicate exact group Field"); }
    Ok(out)
}

fn carrier_name(carrier: &Carrier) -> &'static str {
    match carrier { Carrier::String => "string", Carrier::Boolean => "boolean", _ => "unsupported" }
}
