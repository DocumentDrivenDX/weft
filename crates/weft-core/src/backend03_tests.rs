use super::*;
use serde_json::json;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
pub(crate) fn catalog() -> Catalog {
    let raw = include_str!("../../../tests/fixtures/original-commerce-0.8/ontology.json");
    Catalog::prepare(vec![crate::model::ModuleInput {
        document_json: raw.into(),
        pin: crate::ir::ModelPin {
            document_id: "urn:umf:domain:commerce".into(),
            revision: "c7-original".into(),
            umf_version: "0.8.0".into(),
            sha256: crate::json::sha256(raw.as_bytes()),
        },
        selected_module_ids: vec!["domain".into()],
    }])
    .unwrap()
}
pub(crate) fn plan(c: &Catalog) -> crate::path_ir::Plan {
    crate::path_application_resolve::resolve(c,crate::path_query::parse("SELECT RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 20) AS paths FROM order_lines l").unwrap(),Default::default()).unwrap()
}
pub(crate) fn manifest(p: Plan04View<'_>) -> Manifest {
    let pair = json!({"dialectProfile":"weft-sql/0.4.0","irVersion":"weft-ir/0.4.0"});
    serde_json::from_value(json!({"backendId":"fixture-only-03","backendVersion":"0.1.0","interfaceVersion":"weft-backend/0.3.0","languageProfiles":[pair],"bindingProfile":"fixture-03","targetProfiles":[{"id":"fixture-only","engine":"fixture","engineVersion":"1","sessionSettings":{},"storageLayoutRevision":"fixture","publicationRevision":"fixture"}],"capabilities":p.capabilities().iter().map(|id|json!({"id":id,"targetProfiles":["fixture-only"],"languageProfiles":[pair],"logicalDomain":{"fixture":"only"},"resultDomain":{"fixture":"only"},"constraints":[],"obligations":[],"status":"candidate","evidence":[]})).collect::<Vec<_>>(),"evidence":[]})).unwrap()
}
struct Stop {
    manifest: Manifest,
    calls: Arc<AtomicUsize>,
}
impl Backend for Stop {
    type Mapping = ();
    type TargetPlan = ();
    fn describe(&self) -> Result<Manifest> {
        Ok(self.manifest.clone())
    }
    fn validate_binding(&self, _: &Context<'_>) -> Result<Validated<()>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(fail("WFT-BINDING", "fixture stops here"))
    }
    fn assess(&self, _: &Context<'_>, _: &()) -> Result<Vec<Assessment>> {
        panic!("must not assess")
    }
    fn lower(&self, _: &Context<'_>, _: &()) -> Result<()> {
        panic!("must not lower")
    }
    fn emit(&self, _: &Context<'_>, _: &()) -> Result<Emission> {
        panic!("must not emit")
    }
}
fn target() -> Target {
    Target {
        backend_id: "fixture-only-03".into(),
        backend_version: "0.1.0".into(),
        profile_id: "fixture-only".into(),
        allow_candidate: true,
    }
}
fn binding() -> BindingInput {
    BindingInput {
        profile: "fixture-03".into(),
        json: "{}".into(),
        sha256: crate::json::sha256(b"{}"),
    }
}
#[test]
fn closed_manifest_and_old_interface_refuse() {
    let c = catalog();
    let p = plan(&c);
    let m = manifest(Plan04View::new(&p));
    let value = serde_json::to_value(&m).unwrap();
    validate_manifest_json(&value.to_string()).unwrap();
    for mutate in 0..4 {
        let mut v = value.clone();
        match mutate {
            0 => v["interfaceVersion"] = json!("weft-backend/0.2.0"),
            1 => v["languageProfiles"][0]["irVersion"] = json!("weft-ir/0.3.0"),
            2 => {
                v["capabilities"][0]["languageProfiles"][0]["dialectProfile"] =
                    json!("weft-sql/0.3.0")
            }
            _ => v["forged"] = json!(true),
        }
        assert!(validate_manifest_json(&v.to_string()).is_err());
    }
    assert!(crate::backend::validate_manifest_json(&value.to_string()).is_err());
}
#[test]
fn all_declared_path_capabilities_refuse_before_binding() {
    let c = catalog();
    let p = plan(&c);
    let view = Plan04View::new(&p);
    for index in 0..view.capabilities().len() {
        for mode in 0..3 {
            let mut m = manifest(view);
            match mode {
                0 => {
                    m.capabilities.remove(index);
                }
                1 => m.capabilities[index].status = Status::Unsupported,
                _ => {
                    let mut other = m.target_profiles[0].clone();
                    other.id = "unselected".into();
                    m.target_profiles.push(other);
                    m.capabilities[index].target_profiles = vec!["unselected".into()];
                }
            }
            let calls = Arc::new(AtomicUsize::new(0));
            let mut r = Registry::default();
            r.register(Stop {
                manifest: m,
                calls: calls.clone(),
            })
            .unwrap();
            assert_eq!(
                r.compile(&c, view, &target(), &binding()).unwrap_err().code,
                "WFT-CAPABILITY"
            );
            assert_eq!(calls.load(Ordering::SeqCst), 0);
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let mut r = Registry::default();
    r.register(Stop {
        manifest: manifest(view),
        calls: calls.clone(),
    })
    .unwrap();
    let mut t = target();
    t.allow_candidate = false;
    assert!(r.compile(&c, view, &t, &binding()).is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        r.compile(&c, view, &target(), &binding()).unwrap_err().code,
        "WFT-BINDING"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
#[test]
fn view_retains_full_original_identity_and_binding_digest_gate() {
    let c = catalog();
    let p = plan(&c);
    let view = Plan04View::new(&p);
    let ExpressionView::RelatedPaths { path, .. } = view.outputs().next().unwrap().expression
    else {
        panic!()
    };
    assert_eq!(path.hops()[0].to, path.hops()[1].from);
    assert_eq!(
        path.hops()[0].identity.document_id,
        "urn:umf:domain:commerce"
    );
    let selected = emission::selection(view).unwrap();
    assert_eq!(selected.relationships.len(), 2);
    assert!(selected
        .fields
        .contains(&path.hops()[1].target_key.fields[0]));
    let calls = Arc::new(AtomicUsize::new(0));
    let mut r = Registry::default();
    r.register(Stop {
        manifest: manifest(view),
        calls: calls.clone(),
    })
    .unwrap();
    let mut b = binding();
    b.sha256 = "0".repeat(64);
    assert_eq!(
        r.compile(&c, view, &target(), &b).unwrap_err().code,
        "WFT-PIN"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
#[test]
fn obligation_conflicts_and_bounded_metadata_refuse() {
    let o = Obligation {
        id: "fixture".into(),
        owner: crate::backend::ObligationOwner::Host,
        failure_code: "WFT-OBLIGATION".into(),
        parameters: json!({"a":1}),
    };
    let mut values = vec![];
    registry::merge(&mut values, &[o.clone(), o.clone()]).unwrap();
    assert_eq!(values.len(), 1);
    let mut conflict = o;
    conflict.parameters = json!({"a":2});
    assert!(registry::merge(&mut values, &[conflict]).is_err());
    assert!(bounded_json(&json!({"oversized":"x".repeat(1024)}), 100).is_err());
}
pub(crate) fn path_emission(view: Plan04View<'_>) -> (Emission, Vec<NativeEdgeSource>) {
    let ExpressionView::RelatedPaths { path, bound } = view.outputs().next().unwrap().expression
    else {
        panic!()
    };
    let edges: Vec<_> = path
        .hops()
        .iter()
        .enumerate()
        .map(|(i, h)| NativeEdgeSource {
            relationship: h.identity.clone(),
            table: PinnedTable {
                name: ["fixture".into(), "original".into(), format!("edge{i}")],
                uuid: format!("fixture-uuid{i}"),
                version: 0,
            },
            identity_column: "id".into(),
            native_type: "BIGINT".into(),
        })
        .collect();
    let schemas:Vec<_>=edges.iter().enumerate().map(|(i,e)|json!({"pathIndex":0,"hop":i,"relationship":e.relationship,"table":e.table,"identityColumn":"id","nativeType":"BIGINT"})).collect();
    let checks: Vec<_> = ["edgeEncoding", "intermediateIdentity", "collectionEncoding"]
        .iter()
        .map(|k| json!({"pathIndex":0,"kind":k,"sql":"SELECT '0'","failureCode":"WFT-BINDING"}))
        .collect();
    let o = Obligation {
        id: "ashlar.path.occurrenceIntegrity".into(),
        owner: crate::backend::ObligationOwner::Host,
        failure_code: "WFT-BINDING".into(),
        parameters: json!({"phase":"before-user-query","samePublicationRequired":true,"noPartialPublication":true,"paths":[{"path":path.serialized(),"edgeEncoding":"signed64-decimal/0.1","bound":bound}],"edgeSchemas":schemas,"checks":checks,"success":"one exact STRING count equal to 0 per check"}),
    };
    (
        Emission {
            sql: "SELECT fixture_only".into(),
            parameters: vec![],
            columns: vec![Column {
                position: 1,
                output_name: "paths".into(),
                carrier_name: None,
                representation: Representation::RelatedPaths {
                    path: path.into(),
                    start_record: path.start_record().clone(),
                    bound,
                    edge_encoding: "signed64-decimal/0.1".into(),
                    outer_join: None,
                },
                source_identities: vec![path.start_record().clone()],
                nullable: false,
            }],
            obligations: vec![o],
        },
        edges,
    )
}
#[test]
fn full_path_emission_and_exact_hop_schema_inventory() {
    let c = catalog();
    let p = plan(&c);
    let view = Plan04View::new(&p);
    let m = manifest(view);
    let selected = emission::selection(view).unwrap();
    let q: Vec<_> = m
        .capabilities
        .iter()
        .map(|d| crate::backend::Qualification {
            assessment: Assessment {
                id: d.id.clone(),
                status: Status::Candidate,
                evidence: vec![],
                obligations: vec![],
            },
            declaration: d.clone(),
        })
        .collect();
    let (good, edges) = path_emission(view);
    emission::validate(
        view,
        &good,
        &selected,
        &q,
        &m.target_profiles[0],
        &m,
        &binding(),
        &edges,
        &[],
    )
    .unwrap();
    for mutation in 0..8 {
        let mut bad = good.clone();
        match mutation {
            0 => bad.columns[0].nullable = true,
            1 => {
                let Representation::RelatedPaths { bound, .. } = &mut bad.columns[0].representation
                else {
                    panic!()
                };
                *bound = 19;
            }
            2 => {
                let Representation::RelatedPaths { path, .. } = &mut bad.columns[0].representation
                else {
                    panic!()
                };
                path.hops[1].identity.revision = "forged".into();
            }
            3 => {
                bad.obligations[0].parameters["edgeSchemas"][1]["table"]["uuid"] =
                    json!("different")
            }
            4 => bad.obligations[0].parameters["checks"]
                .as_array_mut()
                .unwrap()
                .remove(2)
                .to_string()
                .clear(),
            5 => bad.obligations[0].parameters["edgeSchemas"][1]["hop"] = json!(0),
            6 => bad.obligations[0].parameters["paths"][0]["bound"] = json!(10),
            _ => bad.obligations[0].parameters["checks"][1]["kind"] = json!("edgeEncoding"),
        }
        assert!(
            emission::validate(
                view,
                &bad,
                &selected,
                &q,
                &m.target_profiles[0],
                &m,
                &binding(),
                &edges,
                &[]
            )
            .is_err(),
            "mutation {mutation}"
        );
    }
}
#[test]
fn ordinary_type_and_exact_parameter_domains_remain_closed() {
    let c = catalog();
    let p = plan(&c);
    let view = Plan04View::new(&p);
    let m = manifest(view);
    let selected = emission::selection(view).unwrap();
    let q: Vec<_> = m
        .capabilities
        .iter()
        .map(|d| crate::backend::Qualification {
            assessment: Assessment {
                id: d.id.clone(),
                status: Status::Candidate,
                evidence: vec![],
                obligations: vec![],
            },
            declaration: d.clone(),
        })
        .collect();
    let (mut e, edges) = path_emission(view);
    e.parameters.push(ParameterSlot {
        position: 1,
        value: "false".into(),
        logical_type: LogicalType {
            family: crate::ir::Family::Boolean,
            facets: json!({}),
            nullable: false,
        },
        origin: json!({"kind":"literal"}),
    });
    emission::validate(
        view,
        &e,
        &selected,
        &q,
        &m.target_profiles[0],
        &m,
        &binding(),
        &edges,
        &[],
    )
    .unwrap();
    for token in ["0", "False", "", "true\0"] {
        e.parameters[0].value = token.into();
        assert!(emission::validate(
            view,
            &e,
            &selected,
            &q,
            &m.target_profiles[0],
            &m,
            &binding(),
            &edges,
            &[]
        )
        .is_err());
    }
    e.parameters[0].value = "9007199254740993".into();
    e.parameters[0].logical_type = LogicalType {
        family: crate::ir::Family::Integer,
        facets: json!({}),
        nullable: false,
    };
    assert!(emission::validate(
        view,
        &e,
        &selected,
        &q,
        &m.target_profiles[0],
        &m,
        &binding(),
        &edges,
        &[]
    )
    .is_err());
    e.parameters[0].logical_type.facets = json!({"integerWidth":{"bits":64,"signed":true}});
    emission::validate(
        view,
        &e,
        &selected,
        &q,
        &m.target_profiles[0],
        &m,
        &binding(),
        &edges,
        &[],
    )
    .unwrap();
    e.parameters[0].value = "9223372036854775808".into();
    assert!(emission::validate(
        view,
        &e,
        &selected,
        &q,
        &m.target_profiles[0],
        &m,
        &binding(),
        &edges,
        &[]
    )
    .is_err());
}

/// Metadata-only fixture backend; never a native capability qualification.
pub(crate) struct FixtureBackend {
    pub manifest: Manifest,
    pub emission: Emission,
    pub edges: Vec<NativeEdgeSource>,
    pub calls: Arc<AtomicUsize>,
}
impl Backend for FixtureBackend {
    type Mapping = ();
    type TargetPlan = ();
    fn describe(&self) -> Result<Manifest> {
        Ok(self.manifest.clone())
    }
    fn validate_binding(&self, c: &Context<'_>) -> Result<Validated<()>> {
        Ok(Validated {
            mapping: (),
            additional_capabilities: vec![],
            coverage: c.selection.clone(),
            obligations: vec![],
            edge_sources: self.edges.clone(),
            record_sources: vec![],
        })
    }
    fn assess(&self, c: &Context<'_>, _: &()) -> Result<Vec<Assessment>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(c.plan
            .capabilities()
            .iter()
            .map(|id| Assessment {
                id: id.clone(),
                status: Status::Candidate,
                evidence: vec![],
                obligations: vec![],
            })
            .collect())
    }
    fn lower(&self, _: &Context<'_>, _: &()) -> Result<()> {
        Ok(())
    }
    fn emit(&self, _: &Context<'_>, _: &()) -> Result<Emission> {
        Ok(self.emission.clone())
    }
}
#[test]
fn registry_success_and_edge_source_refusals_before_assessment() {
    let c = catalog();
    let p = plan(&c);
    let view = Plan04View::new(&p);
    let (e, edges) = path_emission(view);
    for mode in 0..5 {
        let mut declarations = edges.clone();
        match mode {
            1 => declarations[0].identity_column = "forged".into(),
            2 => declarations[0].native_type = "STRING".into(),
            3 => declarations[1] = declarations[0].clone(),
            4 => declarations[0].relationship.revision = "forged".into(),
            _ => {}
        }
        let calls = Arc::new(AtomicUsize::new(0));
        let mut r = Registry::default();
        r.register(FixtureBackend {
            manifest: manifest(view),
            emission: e.clone(),
            edges: declarations,
            calls: calls.clone(),
        })
        .unwrap();
        let result = r.compile(&c, view, &target(), &binding());
        if mode == 0 {
            result.unwrap();
            assert_eq!(calls.load(Ordering::SeqCst), 1);
        } else {
            assert_eq!(result.unwrap_err().code, "WFT-BINDING");
            assert_eq!(calls.load(Ordering::SeqCst), 0);
        }
    }
}
#[test]
fn expansion_counts_require_complete_target_identity_and_both_capacities() {
    let c = catalog();
    let p=crate::path_application_resolve::resolve(&c,crate::path_query::parse("SELECT COUNT(*) AS rows, COUNT_DISTINCT_PATH_TARGETS(p) AS targets FROM order_lines l CROSS JOIN EXPAND_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\") AS p").unwrap(),Default::default()).unwrap();
    let v = Plan04View::new(&p);
    let path = v.expansion().unwrap().path;
    let m = manifest(v);
    let selected = emission::selection(v).unwrap();
    let q: Vec<_> = m
        .capabilities
        .iter()
        .map(|d| crate::backend::Qualification {
            assessment: Assessment {
                id: d.id.clone(),
                status: Status::Candidate,
                evidence: vec![],
                obligations: vec![],
            },
            declaration: d.clone(),
        })
        .collect();
    let base = plan(&c);
    let (mut e, edges) = path_emission(Plan04View::new(&base));
    e.columns = vec![
        Column {
            position: 1,
            output_name: "rows".into(),
            carrier_name: None,
            representation: Representation::Scalar {
                logical_type: LogicalType {
                    family: crate::ir::Family::Integer,
                    facets: json!({}),
                    nullable: false,
                },
                carrier: crate::backend::ScalarCarrier::Text,
                decoder: crate::backend::ScalarDecoder::ExactInteger,
                path_target: None,
            },
            source_identities: vec![path.start_record().clone()],
            nullable: false,
        },
        Column {
            position: 2,
            output_name: "targets".into(),
            carrier_name: None,
            representation: Representation::Scalar {
                logical_type: LogicalType {
                    family: crate::ir::Family::Integer,
                    facets: json!({}),
                    nullable: false,
                },
                carrier: crate::backend::ScalarCarrier::Text,
                decoder: crate::backend::ScalarDecoder::ExactInteger,
                path_target: Some(PathTarget {
                    path_occurrence: v.expansion().unwrap().occurrence.into(),
                    record: path.hops()[1].to.clone(),
                    key: path.hops()[1].target_key.clone(),
                }),
            },
            source_identities: vec![path.hops()[1].to.clone()],
            nullable: false,
        },
    ];
    e.obligations[0].parameters["paths"] =
        json!([{"path":path.serialized(),"edgeEncoding":"signed64-decimal/0.1"}]);
    e.obligations[0].parameters["checks"]
        .as_array_mut()
        .unwrap()
        .pop();
    e.obligations.push(Obligation{id:"ashlar.path.countCapacity".into(),owner:crate::backend::ObligationOwner::Host,failure_code:"WFT-CAPABILITY".into(),parameters:json!({"phase":"before-user-query","samePublicationRequired":true,"noPartialPublication":true,"nativeRepresentation":"signed64","checks":[{"pathOccurrence":v.expansion().unwrap().occurrence,"kind":"pathRows","sql":"SELECT '0'","failureCode":"WFT-CAPABILITY"},{"pathOccurrence":v.expansion().unwrap().occurrence,"kind":"targetDistinct","sql":"SELECT '0'","failureCode":"WFT-CAPABILITY"}],"success":"one exact STRING count equal to 0 per check"})});
    emission::validate(
        v,
        &e,
        &selected,
        &q,
        &m.target_profiles[0],
        &m,
        &binding(),
        &edges,
        &[],
    )
    .unwrap();
    for change in 0..5 {
        let mut bad = e.clone();
        match change {
            0 => {
                bad.obligations.pop();
            }
            1 => {
                bad.obligations[1].parameters["checks"]
                    .as_array_mut()
                    .unwrap()
                    .remove(0);
            }
            2 => bad.obligations[1].parameters["checks"][0]["pathOccurrence"] = json!("forged"),
            3 => {
                let Representation::Scalar {
                    path_target: Some(t),
                    ..
                } = &mut bad.columns[1].representation
                else {
                    panic!()
                };
                t.record.revision = "forged".into();
            }
            _ => {
                let Representation::Scalar {
                    path_target: Some(t),
                    ..
                } = &mut bad.columns[1].representation
                else {
                    panic!()
                };
                t.key.fields[0].element = "forged".into();
            }
        }
        assert!(
            emission::validate(
                v,
                &bad,
                &selected,
                &q,
                &m.target_profiles[0],
                &m,
                &binding(),
                &edges,
                &[]
            )
            .is_err(),
            "count mutation {change}"
        );
    }
}
#[test]
fn left_match_integrity_requires_exact_immutable_record_source() {
    let c = catalog();
    let p=crate::path_application_resolve::resolve(&c,crate::path_query::parse("SELECT RELATED_PATHS(l.\"order_lines.product_id\", \"products.supplier_id\", 20) AS paths FROM order_lines l LEFT JOIN products p ON l.product_id=p.id").unwrap(),Default::default()).unwrap();
    let v = Plan04View::new(&p);
    let m = manifest(v);
    let selected = emission::selection(v).unwrap();
    let q: Vec<_> = m
        .capabilities
        .iter()
        .map(|d| crate::backend::Qualification {
            assessment: Assessment {
                id: d.id.clone(),
                status: Status::Candidate,
                evidence: vec![],
                obligations: vec![],
            },
            declaration: d.clone(),
        })
        .collect();
    let (mut e, edges) = path_emission(v);
    let right = &v.joins()[0].right;
    let source = NativeRecordSource {
        scan: right.occurrence.clone(),
        record: right.record.clone(),
        table: PinnedTable {
            name: ["fixture".into(), "original".into(), "products".into()],
            uuid: "fixture-products".into(),
            version: 2,
        },
        identity_column: "source_specific_id".into(),
        native_type: "STRING".into(),
    };
    e.obligations.push(Obligation{id:"outerJoin.matchIntegrity".into(),owner:crate::backend::ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into(),parameters:json!({"phase":"before-user-query","samePublicationRequired":true,"noPartialPublication":true,"scans":[{"scan":source.scan,"record":source.record,"table":source.table,"identityColumn":source.identity_column,"nativeType":source.native_type,"sql":"SELECT '0'"}]})});
    emission::validate(
        v,
        &e,
        &selected,
        &q,
        &m.target_profiles[0],
        &m,
        &binding(),
        &edges,
        std::slice::from_ref(&source),
    )
    .unwrap();
    for change in 0..5 {
        let mut bad = e.clone();
        match change {
            0 => {
                bad.obligations.pop();
            }
            1 => bad.obligations[1].parameters["scans"][0]["table"]["uuid"] = json!("forged"),
            2 => bad.obligations[1].parameters["scans"][0]["record"]["revision"] = json!("forged"),
            3 => bad.obligations[1].parameters["scans"][0]["scan"] = json!("forged"),
            _ => bad.obligations[1].parameters["scans"][0]["identityColumn"] = json!("forged"),
        };
        assert!(
            emission::validate(
                v,
                &bad,
                &selected,
                &q,
                &m.target_profiles[0],
                &m,
                &binding(),
                &edges,
                std::slice::from_ref(&source)
            )
            .is_err(),
            "LEFT map mutation {change}"
        );
    }
    assert!(emission::validate(
        v,
        &e,
        &selected,
        &q,
        &m.target_profiles[0],
        &m,
        &binding(),
        &edges,
        &[]
    )
    .is_err());
}
#[test]
fn repeated_fields_require_the_exact_positioned_custody_obligation() {
    let c = catalog();
    let p = crate::path_application_resolve::resolve(
        &c,
        crate::path_query::parse("SELECT l.id,l.id FROM order_lines l").unwrap(),
        Default::default(),
    )
    .unwrap();
    let v = Plan04View::new(&p);
    let m = manifest(v);
    let selected = emission::selection(v).unwrap();
    let q: Vec<_> = m
        .capabilities
        .iter()
        .map(|d| crate::backend::Qualification {
            assessment: Assessment {
                id: d.id.clone(),
                status: Status::Candidate,
                evidence: vec![],
                obligations: vec![],
            },
            declaration: d.clone(),
        })
        .collect();
    let columns: Vec<_> = v
        .outputs()
        .enumerate()
        .map(|(i, o)| {
            let ExpressionView::Legacy(crate::arithmetic_plan::Expression::Field {
                identity, ..
            }) = o.expression
            else {
                panic!()
            };
            let crate::application_model::Shape::Scalar { logical_type } = &v
                .type_graph()
                .iter()
                .find(|d| &d.identity == identity)
                .unwrap()
                .shape
            else {
                panic!()
            };
            Column {
                position: i + 1,
                output_name: o.name.into(),
                carrier_name: Some(format!("fixture_ordinal_{i}")),
                representation: Representation::Scalar {
                    logical_type: logical_type.clone(),
                    carrier: crate::backend::ScalarCarrier::Text,
                    decoder: crate::backend::ScalarDecoder::Text,
                    path_target: None,
                },
                source_identities: vec![identity.clone()],
                nullable: false,
            }
        })
        .collect();
    let map=json!(columns.iter().map(|c|json!({"position":c.position,"outputName":c.output_name,"carrierName":c.carrier_name,"sourceIdentities":c.source_identities})).collect::<Vec<_>>());
    let e = Emission {
        sql: "SELECT fixture_only".into(),
        parameters: vec![],
        columns,
        obligations: vec![Obligation {
            id: "weft.output.positioned".into(),
            owner: crate::backend::ObligationOwner::Host,
            failure_code: "WFT-OBLIGATION".into(),
            parameters: json!({"profile":"weft-positioned-output/0.3.0","columns":map}),
        }],
    };
    emission::validate(
        v,
        &e,
        &selected,
        &q,
        &m.target_profiles[0],
        &m,
        &binding(),
        &[],
        &[],
    )
    .unwrap();
    for change in 0..6 {
        let mut bad = e.clone();
        match change {
            0 => bad.obligations.clear(),
            1 => bad.obligations[0].parameters["columns"][0]["carrierName"] = json!("forged"),
            2 => bad.obligations[0].parameters["profile"] = json!("forged"),
            3 => bad.obligations[0].owner = crate::backend::ObligationOwner::Backend,
            4 => bad.obligations[0].failure_code = "WFT-BINDING".into(),
            _ => bad.obligations[0].parameters["columns"][1]["sourceIdentities"] = json!([]),
        }
        assert!(
            emission::validate(
                v,
                &bad,
                &selected,
                &q,
                &m.target_profiles[0],
                &m,
                &binding(),
                &[],
                &[]
            )
            .is_err(),
            "isolated positioned mutation {change}"
        );
    }
}
#[test]
fn manifest_identifiers_and_obligation_codes_preserve_original_rejection_rules() {
    let c = catalog();
    let p = plan(&c);
    let original = serde_json::to_value(manifest(Plan04View::new(&p))).unwrap();
    for bad in ["", "name\0suffix"] {
        for field in ["backendId", "backendVersion", "bindingProfile"] {
            let mut v = original.clone();
            v[field] = json!(bad);
            assert!(validate_manifest_json(&v.to_string()).is_err(), "{field}");
        }
        for field in [
            "id",
            "engine",
            "engineVersion",
            "storageLayoutRevision",
            "publicationRevision",
        ] {
            let mut v = original.clone();
            v["targetProfiles"][0][field] = json!(bad);
            assert!(
                validate_manifest_json(&v.to_string()).is_err(),
                "target {field}"
            );
        }
        let mut v = original.clone();
        v["capabilities"][0]["id"] = json!(bad);
        assert!(validate_manifest_json(&v.to_string()).is_err());
        let mut v = original.clone();
        v["capabilities"][0]["obligations"] =
            json!([{"id":bad,"owner":"host","parameters":{},"failureCode":"WFT-OBLIGATION"}]);
        assert!(validate_manifest_json(&v.to_string()).is_err());
    }
    for code in ["WFT-", "WFT-invalid", "WFT-NUL\0"] {
        let mut v = original.clone();
        v["capabilities"][0]["obligations"] =
            json!([{"id":"fixture-obligation","owner":"host","parameters":{},"failureCode":code}]);
        assert!(validate_manifest_json(&v.to_string()).is_err());
        let o = Obligation {
            id: "fixture-obligation".into(),
            owner: crate::backend::ObligationOwner::Host,
            parameters: json!({}),
            failure_code: code.into(),
        };
        assert!(registry::merge(&mut vec![], &[o]).is_err());
    }
}
