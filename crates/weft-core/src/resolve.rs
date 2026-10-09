use crate::{
    error::{Diagnostic, Result},
    exact,
    ir::{Expression, Family, LogicalPlan, LogicalType, Node, Output, Span},
    model::{Catalog, Record},
    syntax::{Column, Operand, Predicate, Query, Source},
};
use serde_json::json;
use std::collections::BTreeSet;
struct Scan {
    alias: String,
    occurrence: String,
    record: Record,
}
struct Resolver<'a> {
    catalog: &'a Catalog,
    scans: Vec<Scan>,
    capabilities: BTreeSet<String>,
    literals: usize,
}
fn boolean() -> LogicalType {
    LogicalType {
        family: Family::Boolean,
        facets: json!({}),
        nullable: false,
    }
}
impl Resolver<'_> {
    fn cap(&mut self, s: &str) {
        self.capabilities.insert(s.into());
    }
    fn source(&mut self, source: &Source) -> Result<Node> {
        if self.scans.iter().any(|s| s.alias == source.alias.key()) {
            return Err(
                Diagnostic::new("WFT-NAME-AMBIGUOUS", "resolve", "Duplicate scan alias")
                    .at(&source.alias.span),
            );
        }
        let record = self
            .catalog
            .record(source.namespace.as_ref(), &source.name)?;
        let occurrence = format!("s{}", self.scans.len());
        let node = Node::Scan {
            occurrence: occurrence.clone(),
            record: record.identity.clone(),
            pin: record.pin.clone(),
        };
        self.scans.push(Scan {
            alias: source.alias.value.clone(),
            occurrence,
            record,
        });
        self.cap("scan");
        Ok(node)
    }
    fn column(&mut self, c: &Column) -> Result<(Expression, String)> {
        let scan = self
            .scans
            .iter()
            .find(|s| s.alias == c.alias.key())
            .ok_or_else(|| {
                Diagnostic::new("WFT-NAME-MISSING", "resolve", "Scan alias is not visible")
                    .at(&c.alias.span)
            })?;
        let (identity, logical_type, name) = self.catalog.field(&scan.record, &c.field)?;
        let expression = Expression::Field {
            scan: scan.occurrence.clone(),
            identity,
            logical_type: logical_type.clone(),
            span: c.span.clone(),
        };
        self.cap(match logical_type.family {
            Family::String => "type.string",
            Family::Boolean => "type.boolean",
            Family::Integer => "type.integer",
            Family::Decimal => "type.decimal",
        });
        if logical_type.family == Family::Integer && logical_type.facets == json!({}) { self.cap("type.integer.unbounded"); }
        Ok((expression, name))
    }
    fn predicates(&mut self, predicates: &[Predicate]) -> Result<Expression> {
        let mut result: Option<Expression> = None;
        for p in predicates {
            let (left, _) = self.column(&p.left)?;
            let right = match &p.right {
                Operand::Column(c) => self.column(c)?.0,
                Operand::Literal(l) => {
                    self.literals += 1;
                    if self.literals > 1024 {
                        return Err(Diagnostic::new(
                            "WFT-LIMIT",
                            "type",
                            "Literal parameter count exceeds bound",
                        )
                        .at(&l.span));
                    }
                    exact::literal(l, left.logical_type())?
                }
            };
            if left.logical_type().family != right.logical_type().family {
                return Err(Diagnostic::new(
                    "WFT-TYPE",
                    "type",
                    "Equality operands have incompatible families",
                )
                .at(&p.span));
            }
            let equal = Expression::Equal {
                left: Box::new(left),
                right: Box::new(right),
                logical_type: boolean(),
                span: p.span.clone(),
            };
            self.cap("equal");
            result = Some(match result {
                None => equal,
                Some(previous) => {
                    let span = Span {
                        start: previous.span().start,
                        end: equal.span().end,
                    };
                    self.cap("and");
                    Expression::And {
                        left: Box::new(previous),
                        right: Box::new(equal),
                        logical_type: boolean(),
                        span,
                    }
                }
            });
        }
        result.ok_or_else(|| Diagnostic::new("WFT-SYNTAX", "parse", "Predicate list is empty"))
    }
    fn query(mut self, q: Query) -> Result<LogicalPlan> {
        let mut node = self.source(&q.source)?;
        for (source, predicates) in &q.joins {
            let right = self.source(source)?;
            let on = self.predicates(predicates)?;
            self.cap("innerJoin");
            node = Node::InnerJoin {
                left: Box::new(node),
                right: Box::new(right),
                on,
            };
        }
        if !q.predicates.is_empty() {
            let predicate = self.predicates(&q.predicates)?;
            self.cap("filter");
            node = Node::Filter {
                input: Box::new(node),
                predicate,
            }
        }
        let groups = q
            .groups
            .iter()
            .map(|c| self.column(c).map(|(e, _)| e))
            .collect::<Result<Vec<_>>>()?;
        let grouped = !groups.is_empty() || q.outputs.iter().any(|p| p.sum);
        let mut names = BTreeSet::new();
        let mut outputs = Vec::new();
        let mut aggregates = Vec::new();
        for p in &q.outputs {
            let (field, default) = self.column(&p.column)?;
            let name = p
                .alias
                .as_ref()
                .map(|n| n.value.clone())
                .unwrap_or(if p.sum { "sum".into() } else { default });
            if !names.insert(name.clone()) {
                return Err(Diagnostic::new(
                    "WFT-NAME-AMBIGUOUS",
                    "resolve",
                    "Duplicate output label",
                )
                .at(&p.span));
            }
            let expression = if p.sum {
                let arg_type = field.logical_type();
                if !matches!(arg_type.family, Family::Integer | Family::Decimal) {
                    return Err(Diagnostic::new(
                        "WFT-TYPE",
                        "type",
                        "SUM requires exact numeric input",
                    )
                    .at(&p.span));
                }
                let logical_type = LogicalType {
                    family: arg_type.family.clone(),
                    facets: if arg_type.family == Family::Decimal {
                        json!({"scale":arg_type.facets["scale"]})
                    } else {
                        json!({})
                    },
                    nullable: groups.is_empty(),
                };
                self.cap("sum");
                let sum = Expression::Sum {
                    argument: Box::new(field),
                    logical_type,
                    span: p.span.clone(),
                };
                aggregates.push(sum.clone());
                sum
            } else {
                if grouped && !groups.iter().any(|g| field.same_field(g)) {
                    return Err(Diagnostic::new(
                        "WFT-GROUP",
                        "type",
                        "Projection is not one of the grouped fields",
                    )
                    .at(&p.span));
                }
                field
            };
            outputs.push(Output { name, expression });
        }
        if grouped {
            if !groups.is_empty() {
                self.cap("group")
            };
            node = Node::Aggregate {
                input: Box::new(node),
                groups,
                aggregates,
            };
        }
        self.cap("project");
        node = Node::Project {
            input: Box::new(node),
            outputs,
        };
        Ok(LogicalPlan {
            ir_version: "weft-ir/0.1.0".into(),
            module_pins: self.catalog.pins(),
            required_capabilities: self.capabilities.into_iter().collect(),
            root: node,
        })
    }
}
pub fn resolve(catalog: &Catalog, query: Query) -> Result<LogicalPlan> {
    Resolver {
        catalog,
        scans: Vec::new(),
        capabilities: BTreeSet::new(),
        literals: 0,
    }
    .query(query)
}

#[cfg(test)]
mod tests {
    use super::*;
    // @covers US-001-AC3 @covers US-001-AC4 @covers US-006-AC3
    #[test]
    fn typed_ast_resolver_literal_boundary_and_empty_join_refusal() {
        let corpus:serde_json::Value=serde_json::from_str(include_str!("../../../docs/helix/03-test/fixtures/cases.json")).unwrap();
        let modules=serde_json::from_value(corpus[0]["request"]["modules"].clone()).unwrap();
        let catalog=Catalog::prepare(modules).unwrap();
        let original=crate::syntax::parse("SELECT c.name FROM Customer c WHERE c.active = TRUE").unwrap();
        let mut boundary=original.clone();
        boundary.predicates=vec![original.predicates[0].clone();1024];
        let plan=resolve(&catalog,boundary).unwrap();
        assert!(plan.required_capabilities.contains(&"and".to_string()));
        let mut excess=original.clone();
        excess.predicates=vec![original.predicates[0].clone();1025];
        let error=resolve(&catalog,excess).unwrap_err();
        assert_eq!(error.code,"WFT-LIMIT");
        assert_eq!(error.phase,"type");
        assert_eq!(error.message,"Literal parameter count exceeds bound");
        let mut empty_join=original;
        let other=crate::syntax::parse("SELECT d.name FROM Customer d").unwrap().source;
        empty_join.joins.push((other,vec![]));
        let error=resolve(&catalog,empty_join).unwrap_err();
        assert_eq!(error.code,"WFT-SYNTAX");
        assert_eq!(error.phase,"parse");
        assert_eq!(error.message,"Predicate list is empty");
    }
}
