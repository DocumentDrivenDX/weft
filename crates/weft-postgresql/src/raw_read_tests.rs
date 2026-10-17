#[cfg(test)]
mod raw_read_tests {
    use super::compile_json;
    use serde_json::{json, Value};
    fn request() -> Value {
        serde_json::from_str(include_str!(
            "../../../tests/raw-postgresql/read-request.json"
        ))
        .unwrap()
    }
    #[test]
    fn inner_join_refines_only_its_source_ancestor_chain(){
        let mut r=request();let mut doc:Value=serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();
        let mut rel=doc["modules"][0]["relationships"][0].clone();rel["id"]=json!("customer-self");rel["name"]=json!("customer-self");rel["source"][0]["element"]=json!("Customer");doc["modules"][0]["relationships"].as_array_mut().unwrap().push(rel);
        r["binding"]["relationships"].as_array_mut().unwrap().push(json!({"id":"customer-self","source":["commerce","Customer"],"target":["commerce","Customer"],"pairs":[["id","id"]],"toOne":true}));
        let raw=doc.to_string();r["binding"]["modelSha256"]=json!(weft_core::json::sha256(raw.as_bytes()));r["binding"]["modelJson"]=json!(raw);r["parameters"]=json!([]);
        r["sql"]=json!("SELECT a.name AS a FROM commerce.\"Order\" o INNER JOIN commerce.\"Customer\" a ON a.id=a.id");
        assert_eq!(compile(r.clone())["status"],"blocked");
        for (projection,joins,nullable) in [
          ("a.name AS a, b.name AS b", "INNER JOIN commerce.\"Customer\" b ON a.id=b.id",vec![false,false]),
          ("a.name AS a, b.name AS b, c.name AS c", "LEFT JOIN commerce.\"Customer\" b ON a.id=b.id INNER JOIN commerce.\"Customer\" c ON b.id=c.id",vec![false,false,false]),
          ("a.name AS a, sibling.name AS sibling, b.name AS b", "LEFT JOIN commerce.\"Customer\" sibling ON o.customer_id=sibling.id INNER JOIN commerce.\"Customer\" b ON a.id=b.id",vec![false,true,false]),
          ("a.name AS a, child.name AS child, b.name AS b", "LEFT JOIN commerce.\"Customer\" child ON a.id=child.id INNER JOIN commerce.\"Customer\" b ON a.id=b.id",vec![false,true,false]),
        ] {
          r["sql"]=json!(format!("SELECT {projection} FROM commerce.\"Order\" o LEFT JOIN commerce.\"Customer\" a ON o.customer_id=a.id {joins}"));
          let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");
          assert_eq!(p["outputs"].as_array().unwrap().iter().map(|v|v["nullable"].as_bool().unwrap()).collect::<Vec<_>>(),nullable,"{p}");
        }
    }
    fn compile(r: Value) -> Value {
        serde_json::from_str(&compile_json(&r.to_string())).unwrap()
    }
    fn term_request()->Value {
        let search:Value=serde_json::from_str(include_str!("../../../tests/native-search/keyword-request.json")).unwrap();
        let mut r=search["selection"].clone();r["sql"]=json!("SELECT r.id AS id FROM \"search\".\"Doc\" AS r WHERE MATCHES_ALL_TERMS(r.body,:terms)");r["parameters"]=json!([{ "name":"terms","carrier":"string","nullable":false }]);r
    }
    fn change_term_model(r:&mut Value,change:impl FnOnce(&mut Value)) {
        let mut doc:Value=serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();change(&mut doc);let raw=doc.to_string();r["binding"]["modelSha256"]=json!(weft_core::json::sha256(raw.as_bytes()));r["binding"]["modelJson"]=json!(raw);
    }
    #[test]
    fn raw_read_terms_keep_original_index_analyzer_and_native_obligations() {
        for (function,operator) in [("MATCHES_ALL_TERMS","<@"),("MATCHES_ANY_TERM","&&")] {
            for language in ["english","simple"] {
                let mut r=term_request();r["sql"]=json!(format!("SELECT r.id AS id FROM \"search\".\"Doc\" AS r WHERE {function}(r.body,:terms)"));
                if language=="simple" {change_term_model(&mut r,|doc|doc["modules"][0]["extensions"]["umf.access"]["search"][0]["keyword"]["analyzer"]=json!({"language":"simple","stemming":false}));}
                let out=compile(r.clone());assert_eq!(out["status"],"compiled","{out}");assert!(out["sql"].as_str().unwrap().contains(operator));assert!(out["sql"].as_str().unwrap().contains("pg_catalog.tsvector_to_array(pg_catalog.to_tsvector("));assert!(!out["sql"].as_str().unwrap().contains("to_tsquery("));assert_eq!(out["parameterOrder"],json!(["terms"]));
                let obligation=&out["obligations"]["termFilter"];assert_eq!(obligation["version"],"umf.raw-term-filter/0.1");assert_eq!(obligation["nativeCatalogBeforeAfter"],true);assert_eq!(obligation["indexes"].as_array().unwrap().len(),1);
                let index=&obligation["indexes"][0];assert_eq!(index["configuration"],format!("pg_catalog.{language}"));assert_eq!(index["index"]["id"],"text");assert_eq!(index["nativeBinding"]["native"],"doc_text");assert_eq!(index["source"]["documentSha256"],r["binding"]["modelSha256"]);assert_eq!(index["nativeAdmission"]["parameters"].as_array().unwrap().len(),5);assert!(index["nativeAdmission"]["sql"].as_str().unwrap().contains("pg_ts_config_map"));
            }
        }
        let mut r=term_request();r["keyset"]=json!(true);r["sql"]=json!("SELECT r.id AS id FROM \"search\".\"Doc\" AS r WHERE MATCHES_ANY_TERM(r.body,:terms) ORDER BY r.id");assert_eq!(compile(r)["status"],"compiled");
    }
    #[test]
    fn raw_read_terms_refuse_missing_ambiguous_or_unsupported_selected_meaning() {
        for mode in 0..10 {
            let mut r=term_request();
            change_term_model(&mut r,|doc|{
                let payload=&mut doc["modules"][0]["extensions"]["umf.access"];
                match mode {
                    0=>payload["search"]=json!([]),
                    1=>payload["search"][0]["keyword"]["analyzer"]=json!("implementation-defined"),
                    2=>payload["search"][0]["keyword"]["analyzer"]=json!({"language":"english","stemming":false}),
                    3=>payload["search"][0]["keyword"]["syntax"]=json!("operators"),
                    4=>payload["search"][0]["unknownSelectedMeaning"]=json!(true),
                    5=>payload["search"][0]["keyword"]["source"]=json!([]),
                    6=>payload["searchBindings"][0]["freshness"]=json!("asynchronous"),
                    7=>payload["searchBindings"][0]["target"]["platform"]=json!("lakebase"),
                    8=>{let duplicate=payload["search"][0].clone();payload["search"].as_array_mut().unwrap().push(duplicate);},
                    _=>payload["search"][0]["keyword"]["analyzer"]["unknown"]=json!(true),
                }
            });
            let out=compile(r);assert_eq!(out["status"],"blocked","{mode}: {out}");assert!(out.get("sql").is_none());
        }
        for predicate in ["MATCHES_ALL_TERMS(r.id,:terms)","MATCHES_ALL_TERMS(r.body,1)","MATCHES_ANY_TERM(r.body,NULL)","MATCHES_ALL_TERMS(r.body)","evil.MATCHES_ALL_TERMS(r.body,:terms)"] {
            let mut r=term_request();r["sql"]=json!(format!("SELECT r.id AS id FROM \"search\".\"Doc\" AS r WHERE {predicate}"));assert_eq!(compile(r)["status"],"blocked","{predicate}");
        }
    }
    #[test]
    fn raw_read_numeric_buckets_keep_exact_group_identity_and_ties() {
        let mut r=request();r["sql"]=json!("SELECT BUCKET_WIDTH(o.amount, 0.25) AS bucket, SUM(o.id) AS total, AVG_SCALE(o.amount,2) AS average FROM \"commerce\".\"Order\" AS o GROUP BY BUCKET_WIDTH(o.amount, 0.25) ORDER BY average DESC");r["parameters"]=json!([]);
        let out=compile(r);assert_eq!(out["status"],"compiled","{out}");assert_eq!(out["outputs"][0]["carrier"],"decimal");assert_eq!(out["outputs"][1]["carrier"],"integer");let sql=out["sql"].as_str().unwrap();assert!(sql.contains("pg_catalog.sum(\"o\".\"id\"::pg_catalog.numeric)"));assert!(sql.contains("DESC NULLS LAST,"));assert!(sql.ends_with("ASC NULLS LAST LIMIT 10"));
        for query in [
            "SELECT o.amount AS raw, COUNT(*) AS n FROM \"commerce\".\"Order\" AS o GROUP BY BUCKET_WIDTH(o.amount, 1)",
            "SELECT BUCKET_WIDTH(o.amount,1) AS bucket, COUNT(*) AS n FROM \"commerce\".\"Order\" AS o GROUP BY BUCKET_WIDTH(o.amount, 1) HAVING o.amount > 0.0",
            "SELECT BUCKET_WIDTH(o.amount,1) AS bucket, COUNT(*) AS n FROM \"commerce\".\"Order\" AS o GROUP BY BUCKET_WIDTH(o.amount, 1) ORDER BY o.amount",
            "SELECT BUCKET_WIDTH(o.amount,2) AS bucket, COUNT(*) AS n FROM \"commerce\".\"Order\" AS o GROUP BY BUCKET_WIDTH(o.amount, 1)",
        ] {let mut r=request();r["sql"]=json!(query);r["parameters"]=json!([]);assert_eq!(compile(r)["status"],"blocked","{query}");}
        for width in ["0","-1","NULL",":width","'1'","1e999","1.0.0"] {let mut r=request();r["sql"]=json!(format!("SELECT BUCKET_WIDTH(o.amount,{width}) AS bucket FROM \"commerce\".\"Order\" AS o"));r["parameters"]=json!([]);let out=compile(r);assert_eq!(out["status"],"blocked","{width}: {out}");assert!(out.get("sql").is_none());}
    }
    #[test]
    fn raw_read_explicit_scale_average_and_exact_distinct_count() {
        for scale in [0,2,38,1000] {
            let mut r=request();r["sql"]=json!(format!("SELECT AVG_SCALE(o.amount, {scale}) AS average, COUNT(DISTINCT o.status) AS unique_status FROM \"commerce\".\"Order\" AS o"));r["parameters"]=json!([]);
            let out=compile(r);assert_eq!(out["status"],"compiled","{out}");let sql=out["sql"].as_str().unwrap();assert!(sql.contains("pg_catalog.div("));assert!(sql.contains("pg_catalog.mod("));assert!(!sql.contains("pg_catalog.avg("));assert!(sql.contains("count(DISTINCT \"o\".\"status\" COLLATE pg_catalog.\"C\")"));assert_eq!(out["outputs"][0]["carrier"],"decimal");assert_eq!(out["outputs"][0]["nullable"],true);assert_eq!(out["outputs"][1]["carrier"],"integer");assert_eq!(out["outputs"][1]["nullable"],false);
        }
        let mut r=request();r["sql"]=json!("SELECT AVG(o.amount) AS native_average FROM \"commerce\".\"Order\" AS o");r["parameters"]=json!([]);let out=compile(r);assert_eq!(out["status"],"compiled");assert!(out["sql"].as_str().unwrap().contains("pg_catalog.avg("));
        for expression in ["AVG_SCALE(o.status,2)","AVG_SCALE(o.amount)","AVG_SCALE(o.amount,-1)","AVG_SCALE(o.amount,1001)","AVG_SCALE(o.amount,2.0)","AVG_SCALE(o.amount,:scale)","AVG_SCALE(DISTINCT o.amount,2)","AVG_SCALE(o.amount,2,3)","COUNT(DISTINCT *)","SUM(DISTINCT o.amount)"] {
            let mut r=request();r["sql"]=json!(format!("SELECT {expression} AS result FROM \"commerce\".\"Order\" AS o"));r["parameters"]=json!([]);let out=compile(r);assert_eq!(out["status"],"blocked","{expression}: {out}");assert!(out.get("sql").is_none());
        }
    }
    #[test]
    fn raw_read_prefix_is_binary_not_a_pattern() {
        for target in ["postgresql18", "lakebase18"] {
            for prefix in ["", "50%_", "a\\", "é", "é", "𝄞", "O'Reilly"] {
                let mut r = request();
                r["target"] = json!(target);
                r["sql"] = json!(format!("SELECT o.id AS id FROM \"commerce\".\"Order\" AS o WHERE STARTS_WITH(o.status, '{}')", prefix.replace('\'', "''")));
                r["parameters"] = json!([]);
                let out = compile(r);
                assert_eq!(out["status"], "compiled", "{out}");
                assert!(out["sql"].as_str().unwrap().contains("pg_catalog.starts_with("));
                assert!(!out["sql"].as_str().unwrap().contains("LIKE"));
            }
        }
        for nullable in [false,true] {
            let mut r=request();r["sql"]=json!("SELECT o.id AS id FROM \"commerce\".\"Order\" AS o WHERE NOT STARTS_WITH((o.status), (:prefix))");r["parameters"]=json!([{ "name":"prefix", "carrier":"string", "nullable":nullable }]);
            let out=compile(r);assert_eq!(out["status"],"compiled","{out}");assert_eq!(out["parameterOrder"],json!(["prefix"]));assert_eq!(out["parameters"][0]["nullable"],nullable);assert!(out["sql"].as_str().unwrap().contains("$1::pg_catalog.text"));
        }
    }
    #[test]
    fn raw_read_prefix_rejects_wrong_types_modifiers_and_ungrouped_having() {
        for predicate in ["STARTS_WITH(o.amount, '1')", "STARTS_WITH(o.status, 1)", "STARTS_WITH(o.status, NULL)", "STARTS_WITH(o.status, o.status)", "STARTS_WITH('x', 'x')", "evil.starts_with(o.status, 'a')", "STARTS_WITH(DISTINCT o.status, 'a')", "STARTS_WITH(o.status)", "STARTS_WITH(o.status, 'a', 'b')", "STARTS_WITH(o.status, evil())", "MATCHES_ALL_TERMS(o.status, 'a')"] {
            let mut r=request();r["sql"]=json!(format!("SELECT o.id AS id FROM \"commerce\".\"Order\" AS o WHERE {predicate}"));r["parameters"]=json!([]);let out=compile(r);assert_eq!(out["status"],"blocked","{predicate}: {out}");assert!(out.get("sql").is_none());
        }
        let mut r=request();r["sql"]=json!("SELECT count(*) AS n FROM \"commerce\".\"Order\" AS o HAVING STARTS_WITH(o.status, 'a')");r["parameters"]=json!([]);let out=compile(r);assert_eq!(out["status"],"blocked");
        let mut r=request();r["sql"]=json!("SELECT o.status AS status, count(*) AS n FROM \"commerce\".\"Order\" AS o GROUP BY o.status HAVING STARTS_WITH(o.status, 'a')");r["parameters"]=json!([]);let out=compile(r);assert_eq!(out["status"],"compiled","{out}");
    }
    #[test]
    fn raw_read_null_safe_distinctness_retains_typed_nullable_parameters() {
        for op in ["IS DISTINCT FROM", "IS NOT DISTINCT FROM"] {
            for (column, carrier, nullable) in [
                ("amount", "decimal", true),
                ("amount", "decimal", false),
                ("status", "string", true),
                ("status", "string", false),
                ("id", "integer", true),
            ] {
                let mut r = request();
                r["sql"] = json!(format!("SELECT o.id AS id FROM \"commerce\".\"Order\" AS o WHERE o.{column} {op} :value"));
                r["parameters"] = json!([{"name":"value","carrier":carrier,"nullable":nullable}]);
                let out = compile(r);
                assert_eq!(out["status"], "compiled", "{out}");
                assert!(out["sql"].as_str().unwrap().contains(op));
                assert_eq!(out["parameterOrder"], json!(["value"]));
                assert_eq!(out["parameters"][0]["nullable"], nullable);
            }
            for predicate in [
                format!("o.amount {op} 1.20"),
                format!("o.status {op} 'ready'"),
                format!("o.customer_id {op} o.id"),
            ] {
                let mut r = request();
                r["sql"] = json!(format!("SELECT o.id AS id FROM \"commerce\".\"Order\" AS o WHERE {predicate}"));
                r["parameters"] = json!([]);
                assert_eq!(compile(r)["status"], "compiled");
            }
            for (predicate, emitted) in [
                (format!("o.amount {op} NULL"), if op == "IS DISTINCT FROM" { "\"o\".\"amount\" IS NOT NULL" } else { "\"o\".\"amount\" IS NULL" }),
                (format!("(NULL) {op} o.status"), if op == "IS DISTINCT FROM" { "\"o\".\"status\" COLLATE pg_catalog.\"C\" IS NOT NULL" } else { "\"o\".\"status\" COLLATE pg_catalog.\"C\" IS NULL" }),
                (format!("NULL {op} (NULL)"), if op == "IS DISTINCT FROM" { "WHERE FALSE" } else { "WHERE TRUE" }),
            ] {
                let mut r = request();
                r["sql"] = json!(format!("SELECT o.id AS id FROM \"commerce\".\"Order\" AS o WHERE {predicate}"));
                r["parameters"] = json!([]);
                let out = compile(r);
                assert_eq!(out["status"], "compiled", "{out}");
                assert!(out["sql"].as_str().unwrap().contains(emitted), "{out}");
            }
            for unsupported in ["evil()", "TRUE"] {
                let mut r = request();
                r["sql"] = json!(format!("SELECT o.id AS id FROM \"commerce\".\"Order\" AS o WHERE o.amount {op} {unsupported}"));
                r["parameters"] = json!([]);
                let out = compile(r);
                assert_eq!(out["status"], "blocked");
                assert!(out.get("sql").is_none());
            }
            let mut r = request();
            r["sql"] = json!(format!("SELECT o.id AS id FROM \"commerce\".\"Order\" AS o WHERE o.amount {op} o.status"));
            r["parameters"] = json!([]);
            let out = compile(r);
            assert_eq!(out["status"], "blocked");
            assert_eq!(out["diagnostics"][0]["message"], "comparison carrier mismatch");
            assert!(out.get("sql").is_none());

            let mut r = request();
            r["sql"] = json!(format!("SELECT o.status AS status, sum(o.amount) AS total FROM \"commerce\".\"Order\" AS o GROUP BY o.status HAVING o.amount {op} :minimum"));
            let out = compile(r);
            assert_eq!(out["status"], "blocked");
            assert_eq!(out["diagnostics"][0]["message"], "ungrouped HAVING column");
            assert!(out.get("sql").is_none());
            let mut r = request();
            r["sql"] = json!(format!("SELECT o.status AS status, sum(o.amount) AS total FROM \"commerce\".\"Order\" AS o GROUP BY o.status HAVING NULL {op} o.amount"));
            r["parameters"] = json!([]);
            let out = compile(r);
            assert_eq!(out["status"], "blocked");
            assert_eq!(out["diagnostics"][0]["message"], "ungrouped HAVING column");
        }
    }
    #[test]
    fn raw_read_compiles_deterministically_with_native_role_obligations() {
        let r = request();
        let a = compile(r.clone());
        assert_eq!(a, compile(r));
        assert_eq!(a["status"], "compiled");
        assert_eq!(a["obligations"]["principal"], "caller-connection-role");
        assert_eq!(a["obligations"]["transaction"], "read-only");
        assert!(a["sql"].as_str().unwrap().contains("pg_catalog.avg"));
        assert!(a["sql"]
            .as_str()
            .unwrap()
            .contains("$1::pg_catalog.numeric"));
    }
    #[test]
    fn raw_read_refuses_unreviewable_or_side_effecting_shapes() {
        for sql in [
            "DELETE FROM t",
            "SELECT evil() FROM t AS x",
            "SELECT pg_catalog.pg_sleep(:minimum) AS delay FROM t AS x",
            "SELECT x.a FROM t AS x FOR UPDATE",
            "SELECT 1; SELECT 2",
            "WITH t AS (DELETE FROM u RETURNING *) SELECT * FROM t",
        ] {
            let mut r = request();
            r["sql"] = json!(sql);
            let a = compile(r);
            assert_eq!(a["status"], "blocked");
            assert!(a.get("sql").is_none());
        }
    }
    #[test]
    fn raw_read_model_source_and_target_are_build_time_boundaries() {
        let mut r = request();
        r["binding"]["modelSha256"] = json!("0".repeat(64));
        assert_eq!(compile(r)["status"], "blocked");
        let mut r = request();
        r["target"] = json!("postgresql17");
        assert_eq!(compile(r)["status"], "blocked");
        let mut r = request();
        r["target"] = json!("lakebase18");
        assert_eq!(compile(r)["status"], "compiled");
        let mut r = request();
        r["binding"]["tables"][0]["columns"][0]["logical"] = json!("undeclared");
        assert_eq!(compile(r)["status"], "blocked");
    }
    #[test]
    fn raw_read_grouping_and_decimal_input_refuse_before_sql_exists() {
        let mut r = request();
        r["sql"] = json!("SELECT sum(o.status) AS total FROM \"commerce\".\"Order\" AS o");
        r["parameters"] = json!([]);
        assert_eq!(compile(r)["status"], "blocked");
        let mut r = request();
        r["sql"]=json!("SELECT o.status AS status, sum(o.amount) AS total FROM \"commerce\".\"Order\" AS o GROUP BY o.status HAVING o.amount > :minimum");
        assert_eq!(compile(r)["status"], "blocked");
    }
    #[test]
    fn raw_read_three_edges_admit_and_four_refuse_at_build() {
        for (count, status) in [(3, "compiled"), (4, "blocked")] {
            let mut r = request();
            let mut sql = String::from("SELECT o.id AS id FROM \"commerce\".\"Order\" AS o");
            for i in 0..count {
                sql += &format!(
                    " LEFT JOIN \"commerce\".\"Customer\" AS c{i} ON o.customer_id = c{i}.id"
                );
            }
            r["sql"] = json!(sql);
            r["parameters"] = json!([]);
            assert_eq!(compile(r)["status"], status);
        }
    }
    #[test]
    fn raw_read_retains_native_privilege_and_complete_cursor_order_provenance() {
        let mut r=request();
        r["sql"]=json!("SELECT o.id AS id FROM \"commerce\".\"Order\" AS o ORDER BY o.status DESC NULLS FIRST, o.id ASC NULLS LAST LIMIT 2");
        r["parameters"]=json!([]); r["keyset"]=json!(true);
        let out=compile(r);
        assert_eq!(out["status"],"compiled");
        assert_eq!(out["cursorOrdering"],json!([
            {"table":["commerce","Order"],"field":"status","direction":"DESC","nulls":"FIRST"},
            {"table":["commerce","Order"],"field":"id","direction":"ASC","nulls":"LAST"}]));
        assert_eq!(out["referencedTables"].as_array().unwrap().len(),1);
        assert_eq!(out["referencedTables"][0]["native"],"orders");
        assert_eq!(out["referencedTables"][0]["columns"][0]["native"],"id");
    }

}
