//! Descriptor-guided complete row trees, preserving native identity and order.
use crate::{access::ScalarAccess, qualified, CatalogDomain, Identifier, Parameters};
use serde_json::json;
use weft_core::{
    application_model::Descriptor,
    error::{Diagnostic, Result},
    ir::{Family, Identity, LogicalType},
};
pub struct Property<'a> {
    pub namespace: &'a Identifier,
    pub owner: &'a Identifier,
    pub type_id: &'a str,
    pub property_id: &'a str,
}
impl Property<'_> {
    pub fn access(
        &self,
        graph: &[Descriptor],
        identity: &Identity,
        parameters: &mut Parameters,
        occurrence: usize,
    ) -> Result<ScalarAccess> {
        let packed = crate::json_codec::pack(graph)?;
        let root_index = graph
            .iter()
            .position(|d| &d.identity == identity)
            .ok_or_else(|| {
                Diagnostic::new("WFT-BINDING", "lower", "Row root descriptor is missing")
            })?;
        let graph_slot = parameters.push(
            LogicalType {
                family: Family::String,
                facets: json!({}),
                nullable: false,
            },
            packed.to_string(),
            json!({"rowCodecGraph":identity}),
        )?;
        let ty = parameters.catalog(
            CatalogDomain::Int,
            self.type_id,
            json!({"rowCodecOwner":identity}),
        )?;
        let property = parameters.catalog(
            CatalogDomain::Int,
            self.property_id,
            json!({"rowCodecProperty":identity}),
        )?;
        let state = Identifier::new(&format!("weft_state_{occurrence}"))?.sql();
        let node = Identifier::new(&format!("weft_node_{occurrence}"))?.sql();
        let table = |name| qualified(self.namespace, &Identifier::new(name).unwrap());
        let owner = self.owner.sql();
        let joins=vec![format!("LEFT JOIN {} AS {state} ON {state}.owner_kind='object' AND {state}.object_id={owner}.id AND {state}.object_type_id={owner}.type_id AND {state}.property_owner_type_id={ty}::int AND {state}.property_id={property}::int",table("row_home_state")),format!("LEFT JOIN {} AS {node} ON {node}.state_id={state}.state_id AND {node}.node_id={state}.root_node_id",table("row_home_node"))];
        let prefix=r#"WITH RECURSIVE g AS (SELECT d,(ord-1)::int AS id FROM pg_catalog.jsonb_array_elements(@GRAPH@::jsonb) WITH ORDINALITY AS graph(d,ord)), paths(node_id,id,path,sort,depth,seen) AS (SELECT @ROOT@.node_id,@INDEX@::int,ARRAY[]::text[] COLLATE pg_catalog."C",ARRAY[]::text[] COLLATE pg_catalog."C",0,ARRAY[@ROOT@.node_id] UNION ALL SELECT c.node_id,child.id,(p.path||child.key) COLLATE pg_catalog."C",(p.sort||child.sort) COLLATE pg_catalog."C",p.depth+1,p.seen||c.node_id FROM paths p JOIN g ON g.id=p.id JOIN @NODES@ c ON c.state_id=@STATE@.state_id AND c.parent_node_id=p.node_id CROSS JOIN LATERAL (SELECT (g.d->>'item')::int AS id,c.sequence_ordinal::text AS key,'s'||pg_catalog.lpad(c.sequence_ordinal::text,20,'0') AS sort WHERE g.d->>'kind'='sequence' AND c.slot_kind='sequence' AND c.sequence_ordinal>=0 UNION ALL SELECT (g.d->>'item')::int,c.map_key,'m'||c.map_key WHERE g.d->>'kind'='map' AND c.slot_kind='map' AND c.map_key IS NOT NULL UNION ALL SELECT (m->>'id')::int,m->>'name','r'||(m->>'name') FROM pg_catalog.jsonb_array_elements(COALESCE(g.d->'members','[]'::jsonb)) m WHERE g.d->>'kind'='record' AND c.slot_kind='record' AND c.record_field_identity_bytes=pg_catalog.convert_to(m->>'identity','UTF8')) child WHERE p.depth<128 AND NOT c.node_id=ANY(p.seen)), observations AS (SELECT p.*,n.value_kind,g.d,CASE n.value_kind WHEN 'scalar' THEN CASE s.scalar_kind WHEN 'string' THEN pg_catalog.to_jsonb(s.text_value) WHEN 'boolean' THEN pg_catalog.to_jsonb(s.boolean_value) ELSE pg_catalog.to_jsonb(s.numeric_value) END WHEN 'sequence' THEN '[]'::jsonb WHEN 'map' THEN '{}'::jsonb WHEN 'structured' THEN '{}'::jsonb WHEN 'record' THEN '{}'::jsonb ELSE 'null'::jsonb END AS value,s.scalar_kind FROM paths p JOIN @NODES@ n ON n.state_id=@STATE@.state_id AND n.node_id=p.node_id JOIN g ON g.id=p.id LEFT JOIN @PAYLOADS@ s ON s.state_id=n.state_id AND s.node_id=n.node_id)"#
            .replace("@GRAPH@",&graph_slot).replace("@INDEX@",&root_index.to_string()).replace("@NODES@",&table("row_home_node")).replace("@PAYLOADS@",&table("row_home_scalar")).replace("@STATE@",&state).replace("@ROOT@",&node);
        let raw=format!("({prefix}, ordered AS (SELECT *,row_number() OVER (ORDER BY depth,sort) AS seq FROM observations), folded(seq,value) AS (SELECT 0::bigint,'null'::jsonb UNION ALL SELECT o.seq,CASE WHEN cardinality(o.path)=0 THEN o.value ELSE pg_catalog.jsonb_set(f.value,o.path,o.value,true) END FROM folded f JOIN ordered o ON o.seq=f.seq+1) SELECT value FROM folded ORDER BY seq DESC LIMIT 1)");
        let typed=format!("({prefix} SELECT count(*)=(SELECT count(*) FROM {} n WHERE n.state_id={state}.state_id) AND count(*)=count(DISTINCT path) AND count(*)<=100000 AND COALESCE(bool_and(depth<128 AND value_kind=d->>'native' AND (value_kind<>'scalar' OR scalar_kind=d->'type'->>'family')),false) FROM observations)",table("row_home_node"));
        let encoded = crate::json_codec::encode(graph, identity, &raw, parameters)?;
        let tree = crate::tree::integrity(self.namespace, &state, &node);
        Ok(ScalarAccess{value:encoded.value,present:format!("{state}.state_id IS NOT NULL"),native_null:format!("{node}.value_kind='null'"),integrity:format!("({state}.state_id IS NOT NULL AND {node}.node_id IS NOT NULL AND {tree} AND {typed} AND {})",encoded.integrity),joins})
    }
}
