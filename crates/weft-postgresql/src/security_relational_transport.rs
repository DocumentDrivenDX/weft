// SPDX-License-Identifier: MIT OR Apache-2.0
//! Separate guarded relational declaration and original value/result correspondence. No native permit.
use crate::{
    security_graph_binding::GraphSource,
    security_graph_parameters::GraphPolicyParameterCodec,
    security_relational_parameters::RelationalQueryParameterCodec,
    security_relational_sql::{prepare_relational_graph_sql, RelationalGraphSql},
};
use serde_json::{json, Value};
use weft_core::{
    application_ir::Value as QueryValue,
    error::{Diagnostic, Result},
    json::{checked_json, sha256},
    security_backend::SecurityBackendContext,
    security_mapping_transport::with_security_mapping_context,
    security_source_context::SecuritySourceContext,
};
const VERSION: &str = "weft.truss.security.relational-candidate/0.2-candidate";
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-RELATIONAL-TRANSPORT",
        "host",
        "Original guarded relational correspondence refused",
    )
}
struct Buffer {
    bytes: Vec<u8>,
    remaining: usize,
}
impl std::io::Write for Buffer {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        if b.len() > self.remaining {
            return Err(std::io::Error::other("limit"));
        }
        self.remaining -= b.len();
        self.bytes.extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn encode(v: &Value) -> Result<String> {
    let mut b = Buffer {
        bytes: Vec::new(),
        remaining: 16_000_000 - 1,
    };
    serde_json::to_writer(&mut b, v).map_err(|_| fail())?;
    b.bytes.push(b'\n');
    String::from_utf8(b.bytes).map_err(|_| fail())
}
fn wire(
    ctx: &SecurityBackendContext<'_>,
    program: &RelationalGraphSql<'_, '_>,
    raw: &str,
    source: &SecuritySourceContext,
) -> Result<String> {
    let p = program.parameters();
    p.require_context(ctx)?;
    let mut parameters = Vec::new();
    for (binding, value) in p.graph().policy().iter().zip(p.policy_values()) {
        let codec = match binding.codec() {
            GraphPolicyParameterCodec::Boolean => "boolean",
            GraphPolicyParameterCodec::Text => "text",
            GraphPolicyParameterCodec::CoreLiteralJson => "coreLiteralJsonb",
        };
        parameters.push(json!({"ordinal":binding.ordinal(),"channel":"policy","sourcePath":binding.source_path(),"sourceKind":match binding.kind(){weft_core::security_parameter_sources::SecurityPolicyParameterKind::ConditionConstant=>"conditionConstant",weft_core::security_parameter_sources::SecurityPolicyParameterKind::TransformationConstant=>"transformationConstant",weft_core::security_parameter_sources::SecurityPolicyParameterKind::Context=>return Err(fail())},"field":binding.field(),"domain":binding.domain(),"codec":codec,"value":value}));
    }
    let subject = p.graph().subject();
    parameters.push(json!({"ordinal":subject.ordinal(),"channel":"subject","nativeTypeId":subject.native_type_id(),"logicalSubject":subject.subject(),"logicalKeyId":subject.key_id(),"logicalKeyFields":subject.key_fields(),"physicalType":"int8","supplier":"original_authenticated_guarded_host_only"}));
    for binding in p.query() {
        let value = match binding.value() {
            QueryValue::Literal { value, .. } | QueryValue::Parameter { value, .. } => value,
            _ => return Err(fail()),
        };
        let codec = match binding.codec() {
            RelationalQueryParameterCodec::Boolean => "boolean",
            RelationalQueryParameterCodec::Text => "text",
            RelationalQueryParameterCodec::Numeric => "numeric",
        };
        parameters.push(json!({"ordinal":binding.ordinal(),"channel":"query","sourcePath":binding.source_path(),"logicalType":binding.logical_type(),"codec":codec,"value":value}));
    }
    let contract = serde_json::to_string(program.result_contract()).map_err(|_| fail())?;
    encode(
        &json!({"version":VERSION,"securityReady":false,"sourceRequestSha256":sha256(raw.as_bytes()),"sourceContextJson":source.json(),"sourceContextSha256":source.sha256(),"queryContextSha256":program.context_sha256(),"executionSql":program.sql(),"checks":program.checks(),"operandReads":program.operand_reads(),"operandDemands":program.operand_demands(),"parameters":parameters,"resultContract":program.result_contract(),"resultContractJson":contract,"resultContractSha256":sha256(contract.as_bytes()),"nativeAdmission":"required","nativeCodecAdmission":"required","nativePopulationAndSelection":"required","releaseProtection":"required"}),
    )
}
/// Immutable original compiler handoff; no constructor/deserializer or credential.
/// ```compile_fail
/// use weft_postgresql::security_relational_transport::GuardedRelationalCandidate;
/// fn bypass(p:&mut GuardedRelationalCandidate){p.json.clear();}
/// ```
pub struct GuardedRelationalCandidate {
    json: String,
}
impl GuardedRelationalCandidate {
    pub fn json(&self) -> &str {
        &self.json
    }
}
pub fn prepare_guarded_relational_candidate(raw: &str) -> Result<GuardedRelationalCandidate> {
    if raw.len() > 4_000_000 {
        return Err(fail());
    }
    with_security_mapping_context(raw, |ctx| {
        let graph: GraphSource = serde_json::from_str(ctx.binding_json()).map_err(|_| fail())?;
        let source = ctx.source_context()?;
        source.require_sources(ctx)?;
        crate::security_native_codecs::check_postgresql_parameter_codecs(ctx,&graph)?;
        let program = prepare_relational_graph_sql(ctx, &graph)?;
        Ok(GuardedRelationalCandidate {
            json: wire(ctx, &program, raw, &source)?,
        })
    })
}
/// Rebuild exact original producer bytes and check all supplied operand/result
/// cells. This cannot prove native source/population/truth/arithmetic/authority or release.
pub fn inspect_guarded_relational_result(raw: &str) -> Result<Value> {
    if raw.len() > 32_000_000 {
        return Err(fail());
    }
    let v = checked_json(raw).map_err(|_| fail())?;
    let o = v.as_object().ok_or_else(fail)?;
    let keys = [
        "version",
        "sourceRequestJson",
        "sourceOutputJson",
        "operandBatchJson",
        "batchJson",
    ];
    if o.len() != keys.len()
        || keys.iter().any(|k| !o.contains_key(*k))
        || v["version"] != "weft.truss.security.relational-result/0.2-candidate"
    {
        return Err(fail());
    }
    let request = v["sourceRequestJson"].as_str().ok_or_else(fail)?;
    let output = v["sourceOutputJson"].as_str().ok_or_else(fail)?;
    let operands = v["operandBatchJson"].as_str().ok_or_else(fail)?;
    let batch = v["batchJson"].as_str().ok_or_else(fail)?;
    if request.len() > 4_000_000
        || output.len() > 16_000_000
        || operands.len() > 4_000_000
        || batch.len() > 4_000_000
    {
        return Err(fail());
    }
    with_security_mapping_context(request, |ctx| {
        let graph: GraphSource = serde_json::from_str(ctx.binding_json()).map_err(|_| fail())?;
        let source = ctx.source_context()?;
        source.require_sources(ctx)?;
        crate::security_native_codecs::check_postgresql_parameter_codecs(ctx,&graph)?;
        let program = prepare_relational_graph_sql(ctx, &graph)?;
        if wire(ctx, &program, request, &source)? != output {
            return Err(fail());
        }
        crate::security_native_codecs::check_postgresql_operand_codecs(ctx,&graph,operands)?;
        let checked = weft_core::security_operand_values::check_operand_values(ctx, operands)?;
        let contract = serde_json::to_string(program.result_contract()).map_err(|_| fail())?;
        let hash = sha256(contract.as_bytes());
        if ctx.query().application_plan().aggregate {
            weft_core::security_aggregate_results::check_aggregate_result_cells(
                ctx,
                program.result_contract(),
                &contract,
                &hash,
                batch,
            )?;
        } else {
            ctx.check_result_cells(program.result_contract(), &contract, &hash, batch)?;
        }
        Ok(
            json!({"version":"weft.truss.security.relational-result.inspected/0.2-candidate","securityReady":false,"inspected":true,"sourceRequestSha256":sha256(request.as_bytes()),"sourceOutputSha256":sha256(output.as_bytes()),"sourceContextSha256":source.sha256(),"queryContextSha256":program.context_sha256(),"operandBatchSha256":checked.batch_sha256(),"operandCounts":checked.counts(),"batchSha256":sha256(batch.as_bytes()),"nativeAdmission":"required"}),
        )
    })
}

/// Separate authored relationship producer; historical relational entrypoints stay closed.
pub fn prepare_guarded_relationship_candidate(raw: &str) -> Result<GuardedRelationalCandidate> {
    if raw.len() > 4_000_000 {
        return Err(fail());
    }
    weft_core::security_mapping_transport::with_security_relationship_context(raw, |ctx| {
        let graph: GraphSource = serde_json::from_str(ctx.binding_json()).map_err(|_| fail())?;
        let source = ctx.source_context()?;
        source.require_sources(ctx)?;
        crate::security_native_codecs::check_postgresql_parameter_codecs(ctx,&graph)?;
        let program = prepare_relational_graph_sql(ctx, &graph)?;
        Ok(GuardedRelationalCandidate {
            json: wire(ctx, &program, raw, &source)?,
        })
    })
}

/// Separate original relationship result inspection, never native capture or release authority.
pub fn inspect_guarded_relationship_result(raw: &str) -> Result<Value> {
    if raw.len() > 32_000_000 {
        return Err(fail());
    }
    let v = checked_json(raw).map_err(|_| fail())?;
    let o = v.as_object().ok_or_else(fail)?;
    let keys = [
        "version",
        "sourceRequestJson",
        "sourceOutputJson",
        "operandBatchJson",
        "batchJson",
    ];
    if o.len() != keys.len()
        || keys.iter().any(|k| !o.contains_key(*k))
        || v["version"] != "weft.truss.security.relational-result/0.2-candidate"
    {
        return Err(fail());
    }
    let request = v["sourceRequestJson"].as_str().ok_or_else(fail)?;
    let output = v["sourceOutputJson"].as_str().ok_or_else(fail)?;
    let operands = v["operandBatchJson"].as_str().ok_or_else(fail)?;
    let batch = v["batchJson"].as_str().ok_or_else(fail)?;
    if request.len() > 4_000_000
        || output.len() > 16_000_000
        || operands.len() > 4_000_000
        || batch.len() > 4_000_000
    {
        return Err(fail());
    }
    weft_core::security_mapping_transport::with_security_relationship_context(request, |ctx| {
        let graph: GraphSource = serde_json::from_str(ctx.binding_json()).map_err(|_| fail())?;
        let source = ctx.source_context()?;
        source.require_sources(ctx)?;
        crate::security_native_codecs::check_postgresql_parameter_codecs(ctx,&graph)?;
        let program = prepare_relational_graph_sql(ctx, &graph)?;
        if wire(ctx, &program, request, &source)? != output {
            return Err(fail());
        }
        crate::security_native_codecs::check_postgresql_operand_codecs(ctx,&graph,operands)?;
        let checked = weft_core::security_operand_values::check_operand_values(ctx, operands)?;
        let contract = serde_json::to_string(program.result_contract()).map_err(|_| fail())?;
        let hash = sha256(contract.as_bytes());
        if ctx.query().application_plan().aggregate {
            weft_core::security_aggregate_results::check_aggregate_result_cells(
                ctx,
                program.result_contract(),
                &contract,
                &hash,
                batch,
            )?;
        } else {
            ctx.check_result_cells(program.result_contract(), &contract, &hash, batch)?;
        }
        Ok(
            json!({"version":"weft.truss.security.relational-result.inspected/0.2-candidate","securityReady":false,"inspected":true,"sourceRequestSha256":sha256(request.as_bytes()),"sourceOutputSha256":sha256(output.as_bytes()),"sourceContextSha256":source.sha256(),"queryContextSha256":program.context_sha256(),"operandBatchSha256":checked.batch_sha256(),"operandCounts":checked.counts(),"batchSha256":sha256(batch.as_bytes()),"nativeAdmission":"required"}),
        )
    })
}
