// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exact finite base-ten predicate ordering, without Spark numeric coercion.
//! Inputs are owner-admitted plain text or native bounded Integer/Decimal casts.
use serde_json::{json, Value};
use weft_core::raw_read::{Carrier, Request};

pub(crate) fn parameters(request: &Request) -> Option<Value> {
    if request.binding.native_columns.is_none() { return None }
    let parameters: Vec<_> = request.parameters.iter()
        .filter(|p| p.container.is_none() && matches!(p.carrier, Carrier::Integer | Carrier::Decimal))
        .map(|p| json!({"name":p.name,"carrier":p.carrier,"nullable":p.nullable})).collect();
    if parameters.is_empty() { return None }
    Some(json!({"version":"weft.spark-exact-numeric-text/0.1",
        "parameters":parameters,"maximumTokenBytes":10000,"format":"finite-plain-base-ten"}))
}

pub(crate) fn comparison(left: &str, op: &str, right: &str) -> String {
    // Each operand occurs once. Local lambda variables prevent an expression
    // expansion from duplicating parameters or changing their original slots.
    let inputs = format!("array(named_struct('a',{left},'b',{right}))");
    let absolute = format!("transform({inputs}, p -> named_struct('a',regexp_replace(p.a,'^[+-]',''),'b',regexp_replace(p.b,'^[+-]',''),'an',startsWith(p.a,'-'),'bn',startsWith(p.b,'-'),'az',p.a IS NULL,'bz',p.b IS NULL))");
    let parts = format!("transform({absolute}, p -> named_struct('ai',regexp_replace(substring_index(p.a,'.',1),'^0+',''),'bi',regexp_replace(substring_index(p.b,'.',1),'^0+',''),'af',regexp_replace(CASE WHEN instr(p.a,'.')>0 THEN substring_index(p.a,'.',-1) ELSE '' END,'0+$',''),'bf',regexp_replace(CASE WHEN instr(p.b,'.')>0 THEN substring_index(p.b,'.',-1) ELSE '' END,'0+$',''),'an',p.an,'bn',p.bn,'az',p.az,'bz',p.bz))");
    let ordered = format!("transform({parts}, p -> named_struct('s',CASE WHEN p.ai='' AND p.af='' THEN 0 WHEN p.an THEN -1 ELSE 1 END,'t',CASE WHEN p.bi='' AND p.bf='' THEN 0 WHEN p.bn THEN -1 ELSE 1 END,'a',named_struct('width',length(p.ai),'whole',p.ai COLLATE UTF8_BINARY,'fraction',p.af COLLATE UTF8_BINARY),'b',named_struct('width',length(p.bi),'whole',p.bi COLLATE UTF8_BINARY,'fraction',p.bf COLLATE UTF8_BINARY),'az',p.az,'bz',p.bz))");
    format!("element_at(transform({ordered}, p -> CASE WHEN p.az OR p.bz THEN CAST(NULL AS BOOLEAN) ELSE (CASE WHEN p.s<>p.t THEN CASE WHEN p.s<p.t THEN -1 ELSE 1 END WHEN p.a=p.b THEN 0 WHEN p.a<p.b THEN CASE WHEN p.s<0 THEN 1 ELSE -1 END ELSE CASE WHEN p.s<0 THEN -1 ELSE 1 END END {op} 0) END),1)")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_list_program_retains_original_parameter_and_s8_json_transport(){
        let mut r:Value=serde_json::from_str(include_str!("../../../tests/spark-pages/read-request.json")).unwrap();
        r["keyset"]=json!(false);r["snapshots"]=json!({"orders":1});r["parameters"]=json!([{"name":"values","carrier":"string","nullable":false,"container":"list","elementNullable":true,"maxItems":1000}]);
        for keyset in [false,true]{r["keyset"]=json!(keyset);for op in ["IN","NOT IN"]{
            r["sql"]=json!(format!("SELECT o.id AS id FROM commerce.\"Order\" o WHERE o.status {op} (:values) ORDER BY o.id LIMIT 10"));
            let p:Value=serde_json::from_str(&crate::raw_read::compile_json(&r.to_string())).unwrap();assert_eq!(p["status"],"compiled","{p}");
            assert_eq!(p["parameters"],r["parameters"]);assert_eq!(p["parameterOrder"][0],"values");let sql=p["sql"].as_str().unwrap();
            for term in ["from_json(:p1,'ARRAY<STRING>',map('mode','FAILFAST'))","size(__umf_list.items)=0 THEN FALSE","__umf_list.lhs IS NULL THEN CAST(NULL AS BOOLEAN)","__umf_item IS NOT NULL","COLLATE UTF8_BINARY"]{assert!(sql.contains(term),"{term}: {sql}")}
            assert_eq!(sql.matches(":p1").count(),1);assert_eq!(p["obligations"]["listPredicateParameters"]["maximumEncodedBytes"],1000000);
        }}
        r["keyset"]=json!(false);
        for predicate in ["o.status=:values","o.status IN(:values,'x')","o.status IN(:missing)"]{r["sql"]=json!(format!("SELECT o.id AS id FROM commerce.\"Order\" o WHERE {predicate} LIMIT 10"));let p:Value=serde_json::from_str(&crate::raw_read::compile_json(&r.to_string())).unwrap();assert_eq!(p["status"],"blocked","{predicate}: {p}");}
    }
    #[test]
    fn numeric_list_uses_original_full_shape_custody_without_numeric_array_casts(){
        let original:Value=serde_json::from_str(include_str!("../../../tests/spark-pages/full-shape-list-request.json")).unwrap();
        for (field,carrier) in [("big","integer"),("amount","decimal")] {
            let mut r=original.clone();r["parameters"]=json!([{"name":"values","carrier":carrier,"nullable":false,"container":"list","elementNullable":true,"maxItems":1000}]);
            let sql=r["sql"].as_str().unwrap();let (selection,ordering)=sql.split_once(" ORDER BY ").unwrap();r["sql"]=json!(format!("{selection} WHERE o.{field} IN(:values) ORDER BY {ordering}"));
            let p:Value=serde_json::from_str(&crate::raw_read::compile_json(&r.to_string())).unwrap();assert_eq!(p["status"],"compiled","{p}");
            let sql=p["sql"].as_str().unwrap();assert!(sql.contains("from_json(:p1,'ARRAY<STRING>'"));assert!(sql.contains("regexp_replace"));assert!(!sql.contains(" AS DOUBLE")&&!sql.contains("ARRAY<DECIMAL"));
            assert!(p["obligations"].get("numericPredicateParameters").is_none());assert_eq!(p["obligations"]["listPredicateParameters"]["parameters"][0]["carrier"],carrier);
        }
    }
    #[test]
    fn literal_membership_uses_s8_and_empty_resolves_original_field(){
        let mut r:Value=serde_json::from_str(include_str!("../../../tests/spark-pages/read-request.json")).unwrap();r["keyset"]=json!(false);r["snapshots"]=json!({"orders":1});r["parameters"]=json!([]);
        for op in ["IN","NOT IN"]{r["sql"]=json!(format!("SELECT o.id AS id FROM \"commerce\".\"Order\" o WHERE o.status {op} (NULL,'missing') LIMIT 10"));let p:Value=serde_json::from_str(&crate::raw_read::compile_json(&r.to_string())).unwrap();assert_eq!(p["status"],"compiled","{p}");let sql=p["sql"].as_str().unwrap();assert!(sql.contains("THEN CAST(NULL AS BOOLEAN) ELSE (FALSE OR"));assert!(sql.contains("IS NOT NULL AND"));}
        r["sql"]=json!("SELECT o.id AS id FROM \"commerce\".\"Order\" o WHERE NOT IN_EMPTY(o.status) LIMIT 10");let p:Value=serde_json::from_str(&crate::raw_read::compile_json(&r.to_string())).unwrap();assert_eq!(p["status"],"compiled","{p}");assert!(p["sql"].as_str().unwrap().contains("NOT (FALSE)"));
        for expr in ["IN_EMPTY()","IN_EMPTY(o.status,o.id)","IN_EMPTY(foreign.status)","IN_EMPTY(o.missing)","IN_EMPTY(NULL)","IN_EMPTY(o.id+1)","IN_EMPTY(DISTINCT o.status)"]{r["sql"]=json!(format!("SELECT o.id AS id FROM \"commerce\".\"Order\" o WHERE {expr} LIMIT 10"));let p:Value=serde_json::from_str(&crate::raw_read::compile_json(&r.to_string())).unwrap();assert_eq!(p["status"],"blocked","{expr}: {p}");}
    }
    #[test]
    fn operands_are_single_owned_occurrences_and_numeric_coercion_is_absent() {
        let sql = comparison(":p1", ">=", "CAST(`r`.`amount` AS STRING)");
        assert_eq!(sql.matches(":p1").count(), 1);
        assert_eq!(sql.matches("`r`.`amount`").count(), 1);
        assert!(!sql.contains("DOUBLE") && !sql.contains("DECIMAL("));
        assert!(sql.contains("CAST(NULL AS BOOLEAN)"));
    }
}
