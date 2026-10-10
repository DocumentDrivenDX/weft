//! Private 0.4 query syntax; no public dialect or semantic admission is added here.
//! Legacy query segments share their lexer, expression budget and source spans.
use crate::{
    arithmetic_query as legacy,
    arithmetic_syntax::Budget,
    error::{Diagnostic, Result},
    ir::Span,
    path_syntax::{self, PathArguments},
    syntax::{Column, Name, Parser, Source},
};

#[derive(Debug, Clone)]
pub(crate) enum Output {
    Legacy(legacy::Output),
    RelatedPaths {
        path: PathArguments,
        bound: u16,
        bound_span: Span,
        span: Span,
    },
    CountDistinctPathTargets {
        alias: Name,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct Projection {
    pub output: Output,
    pub alias: Option<Name>,
}

#[derive(Debug, Clone)]
pub(crate) struct Expansion {
    pub path: PathArguments,
    pub span: Span,
    pub alias: Name,
}

#[derive(Debug, Clone)]
pub(crate) struct Query {
    pub distinct: bool,
    pub outputs: Vec<Projection>,
    pub source: Source,
    pub joins: Vec<legacy::Join>,
    pub expansion: Option<Expansion>,
    pub predicates: Vec<legacy::Predicate>,
    pub groups: Vec<Column>,
    pub having: Vec<legacy::Having>,
    pub order: Vec<Column>,
    pub limit: Option<u16>,
}

fn projection(p: &mut Parser, budget: &mut Budget) -> Result<Projection> {
    if !p.peek_word("related_paths") && !p.peek_word("count_distinct_path_targets") {
        let legacy::Projection { output, alias } = legacy::projection(p, budget)?;
        return Ok(Projection {
            output: Output::Legacy(output),
            alias,
        });
    }
    let expression = path_syntax::parse(p)?;
    let output = match expression.kind {
        path_syntax::Kind::RelatedPaths {
            path,
            bound,
            bound_span,
        } => Output::RelatedPaths {
            path,
            bound,
            bound_span,
            span: expression.span,
        },
        path_syntax::Kind::CountDistinctPathTargets { alias } => Output::CountDistinctPathTargets {
            alias,
            span: expression.span,
        },
        path_syntax::Kind::ExpandPaths { .. } => unreachable!("projection function selected above"),
    };
    let alias = if p.peek_word("as") {
        p.word("as")?;
        Some(p.name()?)
    } else {
        None
    };
    Ok(Projection { output, alias })
}

fn expansion(p: &mut Parser) -> Result<Option<Expansion>> {
    if !p.peek_word("cross") {
        return Ok(None);
    }
    p.word("cross")?;
    p.word("join")?;
    // Select this function before the fragment parser, which also owns projections.
    if !p.peek_word("expand_paths") {
        return Err(Diagnostic::new(
            "WFT-UNSUPPORTED",
            "parse",
            "Path CROSS JOIN requires EXPAND_PATHS",
        )
        .at(&p.span()));
    }
    let expression = path_syntax::parse(p)?;
    let path_syntax::Kind::ExpandPaths { path } = expression.kind else {
        unreachable!("expansion function selected above")
    };
    if p.peek_word("as") {
        p.word("as")?;
    }
    let alias = p.name()?;
    Ok(Some(Expansion {
        path,
        span: expression.span,
        alias,
    }))
}

/// The resolver owns grouping, alias scope, relationship continuity and admission.
/// Syntax fixes expansion after ordinary joins and retains the existing HAVING form.
pub(crate) fn parse(sql: &str) -> Result<Query> {
    let mut p = Parser::new(sql)?;
    let mut budget = Budget::new();
    p.word("select")?;
    let distinct = if p.peek_word("distinct") {
        p.word("distinct")?;
        true
    } else {
        false
    };
    let mut outputs = Vec::new();
    loop {
        outputs.push(projection(&mut p, &mut budget)?);
        if outputs.len() > 256 {
            return Err(
                Diagnostic::new("WFT-LIMIT", "parse", "Output count exceeds limit").at(&p.span()),
            );
        }
        if !p.peek_symbol(',') {
            break;
        }
        p.symbol(',')?;
    }
    p.word("from")?;
    let source = p.source()?;
    let joins = legacy::joins(&mut p, &mut budget)?;
    let expansion = expansion(&mut p)?;
    let legacy::Tail {
        predicates,
        groups,
        having,
        order,
        limit,
    } = legacy::tail(&mut p, &mut budget)?;
    Ok(Query {
        distinct,
        outputs,
        source,
        joins,
        expansion,
        predicates,
        groups,
        having,
        order,
        limit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const COLLECTION: &str = r#"SELECT l.id,
       RELATED_PATHS(l."order_lines.product_id", "products.supplier_id", 20)
         AS supplier_paths
FROM order_lines l ORDER BY l.id;"#;
    const COUNTS: &str = r#"SELECT l.id, COUNT(*) AS path_count,
       COUNT_DISTINCT_PATH_TARGETS(p) AS supplier_count
FROM order_lines l
CROSS JOIN EXPAND_PATHS(l."order_lines.product_id", "products.supplier_id") AS p
GROUP BY l.id ORDER BY l.id;"#;

    fn original<'a>(sql: &'a str, span: &Span) -> &'a str {
        &sql[span.start..span.end]
    }

    fn legacy_query(query: Query) -> legacy::Query {
        assert!(query.expansion.is_none());
        legacy::Query {
            distinct: query.distinct,
            outputs: query
                .outputs
                .into_iter()
                .map(|p| {
                    let Output::Legacy(output) = p.output else {
                        panic!("legacy fixture acquired a path output")
                    };
                    legacy::Projection {
                        output,
                        alias: p.alias,
                    }
                })
                .collect(),
            source: query.source,
            predicates: query.predicates,
            joins: query.joins,
            groups: query.groups,
            having: query.having,
            order: query.order,
            limit: query.limit,
        }
    }

    #[test]
    fn original_commerce_collection_preserves_function_and_operand_spans() {
        let query = parse(COLLECTION).unwrap();
        assert!(!query.distinct);
        assert!(query.expansion.is_none());
        assert_eq!(query.outputs.len(), 2);
        assert_eq!(query.source.name.value, "order_lines");
        let Output::RelatedPaths {
            path,
            bound,
            bound_span,
            span,
        } = &query.outputs[1].output
        else {
            panic!("expected collection")
        };
        assert_eq!(*bound, 20);
        assert_eq!(original(COLLECTION, bound_span), "20");
        assert_eq!(
            original(COLLECTION, &path.first.span),
            r#"l."order_lines.product_id""#
        );
        assert_eq!(path.first.field.value, "order_lines.product_id");
        assert_eq!(path.second.value, "products.supplier_id");
        assert_eq!(
            original(COLLECTION, &path.second.span),
            r#""products.supplier_id""#
        );
        assert_eq!(
            original(COLLECTION, span),
            r#"RELATED_PATHS(l."order_lines.product_id", "products.supplier_id", 20)"#
        );
        assert_eq!(
            query.outputs[1].alias.as_ref().unwrap().value,
            "supplier_paths"
        );
        assert_eq!(original(COLLECTION, &query.order[0].span), "l.id");
    }

    #[test]
    fn original_commerce_expansion_and_both_counts_retain_occurrence_spans() {
        let query = parse(COUNTS).unwrap();
        assert_eq!(query.outputs.len(), 3);
        assert!(matches!(
            query.outputs[1].output,
            Output::Legacy(legacy::Output::Count)
        ));
        let Output::CountDistinctPathTargets { alias, span } = &query.outputs[2].output else {
            panic!("expected target count")
        };
        assert_eq!(original(COUNTS, span), "COUNT_DISTINCT_PATH_TARGETS(p)");
        assert_eq!(original(COUNTS, &alias.span), "p");
        let expansion = query.expansion.unwrap();
        assert_eq!(expansion.alias.value, "p");
        assert_eq!(original(COUNTS, &expansion.alias.span), "p");
        assert_ne!(alias.span.start, expansion.alias.span.start);
        assert_eq!(
            original(COUNTS, &expansion.span),
            r#"EXPAND_PATHS(l."order_lines.product_id", "products.supplier_id")"#
        );
        assert_eq!(expansion.path.first.alias.value, "l");
        assert_eq!(query.groups.len(), 1);
        assert_eq!(original(COUNTS, &query.groups[0].span), "l.id");
        assert_eq!(original(COUNTS, &query.order[0].span), "l.id");
    }

    #[test]
    fn escaped_unicode_names_and_optional_as_keep_original_byte_offsets() {
        let sql = r#"SELECT COUNT_DISTINCT_PATH_TARGETS("p""å") AS "n""å" FROM "lïnes" "L" CROSS JOIN EXPAND_PATHS("L"."f""ø", "s""é") "p""å""#;
        let query = parse(sql).unwrap();
        let Output::CountDistinctPathTargets { alias, span } = &query.outputs[0].output else {
            panic!("expected target count")
        };
        assert_eq!(alias.value, "p\"å");
        assert!(alias.quoted);
        assert_eq!(original(sql, &alias.span), r#""p""å""#);
        assert_eq!(
            original(sql, span),
            r#"COUNT_DISTINCT_PATH_TARGETS("p""å")"#
        );
        assert_eq!(query.outputs[0].alias.as_ref().unwrap().value, "n\"å");
        let expansion = query.expansion.unwrap();
        assert_eq!(expansion.alias.value, alias.value);
        assert_eq!(expansion.path.first.field.value, "f\"ø");
        assert_eq!(original(sql, &expansion.path.first.field.span), r#""f""ø""#);
        assert_eq!(expansion.path.second.value, "s\"é");
        assert_eq!(original(sql, &expansion.path.second.span), r#""s""é""#);
    }

    #[test]
    fn expansion_follows_ordinary_joins_and_keeps_legacy_tail() {
        let sql = "SELECT DISTINCT l.id,COUNT(*),COUNT_DISTINCT_PATH_TARGETS(p),COUNT(DISTINCT f.name) AS names FROM lines l INNER JOIN fulfillments f ON l.id=f.line_id LEFT JOIN returns r ON l.id=r.line_id CROSS JOIN EXPAND_PATHS(r.product,supplier) AS p WHERE l.quantity-f.quantity+r.quantity>0 AND f.name IN ('A','B') AND r.name IS NOT NULL GROUP BY l.id HAVING COUNT(DISTINCT f.name)>1 ORDER BY l.id ASC LIMIT 10";
        let query = parse(sql).unwrap();
        assert!(query.distinct);
        assert_eq!(query.joins.len(), 2);
        assert!(!query.joins[0].left);
        assert!(query.joins[1].left);
        assert_eq!(query.expansion.unwrap().path.first.alias.value, "r");
        assert_eq!(query.predicates.len(), 3);
        assert!(matches!(
            query.predicates[0],
            legacy::Predicate::ArithmeticCompare { .. }
        ));
        assert!(matches!(
            query.predicates[1],
            legacy::Predicate::StringIn { .. }
        ));
        assert!(matches!(
            query.predicates[2],
            legacy::Predicate::NullTest { negated: true, .. }
        ));
        assert_eq!(query.having.len(), 1);
        assert_eq!(original(sql, &query.having[0].argument.span), "f.name");
        assert_eq!(original(sql, &query.having[0].threshold.span), "1");
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn expansion_placement_arity_alias_and_trailing_tokens_are_closed() {
        for sql in [
            "SELECT l.id FROM EXPAND_PATHS(l.product,supplier) p",
            "SELECT EXPAND_PATHS(l.product,supplier) FROM lines l",
            "SELECT l.id FROM lines l CROSS JOIN products p",
            "SELECT l.id FROM lines l CROSS JOIN RELATED_PATHS(l.product,supplier,1) p",
            "SELECT l.id FROM lines l LEFT JOIN EXPAND_PATHS(l.product,supplier) p",
            "SELECT l.id FROM lines l CROSS JOIN LATERAL EXPAND_PATHS(l.product,supplier) p",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(l.product,supplier)",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(l.product,supplier) AS",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(l.product,supplier) AS p.x",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(l.product,supplier) AS 'p'",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(l.product,supplier) AS select",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(l.product,supplier) AS p(q)",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(l.product,supplier,1) p",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(l.product) p",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(product,supplier) p",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(l.product,p.supplier) p",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(l.product,supplier) p CROSS JOIN EXPAND_PATHS(l.product,supplier) q",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(l.product,supplier) p JOIN products r ON l.id=r.id",
            "SELECT l.id FROM lines l WHERE l.id=1 CROSS JOIN EXPAND_PATHS(l.product,supplier) p",
            "SELECT RELATED_PATHS(l.product,supplier,1) paths FROM lines l",
            "SELECT COUNT_DISTINCT_PATH_TARGETS(p.id) FROM lines l",
            "SELECT COUNT_DISTINCT_PATH_TARGETS(p,q) FROM lines l",
            "SELECT l.id FROM lines l;;",
            "SELECT l.id FROM lines l; junk",
        ] {
            assert!(parse(sql).is_err(), "{sql}");
        }
    }

    #[test]
    fn new_having_forms_are_not_silently_admitted() {
        for having in [
            "COUNT_DISTINCT_PATH_TARGETS(p)>1",
            "COUNT(*)>1",
            "COUNT(DISTINCT l.name)>=1",
            "COUNT(DISTINCT l.name)>:n",
            "COUNT(DISTINCT l.name)>-1",
            "COUNT(DISTINCT l.name)>1.0",
            "COUNT(DISTINCT l.name)>1 AND COUNT(DISTINCT l.name)>2",
        ] {
            let sql = format!("SELECT COUNT_DISTINCT_PATH_TARGETS(p),COUNT(DISTINCT l.name) FROM lines l CROSS JOIN EXPAND_PATHS(l.product,supplier) p HAVING {having}");
            assert!(parse(&sql).is_err(), "{sql}");
        }
    }

    #[test]
    fn shared_limits_cover_all_query_segments() {
        for bound in [
            "0",
            "1001",
            "-1",
            "+1",
            "1.0",
            ":n",
            "999999999999999999999",
        ] {
            let sql = format!("SELECT RELATED_PATHS(l.product,supplier,{bound}) FROM lines l");
            assert!(parse(&sql).is_err(), "{sql}");
        }
        for bound in [1, 1000] {
            assert!(parse(&format!(
                "SELECT RELATED_PATHS(l.product,supplier,{bound}) FROM lines l"
            ))
            .is_ok());
        }
        let outputs = vec!["COUNT_DISTINCT_PATH_TARGETS(p)"; 257].join(",");
        assert_eq!(
            parse(&format!("SELECT {outputs} FROM lines l"))
                .unwrap_err()
                .code,
            "WFT-LIMIT"
        );
        let joins = (0..17)
            .map(|i| format!(" JOIN products p{i} ON l.id=p{i}.id"))
            .collect::<String>();
        assert_eq!(
            parse(&format!("SELECT l.id FROM lines l{joins}"))
                .unwrap_err()
                .code,
            "WFT-LIMIT"
        );
        let arithmetic = (0..86)
            .map(|i| format!("l.id+1 AS n{i}"))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            parse(&format!(
                "SELECT {arithmetic},RELATED_PATHS(l.product,supplier,1) FROM lines l"
            ))
            .unwrap_err()
            .code,
            "WFT-LIMIT"
        );
        let too_long = format!("SELECT l.id FROM lines l{}", " ".repeat(65536));
        assert_eq!(parse(&too_long).unwrap_err().code, "WFT-LIMIT");
    }

    #[test]
    fn syntax_does_not_claim_resolver_admission() {
        // These parseable forms still require resolver rejection or qualification.
        for sql in [
            "SELECT COUNT_DISTINCT_PATH_TARGETS(missing) FROM lines l",
            "SELECT RELATED_PATHS(l.product,supplier,1),COUNT(*) FROM lines l GROUP BY l.id",
            "SELECT RELATED_PATHS(l.product,supplier,1) FROM lines l CROSS JOIN EXPAND_PATHS(l.product,supplier) p",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(missing.product,supplier) l WHERE l.id=1",
        ] {
            assert!(parse(sql).is_ok(), "{sql}");
        }
    }

    #[test]
    fn old_entrypoints_continue_to_refuse_new_query_forms() {
        for sql in [COLLECTION, COUNTS] {
            assert!(crate::syntax::parse(sql).is_err());
            assert!(crate::application_syntax::parse(sql).is_err());
            assert!(legacy::parse(sql).is_err());
        }
    }

    #[test]
    fn legacy_fixture_asts_and_diagnostics_keep_original_spans() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/application/fixtures/cases.json"
        ))
        .unwrap();
        let mut checked = 0;
        for case in cases.as_array().unwrap() {
            let Some(sql) = case
                .get("request")
                .and_then(|r| r.get("sql"))
                .and_then(|s| s.as_str())
            else {
                continue;
            };
            assert_eq!(
                format!("{:?}", parse(sql).map(legacy_query)),
                format!("{:?}", legacy::parse(sql)),
                "{sql}"
            );
            checked += 1;
        }
        assert!(checked >= 600);
        for sql in [
            "SELECT c.id+1 FROM Customer c",
            "SELECT c.* AS all FROM Customer c",
            "SELECT c.id FROM Customer c WHERE c.id > = 2",
            "SELECT COUNT(DISTINCT c.name) FROM Customer c HAVING COUNT(DISTINCT c.name)>:n",
            "SELECT c.id FROM Customer c ORDER BY c.id DESC",
            "SELECT c.id FROM Customer c LIMIT 0",
            "SELECT c.id FROM Customer c;;",
        ] {
            assert_eq!(
                format!("{:?}", parse(sql).unwrap_err()),
                format!("{:?}", legacy::parse(sql).unwrap_err()),
                "{sql}"
            );
        }
    }
}
