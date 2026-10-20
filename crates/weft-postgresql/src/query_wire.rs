// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wire-safe projections and bounded PostgreSQL frames for authored queries.
//!
//! The caller must first resolve and validate the source expression/query. This
//! module only changes its native result encoding and applies transport bounds.
use serde_json::{Value as JsonValue, json};
use sqlparser::ast::*;

type Result<T> = std::result::Result<T, &'static str>;

fn pg_call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Function(Function {
        name: ObjectName(vec![
            ObjectNamePart::Identifier(Ident::new("pg_catalog")),
            ObjectNamePart::Identifier(Ident::new(name)),
        ]),
        uses_odbc_syntax: false,
        parameters: FunctionArguments::None,
        args: FunctionArguments::List(FunctionArgumentList {
            duplicate_treatment: None,
            args: args.into_iter().map(|e| FunctionArg::Unnamed(FunctionArgExpr::Expr(e))).collect(),
            clauses: vec![],
        }),
        within_group: vec![],
        filter: None,
        null_treatment: None,
        over: None,
    })
}

fn pg_cast(expr: Expr, data_type: DataType) -> Expr {
    Expr::Cast { kind: CastKind::Cast, expr: Box::new(expr), data_type, format: None }
}

fn literal(s: &str) -> Expr {
    Expr::Value(ValueWithSpan::from(sqlparser::ast::Value::SingleQuotedString(s.to_owned())))
}

/// Convert one already-resolved PostgreSQL expression to the exact native wire
/// representation expected for its declared carrier.
pub(crate) fn wire_projection(expr: Expr, carrier: &str) -> Result<(Expr, Option<&'static str>)> {
    Ok(match carrier {
        "boolean" => (expr, None),
        "string" | "integer" | "decimal" | "uuid" => (pg_cast(expr, DataType::Text), None),
        "date" => (
            pg_call("to_char", vec![pg_cast(expr, DataType::Timestamp(None, TimezoneInfo::None)), literal("YYYY-MM-DD")]),
            None,
        ),
        "timestamp" => {
            let utc = Expr::AtTimeZone { timestamp: Box::new(expr), time_zone: Box::new(literal("UTC")) };
            (pg_call("to_char", vec![utc, literal("YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"")]), None)
        }
        "float" => (
            pg_call("encode", vec![pg_call("float8send", vec![expr]), literal("hex")]),
            Some("ieee754-binary64-hex"),
        ),
        "bits128" => (pg_call("encode", vec![expr, literal("hex")]), None),
        _ => return Err("query output carrier has no qualified native codec"),
    })
}

fn quoted(value: &str) -> Result<String> {
    if value.is_empty() || value.len() > 63 || value.contains('\0') {
        return Err("query wire output alias outside PostgreSQL identifier profile");
    }
    Ok(format!("\"{}\"", value.replace('"', "\"\"")))
}

fn ident(value: &str) -> Result<Ident> {
    quoted(value)?;
    // sqlparser's formatter escapes embedded quotes itself.
    Ok(Ident::with_quote('"', value.to_owned()))
}

fn source_column(name: &str) -> Result<Expr> {
    Ok(Expr::CompoundIdentifier(vec![ident("__weft_source")?, ident(name)?]))
}

fn qualify_sort_expr(expr: &Expr) -> Result<Expr> {
    match expr {
        Expr::Identifier(i) => Ok(Expr::CompoundIdentifier(vec![ident("__weft_source")?, i.clone()])),
        Expr::Collate { expr, collation } => Ok(Expr::Collate {
            expr: Box::new(qualify_sort_expr(expr)?),
            collation: collation.clone(),
        }),
        _ => Err("query order descriptor must reference a hidden sort alias"),
    }
}

fn sort_sql(order: &[OrderByExpr]) -> Result<String> {
    if order.iter().any(|item| item.with_fill.is_some()) {
        return Err("query order descriptor WITH FILL unsupported");
    }
    order.iter().map(|item| {
        Ok(format!("{}{}", qualify_sort_expr(&item.expr)?, item.options))
    }).collect::<Result<Vec<_>>>().map(|parts| parts.join(", "))
}

/// Wrap the validated, source-resolved, typed SELECT in a bounded wire frame.
/// Authored order/limit/offset stay native and intact. `sort_order` references
/// hidden aliases supplied by the compiler and carries exact direction, nulls,
/// and collation. The business wire projection happens outside the typed query.
pub(crate) fn bound_frame_sql(
    source_query: &str,
    outputs: &[JsonValue],
    sort_order: &[OrderByExpr],
    max_rows: u64,
) -> Result<String> {
    if source_query.trim().is_empty() || outputs.is_empty() || max_rows == 0 {
        return Err("query wire frame requires source SQL, outputs, and positive row bound");
    }
    let cap = max_rows.checked_add(1).ok_or("query row bound overflow")?;
    let mut names = Vec::with_capacity(outputs.len());
    let mut projections = Vec::with_capacity(outputs.len());
    for output in outputs {
        let name = output["name"].as_str().ok_or("query wire output name absent")?;
        let alias = quoted(name)?;
        let carrier = output["carrier"].as_str().ok_or("query wire output carrier absent")?;
        let (expr, _) = wire_projection(source_column(name)?, carrier)?;
        names.push(alias.clone());
        projections.push(format!("{expr} AS {alias}"));
    }
    if names.iter().collect::<std::collections::BTreeSet<_>>().len() != names.len() {
        return Err("query wire output names must be unique");
    }
    let row_order = if sort_order.is_empty() { String::new() } else { format!("ORDER BY {}", sort_sql(sort_order)?) };
    let sizes = names.iter().map(|name| {
        format!("COALESCE(pg_catalog.octet_length(f.{name}::pg_catalog.text)::pg_catalog.int8,0)")
    }).collect::<Vec<_>>().join(" + ");
    Ok(format!(
        "WITH \"__weft_frame\" AS MATERIALIZED (SELECT ROW_NUMBER() OVER ({row_order}) AS \"__weft_ordinal\", {} FROM (SELECT * FROM ({source_query}) AS \"__weft_authored\" LIMIT {cap}) AS \"__weft_source\"), \"__weft_guard\" AS MATERIALIZED (SELECT CASE WHEN COALESCE(pg_catalog.sum({sizes}),0) <= 8388608 THEN 1::pg_catalog.int8 ELSE 1::pg_catalog.int8 / (pg_catalog.count(*) - pg_catalog.count(*)) END AS ok FROM \"__weft_frame\" AS f) SELECT {} FROM \"__weft_frame\" AS f CROSS JOIN \"__weft_guard\" AS b WHERE b.ok=1 ORDER BY f.\"__weft_ordinal\"",
        projections.join(", "),
        names.iter().map(|name| format!("f.{name}")).collect::<Vec<_>>().join(", "),
    ))
}

/// Build cold native-description SQL for public columns of the authored typed
/// query before wire conversion and transport guards. Hidden sort projections
/// are omitted; placeholders remain unchanged.
pub(crate) fn describe_sql(source_query: &str, outputs: &[JsonValue]) -> Result<String> {
    if source_query.trim().is_empty() || outputs.is_empty() { return Err("query description source or outputs absent"); }
    let mut names = Vec::with_capacity(outputs.len());
    for output in outputs {
        names.push(quoted(output["name"].as_str().ok_or("query description output name absent")?)?);
    }
    if names.iter().collect::<std::collections::BTreeSet<_>>().len() != names.len() {
        return Err("query description output names must be unique");
    }
    let projection = names.iter().map(|name| format!("\"__weft_description_source\".{name}")).collect::<Vec<_>>().join(", ");
    Ok(format!("SELECT {projection} FROM ({source_query}) AS \"__weft_description_source\" LIMIT 0"))
}

/// Attach the package's shared `$n` to DBAPI `%s` occurrence mapping and
/// literal-percent escaping to a compiled native program.
pub(crate) fn attach_dbapi(program: &mut JsonValue) -> Result<()> {
    crate::dbapi::attach(program)?;
    if let Some(description_sql) = program["nativeDescriptionSql"].as_str() {
        let mut description = json!({
            "sql": description_sql,
            "parameterOrder": program["parameterOrder"],
        });
        crate::dbapi::attach(&mut description)?;
        program["nativeDescriptionDbapiSql"] = description["dbapiSql"].clone();
        program["nativeDescriptionDbapiParameterOrder"] = description["dbapiParameterOrder"].clone();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlparser::{dialect::PostgreSqlDialect, parser::Parser};
    use serde_json::json;

    fn expression(sql: &str) -> Expr {
        let parsed = Parser::parse_sql(&PostgreSqlDialect {}, &format!("SELECT {sql}")).unwrap();
        let Statement::Query(query) = &parsed[0] else { panic!() };
        let SetExpr::Select(select) = query.body.as_ref() else { panic!() };
        let SelectItem::UnnamedExpr(expr) = &select.projection[0] else { panic!() };
        expr.clone()
    }

    #[test]
    fn float_projection_is_binary64_hex_and_other_scalar_encodings_are_explicit() {
        let (expr, encoding) = wire_projection(expression("0.1::float8"), "float").unwrap();
        assert_eq!(encoding, Some("ieee754-binary64-hex"));
        let rendered = expr.to_string();
        assert!(rendered.contains("float8send"), "{rendered}");
        assert!(rendered.contains("encode"), "{rendered}");
        assert!(rendered.contains("hex"), "{rendered}");
        assert!(rendered.contains("0.1"), "finite input disappeared: {rendered}");
        assert_eq!(wire_projection(expression("t.x"), "boolean").unwrap().1, None);
        assert!(wire_projection(expression("t.x"), "opaque").is_err());
    }

    #[test]
    fn bounded_frame_keeps_typed_source_semantics_and_adds_wire_projection_guard_and_ordinal() {
        let source = "SELECT t.\"a\" AS \"odd\", t.\"sort0\" AS \"__sort0\" FROM \"s\".\"t\" AS t WHERE t.x = $1 ORDER BY t.\"sort0\" DESC LIMIT 90 OFFSET 7";
        let order = vec![OrderByExpr {
            expr: Expr::Identifier(ident("__sort0").unwrap()),
            options: OrderByOptions { sort: Some(OrderBySort::Desc), nulls_first: Some(false) },
            with_fill: None,
        }];
        let sql = bound_frame_sql(source, &[json!({"name":"odd","carrier":"string"})], &order, 20).unwrap();
        assert!(sql.contains("LIMIT 90 OFFSET 7"), "{sql}");
        assert!(sql.contains("ORDER BY t.\"sort0\" DESC"), "{sql}");
        assert!(sql.contains("AS \"__weft_authored\" LIMIT 21"), "{sql}");
        assert!(sql.contains("ROW_NUMBER() OVER (ORDER BY \"__weft_source\".\"__sort0\" DESC NULLS LAST)"), "{sql}");
        assert!(sql.contains("CAST(\"__weft_source\".\"odd\" AS TEXT) AS \"odd\""), "{sql}");
        assert_eq!(sql.matches("octet_length").count(), 1, "{sql}");
        assert!(sql.contains("<= 8388608"), "{sql}");
        assert!(sql.contains("count(*) - pg_catalog.count(*)"), "{sql}");
        Parser::parse_sql(&PostgreSqlDialect {}, &sql).unwrap();
    }

    #[test]
    fn frame_uses_quoted_aliases_and_rejects_duplicate_or_overflowing_metadata() {
        let sql = bound_frame_sql("SELECT 1 AS \"a\"\"b\"", &[json!({"name":"a\"b","carrier":"integer"})], &[], u64::MAX - 1).unwrap();
        assert!(sql.contains("f.\"a\"\"b\"::pg_catalog.text"), "{sql}");
        assert!(bound_frame_sql("SELECT 1", &[json!({"name":"x","carrier":"integer"})], &[], u64::MAX).is_err());
        assert!(bound_frame_sql("SELECT 1", &[json!({"name":"x","carrier":"integer"}),json!({"name":"x","carrier":"integer"})], &[], 2).is_err());
    }

    #[test]
    fn description_keeps_placeholders_and_dbapi_maps_repeats_and_percent_literals() {
        let describe = describe_sql(
            "SELECT $1::numeric AS x, $1::text AS y, $2::int8 AS __sort WHERE '50%' = $2::text",
            &[json!({"name":"x"}), json!({"name":"y"})],
        ).unwrap();
        assert!(describe.contains("$1"));
        assert!(describe.contains("50%"));
        assert!(describe.contains("\"__weft_description_source\".\"x\""));
        assert!(!describe.contains("\"__sort\""));
        let mut program = json!({
            "sql":"SELECT $2, $1, $2, '50%'",
            "nativeDescriptionSql":"SELECT $1::numeric AS x, $1::text AS y, '50%'",
            "parameterOrder":["first","second"]
        });
        attach_dbapi(&mut program).unwrap();
        assert_eq!(program["dbapiSql"], "SELECT %s, %s, %s, '50%%'");
        assert_eq!(program["dbapiParameterOrder"], json!(["second","first","second"]));
        assert_eq!(program["nativeDescriptionDbapiSql"], "SELECT %s::numeric AS x, %s::text AS y, '50%%'");
        assert_eq!(program["nativeDescriptionDbapiParameterOrder"], json!(["first","first"]));
    }

}
