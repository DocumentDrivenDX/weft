//! Required-root one-hop collection, using the independently guarded wide prefix.
use super::*;
use weft_core::application_model::RelationshipRead;

pub(super) struct Collection {
    pub name: String,
    pub scan: String,
    pub encoding_check: String,
    pub capacity_check: String,
}
fn violations(lower: &Lower<'_>, body: &str) -> String {
    format!(
        "WITH {} SELECT CAST({} AS STRING) AS violations FROM ({body}) `_related_keys_violations`",
        lower.ctes.join(", "),
        count_sql()
    )
}
pub(super) fn collect(
    lower: &mut Lower<'_>,
    traversals: &relationships::Traversals,
    scan: &str,
    r: &RelationshipRead,
    bound: u16,
    position: usize,
) -> Result<Collection> {
    let access = traversals.raw(r)?;
    let base = fresh_base(lower, position)?;
    let rows = format!("{base}_rows");
    let ranked = format!("{base}_ranked");
    let name = format!("{base}_collection");
    let mut keys = vec![];
    for (i, (id, ty)) in r
        .target_key
        .fields
        .iter()
        .zip(&r.target_key.types)
        .enumerate()
    {
        let f = Expression::Field {
            scan: access.endpoint.into(),
            identity: id.clone(),
            logical_type: ty.clone(),
            span: weft_core::ir::Span { start: 0, end: 0 },
        };
        keys.push(format!(
            "{} AS {}",
            lower.expression(&f)?,
            binding::quote(&format!("k{i}"))
        ));
    }
    lower.ctes.push(format!(
        "{} AS (SELECT e.{} AS __root,e.id AS __edge,{} FROM {} e JOIN {} ON e.{}={}.__id)",
        binding::quote(&rows),
        access.from_id,
        keys.join(", "),
        binding::quote(access.edge),
        binding::quote(access.endpoint),
        access.to_id,
        binding::quote(access.endpoint)
    ));
    let columns = (0..keys.len())
        .map(|i| format!("p.{}", binding::quote(&format!("k{i}"))))
        .collect::<Vec<_>>();
    let item = format!(
        "array({})",
        columns
            .iter()
            .map(|s| format!("CAST({s} AS STRING)"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let order = columns
        .iter()
        .map(|s| format!("{s} COLLATE UTF8_BINARY ASC"))
        .chain(std::iter::once("p.__edge ASC".into()))
        .collect::<Vec<_>>()
        .join(", ");
    lower.ctes.push(format!("{} AS (SELECT p.__root,{item} AS __item,TRY_SUM(CAST(1 AS DECIMAL(38,0))) OVER (PARTITION BY p.__root ORDER BY {order} ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS __ordinal FROM {} p)",binding::quote(&ranked),binding::quote(&rows)));
    let items=format!("transform(slice(sort_array(collect_list(named_struct('ordinal',__ordinal,'item',__item))),1,{bound}),x -> x.item)");
    lower.ctes.push(format!("{} AS (SELECT __root,{items} AS __items,MAX(__ordinal)>CAST({bound} AS DECIMAL(38,0)) AS __truncated FROM {} WHERE __ordinal<=CAST({} AS DECIMAL(38,0)) GROUP BY __root)",binding::quote(&name),binding::quote(&ranked),u32::from(bound)+1));
    let edge = binding::quote(access.edge);
    let proof=format!("SELECT id FROM {edge} WHERE id IS NULL UNION ALL SELECT id FROM {edge} GROUP BY id HAVING {}>1 UNION ALL SELECT __root FROM {} WHERE NOT (from_json(to_json(__item),'ARRAY<STRING>') <=> __item) UNION ALL SELECT totals.__root FROM (SELECT __root,MAX(__ordinal) AS total FROM {} GROUP BY __root) totals LEFT JOIN {} c ON totals.__root=c.__root WHERE c.__root IS NULL OR size(c.__items)<>LEAST(totals.total,CAST({bound} AS DECIMAL(38,0))) OR c.__truncated<>(totals.total>CAST({bound} AS DECIMAL(38,0)))",count_sql(),binding::quote(&ranked),binding::quote(&ranked),binding::quote(&name));
    Ok(Collection {
        name,
        scan: scan.into(),
        encoding_check: violations(lower, &proof),
        capacity_check: violations(
            lower,
            &format!(
                "SELECT __root FROM {} WHERE __ordinal IS NULL",
                binding::quote(&ranked)
            ),
        ),
    })
}

fn fresh_base(lower: &Lower<'_>, position: usize) -> Result<String> {
    // Every rejected candidate consumes at least one already owned scan/CTE name;
    // this finite inventory bound guarantees either a fresh name or refusal.
    (0..=lower.scans.len() + lower.ctes.len())
        .map(|nonce| {
            if nonce == 0 {
                format!("__weft_keys_{position}")
            } else {
                format!("__weft_keys_{position}_{nonce}")
            }
        })
        .find(|base| {
            ["rows", "ranked", "collection"].iter().all(|suffix| {
                let name = format!("{base}_{suffix}");
                !lower.scans.contains_key(&name)
                    && !lower
                        .ctes
                        .iter()
                        .any(|cte| cte.starts_with(&format!("{} AS ", binding::quote(&name))))
            })
        })
        .ok_or_else(|| fail("WFT-EMIT", "No fresh related-key collection names"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_names_use_actual_scan_and_owned_cte_inventory() {
        let request: Value = serde_json::from_str(include_str!(
            "../../../../tests/ashlar-databricks/fixtures/original-commerce-path-request.json"
        ))
        .unwrap();
        let binding: Binding =
            serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap()).unwrap();
        let mut lower = Lower::new(&binding);
        assert_eq!(fresh_base(&lower, 1).unwrap(), "__weft_keys_1");
        lower.scans.insert(
            "__weft_keys_1_rows".into(),
            binding.records[0].logical.clone(),
        );
        assert_eq!(fresh_base(&lower, 1).unwrap(), "__weft_keys_1_1");
        lower
            .ctes
            .push("`__weft_keys_1_1_ranked` AS (SELECT 1)".into());
        assert_eq!(fresh_base(&lower, 1).unwrap(), "__weft_keys_1_2");
    }
}
