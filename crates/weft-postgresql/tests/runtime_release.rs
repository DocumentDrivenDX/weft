// Compiler-only compatibility: hosts still authenticate the original binding.
use serde_json::{json, Value};
use weft_core::json::sha256;

fn request(release: &str, backend: &str) -> Value {
    let document = json!({"umf":"0.8.0","id":"compat","vocabularies":{},"extensions":{},"modules":[{"id":"m","namespace":"m","elements":[
        {"id":"record","name":"Record","kind":"record","extensions":{},"members":[{"module":"m","element":"id"}],"keys":[{"id":"pk","name":"pk","primary":true,"fields":[{"module":"m","element":"id"}]}]},
        {"id":"id","name":"id","kind":"field","extensions":{},"scalarType":"string","cardinality":"one","nullability":"required"}
    ]}]}).to_string();
    let digest = sha256(document.as_bytes());
    let identity = |element: &str| json!({"documentId":"compat","revision":"r1","module":"m","element":element});
    let binding = json!({"interfaceVersion":"truss-runtime-binding/0.4","schema":"truss","installationId":"fixture",
        "releaseId":release,"releaseSha256":if release=="truss-catalog-runtime/0.67-candidate" {"8c553480e1bcb95ccaebb9aa04150a0253217838c8df6ad09acd5ecece2402ee"} else if release=="truss-catalog-runtime/0.66-candidate" {"99ae7218e0e3e68e0a3682e43949089185d967eb21bbf31deb0b8f8f6fc2de9a"} else {"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},"catalogRevision":1,"maximumScanRows":100,"maximumResultRows":10,"maximumBytes":100000,
        "documents":[{"documentId":"compat","revision":"r1","sha256":digest}],
        "records":[{"identity":identity("record"),"typeId":"1","fields":[{"identity":identity("id"),"propertyId":"1"}],"keys":[{"keyId":"pk","propertyIds":["1"]}]}],"relationships":[]}).to_string();
    json!({"interfaceVersion":"weft-compile/0.2.6","dialect":"weft-sql/0.2.0","timestampComparisonProfile":"umf-rfc3339-exact-instant/1.0.0","options":{"allowCandidate":true},"parameters":{},"exactParameters":{},"sql":"SELECT r.id FROM Record r ORDER BY r.id LIMIT 10",
        "modules":[{"documentJson":document,"pin":{"documentId":"compat","revision":"r1","umfVersion":"0.8.0","sha256":digest},"selectedModuleIds":["m"]}],
        "readProfile":{"version":"weft-application-read/0.2.0","subset":"entity-page"},
        "target":{"backendId":"truss.postgresql.runtime","backendVersion":backend,"targetProfile":"truss-runtime-pg18.6/0.4","bindingJson":binding,"bindingSha256":sha256(binding.as_bytes())}})
}

#[test]
fn owned_runtime_accepts_only_qualified_releases() {
    for release in ["truss-catalog-runtime/0.65-candidate", "truss-catalog-runtime/0.66-candidate", "truss-catalog-runtime/0.67-candidate"] {
        let response: Value = serde_json::from_str(&weft_postgresql::truss_runtime::compile_json(&request(release,if release=="truss-catalog-runtime/0.65-candidate" {"0.4.35"} else {"0.4.36"}).to_string())).unwrap();
        assert_eq!(response["status"], "compiled", "{release}: {response}");
    }
    for (release, backend) in [("truss-catalog-runtime/0.68-candidate","0.4.35"), ("truss-catalog-runtime/0.64-candidate","0.4.35"), ("truss-catalog-runtime/0.66-candidate","0.4.34")] {
        let response: Value = serde_json::from_str(&weft_postgresql::truss_runtime::compile_json(&request(release,backend).to_string())).unwrap();
        assert_ne!(response["status"], "compiled", "{release}/{backend}: {response}");
        assert!(response.get("sql").is_none(), "{response}");
    }
}

#[test]
fn page_and_mutation_admission_keep_original_release_identity() {
    use weft_core::mutation::Artifact;
    use weft_postgresql::truss_mutation::CurrentTrussMutationBackend;
    for (release, accepted) in [("truss-catalog-runtime/0.65-candidate", true), ("truss-catalog-runtime/0.66-candidate", true), ("truss-catalog-runtime/0.67-candidate", true), ("truss-catalog-runtime/0.68-candidate", false), ("truss-catalog-runtime/0.64-candidate", false)] {
        let mut r = request(release, if release=="truss-catalog-runtime/0.65-candidate" {"0.4.35"} else {"0.4.36"});
        r.as_object_mut().unwrap().remove("readProfile");
        r["sql"] = json!("SELECT r.* FROM Record r");
        let page: Value = serde_json::from_str(&weft_postgresql::truss_runtime::compile_page_json(&r.to_string(), 5, None)).unwrap();
        assert_eq!(page["status"] == "compiled", accepted, "{release}: {page}");
        let artifact = |text: &str| Artifact { media_type: "application/json".into(), utf8: text.into(), sha256: sha256(text.as_bytes()) };
        let result = CurrentTrussMutationBackend::new(
            artifact(r["modules"][0]["documentJson"].as_str().unwrap()),
            artifact(r["target"]["bindingJson"].as_str().unwrap()),
            serde_json::from_value(r["modules"][0]["pin"].clone()).unwrap(),
            vec!["fixture-only: no native execution or authority claim".into()],
        );
        assert_eq!(result.is_ok(), accepted, "mutation admission for {release}");
    }
}
