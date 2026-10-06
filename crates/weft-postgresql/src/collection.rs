//! Candidate collection access. Complete-tree authority remains a host obligation.
use crate::{access::ScalarAccess, qualified, CatalogDomain, Identifier, Parameters};
use serde_json::json;
use weft_core::{
    error::{Diagnostic, Result},
    ir::{Family, LogicalType},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Sequence,
    Map,
}
pub struct CollectionProperty<'a> {
    pub namespace: &'a Identifier,
    pub owner: &'a Identifier,
    pub type_id: &'a str,
    pub property_id: &'a str,
    pub item: &'a LogicalType,
    pub kind: Kind,
}
impl CollectionProperty<'_> {
    pub fn props(&self, parameters: &mut Parameters) -> Result<ScalarAccess> {
        let member = parameters.push(
            LogicalType {
                family: Family::String,
                facets: json!({}),
                nullable: false,
            },
            self.property_id.into(),
            json!({"sequenceProperty":self.property_id}),
        )?;
        let root = format!("{}.props", self.owner.sql());
        let leaf = format!("({root} -> {member}::text)");
        let expected = match self.item.family {
            Family::String => "string",
            Family::Boolean => "boolean",
            _ => "number",
        };
        let (native, empty, iterator, aggregate) = match self.kind {
            Kind::Sequence => ("array", "[]", "jsonb_array_elements", "jsonb_agg"),
            Kind::Map => ("object", "{}", "jsonb_each", "jsonb_object_agg"),
        };
        let safe = format!("CASE WHEN pg_catalog.jsonb_typeof({leaf})='{native}' THEN {leaf} ELSE '{empty}'::jsonb END");
        let value = match self.item.family {
            Family::Integer | Family::Decimal => "pg_catalog.to_jsonb((v #>> '{}')::text)",
            _ => "v",
        };
        let value_sql = if self.kind == Kind::Map {
            format!("(SELECT COALESCE(pg_catalog.{aggregate}(k COLLATE pg_catalog.\"C\",{value}),'{empty}'::jsonb) FROM pg_catalog.{iterator}({safe}) AS weft_items(k,v))")
        } else {
            format!("(SELECT COALESCE(pg_catalog.{aggregate}({value} ORDER BY ord),'{empty}'::jsonb) FROM pg_catalog.{iterator}({safe}) WITH ORDINALITY AS weft_items(v,ord))")
        };
        let items = if self.kind == Kind::Map {
            format!("pg_catalog.{iterator}({safe}) AS weft_items(k,v)")
        } else {
            format!("pg_catalog.{iterator}({safe}) AS weft_items(v)")
        };
        Ok(ScalarAccess{
            value:value_sql,
            present:format!("(CASE WHEN pg_catalog.jsonb_typeof({root})='object' THEN {root} ? {member}::text ELSE NULL END)"),native_null:format!("pg_catalog.jsonb_typeof({leaf})='null'"),
            integrity:format!("({root} IS NOT NULL AND pg_catalog.jsonb_typeof({root})='object' AND pg_catalog.jsonb_typeof({leaf})='{native}' AND NOT EXISTS (SELECT 1 FROM {items} WHERE pg_catalog.jsonb_typeof(v) IS DISTINCT FROM '{expected}'))"),joins:vec![]})
    }
    pub fn row(&self, parameters: &mut Parameters, occurrence: usize) -> Result<ScalarAccess> {
        if self.item.nullable {
            return Err(Diagnostic::new(
                "WFT-CAPABILITY",
                "lower",
                "Sequence item native null requires separate qualification",
            ));
        }
        let ty = parameters.catalog(
            CatalogDomain::Int,
            self.type_id,
            json!({"sequenceOwner":self.type_id}),
        )?;
        let property = parameters.catalog(
            CatalogDomain::Int,
            self.property_id,
            json!({"sequenceProperty":self.property_id}),
        )?;
        let state = Identifier::new(&format!("weft_state_{occurrence}"))?.sql();
        let node = Identifier::new(&format!("weft_node_{occurrence}"))?.sql();
        let table = |name| qualified(self.namespace, &Identifier::new(name).unwrap());
        let owner = self.owner.sql();
        let joins = vec![format!("LEFT JOIN {} AS {state} ON {state}.owner_kind='object' AND {state}.object_id={owner}.id AND {state}.object_type_id={owner}.type_id AND {state}.property_owner_type_id={ty}::int AND {state}.property_id={property}::int",table("row_home_state")),format!("LEFT JOIN {} AS {node} ON {node}.state_id={state}.state_id AND {node}.node_id={state}.root_node_id",table("row_home_node"))];
        let children = format!(
            "{} c LEFT JOIN {} p ON p.state_id=c.state_id AND p.node_id=c.node_id",
            table("row_home_node"),
            table("row_home_scalar")
        );
        let selected = format!("c.state_id={node}.state_id AND c.parent_node_id={node}.node_id");
        let (kind, column) = match self.item.family {
            Family::String => ("string", "text_value"),
            Family::Boolean => ("boolean", "boolean_value"),
            Family::Integer => ("integer", "numeric_value"),
            Family::Decimal => ("decimal", "numeric_value"),
        };
        let value = match self.item.family {
            Family::Integer | Family::Decimal => format!("pg_catalog.to_jsonb(p.{column}::text)"),
            _ => format!("pg_catalog.to_jsonb(p.{column})"),
        };
        let (root_kind, empty) = if self.kind == Kind::Map {
            ("map", "{}")
        } else {
            ("sequence", "[]")
        };
        let slot = if self.kind == Kind::Map {
            "c.slot_kind IS DISTINCT FROM 'map' OR c.map_key IS NULL OR c.sequence_ordinal IS NOT NULL"
        } else {
            "c.slot_kind IS DISTINCT FROM 'sequence' OR c.sequence_ordinal IS NULL OR c.sequence_ordinal<0 OR c.map_key IS NOT NULL"
        };
        let invalid = format!("{slot} OR c.value_kind IS DISTINCT FROM 'scalar' OR p.node_id IS NULL OR p.scalar_kind IS DISTINCT FROM '{kind}' OR p.{column} IS NULL");
        let slots = if self.kind == Kind::Map {
            "count(*)=count(DISTINCT c.map_key COLLATE pg_catalog.\"C\")"
        } else {
            "count(*)=count(DISTINCT c.sequence_ordinal) AND COALESCE(min(c.sequence_ordinal),0)=0 AND COALESCE(max(c.sequence_ordinal),-1)=count(*)-1"
        };
        let integrity = format!("({state}.state_id IS NOT NULL AND {node}.node_id IS NOT NULL AND {node}.parent_node_id IS NULL AND {node}.value_kind='{root_kind}' AND {node}.slot_kind='root' AND NOT EXISTS (SELECT 1 FROM {} root_payload WHERE root_payload.state_id={node}.state_id AND root_payload.node_id={node}.node_id) AND NOT EXISTS (SELECT 1 FROM {children} WHERE {selected} AND ({invalid})) AND (SELECT {slots} FROM {} c WHERE {selected}))",table("row_home_scalar"),table("row_home_node"));
        let aggregate = if self.kind == Kind::Map {
            format!("pg_catalog.jsonb_object_agg(c.map_key COLLATE pg_catalog.\"C\",{value})")
        } else {
            format!("pg_catalog.jsonb_agg({value} ORDER BY c.sequence_ordinal)")
        };
        let integrity = format!(
            "({integrity} AND {})",
            crate::tree::integrity(self.namespace, &state, &node)
        );
        Ok(ScalarAccess {
            value: format!(
                "(SELECT COALESCE({aggregate},'{empty}'::jsonb) FROM {children} WHERE {selected})"
            ),
            present: format!("{state}.state_id IS NOT NULL"),
            native_null: format!("{node}.value_kind='null'"),
            integrity,
            joins,
        })
    }
}
