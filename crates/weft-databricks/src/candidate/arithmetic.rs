//! Exact coefficients avoid fractional native arithmetic and its implicit rounding.
use super::*;
use weft_core::{
    application_ir as app,
    application_model::Shape,
    arithmetic_plan as plan,
    arithmetic_resolve::{Domain, Expression as Numeric, Kind},
    ir::Span,
};
const MAX_SCALE: u64 = 18;
#[derive(Clone)]
struct Coefficient {
    sql: String,
    scale: u64,
}
fn scale(domain: &Domain) -> u64 {
    match domain {
        Domain::Integer => 0,
        Domain::Decimal { scale } => *scale,
    }
}
fn field_expression(field: &app::Field) -> Expression {
    Expression::Field {
        scan: field.scan.clone(),
        identity: field.identity.clone(),
        logical_type: field.logical_type.clone(),
        span: field.span.clone(),
    }
}
fn collect_field(lower: &mut Lower<'_>, field: &app::Field) {
    collect_expression(&field_expression(field), &mut lower.fields)
}
fn collect_numeric(lower: &mut Lower<'_>, e: &Numeric) {
    match &e.kind {
        Kind::Field { field } => collect_field(lower, field),
        Kind::Negate { operand } => collect_numeric(lower, operand),
        Kind::Binary { left, right, .. } => {
            collect_numeric(lower, left);
            collect_numeric(lower, right)
        }
        Kind::Literal { .. } | Kind::Parameter { .. } => {}
    }
}
fn numeric_identities(expression: &Numeric, fallback: &Identity) -> Vec<Identity> {
    fn visit(expression: &Numeric, identities: &mut BTreeMap<String, Identity>) {
        match &expression.kind {
            Kind::Field { field } => {
                identities.insert(serde_json::to_string(&field.identity).unwrap(), field.identity.clone());
            }
            Kind::Negate { operand } => visit(operand, identities),
            Kind::Binary { left, right, .. } => {
                visit(left, identities);
                visit(right, identities);
            }
            Kind::Literal { .. } | Kind::Parameter { .. } => {}
        }
    }
    let mut identities = BTreeMap::new();
    visit(expression, &mut identities);
    if identities.is_empty() { vec![fallback.clone()] } else { identities.into_values().collect() }
}
fn collect_value(lower: &mut Lower<'_>, v: &app::Value) {
    if let app::Value::Field { field } = v {
        collect_field(lower, field)
    }
}
fn collect_predicate(lower: &mut Lower<'_>, p: &plan::Predicate) -> Result<()> {
    match p {
        plan::Predicate::StringIn{field,values}=>{collect_field(lower,field);for value in values{collect_value(lower,value)}},
        plan::Predicate::NullTest{field,..}=>collect_field(lower,field),
        plan::Predicate::NullableStringEqual{left,right}=>{collect_field(lower,left);collect_field(lower,right)},
        plan::Predicate::ScalarCompare { left, right, .. } => { collect_field(lower, left); collect_value(lower, right) }
        plan::Predicate::ArithmeticCompare { left, right, .. } | plan::Predicate::ArithmeticCompareExtended { left, right, .. } => {
            collect_numeric(lower, left);
            collect_numeric(lower, right)
        }
        plan::Predicate::Legacy { predicate } => match predicate {
            app::Predicate::Equal { left, right } => {
                collect_field(lower, left);
                collect_value(lower, right)
            }
            app::Predicate::LexicographicGreater { columns, values } => {
                for f in columns {
                    collect_field(lower, f)
                }
                for v in values {
                    collect_value(lower, v)
                }
            }
            app::Predicate::HasRelated { .. } => {
                return Err(fail(
                    "WFT-CAPABILITY",
                    "Arithmetic row profile does not lower relationship subqueries",
                ))
            }
        },
    }
    Ok(())
}
fn checked(sql: String, scale: u64, guards: &mut Vec<String>) -> Result<Coefficient> {
    if scale > MAX_SCALE {
        return Err(fail(
            "WFT-CAPABILITY",
            "Derived scale exceeds the explicit finite coefficient profile",
        ));
    }
    guards.push(format!("({sql}) IS NULL"));
    Ok(Coefficient { sql, scale })
}
fn coefficient_field(
    lower: &mut Lower<'_>,
    field: &app::Field,
    guards: &mut Vec<String>,
) -> Result<Coefficient> {
    let s = match field.logical_type.family {
        Family::Integer => 0,
        Family::Decimal => field.logical_type.facets["scale"]
            .as_u64()
            .ok_or_else(|| fail("WFT-CAPABILITY", "Original decimal scale is required"))?,
        _ => return Err(fail("WFT-CAPABILITY", "Coefficient operand is not numeric")),
    };
    let original = lower.expression(&field_expression(field))?;
    checked(
        format!("try_cast(regexp_replace(CAST({original} AS STRING), '[.]', '') AS DECIMAL(38,0))"),
        s,
        guards,
    )
}
fn token_slot(
    lower: &mut Lower<'_>,
    token: &str,
    ty: LogicalType,
    origin: Value,
    guards: &mut Vec<String>,
) -> Result<Coefficient> {
    let unsigned = token.strip_prefix('-').unwrap_or(token);
    let parts = unsigned.split('.').collect::<Vec<_>>();
    if token.len() > 1024
        || parts.is_empty()
        || parts.len() > 2
        || parts
            .iter()
            .any(|p| p.is_empty() || !p.bytes().all(|c| c.is_ascii_digit()))
    {
        return Err(fail(
            "WFT-CAPABILITY",
            "Coefficient literal requires an exact fixed-base-ten token",
        ));
    }
    let s = parts.get(1).map_or(0, |p| p.len() as u64);
    let digits = parts.join("");
    if digits.trim_start_matches('0').len() > 38 || s > MAX_SCALE {
        return Err(fail(
            "WFT-CAPABILITY",
            "Exact token exceeds finite native coefficient capacity",
        ));
    }
    let slot = lower.slot(ty, token.into(), origin)?;
    checked(
        format!("try_cast(regexp_replace(CAST({slot} AS STRING), '[.]', '') AS DECIMAL(38,0))"),
        s,
        guards,
    )
}
fn align(value: Coefficient, target: u64, guards: &mut Vec<String>) -> Result<Coefficient> {
    if value.scale == target {
        return Ok(value);
    }
    let delta = target
        .checked_sub(value.scale)
        .ok_or_else(|| fail("WFT-EMIT", "Coefficient alignment would lose scale"))?;
    if target > MAX_SCALE {
        return Err(fail(
            "WFT-CAPABILITY",
            "Comparison scale exceeds native coefficient profile",
        ));
    }
    let factor = format!("1{}", "0".repeat(delta as usize));
    checked(
        format!(
            "try_multiply({}, CAST('{factor}' AS DECIMAL(38,0)))",
            value.sql
        ),
        target,
        guards,
    )
}
fn numeric(lower: &mut Lower<'_>, e: &Numeric, guards: &mut Vec<String>) -> Result<Coefficient> {
    let result = match &e.kind {
        Kind::Field { field } => coefficient_field(lower, field, guards)?,
        Kind::Literal { value: token } => {
            let ty = match &e.domain {
                Domain::Integer => LogicalType {
                    family: Family::Integer,
                    facets: json!({}),
                    nullable: false,
                },
                Domain::Decimal { scale } => LogicalType {
                    family: Family::Decimal,
                    facets: json!({"scale":scale}),
                    nullable: false,
                },
            };
            token_slot(
                lower,
                token,
                ty,
                json!({"kind":"literal","sourceSpan":e.span}),
                guards,
            )?
        }
        Kind::Parameter { name, value: token } => {
            let ty = match &e.domain {
                Domain::Integer => LogicalType {
                    family: Family::Integer,
                    facets: json!({}),
                    nullable: false,
                },
                Domain::Decimal { scale } => LogicalType {
                    family: Family::Decimal,
                    facets: json!({"scale":scale}),
                    nullable: false,
                },
            };
            token_slot(
                lower,
                token,
                ty,
                json!({"kind":"namedParameter","name":name,"sourceSpan":e.span}),
                guards,
            )?
        }
        Kind::Negate { operand } => {
            let v = numeric(lower, operand, guards)?;
            checked(
                format!("try_subtract(CAST(0 AS DECIMAL(38,0)), {})", v.sql),
                v.scale,
                guards,
            )?
        }
        Kind::Binary {
            operator,
            left,
            right,
        } => {
            let a = numeric(lower, left, guards)?;
            let b = numeric(lower, right, guards)?;
            match *operator {
                '*' => {
                    let s = a
                        .scale
                        .checked_add(b.scale)
                        .ok_or_else(|| fail("WFT-CAPABILITY", "Scale overflow"))?;
                    checked(format!("try_multiply({}, {})", a.sql, b.sql), s, guards)?
                }
                '+' | '-' => {
                    let s = a.scale.max(b.scale);
                    let a = align(a, s, guards)?;
                    let b = align(b, s, guards)?;
                    checked(
                        format!(
                            "{}({}, {})",
                            if *operator == '+' {
                                "try_add"
                            } else {
                                "try_subtract"
                            },
                            a.sql,
                            b.sql
                        ),
                        s,
                        guards,
                    )?
                }
                _ => return Err(fail("WFT-CAPABILITY", "Unknown arithmetic operator")),
            }
        }
    };
    if result.scale != scale(&e.domain) {
        return Err(fail(
            "WFT-EMIT",
            "Native coefficient changes derived domain",
        ));
    }
    Ok(result)
}
fn numeric_value(
    lower: &mut Lower<'_>,
    value: &app::Value,
    guards: &mut Vec<String>,
) -> Result<Coefficient> {
    match value {
        app::Value::Field { field } => coefficient_field(lower, field, guards),
        app::Value::Literal {
            value,
            logical_type,
            span,
        } => token_slot(
            lower,
            value,
            logical_type.clone(),
            json!({"kind":"literal","sourceSpan":span}),
            guards,
        ),
        app::Value::Parameter {
            name,
            value,
            logical_type,
            span,
        } => token_slot(
            lower,
            value,
            logical_type.clone(),
            json!({"kind":"namedParameter","name":name,"sourceSpan":span}),
            guards,
        ),
    }
}
fn pair(
    lower: &mut Lower<'_>,
    field: &app::Field,
    value: &app::Value,
    guards: &mut Vec<String>,
) -> Result<(String, String)> {
    if matches!(field.logical_type.family, Family::Integer | Family::Decimal) {
        let a = coefficient_field(lower, field, guards)?;
        let b = numeric_value(lower, value, guards)?;
        let s = a.scale.max(b.scale);
        Ok((align(a, s, guards)?.sql, align(b, s, guards)?.sql))
    } else {
        Ok((
            lower.expression(&field_expression(field))?,
            super::application::value(lower, value)?,
        ))
    }
}
fn predicate(
    lower: &mut Lower<'_>,
    p: &plan::Predicate,
    guards: &mut Vec<String>,
) -> Result<String> {
    match p {
        plan::Predicate::StringIn{field,values}=>{
            let left=lower.expression(&field_expression(field))?;
            let right=values.iter().map(|v|application::value(lower,v)).collect::<Result<Vec<_>>>()?;
            Ok(format!("({left} IN ({}))",right.join(", ")))
        },
        plan::Predicate::NullTest{field,negated}=>Ok(format!("({} IS {}NULL)",lower.expression(&field_expression(field))?,if *negated {"NOT "}else{""})),
        plan::Predicate::NullableStringEqual{left,right}=>Ok(format!("({} = {})",lower.expression(&field_expression(left))?,lower.expression(&field_expression(right))?)),

        plan::Predicate::ScalarCompare { left, right, operator } => {
            let (a,b) = pair(lower,left,right,guards)?;
            Ok(format!("({a} {} {b})",operator.sql()))
        }
        plan::Predicate::ArithmeticCompareExtended { left, right, operator } => {
            let a = numeric(lower,left,guards)?;
            let b = numeric(lower,right,guards)?;
            let s = a.scale.max(b.scale);
            Ok(format!("({} {} {})",align(a,s,guards)?.sql,operator.sql(),align(b,s,guards)?.sql))
        }
        plan::Predicate::ArithmeticCompare {
            left,
            right,
            greater,
        } => {
            let a = numeric(lower, left, guards)?;
            let b = numeric(lower, right, guards)?;
            let s = a.scale.max(b.scale);
            Ok(format!(
                "({} {} {})",
                align(a, s, guards)?.sql,
                if *greater { ">" } else { "=" },
                align(b, s, guards)?.sql
            ))
        }
        plan::Predicate::Legacy { predicate } => match predicate {
            app::Predicate::Equal { left, right } => {
                let (a, b) = pair(lower, left, right, guards)?;
                Ok(format!("({a} = {b})"))
            }
            app::Predicate::LexicographicGreater { columns, values } => {
                if columns.is_empty() || columns.len() != values.len() {
                    return Err(fail("WFT-EMIT", "Invalid cursor tuple"));
                }
                let pairs = columns
                    .iter()
                    .zip(values)
                    .map(|(f, v)| pair(lower, f, v, guards))
                    .collect::<Result<Vec<_>>>()?;
                let mut alternatives = vec![];
                for i in 0..pairs.len() {
                    let mut terms = pairs[..i]
                        .iter()
                        .map(|(a, b)| format!("({a} = {b})"))
                        .collect::<Vec<_>>();
                    terms.push(format!("({} > {})", pairs[i].0, pairs[i].1));
                    alternatives.push(format!("({})", terms.join(" AND ")))
                }
                Ok(format!("({})", alternatives.join(" OR ")))
            }
            app::Predicate::HasRelated { .. } => Err(fail(
                "WFT-CAPABILITY",
                "Relationship predicate requires a separately admitted profile",
            )),
        },
    }
}
fn check(
    lower: &Lower<'_>,
    checks: &mut Vec<Value>,
    phase: &str,
    from: &str,
    filter: Option<&str>,
    guards: &[String],
) {
    if guards.is_empty() {
        return;
    }
    let condition = format!("({})", guards.join(" OR "));
    let filter = filter.map_or(condition.clone(), |f| format!("({f}) AND {condition}"));
    checks.push(json!({"phase":phase,"sql":format!("WITH {} SELECT CAST(COUNT(*) AS STRING) AS violations FROM {from} WHERE {filter}",lower.ctes.join(", "))}));
}
fn decimal_text(value: &Coefficient) -> String {
    if value.scale == 0 {
        return format!("CAST({} AS STRING)", value.sql);
    }
    let s = value.scale;
    let digits = format!(
        "lpad(CAST(abs({}) AS STRING), greatest(length(CAST(abs({}) AS STRING)), {}), '0')",
        value.sql,
        value.sql,
        s + 1
    );
    format!("concat(CASE WHEN {} < CAST(0 AS DECIMAL(38,0)) THEN '-' ELSE '' END, substring({digits}, 1, length({digits})-{s}), '.', right({digits}, {s}))",value.sql)
}
fn descriptor<'a>(p: &'a plan::Plan, identity: &Identity) -> Result<&'a LogicalType> {
    let d = p
        .type_graph
        .iter()
        .find(|d| &d.identity == identity)
        .ok_or_else(|| fail("WFT-BINDING", "Original output descriptor is absent"))?;
    let Shape::Scalar { logical_type } = &d.shape else {
        return Err(fail(
            "WFT-CAPABILITY",
            "Arithmetic row profile requires scalar outputs",
        ));
    };
    if logical_type.nullable || !matches!(d.availability.as_deref(),Some("required"|"absent-allowed")) {
        return Err(fail(
            "WFT-CAPABILITY",
            "Arithmetic row profile requires present non-null fields",
        ));
    }
    Ok(logical_type)
}
pub(super) fn lower(
    _context: &Context<'_>,
    binding: &Binding,
    p: &plan::Plan,
) -> Result<TargetPlan> {
    if (!p.having.is_empty() || p.required_capabilities.iter().any(|c|c=="aggregate.countDistinct.optional")) && _context.target.id!=crate::count_having::PROFILE {
        return Err(fail("WFT-CAPABILITY","Optional counts/HAVING require the exact separate target profile"));
    }
    if p.aggregate && !p.outputs.iter().any(|o|matches!(o.expression,plan::Expression::CountDistinct{..})) {
        return Err(fail(
            "WFT-CAPABILITY",
            "Grouped arithmetic requires a separately admitted lowering",
        ));
    }
    if p.aggregate {
        if p.read_profile.is_some() || p.distinct || p.groups.iter().any(|f|f.logical_type.family!=Family::String || f.logical_type.nullable || f.logical_type.facets!=json!({})) || p.outputs.iter().any(|o|!matches!(o.expression,plan::Expression::Field{..}|plan::Expression::CountDistinct{..})) {
            return Err(fail("WFT-CAPABILITY","Distinct-count profile requires exact required String groups and direct grouped outputs only"));
        }
    }
    let positioned = p.required_capabilities.iter().any(|c| c == "project.positionedOutputs");
    let mut lower = Lower::new(binding);
    lower.mathematical_profile = true;
    lower
        .scans
        .insert(p.source.occurrence.clone(), p.source.record.clone());
    for j in &p.joins {
        lower
            .scans
            .insert(j.right.occurrence.clone(), j.right.record.clone());
        for pred in &j.on {
            collect_predicate(&mut lower, pred)?
        }
    }
    for pred in &p.filters {
        collect_predicate(&mut lower, pred)?
    }
    for f in p.groups.iter().chain(&p.order) {
        collect_field(&mut lower, f)
    }
    if let Some(key) = &p.page_key {
        for identity in &key.fields {
            let ty = descriptor(p, identity)?;
            collect_field(
                &mut lower,
                &app::Field {
                    scan: p.source.occurrence.clone(),
                    identity: identity.clone(),
                    logical_type: ty.clone(),
                    span: Span { start: 0, end: 0 },
                },
            )
        }
    }
    for output in &p.outputs {
        match &output.expression {
            plan::Expression::Field { scan, identity } => {
                let ty = descriptor(p, identity)?;
                collect_field(
                    &mut lower,
                    &app::Field {
                        scan: scan.clone(),
                        identity: identity.clone(),
                        logical_type: ty.clone(),
                        span: Span { start: 0, end: 0 },
                    },
                )
            }
            plan::Expression::CountDistinct { argument,.. } => collect_field(&mut lower,argument),
            plan::Expression::Arithmetic { expression } => collect_numeric(&mut lower, expression),
            _ => return Err(fail(
                "WFT-CAPABILITY",
                "Arithmetic row profile does not lower aggregates or relationship output envelopes",
            )),
        }
    }
    for ((scan,key),(identity,_)) in &lower.fields {
        if p.type_graph.iter().any(|d|&d.identity==identity && d.availability.as_deref()==Some("absent-allowed")) {lower.native_null.insert((scan.clone(),key.clone()));}
    }
    lower.prepare()?;
    let mut checks = vec![];
    let mut from = binding::quote(&p.source.occurrence);
    for j in &p.joins {
        let mut guards = vec![];
        let predicates =
            j.on.iter()
                .map(|pred| predicate(&mut lower, pred, &mut guards))
                .collect::<Result<Vec<_>>>()?;
        let candidates = format!(
            "({from} CROSS JOIN {})",
            binding::quote(&j.right.occurrence)
        );
        check(
            &lower,
            &mut checks,
            "join-candidates",
            &candidates,
            None,
            &guards,
        );
        from = format!(
            "({from} INNER JOIN {} ON {})",
            binding::quote(&j.right.occurrence),
            predicates.join(" AND ")
        )
    }
    let mut guards = vec![];
    let filters = p
        .filters
        .iter()
        .map(|pred| predicate(&mut lower, pred, &mut guards))
        .collect::<Result<Vec<_>>>()?;
    check(
        &lower,
        &mut checks,
        "where-candidates",
        &from,
        None,
        &guards,
    );
    let filter = if filters.is_empty() {
        None
    } else {
        Some(filters.join(" AND "))
    };
    let mut projections = vec![];
    let mut columns = vec![];
    let mut guards = vec![];
    for (index, output) in p.outputs.iter().enumerate() {
        let (sql, ty, ids) = match &output.expression {
            plan::Expression::Field { scan, identity } => {
                let ty = descriptor(p, identity)?.clone();
                let field = app::Field {
                    scan: scan.clone(),
                    identity: identity.clone(),
                    logical_type: ty.clone(),
                    span: Span { start: 0, end: 0 },
                };
                (
                    format!(
                        "CAST({} AS STRING)",
                        lower.expression(&field_expression(&field))?
                    ),
                    ty,
                    vec![identity.clone()],
                )
            }
            plan::Expression::CountDistinct{argument,logical_type}=>{
                let arg=lower.expression(&field_expression(argument))?;
                (format!("CAST(COUNT(DISTINCT {arg}) AS STRING)"),logical_type.clone(),vec![argument.identity.clone()])
            }
            plan::Expression::Arithmetic { expression } => {
                let coefficient = numeric(&mut lower, expression, &mut guards)?;
                let ty = match expression.domain {
                    Domain::Integer => LogicalType {
                        family: Family::Integer,
                        facets: json!({}),
                        nullable: false,
                    },
                    Domain::Decimal { scale } => LogicalType {
                        family: Family::Decimal,
                        facets: json!({"scale":scale}),
                        nullable: false,
                    },
                };
                (
                    decimal_text(&coefficient),
                    ty,
                    numeric_identities(expression, &p.source.record),
                )
            }
            _ => unreachable!(),
        };
        let native_null=match &output.expression {plan::Expression::Field{scan,identity}=>lower.native_null.contains(&(scan.clone(),serde_json::to_string(identity).unwrap())),_=>false};
        let sql=if native_null {
            let scalar=if ty.family==Family::Boolean {format!("CAST({sql} AS BOOLEAN)")}else{sql};
            format!("CASE WHEN ({scalar}) IS NULL THEN to_json(named_struct('state','null')) ELSE to_json(named_struct('state','value','value',{scalar})) END")
        }else{sql};
        let carrier_name = positioned.then(||format!("_weft_output_{}", index+1));
        projections.push(format!("{sql} AS {}", binding::quote(carrier_name.as_deref().unwrap_or(&output.name))));
        let decoder = match ty.family {
            Family::String => ScalarDecoder::Text,
            Family::Boolean => ScalarDecoder::Boolean,
            Family::Integer => ScalarDecoder::ExactInteger,
            Family::Decimal => ScalarDecoder::ExactDecimal,
        };
        columns.push(Column {
            position: index + 1,
            carrier_name, output_name: output.name.clone(),
            representation: if native_null {Representation::Value{descriptor:ids[0].clone(),native_null:true}}else{Representation::Scalar {
                logical_type: ty,
                carrier: ScalarCarrier::Text,
                decoder,
            }},
            source_identities: ids,
            nullable: false,
        })
    }
    check(
        &lower,
        &mut checks,
        "projection-survivors",
        &from,
        filter.as_deref(),
        &guards,
    );
    let grouping=p.groups.iter().map(|f|lower.expression(&field_expression(f))).collect::<Result<Vec<_>>>()?;
    for output in &p.outputs {
        if let plan::Expression::CountDistinct{argument,..}=&output.expression {
            let arg=lower.expression(&field_expression(argument))?;
            let mut tuple=grouping.clone();tuple.push(arg.clone());
            let where_sql=if lower.native_null.contains(&(argument.scan.clone(),serde_json::to_string(&argument.identity).unwrap())) {
                format!(" WHERE {}{arg} IS NOT NULL",filter.as_ref().map(|f|format!("({f}) AND ")).unwrap_or_default())
            } else {filter.as_ref().map(|f|format!(" WHERE {f}")).unwrap_or_default()};
            let distinct=format!("SELECT DISTINCT {} FROM {from}{where_sql}",tuple.iter().enumerate().map(|(i,e)|format!("{e} AS `_count_key_{i}`")).collect::<Vec<_>>().join(", "));
            let group_aliases=(0..grouping.len()).map(|i|format!("`_count_key_{i}`")).collect::<Vec<_>>();
            let group_sql=if group_aliases.is_empty(){String::new()}else{format!(" GROUP BY {}",group_aliases.join(", "))};
            let counts=format!("SELECT TRY_SUM(CAST(1 AS DECIMAL(38,0))) AS `_cardinality`, MAX(1) AS `_nonempty` FROM ({distinct}) `_distinct_count_input`{group_sql}");
            checks.push(json!({"phase":"aggregate-candidates","sql":format!("WITH {} SELECT CAST(COUNT(*) AS STRING) AS violations FROM ({counts}) `_count_capacity` WHERE (`_nonempty` IS NOT NULL AND `_cardinality` IS NULL) OR `_cardinality` > CAST('9223372036854775807' AS DECIMAL(38,0))",lower.ctes.join(", "))}));
        }
    }
    let mut sql = format!(
        "WITH {} SELECT {}{} FROM {from}",
        lower.ctes.join(", "),
        if p.distinct { "DISTINCT " } else { "" },
        projections.join(", ")
    );
    if let Some(filter) = filter {
        sql.push_str(&format!(" WHERE {filter}"))
    }
    if !grouping.is_empty(){sql.push_str(&format!(" GROUP BY {}",grouping.join(", ")))}
    if !p.having.is_empty() {
        let mut predicates=Vec::new();
        for h in &p.having {
            let plan::Expression::CountDistinct{argument,..}=&h.count else {return Err(fail("WFT-CAPABILITY","Only exact projected-count HAVING admitted"));};
            let app::Value::Literal{value,logical_type,span}=&h.threshold else {return Err(fail("WFT-CAPABILITY","Only original literal HAVING thresholds admitted"));};
            let n=value.parse::<u64>().ok().filter(|n|*n<=9223372036854775807).ok_or_else(||fail("WFT-CAPABILITY","Exact HAVING threshold exceeds finite native signed64 representation"))?;
            let _=n;
            let slot=lower.slot(logical_type.clone(),value.clone(),json!({"kind":"literal","sourceSpan":span}))?;
            let arg=lower.expression(&field_expression(argument))?;
            predicates.push(format!("COUNT(DISTINCT {arg}) > CAST({slot} AS BIGINT)"));
        }
        sql.push_str(&format!(" HAVING {}",predicates.join(" AND ")));
    }
    if !p.order.is_empty() {
        let order = p
            .order
            .iter()
            .map(|f| {
                if p.distinct || (_context.target.id == crate::count_having::PROFILE && !p.groups.is_empty()) {
                    // DISTINCT and this grouped count profile expose only projected carriers.
                    // Match original scan/Field identity; repeated positions use the first.
                    let index = p.outputs.iter().position(|o| matches!(&o.expression,
                        plan::Expression::Field { scan, identity } if scan == &f.scan && identity == &f.identity))
                        .ok_or_else(|| fail("WFT-CAPABILITY", if p.distinct { "DISTINCT ordering requires an exact projected Field" } else { "Count-group ordering requires an exact projected Field" }))?;
                    let column = &columns[index];
                    let alias=binding::quote(column.carrier_name.as_deref().unwrap_or(&column.output_name));
                    Ok(if p.distinct {format!("{alias} ASC")} else {format!("{alias} COLLATE UTF8_BINARY ASC")})
                } else {
                    lower.expression(&field_expression(f)).map(|e| format!("{e} ASC"))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        sql.push_str(&format!(" ORDER BY {}", order.join(", ")))
    }
    if let Some(limit) = p.limit {
        sql.push_str(&format!(" LIMIT {limit}"))
    }
    let integrity = Obligation {
        id: "ashlar.candidate.scalarIntegrity".into(),
        parameters: json!({"phase":"before-user-query","checks":lower.checks,"success":"one exact STRING count equal to 0 per numeric check; publicSourceOnly requires actual public UMF","parameters":"same emitted ordered slots; values never interpolated","samePublicationRequired":true,"noPartialPublication":true}),
        owner: ObligationOwner::Host,
        failure_code: "WFT-NUMERIC-DOMAIN".into(),
    };
    let exact = Obligation {
        id: "ashlar.arithmetic.exact".into(),
        parameters: json!({"phase":"before-user-query","checks":checks,"samePublicationRequired":true,"noPartialPublication":true,"nativeRepresentation":"DECIMAL(38,0) coefficients","maxScale":MAX_SCALE,"success":"one exact STRING count equal to 0 for every check","guardOrder":"source integrity; each pre-ON candidate bag; complete joined WHERE bag; WHERE projection survivors before ORDER/LIMIT","result":"buffer exact text; refuse unknown obligations and recheck complete original pins before release"}),
        owner: ObligationOwner::Host,
        failure_code: "WFT-CAPABILITY".into(),
    };
    let mut obligations=vec![integrity, exact, publication(binding)];
    if positioned {
        obligations.push(Obligation{id:"weft.output.positioned".into(),parameters:json!({"profile":"weft-positioned-output/0.3.0","columns":columns.iter().map(|c|json!({"position":c.position,"outputName":c.output_name,"carrierName":c.carrier_name,"sourceIdentities":c.source_identities})).collect::<Vec<_>>(),"decoding":"exact ordered row arrays; complete output count/order and unique physical names; no logical-name dictionary","lineage":"each ordinal binds the original logicalPlan output, including scan occurrence","host":"explicit opt-in before SQL; reject unknown carrierName/obligations; buffer and preserve all cells"}),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into()});
    }
    Ok(TargetPlan(Emission {sql,parameters:lower.parameters,columns,obligations}))
}
