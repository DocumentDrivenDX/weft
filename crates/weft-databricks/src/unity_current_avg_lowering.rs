// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exact base-10^19 signed-limb quotient lowering for bounded Unity AVG.
//! This module is pure arithmetic; it does not authorize or execute native SQL.

const BASE: u128 = 10_000_000_000_000_000_000;
const MAX_POPULATION: u64 = i64::MAX as u64;

use serde_json::{json, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use sqlparser::ast::*;
use sqlparser::ast::Value as AstValue;
use super::raw_read::{ident, name, Lower};
use weft_core::raw_read::{AdmittedRead, Carrier};
type JsonResult<T> = std::result::Result<T, &'static str>;

/// Exact source-level reference for the emitted digit-state quotient. This is
/// also used by controls; it never converts the full coefficient to a number.
pub(crate) fn exact_average_text(
    coefficient: &str,
    population: u64,
    source_scale: u16,
    result_scale: u16,
) -> Result<String, &'static str> {
    if population == 0 || population > MAX_POPULATION || source_scale > 38 || result_scale > 1000 {
        return Err("WFT-CAPABILITY: AVG population or scale outside finite profile");
    }
    let (negative, digits) = coefficient.strip_prefix('-').map_or((false, coefficient), |v| (true, v));
    if digits.is_empty() || digits.len() > 57 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err("WFT-CAPABILITY: AVG coefficient malformed");
    }
    let digits = digits.trim_start_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let n = population as u128;
    let (mut kept, up) = if result_scale >= source_scale {
        let stream = format!("{}{}", digits, "0".repeat((result_scale - source_scale) as usize));
        let (q, r) = long_divide(&stream, n)?;
        let odd = q.as_bytes().last().is_some_and(|b| (*b - b'0') % 2 == 1);
        (q, r * 2 > n || (r * 2 == n && odd))
    } else {
        let (q, remainder) = long_divide(digits, n)?;
        let delta = (source_scale - result_scale) as usize;
        let padded = if q.len() <= delta { format!("{}{}", "0".repeat(delta + 1 - q.len()), q) } else { q };
        let split = padded.len() - delta;
        let kept = padded[..split].to_owned();
        let discarded = &padded[split..];
        let guard = discarded.as_bytes()[0] - b'0';
        let sticky = discarded.as_bytes()[1..].iter().any(|b| *b != b'0') || remainder != 0;
        let odd = kept.as_bytes().last().is_some_and(|b| (*b - b'0') % 2 == 1);
        (kept, guard > 5 || (guard == 5 && (sticky || odd)))
    };
    if up { kept = increment_digits(&kept)?; }
    let kept = kept.trim_start_matches('0');
    let kept = if kept.is_empty() { "0" } else { kept };
    let scale = result_scale as usize;
    let padded = if kept.len() <= scale { format!("{}{}", "0".repeat(scale + 1 - kept.len()), kept) } else { kept.to_owned() };
    let body = if scale == 0 { padded } else {
        let split = padded.len() - scale;
        format!("{}.{}", &padded[..split], &padded[split..])
    };
    if negative && kept != "0" { Ok(format!("-{body}")) } else { Ok(body) }
}

fn long_divide(digits: &str, denominator: u128) -> Result<(String, u128), &'static str> {
    if denominator == 0 || denominator > MAX_POPULATION as u128 || digits.is_empty()
        || !digits.bytes().all(|b| b.is_ascii_digit()) { return Err("WFT-CAPABILITY: invalid AVG long-division state"); }
    let mut quotient = String::with_capacity(digits.len());
    let mut remainder = 0u128;
    for byte in digits.bytes() {
        let z = remainder * 10 + (byte - b'0') as u128;
        let mut digit = 0u8;
        for candidate in (1..=9u8).rev() {
            if z >= (candidate as u128) * denominator { digit = candidate; break; }
        }
        remainder = z - (digit as u128) * denominator;
        quotient.push((b'0' + digit) as char);
        debug_assert!(remainder < denominator);
    }
    let quotient = quotient.trim_start_matches('0');
    Ok((if quotient.is_empty() { "0".to_owned() } else { quotient.to_owned() }, remainder))
}

fn increment_digits(digits: &str) -> Result<String, &'static str> {
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) { return Err("WFT-CAPABILITY: invalid AVG carry state"); }
    let mut out = digits.as_bytes().to_vec();
    let mut i = out.len();
    while i > 0 {
        i -= 1;
        if out[i] == b'9' { out[i] = b'0'; } else { out[i] += 1; return String::from_utf8(out).map_err(|_| "WFT-CAPABILITY: AVG carry encoding"); }
    }
    let mut result = String::with_capacity(digits.len() + 1);
    result.push('1');
    result.push_str(&String::from_utf8(out).map_err(|_| "WFT-CAPABILITY: AVG carry encoding")?);
    Ok(result)
}

/// Emit one native long-division pass. The result is a fixed struct with text
/// quotient and DECIMAL(38,0) remainder; later CTEs consume these aliases.
fn long_division_sql(stream: &str, population: &str) -> String {
    let z = format!("(10 * st.r + CAST(SUBSTRING({stream}, i, 1) AS DECIMAL(38,0)))");
    let digit = format!("CASE WHEN {z} >= 9 * {population} THEN 9 WHEN {z} >= 8 * {population} THEN 8 WHEN {z} >= 7 * {population} THEN 7 WHEN {z} >= 6 * {population} THEN 6 WHEN {z} >= 5 * {population} THEN 5 WHEN {z} >= 4 * {population} THEN 4 WHEN {z} >= 3 * {population} THEN 3 WHEN {z} >= 2 * {population} THEN 2 WHEN {z} >= {population} THEN 1 ELSE 0 END");
    format!("aggregate(sequence(1, LENGTH({stream})), named_struct('q', CAST('' AS STRING), 'r', CAST(0 AS DECIMAL(38,0))), (st, i) -> named_struct('q', CONCAT(st.q, CAST(({digit}) AS STRING)), 'r', {z} - CAST(({digit}) AS DECIMAL(38,0)) * {population}))")
}

#[cfg(test)]
mod exact_avg_reference_tests {
    use super::exact_average_text as avg;

    #[test]
    fn half_even_is_exact_for_both_signs_and_scales() {
        for (c, n, ss, ts, want) in [
            ("25", 1, 1, 0, "2"), ("35", 1, 1, 0, "4"),
            ("-25", 1, 1, 0, "-2"), ("-35", 1, 1, 0, "-4"),
            ("125", 1, 2, 1, "1.2"), ("135", 1, 2, 1, "1.4"),
            ("-125", 1, 2, 1, "-1.2"), ("-135", 1, 2, 1, "-1.4"),
            ("995", 1, 2, 1, "10.0"), ("1", 3, 0, 3, "0.333"),
            ("2", 3, 0, 3, "0.667"),
        ] {
            assert_eq!(avg(c, n, ss, ts).unwrap(), want, "{c}/{n} source-scale {ss} result-scale {ts}");
        }
        let thousand = avg("0", 7, 38, 1000).unwrap();
        assert!(thousand.starts_with("0."));
        assert_eq!(thousand.len(), 1002);
    }

    #[test]
    fn exact_long_division_handles_population_boundary_and_zero() {
        assert_eq!(avg("9223372036854775807", 9223372036854775807, 0, 0).unwrap(), "1");
        assert_eq!(avg("-9223372036854775807", 9223372036854775807, 0, 0).unwrap(), "-1");
        assert_eq!(avg("0", 9, 38, 1000).unwrap(), format!("0.{}", "0".repeat(1000)));
        assert!(avg("1", 0, 0, 2).is_err());
        assert!(avg("1", u64::MAX, 0, 2).is_err());
    }
}

/// Lower the closed owner-generated AVG SELECT into a complete bounded native
/// program. The query AST is inspected before any identifier enters SQL.
pub(crate) fn lower_current_exact(a: &AdmittedRead, catalog: &str) -> JsonResult<JsonValue> {
    let r = a.request();
    if r.target != "unity-catalog-current" || r.binding.tables.len() != 1 || r.row_cap == 0 {
        return Err("finite AVG requires one bound current Unity table");
    }
    let q = a.query();
    if q.with.is_some() || q.fetch.is_some()
        || !q.locks.is_empty() || q.for_clause.is_some() || q.settings.is_some()
        || q.format_clause.is_some() || !q.pipe_operators.is_empty()
    { return Err("AVG source query modifiers refused"); }
    let limit = q.limit_clause.as_ref().ok_or("derived aggregate output cap required")?;
    if !matches!(limit, LimitClause::LimitOffset {
        limit: Some(Expr::Value(v)), offset: None, limit_by,
    } if limit_by.is_empty() && matches!(&v.value, AstValue::Number(n, false) if n.parse::<u64>().ok() == Some(u64::from(r.row_cap)))) {
        return Err("derived aggregate output cap differs from admitted rowCap");
    }
    let SetExpr::Select(select) = q.body.as_ref() else { return Err("AVG SELECT required"); };
    let GroupByExpr::Expressions(groups, modifiers) = &select.group_by else { return Err("AVG grouping expression required"); };
    if modifiers.len() != 0 || groups.len() > 3 || select.from.len() != 1 || !select.from[0].joins.is_empty()
        || select.distinct.is_some() || select.top.is_some() || select.having.is_some()
        || select.into.is_some() || select.prewhere.is_some() || select.qualify.is_some()
        || !select.cluster_by.is_empty() || !select.distribute_by.is_empty() || !select.sort_by.is_empty()
        || !select.lateral_views.is_empty() || !select.connect_by.is_empty() || !select.named_window.is_empty()
    { return Err("AVG SELECT outside closed aggregate grammar"); }
    let TableFactor::Table { name: object, alias: Some(table_alias), args: None, with_hints, version: None,
        with_ordinality: false, partitions, json_path: None, sample: None, index_hints } = &select.from[0].relation
    else { return Err("AVG requires ordinary aliased table"); };
    if !with_hints.is_empty() || !partitions.is_empty() || !index_hints.is_empty() || !table_alias.columns.is_empty() {
        return Err("AVG table modifiers refused");
    }
    let table = &r.binding.tables[0];
    let logical = object.0.iter().map(|p| match p { ObjectNamePart::Identifier(i) => Ok(name(i)), _ => Err("static source name required") }).collect::<JsonResult<Vec<_>>>()?;
    if logical != table.logical { return Err("AVG source differs from bound table"); }
    let alias = name(&table_alias.name);
    let mut lower = Lower { request:r, table, alias:alias.clone(), scans:None, slots:vec![], used:BTreeSet::new(), unity_current_binary_strings:true };
    let mut group_specs = Vec::<(String, String, Carrier, bool)>::new();
    for (i, group) in groups.iter().enumerate() {
        let (sql, carrier, nullable)=lower.column(group)?;
        if !matches!(carrier, Carrier::String | Carrier::Boolean) { return Err("AVG groups are STRING/BOOLEAN only"); }
        let field_alias=format!("__weft_avg_g{i}");
        group_specs.push((field_alias,sql,carrier,nullable));
    }
    let mut measure_specs=Vec::new();
    if select.projection.is_empty() || select.projection.len() > 8 { return Err("AVG output bound required"); }
    // Emit the public schema once, below, from the checked grouped plan.
    let mut outputs=Vec::new();
    let mut seen= BTreeSet::new();
    let mut projected_groups=Vec::new();
    let mut saw_measure=false;
    for (ordinal,item) in select.projection.iter().enumerate() {
        let SelectItem::ExprWithAlias { expr, alias: output_alias }=item else { return Err("generated AVG output aliases required"); };
        let output_name=name(output_alias);
        if output_name != format!("__weft_out_{ordinal}") || !seen.insert(output_name.clone()) { return Err("AVG output alias/order mismatch"); }
        if let Expr::CompoundIdentifier(_) = expr {
            if saw_measure { return Err("AVG groups must precede measure outputs"); }
            let (sql,carrier,nullable)=lower.column(expr)?;
            if !groups.iter().any(|g| lower.column(g).is_ok_and(|(candidate,_,_)| candidate == sql)) { return Err("AVG projected group is not grouped"); }
            let field_alias=group_specs.iter().find(|(_,candidate,_,_)|candidate==&sql).map(|(a,_,_,_)|a.clone()).ok_or("group output identity absent")?;
            if projected_groups.contains(&sql) { return Err("duplicate projected AVG group"); }
            projected_groups.push(sql.clone());
            let string=carrier==Carrier::String;
            let _ = (field_alias, nullable);
        } else if let Expr::Function(function)=expr {
            saw_measure=true;
            if function.name.0.len()!=1 || !matches!(&function.name.0[0],ObjectNamePart::Identifier(id) if name(id)=="avg_scale")
                || function.uses_odbc_syntax || !matches!(function.parameters,FunctionArguments::None)
                || !function.within_group.is_empty() || function.filter.is_some() || function.null_treatment.is_some() || function.over.is_some()
            { return Err("only owner-generated unmodified AVG_SCALE(Field, scale) is admitted"); }
            let FunctionArguments::List(arguments)=&function.args else { return Err("AVG argument required"); };
            if arguments.duplicate_treatment.is_some() || !arguments.clauses.is_empty() || arguments.args.len()!=2 { return Err("AVG modifiers refused"); }
            let FunctionArg::Unnamed(FunctionArgExpr::Expr(field_expr))=&arguments.args[0] else { return Err("AVG direct Field required"); };
            let FunctionArg::Unnamed(FunctionArgExpr::Expr(Expr::Value(scale_value)))=&arguments.args[1] else { return Err("AVG literal scale required"); };
            let result_scale = match &scale_value.value { AstValue::Number(n, false) => n.parse::<u16>().ok().filter(|n| *n <= 1000).ok_or("AVG scale outside 0..1000")?, _ => return Err("AVG scale must be unsigned integer literal")? };
            let (column_sql,carrier,nullable)=lower.column(field_expr)?;
            if !matches!(&carrier,Carrier::Integer|Carrier::Decimal) { return Err("AVG supports signed integer or decimal Field only"); }
            let logical_field=name(match field_expr { Expr::CompoundIdentifier(parts) => parts.last().ok_or("AVG Field name absent")?, _ => return Err("AVG direct Field required") });
            let (precision,scale)=numeric_source_facets(a, &logical_field, carrier.clone())?;
            let field_alias=format!("__weft_avg_v{}",measure_specs.len());
            measure_specs.push((output_name.clone(),column_sql,field_alias,logical_field,carrier.clone(),nullable,precision,scale,result_scale));
        } else { return Err("AVG projection outside group/direct AVG union"); }
    }
    if measure_specs.is_empty() || group_specs.len() + measure_specs.len() != select.projection.len()
        || group_specs.len() + measure_specs.len()>8
    { return Err("AVG measure/projection/parameter coverage mismatch"); }
    let expected_groups=group_specs.iter().map(|(_,sql,_,_)|sql.clone()).collect::<Vec<_>>();
    if projected_groups != expected_groups { return Err("AVG projection must preserve every group exactly once before measures"); }
    if measure_specs.iter().enumerate().any(|(i,(output_name,_,_,_,_,_,_,_,_))|
        output_name != &format!("__weft_out_{}",group_specs.len()+i))
    { return Err("AVG measure aliases must follow all group aliases"); }
    let predicate = select.selection.as_ref().map(|e|lower.predicate(e)).transpose()?;
    if lower.used.len() != r.parameters.len() || lower.slots.iter().cloned().collect::<BTreeSet<_>>().len() != r.parameters.len() {
        return Err("AVG predicate parameter coverage differs");
    }
    let table_name=format!("{}.{}.{}",ident(catalog)?,ident(r.binding.schema.as_deref().ok_or("Unity schema required")?)?,ident(&table.native)?);
    let mut base_projection=group_specs.iter().map(|(a,e,c,_)|format!("{} AS {}",if c==&Carrier::String{format!("({e} COLLATE UTF8_BINARY)")}else{e.clone()},ident(a).unwrap())).collect::<Vec<_>>();
    let measure_rows=measure_specs.iter().enumerate().map(|(ordinal,(_,column,_,_,carrier,_,precision,source_scale,result_scale))| {
        let carrier=if carrier==&Carrier::Integer {"integer"} else {"decimal"};
        format!("named_struct('ordinal', CAST({ordinal} AS INT), 'value', CAST({column} AS STRING), 'carrier', '{carrier}', 'precision', CAST({precision} AS INT), 'source_scale', CAST({source_scale} AS INT), 'result_scale', CAST({result_scale} AS INT))")
    }).collect::<Vec<_>>();
    if measure_rows.is_empty() { return Err("AVG measure registry empty"); }
    base_projection.push(format!("array({}) AS __weft_avg_measure_array",measure_rows.join(", ")));
    let mut sql=format!("WITH __weft_avg_input AS (SELECT {} FROM {table_name} AS {}",base_projection.join(", "),ident(&alias)?);
    if let Some(predicate)=predicate { sql.push_str(&format!(" WHERE {predicate}")); }
    let groups_sql=group_specs.iter().map(|(g,_,_,_)|ident(g)).collect::<JsonResult<Vec<_>>>()?;
    let row_groups=groups_sql.iter().map(|g|format!("__weft_avg_input.{g} AS {g}")).collect::<Vec<_>>().join(", ");
    let row_group_prefix=if row_groups.is_empty(){String::new()}else{format!("{row_groups}, ")};
    sql.push_str(&format!("), __weft_avg_rows AS (SELECT {row_group_prefix}__ordinal, __value, __carrier, __precision, __source_scale, __result_scale FROM __weft_avg_input LATERAL VIEW inline(__weft_avg_measure_array) __weft_avg_measure_view AS __ordinal, __value, __carrier, __precision, __source_scale, __result_scale)"));
    // The bounded registry makes measures rows, so the limb/count, quotient,
    // rounding, carry, and formatting CTEs below are emitted once regardless
    // of whether one or eight scales are selected.
    let digits="REGEXP_REPLACE(REGEXP_REPLACE(__value, '[-.]', ''), '^0+', '')";
    let decimal_text="(__value RLIKE '^-?[0-9]+[.][0-9]+$' AND LENGTH(SUBSTRING(__value, INSTR(__value, '.') + 1)) = __source_scale)";
    let integer_text="(__value RLIKE '^-?[0-9]+$')";
    let valid_text=format!("CASE WHEN __carrier='integer' OR __source_scale=0 THEN {integer_text} ELSE {decimal_text} END");
    let max_digits="CASE WHEN __carrier='integer' THEN 19 ELSE __precision END";
    let padded=format!("LPAD({digits}, 38, '0')");
    let hi=format!("TRY_CAST(SUBSTRING({padded},1,19) AS DECIMAL(38,0))");
    let lo=format!("TRY_CAST(SUBSTRING({padded},20,19) AS DECIMAL(38,0))");
    sql.push_str(&format!(", __weft_avg_validated AS (SELECT __weft_avg_rows.*, CASE WHEN __value IS NOT NULL AND (NOT ({valid_text}) OR LENGTH({digits}) > {max_digits}) THEN 1 ELSE 0 END AS __bad, CASE WHEN __value IS NULL THEN NULL ELSE {hi} END AS __hi, CASE WHEN __value IS NULL THEN NULL ELSE {lo} END AS __lo, CASE WHEN SUBSTRING(__value,1,1)='-' THEN 1 ELSE 0 END AS __neg FROM __weft_avg_rows)"));
    let group_select=if groups_sql.is_empty(){String::new()}else{format!("{}, ",groups_sql.join(", "))};
    let grouping_columns=format!("{}__ordinal, __precision, __source_scale, __result_scale",if groups_sql.is_empty(){String::new()}else{format!("{},",groups_sql.join(","))});
    let sum_expr=|condition:&str,limb:&str| format!("TRY_SUM(CAST(CASE WHEN {condition} AND __value IS NOT NULL THEN {limb} ELSE 0 END AS DECIMAL(38,0)))");
    sql.push_str(&format!(", __weft_avg_state AS (SELECT {group_select}__ordinal, __precision, __source_scale, __result_scale, {} AS __ph, {} AS __pl, {} AS __nh, {} AS __nl, TRY_SUM(CAST(CASE WHEN __value IS NOT NULL THEN 1 ELSE 0 END AS DECIMAL(38,0))) AS __n, MAX(CASE WHEN __value IS NOT NULL THEN 1 ELSE 0 END) AS __present, MAX(__bad) AS __bad FROM __weft_avg_validated GROUP BY {grouping_columns})",
        sum_expr("__neg=0","__hi"),sum_expr("__neg=0","__lo"),sum_expr("__neg=1","__hi"),sum_expr("__neg=1","__lo")));
    let mut registry=Vec::new();
    for (ordinal,(_,_,_,_,_,_,precision,source_scale,result_scale)) in measure_specs.iter().enumerate() {
        registry.push(format!("SELECT CAST({ordinal} AS INT) AS __ordinal, CAST({precision} AS INT) AS __precision, CAST({source_scale} AS INT) AS __source_scale, CAST({result_scale} AS INT) AS __result_scale"));
    }
    sql.push_str(&format!(", __weft_avg_registry AS ({})",registry.join(" UNION ALL ")));
    let expected_measure_count=measure_specs.len();
    let expected_last_ordinal=expected_measure_count-1;
    let registry_metadata_checks=measure_specs.iter().enumerate().map(|(ordinal,(_,_,_,_,_,_,precision,source_scale,result_scale))|
        format!("SUM(CASE WHEN __ordinal={ordinal} AND __precision={precision} AND __source_scale={source_scale} AND __result_scale={result_scale} THEN 1 ELSE 0 END)=1")
    ).collect::<Vec<_>>().join(" AND ");
    let registry_complete=format!("COUNT(*)={expected_measure_count} AND COUNT(DISTINCT __ordinal)={expected_measure_count} AND MIN(__ordinal)=0 AND MAX(__ordinal)={expected_last_ordinal} AND {registry_metadata_checks}");
    sql.push_str(&format!(", __weft_avg_registry_guard AS (SELECT CASE WHEN {registry_complete} THEN 1 ELSE raise_error('WFT-CAPABILITY') END AS __registry_valid FROM __weft_avg_registry)"));
    if groups_sql.is_empty() {
        sql.push_str(", __weft_avg_population AS (SELECT r.__ordinal, r.__precision, r.__source_scale, r.__result_scale, s.__ph, s.__pl, s.__nh, s.__nl, s.__n, s.__present, s.__bad, g.__registry_valid FROM __weft_avg_registry AS r LEFT JOIN __weft_avg_state AS s ON r.__ordinal=s.__ordinal CROSS JOIN __weft_avg_registry_guard AS g)");
    } else {
        sql.push_str(", __weft_avg_population AS (SELECT s.*, g.__registry_valid FROM __weft_avg_state AS s CROSS JOIN __weft_avg_registry_guard AS g)");
    }
    let pad_p="LPAD(CAST(__pl AS STRING),38,'0')";
    let pad_n="LPAD(CAST(__nl AS STRING),38,'0')";
    let norm_projection=groups_sql.iter().cloned().chain(["__ordinal","__precision","__source_scale","__result_scale","__ph","__pl","__nh","__nl","__n","__present","__bad","__registry_valid"].into_iter().map(str::to_owned)).chain([
        format!("(__ph + TRY_CAST(SUBSTRING({pad_p},1,19) AS DECIMAL(38,0))) AS __p_hi"),
        format!("TRY_CAST(SUBSTRING({pad_p},20,19) AS DECIMAL(38,0)) AS __p_lo"),
        format!("(__nh + TRY_CAST(SUBSTRING({pad_n},1,19) AS DECIMAL(38,0))) AS __n_hi"),
        format!("TRY_CAST(SUBSTRING({pad_n},20,19) AS DECIMAL(38,0)) AS __n_lo")
    ].into_iter()).collect::<Vec<_>>().join(", ");
    sql.push_str(&format!(", __weft_avg_norm AS (SELECT {norm_projection} FROM __weft_avg_population)"));
    let greater="(__p_hi > __n_hi OR (__p_hi = __n_hi AND __p_lo >= __n_lo))";
    let high=format!("CASE WHEN {greater} THEN (__p_hi - __n_hi - CASE WHEN __p_lo < __n_lo THEN 1 ELSE 0 END) ELSE (__n_hi - __p_hi - CASE WHEN __n_lo < __p_lo THEN 1 ELSE 0 END) END");
    let low=format!("CASE WHEN {greater} THEN CASE WHEN __p_lo >= __n_lo THEN __p_lo - __n_lo ELSE __p_lo + CAST({BASE} AS DECIMAL(38,0)) - __n_lo END ELSE CASE WHEN __n_lo >= __p_lo THEN __n_lo - __p_lo ELSE __n_lo + CAST({BASE} AS DECIMAL(38,0)) - __p_lo END END");
    let empty="(__present IS NULL AND __bad IS NULL AND __n IS NULL AND __ph IS NULL AND __pl IS NULL AND __nh IS NULL AND __nl IS NULL)";
    let all_null="(__present=0 AND __bad=0 AND __n=0 AND __ph=0 AND __pl=0 AND __nh=0 AND __nl=0)";
    let populated=format!("(__present=1 AND __bad=0 AND __n BETWEEN 1 AND {MAX_POPULATION} AND __ph IS NOT NULL AND __pl IS NOT NULL AND __nh IS NOT NULL AND __nl IS NOT NULL AND __ph>=0 AND __pl>=0 AND __nh>=0 AND __nl>=0 AND __p_hi IS NOT NULL AND __p_lo IS NOT NULL AND __n_hi IS NOT NULL AND __n_lo IS NOT NULL)");
    let invalid=format!("CASE WHEN __registry_valid<>1 THEN TRUE WHEN {empty} THEN FALSE WHEN {all_null} THEN FALSE WHEN {populated} THEN FALSE ELSE TRUE END");
    let rawdigits=format!("COALESCE(NULLIF(REGEXP_REPLACE(CONCAT(CAST(({high}) AS STRING),LPAD(CAST(({low}) AS STRING),19,'0')),'^0+',''),''),'0')");
    let negative=format!("(__present=1 AND NOT {greater} AND {rawdigits}<>'0')");
    let safe_digits=format!("CASE WHEN {invalid} OR __present IS NULL OR __present=0 THEN '0' ELSE {rawdigits} END");
    let safe_population=format!("CASE WHEN {invalid} OR __present IS NULL OR __present=0 THEN CAST(1 AS DECIMAL(38,0)) ELSE __n END");
    sql.push_str(&format!(", __weft_avg_pre AS (SELECT __weft_avg_norm.*, {invalid} AS __invalid, {rawdigits} AS __coefficient, {negative} AS __negative, {safe_digits} AS __safe_digits, {safe_population} AS __safe_n FROM __weft_avg_norm)"));
    let stream="CASE WHEN __result_scale >= __source_scale THEN CONCAT(__safe_digits, REPEAT('0', __result_scale - __source_scale)) ELSE __safe_digits END";
    let division=long_division_sql(stream,"__safe_n");
    sql.push_str(&format!(", __weft_avg_division AS (SELECT __weft_avg_pre.*, {division} AS __division FROM __weft_avg_pre)"));
    let quotient="__division.q";
    let delta="(__source_scale - __result_scale)";
    let padded_q=format!("CASE WHEN __result_scale < __source_scale AND LENGTH({quotient}) <= {delta} THEN LPAD({quotient},{delta}+1,'0') ELSE {quotient} END");
    let prefix=format!("SUBSTRING({padded_q},1,LENGTH({padded_q})-{delta})");
    let discarded=format!("SUBSTRING({padded_q},LENGTH({padded_q})-{delta}+1,{delta})");
    let guard=format!("CAST(SUBSTRING({discarded},1,1) AS INT)");
    let sticky=format!("(REGEXP_REPLACE(SUBSTRING({discarded},2,{delta}),'0','')<>'' OR __division.r<>0)");
    let odd_q=format!("CAST(SUBSTRING({quotient},LENGTH({quotient}),1) AS INT)%2=1");
    let odd_prefix=format!("CAST(SUBSTRING({prefix},LENGTH({prefix}),1) AS INT)%2=1");
    let kept=format!("CASE WHEN __result_scale >= __source_scale THEN {quotient} ELSE {prefix} END");
    let round_up=format!("CASE WHEN __result_scale >= __source_scale THEN (2*__division.r>__safe_n OR (2*__division.r=__safe_n AND {odd_q})) ELSE ({guard}>5 OR ({guard}=5 AND ({sticky} OR {odd_prefix}))) END");
    sql.push_str(&format!(", __weft_avg_round AS (SELECT __weft_avg_division.*, {kept} AS __kept, {round_up} AS __round_up FROM __weft_avg_division)"));
    let increment="aggregate(sequence(1,LENGTH(__kept)),named_struct('v',CAST('' AS STRING),'c',1),(st,i)->named_struct('v',CONCAT(CASE WHEN st.c=1 THEN CASE WHEN SUBSTRING(__kept,LENGTH(__kept)-i+1,1)='9' THEN '0' ELSE CAST(CAST(SUBSTRING(__kept,LENGTH(__kept)-i+1,1) AS INT)+1 AS STRING) END ELSE SUBSTRING(__kept,LENGTH(__kept)-i+1,1) END,st.v),'c',CASE WHEN st.c=1 AND SUBSTRING(__kept,LENGTH(__kept)-i+1,1)='9' THEN 1 ELSE 0 END))";
    sql.push_str(&format!(", __weft_avg_carry AS (SELECT __weft_avg_round.*, {increment} AS __increment FROM __weft_avg_round)"));
    sql.push_str(", __weft_avg_rounded AS (SELECT __weft_avg_carry.*, CASE WHEN __round_up THEN CASE WHEN __increment.c=1 THEN CONCAT('1',__increment.v) ELSE __increment.v END ELSE __kept END AS __rounded FROM __weft_avg_carry)");
    sql.push_str(", __weft_avg_magnitude AS (SELECT __weft_avg_rounded.*, COALESCE(NULLIF(REGEXP_REPLACE(__rounded,'^0+',''),''),'0') AS __magnitude FROM __weft_avg_rounded)");
    let padded_magnitude="CASE WHEN LENGTH(__magnitude)<__result_scale+1 THEN LPAD(__magnitude,__result_scale+1,'0') ELSE __magnitude END";
    let formatted=format!("CASE WHEN __result_scale=0 THEN __magnitude ELSE CONCAT(SUBSTRING({padded_magnitude},1,LENGTH({padded_magnitude})-__result_scale),'.',SUBSTRING({padded_magnitude},LENGTH({padded_magnitude})-__result_scale+1,__result_scale)) END");
    sql.push_str(&format!(", __weft_avg_results AS (SELECT __weft_avg_magnitude.*, CASE WHEN __invalid OR LENGTH(__magnitude)>1038 THEN raise_error('WFT-CAPABILITY') WHEN __present IS NULL OR __present=0 THEN NULL ELSE CONCAT(CASE WHEN __negative AND __magnitude<>'0' THEN '-' ELSE '' END,{formatted}) END AS __avg_text, CASE WHEN __invalid THEN raise_error('WFT-CAPABILITY') WHEN __present IS NULL OR __present=0 THEN NULL WHEN __magnitude='0' THEN 0 WHEN __negative THEN -1 ELSE 1 END AS __avg_sign FROM __weft_avg_magnitude)"));
    let group_count=group_specs.len();
    let mut public_projection=Vec::new();
    for (i,(group_alias,_,carrier,nullable)) in group_specs.iter().enumerate() {
        let output_name=format!("__weft_out_{i}");
        public_projection.push(format!("{group_alias} AS {}",ident(&output_name)?));
        let string=carrier==&Carrier::String;
        let mut group_output=json!({"name":output_name,"role":"group","carrier":if string{"string"}else{"boolean"},"nullable":nullable,"decoder":{"kind":if string{"native-string"}else{"native-boolean"}}});
        if string { group_output["collation"] = json!("UTF8_BINARY"); }
        outputs.push(group_output);
    }
    let mut sort_keys=BTreeMap::<String,(String,String,String)>::new();
    let group_metadata_checks=measure_specs.iter().enumerate().map(|(ordinal,(_,_,_,_,_,_,precision,source_scale,result_scale))|
        format!("SUM(CASE WHEN __ordinal={ordinal} AND __precision={precision} AND __source_scale={source_scale} AND __result_scale={result_scale} AND __registry_valid=1 THEN 1 ELSE 0 END)=1")
    ).collect::<Vec<_>>().join(" AND ");
    let group_coverage=format!("COUNT(*)={expected_measure_count} AND COUNT(DISTINCT __ordinal)={expected_measure_count} AND MIN(__ordinal)=0 AND MAX(__ordinal)={expected_last_ordinal} AND {group_metadata_checks}");
    let mut wide_projection=groups_sql.clone();
    wide_projection.push(format!("CASE WHEN {group_coverage} THEN 1 ELSE raise_error('WFT-CAPABILITY') END AS __coverage_valid"));
    for (i,(output_name,_column,_value_alias,logical,_carrier,_nullable,_precision,_source_scale,result_scale)) in measure_specs.iter().enumerate() {
        let output_index=group_count+i;
        let output_alias=format!("__weft_out_{output_index}");
        let condition=format!("__ordinal={i}");
        for (suffix,value) in [("value","__avg_text"),("sign","__avg_sign"),("length","LENGTH(__magnitude)"),("magnitude","__magnitude")] {
            let field_alias=if suffix=="value" {output_alias.clone()} else {format!("__weft_sort_{i}_{suffix}")};
            wide_projection.push(format!("MAX(CASE WHEN {condition} THEN {value} END) AS {}",ident(&field_alias)?));
        }
        public_projection.push(output_alias.clone());
        let source_ref=source_ref(a,logical)?;
        let (native_name,_,_,_)=r.binding.tables[0].columns.iter().find(|c|c.logical==*logical).map(|c|(&c.native,&c.carrier,c.nullable,&c.logical)).ok_or("AVG physical column absent")?;
        let source_facets=source_facet_json(a,logical)?;
        outputs.push(json!({"name":output_alias,"role":"measure","function":"avg","fieldRef":source_ref,"nativeName":native_name,"carrier":"decimal","nullable":true,"sourceFacets":source_facets,"decoder":{"kind":"canonical-fixed-scale-decimal-text","scale":result_scale,"maximumCoefficientDigits":1038}}));
        sort_keys.insert(output_alias,(format!("__weft_sort_{i}_sign"),format!("__weft_sort_{i}_length"),format!("__weft_sort_{i}_magnitude")));
        let _=output_name;
    }
    let group_by=if groups_sql.is_empty(){String::new()}else{format!(" GROUP BY {}",groups_sql.join(", "))};
    sql.push_str(&format!(", __weft_avg_wide AS (SELECT {} FROM __weft_avg_results{group_by})",wide_projection.join(", ")));
    sql.push_str(", __weft_avg_guarded AS (SELECT * FROM __weft_avg_wide WHERE __coverage_valid=1)");
    let first="__weft_avg_guarded";
    let mut order_parts=Vec::new();
    let valid_group_aliases=(0..group_specs.len()).map(|i|format!("__weft_out_{i}")).collect::<BTreeSet<_>>();
    if let Some(order)=&q.order_by {
        if order.interpolate.is_some() { return Err("AVG order interpolation refused"); }
        let OrderByKind::Expressions(items)=&order.kind else { return Err("AVG order ALL refused"); };
        for item in items {
            if item.with_fill.is_some() { return Err("AVG order fill refused"); }
            let Expr::Identifier(identifier)=&item.expr else { return Err("AVG order must reference output alias"); };
            let output=name(&identifier);
            let direction=match &item.options.sort { Some(OrderBySort::Desc)=>"DESC",Some(OrderBySort::Using(_))=>return Err("AVG order USING refused"),_=>"ASC" };
            let nulls=if item.options.nulls_first==Some(true){"FIRST"}else{"LAST"};
            if let Some((sign,length,magnitude))=sort_keys.get(&output) {
                let null_rank=if nulls=="FIRST" {format!("CASE WHEN {sign} IS NULL THEN 0 ELSE 1 END ASC")} else {format!("CASE WHEN {sign} IS NULL THEN 1 ELSE 0 END ASC")};
                order_parts.push(null_rank);
                order_parts.push(format!("CASE WHEN {sign}=-1 THEN -({length}) ELSE ({length}) END {direction}"));
                order_parts.push(format!("CASE WHEN {sign}=-1 THEN TRANSLATE({magnitude}, '0123456789', '9876543210') ELSE {magnitude} END COLLATE UTF8_BINARY {direction}"));
            } else if valid_group_aliases.contains(&output) {
                order_parts.push(format!("{} {direction} NULLS {nulls}",ident(&output)?));
            } else { return Err("AVG order output alias absent"); }
        }
    }
    for (i,(group_alias,_,carrier,_)) in group_specs.iter().enumerate() {
        let out_name=format!("__weft_out_{i}");
        if !order_parts.iter().any(|p|p.contains(&ident(&out_name).unwrap())) {
            order_parts.push(format!("{} ASC NULLS LAST",if carrier==&Carrier::String {format!("({first}.{} COLLATE UTF8_BINARY)",ident(group_alias)?)} else {format!("{first}.{}",ident(group_alias)?)}));
        }
    }
    let mut final_sql=format!("{sql} SELECT {} FROM {first}",public_projection.join(", "));
    if !order_parts.is_empty() { final_sql.push_str(&format!(" ORDER BY {}",order_parts.join(", "))); }
    final_sql.push_str(&format!(" LIMIT {}",r.row_cap));
    if final_sql.as_bytes().len()>65_536 || lower.used.len()!=r.parameters.len() { return Err("expanded AVG SQL/parameter limit or coverage refused"); }
    let referenced_columns=r.binding.tables[0].columns.iter().map(|c|json!({"logical":c.logical,"native":c.native,"carrier":c.carrier,"nullable":c.nullable})).collect::<Vec<_>>();
    Ok(json!({"version":"weft.unity-current-selection/0.6","status":"compiled","sql":final_sql,"parameters":r.parameters,
        "parameterOrder":lower.slots,"outputs":outputs,"nativeChecks":[],"snapshotTables":[],"lockSql":[],"cursorParameters":[],"cursorOutputs":[],"cursorOrdering":[],
        "referencedTables":[{"logical":table.logical,"native":table.native,"columns":referenced_columns}]}))
}

fn source_ref(a:&AdmittedRead, logical:&str)->JsonResult<JsonValue>{
    let doc=a.original_document(); let table=&a.request().binding.tables[0];
    let module=doc["modules"].as_array().ok_or("source modules missing")?.iter().find(|m|m["id"]==table.logical[0]).ok_or("source module missing")?;
    let record=module["elements"].as_array().ok_or("source elements missing")?.iter().find(|e|e["kind"]=="record"&&(e["name"].as_str().or_else(||e["id"].as_str())==Some(table.logical[1].as_str()))).ok_or("source Record missing")?;
    for member in record["members"].as_array().ok_or("Record members missing")?{
        let owner=doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==member["module"]).ok_or("Field module missing")?;
        if let Some(field)=owner["elements"].as_array().unwrap().iter().find(|f|f["id"]==member["element"]&&f["kind"]=="field"&&(f["name"].as_str().or_else(||f["id"].as_str())==Some(logical))){return Ok(json!({"module":member["module"],"element":member["element"]}));}
    }
    Err("source FieldRef absent")
}
fn source_facet_json(a:&AdmittedRead,logical:&str)->JsonResult<JsonValue>{
    let doc=a.original_document();let table=&a.request().binding.tables[0];
    let module=doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==table.logical[0]).ok_or("source module missing")?;
    let record=module["elements"].as_array().unwrap().iter().find(|e|e["kind"]=="record"&&(e["name"].as_str().or_else(||e["id"].as_str())==Some(table.logical[1].as_str()))).ok_or("source Record missing")?;
    for member in record["members"].as_array().ok_or("Record members missing")? {let owner=doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==member["module"]).ok_or("Field module missing")?;if let Some(field)=owner["elements"].as_array().unwrap().iter().find(|f|f["id"]==member["element"]&&f["kind"]=="field"&&(f["name"].as_str().or_else(||f["id"].as_str())==Some(logical))){return Ok(field.get("facets").cloned().unwrap_or_else(||json!({})));}}
    Err("source AVG facets absent")
}
fn numeric_source_facets(a: &AdmittedRead, logical: &str, carrier: Carrier) -> JsonResult<(u64,u64)> {
    let binding=&a.request().binding;
    let doc=&a.original_document();
    let module=doc["modules"].as_array().ok_or("source modules absent")?.iter().find(|m|m["id"]==binding.tables[0].logical[0]).ok_or("source module absent")?;
    let record=module["elements"].as_array().ok_or("source elements absent")?.iter().find(|e|e["kind"]=="record"&&(e["name"].as_str().or_else(||e["id"].as_str())==Some(binding.tables[0].logical[1].as_str()))).ok_or("source Record absent")?;
    for member in record["members"].as_array().ok_or("source Record members absent")? {
        let fm=doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==member["module"]).ok_or("member module absent")?;
        let field=fm["elements"].as_array().unwrap().iter().find(|f|f["id"]==member["element"]&&f["kind"]=="field").ok_or("member Field absent")?;
        if field["name"].as_str().or_else(||field["id"].as_str())==Some(logical) {
            if (carrier==Carrier::Integer && field["scalarType"]!="integer") || (carrier==Carrier::Decimal && field["scalarType"]!="decimal") { return Err("AVG carrier differs from original Field"); }
            if carrier==Carrier::Integer {
                if field["facets"].as_object().is_none_or(|o| o.len()!=1)
                    || field["facets"]["integerWidth"].as_object().is_none_or(|o| o.len()!=2)
                    || field["facets"]["integerWidth"].get("bits").is_none()
                    || field["facets"]["integerWidth"].get("signed").is_none()
                { return Err("AVG integer facets must be exactly signed width"); }
                let bits=field["facets"]["integerWidth"]["bits"].as_u64().filter(|b|matches!(b,8|16|32|64)).ok_or("AVG signed integer width required")?;
                if field["facets"]["integerWidth"]["signed"]!=true { return Err("unsigned AVG unsupported"); }
                return Ok((bits,0));
            }
            if field["facets"].as_object().is_none_or(|o| o.len()!=2)
                || field["facets"].get("precision").is_none()
                || field["facets"].get("scale").is_none()
            { return Err("AVG decimal facets must be exactly precision and scale"); }
            let p=field["facets"]["precision"].as_u64().filter(|p|(1..=38).contains(p)).ok_or("AVG decimal precision required")?;
            let s=field["facets"]["scale"].as_u64().filter(|s|*s<=p).ok_or("AVG decimal scale required")?;
            return Ok((p,s));
        }
    }
    Err("AVG source Field identity absent")
}
