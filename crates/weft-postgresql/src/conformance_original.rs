//! Explicit seven-fixture runtime composition, enabled only for conformance.
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
fn configuration(raw: &str) -> Result<Configuration> {
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
        let raw = text(&p["rowJoin"]["originalJson"])?;
        let d = weft_core::json::checked_json(raw).map_err(|_| fail("Join JSON invalid"))?;
        let artifacts = originals(&p["rowJoin"]["originalArtifacts"], &d)?;
        let join = row_join_definition::Definition::parse(
            raw,
            row_join_definition::Selection {
                profile: &d["profile"],
                original_artifacts: &artifacts,
                relations: &relations,
                columns: &columns,
            },
        )?;
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
            row_join: Some(Arc::new(join)),
            obligations: serde_json::from_value(p["obligations"].clone())
                .map_err(|_| fail("Obligations invalid"))?,
            edge_association: None,
            native_tree: Some(crate::row_tree_mapping::Procedures {
                field_identity: |identity| {
                    serde_json::to_vec(identity)
                        .map_err(|_| fail("Field identity serialization failed"))
                },
                scalar: fixture_native_leaf,
                node_source: fixture_native_source,
            }),
        });
    }
    Ok(Configuration {
        relationships: vec![],
        binding_profile: text(&v["bindingProfile"])?.into(),
        records,
        properties,
        comparators: BTreeMap::new(),
        native: unsupported_expression,
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
    let raw = presets
        .iter()
        .find(|(pin, _)| *pin == target.binding_sha256)
        .map(|(_, raw)| *raw)
        .ok_or_else(|| fail("Binding has no explicitly compiled conformance composition"))?;
    let selected = configuration(raw)?;
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
            family: "string", ..
        } => ("to_jsonb(w.r->>13)", "text", vec![13]),
        _ => panic!("fixture scalar procedure unsupported"),
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
    let numeric = if kind == "integer" {
        " AND CASE WHEN w.r->>16 ~ '^(0|[1-9][0-9]*)$' AND pg_input_is_valid(w.r->>16,'numeric') THEN (w.r->>15)::numeric=(w.r->>16)::numeric ELSE FALSE END"
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
