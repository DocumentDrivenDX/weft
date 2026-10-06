use super::*;
use weft_core::application_model::RelationshipRead;

pub(super) fn validate(
    c: &Context<'_>,
    mapping: &Admission,
    rel: &RelationshipRead,
) -> Result<serde_json::Value> {
    let (index, physical) = mapping.value["relationships"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .find(|(_, r)| r["logical"] == json!(rel.identity))
        .ok_or_else(|| fail("Selected relationship mapping is missing"))?;
    let input = c
        .catalog
        .inputs
        .iter()
        .find(|i| {
            i.pin.document_id == rel.identity.document_id && i.pin.revision == rel.identity.revision
        })
        .ok_or_else(|| fail("Relationship source document is missing"))?;
    let document = weft_core::json::checked_json(&input.document_json)
        .map_err(|_| fail("Relationship source document could not be read"))?;
    let original = document["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == rel.identity.module)
        .and_then(|m| m["relationships"].as_array())
        .and_then(|rs| rs.iter().find(|r| r["id"] == rel.identity.relationship))
        .ok_or_else(|| fail("Relationship definition is missing from the original UMF"))?;
    if mapping.decoded_json(&format!("/relationships/{index}/source"))? != document
        || mapping.decoded_json(&format!("/relationships/{index}/acceptedDefinition"))? != *original
    {
        return Err(fail(
            "Relationship source or accepted definition differs from original UMF",
        ));
    }
    let roles = if rel.inverse {
        [
            ("target", &rel.from, &rel.source_key),
            ("source", &rel.to, &rel.target_key),
        ]
    } else {
        [
            ("source", &rel.from, &rel.source_key),
            ("target", &rel.to, &rel.target_key),
        ]
    };
    // Endpoint keys have independent arities. Never zip source and target keys.
    for (role, record, key) in roles {
        let owner = &mapping.value["entities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["logical"] == json!(record))
            .ok_or_else(|| fail("Relationship endpoint mapping is missing"))?["typeId"];
        let ids = key
            .fields
            .iter()
            .map(|id| {
                mapping.value["properties"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|p| &p["ownerTypeId"] == owner && p["logical"] == json!(id))
                    .map(|p| p["propertyId"].clone())
                    .ok_or_else(|| fail("Relationship key property mapping is missing"))
            })
            .collect::<Result<Vec<_>>>()?;
        if &physical[format!("{role}TypeId")] != owner
            || physical[format!("{role}KeyId")] != key.id
            || physical[format!("{role}OrderedPropertyIds")] != json!(ids)
        {
            return Err(fail(
                "Relationship endpoint type or authored key differs from its binding",
            ));
        }
    }
    Ok(physical.clone())
}
