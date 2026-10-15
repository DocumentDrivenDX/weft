//! Conditional current-source scalar SQL. Declarations are not native admission.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    application_ir as app,
    application_lineage::OrderDirection,
    application_model::Shape,
    current_query_operands::CurrentQueryOperands,
    error::{Diagnostic, Result},
    ir::{Family, Identity, LogicalType},
    json::{checked_json, sha256},
};
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-CURRENT-SCALAR-SQL",
        "capability",
        "Current scalar lowering refused",
    )
}
fn ident(s: &str) -> Result<String> {
    if s.is_empty() || s.len() > 63 || s.contains('\0') {
        return Err(fail());
    }
    Ok(format!("\"{}\"", s.replace('"', "\"\"")))
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Relation {
    schema: String,
    table: String,
    eligible_column: String,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Scan {
    scan: String,
    target: Identity,
    relation: Relation,
}
#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum Mode {
    Disclosed,
    OriginalAuthorized,
    Prohibited,
    Unresolved,
}
#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Exposure {
    Original,
    Transformed,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BindingValue {
    path: String,
    scan: String,
    field: Identity,
    column: String,
    domain: Identity,
    comparison_scope: String,
    mode: Option<Mode>,
    original_action: Option<String>,
    exposure: Option<Exposure>,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Binding {
    version: String,
    source_request_sha256: String,
    designation_sha256: String,
    source_binding_sha256: String,
    scans: Vec<Scan>,
    values: Vec<BindingValue>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub ordinal: usize,
    pub source_path: String,
    pub name: Option<String>,
    pub value: String,
    pub logical_type: LogicalType,
    pub native_type: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultColumn {
    pub position: usize,
    pub name: String,
    pub path: String,
    pub source_field: Option<Identity>,
    pub effective_domain: Option<Identity>,
    pub logical_type: LogicalType,
    pub exposure: String,
    pub encoding: String,
}
/// Immutable compiler output, never a native permit or imported report.
pub struct CurrentScalarSql<'a> {
    context: &'a CurrentQueryOperands<'a>,
    binding_sha256: String,
    sql: String,
    parameters: Vec<Parameter>,
    columns: Vec<ResultColumn>,
    private: Option<(Value, Value)>,
}
impl CurrentScalarSql<'_> {
    pub fn sql(&self) -> &str {
        &self.sql
    }
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }
    pub fn columns(&self) -> &[ResultColumn] {
        &self.columns
    }
    pub fn json_string(&self) -> Result<String> {
        struct Buffer {
            bytes: Vec<u8>,
            left: usize,
        }
        impl std::io::Write for Buffer {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                self.left = self
                    .left
                    .checked_sub(b.len())
                    .ok_or_else(|| std::io::Error::other("budget"))?;
                self.bytes.extend_from_slice(b);
                Ok(b.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut out = Buffer {
            bytes: Vec::new(),
            left: 16_000_000,
        };
        serde_json::to_writer(&mut out, &self.json()).map_err(|_| fail())?;
        String::from_utf8(out.bytes).map_err(|_| fail())
    }
    pub fn json(&self) -> Value {
        let mut value = json!({"version":"weft.postgresql.current-scalar/0.1-candidate","authority":false,"securityReady":false,"nativeAdmission":"required","sourceRequestSha256":sha256(self.context.designation().preparation().source_request_json().as_bytes()),"designationSha256":sha256(self.context.designation().json().as_bytes()),"materializationBindingSha256":self.binding_sha256,"sql":self.sql,"parameters":self.parameters,"resultColumns":self.columns,"nativeRequirements":["genuine authenticated held source/caller/Subject/current cut","exact original static binding/profile/model/source correspondence","private relation producer/OID/ACL/type/encoding/collation correspondence","complete primary-eligible population before relational operations even when empty","actual per-use read/route/disclosure and exact separate action decisions","present uniform per-use domains and paired/cross-row key/transform/runtime scopes","native statement/result budgets and no partial public release","source/caller/current-cut revalidation, result/cell lineage and guarded decoding/release"]});
        if let Some((columns, cells)) = &self.private {
            value["version"] = json!("weft.postgresql.current-scalar/0.2-candidate");
            value["privateResultColumns"] = columns.clone();
            value["publicCellSources"] = cells.clone();
        }
        value
    }
}
struct Selected<'a> {
    source: &'a BindingValue,
    logical_type: LogicalType,
    sql: String,
}
struct Build<'a> {
    selected: BTreeMap<(String, String), Selected<'a>>,
    used: BTreeSet<(String, String)>,
    parameters: Vec<Parameter>,
}
fn scalar_supported(t: &LogicalType) -> bool {
    !t.nullable
        && match t.family {
            Family::String => t.facets == json!({}),
            Family::Integer => t.facets == json!({"integerWidth":{"bits":64,"signed":true}}),
            _ => false,
        }
}
fn count_type(t: &LogicalType) -> Result<()> {
    if t.family != Family::Integer || t.nullable || t.facets != json!({}) {
        return Err(fail());
    }
    Ok(())
}
fn native_type(t: &LogicalType) -> Result<&'static str> {
    if !scalar_supported(t) {
        return Err(fail());
    }
    Ok(if t.family == Family::String {
        "text"
    } else {
        "int8"
    })
}
impl Build<'_> {
    fn field(
        &mut self,
        path: &str,
        scan: &str,
        id: &Identity,
    ) -> Result<(String, LogicalType, String)> {
        let key = (path.to_owned(), scan.to_owned());
        let v = self.selected.get(&key).ok_or_else(fail)?;
        if &v.source.field != id || !self.used.insert(key) {
            return Err(fail());
        }
        Ok((
            v.sql.clone(),
            v.logical_type.clone(),
            v.source.comparison_scope.clone(),
        ))
    }
    fn value(
        &mut self,
        path: &str,
        v: &app::Value,
    ) -> Result<(String, LogicalType, Option<String>)> {
        match v {
            app::Value::Field { field } => {
                let (s, t, scope) = self.field(path, &field.scan, &field.identity)?;
                Ok((s, t, Some(scope)))
            }
            app::Value::Literal {
                value,
                logical_type,
                ..
            }
            | app::Value::Parameter {
                value,
                logical_type,
                ..
            } => {
                let ty = native_type(logical_type)?;
                if value.contains('\0') || self.parameters.len() >= 1024 {
                    return Err(fail());
                }
                let name = if let app::Value::Parameter { name, .. } = v {
                    Some(name.clone())
                } else {
                    None
                };
                let ordinal = self.parameters.len() + 1;
                self.parameters.push(Parameter {
                    ordinal,
                    source_path: path.into(),
                    name,
                    value: value.clone(),
                    logical_type: logical_type.clone(),
                    native_type: ty.into(),
                });
                let sql = format!(
                    "${ordinal}::pg_catalog.{ty}{}",
                    if ty == "text" {
                        " COLLATE pg_catalog.\"C\""
                    } else {
                        ""
                    }
                );
                Ok((sql, logical_type.clone(), None))
            }
        }
    }
    fn predicate(&mut self, path: &str, p: &app::Predicate) -> Result<String> {
        match p {
            app::Predicate::Boolean { value } => Ok(if *value { "TRUE" } else { "FALSE" }.into()),
            app::Predicate::Equal { left, right } => {
                let (a, ta, sa) =
                    self.field(&format!("{path}/left"), &left.scan, &left.identity)?;
                let (b, tb, sb) = self.value(&format!("{path}/right"), right)?;
                if ta != tb || sb.is_some_and(|s| s != sa) {
                    return Err(fail());
                }
                Ok(format!("({a} OPERATOR(pg_catalog.=) {b})"))
            }
            _ => Err(fail()),
        }
    }
}
/// Lower only through the actual issued typed operand context. The supplied
/// native materialization binding is a declaration, not authentication or proof
/// of its values/modes. It must be supplied and verified by a held native issuer.
pub fn lower<'a>(
    context: &'a CurrentQueryOperands<'a>,
    binding_json: &str,
) -> Result<CurrentScalarSql<'a>> {
    if binding_json.len() > 4_000_000 {
        return Err(fail());
    }
    let raw = checked_json(binding_json).map_err(|_| fail())?;
    let binding: Binding = serde_json::from_value(raw.clone()).map_err(|_| fail())?;
    if serde_json::to_value(&binding).map_err(|_| fail())? != raw {
        return Err(fail());
    }
    lower_binding(context, binding, sha256(binding_json.as_bytes()), None)
}
fn lower_binding<'a>(
    context: &'a CurrentQueryOperands<'a>,
    binding: Binding,
    binding_sha256: String,
    row_keys: Option<BTreeMap<String, String>>,
) -> Result<CurrentScalarSql<'a>> {
    let designation = context.designation();
    let original = designation.preparation();
    let plan = original.plan();
    if binding.version != "weft.postgresql.current-values/0.1-candidate"
        || binding.source_request_sha256 != sha256(original.source_request_json().as_bytes())
        || binding.designation_sha256 != sha256(designation.json().as_bytes())
        || binding.source_binding_sha256 != designation.binding_scope().sha256
        || binding.scans.len() > 16
        || binding.values.len() > 4096
        || plan.joins.len() > 8
    {
        return Err(fail());
    }
    if row_keys.is_some() && plan.aggregate {
        return Err(fail());
    }
    let mut scans = BTreeMap::new();
    let mut ctes = Vec::new();
    for scan in &binding.scans {
        let expected = std::iter::once(&plan.source)
            .chain(plan.joins.iter().map(|j| &j.right))
            .find(|s| s.occurrence == scan.scan)
            .ok_or_else(fail)?;
        if expected.record != scan.target || scans.insert(scan.scan.clone(), scan).is_some() {
            return Err(fail());
        }
        let alias = ident(&scan.scan)?;
        let mut columns = binding
            .values
            .iter()
            .filter(|v| v.scan == scan.scan)
            .map(|v| ident(&v.column))
            .collect::<Result<BTreeSet<_>>>()?;
        if let Some(keys) = &row_keys {
            columns.insert(ident(keys.get(&scan.scan).ok_or_else(fail)?)?);
        }
        let select = if columns.is_empty() {
            "1 AS \"__wft_row\"".to_owned()
        } else {
            columns.into_iter().collect::<Vec<_>>().join(", ")
        };
        ctes.push(format!(
            "{alias} AS MATERIALIZED (SELECT {select} FROM {}.{} WHERE {} IS TRUE)",
            ident(&scan.relation.schema)?,
            ident(&scan.relation.table)?,
            ident(&scan.relation.eligible_column)?
        ));
    }
    if row_keys.as_ref().is_some_and(|keys| {
        keys.len() != scans.len() || keys.keys().any(|k| !scans.contains_key(k))
    }) {
        return Err(fail());
    }
    if scans.len() != 1 + plan.joins.len() {
        return Err(fail());
    }
    let mut expected = BTreeMap::new();
    for operation in context.operations() {
        for operand in operation.operands() {
            if let weft_core::current_query_operands::Input::Field { scan, identity, .. } =
                operand.input()
            {
                if expected
                    .insert(
                        (operand.path().to_owned(), (*scan).to_owned()),
                        ((*identity).clone(), false),
                    )
                    .is_some()
                {
                    return Err(fail());
                }
            }
        }
    }
    for (i, output) in plan.outputs.iter().enumerate() {
        if let app::Expression::Field { scan, identity } = &output.expression {
            if expected
                .insert(
                    (format!("outputs/{i}/expression"), scan.clone()),
                    (identity.clone(), true),
                )
                .is_some()
            {
                return Err(fail());
            }
        }
    }
    let mut selected = BTreeMap::new();
    for value in &binding.values {
        let key = (value.path.clone(), value.scan.clone());
        let (field, projection) = expected.get(&key).ok_or_else(fail)?;
        if field != &value.field
            || !scans.contains_key(&value.scan)
            || !digest(&value.comparison_scope)
        {
            return Err(fail());
        }
        // Both the original selected meaning and its effective domain must be supported;
        // a different DTO cannot hide an unsupported original structured/nullable Field.
        for id in [field, &value.domain] {
            let descriptor = original
                .catalog()
                .scalar_descriptor_by_identity(id)
                .map_err(|_| fail())?;
            let Shape::Scalar { logical_type } = descriptor.shape else {
                return Err(fail());
            };
            if !scalar_supported(&logical_type) {
                return Err(fail());
            }
        }
        if *projection {
            if value.mode.is_some()
                || value.original_action.is_some()
                || value.exposure.is_none()
                || value.exposure == Some(Exposure::Original) && value.domain != value.field
            {
                return Err(fail());
            }
        } else {
            if value.exposure.is_some() {
                return Err(fail());
            }
            match value.mode {
                Some(Mode::Disclosed) => {
                    if value.original_action.is_some() {
                        return Err(fail());
                    }
                }
                Some(Mode::OriginalAuthorized) => {
                    if value.domain != value.field
                        || value.original_action.as_deref()
                            != designation.original_action(&value.path, &value.scan)?
                        || value.original_action.is_none()
                    {
                        return Err(fail());
                    }
                }
                _ => return Err(fail()),
            }
        }
        let descriptor = original
            .catalog()
            .scalar_descriptor_by_identity(&value.domain)
            .map_err(|_| fail())?;
        let Shape::Scalar { logical_type } = descriptor.shape else {
            return Err(fail());
        };
        let ty = native_type(&logical_type)?;
        let sql = format!(
            "{}.{}{}",
            ident(&value.scan)?,
            ident(&value.column)?,
            if ty == "text" {
                " COLLATE pg_catalog.\"C\""
            } else {
                ""
            }
        );
        if selected
            .insert(
                key,
                Selected {
                    source: value,
                    logical_type,
                    sql,
                },
            )
            .is_some()
        {
            return Err(fail());
        }
    }
    if selected.len() != expected.len() {
        return Err(fail());
    }
    // Grouped scalar projection requires the exact same effective scalar expression.
    // Different read/raw domains or contributors need a separately qualified fold.
    for (i, o) in plan.outputs.iter().enumerate() {
        if let app::Expression::Field { scan, identity } = &o.expression {
            if plan.aggregate {
                let projection = selected
                    .get(&(format!("outputs/{i}/expression"), scan.clone()))
                    .ok_or_else(fail)?;
                let same = plan.groups.iter().enumerate().any(|(g, f)| {
                    f.scan == *scan
                        && f.identity == *identity
                        && selected
                            .get(&(format!("groups/{g}"), scan.clone()))
                            .is_some_and(|v| {
                                v.sql == projection.sql
                                    && v.source.domain == projection.source.domain
                                    && v.source.comparison_scope
                                        == projection.source.comparison_scope
                            })
                });
                if !same {
                    return Err(fail());
                }
            }
        }
    }
    if plan.aggregate {
        for (i, f) in plan.order.iter().enumerate() {
            let ordered = selected
                .get(&(format!("order/{i}"), f.scan.clone()))
                .ok_or_else(fail)?;
            let grouped = plan.groups.iter().enumerate().any(|(g, key)| {
                key.scan == f.scan
                    && key.identity == f.identity
                    && selected
                        .get(&(format!("groups/{g}"), f.scan.clone()))
                        .is_some_and(|v| {
                            v.sql == ordered.sql
                                && v.source.domain == ordered.source.domain
                                && v.source.comparison_scope == ordered.source.comparison_scope
                        })
            });
            if !grouped {
                return Err(fail());
            }
        }
    }
    let mut build = Build {
        selected,
        used: BTreeSet::new(),
        parameters: Vec::new(),
    };
    let mut joins = Vec::new();
    for (i, j) in plan.joins.iter().enumerate() {
        if j.on.is_empty() {
            return Err(fail());
        }
        let conditions =
            j.on.iter()
                .enumerate()
                .map(|(k, p)| build.predicate(&format!("joins/{i}/on/{k}"), p))
                .collect::<Result<Vec<_>>>()?;
        joins.push(format!(
            " JOIN {} ON {}",
            ident(&j.right.occurrence)?,
            conditions.join(" AND ")
        ));
    }
    let filters = plan
        .filters
        .iter()
        .enumerate()
        .map(|(i, p)| build.predicate(&format!("filters/{i}"), p))
        .collect::<Result<Vec<_>>>()?;
    let groups = plan
        .groups
        .iter()
        .enumerate()
        .map(|(i, f)| {
            build
                .field(&format!("groups/{i}"), &f.scan, &f.identity)
                .map(|v| v.0)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut orders = Vec::new();
    for (i, f) in plan.order.iter().enumerate() {
        let (s, _, _) = build.field(&format!("order/{i}"), &f.scan, &f.identity)?;
        orders.push(format!(
            "{s} {}",
            match original.order_directions()[i] {
                OrderDirection::Asc => "ASC",
                OrderDirection::Desc => "DESC",
            }
        ));
    }
    let mut output_sql = Vec::new();
    let mut columns = Vec::new();
    for (i, o) in plan.outputs.iter().enumerate() {
        let path = format!("outputs/{i}");
        let (sql, source, domain, t, exposure) = match &o.expression {
            app::Expression::Field { scan, identity } => {
                let key = (format!("{path}/expression"), scan.clone());
                let v = build.selected.get(&key).ok_or_else(fail)?;
                let domain = v.source.domain.clone();
                let exposure = if v.source.exposure == Some(Exposure::Original) {
                    "original"
                } else {
                    "transformed"
                };
                let (s, t, _) = build.field(&key.0, scan, identity)?;
                (s, Some(identity.clone()), Some(domain), t, exposure)
            }
            app::Expression::Count { logical_type } => {
                count_type(logical_type)?;
                (
                    "pg_catalog.count(*)".into(),
                    None,
                    None,
                    logical_type.clone(),
                    "computed",
                )
            }
            app::Expression::CountDistinct {
                argument,
                logical_type,
            } => {
                count_type(logical_type)?;
                let (s, _, _) = build.field(
                    &format!("{path}/argument"),
                    &argument.scan,
                    &argument.identity,
                )?;
                (
                    format!("pg_catalog.count(DISTINCT {s})"),
                    Some(argument.identity.clone()),
                    None,
                    logical_type.clone(),
                    "computed",
                )
            }
            _ => return Err(fail()),
        };
        output_sql.push(format!(
            "({sql})::pg_catalog.text AS {}",
            ident(&format!("output_{i}"))?
        ));
        columns.push(ResultColumn {
            position: i,
            name: o.name.clone(),
            path,
            source_field: source,
            effective_domain: domain,
            logical_type: t,
            exposure: exposure.into(),
            encoding: if exposure == "computed" {
                "native-int8-nonnegative-text"
            } else {
                "native-text-exact-required"
            }
            .into(),
        });
    }
    if build.used.len() != expected.len() {
        return Err(fail());
    }
    let private = if let Some(keys) = &row_keys {
        let mut row_sql = Vec::new();
        let mut scan_ids = Vec::new();
        for source in std::iter::once(&plan.source).chain(plan.joins.iter().map(|j| &j.right)) {
            let scan = &source.occurrence;
            let key = keys.get(scan).ok_or_else(fail)?;
            let literal = format!("'{}'", scan.replace('\'', "''"));
            row_sql.push(format!(
                "pg_catalog.jsonb_build_object('scan',{literal},'key',{}.{})",
                ident(scan)?,
                ident(key)?
            ));
            scan_ids.push(scan.clone());
        }
        output_sql.push(format!("pg_catalog.jsonb_build_object('version','weft.stage-row-tuple/0.1-candidate','rows',pg_catalog.jsonb_build_array({}))::pg_catalog.text AS \"__weft_source_rows\"",row_sql.join(",")));
        let mut cells = Vec::new();
        for (position, output) in plan.outputs.iter().enumerate() {
            let app::Expression::Field { scan, identity } = &output.expression else {
                return Err(fail());
            };
            cells.push(json!({"position":position,"sourcePath":format!("outputs/{position}/expression"),"scan":scan,"field":identity}));
        }
        Some((
            json!([{"position":columns.len(),"name":"__weft_source_rows","purpose":"source-row-contribution","encoding":"jsonb-text:weft.stage-row-tuple/0.1-candidate","scans":scan_ids,"maxKeyUtf8Bytes":256}]),
            json!(cells),
        ))
    } else {
        None
    };
    let sql = format!(
        "WITH {} SELECT {} FROM {}{}{}{}{}{}",
        ctes.join(", "),
        output_sql.join(", "),
        ident(&plan.source.occurrence)?,
        joins.join(""),
        if filters.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", filters.join(" AND "))
        },
        if groups.is_empty() {
            String::new()
        } else {
            format!(" GROUP BY {}", groups.join(", "))
        },
        if orders.is_empty() {
            String::new()
        } else {
            format!(" ORDER BY {}", orders.join(", "))
        },
        plan.limit
            .map(|n| format!(" LIMIT {n}"))
            .unwrap_or_default()
    );
    if sql.len() > 1_000_000 {
        return Err(fail());
    }
    Ok(CurrentScalarSql {
        context,
        binding_sha256,
        sql,
        parameters: build.parameters,
        columns,
        private,
    })
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RowRelation {
    schema: String,
    table: String,
    eligible_column: String,
    row_key_column: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RowScan {
    scan: String,
    target: Identity,
    relation: RowRelation,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RowBinding {
    version: String,
    source_request_sha256: String,
    designation_sha256: String,
    source_binding_sha256: String,
    scans: Vec<RowScan>,
    values: Vec<BindingValue>,
}
/// Nonaggregate current rows with private source contribution, never native admission.
/// The host must validate private TEXT keys and keep the sideband from public release.
pub fn lower_rows<'a>(
    context: &'a CurrentQueryOperands<'a>,
    binding_json: &str,
) -> Result<CurrentScalarSql<'a>> {
    if binding_json.len() > 4_000_000 {
        return Err(fail());
    }
    let raw = checked_json(binding_json).map_err(|_| fail())?;
    let binding: RowBinding = serde_json::from_value(raw.clone()).map_err(|_| fail())?;
    if binding.version != "weft.postgresql.current-values/0.2-candidate"
        || serde_json::to_value(&binding).map_err(|_| fail())? != raw
    {
        return Err(fail());
    }
    let mut keys = BTreeMap::new();
    let mut scans = Vec::new();
    for scan in binding.scans {
        ident(&scan.relation.row_key_column)?;
        if keys
            .insert(scan.scan.clone(), scan.relation.row_key_column)
            .is_some()
        {
            return Err(fail());
        }
        scans.push(Scan {
            scan: scan.scan,
            target: scan.target,
            relation: Relation {
                schema: scan.relation.schema,
                table: scan.relation.table,
                eligible_column: scan.relation.eligible_column,
            },
        });
    }
    lower_binding(
        context,
        Binding {
            version: "weft.postgresql.current-values/0.1-candidate".into(),
            source_request_sha256: binding.source_request_sha256,
            designation_sha256: binding.designation_sha256,
            source_binding_sha256: binding.source_binding_sha256,
            scans,
            values: binding.values,
        },
        sha256(binding_json.as_bytes()),
        Some(keys),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use weft_core::{application_lineage, current_query_designation, current_query_operands};
    // @covers US-009-AC2
    #[test]
    fn effective_scalar_lookup_preserves_original_identity_and_refuses_non_scalar() {
        let r = request("mixed");
        let p = application_lineage::prepare(&r.to_string()).unwrap();
        let c = p.catalog();
        let dto = c
            .scalar_descriptor_by_identity(&identity("salaryMaskLabel"))
            .unwrap();
        assert!(
            matches!(dto.shape, Shape::Scalar { logical_type } if logical_type.family == Family::String)
        );
        for id in [
            Identity {
                revision: "other".into(),
                ..identity("salaryMaskLabel")
            },
            Identity {
                module: "unselected".into(),
                ..identity("salaryMaskLabel")
            },
            identity("Resource"),
            identity("missing"),
        ] {
            assert!(c.scalar_descriptor_by_identity(&id).is_err());
        }
    }
    fn rows(r: &Value, edit: impl FnOnce(&mut Value)) -> Result<Value> {
        let p = application_lineage::prepare(&r.to_string())?;
        let d = current_query_designation::derive(&p)?;
        let g = current_query_operands::derive(&d)?;
        let mut b = binding(&g);
        b["version"] = json!("weft.postgresql.current-values/0.2-candidate");
        for scan in b["scans"].as_array_mut().unwrap() {
            scan["relation"]["rowKeyColumn"] = json!("__native_row_key")
        }
        edit(&mut b);
        lower_rows(&g, &b.to_string()).map(|sql| sql.json())
    }
    // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn private_row_contribution_is_separate_from_public_columns_and_preserves_v1() {
        let r = request("mixed");
        let old = run(&r, |_| {}).unwrap();
        let out = rows(&r, |_| {}).unwrap();
        assert_eq!(out["resultColumns"], old["resultColumns"]);
        assert_eq!(out["parameters"], old["parameters"]);
        assert_eq!(
            out["version"],
            "weft.postgresql.current-scalar/0.2-candidate"
        );
        assert_eq!(out["privateResultColumns"][0]["position"], old["resultColumns"].as_array().unwrap().len());
        assert_eq!(out["privateResultColumns"][0]["scans"], json!(["s0"]));
        assert_eq!(
            out["publicCellSources"][0]["sourcePath"],
            "outputs/0/expression"
        );
        assert_eq!(
            out["publicCellSources"][0]["field"],
            json!(identity("salary"))
        );
        let sql = out["sql"].as_str().unwrap();
        assert!(sql.contains("pg_catalog.jsonb_build_object"));
        assert!(sql.contains("\"s0\".\"__native_row_key\""));
        assert!(old.get("privateResultColumns").is_none());
        assert!(!old["sql"].as_str().unwrap().contains("__native_row_key"));
        assert!(rows(&r, |b| {
            b["scans"][0]["relation"]
                .as_object_mut()
                .unwrap()
                .remove("rowKeyColumn");
        })
        .is_err());
        assert!(rows(&r, |b| b["scans"][0]["relation"]["rowKeyColumn"] =
            json!("bad\0key"))
        .is_err());
        assert!(rows(&r, |b| b["scans"][0]["relation"]["rowKeyColumn"] =
            Value::Null)
        .is_err());
    }
    // @covers US-009-AC2
    #[test]
    fn row_versions_and_legacy_unknown_keys_refuse(){
        let r=request("mixed");
        assert!(rows(&r,|b|b["version"]=json!("weft.postgresql.current-values/0.1-candidate")).is_err());
        assert!(run(&r,|b|b["scans"][0]["relation"]["rowKeyColumn"]=json!("__native_row_key")).is_err());
    }
    // @covers US-009-AC1 @covers US-009-AC2
    #[test]
    fn joined_rows_retain_both_sources_and_aggregates_do_not_claim_contribution() {
        let mut r = request("join");
        assert!(rows(&r, |_| {}).is_err());
        r["sql"] = json!("SELECT a.token FROM Resource a JOIN Resource b ON a.token=b.token");
        let out = rows(&r, |_| {}).unwrap();
        assert_eq!(out["privateResultColumns"][0]["scans"], json!(["s0", "s1"]));
        assert_eq!(out["publicCellSources"][0]["scan"], "s0");
        let sql = out["sql"].as_str().unwrap();
        assert!(sql.contains("\"s1\".\"__native_row_key\""));
        r["sql"] = json!("SELECT COUNT(*) FROM Resource a WHERE FALSE LIMIT 0");
        assert!(rows(&r, |_| {}).is_err());
        r["sql"] = json!("SELECT a.token FROM Resource a WHERE FALSE LIMIT 0");
        assert!(rows(&r, |_| {}).unwrap()["sql"]
            .as_str()
            .unwrap()
            .ends_with(" LIMIT 0"));
    }
    fn request(name: &str) -> Value {
        serde_json::from_str(match name{
  "join"=>include_str!("../../../tests/current-query-designation/fixtures/disclosed-token-selfjoin.request.json"),
  _=>include_str!("../../../tests/current-query-designation/fixtures/mixed-original-uses.request.json"),
 }).unwrap()
    }
    fn identity(element: &str) -> Identity {
        Identity {
            document_id: "domain".into(),
            revision: "query-value-core-r1".into(),
            module: "m".into(),
            element: element.into(),
        }
    }
    fn binding(g: &CurrentQueryOperands<'_>) -> Value {
        let d = g.designation();
        let original = d.preparation();
        let p = original.plan();
        let scans=std::iter::once(&p.source).chain(p.joins.iter().map(|j|&j.right)).map(|s|json!({"scan":s.occurrence,"target":s.record,"relation":{"schema":"private_stage","table":"current_values","eligibleColumn":"eligible"}})).collect::<Vec<_>>();
        let mut values = Vec::new();
        for op in g.operations() {
            for operand in op.operands() {
                if let weft_core::current_query_operands::Input::Field {
                    scan,
                    identity,
                    original_action,
                    ..
                } = operand.input()
                {
                    values.push(json!({"path":operand.path(),"scan":scan,"field":identity,"column":identity.element,"domain":identity,"comparisonScope":"a".repeat(64),"mode":if original_action.is_some(){"original-authorized"}else{"disclosed"},"originalAction":original_action,"exposure":null}));
                }
            }
        }
        for (i, o) in p.outputs.iter().enumerate() {
            if let app::Expression::Field { scan, identity: id } = &o.expression {
                let salary = id.element == "salary";
                values.push(json!({"path":format!("outputs/{i}/expression"),"scan":scan,"field":id,"column":if salary{"salary_public"}else{id.element.as_str()},"domain":if salary{identity("salaryMaskLabel")}else{id.clone()},"comparisonScope":"a".repeat(64),"mode":null,"originalAction":null,"exposure":if salary{"transformed"}else{"original"}}));
            }
        }
        json!({"version":"weft.postgresql.current-values/0.1-candidate","sourceRequestSha256":sha256(original.source_request_json().as_bytes()),"designationSha256":sha256(d.json().as_bytes()),"sourceBindingSha256":d.binding_scope().sha256,"scans":scans,"values":values})
    }
    fn run(r: &Value, edit: impl FnOnce(&mut Value)) -> Result<Value> {
        let p = application_lineage::prepare(&r.to_string())?;
        let d = current_query_designation::derive(&p)?;
        let g = current_query_operands::derive(&d)?;
        let mut b = binding(&g);
        edit(&mut b);
        lower(&g, &b.to_string()).map(|s| s.json())
    }
    // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn actual_original_context_lowers_masked_projection_and_separately_typed_raw_predicate() {
        let r = run(&request("mixed"), |_| {}).unwrap();
        let sql = r["sql"].as_str().unwrap();
        assert!(sql.contains("AS MATERIALIZED"));
        assert!(!sql.contains("SELECT *"));
        assert!(sql.contains("\"s0\".\"salary_public\" COLLATE pg_catalog.\"C\""));
        assert!(sql.contains("\"s0\".\"salary\" OPERATOR(pg_catalog.=) $1::pg_catalog.int8"));
        assert!(sql.contains("DESC"));
        assert!(sql.contains("::pg_catalog.text AS \"output_0\""));
        assert_eq!(r["parameters"][0]["value"], "100");
        assert_eq!(
            r["resultColumns"][0]["effectiveDomain"]["element"],
            "salaryMaskLabel"
        );
        assert_eq!(r["resultColumns"][0]["logicalType"]["family"], "string");
        assert_eq!(r["authority"], false);
        assert_eq!(r["securityReady"], false);
        assert_eq!(r["nativeAdmission"], "required");
    }
    // @covers US-009-AC1 @covers US-009-AC2
    #[test]
    fn joins_and_distinct_require_each_occurrence_and_matching_effective_scope() {
        let r = request("join");
        let result = run(&r, |_| {}).unwrap();
        assert!(result["sql"].as_str().unwrap().contains(" JOIN \"s1\""));
        assert!(result["sql"]
            .as_str()
            .unwrap()
            .contains("pg_catalog.count(DISTINCT"));
        assert!(run(&r, |b| b["values"][1]["comparisonScope"] =
            json!("b".repeat(64)))
        .is_err());
        assert!(run(&r, |b| {
            b["values"].as_array_mut().unwrap().pop();
        })
        .is_err());
    }
    // @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn literal_domain_mismatch_wrong_action_and_prohibited_empty_refuse() {
        let r = request("mixed");
        assert!(run(&r, |b| b["values"][0]["originalAction"] = json!("read")).is_err());
        assert!(run(&r, |b| {
            b["values"][0]["mode"] = json!("disclosed");
            b["values"][0]["originalAction"] = Value::Null;
            b["values"][0]["domain"] = json!(identity("salaryMaskLabel"));
        })
        .is_err());
        let mut r = r;
        r["sql"] = json!("SELECT r.token FROM Resource r WHERE FALSE AND r.salary=100 LIMIT 0");
        assert!(run(&r, |b| b["values"][0]["mode"] = json!("prohibited")).is_err());
        let out = run(&r, |_| {}).unwrap();
        assert!(out["sql"].as_str().unwrap().contains("WHERE FALSE AND"));
        assert!(out["sql"].as_str().unwrap().ends_with(" LIMIT 0"));
        assert_eq!(out["parameters"].as_array().unwrap().len(), 1);
    }
    // @covers US-009-AC1 @covers US-009-AC2
    #[test]
    fn hidden_group_with_count_and_canonical_group_projection_are_distinct_from_unsafe_projection_fold(
    ) {
        let mut r = request("mixed");
        r["sql"] = json!("SELECT COUNT(*) AS n FROM Resource r GROUP BY r.salary");
        let out = run(&r, |_| {}).unwrap();
        assert!(out["sql"]
            .as_str()
            .unwrap()
            .contains("GROUP BY \"s0\".\"salary\""));
        r["sql"] = json!(
            "SELECT r.token,COUNT(*) AS n FROM Resource r GROUP BY r.token ORDER BY r.token DESC"
        );
        assert!(run(&r, |_| {}).is_ok());
        r["sql"] = json!("SELECT r.salary,COUNT(*) AS n FROM Resource r GROUP BY r.salary");
        assert!(run(&r, |_| {}).is_err());
    }
    // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn scope_pins_closed_shapes_unknown_sources_and_identifier_injection_refuse_or_quote() {
        let r = request("mixed");
        for key in [
            "sourceRequestSha256",
            "designationSha256",
            "sourceBindingSha256",
        ] {
            assert!(run(&r, |b| b[key] = json!("0".repeat(64))).is_err());
        }
        assert!(run(&r, |b| b["authority"] = json!(true)).is_err());
        assert!(run(&r, |b| b["values"][0]["domain"]["authority"] = json!(true)).is_err());
        assert!(run(&r, |b| {
            let v = b["values"][0].clone();
            b["values"].as_array_mut().unwrap().push(v);
        })
        .is_err());
        let out = run(&r, |b| {
            b["scans"][0]["relation"]["table"] = json!("x\"; DROP TABLE x;--")
        })
        .unwrap();
        assert!(out["sql"]
            .as_str()
            .unwrap()
            .contains("\"x\"\"; DROP TABLE x;--\""));
        assert!(run(&r, |b| b["values"][0]["column"] = json!("bad\0column")).is_err());
    }
    // @covers US-009-AC1 @covers US-009-AC2
    #[test]
    fn named_parameters_preserve_original_payload_and_no_field_count_has_eligibility_source() {
        let mut r = request("mixed");
        r["sql"] = json!("SELECT r.salary FROM Resource r WHERE r.salary=:minimum");
        r["parameters"] = json!({"minimum":{"family":"integer","value":"100"}});
        let out = run(&r, |_| {}).unwrap();
        assert_eq!(out["parameters"][0]["name"], "minimum");
        assert_eq!(out["parameters"][0]["value"], "100");
        r["parameters"] = json!({});
        r["sql"] = json!("SELECT COUNT(*) AS n FROM Resource r WHERE FALSE LIMIT 0");
        let out = run(&r, |_| {}).unwrap();
        assert!(out["sql"]
            .as_str()
            .unwrap()
            .contains("SELECT 1 AS \"__wft_row\""));
        assert!(out["sql"]
            .as_str()
            .unwrap()
            .contains("\"eligible\" IS TRUE"));
    }
}
