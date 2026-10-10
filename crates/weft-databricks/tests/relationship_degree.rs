//! Compiler-only participation degree checks; no native engine qualification.
#[path="../../../tests/ashlar-databricks/common.rs"]
mod common;
use serde_json::{json,Value};
use weft_core::{backend::Registry,compile::Compiler,json::sha256};
use weft_databricks::candidate::Candidate;
use std::collections::BTreeSet;

fn request(sql:&str)->Value {
    let mut q=common::relationship_request(sql,false);
    let mut model:Value=serde_json::from_str(q["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    model["modules"][0]["relationships"][0]["targetMultiplicity"]=json!({"min":1,"max":1});
    let raw=model.to_string();let digest=sha256(raw.as_bytes());q["modules"][0]["documentJson"]=json!(raw);q["modules"][0]["pin"]["sha256"]=json!(digest);
    let mut b:Value=serde_json::from_str(q["target"]["bindingJson"].as_str().unwrap()).unwrap();
    b["modelPins"][0]["sha256"]=json!(digest);b["relationships"][0]["acceptedDefinition"]=model["modules"][0]["relationships"][0].clone();
    let raw=b.to_string();q["target"]["bindingJson"]=json!(raw);q["target"]["bindingSha256"]=json!(sha256(q["target"]["bindingJson"].as_str().unwrap().as_bytes()));q
}
fn compile(q:&Value)->Value {
    let mut registry=Registry::default();registry.register(Candidate).unwrap();
    serde_json::from_str(&Compiler{registry}.compile_json(&q.to_string())).unwrap()
}
#[test]
fn max_one_counts_distinct_pairs_without_deduplicating_occurrence_output() {
    // Independent fixture: two different occurrence IDs, one associated Record.
    let occurrences=[(101,1,7),(102,1,7)];
    assert_eq!(occurrences.len(),2);
    let second_neighbor=[(101,1,7),(102,1,8)];
    assert_eq!(second_neighbor.iter().map(|(_,s,t)|(*s,*t)).collect::<BTreeSet<_>>().len(),2,"A second distinct target must violate max1, unlike a parallel occurrence");
    assert_eq!(occurrences.iter().map(|(_,s,t)|(*s,*t)).collect::<BTreeSet<_>>().len(),1);
    let a=compile(&request("SELECT c.id,RELATED_KEYS(c.orders,1) AS related FROM Customer c"));
    assert_eq!(a["status"],"compiled","{a}");
    let guards=a["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.candidate.relationshipIntegrity").unwrap()["parameters"]["checks"].as_array().unwrap();
    let degree:Vec<_>=guards.iter().filter_map(|c|c["sql"].as_str()).filter(|s|s.contains(") degree ON")).collect();
    assert_eq!(degree.len(),2);
    assert!(degree.iter().all(|s|s.contains("SELECT DISTINCT source_id, target_id FROM")));
    assert!(degree.iter().any(|s|s.contains("associated GROUP BY source_id")&&s.contains(" > 1")));
    assert!(degree.iter().any(|s|s.contains("associated GROUP BY target_id")));
    assert!(degree.iter().all(|s|s.contains("LEFT JOIN")&&s.contains("coalesce(n, CAST(0 AS DECIMAL(38,0)))")));
    let sql=a["sql"].as_str().unwrap();assert!(!sql.contains("DISTINCT"));
    assert!(sql.contains("e.id ASC"));assert!(sql.contains("row_number()"));assert!(sql.contains("ord <= 2"));
    assert!(guards.iter().any(|c|c["sql"].as_str().unwrap().contains("SELECT id FROM")&&c["sql"].as_str().unwrap().contains("duplicates")));
}
#[test]
fn inverse_retains_forward_degree_orientation_and_exact_existential_bag() {
    let a=compile(&request("SELECT o.customer_id,RELATED_KEYS(o.customer,2) AS related FROM Orders o"));
    assert_eq!(a["status"],"compiled","{a}");
    let guards=a["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.candidate.relationshipIntegrity").unwrap()["parameters"]["checks"].as_array().unwrap();
    assert_eq!(guards.iter().filter(|c|c["sql"].as_str().unwrap().contains("SELECT DISTINCT source_id, target_id")).count(),2);
    let degree:Vec<_>=guards.iter().filter_map(|c|c["sql"].as_str()).filter(|s|s.contains(") degree ON")).collect();
    let forward=degree.iter().find(|s|s.contains("associated GROUP BY source_id")).unwrap();
    let reverse=degree.iter().find(|s|s.contains("associated GROUP BY target_id")).unwrap();
    assert!(forward.contains("coalesce(n, CAST(0 AS DECIMAL(38,0))) < 1"));
    assert!(forward.contains("coalesce(n, CAST(0 AS DECIMAL(38,0))) > 1"));
    assert!(reverse.contains("coalesce(n, CAST(0 AS DECIMAL(38,0))) < 0"));
    assert!(reverse.contains("coalesce(n, CAST(0 AS DECIMAL(38,0))) > 9223372036854775807"));
    assert!(!reverse.contains("coalesce(n, CAST(0 AS DECIMAL(38,0))) > 1"));
    assert!(degree.iter().all(|s|s.contains("LEFT JOIN")&&s.contains("degree ON v.__id=degree.")));

    let exists=compile(&request("SELECT c.id FROM Customer c WHERE HAS_RELATED(c.orders,KEY(7))"));
    assert_eq!(exists["status"],"compiled","{exists}");assert!(exists["sql"].as_str().unwrap().contains("EXISTS"));assert!(!exists["sql"].as_str().unwrap().contains("DISTINCT"));
}
