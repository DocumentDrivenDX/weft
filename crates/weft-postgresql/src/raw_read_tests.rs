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
    fn signed_numeric_literals_preserve_carrier_and_refuse_signed_expressions() {
        let mut r=request();r["parameters"]=json!([]);
        for predicate in ["o.customer_id = -1", "o.customer_id IN (-1,+2,NULL)", "o.amount = -1.0", "HAS_RELATED(o.customer,KEY(-1))"] {
            r["sql"]=json!(format!("SELECT o.id AS id FROM commerce.\"Order\" o WHERE {predicate} LIMIT 10"));
            let p=compile(r.clone());assert_eq!(p["status"],"compiled","{predicate}: {p}");
            assert!(p["sql"].as_str().unwrap().contains("(-1"));
        }
        for predicate in ["o.customer_id = -o.customer_id", "o.customer_id = -'1'", "o.customer_id = -1.0"] {
            r["sql"]=json!(format!("SELECT o.id AS id FROM commerce.\"Order\" o WHERE {predicate} LIMIT 10"));
            assert_eq!(compile(r.clone())["status"],"blocked","{predicate}");
        }
    }
    #[test]
    fn explicit_inverse_self_relationship_retains_canonical_identity_and_default_key(){
        let mut r=request();let mut doc:Value=serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();let module=&mut doc["modules"][0];
        let mut parent=module["elements"].as_array().unwrap().iter().find(|f|f["id"]=="Customer_id").unwrap().clone();parent["id"]=json!("Customer_parent_id");parent["name"]=json!("parent_id");parent["nullability"]=json!("absent-allowed");module["elements"].as_array_mut().unwrap().push(parent);
        let mut code=module["elements"].as_array().unwrap().iter().find(|f|f["id"]=="Customer_name").unwrap().clone();code["id"]=json!("Customer_code");code["name"]=json!("code");code["nullability"]=json!("required");module["elements"].as_array_mut().unwrap().push(code);
        let record=module["elements"].as_array_mut().unwrap().iter_mut().find(|f|f["id"]=="Customer").unwrap();record["members"].as_array_mut().unwrap().extend([json!({"module":"commerce","element":"Customer_parent_id"}),json!({"module":"commerce","element":"Customer_code"})]);record["keys"].as_array_mut().unwrap().push(json!({"id":"Customer_code_key","name":"Customer code key","fields":[{"module":"commerce","element":"Customer_code"}]}));
        let mut relation=module["relationships"][0].clone();relation["id"]=json!("self-parent");relation["name"]=json!("parent");relation["source"][0]["element"]=json!("Customer");module["relationships"].as_array_mut().unwrap().push(relation);
        let raw=doc.to_string();r["binding"]["modelSha256"]=json!(weft_core::json::sha256(raw.as_bytes()));r["binding"]["modelJson"]=json!(raw);
        let table=r["binding"]["tables"].as_array_mut().unwrap().iter_mut().find(|t|t["logical"]==json!(["commerce","Customer"])).unwrap();table["key"]=json!(["code"]);table["columns"].as_array_mut().unwrap().extend([json!({"logical":"parent_id","native":"parent_id","carrier":"integer","nullable":true}),json!({"logical":"code","native":"code","carrier":"string","nullable":false})]);
        r["binding"]["relationships"].as_array_mut().unwrap().push(json!({"module":"commerce","id":"self-parent","source":["commerce","Customer"],"target":["commerce","Customer"],"pairs":[["parent_id","id"]],"toOne":true}));r["parameters"]=json!([{"name":"id","carrier":"integer","nullable":false}]);
        r["sql"]=json!("SELECT c.id AS id, RELATED_KEYS(c.parent,2) AS parent, INVERSE_RELATED_KEYS(c.parent,2,'commerce','self-parent') AS children FROM commerce.\"Customer\" c WHERE HAS_INVERSE_RELATED(c.parent,KEY(:id),'commerce','self-parent') LIMIT 10");let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["outputs"][2]["decoder"]["key"]["key"],"Customer_key");assert_eq!(p["outputs"][2]["decoder"]["relationship"]["direction"],"inverse");let sql=p["sql"].as_str().unwrap();assert!(sql.contains("\"c\".\"parent_id\" = \"__umf_relationship\".\"id\""),"{sql}");assert!(sql.contains("\"c\".\"id\" = \"__umf_relationship\".\"parent_id\""),"{sql}");
        for expression in ["HAS_INVERSE_RELATED(c.parent,KEY(:id))","HAS_INVERSE_RELATED(c.parent,KEY(:id),'commerce','wrong')","HAS_INVERSE_RELATED(c.parent,KEY(:id),'wrong','self-parent')"]{r["sql"]=json!(format!("SELECT c.id AS id FROM commerce.\"Customer\" c WHERE {expression} LIMIT 10"));assert_eq!(compile(r.clone())["status"],"blocked");}
    }
    #[test]
    fn selected_original_relationship_unknown_meaning_is_not_erased(){
        let r=request();for (key,value)in [("opaque",json!({"unrecognized":true})),("targetLifecycle",json!("unknown")),("extensions",json!({"unknown":{"x":1}})),("extensions",json!("malformed"))]{let mut bad=r.clone();let mut doc:Value=serde_json::from_str(bad["binding"]["modelJson"].as_str().unwrap()).unwrap();doc["modules"][0]["relationships"][0][key]=value;let raw=doc.to_string();bad["binding"]["modelSha256"]=json!(weft_core::json::sha256(raw.as_bytes()));bad["binding"]["modelJson"]=json!(raw);assert_eq!(compile(bad)["status"],"blocked", "{key}");}
    }
    #[test]
    fn empty_membership_preserves_semantic_field_without_fictitious_sql_column_use(){
        let mut r=request();r["parameters"]=json!([]);r["sql"]=json!("SELECT o.id AS id FROM commerce.\"Order\" o WHERE NOT IN_EMPTY(o.status) LIMIT 10");let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");assert!(p["sql"].as_str().unwrap().contains("NOT (FALSE)"));assert_eq!(p["privilegeInventory"]["tables"][0]["columns"],json!(["id"]));assert_eq!(p["privilegeInventory"]["semanticFields"][0]["field"],"status");
        for expr in ["IN_EMPTY()","IN_EMPTY(o.status,o.id)","IN_EMPTY(wrong.status)","IN_EMPTY(o.missing)","IN_EMPTY(NULL)","IN_EMPTY(o.id+1)","IN_EMPTY(DISTINCT o.status)"]{r["sql"]=json!(format!("SELECT o.id AS id FROM commerce.\"Order\" o WHERE {expr} LIMIT 10"));assert_eq!(compile(r.clone())["status"],"blocked","{expr}");}
    }
    #[test]
    fn native_fk_admission_requires_enforced_immediate_pairs_and_degree_snapshot(){
        let mut r=request();r["parameters"]=json!([]);r["sql"]=json!("SELECT o.id AS id FROM commerce.\"Order\" o INNER JOIN commerce.\"Customer\" c ON o.customer_id=c.id LIMIT 10");let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");let fk=&p["obligations"]["nativeRelationships"]["relationships"][0];assert_eq!(fk["nativePairs"],json!([["customer_id","id"]]));assert_eq!(fk["completeNativeSnapshotRequired"],false);let sql=fk["nativeAdmission"]["sql"].as_str().unwrap();for fragment in ["c.convalidated AND c.conenforced","NOT c.condeferrable","NOT c.conperiod","c.conpfeqop","tr.tgenabled IN ('O','A')","session_replication_role","fc.collisdeterministic","fi.indexrelid=c.conindid","fi.indcollation","am.amname='btree'","op.opcname=CASE ty.typname"]{assert!(sql.contains(fragment),"{fragment}: {sql}")}
        let mut doc:Value=serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();doc["modules"][0]["relationships"][0]["sourceMultiplicity"]["min"]=json!(1);let raw=doc.to_string();r["binding"]["modelSha256"]=json!(weft_core::json::sha256(raw.as_bytes()));r["binding"]["modelJson"]=json!(raw);let p=compile(r);assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["obligations"]["readIsolation"],"repeatable-read");let sql=p["obligations"]["nativeRelationships"]["relationships"][0]["nativeAdmission"]["dbapiSql"].as_str().unwrap();for fragment in ["NOT pg_catalog.row_security_active(s.oid)","NOT pg_catalog.row_security_active(t.oid)","transaction_isolation","count(DISTINCT ROW(","transaction_read_only"]{assert!(sql.contains(fragment),"{fragment}: {sql}")}
    }
    #[test]
    fn relationship_exists_and_bounded_keys_keep_original_endpoint_key(){
        let mut r=request();r["parameters"]=json!([{"name":"customer","carrier":"integer","nullable":false}]);r["sql"]=json!("SELECT o.id AS id, RELATED_KEYS(o.customer,2) AS customer_keys FROM commerce.\"Order\" o WHERE HAS_RELATED(o.customer,KEY(:customer)) LIMIT 10");
        let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["outputs"][1]["carrier"],"related-keys");assert_eq!(p["outputs"][1]["decoder"]["key"]["key"],"Customer_key");assert_eq!(p["outputs"][1]["decoder"]["bound"],2);let sql=p["sql"].as_str().unwrap();assert!(sql.contains("EXISTS(SELECT 1 FROM"));assert!(sql.contains("LIMIT 3"));assert!(sql.contains("pg_catalog.count(*)>2"));assert_eq!(p["lockSql"].as_array().unwrap().len(),2);
        for q in ["SELECT RELATED_KEYS(o.customer,0) AS keys FROM commerce.\"Order\" o LIMIT 10","SELECT RELATED_KEYS(o.customer,1001) AS keys FROM commerce.\"Order\" o LIMIT 10","SELECT o.id AS id FROM commerce.\"Order\" o WHERE HAS_RELATED(o.customer,KEY()) LIMIT 10","SELECT \"__umf_relationship\".id AS id, RELATED_KEYS(o.customer,2) AS keys FROM commerce.\"Order\" o LIMIT 10"]{let mut bad=r.clone();bad["parameters"]=json!([]);bad["sql"]=json!(q);assert_eq!(compile(bad)["status"],"blocked");}
        let mut doc:Value=serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();doc["modules"][0]["relationships"][0]["inverse"]=json!("orders");let raw=doc.to_string();r["binding"]["modelSha256"]=json!(weft_core::json::sha256(raw.as_bytes()));r["binding"]["modelJson"]=json!(raw);r["parameters"]=json!([]);r["sql"]=json!("SELECT c.id AS id, RELATED_KEYS(c.orders,1) AS orders FROM commerce.\"Customer\" c LIMIT 10");let p=compile(r);assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["outputs"][1]["decoder"]["key"]["key"],"Order_key");assert_eq!(p["outputs"][1]["decoder"]["relationship"]["direction"],"inverse");
    }
    #[test]
    fn list_membership_retains_bounded_container_and_s8_null_semantics(){
        let mut r=request();r["parameters"]=json!([{"name":"ids","carrier":"integer","nullable":false,"container":"list","elementNullable":true,"maxItems":1000}]);
        for op in ["IN","NOT IN"]{r["sql"]=json!(format!("SELECT o.id AS id FROM commerce.\"Order\" o WHERE o.id {op} (:ids) LIMIT 10"));let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["parameters"],r["parameters"]);let sql=p["sql"].as_str().unwrap();assert!(sql.contains("pg_catalog.cardinality($1::pg_catalog.numeric[])=0 THEN FALSE"),"{sql}");assert!(sql.contains("IS NOT NULL AND"));assert_eq!(p["parameterOrder"],json!(["ids"]));}
        for sql in ["SELECT :ids AS ids FROM commerce.\"Order\" o LIMIT 10","SELECT o.id AS id FROM commerce.\"Order\" o WHERE o.id IN (:ids,1) LIMIT 10"]{r["sql"]=json!(sql);assert_eq!(compile(r.clone())["status"],"blocked");}
        r["sql"]=json!("SELECT o.id AS id FROM commerce.\"Order\" o WHERE o.id IN (:ids) LIMIT 10");for (key,value) in [("nullable",json!(true)),("maxItems",json!(0)),("maxItems",json!(1001)),("container",json!("set"))]{let mut bad=r.clone();bad["parameters"][0][key]=value;assert_eq!(compile(bad)["status"],"blocked");}
        let mut bad=r.clone();bad["parameters"][0].as_object_mut().unwrap().remove("elementNullable");assert_eq!(compile(bad)["status"],"blocked");
        r["parameters"]=json!([]);r["sql"]=json!("SELECT o.id AS id FROM commerce.\"Order\" o WHERE o.id IN (NULL,1) LIMIT 10");let p=compile(r);assert_eq!(p["status"],"compiled","{p}");assert!(p["sql"].as_str().unwrap().contains("ELSE (FALSE OR"));
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
    #[test]
    fn native_projection_retains_original_binary_vector_and_admission(){
        let r:Value=serde_json::from_str(include_str!("../../../tests/raw-read/postgresql-projection-request.json")).unwrap();
        let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");
        assert_eq!(p["outputs"][2]["carrier"],"vector");assert_eq!(p["outputs"][5]["decoder"]["profile"],"pg-bytea/0.1");
        assert_eq!(p["obligations"]["nativeProjection"]["columns"].as_array().unwrap().len(),7);
        let inventory=&p["obligations"]["nativeProjection"]["columns"][6]["nativeAdmission"];
        assert!(inventory["sql"].as_str().unwrap().contains("a.attnum>0 AND NOT a.attisdropped"));
        assert!(inventory["sql"].as_str().unwrap().contains("a.attname COLLATE pg_catalog.\"C\""));
        let names:Vec<String>=serde_json::from_str(inventory["parameters"][2]["source"]["literal"]["string"].as_str().unwrap()).unwrap();
        assert_eq!(names,vec!["amount","body","count","embedding","id","payload"]);
        for sql in ["SELECT r.embedding AS embedding FROM search.\"Doc\" r ORDER BY embedding LIMIT 10","SELECT COUNT(r.embedding) AS n FROM search.\"Doc\" r LIMIT 10"] {let mut bad=r.clone();bad["sql"]=json!(sql);assert_eq!(compile(bad)["status"],"blocked");}
        let mut bad=r.clone();bad["binding"]["nativeColumns"]["vectorExtension"]["version"]=json!("0.8.8");assert_eq!(compile(bad)["status"],"blocked");
        for (field,ty) in [("payload","text"),("embedding","array<double>"),("count","numeric(20,0)"),("amount","numeric(38,9)")] {
            let mut bad=r.clone();let mut doc:Value=serde_json::from_str(bad["binding"]["modelJson"].as_str().unwrap()).unwrap();
            let cols=doc["modules"][0]["extensions"]["umf.access"]["columns"][0]["columns"].as_array_mut().unwrap();for c in cols {if c["field"]["element"]==field {c["type"]=json!(ty);}}
            let raw=doc.to_string();let sha=weft_core::json::sha256(raw.as_bytes());bad["binding"]["modelJson"]=json!(raw);bad["binding"]["modelSha256"]=json!(sha);bad["binding"]["nativeColumns"]["sets"][0]["columnSet"]["documentSha256"]=json!(sha);
            assert_eq!(compile(bad)["status"],"blocked","{field}/{ty}");
        }
        let mut bad=r.clone();bad["binding"]["nativeColumns"]["target"]["platform"]=json!("spark");assert_eq!(compile(bad)["status"],"blocked");
        let mut bad=r.clone();bad["binding"]["nativeColumns"]["sets"][0]["binding"]["documentSha256"]=json!("0".repeat(64));assert_eq!(compile(bad)["status"],"blocked");

    }
    #[test]
    fn source_exact_verification_requires_complete_authored_collation_facts(){
        let native:Value=serde_json::from_str(include_str!("../../../tests/raw-read/postgresql-source-exact-drift-request.json")).unwrap();
        let np=compile(native);assert_eq!(np["status"],"compiled","{np}");assert_eq!(np["outputs"].as_array().unwrap().len(),1);assert_eq!(np["obligations"]["nativeProjection"]["columns"].as_array().unwrap().len(),4);

        let r:Value=serde_json::from_str(include_str!("../../../tests/raw-read/postgresql-source-exact-request.json")).unwrap();
        let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");
        assert_eq!(p["obligations"]["nativeProjection"]["verificationProfile"],"source-exact/0.1");
        let admissions=p["obligations"]["nativeProjection"]["columns"].as_array().unwrap();
        assert_eq!(admissions.len(),7);
        assert!(admissions.iter().any(|a|a["nativeAdmission"]["sql"].as_str().unwrap().contains("co.oid=a.attcollation")));
        assert!(admissions.iter().any(|a|a["nativeAdmission"]["sql"].as_str().unwrap().contains("a.attcollation=0")));
        assert!(admissions.iter().any(|a|a["nativeAdmission"]["sql"].as_str().unwrap().contains("dep.objid=ty.oid")));
        assert!(admissions.iter().any(|a|a["nativeAdmission"]["parameters"].as_array().unwrap().iter().any(|p|p["source"]["literal"]["string"]=="default")));
        for profile in ["source-exact/0.2",""]{let mut b=r.clone();b["verificationProfile"]=json!(profile);assert_eq!(compile(b)["status"],"blocked");}
        let mut b=r.clone();b["binding"].as_object_mut().unwrap().remove("nativeColumns");assert_eq!(compile(b)["status"],"blocked");
        for value in [None,Some(json!({"schema":"pg_catalog"})),Some(json!("default")),Some(json!(null)),Some(json!({"schema":"", "name":"default"})),Some(json!({"schema":"pg_catalog", "name":"a".repeat(64)})),Some(json!({"schema":"pg_catalog", "name":"é".repeat(32)}))]{
            let mut b=r.clone();let mut d:Value=serde_json::from_str(b["binding"]["modelJson"].as_str().unwrap()).unwrap();
            let c=&mut d["modules"][0]["extensions"]["umf.access"]["columns"][0]["columns"][0];
            if let Some(v)=value{c["collation"]=v}else{c.as_object_mut().unwrap().remove("collation");}
            let raw=d.to_string();let sha=weft_core::json::sha256(raw.as_bytes());b["binding"]["modelJson"]=json!(raw);b["binding"]["modelSha256"]=json!(sha);b["binding"]["nativeColumns"]["sets"][0]["columnSet"]["documentSha256"]=json!(sha);
            assert_eq!(compile(b)["status"],"blocked");
        }
        let legacy:Value=serde_json::from_str(include_str!("../../../tests/raw-read/postgresql-projection-request.json")).unwrap();
        let lp=compile(legacy);assert!(lp["obligations"]["nativeProjection"].get("verificationProfile").is_none());assert!(lp["obligations"]["nativeProjection"]["columns"].as_array().unwrap().iter().all(|a|!a["nativeAdmission"]["sql"].as_str().unwrap().contains("dep.objid=ty.oid")));
    }
    fn compile(r: Value) -> Value {
        serde_json::from_str(&compile_json(&r.to_string())).unwrap()
    }
    fn term_request()->Value {
        let search:Value=serde_json::from_str(include_str!("../../../tests/native-search/keyword-request.json")).unwrap();
        let mut r=search["selection"].clone();r["binding"]["nativeCatalogFence"]=json!({"version":"umf.pg-ts-fence/0.1","schema":"native_fence","view":"ts_catalog_lock","owner":"fence_owner","ownerProfile":"non-login-maintainer/0.1"});r["sql"]=json!("SELECT r.id AS id FROM \"search\".\"Doc\" AS r WHERE MATCHES_ALL_TERMS(r.body,:terms)");r["parameters"]=json!([{ "name":"terms","carrier":"string","nullable":false }]);r
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

    #[test]
    fn explicit_schema_binding_and_stream_limit_are_owner_bound(){
        let mut r=request();r["parameters"]=json!([]);r["sql"]=json!("SELECT o.id AS id FROM commerce.\"Order\" o ORDER BY o.id LIMIT 7");
        r["binding"]["schemaBinding"]=json!("qualified");r["binding"]["schema"]=json!("app_one");
        let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["obligations"]["schemaBinding"],"qualified");
        r["binding"]["schemaBinding"]=json!("search-path");assert_eq!(compile(r.clone())["status"],"blocked");
        r["binding"]["schema"]=Value::Null;r["keyset"]=json!(true);r["streamPageSize"]=json!(true);
        let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["obligations"]["streamPageSize"],json!({"parameter":"__umf_page_size","minimum":1,"maximum":7}));
        assert!(p["sql"].as_str().unwrap().ends_with("::pg_catalog.int8"));assert_eq!(p["cursorParameters"].as_array().unwrap().last().unwrap()["name"],"__umf_page_size");
        r["keyset"]=json!(false);assert_eq!(compile(r)["status"],"blocked");
    }

    #[test]
    fn alternate_relationship_key_preserves_selected_key_and_requires_native_uniqueness(){
        let mut r=request();let mut doc:Value=serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();
        doc["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":"Customer_code","name":"code","kind":"field","scalarType":"integer","cardinality":"one","nullability":"required","extensions":{}}));
        r["binding"]["tables"][1]["columns"].as_array_mut().unwrap().push(json!({"logical":"code","native":"code","carrier":"integer","nullable":false}));
        let record=doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|v|v["id"]=="Customer").unwrap();
        record["members"].as_array_mut().unwrap().push(json!({"module":"commerce","element":"Customer_code"}));
        record["keys"].as_array_mut().unwrap().push(json!({"id":"Customer_code_key","name":"Customer code key","fields":[{"module":"commerce","element":"Customer_code"}],"primary":false}));
        doc["modules"][0]["relationships"][0]["target"][0]["key"]=json!("Customer_code_key");
        let raw=doc.to_string();r["binding"]["modelSha256"]=json!(weft_core::json::sha256(raw.as_bytes()));r["binding"]["modelJson"]=json!(raw);r["binding"]["relationships"][0]["pairs"]=json!([["customer_id","code"]]);r["parameters"]=json!([]);
        r["sql"]=json!("SELECT o.id AS id, c.id AS customer_id FROM commerce.\"Order\" o LEFT JOIN commerce.\"Customer\" c ON o.customer_id=c.code LIMIT 10");
        let p=compile(r.clone());assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["binding"]["tables"][1]["key"],json!(["id"]));assert_eq!(p["obligations"]["nativeRelationshipKeys"]["keys"][0]["logicalFields"],json!(["code"]));let key_sql=p["obligations"]["nativeRelationshipKeys"]["keys"][0]["nativeAdmission"]["sql"].as_str().unwrap();assert!(key_sql.contains("i.indcollation")&&key_sql.contains("co.collisdeterministic"));
        r["binding"]["relationships"][0]["pairs"]=json!([["customer_id","id"]]);assert_eq!(compile(r)["status"],"blocked");
    }

}
