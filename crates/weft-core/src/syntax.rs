use crate::{
    error::{Diagnostic, Result},
    ir::Span,
};
#[derive(Debug, Clone)]
pub struct Name {
    pub value: String,
    pub quoted: bool,
    pub span: Span,
}
impl Name {
    pub fn matches(&self, value: &str) -> bool {
        if self.quoted {
            self.value == value
        } else {
            self.value == value.to_ascii_lowercase()
        }
    }
    pub fn key(&self) -> &str {
        &self.value
    }
}
#[derive(Debug, Clone)]
pub struct Column {
    pub alias: Name,
    pub field: Name,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub enum LiteralKind {
    Number,
    String,
    Boolean,
}
#[derive(Debug, Clone)]
pub struct Literal {
    pub value: String,
    pub kind: LiteralKind,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub enum Operand {
    Column(Column),
    Literal(Literal),
}
#[derive(Debug, Clone)]
pub struct Predicate {
    pub left: Column,
    pub right: Operand,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct Projection {
    pub column: Column,
    pub sum: bool,
    pub span: Span,
    pub alias: Option<Name>,
}
#[derive(Debug, Clone)]
pub struct Source {
    pub namespace: Option<Name>,
    pub name: Name,
    pub alias: Name,
}
#[derive(Debug, Clone)]
pub struct Query {
    pub outputs: Vec<Projection>,
    pub source: Source,
    pub joins: Vec<(Source, Vec<Predicate>)>,
    pub predicates: Vec<Predicate>,
    pub groups: Vec<Column>,
}
#[derive(Debug, Clone)]
enum Kind {
    Word(String),
    Quoted(String),
    String(String),
    Number(String),
    Symbol(char),
}
#[derive(Debug, Clone)]
struct Token {
    kind: Kind,
    span: Span,
}
fn error(code: &str, message: &str, span: &Span) -> Diagnostic {
    Diagnostic::new(code, "parse", message).at(span)
}
fn keyword(value: &str) -> bool {
    [
        "select", "from", "as", "where", "and", "or", "group", "by", "sum", "inner", "join", "on",
        "distinct", "order", "limit", "union", "left", "right", "full", "cross", "having", "null",
        "true", "false", "in", "is", "outer", "offset", "fetch",
    ]
    .contains(&value)
}
fn lex(sql: &str) -> Result<Vec<Token>> {
    if sql.len() > 65536 {
        return Err(Diagnostic::new(
            "WFT-LIMIT",
            "input",
            "SQL exceeds byte limit",
        ));
    }
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < sql.len() {
        let c = sql[i..].chars().next().unwrap();
        let start = i;
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if sql[i..].starts_with("--") || sql[i..].starts_with("/*") {
            return Err(error(
                "WFT-UNSUPPORTED",
                "Comments are outside this dialect",
                &Span { start, end: i + 2 },
            ));
        }
        let kind = if c == '\'' || c == '"' {
            let quote = c;
            i += 1;
            let mut value = String::new();
            let mut closed = false;
            while i < sql.len() {
                let ch = sql[i..].chars().next().unwrap();
                i += ch.len_utf8();
                if ch == '\0' {
                    return Err(error(
                        "WFT-SYNTAX",
                        "NUL is outside the initial text profile",
                        &Span { start, end: i },
                    ));
                }
                if ch == quote {
                    if sql[i..].starts_with(quote) {
                        value.push(quote);
                        i += 1
                    } else {
                        closed = true;
                        break;
                    }
                } else {
                    value.push(ch)
                }
            }
            if !closed {
                return Err(error(
                    "WFT-SYNTAX",
                    "Unclosed quoted token",
                    &Span { start, end: i },
                ));
            }
            if quote == '\'' {
                Kind::String(value)
            } else {
                Kind::Quoted(value)
            }
        } else if c.is_ascii_alphabetic() || c == '_' {
            i += 1;
            while i < sql.len()
                && ((sql.as_bytes()[i] as char).is_ascii_alphanumeric()
                    || sql.as_bytes()[i] == b'_')
            {
                i += 1
            }
            Kind::Word(sql[start..i].to_ascii_lowercase())
        } else if c.is_ascii_digit() {
            i += 1;
            while i < sql.len() && sql.as_bytes()[i].is_ascii_digit() {
                i += 1
            }
            if sql[i..].starts_with('.')
                && sql.as_bytes().get(i + 1).is_some_and(u8::is_ascii_digit)
            {
                i += 1;
                while i < sql.len() && sql.as_bytes()[i].is_ascii_digit() {
                    i += 1
                }
            }
            Kind::Number(sql[start..i].into())
        } else {
            i += c.len_utf8();
            Kind::Symbol(c)
        };
        tokens.push(Token {
            kind,
            span: Span { start, end: i },
        });
        if tokens.len() > 4096 {
            return Err(error(
                "WFT-LIMIT",
                "SQL token count exceeds limit",
                &Span { start, end: i },
            ));
        }
    }
    Ok(tokens)
}
pub(crate) struct Parser {
    tokens: Vec<Token>,
    index: usize,
    end: usize,
}
impl Parser {
    pub(crate) fn peek_identifier(&self) -> bool {
        matches!(
            self.tokens.get(self.index),
            Some(Token {
                kind: Kind::Quoted(_),
                ..
            })
        ) || matches!(self.tokens.get(self.index),Some(Token{kind:Kind::Word(v),..}) if !keyword(v))
    }

    pub(crate) fn new(sql: &str) -> Result<Self> {
        Ok(Self {
            tokens: lex(sql)?,
            index: 0,
            end: sql.len(),
        })
    }
    pub(crate) fn finish_application(&mut self) -> Result<()> {
        if self.peek_symbol(';') {
            self.symbol(';')?;
        }
        if self.index != self.tokens.len() {
            let construct = match &self.tokens[self.index].kind {
                Kind::Word(v) => v.as_str(),
                Kind::Symbol('>') => ">",
                Kind::Symbol('*') => "*",
                _ => "unrecognized syntax",
            };
            return Err(error(
                "WFT-UNSUPPORTED",
                &format!("Excluded application construct: {construct}"),
                &self.span(),
            ));
        }
        Ok(())
    }
    pub(crate) fn finish(&mut self) -> Result<()> {
        if self.peek_symbol(';') {
            self.symbol(';')?;
        }
        if self.index != self.tokens.len() {
            return Err(error(
                "WFT-UNSUPPORTED",
                "Trailing syntax is outside this dialect",
                &self.span(),
            ));
        }
        Ok(())
    }

    pub(crate) fn peek_word(&self, s: &str) -> bool {
        matches!(self.tokens.get(self.index),Some(Token{kind:Kind::Word(v),..}) if v==s)
    }
    pub(crate) fn peek_symbol(&self, c: char) -> bool {
        matches!(self.tokens.get(self.index),Some(Token{kind:Kind::Symbol(v),..}) if *v==c)
    }
    pub(crate) fn span(&self) -> Span {
        self.tokens
            .get(self.index)
            .map(|t| t.span.clone())
            .unwrap_or(Span {
                start: self.end,
                end: self.end,
            })
    }
    fn take(&mut self) -> Result<Token> {
        let t = self
            .tokens
            .get(self.index)
            .cloned()
            .ok_or_else(|| error("WFT-SYNTAX", "Unexpected end of query", &self.span()))?;
        self.index += 1;
        Ok(t)
    }
    pub(crate) fn word(&mut self, s: &str) -> Result<()> {
        if self.peek_word(s) {
            self.index += 1;
            Ok(())
        } else {
            Err(error(
                if self.index == self.tokens.len() {
                    "WFT-SYNTAX"
                } else {
                    "WFT-UNSUPPORTED"
                },
                "Expected dialect keyword",
                &self.span(),
            ))
        }
    }
    pub(crate) fn symbol(&mut self, c: char) -> Result<()> {
        if self.peek_symbol(c) {
            self.index += 1;
            Ok(())
        } else {
            Err(error(
                if self.index == self.tokens.len() {
                    "WFT-SYNTAX"
                } else {
                    "WFT-UNSUPPORTED"
                },
                "Expected dialect punctuation",
                &self.span(),
            ))
        }
    }
    pub(crate) fn name(&mut self) -> Result<Name> {
        let t = self.take()?;
        match t.kind {
            Kind::Word(v) if !keyword(&v) => Ok(Name {
                value: v,
                quoted: false,
                span: t.span,
            }),
            Kind::Quoted(v) if !v.is_empty() => Ok(Name {
                value: v,
                quoted: true,
                span: t.span,
            }),
            _ => Err(error(
                "WFT-UNSUPPORTED",
                "Expected logical identifier",
                &t.span,
            )),
        }
    }
    pub(crate) fn column(&mut self) -> Result<Column> {
        let alias = self.name()?;
        self.symbol('.')?;
        let field = self.name()?;
        let span = Span {
            start: alias.span.start,
            end: field.span.end,
        };
        Ok(Column { alias, field, span })
    }
    pub(crate) fn source(&mut self) -> Result<Source> {
        let first = self.name()?;
        let (namespace, name) = if self.peek_symbol('.') {
            self.index += 1;
            (Some(first), self.name()?)
        } else {
            (None, first)
        };
        let alias = if self.peek_word("as") {
            self.index += 1;
            self.name()?
        } else if matches!(self.tokens.get(self.index),Some(Token{kind:Kind::Word(v),..}) if !keyword(v))
            || matches!(
                self.tokens.get(self.index),
                Some(Token {
                    kind: Kind::Quoted(_),
                    ..
                })
            )
        {
            self.name()?
        } else {
            name.clone()
        };
        Ok(Source {
            namespace,
            name,
            alias,
        })
    }
    pub(crate) fn literal(&mut self) -> Result<Literal> {
        let start = self.span().start;
        let negative = self.peek_symbol('-');
        if negative {
            self.index += 1
        }
        let t = self.take()?;
        let (value, kind) = match t.kind {
            Kind::Number(v) => (
                format!("{}{v}", if negative { "-" } else { "" }),
                LiteralKind::Number,
            ),
            Kind::String(v) if !negative => (v, LiteralKind::String),
            Kind::Word(v) if !negative && (v == "true" || v == "false") => {
                (v, LiteralKind::Boolean)
            }
            _ => {
                return Err(error(
                    "WFT-UNSUPPORTED",
                    "Unsupported predicate operand",
                    &t.span,
                ))
            }
        };
        Ok(Literal {
            value,
            kind,
            span: Span {
                start,
                end: t.span.end,
            },
        })
    }
    fn predicates(&mut self) -> Result<Vec<Predicate>> {
        let mut predicates = Vec::new();
        loop {
            let left = self.column()?;
            self.symbol('=')?;
            let right = if matches!(
                self.tokens.get(self.index),
                Some(Token {
                    kind: Kind::Quoted(_),
                    ..
                })
            ) || matches!(self.tokens.get(self.index),Some(Token{kind:Kind::Word(v),..}) if !keyword(v))
            {
                Operand::Column(self.column()?)
            } else {
                Operand::Literal(self.literal()?)
            };
            let end = match &right {
                Operand::Column(c) => c.span.end,
                Operand::Literal(l) => l.span.end,
            };
            let span = Span {
                start: left.span.start,
                end,
            };
            predicates.push(Predicate { left, right, span });
            if !self.peek_word("and") {
                break;
            }
            self.index += 1;
        }
        Ok(predicates)
    }
    fn query(&mut self) -> Result<Query> {
        self.word("select")?;
        let mut outputs = Vec::new();
        loop {
            let start = self.span().start;
            let sum = self.peek_word("sum");
            if sum {
                self.index += 1;
                self.symbol('(')?
            }
            let column = self.column()?;
            let end = if sum {
                self.symbol(')')?;
                self.tokens[self.index - 1].span.end
            } else {
                column.span.end
            };
            let alias = if self.peek_word("as") {
                self.index += 1;
                Some(self.name()?)
            } else {
                None
            };
            outputs.push(Projection {
                column,
                sum,
                span: Span { start, end },
                alias,
            });
            if outputs.len() > 256 {
                return Err(error(
                    "WFT-LIMIT",
                    "Output count exceeds limit",
                    &self.span(),
                ));
            }
            if !self.peek_symbol(',') {
                break;
            }
            self.index += 1;
        }
        self.word("from")?;
        let source = self.source()?;
        let mut joins = Vec::new();
        while self.peek_word("inner") || self.peek_word("join") {
            if joins.len() == 16 {
                return Err(error("WFT-LIMIT", "Join count exceeds limit", &self.span()));
            }
            if self.peek_word("inner") {
                self.index += 1
            }
            self.word("join")?;
            let right = self.source()?;
            self.word("on")?;
            joins.push((right, self.predicates()?));
        }
        let predicates = if self.peek_word("where") {
            self.index += 1;
            self.predicates()?
        } else {
            Vec::new()
        };
        let mut groups = Vec::new();
        if self.peek_word("group") {
            self.index += 1;
            self.word("by")?;
            loop {
                groups.push(self.column()?);
                if !self.peek_symbol(',') {
                    break;
                }
                self.index += 1
            }
        }
        if self.peek_symbol(';') {
            self.index += 1
        }
        if self.index != self.tokens.len() {
            return Err(error(
                "WFT-UNSUPPORTED",
                "Trailing syntax is outside this dialect",
                &self.span(),
            ));
        }
        Ok(Query {
            outputs,
            source,
            joins,
            predicates,
            groups,
        })
    }
}
pub fn parse(sql: &str) -> Result<Query> {
    Parser {
        tokens: lex(sql)?,
        index: 0,
        end: sql.len(),
    }
    .query()
}
