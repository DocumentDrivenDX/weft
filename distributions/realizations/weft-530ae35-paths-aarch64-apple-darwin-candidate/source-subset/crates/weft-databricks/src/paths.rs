//! Explicit Backend03 candidate. Compiler fixtures do not qualify native execution.
use crate::binding::{self, Binding, RecordKind};
use serde_json::json;
use weft_core::{
    backend03::*,
    error::{Diagnostic, Result},
};

pub const ID: &str = "ashlar.databricks.paths";
pub const VERSION: &str = "0.4.0-paths-candidate";
pub const PROFILE: &str = "spark4-delta4-paths-candidate";
pub struct Paths;
pub struct PathTargetPlan(pub(crate) Emission);

fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "lower", message)
}
fn table(t: &binding::Table) -> PinnedTable {
    PinnedTable {
        name: t.name.clone(),
        uuid: t.uuid.clone(),
        version: t.version,
    }
}
impl Backend for Paths {
    type Mapping = Binding;
    type TargetPlan = PathTargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let languages = vec![LanguageProfile {
            dialect_profile: "weft-sql/0.4.0".into(),
            ir_version: "weft-ir/0.4.0".into(),
        }];
        let capabilities = [
            "scan",
            "project",
            "filter",
            "innerJoin",
            "equal",
            "and",
            "parameter.named",
            "compare.lexicographicGreater",
            "order.asc",
            "limit",
            "type.string",
            "type.boolean",
            "type.integer",
            "type.decimal",
            "value.presence",
            "value.nativeNull",
            "predicate.nativeNull",
            "compare.nullAwareStringEqual",
            "project.positionedOutputs",
            "project.distinct",
            "compare.less",
            "compare.lessEqual",
            "compare.greaterEqual",
            "compare.notEqual",
            "compare.scalarJoin",
            "arithmetic.exact.integer",
            "arithmetic.exact.decimal",
            "arithmetic.+",
            "arithmetic.-",
            "arithmetic.*",
            "arithmetic.negate",
            "arithmetic.compareExact",
            "type.integer.unbounded",
            "aggregate",
            "group",
            "aggregate.count",
            "aggregate.countDistinct",
            "aggregate.countDistinct.optional",
            "aggregate.havingCountDistinctGreater",
            "predicate.stringIn",
            "join.left",
            "value.outerJoinPresence",
            "relationship.twoHopPaths",
            "relationship.pathExpansion",
            "relationship.inverse",
            "result.pathOccurrences",
            "aggregate.pathTargetDistinctCount",
        ];
        Ok(Manifest{
            backend_id:ID.into(),backend_version:VERSION.into(),interface_version:"weft-backend/0.3.0".into(),
            language_profiles:languages.clone(),binding_profile:binding::PROFILE.into(),
            target_profiles:vec![TargetProfile{id:PROFILE.into(),engine:"spark-sql".into(),engine_version:"4.0.1".into(),session_settings:json!({"comparison":"UTF8_BINARY","spark.sql.ansi.enabled":true,"spark.sql.session.timeZone":"UTC","pathEdgeEncoding":"signed64-decimal/0.1","pathCountRepresentation":"signed64 with independent DECIMAL38 capacity","sourceValidity":"original public UMF independently; finite native capacity is separate"}),storage_layout_revision:"ashlar-delta/0.3".into(),publication_revision:"ashlar-resolver/0.1-candidate".into()}],
            capabilities:capabilities.into_iter().map(|id|Capability{id:id.into(),target_profiles:vec![PROFILE.into()],language_profiles:languages.clone(),logical_domain:json!({"subset":"original authored two-hop occurrences and scalar relational inputs; exact non-LEFT arithmetic; required String groups and legacy String count-distinct/HAVING; String LEFT inputs; no SUM/compound/read-profile widening"}),result_domain:json!({"path":"complete parallel edge-pair bag; typed intermediate/terminal key order then signed native BIGINT edge order","collection":"bounded ordered exact String-key/token carrier with full-population prefix proof","counts":"mathematical Integer; independently wide complete pre-HAVING/ORDER/LIMIT capacity or refusal"}),constraints:vec!["Native/schema/policy/publication checks remain mandatory before any buffered result is released".into(),"Candidate compiler metadata is not native qualification; unmatched LEFT numeric consumption, grouped/ordered absence carriers and optional/LEFT DISTINCT refuse".into()],obligations:vec![],status:Status::Candidate,evidence:vec![]}).collect(),evidence:vec![]
        })
    }
    fn validate_binding(&self, c: &Context<'_>) -> Result<Validated<Binding>> {
        if c.target.id != PROFILE {
            return Err(fail("Explicit path target required"));
        }
        let b = binding::admit03(c.catalog, c.binding_value)?;
        for id in &c.selection.records {
            if !b.records.iter().any(|r| &r.logical == id) {
                return Err(fail("Selected original Record lacks mapping"));
            }
        }
        for id in &c.selection.fields {
            if !b
                .records
                .iter()
                .any(|r| r.properties.iter().any(|p| &p.logical == id))
            {
                return Err(fail("Selected original Field lacks mapping"));
            }
        }
        let mut edges = vec![];
        for id in &c.selection.relationships {
            let r = b
                .relationships
                .iter()
                .find(|r| r.logical == json!(id))
                .ok_or_else(|| fail("Selected original relationship lacks mapping"))?;
            if r.kind != RecordKind::Edge {
                return Err(fail(
                    "Initial path profile requires original edge-current BIGINT identities",
                ));
            }
            edges.push(NativeEdgeSource {
                relationship: id.clone(),
                table: table(&b.publication.tables[r.table]),
                identity_column: "id".into(),
                native_type: "BIGINT".into(),
            });
        }
        let mut records = vec![];
        for j in c
            .plan
            .joins()
            .iter()
            .filter(|j| j.kind == Some(weft_core::arithmetic_plan::JoinKind::Left))
        {
            let r = b
                .records
                .iter()
                .find(|r| r.logical == j.right.record)
                .ok_or_else(|| fail("LEFT Record lacks mapping"))?;
            if r.kind != RecordKind::Object {
                return Err(fail(
                    "Initial LEFT path profile requires original object-current identities",
                ));
            }
            records.push(NativeRecordSource {
                scan: j.right.occurrence.clone(),
                record: j.right.record.clone(),
                table: table(&b.publication.tables[r.table]),
                identity_column: "id".into(),
                native_type: "BIGINT".into(),
            });
        }
        Ok(Validated {
            mapping: b,
            additional_capabilities: vec![],
            coverage: c.selection.clone(),
            obligations: vec![],
            edge_sources: edges,
            record_sources: records,
        })
    }
    fn assess(&self, c: &Context<'_>, _: &Binding) -> Result<Vec<Assessment>> {
        Ok(c.plan
            .capabilities()
            .iter()
            .map(|id| Assessment {
                id: id.clone(),
                status: Status::Candidate,
                evidence: vec![],
                obligations: vec![],
            })
            .collect())
    }
    fn lower(&self, c: &Context<'_>, b: &Binding) -> Result<PathTargetPlan> {
        crate::candidate::path_lowering::lower(c, b)
    }
    fn emit(&self, _: &Context<'_>, p: &PathTargetPlan) -> Result<Emission> {
        Ok(Emission {
            sql: p.0.sql.clone(),
            parameters: p.0.parameters.clone(),
            columns: p.0.columns.clone(),
            obligations: p.0.obligations.clone(),
        })
    }
}
