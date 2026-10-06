//! Static physical selector correspondence for original row-home join definitions.
use crate::leaf_codec_definition::OriginalArtifact;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    error::{Diagnostic, Result},
    json::{checked_json, sha256},
};
#[jsonschema::validator(path = "../../tests/truss-postgresql/upstream/row-join-schema-bundle.json")]
struct Shape;
#[derive(Debug, Clone)]
pub struct Column {
    pub relation_identity: String,
    pub name: String,
}
/// The registry must establish this inventory's correspondence to its original
/// bytes. Shape validation alone cannot construct this trusted selection.
pub struct Selection<'a> {
    pub profile: &'a Value,
    pub original_artifacts: &'a BTreeMap<String, OriginalArtifact>,
    pub relations: &'a BTreeMap<String, String>,
    pub columns: &'a BTreeMap<String, Column>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum RecordKind {
    Object,
    Edge,
}
#[derive(Debug)]
pub struct Definition {
    pub original_json: String,
    pub original_artifacts: BTreeMap<String, Vec<u8>>,
    pub record_kind: RecordKind,
}
/// Physical root location; stored-domain integrity and codec decoding follow.
#[derive(Debug)]
pub struct RootLocation {
    pub joins: Vec<String>,
    /// Prerequisite in the same complete authorized view, never a query filter.
    pub structural_integrity: String,
    pub state_alias: crate::Identifier,
    pub node_alias: crate::Identifier,
    pub scalar_alias: crate::Identifier,
}
/// Raw custody for a selected scalar decoder. Native numeric output is text;
/// the original token remains independent. This is not a decoded logical value.
#[derive(Debug)]
pub struct ScalarObservation {
    pub present: String,
    pub kind: String,
    pub text: String,
    pub boolean: String,
    pub native_numeric_text: String,
    pub original_numeric_token: String,
    pub codec_bytes_hex: String,
    pub source_bytes_hex: String,
    pub other_payloads_absent: String,
}
impl RootLocation {
    pub fn scalar_observation(&self) -> ScalarObservation {
        let scalar = self.scalar_alias.sql();
        let column = |name: &str| format!("{scalar}.\"{name}\"");
        let hex = |name: &str| format!("pg_catalog.encode({},'hex')", column(name));
        ScalarObservation {
            present: format!("({} IS NOT NULL)", column("node_id")),
            kind: column("scalar_kind"),
            text: column("text_value"),
            boolean: column("boolean_value"),
            native_numeric_text: format!("{}::pg_catalog.text", column("numeric_value")),
            original_numeric_token: column("numeric_token"),
            codec_bytes_hex: hex("codec_definition_bytes"),
            source_bytes_hex: hex("original_source_bytes"),
            other_payloads_absent: format!(
                "({})",
                [
                    "binary_value",
                    "temporal_text",
                    "temporal_instant",
                    "opaque_bytes"
                ]
                .iter()
                .map(|name| format!("{} IS NULL", column(name)))
                .collect::<Vec<_>>()
                .join(" AND ")
            ),
        }
    }
}
/// Use the captured admitted join bytes, never a fresh registry lookup.
pub(crate) fn root_location(
    raw: &str,
    namespace: &crate::Identifier,
    owner: &crate::Identifier,
    owner_type_id: &str,
    property_id: &str,
    occurrence: usize,
    parameters: &mut crate::Parameters,
) -> Result<RootLocation> {
    let value = checked_json(raw).map_err(|_| fail("Captured row join JSON refused"))?;
    if !Shape::is_valid(&value) {
        return Err(fail("Captured row join grammar refused"));
    }
    // Validate both domains before mutating the caller's parameter collection.
    let mut validation = crate::Parameters::default();
    validation.catalog(
        crate::CatalogDomain::Int,
        owner_type_id,
        serde_json::json!({}),
    )?;
    validation.catalog(
        crate::CatalogDomain::Int,
        property_id,
        serde_json::json!({}),
    )?;
    let state_alias = crate::Identifier::new(&format!("weft_state_{occurrence}"))?;
    let node_alias = crate::Identifier::new(&format!("weft_node_{occurrence}"))?;
    let scalar_alias = crate::Identifier::new(&format!("weft_scalar_{occurrence}"))?;
    let observed_state = crate::Identifier::new(&format!("weft_check_state_{occurrence}"))?;
    let observed_node = crate::Identifier::new(&format!("weft_check_node_{occurrence}"))?;
    let observed_scalar = crate::Identifier::new(&format!("weft_check_scalar_{occurrence}"))?;
    if [
        &state_alias,
        &node_alias,
        &scalar_alias,
        &observed_state,
        &observed_node,
        &observed_scalar,
    ]
    .iter()
    .any(|alias| *alias == owner)
    {
        return Err(fail("Owner alias collides with native row access aliases"));
    }
    let table = |role: &str| -> Result<String> {
        Ok(crate::qualified(
            namespace,
            &crate::Identifier::new(value[role]["relationName"].as_str().unwrap())?,
        ))
    };
    let state_table = table("state")?;
    let node_table = table("node")?;
    let scalar_table = table("scalar")?;
    let mut staged = parameters.clone();
    let type_slot = staged.catalog(
        crate::CatalogDomain::Int,
        owner_type_id,
        serde_json::json!({"typeId":owner_type_id}),
    )?;
    let property_slot = staged.catalog(
        crate::CatalogDomain::Int,
        property_id,
        serde_json::json!({"propertyId":property_id}),
    )?;
    let (kind, id_column, discriminator, owner_discriminator) = if value["recordKind"] == "object" {
        ("object", "object_id", "object_type_id", "type_id")
    } else {
        ("edge", "edge_id", "relationship_type_id", "rel_type_id")
    };
    let state = state_alias.sql();
    let node = node_alias.sql();
    let scalar = scalar_alias.sql();
    let owner = owner.sql();
    let joins = vec![
        format!("LEFT JOIN {state_table} AS {state} ON {state}.\"owner_kind\"='{kind}' AND {state}.\"{id_column}\"={owner}.\"id\" AND {state}.\"{discriminator}\"={owner}.\"{owner_discriminator}\" AND {state}.\"property_owner_type_id\"={type_slot}::pg_catalog.int4 AND {state}.\"property_id\"={property_slot}::pg_catalog.int4"),
        format!("LEFT JOIN {node_table} AS {node} ON {node}.\"state_id\"={state}.\"state_id\" AND {node}.\"node_id\"={state}.\"root_node_id\""),
        format!("LEFT JOIN {scalar_table} AS {scalar} ON {scalar}.\"state_id\"={node}.\"state_id\" AND {scalar}.\"node_id\"={node}.\"node_id\""),
    ];
    let observed_state = observed_state.sql();
    let observed_node = observed_node.sql();
    let observed_scalar = observed_scalar.sql();
    let state_count = format!("(SELECT count(*) FROM {state_table} AS {observed_state} WHERE {observed_state}.\"owner_kind\"='{kind}' AND {observed_state}.\"{id_column}\"={owner}.\"id\" AND {observed_state}.\"{discriminator}\"={owner}.\"{owner_discriminator}\" AND {observed_state}.\"property_owner_type_id\"={type_slot}::pg_catalog.int4 AND {observed_state}.\"property_id\"={property_slot}::pg_catalog.int4)");
    let node_count = format!("(SELECT count(*) FROM {node_table} AS {observed_node} WHERE {observed_node}.\"state_id\"={state}.\"state_id\" AND {observed_node}.\"node_id\"={state}.\"root_node_id\")");
    let scalar_count = format!("(SELECT count(*) FROM {scalar_table} AS {observed_scalar} WHERE {observed_scalar}.\"state_id\"={node}.\"state_id\" AND {observed_scalar}.\"node_id\"={node}.\"node_id\")");
    let structural_integrity = format!("(({state_count}=0 AND {state}.\"state_id\" IS NULL) OR ({state_count}=1 AND {state}.\"state_id\" IS NOT NULL AND {node}.\"node_id\" IS NOT NULL AND {node}.\"parent_node_id\" IS NULL AND {node_count}=1 AND {scalar_count}<=1))");
    *parameters = staged;
    Ok(RootLocation {
        joins,
        structural_integrity,
        state_alias,
        node_alias,
        scalar_alias,
    })
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
impl Definition {
    /// Compose the admitted join with its enclosing original property home.
    /// This does not establish the runtime procedure behind a host obligation.
    pub fn verify_home(
        &self,
        binding: &crate::binding::Admission,
        index: usize,
        registered_obligations: &BTreeSet<String>,
        edge_association_profile: Option<&Value>,
    ) -> Result<()> {
        let home = binding.original_home_definition(index)?;
        let property = &binding.value["properties"][index];
        let definition = checked_json(&self.original_json)
            .map_err(|_| fail("Original row join JSON refused"))?;
        if property["home"] != "row"
            || home["recordKind"] != definition["recordKind"]
            || home["joinProfile"] != definition["profile"]
            || home["layoutInventory"] != definition["layoutInventory"]
            || home["valueDefinition"] != property["valueDefinition"]
            || home["presenceDefinition"] != property["presenceDefinition"]
        {
            return Err(fail(
                "Row home differs from its original selected join/value/presence",
            ));
        }
        for (role, key) in [
            ("state", "stateRelationPhysicalIdentity"),
            ("node", "nodeRelationPhysicalIdentity"),
            ("scalar", "scalarRelationPhysicalIdentity"),
        ] {
            if home[key] != definition[role]["relationPhysicalIdentity"] {
                return Err(fail("Row home redirects an admitted join relation"));
            }
        }
        let artifact = &home["joinDefinition"];
        let encoded = artifact["bytesBase64"]
            .as_str()
            .ok_or_else(|| fail("Original join artifact is not base64 text"))?;
        let bytes = STANDARD
            .decode(encoded)
            .map_err(|_| fail("Original join artifact base64 refused"))?;
        if STANDARD.encode(&bytes) != encoded
            || artifact["sha256"] != sha256(&bytes)
            || bytes != self.original_json.as_bytes()
        {
            return Err(fail(
                "Row home join artifact differs from admitted original bytes",
            ));
        }
        if !registered_obligations.contains(home["storedDomainObligation"].as_str().unwrap()) {
            return Err(fail(
                "Row home stored-domain obligation has no registered meaning",
            ));
        }
        if self.record_kind == RecordKind::Edge
            && (edge_association_profile
                .is_none_or(|profile| &home["edgeAssociationProfile"] != profile)
                || home["edgeAssociationDefinition"] != definition["edgeAssociationDefinition"])
        {
            return Err(fail(
                "Row home edge association differs from registered original meaning",
            ));
        }
        Ok(())
    }
    pub fn parse(raw: &str, selected: Selection<'_>) -> Result<Self> {
        if raw.len() > 4 * 1024 * 1024 {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "binding",
                "Row join exceeds candidate byte bound",
            ));
        }
        let value =
            checked_json(raw).map_err(|_| fail("Malformed or duplicate-member row join"))?;
        if !Shape::is_valid(&value) || &value["profile"] != selected.profile {
            return Err(fail("Row join grammar or registered profile differs"));
        }
        let record_kind = if value["recordKind"] == "object" {
            RecordKind::Object
        } else {
            RecordKind::Edge
        };
        let mut relations = BTreeSet::new();
        let mut columns = BTreeSet::new();
        for role in ["owner", "state", "node", "scalar"] {
            let relation = &value[role];
            let id = relation["relationPhysicalIdentity"].as_str().unwrap();
            if !relations.insert(id)
                || selected
                    .relations
                    .get(id)
                    .is_none_or(|name| relation["relationName"] != *name)
            {
                return Err(fail(
                    "Row join relation differs from original registered physical inventory",
                ));
            }
            let mut check = |physical: &Value, name: &str| -> Result<()> {
                let column_id = physical
                    .as_str()
                    .ok_or_else(|| fail("Column physical identity is not text"))?;
                if !columns.insert(column_id.to_string())
                    || selected
                        .columns
                        .get(column_id)
                        .is_none_or(|column| column.relation_identity != id || column.name != name)
                {
                    return Err(fail(
                        "Row join column differs from its original relation/name association",
                    ));
                }
                Ok(())
            };
            if role == "owner" {
                check(&relation["idColumnPhysicalIdentity"], "id")?;
                check(
                    &relation["discriminatorColumnPhysicalIdentity"],
                    if record_kind == RecordKind::Object {
                        "type_id"
                    } else {
                        "rel_type_id"
                    },
                )?;
            } else {
                for (name, physical) in relation["columns"].as_object().unwrap() {
                    check(physical, name)?;
                }
            }
        }
        let mut paths = vec![
            "layoutInventory".to_string(),
            "storedDomainDefinition".into(),
        ];
        for index in 0..value["constraintEvidence"].as_array().unwrap().len() {
            paths.push(format!("constraintEvidence/{index}"));
        }
        if record_kind == RecordKind::Edge {
            paths.push("edgeAssociationDefinition".into());
        }
        if paths.len() != selected.original_artifacts.len() {
            return Err(fail("Row join original artifact closure differs"));
        }
        let mut artifacts = BTreeMap::new();
        let mut total = 0usize;
        for path in paths {
            let mut artifact = &value;
            for component in path.split('/') {
                artifact = if let Some(array) = artifact.as_array() {
                    array
                        .get(
                            component
                                .parse::<usize>()
                                .map_err(|_| fail("Invalid artifact index"))?,
                        )
                        .ok_or_else(|| fail("Missing original artifact"))?
                } else {
                    &artifact[component]
                };
            }
            let encoded = artifact["bytesBase64"].as_str().unwrap();
            let bytes = STANDARD
                .decode(encoded)
                .map_err(|_| fail("Row join artifact base64 refused"))?;
            total = total
                .checked_add(bytes.len())
                .ok_or_else(|| fail("Row join artifact accounting overflow"))?;
            if total > 4 * 1024 * 1024 {
                return Err(Diagnostic::new(
                    "WFT-LIMIT",
                    "binding",
                    "Row join artifact bound exceeded",
                ));
            }
            if STANDARD.encode(&bytes) != encoded
                || artifact["sha256"] != sha256(&bytes)
                || selected
                    .original_artifacts
                    .get(&path)
                    .is_none_or(|original| {
                        original.bytes != bytes || artifact["identity"] != original.identity
                    })
            {
                return Err(fail(
                    "Row join artifact differs from original registered selection",
                ));
            }
            artifacts.insert(path, bytes);
        }
        Ok(Self {
            original_json: raw.into(),
            original_artifacts: artifacts,
            record_kind,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    struct Fixture {
        value: Value,
        artifacts: BTreeMap<String, OriginalArtifact>,
        relations: BTreeMap<String, String>,
        columns: BTreeMap<String, Column>,
    }
    fn fixture(edge: bool) -> Fixture {
        let bundle: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/upstream/row-join-schema-bundle.json"
        ))
        .unwrap();
        let schema = &bundle["$defs"]["resource1"];
        let mut value = json!({});
        for (key, rule) in schema["properties"].as_object().unwrap() {
            if let Some(constant) = rule.get("const") {
                value[key] = constant.clone();
            }
        }
        let pin = json!({"identity":"fixture","version":"0.1.0","sha256":sha256(b"{}")});
        let artifact = json!({"identity":"fixture","bytesBase64":STANDARD.encode(b"{}"),"sha256":sha256(b"{}")});
        value["profile"] = pin;
        value["recordKind"] = json!(if edge { "edge" } else { "object" });
        value["ownerJoin"] = json!(if edge {
            "edge-id-and-relationship"
        } else {
            "object-id-and-type"
        });
        let mut relations = BTreeMap::new();
        let mut columns = BTreeMap::new();
        for role in ["owner", "state", "node", "scalar"] {
            let name = if role == "owner" {
                if edge {
                    "edge"
                } else {
                    "object"
                }
            } else {
                schema["properties"][role]["properties"]["relationName"]["const"]
                    .as_str()
                    .unwrap()
            };
            let id = format!("relation:{name}");
            relations.insert(id.clone(), name.into());
            let mut relation = json!({"relationPhysicalIdentity":id,"relationName":name});
            if role == "owner" {
                for (key, name) in [
                    ("idColumnPhysicalIdentity", "id"),
                    (
                        "discriminatorColumnPhysicalIdentity",
                        if edge { "rel_type_id" } else { "type_id" },
                    ),
                ] {
                    let cid = format!("{id}:{name}");
                    columns.insert(
                        cid.clone(),
                        Column {
                            relation_identity: id.clone(),
                            name: name.into(),
                        },
                    );
                    relation[key] = json!(cid);
                }
            } else {
                relation["columns"] = json!({});
                for name in schema["properties"][role]["properties"]["columns"]["properties"]
                    .as_object()
                    .unwrap()
                    .keys()
                {
                    let cid = format!("{id}:{name}");
                    columns.insert(
                        cid.clone(),
                        Column {
                            relation_identity: id.clone(),
                            name: name.clone(),
                        },
                    );
                    relation["columns"][name] = json!(cid);
                }
            }
            value[role] = relation;
        }
        let mut artifacts: BTreeMap<String, OriginalArtifact> = [
            "layoutInventory",
            "storedDomainDefinition",
            "constraintEvidence/0",
        ]
        .into_iter()
        .map(|key| {
            (
                key.into(),
                OriginalArtifact {
                    identity: "fixture".into(),
                    bytes: b"{}".to_vec(),
                },
            )
        })
        .collect();
        value["layoutInventory"] = artifact.clone();
        value["storedDomainDefinition"] = artifact.clone();
        value["constraintEvidence"] = json!([artifact]);
        if edge {
            value["edgeAssociationDefinition"] = artifact;
            artifacts.insert(
                "edgeAssociationDefinition".into(),
                OriginalArtifact {
                    identity: "fixture".into(),
                    bytes: b"{}".to_vec(),
                },
            );
        }
        Fixture {
            value,
            artifacts,
            relations,
            columns,
        }
    }
    fn parse(value: &Value, f: &Fixture) -> Result<Definition> {
        Definition::parse(
            &value.to_string(),
            Selection {
                profile: &f.value["profile"],
                original_artifacts: &f.artifacts,
                relations: &f.relations,
                columns: &f.columns,
            },
        )
    }
    #[test]
    fn root_locations_keep_object_and_edge_ownership_and_atomic_slots() {
        let mut captures = Vec::new();
        for edge in [false, true] {
            let f = fixture(edge);
            let mut parameters = crate::Parameters::default();
            let location = root_location(
                &f.value.to_string(),
                &crate::Identifier::new("schema.with.dot").unwrap(),
                &crate::Identifier::new("owner").unwrap(),
                "-1",
                "42",
                3,
                &mut parameters,
            )
            .unwrap();
            assert_eq!(location.joins.len(), 3);
            let observation = location.scalar_observation();
            assert_eq!(
                observation.native_numeric_text,
                "\"weft_scalar_3\".\"numeric_value\"::pg_catalog.text"
            );
            assert_eq!(
                observation.original_numeric_token,
                "\"weft_scalar_3\".\"numeric_token\""
            );
            assert!(observation.codec_bytes_hex.ends_with("'hex')"));
            assert!(observation
                .source_bytes_hex
                .contains("\"original_source_bytes\""));
            assert!(observation
                .other_payloads_absent
                .contains("\"opaque_bytes\" IS NULL"));
            assert!(location.structural_integrity.contains("count(*)"));
            assert!(location
                .structural_integrity
                .contains("\"parent_node_id\" IS NULL"));
            let mut collision_slots = crate::Parameters::default();
            assert!(root_location(
                &f.value.to_string(),
                &crate::Identifier::new("s").unwrap(),
                &crate::Identifier::new("weft_check_state_3").unwrap(),
                "1",
                "42",
                3,
                &mut collision_slots
            )
            .is_err());
            assert!(collision_slots.into_slots().is_empty());
            assert!(location.joins[0].contains("\"schema.with.dot\".\"row_home_state\""));
            assert!(location.joins[0].contains(if edge { "\"edge_id\"" } else { "\"object_id\"" }));
            assert!(location.joins[0].contains(if edge {
                "\"rel_type_id\""
            } else {
                "\"type_id\""
            }));
            parse(&f.value, &f).unwrap();
            captures.push(json!({"kind":if edge {"edge"} else {"object"},
                "joins":location.joins,"integrity":location.structural_integrity}));
            let slots = parameters.into_slots();
            assert_eq!(
                slots
                    .iter()
                    .map(|slot| slot.value.as_str())
                    .collect::<Vec<_>>(),
                vec!["-1", "42"]
            );
            let mut parameters = crate::Parameters::default();
            for _ in 0..1023 {
                parameters
                    .catalog(crate::CatalogDomain::Int, "1", json!({}))
                    .unwrap();
            }
            assert!(root_location(
                &f.value.to_string(),
                &crate::Identifier::new("s").unwrap(),
                &crate::Identifier::new("o").unwrap(),
                "1",
                "42",
                0,
                &mut parameters
            )
            .is_err());
            assert_eq!(parameters.into_slots().len(), 1023);
        }
        if let Ok(path) = std::env::var("WEFT_ROW_LOCATION_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
        }
    }
    #[test]
    fn complete_native_selector_closure_retains_originals_for_both_owner_kinds() {
        for edge in [false, true] {
            let f = fixture(edge);
            let definition = parse(&f.value, &f).unwrap();
            assert_eq!(
                definition.record_kind,
                if edge {
                    RecordKind::Edge
                } else {
                    RecordKind::Object
                }
            );
            assert_eq!(definition.original_json, f.value.to_string());
            assert_eq!(
                definition.original_artifacts.len(),
                if edge { 4 } else { 3 }
            );
        }
    }
    #[test]
    fn redirected_columns_foreign_inventory_missing_tokens_and_wrong_owner_refuse() {
        let f = fixture(false);
        let mut value = f.value.clone();
        value["scalar"]["columns"]["numeric_token"] =
            value["scalar"]["columns"]["numeric_value"].clone();
        assert!(parse(&value, &f).is_err());
        let mut value = f.value.clone();
        value["node"]["columns"]["state_id"] = value["state"]["columns"]["state_id"].clone();
        assert!(parse(&value, &f).is_err());
        let mut value = f.value.clone();
        value["scalar"]["columns"]
            .as_object_mut()
            .unwrap()
            .remove("original_source_bytes");
        assert!(parse(&value, &f).is_err());
        let mut value = f.value.clone();
        value["ownerJoin"] = json!("edge-id-and-relationship");
        assert!(parse(&value, &f).is_err());
        let mut value = f.value.clone();
        value["layoutInventory"] = json!({"identity":"fixture","bytesBase64":STANDARD.encode(b"changed"),"sha256":sha256(b"changed")});
        assert!(parse(&value, &f).is_err());
        let mut value = f.value.clone();
        value["edgeAssociationDefinition"] = value["layoutInventory"].clone();
        assert!(parse(&value, &f).is_err());
        let mut value = fixture(true).value;
        value
            .as_object_mut()
            .unwrap()
            .remove("edgeAssociationDefinition");
        assert!(parse(&value, &fixture(true)).is_err());
    }
    #[test]
    fn original_home_composes_exact_join_without_using_candidate_physical_ids() {
        use crate::binding::Admission;
        let mut f = fixture(false);
        let mut binding: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/binding-row.json"
        ))
        .unwrap();
        f.value["layoutInventory"] = binding["basis"]["layoutInventory"].clone();
        f.artifacts.get_mut("layoutInventory").unwrap().identity = binding["basis"]
            ["layoutInventory"]["identity"]
            .as_str()
            .unwrap()
            .into();
        let definition = parse(&f.value, &f).unwrap();
        let home_bytes = STANDARD
            .decode(
                binding["properties"][0]["homeDefinition"]["bytesBase64"]
                    .as_str()
                    .unwrap(),
            )
            .unwrap();
        let mut home: Value = serde_json::from_slice(&home_bytes).unwrap();
        home["joinProfile"] = f.value["profile"].clone();
        let bytes = definition.original_json.as_bytes();
        home["joinDefinition"] = json!({"identity":"original-selected-join","bytesBase64":STANDARD.encode(bytes),"sha256":sha256(bytes)});
        for (role, key) in [
            ("state", "stateRelationPhysicalIdentity"),
            ("node", "nodeRelationPhysicalIdentity"),
            ("scalar", "scalarRelationPhysicalIdentity"),
        ] {
            home[key] = f.value[role]["relationPhysicalIdentity"].clone();
        }
        let admit = |home: &Value| {
            let mut bound = binding.clone();
            let bytes = home.to_string().into_bytes();
            bound["properties"][0]["homeDefinition"]["bytesBase64"] =
                json!(STANDARD.encode(&bytes));
            bound["properties"][0]["homeDefinition"]["sha256"] = json!(sha256(&bytes));
            Admission::parse(&bound.to_string(), "candidate-test-binding/0.1.0").unwrap()
        };
        let obligations =
            BTreeSet::from([home["storedDomainObligation"].as_str().unwrap().to_string()]);
        let admitted = admit(&home);
        definition
            .verify_home(&admitted, 0, &obligations, None)
            .unwrap();
        let inventory = OriginalArtifact {
            identity: binding["basis"]["layoutInventory"]["identity"]
                .as_str()
                .unwrap()
                .into(),
            bytes: b"{}".to_vec(),
        };
        let home_admission = crate::property_definition::admit_home(
            &admitted,
            0,
            crate::property_definition::PhysicalSelection {
                profile: &binding["properties"][0]["homeProfile"],
                inventory: &inventory,
                relations: &f.relations,
                columns: &f.columns,
                row_join: Some(&definition),
                obligations: &obligations,
                edge_association: None,
            },
        )
        .unwrap();
        assert!(matches!(
            home_admission,
            crate::property_definition::HomeAdmission::Row { .. }
        ));
        let mut parameters = crate::Parameters::default();
        assert!(home_admission
            .props_location(&crate::Identifier::new("owner").unwrap(), &mut parameters)
            .is_err());
        assert!(parameters.into_slots().is_empty());
        assert!(admitted.property_home(0).is_err()); // Fixed candidate IDs remain a distinct profile.
        assert!(definition
            .verify_home(&admitted, 0, &BTreeSet::new(), None)
            .is_err());
        let mut wrong = home.clone();
        wrong["nodeRelationPhysicalIdentity"] = json!("foreign-node");
        assert!(definition
            .verify_home(&admit(&wrong), 0, &obligations, None)
            .is_err());
        let mut wrong = home.clone();
        wrong["joinDefinition"]["sha256"] = json!("0".repeat(64));
        assert!(definition
            .verify_home(&admit(&wrong), 0, &obligations, None)
            .is_err());
        let mut wrong = home.clone();
        wrong["joinProfile"]["identity"] = json!("foreign-profile");
        assert!(definition
            .verify_home(&admit(&wrong), 0, &obligations, None)
            .is_err());
        let mut wrong = home;
        wrong["valueDefinition"]["identity"] = json!("foreign-value");
        assert!(definition
            .verify_home(&admit(&wrong), 0, &obligations, None)
            .is_err());
    }
}
