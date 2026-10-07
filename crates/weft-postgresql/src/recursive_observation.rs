//! Original-topology JSONB storage observation. Logical decoding is separate.
use crate::{
    property_definition::{PropertyAdmission, PropsLocation},
    value_definition::LayoutShape,
    Parameters,
};
use serde_json::json;
use weft_core::{
    application_model::Shape,
    error::{Diagnostic, Result},
    ir::{Family, LogicalType},
};
/// Per-value physical prerequisite. Parameters commit only after every original
/// graph, member-presence and leaf-codec correspondence check succeeds.
pub fn props(
    property: &PropertyAdmission,
    leaf: &str,
    parameters: &mut Parameters,
) -> Result<String> {
    crate::result_definition::property_column(property, 1, "recursive_observation")?;
    property
        .value
        .graph
        .verify_record_presence(&property.value.admitted_record_presence)?;
    let layout = property.value.graph.layout()?;
    let nodes = property.value.graph.value["nodes"].as_array().unwrap();
    let mut packed = Vec::new();
    let mut scalar_guards = Vec::new();
    for (i, node) in layout.nodes.iter().enumerate() {
        let descriptor = property
            .value
            .descriptors()
            .iter()
            .find(|d| json!(d.identity) == nodes[i]["authoredIdentity"])
            .ok_or_else(|| {
                Diagnostic::new(
                    "WFT-BINDING",
                    "emit",
                    "Recursive observation lacks original descriptor",
                )
            })?;
        let mut entry = json!({"optional":descriptor.availability.as_deref()==Some("absent-allowed"),"nullable":matches!(&descriptor.shape,Shape::Scalar {logical_type} if logical_type.nullable)});
        match &node.shape {
            LayoutShape::Scalar { .. } => {
                entry["kind"] = json!("scalar");
                let location = PropsLocation {
                    root: "w.v".into(),
                    leaf: "w.v".into(),
                    text: "(w.v #>> '{}')".into(),
                    present: "(w.v IS NOT NULL)".into(),
                    native_null: "(w.v='null'::jsonb)".into(),
                    root_integrity: "TRUE".into(),
                };
                let storage = property
                    .value
                    .props_node_storage(i, &location)?
                    .ok_or_else(|| {
                        Diagnostic::new(
                            "WFT-BINDING",
                            "emit",
                            "Missing original recursive scalar codec",
                        )
                    })?;
                scalar_guards.push(format!("WHEN {i} THEN {}", storage.storage_integrity));
            }
            LayoutShape::Sequence { item } | LayoutShape::Map { item } => {
                entry["kind"] = json!(if matches!(node.shape, LayoutShape::Sequence { .. }) {
                    "sequence"
                } else {
                    "map"
                });
                entry["item"] = json!(item);
            }
            LayoutShape::Structured { record } => {
                entry["kind"] = json!("structured");
                entry["record"] = json!(record);
            }
            LayoutShape::Record { members } => {
                entry["kind"] = json!("record");
                entry["members"] = json!(members
                    .iter()
                    .map(|m| json!({"name":m.stored_name,"id":m.value_node}))
                    .collect::<Vec<_>>());
            }
        }
        packed.push(entry);
    }
    let mut staged = parameters.clone();
    let graph = staged.push(
        LogicalType {
            family: Family::String,
            facets: json!({}),
            nullable: false,
        },
        json!(packed).to_string(),
        json!({"originalRecursiveObservation":property.identity}),
    )?;
    let scalar = if scalar_guards.is_empty() {
        "FALSE".into()
    } else {
        format!("CASE w.id {} ELSE FALSE END", scalar_guards.join(" "))
    };
    let sql = format!(
        r#"(WITH RECURSIVE g AS (SELECT d,(ord-1)::int AS id FROM pg_catalog.jsonb_array_elements({graph}::jsonb) WITH ORDINALITY AS graph(d,ord)), walk(v,id,depth) AS (SELECT ({leaf})::jsonb,{root}::int,0 UNION ALL SELECT child.v,child.id,w.depth+1 FROM walk w JOIN g ON g.id=w.id CROSS JOIN LATERAL (SELECT w.v->(m->>'name'),(m->>'id')::int FROM pg_catalog.jsonb_array_elements(COALESCE(g.d->'members','[]'::jsonb)) m WHERE g.d->>'kind'='record' AND pg_catalog.jsonb_typeof(w.v)='object' UNION ALL SELECT item.v,(g.d->>'item')::int FROM pg_catalog.jsonb_array_elements(CASE WHEN g.d->>'kind'='sequence' AND pg_catalog.jsonb_typeof(w.v)='array' THEN w.v ELSE '[]'::jsonb END) item(v) UNION ALL SELECT item.v,(g.d->>'item')::int FROM pg_catalog.jsonb_each(CASE WHEN g.d->>'kind'='map' AND pg_catalog.jsonb_typeof(w.v)='object' THEN w.v ELSE '{{}}'::jsonb END) item(k,v) UNION ALL SELECT w.v,(g.d->>'record')::int WHERE g.d->>'kind'='structured' AND pg_catalog.jsonb_typeof(w.v)='object') child(v,id) WHERE w.depth<128) SELECT COALESCE(bool_and(w.depth<128 AND CASE WHEN w.v IS NULL THEN (g.d->>'optional')::bool WHEN w.v='null'::jsonb THEN (g.d->>'nullable')::bool ELSE CASE g.d->>'kind' WHEN 'scalar' THEN {scalar} WHEN 'sequence' THEN pg_catalog.jsonb_typeof(w.v)='array' WHEN 'map' THEN pg_catalog.jsonb_typeof(w.v)='object' WHEN 'structured' THEN pg_catalog.jsonb_typeof(w.v)='object' WHEN 'record' THEN pg_catalog.jsonb_typeof(w.v)='object' AND NOT EXISTS (SELECT 1 FROM pg_catalog.jsonb_object_keys(CASE WHEN pg_catalog.jsonb_typeof(w.v)='object' THEN w.v ELSE '{{}}'::jsonb END) keys(k) WHERE NOT EXISTS (SELECT 1 FROM pg_catalog.jsonb_array_elements(g.d->'members') m WHERE m->>'name'=keys.k)) ELSE FALSE END END),FALSE) AND count(*)<=100000 FROM walk w JOIN g ON g.id=w.id)"#,
        root = layout.root
    );
    *parameters = staged;
    Ok(sql)
}
