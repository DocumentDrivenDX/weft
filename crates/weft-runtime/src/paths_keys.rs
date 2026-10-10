//! Explicit 0.4 composition; hosts still own execution and publication custody.
use weft_core::{
    backend03::{Plan04View, Registry},
    compile::{v04::Compiler, CompositionInput},
    error::Result,
    model::Catalog,
};

/// Compile only through the selected Backend03 path adapter. The existing runtime
/// entrypoint and registrations remain governed by their original interfaces.
/// This candidate composition supplies no native conformance or engine execution.
pub fn compile_json(request: &str) -> String {
    let mut factory =
        |_: &Catalog, _: Plan04View<'_>, _: CompositionInput<'_>| -> Result<Registry> {
            let mut registry = Registry::default();
            registry.register(weft_databricks::paths_keys::PathsKeys)?;
            Ok(registry)
        };
    Compiler::default().compile_json_with_factory(request, &mut factory)
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    fn request(sql: &str) -> Value {
        let mut r: Value = serde_json::from_str(include_str!(
            "../../../tests/ashlar-databricks/fixtures/original-commerce-path-request.json"
        ))
        .unwrap();
        r["sql"] = json!(sql);
        r["target"]["backendId"] = json!(weft_databricks::paths_keys::ID);
        r["target"]["backendVersion"] = json!(weft_databricks::paths_keys::VERSION);
        r["target"]["targetProfile"] = json!(weft_databricks::paths_keys::PROFILE);
        r
    }
    fn compile(r: &Value) -> Value {
        serde_json::from_str(&super::compile_json(&r.to_string())).unwrap()
    }
    #[test]
    fn original_required_onehop_emits_exact_wide_collection_and_standalone_schema() {
        let r=request("SELECT p.id, RELATED_KEYS(p.\"products.supplier_id\", 2) AS suppliers FROM products p ORDER BY p.id");
        let a = compile(&r);
        assert_eq!(a["status"], "compiled", "{a}");
        assert_eq!(a["backend"]["backendId"], weft_databricks::paths_keys::ID);
        let sql = a["sql"].as_str().unwrap();
        assert!(sql.contains("TRY_SUM(CAST(1 AS DECIMAL(38,0))) OVER"));
        assert!(!sql.contains("ROW_NUMBER"));
        let obligations = a["obligations"].as_array().unwrap();
        let collection = obligations
            .iter()
            .find(|o| o["id"] == "ashlar.relatedKeys.collectionIntegrity")
            .unwrap();
        assert_eq!(
            collection["parameters"]["collections"][0]["outputPosition"],
            2
        );
        assert_eq!(
            collection["parameters"]["edgeSchemas"][0]["identityColumn"],
            "id"
        );
        assert_eq!(
            collection["parameters"]["edgeSchemas"][0]["nativeType"],
            "BIGINT"
        );
        let guard = collection["parameters"]["checks"][0]["sql"]
            .as_str()
            .unwrap();
        assert!(guard.contains("WHERE id IS NULL"));
        assert!(guard.contains("GROUP BY id HAVING"));
        assert!(!guard.ends_with("LIMIT 2"));
        let ordinal = obligations
            .iter()
            .find(|o| o["id"] == "ashlar.relatedKeys.ordinalCapacity")
            .unwrap();
        assert_eq!(ordinal["parameters"]["nativeRepresentation"], "decimal38");
        assert_eq!(
            ordinal["parameters"]["maximum"],
            "99999999999999999999999999999999999999"
        );
        assert!(ordinal["parameters"]["checks"][0]["sql"]
            .as_str()
            .unwrap()
            .contains("WHERE __ordinal IS NULL"));
        assert_eq!(a["columns"][1]["representation"]["kind"], "relatedKeys");
    }
    #[test]
    fn twohop_collection_retains_original_obligations_and_adds_full_prefix_capacity() {
        let r=request("SELECT l.id, RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 2) AS paths FROM order_lines l ORDER BY l.id LIMIT 1");
        let a = compile(&r);
        assert_eq!(a["status"], "compiled", "{a}");
        let o = a["obligations"].as_array().unwrap();
        assert!(o
            .iter()
            .any(|x| x["id"] == "ashlar.path.occurrenceIntegrity"));
        let wide = o
            .iter()
            .find(|x| x["id"] == "ashlar.relatedKeys.ordinalCapacity")
            .unwrap();
        assert_eq!(
            wide["parameters"]["collections"],
            json!([{"outputPosition":2,"kind":"relatedPaths"}])
        );
        assert!(!o
            .iter()
            .any(|x| x["id"] == "ashlar.relatedKeys.collectionIntegrity"));
    }
    #[test]
    fn registration_is_explicit_candidate_optin_and_namespace_stays04() {
        use weft_core::backend03::Backend;
        let old = weft_databricks::paths::Paths.describe().unwrap();
        let new = weft_databricks::paths_keys::PathsKeys.describe().unwrap();
        assert_eq!(old.capabilities.len(), 47);
        assert_eq!(new.capabilities.len(), 48);
        assert_eq!(
            new.capabilities[..47]
                .iter()
                .map(|c| &c.id)
                .collect::<Vec<_>>(),
            old.capabilities.iter().map(|c| &c.id).collect::<Vec<_>>()
        );
        let mut r = request("SELECT p.id FROM products p");
        r["options"]["allowCandidate"] = json!(false);
        assert_eq!(compile(&r)["status"], "blocked");
        r["options"]["allowCandidate"] = json!(true);
        r["target"]["targetProfile"] = json!(weft_databricks::paths::PROFILE);
        assert_eq!(compile(&r)["status"], "blocked");
        let a: Value = serde_json::from_str(&super::compile_json(
            "{\"interfaceVersion\":\"weft-compile/0.3.0\",\"dialect\":\"weft-sql/0.3.0\"}",
        ))
        .unwrap();
        assert_eq!(a["diagnostics"][0]["code"], "WFT-VERSION");
    }
    #[test]
    fn repeated_and_mixed_collections_keep_output_order_and_complete_capacity_inventory() {
        for (sql,expected) in [
            ("SELECT RELATED_KEYS(p.\"products.supplier_id\", 1) AS a, RELATED_KEYS(p.\"products.supplier_id\", 2) AS b FROM products p",json!([{"outputPosition":1,"kind":"relatedKeys"},{"outputPosition":2,"kind":"relatedKeys"}])),
            ("SELECT RELATED_KEYS(l.\"order_lines.product_id\", 2) AS products, RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 2) AS suppliers FROM order_lines l",json!([{"outputPosition":1,"kind":"relatedKeys"},{"outputPosition":2,"kind":"relatedPaths"}]))
        ] {
            let a=compile(&request(sql));assert_eq!(a["status"],"compiled","{a}");
            let o=a["obligations"].as_array().unwrap();let wide=o.iter().find(|x|x["id"]=="ashlar.relatedKeys.ordinalCapacity").unwrap();
            assert_eq!(wide["parameters"]["collections"],expected);assert_eq!(wide["parameters"]["checks"].as_array().unwrap().len(),2);
            let one=o.iter().find(|x|x["id"]=="ashlar.relatedKeys.collectionIntegrity").unwrap();
            assert_eq!(one["parameters"]["collections"].as_array().unwrap().len(),if sql.contains("RELATED_PATHS"){1}else{2});
            assert!(!a["sql"].as_str().unwrap().contains("ROW_NUMBER"));
        }
    }
    #[test]
    fn bound_and_unsupported_composition_controls_are_independent() {
        for bound in [1, 1000] {
            let a=compile(&request(&format!("SELECT RELATED_KEYS(p.\"products.supplier_id\", {bound}) AS suppliers FROM products p")));
            assert_eq!(a["status"], "compiled", "{a}");
            assert_eq!(a["columns"][0]["representation"]["bound"], bound);
        }
        for sql in [
            "SELECT RELATED_KEYS(p.\"products.supplier_id\", 0) AS suppliers FROM products p",
            "SELECT RELATED_KEYS(p.\"products.supplier_id\", 1001) AS suppliers FROM products p",
            "SELECT COUNT(*), RELATED_KEYS(p.\"products.supplier_id\", 2) AS suppliers FROM products p",
            "SELECT p.id, RELATED_KEYS(p.\"products.supplier_id\", 2) AS suppliers FROM products p GROUP BY p.id",
            "SELECT RELATED_KEYS(p.\"products.supplier_id\", 2) AS suppliers FROM order_lines l LEFT JOIN products p ON l.product_id = p.id",
            "SELECT RELATED_KEYS(l.\"order_lines.product_id\", 2) AS products FROM order_lines l CROSS JOIN EXPAND_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\") AS p"
        ] {let a=compile(&request(sql));assert_eq!(a["status"],"blocked","{sql}: {a}");assert!(a.get("sql").is_none());}
    }
    #[test]
    fn scalar_arithmetic_join_and_expanded_grouped_counts_remain_selected() {
        for sql in [
            "SELECT l.id, l.quantity + 1 AS quantity FROM order_lines l",
            "SELECT l.id, p.id AS product FROM order_lines l JOIN products p ON l.product_id = p.id",
            "SELECT l.id, COUNT(*) AS rows FROM order_lines l CROSS JOIN EXPAND_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\") AS p GROUP BY l.id ORDER BY l.id",
            "SELECT COUNT_DISTINCT_PATH_TARGETS(p) AS suppliers FROM order_lines l CROSS JOIN EXPAND_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\") AS p"
        ] {let a=compile(&request(sql));assert_eq!(a["status"],"compiled","{sql}: {a}");}
    }

    fn authored_variant(r: &mut Value, change: impl FnOnce(&mut Value)) {
        use weft_core::json::sha256;
        let mut d: Value =
            serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
        change(&mut d);
        let raw = d.to_string();
        let hash = sha256(raw.as_bytes());
        r["modules"][0]["documentJson"] = json!(raw);
        r["modules"][0]["pin"]["sha256"] = json!(hash);
        let mut b: Value =
            serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();
        b["modelPins"][0]["sha256"] = json!(hash);
        for physical in b["relationships"].as_array_mut().unwrap() {
            let definition = d["modules"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["id"] == physical["logical"]["module"])
                .unwrap()["relationships"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == physical["logical"]["relationship"])
                .unwrap();
            physical["acceptedDefinition"] = definition.clone();
        }
        let raw = b.to_string();
        r["target"]["bindingSha256"] = json!(sha256(raw.as_bytes()));
        r["target"]["bindingJson"] = json!(raw);
    }
    #[test]
    fn explicitly_authored_inverse_and_composite_string_key_variants_preserve_lineage() {
        let mut inverse =
            request("SELECT s.id, RELATED_KEYS(s.products, 2) AS products FROM suppliers s");
        authored_variant(&mut inverse, |d| {
            for m in d["modules"].as_array_mut().unwrap() {
                for r in m["relationships"].as_array_mut().unwrap() {
                    if r["id"] == "products.supplier_id" {
                        r["inverse"] = json!("products");
                    }
                }
            }
        });
        let a = compile(&inverse);
        assert_eq!(a["status"], "compiled", "{a}");
        let integrity = a["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["id"] == "ashlar.relatedKeys.collectionIntegrity")
            .unwrap();
        assert_eq!(
            integrity["parameters"]["collections"][0]["relationship"]["inverse"],
            true
        );
        assert_eq!(
            integrity["parameters"]["edgeSchemas"][0]["relationship"]["relationship"],
            "products.supplier_id"
        );
        assert!(a["sql"].as_str().unwrap().contains("e.target_id AS __root"));
        let mut composite = request(
            "SELECT RELATED_KEYS(p.\"products.supplier_id\", 2) AS suppliers FROM products p",
        );
        authored_variant(&mut composite, |d| {
            for m in d["modules"].as_array_mut().unwrap() {
                for e in m["elements"].as_array_mut().unwrap() {
                    if e["id"] == "suppliers" {
                        e["keys"][0]["fields"]
                            .as_array_mut()
                            .unwrap()
                            .push(json!({"module":"domain","element":"suppliers.name"}));
                    }
                }
            }
        });
        let a = compile(&composite);
        assert_eq!(a["status"], "compiled", "{a}");
        assert_eq!(
            a["columns"][0]["representation"]["key"]["fields"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert!(a["sql"].as_str().unwrap().contains(
            "p.`k0` COLLATE UTF8_BINARY ASC, p.`k1` COLLATE UTF8_BINARY ASC, p.__edge ASC"
        ));
    }
    #[test]
    fn optional_and_numeric_key_variants_refuse_before_lowering() {
        for (member, value) in [("nullability", "absent-allowed"), ("scalarType", "integer")] {
            let mut r = request(
                "SELECT RELATED_KEYS(p.\"products.supplier_id\", 2) AS suppliers FROM products p",
            );
            authored_variant(&mut r, |d| {
                for m in d["modules"].as_array_mut().unwrap() {
                    for e in m["elements"].as_array_mut().unwrap() {
                        if e["id"] == "suppliers.id" {
                            e[member] = json!(value);
                        }
                    }
                }
            });
            let a = compile(&r);
            assert_eq!(a["status"], "blocked", "{a}");
            assert!(a.get("sql").is_none());
            assert_eq!(
                a["diagnostics"][0]["code"],
                if member == "nullability" {
                    "WFT-TYPE"
                } else {
                    "WFT-CAPABILITY"
                }
            );
        }
    }
    #[cfg(feature = "ashlar-databricks-paths")]
    #[test]
    fn historical_profile_still_refuses_onehop_and_does_not_register_new_profile() {
        let mut r = request(
            "SELECT RELATED_KEYS(p.\"products.supplier_id\", 2) AS suppliers FROM products p",
        );
        let new_on_old: Value =
            serde_json::from_str(&crate::paths::compile_json(&r.to_string())).unwrap();
        assert_eq!(new_on_old["status"], "blocked");
        r["target"]["backendId"] = json!(weft_databricks::paths::ID);
        r["target"]["backendVersion"] = json!(weft_databricks::paths::VERSION);
        r["target"]["targetProfile"] = json!(weft_databricks::paths::PROFILE);
        let old: Value = serde_json::from_str(&crate::paths::compile_json(&r.to_string())).unwrap();
        assert_eq!(old["status"], "blocked");
        assert_eq!(
            old["diagnostics"][0]["message"],
            "Selected04 capability refused before binding"
        );
        assert_eq!(compile(&r)["status"], "blocked");
    }
    #[test]
    fn authored_alias_collisions_allocate_fresh_private_collection_names() {
        for suffix in ["rows", "ranked", "collection"] {
            let alias = format!("__weft_keys_1_{suffix}");
            let a=compile(&request(&format!("SELECT RELATED_KEYS(\"{alias}\".\"products.supplier_id\", 2) AS suppliers FROM products AS \"{alias}\"")));
            assert_eq!(a["status"], "compiled", "{a}");
            let sql = a["sql"].as_str().unwrap();
            assert_eq!(sql.matches("`__weft_keys_1_rows` AS (").count(), 1);
            assert_ne!(a["ir"]["source"]["occurrence"], alias);
        }
    }
}
