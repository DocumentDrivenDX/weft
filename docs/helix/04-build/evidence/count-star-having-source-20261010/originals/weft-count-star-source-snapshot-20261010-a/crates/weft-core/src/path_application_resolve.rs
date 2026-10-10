//! Private full-query 0.4 resolution through the shared scalar/relational driver.
//! Path aliases are separate typed occurrences, never synthetic Records or Fields.
use crate::{
    application_resolve::Parameters,
    application_scope::Scope,
    arithmetic_application_resolve::{
        self as shared, NamedExpressions, ProjectionSource, ProjectionView, QueryExtension,
        QueryView,
    },
    arithmetic_plan as legacy,
    error::{Diagnostic, Result},
    ir::Span,
    model::Catalog,
    path_ir::{self as ir, Expression, PathExpansion, PathRead},
    path_query as ast, path_resolve,
    syntax::Name,
};

fn fail(code: &str, message: &str, span: &Span) -> Diagnostic {
    Diagnostic::new(code, "resolve", message).at(span)
}

struct ExpansionBinding {
    alias: Name,
    expansion: PathExpansion,
}

struct Paths<'a> {
    expansion: Option<&'a ast::Expansion>,
    has_having: bool,
}

fn path_capabilities(scope: &mut Scope<'_>, path: &PathRead) {
    scope.caps.insert("relationship.twoHopPaths".into());
    if path.hops().iter().any(|h| h.inverse) {
        scope.caps.insert("relationship.inverse".into());
    }
}

impl<'a> QueryExtension for Paths<'a> {
    type Projection = &'a ast::Output;
    type Expression = Expression;
    type State = Option<ExpansionBinding>;

    fn after_joins(&self, scope: &mut Scope<'_>) -> Result<Self::State> {
        let Some(input) = self.expansion else {
            return Ok(None);
        };
        if scope
            .records
            .iter()
            .any(|(alias, _, _)| alias.value == input.alias.value)
        {
            return Err(fail(
                "WFT-NAME-AMBIGUOUS",
                "Path alias repeats a visible source alias",
                &input.alias.span,
            ));
        }
        let path = path_resolve::resolve(scope.catalog, &scope.records, &input.path, &input.span)?;
        path_capabilities(scope, &path);
        scope.caps.insert("relationship.pathExpansion".into());
        // The shared scope owns s0, s1, ...; the one path occurrence has its own ID.
        let expansion = PathExpansion::new("p0".into(), path).map_err(|_| {
            fail(
                "WFT-TYPE",
                "Resolved path expansion is inconsistent",
                &input.span,
            )
        })?;
        Ok(Some(ExpansionBinding {
            alias: input.alias.clone(),
            expansion,
        }))
    }

    fn is_aggregate(&self, output: &Self::Projection) -> bool {
        matches!(output, ast::Output::CountDistinctPathTargets { .. })
    }

    fn projection(
        &self,
        scope: &mut Scope<'_>,
        state: &Self::State,
        output: &Self::Projection,
        aggregate: bool,
        groups: &Vec<legacy::Field>,
    ) -> Result<NamedExpressions<Expression>> {
        let (label, expression) = match output {
            ast::Output::RelatedPaths {
                path,
                bound,
                bound_span,
                span,
            } => {
                if aggregate || !groups.is_empty() || self.has_having || state.is_some() {
                    return Err(fail(
                        "WFT-GROUPING",
                        "RELATED_PATHS requires a nongrouped nonaggregate query without expansion",
                        span,
                    ));
                }
                let resolved = path_resolve::resolve(scope.catalog, &scope.records, path, span)?;
                path_capabilities(scope, &resolved);
                scope.caps.insert("result.pathOccurrences".into());
                let expression = Expression::related_paths(resolved, *bound).map_err(|_| {
                    fail(
                        "WFT-LIMIT",
                        "Path bound must be from 1 through 1000",
                        bound_span,
                    )
                })?;
                // Inherit RELATED_KEYS' relationship-name default and common label rules.
                (path.first.field.value.clone(), expression)
            }
            ast::Output::CountDistinctPathTargets { alias, .. } => {
                let binding = state
                    .as_ref()
                    .filter(|b| b.alias.value == alias.value)
                    .ok_or_else(|| {
                        fail(
                            "WFT-NAME-MISSING",
                            "Path count alias is not the visible expansion",
                            &alias.span,
                        )
                    })?;
                scope
                    .caps
                    .insert("aggregate.pathTargetDistinctCount".into());
                let expression =
                    Expression::count_distinct_path_targets(binding.expansion.occurrence().into())
                        .map_err(|_| {
                            fail(
                                "WFT-TYPE",
                                "Resolved path count is inconsistent",
                                &alias.span,
                            )
                        })?;
                ("count".into(), expression)
            }
            ast::Output::Legacy(_) => {
                unreachable!("legacy projection selected by the shared driver")
            }
        };
        Ok(NamedExpressions {
            default: label.clone(),
            expressions: vec![(label, expression)],
        })
    }

    fn lift_legacy(expression: legacy::Expression) -> Expression {
        Expression::Legacy(expression)
    }

    fn as_legacy(expression: &Expression) -> Option<&legacy::Expression> {
        match expression {
            Expression::Legacy(expression) => Some(expression),
            Expression::Path(_) => None,
        }
    }
}

pub(crate) fn resolve(
    catalog: &Catalog,
    query: ast::Query,
    parameters: Parameters,
) -> Result<ir::Plan> {
    let view = QueryView {
        distinct: query.distinct,
        outputs: query
            .outputs
            .iter()
            .map(|p| ProjectionView {
                output: match &p.output {
                    ast::Output::Legacy(output) => ProjectionSource::Legacy(output),
                    output => ProjectionSource::Extra(output),
                },
                alias: p.alias.as_ref(),
            })
            .collect(),
        source: &query.source,
        joins: &query.joins,
        predicates: &query.predicates,
        groups: &query.groups,
        having: &query.having,
        order: &query.order,
        limit: query.limit,
    };
    let paths = Paths {
        expansion: query.expansion.as_ref(),
        has_having: !query.having.is_empty(),
    };
    let p = shared::resolve_parts(catalog, view, parameters, &paths)?;
    if let Some(expansion) = &query.expansion {
        if p.outputs.iter().any(|o| {
            matches!(
                o.expression,
                Expression::Legacy(legacy::Expression::RelatedKeys { .. })
            )
        }) {
            return Err(fail(
                "WFT-GROUPING",
                "Bounded related collections cannot be mixed with path expansion",
                &expansion.span,
            ));
        }
    }
    ir::Plan::new_version(ir::PlanParts {
        distinct: p.distinct,
        module_pins: p.module_pins,
        required_capabilities: p.required_capabilities,
        type_graph: p.type_graph,
        source: p.source,
        page_key: None,
        joins: p.joins,
        filters: p.filters,
        groups: p.groups,
        having: p
            .having
            .into_iter()
            .map(|h| ir::Having {
                count: h.count,
                threshold: h.threshold,
            })
            .collect(),
        aggregate: p.aggregate,
        outputs: p
            .outputs
            .into_iter()
            .map(|o| ir::Output {
                name: o.name,
                expression: o.expression,
            })
            .collect(),
        order: p.order,
        limit: p.limit,
        path_expansion: p.extension.map(|binding| binding.expansion),
    }, query.count_star_having)
    .map_err(|_| {
        Diagnostic::new(
            "WFT-TYPE",
            "resolve",
            "Resolved query does not satisfy the closed 0.4 plan",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ir::ModelPin, model::ModuleInput};
    use serde_json::{json, Value};

    #[jsonschema::validator(
        path = "../../docs/helix/02-design/contracts/logical-plan-v0.4.schema.json"
    )]
    struct Schema04;

    const COLLECTION: &str = r#"SELECT l.id,
       RELATED_PATHS(l."order_lines.product_id", "products.supplier_id", 20)
         AS supplier_paths
FROM order_lines l ORDER BY l.id;"#;
    const COUNTS: &str = r#"SELECT l.id, COUNT(*) AS path_count,
       COUNT_DISTINCT_PATH_TARGETS(p) AS supplier_count
FROM order_lines l
CROSS JOIN EXPAND_PATHS(l."order_lines.product_id", "products.supplier_id") AS p
GROUP BY l.id ORDER BY l.id;"#;
    const EXPANSION: &str =
        r#"CROSS JOIN EXPAND_PATHS(l."order_lines.product_id", "products.supplier_id") AS p"#;

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

    fn run(sql: &str, parameters: Value) -> Result<Value> {
        let plan = resolve(
            &catalog(),
            ast::parse(sql)?,
            serde_json::from_value(parameters).unwrap(),
        )?;
        let value = serde_json::to_value(plan).unwrap();
        assert!(Schema04::is_valid(&value), "{value}");
        Ok(value)
    }

    fn caps(value: &Value) -> Vec<&str> {
        value["requiredCapabilities"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap())
            .collect()
    }

    fn original<'a>(sql: &'a str, span: &Value) -> &'a str {
        &sql[span["start"].as_u64().unwrap() as usize..span["end"].as_u64().unwrap() as usize]
    }

    #[test]
    fn original_commerce_collection_keeps_pins_relationships_keys_and_spans() {
        let value = run(COLLECTION, json!({})).unwrap();
        assert_eq!(value["irVersion"], "weft-ir/0.4.0");
        assert!(value["readProfile"].is_null());
        assert_eq!(value["aggregate"], false);
        assert!(value.get("pathExpansion").is_none());
        let expression = &value["outputs"][1]["expression"];
        assert_eq!(expression["op"], "relatedPaths");
        assert_eq!(expression["bound"], 20);
        let path = &expression["path"];
        assert_eq!(path["startScan"], "s0");
        assert_eq!(
            path["hops"][0]["identity"]["relationship"],
            "order_lines.product_id"
        );
        assert_eq!(
            path["hops"][1]["identity"]["relationship"],
            "products.supplier_id"
        );
        assert_eq!(path["hops"][0]["to"], path["hops"][1]["from"]);
        assert_eq!(path["hops"][1]["to"]["element"], "suppliers");
        assert_eq!(
            path["hops"][0]["targetKey"]["fields"][0]["element"],
            "products.id"
        );
        assert_eq!(
            path["hops"][1]["targetKey"]["fields"][0]["element"],
            "suppliers.id"
        );
        assert_eq!(path["hops"][1]["targetKey"]["types"][0]["family"], "string");
        assert_eq!(
            original(COLLECTION, &path["hopSpans"][0]),
            r#""order_lines.product_id""#
        );
        assert_eq!(
            original(COLLECTION, &path["hopSpans"][1]),
            r#""products.supplier_id""#
        );
        assert_eq!(
            original(COLLECTION, &path["span"]),
            r#"RELATED_PATHS(l."order_lines.product_id", "products.supplier_id", 20)"#
        );
        assert_eq!(
            value["modulePins"],
            serde_json::to_value(catalog().pins()).unwrap()
        );
        for hop in path["hops"].as_array().unwrap() {
            for id in ["identity", "from", "to"] {
                assert_eq!(hop[id]["documentId"], "urn:umf:domain:commerce");
                assert_eq!(hop[id]["revision"], "c7-original");
            }
        }
        assert!(caps(&value).contains(&"relationship.twoHopPaths"));
        assert!(caps(&value).contains(&"result.pathOccurrences"));
        assert!(!caps(&value).contains(&"relationship.pathExpansion"));
    }

    #[test]
    fn grouped_and_global_counts_use_one_complete_expansion_and_exact_integer() {
        let grouped = run(COUNTS, json!({})).unwrap();
        assert_eq!(grouped["aggregate"], true);
        assert_eq!(grouped["groups"].as_array().unwrap().len(), 1);
        assert_eq!(grouped["outputs"][1]["expression"]["op"], "count");
        assert_eq!(
            grouped["outputs"][2]["expression"],
            json!({"op":"countDistinctPathTargets","pathOccurrence":"p0","type":{"family":"integer","facets":{},"nullable":false}})
        );
        assert_eq!(grouped["pathExpansion"]["occurrence"], "p0");
        for cap in [
            "relationship.twoHopPaths",
            "relationship.pathExpansion",
            "aggregate.pathTargetDistinctCount",
            "aggregate",
            "aggregate.count",
            "group",
        ] {
            assert!(caps(&grouped).contains(&cap), "{cap}");
        }
        assert!(!caps(&grouped).contains(&"result.pathOccurrences"));
        let only = run(
            &format!("SELECT COUNT_DISTINCT_PATH_TARGETS(p) FROM order_lines l {EXPANSION}"),
            json!({}),
        )
        .unwrap();
        assert_eq!(only["aggregate"], true);
        assert_eq!(only["groups"], json!([]));
        assert_eq!(only["outputs"][0]["name"], "count");
        assert!(!caps(&only).contains(&"aggregate.count"));
        let both = run(&format!("SELECT COUNT_DISTINCT_PATH_TARGETS(p) AS a,COUNT_DISTINCT_PATH_TARGETS(p) AS b FROM order_lines l {EXPANSION}"), json!({})).unwrap();
        assert_eq!(
            both["outputs"][0]["expression"],
            both["outputs"][1]["expression"]
        );
        let rows = run(
            &format!("SELECT COUNT(*) FROM order_lines l {EXPANSION}"),
            json!({}),
        )
        .unwrap();
        assert!(caps(&rows).contains(&"aggregate.count"));
        assert!(!caps(&rows).contains(&"aggregate.pathTargetDistinctCount"));
    }

    #[test]
    fn ordinary_join_filter_group_legacy_having_order_and_parameters_share_semantics() {
        let sql = format!("SELECT l.id,COUNT(*) AS rows,COUNT_DISTINCT_PATH_TARGETS(p) AS suppliers,COUNT(DISTINCT product.sku) AS skus FROM order_lines l JOIN products product ON l.product_id=product.id {EXPANSION} WHERE l.quantity>:n GROUP BY l.id HAVING COUNT(DISTINCT product.sku)>0 ORDER BY l.id LIMIT 10");
        let value = run(&sql, json!({"n":{"family":"integer","value":"0"}})).unwrap();
        assert_eq!(value["joins"].as_array().unwrap().len(), 1);
        assert_eq!(value["filters"].as_array().unwrap().len(), 1);
        assert_eq!(value["having"][0]["count"]["op"], "countDistinct");
        assert_eq!(value["having"][0]["count"]["argument"]["scan"], "s1");
        assert_eq!(
            original(&sql, &value["having"][0]["count"]["argument"]["span"]),
            "product.sku"
        );
        assert_eq!(
            original(&sql, &value["having"][0]["threshold"]["span"]),
            "0"
        );
        assert_ne!(
            value["having"][0]["count"]["argument"]["span"],
            value["outputs"][3]["expression"]["argument"]["span"]
        );
        assert_eq!(value["limit"], 10);
        for cap in [
            "innerJoin",
            "filter",
            "group",
            "parameter.named",
            "aggregate.havingCountDistinctGreater",
            "order.asc",
            "limit",
        ] {
            assert!(caps(&value).contains(&cap), "{cap}");
        }
    }

    #[test]
    fn left_join_roots_keep_original_scan_and_outer_join_metadata() {
        let collection = r#"SELECT l.id,RELATED_PATHS(r."order_lines.product_id","products.supplier_id",2) AS paths FROM order_lines l LEFT JOIN order_lines r ON l.id=r.id ORDER BY r.id"#;
        let value = run(collection, json!({})).unwrap();
        assert_eq!(value["outerJoinScans"], json!(["s1"]));
        assert_eq!(value["joins"][0]["kind"], "left");
        assert_eq!(value["outputs"][1]["expression"]["path"]["startScan"], "s1");
        assert_eq!(value["order"][0]["type"]["nullable"], false);
        for cap in ["join.left", "value.outerJoinPresence"] {
            assert!(caps(&value).contains(&cap));
        }
        let expanded = r#"SELECT COUNT_DISTINCT_PATH_TARGETS(p) FROM order_lines l LEFT JOIN order_lines r ON l.id=r.id CROSS JOIN EXPAND_PATHS(r."order_lines.product_id","products.supplier_id") p"#;
        let value = run(expanded, json!({})).unwrap();
        assert_eq!(value["outerJoinScans"], json!(["s1"]));
        assert_eq!(value["pathExpansion"]["path"]["startScan"], "s1");
    }

    #[test]
    fn alias_binding_is_separate_from_record_and_field_scope() {
        let quoted = r#"SELECT COUNT_DISTINCT_PATH_TARGETS("P") AS n FROM order_lines l CROSS JOIN EXPAND_PATHS(l."order_lines.product_id","products.supplier_id") "P""#;
        assert!(run(quoted, json!({})).is_ok());
        for sql in [
            quoted.replace("TARGETS(\"P\")", "TARGETS(p)"),
            format!("SELECT COUNT_DISTINCT_PATH_TARGETS(missing) FROM order_lines l {EXPANSION}"),
            format!("SELECT p.id FROM order_lines l {EXPANSION}"),
            format!("SELECT p.* FROM order_lines l {EXPANSION}"),
            format!("SELECT COUNT(*) FROM order_lines l {EXPANSION} WHERE p.id='x'"),
            format!("SELECT COUNT(*) FROM order_lines l {EXPANSION} GROUP BY p.id"),
            format!("SELECT COUNT(*) FROM order_lines l {EXPANSION} ORDER BY p.id"),
        ] {
            assert_eq!(
                run(&sql, json!({})).unwrap_err().code,
                "WFT-NAME-MISSING",
                "{sql}"
            );
        }
        let collision = EXPANSION.replace("AS p", "AS l");
        assert_eq!(
            run(
                &format!("SELECT COUNT(*) FROM order_lines l {collision}"),
                json!({})
            )
            .unwrap_err()
            .code,
            "WFT-NAME-AMBIGUOUS"
        );
        let missing =
            format!("SELECT COUNT_DISTINCT_PATH_TARGETS(missing) FROM order_lines l {EXPANSION}");
        let diagnostic = run(&missing, json!({})).unwrap_err();
        assert_eq!(
            original(
                &missing,
                &serde_json::to_value(diagnostic.source_span.unwrap()).unwrap()
            ),
            "missing"
        );
    }

    #[test]
    fn aggregate_classification_collection_composition_and_labels_are_closed() {
        for sql in [
            format!("SELECT l.id,COUNT_DISTINCT_PATH_TARGETS(p) FROM order_lines l {EXPANSION}"),
            r#"SELECT RELATED_PATHS(l."order_lines.product_id","products.supplier_id",1),COUNT(*) FROM order_lines l"#.into(),
            r#"SELECT RELATED_PATHS(l."order_lines.product_id","products.supplier_id",1) FROM order_lines l GROUP BY l.id"#.into(),
            format!(r#"SELECT RELATED_PATHS(l."order_lines.product_id","products.supplier_id",1) FROM order_lines l {EXPANSION}"#),
            format!(r#"SELECT RELATED_KEYS(l."order_lines.product_id",1) FROM order_lines l {EXPANSION}"#),
            format!("SELECT COUNT_DISTINCT_PATH_TARGETS(p) FROM order_lines l {EXPANSION} HAVING COUNT(DISTINCT l.id)>0"),
        ] { assert_eq!(run(&sql, json!({})).unwrap_err().code, "WFT-GROUPING", "{sql}"); }
        for sql in [
            format!("SELECT COUNT(*),COUNT_DISTINCT_PATH_TARGETS(p) FROM order_lines l {EXPANSION}"),
            r#"SELECT RELATED_PATHS(l."order_lines.product_id","products.supplier_id",1) AS x,l.id AS x FROM order_lines l"#.into(),
        ] { assert_eq!(run(&sql, json!({})).unwrap_err().code, "WFT-OUTPUT-NAME", "{sql}"); }
        let default = run(r#"SELECT RELATED_PATHS(l."order_lines.product_id","products.supplier_id",1) FROM order_lines l"#, json!({})).unwrap();
        assert_eq!(default["outputs"][0]["name"], "order_lines.product_id");
        assert_eq!(
            run(
                &format!(
                    "SELECT DISTINCT COUNT_DISTINCT_PATH_TARGETS(p) FROM order_lines l {EXPANSION}"
                ),
                json!({})
            )
            .unwrap_err()
            .code,
            "WFT-CAPABILITY"
        );
    }

    #[test]
    fn scalar_parameter_occurrences_and_surplus_rules_remain_shared() {
        let sql = r#"SELECT l.quantity+:n AS quantity,RELATED_PATHS(l."order_lines.product_id","products.supplier_id",1) AS paths FROM order_lines l WHERE l.quantity>:n"#;
        let value = run(sql, json!({"n":{"family":"integer","value":"1"}})).unwrap();
        assert_eq!(value["outputs"][0]["expression"]["op"], "arithmetic");
        assert!(caps(&value).contains(&"parameter.named"));
        assert!(caps(&value).contains(&"arithmetic.+"));
        assert_eq!(
            run(sql, json!({"n":{"family":"string","value":"1"}}))
                .unwrap_err()
                .code,
            "WFT-PARAMETER"
        );
        assert_eq!(
            run(
                COLLECTION,
                json!({"unused":{"family":"integer","value":"1"}})
            )
            .unwrap_err()
            .code,
            "WFT-PARAMETER"
        );
        assert_eq!(run(sql, json!({})).unwrap_err().code, "WFT-PARAMETER");
    }

    #[test]
    fn legacy_only_queries_keep_the_same_resolved_members() {
        let catalog = catalog();
        for (sql, parameters) in [
            ("SELECT l.id,l.quantity+1 AS quantity FROM order_lines l WHERE l.quantity>0 ORDER BY l.id LIMIT 2", json!({})),
            ("SELECT l.id,COUNT(*) AS n FROM order_lines l GROUP BY l.id ORDER BY l.id", json!({})),
            ("SELECT COUNT(DISTINCT l.id) AS n FROM order_lines l HAVING COUNT(DISTINCT l.id)>0", json!({})),
            ("SELECT l.id FROM order_lines l LEFT JOIN order_lines r ON l.id=r.id ORDER BY r.id", json!({})),
            ("SELECT DISTINCT l.id FROM order_lines l ORDER BY l.id", json!({})),
            ("SELECT l.quantity+:n AS n FROM order_lines l WHERE l.quantity=:n", json!({"n":{"family":"integer","value":"1"}})),
        ] {
            let mut old = serde_json::to_value(shared::resolve(&catalog, crate::arithmetic_query::parse(sql).unwrap(), serde_json::from_value(parameters.clone()).unwrap(), None).unwrap()).unwrap();
            let new = serde_json::to_value(resolve(&catalog, ast::parse(sql).unwrap(), serde_json::from_value(parameters).unwrap()).unwrap()).unwrap();
            old["irVersion"] = json!("weft-ir/0.4.0");
            assert_eq!(new, old, "{sql}");
        }
    }

    #[test]
    fn old_version_entrypoints_still_refuse_new_path_queries() {
        for sql in [COLLECTION, COUNTS] {
            assert!(crate::syntax::parse(sql).is_err());
            assert!(crate::application_syntax::parse(sql).is_err());
            assert!(crate::arithmetic_query::parse(sql).is_err());
        }
    }

    #[test]
    fn separately_declared_inverse_and_optional_count_fixture_keeps_own_semantics() {
        let mut doc: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/original-commerce-0.8/ontology.json"
        ))
        .unwrap();
        doc["id"] = json!("path-inverse-optional-fixture");
        for rel in doc["modules"][0]["relationships"].as_array_mut().unwrap() {
            if rel["id"] == "products.supplier_id" {
                rel["inverse"] = json!("supplier-products");
            }
            if rel["id"] == "order_lines.product_id" {
                rel["inverse"] = json!("product-lines");
            }
        }
        for element in doc["modules"][0]["elements"].as_array_mut().unwrap() {
            if element["id"] == "products.sku" {
                element["nullability"] = json!("absent-allowed");
            }
        }
        let raw = doc.to_string();
        let catalog = Catalog::prepare(vec![ModuleInput {
            document_json: raw.clone(),
            pin: ModelPin {
                document_id: "path-inverse-optional-fixture".into(),
                revision: "test-only".into(),
                umf_version: "0.8.0".into(),
                sha256: crate::json::sha256(raw.as_bytes()),
            },
            selected_module_ids: vec!["domain".into()],
        }])
        .unwrap();
        let inverse = r#"SELECT s.id,RELATED_PATHS(s."supplier-products","product-lines",2) AS lines FROM suppliers s"#;
        let value = serde_json::to_value(
            resolve(&catalog, ast::parse(inverse).unwrap(), Default::default()).unwrap(),
        )
        .unwrap();
        assert!(Schema04::is_valid(&value));
        assert!(caps(&value).contains(&"relationship.inverse"));
        let hops = value["outputs"][1]["expression"]["path"]["hops"]
            .as_array()
            .unwrap();
        assert!(hops.iter().all(|h| h["inverse"] == true));
        assert_eq!(hops[1]["to"]["element"], "order_lines");
        assert_eq!(hops[1]["to"]["documentId"], "path-inverse-optional-fixture");
        let optional = format!("SELECT COUNT_DISTINCT_PATH_TARGETS(p) AS targets,COUNT(DISTINCT product.sku) AS skus FROM order_lines l LEFT JOIN products product ON l.product_id=product.id {EXPANSION} HAVING COUNT(DISTINCT product.sku)>0");
        let value = serde_json::to_value(
            resolve(&catalog, ast::parse(&optional).unwrap(), Default::default()).unwrap(),
        )
        .unwrap();
        assert!(Schema04::is_valid(&value));
        assert!(caps(&value).contains(&"aggregate.countDistinct.optional"));
        assert!(caps(&value).contains(&"value.nativeNull"));
        assert_eq!(value["outerJoinScans"], json!(["s1"]));
        assert_eq!(value["having"][0]["count"]["argument"]["scan"], "s1");
        assert_eq!(
            value["having"][0]["count"]["argument"]["type"]["nullable"],
            false
        );
    }
}
