// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exact base-10^19 signed-limb arithmetic used by the finite Unity SUM profile.
//! This module is pure arithmetic; it does not authorize or execute native SQL.

const BASE: u128 = 10_000_000_000_000_000_000;
const MAX_POPULATION: u64 = i64::MAX as u64;

use serde_json::{json, Value as JsonValue};
use sqlparser::ast::Value as AstValue;
use std::collections::{BTreeMap, BTreeSet};
use sqlparser::ast::*;
use super::raw_read::{ident, name, Lower};
use weft_core::raw_read::{AdmittedRead, Carrier};
type JsonResult<T> = std::result::Result<T, &'static str>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SignedLimbs {
    pub positive_high: u128,
    pub positive_low: u128,
    pub negative_high: u128,
    pub negative_low: u128,
    pub nonnull_count: Option<u64>,
    pub has_nonnull: bool,
    pub malformed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Magnitude {
    high: u128,
    low: u64,
}

/// Reconstruct a canonical coefficient from checked aggregate state.
/// `None` denotes the SQL NULL result for empty/all-null input.
pub(crate) fn coefficient(state: SignedLimbs) -> Result<Option<String>, &'static str> {
    if state.malformed {
        return Err("WFT-CAPABILITY: malformed native numeric text");
    }
    let n = match (state.has_nonnull, state.nonnull_count) {
        (false, None) if state.positive_high == 0 && state.positive_low == 0
            && state.negative_high == 0 && state.negative_low == 0 => return Ok(None),
        (false, Some(0)) if state.positive_high == 0 && state.positive_low == 0
            && state.negative_high == 0 && state.negative_low == 0 => return Ok(None),
        (false, None | Some(0)) => return Err("WFT-CAPABILITY: empty population has nonzero limbs"),
        (false, Some(_)) => return Err("WFT-CAPABILITY: inconsistent population state"),
        (true, None) => return Err("WFT-CAPABILITY: population aggregate overflow"),
        (true, Some(0)) => return Err("WFT-CAPABILITY: inconsistent population state"),
        (true, Some(n)) => n,
    };
    if n > MAX_POPULATION {
        return Err("WFT-CAPABILITY: population exceeds exact domain");
    }
    let positive = normalize(state.positive_high, state.positive_low, n)?;
    let negative = normalize(state.negative_high, state.negative_low, n)?;
    let (negative_result, magnitude) = match positive.cmp(&negative) {
        std::cmp::Ordering::Equal => (false, Magnitude { high: 0, low: 0 }),
        std::cmp::Ordering::Greater => (false, subtract(positive, negative)?),
        std::cmp::Ordering::Less => (true, subtract(negative, positive)?),
    };
    let mut digits = format!("{}{:019}", magnitude.high, magnitude.low);
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() {
        return Ok(Some("0".into()));
    }
    digits = trimmed.to_owned();
    if digits.len() > 57 {
        return Err("WFT-CAPABILITY: reconstructed coefficient exceeds 57 digits");
    }
    if negative_result {
        digits.insert(0, '-');
    }
    Ok(Some(digits))
}

fn normalize(high: u128, low_total: u128, n: u64) -> Result<Magnitude, &'static str> {
    // Each term is < B and there are at most n terms. The proof bound also
    // guards corrupted owner/native aggregate state before reconstruction.
    let bound = (n as u128)
        .checked_mul(BASE - 1)
        .ok_or("WFT-CAPABILITY: limb bound overflow")?;
    if low_total > bound || high > bound {
        return Err("WFT-CAPABILITY: limb total outside population bound");
    }
    let carry = low_total / BASE;
    let low = (low_total % BASE) as u64;
    let high = high
        .checked_add(carry)
        .ok_or("WFT-CAPABILITY: high limb overflow")?;
    if high >= (n as u128).saturating_mul(BASE) {
        return Err("WFT-CAPABILITY: normalized high limb outside population bound");
    }
    Ok(Magnitude { high, low })
}

pub(crate) fn fixed_scale_decimal(coefficient: &str, scale: u8) -> Result<String, &'static str> {
    if scale > 38 { return Err("WFT-CAPABILITY: decimal scale outside profile"); }
    let negative = coefficient.starts_with('-');
    let digits = coefficient.strip_prefix('-').unwrap_or(coefficient);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) || digits.len() > 57 {
        return Err("WFT-CAPABILITY: coefficient text malformed");
    }
    let canonical = digits.trim_start_matches('0');
    let canonical = if canonical.is_empty() { "0" } else { canonical };
    let mut rendered = if scale == 0 {
        canonical.to_owned()
    } else {
        let scale = scale as usize;
        let padded = if canonical.len() <= scale {
            format!("{}{}", "0".repeat(scale + 1 - canonical.len()), canonical)
        } else { canonical.to_owned() };
        let split = padded.len() - scale;
        format!("{}.{}", &padded[..split], &padded[split..])
    };
    if negative && canonical != "0" { rendered.insert(0, '-'); }
    Ok(rendered)
}

fn subtract(larger: Magnitude, smaller: Magnitude) -> Result<Magnitude, &'static str> {
    if larger < smaller {
        return Err("WFT-CAPABILITY: signed magnitude order mismatch");
    }
    if larger.low >= smaller.low {
        Ok(Magnitude {
            high: larger.high - smaller.high,
            low: larger.low - smaller.low,
        })
    } else {
        if larger.high == 0 {
            return Err("WFT-CAPABILITY: limb borrow underflow");
        }
        let low = BASE as u64 - (smaller.low - larger.low);
        Ok(Magnitude {
            high: larger.high - smaller.high - 1,
            low,
        })
    }
}

/// Lower the closed owner-generated SUM SELECT into a complete bounded native
/// program. The query AST is inspected before any identifier enters SQL.
pub(crate) fn lower_current_exact(a: &AdmittedRead, catalog: &str) -> JsonResult<JsonValue> {
    let r = a.request();
    if r.target != "unity-catalog-current" || r.binding.tables.len() != 1 || r.row_cap == 0 {
        return Err("finite SUM requires one bound current Unity table");
    }
    let q = a.query();
    if q.with.is_some() || q.fetch.is_some()
        || !q.locks.is_empty() || q.for_clause.is_some() || q.settings.is_some()
        || q.format_clause.is_some() || !q.pipe_operators.is_empty()
    { return Err("SUM source query modifiers refused"); }
    if q.offset.is_some() || !q.limit_by.is_empty() { return Err("SUM offset or limit-by refused"); }
    let limit = q.limit_clause.as_ref().ok_or("derived aggregate output cap required")?;
    if !matches!(limit, LimitClause::LimitOffset {
        limit: Some(Expr::Value(v)), offset: None, limit_by,
    } if limit_by.is_empty() && matches!(&v.value, AstValue::Number(n, false) if n.parse::<u64>().ok() == Some(r.row_cap))) {
        return Err("derived aggregate output cap differs from admitted rowCap");
    }
    let SetExpr::Select(select) = q.body.as_ref() else { return Err("SUM SELECT required"); };
    let GroupByExpr::Expressions(groups, modifiers) = &select.group_by else { return Err("SUM grouping expression required"); };
    if modifiers.len() != 0 || groups.len() > 3 || select.from.len() != 1 || !select.from[0].joins.is_empty()
        || select.distinct.is_some() || select.top.is_some() || select.having.is_some()
        || select.into.is_some() || select.prewhere.is_some() || select.qualify.is_some()
        || !select.cluster_by.is_empty() || !select.distribute_by.is_empty() || !select.sort_by.is_empty()
        || !select.lateral_views.is_empty() || !select.connect_by.is_empty() || !select.named_window.is_empty()
    { return Err("SUM SELECT outside closed aggregate grammar"); }
    let TableFactor::Table { name: object, alias: Some(table_alias), args: None, with_hints, version: None,
        with_ordinality: false, partitions, json_path: None, sample: None, index_hints } = &select.from[0].relation
    else { return Err("SUM requires ordinary aliased table"); };
    if !with_hints.is_empty() || !partitions.is_empty() || !index_hints.is_empty() || !table_alias.columns.is_empty() {
        return Err("SUM table modifiers refused");
    }
    let table = &r.binding.tables[0];
    let logical = object.0.iter().map(|p| match p { ObjectNamePart::Identifier(i) => Ok(name(i)), _ => Err("static source name required") }).collect::<JsonResult<Vec<_>>>()?;
    if logical != table.logical { return Err("SUM source differs from bound table"); }
    let alias = name(&table_alias.name);
    let mut lower = Lower { request:r, table, alias:alias.clone(), scans:None, slots:vec![], used:BTreeSet::new(), unity_current_binary_strings:true };
    let mut group_specs = Vec::<(String, String, Carrier, bool)>::new();
    for (i, group) in groups.iter().enumerate() {
        let (sql, carrier, nullable)=lower.column(group)?;
        if !matches!(carrier, Carrier::String | Carrier::Boolean) { return Err("SUM groups are STRING/BOOLEAN only"); }
        let field_alias=format!("__weft_sum_g{i}");
        group_specs.push((field_alias,sql,carrier,nullable));
    }
    let mut measure_specs=Vec::new();
    if select.projection.is_empty() || select.projection.len() > 8 { return Err("SUM output bound required"); }
    // Emit the public schema once, below, from the checked grouped plan.
    let mut outputs=Vec::new();
    let mut seen= BTreeSet::new();
    let mut projected_groups=Vec::new();
    let mut saw_measure=false;
    for (ordinal,item) in select.projection.iter().enumerate() {
        let SelectItem::ExprWithAlias { expr, alias: output_alias }=item else { return Err("generated SUM output aliases required"); };
        let output_name=name(output_alias);
        if output_name != format!("__weft_out_{ordinal}") || !seen.insert(output_name.clone()) { return Err("SUM output alias/order mismatch"); }
        if let Expr::CompoundIdentifier(_) = expr {
            if saw_measure { return Err("SUM groups must precede measure outputs"); }
            let (sql,carrier,nullable)=lower.column(expr)?;
            if !groups.iter().any(|g| lower.column(g).is_ok_and(|(candidate,_,_)| candidate == sql)) { return Err("SUM projected group is not grouped"); }
            let field_alias=group_specs.iter().find(|(_,candidate,_,_)|candidate==&sql).map(|(a,_,_,_)|a.clone()).ok_or("group output identity absent")?;
            if projected_groups.contains(&sql) { return Err("duplicate projected SUM group"); }
            projected_groups.push(sql.clone());
            let string=carrier==Carrier::String;
            let _ = (field_alias, nullable);
        } else if let Expr::Function(function)=expr {
            saw_measure=true;
            if function.name.0.len()!=1 || !matches!(&function.name.0[0],ObjectNamePart::Identifier(id) if name(id)=="sum")
                || function.uses_odbc_syntax || !matches!(function.parameters,FunctionArguments::None)
                || !function.within_group.is_empty() || function.filter.is_some() || function.null_treatment.is_some() || function.over.is_some()
            { return Err("only direct unmodified SUM(Field) is admitted"); }
            let FunctionArguments::List(arguments)=&function.args else { return Err("SUM argument required"); };
            if arguments.duplicate_treatment.is_some() || !arguments.clauses.is_empty() || arguments.args.len()!=1 { return Err("SUM modifiers refused"); }
            let FunctionArg::Unnamed(FunctionArgExpr::Expr(field_expr))=&arguments.args[0] else { return Err("SUM direct Field required"); };
            let (column_sql,carrier,nullable)=lower.column(field_expr)?;
            if !matches!(&carrier,Carrier::Integer|Carrier::Decimal) { return Err("SUM supports signed integer or decimal Field only"); }
            let logical_field=name(match field_expr { Expr::CompoundIdentifier(parts) => parts.last().ok_or("SUM Field name absent")?, _ => return Err("SUM direct Field required") });
            let (precision,scale)=numeric_source_facets(a, &logical_field, carrier.clone())?;
            let field_alias=format!("__weft_sum_v{}",measure_specs.len());
            measure_specs.push((output_name.clone(),column_sql,field_alias,logical_field,carrier.clone(),nullable,precision,scale));
        } else { return Err("SUM projection outside group/direct SUM union"); }
    }
    if measure_specs.is_empty() || group_specs.len() + measure_specs.len() != select.projection.len()
        || group_specs.len() + measure_specs.len()>8
    { return Err("SUM measure/projection/parameter coverage mismatch"); }
    let expected_groups=group_specs.iter().map(|(_,sql,_,_)|sql.clone()).collect::<Vec<_>>();
    if projected_groups != expected_groups { return Err("SUM projection must preserve every group exactly once before measures"); }
    if measure_specs.iter().enumerate().any(|(i,(output_name,_,_,_,_,_,_,_))|
        output_name != &format!("__weft_out_{}",group_specs.len()+i))
    { return Err("SUM measure aliases must follow all group aliases"); }
    let predicate = select.selection.as_ref().map(|e|lower.predicate(e)).transpose()?;
    if lower.used.len() != r.parameters.len() || lower.slots.iter().cloned().collect::<BTreeSet<_>>().len() != r.parameters.len() {
        return Err("SUM predicate parameter coverage differs");
    }
    let table_name=format!("{}.{}.{}",ident(catalog)?,ident(r.binding.schema.as_deref().ok_or("Unity schema required")?)?,ident(&table.native)?);
    let mut base_projection=group_specs.iter().map(|(a,e,c,_)|format!("{} AS {}",if c==&Carrier::String{format!("({e} COLLATE UTF8_BINARY)")}else{e.clone()},ident(a).unwrap())).collect::<Vec<_>>();
    for (_,column,alias,_,_,_,_,_) in &measure_specs { base_projection.push(format!("CAST({column} AS STRING) AS {}",ident(alias)?)); }
    let mut sql=format!("WITH __weft_sum_input AS (SELECT {} FROM {table_name} AS {}",base_projection.join(", "),ident(&alias)?);
    if let Some(predicate)=predicate { sql.push_str(&format!(" WHERE {predicate}")); }
    sql.push_str(")");
    let mut state_names=Vec::new();
    for (index,(_,_,value_alias,_,carrier,_,precision,scale)) in measure_specs.iter().enumerate() {
        let state=format!("__weft_sum_s{index}"); state_names.push(state.clone());
        let v=ident(value_alias)?;
        let coeff=format!("REGEXP_REPLACE({v}, '[-.]', '')");
        let digits=format!("REGEXP_REPLACE({coeff}, '^0+', '')");
        let integer= carrier==&Carrier::Integer;
        let regex=if integer {"^-?[0-9]+$".to_owned()} else if *scale==0 {"^-?[0-9]+$".to_owned()} else {format!("^-?[0-9]+[.][0-9]{{{}}}$",scale)};
        let max_digits=if integer {19} else {*precision as usize};
        let malformed=format!("CASE WHEN {v} IS NOT NULL AND (NOT ({v} RLIKE '{}') OR LENGTH({digits}) > {max_digits}) THEN 1 ELSE 0 END",regex.replace('\'',"''"));
        let padded=format!("LPAD({digits}, 38, '0')");
        let high=format!("TRY_CAST(SUBSTRING({padded}, 1, 19) AS DECIMAL(38,0))");
        let low=format!("TRY_CAST(SUBSTRING({padded}, 20, 19) AS DECIMAL(38,0))");
        let negative=format!("CASE WHEN SUBSTRING({v},1,1)='-' THEN 1 ELSE 0 END");
        let token=format!("__weft_sum_t{index}");
        sql.push_str(&format!(", {token} AS (SELECT __weft_sum_input.*, {malformed} AS __bad, CASE WHEN {v} IS NULL THEN NULL ELSE {high} END AS __hi, CASE WHEN {v} IS NULL THEN NULL ELSE {low} END AS __lo, {negative} AS __neg FROM __weft_sum_input)"));
        let groups_sql=group_specs.iter().map(|(g,_,_,_)|ident(g)).collect::<JsonResult<Vec<_>>>()?;
        let group_select=if groups_sql.is_empty(){String::new()}else{format!("{}, ",groups_sql.join(", "))};
        let group_by=if groups_sql.is_empty(){String::new()}else{format!(" GROUP BY {}",groups_sql.join(", "))};
        let sum_expr=|condition:&str,limb:&str| format!("TRY_SUM(CAST(CASE WHEN {condition} AND {v} IS NOT NULL THEN {limb} ELSE 0 END AS DECIMAL(38,0)))");
        sql.push_str(&format!(", {state} AS (SELECT {group_select}{} AS __ph, {} AS __pl, {} AS __nh, {} AS __nl, TRY_SUM(CAST(CASE WHEN {v} IS NOT NULL THEN 1 ELSE 0 END AS DECIMAL(38,0))) AS __n, MAX(CASE WHEN {v} IS NOT NULL THEN 1 ELSE 0 END) AS __present, MAX(__bad) AS __bad FROM {token}{group_by})",
            sum_expr("__neg=0","__hi"),sum_expr("__neg=0","__lo"),sum_expr("__neg=1","__hi"),sum_expr("__neg=1","__lo")));
    }
    let group_names=group_specs.iter().map(|(g,_,_,_)|ident(g)).collect::<JsonResult<Vec<_>>>()?;
    let mut norm_names=Vec::new();
    for (i,state) in state_names.iter().enumerate() {
        let norm=format!("__weft_sum_n{i}"); norm_names.push(norm.clone());
        let pad_p=format!("LPAD(CAST(__pl AS STRING), 38, '0')");
        let pad_n=format!("LPAD(CAST(__nl AS STRING), 38, '0')");
        sql.push_str(&format!(", {norm} AS (SELECT {state}.*, (__ph + TRY_CAST(SUBSTRING({pad_p},1,19) AS DECIMAL(38,0))) AS __p_hi, TRY_CAST(SUBSTRING({pad_p},20,19) AS DECIMAL(38,0)) AS __p_lo, (__nh + TRY_CAST(SUBSTRING({pad_n},1,19) AS DECIMAL(38,0))) AS __n_hi, TRY_CAST(SUBSTRING({pad_n},20,19) AS DECIMAL(38,0)) AS __n_lo FROM {state})"));
        let _=i;
    }
    let first=&norm_names[0];
    let mut joins=String::new();
    for (i,norm) in norm_names.iter().enumerate().skip(1) {
        let equality=group_names.iter().map(|g|format!("{first}.{g} <=> {norm}.{g}")).collect::<Vec<_>>();
        let on=if equality.is_empty(){"1=1".to_owned()}else{equality.join(" AND ")};
        joins.push_str(&format!(" JOIN {norm} ON {on}"));
        let _=i;
    }
    let mut public_projection=Vec::new();
    for (i,(group_alias,_,carrier,nullable)) in group_specs.iter().enumerate() {
        let output_name=format!("__weft_out_{i}");
        public_projection.push(format!("{first}.{} AS {}",ident(group_alias)?,ident(&output_name)?));
        let string=carrier==&Carrier::String;
        let mut group_output=json!({"name":output_name,"role":"group","carrier":if string{"string"}else{"boolean"},"nullable":nullable,
            "decoder":{"kind":if string{"native-string"}else{"native-boolean"}}});
        if string { group_output["collation"] = json!("UTF8_BINARY"); }
        outputs.push(group_output);
    }
    let group_count=group_specs.len();
    let mut sort_keys=BTreeMap::<String,(String,String,String)>::new();
    for (i,(output_name,_column,_value_alias,logical,carrier,nullable,precision,scale)) in measure_specs.iter().enumerate() {
        let state=&norm_names[i];
        let base=10_u64.pow(0); let _=base;
        let pos=|column:&str| format!("{state}.{column}");
        let greater=format!("({} > {} OR ({} = {} AND {} >= {}))",pos("__p_hi"),pos("__n_hi"),pos("__p_hi"),pos("__n_hi"),pos("__p_lo"),pos("__n_lo"));
        let high=format!("CASE WHEN {greater} THEN ({} - {} - CASE WHEN {} < {} THEN 1 ELSE 0 END) ELSE ({} - {} - CASE WHEN {} < {} THEN 1 ELSE 0 END) END",pos("__p_hi"),pos("__n_hi"),pos("__p_lo"),pos("__n_lo"),pos("__n_hi"),pos("__p_hi"),pos("__n_lo"),pos("__p_lo"));
        let low=format!("CASE WHEN {greater} THEN CASE WHEN {} >= {} THEN {} - {} ELSE {} + CAST({BASE} AS DECIMAL(38,0)) - {} END ELSE CASE WHEN {} >= {} THEN {} - {} ELSE {} + CAST({BASE} AS DECIMAL(38,0)) - {} END END",pos("__p_lo"),pos("__n_lo"),pos("__p_lo"),pos("__n_lo"),pos("__p_lo"),pos("__n_lo"),pos("__n_lo"),pos("__p_lo"),pos("__n_lo"),pos("__p_lo"),pos("__n_lo"),pos("__p_lo"));
        let present=pos("__present");
        let n=pos("__n"); let ph=pos("__ph"); let pl=pos("__pl"); let nh=pos("__nh"); let nl=pos("__nl");
        let bad=pos("__bad");
        // Preserve the three distinct native states: truly empty global
        // aggregate (all MAX/SUM values NULL), nonempty all-null input (0
        // flags/count/limbs), and at least one exact nonnull value. Every
        // other combination is uncertain and therefore a capability error.
        let empty=format!("{present} IS NULL AND {bad} IS NULL AND {n} IS NULL AND {ph} IS NULL AND {pl} IS NULL AND {nh} IS NULL AND {nl} IS NULL");
        let all_null=format!("{present}=0 AND {bad}=0 AND {n}=0 AND {ph}=0 AND {pl}=0 AND {nh}=0 AND {nl}=0");
        let populated=format!("{present}=1 AND {bad}=0 AND {n} BETWEEN 1 AND {MAX_POPULATION} AND {ph} IS NOT NULL AND {pl} IS NOT NULL AND {nh} IS NOT NULL AND {nl} IS NOT NULL AND {ph}>=0 AND {pl}>=0 AND {nh}>=0 AND {nl}>=0 AND {} IS NOT NULL AND {} IS NOT NULL AND {} IS NOT NULL AND {} IS NOT NULL",
            pos("__p_hi"),pos("__p_lo"),pos("__n_hi"),pos("__n_lo"));
        let invalid=format!("CASE WHEN {empty} THEN FALSE WHEN {all_null} THEN FALSE WHEN {populated} THEN FALSE ELSE TRUE END");
        let rawdigits=format!("COALESCE(NULLIF(REGEXP_REPLACE(CONCAT(CAST(({high}) AS STRING), LPAD(CAST(({low}) AS STRING), 19, '0')), '^0+', ''), ''), '0')");
        let neg=format!("({present}=1 AND ({greater})=FALSE AND {rawdigits}<>'0')");
        let formatted=if carrier==&Carrier::Integer {
            format!("CASE WHEN {invalid} THEN raise_error('WFT-CAPABILITY') WHEN {present} IS NULL OR {present}=0 THEN NULL ELSE CONCAT(CASE WHEN {neg} THEN '-' ELSE '' END, {rawdigits}) END")
        } else if *scale==0 {
            format!("CASE WHEN {invalid} THEN raise_error('WFT-CAPABILITY') WHEN {present} IS NULL OR {present}=0 THEN NULL ELSE CONCAT(CASE WHEN {neg} THEN '-' ELSE '' END, {rawdigits}) END")
        } else {
            let padded=format!("CASE WHEN LENGTH({rawdigits}) < {} THEN LPAD({rawdigits}, {}, '0') ELSE {rawdigits} END",(*scale as usize)+1,(*scale as usize)+1);
            format!("CASE WHEN {invalid} THEN raise_error('WFT-CAPABILITY') WHEN {present} IS NULL OR {present}=0 THEN NULL ELSE CONCAT(CASE WHEN {neg} THEN '-' ELSE '' END, SUBSTRING({padded},1,LENGTH({padded})-{}),'.',SUBSTRING({padded},LENGTH({padded})-{}+1,{})) END",scale,scale,scale)
        };
        let output_index=group_count+i;
        public_projection.push(format!("{formatted} AS {}",ident(&format!("__weft_out_{output_index}"))?));
        let source_ref=source_ref(a,logical)?;
        let (native_name,_,_,_)=r.binding.tables[0].columns.iter().find(|c|c.logical==*logical).map(|c|(&c.native,&c.carrier,c.nullable,&c.logical)).ok_or("SUM physical column absent")?;
        let source_facets=source_facet_json(a,logical)?;
        outputs.push(json!({"name":format!("__weft_out_{output_index}"),"role":"measure","function":"sum","fieldRef":source_ref,
            "nativeName":native_name,"carrier":if carrier==&Carrier::Integer{"integer"}else{"decimal"},"nullable":true,"sourceFacets":source_facets,
            "decoder":if carrier==&Carrier::Integer{json!({"kind":"canonical-signed-integer-text","maximumDigits":57})}else{json!({"kind":"canonical-fixed-scale-decimal-text","scale":scale,"maximumCoefficientDigits":57})}}));
        let sign_expr=format!("CASE WHEN {present} IS NULL OR {present}=0 THEN NULL WHEN {greater} THEN 1 ELSE -1 END");
        let high_expr=high.clone(); let low_expr=low.clone();
        sort_keys.insert(format!("__weft_out_{output_index}"),(sign_expr,high_expr,low_expr));
        let _=(nullable,precision);
    }
    let mut order_parts=Vec::new();
    let valid_group_aliases=(0..group_specs.len()).map(|i|format!("__weft_out_{i}")).collect::<BTreeSet<_>>();
    if let Some(order)=&q.order_by {
        for item in &order.exprs {
            let Expr::Identifier(identifier)=&item.expr else { return Err("SUM order must reference output alias"); };
            let output=name(identifier);
            let direction=if item.asc==Some(false){"DESC"}else{"ASC"};
            let nulls=if item.nulls_first==Some(true){"FIRST"}else if item.nulls_first==Some(false){"LAST"}else{"LAST"};
            if let Some((sign,high,low))=sort_keys.get(&output) {
                let null_rank=if nulls=="FIRST" {format!("CASE WHEN {sign} IS NULL THEN 0 ELSE 1 END ASC")} else {format!("CASE WHEN {sign} IS NULL THEN 1 ELSE 0 END ASC")};
                let native_dir=direction;
                order_parts.push(null_rank);
                order_parts.push(format!("CASE WHEN {sign}=-1 THEN -({high}) ELSE ({high}) END {native_dir}"));
                order_parts.push(format!("CASE WHEN {sign}=-1 THEN -({low}) ELSE ({low}) END {native_dir}"));
            } else if valid_group_aliases.contains(&output) {
                order_parts.push(format!("{} {direction} NULLS {nulls}",ident(&output)?));
            } else { return Err("SUM order output alias absent"); }
        }
    }
    for (i,(group_alias,_,carrier,_)) in group_specs.iter().enumerate() {
        let out_name=format!("__weft_out_{i}");
        if !order_parts.iter().any(|p|p.contains(&ident(&out_name).unwrap())) {
            order_parts.push(format!("{} ASC NULLS LAST",if carrier==&Carrier::String {format!("({}.{} COLLATE UTF8_BINARY)",first,ident(group_alias)?)} else {format!("{}.{}",first,ident(group_alias)?)}));
        }
    }
    let mut final_sql=format!("{sql} SELECT {} FROM {first}{joins}",public_projection.join(", "));
    if !order_parts.is_empty() { final_sql.push_str(&format!(" ORDER BY {}",order_parts.join(", "))); }
    final_sql.push_str(&format!(" LIMIT {}",r.row_cap));
    if final_sql.len()>65_536 || lower.used.len()!=r.parameters.len() { return Err("expanded SUM SQL/parameter limit or coverage refused"); }
    let referenced_columns=r.binding.tables[0].columns.iter().map(|c|json!({"logical":c.logical,"native":c.native,"carrier":c.carrier,"nullable":c.nullable})).collect::<Vec<_>>();
    Ok(json!({"version":"weft.unity-current-selection/0.5","status":"compiled","sql":final_sql,"parameters":r.parameters,
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
    Err("source SUM facets absent")
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
            if (carrier==Carrier::Integer && field["scalarType"]!="integer") || (carrier==Carrier::Decimal && field["scalarType"]!="decimal") { return Err("SUM carrier differs from original Field"); }
            if carrier==Carrier::Integer {
                if field["facets"].as_object().is_none_or(|o| o.len()!=1)
                    || field["facets"]["integerWidth"].as_object().is_none_or(|o| o.len()!=2)
                    || field["facets"]["integerWidth"].get("bits").is_none()
                    || field["facets"]["integerWidth"].get("signed").is_none()
                { return Err("SUM integer facets must be exactly signed width"); }
                let bits=field["facets"]["integerWidth"]["bits"].as_u64().filter(|b|matches!(b,8|16|32|64)).ok_or("SUM signed integer width required")?;
                if field["facets"]["integerWidth"]["signed"]!=true { return Err("unsigned SUM unsupported"); }
                return Ok((bits,0));
            }
            if field["facets"].as_object().is_none_or(|o| o.len()!=2)
                || field["facets"].get("precision").is_none()
                || field["facets"].get("scale").is_none()
            { return Err("SUM decimal facets must be exactly precision and scale"); }
            let p=field["facets"]["precision"].as_u64().filter(|p|(1..=38).contains(p)).ok_or("SUM decimal precision required")?;
            let s=field["facets"]["scale"].as_u64().filter(|s|*s<=p).ok_or("SUM decimal scale required")?;
            return Ok((p,s));
        }
    }
    Err("SUM source Field identity absent")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(ph: u128, pl: u128, nh: u128, nl: u128, n: u64) -> SignedLimbs {
        SignedLimbs { positive_high: ph, positive_low: pl, negative_high: nh,
            negative_low: nl, nonnull_count: Some(n), has_nonnull: n > 0, malformed: false }
    }

    #[test]
    fn exact_carry_borrow_and_sign_cancellation() {
        assert_eq!(coefficient(state(0, BASE - 1, 0, 0, 1)).unwrap(), Some("9999999999999999999".into()));
        assert_eq!(coefficient(state(0, BASE, 0, 0, 2)).unwrap(), Some("10000000000000000000".into()));
        assert_eq!(coefficient(state(1, 0, 0, 1, 2)).unwrap(), Some("9999999999999999999".into()));
        assert_eq!(coefficient(state(1, 0, 1, 0, 2)).unwrap(), Some("0".into()));
        assert_eq!(coefficient(state(0, 0, 0, 1, 1)).unwrap(), Some("-1".into()));
        assert_eq!(coefficient(state(1, 8_446_744_073_709_551_614, 0, 0, 2)).unwrap(),
            Some("18446744073709551614".into()));
        assert_eq!(coefficient(state(0, 0, 0, 9_223_372_036_854_775_808, 2)).unwrap(),
            Some("-9223372036854775808".into()));
    }

    #[test]
    fn empty_all_null_and_population_boundary_are_distinct() {
        let mut empty = state(0, 0, 0, 0, 0);
        empty.nonnull_count = None;
        empty.has_nonnull = false;
        assert_eq!(coefficient(empty).unwrap(), None);
        assert_eq!(coefficient(state(0, 0, 0, 0, i64::MAX as u64)).unwrap(), Some("0".into()));
        assert!(coefficient(state(0, 0, 0, 0, i64::MAX as u64 + 1)).is_err());
    }

    #[test]
    fn maximum_decimal_coefficient_and_corrupt_state() {
        let max38: u128 = 10_u128.pow(38) - 1;
        let twice_max38 = max38 * 2;
        let high = twice_max38 / BASE;
        let low = twice_max38 % BASE;
        // Two non-null maximum coefficients are represented by the true
        // aggregate limb totals; `nonnull_count` validates population and is
        // never used as an implicit multiplier.
        assert_eq!(coefficient(state(high, low, 0, 0, 2)).unwrap(),
            Some("199999999999999999999999999999999999998".into()));
        let mut bad = state(0, 0, 0, 0, 1);
        bad.malformed = true;
        assert!(coefficient(bad).unwrap_err().contains("WFT-CAPABILITY"));
        let mut overflow = state(0, 0, 0, 0, 1);
        overflow.nonnull_count = None;
        assert!(coefficient(overflow).is_err());
    }

    #[test]
    fn decimal_scale_is_exact_and_zero_has_no_negative_sign() {
        assert_eq!(fixed_scale_decimal("123", 0).unwrap(), "123");
        assert_eq!(fixed_scale_decimal("123", 2).unwrap(), "1.23");
        assert_eq!(fixed_scale_decimal("-1", 10).unwrap(), "-0.0000000001");
        assert_eq!(fixed_scale_decimal("-000", 38).unwrap(), format!("0.{}", "0".repeat(38)));
    }
}
