//! Original native topology packet for SQL lowering; identities are selected code.
use crate::{
    property_definition::PropertyAdmission,
    registered_access::{Access, Location},
    value_definition::LayoutShape,
    Parameters,
};
use serde_json::{json, Value};
use weft_core::{
    error::{Diagnostic, Result},
    ir::{Family, LogicalType},
};
pub struct Mapping {
    pub topology_parameter: String,
    /// Private custody SQL, not the public value carrier.
    pub custody_sql: String,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "emit", message)
}
/// Admit original topology and stage one typed metadata parameter atomically.
/// The backend-selected procedure determines exact native field identity bytes;
/// model bytes cannot select or load executable code.
pub fn encode(
    property: &PropertyAdmission,
    access: &Access<'_>,
    parameters: &mut Parameters,
    mut field_identity: impl FnMut(&Value) -> Result<Vec<u8>>,
) -> Result<Mapping> {
    access.verify_property(property)?;
    let Location::Row(location) = &access.location else {
        return Err(fail("Native mapping requires row access"));
    };
    if !matches!(
        property.home,
        crate::property_definition::HomeAdmission::Row { .. }
    ) {
        return Err(fail("Native mapping requires original row home"));
    }
    crate::result_definition::property_column(property, 1, "native_mapping")?;
    property
        .value
        .graph
        .verify_record_presence(&property.value.admitted_record_presence)?;
    property.value.verify_leaf_codec_custody()?;
    let layout = property.value.graph.layout()?;
    let hex = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let mut packed = Vec::new();
    for node in &layout.nodes {
        let shape = match &node.shape {
            LayoutShape::Scalar {
                family,
                storage_representation,
            } => {
                json!({"kind":"scalar","family":family,"storageRepresentation":storage_representation})
            }
            LayoutShape::Sequence { item } => json!({"kind":"sequence","item":item}),
            LayoutShape::Map { item } => json!({"kind":"map","item":item}),
            LayoutShape::Structured { record } => json!({"kind":"structured","record":record}),
            LayoutShape::Record { members } => {
                let mut identities = std::collections::BTreeSet::new();
                let mut entries = Vec::new();
                for member in members {
                    let bytes = field_identity(member.field_identity)?;
                    if !identities.insert(bytes.clone()) {
                        return Err(fail(
                            "Selected native field encoding aliases original members",
                        ));
                    }
                    entries.push(json!({"fieldIdentity":member.field_identity,"identityHex":hex(&bytes),"storedName":member.stored_name,"valueNode":member.value_node,"presenceHex":hex(member.presence_bytes)}));
                }
                json!({"kind":"record","members":entries})
            }
        };
        packed.push(json!({"codecHex":hex(node.codec_bytes),"shape":shape}));
    }
    let mut staged = parameters.clone();
    let topology_parameter = staged.push(
        LogicalType {
            family: Family::String,
            facets: json!({}),
            nullable: false,
        },
        json!({"root":layout.root,"nodes":packed}).to_string(),
        json!({"use":"original-native-tree-topology","property":property.identity}),
    )?;
    let mapping = Mapping {
        topology_parameter,
        custody_sql: location.tree_custody(),
    };
    *parameters = staged;
    Ok(mapping)
}
