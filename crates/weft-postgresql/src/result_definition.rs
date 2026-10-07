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
/// A physical scalar projection plus mandatory owner-wide payload observation.
/// Original codec/presence bytes remain separate source/native admission inputs.
#[derive(Debug)]
pub struct ScalarProjection {
    pub sql: String,
    pub column: Column,
    pub payload_check_sql: String,
    pub codec_bytes: Vec<u8>,
    pub presence_bytes: Vec<u8>,
}
/// Payload observations for every prepared read, including predicate-only reads.
/// These are physical prerequisites, not source/domain or host qualification.
#[derive(Debug)]
pub struct ReadPayloadObservation {
    pub scan: String,
    pub field: weft_core::ir::Identity,
    pub sql: String,
    pub codec_bytes: Vec<u8>,
    pub presence_bytes: Vec<u8>,
}
pub fn read_payload_observations(
    prepared: &crate::registered_access::Prepared<'_>,
    properties: &std::collections::BTreeMap<String, PropertyAdmission>,
) -> Result<Vec<ReadPayloadObservation>> {
    let mut observations = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for access in &prepared.accesses {
        let key = crate::comparator_requirements::registration_key(&access.owner, &access.field);
        if !seen.insert((access.scan.clone(), key.clone())) {
            return Err(fail("Repeated prepared payload read"));
        }
        let property = properties
            .get(&key)
            .ok_or_else(|| fail("Payload read lacks original property admission"))?;
        let projection = property_projection(property, access, 1, "weft_payload")?;
        observations.push(ReadPayloadObservation {
            scan: access.scan.clone(),
            field: access.field.clone(),
            sql: projection.payload_check_sql,
            codec_bytes: projection.codec_bytes,
            presence_bytes: projection.presence_bytes,
        });
    }
    Ok(observations)
}
pub fn property_projection(
    property: &PropertyAdmission,
    access: &crate::registered_access::Access<'_>,
    position: usize,
    output_name: &str,
) -> Result<ScalarProjection> {
    access.verify_property(property)?;
    let column = property_column(property, position, output_name)?;
    if !matches!(column.representation, Representation::Scalar { .. }) {
        return Err(Diagnostic::new(
            "WFT-CAPABILITY",
            "emit",
            "Selected value needs its recursive/presence result bridge",
        ));
    }
    let codec_bytes = property
        .value
        .graph
        .artifacts
        .get(&format!(
            "/nodes/{}/codecDefinition",
            property.value.graph.root
        ))
        .ok_or_else(|| fail("Result lacks original codec bytes"))?
        .clone();
    let (carrier, integrity, source) = match &access.location {
        crate::registered_access::Location::Props(location) => {
            let storage = property
                .value
                .props_scalar_storage(location)?
                .ok_or_else(|| fail("Scalar result lacks original scalar storage codec"))?;
            (
                storage.carrier,
                storage.storage_integrity,
                access.owner_source.sql.clone(),
            )
        }
        crate::registered_access::Location::Row(location) => {
            if !matches!(
                property.home,
                crate::property_definition::HomeAdmission::Row { .. }
            ) {
                return Err(fail("Native projection differs from original row home"));
            }
            let Representation::Scalar { logical_type, .. } = &column.representation else {
                unreachable!()
            };
            let observation = location.scalar_observation();
            let carrier = match logical_type.family {
                Family::String => observation.text.clone(),
                Family::Boolean => format!("{}::pg_catalog.text", observation.boolean),
                Family::Integer | Family::Decimal => observation.original_numeric_token.clone(),
            };
            let hex: String = codec_bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            let integrity = format!(
                "({} AND {} = '{}')",
                observation.payload_integrity(logical_type.family.clone()),
                observation.codec_bytes_hex,
                hex
            );
            let source = format!("{} {}", access.owner_source.sql, location.joins.join(" "));
            (carrier, integrity, source)
        }
    };
    Ok(ScalarProjection {
        sql: format!(
            "({})::pg_catalog.text AS {}",
            carrier,
            crate::Identifier::new(output_name)?.sql()
        ),
        payload_check_sql: format!(
            "SELECT count(*) AS violations FROM {} WHERE {} AND ({}) IS DISTINCT FROM TRUE",
            source, access.owner_source.discriminator, integrity
        ),
        codec_bytes,
        presence_bytes: property.value.presence.original_json.as_bytes().to_vec(),
        column,
    })
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
/// Result columns in original projection order. Source execution/encoding still
/// requires the separately selected native bridge; metadata never grants it.
pub fn projection_columns(
    context: &weft_core::backend::Context<'_>,
    properties: &std::collections::BTreeMap<String, PropertyAdmission>,
    comparators: &std::collections::BTreeMap<
        String,
        crate::native_comparator_definition::Definition,
    >,
) -> Result<Vec<Column>> {
    use std::collections::BTreeMap;
    use weft_core::{
        application_ir as app,
        backend::Plan,
        ir::{Expression, Node},
    };
    crate::comparator_requirements::admit_context(context, properties, comparators)?;
    let mut owners = BTreeMap::new();
    let mut add_owner = |occurrence: &String, owner: &weft_core::ir::Identity| -> Result<()> {
        if owners.insert(occurrence.clone(), owner.clone()).is_some() {
            return Err(fail("Repeated result scan occurrence"));
        }
        Ok(())
    };
    match context.plan {
        Plan::V01(plan) => {
            let mut nodes = vec![&plan.root];
            while let Some(node) = nodes.pop() {
                match node {
                    Node::Scan {
                        occurrence, record, ..
                    } => add_owner(occurrence, record)?,
                    Node::InnerJoin { left, right, .. } => {
                        nodes.extend([left.as_ref(), right.as_ref()])
                    }
                    Node::Filter { input, .. }
                    | Node::Aggregate { input, .. }
                    | Node::Project { input, .. } => nodes.push(input),
                }
            }
        }
        Plan::V02(plan) => {
            add_owner(&plan.source.occurrence, &plan.source.record)?;
            for join in &plan.joins {
                add_owner(&join.right.occurrence, &join.right.record)?;
            }
        }
    }
    let selected_property =
        |scan: &String, identity: &weft_core::ir::Identity| -> Result<&PropertyAdmission> {
            let owner = owners
                .get(scan)
                .ok_or_else(|| fail("Result field has no original scan owner"))?;
            properties
                .get(&crate::comparator_requirements::registration_key(
                    owner, identity,
                ))
                .ok_or_else(|| fail("Result field lacks exact original owning property"))
        };
    let mut columns = Vec::new();
    match context.plan {
        Plan::V01(plan) => {
            let Node::Project { outputs, .. } = &plan.root else {
                return Err(fail("Result plan has no outer projection"));
            };
            for (index, output) in outputs.iter().enumerate() {
                let result = if let Expression::Field {
                    scan,
                    identity,
                    logical_type,
                    ..
                } = &output.expression
                {
                    let property = selected_property(scan, identity)?;
                    if !property.value.descriptors().iter().any(|descriptor| &descriptor.identity == identity && matches!(&descriptor.shape, Shape::Scalar { logical_type: original } if original == logical_type)) {
                        return Err(fail("Projection type differs from original admitted property"));
                    }
                    property_column(property, index + 1, &output.name)?
                } else {
                    let mut pending = vec![&output.expression];
                    let mut sources = Vec::new();
                    while let Some(expression) = pending.pop() {
                        match expression {
                            Expression::Field { identity, .. } => {
                                if !sources.contains(identity) {
                                    sources.push(identity.clone());
                                }
                            }
                            Expression::Equal { left, right, .. }
                            | Expression::And { left, right, .. } => {
                                pending.push(right);
                                pending.push(left);
                            }
                            Expression::Sum { argument, .. } => pending.push(argument),
                            Expression::Literal { .. } => {}
                        }
                    }
                    scalar_column(
                        output.expression.logical_type(),
                        index + 1,
                        &output.name,
                        sources,
                    )?
                };
                columns.push(result);
            }
        }
        Plan::V02(plan) => {
            for (index, output) in plan.outputs.iter().enumerate() {
                columns.push(match &output.expression {
                    app::Expression::Field { scan, identity } => property_column(
                        selected_property(scan, identity)?,
                        index + 1,
                        &output.name,
                    )?,
                    app::Expression::Count { logical_type } => scalar_column(
                        logical_type,
                        index + 1,
                        &output.name,
                        vec![plan.source.record.clone()],
                    )?,
                    app::Expression::Sum {
                        argument,
                        logical_type,
                    } => scalar_column(
                        logical_type,
                        index + 1,
                        &output.name,
                        vec![argument.identity.clone()],
                    )?,
                    app::Expression::RelatedKeys {
                        relationship,
                        bound,
                        ..
                    } => {
                        validate_output(index + 1, &output.name)?;
                        Column {
                            position: index + 1,
                            output_name: output.name.clone(),
                            representation: Representation::RelatedKeys {
                                relationship: relationship.identity.clone(),
                                key: relationship.target_key.clone(),
                                bound: *bound,
                            },
                            source_identities: relationship.target_key.fields.clone(),
                            nullable: false,
                        }
                    }
                });
            }
        }
    }
    Ok(columns)
}
fn validate_output(position: usize, name: &str) -> Result<()> {
    if position == 0 || name.is_empty() || name.contains('\0') {
        return Err(fail("Result position or output name is invalid"));
    }
    Ok(())
}
fn scalar_column(
    logical_type: &weft_core::ir::LogicalType,
    position: usize,
    name: &str,
    sources: Vec<weft_core::ir::Identity>,
) -> Result<Column> {
    validate_output(position, name)?;
    Ok(Column {
        position,
        output_name: name.into(),
        representation: Representation::Scalar {
            logical_type: logical_type.clone(),
            carrier: ScalarCarrier::Text,
            decoder: match logical_type.family {
                Family::String => ScalarDecoder::Text,
                Family::Boolean => ScalarDecoder::Boolean,
                Family::Integer => ScalarDecoder::ExactInteger,
                Family::Decimal => ScalarDecoder::ExactDecimal,
            },
        },
        source_identities: sources,
        nullable: logical_type.nullable,
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
    fn aggregate_nullability_and_exact_result_facets_are_retained() {
        let logical = weft_core::ir::LogicalType {
            family: Family::Decimal,
            facets: json!({"precision":28,"scale":9}),
            nullable: true,
        };
        let result = scalar_column(&logical, 2, "total", vec![identity()]).unwrap();
        assert!(result.nullable);
        assert_eq!(result.source_identities, [identity()]);
        let wire = serde_json::to_value(result).unwrap();
        assert_eq!(wire["representation"]["logicalType"], json!(logical));
        assert_eq!(wire["representation"]["decoder"], "exact-decimal");
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
