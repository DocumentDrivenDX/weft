//! Read-only public views; construction remains in the private 0.4 frontend.
use crate::{
    application_ir::{Field, Scan, Value},
    application_model::{Descriptor, RelationshipRead},
    arithmetic_plan::{Join, Predicate},
    ir::{Identity, ModelPin, Span},
    path_ir,
};
#[derive(Clone, Copy)]
pub struct Plan04View<'a> {
    pub(super) plan: &'a path_ir::Plan,
}
#[derive(Clone, Copy)]
pub struct PathView<'a> {
    pub(super) path: &'a path_ir::PathRead,
}
impl<'a> PathView<'a> {
    pub fn start_scan(self) -> &'a str {
        self.path.start_scan()
    }
    pub fn start_record(self) -> &'a Identity {
        self.path.start_record()
    }
    pub fn hops(self) -> &'a [RelationshipRead; 2] {
        self.path.hops()
    }
    pub fn span(self) -> &'a Span {
        self.path.span()
    }
    pub fn hop_spans(self) -> &'a [Span; 2] {
        self.path.hop_spans()
    }
    pub fn serialized(self) -> serde_json::Value {
        serde_json::to_value(self.path).expect("owned path serialization")
    }
}
#[derive(Clone, Copy)]
pub enum ExpressionView<'a> {
    Legacy(&'a crate::arithmetic_plan::Expression),
    RelatedPaths { path: PathView<'a>, bound: u16 },
    CountDistinctPathTargets { occurrence: &'a str },
}
#[derive(Clone, Copy)]
pub struct OutputView<'a> {
    pub name: &'a str,
    pub expression: ExpressionView<'a>,
}
#[derive(Clone, Copy)]
pub struct ExpansionView<'a> {
    pub occurrence: &'a str,
    pub path: PathView<'a>,
}
#[derive(Clone, Copy)]
pub struct HavingView<'a> {
    pub count: ExpressionView<'a>,
    pub threshold: &'a Value,
}
fn expression(e: &path_ir::Expression) -> ExpressionView<'_> {
    match e {
        path_ir::Expression::Legacy(v) => ExpressionView::Legacy(v),
        path_ir::Expression::Path(path_ir::PathExpression::RelatedPaths { path, bound }) => {
            ExpressionView::RelatedPaths {
                path: PathView { path },
                bound: *bound,
            }
        }
        path_ir::Expression::Path(path_ir::PathExpression::CountDistinctPathTargets {
            path_occurrence,
            ..
        }) => ExpressionView::CountDistinctPathTargets {
            occurrence: path_occurrence,
        },
    }
}
impl<'a> Plan04View<'a> {
    pub(crate) fn new(plan: &'a path_ir::Plan) -> Self {
        Self { plan }
    }
    pub fn ir_version(self) -> &'static str {
        self.plan.ir_version()
    }
    pub fn source(self) -> &'a Scan {
        self.plan.source()
    }
    pub fn pins(self) -> &'a [ModelPin] {
        self.plan.pins()
    }
    pub fn capabilities(self) -> &'a [String] {
        self.plan.capabilities()
    }
    pub fn joins(self) -> &'a [Join] {
        self.plan.joins()
    }
    pub fn filters(self) -> &'a [Predicate] {
        self.plan.filters()
    }
    pub fn groups(self) -> &'a [Field] {
        self.plan.groups()
    }
    pub fn order(self) -> &'a [Field] {
        self.plan.order()
    }
    pub fn type_graph(self) -> &'a [Descriptor] {
        self.plan.type_graph()
    }
    pub fn outer_join_scans(self) -> &'a [String] {
        self.plan.outer_join_scans()
    }
    pub fn limit(self) -> Option<u16> {
        self.plan.limit()
    }
    pub fn distinct(self) -> bool {
        self.plan.distinct()
    }
    pub fn aggregate(self) -> bool {
        self.plan.aggregate()
    }
    pub fn outputs(self) -> impl Iterator<Item = OutputView<'a>> {
        self.plan.outputs().iter().map(|o| OutputView {
            name: &o.name,
            expression: expression(&o.expression),
        })
    }
    pub fn having(self) -> impl Iterator<Item = HavingView<'a>> {
        self.plan.having().iter().map(|h| HavingView {
            count: expression(&h.count),
            threshold: &h.threshold,
        })
    }
    pub fn expansion(self) -> Option<ExpansionView<'a>> {
        self.plan.expansion().map(|p| ExpansionView {
            occurrence: p.occurrence(),
            path: PathView { path: p.path() },
        })
    }
    pub fn serialized(self) -> serde_json::Value {
        serde_json::to_value(self.plan).expect("owned plan serialization")
    }
}
