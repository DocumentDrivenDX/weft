//! Owned declarations returned by trusted security backends. Not admitted artifacts.
use serde::Serialize;
use serde_json::Value;
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityIdentity {
    pub document_id: String,
    pub revision: String,
    pub module: String,
    pub element: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SecurityResultDomain {
    Scalar { r#type: crate::ir::LogicalType },
    Model { field: SecurityIdentity },
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityDispositionSource {
    pub rule_id: String,
    pub target: SecurityIdentity,
    pub field: SecurityIdentity,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum SecurityTransform {
    Constant { version: String, output_field: SecurityIdentity, literal: Value },
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "disposition", rename_all = "lowercase", rename_all_fields = "camelCase")]
pub enum SecurityResultOutcome {
    Original { id: String, domain: SecurityResultDomain },
    Transformed { id: String, transform: SecurityTransform, disposition_sources: Vec<SecurityDispositionSource>, domain: SecurityResultDomain },
    Withheld { id: String },
    Absent { id: String },
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityResultColumn {
    pub position: usize,
    pub output_name: String,
    pub source_fields: Vec<SecurityIdentity>,
    pub outcomes: Vec<SecurityResultOutcome>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityResultContract {
    pub version: String,
    pub encoding: String,
    pub columns: Vec<SecurityResultColumn>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityOperation {
    pub id: String,
    pub sql: String,
    pub parameters: Vec<crate::backend::ParameterSlot>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPhysicalPlan {
    pub version: String,
    pub installation: Vec<SecurityOperation>,
    pub execution: SecurityOperation,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SecurityEnforcementSite { Compiler, Backend, Host, Native }
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityAdmissionObligation {
    pub id: String,
    pub semantic_sources: Vec<String>,
    pub enforcement_site: SecurityEnforcementSite,
    pub prerequisites: Vec<String>,
    pub failure_code: String,
    pub evidence_case_ids: Vec<String>,
}
/// Public construction is declaration only. Compiler-owned coverage admission
/// is required before these values may appear in a compiled response.
#[derive(Debug, Clone)]
pub struct SecurityLowering {
    pub lowering: SecurityPhysicalPlan,
    pub result_contract: SecurityResultContract,
    pub obligations: Vec<SecurityAdmissionObligation>,
    pub capability_ids: Vec<String>,
}
