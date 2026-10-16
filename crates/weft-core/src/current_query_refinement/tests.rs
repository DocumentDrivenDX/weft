use super::*;
use crate::{application_lineage, current_query_designation, current_query_operands};
use serde_json::Value;
fn source(sum: bool) -> Value {
    serde_json::from_str(if sum{
 include_str!("../../../../tests/current-query-designation/fixtures/live-original-integer-sum.request.json")
}else{include_str!("../../../../tests/current-query-designation/fixtures/mixed-original-uses.request.json")}).unwrap()
}
fn id(element: &str) -> Identity {
    Identity {
        document_id: "domain".into(),
        revision: "query-value-core-r1".into(),
        module: "m".into(),
        element: element.into(),
    }
}
fn spec(context: &CurrentQueryOperands<'_>) -> Vec<Selection> {
    let mut specs = Vec::new();
    for op in context.operations() {
        for operand in op.operands() {
            if let Input::Field {
                scan,
                identity,
                original_action,
                ..
            } = operand.input()
            {
                specs.push(Selection {
                    path: operand.path().into(),
                    scan: (*scan).into(),
                    field: (*identity).clone(),
                    domain: (*identity).clone(),
                    column: identity.element.clone(),
                    logical_type: domain(context, identity),
                    comparison_scope: "a".repeat(64),
                    mode: Some(
                        if original_action.is_some() {
                            "original-authorized"
                        } else {
                            "disclosed"
                        }
                        .into(),
                    ),
                    original_action: original_action.map(str::to_owned),
                    exposure: None,
                });
            }
        }
    }
    for (i, o) in context
        .designation()
        .preparation()
        .plan()
        .outputs
        .iter()
        .enumerate()
    {
        if let app::Expression::Field { scan, identity } = &o.expression {
            let salary = identity.element == "salary";
            let d = if salary {
                id("salaryMaskLabel")
            } else {
                identity.clone()
            };
            specs.push(Selection {
                path: format!("outputs/{i}/expression"),
                scan: scan.clone(),
                field: identity.clone(),
                domain: d.clone(),
                column: if salary {
                    "salary_public".into()
                } else {
                    identity.element.clone()
                },
                logical_type: domain(context, &d),
                comparison_scope: "a".repeat(64),
                mode: None,
                original_action: None,
                exposure: Some(if salary { "transformed" } else { "original" }.into()),
            });
        }
    }
    specs
}
fn domain(context: &CurrentQueryOperands<'_>, id: &Identity) -> LogicalType {
    let Shape::Scalar { logical_type } = context
        .designation()
        .preparation()
        .catalog()
        .scalar_descriptor_by_identity(id)
        .unwrap()
        .shape
    else {
        panic!()
    };
    logical_type
}
fn bag(keys: &[&[&str]]) -> String {
    json!({"version":"weft.stage-contributor-bag/0.1-candidate","tuples":keys.iter().map(|t|t.iter().enumerate().map(|(i,k)|json!({"scan":format!("s{i}"),"key":k})).collect::<Vec<_>>()).collect::<Vec<_>>()} ).to_string()
}
fn row(public: &[Option<&str>], keys: &[&[&str]]) -> Value {
    let mut row = public
        .iter()
        .map(|v| v.map_or(Value::Null, |s| json!(s)))
        .collect::<Vec<_>>();
    row.push(json!(bag(keys)));
    json!(row)
}
fn run(
    sql: &str,
    sum: bool,
    result: Value,
    edit: impl FnOnce(&mut Value, &mut Vec<Selection>),
) -> Result<()> {
    run_variant(sql, sum, result, Encoding::Contributors, edit, |s| s)
}
fn run_variant(
    sql: &str,
    sum: bool,
    result: Value,
    encoding: Encoding,
    edit: impl FnOnce(&mut Value, &mut Vec<Selection>),
    raw_edit: impl FnOnce(String) -> String,
) -> Result<()> {
    let mut request = source(sum);
    request["sql"] = json!(sql);
    request["parameters"] = json!({});
    let prepared = application_lineage::prepare(&request.to_string())?;
    let designation = current_query_designation::derive(&prepared)?;
    let context = current_query_operands::derive(&designation)?;
    let mut selections = spec(&context);
    let types = prepared
        .plan()
        .outputs
        .iter()
        .enumerate()
        .map(|(i, o)| match &o.expression {
            app::Expression::Field { scan, .. } => selections
                .iter()
                .find(|s| s.path == format!("outputs/{i}/expression") && s.scan == *scan)
                .unwrap()
                .logical_type
                .clone(),
            app::Expression::Count { logical_type }
            | app::Expression::CountDistinct { logical_type, .. }
            | app::Expression::Sum { logical_type, .. } => logical_type.clone(),
            _ => panic!(),
        })
        .collect::<Vec<_>>();
    let scans=std::iter::once(&prepared.plan().source).chain(prepared.plan().joins.iter().map(|j|&j.right)).map(|scan|{
  let columns:BTreeMap<_,_>=selections.iter().filter(|s|s.scan==scan.occurrence).map(|s|(s.column.clone(),s.logical_type.clone())).collect();
  let rows=[("k1","r1",i64::MAX,"a"),("k3","r3",i64::MAX,"z"),("k4","r4",i64::MIN,"a"),("k5","r5",i64::MIN,"z")].iter().map(|(key,rid,salary,token)|{
   let cells=columns.iter().map(|(column,t)|{let value=match column.as_str(){"salary"=>salary.to_string(),"salary_public"=>"[private]".into(),"resourceId"=>(*rid).into(),"token"=>(*token).into(),_=>panic!()};
    (column.clone(),json!({"family":if t.family==Family::Integer{"integer"}else{"string"},"value":value}))}).collect::<serde_json::Map<_,_>>();
   json!({"key":key,"cells":cells})}).collect::<Vec<_>>();json!({"scan":scan.occurrence,"expectedRowCount":"4","rows":rows})
 }).collect::<Vec<_>>();
    let binding = "b".repeat(64);
    let artifact = "c".repeat(64);
    let mut frame = json!({"version":"weft.current-query.refinement-input/0.1-candidate","sourceRequestSha256":sha256(prepared.source_request_json().as_bytes()),"designationSha256":sha256(designation.json().as_bytes()),"materializationBindingSha256":binding,"compiledArtifactSha256":artifact,"scans":scans,"resultRows":result});
    edit(&mut frame, &mut selections);
    compare(
        &context,
        Lowered {
            binding_sha256: &binding,
            artifact_sha256: &artifact,
            selections: &selections,
            output_types: &types,
            encoding,
        },
        &raw_edit(frame.to_string()),
    )
}
fn one() -> Vec<&'static [&'static str]> {
    vec![&["k1"], &["k3"], &["k4"], &["k5"]]
}
#[test]
fn count_distinct_and_global_sum_use_all_original_contributors() {
    assert!(run(
        "SELECT COUNT(*) AS n FROM Resource r",
        false,
        json!([row(&[Some("4")], &one())]),
        |_, _| {}
    )
    .is_ok());
    assert!(run(
        "SELECT COUNT(DISTINCT r.token) AS n FROM Resource r",
        false,
        json!([row(&[Some("2")], &one())]),
        |_, _| {}
    )
    .is_ok());
    assert!(run(
        "SELECT SUM(r.salary) AS n FROM Resource r",
        true,
        json!([row(&[Some("-2")], &one())]),
        |_, _| {}
    )
    .is_ok());
    assert!(run(
        "SELECT SUM(r.salary) AS n FROM Resource r WHERE r.salary=9223372036854775807",
        true,
        json!([row(&[Some("18446744073709551614")], &[&["k1"], &["k3"]])]),
        |_, _| {}
    )
    .is_ok());
    assert!(run(
        "SELECT SUM(r.salary) AS n FROM Resource r WHERE r.salary=-9223372036854775808",
        true,
        json!([row(&[Some("-18446744073709551616")], &[&["k4"], &["k5"]])]),
        |_, _| {}
    )
    .is_ok());
}
#[test]
fn grouped_and_global_empty_results_have_distinct_null_and_bag_semantics() {
    assert!(run(
        "SELECT SUM(r.salary),COUNT(*) FROM Resource r WHERE FALSE",
        true,
        json!([row(&[None, Some("0")], &[])]),
        |_, _| {}
    )
    .is_ok());
    assert!(run(
        "SELECT r.token,SUM(r.salary) FROM Resource r WHERE FALSE GROUP BY r.token",
        true,
        json!([]),
        |_, _| {}
    )
    .is_ok());
    assert!(run(
        "SELECT COUNT(*) FROM Resource r WHERE FALSE",
        false,
        json!([]),
        |_, _| {}
    )
    .is_err());
    assert!(run(
        "SELECT SUM(r.salary) FROM Resource r WHERE FALSE",
        true,
        json!([row(&[Some("0")], &[])]),
        |_, _| {}
    )
    .is_err());
}
#[test]
fn masked_projection_and_original_predicate_are_separate_occurrences() {
    let results = json!([
        row(&[Some("[private]")], &[&["k1"]]),
        row(&[Some("[private]")], &[&["k3"]])
    ]);
    assert!(run(
        "SELECT r.salary FROM Resource r WHERE r.salary=9223372036854775807",
        false,
        results.clone(),
        |_, _| {}
    )
    .is_ok());
    assert!(run(
        "SELECT r.salary FROM Resource r WHERE r.salary=9223372036854775807",
        false,
        results,
        |_, s| {
            let p = s
                .iter_mut()
                .find(|s| s.path == "outputs/0/expression")
                .unwrap();
            p.domain = id("salary");
            p.column = "salary".into();
            p.logical_type = LogicalType {
                family: Family::Integer,
                nullable: false,
                facets: json!({"integerWidth":{"bits":64,"signed":true}}),
            };
        }
    )
    .is_err());
}
#[test]
fn raw_hidden_order_keys_and_partial_ties_do_not_use_public_mask_order() {
    let sql = "SELECT r.salary FROM Resource r ORDER BY r.salary ASC LIMIT 1";
    assert!(run(
        sql,
        false,
        json!([row(&[Some("[private]")], &[&["k4"]])]),
        |_, _| {}
    )
    .is_ok());
    assert!(run(
        sql,
        false,
        json!([row(&[Some("[private]")], &[&["k5"]])]),
        |_, _| {}
    )
    .is_ok());
    assert!(run(
        sql,
        false,
        json!([row(&[Some("[private]")], &[&["k1"]])]),
        |_, _| {}
    )
    .is_err());
    let desc = "SELECT r.salary FROM Resource r ORDER BY r.salary DESC LIMIT 1";
    assert!(run(
        desc,
        false,
        json!([row(&[Some("[private]")], &[&["k3"]])]),
        |_, _| {}
    )
    .is_ok());
}
#[test]
fn unordered_limits_select_subbags_but_cannot_repeat_an_occurrence() {
    let sql = "SELECT r.token FROM Resource r LIMIT 2";
    assert!(run(
        sql,
        false,
        json!([row(&[Some("z")], &[&["k5"]]), row(&[Some("a")], &[&["k1"]])]),
        |_, _| {}
    )
    .is_ok());
    assert!(run(
        sql,
        false,
        json!([row(&[Some("a")], &[&["k1"]]), row(&[Some("a")], &[&["k1"]])]),
        |_, _| {}
    )
    .is_err());
}
#[test]
fn complete_join_bags_count_all_pairs_and_reject_same_count_identity_substitution() {
    let keys = vec![
        &["k1", "k1"][..],
        &["k1", "k4"],
        &["k4", "k1"],
        &["k4", "k4"],
        &["k3", "k3"],
        &["k3", "k5"],
        &["k5", "k3"],
        &["k5", "k5"],
    ];
    let sql = "SELECT COUNT(*) FROM Resource a JOIN Resource b ON a.token=b.token";
    assert!(run(sql, false, json!([row(&[Some("8")], &keys)]), |_, _| {}).is_ok());
    let mut wrong = keys.clone();
    wrong[1] = wrong[0];
    assert!(run(sql, false, json!([row(&[Some("8")], &wrong)]), |_, _| {}).is_err());
}
#[test]
fn groups_with_same_public_sum_keep_distinct_complete_source_bags() {
    let sql = "SELECT SUM(r.salary) FROM Resource r GROUP BY r.token";
    let rows = json!([
        row(&[Some("-1")], &[&["k1"], &["k4"]]),
        row(&[Some("-1")], &[&["k3"], &["k5"]])
    ]);
    assert!(run(sql, true, rows, |_, _| {}).is_ok());
    assert!(run(
        sql,
        true,
        json!([
            row(&[Some("-1")], &[&["k1"], &["k4"]]),
            row(&[Some("-1")], &[&["k1"], &["k4"]])
        ]),
        |_, _| {}
    )
    .is_err());
}
#[test]
fn false_and_limit_zero_still_validate_all_captured_cells_and_metadata() {
    let sql = "SELECT r.salary FROM Resource r WHERE FALSE LIMIT 0";
    assert!(run(sql, false, json!([]), |_, _| {}).is_ok());
    assert!(run(sql, false, json!([]), |f, _| f["scans"][0]["rows"][3]
        ["cells"]["salary_public"]["value"] =
        Value::Null)
    .is_err());
    assert!(run(sql, false, json!([]), |_, s| s[0].comparison_scope =
        "bad".into())
    .is_err());
    assert!(run(sql, false, json!([]), |f, _| f["scans"][0]
        ["expectedRowCount"] =
        json!("3"))
    .is_err());
}
#[test]
fn keys_counts_column_sets_and_numeric_carriers_are_exact() {
    let q = "SELECT SUM(r.salary) FROM Resource r";
    let r = json!([row(&[Some("-2")], &one())]);
    for change in 0..6 {
        assert!(
            run(q, true, r.clone(), |f, _| match change {
                0 =>
                    f["scans"][0]["rows"][0]["cells"]["salary"]["value"] =
                        json!("+9223372036854775807"),
                1 =>
                    f["scans"][0]["rows"][0]["cells"]["salary"]["value"] =
                        json!("9223372036854775808"),
                2 => f["scans"][0]["rows"][1]["key"] = json!("k1"),
                3 => f["scans"][0]["rows"][0]["key"] = json!("é".repeat(129)),
                4 =>
                    f["scans"][0]["rows"][0]["cells"]["extra"] =
                        json!({"family":"string","value":"hidden"}),
                _ => f["compiledArtifactSha256"] = json!("d".repeat(64)),
            })
            .is_err(),
            "{change}"
        );
    }
}
#[test]
fn consistent_forged_empty_world_is_conditional_not_native_completeness() {
    assert!(run(
        "SELECT COUNT(*) FROM Resource r",
        false,
        json!([row(&[Some("0")], &[])]),
        |f, _| {
            f["scans"][0]["rows"] = json!([]);
            f["scans"][0]["expectedRowCount"] = json!("0");
        }
    )
    .is_ok());
}
#[test]
fn row_tuple_encoding_is_closed_and_cannot_enroll_aggregates() {
    let carrier =
        json!({"version":"weft.stage-row-tuple/0.1-candidate","rows":[{"scan":"s0","key":"k5"}]})
            .to_string();
    assert!(run_variant(
        "SELECT r.token FROM Resource r LIMIT 1",
        false,
        json!([["z", carrier]]),
        Encoding::Rows,
        |_, _| {},
        |s| s
    )
    .is_ok());
    assert!(run_variant(
        "SELECT r.token FROM Resource r LIMIT 1",
        false,
        json!([row(&[Some("z")], &[&["k5"]])]),
        Encoding::Rows,
        |_, _| {},
        |s| s
    )
    .is_err());
    assert!(run_variant(
        "SELECT COUNT(*) FROM Resource r",
        false,
        json!([row(&[Some("4")], &one())]),
        Encoding::Rows,
        |_, _| {},
        |s| s
    )
    .is_err());
}
#[test]
fn duplicate_outer_and_private_members_and_extra_fields_refuse_privately() {
    let q = "SELECT COUNT(*) FROM Resource r";
    let result = json!([row(&[Some("4")], &one())]);
    let e = run_variant(
        q,
        false,
        result.clone(),
        Encoding::Contributors,
        |_, _| {},
        |s| s.replacen('{', "{\"version\":\"forged\",", 1),
    )
    .unwrap_err();
    assert_eq!(e.code, "WFT-CURRENT-QUERY-REFINEMENT");
    assert_eq!(e.message, "Conditional current-query refinement refused");
    assert!(e.source_span.is_none());
    assert!(run(q, false, result.clone(), |f, _| f["extra"] =
        json!("private"))
    .is_err());
    assert!(run(q,false,result,|f,_|f["resultRows"][0][1]=json!("{\"version\":\"weft.stage-contributor-bag/0.1-candidate\",\"version\":\"weft.stage-contributor-bag/0.1-candidate\",\"tuples\":[]}")).is_err());
}
fn population(n: usize, token: &str) -> Value {
    json!((0..n)
        .map(|i| json!({"key":format!("k{i}"),"cells":{"token":{"family":"string","value":token}}}))
        .collect::<Vec<_>>())
}
#[test]
fn static_join_product_boundary_and_intermediate_overflow_refuse_without_prefix() {
    let q = "SELECT COUNT(*) FROM Resource a JOIN Resource b ON a.token=b.token";
    assert!(run(q, false, json!([row(&[Some("0")], &[])]), |f, _| {
        for i in 0..2 {
            f["scans"][i]["rows"] = population(1000, if i == 0 { "a" } else { "b" });
            f["scans"][i]["expectedRowCount"] = json!("1000");
        }
    })
    .is_ok());
    assert!(run(q, false, json!([row(&[Some("0")], &[])]), |f, _| {
        for i in 0..2 {
            let n = if i == 0 { 1001 } else { 1000 };
            f["scans"][i]["rows"] = population(n, if i == 0 { "a" } else { "b" });
            f["scans"][i]["expectedRowCount"] = json!(n.to_string());
        }
    })
    .is_err());
    assert!(run(
        "SELECT COUNT(*) FROM Resource a JOIN Resource b ON a.token=b.token WHERE FALSE LIMIT 0",
        false,
        json!([]),
        |f, _| {
            for i in 0..2 {
                let n = if i == 0 { 100 } else { 101 };
                f["scans"][i]["rows"] = population(n, "same");
                f["scans"][i]["expectedRowCount"] = json!(n.to_string());
            }
        }
    )
    .is_err());
}
#[test]
fn full_population_limit_and_byte_limit_are_exact_not_prefix_bounds() {
    assert!(run(
        "SELECT COUNT(*) FROM Resource r",
        false,
        json!([]),
        |f, _| {
            let rows = (0..ROWS)
                .map(|i| json!({"key":format!("k{i}"),"cells":{}}))
                .collect::<Vec<_>>();
            f["scans"][0]["rows"] = json!(rows);
            f["scans"][0]["expectedRowCount"] = json!(ROWS.to_string());
            let tuples = (0..ROWS)
                .map(|i| json!([{"scan":"s0","key":format!("k{i}")}]))
                .collect::<Vec<_>>();
            f["resultRows"] = json!([[
                ROWS.to_string(),
                json!({"version":"weft.stage-contributor-bag/0.1-candidate","tuples":tuples})
                    .to_string()
            ]]);
        }
    )
    .is_ok());
    assert!(run(
        "SELECT COUNT(*) FROM Resource r LIMIT 0",
        false,
        json!([]),
        |f, _| {
            f["scans"][0]["rows"] = json!((0..=ROWS)
                .map(|i| json!({"key":format!("k{i}"),"cells":{}}))
                .collect::<Vec<_>>());
            f["scans"][0]["expectedRowCount"] = json!((ROWS + 1).to_string());
        }
    )
    .is_err());
    assert!(run_variant(
        "SELECT COUNT(*) FROM Resource r LIMIT 0",
        false,
        json!([]),
        Encoding::Contributors,
        |_, _| {},
        |s| {
            let len = s.len();
            s + &" ".repeat(BYTES - len)
        }
    )
    .is_ok());
    assert!(run_variant(
        "SELECT COUNT(*) FROM Resource r LIMIT 0",
        false,
        json!([]),
        Encoding::Contributors,
        |_, _| {},
        |s| {
            let len = s.len();
            s + &" ".repeat(BYTES - len + 1)
        }
    )
    .is_err());
}
#[test]
fn grouped_hidden_order_and_multiple_directions_use_own_captured_occurrences() {
    let q = "SELECT COUNT(*) FROM Resource r GROUP BY r.token ORDER BY r.token DESC";
    assert!(run(
        q,
        false,
        json!([
            row(&[Some("2")], &[&["k3"], &["k5"]]),
            row(&[Some("2")], &[&["k1"], &["k4"]])
        ]),
        |_, _| {}
    )
    .is_ok());
    let q = "SELECT r.token FROM Resource r ORDER BY r.token ASC,r.salary DESC LIMIT 2";
    assert!(run(
        q,
        false,
        json!([row(&[Some("a")], &[&["k1"]]), row(&[Some("a")], &[&["k4"]])]),
        |_, _| {}
    )
    .is_ok());
    assert!(run(
        q,
        false,
        json!([row(&[Some("a")], &[&["k4"]]), row(&[Some("a")], &[&["k1"]])]),
        |_, _| {}
    )
    .is_err());
}
fn permutations(items: &[usize]) -> Vec<Vec<usize>> {
    if items.is_empty() {
        return vec![vec![]];
    }
    let mut out = Vec::new();
    for (i, n) in items.iter().enumerate() {
        let mut rest = items.to_vec();
        rest.remove(i);
        for mut tail in permutations(&rest) {
            tail.insert(0, *n);
            out.push(tail)
        }
    }
    out
}
fn words(alphabet: usize, width: usize) -> Vec<Vec<usize>> {
    if width == 0 {
        return vec![vec![]];
    }
    let mut out = Vec::new();
    for tail in words(alphabet, width - 1) {
        for n in 0..alphabet {
            let mut v = tail.clone();
            v.push(n);
            out.push(v)
        }
    }
    out
}
#[test]
fn actual_core_order_limit_matches_independent_full_permutation_oracle() {
    let mut checked = 0;
    for n in 0..4usize {
        let labels = (0..n).collect::<Vec<_>>();
        for mask in 0..(1usize << n) {
            let keys = (0..n).map(|i| (mask >> i) & 1).collect::<Vec<_>>();
            for direction in [None, Some(false), Some(true)] {
                for limit in std::iter::once(None).chain((0..n + 2).map(Some)) {
                    let clause = match direction {
                        None => "",
                        Some(false) => " ORDER BY r.token ASC",
                        Some(true) => " ORDER BY r.token DESC",
                    };
                    let end = limit.map_or(String::new(), |n| format!(" LIMIT {n}"));
                    let mut r = source(false);
                    r["sql"] = json!(format!("SELECT r.token FROM Resource r{clause}{end}"));
                    r["parameters"] = json!({});
                    let parent = application_lineage::prepare(&r.to_string()).unwrap();
                    let d = current_query_designation::derive(&parent).unwrap();
                    let ctx = current_query_operands::derive(&d).unwrap();
                    let selections = spec(&ctx);
                    let types = vec![domain(&ctx, &id("token"))];
                    let size = limit.map_or(n, |v| v.min(n));
                    let allowed = permutations(&labels)
                        .into_iter()
                        .filter(|p| {
                            direction.is_none()
                                || p.windows(2).all(|w| {
                                    if direction == Some(true) {
                                        keys[w[0]] >= keys[w[1]]
                                    } else {
                                        keys[w[0]] <= keys[w[1]]
                                    }
                                })
                        })
                        .map(|p| p[..size].to_vec())
                        .collect::<BTreeSet<_>>();
                    let binding = "b".repeat(64);
                    let artifact = "c".repeat(64);
                    let mut frame = json!({"version":"weft.current-query.refinement-input/0.1-candidate","sourceRequestSha256":sha256(parent.source_request_json().as_bytes()),"designationSha256":sha256(d.json().as_bytes()),"materializationBindingSha256":binding,"compiledArtifactSha256":artifact,"scans":[{"scan":"s0","expectedRowCount":n.to_string(),"rows":(0..n).map(|i|json!({"key":format!("k{i}"),"cells":{"token":{"family":"string","value":keys[i].to_string()}}})).collect::<Vec<_>>()}],"resultRows":[]});
                    for width in 0..n + 2 {
                        for observed in words(n + 1, width) {
                            frame["resultRows"]=json!(observed.iter().map(|i|json!([keys.get(*i).copied().unwrap_or(0).to_string(),json!({"version":"weft.stage-contributor-bag/0.1-candidate","tuples":[[{"scan":"s0","key":format!("k{i}")}]]}).to_string()])).collect::<Vec<_>>());
                            let yes = compare(
                                &ctx,
                                Lowered {
                                    binding_sha256: &binding,
                                    artifact_sha256: &artifact,
                                    selections: &selections,
                                    output_types: &types,
                                    encoding: Encoding::Contributors,
                                },
                                &frame.to_string(),
                            )
                            .is_ok();
                            assert_eq!(yes,allowed.contains(&observed),"n={n},mask={mask},direction={direction:?},limit={limit:?},observed={observed:?}");
                            checked += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(checked, 51_690);
}

#[test]
fn unicode_empty_values_and_byte_bounded_keys_preserve_exact_identity() {
    for value in ["", "é", "e\u{301}", "😀"] {
        assert!(run("SELECT r.token FROM Resource r LIMIT 1",false,json!([[value,json!({"version":"weft.stage-contributor-bag/0.1-candidate","tuples":[[{"scan":"s0","key":"k1"}]]}).to_string()]]),|f,_|f["scans"][0]["rows"][0]["cells"]["token"]["value"]=json!(value)).is_ok());
    }
    assert!(run(
        "SELECT r.token FROM Resource r LIMIT 1",
        false,
        json!([row(&[Some("é")], &[&["k1"]])]),
        |f, _| f["scans"][0]["rows"][0]["cells"]["token"]["value"] = json!("e\u{301}")
    )
    .is_err());
    let key = "é".repeat(128);
    let native=json!([["a",json!({"version":"weft.stage-contributor-bag/0.1-candidate","tuples":[[{"scan":"s0","key":key}]]}).to_string()]]);
    assert!(run(
        "SELECT r.token FROM Resource r LIMIT 1",
        false,
        native,
        |f, _| f["scans"][0]["rows"][0]["key"] = json!(key)
    )
    .is_ok());
}
#[test]
fn complete_intermediate_join_tuple_limit_is_inclusive() {
    let tuples=(0..100).flat_map(|a|(0..100).map(move |b|json!([{"scan":"s0","key":format!("k{a}")},{"scan":"s1","key":format!("k{b}")}]))).collect::<Vec<_>>();
    let result = json!([[
        "10000",
        json!({"version":"weft.stage-contributor-bag/0.1-candidate","tuples":tuples}).to_string()
    ]]);
    assert!(run(
        "SELECT COUNT(*) FROM Resource a JOIN Resource b ON a.token=b.token",
        false,
        result,
        |f, _| {
            for i in 0..2 {
                f["scans"][i]["rows"] = population(100, "same");
                f["scans"][i]["expectedRowCount"] = json!("100");
            }
        }
    )
    .is_ok());
}

#[test]
fn repeated_projection_expansion_is_bounded_before_limit_zero() {
    let columns = (0..100)
        .map(|i| format!("r.token AS t{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!("SELECT {columns} FROM Resource r LIMIT 0");
    for (bytes, accepted) in [(1_000, true), (50_000, false)] {
        assert_eq!(
            run(&sql, false, json!([]), |f, _| {
                for row in f["scans"][0]["rows"].as_array_mut().unwrap() {
                    row["cells"]["token"]["value"] = json!("x".repeat(bytes));
                }
            })
            .is_ok(),
            accepted
        );
    }
}

#[test]
fn expected_cell_expansion_is_bounded_before_limit_zero() {
    for (columns, accepted) in [(100, true), (101, false)] {
        let projection = (0..columns)
            .map(|i| format!("r.token AS t{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!("SELECT {projection} FROM Resource r LIMIT 0");
        assert_eq!(
            run(&sql, false, json!([]), |f, _| {
                f["scans"][0]["rows"] = population(10_000, "x");
                f["scans"][0]["expectedRowCount"] = json!("10000");
            })
            .is_ok(),
            accepted
        );
    }
}

#[test]
fn cumulative_utf8_expected_payload_boundary_counts_repeated_cells() {
    // Four two-byte keys contribute eight bytes independently of public cells.
    // Repeating one UTF-8 source value twice leaves a one-byte field to probe
    // immediately below, exactly at, and immediately above the semantic limit.
    for (characters, suffix, accepted) in [
        (4_194_301, "xxx", true),
        (4_194_302, "", true),
        (4_194_302, "x", false),
    ] {
        assert_eq!(
            run(
                "SELECT r.token AS a,r.token AS b,r.resourceId AS c FROM Resource r LIMIT 0",
                false,
                json!([]),
                |f, _| {
                    for row in f["scans"][0]["rows"].as_array_mut().unwrap() {
                        row["cells"]["token"]["value"] = json!("");
                        row["cells"]["resourceId"]["value"] = json!("");
                    }
                    f["scans"][0]["rows"][0]["cells"]["token"]["value"] =
                        json!("é".repeat(characters));
                    f["scans"][0]["rows"][0]["cells"]["resourceId"]["value"] = json!(suffix);
                }
            )
            .is_ok(),
            accepted
        );
    }
}
