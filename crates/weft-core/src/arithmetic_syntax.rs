//! Bounded scalar expression syntax for the explicit 0.3 profile.
//! This module is not yet connected to public compile transport.
use crate::{
    error::{Diagnostic, Result},
    ir::Span,
    syntax::{Column, Literal, LiteralKind, Name, Parser},
};
pub(crate) const MAX_NODES: usize = 256;
pub(crate) const MAX_DEPTH: usize = 32;
pub(crate) const MAX_LITERAL_BYTES: usize = 1024;
#[derive(Debug, Clone)]
pub(crate) enum Kind {
    Field(Column),
    Literal(Literal),
    Parameter(Name),
    Negate(Box<Expression>),
    Binary {
        operator: char,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}
#[derive(Debug, Clone)]
pub(crate) struct Expression {
    pub kind: Kind,
    pub span: Span,
    pub depth: usize,
}
pub(crate) struct Budget {
    nodes: usize,
}
impl Budget {
    pub(crate) fn new() -> Self {
        Self { nodes: 0 }
    }
    fn reserve(&mut self, p: &Parser) -> Result<()> {
        if self.nodes >= MAX_NODES {
            return Err(limit(p, "Expression node limit exceeded"));
        }
        self.nodes += 1;
        Ok(())
    }
}
fn limit(p: &Parser, message: &str) -> Diagnostic {
    Diagnostic::new("WFT-LIMIT", "parse", message).at(&p.span())
}
fn descend(p: &Parser, depth: usize) -> Result<()> {
    if depth > MAX_DEPTH {
        return Err(limit(p, "Expression nesting limit exceeded"));
    }
    Ok(())
}
fn node(p: &Parser, kind: Kind, span: Span, depth: usize) -> Result<Expression> {
    descend(p, depth)?;
    Ok(Expression { kind, span, depth })
}
fn atom(p: &mut Parser, budget: &mut Budget, depth: usize) -> Result<Expression> {
    descend(p, depth)?;
    let start = p.span().start;
    if p.peek_symbol('-') {
        budget.reserve(p)?;
        p.symbol('-')?;
        let child = atom(p, budget, depth + 1)?;
        let span = Span {
            start,
            end: child.span.end,
        };
        let height = child.depth + 1;
        descend(p, height)?;
        return node(p, Kind::Negate(Box::new(child)), span, height);
    }
    if p.peek_symbol('(') {
        p.symbol('(')?;
        let mut child = sum(p, budget, depth + 1)?;
        let end = p.span().end;
        p.symbol(')')?;
        child.span = Span { start, end };
        return Ok(child);
    }
    budget.reserve(p)?;
    let (kind, span) = if p.peek_symbol(':') {
        p.symbol(':')?;
        let name = p.name()?;
        let span = Span {
            start,
            end: name.span.end,
        };
        (Kind::Parameter(name), span)
    } else if p.peek_identifier() {
        let column = p.column()?;
        let span = column.span.clone();
        (Kind::Field(column), span)
    } else {
        let literal = p.literal()?;
        if !matches!(literal.kind, LiteralKind::Number) {
            return Err(Diagnostic::new(
                "WFT-UNSUPPORTED",
                "parse",
                "Arithmetic requires a numeric literal",
            )
            .at(&literal.span));
        }
        if literal.value.len() > MAX_LITERAL_BYTES {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "parse",
                "Numeric literal exceeds expression limit",
            )
            .at(&literal.span));
        }
        let span = literal.span.clone();
        (Kind::Literal(literal), span)
    };
    node(p, kind, span, 1)
}
fn binary(p: &Parser, operator: char, left: Expression, right: Expression) -> Result<Expression> {
    let span = Span {
        start: left.span.start,
        end: right.span.end,
    };
    let depth = left.depth.max(right.depth) + 1;
    descend(p, depth)?;
    node(
        p,
        Kind::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        },
        span,
        depth,
    )
}
fn product(p: &mut Parser, budget: &mut Budget, depth: usize) -> Result<Expression> {
    let mut left = atom(p, budget, depth)?;
    while p.peek_symbol('*') {
        budget.reserve(p)?;
        p.symbol('*')?;
        let right = atom(p, budget, depth)?;
        left = binary(p, '*', left, right)?;
    }
    Ok(left)
}
fn sum(p: &mut Parser, budget: &mut Budget, depth: usize) -> Result<Expression> {
    let mut left = product(p, budget, depth)?;
    while p.peek_symbol('+') || p.peek_symbol('-') {
        let op = if p.peek_symbol('+') { '+' } else { '-' };
        budget.reserve(p)?;
        p.symbol(op)?;
        let right = product(p, budget, depth)?;
        left = binary(p, op, left, right)?;
    }
    Ok(left)
}
pub(crate) fn parse(p: &mut Parser, budget: &mut Budget) -> Result<Expression> {
    sum(p, budget, 1)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn expression(raw: &str) -> Result<Expression> {
        let mut p = Parser::new(raw)?;
        let e = parse(&mut p, &mut Budget::new())?;
        p.finish()?;
        Ok(e)
    }
    fn shape(e: &Expression) -> String {
        match &e.kind {
            Kind::Field(c) => format!("{}.{}", c.alias.value, c.field.value),
            Kind::Literal(l) => l.value.clone(),
            Kind::Parameter(n) => format!(":{}", n.value),
            Kind::Negate(c) => format!("(-{})", shape(c)),
            Kind::Binary {
                operator,
                left,
                right,
            } => format!("({}{}{})", shape(left), operator, shape(right)),
        }
    }
    #[test]
    fn precedence_association_tokens_and_spans() {
        let e = expression("-l.quantity * 12.50 - :used + r.quantity").unwrap();
        assert_eq!(shape(&e), "((((-l.quantity)*12.50)-:used)+r.quantity)");
        assert_eq!((e.span.start, e.span.end), (0, 40));
        assert_eq!(
            shape(&expression("l.quantity - (f.quantity + r.quantity)").unwrap()),
            "(l.quantity-(f.quantity+r.quantity))"
        );
    }
    #[test]
    fn nesting_and_tree_depth_boundaries() {
        let bounded = format!(
            "{}1{}",
            "(".repeat(MAX_DEPTH - 1),
            ")".repeat(MAX_DEPTH - 1)
        );
        assert!(expression(&bounded).is_ok());
        let excessive = format!("({bounded})");
        let e = expression(&excessive).unwrap_err();
        assert_eq!(e.code, "WFT-LIMIT");
        assert!(e.source_span.is_some());
        assert!(expression(&format!("{}1", "- ".repeat(MAX_DEPTH - 1))).is_ok());
        assert_eq!(
            expression(&format!("{}1", "- ".repeat(MAX_DEPTH)))
                .unwrap_err()
                .code,
            "WFT-LIMIT"
        );
        assert_eq!(
            expression(&vec!["1"; MAX_DEPTH + 1].join("+"))
                .unwrap_err()
                .code,
            "WFT-LIMIT"
        );
    }
    #[test]
    fn common_budget_and_literal_boundaries() {
        fn balanced(n: usize) -> String {
            if n == 1 {
                "1".into()
            } else {
                format!("({}+{})", balanced(n / 2), balanced(n - n / 2))
            }
        }
        assert!(expression(&balanced(128)).is_ok());
        assert_eq!(expression(&balanced(129)).unwrap_err().code, "WFT-LIMIT");
        let mut budget = Budget::new();
        for _ in 0..MAX_NODES {
            parse(&mut Parser::new("1").unwrap(), &mut budget).unwrap();
        }
        assert_eq!(
            parse(&mut Parser::new("1").unwrap(), &mut budget)
                .unwrap_err()
                .code,
            "WFT-LIMIT"
        );
        assert!(expression(&"1".repeat(MAX_LITERAL_BYTES)).is_ok());
        assert_eq!(
            expression(&"1".repeat(MAX_LITERAL_BYTES + 1))
                .unwrap_err()
                .code,
            "WFT-LIMIT"
        );
    }
    #[test]
    fn unsupported_constructs_and_old_profile_isolation() {
        for raw in [
            "'1'",
            "true",
            "1/2",
            "sum(l.quantity)",
            "l.quantity;1",
            "--1",
        ] {
            assert!(expression(raw).is_err(), "{raw}");
        }
        assert!(
            crate::application_syntax::parse("SELECT l.quantity + 1 AS n FROM lines l").is_err()
        );
    }
}
