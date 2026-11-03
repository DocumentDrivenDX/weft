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

#[cfg(test)]
mod cohort_tests {
    use super::*;
    use umf_core::{native_json::NativeJsonTree, serialization, Limits, ResourceProfile};

    #[test]
    fn cohort_umf_profiles_preserve_or_refuse_exact_number_tokens() {
        let vectors = crate::json::checked_json(include_str!(
            "../../../../tests/engine-catalog/temporal-cohort-vectors.json"
        ))
        .unwrap();
        for case in vectors["vectors"].as_array().unwrap() {
            let raw = case["raw"].as_str().unwrap();
            let source = Utf16::from(raw);
            // The dependency's lossless tree never routes numeric lexemes or
            // reserved-looking object keys through serde_json::Value.
            let tree =
                NativeJsonTree::parse_json(source.units(), &mut Context::ordinary()).unwrap();
            let exact = tree
                .render_compact(&mut Context::ordinary())
                .unwrap()
                .to_utf8()
                .unwrap();
            assert_eq!(exact, raw, "{}", case["name"]);
            assert_eq!(umf_core::operation_values::loads(raw).unwrap(), raw);
            let admitted = parse(raw, &mut Context::ordinary());
            if let Some(code) = case["ordinaryError"].as_str() {
                assert_eq!(
                    admitted.unwrap_err().diagnostics[0]["code"],
                    code,
                    "{}",
                    case["name"]
                );
            } else {
                let admitted = admitted.unwrap();
                let compact =
                    serialization::stringify_compact_metered(&admitted, &mut Context::ordinary())
                        .unwrap()
                        .to_utf8()
                        .unwrap();
                assert_eq!(
                    compact,
                    case["ordinary"].as_str().unwrap(),
                    "{}",
                    case["name"]
                );
                // Exact inspector outbound bridge used by product lowering.
                assert_eq!(
                    json(&admitted).unwrap(),
                    crate::json::checked_json(&compact).unwrap()
                );
            }
        }
    }

    #[test]
    fn cohort_order_canonical_digest_and_original_bytes_are_distinct() {
        let a = " {\"z\":1.00e0,\"a\":9007199254740993} \n";
        let b = "{\"a\":9007199254740993,\"z\":1.00e0}";
        let expected = "{\"a\":9007199254740993,\"z\":1.00e0}";
        assert_eq!(umf_core::operation_values::canonical(a).unwrap(), expected);
        assert_eq!(umf_core::operation_values::canonical(b).unwrap(), expected);
        let hash = crate::json::sha256(expected.as_bytes());
        assert_eq!(umf_core::operation_values::digest(a).unwrap(), hash);
        assert_eq!(umf_core::operation_values::digest(b).unwrap(), hash);
        assert_eq!(
            crate::mutation::canonical(&crate::json::checked_json(a).unwrap()),
            expected
        );
        assert_ne!(crate::json::sha256(a.as_bytes()), hash);
        assert_eq!(
            umf_core::operation_values::loads(a).unwrap().as_bytes(),
            a.as_bytes()
        );
        let ordered = r#"{"z":1,"a":2}"#;
        let value = parse(ordered, &mut Context::ordinary()).unwrap();
        assert_eq!(
            serialization::stringify_compact(&value)
                .unwrap()
                .to_utf8()
                .unwrap(),
            ordered
        );
    }

    #[test]
    fn cohort_duplicate_and_resource_limits_still_refuse() {
        for raw in [r#"{"x":1,"x":2}"#, r#"{"x":1,"\u0078":2}"#] {
            assert!(parse(raw, &mut Context::ordinary()).is_err());
            assert!(
                NativeJsonTree::parse_json(Utf16::from(raw).units(), &mut Context::ordinary())
                    .is_err()
            );
            assert!(umf_core::operation_values::canonical(raw).is_err());
            assert_eq!(crate::json::checked_json(raw), Err("WFT-JSON-DUPLICATE"));
        }
        let mut limits = Limits::ordinary();
        limits.max_values = 2;
        let mut ctx = Context::with_limits(ResourceProfile::Cooperative, limits).unwrap();
        assert_eq!(
            parse("[1,2]", &mut ctx).unwrap_err().diagnostics[0]["code"],
            "LIMIT"
        );
        limits = Limits::ordinary();
        limits.max_depth = 1;
        let mut ctx = Context::with_limits(ResourceProfile::Cooperative, limits).unwrap();
        assert_eq!(
            parse("[[1]]", &mut ctx).unwrap_err().diagnostics[0]["code"],
            "LIMIT"
        );
        limits = Limits::ordinary();
        limits.max_text_units = 4;
        let mut ctx = Context::with_limits(ResourceProfile::Cooperative, limits).unwrap();
        assert_eq!(
            parse("\"hello\"", &mut ctx).unwrap_err().diagnostics[0]["code"],
            "LIMIT"
        );
        let over = format!("{}0{}", "[".repeat(129), "]".repeat(129));
        assert_eq!(crate::json::checked_json(&over), Err("WFT-LIMIT"));
    }
}
