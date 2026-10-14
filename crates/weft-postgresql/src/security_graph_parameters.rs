//! Compiler-owned graph ordinal/source correspondence, not native authority.
use serde_json::Value;
use weft_core::{
    error::{Diagnostic, Result},
    security_backend::SecurityBackendContext,
    security_ontology::SecurityRef,
    security_parameter_sources::SecurityPolicyParameterKind,
};
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-GRAPH-PARAMETER",
        "capability",
        "Original graph parameter correspondence refused",
    )
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphPolicyParameterCodec {
    Boolean,
    Text,
    CoreLiteralJson,
}
/// Checked metadata for an actual immutable graph candidate parameter ordinal.
/// No constructor or deserializer. This is not an issuer or execution credential.
pub struct GraphPolicyParameterBinding {
    ordinal: usize,
    source_path: String,
    kind: SecurityPolicyParameterKind,
    field: SecurityRef,
    domain: Value,
    codec: GraphPolicyParameterCodec,
}
impl GraphPolicyParameterBinding {
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn source_path(&self) -> &str {
        &self.source_path
    }
    pub fn kind(&self) -> SecurityPolicyParameterKind {
        self.kind
    }
    pub fn field(&self) -> &SecurityRef {
        &self.field
    }
    pub fn domain(&self) -> &Value {
        &self.domain
    }
    pub fn codec(&self) -> GraphPolicyParameterCodec {
        self.codec
    }
}
/// A native subject locator demand, deliberately without a caller-supplied value.
pub struct GraphSubjectParameterBinding {
    ordinal: usize,
    native_type_id: i64,
    subject: SecurityRef,
    key_id: String,
    key_fields: Vec<SecurityRef>,
}
impl GraphSubjectParameterBinding {
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn native_type_id(&self) -> i64 {
        self.native_type_id
    }
    pub fn subject(&self) -> &SecurityRef {
        &self.subject
    }
    pub fn key_id(&self) -> &str {
        &self.key_id
    }
    pub fn key_fields(&self) -> &[SecurityRef] {
        &self.key_fields
    }
    pub fn physical_type(&self) -> &str {
        "int8"
    }
}
/// Original policy and native-subject channels remain distinct. No mutation,
/// deserialization or native locator binding/release permission is exposed.
/// ```compile_fail
/// use weft_postgresql::security_graph_parameters::GraphParameterBindings;
/// fn replace(p: &mut GraphParameterBindings) { p.policy.clear(); }
/// ```
pub struct GraphParameterBindings {
    policy: Vec<GraphPolicyParameterBinding>,
    subject: GraphSubjectParameterBinding,
}
impl GraphParameterBindings {
    pub fn policy(&self) -> &[GraphPolicyParameterBinding] {
        &self.policy
    }
    pub fn subject(&self) -> &GraphSubjectParameterBinding {
        &self.subject
    }
}
/// Count serialized source metadata before cloning. This is a finite metadata
/// ledger, not a theorem about total heap/CPU or native codec execution.
struct MetadataBudget(usize);
impl std::io::Write for MetadataBudget {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.0 {
            return Err(std::io::Error::other("Source metadata limit"));
        }
        self.0 -= bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl MetadataBudget {
    fn charge<T: serde::Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        serde_json::to_writer(self, value).map_err(|_| fail())
    }
}
pub(crate) fn issue(
    ctx: &SecurityBackendContext<'_>,
    paths: &[String],
    parameters: &[Value],
    types: &[Value],
    native_type_id: i64,
) -> Result<GraphParameterBindings> {
    issue_channels(ctx, paths, parameters, types, native_type_id, false)
}
pub(crate) fn issue_channels(
    ctx: &SecurityBackendContext<'_>, paths: &[String], parameters: &[Value],
    types: &[Value], native_type_id: i64, include_query: bool,
) -> Result<GraphParameterBindings> {
    if parameters.len() != paths.len()
        || parameters.len() != types.len()
        || parameters.len() > 1024
        || !(1..=i32::MAX as i64).contains(&native_type_id)
    {
        return Err(fail());
    }
    let inventory = ctx.parameter_sources()?;
    inventory.require_context(ctx)?;
    // Legacy issue() still refuses every query value. The separate relational
    // issuer must retain all of them in its own original query channel.
    if !include_query && !inventory.query_values().is_empty() {
        return Err(fail());
    }
    let mut sources = std::collections::BTreeMap::new();
    for source in inventory.policy_operands() {
        if sources.insert(source.path(), source).is_some() {
            return Err(fail());
        }
    }
    let mut budget = MetadataBudget(1_048_576);
    let mut policy = Vec::new();
    for (i, ((path, value), physical)) in paths.iter().zip(parameters).zip(types).enumerate() {
        let source = sources.get(path.as_str()).ok_or_else(fail)?;
        let literal = source.literal().ok_or_else(fail)?;
        let codec = match source.kind() {
            SecurityPolicyParameterKind::ConditionConstant => {
                let (codec, family, physical_type) = match source.domain()["scalarType"].as_str() {
                    Some("boolean") => (GraphPolicyParameterCodec::Boolean, "boolean", "boolean"),
                    Some("string") => (GraphPolicyParameterCodec::Text, "string", "text"),
                    _ => return Err(fail()),
                };
                let expected = if literal.is_null() {
                    literal
                } else {
                    literal.get(family).ok_or_else(fail)?
                };
                if physical != physical_type || value != expected {
                    return Err(fail());
                }
                codec
            }
            SecurityPolicyParameterKind::TransformationConstant => {
                if physical != "jsonb" || value != literal {
                    return Err(fail());
                }
                GraphPolicyParameterCodec::CoreLiteralJson
            }
            SecurityPolicyParameterKind::Context => return Err(fail()),
        };
        budget.charge(path)?;
        budget.charge(source.field())?;
        budget.charge(source.domain())?;
        policy.push(GraphPolicyParameterBinding {
            ordinal: i + 1,
            source_path: path.clone(),
            kind: source.kind(),
            field: source.field().clone(),
            domain: source.domain().clone(),
            codec,
        });
    }
    let original = inventory.subject();
    budget.charge(original.subject())?;
    budget.charge(original.key_id())?;
    budget.charge(original.key_fields())?;
    let subject = GraphSubjectParameterBinding {
        ordinal: parameters.len() + 1,
        native_type_id,
        subject: original.subject().clone(),
        key_id: original.key_id().into(),
        key_fields: original.key_fields().to_vec(),
    };
    Ok(GraphParameterBindings { policy, subject })
}
#[cfg(test)]
mod tests {
    use super::*;
    const REQUEST: &str = include_str!(
        "../../../docs/helix/03-test/fixtures/security-graph-source/mapping-request.json"
    );
    #[test]
    fn original_condition_ordinal_refuses_source_value_codec_and_shape_substitution() {
        weft_core::security_mapping_transport::with_security_mapping_context(REQUEST, |ctx| {
            let paths = vec!["logicalRules/1/condition/where/args/1/where/args/2/right".into()];
            let values = vec![Value::Bool(true)];
            let types = vec![Value::String("boolean".into())];
            let bindings = issue(ctx, &paths, &values, &types, 6)?;
            assert_eq!(bindings.policy()[0].ordinal(), 1);
            assert_eq!(bindings.subject().ordinal(), 2);
            for mutation in 0..7 {
                let mut p = paths.clone();
                let mut v = values.clone();
                let mut t = types.clone();
                let mut native = 6;
                match mutation {
                    0 => p[0] = "logicalRules/1/condition/where/args/1/where/args/2/left".into(),
                    1 => v[0] = Value::Bool(false),
                    2 => t[0] = Value::String("jsonb".into()),
                    3 => p.clear(),
                    4 => v.clear(),
                    5 => t.clear(),
                    _ => native = 0,
                };
                assert!(
                    issue(ctx, &p, &v, &t, native).is_err(),
                    "mutation {mutation}"
                );
            }
            Ok(())
        })
        .unwrap();
    }
    #[test]
    fn metadata_ledger_refuses_before_retaining_a_large_clone() {
        let mut budget = MetadataBudget(2);
        assert!(budget.charge("text").is_err());
        let mut budget = MetadataBudget(2);
        budget
            .charge(&false)
            .expect_err("serialized bool must exceed two-byte budget");
    }
}
