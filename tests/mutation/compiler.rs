// @covers US-010-AC1: first owner-backed portable UPDATE proof.
use serde_json::Value;
use weft_core::mutation::{MutationRegistry,ActionRegistration};
fn registry()->MutationRegistry{let value:Value=serde_json::from_str(include_str!("fixtures/registry.json")).unwrap();let actions:Vec<ActionRegistration>=serde_json::from_value(value["actions"].clone()).unwrap();MutationRegistry::new(actions).unwrap()}
#[test] fn original_preparation_compiles_update(){let request=include_str!("fixtures/request.json");let response:Value=serde_json::from_str(&registry().compile_json(request)).unwrap();assert_eq!(response["status"],"compiled","{response}");let plan:Value=serde_json::from_str(response["plan"]["utf8"].as_str().unwrap()).unwrap();assert_eq!(plan["inputs"]["assignments"][0]["value"]["boolean"],false);assert_eq!(plan["selection"]["dependencies"][0]["element"],"last-seen");}

fn request()->Value{serde_json::from_str(include_str!("fixtures/request.json")).unwrap()}
fn run(value:&Value)->Value{serde_json::from_str(&registry().compile_json(&value.to_string())).unwrap()}
// @covers US-010-AC1 exact bytes from independent UMF canonical writer/native SHA256.
#[test]fn independent_owner_golden(){let expected:Value=serde_json::from_str(include_str!("fixtures/golden.json")).unwrap();assert_eq!(run(&request()),expected);}
// @covers US-010-AC2 @covers US-010-AC3 @covers US-010-AC5
#[test]fn unsupported_and_malformed_requests_never_emit_a_plan(){let source=request();for sql in ["UPDATE Customer SET active=FALSE","UPDATE Customer SET active=NULL WHERE active=TRUE","UPDATE Customer SET id='other' WHERE active=TRUE","UPDATE Customer SET active=TRUE,active=FALSE WHERE active=TRUE","UPDATE Customer SET active=FALSE WHERE active<TRUE","UPDATE Customer SET active=FALSE WHERE last_seen<100.0","UPDATE Customer SET active=FALSE WHERE last_seen<01","UPDATE Customer SET active=FALSE WHERE last_seen<1e2","UPDATE Customer SET active=FALSE WHERE NOT active=TRUE","UPDATE Customer SET active=FALSE WHERE last_seen<100; SELECT x", "UPDATE Customer SET active=FALSE WHERE last_seen<100 -- comment"]{let mut r=source.clone();r["sql"]=serde_json::json!(sql);r["parameters"]=serde_json::json!({});let response=run(&r);assert_eq!(response["status"],"blocked","{sql}: {response}");assert!(response.get("plan").is_none());}
for key in ["credentials","preparation","registry"]{let mut r=source.clone();r[key]=serde_json::json!({});assert_eq!(run(&r)["status"],"blocked");}
let mut r=source.clone();r["parameters"]=serde_json::json!({});assert_eq!(run(&r)["status"],"blocked");r=source.clone();r["parameters"]["unused"]=serde_json::json!({"boolean":false});assert_eq!(run(&r)["status"],"blocked");r=source.clone();r["batchSize"]=serde_json::json!(101);assert_eq!(run(&r)["status"],"blocked");}
// @covers US-010-AC4 exact domains, explicit precedence and first-use dependencies.
#[test]fn canonical_predicates_and_exact_parameter_carriers(){let mut r=request();r["sql"]=serde_json::json!("UPDATE Customer SET active=FALSE WHERE (last_seen<:cutoff OR last_seen=99) AND active=TRUE");r["parameters"]["cutoff"]=serde_json::json!({"integerToken":"9007199254740993"});let response=run(&r);assert_eq!(response["status"],"compiled","{response}");let plan:Value=serde_json::from_str(response["plan"]["utf8"].as_str().unwrap()).unwrap();let selection:Value=serde_json::from_str(plan["selection"]["artifact"]["utf8"].as_str().unwrap()).unwrap();assert!(selection["predicate"]["and"][0]["or"].is_array(),"{selection}");assert_eq!(selection["predicate"]["and"][0]["or"][0]["compare"]["value"]["integerToken"],"9007199254740993");assert_eq!(plan["selection"]["dependencies"],serde_json::json!([{"module":"sales","element":"last-seen"},{"module":"sales","element":"active"}]));
r["sql"]=serde_json::json!("UPDATE Customer SET active=FALSE WHERE last_seen=:cutoff");for token in ["-0","1e2"]{r["parameters"]["cutoff"]=serde_json::json!({"integerToken":token});assert_eq!(run(&r)["status"],"compiled");}}
// @covers US-010-AC2 marker occurrences are bound to exact Field identity.
#[test]fn marker_reuse_and_source_pins_refuse(){let mut r=request();r["sql"]=serde_json::json!("UPDATE Customer SET active=:cutoff WHERE last_seen=:cutoff");r["parameters"]["cutoff"]=serde_json::json!({"boolean":false});assert_eq!(run(&r)["status"],"blocked");r=request();r["modelPin"]["revision"]=serde_json::json!("changed");assert_eq!(run(&r)["status"],"blocked");r=request();r["model"]["sha256"]=serde_json::json!("0".repeat(64));assert_eq!(run(&r)["status"],"blocked");}

// @covers US-010-AC1 @covers US-010-AC2 source SET order and exact same-shaped Field marker identity.
#[test]fn source_order_and_same_family_markers(){let config=include_str!("fixtures/multi-registry.json");let compiler=MutationRegistry::from_json(config).unwrap();let mut request:Value=serde_json::from_str(include_str!("fixtures/multi-request.json")).unwrap();let response:Value=serde_json::from_str(&compiler.compile_json(&request.to_string())).unwrap();assert_eq!(response["status"],"compiled","{response}");let plan:Value=serde_json::from_str(response["plan"]["utf8"].as_str().unwrap()).unwrap();assert_eq!(plan["inputs"]["assignments"][0]["input"],"enabled-value");request["sql"]=serde_json::json!("UPDATE Customer SET active=:shared,enabled=FALSE WHERE enabled=:shared");request["parameters"]=serde_json::json!({"shared":{"boolean":true}});let blocked:Value=serde_json::from_str(&compiler.compile_json(&request.to_string())).unwrap();assert_eq!(blocked["diagnostics"][0]["code"],"WFT-MUTATION-TYPE");}
// @covers US-010-AC5 parser grouping is bounded independently of decoded predicate depth.
#[test]fn grouping_collapse_and_unknown_registry_payloads(){let mut r=request();r["sql"]=serde_json::json!(format!("UPDATE Customer SET active=FALSE WHERE {}last_seen<:cutoff{}","(".repeat(64),")".repeat(64)));assert_eq!(run(&r)["status"],"compiled");let mut config:Value=serde_json::from_str(include_str!("fixtures/registry.json")).unwrap();config["actions"][0]["preparation"]["sha256"]=serde_json::json!("0".repeat(64));assert!(MutationRegistry::from_json(&config.to_string()).is_err());}

#[test]fn case_sensitive_markers_and_exact_grouping_bounds(){
let mut r=request();r["sql"]=serde_json::json!("uPdAtE CUSTOMER SeT ACTIVE=fAlSe WhErE LAST_SEEN<:Cutoff");r["parameters"]=serde_json::json!({"Cutoff":{"integerToken":"100"}});assert_eq!(run(&r)["status"],"compiled");r["parameters"]=serde_json::json!({"cutoff":{"integerToken":"100"}});assert_eq!(run(&r)["status"],"blocked");
r["sql"]=serde_json::json!("UPDATE Customer SET active=FALSE WHERE last_seen<:Cutoff AND last_seen>:cutoff");r["parameters"]=serde_json::json!({"Cutoff":{"integerToken":"100"},"cutoff":{"integerToken":"1"}});assert_eq!(run(&r)["status"],"compiled");
for depth in [128,129]{r=request();r["sql"]=serde_json::json!(format!("UPDATE Customer SET active=FALSE WHERE {}last_seen<:cutoff{}","(".repeat(depth),")".repeat(depth)));assert_eq!(run(&r)["status"],if depth==128{"compiled"}else{"blocked"});}
r=request();let atom=format!("{}last_seen<:cutoff{}","(".repeat(32),")".repeat(32));r["sql"]=serde_json::json!(format!("UPDATE Customer SET active=FALSE WHERE {}",vec![atom;64].join(" AND ")));assert_eq!(run(&r)["status"],"compiled");
}

#[test]
fn original_local_preparation_compiles_without_crossing_native_producer() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/local-products-input.json")).unwrap();
    let registry = MutationRegistry::from_json(&fixture["configuration"].to_string()).unwrap();
    let response: Value = serde_json::from_str(&registry.compile_json(&fixture["request"].to_string())).unwrap();
    assert_eq!(response["status"], "compiled", "{response}");
    let plan: Value = serde_json::from_str(response["plan"]["utf8"].as_str().unwrap()).unwrap();
    assert_eq!(plan["action"]["invocation"]["id"], "umf.actions.local-tx");
    assert_eq!(plan["inputs"]["assignments"][0]["value"]["string"], "Foo");
    let mut crossed = fixture["configuration"].clone();
    let mut preparation: Value = serde_json::from_str(crossed["actions"][0]["preparation"]["utf8"].as_str().unwrap()).unwrap();
    let native: Value = serde_json::from_str(include_str!("../../spec/upstream/actions-plan/owner-pins.json")).unwrap();
    preparation["producer"] = native["producer"].clone();
    let bytes = preparation.to_string();
    crossed["actions"][0]["preparation"]["utf8"] = bytes.clone().into();
    crossed["actions"][0]["preparation"]["sha256"] = weft_core::json::sha256(bytes.as_bytes()).into();
    assert!(MutationRegistry::from_json(&crossed.to_string()).is_err());
    // A rewritten preparation cannot relabel unchanged local source as native.
    let native_invocation = serde_json::json!({"id":"umf.actions.tx","version":"1"});
    crossed["actions"][0]["pin"]["invocation"] = native_invocation.clone();
    preparation["action"]["invocation"] = native_invocation;
    let bytes = preparation.to_string();
    crossed["actions"][0]["preparation"]["utf8"] = bytes.clone().into();
    crossed["actions"][0]["preparation"]["sha256"] = weft_core::json::sha256(bytes.as_bytes()).into();
    assert!(MutationRegistry::from_json(&crossed.to_string()).is_err());
}
