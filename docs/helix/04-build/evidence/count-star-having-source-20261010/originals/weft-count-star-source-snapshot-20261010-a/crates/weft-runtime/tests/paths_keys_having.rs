#![cfg(feature = "ashlar-databricks-paths-keys")]
use serde_json::{json, Value};
fn request(sql: &str) -> Value {
    let mut r: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/original-supply-chain-having/request.json"
    ))
    .unwrap();
    r["sql"] = json!(sql);
    r["interfaceVersion"] = json!("weft-compile/0.4.1");
    r["dialect"] = json!("weft-sql/0.4.1");
    r["target"]["backendVersion"] = json!("0.4.1-count-star-having-candidate");
    r
}
fn compile(r: &Value) -> Value {
    serde_json::from_str(&weft_runtime::paths_keys::compile_json(&r.to_string())).unwrap()
}
const ORIGINAL: &str =
    "SELECT upstream_event_id,COUNT(*) FROM events GROUP BY upstream_event_id HAVING COUNT(*)>1";
#[test]
fn original_replay_preserves_grouped_count_and_pre_having_capacity() {
    let r = request(ORIGINAL);
    let a = compile(&r);
    assert_eq!(a["status"], "compiled", "{a}");
    assert_eq!(a["dialect"], "weft-sql/0.4.1");
    assert_eq!(a["logicalPlan"]["irVersion"], "weft-ir/0.4.1");
    assert!(a["sql"]
        .as_str()
        .unwrap()
        .contains(" HAVING COUNT(*)>CAST("));
    assert!(a["logicalPlan"]["requiredCapabilities"]
        .as_array()
        .unwrap()
        .contains(&json!("aggregate.havingCountStarGreater")));
    let o = a["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == "ashlar.arithmetic.exact")
        .unwrap();
    assert_eq!(o["parameters"]["phase"], "before-user-query");
    let checks = o["parameters"]["checks"].as_array().unwrap();
    let c = checks
        .iter()
        .find(|c| c["phase"] == "aggregate-candidates")
        .unwrap();
    assert!(c["sql"].as_str().unwrap().contains("TRY_SUM"));
    assert!(!c["sql"].as_str().unwrap().contains("HAVING COUNT(*)>"));
    assert!(!a["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == "ashlar.path.countCapacity"));
}
#[test]
fn exact_threshold_boundaries_and_excluded_syntax() {
    for n in ["0", "1", "9223372036854775807"] {
        assert_eq!(
            compile(&request(&ORIGINAL.replace(">1", &format!(">{n}"))))["status"],
            "compiled"
        );
    }
    assert_eq!(
        compile(&request(&ORIGINAL.replace(">1", ">9223372036854775808")))["diagnostics"][0]
            ["code"],
        "WFT-CAPABILITY"
    );
    for predicate in [
        "COUNT(*)>=1",
        "COUNT(*)>:n",
        "COUNT(*)>-1",
        "COUNT(*)>1.0",
        "COUNT(upstream_event_id)>1",
        "COUNT(*)>1 AND COUNT(*)>2",
    ] {
        assert_eq!(
            compile(&request(&ORIGINAL.replace("COUNT(*)>1", predicate)))["status"],
            "blocked",
            "{predicate}"
        );
    }
    for sql in [
        "SELECT COUNT(*) FROM events HAVING COUNT(*)>1",
        "SELECT upstream_event_id FROM events GROUP BY upstream_event_id HAVING COUNT(*)>1",
    ] {
        assert_eq!(
            compile(&request(sql))["diagnostics"][0]["code"],
            "WFT-GROUPING"
        );
    }
}
#[test]
fn old_versions_and_mismatched_realizations_remain_closed() {
    let mut old = request(ORIGINAL);
    old["interfaceVersion"] = json!("weft-compile/0.4.0");
    old["dialect"] = json!("weft-sql/0.4.0");
    old["target"]["backendVersion"] = json!("0.4.0-paths-keys-candidate");
    let a = compile(&old);
    assert_eq!(a["diagnostics"][0]["code"], "WFT-UNSUPPORTED");
    assert_eq!(
        a["diagnostics"][0]["sourceSpan"],
        json!({"start":86,"end":87})
    );
    let mut mismatch = request(ORIGINAL);
    mismatch["dialect"] = json!("weft-sql/0.4.0");
    assert_eq!(compile(&mismatch)["diagnostics"][0]["code"], "WFT-VERSION");
    let mut backend = request(ORIGINAL);
    backend["target"]["backendVersion"] = json!("0.4.0-paths-keys-candidate");
    assert_eq!(compile(&backend)["status"], "blocked");
}
#[test]
fn exact_old_public_response_bytes_stay_unchanged() {
    let request =
        include_str!("../../../tests/fixtures/original-supply-chain-having/old-request.json");
    let prior =
        include_str!("../../../tests/fixtures/original-supply-chain-having/old-response.json");
    assert_eq!(
        weft_runtime::paths_keys::compile_json(request).as_bytes(),
        prior.trim_end_matches('\n').as_bytes()
    );
}
#[test]
fn complete_group_bag_guard_expression_and_bound_slot_are_retained() {
    let a = compile(&request(&format!(
        "{ORIGINAL} ORDER BY upstream_event_id LIMIT 1"
    )));
    assert_eq!(a["status"], "compiled", "{a}");
    let o = a["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == "ashlar.arithmetic.exact")
        .unwrap();
    assert_eq!(o["parameters"]["samePublicationRequired"], true);
    assert_eq!(o["parameters"]["noPartialPublication"], true);
    let checks = o["parameters"]["checks"].as_array().unwrap();
    let c = checks
        .iter()
        .find(|c| c["phase"] == "aggregate-candidates")
        .unwrap();
    let guard = c["sql"].as_str().unwrap();
    assert!(guard.contains("TRY_SUM(CAST(1 AS DECIMAL(38,0))) AS __n, MAX(1) AS __nonempty"));
    assert!(guard.contains("WHERE (__nonempty IS NOT NULL AND __n IS NULL) OR __n>CAST('9223372036854775807' AS DECIMAL(38,0))"));
    assert!(guard.contains(" GROUP BY `_path_key_0`"));
    assert!(!guard.contains("SELECT DISTINCT"));
    assert!(!guard.contains("LIMIT 1"));
    assert!(!guard.contains("ORDER BY"));
    let threshold = a["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| {
            p["origin"]["kind"] == "literal"
                && p["value"] == "1"
                && p["logicalType"]["family"] == "integer"
        })
        .unwrap();
    assert_eq!(
        threshold["logicalType"],
        json!({"family":"integer","facets":{},"nullable":false})
    );
    assert_eq!(
        threshold["origin"]["sourceSpan"],
        json!({"start":89,"end":90})
    );
    // Independent exact finite-bag oracle: duplicates count; empty input forms no group.
    use std::collections::BTreeMap;
    let mut counts = BTreeMap::new();
    for key in ["replay", "replay", "singleton"] {
        *counts.entry(key).or_insert(0u128) += 1;
    }
    assert_eq!(
        counts
            .iter()
            .filter(|(_, n)| **n > 1)
            .map(|(k, n)| (*k, *n))
            .collect::<Vec<_>>(),
        vec![("replay", 2)]
    );
    let violates = |n: u128| n > i64::MAX as u128;
    assert!(!violates(i64::MAX as u128));
    assert!(violates(i64::MAX as u128 + 1)); // A valid threshold cannot conceal positive row-count overflow; ORDER/LIMIT can.
    let populations = [("a", 1u128), ("z", i64::MAX as u128 + 1)];
    let visible = &populations[..1];
    assert!(visible.iter().all(|(_, n)| !violates(*n)));
    assert!(populations.iter().any(|(_, n)| violates(*n)));
}
#[test]
fn exact_old_distinct_public_artifact_bytes_stay_unchanged() {
    let request = include_str!(
        "../../../tests/fixtures/original-supply-chain-having/old-distinct-request.json"
    );
    let prior = include_str!(
        "../../../tests/fixtures/original-supply-chain-having/old-distinct-response.json"
    );
    assert_eq!(
        weft_runtime::paths_keys::compile_json(request).as_bytes(),
        prior.trim_end_matches('\n').as_bytes()
    );
    let mut next: Value = serde_json::from_str(request).unwrap();
    next["interfaceVersion"] = json!("weft-compile/0.4.1");
    next["dialect"] = json!("weft-sql/0.4.1");
    next["target"]["backendVersion"] = json!("0.4.1-count-star-having-candidate");
    let newer = compile(&next);
    assert_eq!(newer["status"], "compiled", "{newer}");
    let older: Value = serde_json::from_str(prior).unwrap();
    assert_eq!(
        newer["logicalPlan"]["having"],
        older["logicalPlan"]["having"]
    );
    assert_eq!(newer["sql"], older["sql"]);
    assert_eq!(newer["parameters"], older["parameters"]);
    assert_eq!(newer["columns"], older["columns"]);
    assert_eq!(newer["obligations"], older["obligations"]);
}
#[test]
fn original_model_binding_pins_and_cross_profile_refusals_are_explicit(){
 let r=request(ORIGINAL);let a=compile(&r);let binding:Value=serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();assert_eq!(a["modelPins"],binding["modelPins"]);assert_eq!(a["bindingSha256"],r["target"]["bindingSha256"]);let publication=a["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.candidate.publication").unwrap();assert_eq!(publication["parameters"]["publication"],binding["publication"]);
 let mut wrong=r.clone();wrong["target"]["targetProfile"]=json!("spark4-delta4-paths-candidate");assert_eq!(compile(&wrong)["diagnostics"][0]["code"],"WFT-BACKEND-VERSION");
 let mut wrong=r.clone();wrong["target"]["backendId"]=json!("ashlar.databricks.count-having");assert_eq!(compile(&wrong)["diagnostics"][0]["code"],"WFT-BACKEND-MISSING");
 let mut wrong=r;wrong["interfaceVersion"]=json!("weft-compile/0.4.0");assert_eq!(compile(&wrong)["diagnostics"][0]["code"],"WFT-VERSION");
}
