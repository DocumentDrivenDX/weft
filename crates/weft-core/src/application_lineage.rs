//! Policy-independent original application AST lineage. Opaque sources confer no authority.
use crate::{
    application_ir::ReadProfile,
    application_resolve::Parameters,
    error::{Diagnostic, Result},
    json::{checked_json, sha256},
    model::{Catalog, ModuleInput},
    security_ontology::SecurityRef,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
#[jsonschema::validator(
    path = "../../docs/helix/02-design/contracts/application-lineage-request-v0.1-candidate.schema.json"
)]
struct RequestSchema;
#[jsonschema::validator(
    path = "../../docs/helix/02-design/contracts/application-lineage-v0.1-candidate.schema.json"
)]
struct ReportSchema;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SourceIdentity {
    document_id: String,
    revision: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OpaqueSource {
    role: String,
    identity: SourceIdentity,
    media_type: String,
    utf8: String,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    interface_version: String,
    modules: Vec<ModuleInput>,
    sql: String,
    parameters: Parameters,
    read_profile: Option<ReadProfile>,
    opaque_sources: Vec<OpaqueSource>,
}
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-APPLICATION-LINEAGE",
        "input",
        "Bounded original application lineage refused",
    )
}
/// Immutable compiler result; not a current policy/catalog/storage/native permit.
/// ```compile_fail
/// use weft_core::application_lineage::ApplicationLineage;
/// fn replace(p:&mut ApplicationLineage){p.json.clear();}
/// ```
pub struct ApplicationLineage {
    json: String,
}
impl ApplicationLineage {
    pub fn json(&self) -> &str {
        &self.json
    }
}
struct Buffer {
    bytes: Vec<u8>,
    remaining: usize,
}
impl std::io::Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.remaining {
            return Err(std::io::Error::other("limit"));
        }
        self.remaining -= bytes.len();
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn encode(value: &impl serde::Serialize, limit: usize) -> Result<String> {
    let mut buffer = Buffer {
        bytes: Vec::new(),
        remaining: limit,
    };
    serde_json::to_writer(&mut buffer, value).map_err(|_| fail())?;
    String::from_utf8(buffer.bytes).map_err(|_| fail())
}
fn text(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && !value.contains('\0')
}
/// Resolve only original core models/SQL. Current policy/catalog bodies are retained
/// byte-for-byte and explicitly unresolved; no synthetic legacy context is constructed.
pub fn prepare(raw: &str) -> Result<ApplicationLineage> {
    if raw.len() > 4_000_000 {
        return Err(fail());
    }
    let value = checked_json(raw).map_err(|_| fail())?;
    if !RequestSchema::is_valid(&value) {
        return Err(fail());
    }
    let request: Request = serde_json::from_value(value).map_err(|_| fail())?;
    if request.interface_version != "weft-application-lineage/0.1-candidate"
        || request.opaque_sources.len() > 16
    {
        return Err(fail());
    }
    let mut identities = BTreeSet::new();
    let mut unresolved = Vec::new();
    for source in &request.opaque_sources {
        if !text(&source.role, 128)
            || !text(&source.identity.document_id, 4096)
            || !text(&source.identity.revision, 4096)
            || !text(&source.media_type, 256)
            || source.utf8.len() > 4_000_000
            || source.sha256 != sha256(source.utf8.as_bytes())
            || !identities.insert((
                &source.role,
                &source.identity.document_id,
                &source.identity.revision,
            ))
        {
            return Err(fail());
        }
        unresolved.push(json!({"role":source.role,"identity":{"documentId":source.identity.document_id,"revision":source.identity.revision},"mediaType":source.media_type,"sha256":source.sha256,"status":"uninterpreted"}));
    }
    // Reuse the exact occurrence-sensitive original parameter preflight, not a second parser.
    crate::security_mapping_transport::parameter_payload_budget(
        &request.sql,
        &request.parameters,
        8_000_000,
    )?;
    let catalog = Catalog::prepare_security(request.modules)?;
    let (syntax, directions) = crate::application_syntax::parse_security(&request.sql)?;
    let mut plan = crate::application_resolve::resolve(
        &catalog,
        syntax,
        request.parameters,
        request.read_profile,
    )?;
    if directions.len() != plan.order.len() {
        return Err(fail());
    }
    if directions.iter().any(|d| *d) {
        if plan.page_key.is_some()
            || plan.filters.iter().any(|p| {
                matches!(
                    p,
                    crate::application_ir::Predicate::LexicographicGreater { .. }
                )
            })
        {
            return Err(fail());
        }
        plan.required_capabilities.retain(|c| c != "order.asc");
        if directions.iter().any(|d| !*d) {
            plan.required_capabilities.push("order.asc".into());
        }
        plan.required_capabilities.push("order.desc".into());
        plan.required_capabilities.sort();
    }
    let scans = crate::application_property_uses::scans(&plan)?;
    let mut uses = Vec::new();
    let mut remaining = 8_000_000usize;
    crate::application_property_uses::visit(&plan, &catalog, |u| {
        if uses.len() >= 4096 {
            return Err(fail());
        }
        let target = scans.get(u.scan).ok_or_else(fail)?;
        let row = json!({"path":u.path,"scan":u.scan,"type":SecurityRef{document_id:target.document_id.clone(),module_id:target.module.clone(),element_id:target.element.clone()},"field":SecurityRef{document_id:u.identity.document_id.clone(),module_id:u.identity.module.clone(),element_id:u.identity.element.clone()},"fieldIdentity":u.identity,"logicalType":u.logical_type,"operator":u.operator,"operation":u.operation,"required":u.required.map(|p|vec![p]),"propertyStatus":if u.required.is_some(){"derived"}else{"unsupported"}});
        let mut row = row;
        if let Some(output) = u.output {
            row["outputType"] = json!(output);
        }
        let encoded = encode(&row, remaining)?;
        remaining = remaining.checked_sub(encoded.len()).ok_or_else(fail)?;
        uses.push(row);
        Ok(())
    })?;
    let plan_json = encode(&plan, 8_000_000)?;
    let report = json!({"version":"weft.application.lineage/0.1-candidate","securityReady":false,"authority":false,"sourceRequestJson":raw,"sourceRequestSha256":sha256(raw.as_bytes()),"modelPins":catalog.pins(),"sqlSha256":sha256(request.sql.as_bytes()),"applicationPlanJson":plan_json,"applicationPlanSha256":sha256(plan_json.as_bytes()),"orderDirections":directions.iter().map(|d|if *d{"desc"}else{"asc"}).collect::<Vec<_>>(),"scans":scans,"uses":uses,"unresolvedSources":unresolved});
    if !ReportSchema::is_valid(&report) {
        return Err(fail());
    }
    Ok(ApplicationLineage {
        json: encode(&report, 16_000_000)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Value {
        let source: Value = serde_json::from_str(include_str!(
            "../../../tests/query-properties/fixtures/operators.request.json"
        ))
        .unwrap();
        let current: Value = serde_json::from_str(include_str!(
            "../../../tests/application-lineage/fixtures/original-umf-obligations.json"
        ))
        .unwrap();
        let opaque = current["base"]["policy"].to_string();
        json!({"interfaceVersion":"weft-application-lineage/0.1-candidate","modules":source["modules"],"sql":source["sql"],"parameters":{},"opaqueSources":[{"role":"policy","identity":{"documentId":"independent-policy","revision":"policy-r1"},"mediaType":"application/json","utf8":opaque,"sha256":sha256(opaque.as_bytes())}]})
    }
    fn report(request: &Value) -> Value {
        serde_json::from_str(prepare(&request.to_string()).unwrap().json()).unwrap()
    }
    #[test]
    fn selected_unknown_semantics_refuse_but_opaque_module_content_survives() {
        let mut request = fixture();
        let mut doc: Value =
            serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
        doc["modules"][0]["extensions"] = json!({"future":{"payload":"opaque\r\n😀"}});
        let text = doc.to_string();
        request["modules"][0]["documentJson"] = json!(text);
        request["modules"][0]["pin"]["sha256"] = json!(sha256(text.as_bytes()));
        let r = report(&request);
        let retained: Value =
            serde_json::from_str(r["sourceRequestJson"].as_str().unwrap()).unwrap();
        assert_eq!(retained["modules"], request["modules"]);
        let field = doc["modules"][0]["elements"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["id"] == "salary")
            .unwrap();
        field["extensions"] = json!({"future":{"meaning":"unestablished"}});
        let text = doc.to_string();
        request["modules"][0]["documentJson"] = json!(text);
        request["modules"][0]["pin"]["sha256"] = json!(sha256(text.as_bytes()));
        assert_eq!(
            prepare(&request.to_string()).err().unwrap().code,
            "WFT-TYPE"
        );
    }
    #[test]
    fn repeated_parameter_payload_is_bounded_before_ast_replication() {
        let mut request = fixture();
        let atom = "r.resourceId=:v";
        request["sql"] = json!(format!(
            "SELECT r.resourceId FROM Resource r WHERE {}",
            vec![atom; 32].join(" AND ")
        ));
        request["parameters"] = json!({"v":{"family":"string","value":"x".repeat(1_000_000)}});
        assert_eq!(
            prepare(&request.to_string()).err().unwrap().code,
            "WFT-SECURITY-MAPPING-PARAMETER-LIMIT"
        );
    }
    #[test]
    fn original_policy_independent_ast_keeps_occurrences_domains_and_unresolved_current_source() {
        let request = fixture();
        let r = report(&request);
        assert_eq!(r["version"], "weft.application.lineage/0.1-candidate");
        assert_eq!(r["authority"], false);
        assert_eq!(r["securityReady"], false);
        assert_eq!(r["unresolvedSources"][0]["status"], "uninterpreted");
        let retained: Value =
            serde_json::from_str(r["sourceRequestJson"].as_str().unwrap()).unwrap();
        assert_eq!(retained, request);
        let uses = r["uses"].as_array().unwrap();
        assert_eq!(uses.len(), 6);
        assert_eq!(uses[0]["scan"], "s0");
        assert_eq!(uses[1]["scan"], "s1");
        assert_ne!(uses[0]["path"], uses[1]["path"]);
        for u in uses {
            for absent in ["mode", "modeSource", "originalAction"] {
                assert!(u.get(absent).is_none());
            }
        }
        let sum = uses.iter().find(|u| u["operation"] == "sum").unwrap();
        assert_eq!(sum["propertyStatus"], "unsupported");
        assert!(sum["required"].is_null());
        assert_eq!(sum["outputType"]["family"], "integer");
        let mut bad = r.clone();
        bad["uses"][0]["mode"] = json!("disclosed");
        assert!(!ReportSchema::is_valid(&bad));
    }
    #[test]
    fn raw_source_bytes_are_not_reinterpreted_and_source_overrides_refuse() {
        let mut request = fixture();
        let opaque = "opaque\r\n😀\0{\"duplicate\":1,\"duplicate\":2}";
        request["opaqueSources"][0]["utf8"] = json!(opaque);
        request["opaqueSources"][0]["sha256"] = json!(sha256(opaque.as_bytes()));
        let raw = format!(" \n{}\t", request);
        let r: Value = serde_json::from_str(prepare(&raw).unwrap().json()).unwrap();
        assert_eq!(r["sourceRequestJson"], raw);
        assert_eq!(r["sourceRequestSha256"], sha256(raw.as_bytes()));
        assert_eq!(r["unresolvedSources"][0]["status"], "uninterpreted");
        request["opaqueSources"][0]["sha256"] = json!("0".repeat(64));
        assert!(prepare(&request.to_string()).is_err());
        for key in [
            "mode",
            "originalAction",
            "required",
            "uses",
            "applicationPlan",
            "authority",
        ] {
            let mut bad = fixture();
            bad[key] = json!(true);
            assert!(prepare(&bad.to_string()).is_err());
        }
    }
    #[test]
    fn original_unnamed_current_model_is_not_silently_given_sql_names() {
        let current: Value = serde_json::from_str(include_str!(
            "../../../tests/application-lineage/fixtures/original-umf-obligations.json"
        ))
        .unwrap();
        let doc = &current["base"]["closure"]["documents"][0]["document"];
        let text = doc.to_string();
        let mut request = fixture();
        request["modules"] = json!([{"documentJson":text,"pin":{"documentId":doc["id"],"revision":"core-r1","umfVersion":"0.8.0","sha256":sha256(text.as_bytes())},"selectedModuleIds":["records"]}]);
        request["sql"] = json!("SELECT r.token FROM Resource r");
        let refused = prepare(&request.to_string()).err().unwrap();
        assert_eq!(refused.code, "WFT-NAME-MISSING");
    }
    #[test]
    fn field_free_counts_hidden_range_and_order_directions_keep_the_actual_ast() {
        let mut request = fixture();
        request["sql"] = json!("SELECT COUNT(*) FROM Resource r");
        let r = report(&request);
        assert!(r["uses"].as_array().unwrap().is_empty());
        assert_eq!(r["scans"].as_object().unwrap().len(), 1);
        request["sql"]=json!("SELECT r.resourceId FROM Resource r WHERE (r.salary)>(100) AND r.salary=100 AND r.salary=101 ORDER BY r.salary LIMIT 1");
        let r = report(&request);
        assert_eq!(r["uses"][0]["required"], json!(["order"]));
        assert_eq!(r["uses"][1]["required"], json!(["equality"]));
        assert_eq!(r["uses"][2]["required"], json!(["equality"]));
        request["sql"] = json!("SELECT r.resourceId FROM Resource r ORDER BY r.salary DESC");
        let r = report(&request);
        assert_eq!(r["orderDirections"], json!(["desc"]));
        let plan: Value = serde_json::from_str(r["applicationPlanJson"].as_str().unwrap()).unwrap();
        assert!(plan["requiredCapabilities"]
            .as_array()
            .unwrap()
            .contains(&json!("order.desc")));
        assert!(!plan["requiredCapabilities"]
            .as_array()
            .unwrap()
            .contains(&json!("order.asc")));
    }
    #[test]
    fn relationship_terminal_key_uses_original_model_without_a_policy_context() {
        let source: Value = serde_json::from_str(include_str!(
            "../../weft-postgresql/tests/security-public-relationship/compiler-request.json"
        ))
        .unwrap();
        let mut request = fixture();
        request["modules"] = source["modules"].clone();
        request["sql"] = source["sql"].clone();
        request["parameters"] = source["parameters"].clone();
        let r = report(&request);
        let key = r["uses"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["operation"] == "hasRelatedKey")
            .unwrap();
        assert_eq!(key["scan"], "t0");
        assert_eq!(key["required"], json!(["equality"]));
        assert!(r["scans"].get("t0").is_some());
    }
    #[test]
    fn malformed_selected_semantics_unknown_wrappers_and_large_payloads_still_refuse() {
        for raw in [
            "{}",
            "[]",
            "{\"interfaceVersion\":1,\"interfaceVersion\":2}",
        ] {
            assert!(prepare(raw).is_err());
        }
        assert!(prepare(&" ".repeat(4_000_001)).is_err());
        let mut request = fixture();
        let duplicate = request["opaqueSources"][0].clone();
        request["opaqueSources"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        assert!(prepare(&request.to_string()).is_err());
        for sql in [
            "SELECT COUNT(DISTINCT r.salary) FROM Resource r",
            "SELECT COUNT(*) FROM Resource r LIMIT 0",
            "SELECT COUNT(*) FROM Resource r WHERE FALSE",
        ] {
            let mut request = fixture();
            request["sql"] = json!(sql);
            assert!(prepare(&request.to_string()).is_err());
        }
    }
}
