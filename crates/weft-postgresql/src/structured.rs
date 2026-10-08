//! Candidate structured scalar-member codec; source/native authority is explicit.
use crate::{access::ScalarAccess, qualified, CatalogDomain, Identifier, Parameters};
use serde_json::json;
use weft_core::{
    error::Result,
    ir::{Family, Identity, LogicalType},
};
pub struct Member {
    pub name: String,
    pub identity: Identity,
    pub logical_type: LogicalType,
    pub optional: bool,
}
pub struct StructuredProperty<'a> {
    pub namespace: &'a Identifier,
    pub owner: &'a Identifier,
    pub type_id: &'a str,
    pub property_id: &'a str,
    pub members: &'a [Member],
}
fn string(parameters: &mut Parameters, value: String, origin: serde_json::Value) -> Result<String> {
    parameters.push(
        LogicalType {
            family: Family::String,
            facets: json!({}),
            nullable: false,
        },
        value,
        origin,
    )
}
impl StructuredProperty<'_> {
    pub fn props(&self, parameters: &mut Parameters) -> Result<ScalarAccess> {
        let property = string(
            parameters,
            self.property_id.into(),
            json!({"structuredProperty":self.property_id}),
        )?;
        let root = format!("{}.props", self.owner.sql());
        let leaf = format!("({root} -> {property}::text)");
        let safe = format!(
            "CASE WHEN pg_catalog.jsonb_typeof({leaf})='object' THEN {leaf} ELSE '{{}}'::jsonb END"
        );
        let mut projection = vec![];
        let mut guards = vec![];
        let mut keys = vec![];
        for member in self.members {
            let key = string(
                parameters,
                member.name.clone(),
                json!({"structuredMember":member.identity}),
            )?;
            keys.push(format!("{key}::text"));
            let child = format!("({leaf} -> {key}::text)");
            let expected = match member.logical_type.family {
                Family::String => "string",
                Family::Boolean => "boolean",
                _ => "number",
            };
            let scalar = match member.logical_type.family {
                Family::Integer | Family::Decimal => {
                    format!("pg_catalog.to_jsonb(({child} #>> '{{}}')::text)")
                }
                _ => child.clone(),
            };
            let present = format!("({leaf} ? {key}::text)");
            let valid = format!("pg_catalog.jsonb_typeof({child})='{expected}'");
            guards.push(if member.optional {
                format!("(NOT {present} OR {valid})")
            } else {
                valid
            });
            let value = if member.optional {
                format!("CASE WHEN NOT {present} THEN pg_catalog.jsonb_build_object('state','absent') ELSE pg_catalog.jsonb_build_object('state','value','value',{scalar}) END")
            } else {
                scalar
            };
            projection.extend([format!("{key}::text"), value]);
        }
        let unknown = if keys.is_empty() {
            format!("NOT EXISTS (SELECT 1 FROM pg_catalog.jsonb_object_keys({safe}))")
        } else {
            format!("NOT EXISTS (SELECT 1 FROM pg_catalog.jsonb_object_keys({safe}) AS names(k) WHERE k NOT IN ({}))",keys.join(", "))
        };
        Ok(ScalarAccess{value:format!("pg_catalog.jsonb_build_object({})",projection.join(", ")),present:format!("CASE WHEN pg_catalog.jsonb_typeof({root})='object' THEN {root} ? {property}::text ELSE NULL END"),native_null:format!("pg_catalog.jsonb_typeof({leaf})='null'"),integrity:format!("({root} IS NOT NULL AND pg_catalog.jsonb_typeof({root})='object' AND pg_catalog.jsonb_typeof({leaf})='object' AND {unknown} {})",guards.iter().map(|g|format!("AND ({g})")).collect::<Vec<_>>().join(" ")),joins:vec![]})
    }
    pub fn row(&self, parameters: &mut Parameters, occurrence: usize) -> Result<ScalarAccess> {
        let ty = parameters.catalog(
            CatalogDomain::Int,
            self.type_id,
            json!({"structuredOwner":self.type_id}),
        )?;
        let property = parameters.catalog(
            CatalogDomain::Int,
            self.property_id,
            json!({"structuredProperty":self.property_id}),
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
        let scope = format!("c.state_id={node}.state_id AND c.parent_node_id={node}.node_id");
        let mut projection = vec![];
        let mut guards = vec![];
        let mut identities = vec![];
        for member in self.members {
            let name = string(
                parameters,
                member.name.clone(),
                json!({"structuredMemberName":member.identity}),
            )?;
            let identity = string(
                parameters,
                json!(member.identity).to_string(),
                json!({"structuredMemberIdentity":member.identity}),
            )?;
            let bytes = format!("pg_catalog.convert_to({identity}::text,'UTF8')");
            identities.push(bytes.clone());
            let selected = format!("{scope} AND c.record_field_identity_bytes={bytes}");
            let (kind, column) = match member.logical_type.family {
                Family::String => ("string", "text_value"),
                Family::Boolean => ("boolean", "boolean_value"),
                Family::Integer => ("integer", "numeric_value"),
                Family::Decimal => ("decimal", "numeric_value"),
            };
            let scalar = match member.logical_type.family {
                Family::Integer | Family::Decimal => {
                    format!("pg_catalog.to_jsonb(p.{column}::text)")
                }
                _ => format!("pg_catalog.to_jsonb(p.{column})"),
            };
            let value = format!("(SELECT {scalar} FROM {children} WHERE {selected})");
            let present = format!(
                "EXISTS (SELECT 1 FROM {} c WHERE {selected})",
                table("row_home_node")
            );
            let count = format!(
                "(SELECT count(*) FROM {} c WHERE {selected})",
                table("row_home_node")
            );
            guards.push(format!("{count}{} AND NOT EXISTS (SELECT 1 FROM {children} WHERE {selected} AND (c.value_kind IS DISTINCT FROM 'scalar' OR p.node_id IS NULL OR p.scalar_kind IS DISTINCT FROM '{kind}' OR p.{column} IS NULL))",if member.optional {"<=1"} else {"=1"}));
            let value = if member.optional {
                format!("CASE WHEN NOT ({present}) THEN pg_catalog.jsonb_build_object('state','absent') ELSE pg_catalog.jsonb_build_object('state','value','value',{value}) END")
            } else {
                value
            };
            projection.extend([format!("{name}::text"), value]);
        }
        let unknown = if identities.is_empty() {
            "TRUE".into()
        } else {
            format!("c.record_field_identity_bytes IS NULL OR c.record_field_identity_bytes NOT IN ({})",identities.join(", "))
        };
        let integrity = format!("({state}.state_id IS NOT NULL AND {node}.node_id IS NOT NULL AND {node}.parent_node_id IS NULL AND {node}.slot_kind='root' AND {node}.value_kind='structured' AND NOT EXISTS (SELECT 1 FROM {} p WHERE p.state_id={node}.state_id AND p.node_id={node}.node_id) AND NOT EXISTS (SELECT 1 FROM {} c WHERE {scope} AND (c.slot_kind IS DISTINCT FROM 'record' OR c.sequence_ordinal IS NOT NULL OR c.map_key IS NOT NULL OR {unknown})) {})",table("row_home_scalar"),table("row_home_node"),guards.iter().map(|g|format!("AND ({g})")).collect::<Vec<_>>().join(" "));
        let integrity = format!(
            "({integrity} AND {})",
            crate::tree::integrity(self.namespace, &state, &node)
        );
        Ok(ScalarAccess {
            value: format!("pg_catalog.jsonb_build_object({})", projection.join(", ")),
            present: format!("{state}.state_id IS NOT NULL"),
            native_null: format!("{node}.value_kind='null'"),
            integrity,
            joins,
        })
    }
}
