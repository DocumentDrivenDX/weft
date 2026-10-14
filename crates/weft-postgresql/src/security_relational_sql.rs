//! Original relational graph SQL candidate. Requires guarded native operand validation and admission.
use crate::{
    security_graph_binding::{GraphBinding, GraphSource},
    security_relational_parameters::{
        prepare_relational_graph_parameters, RelationalGraphParameterBindings,
        RelationalQueryParameterCodec,
    },
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    application_ir as app,
    error::{Diagnostic, Result},
    ir::{Family, Identity},
    security_backend::SecurityBackendContext,
    security_ir::QueryOperator,
    security_lowering::*,
    security_ontology::SecurityRef,
    security_requirements::SecurityOperatorMode,
};
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-RELATIONAL-SQL",
        "capability",
        "Original relational graph SQL correspondence refused",
    )
}
fn reference(i: &Identity) -> SecurityRef {
    SecurityRef {
        document_id: i.document_id.clone(),
        module_id: i.module.clone(),
        element_id: i.element.clone(),
    }
}
/// Compiler-owned immutable SQL/checks/result/parameter lineage. No native permit.
/// ```compile_fail
/// use weft_postgresql::security_relational_sql::RelationalGraphSql;
/// fn bypass(p:&mut RelationalGraphSql<'_, '_>){p.checks.clear();}
/// ```
pub struct RelationalGraphSql<'a, 'owner> {
    context: &'a SecurityBackendContext<'owner>,
    parameters: RelationalGraphParameterBindings<'a, 'owner>,
    sql: String,
    checks: Vec<Value>,
    contract: SecurityResultContract,
    operands: Vec<Value>,
    operand_reads: Vec<Value>,
}
impl<'a, 'owner> RelationalGraphSql<'a, 'owner> {
    pub fn sql(&self) -> &str {
        &self.sql
    }
    pub fn checks(&self) -> &[Value] {
        &self.checks
    }
    pub fn result_contract(&self) -> &SecurityResultContract {
        &self.contract
    }
    pub fn parameters(&self) -> &RelationalGraphParameterBindings<'a, 'owner> {
        &self.parameters
    }
    /// Original model operand demands. The native host must validate complete
    /// primary-permitted original/disclosed operand populations before executing SQL.
    pub fn operand_reads(&self) -> &[Value] {
        &self.operand_reads
    }
    pub fn operand_demands(&self) -> &[Value] {
        &self.operands
    }
    pub fn context_sha256(&self) -> String {
        self.context.execution_query_context_sha256()
    }
}
struct Build<'a, 'owner> {
    parameters: &'a RelationalGraphParameterBindings<'a, 'owner>,
    columns: BTreeMap<(String, SecurityRef, Option<QueryOperator>), String>,
    used: BTreeSet<String>,
}
impl Build<'_, '_> {
    fn field(&self, scan: &str, id: &Identity, op: Option<QueryOperator>) -> Result<String> {
        self.columns
            .get(&(scan.into(), reference(id), op))
            .cloned()
            .ok_or_else(fail)
    }
    fn value(&mut self, v: &app::Value, path: &str, op: QueryOperator) -> Result<String> {
        if let app::Value::Field { field } = v {
            return self.field(&field.scan, &field.identity, Some(op));
        }
        let source = self
            .parameters
            .query()
            .iter()
            .find(|p| p.source_path() == path)
            .ok_or_else(fail)?;
        // Actual AST object correspondence, not equality of annotations/value strings.
        if !std::ptr::eq(source.value(), v) || !self.used.insert(path.into()) {
            return Err(fail());
        }
        Ok(match source.codec() {
            RelationalQueryParameterCodec::Text => format!(
                "${}::pg_catalog.text COLLATE pg_catalog.\"C\"",
                source.ordinal()
            ),
            RelationalQueryParameterCodec::Boolean => {
                format!("${}::pg_catalog.bool", source.ordinal())
            }
            RelationalQueryParameterCodec::Numeric => {
                format!("${}::pg_catalog.numeric", source.ordinal())
            }
        })
    }
    fn predicate(&mut self, p: &app::Predicate, path: &str, op: QueryOperator) -> Result<String> {
        Ok(match p {
            app::Predicate::Equal { left, right } => format!(
                "({})=({})",
                self.field(&left.scan, &left.identity, Some(op))?,
                self.value(right, &format!("{path}/right"), op)?
            ),
            app::Predicate::LexicographicGreater { columns, values } => {
                if columns.len() != values.len() || columns.is_empty() {
                    return Err(fail());
                }
                let l = columns
                    .iter()
                    .map(|f| self.field(&f.scan, &f.identity, Some(op)))
                    .collect::<Result<Vec<_>>>()?;
                let r = values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| self.value(v, &format!("{path}/values/{i}"), op))
                    .collect::<Result<Vec<_>>>()?;
                format!("({})>({})", l.join(","), r.join(","))
            }
            _ => return Err(fail()),
        })
    }
}
fn model_field(ctx: &SecurityBackendContext<'_>, r: &SecurityRef) -> Result<Value> {
    let input = ctx
        .catalog()
        .inputs()
        .iter()
        .find(|d| d.pin.document_id == r.document_id)
        .ok_or_else(fail)?;
    let d = weft_core::json::checked_json(&input.document_json).map_err(|_| fail())?;
    let m = d["modules"]
        .as_array()
        .ok_or_else(fail)?
        .iter()
        .find(|m| m["id"] == r.module_id)
        .ok_or_else(fail)?;
    Ok(m["elements"]
        .as_array()
        .ok_or_else(fail)?
        .iter()
        .find(|f| f["id"] == r.element_id && f["kind"] == "field")
        .ok_or_else(fail)?
        .clone())
}
fn cell(group: &Value, r: &SecurityRef) -> Result<String> {
    let mut found = None;
    for f in group["disclosure"].as_array().ok_or_else(fail)? {
        if f["field"] == json!(r) {
            if found.is_some() {
                return Err(fail());
            }
            found = Some(f["cellExpression"].as_str().ok_or_else(fail)?.to_string());
        }
    }
    found.ok_or_else(fail)
}
fn anchor_sql(sql: &str, anchor: &str) -> String {
    if let Some(rest) = sql.strip_prefix("WITH ") {
        format!("WITH {anchor},{rest}")
    } else {
        format!("WITH {anchor} {sql}")
    }
}
fn scalar(literal: &str, family: &str) -> Result<String> {
    Ok(match family {
        "string" => format!("(({literal})->>'string') COLLATE pg_catalog.\"C\""),
        "boolean" => format!("(({literal})->>'boolean')::pg_catalog.bool"),
        "integer" => format!("(({literal})->>'integerToken')::pg_catalog.numeric"),
        "decimal" => format!("(({literal})->>'decimalToken')::pg_catalog.numeric"),
        _ => return Err(fail()),
    })
}
fn original_literal(raw: &str, family: &str) -> Result<String> {
    let (key, payload) = match family {
        "string" => ("string", format!("({raw})->'text'")),
        "boolean" => ("boolean", format!("({raw})->'value'")),
        "integer" => ("integerToken", format!("({raw})->'text'")),
        "decimal" => ("decimalToken", format!("({raw})->'text'")),
        _ => return Err(fail()),
    };
    Ok(format!("CASE WHEN ({raw})->>'kind'='null' THEN 'null'::pg_catalog.jsonb ELSE pg_catalog.jsonb_build_object('{key}',{payload}) END"))
}
/// Emits the actual original scans/joins/filters/groups/COUNT/SUM/projection/order/
/// limit. SQL is a candidate, never an authenticated executable artifact. Every
/// returned guard and original operand demand must be discharged under held
/// native source/Subject/current-authority cuts before execution and publication.
pub fn prepare_relational_graph_sql<'a, 'owner>(
    ctx: &'a SecurityBackendContext<'owner>,
    source: &GraphSource,
) -> Result<RelationalGraphSql<'a, 'owner>> {
    let parameters = prepare_relational_graph_parameters(ctx, source)?;
    let binding = GraphBinding::read(ctx.logical_plan(), ctx.catalog(), source)?;
    let graph =
        crate::security_predicate_spike::render_graph(ctx.logical_plan(), ctx.catalog(), source)?;
    let guards = crate::security_query_use_guards::prepare_graph_query_use_guards(ctx, source)?;
    let app = ctx.query().application_plan();
    if app.outputs.is_empty() || app.outputs.len() > 64 || guards.scans().len() > 64 {
        return Err(fail());
    }
    let subject = parameters.graph().subject();
    let subject_tid = subject.native_type_id();
    let slot = subject.ordinal();
    let mut checks = binding.integrity_checks().to_vec();
    checks.push(json!({"failureCode":"WFT-SECURITY-SUBJECT","sql":format!("SELECT CASE WHEN pg_catalog.count(*)=1 THEN '0' ELSE '1' END::pg_catalog.text AS violations FROM truss.object s WHERE s.type_id={subject_tid} AND s.id=${slot}::pg_catalog.int8")}));
    let mut types = parameters
        .graph()
        .policy()
        .iter()
        .map(|p| {
            let t = match p.codec() {
                crate::security_graph_parameters::GraphPolicyParameterCodec::Boolean => "bool",
                crate::security_graph_parameters::GraphPolicyParameterCodec::Text => "text",
                crate::security_graph_parameters::GraphPolicyParameterCodec::CoreLiteralJson => {
                    "jsonb"
                }
            };
            format!("${}::pg_catalog.{t} AS p{}", p.ordinal(), p.ordinal())
        })
        .collect::<Vec<_>>();
    types.push(format!("${slot}::pg_catalog.int8 AS p{slot}"));
    for p in parameters.query() {
        let t = match p.codec() {
            RelationalQueryParameterCodec::Boolean => "bool",
            RelationalQueryParameterCodec::Text => "text",
            RelationalQueryParameterCodec::Numeric => "numeric",
        };
        types.push(format!(
            "${}::pg_catalog.{t} AS p{}",
            p.ordinal(),
            p.ordinal()
        ));
    }
    let anchor = format!("parameter_types AS (SELECT {})", types.join(","));
    let mut ctes = Vec::new();
    let mut columns = BTreeMap::new();
    let mut targets = BTreeMap::new();
    let mut groups = BTreeMap::new();
    let mut operands = Vec::new();
    let mut operand_reads = Vec::new();
    for (i, scan) in guards.scans().iter().enumerate() {
        let alias = format!("a{i}");
        let tid = binding.type_id(scan.target())?;
        let truth = format!("t{i}");
        let group = graph["eligibility"]
            .as_array()
            .ok_or_else(fail)?
            .iter()
            .find(|g| {
                g["target"] == json!(scan.target())
                    && g["action"] == ctx.requirements().primary_action()
            })
            .ok_or_else(fail)?;
        let decision = group["decisionExpression"].as_str().ok_or_else(fail)?;
        let truth_fields = scan
            .rule_truths()
            .iter()
            .map(|r| format!("({}) AS {}", r.expression(), r.column()))
            .collect::<Vec<_>>();
        let base=format!("{truth} AS MATERIALIZED (SELECT r.id,r.props{} FROM truss.object r CROSS JOIN truss.object s WHERE r.type_id={tid} AND s.type_id={subject_tid} AND s.id=${slot}::pg_catalog.int8)",if truth_fields.is_empty(){String::new()}else{format!(",{}",truth_fields.join(","))});
        checks.push(json!({"failureCode":"WFT-SECURITY-QUERY-USE","sql":format!("WITH {base} SELECT pg_catalog.count(*)::pg_catalog.text AS violations FROM {truth} r WHERE {}",scan.refusal_expression())}));
        ctes.push(base);
        let mut fields = Vec::new();
        let mut captured = Vec::new();
        for output in &app.outputs {
            if let app::Expression::Field {
                scan: occurrence,
                identity,
            } = &output.expression
            {
                if occurrence == scan.scan() {
                    let r = reference(identity);
                    let key = (occurrence.clone(), r.clone(), None);
                    if !columns.contains_key(&key) {
                        let col = format!("c{}", fields.len());
                        fields.push(format!("({}) AS {col}", cell(group, &r)?));
                        columns.insert(key, format!("{alias}.{col}"));
                    }
                }
            }
        }
        for (operand_index, requirement) in ctx.requirements().operators().iter().enumerate() {
            let usage = requirement.usage();
            if usage.scan != scan.scan() {
                continue;
            }
            let r = &usage.field;
            let key = (usage.scan.clone(), r.clone(), Some(usage.operator));
            if columns.contains_key(&key) {
                continue;
            }
            let model = model_field(ctx, r)?;
            let family = model["scalarType"].as_str().ok_or_else(fail)?;
            let disclosure = group["disclosure"]
                .as_array()
                .ok_or_else(fail)?
                .iter()
                .find(|f| f["field"] == json!(r))
                .ok_or_else(fail)?;
            if matches!(requirement.mode(), SecurityOperatorMode::Disclosed) {
                for t in disclosure["transformations"].as_array().ok_or_else(fail)? {
                    if t["disposition"]["transformed"]["domain"]["scalarType"] != family {
                        return Err(fail());
                    }
                }
            }
            let literal = match requirement.mode() {
                SecurityOperatorMode::Disclosed => format!("({})->'value'", cell(group, r)?),
                SecurityOperatorMode::OriginalAuthorized(_) => {
                    original_literal(&binding.raw_cell(scan.target(), r, "r")?, family)?
                }
            };
            let capture = match requirement.mode() {
                SecurityOperatorMode::OriginalAuthorized(_) => {
                    let raw = binding.raw_cell(scan.target(), r, "r")?;
                    format!("CASE WHEN ({raw}) IS NULL THEN pg_catalog.jsonb_build_object('disposition','absent') ELSE pg_catalog.jsonb_build_object('disposition','original','value',({literal})) END")
                }
                SecurityOperatorMode::Disclosed => {
                    let choices = disclosure["transformations"]
                        .as_array()
                        .ok_or_else(fail)?
                        .iter()
                        .map(|t| {
                            let slot = t["slot"].as_u64().ok_or_else(fail)?;
                            let id = t["ruleId"].as_str().ok_or_else(fail)?.replace("'", "''");
                            Ok(format!("WHEN {slot} THEN '{id}'"))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let base = cell(group, r)?;
                    if choices.is_empty() {
                        format!("({base})-'outcomeId'")
                    } else {
                        let choice = disclosure["transformationChoiceExpression"]
                            .as_str()
                            .ok_or_else(fail)?;
                        format!("CASE WHEN ({base})->>'disposition'='transformed' THEN (({base})-'outcomeId') || pg_catalog.jsonb_build_object('ruleId',CASE ({choice}) {} ELSE NULL END) ELSE ({base})-'outcomeId' END",choices.join(" "))
                    }
                }
            };
            captured.push(format!(
                "pg_catalog.jsonb_build_object('index',{operand_index},'cell',({capture}))"
            ));
            let col = format!("c{}", fields.len());
            let scalar_value = scalar(&literal, family)?;
            let scalar_value = if usage.operator == QueryOperator::Group {
                let absent = match requirement.mode() {
                    SecurityOperatorMode::Disclosed => {
                        format!("({})->>'disposition'='absent'", cell(group, r)?)
                    }
                    SecurityOperatorMode::OriginalAuthorized(_) => {
                        format!("({}) IS NULL", binding.raw_cell(scan.target(), r, "r")?)
                    }
                };
                format!("ROW(({absent}),({scalar_value}))")
            } else {
                scalar_value
            };
            fields.push(format!(
                "CASE WHEN ({decision})='permit' THEN ({scalar_value}) ELSE NULL END AS {col}"
            ));
            columns.insert(key, format!("{alias}.{col}"));
            operands.push(json!({"scan":usage.scan,"target":scan.target(),"field":r,"operator":usage.operator,"domain":model,"disclosureSources":disclosure,"mode":match requirement.mode(){SecurityOperatorMode::Disclosed=>"disclosed",SecurityOperatorMode::OriginalAuthorized(_)=>"original-authorized"},"validation":"complete_original_owner_domain_presence_before_query_evaluation"}));
        }
        operand_reads.push(json!({"scan":scan.scan(),"sql":format!("WITH {} SELECT pg_catalog.jsonb_build_object('nativeRowId',r.id::pg_catalog.text,'operands',pg_catalog.jsonb_build_array({}))::pg_catalog.text AS operand_row FROM {truth} r WHERE ({decision})='permit' LIMIT 4097",ctes.last().ok_or_else(fail)?,captured.join(",")),"scope":"complete_primary_permitted_population_under_held_native_cuts","nativeAdmission":"required"}));
        ctes.push(format!(
            "{alias} AS MATERIALIZED (SELECT r.id{} FROM {truth} r WHERE ({decision})='permit')",
            if fields.is_empty() {
                String::new()
            } else {
                format!(",{}", fields.join(","))
            }
        ));
        targets.insert(scan.scan().to_string(), (alias, scan.target().clone()));
        groups.insert(scan.scan().to_string(), group);
    }
    let mut build = Build {
        parameters: &parameters,
        columns,
        used: BTreeSet::new(),
    };
    let mut from = targets
        .get(&app.source.occurrence)
        .ok_or_else(fail)?
        .0
        .clone();
    for (i, join) in app.joins.iter().enumerate() {
        let on = join
            .on
            .iter()
            .enumerate()
            .map(|(j, p)| build.predicate(p, &format!("joins/{i}/on/{j}"), QueryOperator::Join))
            .collect::<Result<Vec<_>>>()?;
        if on.is_empty() {
            return Err(fail());
        }
        from += &format!(
            " JOIN {} ON ({})",
            targets.get(&join.right.occurrence).ok_or_else(fail)?.0,
            on.join(") AND (")
        );
    }
    let filters = app
        .filters
        .iter()
        .enumerate()
        .map(|(i, p)| build.predicate(p, &format!("filters/{i}"), QueryOperator::Predicate))
        .collect::<Result<Vec<_>>>()?;
    let relation = format!(
        "FROM {from}{}",
        if filters.is_empty() {
            String::new()
        } else {
            format!(" WHERE ({})", filters.join(") AND ("))
        }
    );
    let grouping = app
        .groups
        .iter()
        .map(|f| build.field(&f.scan, &f.identity, Some(QueryOperator::Group)))
        .collect::<Result<Vec<_>>>()?;
    let grouped = if grouping.is_empty() {
        String::new()
    } else {
        format!(" GROUP BY {}", grouping.join(","))
    };
    let input = json!({"modules":ctx.catalog().inputs(),"policyJson":ctx.logical_plan().source().policy_json(),"ontologyJson":ctx.logical_plan().source().ontology_json(),"queryProfileJson":json!({"action":ctx.requirements().primary_action()}).to_string()});
    let mut result_columns = Vec::new();
    let mut output_sql = Vec::new();
    let mut uniform = Vec::new();
    for (i, output) in app.outputs.iter().enumerate() {
        match &output.expression {
            app::Expression::Field { scan, identity } => {
                let value = build.field(scan, identity, None)?;
                output_sql.push(if app.aggregate {
                    uniform.push(format!("pg_catalog.count(DISTINCT {value})>1"));
                    format!("pg_catalog.jsonb_agg({value})->0")
                } else {
                    value
                });
                result_columns.push(crate::security_graph_query::column(&input,&graph,&json!({"target":targets.get(scan).ok_or_else(fail)?.1,"field":reference(identity),"outputName":output.name}),i+1)?);
            }
            app::Expression::Count { logical_type } | app::Expression::Sum { logical_type, .. } => {
                let (value, source_fields) = match &output.expression {
                    app::Expression::Count { .. } => {
                        ("pg_catalog.count(*)::pg_catalog.text".into(), vec![])
                    }
                    app::Expression::Sum { argument, .. } => (
                        format!(
                            "pg_catalog.sum({})::pg_catalog.text",
                            build.field(
                                &argument.scan,
                                &argument.identity,
                                Some(QueryOperator::Aggregate)
                            )?
                        ),
                        vec![SecurityIdentity {
                            document_id: argument.identity.document_id.clone(),
                            revision: argument.identity.revision.clone(),
                            module: argument.identity.module.clone(),
                            element: argument.identity.element.clone(),
                        }],
                    ),
                    _ => return Err(fail()),
                };
                let wrapper = match logical_type.family {
                    Family::Integer => "integerToken",
                    Family::Decimal => "decimalToken",
                    _ => return Err(fail()),
                };
                output_sql.push(format!("pg_catalog.jsonb_build_object('outcomeId','computed','disposition','computed','value',CASE WHEN ({value}) IS NULL THEN 'null'::pg_catalog.jsonb ELSE pg_catalog.jsonb_build_object('{wrapper}',({value})) END)"));
                result_columns.push(SecurityResultColumn {
                    position: i + 1,
                    output_name: output.name.clone(),
                    source_fields,
                    outcomes: vec![SecurityResultOutcome::Computed {
                        id: "computed".into(),
                        domain: SecurityResultDomain::Scalar {
                            r#type: weft_core::security_aggregate_results::aggregate_result_type(ctx,output)?,
                        },
                    }],
                });
            }
            _ => return Err(fail()),
        }
    }
    let order = app
        .order
        .iter()
        .enumerate()
        .map(|(i,f)| {
            let value = build.field(&f.scan, &f.identity, Some(QueryOperator::Order))?;
            let value = if app.aggregate {
                uniform.push(format!("pg_catalog.count(DISTINCT {value})+CASE WHEN pg_catalog.bool_or(({value}) IS NULL) THEN 1 ELSE 0 END>1"));
                if f.logical_type.family==Family::Boolean{format!("pg_catalog.bool_or({value})")}else{format!("pg_catalog.min({value})")}
            } else {
                value
            };
            Ok(if ctx.query().order_descending()[i] {format!("{value} DESC NULLS FIRST")} else {value})
        })
        .collect::<Result<Vec<_>>>()?;
    if build.used.len() != parameters.query().len() {
        return Err(fail());
    }
    let prefix = format!("WITH {}", ctes.join(","));
    if app.aggregate && !uniform.is_empty() {
        checks.push(json!({"failureCode":"WFT-SECURITY-GROUP-DISCLOSURE","sql":format!("{prefix} SELECT pg_catalog.count(*)::pg_catalog.text AS violations FROM (SELECT 1 {relation}{grouped} HAVING {}) grouped_disclosure",uniform.join(" OR "))}));
    }
    let sql=format!("{prefix} SELECT pg_catalog.jsonb_build_array({})::pg_catalog.text AS cells {relation}{grouped}{}{}",output_sql.join(","),if order.is_empty(){String::new()}else{format!(" ORDER BY {}",order.join(","))},app.limit.map(|n|format!(" LIMIT {n}")).unwrap_or_default());
    let contract = SecurityResultContract {
        version: if app.aggregate {
            weft_core::security_aggregate_results::VERSION
        } else {
            "weft.security.result-contract/0.1.0"
        }
        .into(),
        encoding: if app.aggregate {
            weft_core::security_aggregate_results::ENCODING
        } else {
            "weft.security.cells/0.1.0"
        }
        .into(),
        columns: result_columns,
    };
    if app.aggregate {
        weft_core::security_aggregate_results::check_aggregate_result_declaration(ctx, &contract)?;
    } else {
        ctx.check_result_declaration(&contract)?;
    }
    let sql = anchor_sql(&sql, &anchor);
    for read in &mut operand_reads {
        let original = read["sql"].as_str().ok_or_else(fail)?;
        read["sql"] = json!(anchor_sql(original, &anchor));
    }
    for check in &mut checks {
        check["stage"]=json!(if check["failureCode"]=="WFT-SECURITY-GROUP-DISCLOSURE"{"afterOperandValidation"}else{"beforeOperandCapture"});
        let original = check["sql"].as_str().ok_or_else(fail)?;
        check["sql"] = json!(anchor_sql(original, &anchor));
    }
    if sql.len() > 4_000_000
        || serde_json::to_vec(&operand_reads).map_err(|_| fail())?.len() > 16_000_000
        || serde_json::to_vec(&checks).map_err(|_| fail())?.len() > 16_000_000
        || serde_json::to_vec(&operands).map_err(|_| fail())?.len() > 4_000_000
    {
        return Err(fail());
    }
    drop(build);
    Ok(RelationalGraphSql {
        context: ctx,
        parameters,
        sql,
        checks,
        contract,
        operands,
        operand_reads,
    })
}
