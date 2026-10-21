// SPDX-License-Identifier: MIT OR Apache-2.0
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
    if request.len()>16*1024*1024 {return Compiler::default().compile_json(request);}
    let version041=weft_core::json::checked_json(request).ok().is_some_and(|r|r["interfaceVersion"]=="weft-compile/0.4.1");
    if version041 {
        let current_spark42=weft_core::json::checked_json(request).ok().is_some_and(|r|
            r["target"]["backendId"]==weft_databricks::paths_keys_having_spark42::ID);
        let managed_current=weft_core::json::checked_json(request).ok().is_some_and(|r|
            r["target"]["backendId"]==weft_databricks::managed_current_core::ID);
        let managed_current_xxhash64=weft_core::json::checked_json(request).ok().is_some_and(|r|
            r["target"]["backendId"]==weft_databricks::managed_current_core_xxhash64::ID);
        let mut factory=move |_: &Catalog, _: Plan04View<'_>, _: CompositionInput<'_>| -> Result<Registry>{
            let mut r=Registry::default();
            if managed_current_xxhash64 {r.register(weft_databricks::managed_current_core_xxhash64::ManagedCurrentCoreXxhash64)?;}
            else if managed_current {r.register(weft_databricks::managed_current_core::ManagedCurrentCore)?;}
            else if current_spark42 {r.register(weft_databricks::paths_keys_having_spark42::CountStarHavingSpark42)?;}
            else {r.register(weft_databricks::paths_keys_having::CountStarHaving)?;}
            Ok(r)
        };
        weft_core::compile::v041::Compiler::default().compile_json_with_exact_numbers_factory(request,&mut factory)
    } else {
        let mut factory=|_: &Catalog, _: Plan04View<'_>, _: CompositionInput<'_>| -> Result<Registry>{let mut r=Registry::default();r.register(weft_databricks::paths_keys::PathsKeys)?;Ok(r)};
        Compiler::default().compile_json_with_factory(request,&mut factory)
    }
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
    fn current_spark42_count_star_target_is_exact_and_old_target_stays_separate() {
        let mut current = request(
            "SELECT c.name, COUNT(*) AS total FROM customers c GROUP BY c.name HAVING COUNT(*) > 1 ORDER BY c.name",
        );
        current["interfaceVersion"] = json!("weft-compile/0.4.1");
        current["dialect"] = json!("weft-sql/0.4.1");
        current["target"]["backendId"] = json!(weft_databricks::paths_keys_having_spark42::ID);
        current["target"]["backendVersion"] =
            json!(weft_databricks::paths_keys_having_spark42::VERSION);
        current["target"]["targetProfile"] =
            json!(weft_databricks::paths_keys_having_spark42::PROFILE);

        let described=weft_core::backend03::Backend::describe(&weft_databricks::paths_keys_having_spark42::CountStarHavingSpark42).unwrap();
        assert_eq!(described.backend_version,current["target"]["backendVersion"].as_str().unwrap());
        assert_eq!(described.backend_id,current["target"]["backendId"].as_str().unwrap());

        let current_artifact = compile(&current);
        assert_eq!(current_artifact["status"], "compiled", "{current_artifact}; request target={}",current["target"]);
        assert_eq!(
            current_artifact["backend"]["backendId"],
            weft_databricks::paths_keys_having_spark42::ID
        );
        assert_eq!(
            current_artifact["backend"]["backendVersion"],
            weft_databricks::paths_keys_having_spark42::VERSION
        );
        assert_eq!(
            current_artifact["backend"]["targetProfile"],
            weft_databricks::paths_keys_having_spark42::PROFILE
        );
        let obligation = current_artifact["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|obligation| obligation["id"] == "ashlar.nativeProfile")
            .unwrap();
        assert_eq!(obligation["parameters"]["engineVersion"], "4.2.0");
        assert_eq!(obligation["parameters"]["deltaVersion"], "4.4.1");
        assert_eq!(obligation["parameters"]["jdkVersion"], "21");
        assert_eq!(
            obligation["parameters"]["deltaJars"]["delta-spark_4.2_2.13-4.4.1.jar"],
            weft_databricks::paths_keys_having_spark42::DELTA_SPARK_SHA256
        );

        let mut legacy = request("SELECT c.name, COUNT(*) AS total FROM customers c GROUP BY c.name");
        legacy["interfaceVersion"] = json!("weft-compile/0.4.1");
        legacy["dialect"] = json!("weft-sql/0.4.1");
        legacy["target"]["backendVersion"] = json!(weft_databricks::paths_keys_having::VERSION);
        let legacy_artifact = compile(&legacy);
        assert_eq!(legacy_artifact["status"], "compiled", "{legacy_artifact}");
        assert_eq!(
            legacy_artifact["backend"]["backendVersion"],
            weft_databricks::paths_keys_having::VERSION
        );

        for (version, profile) in [
            (
                weft_databricks::paths_keys_having::VERSION,
                weft_databricks::paths_keys_having_spark42::PROFILE,
            ),
            (
                weft_databricks::paths_keys_having_spark42::VERSION,
                weft_databricks::paths_keys::PROFILE,
            ),
        ] {
            let mut mixed = current.clone();
            mixed["target"]["backendVersion"] = json!(version);
            mixed["target"]["targetProfile"] = json!(profile);
            assert_ne!(compile(&mixed)["status"], "compiled", "{mixed}");
        }
    }
    #[test]
    fn managed_current_core_grouped_count_retains_uc_versions_and_refuses_local_target(){
        let mut r=request("SELECT c.name, COUNT(*) AS total FROM customers c GROUP BY c.name HAVING COUNT(*) > 1 ORDER BY c.name");
        r["interfaceVersion"]=json!("weft-compile/0.4.1");r["dialect"]=json!("weft-sql/0.4.1");
        let mut binding:Value=serde_json::from_str(r["target"]["bindingJson"].as_str().unwrap()).unwrap();
        binding["profile"]=json!("ashlar-managed-current-core/0.1");
        let roles=["object_current","edge_current","tombstone","whole_source_history"];
        let mut tables=binding["publication"]["tables"].as_array().unwrap().clone();
        assert_eq!(tables.len(),4);
        for (index,table) in tables.iter_mut().enumerate(){
            table["name"]=json!(["client_dev","managed_current_test",roles[index]]);
            table["uuid"]=json!(format!("00000000-0000-4000-8000-{:012x}",index+1));
            table["version"]=json!(21+index);
        }
        let object_index=roles.iter().position(|role|*role=="object_current").unwrap();
        let edge_index=roles.iter().position(|role|*role=="edge_current").unwrap();
        for record in binding["records"].as_array_mut().unwrap(){record["table"]=json!(object_index);}
        for relationship in binding["relationships"].as_array_mut().unwrap(){relationship["table"]=json!(edge_index);}
        binding["publication"]["tables"]=json!(tables);
        let raw=binding.to_string();let digest=weft_core::json::sha256(raw.as_bytes());
        r["target"]=json!({"backendId":weft_databricks::managed_current_core::ID,"backendVersion":weft_databricks::managed_current_core::VERSION,"targetProfile":weft_databricks::managed_current_core::PROFILE,"bindingJson":raw,"bindingSha256":digest});
        let a=compile(&r);assert_eq!(a["status"],"compiled","{a}");
        assert_eq!(a["backend"]["backendId"],weft_databricks::managed_current_core::ID);
        assert_eq!(a["backend"]["backendVersion"],weft_databricks::managed_current_core::VERSION);
        assert_eq!(a["backend"]["targetProfile"],weft_databricks::managed_current_core::PROFILE);
        assert!(a["sql"].as_str().unwrap().contains("`client_dev`.`managed_current_test`.`object_current` VERSION AS OF 21"));
        assert!(a["sql"].as_str().unwrap().contains("COUNT(*)"));
        assert!(a["obligations"].as_array().unwrap().iter().any(|o|o["id"]=="ashlar.nativeProfile"&&o["parameters"]["targetProfile"]==weft_databricks::managed_current_core::PROFILE));
        let publication=a["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.candidate.publication").unwrap();
        assert!(publication["parameters"]["nativeProfile"]["warehouseRelease"].is_null());
        assert_eq!(publication["parameters"]["nativeProfile"]["engine"],"databricks-serverless-spark-jobs");
        assert!(publication["parameters"]["nativeProfile"]["requirements"].as_array().unwrap().iter().any(|item|item=="keep the six-role installation inventory separate from the four-role manifest data vector"));
        let mut xxhash64 = r.clone();
        let mut v4_binding: Value = serde_json::from_str(xxhash64["target"]["bindingJson"].as_str().unwrap()).unwrap();
        v4_binding["layoutRevision"] = json!("ashlar-delta/0.4-xxhash64");
        v4_binding["layoutSha256"] = json!("a".repeat(64));
        v4_binding["lookupHashProfile"] = json!("spark-xxhash64-typed-key/0.1");
        let raw = v4_binding.to_string(); let digest = weft_core::json::sha256(raw.as_bytes());
        xxhash64["target"] = json!({"backendId":weft_databricks::managed_current_core_xxhash64::ID,"backendVersion":weft_databricks::managed_current_core_xxhash64::VERSION,"targetProfile":weft_databricks::managed_current_core_xxhash64::PROFILE,"bindingJson":raw,"bindingSha256":digest});
        let v4 = compile(&xxhash64); assert_eq!(v4["status"], "compiled", "{v4}");
        assert_eq!(v4["backend"]["backendId"], weft_databricks::managed_current_core_xxhash64::ID);
        assert_eq!(v4["backend"]["backendVersion"], weft_databricks::managed_current_core_xxhash64::VERSION);
        assert_eq!(v4["backend"]["targetProfile"], weft_databricks::managed_current_core_xxhash64::PROFILE);
        assert_eq!(v4["backend"]["interfaceVersion"], "weft-backend/0.3.0");
        assert_eq!(v4["targetContext"]["id"], weft_databricks::managed_current_core_xxhash64::PROFILE);
        let v4_pub=v4["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.candidate.publication").unwrap();
        assert_eq!(v4_pub["parameters"]["layoutRevision"],"ashlar-delta/0.4-xxhash64");
        assert_eq!(v4_pub["parameters"]["layoutSha256"],"a".repeat(64));
        assert_eq!(v4_pub["parameters"]["nativeProfile"]["lookupHashProfile"],"spark-xxhash64-typed-key/0.1");
        assert_eq!(v4["sql"], a["sql"]); assert_eq!(v4["parameters"], a["parameters"]); assert_eq!(v4["logicalPlan"], a["logicalPlan"]);
        assert_eq!(v4["obligations"].as_array().unwrap().iter().find(|o|o["id"]=="ashlar.nativeProfile").unwrap()["parameters"]["lookupHashProfile"], "spark-xxhash64-typed-key/0.1");
        let mut crossed_layout = r.clone(); crossed_layout["target"]["backendId"] = json!(weft_databricks::managed_current_core_xxhash64::ID); crossed_layout["target"]["backendVersion"] = json!(weft_databricks::managed_current_core_xxhash64::VERSION); crossed_layout["target"]["targetProfile"] = json!(weft_databricks::managed_current_core_xxhash64::PROFILE);
        assert_ne!(compile(&crossed_layout)["status"], "compiled");
        let mut crossed_target = xxhash64.clone(); crossed_target["target"]["backendId"] = json!(weft_databricks::managed_current_core::ID); crossed_target["target"]["backendVersion"] = json!(weft_databricks::managed_current_core::VERSION); crossed_target["target"]["targetProfile"] = json!(weft_databricks::managed_current_core::PROFILE);
        assert_ne!(compile(&crossed_target)["status"], "compiled");
        let mut old=r.clone();old["target"]["backendId"]=json!(weft_databricks::paths_keys_having_spark42::ID);old["target"]["backendVersion"]=json!(weft_databricks::paths_keys_having_spark42::VERSION);old["target"]["targetProfile"]=json!(weft_databricks::paths_keys_having_spark42::PROFILE);
        assert_ne!(compile(&old)["status"],"compiled");
        let mut crossed=r.clone();let mut crossed_binding:Value=serde_json::from_str(crossed["target"]["bindingJson"].as_str().unwrap()).unwrap();crossed_binding["records"][0]["table"]=json!(edge_index);let crossed_raw=crossed_binding.to_string();let crossed_digest=weft_core::json::sha256(crossed_raw.as_bytes());crossed["target"]["bindingJson"]=json!(crossed_raw);crossed["target"]["bindingSha256"]=json!(crossed_digest);assert_ne!(compile(&crossed)["status"],"compiled");
        let mut crossed_edge=r.clone();let mut crossed_binding:Value=serde_json::from_str(crossed_edge["target"]["bindingJson"].as_str().unwrap()).unwrap();crossed_binding["relationships"][0]["table"]=json!(object_index);let crossed_raw=crossed_binding.to_string();let crossed_digest=weft_core::json::sha256(crossed_raw.as_bytes());crossed_edge["target"]["bindingJson"]=json!(crossed_raw);crossed_edge["target"]["bindingSha256"]=json!(crossed_digest);assert_ne!(compile(&crossed_edge)["status"],"compiled");
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
                    // A facetless 0.10 integer selects an exact mathematical
                    // domain outside this path-key profile, so type admission
                    // refuses before the backend capability check.
                    "WFT-TYPE"
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
