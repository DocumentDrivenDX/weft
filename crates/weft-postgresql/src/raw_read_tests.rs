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
}
