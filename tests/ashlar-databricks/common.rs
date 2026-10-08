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

pub fn relationship_request(sql: &str, projection: bool) -> Value {
    let mut request = request(sql);
    request["interfaceVersion"] = json!("weft-compile/0.2.0");
    request["dialect"] = json!("weft-sql/0.2.0");
    let mut document: Value =
        serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    let elements = document["modules"][0]["elements"].as_array_mut().unwrap();
    elements[0]["keys"] = json!([{"id":"customer-pk","name":"primary","fields":[{"module":"sales","element":"customer-id"}],"primary":true}]);
    elements[1]["keys"] = json!([{"id":"order-pk","name":"primary","fields":[{"module":"sales","element":"order-customer"}],"primary":true}]);
    let definition = json!({"id":"customer-orders","name":"orders","inverse":"customer","source":[{"module":"sales","element":"customer"}],"target":[{"module":"sales","element":"orders","key":"order-pk"}],"directed":true,"sourceMultiplicity":{"min":0,"max":"*"},"targetMultiplicity":{"min":0,"max":"*"},"targetLifecycle":"independent"});
    document["modules"][0]["relationships"] = json!([definition.clone()]);
    let raw = document.to_string();
    let digest = sha256(raw.as_bytes());
    request["modules"][0]["documentJson"] = json!(raw);
    request["modules"][0]["pin"]["sha256"] = json!(digest);
    let mut binding: Value =
        serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
    binding["modelPins"][0]["sha256"] = json!(digest);
    if projection {
        for record in binding["records"].as_array_mut().unwrap() {
            record["kind"] = json!("nodeProjection");
        }
    }
    binding["publication"]["tables"].as_array_mut().unwrap().push(json!({"name":["client_dev","weft_b006_fixture",if projection {"edge_ab"} else {"edge_current"}],"uuid":"fixture-edge-uuid","version":0}));
    binding["relationships"] = json!([{"logical":{"documentId":"sales-fixture","revision":"1","module":"sales","relationship":"customer-orders"},"acceptedDefinition":definition,"table":1,"kind":if projection {"edgeProjection"} else {"edge"},"sourceSystem":"weft-synthetic","typeId":"3","schemaRevision":"fixture-schema-1","source":identity("customer"),"target":identity("orders")}]);
    let raw = binding.to_string();
    request["target"]["bindingJson"] = json!(raw);
    request["target"]["bindingSha256"] = json!(sha256(raw.as_bytes()));
    request
}

pub fn compound_request(shape: &str, optional: bool) -> Value {
    let mut request = request("SELECT c.payload FROM Customer c");
    request["interfaceVersion"] = json!("weft-compile/0.2.0");
    request["dialect"] = json!("weft-sql/0.2.0");
    let mut doc:Value=serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    let elements=doc["modules"][0]["elements"].as_array_mut().unwrap();
    elements[0]["members"].as_array_mut().unwrap().push(json!({"module":"sales","element":"payload"}));
    let mut root=json!({"id":"payload","name":"payload","kind":"field","cardinality":"array","nullability":if optional {"absent-allowed"} else {"required"},"itemType":{"module":"sales","element":"leaf"},"extensions":{}});
    let leaf=json!({"id":"leaf","name":"leaf","kind":"field","cardinality":"one","nullability":"required","scalarType":"integer","facets":{"integerWidth":{"bits":64,"signed":true}},"extensions":{}});
    match shape {
        "map" => root["cardinality"]=json!("map"),
        "nested" => {
            root["itemType"]=json!({"module":"sales","element":"row"});
            elements.push(json!({"id":"row","name":"row","kind":"field","cardinality":"array","nullability":"required","itemType":{"module":"sales","element":"leaf"},"extensions":{}}));
        },
        "structured" | "cyclic" => {
            root["cardinality"]=json!("one");root.as_object_mut().unwrap().remove("itemType");
            root["references"]=json!([{"module":"sales","element":"record","role":"record-type"}]);
            let mut members=vec![json!({"module":"sales","element":"leaf"}),json!({"module":"sales","element":"note"})];
            elements.push(json!({"id":"note","name":"note","kind":"field","cardinality":"one","nullability":"absent-allowed","scalarType":"string","extensions":{}}));
            if shape=="cyclic" {
                members.push(json!({"module":"sales","element":"next"}));
                elements.push(json!({"id":"next","name":"next","kind":"field","cardinality":"one","nullability":"absent-allowed","references":[{"module":"sales","element":"record","role":"record-type"}],"extensions":{}}));
            }
            elements.push(json!({"id":"record","name":"PayloadRecord","kind":"record","members":members,"extensions":{}}));
        },
        _ => {}
    }
    elements.push(root);elements.push(leaf);
    let raw=doc.to_string();let digest=sha256(raw.as_bytes());
    request["modules"][0]["documentJson"]=json!(raw);request["modules"][0]["pin"]["sha256"]=json!(digest);
    let mut binding:Value=serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
    binding["modelPins"][0]["sha256"]=json!(digest);
    binding["records"][0]["properties"].as_array_mut().unwrap().push(json!({"logical":identity("payload"),"home":{"kind":"props","propertyId":"28","encoding":"ashlar-weft-json-value/0.1-candidate"}}));
    let raw=binding.to_string();request["target"]["bindingJson"]=json!(raw);request["target"]["bindingSha256"]=json!(sha256(raw.as_bytes()));request
}
