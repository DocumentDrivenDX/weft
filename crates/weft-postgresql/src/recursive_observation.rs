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
pub struct Encoded {
    pub integrity: String,
    /// Logical JSONB body. Publish only after integrity/owner prerequisites pass.
    pub body: String,
}
pub fn props(
    property: &PropertyAdmission,
    leaf: &str,
    parameters: &mut Parameters,
) -> Result<String> {
    Ok(encode(property, leaf, parameters)?.integrity)
}
/// Per-value physical prerequisite. Parameters commit only after every original
/// graph, member-presence and leaf-codec correspondence check succeeds.
pub fn encode(
    property: &PropertyAdmission,
    leaf: &str,
    parameters: &mut Parameters,
) -> Result<Encoded> {
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
                let guard = if let Shape::Scalar { logical_type } = &descriptor.shape {
                    if matches!(logical_type.family, Family::Integer | Family::Decimal) {
                        let domain = crate::native_comparator_definition::numeric_domain_for_type(
                            logical_type,
                            &storage.carrier,
                        )?;
                        format!("({} AND {domain})", storage.storage_integrity)
                    } else {
                        storage.storage_integrity
                    }
                } else {
                    unreachable!()
                };
                scalar_guards.push(format!("WHEN {i} THEN {guard}"));
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
                let Shape::Record {
                    members: logical_members,
                } = &descriptor.shape
                else {
                    unreachable!()
                };
                let mut names = std::collections::BTreeSet::new();
                for m in logical_members {
                    if !names.insert(&m.name) {
                        return Err(Diagnostic::new(
                            "WFT-CAPABILITY",
                            "emit",
                            "Recursive logical object needs unique authored names",
                        ));
                    }
                }
                entry["members"]=json!(members.iter().zip(logical_members).map(|(stored,logical)|json!({"name":stored.stored_name,"logicalName":logical.name,"id":stored.value_node})).collect::<Vec<_>>());
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
        r#"(WITH RECURSIVE g AS (SELECT d,(ord-1)::int AS id FROM pg_catalog.jsonb_array_elements({graph}::jsonb) WITH ORDINALITY AS graph(d,ord)), walk(v,id,depth,path) AS (SELECT ({leaf})::jsonb,{root}::int,0,ARRAY[]::text[] UNION ALL SELECT child.v,child.id,w.depth+1,w.path||child.path FROM walk w JOIN g ON g.id=w.id CROSS JOIN LATERAL (SELECT w.v->(m->>'name'),(m->>'id')::int,ARRAY[m->>'name'] FROM pg_catalog.jsonb_array_elements(COALESCE(g.d->'members','[]'::jsonb)) m WHERE g.d->>'kind'='record' AND pg_catalog.jsonb_typeof(w.v)='object' UNION ALL SELECT item.v,(g.d->>'item')::int,ARRAY[(item.ord-1)::text] FROM pg_catalog.jsonb_array_elements(CASE WHEN g.d->>'kind'='sequence' AND pg_catalog.jsonb_typeof(w.v)='array' THEN w.v ELSE '[]'::jsonb END) WITH ORDINALITY item(v,ord) UNION ALL SELECT item.v,(g.d->>'item')::int,ARRAY[item.k] FROM pg_catalog.jsonb_each(CASE WHEN g.d->>'kind'='map' AND pg_catalog.jsonb_typeof(w.v)='object' THEN w.v ELSE '{{}}'::jsonb END) item(k,v) UNION ALL SELECT w.v,(g.d->>'record')::int,ARRAY[]::text[] WHERE g.d->>'kind'='structured' AND pg_catalog.jsonb_typeof(w.v)='object') child(v,id,path) WHERE w.depth<128) SELECT COALESCE(bool_and(w.depth<128 AND CASE WHEN w.v IS NULL THEN (g.d->>'optional')::bool WHEN w.v='null'::jsonb THEN (g.d->>'nullable')::bool ELSE CASE g.d->>'kind' WHEN 'scalar' THEN {scalar} WHEN 'sequence' THEN pg_catalog.jsonb_typeof(w.v)='array' WHEN 'map' THEN pg_catalog.jsonb_typeof(w.v)='object' WHEN 'structured' THEN pg_catalog.jsonb_typeof(w.v)='object' WHEN 'record' THEN pg_catalog.jsonb_typeof(w.v)='object' AND NOT EXISTS (SELECT 1 FROM pg_catalog.jsonb_object_keys(CASE WHEN pg_catalog.jsonb_typeof(w.v)='object' THEN w.v ELSE '{{}}'::jsonb END) keys(k) WHERE NOT EXISTS (SELECT 1 FROM pg_catalog.jsonb_array_elements(g.d->'members') m WHERE m->>'name'=keys.k)) ELSE FALSE END END),FALSE) AND count(*)<=100000 FROM walk w JOIN g ON g.id=w.id)"#,
        root = layout.root
    );
    let prefix = sql
        .rsplit_once(" SELECT COALESCE")
        .ok_or_else(|| {
            Diagnostic::new("WFT-BINDING", "emit", "Recursive SQL prefix is unavailable")
        })?
        .0;
    let body = format!(
        r#"{prefix}, patches AS (SELECT w.path,g.d,row_number() OVER (ORDER BY w.depth DESC,w.path,w.id) AS seq FROM walk w JOIN g ON g.id=w.id WHERE g.d->>'kind'='record' AND pg_catalog.jsonb_typeof(w.v)='object'), folded(seq,value) AS (SELECT 0::bigint,({leaf})::jsonb UNION ALL SELECT p.seq,CASE WHEN cardinality(p.path)=0 THEN assembled.v ELSE pg_catalog.jsonb_set(f.value,p.path,assembled.v,true) END FROM folded f JOIN patches p ON p.seq=f.seq+1 CROSS JOIN LATERAL (SELECT COALESCE(pg_catalog.jsonb_object_agg(m->>'logicalName',CASE WHEN f.value#>(p.path||ARRAY[m->>'name']) IS NULL THEN pg_catalog.jsonb_build_object('state','absent') WHEN f.value#>(p.path||ARRAY[m->>'name'])='null'::jsonb AND (child.d->>'nullable')::bool THEN pg_catalog.jsonb_build_object('state','null') WHEN (child.d->>'optional')::bool OR (child.d->>'nullable')::bool THEN pg_catalog.jsonb_build_object('state','value','value',f.value#>(p.path||ARRAY[m->>'name'])) ELSE f.value#>(p.path||ARRAY[m->>'name']) END),'{{}}'::jsonb) AS v FROM pg_catalog.jsonb_array_elements(p.d->'members') m JOIN g child ON child.id=(m->>'id')::int) assembled) SELECT value FROM folded ORDER BY seq DESC LIMIT 1)"#
    );
    *parameters = staged;
    Ok(Encoded {
        integrity: sql,
        body,
    })
}

/// Complete-owner physical prerequisite from an admitted props home. No query
/// filter/order/limit participates in this observation.
pub fn props_owner(
    property: &PropertyAdmission,
    namespace: &crate::Identifier,
    alias: &crate::Identifier,
    parameters: &mut Parameters,
) -> Result<String> {
    if !matches!(
        property.home,
        crate::property_definition::HomeAdmission::Props { .. }
    ) {
        return Err(Diagnostic::new(
            "WFT-CAPABILITY",
            "emit",
            "Recursive native row observation is not implemented",
        ));
    }
    let mut staged = parameters.clone();
    let source = property.home.owner_mapping().source(
        namespace,
        alias,
        &property.owner_catalog_id,
        &mut staged,
    )?;
    let location = property.home.props_location(alias, &mut staged)?;
    let integrity = props(property, &location.leaf, &mut staged)?;
    let sql = format!(
        "SELECT count(*) AS violations FROM {} WHERE {} AND ({} AND {}) IS DISTINCT FROM TRUE",
        source.sql, source.discriminator, location.root_integrity, integrity
    );
    *parameters = staged;
    Ok(sql)
}
