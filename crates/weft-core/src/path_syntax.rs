//! Private CONTRACT-005 syntax fragments, intentionally unwired from query entrypoints.
//! Parsing does not admit model semantics, relational composition or a backend.
use crate::{
    error::{Diagnostic, Result},
    ir::Span,
    syntax::{Column, LiteralKind, Name, Parser},
};

#[derive(Debug, Clone)]
pub(crate) struct PathArguments {
    pub first: Column,
    pub second: Name,
}

#[derive(Debug, Clone)]
pub(crate) enum Kind {
    RelatedPaths {
        path: PathArguments,
        bound: u16,
        bound_span: Span,
    },
    ExpandPaths {
        path: PathArguments,
    },
    CountDistinctPathTargets {
        alias: Name,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct Expression {
    pub kind: Kind,
    pub span: Span,
}

fn path_arguments(p: &mut Parser) -> Result<PathArguments> {
    // The first relationship must remain scan-qualified; the second is one name.
    let first = p.column()?;
    p.symbol(',')?;
    let second = p.name()?;
    Ok(PathArguments { first, second })
}

fn bound(p: &mut Parser) -> Result<(u16, Span)> {
    let literal = p.literal()?;
    let value = if matches!(literal.kind, LiteralKind::Number)
        && literal.value.bytes().all(|c| c.is_ascii_digit())
    {
        literal.value.parse::<u16>().ok()
    } else {
        None
    };
    let value = value.filter(|n| (1..=1000).contains(n)).ok_or_else(|| {
        Diagnostic::new(
            "WFT-LIMIT",
            "parse",
            "Path bound must be an unsigned integer literal from 1 through 1000",
        )
        .at(&literal.span)
    })?;
    Ok((value, literal.span))
}

/// Consume one function fragment from the shared parser, leaving query delimiters
/// to its caller. The existing lexer owns identifier rules and input/token limits.
pub(crate) fn parse(p: &mut Parser) -> Result<Expression> {
    let start = p.span().start;
    let kind = if p.peek_word("related_paths") {
        p.word("related_paths")?;
        p.symbol('(')?;
        let path = path_arguments(p)?;
        p.symbol(',')?;
        let (bound, bound_span) = bound(p)?;
        Kind::RelatedPaths {
            path,
            bound,
            bound_span,
        }
    } else if p.peek_word("expand_paths") {
        p.word("expand_paths")?;
        p.symbol('(')?;
        Kind::ExpandPaths {
            path: path_arguments(p)?,
        }
    } else if p.peek_word("count_distinct_path_targets") {
        p.word("count_distinct_path_targets")?;
        p.symbol('(')?;
        Kind::CountDistinctPathTargets { alias: p.name()? }
    } else {
        return Err(Diagnostic::new(
            "WFT-UNSUPPORTED",
            "parse",
            "Expected an authored path function",
        )
        .at(&p.span()));
    };
    let end = p.span().end;
    p.symbol(')')?;
    Ok(Expression {
        kind,
        span: Span { start, end },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fragment(sql: &str) -> Result<Expression> {
        let mut parser = Parser::new(sql)?;
        let expression = parse(&mut parser)?;
        parser.finish()?;
        Ok(expression)
    }

    fn original<'a>(sql: &'a str, span: &Span) -> &'a str {
        &sql[span.start..span.end]
    }

    #[test]
    fn original_commerce_collection_operands_and_spans() {
        let sql = r#"  RELATED_PATHS(l."order_lines.product_id", "products.supplier_id", 20)  "#;
        let expression = fragment(sql).unwrap();
        assert_eq!(original(sql, &expression.span), sql.trim());
        let Kind::RelatedPaths {
            path,
            bound,
            bound_span,
        } = expression.kind
        else {
            panic!("expected related paths")
        };
        assert!(!path.first.unqualified);
        assert_eq!(path.first.alias.value, "l");
        assert!(!path.first.alias.quoted);
        assert_eq!(original(sql, &path.first.alias.span), "l");
        assert_eq!(path.first.field.value, "order_lines.product_id");
        assert!(path.first.field.quoted);
        assert_eq!(
            original(sql, &path.first.span),
            r#"l."order_lines.product_id""#
        );
        assert_eq!(
            original(sql, &path.first.field.span),
            r#""order_lines.product_id""#
        );
        assert_eq!(path.second.value, "products.supplier_id");
        assert!(path.second.quoted);
        assert_eq!(
            original(sql, &path.second.span),
            r#""products.supplier_id""#
        );
        assert_eq!(bound, 20);
        assert_eq!(original(sql, &bound_span), "20");
    }

    #[test]
    fn quoted_escaping_unicode_bytes_and_case_folding() {
        let sql = r#" eXpAnD_PaThS("ré""c"."to""p", "next.β""key") "#;
        let expression = fragment(sql).unwrap();
        assert_eq!(original(sql, &expression.span), sql.trim());
        let Kind::ExpandPaths { path } = expression.kind else {
            panic!("expected expansion")
        };
        assert_eq!(path.first.alias.value, "ré\"c");
        assert_eq!(path.first.field.value, "to\"p");
        assert_eq!(path.second.value, "next.β\"key");
        assert_eq!(original(sql, &path.first.alias.span), r#""ré""c""#);
        assert_eq!(original(sql, &path.first.field.span), r#""to""p""#);
        assert_eq!(original(sql, &path.second.span), r#""next.β""key""#);
        let Kind::ExpandPaths { path } =
            fragment("EXPAND_PATHS(L.Product, Supplier)").unwrap().kind
        else {
            panic!("expected expansion")
        };
        assert_eq!(path.first.alias.value, "l");
        assert_eq!(path.first.field.value, "product");
        assert_eq!(path.second.value, "supplier");
        assert!(!path.second.quoted);
    }

    #[test]
    fn distinct_target_alias_has_its_original_name_span() {
        let sql = r#"  COUNT_DISTINCT_PATH_TARGETS("p""å") "#;
        let expression = fragment(sql).unwrap();
        assert_eq!(original(sql, &expression.span), sql.trim());
        let Kind::CountDistinctPathTargets { alias } = expression.kind else {
            panic!("expected distinct targets")
        };
        assert_eq!(alias.value, "p\"å");
        assert!(alias.quoted);
        assert_eq!(original(sql, &alias.span), r#""p""å""#);
        let Kind::CountDistinctPathTargets { alias } =
            fragment("COUNT_DISTINCT_PATH_TARGETS(PathAlias)")
                .unwrap()
                .kind
        else {
            panic!("expected distinct targets")
        };
        assert_eq!(alias.value, "pathalias");
        assert!(!alias.quoted);
    }

    #[test]
    fn bound_endpoints_and_invalid_literal_domains() {
        for n in [1, 1000] {
            let Kind::RelatedPaths { bound, .. } =
                fragment(&format!("RELATED_PATHS(l.product, supplier, {n})"))
                    .unwrap()
                    .kind
            else {
                panic!("expected collection")
            };
            assert_eq!(bound, n);
        }
        for token in [
            "-1",
            "+1",
            "1.0",
            ":bound",
            "0",
            "1001",
            "65536",
            "999999999999999999999999999999",
            "'1'",
            "true",
            "1e1",
            "(1)",
        ] {
            let sql = format!("RELATED_PATHS(l.product, supplier, {token})");
            let error = fragment(&sql).unwrap_err();
            assert_eq!(error.phase, "parse", "{sql}");
            assert!(error.source_span.is_some(), "{sql}");
        }
        let sql = "  RELATED_PATHS(l.product, supplier, 1001)";
        let error = fragment(sql).unwrap_err();
        assert_eq!(error.code, "WFT-LIMIT");
        assert_eq!(original(sql, &error.source_span.unwrap()), "1001");
    }

    #[test]
    fn qualified_first_and_single_second_names_are_required() {
        for sql in [
            "EXPAND_PATHS(product, supplier)",
            "EXPAND_PATHS(l.product.extra, supplier)",
            "EXPAND_PATHS(l.product, p.supplier)",
            "EXPAND_PATHS(:product, supplier)",
            "EXPAND_PATHS(l.product, :supplier)",
            "EXPAND_PATHS(l.product, 'supplier')",
            "EXPAND_PATHS(l.product, \"\")",
            "EXPAND_PATHS(l.select, supplier)",
        ] {
            assert!(fragment(sql).is_err(), "{sql}");
        }
        assert!(fragment(r#"EXPAND_PATHS(l."select", "from")"#).is_ok());
    }

    #[test]
    fn function_arity_aliases_and_trailing_syntax_refuse() {
        for sql in [
            "RELATED_PATHS(l.product, supplier)",
            "RELATED_PATHS(l.product, supplier, 1, 2)",
            "EXPAND_PATHS()",
            "EXPAND_PATHS(l.product)",
            "EXPAND_PATHS(l.product, supplier, 1)",
            "COUNT_DISTINCT_PATH_TARGETS()",
            "COUNT_DISTINCT_PATH_TARGETS(p, q)",
            "COUNT_DISTINCT_PATH_TARGETS(p.id)",
            "COUNT_DISTINCT_PATH_TARGETS(:p)",
            "COUNT_DISTINCT_PATH_TARGETS('p')",
            "COUNT_DISTINCT_PATH_TARGETS(\"\")",
            "COUNT_DISTINCT_PATH_TARGETS(1)",
            "COUNT_DISTINCT_PATH_TARGETS(select)",
            "COUNT_DISTINCT_PATH_TARGETS(p q)",
            "COUNT_DISTINCT_PATH_TARGETS(p",
            "RELATED_PATHS(l.product, supplier, 1) junk",
            "EXPAND_PATHS(l.product, supplier); SELECT p.id FROM products p",
            "COUNT_DISTINCT_PATH_TARGETS(p);;",
            "\"RELATED_PATHS\"(l.product, supplier, 1)",
        ] {
            assert!(fragment(sql).is_err(), "{sql}");
        }
    }

    #[test]
    fn fragments_leave_surrounding_query_tokens_to_the_existing_parser() {
        let sql = "RELATED_PATHS(l.product, supplier, 1), COUNT_DISTINCT_PATH_TARGETS(p) FROM";
        let mut parser = Parser::new(sql).unwrap();
        let first = parse(&mut parser).unwrap();
        assert_eq!(
            original(sql, &first.span),
            "RELATED_PATHS(l.product, supplier, 1)"
        );
        parser.symbol(',').unwrap();
        let second = parse(&mut parser).unwrap();
        assert_eq!(
            original(sql, &second.span),
            "COUNT_DISTINCT_PATH_TARGETS(p)"
        );
        assert!(parser.peek_word("from"));
        parser.word("from").unwrap();
        parser.finish().unwrap();
    }

    #[test]
    fn shared_lexer_limits_and_existing_frontend_refusals_remain() {
        assert_eq!(fragment(&" ".repeat(65537)).unwrap_err().code, "WFT-LIMIT");
        assert_eq!(fragment(&"x ".repeat(4097)).unwrap_err().code, "WFT-LIMIT");
        assert!(fragment("EXPAND_PATHS(l.product, supplier) -- comment").is_err());
        assert!(fragment("EXPAND_PATHS(l.product, \"bad\0name\")").is_err());
        for sql in [
            "SELECT RELATED_PATHS(l.product, supplier, 1) FROM lines l",
            "SELECT COUNT_DISTINCT_PATH_TARGETS(p) FROM lines l",
            "SELECT l.id FROM lines l CROSS JOIN EXPAND_PATHS(l.product, supplier) AS p",
        ] {
            assert!(crate::syntax::parse(sql).is_err(), "v0.1: {sql}");
            assert!(
                crate::application_syntax::parse(sql).is_err(),
                "v0.2: {sql}"
            );
            assert!(crate::arithmetic_query::parse(sql).is_err(), "v0.3: {sql}");
        }
    }
}
