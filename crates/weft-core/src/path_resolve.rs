//! Path-only resolution through the original Catalog. No query or backend admission.
use crate::{
    error::{Diagnostic, Result},
    ir::Span,
    model::{Catalog, Record},
    path_ir::PathRead,
    path_syntax::PathArguments,
    syntax::Name,
};
/// Visible aliases/Records/opaque occurrences belong to the owning query scope.
/// The second hop is resolved from the first exact terminal, never a visible name.
pub(crate) fn resolve(
    catalog: &Catalog,
    visible: &[(Name, Record, String)],
    arguments: &PathArguments,
    full_span: &Span,
) -> Result<PathRead> {
    if arguments.first.unqualified {
        return Err(Diagnostic::new(
            "WFT-NAME-MISSING",
            "resolve",
            "First path relationship requires a visible scan alias",
        )
        .at(&arguments.first.span));
    }
    let matches: Vec<_> = visible
        .iter()
        .filter(|(alias, _, _)| alias.value == arguments.first.alias.value)
        .collect();
    if matches.len() != 1 {
        return Err(Diagnostic::new(
            if matches.is_empty() {
                "WFT-NAME-MISSING"
            } else {
                "WFT-NAME-AMBIGUOUS"
            },
            "resolve",
            "Path root alias must identify exactly one visible scan",
        )
        .at(&arguments.first.alias.span));
    }
    let (_, record, occurrence) = matches[0];
    let root = catalog
        .record_by_identity(&record.identity)
        .map_err(|e| e.at(&arguments.first.alias.span))?;
    if root.pin.document_id != record.pin.document_id
        || root.pin.revision != record.pin.revision
        || root.pin.sha256 != record.pin.sha256
        || root.pin.umf_version != record.pin.umf_version
    {
        return Err(Diagnostic::new(
            "WFT-PIN",
            "resolve",
            "Visible path root does not retain the original Catalog pin",
        )
        .at(&arguments.first.alias.span));
    }
    let first = catalog
        .relationship_read(&root, &arguments.first.field)
        .map_err(|e| e.at(&arguments.first.field.span))?;
    let intermediate = catalog
        .record_by_identity(&first.to)
        .map_err(|e| e.at(&arguments.first.field.span))?;
    let second = catalog
        .relationship_read(&intermediate, &arguments.second)
        .map_err(|e| e.at(&arguments.second.span))?;
    PathRead::new(
        occurrence.clone(),
        [first, second],
        full_span.clone(),
        [
            arguments.first.field.span.clone(),
            arguments.second.span.clone(),
        ],
    )
    .map_err(|_| {
        Diagnostic::new(
            "WFT-TYPE",
            "resolve",
            "Resolved original path metadata is inconsistent",
        )
        .at(full_span)
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ir::{Identity, ModelPin},
        model::ModuleInput,
        path_syntax::{self, Kind},
        syntax::Parser,
    };
    fn doc() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../../../tests/fixtures/original-commerce-0.8/ontology.json"
        ))
        .unwrap()
    }
    fn input(mut doc: serde_json::Value, id: &str, revision: &str) -> ModuleInput {
        doc["id"] = serde_json::json!(id);
        let document_json = doc.to_string();
        ModuleInput {
            pin: ModelPin {
                document_id: id.into(),
                revision: revision.into(),
                umf_version: "0.8.0".into(),
                sha256: crate::json::sha256(document_json.as_bytes()),
            },
            document_json,
            selected_module_ids: vec!["domain".into()],
        }
    }
    fn catalog() -> Catalog {
        let raw = include_str!("../../../tests/fixtures/original-commerce-0.8/ontology.json");
        Catalog::prepare(vec![ModuleInput {
            document_json: raw.into(),
            pin: ModelPin {
                document_id: "urn:umf:domain:commerce".into(),
                revision: "c7-original".into(),
                umf_version: "0.8.0".into(),
                sha256: crate::json::sha256(raw.as_bytes()),
            },
            selected_module_ids: vec!["domain".into()],
        }])
        .unwrap()
    }
    fn record(c: &Catalog, id: &str, revision: &str, element: &str) -> Record {
        c.record_by_identity(&Identity {
            document_id: id.into(),
            revision: revision.into(),
            module: "domain".into(),
            element: element.into(),
        })
        .unwrap()
    }
    fn name(value: &str) -> Name {
        Name {
            value: value.into(),
            quoted: false,
            span: Span { start: 0, end: 1 },
        }
    }
    fn fragment(sql: &str) -> (PathArguments, Span) {
        let mut p = Parser::new(sql).unwrap();
        let e = path_syntax::parse(&mut p).unwrap();
        p.finish().unwrap();
        let Kind::RelatedPaths { path, .. } = e.kind else {
            panic!("collection")
        };
        (path, e.span)
    }
    #[test]
    fn unchanged_original_commerce_and_name_spans() {
        let c = catalog();
        let sql = r#"RELATED_PATHS(l."order_lines.product_id", "products.supplier_id", 2)"#;
        let (args, span) = fragment(sql);
        let p = resolve(
            &c,
            &[(
                name("l"),
                record(&c, "urn:umf:domain:commerce", "c7-original", "order_lines"),
                "s0".into(),
            )],
            &args,
            &span,
        )
        .unwrap();
        assert_eq!(p.start_scan(), "s0");
        assert_eq!(p.hops()[0].identity.relationship, "order_lines.product_id");
        assert_eq!(p.hops()[1].identity.relationship, "products.supplier_id");
        assert_eq!(p.hops()[0].to, p.hops()[1].from);
        let v = serde_json::to_value(p).unwrap();
        for (index, expected) in [
            (0, "\"order_lines.product_id\""),
            (1, "\"products.supplier_id\""),
        ] {
            let s = &v["hopSpans"][index];
            assert_eq!(
                &sql[s["start"].as_u64().unwrap() as usize..s["end"].as_u64().unwrap() as usize],
                expected
            );
        }
    }
    #[test]
    fn missing_alias_relationship_and_stale_root_refuse() {
        let c = catalog();
        let visible = vec![(
            name("l"),
            record(&c, "urn:umf:domain:commerce", "c7-original", "order_lines"),
            "s0".into(),
        )];
        for sql in [
            r#"RELATED_PATHS(x."order_lines.product_id", "products.supplier_id", 2)"#,
            r#"RELATED_PATHS(l."missing", "products.supplier_id", 2)"#,
            r#"RELATED_PATHS(l."order_lines.product_id", "missing", 2)"#,
        ] {
            let (a, s) = fragment(sql);
            assert!(resolve(&c, &visible, &a, &s).is_err())
        }
        let (a, s) =
            fragment(r#"RELATED_PATHS(l."order_lines.product_id", "products.supplier_id", 2)"#);
        let mut bad = visible.clone();
        bad[0].1.pin.sha256 = "b".repeat(64);
        assert_eq!(resolve(&c, &bad, &a, &s).unwrap_err().code, "WFT-PIN");
    }
    #[test]
    fn equal_local_names_remain_scoped_to_original_document_revision() {
        let c =
            Catalog::prepare(vec![input(doc(), "one", "r1"), input(doc(), "two", "r1")]).unwrap();
        let (a, s) =
            fragment(r#"RELATED_PATHS(l."order_lines.product_id", "products.supplier_id", 2)"#);
        for (d, r) in [("one", "r1"), ("two", "r1")] {
            let p = resolve(
                &c,
                &[(name("l"), record(&c, d, r, "order_lines"), "s0".into())],
                &a,
                &s,
            )
            .unwrap();
            for hop in p.hops() {
                assert_eq!(hop.identity.document_id, d);
                assert_eq!(hop.identity.revision, r);
                assert_eq!(hop.to.document_id, d);
                assert_eq!(hop.to.revision, r)
            }
        }
        let next = Catalog::prepare(vec![input(doc(), "one", "r2")]).unwrap();
        let next_root = record(&next, "one", "r2", "order_lines");
        let p = resolve(&next, &[(name("l"), next_root, "s0".into())], &a, &s).unwrap();
        assert!(p.hops().iter().all(|hop| hop.identity.revision == "r2"));
        let stale = record(&c, "one", "r1", "order_lines");
        assert!(resolve(&next, &[(name("l"), stale, "s0".into())], &a, &s).is_err());
    }
    #[test]
    fn separately_declared_inverse_fixture_preserves_orientation() {
        let mut d = doc();
        for rel in d["modules"][0]["relationships"].as_array_mut().unwrap() {
            if rel["id"] == "order_lines.product_id" {
                rel["inverse"] = serde_json::json!("product-lines")
            }
            if rel["id"] == "products.supplier_id" {
                rel["inverse"] = serde_json::json!("supplier-products")
            }
        }
        let c = Catalog::prepare(vec![input(d, "inverse-test-fixture", "test")]).unwrap();
        let (a, s) = fragment(r#"RELATED_PATHS(s."supplier-products", "product-lines", 2)"#);
        let p = resolve(
            &c,
            &[(
                name("s"),
                record(&c, "inverse-test-fixture", "test", "suppliers"),
                "s0".into(),
            )],
            &a,
            &s,
        )
        .unwrap();
        assert!(p.hops().iter().all(|h| h.inverse));
        assert_eq!(p.terminal_record().element, "order_lines");
        assert_eq!(p.hops()[0].identity.relationship, "products.supplier_id");
    }
    #[test]
    fn alias_case_quoting_and_ambiguity_follow_existing_scope() {
        let c = catalog();
        let r = record(&c, "urn:umf:domain:commerce", "c7-original", "order_lines");
        let visible = vec![(name("l"), r.clone(), "s0".into())];
        let (a, s) =
            fragment(r#"RELATED_PATHS(L."order_lines.product_id", "products.supplier_id", 2)"#);
        assert!(resolve(&c, &visible, &a, &s).is_ok());
        let (a, s) =
            fragment(r#"RELATED_PATHS("L"."order_lines.product_id", "products.supplier_id", 2)"#);
        assert!(resolve(&c, &visible, &a, &s).is_err());
        let upper = vec![(
            Name {
                value: "L".into(),
                quoted: true,
                span: Span { start: 0, end: 3 },
            },
            r.clone(),
            "s1".into(),
        )];
        assert_eq!(resolve(&c, &upper, &a, &s).unwrap().start_scan(), "s1");
        let (a, s) =
            fragment(r#"RELATED_PATHS(l."order_lines.product_id", "products.supplier_id", 2)"#);
        let mut duplicates = visible.clone();
        duplicates.push((name("l"), r, "s2".into()));
        assert_eq!(
            resolve(&c, &duplicates, &a, &s).unwrap_err().code,
            "WFT-NAME-AMBIGUOUS"
        );
    }
}
