//! Explicit backend interface 0.3 for owned 0.4 plans; no Backend02 widening.
#[path = "backend03_emission.rs"]
mod emission;
#[path = "backend03_manifest.rs"]
mod manifest;
#[path = "backend03_registry.rs"]
mod registry;
#[path = "backend03_view.rs"]
mod view;
pub use crate::backend::{
    Assessment, BindingInput, Capability, LanguageProfile, Obligation, ParameterSlot, Selection,
    Status, Target, TargetProfile,
};
use crate::{
    application_model::{AuthoredKey, RelationshipRead},
    error::{Diagnostic, Result},
    ir::{Identity, LogicalType},
    model::Catalog,
};
pub(crate) use emission::validate_response_schema;
pub use manifest::{validate_manifest_json, Manifest};
pub use registry::Registry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub use view::{ExpansionView, ExpressionView, HavingView, OutputView, PathView, Plan04View};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PinnedTable {
    pub name: [String; 3],
    pub uuid: String,
    pub version: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeEdgeSource {
    pub relationship: crate::application_model::RelationshipIdentity,
    pub table: PinnedTable,
    pub identity_column: String,
    pub native_type: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeRecordSource {
    pub scan: String,
    pub record: Identity,
    pub table: PinnedTable,
    pub identity_column: String,
    pub native_type: String,
}
pub struct Validated<T> {
    pub mapping: T,
    pub additional_capabilities: Vec<String>,
    pub coverage: Selection,
    pub obligations: Vec<Obligation>,
    pub edge_sources: Vec<NativeEdgeSource>,
    pub record_sources: Vec<NativeRecordSource>,
}
pub struct Context<'a> {
    pub catalog: &'a Catalog,
    pub plan: Plan04View<'a>,
    pub target: &'a TargetProfile,
    pub binding: &'a BindingInput,
    pub binding_value: &'a Value,
    pub selection: &'a Selection,
}
pub trait Backend: Send + Sync + 'static {
    type Mapping: Send + Sync;
    type TargetPlan: Send + Sync;
    fn describe(&self) -> Result<Manifest>;
    fn validate_binding(&self, context: &Context<'_>) -> Result<Validated<Self::Mapping>>;
    fn assess(&self, context: &Context<'_>, mapping: &Self::Mapping) -> Result<Vec<Assessment>>;
    fn lower(&self, context: &Context<'_>, mapping: &Self::Mapping) -> Result<Self::TargetPlan>;
    fn emit(&self, context: &Context<'_>, plan: &Self::TargetPlan) -> Result<Emission>;
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WirePath {
    pub start_scan: String,
    pub hops: [RelationshipRead; 2],
    pub span: crate::ir::Span,
    pub hop_spans: [crate::ir::Span; 2],
}
impl From<PathView<'_>> for WirePath {
    fn from(p: PathView<'_>) -> Self {
        Self {
            start_scan: p.start_scan().into(),
            hops: p.hops().clone(),
            span: p.span().clone(),
            hop_spans: p.hop_spans().clone(),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathTarget {
    pub path_occurrence: String,
    pub record: Identity,
    pub key: AuthoredKey,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Representation {
    Scalar {
        #[serde(rename = "logicalType")]
        logical_type: LogicalType,
        carrier: crate::backend::ScalarCarrier,
        decoder: crate::backend::ScalarDecoder,
        #[serde(rename = "pathTarget", skip_serializing_if = "Option::is_none")]
        path_target: Option<PathTarget>,
    },
    RelatedPaths {
        path: WirePath,
        #[serde(rename = "startRecord")]
        start_record: Identity,
        bound: u16,
        #[serde(rename = "edgeEncoding")]
        edge_encoding: String,
        #[serde(rename = "outerJoin", skip_serializing_if = "Option::is_none")]
        outer_join: Option<crate::backend::OuterJoin>,
    },
    #[serde(untagged)]
    Legacy(crate::backend::Representation),
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    pub position: usize,
    pub output_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carrier_name: Option<String>,
    pub representation: Representation,
    pub source_identities: Vec<Identity>,
    pub nullable: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct Emission {
    pub sql: String,
    pub parameters: Vec<ParameterSlot>,
    pub columns: Vec<Column>,
    pub obligations: Vec<Obligation>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Compilation {
    pub backend_id: String,
    pub backend_version: String,
    pub target_profile: TargetProfile,
    pub binding_profile: String,
    pub binding_sha256: String,
    pub qualifications: Vec<crate::backend::Qualification>,
    pub emission: Emission,
}
pub(super) fn fail(code: &str, message: &str) -> Diagnostic {
    Diagnostic::new(code, "capability", message)
}
pub(super) fn unique(v: &[String]) -> bool {
    let mut s = std::collections::BTreeSet::new();
    v.iter()
        .all(|x| !x.is_empty() && !x.contains('\0') && s.insert(x))
}

/// Serialize trusted backend metadata without first allocating an unbounded JSON string.
fn bounded_json<T: Serialize>(value: &T, limit: usize) -> Result<Value> {
    struct Buffer {
        bytes: Vec<u8>,
        limit: usize,
    }
    impl std::io::Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
                return Err(std::io::Error::other("JSON bound exceeded"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut buffer = Buffer {
        bytes: Vec::new(),
        limit,
    };
    serde_json::to_writer(&mut buffer, value).map_err(|_| {
        fail(
            "WFT-LIMIT",
            "Backend metadata exceeds its bounded JSON envelope",
        )
    })?;
    serde_json::from_slice(&buffer.bytes)
        .map_err(|_| fail("WFT-BACKEND-VERSION", "Invalid serialized backend metadata"))
}

#[cfg(test)]
#[path = "backend03_tests.rs"]
pub(crate) mod tests;

fn valid_obligation(o: &Obligation) -> bool {
    !o.id.is_empty()
        && !o.id.contains('\0')
        && o.parameters.is_object()
        && o.failure_code.starts_with("WFT-")
        && o.failure_code.len() > 4
        && o.failure_code[4..]
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-')
}
