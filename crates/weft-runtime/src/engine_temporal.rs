// SPDX-License-Identifier: MIT OR Apache-2.0
//! Whole-program temporal custody, shared by native Python and portable WASM.
//! No SQL construction, native access, or authority is owned by this module.
use serde::Deserialize;
use serde_json::{json, Value};
use weft_core::application_temporal::KeyRef;
type Result<T> = std::result::Result<T, Value>;
fn category(page: bool) -> &'static str {
    if page {
        "WFT-ENGINE-PAGE"
    } else {
        "WFT-ENGINE-READ"
    }
}
fn blocked(page: bool, message: &str) -> Value {
    json!({"status":"blocked","diagnostics":[{"code":category(page),"phase":"engine","severity":"error","message":message}]})
}
fn parse(raw: &str, bound: usize, page: bool) -> Result<Value> {
    if raw.len() > bound {
        return Err(
            json!({"status":"blocked","diagnostics":[{"code":"WFT-LIMIT","phase":"input","severity":"error","message":"Temporal JSON byte bound exceeded"}]}),
        );
    }
    weft_core::json::checked_json(raw).map_err(|code|json!({"status":"blocked","diagnostics":[{"code":code,"phase":"input","severity":"error","message":"Temporal JSON framing refused"}]})).and_then(|v|if v.is_object()||v.is_null(){Ok(v)}else{Err(blocked(page,"JSON object required"))})
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadPacket {
    version: String,
    compiler: String,
    request: Value,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PagePacket {
    version: String,
    compiler: String,
    request: Value,
    selected_key: KeyRef,
    page_size: u16,
}
fn encode(v: &Value) -> String {
    weft_core::mutation::canonical(v)
}
fn seal(program: &mut Value, page: bool) {
    program["structuralDigest"] = json!({"owner":if page{"weft-runtime-engine-page"}else{"weft-runtime-engine-read"},"version":"0.4","sha256":weft_core::json::sha256(encode(program).as_bytes())});
}
fn compile(request: &Value, key: Option<(&KeyRef, u16)>, after: &Value) -> Result<Value> {
    let page = key.is_some();
    let result=weft_postgresql::truss_runtime::temporal::compile_value(request,key,after).map_err(|failure|{
  let original=json!({"interfaceVersion":"weft-compile/0.2.10","status":"blocked","diagnostics":failure.diagnostics});
  json!({"status":"blocked","diagnostics":original["diagnostics"],"original":original})
 })?;
    let mut program = json!({"version":if page{"weft.engine-page/0.4"}else{"weft.engine-read/0.4"},"status":"compiled","compiler":"truss-runtime-temporal","request":request,"artifact":result["artifact"],"outputs":result["outputs"]});
    if let Some((key, size)) = key {
        program["selectedKey"] = json!(key);
        program["pageSize"] = json!(size);
        program["page"] = result["page"].clone();
        program["after"] = after.clone();
    }
    finalize(program, page)
}
fn finalize(mut program: Value, page: bool) -> Result<Value> {
    seal(&mut program, page);
    // Bound the actual public bytes, including the digest. A successful program
    // must fit the same byte envelope its checker admits.
    let encoded = program.to_string();
    if encoded.len() > 4_194_304 {
        return Err(blocked(
            page,
            "Whole sealed temporal program exceeds four MiB",
        ));
    }
    weft_core::json::checked_json(&encoded).map_err(|code|json!({"status":"blocked","diagnostics":[{"code":code,"phase":"emit","severity":"error","message":"Whole sealed temporal program exceeds checker framing limits"}]}))?;
    Ok(program)
}

pub fn compile_json(raw: &str, page: bool) -> String {
    finish((|| {
        let value = parse(raw, 4_194_304, page)?;
        if page {
            let packet: PagePacket = serde_json::from_value(value)
                .map_err(|_| blocked(true, "Closed temporal page envelope required"))?;
            if packet.version != "weft.engine-page/0.4"
                || packet.compiler != "truss-runtime-temporal"
                || !(1..=999).contains(&packet.page_size)
            {
                return Err(blocked(
                    true,
                    "Exact temporal page selector and bound required",
                ));
            }
            compile(
                &packet.request,
                Some((&packet.selected_key, packet.page_size)),
                &Value::Null,
            )
        } else {
            let packet: ReadPacket = serde_json::from_value(value)
                .map_err(|_| blocked(false, "Closed temporal read envelope required"))?;
            if packet.version != "weft.engine-read/0.4"
                || packet.compiler != "truss-runtime-temporal"
            {
                return Err(blocked(false, "Exact temporal read selector required"));
            }
            compile(&packet.request, None, &Value::Null)
        }
    })())
}
fn original(program: &Value, page: bool) -> Result<Option<(KeyRef, u16)>> {
    if program["version"]
        != if page {
            "weft.engine-page/0.4"
        } else {
            "weft.engine-read/0.4"
        }
        || program["status"] != "compiled"
        || program["compiler"] != "truss-runtime-temporal"
    {
        return Err(blocked(
            page,
            "Original compiled temporal engine0.4 program required",
        ));
    }
    let key = if page {
        if program["after"] != Value::Null {
            return Err(blocked(true, "Original first page after:null required"));
        }
        let key: KeyRef = serde_json::from_value(program["selectedKey"].clone())
            .map_err(|_| blocked(true, "Original selectedKey required"))?;
        let size = program["pageSize"]
            .as_u64()
            .and_then(|v| u16::try_from(v).ok())
            .filter(|n| (1..=999).contains(n))
            .ok_or_else(|| blocked(true, "Original page bound required"))?;
        Some((key, size))
    } else {
        None
    };
    let rebuilt = compile(
        &program["request"],
        key.as_ref().map(|(key, size)| (key, *size)),
        &Value::Null,
    )?;
    if rebuilt != *program {
        return Err(blocked(
            page,
            "Whole original program differs from independent recompilation",
        ));
    }
    Ok(key)
}
fn current_request(original: &Value, current: &Value, page: bool) -> Result<()> {
    let mut values = original.clone();
    for map in ["parameters", "temporalParameters"] {
        let before = original[map]
            .as_object()
            .ok_or_else(|| blocked(page, "Original declared parameter map required"))?;
        let now = current[map]
            .as_object()
            .ok_or_else(|| blocked(page, "Current declared parameter map required"))?;
        if before.len() != now.len() || before.keys().any(|k| !now.contains_key(k)) {
            return Err(blocked(page, "Original parameter inventory changed"));
        }
        for (name, param) in before {
            let mut changed = param.clone();
            if !now[name]["value"].is_string() {
                return Err(blocked(page, "Current declared token required"));
            }
            changed["value"] = now[name]["value"].clone();
            if changed != now[name] {
                return Err(blocked(page, "Current parameter kind/family/shape changed"));
            }
        }
        values[map] = current[map].clone();
    }
    if values != *current {
        return Err(blocked(
            page,
            "Current request changed original context or source",
        ));
    }
    Ok(())
}
pub fn check_json(
    program: &str,
    request: &str,
    after: Option<&str>,
    page: bool,
    shape: bool,
) -> String {
    finish((|| {
        let program = parse(program, 4_194_304, page)?;
        let request = parse(request, 4_194_304, page)?;
        // Bound all three inputs before any recompile. Framing always uses the same
        // strict Rust reader, including duplicate-member and invalid Unicode refusal.
        let after = match after {
            Some(raw) => parse(raw, 1_048_576, page)?,
            None => Value::Null,
        };
        let key = original(&program, page)?;
        if shape {
            if program["request"] != request {
                return Err(blocked(page, "Supplied original request differs in full"));
            }
            return Ok(json!({"status":"checked","program":program}));
        }
        current_request(&program["request"], &request, page)?;
        let fresh = compile(
            &request,
            key.as_ref().map(|(key, size)| (key, *size)),
            &after,
        )?;
        Ok(json!({"status":"checked","program":fresh}))
    })())
}
fn finish(result: Result<Value>) -> String {
    result.unwrap_or_else(|v| v).to_string()
}

#[cfg(test)]
mod size_tests {
    use super::*;
    #[test]
    fn sealed_program_byte_bound_includes_digest() {
        for page in [false, true] {
            let mut p = json!({"payload":""});
            let base = p.to_string().len();
            p["payload"] = json!("a".repeat(4_194_304 - base));
            assert_eq!(p.to_string().len(), 4_194_304);
            assert!(finalize(p,page).is_err(),"An unsigned program exactly at the input cap must refuse once its digest would exceed that cap");
        }
        for page in [false, true] {
            let mut p = json!({"payload":""});
            seal(&mut p, page);
            let size = p.to_string().len();
            p.as_object_mut().unwrap().remove("structuralDigest");
            p["payload"] = json!("a".repeat(4_194_304 - size));
            let admitted = finalize(p.clone(), page).unwrap();
            assert_eq!(admitted.to_string().len(), 4_194_304);
            p["payload"].as_str().unwrap();
            p["payload"] = json!("a".repeat(4_194_305 - size));
            assert!(finalize(p, page).is_err());
        }
    }
}
#[cfg(test)]
mod red_controls {
    use super::*;
    #[test]
    fn temporal_red_controls_node_budget_matches_own_checker() {
        let p = json!({"payload":vec![Value::Null;100_000]});
        assert!(p.to_string().len() < 4_194_304);
        assert!(weft_core::json::checked_json(&p.to_string()).is_err());
        assert!(finalize(p,false).is_err(),"Compiler finalization must apply the stricter framing budget that its checker requires");
    }
}
