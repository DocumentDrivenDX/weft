// @covers US-002-AC1 @covers US-002-AC2 @covers US-002-AC3 @covers US-002-AC4
use serde_json::{json, Value};
use weft_core::{
    backend::*,
    ir::{Expression, Family, Identity, Node},
    model::Catalog,
};
#[derive(Clone, Copy)]
pub(crate) enum Behavior {
    Normal,
    WrongLabel,
    MissingColumns,
    WrongCarrier,
    WrongType,
    BadSlots,
    ParameterLexical,
    UnknownSource,
    WrongNullable,
    MissingCoverage,
    MissingAssessment,
    Upgrade,
    Panics,
    EmptySql,
    LowerFailure,
}
pub(crate) struct Third {
    pub(crate) manifest: Manifest,
    pub(crate) behavior: Behavior,
}
pub(crate) struct Mapping {
    table: String,
    column: String,
}
pub(crate) struct Select {
    table: String,
    column: String,
    label: String,
    identity: Identity,
}
fn error(code: &str) -> weft_core::error::Diagnostic {
    weft_core::error::Diagnostic::new(code, "lower", "Fixture refusal")
}
pub(crate) fn manifest(status: Status) -> Manifest {
    let language = vec![
        LanguageProfile {
            dialect_profile: "weft-sql/0.1.0".into(),
            ir_version: "weft-ir/0.1.0".into(),
        },
        LanguageProfile {
            dialect_profile: "weft-sql/0.2.0".into(),
            ir_version: "weft-ir/0.2.0".into(),
        },
    ];
    Manifest {
        backend_id: "test.third".into(),
        backend_version: "0.1.0".into(),
        interface_version: "weft-backend/0.2.0".into(),
        language_profiles: language.clone(),
        binding_profile: "test-third-binding/0.1.0".into(),
        target_profiles: vec![TargetProfile {
            id: "fixture-only".into(),
            engine: "synthetic".into(),
            engine_version: "test-1".into(),
            session_settings: json!({"comparison":"unicode-scalar"}),
            storage_layout_revision: "fixture-1".into(),
            publication_revision: "fixture-1".into(),
        }],
        capabilities: ["scan", "project", "type.string"]
            .iter()
            .map(|id| Capability {
                id: (*id).into(),
                target_profiles: vec!["fixture-only".into()],
                language_profiles: language.clone(),
                logical_domain: json!({"subset":"single required string projection"}),
                result_domain: json!({"bag":"preserved"}),
                constraints: vec![],
                obligations: vec![],
                status: status.clone(),
                evidence: if status == Status::Supported {
                    vec!["fixture-proof".into()]
                } else {
                    vec![]
                },
            })
            .collect(),
        evidence: vec!["fixture-proof".into()],
    }
}
fn only(value: &Value, keys: &[&str]) -> bool {
    value
        .as_object()
        .is_some_and(|m| m.keys().all(|k| keys.contains(&k.as_str())))
}
fn identifier(s: &str) -> bool {
    s.as_bytes()
        .first()
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
impl Backend for Third {
    type Mapping = Mapping;
    type TargetPlan = Select;
    fn describe(&self) -> weft_core::error::Result<Manifest> {
        Ok(self.manifest.clone())
    }
    fn validate_binding(&self, c: &Context<'_>) -> weft_core::error::Result<Validated<Mapping>> {
        let v = c.binding_value;
        if !only(v, &["pins", "record", "field", "extensions"])
            || !only(&v["record"], &["identity", "table"])
            || !only(&v["field"], &["identity", "column"])
            || ![&v["record"]["identity"], &v["field"]["identity"]]
                .iter()
                .all(|r| only(r, &["documentId", "revision", "module", "element"]))
        {
            return Err(error("WFT-BINDING"));
        }
        if v["pins"] != serde_json::to_value(c.catalog.pins()).unwrap()
            || c.selection.records.len() != 1
            || c.selection.fields.len() != 1
            || v["record"]["identity"] != serde_json::to_value(&c.selection.records[0]).unwrap()
            || v["field"]["identity"] != serde_json::to_value(&c.selection.fields[0]).unwrap()
        {
            return Err(error("WFT-BINDING"));
        }
        let table = v["record"]["table"]
            .as_str()
            .filter(|v| identifier(v))
            .ok_or_else(|| error("WFT-BINDING"))?;
        let column = v["field"]["column"]
            .as_str()
            .filter(|v| identifier(v))
            .ok_or_else(|| error("WFT-BINDING"))?;
        Ok(Validated {
            additional_capabilities: vec![],
            mapping: Mapping {
                table: table.into(),
                column: column.into(),
            },
            coverage: if matches!(self.behavior, Behavior::MissingCoverage) {
                Selection::default()
            } else {
                c.selection.clone()
            },
            obligations: vec![],
        })
    }
    fn assess(&self, c: &Context<'_>, _: &Mapping) -> weft_core::error::Result<Vec<Assessment>> {
        Ok(c.plan
            .capabilities()
            .iter()
            .enumerate()
            .filter(|(i, _)| !matches!(self.behavior, Behavior::MissingAssessment) || *i != 0)
            .map(|(_, id)| {
                let Some(declaration) = self.manifest.capabilities.iter().find(|d| &d.id == id)
                else {
                    return Assessment {
                        id: id.clone(),
                        status: Status::Unsupported,
                        evidence: vec![],
                        obligations: vec![],
                    };
                };
                Assessment {
                    id: id.clone(),
                    status: if matches!(self.behavior, Behavior::Upgrade) {
                        Status::Supported
                    } else {
                        declaration.status.clone()
                    },
                    evidence: declaration.evidence.clone(),
                    obligations: vec![],
                }
            })
            .collect())
    }
    fn lower(&self, c: &Context<'_>, m: &Mapping) -> weft_core::error::Result<Select> {
        if matches!(self.behavior, Behavior::Panics) {
            panic!("fixture panic");
        }
        if matches!(self.behavior, Behavior::LowerFailure) {
            return Err(error("WFT-CAPABILITY"));
        }
        let label = match c.plan {
            Plan::V03(_) => return Err(error("WFT-BACKEND-VERSION")),
            Plan::V01(p) => {
                let Node::Project { input, outputs } = &p.root else {
                    return Err(error("WFT-CAPABILITY"));
                };
                if !matches!(**input, Node::Scan { .. })
                    || outputs.len() != 1
                    || !matches!(outputs[0].expression,Expression::Field{ref logical_type,..} if logical_type.family==Family::String)
                {
                    return Err(error("WFT-CAPABILITY"));
                }
                outputs[0].name.clone()
            }
            Plan::V02(p) => {
                if !p.joins.is_empty()
                    || !p.filters.is_empty()
                    || p.aggregate
                    || !p.order.is_empty()
                    || p.limit.is_some()
                    || p.outputs.len() != 1
                {
                    return Err(error("WFT-CAPABILITY"));
                }
                p.outputs[0].name.clone()
            }
        };
        Ok(Select {
            table: m.table.clone(),
            column: m.column.clone(),
            label,
            identity: c.selection.fields[0].clone(),
        })
    }
    fn emit(&self, _: &Context<'_>, p: &Select) -> weft_core::error::Result<Emission> {
        let label = p.label.replace('"', "\"\"");
        let mut output = Emission {
            sql: if matches!(self.behavior, Behavior::EmptySql) {
                String::new()
            } else {
                format!(
                    "SELECT \"{}\" AS \"{}\" FROM \"{}\"",
                    p.column, label, p.table
                )
            },
            parameters: vec![],
            columns: vec![Column {
                position: 1,
                carrier_name: None, output_name: p.label.clone(),
                representation: Representation::Scalar {
                    logical_type: weft_core::ir::LogicalType {
                        family: Family::String,
                        facets: json!({}),
                        nullable: false,
                    },
                    carrier: ScalarCarrier::Text,
                    decoder: ScalarDecoder::Text,
                },
                source_identities: vec![p.identity.clone()],
                nullable: false,
            }],
            obligations: vec![],
        };
        match self.behavior {
            Behavior::WrongLabel => output.columns[0].output_name = "other".into(),
            Behavior::MissingColumns => output.columns.clear(),
            Behavior::WrongCarrier => {
                if let Representation::Scalar { carrier, .. } =
                    &mut output.columns[0].representation
                {
                    *carrier = ScalarCarrier::Boolean;
                }
            }
            Behavior::WrongType => {
                if let Representation::Scalar { logical_type, .. } =
                    &mut output.columns[0].representation
                {
                    logical_type.family = Family::Integer;
                }
            }
            Behavior::WrongNullable => output.columns[0].nullable = true,
            Behavior::UnknownSource => {
                output.columns[0].source_identities[0].element = "unselected".into()
            }
            Behavior::BadSlots => output.parameters.push(ParameterSlot {
                position: 2,
                logical_type: weft_core::ir::LogicalType {
                    family: Family::String,
                    facets: json!({}),
                    nullable: false,
                },
                value: "safe".into(),
                origin: json!({"kind":"discriminator"}),
            }),
            Behavior::ParameterLexical => output.parameters.push(ParameterSlot {
                position: 1,
                logical_type: weft_core::ir::LogicalType {
                    family: Family::Integer,
                    facets: json!({"integerWidth":{"bits":64,"signed":false}}),
                    nullable: false,
                },
                value: "1; DROP TABLE x".into(),
                origin: json!({"kind":"discriminator"}),
            }),
            _ => {}
        }
        Ok(output)
    }
}
pub(crate) fn modules() -> Vec<weft_core::model::ModuleInput> {
    let cases: Value =
        serde_json::from_str(include_str!("../../docs/helix/03-test/fixtures/cases.json")).unwrap();
    serde_json::from_value(cases[0]["request"]["modules"].clone()).unwrap()
}
pub(crate) fn target(candidate: bool) -> Target {
    Target {
        backend_id: "test.third".into(),
        backend_version: "0.1.0".into(),
        profile_id: "fixture-only".into(),
        allow_candidate: candidate,
    }
}
pub(crate) fn binding(c: &Catalog) -> BindingInput {
    let text=json!({"pins":c.pins(),"record":{"identity":{"documentId":"sales-fixture","revision":c.pins()[0].revision,"module":"sales","element":"customer"},"table":"fixture_customers"},"field":{"identity":{"documentId":"sales-fixture","revision":c.pins()[0].revision,"module":"sales","element":"customer-name"},"column":"display_name"},"extensions":{"unknown":"retained"}}).to_string();
    BindingInput {
        profile: "test-third-binding/0.1.0".into(),
        sha256: weft_core::json::sha256(text.as_bytes()),
        json: text,
    }
}
pub(crate) fn registry(status: Status, behavior: Behavior) -> Registry {
    let mut r = Registry::default();
    r.register(Third {
        manifest: manifest(status),
        behavior,
    })
    .unwrap();
    r
}
