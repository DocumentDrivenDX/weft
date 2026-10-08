//! Candidate registered SQL lowering. Native execution/custody stays in hosts.
mod application;
use crate::binding::{self, Binding, Home, RecordKind};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    backend::*,
    error::{Diagnostic, Result},
    ir::{Expression, Family, Identity, LogicalType, Node},
};

pub struct Candidate;
pub struct TargetPlan(Emission);
fn fail(code: &str, message: &str) -> Diagnostic {
    Diagnostic::new(code, "lower", message)
}
fn string_type() -> LogicalType {
    LogicalType {
        family: Family::String,
        facets: json!({}),
        nullable: false,
    }
}
fn publication(binding: &Binding) -> Obligation {
    Obligation {
        id: "ashlar.candidate.publication".into(),
        parameters: json!({
        "publication":binding.publication,"modelPins":binding.model_pins,"layoutRevision":binding.layout_revision,"layoutSha256":binding.layout_sha256,
        "requirements":["authenticate effective caller and establish complete authorized input view","verify one complete immutable manifest with trusted UUID and all schema revisions/projection dependencies","verify every consumed native schema, table UUID, version and retained data-file custody","verify source/type/id uniqueness, typed endpoint integrity and complete serving projection coverage at this publication","check policy and pin custody before execution and again before buffered result publication","refuse unknown obligations or unavailable evidence; no latest version or broader principal fallback","validate original mapping correspondence independently; identifiers and physical IDs are not logical identity"],
        "nativeProfile":{"versionReported":"4.2.0 zero build hash","warehouseRelease":"unqualified","comparison":"UTF8_BINARY","arithmetic":"ANSI exact-or-error"},
        "payloadValidation":["exact validated JSON text with duplicate-key refusal","well-formed Unicode scalar strings with no NUL","registered fixed-base-ten numeric carrier; exponent/DOUBLE representations refuse","no rounding, coercion, missing/null substitution or hidden corrupt inputs"],
        "visibility":"hidden input is not evidence of absence or integrity","publicationPhase":"after-complete-buffer-and-context-recheck"}),
        owner: ObligationOwner::Host,
        failure_code: "WFT-OBLIGATION".into(),
    }
}
impl Backend for Candidate {
    type Mapping = Binding;
    type TargetPlan = TargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let languages = ["0.1.0", "0.2.0"]
            .into_iter()
            .map(|v| LanguageProfile {
                dialect_profile: format!("weft-sql/{v}"),
                ir_version: format!("weft-ir/{v}"),
            })
            .collect::<Vec<_>>();
        Ok(Manifest{backend_id:"ashlar.databricks".into(),backend_version:"0.1.0-candidate".into(),interface_version:"weft-backend/0.2.0".into(),language_profiles:languages.clone(),binding_profile:binding::PROFILE.into(),
            target_profiles:vec![TargetProfile{id:"dbsql-candidate".into(),engine:"databricks-sql".into(),engine_version:"unqualified-warehouse-release".into(),session_settings:json!({"comparison":"UTF8_BINARY","arithmetic":"ANSI exact-or-error","variantCarrier":"fixed-base-ten-exact-only"}),storage_layout_revision:"ashlar-delta/0.3".into(),publication_revision:"ashlar-resolver/0.1-candidate".into()}],
            capabilities:["scan","project","project.entity","filter","innerJoin","equal","and","sum","group","aggregate","aggregate.count","parameter.named","compare.lexicographicGreater","order.asc","limit","key.uniqueStable","value.presence","type.string","type.boolean","type.integer","type.decimal"].into_iter().map(|id|Capability{id:id.into(),target_profiles:vec!["dbsql-candidate".into()],language_profiles:languages.clone(),logical_domain:if id == "value.presence" { json!({"subset":"optional scalar envelopes; absent or exact value; explicit native null refuses"}) } else { json!({"subset":"required scalar relational/application operations with admitted exact homes"}) },result_domain:json!({"carrier":"exact text","sumCount":"finite DECIMAL38 or error; empty count zero"}),constraints:vec!["Candidate requires native/schema/policy/publication host verification".into(),"No broader warehouse or production qualification".into()],obligations:vec![],status:Status::Candidate,evidence:vec![]}).collect(),evidence:vec![]})
    }
    fn validate_binding(&self, c: &Context<'_>) -> Result<Validated<Binding>> {
        let binding = binding::admit(c.catalog, c.binding_value)?;
        for id in &c.selection.records {
            if !binding.records.iter().any(|r| &r.logical == id) {
                return Err(fail(
                    "WFT-BINDING",
                    "Selected original record has no mapping",
                ));
            }
        }
        for id in &c.selection.fields {
            if !binding
                .records
                .iter()
                .any(|r| r.properties.iter().any(|p| &p.logical == id))
            {
                return Err(fail(
                    "WFT-BINDING",
                    "Selected original field has no mapping",
                ));
            }
        }
        Ok(Validated {
            obligations: vec![publication(&binding)],
            mapping: binding,
            additional_capabilities: vec![],
            coverage: c.selection.clone(),
        })
    }
    fn assess(&self, c: &Context<'_>, _: &Binding) -> Result<Vec<Assessment>> {
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
    fn lower(&self, c: &Context<'_>, binding: &Binding) -> Result<TargetPlan> {
        if let Plan::V02(plan) = c.plan {
            return application::lower(c, binding, plan);
        }
        let Plan::V01(plan) = c.plan else {
            return Err(fail(
                "WFT-CAPABILITY",
                "Application reads require their own admitted lowering",
            ));
        };
        let Node::Project { input, outputs } = &plan.root else {
            return Err(fail("WFT-EMIT", "Projection root is required"));
        };
        let mut lower = Lower::new(binding);
        collect(&plan.root, &mut lower.scans, &mut lower.fields);
        lower.prepare()?;
        let mut sql_outputs = Vec::new();
        let mut columns = Vec::new();
        for (index, output) in outputs.iter().enumerate() {
            let expression = lower.expression(&output.expression)?;
            sql_outputs.push(format!(
                "CAST({expression} AS STRING) AS {}",
                binding::quote(&output.name)
            ));
            let ty = output.expression.logical_type();
            columns.push(Column {
                position: index + 1,
                output_name: output.name.clone(),
                representation: Representation::Scalar {
                    logical_type: ty.clone(),
                    carrier: ScalarCarrier::Text,
                    decoder: match ty.family {
                        Family::String => ScalarDecoder::Text,
                        Family::Boolean => ScalarDecoder::Boolean,
                        Family::Integer => ScalarDecoder::ExactInteger,
                        Family::Decimal => ScalarDecoder::ExactDecimal,
                    },
                },
                source_identities: sources(&output.expression),
                nullable: ty.nullable,
            });
        }
        let mut filters = Vec::new();
        let mut groups = Vec::new();
        let from = lower.from(input, &mut filters, &mut groups)?;
        let mut sql = format!(
            "WITH {} SELECT {} FROM {from}",
            lower.ctes.join(", "),
            sql_outputs.join(", ")
        );
        if !filters.is_empty() {
            sql.push_str(&format!(" WHERE {}", filters.join(" AND ")));
        }
        if !groups.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", groups.join(", ")));
        }
        let integrity = Obligation {
            id: "ashlar.candidate.scalarIntegrity".into(),
            parameters: json!({"phase":"before-user-query","checks":lower.checks,"success":"one exact STRING count equal to 0 per check","parameters":"same emitted ordered slots; values never interpolated","samePublicationRequired":true,"noPartialPublication":true}),
            owner: ObligationOwner::Host,
            failure_code: "WFT-NUMERIC-DOMAIN".into(),
        };
        Ok(TargetPlan(Emission {
            sql,
            parameters: lower.parameters,
            columns,
            obligations: vec![integrity, publication(binding)],
        }))
    }
    fn emit(&self, _: &Context<'_>, p: &TargetPlan) -> Result<Emission> {
        Ok(Emission {
            sql: p.0.sql.clone(),
            parameters: p.0.parameters.clone(),
            columns: p.0.columns.clone(),
            obligations: p.0.obligations.clone(),
        })
    }
}
fn sources(e: &Expression) -> Vec<Identity> {
    match e {
        Expression::Field { identity, .. } => vec![identity.clone()],
        Expression::Sum { argument, .. } => sources(argument),
        Expression::Equal { left, right, .. } | Expression::And { left, right, .. } => {
            let mut out = sources(left);
            out.extend(sources(right));
            out
        }
        Expression::Literal { .. } => vec![],
    }
}
fn collect_expression(
    e: &Expression,
    fields: &mut BTreeMap<(String, String), (Identity, LogicalType)>,
) {
    match e {
        Expression::Field {
            scan,
            identity,
            logical_type,
            ..
        } => {
            fields.insert(
                (scan.clone(), serde_json::to_string(identity).unwrap()),
                (identity.clone(), logical_type.clone()),
            );
        }
        Expression::Equal { left, right, .. } | Expression::And { left, right, .. } => {
            collect_expression(left, fields);
            collect_expression(right, fields);
        }
        Expression::Sum { argument, .. } => collect_expression(argument, fields),
        Expression::Literal { .. } => {}
    }
}
fn collect(
    node: &Node,
    scans: &mut BTreeMap<String, Identity>,
    fields: &mut BTreeMap<(String, String), (Identity, LogicalType)>,
) {
    match node {
        Node::Scan {
            occurrence, record, ..
        } => {
            scans.insert(occurrence.clone(), record.clone());
        }
        Node::InnerJoin { left, right, on } => {
            collect(left, scans, fields);
            collect(right, scans, fields);
            collect_expression(on, fields);
        }
        Node::Filter { input, predicate } => {
            collect(input, scans, fields);
            collect_expression(predicate, fields);
        }
        Node::Aggregate {
            input,
            groups,
            aggregates,
        } => {
            collect(input, scans, fields);
            for e in groups.iter().chain(aggregates) {
                collect_expression(e, fields);
            }
        }
        Node::Project { input, outputs } => {
            collect(input, scans, fields);
            for output in outputs {
                collect_expression(&output.expression, fields);
            }
        }
    }
}
struct Lower<'a> {
    binding: &'a Binding,
    scans: BTreeMap<String, Identity>,
    fields: BTreeMap<(String, String), (Identity, LogicalType)>,
    expressions: BTreeMap<(String, String), String>,
    optional: BTreeSet<(String, String)>,
    presence: BTreeMap<(String, String), String>,
    parameters: Vec<ParameterSlot>,
    ctes: Vec<String>,
    checks: Vec<Value>,
}
impl<'a> Lower<'a> {
    fn new(binding: &'a Binding) -> Self {
        Self {
            binding,
            scans: BTreeMap::new(),
            fields: BTreeMap::new(),
            expressions: BTreeMap::new(),
            optional: BTreeSet::new(),
            presence: BTreeMap::new(),
            parameters: vec![],
            ctes: vec![],
            checks: vec![],
        }
    }
    fn slot(&mut self, ty: LogicalType, value: String, origin: Value) -> Result<String> {
        if self.parameters.len() >= 1024 {
            return Err(fail("WFT-LIMIT", "Databricks slots exceed 1024"));
        }
        let position = self.parameters.len() + 1;
        self.parameters.push(ParameterSlot {
            position,
            logical_type: ty,
            value,
            origin,
        });
        Ok(format!(":p{position}"))
    }
    fn prepare(&mut self) -> Result<()> {
        for (scan, id) in self.scans.clone() {
            let record = self
                .binding
                .records
                .iter()
                .find(|r| r.logical == id)
                .cloned()
                .ok_or_else(|| fail("WFT-BINDING", "Missing scan mapping"))?;
            let table = self.binding.publication.tables[record.table].sql();
            let source = self.slot(
                string_type(),
                record.source_system.clone(),
                json!({"kind":"sourceDiscriminator","record":id}),
            )?;
            let type_slot = self.slot(
                LogicalType {
                    family: Family::Integer,
                    facets: json!({"integerWidth":{"bits":64,"signed":true}}),
                    nullable: false,
                },
                record.type_id.clone(),
                json!({"kind":"typeDiscriminator","record":id}),
            )?;
            let type_column =
                if matches!(record.kind, RecordKind::Object | RecordKind::NodeProjection) {
                    "type_id"
                } else {
                    "rel_type_id"
                };
            let owner=format!("(r.source_system COLLATE UTF8_BINARY) = ({source} COLLATE UTF8_BINARY) AND r.{type_column} = CAST({type_slot} AS BIGINT)");
            let revision = if matches!(record.kind, RecordKind::Object | RecordKind::Edge) {
                let p = self.slot(
                    string_type(),
                    record.schema_revision.clone(),
                    json!({"kind":"schemaRevision","record":id}),
                )?;
                format!("(r.schema_revision COLLATE UTF8_BINARY) = ({p} COLLATE UTF8_BINARY)")
            } else {
                "TRUE".into()
            };
            let mut projections = Vec::new();
            for ((occurrence, key), (identity, ty)) in self.fields.clone() {
                if occurrence != scan {
                    continue;
                }
                let property = record
                    .properties
                    .iter()
                    .find(|p| p.logical == identity)
                    .ok_or_else(|| {
                        fail("WFT-BINDING", "Mapped field does not belong to its scan")
                    })?;
                let optional = self.optional.contains(&(scan.clone(), key.clone()));
                let (value, valid, present) = match &property.home {
                    Home::Props { property_id } => {
                        let path = self.slot(
                            string_type(),
                            format!("$.{property_id}"),
                            json!({"kind":"propertyPath","field":identity}),
                        )?;
                        let variant = format!("variant_get(parse_json(r.props_json), {path})");
                        let schema = format!("schema_of_variant({variant})");
                        let scalar = format!("CAST({variant} AS STRING)");
                        let valid=match ty.family {
                            Family::String=>format!("{schema} = 'STRING' AND instr({scalar}, char(0)) = 0"),
                            Family::Boolean=>format!("{schema} = 'BOOLEAN'"),
                            Family::Integer=>format!("({schema} = 'BIGINT' OR {schema} RLIKE '^DECIMAL\\\\([0-9]+,0\\\\)$') AND {}",numeric_guard(&scalar,&ty)),
                            Family::Decimal=>format!("({schema} = 'BIGINT' OR ({schema} RLIKE '^DECIMAL\\\\([0-9]+,[0-9]+\\\\)$' AND coalesce(try_cast(regexp_extract({schema}, ',([0-9]+)\\\\)$', 1) AS INT), 0) <= {})) AND {}",ty.facets["scale"],numeric_guard(&scalar,&ty)),
                        };
                        let present = format!("{variant} IS NOT NULL");
                        let valid = if optional {
                            format!("schema_of_variant(parse_json(r.props_json)) RLIKE '^OBJECT' AND (({variant} IS NULL) OR ({valid}))")
                        } else {
                            valid
                        };
                        (typed(&scalar, &ty), valid, present)
                    }
                    Home::Column { value, present, .. } => {
                        let column = format!("r.{}", binding::quote(value));
                        let mut valid = format!("{column} IS NOT NULL");
                        if let Some(present) = present {
                            valid.push_str(&format!(" AND r.{} = TRUE", binding::quote(present)));
                        }
                        if ty.family == Family::Integer {
                            valid.push_str(&format!(" AND {}", numeric_guard(&column, &ty)));
                        }
                        if ty.family == Family::String {
                            valid.push_str(&format!(" AND instr({column}, char(0)) = 0"));
                        }
                        let presence = present.as_ref().map(|p| format!("r.{}", binding::quote(p)));
                        let valid = if optional {
                            if let Some(p) = &presence {
                                format!("(({p} = FALSE AND {column} IS NULL) OR ({valid}))")
                            } else {
                                valid
                            }
                        } else {
                            valid
                        };
                        (
                            typed(&column, &ty),
                            valid,
                            presence.unwrap_or_else(|| "TRUE".into()),
                        )
                    }
                };
                let field_alias = format!("f{}", projections.len());
                projections.push(format!("{value} AS {}", binding::quote(&field_alias)));
                if optional {
                    let present_alias = format!("p{}", projections.len());
                    projections.push(format!("({present}) AS {}", binding::quote(&present_alias)));
                    self.presence.insert(
                        (scan.clone(), key.clone()),
                        format!(
                            "{}.{}",
                            binding::quote(&scan),
                            binding::quote(&present_alias)
                        ),
                    );
                }
                self.expressions.insert(
                    (scan.clone(), key),
                    format!("{}.{}", binding::quote(&scan), binding::quote(&field_alias)),
                );
                let count = count_sql();
                self.checks.push(json!({"field":identity,"record":id,"sql":format!("SELECT CAST({count} AS STRING) AS violations FROM {table} r WHERE {owner} AND CASE WHEN ({revision}) AND ({valid}) THEN FALSE ELSE TRUE END"),"failureCode":"WFT-NUMERIC-DOMAIN"}));
            }
            if projections.is_empty() {
                projections.push("1 AS __row".into());
                let count = count_sql();
                self.checks.push(json!({"record":id,"sql":format!("SELECT CAST({count} AS STRING) AS violations FROM {table} r WHERE {owner} AND CASE WHEN ({revision}) THEN FALSE ELSE TRUE END"),"failureCode":"WFT-BINDING"}));
            }
            self.ctes.push(format!(
                "{} AS (SELECT {} FROM {table} r WHERE {owner})",
                binding::quote(&scan),
                projections.join(", ")
            ));
        }
        Ok(())
    }
    fn expression(&mut self, e: &Expression) -> Result<String> {
        Ok(match e {
            Expression::Field {
                scan,
                identity,
                logical_type,
                ..
            } => {
                let value = self
                    .expressions
                    .get(&(scan.clone(), serde_json::to_string(identity).unwrap()))
                    .ok_or_else(|| fail("WFT-BINDING", "Selected field lacks its scan home"))?
                    .clone();
                if logical_type.family == Family::String {
                    format!("({value} COLLATE UTF8_BINARY)")
                } else {
                    value
                }
            }
            Expression::Literal {
                value,
                logical_type,
                span,
            } => {
                let slot = self.slot(
                    logical_type.clone(),
                    value.clone(),
                    json!({"kind":"literal","sourceSpan":span}),
                )?;
                typed(&slot, logical_type)
            }
            Expression::Equal { left, right, .. } => {
                format!("({} = {})", self.expression(left)?, self.expression(right)?)
            }
            Expression::And { left, right, .. } => format!(
                "({} AND {})",
                self.expression(left)?,
                self.expression(right)?
            ),
            Expression::Sum { argument, .. } => {
                let value = self.expression(argument)?;
                // TRY_SUM makes finite aggregate overflow observable regardless
                // of ANSI settings. Empty global aggregates retain SQL NULL.
                format!("CASE WHEN MAX(1) IS NOT NULL AND TRY_SUM({value}) IS NULL THEN raise_error('WFT-NUMERIC-DOMAIN') ELSE TRY_SUM({value}) END")
            }
        })
    }
    fn from(
        &mut self,
        node: &Node,
        filters: &mut Vec<String>,
        groups: &mut Vec<String>,
    ) -> Result<String> {
        match node {
            Node::Scan { occurrence, .. } => Ok(binding::quote(occurrence)),
            Node::InnerJoin { left, right, on } => Ok(format!(
                "({} INNER JOIN {} ON {})",
                self.from(left, filters, groups)?,
                self.from(right, filters, groups)?,
                self.expression(on)?
            )),
            Node::Filter { input, predicate } => {
                filters.push(self.expression(predicate)?);
                self.from(input, filters, groups)
            }
            Node::Aggregate {
                input, groups: g, ..
            } => {
                for e in g {
                    groups.push(self.expression(e)?);
                }
                self.from(input, filters, groups)
            }
            Node::Project { .. } => Err(fail(
                "WFT-EMIT",
                "Nested projection is not in the resolved relational slice",
            )),
        }
    }
}
fn count_sql() -> String {
    "CASE WHEN MAX(1) IS NULL THEN CAST(0 AS DECIMAL(38,0)) WHEN TRY_SUM(CAST(1 AS DECIMAL(38,0))) IS NULL THEN raise_error('WFT-NUMERIC-DOMAIN') ELSE TRY_SUM(CAST(1 AS DECIMAL(38,0))) END".into()
}
fn typed(value: &str, ty: &LogicalType) -> String {
    match ty.family {
        Family::String => format!("(CAST({value} AS STRING) COLLATE UTF8_BINARY)"),
        Family::Boolean => format!("CAST({value} AS BOOLEAN)"),
        Family::Integer => format!("CAST({value} AS DECIMAL(38,0))"),
        Family::Decimal => format!("CAST({value} AS DECIMAL(38,{}))", ty.facets["scale"]),
    }
}
fn numeric_guard(value: &str, ty: &LogicalType) -> String {
    match ty.family {
        Family::Integer => {
            let bits = ty.facets["integerWidth"]["bits"].as_u64().unwrap();
            let (min, max) = if ty.facets["integerWidth"]["signed"] == true {
                (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
            } else {
                (0, (1i128 << bits) - 1)
            };
            format!("try_cast({value} AS DECIMAL(38,0)) BETWEEN {min} AND {max}")
        }
        Family::Decimal => format!(
            "try_cast({value} AS DECIMAL({},{})) IS NOT NULL",
            ty.facets["precision"], ty.facets["scale"]
        ),
        _ => "FALSE".into(),
    }
}
