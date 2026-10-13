//! Data-only original storage admission; executable procedures are host-selected Rust.
use crate::{
    leaf_codec_definition::{self, OriginalArtifact},
    original_admission::*,
    row_join_definition,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};
use weft_core::{
    error::{Diagnostic, Result},
    model::Catalog,
};
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-CAPABILITY", "compose", message)
}
fn text(v: &Value) -> Result<&str> {
    v.as_str()
        .ok_or_else(|| fail("Storage composition string missing"))
}
pub(crate) fn bytes(v: &Value) -> Result<Vec<u8>> {
    STANDARD
        .decode(text(v)?)
        .map_err(|_| fail("Storage composition bytes malformed"))
}
pub(crate) fn artifact(v: &Value) -> Result<OriginalArtifact> {
    let bytes = bytes(&v["bytesBase64"])?;
    if v["sha256"] != weft_core::json::sha256(&bytes) {
        return Err(fail("Storage composition artifact pin differs"));
    }
    Ok(OriginalArtifact {
        identity: text(&v["identity"])?.into(),
        bytes,
    })
}
fn columns(v: &Value) -> Result<BTreeMap<String, row_join_definition::Column>> {
    v.as_object()
        .ok_or_else(|| fail("Storage composition columns missing"))?
        .iter()
        .map(|(id, c)| {
            Ok((
                id.clone(),
                row_join_definition::Column {
                    relation_identity: text(&c["relationIdentity"])?.into(),
                    name: text(&c["name"])?.into(),
                },
            ))
        })
        .collect()
}
fn originals(v: &Value, document: &Value) -> Result<BTreeMap<String, OriginalArtifact>> {
    v.as_object()
        .ok_or_else(|| fail("Storage composition original artifacts missing"))?
        .iter()
        .map(|(path, encoded)| {
            let mut a = document;
            for part in path.split('/') {
                a = if a.is_array() {
                    a.get(
                        part.parse::<usize>()
                            .map_err(|_| fail("Artifact array path invalid"))?,
                    )
                    .ok_or_else(|| fail("Artifact path missing"))?
                } else {
                    a.get(part).ok_or_else(|| fail("Artifact path missing"))?
                };
            }
            let selected = artifact(a)?;
            if selected.bytes != bytes(encoded)? {
                return Err(fail("Storage composition original custody differs"));
            }
            Ok((path.clone(), selected))
        })
        .collect()
}
pub(crate) fn configuration(
    raw: &str,
    catalog: &Catalog,
    version: &str,
    native_tree: Option<crate::row_tree_mapping::Procedures>,
    native: crate::original_backend::Native,
    fallback: crate::original_backend::Native,
) -> Result<Configuration> {
    if raw.len() > 4 * 1024 * 1024 {
        return Err(fail("Storage composition exceeds 4 MiB"));
    }
    let v = weft_core::json::checked_json(raw)
        .map_err(|_| fail("Embedded storage composition JSON invalid"))?;
    if v["interfaceVersion"] != version {
        return Err(fail("Storage composition version differs"));
    }
    let mut records = Vec::new();
    for r in v["records"]
        .as_array()
        .ok_or_else(|| fail("Storage composition records missing"))?
    {
        records.push(OwnedRecordSelection {
            index: r["index"]
                .as_u64()
                .ok_or_else(|| fail("Record index missing"))? as usize,
            inventory: artifact(&r["inventory"])?,
            relation_identity: text(&r["relationIdentity"])?.into(),
            discriminator_identity: text(&r["discriminatorIdentity"])?.into(),
            relations: serde_json::from_value(r["relations"].clone())
                .map_err(|_| fail("Relations invalid"))?,
            columns: columns(&r["columns"])?,
        });
    }
    let mut properties = Vec::new();
    for p in v["properties"]
        .as_array()
        .ok_or_else(|| fail("Storage composition properties missing"))?
    {
        let mut leaves = BTreeMap::new();
        for (id, c) in p["leafCodecs"]
            .as_object()
            .ok_or_else(|| fail("Leaf definitions missing"))?
        {
            let raw = text(&c["originalJson"])?;
            let d = weft_core::json::checked_json(raw).map_err(|_| fail("Leaf JSON invalid"))?;
            let artifacts = originals(&c["originalArtifacts"], &d)?;
            leaves.insert(
                id.clone(),
                leaf_codec_definition::Definition::parse(
                    raw,
                    leaf_codec_definition::Selection {
                        profile: &d["profile"],
                        source_profile: &d["sourceInterpretationProfile"],
                        native_profile: &d["nativeDomainProfile"],
                        original_artifacts: &artifacts,
                    },
                )?,
            );
        }
        let mut presence = BTreeMap::new();
        for (path, d) in p["recordPresence"]
            .as_object()
            .ok_or_else(|| fail("Member presence missing"))?
        {
            presence.insert(
                path.clone(),
                crate::presence_definition::Definition::parse(
                    text(&d["originalJson"])?,
                    &p["presenceProfile"],
                    &bytes(&d["acceptedDefinitionBase64"])?,
                )?,
            );
        }
        let relations = serde_json::from_value(p["relations"].clone())
            .map_err(|_| fail("Relations invalid"))?;
        let columns = columns(&p["columns"])?;
        let row_join = if p["rowJoin"].is_null() {
            None
        } else {
            let raw = text(&p["rowJoin"]["originalJson"])?;
            let d = weft_core::json::checked_json(raw).map_err(|_| fail("Join JSON invalid"))?;
            let artifacts = originals(&p["rowJoin"]["originalArtifacts"], &d)?;
            Some(Arc::new(row_join_definition::Definition::parse(
                raw,
                row_join_definition::Selection {
                    profile: &d["profile"],
                    original_artifacts: &artifacts,
                    relations: &relations,
                    columns: &columns,
                },
            )?))
        };
        properties.push(OwnedPropertySelection {
            index: p["index"]
                .as_u64()
                .ok_or_else(|| fail("Property index missing"))? as usize,
            value_profile: p["valueProfile"].clone(),
            presence_profile: p["presenceProfile"].clone(),
            leaf_codecs: leaves,
            record_presence: presence,
            physical_profile: p["physicalProfile"].clone(),
            inventory: artifact(&p["inventory"])?,
            relations,
            columns,
            row_join,
            obligations: serde_json::from_value(p["obligations"].clone())
                .map_err(|_| fail("Obligations invalid"))?,
            edge_association: None,
            native_tree: if p["nativeTree"] == false {
                None
            } else {
                Some(
                    native_tree.ok_or_else(|| {
                        fail("Selected storage profile has no native tree procedure")
                    })?,
                )
            },
        });
    }
    let mut comparators = BTreeMap::new();
    if let Some(items) = v["comparators"].as_object() {
        for (key, definition) in items {
            let identity = weft_core::json::checked_json(key)
                .map_err(|_| fail("Comparator identity invalid"))?;
            let owner = serde_json::from_value(identity["owner"].clone())
                .map_err(|_| fail("Comparator owner invalid"))?;
            let field = serde_json::from_value(identity["field"].clone())
                .map_err(|_| fail("Comparator field invalid"))?;
            let record = catalog.record_by_identity(&owner)?;
            let (_, descriptors) = catalog.member_descriptor_by_identity(&record, &field)?;
            let logical = descriptors
                .iter()
                .find_map(|d| {
                    if d.identity == field {
                        if let weft_core::application_model::Shape::Scalar { logical_type } =
                            &d.shape
                        {
                            Some(logical_type)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    fail("Storage composition comparator requires scalar original field")
                })?;
            let raw = text(&definition["originalJson"])?;
            let document =
                weft_core::json::checked_json(raw).map_err(|_| fail("Comparator JSON invalid"))?;
            let artifacts = originals(&definition["originalArtifacts"], &document)?;
            use crate::native_comparator_definition::{Definition, Operation, Selection};
            let parsed = Definition::parse(
                raw,
                Selection {
                    profile: &document["profile"],
                    native_profile: &document["nativeDomainProfile"],
                    original_artifacts: &artifacts,
                    operations: &std::collections::BTreeSet::from([
                        Operation::Equality,
                        Operation::Ordering,
                        Operation::Key,
                        Operation::Sum,
                    ]),
                },
                logical,
            )?;
            comparators.insert(key.clone(), parsed);
        }
    }
    let mut relationships = Vec::new();
    if let Some(items) = v["relationships"].as_array() {
        for r in items {
            relationships.push(OwnedRelationshipSelection {
                index: r["index"]
                    .as_u64()
                    .ok_or_else(|| fail("Relationship index missing"))?
                    as usize,
                inverse: r["inverse"]
                    .as_bool()
                    .ok_or_else(|| fail("Relationship direction missing"))?,
                profile: r["profile"].clone(),
                inventory: artifact(&r["inventory"])?,
                relation_identity: text(&r["relationIdentity"])?.into(),
                relations: serde_json::from_value(r["relations"].clone())
                    .map_err(|_| fail("Relationship relations invalid"))?,
                columns: columns(&r["columns"])?,
                relationship_type: text(&r["relationshipType"])?.into(),
                source_id: text(&r["sourceId"])?.into(),
                source_type: text(&r["sourceType"])?.into(),
                target_id: text(&r["targetId"])?.into(),
                target_type: text(&r["targetType"])?.into(),
            });
        }
    }
    let numeric = !comparators.is_empty();
    Ok(Configuration {
        relationships,
        binding_profile: text(&v["bindingProfile"])?.into(),
        records,
        properties,
        comparators,
        native: if numeric { native } else { fallback },
    })
}
pub(crate) fn scalar_expression(
    node: &weft_core::ir::Expression,
    operands: &[String],
    access: Option<&crate::registered_access::Access<'_>>,
    parameters: &mut crate::Parameters,
) -> Result<String> {
    use weft_core::ir::{Expression, Family};
    let logical = node.logical_type();
    let integer_domain = logical.family == Family::Integer
        && logical.facets["integerWidth"]["bits"]
            .as_u64()
            .is_some_and(|bits| {
                (1..=64).contains(&bits)
                    && logical.facets["integerWidth"]["signed"]
                        .as_bool()
                        .is_some_and(|signed| {
                            logical.facets
                                == serde_json::json!({"integerWidth":{"bits":bits,"signed":signed}})
                        })
            });
    let decimal_domain = logical.family == Family::Decimal
        && logical.facets["precision"]
            .as_u64()
            .is_some_and(|precision| {
                (1..=28).contains(&precision)
                    && logical.facets["scale"].as_u64().is_some_and(|scale| {
                        scale <= precision
                            && logical.facets
                                == serde_json::json!({"precision":precision,"scale":scale})
                    })
            });
    let selected_domain = integer_domain || decimal_domain;
    let string_domain = logical.family == Family::String && logical.facets == serde_json::json!({});
    let boolean_domain =
        logical.family == Family::Boolean && logical.facets == serde_json::json!({});
    if logical.nullable || !(selected_domain || string_domain || boolean_domain) {
        return Err(fail(
            "Registered scalar procedure only selects required signed/unsigned integer widths 1..64, decimal precision 1..28 with scale 0..precision, Boolean or Unicode string operands",
        ));
    }
    match node {
        Expression::Field { .. } => {
            let access = access.ok_or_else(|| fail("Numeric field access missing"))?;
            let carrier = match &access.location {
                crate::registered_access::Location::Row(location) => {
                    if string_domain {
                        location.scalar_observation().text
                    } else if boolean_domain {
                        location.scalar_observation().boolean
                    } else {
                        location.scalar_observation().native_numeric_text
                    }
                }
                crate::registered_access::Location::Props(_) => access
                    .scalar_storage
                    .as_ref()
                    .ok_or_else(|| fail("Numeric props carrier missing"))?
                    .carrier
                    .clone(),
            };
            Ok(if string_domain {
                format!("({carrier})::pg_catalog.text COLLATE pg_catalog.\"C\"")
            } else if boolean_domain {
                format!("({carrier})::pg_catalog.bool")
            } else {
                format!("({carrier})::pg_catalog.numeric")
            })
        }
        Expression::Literal {
            value,
            logical_type,
            span,
        } => {
            let slot = parameters.push(
                logical_type.clone(),
                value.clone(),
                serde_json::json!({"literalSpan":span}),
            )?;
            Ok(if string_domain {
                format!("{slot}::pg_catalog.text COLLATE pg_catalog.\"C\"")
            } else if boolean_domain {
                format!("{slot}::pg_catalog.bool")
            } else {
                format!("{slot}::pg_catalog.numeric")
            })
        }
        Expression::Equal { .. } if boolean_domain && operands.len() == 2 => {
            Ok(format!("({} = {})", operands[0], operands[1]))
        }
        Expression::And { .. } if boolean_domain && operands.len() == 2 => {
            Ok(format!("({} AND {})", operands[0], operands[1]))
        }
        _ => Err(fail("Unselected registered scalar operation")),
    }
}
