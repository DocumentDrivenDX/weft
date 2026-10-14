//! Immutable original source capsule, separate from query lineage and native authority.
use crate::{
    error::{Diagnostic, Result},
    model::ModuleInput,
    security_backend::SecurityBackendContext,
};
use serde::Serialize;
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-SOURCE-CONTEXT",
        "capability",
        "Original source context correspondence refused",
    )
}
pub const MAX_SOURCE_CONTEXT_BYTES: usize = 16_000_000;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Capsule<'a> {
    version: &'static str,
    security_ready: bool,
    modules: &'a [ModuleInput],
    policy_json: &'a str,
    ontology_json: &'a str,
    query_profile_json: &'a str,
    binding_json: &'a str,
    backend_id: &'a str,
    backend_version: &'a str,
    target_profile: &'a str,
}
/// Exact source bytes only. No public constructor/deserializer, principal,
/// native authority generation, SQL, parameter values or result contract.
/// ```compile_fail
/// use weft_core::security_source_context::SecuritySourceContext;
/// fn substitute(s: &mut SecuritySourceContext) { s.json = "{}".into(); }
/// ```
pub struct SecuritySourceContext {
    json: String,
    sha256: String,
}
impl SecuritySourceContext {
    pub fn json(&self) -> &str {
        &self.json
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    /// Pure original byte correspondence; success authenticates no native source.
    /// Different queries may share these sources, but must have their own query
    /// context, result lineage and genuinely issued native admission.
    pub fn require_sources(&self, context: &SecurityBackendContext<'_>) -> Result<()> {
        if self.json != encode(context, MAX_SOURCE_CONTEXT_BYTES)? {
            return Err(fail());
        }
        Ok(())
    }
}
struct BoundedBuffer {
    bytes: Vec<u8>,
    remaining: usize,
}
impl std::io::Write for BoundedBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.remaining {
            return Err(std::io::Error::other("Source context limit"));
        }
        self.remaining -= bytes.len();
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn encode(context: &SecurityBackendContext<'_>, limit: usize) -> Result<String> {
    context.profiled_query().require_sources(
        context.logical_plan(),
        context.catalog(),
        context.binding_json(),
        context.backend_id(),
        context.backend_version(),
        context.target_profile(),
    )?;
    let capsule = Capsule {
        version: "weft.security.source-context/0.1-candidate",
        security_ready: false,
        modules: context.catalog().inputs(),
        policy_json: context.logical_plan().source().policy_json(),
        ontology_json: context.logical_plan().source().ontology_json(),
        query_profile_json: context.profiled_query().profile_source_json(),
        binding_json: context.binding_json(),
        backend_id: context.backend_id(),
        backend_version: context.backend_version(),
        target_profile: context.target_profile(),
    };
    let mut writer = BoundedBuffer {
        bytes: Vec::new(),
        remaining: limit,
    };
    serde_json::to_writer(&mut writer, &capsule).map_err(|_| fail())?;
    String::from_utf8(writer.bytes).map_err(|_| fail())
}
pub(crate) fn issue(context: &SecurityBackendContext<'_>) -> Result<SecuritySourceContext> {
    let json = encode(context, MAX_SOURCE_CONTEXT_BYTES)?;
    let sha256 = crate::json::sha256(json.as_bytes());
    Ok(SecuritySourceContext { json, sha256 })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[jsonschema::validator(
        path = "../../docs/helix/02-design/contracts/security-source-context-v0.1-candidate.schema.json"
    )]
    struct SourceContextSchema;
    #[test]
    fn exact_and_over_source_serialization_bounds_refuse_without_partial_capsule() {
        let raw = include_str!(
            "../../../docs/helix/03-test/fixtures/security-graph-source/mapping-request.json"
        );
        crate::security_mapping_transport::with_security_mapping_context(raw, |ctx| {
            let original = encode(ctx, MAX_SOURCE_CONTEXT_BYTES)?;
            assert!(SourceContextSchema::is_valid(
                &serde_json::from_str::<serde_json::Value>(&original).unwrap()
            ));
            let value = serde_json::from_str::<serde_json::Value>(&original).unwrap();
            for field in ["sql", "principal", "epoch", "permit"] {
                let mut extra = value.clone();
                extra[field] = serde_json::json!("forged");
                assert!(!SourceContextSchema::is_valid(&extra));
            }
            let mut ready = value.clone();
            ready["securityReady"] = serde_json::json!(true);
            assert!(!SourceContextSchema::is_valid(&ready));
            let mut wrong = value.clone();
            wrong["version"] = serde_json::json!("weft.security.source-context/0.2");
            assert!(!SourceContextSchema::is_valid(&wrong));
            assert_eq!(encode(ctx, original.len())?, original);
            for limit in [0, 1, original.len() - 1] {
                assert!(encode(ctx, limit).is_err());
            }
            Ok(())
        })
        .unwrap();
    }
}
