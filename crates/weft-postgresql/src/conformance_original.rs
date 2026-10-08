//! Explicit pinned-fixture runtime composition, enabled only for conformance.
use crate::{
    leaf_codec_definition::{self, OriginalArtifact},
    original_admission::*,
    row_join_definition,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};
use weft_core::{
    backend::{BindingInput, Plan, Registry},
    compile::CompositionInput,
    error::{Diagnostic, Result},
    model::Catalog,
};
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-CAPABILITY", "compose", message)
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(|| fail("Conformance string missing"))
}
fn bytes(v: &Value) -> Result<Vec<u8>> {
    STANDARD
        .decode(text(v)?)
        .map_err(|_| fail("Conformance bytes malformed"))
}
fn artifact(v: &Value) -> Result<OriginalArtifact> {
    let bytes = bytes(&v["bytesBase64"])?;
    if v["sha256"] != weft_core::json::sha256(&bytes) {
        return Err(fail("Conformance artifact pin differs"));
    }
    Ok(OriginalArtifact {
        identity: text(&v["identity"])?.into(),
        bytes,
    })
}
fn columns(v: &Value) -> Result<BTreeMap<String, row_join_definition::Column>> {
    v.as_object()
        .ok_or_else(|| fail("Conformance columns missing"))?
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
        .ok_or_else(|| fail("Conformance original artifacts missing"))?
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
                return Err(fail("Conformance original custody differs"));
            }
            Ok((path.clone(), selected))
        })
        .collect()
}
fn configuration(raw: &str, catalog: &Catalog) -> Result<Configuration> {
    let v = weft_core::json::checked_json(raw)
        .map_err(|_| fail("Embedded conformance JSON invalid"))?;
    if v["interfaceVersion"] != "weft-original-conformance-composition/0.1.0" {
        return Err(fail("Conformance version differs"));
    }
    let mut records = Vec::new();
    for r in v["records"]
        .as_array()
        .ok_or_else(|| fail("Conformance records missing"))?
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
        .ok_or_else(|| fail("Conformance properties missing"))?
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
                Some(crate::row_tree_mapping::Procedures {
                    field_identity: |identity| {
                        serde_json::to_vec(identity)
                            .map_err(|_| fail("Field identity serialization failed"))
                    },
                    scalar: fixture_native_leaf,
                    node_source: fixture_native_source,
                })
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
                .ok_or_else(|| fail("Conformance comparator requires scalar original field"))?;
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
        native: if numeric {
            numeric_expression
        } else {
            unsupported_expression
        },
    })
}
fn unsupported_expression(
    _: &weft_core::ir::Expression,
    _: &[String],
    _: Option<&crate::registered_access::Access<'_>>,
    _: &mut crate::Parameters,
) -> Result<String> {
    Err(fail(
        "Conformance adapter only qualifies native compound projection",
    ))
}
pub fn registry(catalog: &Catalog, _: Plan<'_>, target: CompositionInput<'_>) -> Result<Registry> {
    if target.backend_id != "truss.postgresql.original" {
        return Err(fail("Conformance backend differs"));
    }
    let presets=[
 ("fa0f5513e18524e01adf4129cba88e69d8f9db3e6628f7c956e3553ea694353d",include_str!("../../../tests/truss-postgresql/fixtures/original-address-composition.json")),
 ("26414b00fff45ffdd1c7e3a6d2cffa5c5f9b0333d0324b6a181d4b825b73ada8",include_str!("../../../tests/truss-postgresql/fixtures/original-cyclic-composition.json")),
 ("89783ef3ceee90c98785bee4b65ec879a27c67f0e6b83769f2afd09fc0ca8f95",include_str!("../../../tests/truss-postgresql/fixtures/original-map-composition.json")),
 ("8e1ebe9c7516423fc7fc38e40fc86c206ed94e84fa97b978279732906947a937",include_str!("../../../tests/truss-postgresql/fixtures/original-nested-sequence-composition.json")),
 ("48f639a935625aa8cb586acf9efe0c693090416356c6b83ae17492fe9604d7e5",include_str!("../../../tests/truss-postgresql/fixtures/original-numeric-address-composition.json")),
 ("45c68c4b4ef0067e636c504b0f32dc4e4ae72121732fda00bc82ad1094ea0600",include_str!("../../../tests/truss-postgresql/fixtures/original-numeric-map-composition.json")),
 ("3f87a9cf0298f6b8a9e0cdabf3be8e7d10b71ed52aafdbdf7339a1d5516a2300",include_str!("../../../tests/truss-postgresql/fixtures/original-tags-composition.json")),
 ];
    let optional_presets = [
        (
            "7a1d2b62fc16aa20ebe14dd31b2b957cde4301671bedb66d959328539b033944",
            include_str!(
                "../../../tests/truss-postgresql/fixtures/original-optional-row-composition.json"
            ),
        ),
        (
            "1817b74ad295a9ae1d24b817a53d597e752bc60f954417fa23258e98244995f4",
            include_str!(
                "../../../tests/truss-postgresql/fixtures/original-optional-props-composition.json"
            ),
        ),
    ];
    let entity_presets=[
        ("42d91b22f16443e0dcfb62039326abbfae5510c49c96c68c1ca9eb600987468f",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-address-props-composition.json")),
        ("cacc904f4ed069c3110276228148c7357e2508d9bb68cb9c97f21d1a66a69333",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-address-row-composition.json")),
        ("eb9510b807adc239e3a01268ae3a5a84bdbef645e7dd31fe5d1fec2a751c6bde",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-cyclic-props-composition.json")),
        ("6668ec46460f2f996cbedd95151b391d3623aceb5a290af6fbfe4bb81e3d40b7",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-cyclic-row-composition.json")),
        ("6b1f260bc54b30082c32522e7ed273bfcfdaa1a90d0b5ada3f49b0417c0345b5",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-map-props-composition.json")),
        ("85c53de7842427b0ef90ff0e88a207f4a4306fdeec724262de6dcfa9aa7e4b7a",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-map-row-composition.json")),
        ("58497d001dfb10292ca3a3b94f8a85a20ca67425361fe966d8d602568e8c7442",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-nested-sequence-props-composition.json")),
        ("68373a0a8c90507e35f0b235e52fa2c3f4b670e7c144f538acd10901f9c4e36e",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-nested-sequence-row-composition.json")),
        ("cc3c742a0fe5c554e7e70d7a4d46f671ce3201a9e42211f2cd5b98b9bb3705a2",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-numeric-address-props-composition.json")),
        ("4b7800afd3e882e439fdcf7ec072911cba1e1f8bfd665709755a246e2355fb77",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-numeric-address-row-composition.json")),
        ("0066885175b70b6e5dd2c59e5ec43b7c239a9f687d4dbd08397735b94da1df08",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-numeric-map-props-composition.json")),
        ("992165da9c2fa413f4908208a630861d02bfddcad58b26928af681a69f760239",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-numeric-map-row-composition.json")),
        ("b083d73632cc6b178201b45bc9cdba9c95eb4789d83360534b664dd9f376a49d",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-tags-props-composition.json")),
        ("da21b0e6782a3724db44e3e5f55c8f46b1a02a6ccd24fcb78cd8d0a949f262d5",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-tags-row-composition.json")),
    ];
    let signed_presets = [
        ("4157653f754453c15f3110c7f23b4b70d3b8e3ffbc2e44a0e8a7499ea93d38b7", include_str!("../../../tests/truss-postgresql/fixtures/original-signed-1-row-composition.json")),
        ("1125a4a9bc0f070a0212aad8044f1de8a79764b1396aab51ed576f41d6b17458", include_str!("../../../tests/truss-postgresql/fixtures/original-signed-1-props-composition.json")),
        ("108095f572c0e3b0985ba87cfd4578d1e22ceb83dea9543a5b07bc222308c45a", include_str!("../../../tests/truss-postgresql/fixtures/original-signed-8-row-composition.json")),
        ("9928c3fc53ede1b64cd24dce847fbadbd94a33d5ee714d0e8bd2beead4377b11", include_str!("../../../tests/truss-postgresql/fixtures/original-signed-8-props-composition.json")),
        ("988f5475e89270d43aafa9bc58e25cc6dd5100c0bf628d63d433a79036fa063c", include_str!("../../../tests/truss-postgresql/fixtures/original-signed-16-row-composition.json")),
        ("ed70b7ba935c8dc44ad346175dfa3ff143b106e70be32184fc7e16adb27bb626", include_str!("../../../tests/truss-postgresql/fixtures/original-signed-16-props-composition.json")),
        ("a0fbe78ffc14e0d15a9272f606b972653aa2394324bff00c6e2146b436849949", include_str!("../../../tests/truss-postgresql/fixtures/original-signed-32-row-composition.json")),
        ("0b3c8497e72c16c2427c6cc051a107f14626fd0fa792005153c04eda1adf5840", include_str!("../../../tests/truss-postgresql/fixtures/original-signed-32-props-composition.json")),
        ("508d3a00a8bc7e505746cc528569d4f8404cab2dbb21eee825fa4e95bdc90490", include_str!("../../../tests/truss-postgresql/fixtures/original-signed-64-row-composition.json")),
        ("a93a3cb75a6bc79a9131a9327dc64831e9f5ccc6bc53acce7f52acbed07e68cc", include_str!("../../../tests/truss-postgresql/fixtures/original-signed-64-props-composition.json")),
    ];
    let boolean_presets = [
        ("620a6b9713c53868a0e9a71a74fdcc4b4de53fc2c9a803aea8d68ad8ea728e48",include_str!("../../../tests/truss-postgresql/fixtures/original-multi-recursive-entity-composition.json")),
        ("ee1287b66c62f77991d44c6ad62bf07c7205544858ac1f13150889ed0d4bb959",include_str!("../../../tests/truss-postgresql/fixtures/original-boolean-sequence-composition.json")),
        (
            "b39fb625b813e9fa7e1192aba8610bb024ed99a782bfac5cd933b5ea834eef77",
            include_str!(
                "../../../tests/truss-postgresql/fixtures/original-boolean-row-composition.json"
            ),
        ),
        (
            "d67b0ae1446c6ad4238ef82707bf8d91ac28d9e11a7e1c465b4db09c04a10a77",
            include_str!(
                "../../../tests/truss-postgresql/fixtures/original-boolean-props-composition.json"
            ),
        ),
    ];
    let raw = if target.binding_sha256
        == "f598a497fee406abd64999f3d60a4a2ba2eeb192926987c48656f40590f7b1ba"
    {
        include_str!(
            "../../../tests/truss-postgresql/fixtures/original-relationship-composition.json"
        )
    } else {
        presets
            .iter()
            .chain(optional_presets.iter())
            .chain(entity_presets.iter())
            .chain(signed_presets.iter())
            .chain(boolean_presets.iter())
            .find(|(pin, _)| *pin == target.binding_sha256)
            .map(|(_, raw)| *raw)
            .ok_or_else(|| fail("Binding has no explicitly compiled conformance composition"))?
    };
    let selected = configuration(raw, catalog)?;
    let binding = BindingInput {
        profile: selected.binding_profile.clone(),
        json: target.binding_json.into(),
        sha256: target.binding_sha256.into(),
    };
    let mut registry = Registry::default();
    registry.register(selected.backend(catalog, &binding)?)?;
    Ok(registry)
}
/// Test-only host-selected metadata; request/model content never supplies this argument.
pub fn registry_with_configuration(
    catalog: &Catalog,
    target: CompositionInput<'_>,
    raw: &str,
) -> Result<Registry> {
    if target.backend_id != "truss.postgresql.original" {
        return Err(fail("Conformance backend differs"));
    }
    if raw.len() > 4 * 1024 * 1024 {
        return Err(fail("Conformance configuration exceeds 4 MiB"));
    }
    let selected = configuration(raw, catalog)?;
    let binding = BindingInput {
        profile: selected.binding_profile.clone(),
        json: target.binding_json.into(),
        sha256: target.binding_sha256.into(),
    };
    let mut registry = Registry::default();
    registry.register(selected.backend(catalog, &binding)?)?;
    Ok(registry)
}
fn fixture_native_leaf(
    _: usize,
    node: &crate::value_definition::LayoutNode<'_>,
) -> weft_core::error::Result<crate::row_tree_mapping::NativeLeaf> {
    let (value, kind, allowed) = match node.shape {
        crate::value_definition::LayoutShape::Scalar {
            family: "integer", ..
        } => ("to_jsonb(w.r->>16)", "integer", vec![15, 16]),
        crate::value_definition::LayoutShape::Scalar {
            family: "decimal", ..
        } => ("to_jsonb(w.r->>16)", "decimal", vec![15, 16]),
        crate::value_definition::LayoutShape::Scalar {
            family: "string", ..
        } => ("to_jsonb(w.r->>13)", "text", vec![13]),
        crate::value_definition::LayoutShape::Scalar {
            family: "boolean", ..
        } => (
            "CASE WHEN w.r->>14 IN ('true','false') THEN to_jsonb((w.r->>14)::bool) ELSE NULL END",
            "boolean",
            vec![14],
        ),
        _ => {
            return Err(fail(
                "Native leaf family or shape has no qualified conformance procedure",
            ))
        }
    };
    let codec: String = node
        .codec_bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let extra = (13..=20)
        .filter(|i| !allowed.contains(i))
        .map(|i| format!("w.r->>{i} IS NULL"))
        .collect::<Vec<_>>()
        .join(" AND ");
    let numeric = if kind == "integer" || kind == "decimal" {
        let codec = weft_core::json::checked_json(
            std::str::from_utf8(node.codec_bytes)
                .map_err(|_| fail("Numeric codec bytes are not UTF8"))?,
        )
        .map_err(|_| fail("Numeric codec JSON invalid"))?;
        let authored = artifact(&codec["authoredDefinition"])?;
        let field = weft_core::json::checked_json(
            std::str::from_utf8(&authored.bytes)
                .map_err(|_| fail("Numeric authored definition is not UTF8"))?,
        )
        .map_err(|_| fail("Numeric authored definition JSON invalid"))?;
        if field["scalarType"] != kind {
            return Err(fail("Native numeric family differs from authored Field"));
        }
        if kind == "decimal" {
            " AND CASE WHEN w.r->>16 ~ '^-?(0|[1-9][0-9]*)([.][0-9]+)?$' AND pg_input_is_valid(w.r->>16,'numeric') THEN (w.r->>15)::numeric=(w.r->>16)::numeric ELSE FALSE END"
        } else if field["facets"]["integerWidth"]["signed"] == true {
            " AND CASE WHEN w.r->>16 ~ '^-?(0|[1-9][0-9]*)$' AND pg_input_is_valid(w.r->>16,'numeric') THEN (w.r->>15)::numeric=(w.r->>16)::numeric ELSE FALSE END"
        } else if field["facets"]["integerWidth"]["signed"] == false {
            " AND CASE WHEN w.r->>16 ~ '^(0|[1-9][0-9]*)$' AND pg_input_is_valid(w.r->>16,'numeric') THEN (w.r->>15)::numeric=(w.r->>16)::numeric ELSE FALSE END"
        } else {
            return Err(fail(
                "Native integer procedure requires authored signedness",
            ));
        }
    } else if kind == "boolean" {
        " AND w.r->>14 IN ('true','false')"
    } else {
        ""
    };
    Ok(crate::row_tree_mapping::NativeLeaf {value_sql:value.into(),integrity_sql:format!("w.r->>3='scalar' AND w.r->>12='{kind}' AND w.r->>21='{codec}' AND w.r->>22='' AND {extra}{numeric}")})
}
fn fixture_native_source(
    _: usize,
    node: &crate::value_definition::LayoutNode<'_>,
) -> weft_core::error::Result<String> {
    let codec: String = node
        .codec_bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok(format!("w.r->>8='{codec}' AND w.r->>9=''"))
}

fn numeric_expression(
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
            "Conformance numeric procedure only selects required signed/unsigned integer widths 1..64, decimal precision 1..28 with scale 0..precision, Boolean or Unicode string operands",
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
        _ => Err(fail("Unselected conformance scalar operation")),
    }
}

#[cfg(test)]
mod composite_tests {
    use super::*;
    #[test]
    fn composite_relationships_compile_through_original_owned_configuration() {
        let inputs: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-composite-relationship-inputs.json"
        ))
        .unwrap();
        exercise(
            inputs,
            "WEFT_ORIGINAL_COMPOSITE_RELATIONSHIP_CAPTURE",
            "WEFT_ORIGINAL_COMPOSITE_MIXED_CAPTURE",
        );
    }
    #[test]
    fn heterogeneous_composite_relationships_compile_through_original_owned_configuration() {
        let inputs:Value=serde_json::from_str(include_str!("../../../tests/truss-postgresql/fixtures/original-heterogeneous-relationship-inputs.json")).unwrap();
        exercise(
            inputs,
            "WEFT_ORIGINAL_HETEROGENEOUS_CAPTURE",
            "WEFT_ORIGINAL_HETEROGENEOUS_MIXED_CAPTURE",
        );
    }
    #[test]
    fn string_composite_relationships_compile_through_original_owned_configuration() {
        let inputs: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-string-relationship-inputs.json"
        ))
        .unwrap();
        exercise(
            inputs,
            "WEFT_ORIGINAL_STRING_CAPTURE",
            "WEFT_ORIGINAL_STRING_MIXED_CAPTURE",
        );
    }
    fn exercise(inputs: Value, native_variable: &str, mixed_variable: &str) {
        let mut captures = Vec::new();
        let mut mixed_captures = Vec::new();
        let baseline: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/application-cases.json"
        ))
        .unwrap();
        let base_binding: Value = serde_json::from_str(
            baseline[0]["request"]["target"]["bindingJson"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        for props_pids in [vec![], vec![0, 20], vec![6, 21, 22], vec![0, 21]] {
            for case in inputs["requests"].as_array().unwrap() {
                let mut request = case["request"].clone();
                let mut binding: Value =
                    serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap())
                        .unwrap();
                let mut config_json = inputs["composition"].clone();
                for selected in config_json["properties"].as_array_mut().unwrap() {
                    let index = selected["index"].as_u64().unwrap() as usize;
                    let prop = &mut binding["properties"][index];
                    let pid = prop["propertyId"].as_str().unwrap().parse::<i32>().unwrap();
                    if props_pids.contains(&pid) {
                        let template = &base_binding["properties"]
                            [if prop["ownerTypeId"] == "-1" { 0 } else { 6 }];
                        let mut home: Value = serde_json::from_slice(
                            &bytes(&template["homeDefinition"]["bytesBase64"]).unwrap(),
                        )
                        .unwrap();
                        home["propertyCatalogId"] = prop["propertyId"].clone();
                        home["memberName"] = prop["propertyId"].clone();
                        let raw = home.to_string();
                        prop["home"] = serde_json::json!("props");
                        prop["homeDefinition"] = serde_json::json!({"identity":format!("composite-props-{pid}"),"bytesBase64":STANDARD.encode(raw.as_bytes()),"sha256":weft_core::json::sha256(raw.as_bytes())});
                    }
                }
                let raw = binding.to_string();
                request["target"]["bindingJson"] = serde_json::json!(raw);
                request["target"]["bindingSha256"] =
                    serde_json::json!(weft_core::json::sha256(raw.as_bytes()));
                let catalog =
                    Catalog::prepare(serde_json::from_value(request["modules"].clone()).unwrap())
                        .unwrap();
                let mut config = configuration(&config_json.to_string(), &catalog).unwrap();
                for selected in &mut config.properties {
                    let prop = &binding["properties"][selected.index];
                    if prop["home"] == "props" {
                        selected.relations =
                            BTreeMap::from([("object-table".into(), "object".into())]);
                        selected.columns = BTreeMap::from([
                            (
                                "object-type".into(),
                                row_join_definition::Column {
                                    relation_identity: "object-table".into(),
                                    name: "type_id".into(),
                                },
                            ),
                            (
                                "object-props".into(),
                                row_join_definition::Column {
                                    relation_identity: "object-table".into(),
                                    name: "props".into(),
                                },
                            ),
                        ]);
                        selected.row_join = None;
                        selected.obligations.clear();
                    }
                }
                let response: Value =
                    serde_json::from_str(&config.compile_json(&request.to_string())).unwrap();
                assert_eq!(response["status"], "compiled", "{response}");
                assert_eq!(
                    response,
                    serde_json::from_str::<Value>(&config.compile_json(&request.to_string()))
                        .unwrap()
                );
                let keys: Vec<_> = config.comparators.keys().cloned().collect();
                assert_eq!(keys.len(), 5);
                for key in keys {
                    let removed = config.comparators.remove(&key).unwrap();
                    let refused: Value =
                        serde_json::from_str(&config.compile_json(&request.to_string())).unwrap();
                    assert_eq!(refused["status"], "blocked", "{key}: {refused}");
                    assert!(refused.get("sql").is_none());
                    assert!(refused.get("parameters").is_none());
                    config.comparators.insert(key, removed);
                }
                assert_eq!(
                    response,
                    serde_json::from_str::<Value>(&config.compile_json(&request.to_string()))
                        .unwrap()
                );
                let checks: Vec<_> = response["obligations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|o| o["parameters"].get("sql").and_then(|s| s.as_str()))
                    .collect();
                let capture = serde_json::json!({"direction":case["direction"],"kind":case["kind"],"sql":response["sql"],"parameters":response["parameters"],"columns":response["columns"],"checks":checks,"propsPropertyIds":props_pids});
                if props_pids.is_empty() {
                    captures.push(capture);
                } else {
                    mixed_captures.push(capture);
                }
            }
        }
        if let Ok(path) = std::env::var(mixed_variable) {
            std::fs::write(path, serde_json::to_vec_pretty(&mixed_captures).unwrap()).unwrap();
        }
        if let Ok(path) = std::env::var(native_variable) {
            std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
        }
    }
}

#[cfg(test)]
mod optional_tests {
    use super::*;
    #[test]
    fn optional_scalar_and_complete_entity_retain_original_presence_and_member_order() {
        let inputs: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-optional-scalar-inputs.json"
        ))
        .unwrap();
        let baseline: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/application-cases.json"
        ))
        .unwrap();
        let props_template: Value = serde_json::from_str(
            baseline[0]["request"]["target"]["bindingJson"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        let mut captures = Vec::new();
        let mut transports = Vec::new();
        for home in ["row", "props"] {
            for case in inputs["requests"].as_array().unwrap() {
                let mut request = case["request"].clone();
                let mut binding: Value =
                    serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap())
                        .unwrap();
                let note_index = binding["properties"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .position(|p| p["propertyId"] == "23")
                    .unwrap();
                if home == "props" {
                    let mut definition: Value = serde_json::from_slice(
                        &bytes(&props_template["properties"][0]["homeDefinition"]["bytesBase64"])
                            .unwrap(),
                    )
                    .unwrap();
                    definition["propertyCatalogId"] = serde_json::json!("23");
                    definition["memberName"] = serde_json::json!("23");
                    let raw = definition.to_string();
                    binding["properties"][note_index]["home"] = serde_json::json!("props");
                    binding["properties"][note_index]["homeDefinition"] = serde_json::json!({"identity":"optional-props","bytesBase64":STANDARD.encode(raw.as_bytes()),"sha256":weft_core::json::sha256(raw.as_bytes())});
                }
                let raw = binding.to_string();
                request["target"]["bindingJson"] = serde_json::json!(raw);
                request["target"]["bindingSha256"] =
                    serde_json::json!(weft_core::json::sha256(raw.as_bytes()));
                let catalog =
                    Catalog::prepare(serde_json::from_value(request["modules"].clone()).unwrap())
                        .unwrap();
                let mut config =
                    configuration(&inputs["composition"].to_string(), &catalog).unwrap();
                if home == "props" {
                    let selected = config
                        .properties
                        .iter_mut()
                        .find(|p| p.index == note_index)
                        .unwrap();
                    selected.relations = BTreeMap::from([("object-table".into(), "object".into())]);
                    selected.columns = BTreeMap::from([
                        (
                            "object-type".into(),
                            row_join_definition::Column {
                                relation_identity: "object-table".into(),
                                name: "type_id".into(),
                            },
                        ),
                        (
                            "object-props".into(),
                            row_join_definition::Column {
                                relation_identity: "object-table".into(),
                                name: "props".into(),
                            },
                        ),
                    ]);
                    selected.row_join = None;
                    selected.obligations.clear();
                }
                let response: Value =
                    serde_json::from_str(&config.compile_json(&request.to_string())).unwrap();
                assert_eq!(response["status"], "compiled", "{response}");
                let names: Vec<_> = response["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| c["outputName"].as_str().unwrap())
                    .collect();
                assert_eq!(
                    names,
                    if case["kind"].as_str().unwrap().starts_with("entity") {
                        vec!["note", "id", "part"]
                    } else {
                        vec!["id", "note"]
                    }
                );
                let note = response["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|c| c["outputName"] == "note")
                    .unwrap();
                assert_eq!(note["representation"]["kind"], "value");
                assert_eq!(note["representation"]["nativeNull"], false);
                let selected = config
                    .properties
                    .iter()
                    .position(|p| p.index == note_index)
                    .unwrap();
                let removed = config.properties.remove(selected);
                let refused: Value =
                    serde_json::from_str(&config.compile_json(&request.to_string())).unwrap();
                assert_eq!(refused["status"], "blocked");
                assert!(refused.get("sql").is_none());
                config.properties.insert(selected, removed);
                assert_eq!(
                    response,
                    serde_json::from_str::<Value>(&config.compile_json(&request.to_string()))
                        .unwrap()
                );
                if let Ok(directory) = std::env::var("WEFT_ORIGINAL_OPTIONAL_COMPOSITION_CAPTURE") {
                    let path = std::path::Path::new(&directory)
                        .join(format!("original-optional-{home}-composition.json"));
                    std::fs::write(
                        path,
                        serde_json::to_vec_pretty(&config.conformance_capture()).unwrap(),
                    )
                    .unwrap();
                }
                transports.push(serde_json::json!({"request":request,"response":response}));
                let checks: Vec<_> = response["obligations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|o| o["parameters"].get("sql").and_then(|s| s.as_str()))
                    .collect();
                captures.push(serde_json::json!({"home":home,"kind":case["kind"],"sql":response["sql"],"columns":response["columns"],"parameters":response["parameters"],"checks":checks}));
            }
        }
        if let Ok(path) = std::env::var("WEFT_ORIGINAL_OPTIONAL_TRANSPORT_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&transports).unwrap()).unwrap();
        }
        if let Ok(path) = std::env::var("WEFT_ORIGINAL_OPTIONAL_SCALAR_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
        }
    }
}

#[cfg(test)]
mod recursive_entity_tests {
    use super::*;
    #[test]
    fn complete_entities_combine_scalar_and_original_recursive_roots() {
        let inputs: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-recursive-entity-inputs.json"
        ))
        .unwrap();
        let baseline: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/application-cases.json"
        ))
        .unwrap();
        let props_template: Value = serde_json::from_str(
            baseline[0]["request"]["target"]["bindingJson"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        let mut captures = Vec::new();
        let mut transports = Vec::new();
        for input in inputs.as_array().unwrap() {
            for note_home in ["row", "props"] {
                for case in input["requests"].as_array().unwrap() {
                    let mut request = case["request"].clone();
                    let mut binding: Value =
                        serde_json::from_str(request["target"]["bindingJson"].as_str().unwrap())
                            .unwrap();
                    let note_index = binding["properties"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .position(|p| p["propertyId"] == "23")
                        .unwrap();
                    if note_home == "props" {
                        let mut definition: Value = serde_json::from_slice(
                            &bytes(
                                &props_template["properties"][0]["homeDefinition"]["bytesBase64"],
                            )
                            .unwrap(),
                        )
                        .unwrap();
                        definition["propertyCatalogId"] = serde_json::json!("23");
                        definition["memberName"] = serde_json::json!("23");
                        let raw = definition.to_string();
                        binding["properties"][note_index]["home"] = serde_json::json!("props");
                        binding["properties"][note_index]["homeDefinition"] = serde_json::json!({"identity":"recursive-entity-note-props","bytesBase64":STANDARD.encode(raw.as_bytes()),"sha256":weft_core::json::sha256(raw.as_bytes())});
                    }
                    let raw = binding.to_string();
                    request["target"]["bindingJson"] = serde_json::json!(raw);
                    request["target"]["bindingSha256"] =
                        serde_json::json!(weft_core::json::sha256(raw.as_bytes()));
                    let catalog = Catalog::prepare(
                        serde_json::from_value(request["modules"].clone()).unwrap(),
                    )
                    .unwrap();
                    let mut config =
                        configuration(&input["composition"].to_string(), &catalog).unwrap();
                    if note_home == "props" {
                        let selected = config
                            .properties
                            .iter_mut()
                            .find(|p| p.index == note_index)
                            .unwrap();
                        selected.relations =
                            BTreeMap::from([("object-table".into(), "object".into())]);
                        selected.columns = BTreeMap::from([
                            (
                                "object-type".into(),
                                row_join_definition::Column {
                                    relation_identity: "object-table".into(),
                                    name: "type_id".into(),
                                },
                            ),
                            (
                                "object-props".into(),
                                row_join_definition::Column {
                                    relation_identity: "object-table".into(),
                                    name: "props".into(),
                                },
                            ),
                        ]);
                        selected.row_join = None;
                        selected.obligations.clear();
                    }
                    let response: Value =
                        serde_json::from_str(&config.compile_json(&request.to_string())).unwrap();
                    assert_eq!(
                        response["status"], "compiled",
                        "{} {note_home}: {response}",
                        input["fixture"]
                    );
                    let root = binding["properties"][input["index"].as_u64().unwrap() as usize]
                        ["logical"]["element"]
                        .as_str()
                        .unwrap();
                    let label = if root == "address" { "address" } else { "tags" };
                    let names: Vec<_> = response["columns"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|c| c["outputName"].as_str().unwrap())
                        .collect();
                    assert_eq!(names, vec![label, "note", "id", "part"]);
                    let index = config
                        .properties
                        .iter()
                        .position(|p| p.index == input["index"].as_u64().unwrap() as usize)
                        .unwrap();
                    let removed = config.properties.remove(index);
                    let refused: Value =
                        serde_json::from_str(&config.compile_json(&request.to_string())).unwrap();
                    assert_eq!(refused["status"], "blocked");
                    assert!(refused.get("sql").is_none());
                    config.properties.insert(index, removed);
                    assert_eq!(
                        response,
                        serde_json::from_str::<Value>(&config.compile_json(&request.to_string()))
                            .unwrap()
                    );
                    if let Ok(directory) = std::env::var("WEFT_ORIGINAL_ENTITY_COMPOSITION_CAPTURE")
                    {
                        let path = std::path::Path::new(&directory).join(format!(
                            "original-entity-{}-{note_home}-composition.json",
                            input["fixture"].as_str().unwrap()
                        ));
                        std::fs::write(
                            path,
                            serde_json::to_vec_pretty(&config.conformance_capture()).unwrap(),
                        )
                        .unwrap();
                    }
                    transports.push(serde_json::json!({"fixture":input["fixture"],"noteHome":note_home,"request":request,"response":response}));
                    let checks: Vec<_> = response["obligations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter_map(|o| o["parameters"].get("sql").and_then(|s| s.as_str()))
                        .collect();
                    captures.push(serde_json::json!({"fixture":input["fixture"],"noteHome":note_home,"bound":case["bound"],"propertyId":binding["properties"][input["index"].as_u64().unwrap() as usize]["propertyId"],"sql":response["sql"],"parameters":response["parameters"],"columns":response["columns"],"checks":checks}));
                }
            }
        }
        if let Ok(path) = std::env::var("WEFT_ORIGINAL_ENTITY_TRANSPORT_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&transports).unwrap()).unwrap();
        }
        if let Ok(path) = std::env::var("WEFT_ORIGINAL_RECURSIVE_ENTITY_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
        }
    }
}

#[cfg(test)]
mod native_leaf_refusal_tests {
    use super::*;
    use crate::value_definition::{LayoutNode, LayoutShape};

    #[test]
    fn unqualified_native_leaf_shapes_return_diagnostics_without_panicking() {
        for family in ["decimal", "timestamp", "binary", "unknown"] {
            let node = LayoutNode {
                codec_bytes: b"{}",
                shape: LayoutShape::Scalar {
                    family,
                    storage_representation: "native",
                },
            };
            let result = fixture_native_leaf(0, &node);
            assert!(result.is_err(), "unqualified family {family} must refuse");
        }
        let node = LayoutNode {
            codec_bytes: b"{}",
            shape: LayoutShape::Sequence { item: 0 },
        };
        assert!(fixture_native_leaf(0, &node).is_err());
    }
}

#[cfg(test)]
mod integer_operand_tests {
    use super::*;
    use weft_core::ir::Span;
    use weft_core::ir::{Expression, Family, LogicalType};

    #[test]
    fn admitted_integer_widths_keep_signedness_and_exact_boundary_tokens() {
        let mut captures = Vec::new();
        for bits in 1..=64 {
            for signed in [false, true] {
                let (minimum, maximum) = if signed {
                    (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
                } else {
                    (0, (1i128 << bits) - 1)
                };
                for value in [minimum.to_string(), maximum.to_string()] {
                    let logical_type = LogicalType {
                        family: Family::Integer,
                        facets: serde_json::json!({"integerWidth":{"bits":bits,"signed":signed}}),
                        nullable: false,
                    };
                    let node = Expression::Literal {
                        value: value.clone(),
                        logical_type: logical_type.clone(),
                        span: Span {
                            start: 0,
                            end: value.len(),
                        },
                    };
                    let mut parameters = crate::Parameters::default();
                    let sql = numeric_expression(&node, &[], None, &mut parameters).unwrap();
                    let slots = parameters.into_slots();
                    assert_eq!(sql, "$1::pg_catalog.numeric");
                    assert_eq!(slots[0].value, value);
                    assert_eq!(slots[0].logical_type, logical_type);
                    captures.push(
                        serde_json::json!({"bits":bits,"signed":signed,"value":value,"sql":sql}),
                    );
                }
            }
        }
        if let Ok(path) = std::env::var("WEFT_INTEGER_OPERAND_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
        }
    }
}

#[cfg(test)]
mod integer_domain_refusal_tests {
    use super::*;
    use weft_core::ir::{Expression, Family, LogicalType, Span};
    #[test]
    fn integer_operands_refuse_unknown_facets_and_nullable_domains_without_slots() {
        for (facets, nullable) in [
            (
                serde_json::json!({"integerWidth":{"bits":0,"signed":true}}),
                false,
            ),
            (
                serde_json::json!({"integerWidth":{"bits":65,"signed":true}}),
                false,
            ),
            (
                serde_json::json!({"integerWidth":{"bits":64,"signed":"true"}}),
                false,
            ),
            (
                serde_json::json!({"integerWidth":{"bits":64,"signed":true,"future":true}}),
                false,
            ),
            (
                serde_json::json!({"integerWidth":{"bits":64,"signed":true},"future":true}),
                false,
            ),
            (
                serde_json::json!({"integerWidth":{"bits":64,"signed":true}}),
                true,
            ),
        ] {
            let node = Expression::Literal {
                value: "0".into(),
                logical_type: LogicalType {
                    family: Family::Integer,
                    facets,
                    nullable,
                },
                span: Span { start: 0, end: 1 },
            };
            let mut slots = crate::Parameters::default();
            assert!(numeric_expression(&node, &[], None, &mut slots).is_err());
            assert!(slots.into_slots().is_empty());
        }
    }
}

#[cfg(test)]
mod boolean_property_tests {
    use super::*;
    #[test]
    fn original_boolean_properties_compile_pages_and_typed_filters() {
        let inputs: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-boolean-inputs.json"
        ))
        .unwrap();
        let mut transports = Vec::new();
        for cut in inputs.as_array().unwrap() {
            for case in cut["requests"].as_array().unwrap() {
                let request = &case["request"];
                let catalog =
                    Catalog::prepare(serde_json::from_value(request["modules"].clone()).unwrap())
                        .unwrap();
                let config = configuration(&cut["composition"].to_string(), &catalog).unwrap();
                let response: Value =
                    serde_json::from_str(&config.compile_json(&request.to_string())).unwrap();
                assert_eq!(response["status"], "compiled", "{response}");
                assert_eq!(
                    config.compile_json(&request.to_string()),
                    response.to_string()
                );
                let mut invalid = request.clone();
                invalid["sql"] = serde_json::json!("SELECT SUM(c.id) AS total FROM Customer c");
                invalid.as_object_mut().unwrap().remove("readProfile");
                let refused: Value =
                    serde_json::from_str(&config.compile_json(&invalid.to_string())).unwrap();
                assert_eq!(refused["status"], "blocked");
                assert!(refused.get("sql").is_none());
                transports.push(serde_json::json!({"home":cut["home"],"kind":case["kind"],"request":request,"response":response}));
            }
        }
        if let Ok(path) = std::env::var("WEFT_BOOLEAN_TRANSPORT_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&transports).unwrap()).unwrap();
        }
    }
}

#[cfg(test)]
mod boolean_sequence_tests {
    use super::*;
    #[test]
    fn original_boolean_sequence_compiles_without_string_coercion() {
        let cut: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-boolean-sequence-inputs.json"
        ))
        .unwrap();
        let request = &cut["request"];
        let catalog =
            Catalog::prepare(serde_json::from_value(request["modules"].clone()).unwrap()).unwrap();
        let config = configuration(&cut["composition"].to_string(), &catalog).unwrap();
        let response: Value =
            serde_json::from_str(&config.compile_json(&request.to_string())).unwrap();
        assert_eq!(response["status"], "compiled", "{response}");
        if let Ok(path) = std::env::var("WEFT_BOOLEAN_SEQUENCE_CAPTURE") {
            std::fs::write(
                path,
                serde_json::to_vec_pretty(
                    &serde_json::json!({"request":request,"response":response}),
                )
                .unwrap(),
            )
            .unwrap();
        }
    }
}

#[cfg(test)]
mod multi_recursive_entity_tests {
    use super::*;
    #[test]
    fn complete_entity_retains_multiple_independent_recursive_roots() {
        let cut: Value = serde_json::from_str(include_str!(
            "../../../tests/truss-postgresql/fixtures/original-multi-recursive-entity-inputs.json"
        ))
        .unwrap();
        let mut transports = Vec::new();
        for case in cut["requests"].as_array().unwrap() {
            let request = &case["request"];
            let catalog =
                Catalog::prepare(serde_json::from_value(request["modules"].clone()).unwrap())
                    .unwrap();
            let mut config = configuration(&cut["composition"].to_string(), &catalog).unwrap();
            let response: Value =
                serde_json::from_str(&config.compile_json(&request.to_string())).unwrap();
            assert_eq!(response["status"], "compiled", "{response}");
            assert_eq!(
                response["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| c["outputName"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                ["tags", "address", "note", "id", "part"]
            );
            for index in [4, 5] {
                let position = config
                    .properties
                    .iter()
                    .position(|p| p.index == index)
                    .unwrap();
                let removed = config.properties.remove(position);
                let refused: Value =
                    serde_json::from_str(&config.compile_json(&request.to_string())).unwrap();
                assert_eq!(refused["status"], "blocked");
                assert!(refused.get("sql").is_none());
                config.properties.insert(position, removed);
                assert_eq!(
                    config.compile_json(&request.to_string()),
                    response.to_string()
                );
            }
            transports.push(
                serde_json::json!({"bound":case["bound"],"request":request,"response":response}),
            );
        }
        if let Ok(path) = std::env::var("WEFT_MULTI_RECURSIVE_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&transports).unwrap()).unwrap();
        }
    }
}

#[cfg(test)]
mod decimal_operand_tests {
    use super::*;
    use weft_core::ir::{Expression, Family, LogicalType, Span};
    #[test]
    fn admitted_decimal_precision_scale_pairs_preserve_exact_operands() {
        let mut captures = Vec::new();
        for precision in 1..=28usize {
            for scale in 0..=precision {
                let digits = "9".repeat(precision);
                let maximum = if scale == 0 {
                    digits
                } else if scale == precision {
                    format!("0.{digits}")
                } else {
                    format!(
                        "{}.{}",
                        &digits[..precision - scale],
                        &digits[precision - scale..]
                    )
                };
                for value in [maximum.clone(), format!("-{maximum}")] {
                    let logical_type = LogicalType {
                        family: Family::Decimal,
                        facets: serde_json::json!({"precision":precision,"scale":scale}),
                        nullable: false,
                    };
                    let node = Expression::Literal {
                        value: value.clone(),
                        logical_type: logical_type.clone(),
                        span: Span {
                            start: 0,
                            end: value.len(),
                        },
                    };
                    let mut parameters = crate::Parameters::default();
                    let sql = numeric_expression(&node, &[], None, &mut parameters).unwrap();
                    let slots = parameters.into_slots();
                    assert_eq!(sql, "$1::pg_catalog.numeric");
                    assert_eq!(slots[0].value, value);
                    assert_eq!(slots[0].logical_type, logical_type);
                    captures.push(serde_json::json!({"precision":precision,"scale":scale,"value":value,"sql":sql}));
                }
            }
        }
        if let Ok(path) = std::env::var("WEFT_DECIMAL_OPERAND_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
        }
    }
}

#[cfg(test)]
mod decimal_operand_refusal_tests {
    use super::*;
    use weft_core::ir::{Expression, Family, LogicalType, Span};
    #[test]
    fn decimal_operands_refuse_unadmitted_facets_without_slots() {
        for facets in [
            serde_json::json!({"precision":0,"scale":0}),
            serde_json::json!({"precision":29,"scale":0}),
            serde_json::json!({"precision":28,"scale":29}),
            serde_json::json!({"precision":28,"scale":-1}),
            serde_json::json!({"precision":28,"scale":2,"future":true}),
            serde_json::json!({"precision":28}),
        ] {
            let node = Expression::Literal {
                value: "0".into(),
                logical_type: LogicalType {
                    family: Family::Decimal,
                    facets,
                    nullable: false,
                },
                span: Span { start: 0, end: 1 },
            };
            let mut parameters = crate::Parameters::default();
            assert!(numeric_expression(&node, &[], None, &mut parameters).is_err());
            assert!(parameters.into_slots().is_empty());
        }
    }
}

#[cfg(test)]
mod numeric_sequence_tests {
    use super::*;
    #[test]
    fn original_signed_and_decimal_sequences_compile_exact_numeric_leaves() {
        for (label,raw) in [
            ("signed",include_str!("../../../tests/truss-postgresql/fixtures/original-signed-sequence-inputs.json")),
            ("decimal",include_str!("../../../tests/truss-postgresql/fixtures/original-decimal-sequence-inputs.json")),
        ] {
            let cut:Value=serde_json::from_str(raw).unwrap();
            let request=&cut["request"];
            let catalog=Catalog::prepare(serde_json::from_value(request["modules"].clone()).unwrap()).unwrap();
            let config=configuration(&cut["composition"].to_string(),&catalog).unwrap();
            let compiled=config.compile_json(&request.to_string());
            let response:Value=serde_json::from_str(&compiled).unwrap();
            assert_eq!(response["status"],"compiled","{label}: {response}");
            assert_eq!(compiled,config.compile_json(&request.to_string()));
            if let Ok(directory)=std::env::var("WEFT_NUMERIC_SEQUENCE_CAPTURE") {
                std::fs::write(std::path::Path::new(&directory).join(format!("original-{label}-sequence-public.json")),serde_json::to_vec_pretty(&serde_json::json!({"request":request,"response":response})).unwrap()).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod numeric_container_tests {
    use super::*;
    #[test]
    fn original_signed_and_decimal_maps_and_structures_retain_declared_shapes() {
        for (label,raw) in [
            ("signed-map",include_str!("../../../tests/truss-postgresql/fixtures/original-signed-map-inputs.json")),
            ("decimal-map",include_str!("../../../tests/truss-postgresql/fixtures/original-decimal-map-inputs.json")),
            ("signed-structured",include_str!("../../../tests/truss-postgresql/fixtures/original-signed-structured-inputs.json")),
            ("decimal-structured",include_str!("../../../tests/truss-postgresql/fixtures/original-decimal-structured-inputs.json")),
        ] {
            let cut:Value=serde_json::from_str(raw).unwrap();
            let request=&cut["request"];
            let catalog=Catalog::prepare(serde_json::from_value(request["modules"].clone()).unwrap()).unwrap();
            let config=configuration(&cut["composition"].to_string(),&catalog).unwrap();
            let compiled=config.compile_json(&request.to_string());
            let response:Value=serde_json::from_str(&compiled).unwrap();
            assert_eq!(response["status"],"compiled","{label}: {response}");
            assert_eq!(compiled,config.compile_json(&request.to_string()));
            if let Ok(directory)=std::env::var("WEFT_NUMERIC_CONTAINER_CAPTURE") {
                std::fs::write(std::path::Path::new(&directory).join(format!("original-{label}-public.json")),serde_json::to_vec_pretty(&serde_json::json!({"request":request,"response":response})).unwrap()).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod numeric_entity_tests {
    use super::*;
    #[test]
    fn complete_entities_retain_independent_signed_and_decimal_native_roots() {
        for (label,raw) in [
            ("signed-sequence-decimal-structured",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-signed-sequence-decimal-structured-inputs.json")),
            ("decimal-sequence-signed-structured",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-decimal-sequence-signed-structured-inputs.json")),
            ("signed-map-decimal-structured",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-signed-map-decimal-structured-inputs.json")),
            ("decimal-map-signed-structured",include_str!("../../../tests/truss-postgresql/fixtures/original-entity-decimal-map-signed-structured-inputs.json")),
        ] {
            let cut:Value=serde_json::from_str(raw).unwrap();let mut transports=Vec::new();
            for case in cut["requests"].as_array().unwrap() {
                let request=&case["request"];
                let catalog=Catalog::prepare(serde_json::from_value(request["modules"].clone()).unwrap()).unwrap();
                let config=configuration(&cut["composition"].to_string(),&catalog).unwrap();
                let compiled=config.compile_json(&request.to_string());let response:Value=serde_json::from_str(&compiled).unwrap();
                assert_eq!(response["status"],"compiled","{label}: {response}");
                assert_eq!(compiled,config.compile_json(&request.to_string()));
                assert_eq!(response["columns"].as_array().unwrap().iter().map(|c|c["outputName"].as_str().unwrap()).collect::<Vec<_>>(),vec!["tags","address","note","id","part"]);
                transports.push(serde_json::json!({"bound":case["bound"],"request":request,"response":response}));
            }
            if let Ok(directory)=std::env::var("WEFT_NUMERIC_ENTITY_CAPTURE") {
                std::fs::write(std::path::Path::new(&directory).join(format!("original-entity-{label}-public.json")),serde_json::to_vec_pretty(&transports).unwrap()).unwrap();
            }
        }
    }
}
