//! Original per-operation operands for subsequent effective-domain checking.
//! No route selection, type rewriting, SQL or native permission is performed.
use crate::{
    application_ir as app,
    application_lineage::OrderDirection,
    current_query_designation::CurrentQueryDesignations,
    error::{Diagnostic, Result},
    ir::{Identity, LogicalType},
    security_ir::QueryOperator,
};
use std::collections::BTreeSet;
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-CURRENT-QUERY-OPERANDS",
        "capability",
        "Original current query operands refused",
    )
}
#[derive(Debug)]
pub enum Input<'a> {
    Field {
        scan: &'a str,
        identity: &'a Identity,
        logical_type: &'a LogicalType,
        original_action: Option<&'a str>,
    },
    Literal {
        value: &'a str,
        logical_type: &'a LogicalType,
    },
    Parameter {
        name: &'a str,
        value: &'a str,
        logical_type: &'a LogicalType,
    },
}
/// Operand position is significant, including each side of a join/self-join.
#[derive(Debug)]
pub struct Operand<'a> {
    path: String,
    input: Input<'a>,
}
impl<'a> Operand<'a> {
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn input(&self) -> &Input<'a> {
        &self.input
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Equal,
    LexicographicGreater,
    Group,
    Order(OrderDirection),
    Count,
    CountDistinct,
    Boolean(bool),
}
#[derive(Debug)]
pub struct Operation<'a> {
    path: String,
    operator: Option<QueryOperator>,
    kind: Kind,
    operands: Vec<Operand<'a>>,
    output: Option<&'a LogicalType>,
}
impl<'a> Operation<'a> {
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn operator(&self) -> Option<QueryOperator> {
        self.operator
    }
    pub fn kind(&self) -> Kind {
        self.kind
    }
    pub fn operands(&self) -> &[Operand<'a>] {
        &self.operands
    }
    pub fn output_type(&self) -> Option<&'a LogicalType> {
        self.output
    }
}
/// Factory-issued original operand ledger, borrowed from the actual designation.
/// Public Input/Kind values are descriptions, not an import path or permission.
/// ```compile_fail
/// use weft_core::current_query_operands::CurrentQueryOperands;
/// fn import(raw:&str)->CurrentQueryOperands<'_>{serde_json::from_str(raw).unwrap()}
/// ```
pub struct CurrentQueryOperands<'a> {
    designation: &'a CurrentQueryDesignations<'a>,
    operations: Vec<Operation<'a>>,
}
impl<'a> CurrentQueryOperands<'a> {
    pub fn designation(&self) -> &CurrentQueryDesignations<'a> {
        self.designation
    }
    /// Occurrence ledger, not an execution schedule. The original Plan owns
    /// relational stage order; its read requirements own projection disclosure.
    pub fn operations(&self) -> &[Operation<'a>] {
        &self.operations
    }
}
struct Builder<'a> {
    designation: &'a CurrentQueryDesignations<'a>,
    seen: BTreeSet<(String, String)>,
    operations: Vec<Operation<'a>>,
}
impl<'a> Builder<'a> {
    fn field(&mut self, path: String, f: &'a app::Field) -> Result<Operand<'a>> {
        if self.seen.len() >= 4096 || !self.seen.insert((path.clone(), f.scan.clone())) {
            return Err(fail());
        }
        Ok(Operand {
            input: Input::Field {
                scan: &f.scan,
                identity: &f.identity,
                logical_type: &f.logical_type,
                original_action: self.designation.original_action(&path, &f.scan)?,
            },
            path,
        })
    }
    fn value(&mut self, path: String, v: &'a app::Value) -> Result<Operand<'a>> {
        Ok(match v {
            app::Value::Field { field } => return self.field(path, field),
            app::Value::Literal {
                value,
                logical_type,
                ..
            } => Operand {
                path,
                input: Input::Literal {
                    value,
                    logical_type,
                },
            },
            app::Value::Parameter {
                name,
                value,
                logical_type,
                ..
            } => Operand {
                path,
                input: Input::Parameter {
                    name,
                    value,
                    logical_type,
                },
            },
        })
    }
    fn push(
        &mut self,
        path: String,
        operator: Option<QueryOperator>,
        kind: Kind,
        operands: Vec<Operand<'a>>,
        output: Option<&'a LogicalType>,
    ) -> Result<()> {
        if self.operations.len() >= 4096 {
            return Err(fail());
        }
        self.operations.push(Operation {
            path,
            operator,
            kind,
            operands,
            output,
        });
        Ok(())
    }
    fn predicate(
        &mut self,
        path: String,
        p: &'a app::Predicate,
        operator: QueryOperator,
    ) -> Result<()> {
        match p {
            app::Predicate::Boolean { value } => {
                self.push(path, None, Kind::Boolean(*value), vec![], None)
            }
            app::Predicate::Equal { left, right } => {
                let a = self.field(format!("{path}/left"), left)?;
                let b = self.value(format!("{path}/right"), right)?;
                self.push(path, Some(operator), Kind::Equal, vec![a, b], None)
            }
            app::Predicate::LexicographicGreater { columns, values } => {
                if columns.is_empty() || columns.len() != values.len() {
                    return Err(fail());
                }
                let mut operands = Vec::new();
                for (i, (f, v)) in columns.iter().zip(values).enumerate() {
                    operands.push(self.field(format!("{path}/columns/{i}"), f)?);
                    operands.push(self.value(format!("{path}/values/{i}"), v)?);
                }
                self.push(
                    path,
                    Some(operator),
                    Kind::LexicographicGreater,
                    operands,
                    None,
                )
            }
            _ => Err(fail()),
        }
    }
}
/// Derive once from actual typed nodes, never from report JSON or reparsed SQL.
/// Effective routes must subsequently check each operand, presence, literal and
/// paired transform/key/runtime scope. A matching logical family alone is not
/// permission or evidence that two disclosed values may be compared.
pub fn derive<'a>(
    designation: &'a CurrentQueryDesignations<'a>,
) -> Result<CurrentQueryOperands<'a>> {
    let original = designation.preparation();
    let plan = original.plan();
    if original.order_directions().len() != plan.order.len() {
        return Err(fail());
    }
    let mut b = Builder {
        designation,
        seen: BTreeSet::new(),
        operations: Vec::new(),
    };
    for (i, join) in plan.joins.iter().enumerate() {
        for (j, p) in join.on.iter().enumerate() {
            b.predicate(format!("joins/{i}/on/{j}"), p, QueryOperator::Join)?;
        }
    }
    for (i, p) in plan.filters.iter().enumerate() {
        b.predicate(format!("filters/{i}"), p, QueryOperator::Predicate)?;
    }
    for (i, f) in plan.groups.iter().enumerate() {
        let path = format!("groups/{i}");
        let field = b.field(path.clone(), f)?;
        b.push(
            path,
            Some(QueryOperator::Group),
            Kind::Group,
            vec![field],
            None,
        )?;
    }
    for (i, f) in plan.order.iter().enumerate() {
        let path = format!("order/{i}");
        let field = b.field(path.clone(), f)?;
        b.push(
            path,
            Some(QueryOperator::Order),
            Kind::Order(original.order_directions()[i]),
            vec![field],
            None,
        )?;
    }
    for (i, o) in plan.outputs.iter().enumerate() {
        let path = format!("outputs/{i}");
        match &o.expression {
            app::Expression::Field { .. } => {}
            app::Expression::Count { logical_type } => {
                b.push(path, None, Kind::Count, vec![], Some(logical_type))?
            }
            app::Expression::CountDistinct {
                argument,
                logical_type,
            } => {
                let field = b.field(format!("{path}/argument"), argument)?;
                b.push(
                    path,
                    Some(QueryOperator::Aggregate),
                    Kind::CountDistinct,
                    vec![field],
                    Some(logical_type),
                )?;
            }
            // SUM has no qualified preservation property in this candidate. No partial graph.
            _ => return Err(fail()),
        }
    }
    let mut expected = BTreeSet::new();
    crate::application_property_uses::visit(plan, original.catalog(), |u| {
        if !expected.insert((u.path, u.scan.to_owned())) {
            return Err(fail());
        }
        Ok(())
    })?;
    if expected != b.seen {
        return Err(fail());
    }
    Ok(CurrentQueryOperands {
        designation,
        operations: b.operations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    fn request(name: &str) -> Value {
        let raw=match name{
  "join"=>include_str!("../../../tests/current-query-designation/fixtures/disclosed-token-selfjoin.request.json"),
  "sum"=>include_str!("../../../tests/current-query-designation/fixtures/unsupported-salary-sum.request.json"),
  _=>include_str!("../../../tests/current-query-designation/fixtures/mixed-original-uses.request.json"),
 };
        serde_json::from_str(raw).unwrap()
    }
    // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn self_join_keeps_paired_occurrences_and_distinct_argument() {
        let p = crate::application_lineage::prepare(&request("join").to_string()).unwrap();
        let d = crate::current_query_designation::derive(&p).unwrap();
        let g = derive(&d).unwrap();
        assert!(std::ptr::eq(g.designation(), &d));
        assert_eq!(g.operations().len(), 2);
        let join = &g.operations()[0];
        assert_eq!(join.kind(), Kind::Equal);
        assert_eq!(join.operator(), Some(QueryOperator::Join));
        assert_eq!(join.operands().len(), 2);
        let Input::Field {
            scan: a,
            identity: ai,
            original_action: aa,
            ..
        } = join.operands()[0].input()
        else {
            panic!()
        };
        let Input::Field {
            scan: b,
            identity: bi,
            original_action: ba,
            ..
        } = join.operands()[1].input()
        else {
            panic!()
        };
        assert_eq!((*a, *b), ("s0", "s1"));
        assert_eq!(ai, bi);
        assert_eq!((*aa, *ba), (None, None));
        assert_eq!(join.operands()[0].path(), "joins/0/on/0/left");
        assert_eq!(join.operands()[1].path(), "joins/0/on/0/right");
        let distinct = &g.operations()[1];
        assert_eq!(distinct.kind(), Kind::CountDistinct);
        assert_eq!(distinct.operands()[0].path(), "outputs/0/argument");
        assert!(matches!(
            distinct.operands()[0].input(),
            Input::Field { scan: "s1", .. }
        ));
        assert!(distinct.output_type().is_some());
    }
    // @covers US-009-AC1 @covers US-009-AC2
    #[test]
    fn hidden_original_salary_does_not_rewrite_projection_or_literal() {
        let p = crate::application_lineage::prepare(&request("mixed").to_string()).unwrap();
        let original = p.json().to_owned();
        let d = crate::current_query_designation::derive(&p).unwrap();
        let g = derive(&d).unwrap();
        let predicate = &g.operations()[0];
        let Input::Field {
            identity,
            logical_type,
            original_action,
            ..
        } = predicate.operands()[0].input()
        else {
            panic!()
        };
        assert_eq!(identity.element, "salary");
        assert_eq!(logical_type.family, crate::ir::Family::Integer);
        assert_eq!(*original_action, Some("query-original-salary-predicate"));
        let app::Predicate::Equal { left, right } = &p.plan().filters[0] else {
            panic!()
        };
        assert!(std::ptr::eq(*identity, &left.identity));
        let Input::Literal {
            value,
            logical_type,
        } = predicate.operands()[1].input()
        else {
            panic!()
        };
        let app::Value::Literal {
            value: expected,
            logical_type: expected_type,
            ..
        } = right
        else {
            panic!()
        };
        assert_eq!(*value, expected);
        assert!(std::ptr::eq(*logical_type, expected_type));
        assert_eq!(predicate.operands()[1].path(), "filters/0/right");
        assert_eq!(p.json(), original);
    }
    // @covers US-009-AC1 @covers US-009-AC2
    #[test]
    fn parameter_name_payload_and_original_type_remain_borrowed() {
        let mut r = request("mixed");
        r["sql"] =
            json!("SELECT r.salary FROM Resource r WHERE r.salary=:minimum ORDER BY r.token DESC");
        r["parameters"] = json!({"minimum":{"family":"integer","value":"200"}});
        let p = crate::application_lineage::prepare(&r.to_string()).unwrap();
        let d = crate::current_query_designation::derive(&p).unwrap();
        let g = derive(&d).unwrap();
        let Input::Parameter {
            name,
            value,
            logical_type,
        } = g.operations()[0].operands()[1].input()
        else {
            panic!()
        };
        assert_eq!((*name, *value), ("minimum", "200"));
        assert_eq!(logical_type.family, crate::ir::Family::Integer);
        assert_eq!(
            logical_type.facets,
            json!({"integerWidth":{"bits":64,"signed":true}})
        );
    }
    // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn false_limit_zero_keeps_original_use_and_field_free_count() {
        let mut r = request("mixed");
        r["sql"] =
            json!("SELECT COUNT(*) AS n FROM Resource r WHERE FALSE AND r.salary=200 LIMIT 0");
        let p = crate::application_lineage::prepare(&r.to_string()).unwrap();
        let d = crate::current_query_designation::derive(&p).unwrap();
        let g = derive(&d).unwrap();
        assert_eq!(
            g.operations().iter().map(|o| o.kind()).collect::<Vec<_>>(),
            vec![Kind::Boolean(false), Kind::Equal, Kind::Count]
        );
        assert_eq!(g.operations()[1].operands()[0].path(), "filters/1/left");
        assert!(g.operations()[2].operands().is_empty());
        assert!(g.operations()[2].output_type().is_some());
        assert_eq!(p.plan().limit, Some(0));
    }
    // @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn unsupported_sum_refuses_without_partial_graph() {
        let p = crate::application_lineage::prepare(&request("sum").to_string()).unwrap();
        let d = crate::current_query_designation::derive(&p).unwrap();
        assert_eq!(derive(&d).err().unwrap().code, "WFT-CURRENT-QUERY-OPERANDS");
    }
    // @covers US-009-AC1 @covers US-009-AC2
    #[test]
    fn byte_identical_plan_preserves_opposite_original_order_vectors() {
        let mut r = request("mixed");
        let mut observed = Vec::new();
        let mut plans = Vec::new();
        for sql in [
            "SELECT r.token FROM Resource r ORDER BY r.token DESC, r.salary ASC ",
            "SELECT r.token FROM Resource r ORDER BY r.token ASC , r.salary DESC",
        ] {
            r["sql"] = json!(sql);
            let p = crate::application_lineage::prepare(&r.to_string()).unwrap();
            let report: Value = serde_json::from_str(p.json()).unwrap();
            plans.push(report["applicationPlanJson"].clone());
            let d = crate::current_query_designation::derive(&p).unwrap();
            let g = derive(&d).unwrap();
            observed.push(g.operations().iter().map(|o| o.kind()).collect::<Vec<_>>());
        }
        assert_eq!(plans[0], plans[1]);
        assert_eq!(
            observed,
            vec![
                vec![
                    Kind::Order(OrderDirection::Desc),
                    Kind::Order(OrderDirection::Asc)
                ],
                vec![
                    Kind::Order(OrderDirection::Asc),
                    Kind::Order(OrderDirection::Desc)
                ]
            ]
        );
    }
    // @covers US-009-AC1 @covers US-009-AC2
    #[test]
    fn group_order_and_distinct_do_not_deduplicate_one_fields_distinct_uses() {
        let mut r = request("mixed");
        r["sql"]=json!("SELECT r.token,COUNT(DISTINCT r.token) AS n FROM Resource r GROUP BY r.token ORDER BY r.token DESC");
        let p = crate::application_lineage::prepare(&r.to_string()).unwrap();
        let d = crate::current_query_designation::derive(&p).unwrap();
        let g = derive(&d).unwrap();
        assert_eq!(
            g.operations().iter().map(|o| o.kind()).collect::<Vec<_>>(),
            vec![
                Kind::Group,
                Kind::Order(OrderDirection::Desc),
                Kind::CountDistinct
            ]
        );
        assert_eq!(
            g.operations()
                .iter()
                .map(|o| o.operands()[0].path())
                .collect::<Vec<_>>(),
            vec!["groups/0", "order/0", "outputs/1/argument"]
        );
        for (op, expected) in
            g.operations()
                .iter()
                .zip([None, Some("query-original-token-order"), None])
        {
            let Input::Field {
                original_action, ..
            } = op.operands()[0].input()
            else {
                panic!()
            };
            assert_eq!(*original_action, expected)
        }
    }
    // @covers US-009-AC1 @covers US-009-AC2
    #[test]
    fn tuple_comparison_retains_column_value_component_pairing() {
        let mut r = request("mixed");
        r["sql"] =
            json!("SELECT r.token FROM Resource r WHERE (r.salary,r.token) > (:salary,:token)");
        r["parameters"] = json!({"salary":{"family":"integer","value":"200"},"token":{"family":"string","value":"opaque-token"}});
        let p = crate::application_lineage::prepare(&r.to_string()).unwrap();
        let d = crate::current_query_designation::derive(&p).unwrap();
        let g = derive(&d).unwrap();
        assert_eq!(g.operations().len(), 1);
        let operation = &g.operations()[0];
        assert_eq!(operation.kind(), Kind::LexicographicGreater);
        assert_eq!(
            operation
                .operands()
                .iter()
                .map(|v| v.path())
                .collect::<Vec<_>>(),
            vec![
                "filters/0/columns/0",
                "filters/0/values/0",
                "filters/0/columns/1",
                "filters/0/values/1"
            ]
        );
        assert!(matches!(
            operation.operands()[0].input(),
            Input::Field {
                original_action: Some("query-original-salary-predicate"),
                ..
            }
        ));
        assert!(matches!(
            operation.operands()[1].input(),
            Input::Parameter {
                name: "salary",
                value: "200",
                ..
            }
        ));
        assert!(matches!(
            operation.operands()[2].input(),
            Input::Field {
                original_action: None,
                ..
            }
        ));
        assert!(matches!(
            operation.operands()[3].input(),
            Input::Parameter {
                name: "token",
                value: "opaque-token",
                ..
            }
        ));
    }
}
