//! Additive 0.3 query syntax foundation; public compile transport remains unchanged.
use crate::arithmetic_syntax::{self, Budget, Expression, Kind};
use crate::{
    error::{Diagnostic, Result},
    syntax::{Column, Literal, LiteralKind, Name, Parser, Source},
};
#[derive(Debug, Clone)]
pub enum Value {
    Literal(Literal),
    Parameter(Name),
    Field(Column),
}
#[derive(Debug, Clone)]
pub enum Output {
    Arithmetic(Expression),
    Field(Column),
    Entity(Name),
    Count,
    Sum(Column),
    Related { relationship: Column, bound: u16 },
}
#[derive(Debug, Clone)]
pub struct Projection {
    pub output: Output,
    pub alias: Option<Name>,
}
#[derive(Debug, Clone)]
pub enum Predicate {
    ArithmeticCompare {
        left: Expression,
        right: Expression,
        greater: bool,
    },
    Compare {
        columns: Vec<Column>,
        values: Vec<Value>,
        greater: bool,
    },
    HasRelated {
        relationship: Column,
        key: Vec<Value>,
    },
}
#[derive(Debug, Clone)]
pub struct Query {
    pub outputs: Vec<Projection>,
    pub source: Source,
    pub predicates: Vec<Predicate>,
    pub joins: Vec<(Source, Vec<Predicate>)>,
    pub groups: Vec<Column>,
    pub order: Vec<Column>,
    pub limit: Option<u16>,
}
fn fail(p: &Parser, message: &str) -> Diagnostic {
    Diagnostic::new("WFT-UNSUPPORTED", "parse", message).at(&p.span())
}
fn value(p: &mut Parser) -> Result<Value> {
    if p.peek_symbol(':') {
        p.symbol(':')?;
        Ok(Value::Parameter(p.name()?))
    } else if p.peek_identifier() {
        Ok(Value::Field(p.column03()?))
    } else {
        Ok(Value::Literal(p.literal()?))
    }
}
fn bound(p: &mut Parser) -> Result<u16> {
    let literal = p.literal()?;
    if !matches!(literal.kind, LiteralKind::Number) {
        return Err(Diagnostic::new(
            "WFT-LIMIT",
            "parse",
            "Application bound must be an integer literal",
        )
        .at(&literal.span));
    }
    let n = literal
        .value
        .parse::<u16>()
        .ok()
        .filter(|n| (1..=1000).contains(n));
    n.ok_or_else(|| {
        Diagnostic::new(
            "WFT-LIMIT",
            "parse",
            "Application bound must be an integer from 1 through 1000",
        )
        .at(&literal.span)
    })
}
fn columns(p: &mut Parser) -> Result<Vec<Column>> {
    let mut values = vec![p.column03()?];
    while p.peek_symbol(',') {
        p.symbol(',')?;
        values.push(p.column03()?);
    }
    Ok(values)
}
fn predicates(p: &mut Parser, budget: &mut Budget) -> Result<Vec<Predicate>> {
    let mut predicates = Vec::new();
    loop {
        let predicate = if p.peek_word("has_related") {
            p.word("has_related")?;
            p.symbol('(')?;
            let relationship = p.column()?;
            p.symbol(',')?;
            p.word("key")?;
            p.symbol('(')?;
            let mut key = vec![value(p)?];
            while p.peek_symbol(',') {
                p.symbol(',')?;
                key.push(value(p)?);
            }
            p.symbol(')')?;
            p.symbol(')')?;
            Predicate::HasRelated { relationship, key }
        } else if let Some(predicate) = arithmetic_predicate(p, budget)? {
            predicate
        } else {
            let tuple = p.peek_symbol('(');
            let columns = if tuple {
                p.symbol('(')?;
                let c = columns(p)?;
                p.symbol(')')?;
                c
            } else {
                vec![p.column03()?]
            };
            let greater = p.peek_symbol('>');
            p.symbol(if greater { '>' } else { '=' })?;
            if tuple && !greater {
                return Err(fail(p, "Tuple equality is outside the application subset"));
            }
            let values = if tuple {
                p.symbol('(')?;
                let mut v = vec![value(p)?];
                while p.peek_symbol(',') {
                    p.symbol(',')?;
                    v.push(value(p)?);
                }
                p.symbol(')')?;
                v
            } else {
                vec![value(p)?]
            };
            if columns.len() != values.len() {
                return Err(fail(p, "Cursor tuple arity mismatch"));
            }
            Predicate::Compare {
                columns,
                values,
                greater,
            }
        };
        predicates.push(predicate);
        if !p.peek_word("and") {
            break;
        }
        p.word("and")?;
    }
    Ok(predicates)
}
fn arithmetic_output(p: &mut Parser, budget: &mut Budget) -> Result<Option<Expression>> {
    let checkpoint = p.clone();
    match arithmetic_syntax::parse(p, budget) {
        Ok(e) => Ok(Some(e)),
        Err(e) if e.code == "WFT-LIMIT" => Err(e),
        Err(_) => {
            *p = checkpoint;
            Ok(None)
        }
    }
}
fn arithmetic_predicate(p: &mut Parser, budget: &mut Budget) -> Result<Option<Predicate>> {
    let checkpoint = p.clone();
    let result: Result<Predicate> = (|| {
        let left = arithmetic_syntax::parse(p, budget)?;
        let greater = p.peek_symbol('>');
        p.symbol(if greater { '>' } else { '=' })?;
        let right = arithmetic_syntax::parse(p, budget)?;
        // Operator-free comparisons retain the established application resolver.
        if let Kind::Field(column) = &left.kind {
            let value = match &right.kind {
                Kind::Field(column) => Some(Value::Field(column.clone())),
                Kind::Parameter(name) => Some(Value::Parameter(name.clone())),
                Kind::Literal(literal) => Some(Value::Literal(literal.clone())),
                _ => None,
            };
            if let Some(value) = value {
                return Ok(Predicate::Compare {
                    columns: vec![column.clone()],
                    values: vec![value],
                    greater,
                });
            }
        }
        Ok(Predicate::ArithmeticCompare {
            left,
            right,
            greater,
        })
    })();
    match result {
        Ok(e) => Ok(Some(e)),
        Err(e) if e.code == "WFT-LIMIT" => Err(e),
        Err(_) => {
            *p = checkpoint;
            Ok(None)
        }
    }
}
pub fn parse(sql: &str) -> Result<Query> {
    let mut p = Parser::new(sql)?;
    let mut budget = Budget::new();
    p.word("select")?;
    let mut outputs = Vec::new();
    loop {
        let output = if p.peek_word("count") {
            p.word("count")?;
            p.symbol('(')?;
            p.symbol('*')?;
            p.symbol(')')?;
            Output::Count
        } else if p.peek_word("sum") {
            p.word("sum")?;
            p.symbol('(')?;
            let c = p.column03()?;
            p.symbol(')')?;
            Output::Sum(c)
        } else if p.peek_word("related_keys") {
            p.word("related_keys")?;
            p.symbol('(')?;
            let relationship = p.column()?;
            p.symbol(',')?;
            let bound = bound(&mut p)?;
            p.symbol(')')?;
            Output::Related {
                relationship,
                bound,
            }
        } else if let Some(expression) = arithmetic_output(&mut p, &mut budget)? {
            match expression.kind {
                Kind::Field(column) => Output::Field(column),
                _ => Output::Arithmetic(expression),
            }
        } else {
            let alias = p.name()?;
            p.symbol('.')?;
            if p.peek_symbol('*') {
                p.symbol('*')?;
                Output::Entity(alias)
            } else {
                let field = p.name()?;
                let span = crate::ir::Span {
                    start: alias.span.start,
                    end: field.span.end,
                };
                Output::Field(Column { unqualified: false, alias, field, span })
            }
        };
        let alias = if p.peek_word("as") {
            p.word("as")?;
            Some(p.name()?)
        } else {
            None
        };
        if matches!(output, Output::Arithmetic(_)) && alias.is_none() {
            return Err(fail(
                &p,
                "Computed arithmetic projection requires an explicit alias",
            ));
        }
        if matches!(output, Output::Entity(_)) && alias.is_some() {
            return Err(fail(
                &p,
                "Whole-entity projection cannot have a single alias",
            ));
        }
        outputs.push(Projection { output, alias });
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
    let mut joins = Vec::new();
    while p.peek_word("inner") || p.peek_word("join") {
        if joins.len() >= 16 {
            return Err(
                Diagnostic::new("WFT-LIMIT", "parse", "Join count exceeds limit").at(&p.span()),
            );
        }
        if p.peek_word("inner") {
            p.word("inner")?;
        }
        p.word("join")?;
        let right = p.source()?;
        p.word("on")?;
        joins.push((right, predicates(&mut p, &mut budget)?));
    }
    let predicates = if p.peek_word("where") {
        p.word("where")?;
        predicates(&mut p, &mut budget)?
    } else {
        vec![]
    };
    let groups = if p.peek_word("group") {
        p.word("group")?;
        p.word("by")?;
        columns(&mut p)?
    } else {
        vec![]
    };
    let mut order = Vec::new();
    if p.peek_word("order") {
        p.word("order")?;
        p.word("by")?;
        loop {
            order.push(p.column03()?);
            if p.peek_word("asc") {
                p.word("asc")?;
            }
            if !p.peek_symbol(',') {
                break;
            }
            p.symbol(',')?;
        }
    }
    let limit = if p.peek_word("limit") {
        p.word("limit")?;
        Some(bound(&mut p)?)
    } else {
        None
    };
    p.finish_application()?;
    Ok(Query {
        outputs,
        source,
        predicates,
        joins,
        groups,
        order,
        limit,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn commerce_arithmetic_in_projection_filter_and_join() {
        let q = parse("SELECT l.quantity-f.quantity+r.quantity AS remaining FROM lines l JOIN fulfillments f ON l.id=f.line_id JOIN returns r ON l.id=r.line_id WHERE l.quantity-f.quantity+r.quantity > 0").unwrap();
        assert!(matches!(q.outputs[0].output, Output::Arithmetic(_)));
        assert!(matches!(
            q.predicates[0],
            Predicate::ArithmeticCompare { .. }
        ));
        let q = parse("SELECT r.id FROM refunds r JOIN returns t ON r.amount=t.quantity*12.50")
            .unwrap();
        assert!(matches!(
            q.joins[0].1[0],
            Predicate::ArithmeticCompare { .. }
        ));
    }
    #[test]
    fn one_budget_for_entire_query_and_old_profile_isolation() {
        let projections = vec!["l.quantity + 1 AS n"; 86].join(",");
        assert_eq!(
            parse(&format!("SELECT {projections} FROM lines l"))
                .unwrap_err()
                .code,
            "WFT-LIMIT"
        );
        let sql = "SELECT l.quantity + 1 AS n FROM lines l";
        assert!(parse(sql).is_ok());
        assert!(crate::application_syntax::parse(sql).is_err());
    }
    #[test]
    fn operator_free_comparisons_preserve_existing_semantics() {
        let q = parse("SELECT l.id FROM lines l JOIN products p ON l.product_id=p.id WHERE l.quantity=:quantity").unwrap();
        assert!(matches!(q.joins[0].1[0], Predicate::Compare { .. }));
        assert!(matches!(q.predicates[0], Predicate::Compare { .. }));
        let q = parse("SELECT l.id FROM lines l WHERE l.quantity+1=:quantity").unwrap();
        assert!(matches!(
            q.predicates[0],
            Predicate::ArithmeticCompare { .. }
        ));
    }
    #[test]
    fn aggregate_and_order_positions_remain_field_only() {
        for sql in [
            "SELECT l.quantity+1 FROM lines l",
            "SELECT SUM(l.quantity+1) AS n FROM lines l",
            "SELECT l.id FROM lines l GROUP BY l.id+1",
            "SELECT l.id FROM lines l ORDER BY l.id+1",
        ] {
            assert!(parse(sql).is_err(), "{sql}");
        }
        assert!(parse("SELECT SUM(l.quantity) AS n FROM lines l").is_ok());
        assert!(parse("SELECT l.* FROM lines l").is_ok());
        assert!(parse("SELECT l.id FROM lines l WHERE l.name='original'").is_ok());
        assert!(parse("SELECT l.id FROM lines l WHERE (l.id,l.quantity)>(:id,:quantity)").is_ok());
    }
}
