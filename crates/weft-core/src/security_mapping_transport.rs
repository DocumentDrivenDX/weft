//! Original source-bound mapping preparation, never native compilation or permission.
use crate::{application_ir::ReadProfile, application_resolve::Parameters,
    error::{Diagnostic, Result}, json::checked_json, model::{Catalog, ModuleInput},
    security_ir::SecurityLogicalPlan, security_query_profile::SecurityQueryProfile,
    security_query_uses::SecurityResolvedQuery, security_source::SecuritySourcePacket};
use serde::Deserialize;
use serde_json::{json, Value};

pub const MAX_REQUEST_BYTES: usize = 4_000_000;
const MAX_RESPONSE_BYTES: usize = 16_000_000;
#[jsonschema::validator(path="../../docs/helix/02-design/contracts/security-mapping-request-v0.1.schema.json")]
struct RequestSchema;
#[derive(Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
struct Request {
    interface_version: String,
    modules: Vec<ModuleInput>,
    policy_json: String,
    ontology_json: String,
    query_profile_json: String,
    binding_json: String,
    backend_id: String,
    backend_version: String,
    target_profile: String,
    sql: String,
    parameters: Parameters,
    read_profile: Option<ReadProfile>,
}
fn fail() -> Diagnostic {
    Diagnostic::new("WFT-SECURITY-MAPPING-TRANSPORT", "input", "Security mapping transport refused")
}
// The resolver owns substitution and retains a parameter value at every actual
// AST occurrence. Bound that selected payload before it can be replicated.
// Conservative JSON control-character charges avoid allocating escaped copies.
fn parameter_payload_budget(sql:&str,parameters:&Parameters,limit:usize)->Result<()> {
    use crate::application_syntax::{Predicate,Value};
    let (syntax,_)=crate::application_syntax::parse_security(sql)?;
    let mut sizes=std::collections::BTreeMap::new();
    for (name,parameter) in parameters {
        let mut size=2usize; // JSON string quotes.
        for byte in parameter.value.bytes() {
            let charge=if byte<32 {6} else if byte==b'"'||byte==b'\\' {2} else {1};
            size=size.checked_add(charge).ok_or_else(fail)?;
        }
        if sizes.insert(name.to_ascii_lowercase(),size).is_some(){return Err(fail());}
    }
    let mut used=0usize;
    for predicate in syntax.predicates.iter().chain(syntax.joins.iter().flat_map(|(_,predicates)|predicates)) {
        let values=match predicate {Predicate::Compare{values,..}=>values,Predicate::HasRelated{key,..}=>key};
        for value in values {
            if let Value::Parameter(name)=value {
                if name.quoted{return Err(fail());}
                let size=*sizes.get(&name.value).ok_or_else(fail)?;
                used=used.checked_add(size).ok_or_else(fail)?;
                if used>limit{return Err(Diagnostic::new("WFT-SECURITY-MAPPING-PARAMETER-LIMIT","input",
                    "Selected parameter payload exceeds mapping preparation budget"));}
            }
        }
    }
    Ok(())
}
fn read_request(raw: &str) -> Result<Request> {
    if raw.len() > MAX_REQUEST_BYTES { return Err(fail()); }
    let value = checked_json(raw).map_err(|_| fail())?;
    if !RequestSchema::is_valid(&value) { return Err(fail()); }
    let request: Request = serde_json::from_value(value).map_err(|_| fail())?;
    if request.interface_version != "weft-security-mapping/0.1.0" { return Err(fail()); }
    Ok(request)
}
fn prepare(raw: &str) -> Result<Value> {
    let request=read_request(raw)?;
    let catalog = Catalog::prepare_security(request.modules)?;
    let packet = SecuritySourcePacket::read(&request.policy_json, &request.ontology_json, &catalog)?;
    let plan = SecurityLogicalPlan::read(packet, &catalog)?;
    let profile = SecurityQueryProfile::read(&request.query_profile_json, &plan, &catalog,
        &request.binding_json, &request.backend_id, &request.backend_version, &request.target_profile)?;
    parameter_payload_budget(&request.sql,&request.parameters,MAX_RESPONSE_BYTES)?;
    let query = SecurityResolvedQuery::resolve(&request.sql, &catalog, &plan,
        request.parameters, request.read_profile)?;
    let profiled = profile.admit_resolved_query(&query, &plan, &catalog, &request.binding_json,
        &request.backend_id, &request.backend_version, &request.target_profile)?;
    let handoff = profiled.mapping_handoff_json(&plan, &catalog, &request.binding_json,
        &request.backend_id, &request.backend_version, &request.target_profile)?;
    // Parse owner-generated JSON only; the transport does not accept caller IR,
    // scan annotations, action overrides, manifests or execution receipts.
    Ok(json!({"interfaceVersion":"weft-security-mapping/0.1.0", "status":"prepared",
        "securityReady":false, "handoff":serde_json::from_str::<Value>(&handoff).map_err(|_| fail())?}))
}
/// Pure original-source-bound cell/declaration inspection. This does not admit
/// backend registration, physical plans, source cuts, selection or release.
pub fn inspect_security_mapping_result_cells(raw:&str,contract:&crate::security_lowering::SecurityResultContract,contract_json:&str,contract_sha256:&str,batch_json:&str)->Result<()> {
    inspect_result(raw,contract,contract_json,contract_sha256,batch_json,None)
}
/// Conditional original-owner selection from declared facts, without native
/// provenance, population authentication or protected release authority.
pub fn inspect_security_mapping_result_facts(raw:&str,contract:&crate::security_lowering::SecurityResultContract,contract_json:&str,contract_sha256:&str,batch_json:&str,cut_json:&str,rows_json:&str)->Result<()> {
    inspect_result(raw,contract,contract_json,contract_sha256,batch_json,Some((cut_json,rows_json)))
}
fn inspect_result(raw:&str,contract:&crate::security_lowering::SecurityResultContract,contract_json:&str,contract_sha256:&str,batch_json:&str,facts:Option<(&str,&str)>)->Result<()> {
    let request=read_request(raw)?;
    let catalog=Catalog::prepare_security(request.modules)?;
    let packet=SecuritySourcePacket::read(&request.policy_json,&request.ontology_json,&catalog)?;
    let plan=SecurityLogicalPlan::read(packet,&catalog)?;
    let profile=SecurityQueryProfile::read(&request.query_profile_json,&plan,&catalog,&request.binding_json,&request.backend_id,&request.backend_version,&request.target_profile)?;
    parameter_payload_budget(&request.sql,&request.parameters,MAX_RESPONSE_BYTES)?;
    let query=SecurityResolvedQuery::resolve(&request.sql,&catalog,&plan,request.parameters,request.read_profile)?;
    let profiled=profile.admit_resolved_query(&query,&plan,&catalog,&request.binding_json,&request.backend_id,&request.backend_version,&request.target_profile)?;
    let context=crate::security_backend::SecurityBackendContext::new(&catalog,&plan,&query,&profiled,&request.binding_json,&request.backend_id,&request.backend_version,&request.target_profile)?;
    match facts {
        Some((cut,rows))=>context.check_simulated_fact_selection(contract,contract_json,contract_sha256,batch_json,cut,rows),
        None=>context.check_result_cells(contract,contract_json,contract_sha256,batch_json)
    }
}
/// Original source/query context for a pure trusted candidate preparation.
/// The callback cannot return references to temporary owner objects. Successful
/// preparation supplies no native source/caller/issuer or release authority.
pub fn with_security_mapping_context<T>(raw:&str,callback:impl for<'a> FnOnce(&crate::security_backend::SecurityBackendContext<'a>)->Result<T>)->Result<T> {
    let request=read_request(raw)?;
    let catalog=Catalog::prepare_security(request.modules)?;
    let packet=SecuritySourcePacket::read(&request.policy_json,&request.ontology_json,&catalog)?;
    let plan=SecurityLogicalPlan::read(packet,&catalog)?;
    let profile=SecurityQueryProfile::read(&request.query_profile_json,&plan,&catalog,&request.binding_json,&request.backend_id,&request.backend_version,&request.target_profile)?;
    parameter_payload_budget(&request.sql,&request.parameters,MAX_RESPONSE_BYTES)?;
    let query=SecurityResolvedQuery::resolve(&request.sql,&catalog,&plan,request.parameters,request.read_profile)?;
    let profiled=profile.admit_resolved_query(&query,&plan,&catalog,&request.binding_json,&request.backend_id,&request.backend_version,&request.target_profile)?;
    let context=crate::security_backend::SecurityBackendContext::new(&catalog,&plan,&query,&profiled,&request.binding_json,&request.backend_id,&request.backend_version,&request.target_profile)?;
    callback(&context).map_err(|d|crate::security_backend::callback_error(d,request.sql.len()))
}

/// Separate original authored relationship context. Source admission is not native execution authority.
pub fn with_security_relationship_context<T>(raw:&str,callback:impl for<'a> FnOnce(&crate::security_backend::SecurityBackendContext<'a>)->Result<T>)->Result<T> {
    let request=read_request(raw)?;
    let catalog=Catalog::prepare_security(request.modules)?;
    let packet=SecuritySourcePacket::read(&request.policy_json,&request.ontology_json,&catalog)?;
    let plan=SecurityLogicalPlan::read(packet,&catalog)?;
    let profile=SecurityQueryProfile::read_relationship_candidate(&request.query_profile_json,&plan,&catalog,&request.binding_json,&request.backend_id,&request.backend_version,&request.target_profile)?;
    parameter_payload_budget(&request.sql,&request.parameters,MAX_RESPONSE_BYTES)?;
    let query=SecurityResolvedQuery::resolve_relationship_candidate(&request.sql,&catalog,&plan,request.parameters,request.read_profile)?;
    let profiled=profile.admit_resolved_query(&query,&plan,&catalog,&request.binding_json,&request.backend_id,&request.backend_version,&request.target_profile)?;
    let context=crate::security_backend::SecurityBackendContext::new(&catalog,&plan,&query,&profiled,&request.binding_json,&request.backend_id,&request.backend_version,&request.target_profile)?;
    callback(&context).map_err(|d|crate::security_backend::callback_error(d,request.sql.len()))
}

/// Pure original-ontology subject Record/Key/source selection correspondence.
/// No native principal binding, key existence, issuer or release authority.
pub fn inspect_security_mapping_subject_key(raw:&str,selection_json:&str)->Result<()> {
    let refused=||crate::error::Diagnostic::new("WFT-SECURITY-SUBJECT-KEY","model","Original ontology subject key selection refused");
    let selection=crate::json::checked_json_bounded(selection_json,16_384,8,64).map_err(|_|refused())?;
    let object=selection.as_object().ok_or_else(refused)?;
    let identity=selection["identity"].as_object().ok_or_else(refused)?;
    if object.len()!=2||!object.contains_key("sourceSha256")||identity.len()!=3||!["module","element","key"].iter().all(|k|identity.get(*k).is_some_and(|v|v.as_str().is_some_and(|s|!s.is_empty()&&s.len()<=4096&&!s.contains('\0')))){return Err(refused());}
    let request=read_request(raw)?;
    let catalog=Catalog::prepare_security(request.modules)?;
    let packet=SecuritySourcePacket::read(&request.policy_json,&request.ontology_json,&catalog)?;
    let plan=SecurityLogicalPlan::read(packet,&catalog)?;
    let profile=SecurityQueryProfile::read(&request.query_profile_json,&plan,&catalog,&request.binding_json,&request.backend_id,&request.backend_version,&request.target_profile)?;
    parameter_payload_budget(&request.sql,&request.parameters,MAX_RESPONSE_BYTES)?;
    let query=SecurityResolvedQuery::resolve(&request.sql,&catalog,&plan,request.parameters,request.read_profile)?;
    profile.admit_resolved_query(&query,&plan,&catalog,&request.binding_json,&request.backend_id,&request.backend_version,&request.target_profile)?;
    let ontology=crate::security_ontology::SecurityOntologyClosure::read(plan.source(),&catalog)?;
    let subject=ontology.subject();let binding=ontology.types.get(subject).ok_or_else(refused)?;
    let pin=catalog.pins().into_iter().find(|p|p.document_id==subject.document_id).ok_or_else(refused)?;
    if selection["sourceSha256"]!=pin.sha256||selection["identity"]["module"]!=subject.module_id||selection["identity"]["element"]!=subject.element_id||selection["identity"]["key"]!=binding.source["keyId"]{return Err(refused());}
    Ok(())
}
/// Preserve the separate API and fail-closed ordinary security compile boundary.
/// Pure source correspondence response; never a protected execution credential.
pub fn inspect_security_mapping_subject_key_json(raw: &str, selection_json: &str) -> String {
    match inspect_security_mapping_subject_key(raw, selection_json) {
        Ok(()) => json!({"interfaceVersion":"weft-security-subject-selection/0.1.0",
            "status":"inspected","securityReady":false}).to_string(),
        Err(diagnostic) => json!({"interfaceVersion":"weft-security-subject-selection/0.1.0",
            "status":"refused","securityReady":false,"diagnostics":[diagnostic]}).to_string(),
    }
}

/// Input strings retain original sources; successful output is a mapping input,
/// not authenticated issuance, native SQL or a credential.
pub fn prepare_security_mapping_json(raw: &str) -> String {
    let result = prepare(raw).and_then(|value| {
        let response=value.to_string();
        if response.len()>MAX_RESPONSE_BYTES { Err(fail()) } else { Ok(response) }
    });
    match result {
        Ok(response) => response,
        Err(diagnostic) => json!({"interfaceVersion":"weft-security-mapping/0.1.0",
            "status":"refused", "securityReady":false, "diagnostics":[diagnostic]}).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::sha256;
    fn fixture() -> Value {
        let corpus:Value=serde_json::from_str(include_str!("../tests/security-evaluation-oracle.json")).unwrap();
        let case=corpus["cases"].as_array().unwrap().iter().find(|c|c["name"]=="query-predicate-original-grant").unwrap();
        let original:Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();
        let source=&original["resolution"]["documents"][0];
        let mut document=source["document"].clone();
        for field in document["modules"][0]["elements"].as_array_mut().unwrap() {
            field["name"]=field["id"].clone();
            if field["scalarType"]=="integer" {field["facets"]=json!({"integerWidth":{"bits":64,"signed":true}});}
        }
        let text=document.to_string();
        let modules=json!([{"documentJson":text,"pin":{"documentId":document["id"],"revision":source["revision"],
            "umfVersion":"0.8.0","sha256":sha256(text.as_bytes())},"selectedModuleIds":["m"]}]);
        let mut ontology=case["ontology"].clone();
        let salary=ontology["entities"].as_array_mut().unwrap().iter_mut().find(|e|e["type"]["elementId"]=="Resource").unwrap()
            ["fields"].as_array_mut().unwrap().iter_mut().find(|f|f["ref"]["elementId"]=="salary").unwrap();
        salary["queryUse"]=json!({"predicate":"original-authorized","order":"original-authorized",
            "group":"original-authorized","join":"original-authorized","aggregate":"original-authorized"});
        let policy=case["policy"].to_string();let ontology=ontology.to_string();
        let binding=json!({"version":"fixture-only","layout":"opaque-unqualified-storage"}).to_string();
        let entries:Vec<Value>=["predicate","order","group","join","aggregate"].iter().map(|operator|
            json!({"target":case["request"]["targets"][0],"field":case["request"]["queryUses"][0]["field"],
                "operator":operator,"originalAction":"query-original"})).collect();
        let profile=json!({"version":"weft.security.query-profile/0.1.0","id":"fixture-profile","revision":"p1","action":"read",
            "modelPins":[modules[0]["pin"]],"policySha256":sha256(policy.as_bytes()),"ontologySha256":sha256(ontology.as_bytes()),
            "binding":{"backendId":"fixture","backendVersion":"unqualified","targetProfile":"fixture-only","sha256":sha256(binding.as_bytes())},
            "targets":case["request"]["targets"],"bindings":entries}).to_string();
        json!({"interfaceVersion":"weft-security-mapping/0.1.0","modules":modules,"policyJson":policy,"ontologyJson":ontology,
            "queryProfileJson":profile,"bindingJson":binding,"backendId":"fixture","backendVersion":"unqualified","targetProfile":"fixture-only",
            "sql":"SELECT r.salary AS amount,SUM(r.salary) AS total FROM Resource r JOIN Resource s ON r.salary=s.salary WHERE r.salary=100 GROUP BY r.salary ORDER BY r.salary",
            "parameters":{}})
    }
    fn response(request:&Value)->Value {serde_json::from_str(&prepare_security_mapping_json(&request.to_string())).unwrap()}
    #[test]
    fn subject_selection_is_original_ontology_bound_and_closed() {
        let request=fixture();
        let ontology:Value=serde_json::from_str(request["ontologyJson"].as_str().unwrap()).unwrap();
        let subject=&ontology["subject"];
        let entity=ontology["entities"].as_array().unwrap().iter().find(|e|e["type"]==*subject).unwrap();
        let selection=json!({"sourceSha256":request["modules"][0]["pin"]["sha256"],
            "identity":{"module":subject["moduleId"],"element":subject["elementId"],"key":entity["keyId"]}});
        assert!(inspect_security_mapping_subject_key(&request.to_string(),&selection.to_string()).is_ok());
        let response:Value=serde_json::from_str(&inspect_security_mapping_subject_key_json(&request.to_string(),&selection.to_string())).unwrap();
        assert_eq!(response,json!({"interfaceVersion":"weft-security-subject-selection/0.1.0","status":"inspected","securityReady":false}));
        for member in ["module","element","key"] {
            let mut changed=selection.clone();changed["identity"][member]=json!("foreign");
            assert!(inspect_security_mapping_subject_key(&request.to_string(),&changed.to_string()).is_err());
        }
        let mut changed=selection.clone();changed["sourceSha256"]=json!("0".repeat(64));
        assert!(inspect_security_mapping_subject_key(&request.to_string(),&changed.to_string()).is_err());
        let response:Value=serde_json::from_str(&inspect_security_mapping_subject_key_json(&request.to_string(),&changed.to_string())).unwrap();
        assert_eq!(response["status"],"refused");assert_eq!(response["securityReady"],false);
        assert_eq!(response.as_object().unwrap().len(),4);
        assert_eq!(response["diagnostics"][0]["code"],"WFT-SECURITY-SUBJECT-KEY");
        let mut changed=selection.clone();changed["identity"]["extra"]=json!(true);
        assert!(inspect_security_mapping_subject_key(&request.to_string(),&changed.to_string()).is_err());
        let mut changed=request.clone();changed["sql"]=json!("SELECT absent FROM Resource");
        assert!(inspect_security_mapping_subject_key(&changed.to_string(),&selection.to_string()).is_err());
    }
    #[test]
    fn actual_sql_derives_all_five_operators_and_separate_action_closures() {
        let request=fixture();let result=response(&request);
        assert_eq!(result["status"],"prepared","{result}");assert_eq!(result["securityReady"],false);
        let handoff=&result["handoff"];
        assert_eq!(handoff["sqlSha256"],sha256(request["sql"].as_str().unwrap().as_bytes()));
        let uses=handoff["uses"].as_array().unwrap();assert_eq!(uses.len(),6);
        let actual:std::collections::BTreeSet<_>=uses.iter().map(|u|{
            assert_eq!(u["originalAction"],"query-original");assert_eq!(u["field"]["elementId"],"salary");
            (u["scan"].as_str().unwrap(),u["operator"].as_str().unwrap())}).collect();
        assert_eq!(actual,std::collections::BTreeSet::from([("s0","predicate"),("s0","order"),
            ("s0","group"),("s0","join"),("s0","aggregate"),("s1","join")]));
        assert_eq!(handoff["scans"].as_array().unwrap().len(),2);
        for scan in handoff["scans"].as_array().unwrap() {
            assert!(scan["actions"].as_array().unwrap().iter().any(|a|a["action"]=="read"));
            assert!(scan["actions"].as_array().unwrap().iter().any(|a|a["action"]=="query-original"));
        }
    }
    #[test]
    fn transport_refuses_caller_ir_action_override_and_source_drift() {
        let original=fixture();
        for key in ["handoff","applicationPlan","uses","originalAction","securityReady","manifest"] {
            let mut request=original.clone();request[key]=json!(true);assert_eq!(response(&request)["status"],"refused", "{key}");
        }
        for key in ["policyJson","ontologyJson","bindingJson","backendId","backendVersion","targetProfile"] {
            let mut request=original.clone();request[key]=json!(format!("{} ",request[key].as_str().unwrap()));
            assert_eq!(response(&request)["status"],"refused", "{key}");
        }
        let mut request=original.clone();request["modules"][0]["documentJson"]=json!(format!("{} ",request["modules"][0]["documentJson"].as_str().unwrap()));
        assert_eq!(response(&request)["status"],"refused");
        let mut profile:Value=serde_json::from_str(original["queryProfileJson"].as_str().unwrap()).unwrap();profile["bindings"]=json!([]);
        let mut request=original;request["queryProfileJson"]=json!(profile.to_string());assert_eq!(response(&request)["status"],"refused");
    }
    #[test]
    fn profile_source_is_retained_not_authenticated_by_the_transport() {
        let mut request=fixture();
        let changed=format!("{} ",request["queryProfileJson"].as_str().unwrap());
        request["queryProfileJson"]=json!(changed);
        let result=response(&request);assert_eq!(result["status"],"prepared");
        assert_eq!(result["handoff"]["profile"]["sha256"],sha256(changed.as_bytes()));
        assert_eq!(result["securityReady"],false);
    }
    #[test]
    fn duplicate_transport_keys_wrong_version_and_byte_overflow_refuse() {
        let original=fixture().to_string();
        let duplicate=format!("{{\"interfaceVersion\":\"weft-security-mapping/0.1.0\",{}",&original[1..]);
        for raw in [duplicate,original.replace("weft-security-mapping/0.1.0","weft-security-mapping/0.2.0")," ".repeat(MAX_REQUEST_BYTES+1)] {
            let value:Value=serde_json::from_str(&prepare_security_mapping_json(&raw)).unwrap();
            assert_eq!(value["status"],"refused");assert_eq!(value["securityReady"],false);assert!(value.get("handoff").is_none());
        }
    }
    #[test]
    fn parameter_payload_counts_owner_ast_occurrences_before_substitution() {
        let parameters:Parameters=serde_json::from_value(json!({"P":{"family":"string","value":"é\n\\"}})).unwrap();
        // Quotes2 + UTF8é2 + conservative newline6 + escaped slash2 =12.
        let sql="SELECT r.resourceId FROM Resource r WHERE r.resourceId=:p AND r.resourceId=:p";
        assert!(parameter_payload_budget(sql,&parameters,24).is_ok());
        assert_eq!(parameter_payload_budget(sql,&parameters,23).unwrap_err().code,"WFT-SECURITY-MAPPING-PARAMETER-LIMIT");
        // Literal spelling does not manufacture a parameter occurrence.
        assert!(parameter_payload_budget("SELECT r.resourceId FROM Resource r WHERE r.resourceId=':p'",&parameters,0).is_ok());
        let joined="SELECT r.resourceId FROM Resource r JOIN Resource s ON r.resourceId=:p WHERE s.resourceId=:p";
        assert_eq!(parameter_payload_budget(joined,&parameters,23).unwrap_err().code,"WFT-SECURITY-MAPPING-PARAMETER-LIMIT");
    }
    #[test]
    fn repeated_large_parameter_refuses_before_mapping_materialization() {
        let mut request=fixture();
        request["parameters"]=json!({"p":{"family":"string","value":"x".repeat(65536)}});
        request["sql"]=json!(format!("SELECT r.resourceId FROM Resource r WHERE {}",vec!["r.resourceId=:p";300].join(" AND ")));
        assert!(request.to_string().len()<MAX_REQUEST_BYTES);
        let result=response(&request);assert_eq!(result["status"],"refused");
        assert_eq!(result["diagnostics"][0]["code"],"WFT-SECURITY-MAPPING-PARAMETER-LIMIT");
        assert!(result.get("handoff").is_none());
    }
}
