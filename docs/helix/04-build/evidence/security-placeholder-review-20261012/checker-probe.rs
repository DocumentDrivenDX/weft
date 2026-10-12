use serde_json::{json,Value};
use weft_core::{compile::Compiler,json::sha256};
fn request()->Value {
    let cases:Value=serde_json::from_str(include_str!("/private/tmp/weft-security-main-integration/docs/helix/03-test/fixtures/cases.json")).unwrap();
    let mut request=cases.as_array().unwrap().iter().find(|c|c["expected"]["status"]=="compiled").unwrap()["request"].clone();
    request["interfaceVersion"]=json!("weft-compile/0.5.0");request["dialect"]=json!("weft-sql/0.2.0");
    for module in request["modules"].as_array_mut().unwrap(){
        let mut doc:Value=serde_json::from_str(module["documentJson"].as_str().unwrap()).unwrap();doc["umf"]=json!("0.8.0");
        let text=doc.to_string();module["documentJson"]=json!(text);module["pin"]["umfVersion"]=json!("0.8.0");module["pin"]["sha256"]=json!(sha256(text.as_bytes()));
    }
    let fixture:Value=serde_json::from_str(include_str!("/private/tmp/weft-security-main-integration/crates/weft-core/tests/security-source-fixture.json")).unwrap();
    let doc=&fixture["resolution"]["documents"][0];let text=doc["document"].to_string();
    request["modules"]=json!([{"documentJson":text,"pin":{"documentId":doc["document"]["id"],"revision":doc["revision"],"umfVersion":"0.8.0","sha256":sha256(text.as_bytes())},"selectedModuleIds":["m"]}]);
    request["security"]=json!({"version":"umf.security/0.1.0","policyJson":fixture["policy"].to_string(),"ontologyJson":fixture["resolution"]["ontology"].to_string()});request
}
fn query_profile_request()->Value{
 let mut r=request();let mut doc:Value=serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();for element in doc["modules"][0]["elements"].as_array_mut().unwrap(){element["name"]=element["id"].clone();if element["scalarType"]=="integer"{element["facets"]=json!({"integerWidth":{"bits":64,"signed":true}});}}let text=doc.to_string();r["modules"][0]["documentJson"]=json!(text);r["modules"][0]["pin"]["sha256"]=json!(sha256(text.as_bytes()));r["sql"]=json!("SELECT r.resourceId FROM Resource r");let p=json!({"version":"weft.security.query-profile/0.1.0","id":"compile-profile-fixture","revision":"p1","action":"read","modelPins":r["modules"].as_array().unwrap().iter().map(|m|m["pin"].clone()).collect::<Vec<_>>(),"policySha256":sha256(r["security"]["policyJson"].as_str().unwrap().as_bytes()),"ontologySha256":sha256(r["security"]["ontologyJson"].as_str().unwrap().as_bytes()),"binding":{"backendId":r["target"]["backendId"],"backendVersion":r["target"]["backendVersion"],"targetProfile":r["target"]["targetProfile"],"sha256":r["target"]["bindingSha256"]},"targets":[{"documentId":"domain","moduleId":"m","elementId":"Resource"}],"bindings":[]});r["security"]["queryProfileJson"]=json!(p.to_string());r
}
fn main(){
 use weft_core::{security_backend::SecurityRegistry,security_lowering::*,ir::{Family,LogicalType},backend::ParameterSlot};
 let mut req=query_profile_request();req["interfaceVersion"]=json!("weft-security-compile/0.2.0");req["sql"]=json!("SELECT r.resourceId FROM Resource r WHERE r.resourceId='r1'");let mut calls=0;
 let raw=Compiler::default().compile_json_with_security_factory(&req.to_string(),&mut |ctx|{
  calls+=1;
  let mut plan=SecurityPhysicalPlan{version:"weft.security.physical-plan/0.1.0".into(),installation:vec![],execution:SecurityOperation{id:"execute".into(),sql:"SELECT $1".into(),parameters:vec![ParameterSlot{position:1,logical_type:LogicalType{family:Family::String,facets:json!({}),nullable:false},value:"r1".into(),origin:json!({"kind":"queryValue","sourcePath":"filters/0/right","sourceContextSha256":ctx.execution_query_context_sha256()})}]}};
  for sql in ["SELECT 1%-- $1\n1", "SELECT 1%/*$1*/1", "SELECT $1, $2$abc$2$", "SELECT $1😀", "SELECT $1, $😀$hello $2$😀$", "SELECT $1, U&'hello\\world' UESCAPE '!'", "SELECT E'foo'\n'\\\' $1 --'", "SELECT $1, $0$abc$0$", "SELECT $1, $01$abc$01$"] {
   plan.execution.sql=sql.into();
   println!("{sql:?} actual-checker={:?}",ctx.check_postgresql_query_placeholders(&plan));
  }
  Ok(SecurityRegistry::default())
 });
 assert_eq!(calls,1,"{raw}");
 let response:Value=serde_json::from_str(&raw).unwrap();
 println!("public compiler status={} diagnostic={}",response["status"],response["diagnostics"][0]["code"]);
}
