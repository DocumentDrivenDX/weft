// SPDX-License-Identifier: MIT OR Apache-2.0
//! Original guarded graph handoff and result correspondence. No native admission or release permit.
use crate::{
    security_graph_binding::GraphSource,
    security_graph_parameters::{GraphParameterBindings, GraphPolicyParameterCodec},
    security_graph_query::{prepare_graph_query_checked, GraphQueryCandidate},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use weft_core::{
    error::{Diagnostic, Result},
    json::{checked_json, sha256},
    security_backend::SecurityBackendContext,
    security_mapping_transport::with_security_mapping_context,
    security_ontology::SecurityRef,
    security_parameter_sources::SecurityPolicyParameterKind,
    security_source_context::SecuritySourceContext,
};
const MAX_OUTPUT: usize = 16_000_000;
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-GUARDED-TRANSPORT",
        "host",
        "Original guarded graph correspondence refused",
    )
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PolicyWire<'a> {
    ordinal: usize,
    source_path: &'a str,
    source_kind: &'static str,
    field: &'a SecurityRef,
    domain: &'a Value,
    physical_codec: &'static str,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubjectWire<'a> {
    ordinal: usize,
    native_type_id: i64,
    logical_subject: &'a SecurityRef,
    logical_key_id: &'a str,
    logical_key_fields: &'a [SecurityRef],
    physical_type: &'a str,
    supplier: &'static str,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Wire<'a> {
    version: &'static str,
    security_ready: bool,
    source_request_sha256: &'a str,
    source_context_json: &'a str,
    source_context_sha256: &'a str,
    query_context_sha256: &'a str,
    query: &'a Value,
    policy_parameter_bindings: Vec<PolicyWire<'a>>,
    subject_parameter_binding: SubjectWire<'a>,
    native_admission: &'static str,
}
fn bindings(parameters: &GraphParameterBindings) -> Result<(Vec<PolicyWire<'_>>, SubjectWire<'_>)> {
    let policy = parameters
        .policy()
        .iter()
        .map(|p| {
            Ok(PolicyWire {
                ordinal: p.ordinal(),
                source_path: p.source_path(),
                source_kind: match p.kind() {
                    SecurityPolicyParameterKind::ConditionConstant => "conditionConstant",
                    SecurityPolicyParameterKind::TransformationConstant => "transformationConstant",
                    SecurityPolicyParameterKind::Context => return Err(fail()),
                },
                field: p.field(),
                domain: p.domain(),
                physical_codec: match p.codec() {
                    GraphPolicyParameterCodec::Boolean => "boolean",
                    GraphPolicyParameterCodec::Text => "text",
                    GraphPolicyParameterCodec::CoreLiteralJson => "coreLiteralJsonb",
                },
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let s = parameters.subject();
    Ok((
        policy,
        SubjectWire {
            ordinal: s.ordinal(),
            native_type_id: s.native_type_id(),
            logical_subject: s.subject(),
            logical_key_id: s.key_id(),
            logical_key_fields: s.key_fields(),
            physical_type: s.physical_type(),
            supplier: "original_authenticated_guarded_host_only",
        },
    ))
}
struct Buffer {
    bytes: Vec<u8>,
    remaining: usize,
}
impl std::io::Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.remaining {
            return Err(std::io::Error::other("Guarded transport limit"));
        }
        self.remaining -= bytes.len();
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
/// Immutable original request/source/query/parameter/result declarations. This
/// checked value contains no caller or native subject value and grants no permit.
/// ```compile_fail
/// use weft_postgresql::security_guarded_transport::GuardedGraphCandidate;
/// fn replace(c: &mut GuardedGraphCandidate) { c.json = "{}".into(); }
/// ```
pub struct GuardedGraphCandidate {
    json: String,
    source_request_sha256: String,
    source: SecuritySourceContext,
    query_context_sha256: String,
    query: GraphQueryCandidate,
}
impl GuardedGraphCandidate {
    /// Exact canonical producer output INCLUDING its final newline.
    pub fn json(&self) -> &str {
        &self.json
    }
    pub fn source(&self) -> &SecuritySourceContext {
        &self.source
    }
    pub fn query_context_sha256(&self) -> &str {
        &self.query_context_sha256
    }
    pub fn query(&self) -> &GraphQueryCandidate {
        &self.query
    }
    fn check_result(
        &self,
        context: &SecurityBackendContext<'_>,
        source_output: &str,
        batch: &str,
    ) -> Result<()> {
        self.source.require_sources(context)?;
        if self.query_context_sha256 != context.execution_query_context_sha256()
            || source_output != self.json
        {
            return Err(fail());
        }
        let q = self.query.value();
        context.check_result_cells(
            self.query.result_contract(),
            q["resultContractJson"].as_str().ok_or_else(fail)?,
            q["resultContractSha256"].as_str().ok_or_else(fail)?,
            batch,
        )
    }
}
/// Source-only candidate preparation via the original closed mapping request.
/// Native source/caller/current-authority admission and guarded release remain required.
pub fn prepare_guarded_graph_candidate(raw: &str) -> Result<GuardedGraphCandidate> {
    if raw.len() > 4_000_000 {
        return Err(fail());
    }
    let request = checked_json(raw).map_err(|_| fail())?;
    if request["interfaceVersion"] != "weft-security-mapping/0.1.0" {
        return Err(fail());
    }
    let binding = request["bindingJson"].as_str().ok_or_else(fail)?;
    let graph_source: GraphSource =
        serde_json::from_value(checked_json(binding).map_err(|_| fail())?).map_err(|_| fail())?;
    with_security_mapping_context(raw, |context| {
        let source = context.source_context()?;
        let query = prepare_graph_query_checked(context, &graph_source)?;
        let source_request_sha256 = sha256(raw.as_bytes());
        let query_context_sha256 = context.execution_query_context_sha256();
        let (policy, subject) = bindings(query.parameter_bindings())?;
        let wire = Wire {
            version: "weft.truss.security.guarded-candidate/0.1-candidate",
            security_ready: false,
            source_request_sha256: &source_request_sha256,
            source_context_json: source.json(),
            source_context_sha256: source.sha256(),
            query_context_sha256: &query_context_sha256,
            query: query.value(),
            policy_parameter_bindings: policy,
            subject_parameter_binding: subject,
            native_admission: "required",
        };
        let mut buffer = Buffer {
            bytes: Vec::new(),
            remaining: MAX_OUTPUT - 1,
        };
        serde_json::to_writer(&mut buffer, &wire).map_err(|_| fail())?;
        buffer.bytes.push(b'\n');
        let json = String::from_utf8(buffer.bytes).map_err(|_| fail())?;
        Ok(GuardedGraphCandidate {
            json,
            source_request_sha256,
            source,
            query_context_sha256,
            query,
        })
    })
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ResultInput {
    version: String,
    source_request_json: String,
    source_output_json: String,
    batch_json: String,
}
/// Regenerate original producer output and check complete original-owner result
/// cells. Does not authenticate native rows, authorize selection or publish data.
pub fn inspect_guarded_graph_result(raw: &str) -> Result<String> {
    if raw.len() > 32_000_000 {
        return Err(fail());
    }
    let value=checked_json(raw).map_err(|_|fail())?;
    if !value.is_object(){return Err(fail());}
    let input: ResultInput = serde_json::from_value(value).map_err(|_|fail())?;
    if input.version != "weft.truss.security.guarded-result/0.1-candidate"
        || input.source_output_json.len() > MAX_OUTPUT
        || input.batch_json.len() > 4_000_000
    {
        return Err(fail());
    }
    let candidate = prepare_guarded_graph_candidate(&input.source_request_json)?;
    with_security_mapping_context(&input.source_request_json, |context| {
        candidate.check_result(context, &input.source_output_json, &input.batch_json)
    })?;
    Ok(json!({"version":"weft.truss.security.guarded-result/0.1-candidate","status":"inspected","securityReady":false,"sourceRequestSha256":candidate.source_request_sha256,"sourceOutputSha256":sha256(input.source_output_json.as_bytes()),"sourceContextSha256":candidate.source.sha256(),"queryContextSha256":candidate.query_context_sha256,"batchSha256":sha256(input.batch_json.as_bytes())}).to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_writer_refuses_before_retaining_an_overrun_chunk() {
        use std::io::Write;
        let mut b = Buffer {
            bytes: Vec::new(),
            remaining: 2,
        };
        b.write_all(b"ab").unwrap();
        assert!(b.write_all(b"c").is_err());
        assert_eq!(b.bytes, b"ab");
    }
}
