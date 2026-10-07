//! Original native topology packet for SQL lowering; identities are selected code.
use crate::{
    property_definition::PropertyAdmission,
    registered_access::{Access, Location},
    value_definition::LayoutShape,
    Parameters,
};
use serde_json::{json, Value};
use weft_core::{
    error::{Diagnostic, Result},
    ir::{Family, LogicalType},
};
pub struct Mapping {
    pub topology_parameter: String,
    /// Private custody SQL, not the public value carrier.
    pub custody_sql: String,
    /// Native rows paired with original topology and stored-member paths.
    pub walk_sql: String,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "emit", message)
}
/// Admit original topology and stage one typed metadata parameter atomically.
/// The backend-selected procedure determines exact native field identity bytes;
/// model bytes cannot select or load executable code.
pub fn encode(
    property: &PropertyAdmission,
    access: &Access<'_>,
    parameters: &mut Parameters,
    mut field_identity: impl FnMut(&Value) -> Result<Vec<u8>>,
) -> Result<Mapping> {
    access.verify_property(property)?;
    let Location::Row(location) = &access.location else {
        return Err(fail("Native mapping requires row access"));
    };
    if !matches!(
        property.home,
        crate::property_definition::HomeAdmission::Row { .. }
    ) {
        return Err(fail("Native mapping requires original row home"));
    }
    crate::result_definition::property_column(property, 1, "native_mapping")?;
    property
        .value
        .graph
        .verify_record_presence(&property.value.admitted_record_presence)?;
    property.value.verify_leaf_codec_custody()?;
    let layout = property.value.graph.layout()?;
    let hex = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let mut packed = Vec::new();
    for node in &layout.nodes {
        let shape = match &node.shape {
            LayoutShape::Scalar {
                family,
                storage_representation,
            } => {
                json!({"kind":"scalar","family":family,"storageRepresentation":storage_representation})
            }
            LayoutShape::Sequence { item } => json!({"kind":"sequence","item":item}),
            LayoutShape::Map { item } => json!({"kind":"map","item":item}),
            LayoutShape::Structured { record } => json!({"kind":"structured","record":record}),
            LayoutShape::Record { members } => {
                let mut identities = std::collections::BTreeSet::new();
                let mut entries = Vec::new();
                for member in members {
                    let bytes = field_identity(member.field_identity)?;
                    if !identities.insert(bytes.clone()) {
                        return Err(fail(
                            "Selected native field encoding aliases original members",
                        ));
                    }
                    entries.push(json!({"fieldIdentity":member.field_identity,"identityHex":hex(&bytes),"storedName":member.stored_name,"valueNode":member.value_node,"presenceHex":hex(member.presence_bytes)}));
                }
                json!({"kind":"record","members":entries})
            }
        };
        packed.push(json!({"codecHex":hex(node.codec_bytes),"shape":shape}));
    }
    let mut staged = parameters.clone();
    let topology_parameter = staged.push(
        LogicalType {
            family: Family::String,
            facets: json!({}),
            nullable: false,
        },
        json!({"root":layout.root,"nodes":packed}).to_string(),
        json!({"use":"original-native-tree-topology","property":property.identity}),
    )?;
    let custody_sql = location.tree_custody();
    let walk_sql = walk(&topology_parameter, &custody_sql);
    let mapping = Mapping {
        topology_parameter,
        custody_sql,
        walk_sql,
    };
    *parameters = staged;
    Ok(mapping)
}

fn prefix(parameter: &str, custody: &str) -> String {
    format!(
        r#"(WITH RECURSIVE metadata AS (SELECT {parameter}::pg_catalog.jsonb AS v),
    raw AS (SELECT x AS r FROM pg_catalog.jsonb_array_elements({custody}) x),
    walk(r,i,path,depth) AS (
      SELECT r,(metadata.v->>'root')::int,ARRAY[]::text[],0 FROM raw,metadata
      WHERE r->>2 IS NULL AND r->>4='root' AND r->>1 IS NOT NULL
      UNION ALL
      SELECT child.r,next.i,w.path||next.key,w.depth+1
      FROM walk w CROSS JOIN metadata
      CROSS JOIN LATERAL (SELECT metadata.v->'nodes'->w.i->'shape' AS shape) original
      CROSS JOIN LATERAL (SELECT CASE WHEN original.shape->>'kind'='structured'
        THEN metadata.v->'nodes'->(original.shape->>'record')::int->'shape'
        ELSE original.shape END AS shape) container
      JOIN raw child ON child.r->>2=w.r->>1 AND child.r->>0=w.r->>0
      CROSS JOIN LATERAL (
        SELECT (container.shape->>'item')::int AS i,child.r->>5 AS key
        WHERE container.shape->>'kind'='sequence' AND w.r->>3='sequence' AND child.r->>4='sequence'
        UNION ALL SELECT (container.shape->>'item')::int,child.r->>6
        WHERE container.shape->>'kind'='map' AND w.r->>3='map' AND child.r->>4='map'
        UNION ALL SELECT (member->>'valueNode')::int,member->>'storedName'
        FROM pg_catalog.jsonb_array_elements(COALESCE(container.shape->'members','[]'::jsonb)) member
        WHERE container.shape->>'kind'='record' AND w.r->>3 IN ('structured','record')
          AND child.r->>4='record' AND child.r->>7=member->>'identityHex'
      ) next WHERE w.depth<128
    )
"#
    )
}
fn walk(parameter: &str, custody: &str) -> String {
    let prefix = prefix(parameter, custody);
    format!("{prefix}    SELECT pg_catalog.jsonb_build_object('rows',COALESCE((SELECT pg_catalog.jsonb_agg(
      pg_catalog.jsonb_build_object('cells',r,'logicalNode',i,'path',path,'depth',depth) ORDER BY depth,path) FROM walk),'[]'::jsonb),
      'unmatchedRows',(SELECT count(*) FROM raw)-(SELECT count(*) FROM walk),
      'uniqueNodes',(SELECT count(*) FROM raw)=(SELECT count(DISTINCT (r->>0,r->>1)) FROM raw),
      'matchingShapes',COALESCE((SELECT bool_and(CASE metadata.v->'nodes'->i->'shape'->>'kind'
        WHEN 'scalar' THEN r->>3 IN ('scalar','null') ELSE r->>3=metadata.v->'nodes'->i->'shape'->>'kind' END) FROM walk,metadata),FALSE))
 )")
}

pub struct Body {
    pub storage_body: String,
    pub logical_body: String,
    pub logical_integrity: String,
    pub pairing: String,
}
/// Intermediate assembly only: native structural/source prerequisites must still
/// be established before publishing the logical body. Scalar SQL is trusted
/// backend code and returns exact JSONB under alias w.r's native custody cells.
pub fn body(
    property: &PropertyAdmission,
    access: &Access<'_>,
    parameters: &mut Parameters,
    field_identity: impl FnMut(&Value) -> Result<Vec<u8>>,
    mut scalar: impl FnMut(usize, &crate::value_definition::LayoutNode<'_>) -> Result<String>,
) -> Result<Body> {
    let mut staged = parameters.clone();
    let mapping = encode(property, access, &mut staged, field_identity)?;
    let layout = property.value.graph.layout()?;
    let mut leaves = Vec::new();
    for (i, node) in layout.nodes.iter().enumerate() {
        if matches!(node.shape, LayoutShape::Scalar { .. }) {
            leaves.push(format!("WHEN {i} THEN ({})", scalar(i, node)?));
        }
    }
    let leaf = format!("CASE w.i {} ELSE NULL::jsonb END", leaves.join(" "));
    let prefix = prefix(&mapping.topology_parameter, &mapping.custody_sql);
    let storage_body = format!(
        r#"{prefix},
      ordered AS (SELECT w.*,row_number() OVER (ORDER BY depth,path) AS ordinal,
        CASE WHEN w.r->>3 IN ('structured','record','map') THEN '{{}}'::jsonb
          WHEN w.r->>3='sequence' THEN COALESCE((SELECT jsonb_agg('null'::jsonb) FROM walk child WHERE child.r->>2=w.r->>1),'[]'::jsonb)
          WHEN w.r->>3='null' THEN 'null'::jsonb ELSE {leaf} END AS value
        FROM walk w),
      assembled(ordinal,value) AS (SELECT 0::bigint,'null'::jsonb UNION ALL
        SELECT o.ordinal,CASE WHEN cardinality(o.path)=0 THEN o.value
          ELSE jsonb_set(a.value,o.path,o.value,TRUE) END
        FROM assembled a JOIN ordered o ON o.ordinal=a.ordinal+1)
      SELECT value FROM assembled ORDER BY ordinal DESC LIMIT 1)"#
    );
    let logical = crate::recursive_observation::encode(property, &storage_body, &mut staged)?;
    let result = Body {
        storage_body,
        logical_body: logical.body,
        logical_integrity: logical.integrity,
        pairing: mapping.walk_sql,
    };
    *parameters = staged;
    Ok(result)
}
