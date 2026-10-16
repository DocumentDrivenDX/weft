//! Explicit cursor delivery. Original page custody is retained in full;
//! native snapshot, current authorization, fetching and cleanup belong to Truss.
use serde_json::{json,Value};
fn encode(value:&Value)->String { weft_core::mutation::canonical(value) }
fn compile(program:&str,current:&str)->Result<Value,&'static str> {
    if program.len()>4_194_304 || current.len()>4_194_304 {return Err("Bounded original program and current request required")}
    let original=weft_core::json::checked_json(program).map_err(|_|"Original program framing refused")?;
    let request=weft_core::json::checked_json(current).map_err(|_|"Current request framing refused")?;
    let checked:Value=serde_json::from_str(&crate::engine_read::check_page_json(program,current,"null")).map_err(|_|"Original page checker response refused")?;
    if checked["status"]!="compiled" {return Ok(json!({"version":"weft.engine-export/0.1","status":"blocked","original":checked}))}
    if checked["compiler"]!="truss-runtime" || request["target"]["backendVersion"]!="0.4.41" || request["target"]["targetProfile"]!="truss-runtime-pg18.6/0.5" {
        return Err("Explicit original Truss ABI1.1 page program required")
    }
    let size=checked["pageSize"].as_u64().filter(|v|(1..=999).contains(v)).ok_or("Original page bound required")? as u16;
    let mut derived=request.clone();
    derived["target"]["backendVersion"]=json!("0.4.42");
    derived["target"]["targetProfile"]=json!("truss-runtime-pg18.6-export/0.1");
    let response=weft_core::json::checked_json(&weft_postgresql::truss_runtime::compile_cursor_json(&encode(&derived),size)).map_err(|_|"Original cursor compiler response refused")?;
    if response["status"]!="compiled" {return Ok(json!({"version":"weft.engine-export/0.1","status":"blocked","original":response}))}
    if response["interfaceVersion"]!="weft-sql-cursor/0.1" {return Err("Explicit cursor response required")}
    let mut expected_plan=checked["artifact"]["logicalPlan"].clone();
    expected_plan["limit"]=Value::Null;
    let artifact=&response["artifact"];
    if artifact["logicalPlan"]!=expected_plan || artifact["columns"]!=checked["artifact"]["columns"]
        || artifact["parameters"]!=checked["artifact"]["parameters"] || artifact["modelPins"]!=checked["artifact"]["modelPins"]
        || artifact["bindingSha256"]!=checked["artifact"]["bindingSha256"]
        || response["cursor"]["keyOutputs"]!=checked["page"]["keyOutputs"]
        || response["cursor"]["visibleOutputs"]!=checked["page"]["visibleOutputs"]
        || response["cursor"]["batchSize"]!=size {
        return Err("Complete original authored-Key cursor correspondence required")
    }
    let mut out=json!({"version":"weft.engine-export/0.1","status":"compiled","compiler":"truss-runtime-cursor",
        "program":original,"checkedPage":checked,"request":request,"exportRequest":derived,
        "artifact":artifact,"cursor":response["cursor"]});
    out["structuralDigest"]=json!({"owner":"weft-runtime-engine-export","version":"0.1","sha256":weft_core::json::sha256(encode(&out).as_bytes())});
    Ok(out)
}
pub fn compile_json(program:&str,current_request:&str)->String {
    compile(program,current_request).unwrap_or_else(|message|json!({"version":"weft.engine-export/0.1","status":"blocked","diagnostics":[{"code":"WFT-ENGINE-EXPORT","message":message}]})).to_string()
}

#[cfg(test)] mod tests {
    use super::*;
    fn request()->Value {
        // Original accepted core0.8/native ABI1.1 capture; native execution is
        // separately retained in Truss's stream evidence packet.
        serde_json::from_str(include_str!("../../../tests/engine-catalog/native-truss-export-request.json")).unwrap()
    }
    fn page(request:&Value)->Value {serde_json::from_str(&crate::engine_read::compile_page_json(&json!({"version":"weft.engine-page/0.1","compiler":"truss-runtime","request":request,"pageSize":999}).to_string())).unwrap()}
    #[test] fn export_preserves_originals_and_emits_unbounded_ordered_plan() {
        let request=request();let original=page(&request);assert_eq!(original["status"],"compiled","{original}");
        let out=compile(&original.to_string(),&request.to_string()).unwrap();assert_eq!(out["status"],"compiled","{out}");
        assert_eq!(out["program"],original);assert_eq!(out["request"],request);assert_eq!(out["checkedPage"],original);
        assert_eq!(out["artifact"]["logicalPlan"]["limit"],Value::Null);
        assert_eq!(out["artifact"]["backend"]["backendVersion"],"0.4.42");
        assert_eq!(out["artifact"]["backend"]["targetProfile"],"truss-runtime-pg18.6-export/0.1");
        assert!(out["artifact"]["sql"].as_str().unwrap().contains(" ORDER BY "));
        // The ordinary artifact remains bounded. Nested native source functions
        // retain their own resource limits; only the outer result LIMIT is absent.
        assert!(original["artifact"]["sql"].as_str().unwrap().ends_with(" LIMIT 1000"));
        assert!(!out["artifact"]["sql"].as_str().unwrap().ends_with(" LIMIT 1000"));
        assert!(out["artifact"]["obligations"].as_array().unwrap().iter().any(|o|o["id"]=="truss.runtime.cursor-export/0.1"));
        let ordinary:Value=serde_json::from_str(&weft_postgresql::truss_runtime::compile_json(&out["exportRequest"].to_string())).unwrap();assert_eq!(ordinary["status"],"blocked");
        let mut rebound=request.clone();rebound["parameters"]["status"]["value"]=json!("bound ' value");
        let bound=compile(&original.to_string(),&rebound.to_string()).unwrap();assert_eq!(bound["status"],"compiled","{bound}");assert_eq!(bound["request"],rebound);assert_eq!(bound["program"],original);
    }
    #[test] fn altered_program_context_and_source_refuse_without_sql() {
        let request=request();let original=page(&request);
        for mode in 0..8 {let mut program=original.clone();let mut current=request.clone();match mode {
            0=>program["artifact"]["sql"]=json!("SELECT 1"),1=>program["page"]["keyOutputs"]=json!([0]),
            2=>program["after"]=json!(["a"]),3=>current["sql"]=json!("SELECT 1"),
            4=>current["target"]["bindingSha256"]=json!("0".repeat(64)),5=>current["parameters"]["status"]["family"]=json!("integer"),
            6=>current["modules"][0]["documentJson"]=json!("{}"),_=>current["target"]["backendVersion"]=json!("0.4.42")};
            let value:Value=serde_json::from_str(&compile_json(&program.to_string(),&current.to_string())).unwrap();assert_eq!(value["status"],"blocked","{mode}: {value}");assert!(value.get("artifact").is_none());
        }
        for (program,current) in [("{}","{}"),("{\"x\":0,\"x\":1}","{}"),("{}","{\"x\":0,\"x\":1}")] {let value:Value=serde_json::from_str(&compile_json(program,current)).unwrap();assert_eq!(value["status"],"blocked");}
    }
}
