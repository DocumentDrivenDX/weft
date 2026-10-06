//! Result metadata bridge from admitted original descriptors to Weft's existing ABI.
use crate::property_definition::PropertyAdmission;
use weft_core::{
    application_model::{Descriptor, Shape},
    backend::{Column, Representation, ScalarCarrier, ScalarDecoder},
    error::{Diagnostic, Result},
    ir::Family,
};
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "emit", message)
}
/// This selects the required public representation, not an executable codec.
/// The backend must prove its emitted carrier against original codec/presence
/// meaning and refuse publication until all native/host obligations hold.
pub fn property_column(
    property: &PropertyAdmission,
    position: usize,
    output_name: &str,
) -> Result<Column> {
    if weft_core::json::checked_json(&property.value.graph.original_json)
        .map_err(|_| fail("Original value graph JSON refused"))?
        != property.value.graph.value
    {
        return Err(fail("Result graph differs from original admitted bytes"));
    }
    property
        .value
        .graph
        .verify_descriptors(property.value.descriptors(), &property.identity)?;
    let descriptor = property
        .value
        .descriptors()
        .iter()
        .find(|descriptor| descriptor.identity == property.identity)
        .ok_or_else(|| fail("Result lacks original root descriptor"))?;
    column(descriptor, position, output_name)
}
fn column(descriptor: &Descriptor, position: usize, output_name: &str) -> Result<Column> {
    if position == 0 || output_name.is_empty() || output_name.contains('\0') {
        return Err(fail("Result position or output name is invalid"));
    }
    let required = match descriptor.availability.as_deref() {
        Some("required") => true,
        Some("absent-allowed") => false,
        _ => {
            return Err(fail(
                "Property result availability has no original selected meaning",
            ))
        }
    };
    let representation = match &descriptor.shape {
        Shape::Scalar { logical_type } if required && !logical_type.nullable => {
            Representation::Scalar {
                logical_type: logical_type.clone(),
                carrier: ScalarCarrier::Text,
                decoder: match logical_type.family {
                    Family::String => ScalarDecoder::Text,
                    Family::Boolean => ScalarDecoder::Boolean,
                    Family::Integer => ScalarDecoder::ExactInteger,
                    Family::Decimal => ScalarDecoder::ExactDecimal,
                },
            }
        }
        _ => Representation::Value {
            descriptor: descriptor.identity.clone(),
            native_null: false,
        },
    };
    Ok(Column {
        position,
        output_name: output_name.into(),
        representation,
        source_identities: vec![descriptor.identity.clone()],
        nullable: false,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use weft_core::ir::{Identity, LogicalType};
    fn identity() -> Identity {
        Identity {
            document_id: "doc".into(),
            revision: "1".into(),
            module: "model".into(),
            element: "field".into(),
        }
    }
    #[test]
    fn exact_scalar_types_and_optional_nullable_values_preserve_the_existing_abi() {
        for (family, facets, decoder) in [
            (Family::String, json!({}), "text"),
            (Family::Boolean, json!({}), "boolean"),
            (
                Family::Integer,
                json!({"integerWidth":{"bits":64,"signed":false}}),
                "exact-integer",
            ),
            (
                Family::Decimal,
                json!({"precision":28,"scale":9}),
                "exact-decimal",
            ),
        ] {
            let mut descriptor = Descriptor {
                identity: identity(),
                availability: Some("required".into()),
                shape: Shape::Scalar {
                    logical_type: LogicalType {
                        family,
                        facets,
                        nullable: false,
                    },
                },
            };
            let emitted =
                serde_json::to_value(column(&descriptor, 1, "selected").unwrap()).unwrap();
            assert_eq!(emitted["representation"]["decoder"], decoder);
            if let Shape::Scalar { logical_type } = &descriptor.shape {
                assert_eq!(
                    emitted["representation"]["logicalType"],
                    json!(logical_type)
                );
            }
            descriptor.availability = Some("absent-allowed".into());
            let emitted =
                serde_json::to_value(column(&descriptor, 1, "selected").unwrap()).unwrap();
            assert_eq!(emitted["representation"]["kind"], "value");
            assert_eq!(emitted["representation"]["nativeNull"], false);
            descriptor.availability = Some("required".into());
            if let Shape::Scalar { logical_type } = &mut descriptor.shape {
                logical_type.nullable = true;
            }
            assert!(matches!(
                column(&descriptor, 1, "selected").unwrap().representation,
                Representation::Value {
                    native_null: false,
                    ..
                }
            ));
        }
    }
    #[test]
    fn compound_identity_is_finite_and_missing_presence_cannot_be_inferred() {
        for shape in [
            Shape::Sequence { item: identity() },
            Shape::Map { item: identity() },
            Shape::Structured { record: identity() },
        ] {
            let descriptor = Descriptor {
                identity: identity(),
                availability: Some("required".into()),
                shape,
            };
            let result = column(&descriptor, 2, "container").unwrap();
            assert!(
                matches!(result.representation, Representation::Value { descriptor: id, native_null: false } if id == identity())
            );
            assert_eq!(result.source_identities, [identity()]);
            assert!(!result.nullable);
        }
        let descriptor = Descriptor {
            identity: identity(),
            availability: None,
            shape: Shape::Record { members: vec![] },
        };
        assert!(column(&descriptor, 1, "value").is_err());
        let descriptor = Descriptor {
            availability: Some("required".into()),
            ..descriptor
        };
        assert!(column(&descriptor, 0, "value").is_err());
        assert!(column(&descriptor, 1, "").is_err());
    }
}
