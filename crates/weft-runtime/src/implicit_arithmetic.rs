//! Explicit additive 0.4.2 composition; no host execution or authority.
use weft_core::{
    backend03::{Plan04View, Registry},
    compile::CompositionInput,
    error::Result,
    model::Catalog,
};
pub fn compile_json(request: &str) -> String {
    let mut factory =
        |_: &Catalog, _: Plan04View<'_>, _: CompositionInput<'_>| -> Result<Registry> {
            let mut registry = Registry::default();
            registry.register(weft_databricks::implicit_arithmetic::ImplicitArithmeticLabels)?;
            Ok(registry)
        };
    weft_core::compile::v042::Compiler::default().compile_json_with_factory(request, &mut factory)
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    fn request(sql: &str) -> Value {
        let mut r: Value = serde_json::from_str(include_str!(
            "../../../tests/ashlar-databricks/fixtures/original-commerce-path-request.json"
        ))
        .unwrap();
        r["interfaceVersion"] = json!("weft-compile/0.4.2");
        r["dialect"] = json!("weft-sql/0.4.2");
        r["sql"] = json!(sql);
        r["target"]["backendId"] = json!(weft_databricks::paths_keys::ID);
        r["target"]["backendVersion"] = json!(weft_databricks::implicit_arithmetic::VERSION);
        r["target"]["targetProfile"] = json!(weft_databricks::paths_keys::PROFILE);
        r
    }
    fn compile(sql: &str) -> Value {
        serde_json::from_str(&super::compile_json(&request(sql).to_string())).unwrap()
    }
    #[test]
    fn original_partial_return_requires_additive_capability_and_positioned_outputs() {
        let sql="SELECT f.quantity,r.quantity,l.quantity-f.quantity+r.quantity FROM order_lines l JOIN fulfillments f ON f.line_id=l.id JOIN returns r ON r.line_id=l.id WHERE f.quantity<l.quantity";
        let a = compile(sql);
        assert_eq!(a["status"], "compiled", "{a}");
        let p = &a["logicalPlan"];
        assert_eq!(p["outputs"][2]["name"], "l.quantity-f.quantity+r.quantity");
        for cap in [
            "project.implicitArithmeticLabels",
            "project.positionedOutputs",
        ] {
            assert!(
                p["requiredCapabilities"]
                    .as_array()
                    .unwrap()
                    .contains(&json!(cap)),
                "{p}"
            );
        }
        let columns = a["columns"].as_array().unwrap();
        assert_eq!(columns.len(), 3);
        let labels = ["quantity", "quantity", "l.quantity-f.quantity+r.quantity"];
        let mut physical = std::collections::BTreeSet::new();
        for (index, column) in columns.iter().enumerate() {
            assert_eq!(column["position"], json!(index + 1));
            assert_eq!(column["outputName"], labels[index]);
            let carrier = column["carrierName"].as_str().unwrap();
            assert!(!carrier.is_empty() && physical.insert(carrier));
            assert!(a["sql"]
                .as_str()
                .unwrap()
                .contains(&format!("AS `{carrier}`")));
            assert!(!column["sourceIdentities"].as_array().unwrap().is_empty());
        }
        assert_ne!(
            columns[0]["sourceIdentities"],
            columns[1]["sourceIdentities"]
        );
        let computed_sources = columns[2]["sourceIdentities"].as_array().unwrap();
        for column in &columns[..2] {
            for identity in column["sourceIdentities"].as_array().unwrap() {
                assert!(computed_sources.contains(identity));
            }
        }
        assert_eq!(computed_sources.len(), 3);
        assert_eq!(a["interfaceVersion"], "weft-compile/0.4.2");
    }
    #[test]
    fn repeated_computed_or_explicit_logical_names_refuse() {
        for sql in [
            "SELECT l.quantity+1,l.quantity+1 FROM order_lines l",
            "SELECT l.quantity+1,l.quantity AS \"l.quantity+1\" FROM order_lines l",
        ] {
            let a = compile(sql);
            assert_eq!(a["status"], "blocked", "{a}");
            assert_eq!(a["diagnostics"][0]["code"], "WFT-OUTPUT-NAME", "{a}");
        }
    }
    #[test]
    fn wrong_version_pair_refuses_before_factory() {
        let mut r = request("SELECT l.quantity+1 FROM order_lines l");
        r["dialect"] = json!("weft-sql/0.4.1");
        let a: Value = serde_json::from_str(&super::compile_json(&r.to_string())).unwrap();
        assert_eq!(a["diagnostics"][0]["code"], "WFT-VERSION");
    }
    #[test]
    fn all_four_original_commerce_queries_compile_without_sql_rewriting() {
        for sql in [
            "SELECT f.quantity,r.quantity,l.quantity-f.quantity+r.quantity FROM order_lines l JOIN fulfillments f ON f.line_id=l.id JOIN returns r ON r.line_id=l.id WHERE f.quantity<l.quantity",
            "SELECT f.id FROM fulfillments f JOIN order_lines l ON f.line_id=l.id WHERE f.quantity>l.quantity ORDER BY f.id",
            "SELECT p.id FROM payments p JOIN invoices i ON p.invoice_id=i.id WHERE p.amount=i.amount AND p.currency=i.currency",
            "SELECT r.id FROM refunds r JOIN returns x ON x.id=r.return_id JOIN order_lines l ON l.id=x.line_id JOIN products p ON p.id=l.product_id WHERE r.amount=x.quantity*p.unit_price",
        ] {
            let a=compile(sql); assert_eq!(a["status"],"compiled","{sql}: {a}");
        }
    }

    #[test]
    fn exact042_manifest_admitted_and_mixed_or_malformed_pairs_refuse() {
        use weft_core::backend03::Backend;
        let manifest = weft_databricks::implicit_arithmetic::ImplicitArithmeticLabels
            .describe()
            .unwrap();
        let original = serde_json::to_value(manifest).unwrap();
        weft_core::backend03::validate_manifest_json(&original.to_string()).unwrap();
        for index in 0..3 {
            let mut bad = original.clone();
            match index {
                0 => bad["languageProfiles"][0]["irVersion"] = json!("weft-ir/0.4.1"),
                1 => {
                    bad["capabilities"][0]["languageProfiles"][0]["dialectProfile"] =
                        json!("weft-sql/0.4.1")
                }
                _ => bad["unexpected"] = json!(true),
            }
            assert!(weft_core::backend03::validate_manifest_json(&bad.to_string()).is_err());
            if index < 2 {
                let typed: weft_core::backend03::Manifest = serde_json::from_value(bad).unwrap();
                let mut registry = weft_core::backend03::Registry::default();
                // JSON boundary above plus typed registration through a passive fixed backend.
                struct Fixed(weft_core::backend03::Manifest);
                impl weft_core::backend03::Backend for Fixed {
                    type Mapping = ();
                    type TargetPlan = ();
                    fn describe(&self) -> weft_core::error::Result<weft_core::backend03::Manifest> {
                        Ok(self.0.clone())
                    }
                    fn validate_binding(
                        &self,
                        _: &weft_core::backend03::Context<'_>,
                    ) -> weft_core::error::Result<weft_core::backend03::Validated<()>>
                    {
                        unreachable!()
                    }
                    fn assess(
                        &self,
                        _: &weft_core::backend03::Context<'_>,
                        _: &(),
                    ) -> weft_core::error::Result<Vec<weft_core::backend03::Assessment>>
                    {
                        unreachable!()
                    }
                    fn lower(
                        &self,
                        _: &weft_core::backend03::Context<'_>,
                        _: &(),
                    ) -> weft_core::error::Result<()> {
                        unreachable!()
                    }
                    fn emit(
                        &self,
                        _: &weft_core::backend03::Context<'_>,
                        _: &(),
                    ) -> weft_core::error::Result<weft_core::backend03::Emission>
                    {
                        unreachable!()
                    }
                }
                assert!(registry.register(Fixed(typed)).is_err());
            }
        }
    }
    #[test]
    fn exact042_retains_inherited_count_star_having() {
        let a =
            compile("SELECT p.id, COUNT(*) AS n FROM products p GROUP BY p.id HAVING COUNT(*)>0");
        assert_eq!(a["status"], "compiled", "{a}");
        assert!(a["sql"].as_str().unwrap().contains("HAVING COUNT(*)"));
        assert!(a["logicalPlan"]["requiredCapabilities"]
            .as_array()
            .unwrap()
            .contains(&json!("aggregate.havingCountStarGreater")));
    }
}
