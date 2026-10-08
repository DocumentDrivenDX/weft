//! Candidate scalar access templates for JSONB and typed row storage.
//! Returned integrity predicates are prerequisites, never query filters.
use crate::{qualified, CatalogDomain, Identifier, Parameters};
use serde_json::json;
use weft_core::{
    error::{Diagnostic, Result},
    ir::{Family, LogicalType},
};
#[derive(Debug, Clone)]
pub struct ScalarAccess {
    pub value: String,
    pub present: String,
    pub native_null: String,
    pub integrity: String,
    pub joins: Vec<String>,
}
pub struct ObjectProperty<'a> {
    pub namespace: &'a Identifier,
    pub owner_alias: &'a Identifier,
    pub type_id: &'a str,
    pub property_id: &'a str,
    pub logical_type: &'a LogicalType,
}
/// Exact native numeric predicates; no casts to a narrowing SQL domain.
fn numeric_domain(ty: &LogicalType, value: &str) -> Result<String> {
    let invalid = || Diagnostic::new("WFT-BINDING", "lower", "Numeric facets are incomplete");
    Ok(match ty.family {
        Family::Integer => {
            let width = &ty.facets["integerWidth"];
            let bits = width["bits"]
                .as_u64()
                .filter(|n| (1..=64).contains(n))
                .ok_or_else(invalid)?;
            let signed = width["signed"].as_bool().ok_or_else(invalid)?;
            let exponent = bits - u64::from(signed);
            let lower = if signed {
                format!("-pg_catalog.power(2::numeric,{exponent})")
            } else {
                "0::numeric".into()
            };
            format!("({value}=pg_catalog.trunc({value}) AND {value}>={lower} AND {value}<pg_catalog.power(2::numeric,{exponent}))")
        }
        Family::Decimal => {
            let precision = ty.facets["precision"]
                .as_u64()
                .filter(|n| (1..=28).contains(n))
                .ok_or_else(invalid)?;
            let scale = ty.facets["scale"]
                .as_u64()
                .filter(|n| *n <= precision)
                .ok_or_else(invalid)?;
            let exponent = precision - scale;
            format!("({value}=pg_catalog.trunc({value},{scale}) AND pg_catalog.abs({value})<pg_catalog.power(10::numeric,{exponent}))")
        }
        _ => "true".into(),
    })
}
impl ObjectProperty<'_> {
    /// Location templates consume a separately validated candidate mapping.
    /// A host must validate integrity and exact domain in the same admitted view
    /// before evaluating casts, filters or publishing any result.
    pub fn props(&self, parameters: &mut Parameters) -> Result<ScalarAccess> {
        self.require_scalar_domain()?;
        let mut native = Parameters::default();
        native.catalog(CatalogDomain::Int, self.property_id, json!({}))?;
        let member = parameters.push(
            LogicalType {
                family: Family::String,
                facets: json!({}),
                nullable: false,
            },
            self.property_id.into(),
            json!({"propertyId":self.property_id,"use":"jsonb-member"}),
        )?;
        let owner = self.owner_alias.sql();
        let root = format!("{owner}.\"props\"");
        let leaf = format!("({root} -> {member}::text)");
        let expected = match self.logical_type.family {
            Family::String => "string",
            Family::Boolean => "boolean",
            Family::Integer | Family::Decimal => "number",
        };
        let value = match self.logical_type.family {
            Family::String => format!("({root} ->> {member}::text) COLLATE pg_catalog.\"C\""),
            Family::Boolean => format!("({root} ->> {member}::text)::pg_catalog.bool"),
            Family::Integer | Family::Decimal => {
                format!("({root} ->> {member}::text)::pg_catalog.numeric")
            }
        };
        let domain = numeric_domain(self.logical_type, &value)?;
        let guard = format!(
            "CASE WHEN pg_catalog.jsonb_typeof({leaf})='{expected}' THEN {domain} ELSE false END"
        );
        Ok(ScalarAccess{value,present:format!("(CASE WHEN pg_catalog.jsonb_typeof({root})='object' THEN {root} ? {member}::text ELSE NULL END)"),native_null:format!("(pg_catalog.jsonb_typeof({leaf}) = 'null')"),integrity:format!("({root} IS NOT NULL AND pg_catalog.jsonb_typeof({root}) = 'object' AND pg_catalog.jsonb_typeof({leaf}) = '{expected}' AND ({guard}))"),joins:vec![]})
    }
    pub fn row(&self, parameters: &mut Parameters, occurrence: usize) -> Result<ScalarAccess> {
        self.require_scalar_domain()?;
        let ty = parameters.catalog(
            CatalogDomain::Int,
            self.type_id,
            json!({"typeId":self.type_id}),
        )?;
        let property = parameters.catalog(
            CatalogDomain::Int,
            self.property_id,
            json!({"propertyId":self.property_id}),
        )?;
        let state = Identifier::new(&format!("weft_state_{occurrence}"))?.sql();
        let node = Identifier::new(&format!("weft_node_{occurrence}"))?.sql();
        let scalar = Identifier::new(&format!("weft_scalar_{occurrence}"))?.sql();
        let owner = self.owner_alias.sql();
        let table = |name| qualified(self.namespace, &Identifier::new(name).unwrap());
        let joins=vec![format!("LEFT JOIN {} AS {state} ON {state}.owner_kind = 'object' AND {state}.object_id = {owner}.id AND {state}.object_type_id = {owner}.type_id AND {state}.property_owner_type_id = {ty}::int AND {state}.property_id = {property}::int",table("row_home_state")),format!("LEFT JOIN {} AS {node} ON {node}.state_id = {state}.state_id AND {node}.node_id = {state}.root_node_id",table("row_home_node")),format!("LEFT JOIN {} AS {scalar} ON {scalar}.state_id = {node}.state_id AND {scalar}.node_id = {node}.node_id",table("row_home_scalar"))];
        let (column, kind) = match self.logical_type.family {
            Family::String => ("text_value", "string"),
            Family::Boolean => ("boolean_value", "boolean"),
            Family::Integer => ("numeric_value", "integer"),
            Family::Decimal => ("numeric_value", "decimal"),
        };
        let value = if self.logical_type.family == Family::String {
            format!("{scalar}.{column} COLLATE pg_catalog.\"C\"")
        } else {
            format!("{scalar}.{column}")
        };
        let domain = numeric_domain(self.logical_type, &value)?;
        let exclusive = match self.logical_type.family {
            Family::String => {
                format!("{scalar}.boolean_value IS NULL AND {scalar}.numeric_value IS NULL")
            }
            Family::Boolean => {
                format!("{scalar}.text_value IS NULL AND {scalar}.numeric_value IS NULL")
            }
            Family::Integer | Family::Decimal => {
                format!("{scalar}.text_value IS NULL AND {scalar}.boolean_value IS NULL")
            }
        };
        let unique = format!("(SELECT count(*) FROM {} observed_state WHERE observed_state.owner_kind={state}.owner_kind AND observed_state.object_id={state}.object_id AND observed_state.object_type_id={state}.object_type_id AND observed_state.property_owner_type_id={state}.property_owner_type_id AND observed_state.property_id={state}.property_id)=1 AND (SELECT count(*) FROM {} observed_root WHERE observed_root.state_id={state}.state_id AND observed_root.node_id={state}.root_node_id)=1 AND (SELECT count(*) FROM {} observed_payload WHERE observed_payload.state_id={node}.state_id AND observed_payload.node_id={node}.node_id)=1", table("row_home_state"), table("row_home_node"), table("row_home_scalar"));
        let integrity=format!("({state}.state_id IS NOT NULL AND {node}.node_id IS NOT NULL AND {node}.parent_node_id IS NULL AND {node}.value_kind = 'scalar' AND {scalar}.node_id IS NOT NULL AND {scalar}.scalar_kind = '{kind}' AND {scalar}.{column} IS NOT NULL AND ({unique}) AND ({exclusive}) AND ({domain}))");
        Ok(ScalarAccess {
            value,
            present: format!("({state}.state_id IS NOT NULL)"),
            native_null: format!("({node}.value_kind = 'null' AND {scalar}.node_id IS NULL)"),
            integrity,
            joins,
        })
    }
    pub fn require_scalar_domain(&self) -> Result<()> {
        if self.logical_type.nullable {
            return Err(Diagnostic::new(
                "WFT-CAPABILITY",
                "lower",
                "Scalar template requires a separately qualified nonnull domain",
            ));
        }
        Ok(())
    }
}
