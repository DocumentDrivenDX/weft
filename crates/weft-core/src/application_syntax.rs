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
    CountDistinct(Column),
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
    Boolean { value:bool },
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
fn bound(p: &mut Parser) -> Result<u16> { bound_selected(p,false) }
fn bound_selected(p: &mut Parser,zero:bool) -> Result<u16> {
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
        .filter(|n| if zero {(0..=1000).contains(n)}else{(1..=1000).contains(n)});
    n.ok_or_else(|| {
        Diagnostic::new(
            "WFT-LIMIT",
            "parse",
            if zero {"Application bound must be an integer from 0 through 1000"}else{"Application bound must be an integer from 1 through 1000"},
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
fn predicates(p: &mut Parser,extended:bool) -> Result<Vec<Predicate>> {
    let mut predicates = Vec::new();
    loop {
        let predicate = if extended && (p.peek_word("true")||p.peek_word("false")) {
            let value=p.peek_word("true");p.word(if value{"true"}else{"false"})?;Predicate::Boolean{value}
        } else if p.peek_word("has_related") {
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
    parse_selected(sql, false,false).map(|(query, _)| query)
}
/// Security candidate parser; historical application registrations retain ASC only.
pub(crate) fn parse_security(sql: &str) -> Result<(Query, Vec<bool>)> {
    parse_selected(sql, true,false)
}
pub(crate) fn parse_lineage_candidate(sql:&str)->Result<(Query,Vec<bool>)>{parse_selected(sql,true,true)}
fn parse_selected(sql: &str, descending: bool,extended:bool) -> Result<(Query, Vec<bool>)> {
    let mut p = Parser::new(sql)?;
    p.word("select")?;
    let mut outputs = Vec::new();
    loop {
        let output = if p.peek_word("count") {
            p.word("count")?;
            p.symbol('(')?;
            if extended&&p.peek_word("distinct") {p.word("distinct")?;let field=p.column()?;p.symbol(')')?;Output::CountDistinct(field)}else{p.symbol('*')?;p.symbol(')')?;Output::Count}
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
                Output::Field(Column { unqualified: false, alias, field, span })
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
        joins.push((right, predicates(&mut p,extended)?));
    }
    let predicates = if p.peek_word("where") {
        p.word("where")?;
        predicates(&mut p,extended)?
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
    let mut order_descending = Vec::new();
    if p.peek_word("order") {
        p.word("order")?;
        p.word("by")?;
        loop {
            order.push(p.column()?);
            let desc = descending && p.peek_word("desc");
            if desc { p.word("desc")?; } else if p.peek_word("asc") {
                p.word("asc")?;
            }
            order_descending.push(desc);
            if !p.peek_symbol(',') {
                break;
            }
            p.symbol(',')?;
        }
    }
    let limit = if p.peek_word("limit") {
        p.word("limit")?;
        Some(bound_selected(&mut p,extended)?)
    } else {
        None
    };
    p.finish_application()?;
    Ok((Query {
        outputs,
        source,
        predicates,
        joins,
        groups,
        order,
        limit,
    }, order_descending))
}

#[cfg(test)]
mod security_order_tests {
    #[test]
    fn descending_is_selected_only_for_security_and_preserves_each_direction() {
        let sql="SELECT r.id FROM Resource r ORDER BY r.id DESC,r.name ASC LIMIT 1";
        assert!(super::parse(sql).is_err());
        let (query,directions)=super::parse_security(sql).unwrap();
        assert_eq!(query.order.len(),2);
        assert_eq!(directions,vec![true,false]);
        assert_eq!(query.limit,Some(1));
        for suffix in ["DESC DESC","DESC NULLS LAST","DESC; SELECT r.id FROM Resource r"] {
            assert!(super::parse_security(&format!("SELECT r.id FROM Resource r ORDER BY r.id {suffix}")).is_err());
        }
    }
}
