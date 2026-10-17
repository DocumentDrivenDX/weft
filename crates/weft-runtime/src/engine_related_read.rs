//! Engine-owned entity-set navigation. No native SQL is composed here.
use serde::Deserialize;
use serde_json::{json, Value};
type Result<T> = std::result::Result<T, &'static str>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Relationship {
    module: String,
    id: String,
    direction: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    version: String,
    compiler: String,
    source: Value,
    target: Value,
    relationship: Relationship,
}
pub(crate) fn compile_value(packet: &Value) -> Result<Value> {
    let request: Request = serde_json::from_value(packet.clone())
        .map_err(|_| "closed original engine navigation packet required")?;
    if request.version != "weft.engine-related-read/0.1"
        || !matches!(
            request.relationship.direction.as_str(),
            "forward" | "inverse"
        )
    {
        return Err("original engine navigation version/direction differs");
    }
    let source_sql = request.source["sql"]
        .as_str()
        .ok_or("original source logical SQL required")?;
    if source_sql.len() > 65536
        || request.relationship.module.is_empty()
        || request.relationship.id.is_empty()
    {
        return Err("bounded source and canonical relationship required");
    }
    let mut context = request.source.clone();
    let object = context
        .as_object_mut()
        .ok_or("original source compile request required")?;
    object.insert("sql".into(), request.target["sql"].clone());
    if context != request.target {
        return Err("original source/target engine context differs");
    }
    let input = weft_core::application_navigation::Input {
        source_sql: source_sql.into(),
        relationship_module: request.relationship.module,
        relationship_id: request.relationship.id,
        inverse: request.relationship.direction == "inverse",
    };
    let raw = if request.compiler == "truss-runtime" {
        weft_postgresql::truss_runtime::compile_navigation_json(
            &request.target.to_string(),
            &input,
            None,
            false,
        )
    } else {
        let mut factory = |_: &weft_core::model::Catalog,
                           _: weft_core::backend::Plan<'_>,
                           _: weft_core::compile::CompositionInput<'_>| {
            let mut registry = weft_core::backend::Registry::default();
            match request.compiler.as_str() {
                "ashlar-spark4" => registry.register(weft_databricks::local_spark4::LocalSpark4)?,
                "ashlar-dbsql-20261016" => {
                    registry.register(weft_databricks::managed_20261016::Managed20261016)?
                }
                _ => {
                    return Err(weft_core::error::Diagnostic::new(
                        "WFT-ENGINE-NAVIGATION",
                        "register",
                        "Explicit original engine navigation compiler required",
                    ))
                }
            };
            Ok(registry)
        };
        weft_core::compile::Compiler::default().compile_navigation_json_with_factory(
            &request.target.to_string(),
            &input,
            &mut factory,
            false,
            false,
            None,
        )
    };
    let artifact = weft_core::json::checked_json(&raw)
        .map_err(|_| "original native navigation compiler output refused")?;
    let mut out = crate::engine_read::wrap_artifact(&request.compiler, &request.target, artifact)?;
    if out["status"] != "compiled" {
        return Ok(out);
    }
    out.as_object_mut().unwrap().remove("structuralDigest");
    out["relatedRequest"] = packet.clone();
    out["structuralDigest"] = json!({"owner":"weft-runtime-engine-read","version":"0.1","sha256":weft_core::json::sha256(weft_core::mutation::canonical(&out).as_bytes())});
    Ok(out)
}
pub fn compile_json(raw: &str) -> String {
    let result = (|| {
        if raw.len() > 4_194_304 {
            return Err("bounded original engine navigation packet required");
        };
        let packet = weft_core::json::checked_json(raw)
            .map_err(|_| "original navigation packet framing refused")?;
        compile_value(&packet)
    })();
    result.unwrap_or_else(|message|json!({"version":"weft.engine-read/0.1","status":"blocked","diagnostics":[{"code":"WFT-ENGINE-NAVIGATION","message":message}]})).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ashlar() -> Value {
        let mut source: Value = serde_json::from_str(include_str!(
            "../../../tests/engine-catalog/native-ashlar-related-request.json"
        ))
        .unwrap();
        source["parameters"] = json!({"status":{"family":"string","value":"paid"}});
        source["sql"] = json!(
            "SELECT r.id AS id FROM sales.\"order\" r WHERE r.status=:status ORDER BY r.id LIMIT 2"
        );
        let mut target = source.clone();
        target["sql"]=json!("SELECT c.\"customerId\" AS id FROM sales.customer c ORDER BY c.\"customerId\" LIMIT 10");
        json!({"version":"weft.engine-related-read/0.1","compiler":"ashlar-spark4","source":source,"target":target,"relationship":{"module":"sales","id":"customers","direction":"forward"}})
    }
    fn truss() -> Value {
        let bundle: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-query-abi/fixtures/native-query-bundle.json"
        ))
        .unwrap();
        let raw = bundle["bindingJson"].as_str().unwrap();
        let source = json!({"interfaceVersion":"weft-compile/0.2.6","dialect":"weft-sql/0.2.0","timestampComparisonProfile":"umf-rfc3339-exact-instant/1.0.0","modules":bundle["modules"],"sql":"SELECT a.\"A-label\" AS label FROM \"A\" a WHERE a.\"A-label\"=:needle ORDER BY a.\"A-label\" LIMIT 2","parameters":{"needle":{"family":"string","value":"first"}},"exactParameters":{},"options":{"allowCandidate":true},"target":{"backendId":"truss.postgresql.runtime","backendVersion":"0.4.39","targetProfile":"truss-runtime-pg18.6/0.5","bindingJson":raw,"bindingSha256":weft_core::json::sha256(raw.as_bytes())}});
        let mut target = source.clone();
        target["sql"] =
            json!("SELECT b.\"B-label\" AS label FROM \"B\" b ORDER BY b.\"B-label\" LIMIT 10");
        let document: Value =
            serde_json::from_str(source["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
        let module = document["modules"][0]["id"].clone();
        json!({"version":"weft.engine-related-read/0.1","compiler":"truss-runtime","source":source,"target":target,"relationship":{"module":module,"id":"A-B","direction":"forward"}})
    }
    #[test]
    fn original_engine_source_selection_preserves_identity_limit_parameters_and_context() {
        for packet in [ashlar(), truss()] {
            let p = compile_value(&packet).unwrap();
            assert_eq!(p["status"], "compiled", "{p}");
            assert_eq!(p["request"], packet["target"]);
            assert_eq!(p["relatedRequest"], packet);
            assert_eq!(
                p["artifact"]["logicalPlan"]["irVersion"],
                "weft-ir/engine-navigation/0.1"
            );
            assert_eq!(
                p["artifact"]["logicalPlan"]["navigation"]["source"]["limit"],
                2
            );
            assert!(p["artifact"]["sql"].as_str().unwrap().contains("EXISTS"));
            let checked: Value = serde_json::from_str(&crate::engine_read::check_json(
                &p.to_string(),
                &packet["target"].to_string(),
            ))
            .unwrap();
            assert_eq!(checked, p);
            let mut current = packet["target"].clone();
            let name = current["parameters"]
                .as_object()
                .unwrap()
                .keys()
                .next()
                .unwrap()
                .clone();
            current["parameters"][&name]["value"] = json!("second");
            let rebound: Value = serde_json::from_str(&crate::engine_read::check_json(
                &p.to_string(),
                &current.to_string(),
            ))
            .unwrap();
            assert_eq!(rebound["status"], "compiled", "{rebound}");
            assert_eq!(
                rebound["relatedRequest"]["source"]["parameters"],
                current["parameters"]
            );
            let mut unbounded = packet.clone();
            let sql = unbounded["source"]["sql"]
                .as_str()
                .unwrap()
                .strip_suffix(" LIMIT 2")
                .unwrap()
                .to_owned();
            unbounded["source"]["sql"] = json!(sql);
            let q = compile_value(&unbounded).unwrap();
            assert_eq!(q["status"], "compiled", "{q}");
            assert!(q["artifact"]["logicalPlan"]["navigation"]["source"]["limit"].is_null());
            let mut forged = p.clone();
            forged["relatedRequest"]["relationship"]["id"] = json!("other");
            let refused: Value = serde_json::from_str(&crate::engine_read::check_json(
                &forged.to_string(),
                &packet["target"].to_string(),
            ))
            .unwrap();
            assert_eq!(refused["status"], "blocked");
        }
    }
    #[test]
    fn complete_parameter_union_is_owned_across_both_original_queries(){
        for (name,sql) in [("target","SELECT c.\"customerId\" AS id FROM sales.customer c WHERE c.\"customerId\"=:target LIMIT 10"),("status","SELECT c.\"customerId\" AS id FROM sales.customer c WHERE c.\"customerId\"=:status LIMIT 10")]{
            let mut packet=ashlar();packet["target"]["sql"]=json!(sql);packet["source"]["parameters"][name]=json!({"family":"string","value":"paid"});packet["target"]["parameters"]=packet["source"]["parameters"].clone();let p=compile_value(&packet).unwrap();assert_eq!(p["status"],"compiled","{name}: {p}");assert_eq!(p["request"]["parameters"],packet["source"]["parameters"]);
        }
        for mode in 0..3{let mut packet=ashlar();match mode{0=>packet["source"]["parameters"]["unused"]=json!({"family":"string","value":"x"}),1=>packet["source"]["parameters"]["STATUS"]=json!({"family":"string","value":"x"}),_=>{packet["source"]["parameters"].as_object_mut().unwrap().remove("status");}}packet["target"]["parameters"]=packet["source"]["parameters"].clone();let p=compile_value(&packet);assert!(p.is_err()||p.unwrap()["status"]!="compiled","{mode}");}
    }
    #[test]
    fn original_navigation_identity_context_and_grouped_source_refuse() {
        let original = ashlar();
        for mode in 0..6 {
            let mut packet = original.clone();
            match mode {
                0 => packet["source"]["target"]["bindingSha256"] = json!("bad"),
                1 => packet["relationship"]["module"] = json!("foreign"),
                2 => packet["relationship"]["direction"] = json!("sideways"),
                3 => packet["relationship"]["id"] = json!("missing"),
                4 => {
                    packet["source"]["sql"] =
                        json!("SELECT COUNT(*) AS total FROM sales.\"order\" r")
                }
                _ => packet["target"]["sql"] = packet["source"]["sql"].clone(),
            };
            let p = compile_value(&packet);
            assert!(p.is_err() || p.unwrap()["status"] != "compiled", "{mode}");
        }
    }
}
