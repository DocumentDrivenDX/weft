use serde_json::{json, Value};
use std::fs;

const ROOT: &str = "/private/tmp/ashlar-weft-distribution-d2d";
fn read(relative: &str) -> String { fs::read_to_string(format!("{ROOT}/{relative}")).unwrap() }
fn request(relative: &str, sql: &str) -> Value {
    let mut r: Value = serde_json::from_str(&read(relative)).unwrap();
    r["interfaceVersion"] = json!("weft-compile/0.4.1");
    r["dialect"] = json!("weft-sql/0.4.1");
    r["target"]["backendId"] = json!("ashlar.databricks.paths-keys");
    r["target"]["backendVersion"] = json!("0.4.1-count-star-having-candidate");
    r["target"]["targetProfile"] = json!("spark4-delta4-paths-keys-candidate");
    r["sql"] = json!(sql);
    r
}
fn compile(r: &Value) -> Value { serde_json::from_str(&weft_runtime::paths_keys::compile_json(&r.to_string())).unwrap() }
fn main() {
    let original = "SELECT upstream_event_id,COUNT(*) FROM events GROUP BY upstream_event_id HAVING COUNT(*)>1";
    let fixture = "tests/fixtures/original-supply-chain-having/request.json";
    let base = request(fixture, original);
    let mut checks = vec![];
    let baseline = compile(&base);
    assert_eq!(baseline["status"], "compiled", "{baseline}");
    checks.push(json!({"case":"original-replay", "request":base, "response":baseline}));
    let joined = request(fixture, "SELECT e.upstream_event_id,COUNT(*) FROM events e JOIN events f ON e.upstream_event_id=f.upstream_event_id WHERE e.upstream_event_id='wanted' GROUP BY e.upstream_event_id HAVING COUNT(*)>1 ORDER BY e.upstream_event_id LIMIT 1");
    let output = compile(&joined);
    assert_eq!(output["status"], "compiled", "{output}");
    let arithmetic = output["obligations"].as_array().unwrap().iter().find(|x| x["id"]=="ashlar.arithmetic.exact").unwrap();
    let capacity = arithmetic["parameters"]["checks"].as_array().unwrap().iter().find(|x| x["phase"]=="aggregate-candidates").unwrap()["sql"].as_str().unwrap();
    assert!(capacity.contains("TRY_SUM(CAST(1 AS DECIMAL(38,0)))"));
    assert!(capacity.contains(" JOIN "));
    assert!(!capacity.contains("SELECT DISTINCT"));
    assert!(!capacity.contains("ORDER BY"));
    assert!(!capacity.contains("LIMIT 1"));
    assert!(!capacity.contains("HAVING COUNT(*)>"));
    assert!(output["parameters"].as_array().unwrap().iter().any(|x| x["value"]=="wanted"));
    checks.push(json!({"case":"joined-filtered-full-bag", "request":joined, "response":output}));
    let expanded = request("tests/ashlar-databricks/fixtures/original-commerce-path-request.json", "SELECT l.id,COUNT(*) FROM order_lines l CROSS JOIN EXPAND_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\") AS p GROUP BY l.id HAVING COUNT(*)>1 ORDER BY l.id LIMIT 1");
    let output = compile(&expanded);
    assert_eq!(output["status"], "compiled", "{output}");
    let path = output["obligations"].as_array().unwrap().iter().find(|x| x["id"]=="ashlar.path.countCapacity").unwrap();
    assert!(path["parameters"]["checks"].as_array().unwrap().iter().any(|x| x["kind"]=="pathRows"));
    for check in path["parameters"]["checks"].as_array().unwrap() { let sql=check["sql"].as_str().unwrap(); assert!(!sql.contains("HAVING COUNT(*)>")); assert!(!sql.contains("LIMIT 1")); }
    checks.push(json!({"case":"expansion-count-owner", "request":expanded, "response":output}));
    for (id, iface, dialect, backend) in [
        ("old-dialect-new-interface", "weft-compile/0.4.1", "weft-sql/0.4.0", "0.4.1-count-star-having-candidate"),
        ("new-dialect-old-interface", "weft-compile/0.4.0", "weft-sql/0.4.1", "0.4.0-paths-keys-candidate"),
        ("new-language-old-backend", "weft-compile/0.4.1", "weft-sql/0.4.1", "0.4.0-paths-keys-candidate"),
        ("old-language-new-backend", "weft-compile/0.4.0", "weft-sql/0.4.0", "0.4.1-count-star-having-candidate"),
    ] {
        let mut r=base.clone();r["interfaceVersion"]=json!(iface);r["dialect"]=json!(dialect);r["target"]["backendVersion"]=json!(backend);
        let out=compile(&r);assert_eq!(out["status"],"blocked","{id}: {out}");assert!(out.get("sql").is_none());
        checks.push(json!({"case":id,"request":r,"response":out}));
    }
    let mut opted=base.clone();opted["options"]["allowCandidate"]=json!(false);let out=compile(&opted);assert_eq!(out["status"],"blocked");assert!(out.get("sql").is_none());checks.push(json!({"case":"candidate-optout","request":opted,"response":out}));
    for (id, sql) in [
        ("unprojected-count", "SELECT upstream_event_id FROM events GROUP BY upstream_event_id HAVING COUNT(*)>1"),
        ("no-groups", "SELECT COUNT(*) FROM events HAVING COUNT(*)>1"),
        ("output-alias-not-source-field", "SELECT upstream_event_id AS event,COUNT(*) FROM events GROUP BY event HAVING COUNT(*)>1"),
    ] { let r=request(fixture,sql);let out=compile(&r);assert_eq!(out["status"],"blocked","{id}: {out}");assert!(out.get("sql").is_none());checks.push(json!({"case":id,"request":r,"response":out})); }
    for prefix in ["old", "old-distinct"] {
        let r=read(&format!("tests/fixtures/original-supply-chain-having/{prefix}-request.json"));
        let expected=read(&format!("tests/fixtures/original-supply-chain-having/{prefix}-response.json"));
        assert_eq!(weft_runtime::paths_keys::compile_json(&r), expected.trim_end_matches('\n'));
        checks.push(json!({"case":format!("{prefix}-exact-response-parity"),"passed":true}));
    }
    let huge=json!({"interfaceVersion":"weft-compile/0.4.1","padding":"x".repeat(16*1024*1024)}).to_string();
    let out:Value=serde_json::from_str(&weft_runtime::paths_keys::compile_json(&huge)).unwrap();
    assert_eq!(out["status"],"blocked");assert_eq!(out["diagnostics"][0]["code"],"WFT-LIMIT");
    checks.push(json!({"case":"oversized-valid-json-runtime","bytes":huge.len(),"response":out}));
    // A finite independent joined-bag oracle checks multiplicity rather than DISTINCT semantics.
    let lhs=[("g",true),("g",false),("single",true)];let rhs=["g","g","single"];
    let mut counts=std::collections::BTreeMap::new();for (k,keep) in lhs {for target in rhs {if k==target&&keep {*counts.entry(k).or_insert(0)+=1;}}}
    assert_eq!(counts.into_iter().filter(|(_,n)|*n>1).collect::<Vec<_>>(),vec![("g",2)]);
    println!("{}",json!({"scope":"public compiler source-only; no native execution","checks":checks}));
}
