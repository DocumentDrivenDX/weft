use serde_json::{json, Value};
use weft_core::json::sha256;
pub const SALES_SQL: &str = "SELECT c.name, SUM(o.total) AS total FROM Customer c JOIN Orders o ON o.customer_id = c.id GROUP BY c.name";
pub fn identity(element: &str) -> Value {
    json!({"documentId":"sales-fixture","revision":"1","module":"sales","element":element})
}
pub fn request(sql: &str) -> Value {
    let document = include_str!("../../docs/helix/03-test/fixtures/sales.umf.json");
    let pin = json!({"documentId":"sales-fixture","revision":"1","umfVersion":"0.7.0","sha256":sha256(document.as_bytes())});
    let records=[("customer","1",vec![("customer-id","23"),("customer-name","24"),("customer-active","25")]),("orders","2",vec![("order-customer","26"),("order-total","27")])]
        .into_iter().map(|(element,type_id,fields)|json!({"logical":identity(element),"table":0,"kind":"object","sourceSystem":"weft-synthetic","typeId":type_id,"schemaRevision":"fixture-schema-1",
        "properties":fields.into_iter().map(|(e,p)|json!({"logical":identity(e),"home":{"kind":"props","propertyId":p}})).collect::<Vec<_>>()})).collect::<Vec<_>>();
    let binding=json!({"profile":"ashlar-databricks-candidate/0.1.0","layoutRevision":"ashlar-delta/0.3","layoutSha256":"ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e",
    "modelPins":[pin.clone()],"publication":{"id":"weft-fixture-publication-1","manifestUuid":"fixture-manifest-uuid","tables":[{"name":["client_dev","weft_b006_fixture","object_current"],"uuid":"fixture-table-uuid","version":0}]},"records":records}).to_string();
    json!({"interfaceVersion":"weft-compile/0.1.0","dialect":"weft-sql/0.1.0","sql":sql,
    "modules":[{"documentJson":document,"pin":pin,"selectedModuleIds":["sales"]}],
    "target":{"backendId":"ashlar.databricks","backendVersion":"0.1.0-candidate","targetProfile":"dbsql-candidate","bindingJson":binding,"bindingSha256":sha256(binding.as_bytes())},"options":{"allowCandidate":true}})
}
