//! Compiler fixtures only: no native query, schema observation or host discharge.
use serde_json::{json, Value};
use weft_core::{backend03::Registry, compile::v04::Compiler, json::sha256};
use weft_databricks::paths::Paths;
#[path = "../../../tests/ashlar-databricks/common.rs"]
mod common;
fn request(sql: &str) -> Value {
    let mut v: Value = serde_json::from_str(include_str!(
        "../../../tests/ashlar-databricks/fixtures/original-commerce-path-request.json"
    ))
    .unwrap();
    v["sql"] = json!(sql);
    v
}
fn compile(v: &Value) -> Value {
    let mut registry = Registry::default();
    registry.register(Paths).unwrap();
    serde_json::from_str(&Compiler { registry }.compile_json(&v.to_string())).unwrap()
}
fn compiled(sql: &str) -> Value {
    let v = compile(&request(sql));
    assert_eq!(v["status"], "compiled", "{v}");
    v
}
fn obligation<'a>(v: &'a Value, id: &str) -> &'a Value {
    v["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == id)
        .unwrap()
}
const COLLECTION:&str="SELECT l.id, RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 2) AS paths FROM order_lines l ORDER BY l.id LIMIT 1";
const EXPANSION: &str =
    "CROSS JOIN EXPAND_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\") AS p";

#[test]
fn original_collection_retains_pins_keys_wide_order_and_closed_inventory() {
    let v = compiled(COLLECTION);
    let original = request(COLLECTION);
    assert_eq!(
        v["modelPins"],
        json!([original["modules"][0]["pin"].clone()])
    );
    let representation = &v["columns"][1]["representation"];
    assert_eq!(representation["kind"], "relatedPaths");
    assert_eq!(representation["edgeEncoding"], "signed64-decimal/0.1");
    assert_eq!(
        representation["path"]["hops"][1]["targetKey"]["fields"][0]["element"],
        "suppliers.id"
    );
    let sql = v["sql"].as_str().unwrap();
    assert!(sql.contains("TRY_SUM(CAST(1 AS DECIMAL(38,0))) OVER"));
    assert!(!sql.contains("row_number"));
    assert!(sql.contains("p.__edge1, p.__edge2 ROWS"));
    assert!(sql.ends_with("LIMIT 1"));
    let o = obligation(&v, "ashlar.path.occurrenceIntegrity");
    assert_eq!(o["parameters"]["paths"].as_array().unwrap().len(), 1);
    assert_eq!(o["parameters"]["checks"].as_array().unwrap().len(), 3);
    assert_eq!(o["parameters"]["edgeSchemas"].as_array().unwrap().len(), 2);
    assert!(o["parameters"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .all(|c| !c["sql"].as_str().unwrap().contains("LIMIT 1")));
    assert_eq!(v["qualification"]["status"], "candidate");
}
#[test]
fn path_counts_use_complete_filtered_bag_before_having_order_limit() {
    let sql=format!("SELECT l.product_id, COUNT(*) AS rows, COUNT_DISTINCT_PATH_TARGETS(p) AS targets, COUNT(DISTINCT l.id) AS lines FROM order_lines l JOIN products q ON l.product_id=q.id {EXPANSION} WHERE l.id='L1' GROUP BY l.product_id HAVING COUNT(DISTINCT l.id)>0 ORDER BY l.product_id LIMIT 1");
    let v = compiled(&sql);
    let checks = obligation(&v, "ashlar.path.countCapacity")["parameters"]["checks"]
        .as_array()
        .unwrap();
    assert_eq!(checks.len(), 2);
    for c in checks {
        let sql = c["sql"].as_str().unwrap();
        assert!(sql.contains("TRY_SUM(CAST(1 AS DECIMAL(38,0))) AS __n"));
        assert!(sql.contains("__path_input GROUP BY"));
        assert!(sql.contains("INNER JOIN"));
        assert!(sql.contains(" WHERE "));
        assert!(!sql.contains(" HAVING "));
        assert!(!sql.contains(" LIMIT "));
        assert!(!sql.contains("COUNT(*)"));
    }
    let targets = checks
        .iter()
        .find(|c| c["kind"] == "targetDistinct")
        .unwrap()["sql"]
        .as_str()
        .unwrap();
    assert!(targets.contains("SELECT DISTINCT"));
    let rows = checks.iter().find(|c| c["kind"] == "pathRows").unwrap()["sql"]
        .as_str()
        .unwrap();
    assert!(!rows.contains("SELECT DISTINCT"));
    assert!(v["sql"]
        .as_str()
        .unwrap()
        .contains(" HAVING COUNT(DISTINCT"));
}
#[test]
fn global_count_kind_inventory_and_empty_formula_are_selected() {
    for (outputs, kinds) in [
        ("COUNT(*) AS rows", vec!["pathRows"]),
        (
            "COUNT_DISTINCT_PATH_TARGETS(p) AS targets",
            vec!["targetDistinct"],
        ),
        (
            "COUNT(*) AS a, COUNT(*) AS b, COUNT_DISTINCT_PATH_TARGETS(p) AS c",
            vec!["pathRows", "targetDistinct"],
        ),
    ] {
        let v = compiled(&format!("SELECT {outputs} FROM order_lines l {EXPANSION}"));
        let checks = obligation(&v, "ashlar.path.countCapacity")["parameters"]["checks"]
            .as_array()
            .unwrap();
        assert_eq!(checks.len(), kinds.len());
        for kind in kinds {
            assert!(checks.iter().any(|c| c["kind"] == kind));
        }
        assert!(checks
            .iter()
            .all(|c| c["sql"].as_str().unwrap().contains("MAX(1) AS __nonempty")));
    }
}

#[test]
fn repeated_collections_keep_separate_roles_and_plain_expansion_has_no_count_guard() {
    let v=compiled("SELECT RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 2) AS a, RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 2) AS b FROM order_lines l");
    let o = &obligation(&v, "ashlar.path.occurrenceIntegrity")["parameters"];
    assert_eq!(o["paths"].as_array().unwrap().len(), 2);
    assert_eq!(o["checks"].as_array().unwrap().len(), 6);
    assert_eq!(o["edgeSchemas"].as_array().unwrap().len(), 4);
    assert_eq!(o["edgeSchemas"][2]["pathIndex"], 1);
    let v = compiled(&format!("SELECT l.id FROM order_lines l {EXPANSION}"));
    assert!(!v["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == "ashlar.path.countCapacity"));
    assert_eq!(
        obligation(&v, "ashlar.path.occurrenceIntegrity")["parameters"]["checks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn repeated_logical_labels_retain_the_exact_positioned_host_map() {
    let v=compiled("SELECT l.id, l.id, RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 2) AS paths FROM order_lines l");
    assert_eq!(v["columns"][0]["outputName"], v["columns"][1]["outputName"]);
    assert_ne!(
        v["columns"][0]["carrierName"],
        v["columns"][1]["carrierName"]
    );
    let map = &obligation(&v, "weft.output.positioned")["parameters"];
    assert_eq!(map.as_object().unwrap().len(), 2);
    assert_eq!(map["columns"].as_array().unwrap().len(), 3);
    assert_eq!(
        map["columns"][2]["sourceIdentities"],
        v["columns"][2]["sourceIdentities"]
    );
}

#[test]
fn distinct_composition_remains_required_string_only() {
    let v = compiled("SELECT DISTINCT l.id FROM order_lines l ORDER BY l.id");
    assert!(v["sql"]
        .as_str()
        .unwrap()
        .contains("ORDER BY `id` COLLATE UTF8_BINARY ASC"));
    let v = compile(&request(
        "SELECT DISTINCT l.quantity FROM order_lines l ORDER BY l.quantity",
    ));
    assert_eq!(v["status"], "blocked");
    assert!(v.get("sql").is_none());
}

fn synthetic_inverse(sql: &str) -> Value {
    let mut v = common::relationship_request(sql, false);
    let original = request(COLLECTION);
    v["interfaceVersion"] = original["interfaceVersion"].clone();
    v["dialect"] = original["dialect"].clone();
    for k in ["backendId", "backendVersion", "targetProfile"] {
        v["target"][k] = original["target"][k].clone();
    }
    v
}
#[test]
fn same_authored_edge_twice_uses_distinct_occurrence_aliases_and_both_directions() {
    for sql in [
        "SELECT RELATED_PATHS(c.orders, customer, 2) AS paths FROM Customer c",
        "SELECT RELATED_PATHS(o.customer, orders, 2) AS paths FROM Orders o",
    ] {
        let v = compile(&synthetic_inverse(sql));
        assert_eq!(v["status"], "compiled", "{v}");
        let path = &v["columns"][0]["representation"]["path"];
        assert_eq!(path["hops"][0]["identity"], path["hops"][1]["identity"]);
        assert_ne!(path["hops"][0]["inverse"], path["hops"][1]["inverse"]);
        let sql = v["sql"].as_str().unwrap();
        assert!(sql.contains(" e1 JOIN "));
        assert!(sql.contains(" e2 ON "));
        assert!(!sql.contains("e1.id<>e2.id"));
        assert!(sql.contains("source_id=m.__id") || sql.contains("target_id=m.__id"));
        assert_eq!(
            obligation(&v, "ashlar.path.occurrenceIntegrity")["parameters"]["edgeSchemas"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }
}

#[test]
fn authored_self_loop_can_reuse_the_same_edge_occurrence_at_both_hops() {
    // This is a separate declared sales fixture, never a replacement for the
    // original commerce model used above.
    let mut r =
        synthetic_inverse("SELECT RELATED_PATHS(c.orders, orders, 2) AS paths FROM Customer c");
    let mut document: Value =
        serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    document["modules"][0]["relationships"][0]["target"] =
        json!([{"module":"sales","element":"customer","key":"customer-pk"}]);
    let raw = document.to_string();
    let digest = sha256(raw.as_bytes());
    r["modules"][0]["documentJson"] = json!(raw);
    r["modules"][0]["pin"]["sha256"] = json!(digest);
    let mut binding: Value =
        serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();
    binding["modelPins"][0]["sha256"] = json!(digest);
    binding["relationships"][0]["acceptedDefinition"] =
        document["modules"][0]["relationships"][0].clone();
    binding["relationships"][0]["target"] = binding["relationships"][0]["source"].clone();
    let raw = binding.to_string();
    r["target"]["bindingJson"] = json!(raw);
    r["target"]["bindingSha256"] = json!(sha256(raw.as_bytes()));
    let v = compile(&r);
    assert_eq!(v["status"], "compiled", "{v}");
    let hops = &v["columns"][0]["representation"]["path"]["hops"];
    assert_eq!(hops[0], hops[1]);
    assert_eq!(hops[0]["from"], hops[0]["to"]);
    let sql = v["sql"].as_str().unwrap();
    assert!(sql.contains(" e1 JOIN "));
    assert!(sql.contains(" e2 ON "));
    assert!(!sql.contains("e1.id<>e2.id"));
}
#[test]
fn left_root_distinguishes_absence_and_retains_native_match_inventory() {
    let sql="SELECT a.id, RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 2) AS paths FROM products a LEFT JOIN order_lines l ON a.id=l.product_id ORDER BY a.id";
    let v = compiled(sql);
    let rep = &v["columns"][1]["representation"];
    assert!(rep.get("outerJoin").is_some());
    assert_eq!(rep["startRecord"]["element"], "order_lines");
    assert!(v["sql"].as_str().unwrap().contains("'_weft_match_id'") == false);
    assert!(v["sql"]
        .as_str()
        .unwrap()
        .contains("`_weft_match_id` IS NULL THEN to_json(named_struct('state','absent'))"));
    let check = &obligation(&v, "outerJoin.matchIntegrity")["parameters"]["scans"][0];
    assert_eq!(check["nativeType"], "BIGINT");
    assert_eq!(check["identityColumn"], "id");
    let v=compiled(&format!("SELECT a.id, COUNT(*) AS rows, COUNT_DISTINCT_PATH_TARGETS(p) AS targets FROM products a LEFT JOIN order_lines l ON a.id=l.product_id {EXPANSION} GROUP BY a.id"));
    let guard = obligation(&v, "ashlar.path.countCapacity")["parameters"]["checks"][0]["sql"]
        .as_str()
        .unwrap();
    assert!(guard.contains("LEFT JOIN"));
    assert!(guard.contains("INNER JOIN `__weft_path_0_rows`"));
}
#[test]
fn scalar_parameters_and_arithmetic_keep_original_tokens_and_separate_guards() {
    let mut r=request("SELECT l.quantity+1 AS plus, RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 2) AS paths FROM order_lines l WHERE l.id=:id");
    r["parameters"] = json!({"id":{"family":"string","value":"L'1"}});
    let v = compile(&r);
    assert_eq!(v["status"], "compiled", "{v}");
    assert!(!v["sql"].as_str().unwrap().contains("L'1"));
    assert!(v["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["value"] == "L'1"));
    assert_eq!(
        v["columns"][0]["representation"]["logicalType"]["facets"],
        json!({})
    );
    assert!(
        obligation(&v, "ashlar.arithmetic.exact")["parameters"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["phase"] == "projection-survivors")
    );
}
#[test]
fn explicit_profile_refuses_invalid_mapping_unmatched_numeric_and_old_routes() {
    let mut r = request(COLLECTION);
    r["options"]["allowCandidate"] = json!(false);
    assert_eq!(compile(&r)["diagnostics"][0]["code"], "WFT-CAPABILITY");
    let mut r = request(COLLECTION);
    let mut b: Value = serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();
    let rel = b["relationships"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["logical"]["relationship"] == "order_lines.product_id")
        .unwrap();
    rel["acceptedDefinition"]["targetMultiplicity"]["max"] = json!("*");
    let raw = b.to_string();
    r["target"]["bindingJson"] = json!(raw);
    r["target"]["bindingSha256"] = json!(sha256(raw.as_bytes()));
    assert_eq!(compile(&r)["diagnostics"][0]["code"], "WFT-BINDING");
    let r=request("SELECT l.quantity, RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 2) AS paths FROM products a LEFT JOIN order_lines l ON a.id=l.product_id");
    assert_eq!(compile(&r)["diagnostics"][0]["code"], "WFT-CAPABILITY");
    let old: Value = serde_json::from_str(
        &weft_core::compile::Compiler::default().compile_json(&request(COLLECTION).to_string()),
    )
    .unwrap();
    assert_eq!(old["diagnostics"][0]["code"], "WFT-VERSION");
}

// Independent lexical sanity on generated SQL; this deliberately does not
// interpret SQL or claim native acceptance.
fn balanced(sql: &str) {
    let bytes = sql.as_bytes();
    let mut depth = 0i64;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' | b'`' => {
                let quote = bytes[i];
                i += 1;
                loop {
                    assert!(i < bytes.len(), "unclosed quoted SQL token");
                    if bytes[i] == quote {
                        if i + 1 < bytes.len() && bytes[i + 1] == quote {
                            i += 2;
                            continue;
                        }
                        break;
                    }
                    i += 1;
                }
            }
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                assert!(depth >= 0, "extra close parenthesis");
            }
            _ => {}
        }
        i += 1;
    }
    assert_eq!(depth, 0, "unclosed SQL parentheses");
}
#[test]
fn generated_collection_and_guard_sql_are_lexically_closed() {
    for query in [
        COLLECTION,
        &format!("SELECT COUNT(*) AS rows FROM order_lines l {EXPANSION}"),
    ] {
        let v = compiled(query);
        balanced(v["sql"].as_str().unwrap());
        for o in v["obligations"].as_array().unwrap() {
            if let Some(checks) = o["parameters"]["checks"].as_array() {
                for c in checks {
                    if let Some(sql) = c["sql"].as_str() {
                        balanced(sql);
                    }
                }
            }
        }
    }
}

#[test]
fn paths_manifest_declares_only_reachable_initial04_operations() {
    let mut registry = Registry::default();
    registry.register(Paths).unwrap();
    let manifest = registry.manifest(weft_databricks::paths::ID).unwrap();
    let actual = manifest.capabilities.iter().map(|c| c.id.as_str()).collect::<Vec<_>>();
    let expected = [
        "scan",
        "project",
        "filter",
        "innerJoin",
        "equal",
        "and",
        "parameter.named",
        "compare.lexicographicGreater",
        "order.asc",
        "limit",
        "type.string",
        "type.boolean",
        "type.integer",
        "type.decimal",
        "value.presence",
        "value.nativeNull",
        "predicate.nativeNull",
        "compare.nullAwareStringEqual",
        "project.positionedOutputs",
        "project.distinct",
        "compare.less",
        "compare.lessEqual",
        "compare.greaterEqual",
        "compare.notEqual",
        "compare.scalarJoin",
        "arithmetic.exact.integer",
        "arithmetic.exact.decimal",
        "arithmetic.+",
        "arithmetic.-",
        "arithmetic.*",
        "arithmetic.negate",
        "arithmetic.compareExact",
        "type.integer.unbounded",
        "aggregate",
        "group",
        "aggregate.count",
        "aggregate.countDistinct",
        "aggregate.countDistinct.optional",
        "aggregate.havingCountDistinctGreater",
        "predicate.stringIn",
        "join.left",
        "value.outerJoinPresence",
        "relationship.twoHopPaths",
        "relationship.pathExpansion",
        "relationship.inverse",
        "result.pathOccurrences",
        "aggregate.pathTargetDistinctCount",
    ];
    assert_eq!(actual, expected);
    assert!(!actual.contains(&"key.uniqueStable"));
    let mut old = weft_core::backend::Registry::default();
    old.register(weft_databricks::candidate::Candidate).unwrap();
    let old_manifest = old.manifest("ashlar.databricks").unwrap();
    assert_eq!(old_manifest.interface_version, "weft-backend/0.2.0");
    assert!(old_manifest.capabilities.iter().any(|c| c.id == "key.uniqueStable"));
    let mut selected = request(COLLECTION);
    selected["readProfile"] = json!("related-entity-page");
    let refused = compile(&selected);
    assert_eq!(refused["status"], "blocked");
    assert_eq!(refused["diagnostics"][0]["code"], "WFT-INPUT");
    for pair in ["0.1.0", "0.2.0", "0.3.0"] {
        let mut original = request(COLLECTION);
        original["interfaceVersion"] = json!(format!("weft-compile/{pair}"));
        original["dialect"] = json!(format!("weft-sql/{pair}"));
        let refused = compile(&original);
        assert_eq!(refused["status"], "blocked");
        assert_eq!(refused["diagnostics"][0]["code"], "WFT-VERSION");
        assert_eq!(refused["interfaceVersion"], "weft-compile/0.4.0");
    }
}
