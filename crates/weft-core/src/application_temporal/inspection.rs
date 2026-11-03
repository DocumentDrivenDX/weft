// SPDX-License-Identifier: MIT OR Apache-2.0
//! Calls UMF's source inspector and oracle; no local interpretation of access
//! extension payloads. Trusted schemas are unchanged upstream resources.
use super::*;
use std::sync::OnceLock;
use umf_codegen::{CodegenEngine, CodegenResources};
use umf_core::{Context, ResourceSet, Utf16, Value as U};
fn parse(raw: &str, ctx: &mut Context<'_>) -> Result<U> {
    umf_core::serialization::read_json_value(
        Utf16::from(raw).units(),
        umf_core::serialization::Format::Json,
        ctx,
    )
    .map_err(|e| control(e).into())
}
fn control(e: umf_core::EngineError) -> Failure {
    Failure {
        diagnostics: vec![
            json!({"code":e.code,"path":e.path.to_utf8().unwrap_or_default(),"message":e.message.to_utf8().unwrap_or_else(|_|"UMF control refusal".into()),"phase":"model","severity":"error"}),
        ],
    }
}

fn resources() -> Result<&'static CodegenResources> {
    static RESOURCES: OnceLock<std::result::Result<CodegenResources, Failure>> = OnceLock::new();
    RESOURCES
        .get_or_init(|| {
            let ctx = &mut Context::catalog();
            macro_rules! asset {
                ($name:literal) => {
                    parse(
                        include_str!(concat!("../../resources/umf-temporal/", $name, ".json")),
                        ctx,
                    )?
                };
            }
            let tables = asset!("tables");
            let actions = asset!("actions");
            let get = |v: &U, k: &str| -> Result<U> {
                if let U::Object(items) = v {
                    items
                        .iter()
                        .find(|(key, _)| key == &Utf16::from(k))
                        .map(|(_, v)| v.clone())
                        .ok_or_else(|| fail("Required trusted resource member absent").into())
                } else {
                    Err((fail("Trusted resource object required")).into())
                }
            };
            CodegenResources::new(
                ResourceSet::new(
                    asset!("core"),
                    asset!("reference"),
                    asset!("package"),
                    asset!("constraints"),
                    asset!("domain"),
                ),
                tables.clone(),
                get(&tables, "schema")?,
                actions.clone(),
                get(&actions, "schema")?,
                asset!("binding"),
                asset!("stable-binding"),
                ctx,
            )
            .map_err(|e| control(e).into())
        })
        .as_ref()
        .map_err(Clone::clone)
}
fn json(value: &U) -> Result<Value> {
    let text = umf_core::serialization::write_json_value(
        value,
        umf_core::serialization::Format::Json,
        &mut Context::ordinary(),
    )
    .map_err(control)?;
    let text = text
        .to_utf8()
        .map_err(|_| fail("Inspected result must be Unicode scalar JSON"))?;
    crate::json::checked_json(&text)
        .map_err(|code| Diagnostic::new(code, "model", "Inspected JSON framing refused").into())
}
pub fn inspect(document: &str, record: &Identity) -> Result<Value> {
    let resources = resources()?;
    let mut ctx = Context::ordinary();
    let document = parse(document, &mut ctx)?;
    let selected = parse(
        &json!([{"module":record.module,"element":record.element}]).to_string(),
        &mut ctx,
    )?;
    let out = CodegenEngine::new(resources)
        .inspect_access_views(
            &document,
            &selected,
            &U::Array(vec![]),
            &U::string_utf8("logical"),
            &mut ctx,
        )
        .map_err(control)?;
    let out = json(&out)?;
    if out["diagnostics"].as_array().is_none_or(|d| !d.is_empty()) {
        return Err(Failure{diagnostics:out["diagnostics"].as_array().cloned().unwrap_or_else(||vec![json!({"code":"ACCESS_SHAPE","message":"Ordered inspection diagnostics required"})])});
    }
    Ok(out)
}
