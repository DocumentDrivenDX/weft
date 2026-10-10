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
    StringIn { column: Column, literals: Vec<Literal> },
    NullTest { column: Column, negated: bool },
    ExtendedCompare { column: Column, value: Value, operator: crate::arithmetic_plan::ComparisonOperator },
    ArithmeticCompareExtended { left: Expression, right: Expression, operator: crate::arithmetic_plan::ComparisonOperator },
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
pub struct Having {
    pub argument: Option<Column>,
    pub count_star_span: Option<crate::ir::Span>,
    pub threshold: Literal,
}
#[derive(Debug, Clone)]
pub struct Join {
    pub right: Source,
    pub on: Vec<Predicate>,
    pub left: bool,
}
#[derive(Debug, Clone)]
pub struct Query {
    pub distinct: bool,
    pub outputs: Vec<Projection>,
    pub source: Source,
    pub predicates: Vec<Predicate>,
    pub joins: Vec<Join>,
    pub groups: Vec<Column>,
    pub having: Vec<Having>,
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
#[derive(Clone, Copy)]
enum Operator { Equal, Greater, Extended(crate::arithmetic_plan::ComparisonOperator) }
fn operator(p: &mut Parser) -> Result<Operator> {
    use crate::arithmetic_plan::ComparisonOperator as O;
    let span = p.span();
    let first = if p.peek_symbol('<') { '<' } else if p.peek_symbol('>') { '>' } else { '=' };
    p.symbol(first)?;
    if (first == '<' && (p.peek_symbol('=') || p.peek_symbol('>'))) || (first == '>' && p.peek_symbol('=')) {
        if p.span().start != span.end { return Err(fail(p, "Comparison operator symbols must be contiguous")); }
        let second = if p.peek_symbol('=') { '=' } else { '>' };
        p.symbol(second)?;
        return Ok(Operator::Extended(match (first,second) { ('<','=') => O::LessEqual, ('<','>') => O::NotEqual, _ => O::GreaterEqual }));
    }
    Ok(match first { '<' => Operator::Extended(O::Less), '>' => Operator::Greater, _ => Operator::Equal })
}
fn predicates(p: &mut Parser, budget: &mut Budget) -> Result<Vec<Predicate>> {
    let mut predicates = Vec::new();
    loop {
        let predicate = if let Some(predicate) = string_in_predicate(p, budget)? {
            predicate
        } else if let Some(predicate) = null_predicate(p)? {
            predicate
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
            let op = operator(p)?;
            let greater = matches!(op, Operator::Greater);
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
            match op {
                Operator::Extended(operator) => Predicate::ExtendedCompare { column: columns.into_iter().next().unwrap(), value: values.into_iter().next().unwrap(), operator },
                _ => Predicate::Compare { columns, values, greater },
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
fn string_in_predicate(p: &mut Parser, budget: &mut Budget) -> Result<Option<Predicate>> {
    let checkpoint=p.clone();
    let column=match p.column03(){Ok(c)=>c,Err(_)=>{*p=checkpoint;return Ok(None)}};
    if !p.peek_word("in"){*p=checkpoint;return Ok(None)}
    p.word("in")?;p.symbol('(')?;
    let mut literals=Vec::new();
    loop {
        budget.reserve(p)?;
        let literal=p.literal()?;
        if !matches!(literal.kind,LiteralKind::String){return Err(fail(p,"IN accepts exact String literals only"));}
        literals.push(literal);
        if !p.peek_symbol(','){break}p.symbol(',')?;
    }
    p.symbol(')')?;
    Ok(Some(Predicate::StringIn{column,literals}))
}
fn null_predicate(p: &mut Parser) -> Result<Option<Predicate>> {
    let checkpoint=p.clone();
    let column=match p.column03() {Ok(c)=>c,Err(_)=>{*p=checkpoint;return Ok(None)}};
    if !p.peek_word("is") {*p=checkpoint;return Ok(None)}
    p.word("is")?;
    let negated=p.peek_word("not");if negated {p.word("not")?;}
    p.word("null")?;
    Ok(Some(Predicate::NullTest{column,negated}))
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
        let op = operator(p)?;
        let greater = matches!(op, Operator::Greater);
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
                return Ok(match op {
                    Operator::Extended(operator) => Predicate::ExtendedCompare { column: column.clone(), value, operator },
                    _ => Predicate::Compare { columns: vec![column.clone()], values: vec![value], greater },
                });
            }
        }
        Ok(match op {
            Operator::Extended(operator) => Predicate::ArithmeticCompareExtended { left, right, operator },
            _ => Predicate::ArithmeticCompare { left, right, greater },
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
// Shared query segments keep each private dialect on the same legacy grammar.
pub(crate) fn projection(p: &mut Parser, budget: &mut Budget) -> Result<Projection> {
    let output = if p.peek_word("count") {
        p.word("count")?;
        p.symbol('(')?;
        let output=if p.peek_word("distinct") {p.word("distinct")?;Output::CountDistinct(p.column03()?)} else {p.symbol('*')?;Output::Count};
        p.symbol(')')?;
        output
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
        let bound = bound(p)?;
        p.symbol(')')?;
        Output::Related {
            relationship,
            bound,
        }
    } else if let Some(expression) = arithmetic_output(p, budget)? {
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
            p,
            "Computed arithmetic projection requires an explicit alias",
        ));
    }
    if matches!(output, Output::Entity(_)) && alias.is_some() {
        return Err(fail(
            p,
            "Whole-entity projection cannot have a single alias",
        ));
    }
    Ok(Projection { output, alias })
}

pub(crate) fn joins(p: &mut Parser, budget: &mut Budget) -> Result<Vec<Join>> {
    let mut joins = Vec::new();
    while p.peek_word("inner") || p.peek_word("join") || p.peek_word("left") {
        if joins.len() >= 16 {
            return Err(
                Diagnostic::new("WFT-LIMIT", "parse", "Join count exceeds limit").at(&p.span()),
            );
        }
        let left = p.peek_word("left");
        if left { p.word("left")?; }
        else if p.peek_word("inner") { p.word("inner")?; }
        p.word("join")?;
        let right = p.source()?;
        p.word("on")?;
        joins.push(Join { right, on: predicates(p, budget)?, left });
    }
    Ok(joins)
}

#[derive(Debug, Clone)]
pub(crate) struct Tail {
    pub predicates: Vec<Predicate>,
    pub groups: Vec<Column>,
    pub having: Vec<Having>,
    pub order: Vec<Column>,
    pub limit: Option<u16>,
}

pub(crate) fn tail(p: &mut Parser, budget: &mut Budget) -> Result<Tail> {
    tail_version(p, budget, false)
}
pub(crate) fn tail_version(p: &mut Parser, budget: &mut Budget, count_star: bool) -> Result<Tail> {
    let predicates = if p.peek_word("where") {
        p.word("where")?;
        predicates(p, budget)?
    } else {
        vec![]
    };
    let groups = if p.peek_word("group") {
        p.word("group")?;
        p.word("by")?;
        columns(p)?
    } else {
        vec![]
    };
    let mut having = Vec::new();
    if p.peek_word("having") {
        budget.reserve(p)?;
        p.word("having")?;
        p.word("count")?;
        p.symbol('(')?;
        let count_star_span = if count_star && p.peek_symbol('*') {
            Some(p.span())
        } else {
            None
        };
        let argument = if count_star && p.peek_symbol('*') {
            p.symbol('*')?;
            None
        } else {
            p.word("distinct")?;
            Some(p.column03()?)
        };
        p.symbol(')')?;
        p.symbol('>')?;
        budget.reserve(p)?;
        let threshold = p.literal()?;
        if threshold.value.len() > crate::arithmetic_syntax::MAX_LITERAL_BYTES {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "parse",
                "Numeric literal exceeds expression limit",
            )
            .at(&threshold.span));
        }
        if !matches!(threshold.kind, LiteralKind::Number)
            || threshold.value.is_empty()
            || !threshold.value.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(fail(
                p,
                "HAVING admits COUNT DISTINCT > a nonnegative exact Integer literal only",
            ));
        }
        having.push(Having {
            argument,
            count_star_span,
            threshold,
        });
    }
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
        Some(bound(p)?)
    } else {
        None
    };
    p.finish_application()?;
    Ok(Tail {
        predicates,
        groups,
        having,
        order,
        limit,
    })
}

pub fn parse(sql: &str) -> Result<Query> {
    let mut p = Parser::new(sql)?;
    let mut budget = Budget::new();
    p.word("select")?;
    let distinct = if p.peek_word("distinct") { p.word("distinct")?; true } else { false };
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
    let joins = joins(&mut p, &mut budget)?;
    let Tail { predicates, groups, having, order, limit } = tail(&mut p, &mut budget)?;
    Ok(Query {
        distinct,
        outputs,
        source,
        predicates,
        joins,
        groups,
        having,
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
            q.joins[0].on[0],
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
        assert!(matches!(q.joins[0].on[0], Predicate::Compare { .. }));
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
    #[test]
    fn new_comparison_tokens_are_closed_and_old_tuples_remain() {
        for op in ["<", "<=", ">=", "<>"] {
            assert!(matches!(parse(&format!("SELECT c.id FROM Customer c WHERE c.id{op}1")).unwrap().predicates[0], Predicate::ExtendedCompare { .. }));
            assert!(matches!(parse(&format!("SELECT c.id FROM Customer c WHERE c.id+1{op}2")).unwrap().predicates[0], Predicate::ArithmeticCompareExtended { .. }));
            assert!(crate::application_syntax::parse(&format!("SELECT c.id FROM Customer c WHERE c.id{op}1")).is_err());
            assert!(crate::syntax::parse(&format!("SELECT c.id FROM Customer c WHERE c.id{op}1")).is_err());
            assert!(parse(&format!("SELECT c.id FROM Customer c WHERE (c.id,c.id){op}(1,2)")).is_err());
        }
        for op in ["!=", "=>", "=<", "< =", "> =", "< >", "<<", ">>"] {
            assert!(parse(&format!("SELECT c.id FROM Customer c WHERE c.id{op}1")).is_err(), "{op}");
        }
        assert!(parse("SELECT c.id FROM Customer c WHERE (c.id,c.id)>(1,2)").is_ok());
    }

}
