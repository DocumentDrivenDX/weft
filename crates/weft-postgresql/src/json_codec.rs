//! Recursive JSONB codec over a finite identity graph; schemas are not expanded.
use crate::Parameters;
use serde_json::json;
use weft_core::{
    application_model::{Descriptor, Shape},
    error::{Diagnostic, Result},
    ir::{Family, Identity, LogicalType},
};
pub struct Encoded {
    pub value: String,
    pub integrity: String,
}
pub(crate) fn pack(graph: &[Descriptor]) -> Result<serde_json::Value> {
    let index = |id: &Identity| {
        graph.iter().position(|d| &d.identity == id).ok_or_else(|| {
            Diagnostic::new(
                "WFT-BINDING",
                "lower",
                "Recursive value descriptor is missing",
            )
        })
    };
    let mut packed = vec![];
    for d in graph {
        let mut entry = json!({"optional":d.availability.as_deref()==Some("absent-allowed")});
        match &d.shape {
            Shape::Scalar { logical_type } => {
                entry["kind"] = json!("scalar");
                entry["type"] = json!(logical_type);
            }
            Shape::Sequence { item } | Shape::Map { item } => {
                entry["kind"] = json!(if matches!(d.shape, Shape::Sequence { .. }) {
                    "sequence"
                } else {
                    "map"
                });
                entry["item"] = json!(index(item)?);
            }
            Shape::Record { members } => {
                entry["kind"] = json!("record");
                entry["members"] = json!(members
                    .iter()
                    .map(|m| Ok(json!({"name":m.name,"id":index(&m.identity)?,"identity":json!(m.identity).to_string()})))
                    .collect::<Result<Vec<_>>>()?);
            }
            Shape::Structured { record } => {
                let Shape::Record { members } = &graph[index(record)?].shape else {
                    return Err(Diagnostic::new(
                        "WFT-BINDING",
                        "lower",
                        "Structured descriptor does not reference a Record",
                    ));
                };
                entry["kind"] = json!("record");
                entry["members"] = json!(members
                    .iter()
                    .map(|m| Ok(json!({"name":m.name,"id":index(&m.identity)?,"identity":json!(m.identity).to_string()})))
                    .collect::<Result<Vec<_>>>()?);
            }
        }
        if let Some(members) = entry["members"].as_array() {
            let mut names = std::collections::BTreeSet::new();
            for member in members {
                let name = member["name"].as_str().unwrap();
                if name.contains('\0') || !names.insert(name) {
                    return Err(Diagnostic::new(
                        "WFT-CAPABILITY",
                        "lower",
                        "Recursive object encoding requires unique authored member names",
                    ));
                }
            }
        }
        entry["native"] = json!(match d.shape {
            Shape::Structured { .. } => "structured",
            Shape::Record { .. } => "record",
            Shape::Sequence { .. } => "sequence",
            Shape::Map { .. } => "map",
            Shape::Scalar { .. } => "scalar",
        });
        packed.push(entry);
    }
    Ok(json!(packed))
}
pub fn encode(
    graph: &[Descriptor],
    root: &Identity,
    leaf: &str,
    parameters: &mut Parameters,
) -> Result<Encoded> {
    let packed = pack(graph)?;
    let root_index = graph
        .iter()
        .position(|d| &d.identity == root)
        .ok_or_else(|| {
            Diagnostic::new(
                "WFT-BINDING",
                "lower",
                "Recursive root descriptor is missing",
            )
        })?;
    let slot = parameters.push(
        LogicalType {
            family: Family::String,
            facets: json!({}),
            nullable: false,
        },
        packed.to_string(),
        json!({"recursiveCodecGraph":root}),
    )?;
    let prefix=r#"WITH RECURSIVE g AS (SELECT d,(ord-1)::int AS id FROM pg_catalog.jsonb_array_elements(@GRAPH@::jsonb) WITH ORDINALITY AS graph(d,ord)), walk(v,id,path,depth) AS (SELECT @LEAF@,@ROOT@::int,ARRAY[]::text[],0 UNION ALL SELECT child.v,child.id,w.path||child.key,w.depth+1 FROM walk w JOIN g ON g.id=w.id CROSS JOIN LATERAL (SELECT w.v->(m->>'name') AS v,(m->>'id')::int AS id,m->>'name' AS key FROM pg_catalog.jsonb_array_elements(COALESCE(g.d->'members','[]'::jsonb)) m WHERE g.d->>'kind'='record' AND pg_catalog.jsonb_typeof(w.v)='object' UNION ALL SELECT item.v,(g.d->>'item')::int,(item.ord-1)::text FROM pg_catalog.jsonb_array_elements(CASE WHEN g.d->>'kind'='sequence' AND pg_catalog.jsonb_typeof(w.v)='array' THEN w.v ELSE '[]'::jsonb END) WITH ORDINALITY AS item(v,ord) UNION ALL SELECT item.v,(g.d->>'item')::int,item.key FROM pg_catalog.jsonb_each(CASE WHEN g.d->>'kind'='map' AND pg_catalog.jsonb_typeof(w.v)='object' THEN w.v ELSE '{}'::jsonb END) AS item(key,v)) child WHERE w.depth<128)"#
        .replace("@GRAPH@",&slot).replace("@ROOT@",&root_index.to_string()).replace("@LEAF@",leaf);
    let integrity = r#"SELECT COALESCE(bool_and(w.depth<128 AND CASE WHEN w.v IS NULL THEN (g.d->>'optional')::bool AND w.depth>0 ELSE CASE g.d->>'kind' WHEN 'scalar' THEN CASE g.d->'type'->>'family' WHEN 'string' THEN pg_catalog.jsonb_typeof(w.v)='string' WHEN 'boolean' THEN pg_catalog.jsonb_typeof(w.v)='boolean' WHEN 'integer' THEN CASE WHEN pg_catalog.jsonb_typeof(w.v)='number' THEN (w.v#>>'{}')::numeric=pg_catalog.trunc((w.v#>>'{}')::numeric) AND (w.v#>>'{}')::numeric >= CASE WHEN (g.d->'type'->'facets'->'integerWidth'->>'signed')::bool THEN -pg_catalog.power(2::numeric,(g.d->'type'->'facets'->'integerWidth'->>'bits')::int-1) ELSE 0::numeric END AND (w.v#>>'{}')::numeric < pg_catalog.power(2::numeric,(g.d->'type'->'facets'->'integerWidth'->>'bits')::int-CASE WHEN (g.d->'type'->'facets'->'integerWidth'->>'signed')::bool THEN 1 ELSE 0 END) ELSE false END WHEN 'decimal' THEN CASE WHEN pg_catalog.jsonb_typeof(w.v)='number' THEN (w.v#>>'{}')::numeric=pg_catalog.trunc((w.v#>>'{}')::numeric,(g.d->'type'->'facets'->>'scale')::int) AND pg_catalog.abs((w.v#>>'{}')::numeric)<pg_catalog.power(10::numeric,(g.d->'type'->'facets'->>'precision')::int-(g.d->'type'->'facets'->>'scale')::int) ELSE false END ELSE false END WHEN 'sequence' THEN pg_catalog.jsonb_typeof(w.v)='array' WHEN 'map' THEN pg_catalog.jsonb_typeof(w.v)='object' WHEN 'record' THEN pg_catalog.jsonb_typeof(w.v)='object' AND NOT EXISTS (SELECT 1 FROM pg_catalog.jsonb_object_keys(CASE WHEN pg_catalog.jsonb_typeof(w.v)='object' THEN w.v ELSE '{}'::jsonb END) names(key) WHERE NOT EXISTS (SELECT 1 FROM pg_catalog.jsonb_array_elements(g.d->'members') m WHERE m->>'name'=names.key)) ELSE false END END),false) AND count(*)<=100000 FROM walk w JOIN g ON g.id=w.id"#;
    let value=r#", patches AS (SELECT w.path,g.d,row_number() OVER (ORDER BY w.depth DESC,w.path) AS seq FROM walk w JOIN g ON g.id=w.id WHERE (w.depth>0 AND (g.d->>'optional')::bool) OR (g.d->>'kind'='scalar' AND g.d->'type'->>'family' IN ('integer','decimal'))), folded(seq,value) AS (SELECT 0::bigint,@LEAF@ UNION ALL SELECT p.seq,CASE WHEN cardinality(p.path)=0 THEN change.v ELSE pg_catalog.jsonb_set(f.value,p.path,change.v,true) END FROM folded f JOIN patches p ON p.seq=f.seq+1 CROSS JOIN LATERAL (SELECT CASE WHEN (p.d->>'optional')::bool AND cardinality(p.path)>0 THEN CASE WHEN f.value#>p.path IS NULL THEN pg_catalog.jsonb_build_object('state','absent') ELSE pg_catalog.jsonb_build_object('state','value','value',CASE WHEN p.d->>'kind'='scalar' AND p.d->'type'->>'family' IN ('integer','decimal') THEN pg_catalog.to_jsonb(f.value#>>p.path) ELSE f.value#>p.path END) END WHEN p.d->>'kind'='scalar' AND p.d->'type'->>'family' IN ('integer','decimal') THEN pg_catalog.to_jsonb(f.value#>>p.path) ELSE f.value#>p.path END AS v) change) SELECT value FROM folded ORDER BY seq DESC LIMIT 1"#
        .replace("@LEAF@",leaf);
    Ok(Encoded {
        value: format!("({prefix}{value})"),
        integrity: format!("({prefix} {integrity})"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distinct_member_identities_cannot_be_collapsed_into_one_object_name() {
        use weft_core::application_model::Member;
        let id = |element: &str| Identity {
            document_id: "codec-fixture".into(),
            revision: "1".into(),
            module: "m".into(),
            element: element.into(),
        };
        let graph = vec![
            Descriptor {
                identity: id("record"),
                availability: None,
                shape: Shape::Record {
                    members: vec![
                        Member {
                            name: "same".into(),
                            identity: id("a"),
                        },
                        Member {
                            name: "same".into(),
                            identity: id("b"),
                        },
                    ],
                },
            },
            Descriptor {
                identity: id("a"),
                availability: Some("required".into()),
                shape: Shape::Scalar {
                    logical_type: LogicalType {
                        family: Family::String,
                        facets: json!({}),
                        nullable: false,
                    },
                },
            },
            Descriptor {
                identity: id("b"),
                availability: Some("required".into()),
                shape: Shape::Scalar {
                    logical_type: LogicalType {
                        family: Family::String,
                        facets: json!({}),
                        nullable: false,
                    },
                },
            },
        ];
        let error = encode(
            &graph,
            &id("record"),
            "'{}'::jsonb",
            &mut Parameters::default(),
        )
        .err()
        .unwrap();
        assert_eq!(error.code, "WFT-CAPABILITY");
    }
}
