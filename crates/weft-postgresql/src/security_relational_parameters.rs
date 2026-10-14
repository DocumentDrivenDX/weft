//! Separate original relational query/policy/Subject parameter channels. No native binding authority.
use crate::{
    security_graph_binding::{GraphBinding, GraphSource},
    security_graph_parameters::GraphParameterBindings,
};
use serde_json::Value;
use weft_core::{
    application_ir::Value as QueryValue,
    error::{Diagnostic, Result},
    ir::{Family, LogicalType},
    security_backend::SecurityBackendContext,
    security_ontology::SecurityOntologyClosure,
    security_parameter_sources::SecurityQueryParameterValue,
};
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-RELATIONAL-PARAMETER",
        "capability",
        "Original relational parameter channels refused",
    )
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalQueryParameterCodec {
    Boolean,
    Text,
    Numeric,
}
/// Original owner value/type retained by borrow. No annotation constructor or deserializer.
pub struct RelationalQueryParameterBinding<'a> {
    ordinal: usize,
    path: String,
    value: &'a QueryValue,
    logical_type: &'a LogicalType,
    codec: RelationalQueryParameterCodec,
}
impl RelationalQueryParameterBinding<'_> {
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn source_path(&self) -> &str {
        &self.path
    }
    pub fn value(&self) -> &QueryValue {
        self.value
    }
    pub fn logical_type(&self) -> &LogicalType {
        self.logical_type
    }
    pub fn codec(&self) -> RelationalQueryParameterCodec {
        self.codec
    }
}
/// This private owner ties all ordinals to one actual context. The Subject slot
/// remains unvalued and distinct from every SQL literal/parameter occurrence.
/// ```compile_fail
/// use weft_postgresql::security_relational_parameters::RelationalGraphParameterBindings;
/// fn reclassify(p:&mut RelationalGraphParameterBindings<'_, '_>){p.query.clear();}
/// ```
pub struct RelationalGraphParameterBindings<'a, 'owner> {
    context: &'a SecurityBackendContext<'owner>,
    graph: GraphParameterBindings,
    policy_values: Vec<Value>,
    query: Vec<RelationalQueryParameterBinding<'a>>,
}
impl<'a, 'owner> RelationalGraphParameterBindings<'a, 'owner> {
    pub fn graph(&self) -> &GraphParameterBindings {
        &self.graph
    }
    pub fn policy_values(&self) -> &[Value] {
        &self.policy_values
    }
    pub fn query(&self) -> &[RelationalQueryParameterBinding<'a>] {
        &self.query
    }
    pub fn require_context(&self, ctx: &SecurityBackendContext<'_>) -> Result<()> {
        if std::ptr::eq(self.context, ctx) {
            Ok(())
        } else {
            Err(fail())
        }
    }
}
/// Every actual query value is a separate original occurrence, after original
/// policy slots and the unvalued native Subject. This does not emit SQL, register
/// a backend or admit any native value/codec/execution/release.
pub fn prepare_relational_graph_parameters<'a, 'owner>(
    ctx: &'a SecurityBackendContext<'owner>,
    source: &GraphSource,
) -> Result<RelationalGraphParameterBindings<'a, 'owner>> {
    let binding = weft_core::json::checked_json(ctx.binding_json()).map_err(|_| fail())?;
    if binding != serde_json::to_value(source).map_err(|_| fail())? {
        return Err(fail());
    }
    let native = GraphBinding::read_context(ctx, source)?;
    let rendered = crate::security_predicate_spike::render_graph_context(ctx,source)?;
    let ontology = SecurityOntologyClosure::read(ctx.logical_plan().source(), ctx.catalog())?;
    let values = rendered.value["parameters"].as_array().ok_or_else(fail)?;
    let graph = crate::security_graph_parameters::issue_channels(
        ctx,
        &rendered.policy_source_paths,
        values,
        rendered.value["parameterTypes"]
            .as_array()
            .ok_or_else(fail)?,
        native.type_id(ontology.subject())?,
        true,
    )?;
    let inventory = ctx.parameter_sources()?;
    inventory.require_context(ctx)?;
    if graph.policy().len() + 1 + inventory.query_values().len() > 4096 {
        return Err(fail());
    }
    let mut query = Vec::new();
    let mut paths = std::collections::BTreeSet::new();
    let mut text_budget = 1_048_576usize;
    for (i, source) in inventory.query_values().iter().enumerate() {
        if source.path().len() > text_budget || !paths.insert(source.path()) {
            return Err(fail());
        }
        text_budget -= source.path().len();
        let SecurityQueryParameterValue::Scalar(value) = source.value() else {
            return Err(fail());
        };
        let value = *value;
        let logical_type = match value {
            QueryValue::Literal { logical_type, .. }
            | QueryValue::Parameter { logical_type, .. } => logical_type,
            QueryValue::Field { .. } => return Err(fail()),
        };
        let codec = match logical_type.family {
            Family::Boolean => RelationalQueryParameterCodec::Boolean,
            Family::String => RelationalQueryParameterCodec::Text,
            Family::Integer | Family::Decimal => RelationalQueryParameterCodec::Numeric,
        };
        query.push(RelationalQueryParameterBinding {
            ordinal: graph.subject().ordinal() + 1 + i,
            path: source.path().into(),
            value,
            logical_type,
            codec,
        });
    }
    Ok(RelationalGraphParameterBindings {
        context: ctx,
        graph,
        policy_values: values.clone(),
        query,
    })
}
