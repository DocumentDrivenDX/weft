//! Shared semantic scope for versioned application resolvers.
use crate::{
    application_ir as ir,
    application_model::{Descriptor, Shape},
    application_syntax as ast,
    error::{Diagnostic, Result},
    exact,
    ir::{Family, Identity, LogicalType},
    model::{Catalog, Record},
    syntax::{Column, Literal, LiteralKind, Name},
};

use serde_json::json;
use std::collections::BTreeSet;
use crate::application_resolve::Parameters;
fn fail(code: &str, message: &str) -> Diagnostic {
    Diagnostic::new(code, "resolve", message)
}
pub(crate) struct Scope<'a> {
    pub(crate) catalog: &'a Catalog,
    pub(crate) records: Vec<(Name, Record, String)>,
    pub(crate) parameters: Parameters,
    pub(crate) used: BTreeSet<String>,
    pub(crate) caps: BTreeSet<String>,
    pub(crate) graph: Vec<Descriptor>,
}
impl<'a> Scope<'a> {
    pub(crate) fn source(&mut self, s: &ast::Query) -> Result<ir::Scan> {
        self.add_source(&s.source)
    }
    pub(crate) fn add_source(&mut self, s: &crate::syntax::Source) -> Result<ir::Scan> {
        if self
            .records
            .iter()
            .any(|(n, _, _)| n.value == s.alias.value)
        {
            return Err(fail("WFT-NAME-AMBIGUOUS", "Repeated source alias").at(&s.alias.span));
        }
        let record = self.catalog.record(s.namespace.as_ref(), &s.name)?;
        let occurrence = format!("s{}", self.records.len());
        let scan = ir::Scan {
            occurrence: occurrence.clone(),
            record: record.identity.clone(),
            pin: record.pin.clone(),
        };
        self.records.push((s.alias.clone(), record, occurrence));
        self.caps.insert("scan".into());
        Ok(scan)
    }
    pub(crate) fn record(&self, n: &Name) -> Result<(&Record, String)> {
        let (_, r, s) = self
            .records
            .iter()
            .find(|(alias, _, _)| alias.value == n.value)
            .ok_or_else(|| fail("WFT-NAME-MISSING", "Source alias is not in scope").at(&n.span))?;
        Ok((r, s.clone()))
    }
    pub(crate) fn record_for_column(&self, column: &Column) -> Result<(&Record, String)> {
        if !column.unqualified { return self.record(&column.alias); }
        let mut matches = 0;
        let mut selected = None;
        for (_, record, scan) in &self.records {
            let count = self.catalog.member_name_matches(record, &column.field)?;
            matches += count;
            if count > 0 { selected = Some((record, scan.clone())); }
        }
        if matches != 1 {
            return Err(fail(if matches == 0 { "WFT-NAME-MISSING" } else { "WFT-NAME-AMBIGUOUS" },
                "Bare Field name must match exactly one visible Record member").at(&column.span));
        }
        Ok(selected.unwrap())
    }
    pub(crate) fn field(&mut self, c: &Column) -> Result<ir::Field> {
        let (r, s) = self.record_for_column(c)?;
        let (id, t, _) = self.catalog.field(r, &c.field)?;
        if t.family == Family::Integer && t.facets == json!({}) { self.caps.insert("type.integer.unbounded".into()); }
        self.caps.insert(format!(
            "type.{}",
            match t.family {
                Family::Boolean => "boolean",
                Family::String => "string",
                Family::Integer => "integer",
                Family::Decimal => "decimal",
            }
        ));
        Ok(ir::Field {
            scan: s,
            identity: id,
            logical_type: t,
            span: c.span.clone(),
        })
    }
    pub(crate) fn descriptors(&mut self, graph: Vec<Descriptor>) {
        for d in graph {
            match &d.shape {
                Shape::Scalar { logical_type } => {
                    if logical_type.family == Family::Integer && logical_type.facets == json!({}) { self.caps.insert("type.integer.unbounded".into()); }
                    self.caps.insert(format!(
                        "type.{}",
                        match logical_type.family {
                            Family::Boolean => "boolean",
                            Family::String => "string",
                            Family::Integer => "integer",
                            Family::Decimal => "decimal",
                        }
                    ));
                }
                Shape::Sequence { .. } => {
                    self.caps.insert("value.sequence".into());
                }
                Shape::Map { .. } => {
                    self.caps.insert("value.map".into());
                }
                Shape::Structured { .. } => {
                    self.caps.insert("value.structured".into());
                }
                Shape::Record { .. } => {}
            }
            if d.availability.as_deref() == Some("absent-allowed") {
                self.caps.insert("value.presence".into());
            }
            if !self.graph.iter().any(|g| g.identity == d.identity) {
                self.graph.push(d);
            }
        }
    }
    pub(crate) fn value(&mut self, v: &ast::Value, t: &LogicalType) -> Result<ir::Value> {
        match v {
            ast::Value::Field(c) => {
                let f = self.field(c)?;
                if f.logical_type.family != t.family {
                    return Err(
                        fail("WFT-TYPE", "Compared fields require the same exact family")
                            .at(&c.span),
                    );
                }
                Ok(ir::Value::Field { field: f })
            }
            ast::Value::Literal(l) => {
                exact::literal(l, t)?;
                Ok(ir::Value::Literal {
                    value: l.value.clone(),
                    logical_type: t.clone(),
                    span: l.span.clone(),
                })
            }
            ast::Value::Parameter(n) => {
                if n.quoted {
                    return Err(fail(
                        "WFT-PARAMETER",
                        "Source parameter names must be unquoted ASCII identifiers",
                    )
                    .at(&n.span));
                }
                let p = self.parameters.get(&n.value).ok_or_else(|| {
                    fail("WFT-PARAMETER", "Missing source parameter binding").at(&n.span)
                })?;
                if p.family != t.family {
                    return Err(
                        fail("WFT-PARAMETER", "Source parameter family is incompatible")
                            .at(&n.span),
                    );
                }
                let kind = match p.family {
                    Family::String => LiteralKind::String,
                    Family::Boolean => LiteralKind::Boolean,
                    _ => LiteralKind::Number,
                };
                let lexical = match p.family {
                    Family::Boolean => p.value == "true" || p.value == "false",
                    Family::String => !p.value.contains('\0'),
                    _ => number_text(&p.value),
                };
                if !lexical {
                    return Err(fail(
                        "WFT-PARAMETER",
                        "Source parameter value violates the exact lexical grammar",
                    )
                    .at(&n.span));
                }
                exact::literal(
                    &Literal {
                        value: p.value.clone(),
                        kind,
                        span: n.span.clone(),
                    },
                    t,
                )?;
                self.used.insert(n.value.clone());
                self.caps.insert("parameter.named".into());
                Ok(ir::Value::Parameter {
                    name: n.value.clone(),
                    value: p.value.clone(),
                    logical_type: t.clone(),
                    span: n.span.clone(),
                })
            }
        }
    }
    pub(crate) fn predicate(&mut self, p: &ast::Predicate, on: bool) -> Result<ir::Predicate> {
        match p {
            ast::Predicate::Compare {
                columns,
                values,
                greater,
            } => {
                let fields = columns
                    .iter()
                    .map(|c| self.field(c))
                    .collect::<Result<Vec<_>>>()?;
                if on && (*greater || !matches!(values.as_slice(), [ast::Value::Field(_)])) {
                    return Err(fail("WFT-UNSUPPORTED", "JOIN ON requires field equality"));
                }
                let values = values
                    .iter()
                    .zip(&fields)
                    .map(|(v, f)| self.value(v, &f.logical_type))
                    .collect::<Result<Vec<_>>>()?;
                self.caps.insert(
                    if *greater {
                        "compare.lexicographicGreater"
                    } else {
                        "equal"
                    }
                    .into(),
                );
                if *greater {
                    Ok(ir::Predicate::LexicographicGreater {
                        columns: fields,
                        values,
                    })
                } else {
                    Ok(ir::Predicate::Equal {
                        left: fields.into_iter().next().unwrap(),
                        right: values.into_iter().next().unwrap(),
                    })
                }
            }
            ast::Predicate::HasRelated { relationship, key } => {
                if on {
                    return Err(fail("WFT-UNSUPPORTED", "HAS_RELATED is a WHERE predicate"));
                }
                let (r, scan) = self.record(&relationship.alias)?;
                let rel = self.catalog.relationship_read(r, &relationship.field)?;
                if key.len() != rel.target_key.types.len() {
                    return Err(fail("WFT-TYPE", "Related key tuple arity mismatch"));
                }
                if key.iter().any(|v| matches!(v, ast::Value::Field(_))) {
                    return Err(fail(
                        "WFT-UNSUPPORTED",
                        "Related key operands require literals or parameters",
                    ));
                }
                let key = key
                    .iter()
                    .zip(&rel.target_key.types)
                    .map(|(v, t)| self.value(v, t))
                    .collect::<Result<Vec<_>>>()?;
                self.caps.insert("relationship.exists".into());
                if rel.inverse {
                    self.caps.insert("relationship.inverse".into());
                }
                Ok(ir::Predicate::HasRelated {
                    scan,
                    relationship: rel,
                    key,
                })
            }
        }
    }
}
fn number_text(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    let mut parts = s.split('.');
    let whole = parts.next().unwrap_or("");
    !whole.is_empty()
        && whole.bytes().all(|b| b.is_ascii_digit())
        && match parts.next() {
            None => true,
            Some(f) => {
                !f.is_empty() && f.bytes().all(|b| b.is_ascii_digit()) && parts.next().is_none()
            }
        }
}
