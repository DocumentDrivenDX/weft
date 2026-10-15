//! Shared traversal of the actual resolved application AST. No policy interpretation.
use crate::{
    application_ir as app,
    error::{Diagnostic, Result},
    ir::{Identity, LogicalType},
    model::Catalog,
    security_ir::QueryOperator,
};
use std::collections::BTreeMap;
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-APPLICATION-LINEAGE",
        "resolve",
        "Original application dependency derivation refused",
    )
}
pub(crate) struct Use<'a> {
    pub path: String,
    pub scan: &'a str,
    pub identity: &'a Identity,
    pub logical_type: &'a LogicalType,
    pub operator: QueryOperator,
    pub operation: &'static str,
    pub required: Option<&'static str>,
    pub output: Option<&'a LogicalType>,
}
pub(crate) fn scans(plan: &app::Plan) -> Result<BTreeMap<String, Identity>> {
    let mut scans = BTreeMap::new();
    for scan in std::iter::once(&plan.source).chain(plan.joins.iter().map(|j| &j.right)) {
        if scans
            .insert(scan.occurrence.clone(), scan.record.clone())
            .is_some()
        {
            return Err(fail());
        }
    }
    for (i, p) in plan.filters.iter().enumerate() {
        if let app::Predicate::HasRelated {
            scan, relationship, ..
        } = p
        {
            if relationship.inverse
                || relationship.target_lifecycle != "independent"
                || scans.get(scan) != Some(&relationship.from)
                || scans
                    .insert(format!("t{i}"), relationship.to.clone())
                    .is_some()
            {
                return Err(fail());
            }
        }
    }
    if scans.len() > 256 {
        return Err(fail());
    }
    Ok(scans)
}
fn field(
    path: String,
    f: &app::Field,
    operator: QueryOperator,
    operation: &'static str,
    required: &'static str,
    emit: &mut impl for<'a> FnMut(Use<'a>) -> Result<()>,
) -> Result<()> {
    emit(Use {
        path,
        scan: &f.scan,
        identity: &f.identity,
        logical_type: &f.logical_type,
        operator,
        operation,
        required: Some(required),
        output: None,
    })
}
fn value(
    path: String,
    v: &app::Value,
    operator: QueryOperator,
    operation: &'static str,
    required: &'static str,
    emit: &mut impl for<'a> FnMut(Use<'a>) -> Result<()>,
) -> Result<()> {
    if let app::Value::Field { field: f } = v {
        field(path, f, operator, operation, required, emit)?;
    }
    Ok(())
}
fn predicate(
    path: &str,
    p: &app::Predicate,
    operator: QueryOperator,
    emit: &mut impl for<'a> FnMut(Use<'a>) -> Result<()>,
) -> Result<()> {
    match p {
        app::Predicate::Equal { left, right } => {
            field(
                format!("{path}/left"),
                left,
                operator,
                "equal",
                "equality",
                emit,
            )?;
            value(
                format!("{path}/right"),
                right,
                operator,
                "equal",
                "equality",
                emit,
            )?;
        }
        app::Predicate::LexicographicGreater { columns, values } => {
            for (i, f) in columns.iter().enumerate() {
                field(
                    format!("{path}/columns/{i}"),
                    f,
                    operator,
                    "lexicographicGreater",
                    "order",
                    emit,
                )?;
            }
            for (i, v) in values.iter().enumerate() {
                value(
                    format!("{path}/values/{i}"),
                    v,
                    operator,
                    "lexicographicGreater",
                    "order",
                    emit,
                )?;
            }
        }
        _ => return Err(fail()),
    }
    Ok(())
}
pub(crate) fn visit(
    plan: &app::Plan,
    catalog: &Catalog,
    mut emit: impl for<'a> FnMut(Use<'a>) -> Result<()>,
) -> Result<()> {
    let _ = scans(plan)?;
    for (i, join) in plan.joins.iter().enumerate() {
        for (j, p) in join.on.iter().enumerate() {
            predicate(
                &format!("joins/{i}/on/{j}"),
                p,
                QueryOperator::Join,
                &mut emit,
            )?;
        }
    }
    for (i, p) in plan.filters.iter().enumerate() {
        if let app::Predicate::HasRelated { relationship, .. } = p {
            let record = catalog.record_by_identity(&relationship.to)?;
            let scan = format!("t{i}");
            for (j, identity) in relationship.target_key.fields.iter().enumerate() {
                let (_, graph) = catalog.member_descriptor_by_identity(&record, identity)?;
                let field = graph
                    .iter()
                    .find(|f| f.identity == *identity)
                    .ok_or_else(fail)?;
                let crate::application_model::Shape::Scalar { logical_type } = &field.shape else {
                    return Err(fail());
                };
                emit(Use {
                    path: format!("filters/{i}/targetKey/{j}"),
                    scan: &scan,
                    identity,
                    logical_type,
                    operator: QueryOperator::Predicate,
                    operation: "hasRelatedKey",
                    required: Some("equality"),
                    output: None,
                })?;
            }
        } else {
            predicate(
                &format!("filters/{i}"),
                p,
                QueryOperator::Predicate,
                &mut emit,
            )?;
        }
    }
    for (i, f) in plan.groups.iter().enumerate() {
        field(
            format!("groups/{i}"),
            f,
            QueryOperator::Group,
            "group",
            "equality",
            &mut emit,
        )?;
    }
    for (i, f) in plan.order.iter().enumerate() {
        field(
            format!("order/{i}"),
            f,
            QueryOperator::Order,
            "order",
            "order",
            &mut emit,
        )?;
    }
    for (i, o) in plan.outputs.iter().enumerate() {
        match &o.expression {
            app::Expression::Sum {
                argument,
                logical_type,
            } => emit(Use {
                path: format!("outputs/{i}/argument"),
                scan: &argument.scan,
                identity: &argument.identity,
                logical_type: &argument.logical_type,
                operator: QueryOperator::Aggregate,
                operation: "sum",
                required: None,
                output: Some(logical_type),
            })?,
            app::Expression::Field { .. } | app::Expression::Count { .. } => {}
            _ => return Err(fail()),
        }
    }
    Ok(())
}
