//! Explicit 0.2 application-read syntax. Model validation is a separate phase.
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
        Ok(Value::Field(p.column()?))
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
    let mut values = vec![p.column()?];
    while p.peek_symbol(',') {
        p.symbol(',')?;
        values.push(p.column()?);
    }
    Ok(values)
}
fn predicates(p: &mut Parser) -> Result<Vec<Predicate>> {
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
        } else {
            let tuple = p.peek_symbol('(');
            let columns = if tuple {
                p.symbol('(')?;
                let c = columns(p)?;
                p.symbol(')')?;
                c
            } else {
                vec![p.column()?]
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
pub fn parse(sql: &str) -> Result<Query> {
    let mut p = Parser::new(sql)?;
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
            let c = p.column()?;
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
                Output::Field(Column { alias, field, span })
            }
        };
        let alias = if p.peek_word("as") {
            p.word("as")?;
            Some(p.name()?)
        } else {
            None
        };
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
        joins.push((right, predicates(&mut p)?));
    }
    let predicates = if p.peek_word("where") {
        p.word("where")?;
        predicates(&mut p)?
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
            order.push(p.column()?);
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
