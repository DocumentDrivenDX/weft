//! Original AST read intents, independent of policy/action/disposition interpretation.
use crate::{
    application_ir as app,
    application_model::Shape,
    error::{Diagnostic, Result},
    ir::Identity,
    model::Catalog,
    security_ontology::SecurityRef,
};
use serde_json::{json, Value};
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-APPLICATION-READ-REQUIREMENTS",
        "capability",
        "Original AST read requirement derivation refused",
    )
}
fn reference(id: &Identity) -> SecurityRef {
    SecurityRef {
        document_id: id.document_id.clone(),
        module_id: id.module.clone(),
        element_id: id.element.clone(),
    }
}
pub(crate) fn derive(plan: &app::Plan, catalog: &Catalog) -> Result<Value> {
    let owners = crate::application_property_uses::scans(plan)?;
    let mut scans = Vec::new();
    for scan in std::iter::once(&plan.source).chain(plan.joins.iter().map(|j| &j.right)) {
        let path = if scan.occurrence == plan.source.occurrence {
            "source".into()
        } else {
            let i = plan
                .joins
                .iter()
                .position(|j| j.right.occurrence == scan.occurrence)
                .ok_or_else(fail)?;
            format!("joins/{i}/right")
        };
        scans.push(json!({"path":path,"scan":scan.occurrence,"target":reference(&scan.record),"recordIdentity":scan.record,"kind":"rowRead"}));
    }
    let mut relationships = Vec::new();
    for (i, p) in plan.filters.iter().enumerate() {
        if let app::Predicate::HasRelated {
            scan, relationship, ..
        } = p
        {
            let terminal = format!("t{i}");
            scans.push(json!({"path":format!("filters/{i}/target"),"scan":terminal,"target":reference(&relationship.to),"recordIdentity":relationship.to,"kind":"rowRead"}));
            relationships.push(json!({"path":format!("filters/{i}"),"kind":"relationshipObservation","sourceScan":scan,"terminalScan":terminal,"relationship":relationship.identity,"sourceIdentity":relationship.from,"targetIdentity":relationship.to}));
        }
    }
    let mut fields = Vec::new();
    let mut unsupported = Vec::new();
    let mut budget = 8_000_000usize;
    let mut add = |path: &str, scan: &str, identity: &Identity, role: &str| -> Result<()> {
        if fields.len() >= 8192 {
            return Err(fail());
        }
        let owner = owners.get(scan).ok_or_else(fail)?;
        let record = catalog.record_by_identity(owner)?;
        let (_, graph) = catalog.member_descriptor_by_identity(&record, identity)?;
        let descriptor = graph
            .iter()
            .find(|d| d.identity == *identity)
            .ok_or_else(fail)?;
        if descriptor.availability.as_deref() != Some("required") {
            return Err(fail());
        }
        let scalar = matches!(descriptor.shape, Shape::Scalar { .. });
        let row = json!({"path":path,"scan":scan,"target":reference(owner),"recordIdentity":owner,"field":reference(identity),"fieldIdentity":identity,"role":role,"descriptor":descriptor,"sourceStatus":if scalar{"derived"}else{"unsupported-shape"}});
        let bytes = serde_json::to_vec(&row).map_err(|_| fail())?;
        budget = budget.checked_sub(bytes.len()).ok_or_else(fail)?;
        if !scalar {
            unsupported.push(json!({"path":path,"kind":"fieldShape","fieldIdentity":identity}));
        }
        fields.push(row);
        Ok(())
    };
    for (i, output) in plan.outputs.iter().enumerate() {
        if let app::Expression::Field { scan, identity } = &output.expression {
            add(
                &format!("outputs/{i}/expression"),
                scan,
                identity,
                "projection",
            )?;
        }
    }
    crate::application_property_uses::visit(plan, catalog, |u| {
        let role = serde_json::to_value(u.operator).map_err(|_| fail())?;
        add(&u.path, u.scan, u.identity, role.as_str().ok_or_else(fail)?)
    })?;
    for (i, output) in plan.outputs.iter().enumerate() {
        if matches!(output.expression, app::Expression::Sum { .. }) {
            unsupported.push(
                json!({"path":format!("outputs/{i}/argument"),"kind":"numericAggregateProperties"}),
            );
        }
    }
    Ok(
        json!({"scans":scans,"fields":fields,"relationships":relationships,"unsupportedSources":unsupported}),
    )
}
