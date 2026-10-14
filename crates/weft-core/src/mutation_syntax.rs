//! Bounded UPDATE syntax, separate from existing read interfaces.
use super::mutation::{fail, MResult};
use crate::{ir::Span, syntax::Name};
use serde_json::{json, Value};
#[derive(Clone, Debug)]
enum Token {
    Word(String),
    Name(String),
    Text(String),
    Number(String),
    Symbol(char),
}
#[derive(Clone, Debug)]
pub enum Input {
    Literal(Value),
    Marker(String),
}
#[derive(Clone, Debug)]
pub enum Predicate {
    Compare(Name, String, Input),
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
}
pub struct Update {
    pub namespace: Option<Name>,
    pub record: Name,
    pub assignments: Vec<(Name, Input)>,
    pub predicate: Predicate,
}
struct Parser {
    tokens: Vec<Token>,
    at: usize,
    nodes: usize,
}
fn syntax() -> super::mutation::MutationDiagnostic {
    fail("SYNTAX", "parse", "Unsupported UPDATE syntax")
}
fn lex(sql: &str) -> MResult<Vec<Token>> {
    if sql.len() > 65536 {
        return Err(fail("LIMIT", "limit", "SQL byte bound exceeded"));
    }
    let mut out = Vec::new();
    let mut at = 0;
    while at < sql.len() {
        let c = sql[at..].chars().next().unwrap();
        if c.is_ascii_whitespace() {
            at += 1;
            continue;
        }
        if sql[at..].starts_with("--") || sql[at..].starts_with("/*") {
            return Err(syntax());
        }
        let token = if c == '\'' || c == '"' {
            let quote = c;
            at += 1;
            let mut text = String::new();
            let mut closed = false;
            while at < sql.len() {
                let ch = sql[at..].chars().next().unwrap();
                at += ch.len_utf8();
                if ch == quote {
                    if sql[at..].starts_with(quote) {
                        text.push(quote);
                        at += 1;
                    } else {
                        closed = true;
                        break;
                    }
                } else {
                    text.push(ch);
                }
            }
            if !closed {
                return Err(syntax());
            }
            if quote == '\'' {
                Token::Text(text)
            } else {
                Token::Name(text)
            }
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = at;
            at += 1;
            while at < sql.len()
                && (sql.as_bytes()[at].is_ascii_alphanumeric() || sql.as_bytes()[at] == b'_')
            {
                at += 1;
            }
            Token::Word(sql[start..at].into())
        } else if c.is_ascii_digit() || c == '-' {
            let start = at;
            if c == '-' {
                at += 1;
                if at == sql.len() || !sql.as_bytes()[at].is_ascii_digit() {
                    return Err(syntax());
                }
            }
            let digits = at;
            while at < sql.len() && sql.as_bytes()[at].is_ascii_digit() {
                at += 1;
            }
            if at == digits || at - digits > 1 && sql.as_bytes()[digits] == b'0' {
                return Err(syntax());
            }
            Token::Number(sql[start..at].into())
        } else {
            at += c.len_utf8();
            if !matches!(c, '=' | '<' | '>' | ',' | '.' | ':' | '(' | ')' | ';') {
                return Err(syntax());
            }
            Token::Symbol(c)
        };
        out.push(token);
    }
    Ok(out)
}
impl Parser {
    fn peek_word(&self, v: &str) -> bool {
        matches!(self.tokens.get(self.at),Some(Token::Word(w)) if w.eq_ignore_ascii_case(v))
    }
    fn word(&mut self, v: &str) -> MResult<()> {
        if !self.peek_word(v) {
            return Err(syntax());
        }
        self.at += 1;
        Ok(())
    }
    fn peek(&self, c: char) -> bool {
        matches!(self.tokens.get(self.at),Some(Token::Symbol(v)) if *v==c)
    }
    fn symbol(&mut self, c: char) -> MResult<()> {
        if !self.peek(c) {
            return Err(syntax());
        }
        self.at += 1;
        Ok(())
    }
    fn name(&mut self) -> MResult<Name> {
        let token = self.tokens.get(self.at).cloned().ok_or_else(syntax)?;
        self.at += 1;
        let (value, quoted) = match token {
            Token::Word(v)
                if ![
                    "update", "set", "where", "and", "or", "true", "false", "null",
                ]
                .contains(&v.to_ascii_lowercase().as_str()) =>
            {
                (v.to_ascii_lowercase(), false)
            }
            Token::Name(v) if !v.is_empty() => (v, true),
            _ => return Err(syntax()),
        };
        Ok(Name {
            value,
            quoted,
            span: Span { start: 0, end: 0 },
        })
    }
    fn input(&mut self) -> MResult<Input> {
        if self.peek(':') {
            self.at += 1;
            let token = self.tokens.get(self.at).cloned().ok_or_else(syntax)?;
            self.at += 1;
            let Token::Word(marker) = token else {
                return Err(syntax());
            };
            return Ok(Input::Marker(marker));
        }
        let token = self.tokens.get(self.at).cloned().ok_or_else(syntax)?;
        self.at += 1;
        Ok(Input::Literal(match token {
            Token::Text(v) => json!({"string":v}),
            Token::Number(v) => json!({"integerToken":v}),
            Token::Word(v) if v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("false") => json!({"boolean":v.eq_ignore_ascii_case("true")}),
            _ => return Err(syntax()),
        }))
    }
    fn comparison(&mut self) -> MResult<Predicate> {
        let field = self.name()?;
        let op = if self.peek('=') {
            self.at += 1;
            "eq"
        } else if self.peek('<') {
            self.at += 1;
            if self.peek('>') {
                self.at += 1;
                "ne"
            } else if self.peek('=') {
                self.at += 1;
                "le"
            } else {
                "lt"
            }
        } else if self.peek('>') {
            self.at += 1;
            if self.peek('=') {
                self.at += 1;
                "ge"
            } else {
                "gt"
            }
        } else {
            return Err(syntax());
        };
        self.nodes += 1;
        if self.nodes > 256 {
            return Err(fail(
                "LIMIT",
                "limit",
                "Predicate comparison bound exceeded",
            ));
        }
        Ok(Predicate::Compare(field, op.into(), self.input()?))
    }
    // Iterative precedence/grouping avoids host-dependent recursion at the 128-group bound.
    fn predicate(&mut self) -> MResult<Predicate> {
        let mut values = Vec::new();
        let mut operators: Vec<char> = Vec::new();
        let mut groups = 0;
        let mut operand = true;
        loop {
            if operand {
                if self.peek('(') {
                    groups += 1;
                    if groups > 128 { return Err(fail("LIMIT", "limit", "Syntax grouping bound exceeded")); }
                    self.at += 1; operators.push('('); continue;
                }
                values.push(self.comparison()?); operand = false;
            } else if self.peek(')') {
                if groups == 0 { return Err(syntax()); }
                while operators.last() != Some(&'(') { reduce(&mut values, operators.pop().ok_or_else(syntax)?)?; }
                operators.pop(); groups -= 1; self.at += 1;
            } else if self.peek_word("and") || self.peek_word("or") {
                let op = if self.peek_word("and") {'&'} else {'|'};
                while operators.last().is_some_and(|last| *last != '(' && (*last == '&' || op == '|')) {
                    reduce(&mut values, operators.pop().unwrap())?;
                }
                operators.push(op); self.at += 1; operand = true;
            } else { break; }
        }
        if operand || groups != 0 { return Err(syntax()); }
        while let Some(op) = operators.pop() { reduce(&mut values, op)?; }
        if values.len() != 1 { return Err(syntax()); }
        Ok(values.pop().unwrap())
    }

}
pub fn parse(sql: &str) -> MResult<Update> {
    let mut p = Parser {
        tokens: lex(sql)?,
        at: 0,
        nodes: 0,
    };
    p.word("update")?;
    let first = p.name()?;
    let (namespace, record) = if p.peek('.') {
        p.at += 1;
        (Some(first), p.name()?)
    } else {
        (None, first)
    };
    p.word("set")?;
    let mut assignments = Vec::new();
    loop {
        let field = p.name()?;
        p.symbol('=')?;
        assignments.push((field, p.input()?));
        if assignments.len() > 64 {
            return Err(fail("LIMIT", "limit", "Assignment bound exceeded"));
        }
        if !p.peek(',') {
            break;
        }
        p.at += 1;
    }
    p.word("where")?;
    let predicate = p.predicate()?;
    if p.peek(';') {
        p.at += 1;
    }
    if p.at != p.tokens.len() {
        return Err(syntax());
    }
    Ok(Update {
        namespace,
        record,
        assignments,
        predicate,
    })
}

fn reduce(values: &mut Vec<Predicate>, op: char) -> MResult<()> {
    let right = values.pop().ok_or_else(syntax)?;
    let left = values.pop().ok_or_else(syntax)?;
    let mut children = Vec::new();
    for value in [left, right] {
        match (op, value) {
            ('&', Predicate::And(same)) | ('|', Predicate::Or(same)) => children.extend(same),
            (_, other) => children.push(other),
        }
    }
    values.push(match op {'&' => Predicate::And(children), '|' => Predicate::Or(children), _ => return Err(syntax())});
    Ok(())
}
