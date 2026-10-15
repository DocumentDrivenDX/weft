//! Original AST property requirements, separate from storage/disclosure/native authority.
use crate::{
    error::{Diagnostic, Result},
    ir::{Identity, LogicalType},
    security_backend::SecurityBackendContext,
    security_ir::QueryOperator,
    security_ontology::SecurityRef,
    security_requirements::SecurityOperatorMode,
};
use serde_json::{json, Value};
#[jsonschema::validator(
    path = "../../docs/helix/02-design/contracts/security-query-properties-v0.1-candidate.schema.json"
)]
struct ReportSchema;
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-QUERY-PROPERTIES",
        "capability",
        "Original AST query property derivation refused",
    )
}
/// Immutable compiler derivation. No constructor/deserializer or execution authority.
pub struct SecurityQueryProperties {
    json: String,
}
impl SecurityQueryProperties {
    pub fn json(&self) -> &str {
        &self.json
    }
}
struct Derivation<'a, 'b> {
    ctx: &'a SecurityBackendContext<'b>,
    uses: Vec<Value>,
    remaining: usize,
}
impl Derivation<'_, '_> {
    fn add(
        &mut self,
        path: &str,
        scan: &str,
        identity: &Identity,
        logical_type: &LogicalType,
        operator: QueryOperator,
        operation: &str,
        required: Option<&str>,
        output: Option<&LogicalType>,
    ) -> Result<()> {
        if self.uses.len() >= 4096 {
            return Err(fail());
        }
        let field = SecurityRef {
            document_id: identity.document_id.clone(),
            module_id: identity.module.clone(),
            element_id: identity.element.clone(),
        };
        let target = self.ctx.query().scans().get(scan).ok_or_else(fail)?;
        let requirement = self
            .ctx
            .requirements()
            .operators()
            .iter()
            .find(|r| {
                let u = r.usage();
                u.scan == scan && u.target == *target && u.field == field && u.operator == operator
            })
            .ok_or_else(fail)?;
        if !self
            .ctx
            .catalog()
            .pins()
            .iter()
            .any(|p| p.document_id == identity.document_id && p.revision == identity.revision)
        {
            return Err(fail());
        }
        let (mode, action) = match requirement.mode() {
            SecurityOperatorMode::Disclosed => ("disclosed", None),
            SecurityOperatorMode::OriginalAuthorized(a) => ("original-authorized", Some(a)),
        };
        // Charge retained source/type payload before cloning it into an occurrence.
        let type_text = serde_json::to_string(logical_type).map_err(|_| fail())?;
        let output_text = output
            .map(serde_json::to_string)
            .transpose()
            .map_err(|_| fail())?;
        let charge = path.len()
            + scan.len()
            + field.document_id.len()
            + field.module_id.len()
            + field.element_id.len()
            + target.document_id.len()
            + target.module_id.len()
            + target.element_id.len()
            + type_text.len()
            + output_text.as_ref().map_or(0, String::len)
            + action.map_or(0, str::len)
            + 512;
        self.remaining = self
            .remaining
            .checked_sub(charge.checked_mul(6).ok_or_else(fail)?)
            .ok_or_else(fail)?;
        let mut row = json!({"path":path,"scan":scan,"type":target,"field":field,"fieldIdentity":identity,"logicalType":logical_type,"operator":operator,"operation":operation,"required":required.map(|p|vec![p]),"mode":mode,"modeSource":"legacy-ontology-query-profile","propertyStatus":if required.is_some(){"derived"}else{"unsupported"}});
        if let Some(action) = action {
            row["originalAction"] = json!(action);
        }
        if let Some(output) = output {
            row["outputType"] = json!(output);
        }
        self.uses.push(row);
        Ok(())
    }
}
/// Derive from the actual admitted owner context; supplied property annotations are never read.
pub fn derive(ctx: &SecurityBackendContext<'_>) -> Result<SecurityQueryProperties> {
    let query = ctx.query().application_plan();
    let mut d = Derivation {
        ctx,
        uses: Vec::new(),
        remaining: 8_000_000,
    };
    crate::application_property_uses::visit(query, ctx.catalog(), |u| {
        d.add(
            &u.path,
            u.scan,
            u.identity,
            u.logical_type,
            u.operator,
            u.operation,
            u.required,
            u.output,
        )
    })?;
    // Compare complete deduplicated coverage with the admitted old operator inventory.
    for r in ctx.requirements().operators() {
        let u = r.usage();
        if !d.uses.iter().any(|v| {
            v["scan"] == u.scan
                && v["field"] == json!(u.field)
                && v["operator"] == json!(u.operator)
        }) {
            return Err(fail());
        }
    }
    let source = ctx.source_context()?;
    let value = json!({"version":"weft.security.query-properties/0.1-candidate","securityReady":false,"authority":false,"sourceContextJson":source.json(),"sourceContextSha256":source.sha256(),"queryContextSha256":ctx.execution_query_context_sha256(),"sql":ctx.query().sql(),"sqlSha256":crate::json::sha256(ctx.query().sql().as_bytes()),"scans":ctx.query().scans(),"uses":d.uses,"outputs":query.outputs});
    if !ReportSchema::is_valid(&value) {
        return Err(fail());
    }
    let text = serde_json::to_string(&value).map_err(|_| fail())?;
    if text.len() > 16_000_000 {
        return Err(fail());
    }
    Ok(SecurityQueryProperties { json: text })
}

#[cfg(test)]
mod schema_tests {
    use super::*;
    #[test]
    fn relationship_terminal_key_keeps_its_original_scan_and_equality_demand() {
        let raw = include_str!(
            "../../weft-postgresql/tests/security-public-relationship/compiler-request.json"
        );
        crate::security_mapping_transport::with_security_relationship_context(raw, |ctx| {
            let report: Value = serde_json::from_str(derive(ctx)?.json()).unwrap();
            let uses = report["uses"].as_array().unwrap();
            let key = uses
                .iter()
                .find(|u| u["path"] == "filters/0/targetKey/0")
                .unwrap();
            assert_eq!(key["scan"], "t0");
            assert_eq!(key["required"], json!(["equality"]));
            assert_eq!(key["operation"], "hasRelatedKey");
            assert!(report["scans"].get("s0").is_some());
            assert!(report["scans"].get("t0").is_some());
            Ok(())
        })
        .unwrap();
    }
    #[test]
    fn report_schema_refuses_capability_and_property_reclassification() {
        let raw = include_str!(
            "../../../docs/helix/03-test/fixtures/security-graph-source/mapping-request.json"
        );
        let original: Value = serde_json::from_str(
            &crate::security_mapping_transport::prepare_security_query_properties(raw).unwrap(),
        )
        .unwrap();
        assert!(ReportSchema::is_valid(&original));
        for key in ["authority", "securityReady"] {
            let mut bad = original.clone();
            bad[key] = json!(true);
            assert!(!ReportSchema::is_valid(&bad));
        }
        let mut bad = original.clone();
        bad["nativeAdmission"] = json!("permitted");
        assert!(!ReportSchema::is_valid(&bad));
        let mut bad = original.clone();
        bad["version"] = json!("weft.security.mapping-handoff/0.2.0");
        assert!(!ReportSchema::is_valid(&bad));
        let mut bad = original.clone();
        bad["outputs"][0]["expression"]["required"] = json!(["equality"]);
        assert!(!ReportSchema::is_valid(&bad));
    }
}
