// SPDX-License-Identifier: MIT OR Apache-2.0
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
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/application-lineage-request-v0.2-candidate.schema.json")]
struct RequestSchemaV02;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/application-lineage-v0.2-candidate.schema.json")]
struct ReportSchemaV02;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/application-lineage-request-v0.3-candidate.schema.json")]
struct RequestSchemaV03;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/application-lineage-v0.3-candidate.schema.json")]
struct ReportSchemaV03;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/application-lineage-request-v0.4-candidate.schema.json")]
struct RequestSchemaV04;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/application-lineage-v0.4-candidate.schema.json")]
struct ReportSchemaV04;
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
    catalog: Catalog,
    plan: crate::application_ir::Plan,
    order_directions: Vec<OrderDirection>,
    source_request_json: String,
}
/// Direction from the original parser, kept separately from the older plan representation.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum OrderDirection { Asc, Desc }
impl ApplicationLineage {
    pub fn json(&self) -> &str {
        &self.json
    }
    /// Original resolved source context; never a native permission or current cut.
    pub fn catalog(&self) -> &Catalog { &self.catalog }
    /// Original typed plan; native lowering needs a separately selected registration.
    pub fn plan(&self) -> &crate::application_ir::Plan { &self.plan }
    /// Complete original order directions. A lowerer must preserve each occurrence.
    pub fn order_directions(&self) -> &[OrderDirection] { &self.order_directions }
    /// Exact original request UTF8, including whitespace and opaque Artifact source bytes.
    pub fn source_request_json(&self) -> &str { &self.source_request_json }
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
    let presence = value["interfaceVersion"] == "weft-application-lineage/0.4-candidate";
    if presence && !cfg!(feature="tagged-current-projection-candidate"){return Err(fail());}
    let bounded = presence || value["interfaceVersion"] == "weft-application-lineage/0.3-candidate";
    let extended=bounded || value["interfaceVersion"]=="weft-application-lineage/0.2-candidate";
    if !(if presence{RequestSchemaV04::is_valid(&value)}else if bounded{RequestSchemaV03::is_valid(&value)}else if extended{RequestSchemaV02::is_valid(&value)}else{RequestSchema::is_valid(&value)}) {
        return Err(fail());
    }
    let request: Request = serde_json::from_value(value).map_err(|_| fail())?;
    if request.interface_version != if presence{"weft-application-lineage/0.4-candidate"}else if bounded{"weft-application-lineage/0.3-candidate"}else if extended{"weft-application-lineage/0.2-candidate"}else{"weft-application-lineage/0.1-candidate"}
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
    if extended&&request.read_profile.is_some(){return Err(fail());}
    crate::security_mapping_transport::parameter_payload_budget_selected(
        &request.sql,
        &request.parameters,
        8_000_000,extended,
    )?;
    let catalog = if bounded { Catalog::prepare_security_bounded_strings(request.modules)? } else { Catalog::prepare_security(request.modules)? };
    let (syntax, directions) = if extended{crate::application_syntax::parse_lineage_candidate(&request.sql)?}else{crate::application_syntax::parse_security(&request.sql)?};
    // Resolution expands Entity/wildcard syntax into Field expressions. Check
    // the original projection kind before that expansion can erase its meaning.
    if presence && syntax.outputs.iter().any(|p|!matches!(p.output,crate::application_syntax::Output::Field(_))) {
        return Err(fail());
    }
    let mut plan = if bounded { crate::application_resolve::resolve_bounded_current_strings(&catalog,syntax,request.parameters)? } else { crate::application_resolve::resolve(
        &catalog,
        syntax,
        request.parameters,
        request.read_profile,
    )? };
    if presence{plan.ir_version="weft-ir/application-lineage/0.4-candidate".into();}else if bounded{plan.ir_version="weft-ir/application-lineage/0.3-candidate".into();}else if extended{plan.ir_version="weft-ir/application-lineage/0.2-candidate".into();}
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
    if presence {
        // This frontend represents direct disclosure projections only. It does not
        // define value operators over absent/withheld/tagged payloads.
        if !plan.joins.is_empty() || !plan.filters.is_empty() || !plan.groups.is_empty()
            || !plan.order.is_empty() || plan.limit.is_some() || plan.page_key.is_some()
            || plan.outputs.iter().any(|o|!matches!(o.expression,crate::application_ir::Expression::Field{..}))
        {return Err(fail());}
        let record=catalog.record_by_identity(&plan.source.record)?;
        for output in &plan.outputs {
            let crate::application_ir::Expression::Field{identity,..}=&output.expression else{return Err(fail());};
            let (_,graph)=catalog.member_descriptor_by_identity(&record,identity)?;
            let descriptor=graph.iter().find(|d|d.identity==*identity).ok_or_else(fail)?;
            if !matches!(descriptor.shape,crate::application_model::Shape::Scalar{..}) {return Err(fail());}
        }
        plan.required_capabilities.push("tagged-current-projection".into());
        plan.required_capabilities.sort();
    } else {crate::application_property_uses::require_present_fields(&plan,&catalog)?;}
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
        if u.operation=="countDistinct"&&u.logical_type.nullable{row["required"]=json!(["equality","nullity"]);}
        if let Some(output) = u.output {
            row["outputType"] = json!(output);
        }
        let encoded = encode(&row, remaining)?;
        remaining = remaining.checked_sub(encoded.len()).ok_or_else(fail)?;
        uses.push(row);
        Ok(())
    })?;
    let plan_json = encode(&plan, 8_000_000)?;
    let mut report = json!({"version":if presence{"weft.application.lineage/0.4-candidate"}else if bounded{"weft.application.lineage/0.3-candidate"}else if extended{"weft.application.lineage/0.2-candidate"}else{"weft.application.lineage/0.1-candidate"},"securityReady":false,"authority":false,"sourceRequestJson":raw,"sourceRequestSha256":sha256(raw.as_bytes()),"modelPins":catalog.pins(),"sqlSha256":sha256(request.sql.as_bytes()),"applicationPlanJson":plan_json,"applicationPlanSha256":sha256(plan_json.as_bytes()),"orderDirections":directions.iter().map(|d|if *d{"desc"}else{"asc"}).collect::<Vec<_>>(),"scans":scans,"uses":uses,"unresolvedSources":unresolved});
    if presence{report["capabilities"]=json!(["tagged-current-projection"]);}
    if extended{report["readRequirements"]=crate::application_read_requirements::derive(&plan,&catalog)?;}
    if !(if presence{ReportSchemaV04::is_valid(&report)}else if bounded{ReportSchemaV03::is_valid(&report)}else if extended{ReportSchemaV02::is_valid(&report)}else{ReportSchema::is_valid(&report)}) {
        return Err(fail());
    }
    Ok(ApplicationLineage {
        json: encode(&report, 16_000_000)?,
        catalog,
        plan,
        order_directions: directions.into_iter().map(|d|if d{OrderDirection::Desc}else{OrderDirection::Asc}).collect(),
        source_request_json: raw.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn presence_fixture()->Value {
        serde_json::from_str(include_str!("../../../tests/current-query-presence/fixtures/optional-token-lineage.request.json")).unwrap()
    }
    #[test]
    fn presence_frontend_requires_its_build_feature_and_keeps_legacy_refusal() {
        let request=presence_fixture();
        if !cfg!(feature="tagged-current-projection-candidate") {
            assert!(prepare(&request.to_string()).is_err());return;
        }
        let prepared=prepare(&request.to_string()).unwrap();
        let report:Value=serde_json::from_str(prepared.json()).unwrap();
        assert_eq!(report["version"],"weft.application.lineage/0.4-candidate");
        assert_eq!(report["capabilities"],json!(["tagged-current-projection"]));
        assert!(prepared.plan().required_capabilities.iter().any(|v|v=="tagged-current-projection"));
        assert_eq!(report["readRequirements"]["fields"][0]["descriptor"]["availability"],"absent-allowed");
        assert_eq!(report["securityReady"],false);assert_eq!(report["authority"],false);
        let retained:Value=serde_json::from_str(report["sourceRequestJson"].as_str().unwrap()).unwrap();
        assert_eq!(retained,request);
        assert!(crate::current_query_designation::derive(&prepared).is_err());
        for version in ["0.1-candidate","0.2-candidate","0.3-candidate"] {
            let mut old=request.clone();old["interfaceVersion"]=json!(format!("weft-application-lineage/{version}"));
            assert!(prepare(&old.to_string()).is_err(),"{version}");
        }
    }
    #[cfg(feature="tagged-current-projection-candidate")]
    #[test]
    fn presence_frontend_refuses_unimplemented_value_operators_before_lowering() {
        for sql in ["SELECT r.* FROM Resource r",
                    "SELECT r.*, r.token AS extra FROM Resource r",
                    "SELECT r.token FROM Resource r WHERE FALSE",
                    "SELECT r.token FROM Resource r LIMIT 0",
                    "SELECT r.token FROM Resource r ORDER BY r.token",
                    "SELECT COUNT(*) FROM Resource r",
                    "SELECT r.token FROM Resource r JOIN Resource s ON r.token = s.token",
                    "SELECT r.token FROM Resource r GROUP BY r.token"] {
            let mut request=presence_fixture();request["sql"]=json!(sql);
            assert!(prepare(&request.to_string()).is_err(),"{sql}");
        }
    }
    #[cfg(feature="tagged-current-projection-candidate")]
    #[test]
    fn presence_frontend_preserves_repeated_projection_positions_and_refuses_unknown_cardinality() {
        let mut request=presence_fixture();
        request["sql"]=json!("SELECT r.token AS first, r.token AS second FROM Resource r");
        let prepared=prepare(&request.to_string()).unwrap();
        let report:Value=serde_json::from_str(prepared.json()).unwrap();
        let fields=report["readRequirements"]["fields"].as_array().unwrap();
        assert_eq!(fields.len(),2);
        assert_eq!(fields[0]["path"],"outputs/0/expression");assert_eq!(fields[1]["path"],"outputs/1/expression");
        assert_eq!(fields[0]["fieldIdentity"],fields[1]["fieldIdentity"]);
        let mut document:Value=serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
        let token=document["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="token").unwrap();
        token["cardinality"]=json!("many");
        let raw=document.to_string();request["modules"][0]["documentJson"]=json!(raw);
        request["modules"][0]["pin"]["sha256"]=json!(sha256(raw.as_bytes()));
        assert!(prepare(&request.to_string()).is_err());
    }
    #[cfg(feature="tagged-current-projection-candidate")]
    #[test]
    fn presence_frontend_refuses_selected_array_map_without_poisoning_scalar_reads() {
        for cardinality in ["array","map"] {
            let mut request=presence_fixture();
            let mut document:Value=serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
            let token=document["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="token").unwrap().as_object_mut().unwrap();
            token.remove("scalarType");token.remove("facets");
            token.insert("cardinality".into(),json!(cardinality));
            token.insert("itemType".into(),json!({"module":"m","element":"resourceId"}));
            let raw=document.to_string();request["modules"][0]["documentJson"]=json!(raw);
            request["modules"][0]["pin"]["sha256"]=json!(sha256(raw.as_bytes()));
            assert!(prepare(&request.to_string()).is_err(),"selected {cardinality}");
            request["sql"]=json!("SELECT r.resourceId FROM Resource r");
            let prepared=prepare(&request.to_string()).unwrap();
            let report:Value=serde_json::from_str(prepared.json()).unwrap();
            assert_eq!(report["readRequirements"]["fields"][0]["descriptor"]["availability"],"required");
            assert!(!prepared.plan().required_capabilities.iter().any(|v|v=="project.entity"));
        }
    }
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

#[cfg(test)]
mod availability_regressions{
 use super::*;
 #[test]
 fn independent_selected_absence_witnesses_refuse_without_erasing_count_scans(){
  let cases:Value=serde_json::from_str(include_str!("../../../tests/lineage-availability/expanded-before.json")).unwrap();
  assert_eq!(cases["observations"].as_array().unwrap().len(),8);
  for case in cases["observations"].as_array().unwrap(){
   let raw=case["request"].to_string();let result=prepare(&raw);
   if case["expected"]=="refused"{let error=result.err().unwrap();assert_eq!(error.code,"WFT-APPLICATION-LINEAGE-AVAILABILITY","{}",case["id"]);}
   else{let result=result.unwrap();let report:Value=serde_json::from_str(result.json()).unwrap();assert_eq!(report["authority"],false);assert_eq!(report["sourceRequestJson"],raw);assert_eq!(report["scans"].as_object().unwrap().len(),1);if case["id"]=="field-free-count"{assert_eq!(report["uses"],json!([]));}}
  }
 }
}

#[cfg(test)]
// Component prerequisites only; full native acceptance remains open.
// @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
mod v02_tests{
 use super::*;
 fn request(sql:&str)->Value{let mut r:Value=serde_json::from_str(include_str!("../../../tests/application-lineage/fixtures/token-equality.request.json")).unwrap();r["interfaceVersion"]=json!("weft-application-lineage/0.2-candidate");r["sql"]=json!(sql);r}
 fn report(sql:&str)->Value{serde_json::from_str(prepare(&request(sql).to_string()).unwrap().json()).unwrap()}
 #[test]
 fn distinct_count_owns_exact_present_domain_and_nonnull_count_result(){
  let r=report("SELECT COUNT(DISTINCT r.token) AS unique_tokens FROM Resource r");assert_eq!(r["version"],"weft.application.lineage/0.2-candidate");assert_eq!(r["authority"],false);let use0=&r["uses"][0];assert_eq!(use0["operation"],"countDistinct");assert_eq!(use0["operator"],"aggregate");assert_eq!(use0["required"],json!(["equality"]));assert_eq!(use0["logicalType"]["nullable"],false);assert_eq!(use0["outputType"]["nullable"],false);assert_eq!(use0["outputType"]["family"],"integer");assert!(use0.get("mode").is_none());
  let p:Value=serde_json::from_str(r["applicationPlanJson"].as_str().unwrap()).unwrap();assert_eq!(p["irVersion"],"weft-ir/application-lineage/0.2-candidate");assert_eq!(p["outputs"][0]["expression"]["op"],"countDistinct");assert!(p["requiredCapabilities"].as_array().unwrap().contains(&json!("aggregate.countDistinct")));
 }
 #[test]
 fn false_zero_limit_and_self_join_cannot_erase_uses_or_scans(){
  let r=report("SELECT COUNT(DISTINCT s.token) FROM Resource r JOIN Resource s ON r.token=s.token WHERE FALSE AND r.token='opaque-token' LIMIT 0");assert_eq!(r["scans"].as_object().unwrap().len(),2);assert_eq!(r["uses"].as_array().unwrap().len(),4);assert_eq!(r["uses"][0]["scan"],"s0");assert_eq!(r["uses"][1]["scan"],"s1");assert_eq!(r["uses"][2]["path"],"filters/1/left");assert_eq!(r["uses"][3]["scan"],"s1");let p:Value=serde_json::from_str(r["applicationPlanJson"].as_str().unwrap()).unwrap();assert_eq!(p["filters"][0],json!({"op":"boolean","value":false}));assert_eq!(p["limit"],0);assert_eq!(p["joins"].as_array().unwrap().len(),1);
  let r=report("SELECT COUNT(*) FROM Resource r WHERE TRUE AND FALSE LIMIT 0");assert!(r["uses"].as_array().unwrap().is_empty());assert_eq!(r["scans"].as_object().unwrap().len(),1);
  for sql in ["SELECT COUNT(*) FROM Resource r WHERE FALSE AND r.missing='x' LIMIT 0","SELECT COUNT(*) FROM Resource r WHERE FALSE AND r.token=:missing LIMIT 0"]{assert!(prepare(&request(sql).to_string()).is_err());}
 }
 #[test]
 fn prior_protocols_and_absent_arguments_do_not_gain_support(){
  for sql in ["SELECT COUNT(DISTINCT r.token) FROM Resource r","SELECT COUNT(*) FROM Resource r LIMIT 0","SELECT COUNT(*) FROM Resource r WHERE FALSE"]{let mut r=request(sql);r["interfaceVersion"]=json!("weft-application-lineage/0.1-candidate");assert!(prepare(&r.to_string()).is_err());assert!(crate::application_syntax::parse(sql).is_err());assert!(crate::application_syntax::parse_security(sql).is_err());}
  let mut r=request("SELECT COUNT(DISTINCT r.token) FROM Resource r");let mut doc:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();next_absent(&mut doc);let text=doc.to_string();r["modules"][0]["documentJson"]=json!(text);r["modules"][0]["pin"]["sha256"]=json!(sha256(text.as_bytes()));assert!(prepare(&r.to_string()).is_err());
  let mut r=request("SELECT COUNT(DISTINCT r.token) FROM Resource r");r["readProfile"]=json!({"version":"weft-application-read/0.2.0","subset":"count-summary"});assert!(prepare(&r.to_string()).is_err());
 }
 fn next_absent(doc:&mut Value){doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="token").unwrap()["nullability"]=json!("absent-allowed");}
 #[test]
 fn new_syntax_is_bounded_and_does_not_accept_count_expressions_or_join_constants(){
  for sql in ["SELECT COUNT(DISTINCT *) FROM Resource r","SELECT COUNT(DISTINCT r.token,r.salary) FROM Resource r","SELECT COUNT(DISTINCT r.token+1) FROM Resource r","SELECT COUNT(DISTINCT) FROM Resource r","SELECT COUNT(*) FROM Resource r LIMIT -1","SELECT COUNT(*) FROM Resource r LIMIT 1001","SELECT COUNT(*) FROM Resource r LIMIT 0.0","SELECT COUNT(*) FROM Resource r LIMIT 0e0","SELECT COUNT(*) FROM Resource r JOIN Resource s ON FALSE"]{assert!(prepare(&request(sql).to_string()).is_err(),"{sql}");}
  for sql in ["SELECT COUNT(*) AS all_rows,COUNT(DISTINCT r.token) AS distinct_tokens FROM Resource r","SELECT r.token,COUNT(DISTINCT r.salary) AS amounts FROM Resource r GROUP BY r.token ORDER BY r.token LIMIT 0"]{let r=report(sql);assert_eq!(r["authority"],false);}
 }
}

#[cfg(test)]
// Component prerequisites only; full native acceptance remains open.
// @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
mod v02_read_requirements_tests{
 use super::*;
 fn report(sql:&str)->Value{let mut r:Value=serde_json::from_str(include_str!("../../../tests/application-lineage/fixtures/token-equality.request.json")).unwrap();r["interfaceVersion"]=json!("weft-application-lineage/0.2-candidate");r["sql"]=json!(sql);serde_json::from_str(prepare(&r.to_string()).unwrap().json()).unwrap()}
 #[test]
 fn projection_occurrences_and_hidden_uses_have_owner_derived_read_sources(){
  let r=report("SELECT r.token AS first,r.token AS second FROM Resource r WHERE FALSE AND r.token='opaque-token' ORDER BY r.salary LIMIT 0");let reads=&r["readRequirements"];assert_eq!(reads["scans"].as_array().unwrap().len(),1);assert_eq!(reads["scans"][0]["kind"],"rowRead");let fields=reads["fields"].as_array().unwrap();assert_eq!(fields.len(),4);assert_eq!(fields[0]["path"],"outputs/0/expression");assert_eq!(fields[1]["path"],"outputs/1/expression");assert_eq!(fields[0]["role"],"projection");assert_eq!(fields[2]["path"],"filters/1/left");assert_eq!(fields[3]["role"],"order");for f in fields{assert_eq!(f["descriptor"]["availability"],"required");assert_eq!(f["fieldIdentity"]["revision"],"named-query-core-r1");assert!(f.get("mode").is_none());assert!(f.get("action").is_none());}
 }
 #[test]
 fn field_free_false_count_keeps_both_join_row_scans_and_explicit_sum_limit(){
  let r=report("SELECT COUNT(*) FROM Resource r WHERE FALSE LIMIT 0");assert_eq!(r["readRequirements"]["fields"],json!([]));assert_eq!(r["readRequirements"]["scans"].as_array().unwrap().len(),1);
  let r=report("SELECT COUNT(DISTINCT s.token) FROM Resource r JOIN Resource s ON r.token=s.token LIMIT 0");let reads=&r["readRequirements"];assert_eq!(reads["scans"].as_array().unwrap().len(),2);assert_eq!(reads["fields"].as_array().unwrap().len(),3);assert_eq!(reads["fields"][0]["scan"],"s0");assert_eq!(reads["fields"][1]["scan"],"s1");assert_eq!(reads["fields"][2]["role"],"aggregate");
  let r=report("SELECT SUM(r.salary) FROM Resource r LIMIT 0");assert_eq!(r["readRequirements"]["unsupportedSources"],json!([{"path":"outputs/0/argument","kind":"numericAggregateProperties"}]));
 }
 #[test]
 fn public_relationship_observation_and_terminal_keys_remain_separate_read_intents(){
  let source:Value=serde_json::from_str(include_str!("../../weft-postgresql/tests/security-public-relationship/compiler-request.json")).unwrap();let r=json!({"interfaceVersion":"weft-application-lineage/0.2-candidate","modules":source["modules"],"sql":source["sql"],"parameters":source["parameters"],"opaqueSources":[]});let r:Value=serde_json::from_str(prepare(&r.to_string()).unwrap().json()).unwrap();let reads=&r["readRequirements"];assert_eq!(reads["scans"].as_array().unwrap().len(),2);assert_eq!(reads["relationships"][0]["kind"],"relationshipObservation");assert_eq!(reads["relationships"][0]["sourceScan"],"s0");assert_eq!(reads["relationships"][0]["terminalScan"],"t0");assert!(reads["fields"].as_array().unwrap().iter().any(|f|f["scan"]=="t0"&&f["role"]=="predicate"));
 }
}

#[cfg(test)]
mod original_context_tests{
 use super::*;
 // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
 #[test]
 fn typed_preparation_keeps_actual_catalog_plan_and_complete_order_directions(){
  let raw=include_str!("../../../tests/lineage-v02/fixtures/distinct-selfjoin-false-zero.request.json");let prepared=prepare(raw).unwrap();let report:Value=serde_json::from_str(prepared.json()).unwrap();
  assert_eq!(prepared.source_request_json(),raw);
  assert_eq!(serde_json::to_string(prepared.plan()).unwrap(),report["applicationPlanJson"].as_str().unwrap());
  assert_eq!(serde_json::to_value(prepared.catalog().pins()).unwrap(),report["modelPins"]);
  let original:Value=serde_json::from_str(raw).unwrap();assert_eq!(serde_json::to_value(prepared.catalog().inputs()).unwrap(),original["modules"]);
  assert_eq!(prepared.plan().ir_version,"weft-ir/application-lineage/0.2-candidate");assert_eq!(prepared.plan().joins.len(),1);assert_eq!(prepared.plan().limit,Some(0));
  let mut request=original;request["sql"]=json!("SELECT r.resourceId FROM Resource r ORDER BY r.token DESC,r.salary ASC LIMIT 0");let prepared=prepare(&request.to_string()).unwrap();
  assert_eq!(prepared.order_directions(),&[OrderDirection::Desc,OrderDirection::Asc]);assert_eq!(prepared.plan().order.len(),2);
  let report:Value=serde_json::from_str(prepared.json()).unwrap();assert_eq!(report["orderDirections"],json!(["desc","asc"]));
 }
}

#[cfg(test)]
mod bounded_string_tests {
    use super::*;
    fn request() -> Value {
        let mut r:Value=serde_json::from_str(include_str!("../../../tests/current-query-designation/fixtures/mixed-original-uses.request.json")).unwrap();
        r["interfaceVersion"]=json!("weft-application-lineage/0.3-candidate");
        r["sql"]=json!("SELECT r.token FROM Resource r WHERE r.token='😀' ORDER BY r.token DESC");
        let mut d:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
        d["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="token").unwrap()["facets"]=json!({"length":{"min":1,"max":3,"unit":"unicode-scalar"}});
        let raw=d.to_string();r["modules"][0]["pin"]["sha256"]=json!(sha256(raw.as_bytes()));r["modules"][0]["documentJson"]=json!(raw);r
    }
    #[test]
    fn bounded_original_source_and_type_survive_only_explicit_new_preparation() {
        let r=request();let raw=r.to_string();let p=prepare(&raw).unwrap();
        assert_eq!(p.source_request_json(),raw);
        assert_eq!(p.catalog().inputs()[0].document_json,r["modules"][0]["documentJson"].as_str().unwrap());
        assert_eq!(p.plan().ir_version,"weft-ir/application-lineage/0.3-candidate");
        let report:Value=serde_json::from_str(p.json()).unwrap();assert_eq!(report["version"],"weft.application.lineage/0.3-candidate");
        let uses=report["uses"].as_array().unwrap();assert!(uses.len()>=2);
        for u in uses {assert_eq!(u["logicalType"]["facets"],json!({"length":{"min":1,"max":3,"unit":"unicode-scalar"}}));}
        for version in ["0.1-candidate","0.2-candidate"] {let mut old=r.clone();old["interfaceVersion"]=json!(format!("weft-application-lineage/{version}"));assert!(prepare(&old.to_string()).is_err());}
        // Public legacy re-resolution cannot erase the catalog's bounded profile.
        let legacy_query=crate::application_syntax::parse("SELECT r.token FROM Resource r").unwrap();
        assert!(crate::application_resolve::resolve(p.catalog(),legacy_query,Default::default(),None).is_err());
        // The old designation factory must not silently accept this new source context.
        assert!(crate::current_query_designation::derive(&p).is_err());
    }
    #[test]
    fn original_literal_and_parameter_bounds_precede_false_and_limit_zero() {
        for value in ["", "four", "😀😀😀😀"] {
            let mut r=request();r["sql"]=json!(format!("SELECT r.token FROM Resource r WHERE FALSE AND r.token='{value}' LIMIT 0"));
            assert!(prepare(&r.to_string()).is_err());
            r["sql"]=json!("SELECT r.token FROM Resource r WHERE FALSE AND r.token=:v LIMIT 0");
            r["parameters"]=json!({"v":{"family":"string","value":value}});assert!(prepare(&r.to_string()).is_err());
        }
        for value in ["😀", "e\u{301}", "😀éA"] {
            let mut r=request();r["sql"]=json!("SELECT r.token FROM Resource r WHERE FALSE AND r.token=:v LIMIT 0");r["parameters"]=json!({"v":{"family":"string","value":value}});
            let p=prepare(&r.to_string()).unwrap();assert!(p.json().contains(value));
        }
    }
}
