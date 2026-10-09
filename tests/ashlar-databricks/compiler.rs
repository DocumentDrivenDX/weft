// @covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 @covers US-004-AC4
// Compiler component assertions; native/host acceptance is separate.
mod common;
use serde_json::{json, Value};
use weft_core::{backend::Registry, compile::Compiler, json::sha256};
use weft_databricks::candidate::Candidate;
fn compiler() -> Compiler {
    let mut registry = Registry::default();
    registry.register(Candidate).unwrap();
    Compiler { registry }
}
fn run(request: &Value) -> Value {
    serde_json::from_str(&compiler().compile_json(&request.to_string())).unwrap()
}

#[test]
fn sales_compiles_with_pins_exact_carriers_and_prequery_integrity() {
    let request = common::request(common::SALES_SQL);
    let response = run(&request);
    assert_eq!(response["status"], "compiled", "{response}");
    assert_eq!(response["qualification"]["status"], "candidate");
    let sql = response["sql"].as_str().unwrap();
    assert!(sql.contains("VERSION AS OF 0"));
    assert!(sql.contains("SUM("));
    assert!(sql.contains("raise_error('WFT-NUMERIC-DOMAIN')"));
    assert!(sql.contains("MAX(1)"));
    assert!(!sql.contains("DISTINCT"));
    assert!(!sql.contains("weft-synthetic"));
    assert!(!sql.contains("fixture-schema-1"));
    assert_eq!(response["columns"][0]["outputName"], "name");
    assert_eq!(response["columns"][1]["outputName"], "total");
    assert_eq!(response["columns"][1]["decoder"], "exact-decimal");
    let integrity = response["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == "ashlar.candidate.scalarIntegrity")
        .unwrap();
    assert_eq!(
        integrity["parameters"]["checks"].as_array().unwrap().len(),
        4
    );
    assert_eq!(integrity["parameters"]["phase"], "before-user-query");
    assert!(response["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == "ashlar.candidate.publication"));
    let publication=response["obligations"].as_array().unwrap().iter()
        .find(|o| o["id"]=="ashlar.candidate.publication").unwrap();
    assert_eq!(publication["parameters"]["nativeProfile"],json!({
        "warehouseRelease":"unqualified","comparison":"UTF8_BINARY","arithmetic":"ANSI exact-or-error"
    }));
    assert_eq!(run(&request), response);
}
#[test]
fn missing_mapping_and_candidate_optout_return_no_sql() {
    let mut request = common::request(common::SALES_SQL);
    request["options"]["allowCandidate"] = json!(false);
    let result = run(&request);
    assert_eq!(result["status"], "blocked");
    assert!(result.get("sql").is_none());
    let mut request = common::request(common::SALES_SQL);
    let mut binding: Value =
        serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
    binding["records"][0]["properties"]
        .as_array_mut()
        .unwrap()
        .remove(1);
    let raw = binding.to_string();
    request["target"]["bindingJson"] = json!(raw);
    request["target"]["bindingSha256"] = json!(sha256(raw.as_bytes()));
    let result = run(&request);
    assert_eq!(result["status"], "blocked");
    assert!(result.get("sql").is_none());
}
#[test]
fn unicode_quotes_literal_and_boolean_are_bound_in_typed_slots() {
    let response=run(&common::request("SELECT c.name FROM Customer c WHERE c.name = 'x''; DROP TABLE fake;--é' AND c.active = TRUE"));
    assert_eq!(response["status"], "compiled", "{response}");
    assert!(!response["sql"].as_str().unwrap().contains("DROP TABLE"));
    assert!(response["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["value"] == "x'; DROP TABLE fake;--é"));
    assert!(response["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["value"] == "true"));
}

#[test]
fn global_sum_retains_nullable_exact_type_and_empty_input_branch() {
    let response = run(&common::request(
        "SELECT SUM(o.total) AS total FROM Orders o",
    ));
    assert_eq!(response["status"], "compiled", "{response}");
    assert_eq!(response["columns"][0]["nullable"], true);
    assert_eq!(
        response["columns"][0]["logicalType"]["facets"],
        json!({"scale":2})
    );
    assert_eq!(response["columns"][0]["decoder"], "exact-decimal");
    assert!(!response["sql"].as_str().unwrap().contains("GROUP BY"));
}

fn application(sql: &str) -> Value {
    let mut request = common::request(sql);
    request["interfaceVersion"] = json!("weft-compile/0.2.0");
    request["dialect"] = json!("weft-sql/0.2.0");
    request
}
#[test]
fn application_entity_and_count_keep_logical_metadata() {
    for sql in [
        "SELECT c.* FROM Customer c",
        "SELECT COUNT(*) AS total FROM Customer c",
        "SELECT c.name, COUNT(*) AS total FROM Customer c GROUP BY c.name",
        common::SALES_SQL,
    ] {
        let response = run(&application(sql));
        assert_eq!(response["status"], "compiled", "{response}");
        assert!(response["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["id"] == "ashlar.candidate.scalarIntegrity"));
        if sql.contains("COUNT") {
            let representation =
                &response["columns"].as_array().unwrap().last().unwrap()["representation"];
            assert_eq!(representation["decoder"], "exact-integer");
            assert_eq!(representation["logicalType"]["nullable"], false);
        }
        if sql == "SELECT c.* FROM Customer c" {
            assert_eq!(response["columns"].as_array().unwrap().len(), 3);
            assert_eq!(response["columns"][0]["outputName"], "id");
            assert_eq!(response["columns"][1]["outputName"], "name");
            assert_eq!(response["columns"][2]["outputName"], "active");
        }
    }
}
fn page_request() -> Value {
    let mut request =
        application("SELECT c.* FROM Customer c WHERE c.id > :after ORDER BY c.id ASC LIMIT 2");
    let mut document: Value =
        serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    document["modules"][0]["elements"][0]["keys"] = json!([{"id":"primary","name":"primary","primary":true,"fields":[{"module":"sales","element":"customer-id"}]}]);
    let raw = document.to_string();
    let digest = sha256(raw.as_bytes());
    request["modules"][0]["documentJson"] = json!(raw);
    request["modules"][0]["pin"]["sha256"] = json!(digest);
    let mut binding: Value =
        serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
    binding["modelPins"][0]["sha256"] = json!(digest);
    let raw = binding.to_string();
    request["target"]["bindingJson"] = json!(raw);
    request["target"]["bindingSha256"] = json!(sha256(raw.as_bytes()));
    request["parameters"] = json!({"after":{"family":"integer","value":"7"}});
    request["readProfile"] =
        json!({"version":"weft-application-read/0.2.0","subset":"entity-page"});
    request
}
#[test]
fn application_page_retains_authored_key_and_named_cursor() {
    let request=page_request();
    let response = run(&request);
    assert_eq!(response["status"], "compiled", "{response}");
    assert!(response["sql"].as_str().unwrap().contains("ORDER BY"));
    assert!(response["sql"].as_str().unwrap().contains("LIMIT 2"));
    assert!(response["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == "ashlar.candidate.keyIntegrity"));
}

fn optional_request(column: bool) -> Value {
    let mut request = application("SELECT c.name FROM Customer c");
    let mut document: Value =
        serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    let field = document["modules"][0]["elements"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["id"] == "customer-name")
        .unwrap();
    field["nullability"] = json!("absent-allowed");
    let raw = document.to_string();
    let digest = sha256(raw.as_bytes());
    request["modules"][0]["documentJson"] = json!(raw);
    request["modules"][0]["pin"]["sha256"] = json!(digest);
    let mut binding: Value =
        serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
    binding["modelPins"][0]["sha256"] = json!(digest);
    if column {
        binding["records"][0]["kind"] = json!("nodeProjection");
        binding["records"][0]["properties"][1]["home"] = json!({"kind":"column","value":"group_value","present":"group_present","nativeType":"STRING"});
    }
    let raw = binding.to_string();
    request["target"]["bindingJson"] = json!(raw);
    request["target"]["bindingSha256"] = json!(sha256(raw.as_bytes()));
    request
}

#[test]
fn optional_scalar_preserves_presence_envelope_in_both_owner_homes() {
    for column in [false, true] {
        let response = run(&optional_request(column));
        assert_eq!(response["status"], "compiled", "{response}");
        assert_eq!(response["columns"][0]["representation"]["kind"], "value");
        assert_eq!(
            response["columns"][0]["representation"]["nativeNull"],
            false
        );
        assert_eq!(response["columns"][0]["nullable"], false);
        let sql = response["sql"].as_str().unwrap();
        assert!(sql.contains("'absent'"));
        assert!(sql.contains("to_json(named_struct"));
        let check = response["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["id"] == "ashlar.candidate.scalarIntegrity")
            .unwrap()["parameters"]["checks"][0]["sql"]
            .as_str()
            .unwrap();
        if column {
            assert!(check.contains("group_present` = FALSE"));
            assert!(check.contains("group_value` IS NULL"));
        } else {
            assert!(check.contains("RLIKE '^OBJECT'"));
            assert!(check.contains("IS NULL"));
        }
    }
}

#[test]
fn authored_relationships_compile_forward_inverse_and_existential_without_deduplication() {
    for projection in [false, true] {
        for sql in [
            "SELECT c.id, RELATED_KEYS(c.orders, 2) AS related FROM Customer c",
            "SELECT o.customer_id, RELATED_KEYS(o.customer, 2) AS related FROM Orders o",
            "SELECT c.id FROM Customer c WHERE HAS_RELATED(c.orders, KEY(7))",
        ] {
            let response = run(&common::relationship_request(sql, projection));
            assert_eq!(response["status"], "compiled", "{response}");
            let sql = response["sql"].as_str().unwrap();
            assert!(!sql.contains("DISTINCT"));
            assert!(response["obligations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|o| o["id"] == "ashlar.candidate.relationshipIntegrity"));
            if sql.contains("row_number()") {
                assert!(sql.contains("<= 3"));
                assert_eq!(
                    response["columns"][1]["representation"]["kind"],
                    "relatedKeys"
                );
                assert!(sql.contains("'truncated'"));
            } else {
                assert!(sql.contains("EXISTS"));
            }
        }
    }
}

#[test]
fn relationship_binding_refuses_missing_changed_or_unknown_authored_meaning() {
    let base = common::relationship_request(
        "SELECT RELATED_KEYS(c.orders, 2) AS related FROM Customer c",
        false,
    );
    for mutation in 0..8 {
        let mut request = base.clone();
        let mut binding: Value =
            serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
        match mutation {
            0 => {
                binding.as_object_mut().unwrap().remove("relationships");
            }
            1 => {
                binding["relationships"][0]["acceptedDefinition"]["targetLifecycle"] =
                    json!("owned")
            }
            2 => binding["relationships"][0]["logical"]["extra"] = json!(true),
            3 => binding["relationships"][0]["target"] = common::identity("customer"),
            4 => binding["relationships"][0]["source"]["extra"] = json!(true),
            5 => binding["relationships"][0]["kind"] = json!("object"),
            6 => binding["relationships"][0]["sourceSystem"] = json!("foreign-scope"),
            _ => {
                let duplicate = binding["relationships"][0].clone();
                binding["relationships"]
                    .as_array_mut()
                    .unwrap()
                    .push(duplicate);
            }
        }
        let raw = binding.to_string();
        request["target"]["bindingJson"] = json!(raw);
        request["target"]["bindingSha256"] = json!(sha256(raw.as_bytes()));
        let response = run(&request);
        assert_eq!(response["status"], "blocked", "{response}");
        assert!(response.get("sql").is_none());
    }
}

#[test]
fn recursive_compounds_have_explicit_encoding_and_value_metadata() {
    for shape in ["sequence","map","nested","structured","cyclic"] {
        for optional in [false,true] {
            let response=run(&common::compound_request(shape,optional));
            assert_eq!(response["status"],"compiled","{response}");
            assert_eq!(response["columns"][0]["representation"]["kind"],"value");
            assert_eq!(response["columns"][0]["representation"]["nativeNull"],false);
            assert_eq!(response["columns"][0]["nullable"],false);
            let sql=response["sql"].as_str().unwrap();assert!(sql.contains("WITH RECURSIVE"));assert!(sql.contains("MAX RECURSION LEVEL 130"));
            assert!(response["obligations"].as_array().unwrap().iter().any(|o|o["id"]=="ashlar.candidate.compoundIntegrity"));
        }
    }
}

#[test]
fn compound_encoding_is_explicit_and_selected_dependency_meaning_refuses() {
    for mutation in 0..4 {
        let mut request=common::compound_request("structured",false);
        let mut binding:Value=serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
        let home=&mut binding["records"][0]["properties"].as_array_mut().unwrap().last_mut().unwrap()["home"];
        match mutation {
            0=>{home.as_object_mut().unwrap().remove("encoding");},
            1=>home["encoding"]=json!("unknown/1"),
            2=>*home=json!({"kind":"column","value":"group_value","present":"group_present","nativeType":"STRING"}),
            _=>{
                let mut doc:Value=serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
                let note=doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="note").unwrap();
                note["extensions"]["future.vendor"]=json!({"meaning":"unregistered"});
                let raw=doc.to_string();let digest=sha256(raw.as_bytes());
                request["modules"][0]["documentJson"]=json!(raw);request["modules"][0]["pin"]["sha256"]=json!(digest);binding["modelPins"][0]["sha256"]=json!(digest);
            },
        }
        let raw=binding.to_string();request["target"]["bindingJson"]=json!(raw);request["target"]["bindingSha256"]=json!(sha256(raw.as_bytes()));
        let response=run(&request);assert_eq!(response["status"],"blocked","{response}");assert!(response.get("sql").is_none());
    }
}

#[test]
fn candidate_presence_declaration_includes_compound_availability() {
    use weft_core::backend::Backend;
    let manifest = Candidate.describe().unwrap();
    let capability = manifest.capabilities.iter().find(|c| c.id == "value.presence").unwrap();
    assert_eq!(capability.logical_domain, json!({"subset":"optional scalar or compound envelopes; absent or exact value; explicit native null refuses"}));
}

// @covers US-004-AC1 @covers US-004-AC3 @covers US-006-AC4
#[test]
fn compound_generated_cte_names_do_not_collide_with_logical_scan_aliases() {
    for suffix in ["graph","walk","value"] {
        for uppercase in [false,true] {
            let alias=format!("__weft_codec_0_{suffix}");
            let alias=if uppercase {alias.to_ascii_uppercase()} else {alias};
            let mut request=common::compound_request("sequence",false);
            request["sql"]=json!(format!("SELECT \"{alias}\".payload FROM Customer \"{alias}\""));
            let response=run(&request);assert_eq!(response["status"],"compiled","{response}");
            let sql=response["sql"].as_str().unwrap().to_ascii_lowercase();
            let name=format!("`{}`",alias.to_ascii_lowercase());
            let declarations=sql.matches(&format!("{name} AS (" ).to_ascii_lowercase()).count()+sql.matches(&format!("{name}(owner_id,")).count();
            assert_eq!(declarations,1,"native CTE alias collision: {alias}\n{sql}");
            assert_eq!(response["logicalPlan"]["source"]["occurrence"],"s0");
            assert_eq!(sql.matches("`s0` as (").count(),1);
        }
    }
}

// @covers US-004-AC1 @covers US-004-AC2 @covers US-006-AC4
#[test]
fn native_scan_names_preserve_distinct_case_and_unicode_logical_occurrences() {
    for (first,second) in [("C","c"),("K","k")] {
        for app_version in [false,true] {
            let sql=format!("SELECT \"{first}\".id AS first, \"{second}\".id AS second FROM Customer \"{first}\" JOIN Customer \"{second}\" ON \"{first}\".id = \"{second}\".id");
            let request=if app_version {application(&sql)} else {common::request(&sql)};
            let response=run(&request);assert_eq!(response["status"],"compiled","{response}");
            let sql=response["sql"].as_str().unwrap();
            for alias in ["s0","s1"] {
                assert_eq!(sql.matches(&format!("`{alias}` AS (")).count(),1,"{sql}");
            }
            assert_eq!(response["columns"][0]["outputName"],"first");
            assert_eq!(response["columns"][1]["outputName"],"second");
        }
    }
}

// @covers US-004-AC3 @covers US-006-AC3
#[test]
fn structured_json_member_names_require_unique_non_nul_meaning() {
    for duplicate in [false,true] {
        let mut request=common::compound_request("structured",false);
        let mut doc:Value=serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
        let note=doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="note").unwrap();
        note["name"]=json!(if duplicate {"leaf"} else {"note\0"});
        let raw=doc.to_string();let digest=sha256(raw.as_bytes());
        request["modules"][0]["documentJson"]=json!(raw);request["modules"][0]["pin"]["sha256"]=json!(digest);
        let mut binding:Value=serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();binding["modelPins"][0]["sha256"]=json!(digest);
        let raw=binding.to_string();request["target"]["bindingJson"]=json!(raw);request["target"]["bindingSha256"]=json!(sha256(raw.as_bytes()));
        let response=run(&request);
        assert_eq!(response["diagnostics"][0]["code"],"WFT-CAPABILITY","{response}");
        assert_eq!(response["diagnostics"][0]["message"],"JSON record encoding needs unique non-NUL authored member names");
        assert!(response.get("sql").is_none());assert!(response.get("columns").is_none());
    }
}

// @covers US-004-AC3 @covers US-006-AC3
#[test]
fn selected_page_key_extension_meaning_is_not_silently_ignored() {
    let mut request=page_request();
    assert_eq!(run(&request)["status"],"compiled");
    let mut doc:Value=serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    doc["modules"][0]["elements"][0]["keys"][0]["futureMeaning"]=json!(true);
    let raw=doc.to_string();let digest=sha256(raw.as_bytes());
    request["modules"][0]["documentJson"]=json!(raw);request["modules"][0]["pin"]["sha256"]=json!(digest);
    let mut binding:Value=serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();binding["modelPins"][0]["sha256"]=json!(digest);
    let raw=binding.to_string();request["target"]["bindingJson"]=json!(raw);request["target"]["bindingSha256"]=json!(sha256(raw.as_bytes()));
    let response=run(&request);
    assert_eq!(response["diagnostics"][0]["code"],"WFT-BINDING","{response}");
    assert_eq!(response["diagnostics"][0]["message"],"Selected authored key meaning is not registered");
    assert!(response.get("sql").is_none());assert!(response.get("columns").is_none());
}

fn arithmetic_request(sql:&str)->Value {
    let mut request=common::request(sql);
    request["interfaceVersion"]=json!("weft-compile/0.3.0");request["dialect"]=json!("weft-sql/0.3.0");
    request["target"]["backendVersion"]=json!("0.3.0-arithmetic-candidate");request["target"]["targetProfile"]=json!(weft_databricks::arithmetic::PROFILE);request
}
fn arithmetic_compile(request:&Value)->Value {
    let mut registry=Registry::default();registry.register(weft_databricks::arithmetic::Arithmetic).unwrap();
    serde_json::from_str(&Compiler{registry}.compile_json(&request.to_string())).unwrap()
}
#[test]
fn arithmetic_coefficients_preserve_tokens_domains_and_every_guard_phase() {
    let request=arithmetic_request("SELECT c.id-1+2 AS next,o.total*12.5000 AS scaled FROM Customer c JOIN Orders o ON o.customer_id+1=c.id WHERE o.total*2>1 ORDER BY c.id");
    let response=arithmetic_compile(&request);assert_eq!(response["status"],"compiled","{response}");
    assert_eq!(response["logicalPlan"]["irVersion"],"weft-ir/0.3.0");
    assert_eq!(response["columns"][0]["representation"]["logicalType"]["facets"],json!({}));
    assert_eq!(response["columns"][1]["representation"]["logicalType"]["facets"],json!({"scale":6}));
    let slot=response["parameters"].as_array().unwrap().iter().find(|p|p["value"]=="12.5000").unwrap();
    assert_eq!(slot["logicalType"]["facets"],json!({"scale":4}));
    let exact=response["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.arithmetic.exact").unwrap();
    assert_eq!(exact["parameters"]["checks"].as_array().unwrap().iter().map(|c|c["phase"].as_str().unwrap()).collect::<Vec<_>>(),vec!["join-candidates","where-candidates","projection-survivors"]);
    let join=exact["parameters"]["checks"][0]["sql"].as_str().unwrap();assert!(join.contains("CROSS JOIN"));assert!(!join.contains("INNER JOIN"));assert!(join.contains("try_add("));
    for check in exact["parameters"]["checks"].as_array().unwrap(){assert!(check["sql"].as_str().unwrap().contains(" IS NULL"));}
    let sql=response["sql"].as_str().unwrap();assert!(sql.contains("try_multiply("));assert!(sql.contains("right("));assert!(!sql.contains("DOUBLE"));assert!(!sql.contains(" / "));
    assert_eq!(arithmetic_compile(&request),response);
}
#[test]
fn arithmetic_capacity_refuses_without_partial_artifacts() {
    for sql in ["SELECT c.id+100000000000000000000000000000000000000 AS next FROM Customer c","SELECT o.total*1.0000000000000000000 AS total FROM Orders o","SELECT SUM(o.total) AS total FROM Orders o"] {
        let response=arithmetic_compile(&arithmetic_request(sql));assert_eq!(response["status"],"blocked","{response}");assert_eq!(response["diagnostics"][0]["code"],"WFT-CAPABILITY","{response}");for key in ["sql","parameters","logicalPlan","columns","obligations"] {assert!(response.get(key).is_none())}
    }
}
#[test]
fn arithmetic_cancellation_and_where_conjuncts_keep_all_intermediate_checks() {
    let response=arithmetic_compile(&arithmetic_request("SELECT c.id*99999999999999999999999999999999999999-c.id*99999999999999999999999999999999999999 AS canceled FROM Customer c WHERE c.id=0 AND c.id*99999999999999999999999999999999999999>1"));
    assert_eq!(response["status"],"compiled","{response}");let checks=&response["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.arithmetic.exact").unwrap()["parameters"]["checks"];
    let before=checks[0]["sql"].as_str().unwrap();assert!(before.contains("try_multiply("));assert!(!before.rsplit(" FROM ").next().unwrap().contains(" AND "));let projection=checks[1]["sql"].as_str().unwrap();assert!(projection.contains("try_subtract("));assert!(projection.matches("try_multiply(").count()>=4);
}

#[test]
fn arithmetic_outputs_retain_all_field_identities_across_joined_scans() {
    let response=arithmetic_compile(&arithmetic_request("SELECT c.id+o.customer_id+c.id AS combined,o.total*12.5000 AS scaled,1+2 AS constant FROM Customer c JOIN Orders o ON o.customer_id=c.id"));
    assert_eq!(response["status"],"compiled","{response}");
    let outputs=response["logicalPlan"]["outputs"].as_array().unwrap();
    fn collect(value:&Value, identities:&mut Vec<Value>) {
        if value["op"]=="field" { identities.push(value["field"]["identity"].clone()); }
        match value { Value::Object(map)=>for child in map.values(){collect(child,identities)},Value::Array(items)=>for child in items{collect(child,identities)},_=>{} }
    }
    for index in [0,1] {
        let mut expected=vec![];collect(&outputs[index]["expression"],&mut expected);
        expected.sort_by_key(|id|id.to_string());expected.dedup();
        assert_eq!(expected.len(),if index==0 {2}else{1},"{response}");
        assert_eq!(response["columns"][index]["sourceIdentities"],json!(expected));
    }
    assert_eq!(response["columns"][2]["sourceIdentities"],json!([response["logicalPlan"]["source"]["record"].clone()]));
}

#[test]
fn arithmetic_bare_fields_resolve_original_members_without_expanding_old_profiles() {
    for sql in ["SELECT id FROM Customer WHERE id=1 ORDER BY id", "SELECT id+1 AS next FROM Customer", "SELECT total FROM Customer c JOIN Orders o ON o.customer_id=c.id", "SELECT total FROM Customer c JOIN Orders o ON customer_id=id"] {
        let response=arithmetic_compile(&arithmetic_request(sql));assert_eq!(response["status"],"compiled","{response}");
        for request in [common::request(sql),application(sql)] {assert_eq!(run(&request)["status"],"blocked");}
        assert!(response["columns"][0]["sourceIdentities"].as_array().unwrap()[0]["element"].is_string());
    }
    for (sql,code) in [("SELECT id FROM Customer c JOIN Customer other ON other.id=c.id","WFT-NAME-AMBIGUOUS"),("SELECT missing FROM Customer","WFT-NAME-MISSING"),("SELECT c.name FROM Customer c JOIN Customer other ON total=total JOIN Orders future ON future.customer_id=c.id","WFT-NAME-MISSING"),("SELECT id AS invented FROM Customer WHERE invented=1","WFT-NAME-MISSING")] {
        let response=arithmetic_compile(&arithmetic_request(sql));assert_eq!(response["status"],"blocked","{response}");assert_eq!(response["diagnostics"][0]["code"],code,"{response}");assert!(response.get("sql").is_none());
    }
}
#[test]
fn bare_field_ambiguity_precedes_unsupported_matching_member_meaning() {
    let mut request=arithmetic_request("SELECT id FROM Customer c JOIN Orders o ON o.customer_id=c.id");
    let mut document:Value=serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    let field=document["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="order-total").unwrap();field["name"]=json!("id");field["nullability"]=json!("nullable");
    let raw=document.to_string();let hash=sha256(raw.as_bytes());request["modules"][0]["documentJson"]=json!(raw);request["modules"][0]["pin"]["sha256"]=json!(hash);
    let mut binding:Value=serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();binding["modelPins"][0]["sha256"]=json!(hash);let raw=binding.to_string();request["target"]["bindingJson"]=json!(raw);request["target"]["bindingSha256"]=json!(sha256(raw.as_bytes()));
    let response=arithmetic_compile(&request);assert_eq!(response["diagnostics"][0]["code"],"WFT-NAME-AMBIGUOUS","{response}");assert!(response.get("sql").is_none());
    request["sql"]=json!("SELECT o.id FROM Customer c JOIN Orders o ON o.customer_id=c.id");
    let response=arithmetic_compile(&request);assert_eq!(response["diagnostics"][0]["code"],"WFT-TYPE","{response}");assert!(response.get("sql").is_none());
}

#[test]
fn bare_quoted_fields_preserve_case_punctuation_and_source_identity() {
    let mut request=arithmetic_request("SELECT \"odd.Name\" AS exact FROM Customer WHERE \"odd.Name\"='Ada'");
    let mut doc:Value=serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e|e["id"]=="customer-name").unwrap()["name"]=json!("odd.Name");
    let raw=doc.to_string();let hash=sha256(raw.as_bytes());request["modules"][0]["documentJson"]=json!(raw);request["modules"][0]["pin"]["sha256"]=json!(hash);
    let mut binding:Value=serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();binding["modelPins"][0]["sha256"]=json!(hash);let raw=binding.to_string();request["target"]["bindingJson"]=json!(raw);request["target"]["bindingSha256"]=json!(sha256(raw.as_bytes()));
    let response=arithmetic_compile(&request);assert_eq!(response["status"],"compiled","{response}");assert_eq!(response["columns"][0]["outputName"],"exact");assert_eq!(response["columns"][0]["sourceIdentities"],json!([common::identity("customer-name")]));
    request["sql"]=json!("SELECT \"ODD.Name\" FROM Customer");let response=arithmetic_compile(&request);assert_eq!(response["diagnostics"][0]["code"],"WFT-NAME-MISSING","{response}");
}

#[test]
fn bare_field_extension_does_not_expand_relationship_name_positions() {
    for sql in ["SELECT RELATED_KEYS(c.orders,4) AS related FROM Customer c", "SELECT c.id FROM Customer c WHERE HAS_RELATED(c.orders,KEY(1))"] {
        let response=arithmetic_compile(&arithmetic_request(sql));
        assert_eq!(response["status"],"blocked","{response}");
        assert_ne!(response["diagnostics"][0]["phase"],"parse","Qualified relationship grammar must remain accepted: {response}");
    }
    for sql in ["SELECT RELATED_KEYS(orders,4) AS related FROM Customer c", "SELECT c.id FROM Customer c WHERE HAS_RELATED(orders,KEY(1))"] {
        let response=arithmetic_compile(&arithmetic_request(sql));assert_eq!(response["status"],"blocked","{response}");assert_eq!(response["diagnostics"][0]["phase"],"parse","{response}");assert!(response.get("sql").is_none());
    }
}
