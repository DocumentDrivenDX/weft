//! Original relationship/key custody and trusted directed edge access.
use crate::{
    binding::Admission, leaf_codec_definition::OriginalArtifact,
    record_definition::RecordAdmission, row_join_definition::Column, Identifier, Parameters,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use weft_core::{
    application_model::RelationshipRead,
    error::{Diagnostic, Result},
    ir::Span,
    ir::{Family, LogicalType},
    json::{checked_json, sha256},
    model::Catalog,
    syntax::Name,
};
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
/// All physical meanings are selected by trusted Rust composition over an exact
/// inventory. Binding content cannot select executable procedures or SQL.
pub struct Selection<'a> {
    pub profile: &'a Value,
    pub inventory: &'a OriginalArtifact,
    pub relation_identity: &'a str,
    pub relations: &'a BTreeMap<String, String>,
    pub columns: &'a BTreeMap<String, Column>,
    pub relationship_type: &'a str,
    pub source_id: &'a str,
    pub source_type: &'a str,
    pub target_id: &'a str,
    pub target_type: &'a str,
}
#[derive(Debug)]
pub struct RelationshipAdmission {
    binding_sha256: String,
    read_json: Value,
    relation: String,
    relationship_id: String,
    from_type: String,
    to_type: String,
    relationship_type: Identifier,
    from_id: Identifier,
    from_type_column: Identifier,
    to_id: Identifier,
    to_type_column: Identifier,
}
impl RelationshipAdmission {
    pub fn admit(
        binding: &Admission,
        index: usize,
        catalog: &Catalog,
        read: &RelationshipRead,
        records: &BTreeMap<String, RecordAdmission>,
        selected: Selection<'_>,
    ) -> Result<Self> {
        let physical = binding.value["relationships"]
            .get(index)
            .ok_or_else(|| fail("Original relationship mapping missing"))?;
        if physical["logical"] != json!(read.identity)
            || &physical["relationshipProfile"] != selected.profile
        {
            return Err(fail(
                "Original relationship identity or selected profile differs",
            ));
        }
        let input = catalog
            .inputs()
            .iter()
            .find(|i| {
                i.pin.document_id == read.identity.document_id
                    && i.pin.revision == read.identity.revision
                    && i.selected_module_ids.contains(&read.identity.module)
            })
            .ok_or_else(|| fail("Original relationship model selection missing"))?;
        if sha256(input.document_json.as_bytes()) != input.pin.sha256 {
            return Err(fail("Original relationship model digest differs"));
        }
        let document = checked_json(&input.document_json)
            .map_err(|_| fail("Original relationship JSON refused"))?;
        let authored = document["modules"]
            .as_array()
            .and_then(|ms| ms.iter().find(|m| m["id"] == read.identity.module))
            .and_then(|m| m["relationships"].as_array())
            .and_then(|rs| rs.iter().find(|r| r["id"] == read.identity.relationship))
            .ok_or_else(|| fail("Original authored relationship missing"))?;
        if binding.decoded_json(&format!("/relationships/{index}/source"))? != document
            || binding.decoded_json(&format!("/relationships/{index}/acceptedDefinition"))?
                != *authored
        {
            return Err(fail(
                "Relationship source/definition differs from original UMF",
            ));
        }
        let source = catalog.record_by_identity(&read.from)?;
        let name = authored[if read.inverse { "inverse" } else { "name" }]
            .as_str()
            .ok_or_else(|| fail("Original traversal name missing"))?;
        let resolved = catalog.relationship_read(
            &source,
            &Name {
                value: name.into(),
                quoted: true,
                span: Span { start: 0, end: 0 },
            },
        )?;
        let read_json =
            serde_json::to_value(read).map_err(|_| fail("Relationship read encoding refused"))?;
        if serde_json::to_value(resolved)
            .map_err(|_| fail("Original relationship encoding refused"))?
            != read_json
        {
            return Err(fail(
                "Resolved relationship semantics differ from original UMF",
            ));
        }
        let roles = if read.inverse {
            [
                ("target", &read.from, &read.source_key),
                ("source", &read.to, &read.target_key),
            ]
        } else {
            [
                ("source", &read.from, &read.source_key),
                ("target", &read.to, &read.target_key),
            ]
        };
        for (role, identity, key) in roles {
            let record = records
                .get(
                    &serde_json::to_string(identity)
                        .map_err(|_| fail("Endpoint identity encoding refused"))?,
                )
                .ok_or_else(|| fail("Relationship endpoint lacks independent Record admission"))?;
            record.verify_key_mapping(binding, catalog, key)?;
            let entity = binding.value["entities"]
                .as_array()
                .and_then(|es| es.iter().find(|e| e["logical"] == json!(identity)))
                .ok_or_else(|| fail("Relationship endpoint mapping missing"))?;
            let ids: Vec<_> = key
                .fields
                .iter()
                .map(|field| {
                    binding.value["properties"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|p| {
                            p["ownerTypeId"] == entity["typeId"] && p["logical"] == json!(field)
                        })
                        .map(|p| p["propertyId"].clone())
                        .ok_or_else(|| fail("Relationship endpoint key property missing"))
                })
                .collect::<Result<_>>()?;
            if physical[format!("{role}TypeId")] != entity["typeId"]
                || physical[format!("{role}KeyId")] != key.id
                || physical[format!("{role}OrderedPropertyIds")] != json!(ids)
            {
                return Err(fail(
                    "Relationship endpoint/key order differs from original mapping",
                ));
            }
        }
        if binding.value["basis"]["layoutInventory"]["identity"] != selected.inventory.identity
            || binding.artifacts.get("/basis/layoutInventory") != Some(&selected.inventory.bytes)
        {
            return Err(fail("Relationship physical inventory differs"));
        }
        let table = selected
            .relations
            .get(selected.relation_identity)
            .ok_or_else(|| fail("Selected edge relation missing"))?;
        let column = |id: &str| -> Result<Identifier> {
            let column = selected
                .columns
                .get(id)
                .filter(|c| c.relation_identity == selected.relation_identity)
                .ok_or_else(|| fail("Selected edge column belongs to another relation"))?;
            Identifier::new(&column.name)
        };
        let role_ids = [
            selected.relationship_type,
            selected.source_id,
            selected.source_type,
            selected.target_id,
            selected.target_type,
        ];
        let mut names = std::collections::BTreeSet::new();
        for id in role_ids {
            if !names.insert(column(id)?.sql()) {
                return Err(fail("Selected edge roles alias one physical column"));
            }
        }
        let (from_id, from_type_column, to_id, to_type_column, from_type, to_type) = if read.inverse
        {
            (
                selected.target_id,
                selected.target_type,
                selected.source_id,
                selected.source_type,
                "targetTypeId",
                "sourceTypeId",
            )
        } else {
            (
                selected.source_id,
                selected.source_type,
                selected.target_id,
                selected.target_type,
                "sourceTypeId",
                "targetTypeId",
            )
        };
        Ok(Self {
            binding_sha256: sha256(binding.original_json.as_bytes()),
            read_json,
            relation: crate::qualified(
                &Identifier::new(
                    binding.value["basis"]["namespace"]
                        .as_str()
                        .ok_or_else(|| fail("Binding namespace missing"))?,
                )?,
                &Identifier::new(table)?,
            ),
            relationship_id: physical["relationshipId"].as_str().unwrap().into(),
            from_type: physical[from_type].as_str().unwrap().into(),
            to_type: physical[to_type].as_str().unwrap().into(),
            relationship_type: column(selected.relationship_type)?,
            from_id: column(from_id)?,
            from_type_column: column(from_type_column)?,
            to_id: column(to_id)?,
            to_type_column: column(to_type_column)?,
        })
    }
    /// Prepare a separate key-reading target scan using the original target
    /// Record and every authored key component. Target fields retain independent
    /// admissions even when source and target keys have different arities.
    pub fn prepare_target<'a>(
        &self,
        context: &weft_core::backend::Context<'_>,
        read: &RelationshipRead,
        records: &BTreeMap<String, RecordAdmission>,
        properties: &'a BTreeMap<String, crate::property_definition::PropertyAdmission>,
        comparators: &BTreeMap<String, crate::native_comparator_definition::Definition>,
        parameters: &mut Parameters,
        scope: u16,
    ) -> Result<PreparedTarget<'a>> {
        if context.binding.sha256 != self.binding_sha256
            || serde_json::to_value(read).map_err(|_| fail("Relationship encoding refused"))?
                != self.read_json
        {
            return Err(fail(
                "Target preparation substitutes admitted relationship cut",
            ));
        }
        let record = context.catalog.record_by_identity(&read.to)?;
        let binding = Admission::parse(&context.binding.json, &context.binding.profile)?;
        records
            .get(
                &serde_json::to_string(&read.to)
                    .map_err(|_| fail("Target identity encoding refused"))?,
            )
            .ok_or_else(|| fail("Target Record admission missing"))?
            .verify_key_mapping(&binding, context.catalog, &read.target_key)?;
        let occurrence = format!("relationship-target-{scope}");
        let mut graph = BTreeMap::new();
        for identity in &read.target_key.fields {
            let (_, descriptors) = context
                .catalog
                .member_descriptor_by_identity(&record, identity)?;
            for descriptor in descriptors {
                graph.insert(
                    serde_json::to_string(&descriptor.identity)
                        .map_err(|_| fail("Target descriptor encoding refused"))?,
                    descriptor,
                );
            }
        }
        let fields: Vec<_> = read
            .target_key
            .fields
            .iter()
            .zip(&read.target_key.types)
            .map(
                |(identity, logical_type)| weft_core::application_ir::Field {
                    scan: occurrence.clone(),
                    identity: identity.clone(),
                    logical_type: logical_type.clone(),
                    span: Span { start: 0, end: 0 },
                },
            )
            .collect();
        let plan = weft_core::application_ir::Plan {
            ir_version: "weft-ir/0.2.0".into(),
            module_pins: vec![record.pin.clone()],
            read_profile: None,
            required_capabilities: vec![
                "scan".into(),
                "project".into(),
                "key.uniqueStable".into(),
                "order.asc".into(),
            ],
            type_graph: graph.into_values().collect(),
            source: weft_core::application_ir::Scan {
                occurrence: occurrence.clone(),
                record: read.to.clone(),
                pin: record.pin,
            },
            page_key: Some(read.target_key.clone()),
            joins: vec![],
            filters: vec![],
            groups: vec![],
            aggregate: false,
            outputs: fields
                .iter()
                .enumerate()
                .map(|(i, f)| weft_core::application_ir::Output {
                    name: format!("key_{i}"),
                    expression: weft_core::application_ir::Expression::Field {
                        scan: occurrence.clone(),
                        identity: f.identity.clone(),
                    },
                })
                .collect(),
            order: fields,
            limit: None,
        };
        let target_context = weft_core::backend::Context {
            plan: weft_core::backend::Plan::V02(&plan),
            ..*context
        };
        let prepared = crate::registered_access::prepare_relationship_target(
            &target_context,
            records,
            properties,
            comparators,
            parameters,
            scope,
        )?;
        Ok(PreparedTarget { plan, prepared })
    }
    /// Observe all selected relationship edges and owners independently of
    /// result filters/limits. Hosts must enforce zero violations before results.
    pub fn integrity_checks(
        &self,
        binding: &Admission,
        read: &RelationshipRead,
        parameters: &mut Parameters,
    ) -> Result<Vec<String>> {
        if sha256(binding.original_json.as_bytes()) != self.binding_sha256
            || serde_json::to_value(read).map_err(|_| fail("Relationship encoding refused"))?
                != self.read_json
        {
            return Err(fail("Relationship integrity substitutes original cut"));
        }
        let mut staged = parameters.clone();
        let rel = staged.catalog(
            crate::CatalogDomain::Int,
            &self.relationship_id,
            json!({"relationshipGuard":"discriminator"}),
        )?;
        let from = staged.catalog(
            crate::CatalogDomain::Int,
            &self.from_type,
            json!({"relationshipGuard":"from-type"}),
        )?;
        let to = staged.catalog(
            crate::CatalogDomain::Int,
            &self.to_type,
            json!({"relationshipGuard":"to-type"}),
        )?;
        let object = crate::qualified(
            &Identifier::new(binding.value["basis"]["namespace"].as_str().unwrap())?,
            &Identifier::new("object")?,
        );
        let e = "weft_guard_edge";
        let f = "weft_guard_from";
        let t = "weft_guard_to";
        let endpoint=format!("SELECT count(*) AS violations FROM {} {e} LEFT JOIN {object} {f} ON {f}.id={e}.{} AND {f}.type_id={e}.{} LEFT JOIN {object} {t} ON {t}.id={e}.{} AND {t}.type_id={e}.{} WHERE {e}.{}={rel}::pg_catalog.int4 AND ({e}.{} IS DISTINCT FROM {from}::pg_catalog.int4 OR {e}.{} IS DISTINCT FROM {to}::pg_catalog.int4 OR {f}.id IS NULL OR {t}.id IS NULL)",self.relation,self.from_id.sql(),self.from_type_column.sql(),self.to_id.sql(),self.to_type_column.sql(),self.relationship_type.sql(),self.from_type_column.sql(),self.to_type_column.sql());
        let multiplicity = if read.inverse {
            &read.source_multiplicity
        } else {
            &read.target_multiplicity
        };
        let count_type = LogicalType {
            family: Family::Integer,
            nullable: false,
            facets: json!({"integerWidth":{"bits":64,"signed":false}}),
        };
        let minimum = staged.push(
            count_type.clone(),
            multiplicity["min"]
                .as_u64()
                .ok_or_else(|| fail("Original minimum multiplicity missing"))?
                .to_string(),
            json!({"relationshipGuard":"minimum"}),
        )?;
        let mut domain = format!(
            "pg_catalog.count({e}.{}) < {minimum}::pg_catalog.numeric",
            self.from_id.sql()
        );
        if multiplicity["max"] != "*" {
            let maximum = staged.push(
                count_type,
                multiplicity["max"]
                    .as_u64()
                    .ok_or_else(|| fail("Original maximum multiplicity missing"))?
                    .to_string(),
                json!({"relationshipGuard":"maximum"}),
            )?;
            domain.push_str(&format!(
                " OR pg_catalog.count({e}.{}) > {maximum}::pg_catalog.numeric",
                self.from_id.sql()
            ));
        }
        let cardinality=format!("SELECT count(*) AS violations FROM (SELECT {f}.id FROM {object} {f} LEFT JOIN {} {e} ON {e}.{}={f}.id AND {e}.{}={f}.type_id AND {e}.{}={rel}::pg_catalog.int4 AND {e}.{}={to}::pg_catalog.int4 WHERE {f}.type_id={from}::pg_catalog.int4 GROUP BY {f}.id,{f}.type_id HAVING {domain}) AS weft_guard_cardinality",self.relation,self.from_id.sql(),self.from_type_column.sql(),self.relationship_type.sql(),self.to_type_column.sql());
        *parameters = staged;
        Ok(vec![endpoint, cardinality])
    }
    /// Direction and binding cut must match the admitted original traversal.
    /// Correlation uses native object IDs, never logical key tokens.
    pub fn correlate(
        &self,
        binding: &Admission,
        read: &RelationshipRead,
        edge_alias: &Identifier,
        source_alias: &Identifier,
        source_object_id: &Identifier,
        source_object_type: &Identifier,
        parameters: &mut Parameters,
    ) -> Result<EdgeAccess> {
        if sha256(binding.original_json.as_bytes()) != self.binding_sha256
            || serde_json::to_value(read).map_err(|_| fail("Relationship encoding refused"))?
                != self.read_json
        {
            return Err(fail("Relationship access substitutes binding or traversal"));
        }
        let mut staged = parameters.clone();
        let ty = LogicalType {
            family: Family::Integer,
            nullable: false,
            facets: json!({"integerWidth":{"bits":32,"signed":true}}),
        };
        let mut slot = |value: &str, role: &str| {
            staged.push(
                ty.clone(),
                value.into(),
                json!({"relationship":read.identity,"role":role}),
            )
        };
        let rel = slot(&self.relationship_id, "relationship")?;
        let from = slot(&self.from_type, "from-type")?;
        let to = slot(&self.to_type, "to-type")?;
        let e = edge_alias.sql();
        let predicates = vec![
            format!(
                "{e}.{}={rel}::pg_catalog.int4",
                self.relationship_type.sql()
            ),
            format!(
                "{e}.{}={from}::pg_catalog.int4",
                self.from_type_column.sql()
            ),
            format!("{e}.{}={to}::pg_catalog.int4", self.to_type_column.sql()),
            format!(
                "{e}.{}={}.{}",
                self.from_id.sql(),
                source_alias.sql(),
                source_object_id.sql()
            ),
        ];
        let mut predicates = predicates;
        predicates.push(format!(
            "{}.{}={e}.{}",
            source_alias.sql(),
            source_object_type.sql(),
            self.from_type_column.sql()
        ));
        let access = EdgeAccess {
            source: format!("{} AS {e}", self.relation),
            predicates,
            target_object_id: format!("{e}.{}", self.to_id.sql()),
            target_type: format!("{e}.{}", self.to_type_column.sql()),
        };
        *parameters = staged;
        Ok(access)
    }
}
#[derive(Debug)]
pub struct EdgeAccess {
    pub source: String,
    pub predicates: Vec<String>,
    pub target_object_id: String,
    pub target_type: String,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_directed_edges_preserve_direction_keys_and_custody() {
        let cases: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/application-cases.json"
        ))
        .unwrap();
        let fixture = cases
            .iter()
            .find(|c| c["id"] == "related-page-props")
            .unwrap();
        let catalog = Catalog::prepare(
            serde_json::from_value(fixture["request"]["modules"].clone()).unwrap(),
        )
        .unwrap();
        let binding = Admission::parse(
            fixture["request"]["target"]["bindingJson"]
                .as_str()
                .unwrap(),
            checked_json(
                fixture["request"]["target"]["bindingJson"]
                    .as_str()
                    .unwrap(),
            )
            .unwrap()["bindingProfileId"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        let inventory = OriginalArtifact {
            identity: binding.value["basis"]["layoutInventory"]["identity"]
                .as_str()
                .unwrap()
                .into(),
            bytes: binding.artifacts["/basis/layoutInventory"].clone(),
        };
        let relations = BTreeMap::from([
            ("objects".into(), "object".into()),
            ("edges".into(), "edge".into()),
        ]);
        let columns: BTreeMap<String, Column> = [
            ("type", "objects", "type_id"),
            ("rel", "edges", "rel_type_id"),
            ("src-id", "edges", "source_id"),
            ("src-type", "edges", "source_type"),
            ("dst-id", "edges", "target_id"),
            ("dst-type", "edges", "target_type"),
        ]
        .into_iter()
        .map(|(id, relation, name)| {
            (
                id.into(),
                Column {
                    relation_identity: relation.into(),
                    name: name.into(),
                },
            )
        })
        .collect();
        let records: BTreeMap<_, _> = (0..2)
            .map(|index| {
                let r = RecordAdmission::admit(
                    &binding,
                    index,
                    &catalog,
                    crate::record_definition::Selection {
                        inventory: &inventory,
                        relation_identity: "objects",
                        discriminator_identity: "type",
                        relations: &relations,
                        columns: &columns,
                    },
                )
                .unwrap();
                (serde_json::to_string(r.identity()).unwrap(), r)
            })
            .collect();
        let profile = &binding.value["relationships"][0]["relationshipProfile"];
        let select = || Selection {
            profile,
            inventory: &inventory,
            relation_identity: "edges",
            relations: &relations,
            columns: &columns,
            relationship_type: "rel",
            source_id: "src-id",
            source_type: "src-type",
            target_id: "dst-id",
            target_type: "dst-type",
        };
        let mut captures = Vec::new();
        for (record_name, relationship_name, inverse) in
            [("Customer", "orders", false), ("Orders", "customer", true)]
        {
            let name = |s: &str| Name {
                value: s.into(),
                quoted: true,
                span: Span { start: 0, end: 0 },
            };
            let owner = catalog.record(None, &name(record_name)).unwrap();
            let read = catalog
                .relationship_read(&owner, &name(relationship_name))
                .unwrap();
            assert_eq!(read.inverse, inverse);
            let admitted =
                RelationshipAdmission::admit(&binding, 0, &catalog, &read, &records, select())
                    .unwrap();
            admitted
                .verify_registration(
                    &sha256(binding.original_json.as_bytes()),
                    &registration_key(&read),
                )
                .unwrap();
            assert!(admitted
                .verify_registration(&sha256(binding.original_json.as_bytes()), "wrong")
                .is_err());
            assert!(admitted
                .verify_registration("wrong", &registration_key(&read))
                .is_err());
            let mut parameters = Parameters::default();
            let access = admitted
                .correlate(
                    &binding,
                    &read,
                    &Identifier::new("e").unwrap(),
                    &Identifier::new("o").unwrap(),
                    &Identifier::new("id").unwrap(),
                    &Identifier::new("type_id").unwrap(),
                    &mut parameters,
                )
                .unwrap();
            assert!(access.target_object_id.contains(if inverse {
                "source_id"
            } else {
                "target_id"
            }));
            assert_eq!(parameters.clone().into_slots().len(), 3);
            captures.push(json!({"inverse":inverse,"sql":format!("SELECT o.id::pg_catalog.text AS owner, {}::pg_catalog.text AS target FROM pg_temp.object o JOIN {} ON {} ORDER BY o.id, {}",access.target_object_id,access.source,access.predicates.join(" AND "),access.target_object_id),"parameters":parameters.clone().into_slots()}));
            let before = serde_json::to_value(parameters.clone().into_slots()).unwrap();
            let mut wrong = read.clone();
            wrong.inverse = !wrong.inverse;
            assert!(admitted
                .correlate(
                    &binding,
                    &wrong,
                    &Identifier::new("e").unwrap(),
                    &Identifier::new("o").unwrap(),
                    &Identifier::new("id").unwrap(),
                    &Identifier::new("type_id").unwrap(),
                    &mut parameters
                )
                .is_err());
            assert_eq!(
                serde_json::to_value(parameters.into_slots()).unwrap(),
                before
            );
            for mutation in 0..5 {
                let mut wrong = read.clone();
                match mutation {
                    0 => wrong.target_lifecycle = "owned".into(),
                    1 => wrong.target_multiplicity = json!({"min":1,"max":1}),
                    2 => wrong.source_key.fields.reverse(),
                    3 => wrong.to.revision = "wrong".into(),
                    _ => wrong.target_key.id = "wrong".into(),
                }
                // A single-field reverse is unchanged; explicitly substitute its identity.
                if mutation == 2 {
                    wrong.source_key.fields[0].element = "wrong".into();
                }
                assert!(RelationshipAdmission::admit(
                    &binding,
                    0,
                    &catalog,
                    &wrong,
                    &records,
                    select()
                )
                .is_err());
            }
            assert!(RelationshipAdmission::admit(
                &binding,
                0,
                &catalog,
                &read,
                &BTreeMap::new(),
                select()
            )
            .is_err());
            let mut aliased = columns.clone();
            aliased.get_mut("dst-id").unwrap().name = "source_id".into();
            assert!(RelationshipAdmission::admit(
                &binding,
                0,
                &catalog,
                &read,
                &records,
                Selection {
                    columns: &aliased,
                    ..select()
                }
            )
            .is_err());
            let mut foreign = columns.clone();
            foreign.get_mut("dst-id").unwrap().relation_identity = "objects".into();
            assert!(RelationshipAdmission::admit(
                &binding,
                0,
                &catalog,
                &read,
                &records,
                Selection {
                    columns: &foreign,
                    ..select()
                }
            )
            .is_err());
            let mut changed = binding.value.clone();
            changed["relationships"][0]["sourceOrderedPropertyIds"] = json!(["1"]);
            let changed = Admission::parse(
                &changed.to_string(),
                binding.value["bindingProfileId"].as_str().unwrap(),
            )
            .unwrap();
            assert!(
                RelationshipAdmission::admit(&changed, 0, &catalog, &read, &records, select())
                    .is_err()
            );
        }
        if let Ok(path) = std::env::var("WEFT_ORIGINAL_RELATIONSHIP_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
        }
    }
}

/// Target key preparation carries its exact synthetic plan for downstream
/// custody checks. It is not a detached target result query or publication gate.
pub struct PreparedTarget<'a> {
    pub plan: weft_core::application_ir::Plan,
    pub prepared: crate::registered_access::Prepared<'a>,
}

pub fn registration_key(read: &RelationshipRead) -> String {
    json!({"identity":read.identity,"inverse":read.inverse}).to_string()
}
impl RelationshipAdmission {
    pub(crate) fn verify_registration(&self, binding_sha256: &str, key: &str) -> Result<()> {
        let expected =
            json!({"identity":self.read_json["identity"],"inverse":self.read_json["inverse"]})
                .to_string();
        if self.binding_sha256 != binding_sha256 || key != expected {
            return Err(fail(
                "Relationship registration substitutes original cut or direction",
            ));
        }
        Ok(())
    }
}
#[cfg(test)]
mod finite_tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};
    #[test]
    fn finite_original_multiplicity_selects_the_correct_direction() {
        let cases: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/application-cases.json"
        ))
        .unwrap();
        let fixture = cases
            .iter()
            .find(|c| c["id"] == "related-page-props")
            .unwrap();
        let mut inputs: Vec<weft_core::model::ModuleInput> =
            serde_json::from_value(fixture["request"]["modules"].clone()).unwrap();
        let mut document = checked_json(&inputs[0].document_json).unwrap();
        document["modules"][0]["relationships"][0]["sourceMultiplicity"] = json!({"min":1,"max":2});
        document["modules"][0]["relationships"][0]["targetMultiplicity"] = json!({"min":0,"max":1});
        inputs[0].document_json = document.to_string();
        inputs[0].pin.sha256 = sha256(inputs[0].document_json.as_bytes());
        let catalog = Catalog::prepare(inputs).unwrap();
        let mut value = checked_json(
            fixture["request"]["target"]["bindingJson"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        for group in ["entities", "properties", "keys", "relationships"] {
            for mapped in value[group].as_array_mut().unwrap() {
                if mapped.get("source").is_none() {
                    continue;
                }
                let artifact = &mut mapped["source"];
                artifact["bytesBase64"] = json!(STANDARD.encode(document.to_string().as_bytes()));
                artifact["sha256"] = json!(sha256(document.to_string().as_bytes()));
            }
        }
        let bytes = document["modules"][0]["relationships"][0].to_string();
        value["relationships"][0]["acceptedDefinition"]["bytesBase64"] =
            json!(STANDARD.encode(bytes.as_bytes()));
        value["relationships"][0]["acceptedDefinition"]["sha256"] = json!(sha256(bytes.as_bytes()));
        let binding = Admission::parse(
            &value.to_string(),
            value["bindingProfileId"].as_str().unwrap(),
        )
        .unwrap();
        let inventory = OriginalArtifact {
            identity: value["basis"]["layoutInventory"]["identity"]
                .as_str()
                .unwrap()
                .into(),
            bytes: binding.artifacts["/basis/layoutInventory"].clone(),
        };
        let relations = BTreeMap::from([
            ("objects".into(), "object".into()),
            ("edges".into(), "edge".into()),
        ]);
        let columns: BTreeMap<_, _> = [
            ("type", "objects", "type_id"),
            ("rel", "edges", "rel_type_id"),
            ("src-id", "edges", "source_id"),
            ("src-type", "edges", "source_type"),
            ("dst-id", "edges", "target_id"),
            ("dst-type", "edges", "target_type"),
        ]
        .into_iter()
        .map(|(id, relation, name)| {
            (
                id.into(),
                Column {
                    relation_identity: relation.into(),
                    name: name.into(),
                },
            )
        })
        .collect();
        let records: BTreeMap<_, _> = (0..2)
            .map(|index| {
                let r = RecordAdmission::admit(
                    &binding,
                    index,
                    &catalog,
                    crate::record_definition::Selection {
                        inventory: &inventory,
                        relation_identity: "objects",
                        discriminator_identity: "type",
                        relations: &relations,
                        columns: &columns,
                    },
                )
                .unwrap();
                (serde_json::to_string(r.identity()).unwrap(), r)
            })
            .collect();
        let mut captures = Vec::new();
        for (owner, name, inverse) in [("Customer", "orders", false), ("Orders", "customer", true)]
        {
            let make_name = |s: &str| Name {
                value: s.into(),
                quoted: true,
                span: Span { start: 0, end: 0 },
            };
            let owner = catalog.record(None, &make_name(owner)).unwrap();
            let read = catalog.relationship_read(&owner, &make_name(name)).unwrap();
            let admitted = RelationshipAdmission::admit(
                &binding,
                0,
                &catalog,
                &read,
                &records,
                Selection {
                    profile: &value["relationships"][0]["relationshipProfile"],
                    inventory: &inventory,
                    relation_identity: "edges",
                    relations: &relations,
                    columns: &columns,
                    relationship_type: "rel",
                    source_id: "src-id",
                    source_type: "src-type",
                    target_id: "dst-id",
                    target_type: "dst-type",
                },
            )
            .unwrap();
            let mut params = Parameters::default();
            let checks = admitted
                .integrity_checks(&binding, &read, &mut params)
                .unwrap();
            let slots = params.into_slots();
            assert_eq!(slots[3].value, if inverse { "1" } else { "0" });
            assert_eq!(slots[4].value, if inverse { "2" } else { "1" });
            captures.push(json!({"inverse":inverse,"checks":checks,"parameters":slots}));
        }
        if let Ok(path) = std::env::var("WEFT_FINITE_RELATIONSHIP_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
        }
    }
}

#[cfg(test)]
mod composite_tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};

    #[test]
    fn independent_composite_endpoint_keys_preserve_order_and_refuse_substitution() {
        let cases: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/application-cases.json"
        ))
        .unwrap();
        let fixture = cases
            .iter()
            .find(|c| c["id"] == "related-page-props")
            .unwrap();
        for (source_fields, target_fields) in [
            (vec!["customer-name", "customer-id"], vec!["order-id"]),
            (vec!["customer-id"], vec!["order-customer", "order-id"]),
            (
                vec!["customer-name", "customer-id"],
                vec!["order-total", "order-id", "order-customer"],
            ),
        ] {
            let mut inputs: Vec<weft_core::model::ModuleInput> =
                serde_json::from_value(fixture["request"]["modules"].clone()).unwrap();
            let mut document = checked_json(&inputs[0].document_json).unwrap();
            for (record, fields) in [(0, &source_fields), (1, &target_fields)] {
                document["modules"][0]["elements"][record]["keys"][0]["fields"] = json!(fields
                    .iter()
                    .map(|id| json!({"module":"sales","element":id}))
                    .collect::<Vec<_>>());
            }
            inputs[0].document_json = document.to_string();
            inputs[0].pin.sha256 = sha256(inputs[0].document_json.as_bytes());
            let catalog = Catalog::prepare(inputs).unwrap();
            let mut value = checked_json(
                fixture["request"]["target"]["bindingJson"]
                    .as_str()
                    .unwrap(),
            )
            .unwrap();
            let rewrite = |artifact: &mut Value, bytes: &[u8]| {
                artifact["bytesBase64"] = json!(STANDARD.encode(bytes));
                artifact["sha256"] = json!(sha256(bytes));
            };
            for group in ["entities", "properties", "relationships"] {
                for mapped in value[group].as_array_mut().unwrap() {
                    rewrite(&mut mapped["source"], document.to_string().as_bytes());
                }
            }
            for (record, fields) in [(0, &source_fields), (1, &target_fields)] {
                rewrite(
                    &mut value["entities"][record]["acceptedDefinition"],
                    document["modules"][0]["elements"][record]
                        .to_string()
                        .as_bytes(),
                );
                rewrite(
                    &mut value["keys"][record]["acceptedDefinition"],
                    document["modules"][0]["elements"][record]["keys"][0]
                        .to_string()
                        .as_bytes(),
                );
                let ordered: Vec<_> = fields
                    .iter()
                    .map(|id| {
                        value["properties"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|p| p["logical"]["element"] == *id)
                            .unwrap()["propertyId"]
                            .clone()
                    })
                    .collect();
                value["keys"][record]["orderedPropertyIds"] = json!(ordered);
                value["relationships"][0][if record == 0 {
                    "sourceOrderedPropertyIds"
                } else {
                    "targetOrderedPropertyIds"
                }] = json!(ordered);
            }
            let profile = value["bindingProfileId"].as_str().unwrap().to_string();
            let binding = Admission::parse(&value.to_string(), &profile).unwrap();
            let inventory = OriginalArtifact {
                identity: value["basis"]["layoutInventory"]["identity"]
                    .as_str()
                    .unwrap()
                    .into(),
                bytes: binding.artifacts["/basis/layoutInventory"].clone(),
            };
            let relations = BTreeMap::from([
                ("objects".into(), "object".into()),
                ("edges".into(), "edge".into()),
            ]);
            let columns: BTreeMap<_, _> = [
                ("type", "objects", "type_id"),
                ("rel", "edges", "rel_type_id"),
                ("src-id", "edges", "source_id"),
                ("src-type", "edges", "source_type"),
                ("dst-id", "edges", "target_id"),
                ("dst-type", "edges", "target_type"),
            ]
            .into_iter()
            .map(|(id, relation, name)| {
                (
                    id.into(),
                    Column {
                        relation_identity: relation.into(),
                        name: name.into(),
                    },
                )
            })
            .collect();
            let records: BTreeMap<_, _> = (0..2)
                .map(|index| {
                    let r = RecordAdmission::admit(
                        &binding,
                        index,
                        &catalog,
                        crate::record_definition::Selection {
                            inventory: &inventory,
                            relation_identity: "objects",
                            discriminator_identity: "type",
                            relations: &relations,
                            columns: &columns,
                        },
                    )
                    .unwrap();
                    (serde_json::to_string(r.identity()).unwrap(), r)
                })
                .collect();
            let select = || Selection {
                profile: &value["relationships"][0]["relationshipProfile"],
                inventory: &inventory,
                relation_identity: "edges",
                relations: &relations,
                columns: &columns,
                relationship_type: "rel",
                source_id: "src-id",
                source_type: "src-type",
                target_id: "dst-id",
                target_type: "dst-type",
            };
            for (owner_name, read_name, inverse) in
                [("Customer", "orders", false), ("Orders", "customer", true)]
            {
                let name = |s: &str| Name {
                    value: s.into(),
                    quoted: true,
                    span: Span { start: 0, end: 0 },
                };
                let owner = catalog.record(None, &name(owner_name)).unwrap();
                let read = catalog.relationship_read(&owner, &name(read_name)).unwrap();
                let (expected_source, expected_target) = if inverse {
                    (&target_fields, &source_fields)
                } else {
                    (&source_fields, &target_fields)
                };
                assert_eq!(
                    read.source_key
                        .fields
                        .iter()
                        .map(|i| i.element.as_str())
                        .collect::<Vec<_>>(),
                    *expected_source
                );
                assert_eq!(
                    read.target_key
                        .fields
                        .iter()
                        .map(|i| i.element.as_str())
                        .collect::<Vec<_>>(),
                    *expected_target
                );
                let admission =
                    RelationshipAdmission::admit(&binding, 0, &catalog, &read, &records, select())
                        .unwrap();
                let mut slots = Parameters::default();
                admission
                    .correlate(
                        &binding,
                        &read,
                        &Identifier::new("edge_alias").unwrap(),
                        &Identifier::new("source_alias").unwrap(),
                        &Identifier::new("id").unwrap(),
                        &Identifier::new("type_id").unwrap(),
                        &mut slots,
                    )
                    .unwrap();
                assert_eq!(slots.into_slots().len(), 3);
                for role in ["sourceOrderedPropertyIds", "targetOrderedPropertyIds"] {
                    let mut changed = value.clone();
                    let fields = changed["relationships"][0][role].as_array_mut().unwrap();
                    if fields.len() > 1 {
                        fields.reverse();
                    } else {
                        fields.push(fields[0].clone());
                    }
                    let wrong = match Admission::parse(&changed.to_string(), &profile) {
                        Ok(wrong) => wrong,
                        Err(error) => {
                            assert_eq!(
                                error.message,
                                "Repeated relationship endpoint key component"
                            );
                            continue;
                        }
                    };
                    assert!(RelationshipAdmission::admit(
                        &wrong,
                        0,
                        &catalog,
                        &read,
                        &records,
                        select()
                    )
                    .is_err());
                }
                for source in [true, false] {
                    let mut wrong = read.clone();
                    let key = if source {
                        &mut wrong.source_key
                    } else {
                        &mut wrong.target_key
                    };
                    if key.fields.len() > 1 {
                        key.fields.reverse();
                        key.types.reverse();
                    } else {
                        key.id = "foreign-key".into();
                    }
                    assert!(RelationshipAdmission::admit(
                        &binding,
                        0,
                        &catalog,
                        &wrong,
                        &records,
                        select()
                    )
                    .is_err());
                }
            }
        }
    }
}
