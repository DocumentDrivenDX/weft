//! Two-hop SQL assembly from original typed views. No native execution lives here.
use super::arithmetic as scalar;
use super::*;
use weft_core::{
    application_ir as app, application_model::AuthoredKey, arithmetic_plan as arithmetic,
    backend03 as b03, ir::Span,
};

const ITEM: &str = "STRUCT<intermediate:ARRAY<STRING>,terminal:ARRAY<STRING>,edges:ARRAY<STRING>>";
const MAX_COUNT: &str = "CAST('9223372036854775807' AS DECIMAL(38,0))";

struct PathSql<'a> {
    path: b03::PathView<'a>,
    bound: Option<u16>,
    rows: String,
    collection: Option<String>,
    ranked: Option<String>,
}
fn exact_string(f: &app::Field) -> bool {
    f.logical_type.family == Family::String
        && !f.logical_type.nullable
        && f.logical_type.facets == json!({})
}
fn descriptor<'a>(p: b03::Plan04View<'a>, id: &Identity) -> Result<&'a LogicalType> {
    let d = p
        .type_graph()
        .iter()
        .find(|d| &d.identity == id)
        .ok_or_else(|| fail("WFT-BINDING", "Original scalar descriptor missing"))?;
    let weft_core::application_model::Shape::Scalar { logical_type } = &d.shape else {
        return Err(fail(
            "WFT-CAPABILITY",
            "Path scalar projection requires scalar descriptor",
        ));
    };
    if logical_type.nullable
        || !matches!(
            d.availability.as_deref(),
            Some("required" | "absent-allowed")
        )
    {
        return Err(fail("WFT-CAPABILITY","Path scalar projection requires original required or explicit native-null availability"));
    }
    Ok(logical_type)
}
fn field(scan: &str, id: &Identity, ty: &LogicalType) -> app::Field {
    app::Field {
        scan: scan.into(),
        identity: id.clone(),
        logical_type: ty.clone(),
        span: Span { start: 0, end: 0 },
    }
}
fn field_sql(lower: &mut Lower<'_>, f: &app::Field) -> Result<String> {
    lower.expression(&scalar::field_expression(f))
}
fn with_sql(lower: &Lower<'_>, body: &str) -> String {
    format!("WITH {} {body}", lower.ctes.join(", "))
}
fn violations(lower: &Lower<'_>, body: &str) -> String {
    with_sql(
        lower,
        &format!(
            "SELECT CAST({} AS STRING) AS violations FROM ({body}) `_path_violations`",
            count_sql()
        ),
    )
}
fn key_projection(
    lower: &mut Lower<'_>,
    scan: &str,
    key: &AuthoredKey,
    prefix: &str,
) -> Result<Vec<String>> {
    key.fields
        .iter()
        .zip(&key.types)
        .enumerate()
        .map(|(i, (id, ty))| {
            Ok(format!(
                "{} AS {}",
                field_sql(lower, &field(scan, id, ty))?,
                binding::quote(&format!("{prefix}{i}"))
            ))
        })
        .collect()
}
fn key_columns(alias: &str, prefix: &str, key: &AuthoredKey) -> Vec<String> {
    (0..key.fields.len())
        .map(|i| {
            format!(
                "{}.{}",
                binding::quote(alias),
                binding::quote(&format!("{prefix}{i}"))
            )
        })
        .collect()
}
fn key_strings(columns: &[String]) -> String {
    format!(
        "array({})",
        columns
            .iter()
            .map(|c| format!("CAST({c} AS STRING)"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
fn path_check(index: usize, kind: &str, sql: String) -> Value {
    json!({"pathIndex":index,"kind":kind,"sql":sql,"failureCode":"WFT-BINDING"})
}

fn enumerate<'a>(
    lower: &mut Lower<'_>,
    traversals: &relationships::Traversals,
    path: b03::PathView<'a>,
    bound: Option<u16>,
    index: usize,
    checks: &mut Vec<Value>,
) -> Result<PathSql<'a>> {
    let first = &path.hops()[0];
    let second = &path.hops()[1];
    let a = traversals.raw(first)?;
    let b = traversals.raw(second)?;
    let base = format!("__weft_path_{index}");
    let mid = format!("{base}_mid");
    let terminal = format!("{base}_terminal");
    let rows = format!("{base}_rows");
    let mid_keys = key_projection(lower, a.endpoint, &first.target_key, "m")?;
    let terminal_keys = key_projection(lower, b.endpoint, &second.target_key, "t")?;
    lower.ctes.push(format!(
        "{} AS (SELECT __id, {} FROM {})",
        binding::quote(&mid),
        mid_keys.join(", "),
        binding::quote(a.endpoint)
    ));
    lower.ctes.push(format!(
        "{} AS (SELECT __id, {} FROM {})",
        binding::quote(&terminal),
        terminal_keys.join(", "),
        binding::quote(b.endpoint)
    ));
    let m = key_columns("m", "m", &first.target_key);
    let t = key_columns("t", "t", &second.target_key);
    let columns = m.iter().chain(&t).cloned().collect::<Vec<_>>().join(", ");
    lower.ctes.push(format!("{} AS (SELECT e1.{} AS __root, m.__id AS __intermediate, t.__id AS __terminal, e1.id AS __edge1, e2.id AS __edge2, {columns} FROM {} e1 JOIN {} m ON e1.{}=m.__id JOIN {} e2 ON e2.{}=m.__id JOIN {} t ON e2.{}=t.__id)",binding::quote(&rows),a.from_id,binding::quote(a.edge),binding::quote(&mid),a.to_id,binding::quote(b.edge),b.from_id,binding::quote(&terminal),b.to_id));
    // These are full scoped native edge populations, before either path join can hide corruption.
    let edge_bad=[a.edge,b.edge].into_iter().map(|e|format!("SELECT id FROM {edge} WHERE id IS NULL OR typeof(id)<>'bigint' OR CAST(CAST(id AS STRING) AS BIGINT)<>id OR NOT (CAST(id AS STRING) RLIKE '^(0|-?[1-9][0-9]*)$') UNION ALL SELECT id FROM {edge} GROUP BY id HAVING {}>1",count_sql(),edge=binding::quote(e))).collect::<Vec<_>>().join(" UNION ALL ");
    checks.push(path_check(
        index,
        "edgeEncoding",
        violations(lower, &edge_bad),
    ));
    // Each hop's endpoint integrity proves type/source/revision, not just equal local IDs.
    // The bridge is the same original Record and authored key in both directions.
    let mut bridge=format!("SELECT e1.id FROM {} e1 LEFT JOIN {} m ON e1.{}=m.__id WHERE m.__id IS NULL UNION ALL SELECT e2.id FROM {} e2 LEFT JOIN {} m ON e2.{}=m.__id WHERE m.__id IS NULL",binding::quote(a.edge),binding::quote(&mid),a.to_id,binding::quote(b.edge),binding::quote(&mid),b.from_id);
    for record in [path.start_scan(), mid.as_str(), terminal.as_str()] {
        bridge.push_str(&format!(
            " UNION ALL SELECT __id FROM {} WHERE __id IS NULL OR typeof(__id)<>'bigint'",
            binding::quote(record)
        ));
    }
    checks.push(path_check(
        index,
        "intermediateIdentity",
        violations(lower, &bridge),
    ));
    let mut ranked_name = None;
    let collection = if let Some(bound) = bound {
        let ranked = format!("{base}_ranked");
        ranked_name = Some(ranked.clone());
        let encoded = format!("{base}_collection");
        let mk = key_columns("p", "m", &first.target_key);
        let tk = key_columns("p", "t", &second.target_key);
        let mut order = mk.iter().chain(&tk).cloned().collect::<Vec<_>>();
        order.extend(["p.__edge1".into(), "p.__edge2".into()]);
        let item=format!("named_struct('intermediate',{},'terminal',{},'edges',array(CAST(p.__edge1 AS STRING),CAST(p.__edge2 AS STRING)))",key_strings(&mk),key_strings(&tk));
        // ROW_NUMBER uses a signed 32-bit counter in Spark 4.0.1. The decimal
        // prefix is independently guarded across the full population instead.
        lower.ctes.push(format!("{} AS (SELECT p.__root, {item} AS __item, TRY_SUM(CAST(1 AS DECIMAL(38,0))) OVER (PARTITION BY p.__root ORDER BY {} ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS __ordinal FROM {} p)",binding::quote(&ranked),order.join(", "),binding::quote(&rows)));
        let items=format!("transform(slice(sort_array(collect_list(named_struct('ordinal',__ordinal,'item',__item))),1,{bound}),x -> x.item)");
        lower.ctes.push(format!("{} AS (SELECT __root, {items} AS __items, MAX(__ordinal)>CAST({bound} AS DECIMAL(38,0)) AS __truncated FROM {} WHERE __ordinal<=CAST({} AS DECIMAL(38,0)) GROUP BY __root)",binding::quote(&encoded),binding::quote(&ranked),u32::from(bound)+1));
        let proof=format!("SELECT __root FROM {} WHERE __ordinal IS NULL OR NOT (from_json(to_json(__item),'{ITEM}') <=> __item) UNION ALL SELECT totals.__root FROM (SELECT __root, MAX(__ordinal) AS total FROM {} GROUP BY __root) totals LEFT JOIN {} c ON totals.__root=c.__root WHERE c.__root IS NULL OR size(c.__items)<>LEAST(totals.total,CAST({bound} AS DECIMAL(38,0))) OR c.__truncated<>(totals.total>CAST({bound} AS DECIMAL(38,0)))",binding::quote(&ranked),binding::quote(&ranked),binding::quote(&encoded));
        checks.push(path_check(
            index,
            "collectionEncoding",
            violations(lower, &proof),
        ));
        Some(encoded)
    } else {
        None
    };
    Ok(PathSql {
        path,
        bound,
        rows,
        collection,
        ranked: ranked_name,
    })
}

/// Capacity is computed from independent wide additions over the full filtered
/// bag, never from a native COUNT that may already have overflowed.
fn capacity(
    lower: &Lower<'_>,
    from: &str,
    filter: Option<&str>,
    groups: &[String],
    distinct: Option<&[String]>,
) -> String {
    let where_sql = filter.map(|f| format!(" WHERE {f}")).unwrap_or_default();
    let mut tuple = groups.to_vec();
    if let Some(keys) = distinct {
        tuple.extend_from_slice(keys)
    }
    let input = if tuple.is_empty() {
        format!("SELECT 1 AS __one FROM {from}{where_sql}")
    } else {
        format!(
            "SELECT {}{} FROM {from}{where_sql}",
            if distinct.is_some() { "DISTINCT " } else { "" },
            tuple
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{s} AS `_path_key_{i}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let group_sql = if groups.is_empty() {
        String::new()
    } else {
        format!(
            " GROUP BY {}",
            (0..groups.len())
                .map(|i| format!("`_path_key_{i}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let wide=format!("SELECT TRY_SUM(CAST(1 AS DECIMAL(38,0))) AS __n, MAX(1) AS __nonempty FROM ({input}) __path_input{group_sql}");
    violations(lower,&format!("SELECT 1 FROM ({wide}) __path_capacity WHERE (__nonempty IS NOT NULL AND __n IS NULL) OR __n>{MAX_COUNT}"))
}

pub(crate) fn lower(
    c: &b03::Context<'_>,
    binding: &Binding,
) -> Result<crate::paths::PathTargetPlan> {
    lower_profile(c, binding, false)
}

pub(crate) fn lower_keys(
    c: &b03::Context<'_>,
    binding: &Binding,
) -> Result<crate::paths::PathTargetPlan> {
    lower_profile(c, binding, true)
}

fn lower_profile(
    c: &b03::Context<'_>,
    binding: &Binding,
    keys_profile: bool,
) -> Result<crate::paths::PathTargetPlan> {
    let p = c.plan;
    let outputs = p.outputs().collect::<Vec<_>>();
    if p.groups().iter().any(|f| !exact_string(f)) {
        return Err(fail(
            "WFT-CAPABILITY",
            "Path count groups require exact required String Fields",
        ));
    }
    if p.groups()
        .iter()
        .any(|f| p.outer_join_scans().contains(&f.scan))
    {
        return Err(fail("WFT-CAPABILITY","Grouping potentially absent LEFT carriers requires a separate presence grouping profile"));
    }
    if p.order()
        .iter()
        .any(|f| p.outer_join_scans().contains(&f.scan))
    {
        return Err(fail(
            "WFT-CAPABILITY",
            "Ordering potentially absent LEFT inputs requires a separately reviewed presence order",
        ));
    }
    if p.distinct()
        && outputs.iter().any(|o| match o.expression {
            b03::ExpressionView::Legacy(arithmetic::Expression::Field { scan, identity }) => {
                p.outer_join_scans().contains(scan)
                    || descriptor(p, identity).map_or(true, |ty| {
                        ty.family != Family::String || ty.nullable || ty.facets != json!({})
                    })
                    || p.type_graph()
                        .iter()
                        .find(|d| &d.identity == identity)
                        .and_then(|d| d.availability.as_deref())
                        != Some("required")
            }
            _ => true,
        })
    {
        return Err(fail(
            "WFT-CAPABILITY",
            "DISTINCT path profile requires original required String direct Field carriers",
        ));
    }
    let mut lower = Lower::new(binding);
    lower.mathematical_profile = true;
    lower.left_scans = p.outer_join_scans().iter().cloned().collect();
    lower
        .scans
        .insert(p.source().occurrence.clone(), p.source().record.clone());
    for j in p.joins() {
        lower
            .scans
            .insert(j.right.occurrence.clone(), j.right.record.clone());
        for pred in &j.on {
            scalar::collect_predicate(&mut lower, pred)?;
        }
    }
    for pred in p.filters() {
        scalar::collect_predicate(&mut lower, pred)?;
    }
    for f in p.groups().iter().chain(p.order()) {
        scalar::collect_field(&mut lower, f);
    }
    for o in &outputs {
        if let b03::ExpressionView::Legacy(e) = o.expression {
            match e {
                arithmetic::Expression::Field { scan, identity } => scalar::collect_field(
                    &mut lower,
                    &field(scan, identity, descriptor(p, identity)?),
                ),
                arithmetic::Expression::Arithmetic { expression } => {
                    scalar::collect_numeric(&mut lower, expression)
                }
                arithmetic::Expression::CountDistinct { argument, .. } => {
                    if !exact_string(argument) {
                        return Err(fail(
                            "WFT-CAPABILITY",
                            "Legacy distinct count requires scalar String argument",
                        ));
                    }
                    scalar::collect_field(&mut lower, argument);
                }
                arithmetic::Expression::Count { .. } => {}
                arithmetic::Expression::RelatedKeys { .. } if keys_profile => {}
                _ => {
                    return Err(fail(
                        "WFT-CAPABILITY",
                        "Path profile does not admit SUM or one-hop collection composition",
                    ))
                }
            }
        }
    }
    // Missing LEFT inputs are absence, never a numeric capacity failure. This
    // initial profile refuses numeric consumption of such scans explicitly.
    for ((scan, _), (_, ty)) in &lower.fields {
        if lower.left_scans.contains(scan) && ty.family != Family::String {
            return Err(fail(
                "WFT-CAPABILITY",
                "Potentially unmatched path inputs require String scalar consumption",
            ));
        }
    }
    for ((scan, key), (id, _)) in &lower.fields {
        if p.type_graph()
            .iter()
            .any(|d| &d.identity == id && d.availability.as_deref() == Some("absent-allowed"))
        {
            lower.native_null.insert((scan.clone(), key.clone()));
        }
    }
    let mut inventory = outputs
        .iter()
        .filter_map(|o| {
            if let b03::ExpressionView::RelatedPaths { path, bound } = o.expression {
                Some((path, Some(bound)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if let Some(e) = p.expansion() {
        inventory.push((e.path, None));
    }
    let mut reads = inventory
        .iter()
        .flat_map(|(path, _)| {
            path.hops()
                .iter()
                .map(|h| (path.start_scan().to_string(), h.clone()))
        })
        .collect::<Vec<_>>();
    if keys_profile {
        for o in &outputs {
            if let b03::ExpressionView::Legacy(arithmetic::Expression::RelatedKeys {
                scan,
                relationship,
                ..
            }) = o.expression
            {
                reads.push((scan.clone(), relationship.clone()));
            }
        }
    }
    let traversals =
        relationships::Traversals::collect_reads(&mut lower, reads.iter().map(|(s, r)| (s, r)))?;
    lower.prepare()?;
    let relationship_checks = traversals.prepare(&mut lower)?;
    let mut path_checks = vec![];
    let mut paths = vec![];
    for (index, (path, bound)) in inventory.iter().enumerate() {
        paths.push(enumerate(
            &mut lower,
            &traversals,
            *path,
            *bound,
            index,
            &mut path_checks,
        )?);
    }
    let mut key_collections = BTreeMap::new();
    let mut key_inventory = vec![];
    let mut key_schemas = vec![];
    let mut key_checks = vec![];
    let mut ordinal_inventory = vec![];
    let mut ordinal_checks = vec![];
    if keys_profile {
        let mut path_index = 0;
        for (i, o) in outputs.iter().enumerate() {
            let position = i + 1;
            match o.expression {
                b03::ExpressionView::Legacy(arithmetic::Expression::RelatedKeys {
                    scan,
                    relationship,
                    bound,
                }) => {
                    let collection = super::related_keys::collect(
                        &mut lower,
                        &traversals,
                        scan,
                        relationship,
                        *bound,
                        position,
                    )?;
                    let physical = binding
                        .relationships
                        .iter()
                        .find(|r| r.logical == json!(relationship.identity))
                        .ok_or_else(|| fail("WFT-BINDING", "RelatedKeys source missing"))?;
                    key_inventory.push(json!({"outputPosition":position,"startScan":scan,"relationship":relationship,"bound":bound}));
                    key_schemas.push(json!({"outputPosition":position,"relationship":relationship.identity,"table":binding.publication.tables[physical.table],"identityColumn":"id","nativeType":"BIGINT"}));
                    key_checks.push(json!({"outputPosition":position,"kind":"collectionEncoding","sql":collection.encoding_check,"failureCode":"WFT-BINDING"}));
                    ordinal_inventory.push(json!({"outputPosition":position,"kind":"relatedKeys"}));
                    ordinal_checks.push(json!({"outputPosition":position,"kind":"fullOccurrencePrefix","sql":collection.capacity_check,"failureCode":"WFT-CAPABILITY"}));
                    key_collections.insert(position, collection);
                }
                b03::ExpressionView::RelatedPaths { .. } => {
                    let ranked = paths[path_index].ranked.as_ref().unwrap();
                    path_index += 1;
                    ordinal_inventory
                        .push(json!({"outputPosition":position,"kind":"relatedPaths"}));
                    ordinal_checks.push(json!({"outputPosition":position,"kind":"fullOccurrencePrefix","sql":violations(&lower,&format!("SELECT __root FROM {} WHERE __ordinal IS NULL",binding::quote(ranked))),"failureCode":"WFT-CAPABILITY"}));
                }
                _ => {}
            }
        }
    }
    let mut numeric_checks = vec![];
    let mut from = binding::quote(&p.source().occurrence);
    for j in p.joins() {
        let mut guards = vec![];
        let predicates =
            j.on.iter()
                .map(|pred| scalar::predicate(&mut lower, pred, &mut guards))
                .collect::<Result<Vec<_>>>()?;
        let candidates = format!(
            "({from} CROSS JOIN {})",
            binding::quote(&j.right.occurrence)
        );
        scalar::check(
            &lower,
            &mut numeric_checks,
            "join-candidates",
            &candidates,
            None,
            &guards,
        );
        from = format!(
            "({from} {} JOIN {} ON {})",
            if j.kind == Some(arithmetic::JoinKind::Left) {
                "LEFT"
            } else {
                "INNER"
            },
            binding::quote(&j.right.occurrence),
            predicates.join(" AND ")
        );
    }
    for path in &paths {
        if let Some(name) = &path.collection {
            from = format!(
                "({from} LEFT JOIN {} ON {}.__root={}.__id)",
                binding::quote(name),
                binding::quote(name),
                binding::quote(path.path.start_scan())
            );
        }
    }
    for collection in key_collections.values() {
        from = format!(
            "({from} LEFT JOIN {} ON {}.__root={}.__id)",
            binding::quote(&collection.name),
            binding::quote(&collection.name),
            binding::quote(&collection.scan)
        );
    }
    let expansion = paths.iter().find(|p| p.bound.is_none());
    if let Some(path) = expansion {
        from = format!(
            "({from} INNER JOIN {} ON {}.__root={}.__id)",
            binding::quote(&path.rows),
            binding::quote(&path.rows),
            binding::quote(path.path.start_scan())
        );
    }
    let mut guards = vec![];
    let filters = p
        .filters()
        .iter()
        .map(|pred| scalar::predicate(&mut lower, pred, &mut guards))
        .collect::<Result<Vec<_>>>()?;
    scalar::check(
        &lower,
        &mut numeric_checks,
        "where-candidates",
        &from,
        None,
        &guards,
    );
    let filter = (!filters.is_empty()).then(|| filters.join(" AND "));
    let groups = p
        .groups()
        .iter()
        .map(|f| field_sql(&mut lower, f))
        .collect::<Result<Vec<_>>>()?;
    let positioned = p
        .capabilities()
        .iter()
        .any(|s| s == "project.positionedOutputs");
    let mut columns = vec![];
    let mut projections = vec![];
    let mut guards = vec![];
    let mut count_kinds = BTreeSet::new();
    let mut collection_index = 0;
    for (i, o) in outputs.iter().enumerate() {
        let (sql, representation, ids) = match o.expression {
            b03::ExpressionView::RelatedPaths { path, bound } => {
                let encoded = paths[collection_index].collection.as_ref().unwrap();
                collection_index += 1;
                let name = binding::quote(encoded);
                let value=format!("named_struct('items',coalesce({name}.__items,CAST(array() AS ARRAY<{ITEM}>)),'truncated',coalesce({name}.__truncated,FALSE))");
                let outer = p
                    .outer_join_scans()
                    .contains(&path.start_scan().to_string())
                    .then(|| OuterJoin {
                        scan: path.start_scan().into(),
                        record: path.start_record().clone(),
                    });
                let sql = if outer.is_some() {
                    format!("CASE WHEN {}.`_weft_match_id` IS NULL THEN to_json(named_struct('state','absent')) ELSE to_json(named_struct('state','value','value',{value})) END",binding::quote(path.start_scan()))
                } else {
                    format!("to_json({value})")
                };
                let mut ids = vec![path.start_record().clone()];
                for h in path.hops() {
                    for id in [&h.from, &h.to]
                        .into_iter()
                        .chain(h.source_key.fields.iter())
                        .chain(h.target_key.fields.iter())
                    {
                        if !ids.contains(id) {
                            ids.push(id.clone());
                        }
                    }
                }
                (
                    sql,
                    b03::Representation::RelatedPaths {
                        path: path.into(),
                        start_record: path.start_record().clone(),
                        bound,
                        edge_encoding: "signed64-decimal/0.1".into(),
                        outer_join: outer,
                    },
                    ids,
                )
            }
            b03::ExpressionView::CountDistinctPathTargets { occurrence } => {
                let path = expansion
                    .ok_or_else(|| fail("WFT-EMIT", "Path target count lacks expansion"))?;
                let hop = &path.path.hops()[1];
                let keys = key_columns(&path.rows, "t", &hop.target_key);
                count_kinds.insert("targetDistinct");
                (
                    format!(
                        "CAST(COUNT(DISTINCT struct({})) AS STRING)",
                        keys.join(", ")
                    ),
                    b03::Representation::Scalar {
                        logical_type: LogicalType {
                            family: Family::Integer,
                            facets: json!({}),
                            nullable: false,
                        },
                        carrier: ScalarCarrier::Text,
                        decoder: ScalarDecoder::ExactInteger,
                        path_target: Some(b03::PathTarget {
                            path_occurrence: occurrence.into(),
                            record: hop.to.clone(),
                            key: hop.target_key.clone(),
                        }),
                    },
                    std::iter::once(hop.to.clone())
                        .chain(hop.target_key.fields.iter().cloned())
                        .collect(),
                )
            }
            b03::ExpressionView::Legacy(arithmetic::Expression::RelatedKeys {
                scan: _,
                relationship,
                bound,
            }) if keys_profile => {
                let collection = &key_collections[&(i + 1)];
                let name = binding::quote(&collection.name);
                let sql=format!("to_json(named_struct('items',coalesce({name}.__items,CAST(array() AS ARRAY<ARRAY<STRING>>)),'truncated',coalesce({name}.__truncated,FALSE)))");
                let mut ids = vec![relationship.from.clone(), relationship.to.clone()];
                for id in relationship
                    .source_key
                    .fields
                    .iter()
                    .chain(&relationship.target_key.fields)
                {
                    if !ids.contains(id) {
                        ids.push(id.clone());
                    }
                }
                (
                    sql,
                    b03::Representation::Legacy(Representation::RelatedKeys {
                        relationship: relationship.identity.clone(),
                        key: relationship.target_key.clone(),
                        bound: *bound,
                    }),
                    ids,
                )
            }
            b03::ExpressionView::Legacy(e) => {
                let (sql, ty, ids) = match e {
                    arithmetic::Expression::Field { scan, identity } => (
                        format!(
                            "CAST({} AS STRING)",
                            field_sql(
                                &mut lower,
                                &field(scan, identity, descriptor(p, identity)?)
                            )?
                        ),
                        descriptor(p, identity)?.clone(),
                        vec![identity.clone()],
                    ),
                    arithmetic::Expression::Arithmetic { expression } => {
                        let value = scalar::numeric(&mut lower, expression, &mut guards)?;
                        let ty = match expression.domain {
                            weft_core::arithmetic_resolve::Domain::Integer => LogicalType {
                                family: Family::Integer,
                                facets: json!({}),
                                nullable: false,
                            },
                            weft_core::arithmetic_resolve::Domain::Decimal { scale } => {
                                LogicalType {
                                    family: Family::Decimal,
                                    facets: json!({"scale":scale}),
                                    nullable: false,
                                }
                            }
                        };
                        (
                            scalar::decimal_text(&value),
                            ty,
                            scalar::numeric_identities(expression, &p.source().record),
                        )
                    }
                    arithmetic::Expression::Count { logical_type } => {
                        if expansion.is_some() {
                            count_kinds.insert("pathRows");
                        }
                        (
                            "CAST(COUNT(*) AS STRING)".into(),
                            logical_type.clone(),
                            vec![p.source().record.clone()],
                        )
                    }
                    arithmetic::Expression::CountDistinct {
                        argument,
                        logical_type,
                    } => (
                        format!(
                            "CAST(COUNT(DISTINCT {}) AS STRING)",
                            field_sql(&mut lower, argument)?
                        ),
                        logical_type.clone(),
                        vec![argument.identity.clone()],
                    ),
                    _ => unreachable!(),
                };
                let null = matches!(e,arithmetic::Expression::Field{scan,identity} if lower.native_null.contains(&(scan.clone(),serde_json::to_string(identity).unwrap())));
                let outer = match e {
                    arithmetic::Expression::Field { scan, .. }
                        if lower.left_scans.contains(scan) =>
                    {
                        Some(OuterJoin {
                            scan: scan.clone(),
                            record: lower.scans[scan].clone(),
                        })
                    }
                    _ => None,
                };
                let (sql, rep) = if null || outer.is_some() {
                    let value = if ty.family == Family::Boolean {
                        format!("CAST({sql} AS BOOLEAN)")
                    } else {
                        sql
                    };
                    let value = if null {
                        format!("CASE WHEN ({value}) IS NULL THEN to_json(named_struct('state','null')) ELSE to_json(named_struct('state','value','value',{value})) END")
                    } else {
                        format!("to_json(named_struct('state','value','value',{value}))")
                    };
                    let sql = if let Some(o) = &outer {
                        format!("CASE WHEN {}.`_weft_match_id` IS NULL THEN to_json(named_struct('state','absent')) ELSE {value} END",binding::quote(&o.scan))
                    } else {
                        value
                    };
                    (
                        sql,
                        b03::Representation::Legacy(Representation::Value {
                            descriptor: ids[0].clone(),
                            native_null: null,
                            outer_join: outer,
                        }),
                    )
                } else {
                    let decoder = match ty.family {
                        Family::String => ScalarDecoder::Text,
                        Family::Boolean => ScalarDecoder::Boolean,
                        Family::Integer => ScalarDecoder::ExactInteger,
                        Family::Decimal => ScalarDecoder::ExactDecimal,
                    };
                    (
                        sql,
                        b03::Representation::Scalar {
                            logical_type: ty,
                            carrier: ScalarCarrier::Text,
                            decoder,
                            path_target: None,
                        },
                    )
                };
                (sql, rep, ids)
            }
        };
        let carrier = positioned.then(|| format!("_weft_output_{}", i + 1));
        projections.push(format!(
            "{sql} AS {}",
            binding::quote(carrier.as_deref().unwrap_or(o.name))
        ));
        columns.push(b03::Column {
            position: i + 1,
            output_name: o.name.into(),
            carrier_name: carrier,
            representation,
            source_identities: ids,
            nullable: false,
        });
    }
    scalar::check(
        &lower,
        &mut numeric_checks,
        "projection-survivors",
        &from,
        filter.as_deref(),
        &guards,
    );
    let mut capacity_checks = vec![];
    if let Some(path) = expansion {
        for kind in count_kinds {
            let keys = key_columns(&path.rows, "t", &path.path.hops()[1].target_key);
            capacity_checks.push(json!({"pathOccurrence":p.expansion().unwrap().occurrence,"kind":kind,"sql":capacity(&lower,&from,filter.as_deref(),&groups,(kind=="targetDistinct").then_some(keys.as_slice())),"failureCode":"WFT-CAPABILITY"}));
        }
    }
    for o in &outputs {
        if let b03::ExpressionView::Legacy(e) = o.expression {
            match e{
        arithmetic::Expression::CountDistinct{argument,..}=>{let arg=field_sql(&mut lower,argument)?;let nonnull=format!("{}{} IS NOT NULL",filter.as_ref().map(|f|format!("({f}) AND ")).unwrap_or_default(),arg);numeric_checks.push(json!({"phase":"aggregate-candidates","sql":capacity(&lower,&from,Some(&nonnull),&groups,Some(&[arg]))}));},
        arithmetic::Expression::Count{..} if expansion.is_none()=>numeric_checks.push(json!({"phase":"aggregate-candidates","sql":capacity(&lower,&from,filter.as_deref(),&groups,None)})),
        _=>{}
    }
        }
    }
    let mut sql = format!(
        "SELECT {}{} FROM {from}",
        if p.distinct() { "DISTINCT " } else { "" },
        projections.join(", ")
    );
    if let Some(f) = &filter {
        sql.push_str(&format!(" WHERE {f}"));
    }
    if !groups.is_empty() {
        sql.push_str(&format!(" GROUP BY {}", groups.join(", ")));
    }
    let mut having = vec![];
    for h in p.having() {
        let b03::ExpressionView::Legacy(arithmetic::Expression::CountDistinct { argument, .. }) =
            h.count
        else {
            return Err(fail(
                "WFT-CAPABILITY",
                "Only existing projected distinct-count HAVING admitted",
            ));
        };
        let app::Value::Literal {
            value,
            logical_type,
            span,
        } = h.threshold
        else {
            return Err(fail(
                "WFT-CAPABILITY",
                "HAVING requires its original literal",
            ));
        };
        value
            .parse::<u64>()
            .ok()
            .filter(|v| *v <= i64::MAX as u64)
            .ok_or_else(|| {
                fail(
                    "WFT-CAPABILITY",
                    "HAVING threshold exceeds signed64 capacity",
                )
            })?;
        let slot = lower.slot(
            logical_type.clone(),
            value.clone(),
            json!({"kind":"literal","sourceSpan":span}),
        )?;
        having.push(format!(
            "COUNT(DISTINCT {})>CAST({slot} AS BIGINT)",
            field_sql(&mut lower, argument)?
        ));
    }
    if !having.is_empty() {
        sql.push_str(&format!(" HAVING {}", having.join(" AND ")));
    }
    if !p.order().is_empty() {
        let order=p.order().iter().map(|f|{if p.distinct()||!groups.is_empty(){let index=outputs.iter().position(|o|matches!(o.expression,b03::ExpressionView::Legacy(arithmetic::Expression::Field{scan,identity}) if scan==&f.scan&&identity==&f.identity)).ok_or_else(||fail("WFT-CAPABILITY","Set/group ordering requires exact projected Field"))?;let col=&columns[index];Ok(format!("{} COLLATE UTF8_BINARY ASC",binding::quote(col.carrier_name.as_deref().unwrap_or(&col.output_name))))}else{field_sql(&mut lower,f).map(|s|format!("{s} ASC"))}}).collect::<Result<Vec<_>>>()?;
        sql.push_str(&format!(" ORDER BY {}", order.join(", ")));
    }
    if let Some(limit) = p.limit() {
        sql.push_str(&format!(" LIMIT {limit}"));
    }
    let sql = with_sql(&lower, &sql);
    let mut obligations = vec![
        publication(binding),
        Obligation {
            id: "ashlar.candidate.scalarIntegrity".into(),
            owner: ObligationOwner::Host,
            failure_code: "WFT-NUMERIC-DOMAIN".into(),
            parameters: json!({"phase":"before-user-query","checks":lower.checks,"success":"one exact STRING count equal to 0 per numeric check; publicSourceOnly requires actual public UMF","parameters":"same emitted ordered slots; values never interpolated","samePublicationRequired":true,"noPartialPublication":true}),
        },
    ];
    if !numeric_checks.is_empty() {
        obligations.push(Obligation{id:"ashlar.arithmetic.exact".into(),owner:ObligationOwner::Host,failure_code:"WFT-CAPABILITY".into(),parameters:json!({"phase":"before-user-query","checks":numeric_checks,"samePublicationRequired":true,"noPartialPublication":true,"nativeRepresentation":"DECIMAL(38,0) coefficients","maxScale":18,"success":"one exact STRING count equal to 0 for every check","guardOrder":"source integrity; each pre-ON candidate bag; complete joined WHERE bag; WHERE projection survivors before ORDER/LIMIT","result":"buffer exact text; refuse unknown obligations and recheck complete original pins before release"})});
    }
    if !relationship_checks.is_empty() {
        obligations.push(Obligation{id:"ashlar.candidate.relationshipIntegrity".into(),owner:ObligationOwner::Host,failure_code:"WFT-BINDING".into(),parameters:json!({"phase":"before-user-query","checks":relationship_checks,"success":"one exact STRING count equal to 0 per check","samePublicationRequired":true,"policy":"complete authorized source and target inputs; inverse traversal cannot broaden authority","multiplicity":"parallel edges retained; min/max checked in authored orientation","lifecycle":"host verifies the authored lifecycle and projection coverage against original publication"})});
    }
    if !paths.is_empty() {
        let schemas=paths.iter().enumerate().flat_map(|(index,path)|path.path.hops().iter().enumerate().map(move |(hop,h)|{let b=binding.relationships.iter().find(|b|b.logical==json!(h.identity)).unwrap();json!({"pathIndex":index,"hop":hop,"relationship":h.identity,"table":binding.publication.tables[b.table],"identityColumn":"id","nativeType":"BIGINT"})})).collect::<Vec<_>>();
        let path_inventory = paths
            .iter()
            .map(|p| {
                let mut value =
                    json!({"path":p.path.serialized(),"edgeEncoding":"signed64-decimal/0.1"});
                if let Some(bound) = p.bound {
                    value["bound"] = json!(bound);
                }
                value
            })
            .collect::<Vec<_>>();
        obligations.push(Obligation{id:"ashlar.path.occurrenceIntegrity".into(),owner:ObligationOwner::Host,failure_code:"WFT-BINDING".into(),parameters:json!({"phase":"before-user-query","samePublicationRequired":true,"noPartialPublication":true,"paths":path_inventory,"edgeSchemas":schemas,"checks":path_checks,"success":"one exact STRING count equal to 0 per check"})});
    }
    if !key_inventory.is_empty() {
        obligations.push(Obligation{id:"ashlar.relatedKeys.collectionIntegrity".into(),owner:ObligationOwner::Host,failure_code:"WFT-BINDING".into(),parameters:json!({"phase":"before-user-query","samePublicationRequired":true,"noPartialPublication":true,"collections":key_inventory,"edgeSchemas":key_schemas,"checks":key_checks,"success":"one exact STRING count equal to 0 per check"})});
    }
    if !ordinal_inventory.is_empty() {
        obligations.push(Obligation{id:"ashlar.relatedKeys.ordinalCapacity".into(),owner:ObligationOwner::Host,failure_code:"WFT-CAPABILITY".into(),parameters:json!({"phase":"before-user-query","samePublicationRequired":true,"noPartialPublication":true,"nativeRepresentation":"decimal38","maximum":"99999999999999999999999999999999999999","collections":ordinal_inventory,"checks":ordinal_checks,"success":"one exact STRING count equal to 0 per check"})});
    }
    if !capacity_checks.is_empty() {
        obligations.push(Obligation{id:"ashlar.path.countCapacity".into(),owner:ObligationOwner::Host,failure_code:"WFT-CAPABILITY".into(),parameters:json!({"phase":"before-user-query","samePublicationRequired":true,"noPartialPublication":true,"nativeRepresentation":"signed64","checks":capacity_checks,"success":"one exact STRING count equal to 0 per check"})});
    }
    if !p.outer_join_scans().is_empty() {
        let scans = p
            .outer_join_scans()
            .iter()
            .map(|s| {
                lower
                    .match_checks
                    .iter()
                    .find(|m| m["scan"] == *s)
                    .cloned()
                    .ok_or_else(|| fail("WFT-BINDING", "Missing original LEFT match check"))
            })
            .collect::<Result<Vec<_>>>()?;
        obligations.push(Obligation{id:"outerJoin.matchIntegrity".into(),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into(),parameters:json!({"phase":"before-user-query","samePublicationRequired":true,"noPartialPublication":true,"scans":scans})});
    }
    if positioned {
        obligations.push(Obligation{id:"weft.output.positioned".into(),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into(),parameters:json!({"profile":"weft-positioned-output/0.3.0","columns":columns.iter().map(|c|json!({"position":c.position,"outputName":c.output_name,"carrierName":c.carrier_name,"sourceIdentities":c.source_identities})).collect::<Vec<_>>()})});
    }
    Ok(crate::paths::PathTargetPlan(b03::Emission {
        sql,
        parameters: lower.parameters,
        columns,
        obligations,
    }))
}
