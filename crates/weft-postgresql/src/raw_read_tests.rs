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
    fn compile(r: Value) -> Value {
        serde_json::from_str(&compile_json(&r.to_string())).unwrap()
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
                (format!("(NULL) {op} o.status"), if op == "IS DISTINCT FROM" { "\"o\".\"status\" IS NOT NULL" } else { "\"o\".\"status\" IS NULL" }),
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
