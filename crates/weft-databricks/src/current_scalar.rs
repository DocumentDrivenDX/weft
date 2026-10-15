//! Same-original conditional Databricks scalar SQL. Native hosts own authority.
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
        "WFT-CURRENT-DATABRICKS-SQL",
        "capability",
        "Current Databricks lowering refused",
    )
}
fn ident(s: &str) -> Result<String> {
    if s.is_empty() || s.len() > 255 || s.contains('\0') {
        return Err(fail());
    }
    Ok(format!("`{}`", s.replace('`', "``")))
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Relation {
    catalog: String,
    uuid: String,
    version: String,
    schema_sha256: String,
    row_key_column: String,
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
    pub marker: String,
    pub kind: String,
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
    relations: Value,
    sql: String,
    parameters: Vec<Parameter>,
    columns: Vec<ResultColumn>,
    private: Option<PrivateLineage>,
}
struct PrivateLineage {
    version: &'static str,
    columns: Value,
    cells: Value,
    groups: Option<Value>,
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
        let mut value = json!({"version":"weft.databricks.current-scalar/0.1-candidate","authority":false,"securityReady":false,"nativeAdmission":"required","sourceRequestSha256":sha256(self.context.designation().preparation().source_request_json().as_bytes()),"designationSha256":sha256(self.context.designation().json().as_bytes()),"materializationBindingSha256":self.binding_sha256,"sql":self.sql,"parameters":self.parameters,"resultColumns":self.columns,"nativeRequirements":["genuine authenticated held source/caller/Subject/current cut","exact original static binding/profile/model/source correspondence","genuine native Delta UUID/version/schema fingerprint, ACL/policy/encoding/collation and builtin correspondence","complete primary-eligible population before relational operations even when empty","actual per-use read/route/disclosure and exact separate action decisions","present uniform per-use domains and paired/cross-row key/transform/runtime scopes","native statement/result budgets and no partial public release","source/caller/current-cut revalidation, result/cell lineage and guarded decoding/release"]});
        if let Some(private) = &self.private {
            value["version"] = json!(private.version);
            value["privateResultColumns"] = private.columns.clone();
            value["publicCellSources"] = private.cells.clone();
            if let Some(groups) = &private.groups {
                value["privateGroupSources"] = groups.clone();
            }
        }
        value["relations"] = self.relations.clone();
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
        "STRING"
    } else {
        "BIGINT"
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
                    marker: format!("wft_{ordinal}"),
                    kind: "source-value".into(),
                    source_path: path.into(),
                    name,
                    value: value.clone(),
                    logical_type: logical_type.clone(),
                    native_type: ty.into(),
                });
                let sql = format!(
                    "CAST(:wft_{ordinal} AS {ty}){}",
                    if ty == "STRING" {
                        " COLLATE UTF8_BINARY"
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
                Ok(format!("({a} = {b})"))
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
    parse(context, binding_json, false)
}
/// Explicit original signed64 SUM context; never a native permission.
pub fn lower_original_sum<'a>(
    context: &'a CurrentQueryOperands<'a>,
    binding_json: &str,
) -> Result<CurrentScalarSql<'a>> {
    parse(context, binding_json, true)
}
fn parse<'a>(
    context: &'a CurrentQueryOperands<'a>,
    binding_json: &str,
    live_sum: bool,
) -> Result<CurrentScalarSql<'a>> {
    if binding_json.len() > 4_000_000 {
        return Err(fail());
    }
    let raw = checked_json(binding_json).map_err(|_| fail())?;
    let binding: Binding = serde_json::from_value(raw.clone()).map_err(|_| fail())?;
    let version = if live_sum {
        "weft.databricks.current-values/0.2-candidate"
    } else {
        "weft.databricks.current-values/0.1-candidate"
    };
    if binding.version != version || serde_json::to_value(&binding).map_err(|_| fail())? != raw {
        return Err(fail());
    }
    lower_binding(context, binding, sha256(binding_json.as_bytes()), live_sum)
}
fn native_version(s: &str) -> Result<i64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) || s.len() > 1 && s.starts_with('0') {
        return Err(fail());
    }
    s.parse::<i64>().map_err(|_| fail())
}
fn uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
            }
        })
}
fn lower_binding<'a>(
    context: &'a CurrentQueryOperands<'a>,
    binding: Binding,
    binding_sha256: String,
    live_sum: bool,
) -> Result<CurrentScalarSql<'a>> {
    let designation = context.designation();
    let original = designation.preparation();
    let plan = original.plan();
    if binding.source_request_sha256 != sha256(original.source_request_json().as_bytes())
        || binding.designation_sha256 != sha256(designation.json().as_bytes())
        || binding.source_binding_sha256 != designation.binding_scope().sha256
        || binding.scans.len() > 16
        || binding.values.len() > 4096
        || plan.joins.len() > 8
    {
        return Err(fail());
    }
    if designation.supports_live_original_integer_sum() != live_sum {
        return Err(fail());
    }
    let keys: BTreeMap<_, _> = binding
        .scans
        .iter()
        .map(|s| (s.scan.clone(), s.relation.row_key_column.clone()))
        .collect();
    let row_keys = Some(&keys);
    let aliases: BTreeMap<_, _> = std::iter::once(&plan.source)
        .chain(plan.joins.iter().map(|j| &j.right))
        .enumerate()
        .map(|(i, s)| (s.occurrence.clone(), format!("wft_s{i}")))
        .collect();
    let alias = |scan: &str| -> Result<String> { ident(aliases.get(scan).ok_or_else(fail)?) };
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
        if !uuid(&scan.relation.uuid) || !digest(&scan.relation.schema_sha256) {
            return Err(fail());
        }
        let version = native_version(&scan.relation.version)?;
        let alias = alias(&scan.scan)?;
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
            "{alias} AS (SELECT {select} FROM {}.{}.{} VERSION AS OF {version} WHERE {} IS TRUE)",
            ident(&scan.relation.catalog)?,
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
            alias(&value.scan)?,
            ident(&value.column)?,
            if ty == "STRING" {
                " COLLATE UTF8_BINARY"
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
            alias(&j.right.occurrence)?,
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
            app::Expression::Sum {
                argument,
                logical_type,
            } if live_sum => {
                if logical_type.family != Family::Integer
                    || logical_type.nullable != plan.groups.is_empty()
                    || logical_type.facets != json!({})
                {
                    return Err(fail());
                }
                let key = (format!("{path}/argument"), argument.scan.clone());
                let input = build.selected.get(&key).ok_or_else(fail)?;
                if input.source.mode != Some(Mode::OriginalAuthorized)
                    || input.source.domain != argument.identity
                    || input.logical_type.family != Family::Integer
                    || input.logical_type.facets
                        != json!({"integerWidth":{"bits":64,"signed":true}})
                    || input.logical_type.nullable
                {
                    return Err(fail());
                }
                let (sql, _, _) = build.field(&key.0, &argument.scan, &argument.identity)?;
                (
                    format!("CASE WHEN system.builtin.count(*) = 0 THEN CAST(NULL AS DECIMAL(38,0)) WHEN system.builtin.try_sum(CAST({sql} AS DECIMAL(38,0))) IS NULL THEN system.builtin.raise_error('WFT-CAPABILITY') ELSE system.builtin.try_sum(CAST({sql} AS DECIMAL(38,0))) END"),
                    Some(argument.identity.clone()),
                    None,
                    logical_type.clone(),
                    "computed",
                )
            }
            app::Expression::Count { logical_type } => {
                count_type(logical_type)?;
                (
                    "CASE WHEN system.builtin.count(*) < 0 THEN system.builtin.raise_error('WFT-CAPABILITY') ELSE system.builtin.count(*) END".into(),
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
                    format!("CASE WHEN system.builtin.count(DISTINCT {s}) < 0 THEN system.builtin.raise_error('WFT-CAPABILITY') ELSE system.builtin.count(DISTINCT {s}) END"),
                    Some(argument.identity.clone()),
                    None,
                    logical_type.clone(),
                    "computed",
                )
            }
            _ => return Err(fail()),
        };
        output_sql.push(format!(
            "CAST(({sql}) AS STRING) AS {}",
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
            encoding: if let app::Expression::Sum { logical_type, .. } = &o.expression {
                if logical_type.nullable {
                    "native-decimal38-integer-nullable-string"
                } else {
                    "native-decimal38-integer-required-string"
                }
            } else if exposure == "computed" {
                "native-bigint-nonnegative-string"
            } else {
                "native-string-exact-required"
            }
            .into(),
        });
    }
    if build.used.len() != expected.len() {
        return Err(fail());
    }
    let mut row_sql = Vec::new();
    let mut scan_ids = Vec::new();
    for (index, source) in std::iter::once(&plan.source)
        .chain(plan.joins.iter().map(|j| &j.right))
        .enumerate()
    {
        let scan = &source.occurrence;
        let key = keys.get(scan).ok_or_else(fail)?;
        if build.parameters.len() >= 1024 {
            return Err(fail());
        }
        let ordinal = build.parameters.len() + 1;
        let marker = format!("wft_{ordinal}");
        build.parameters.push(Parameter {
            ordinal,
            marker: marker.clone(),
            kind: "internal-lineage-label".into(),
            source_path: format!("$lineage/scans/{index}"),
            name: None,
            value: scan.clone(),
            logical_type: LogicalType {
                family: Family::String,
                nullable: false,
                facets: json!({}),
            },
            native_type: "STRING".into(),
        });
        row_sql.push(format!(
            "system.builtin.named_struct('scan',CAST(:{marker} AS STRING),'key',{}.{})",
            alias(scan)?,
            ident(key)?
        ));
        scan_ids.push(scan.clone());
    }
    let tuple = format!("system.builtin.array({})", row_sql.join(","));
    let tuples = if plan.aggregate {
        format!("system.builtin.collect_list({tuple})")
    } else {
        format!("system.builtin.array({tuple})")
    };
    output_sql.push(format!("system.builtin.to_json(system.builtin.named_struct('version','weft.stage-contributor-bag/0.1-candidate','tuples',{tuples})) AS `__weft_contributors`"));
    let mut cells = Vec::new();
    for (position, output) in plan.outputs.iter().enumerate() {
        let (kind, path, sources) = match &output.expression {
            app::Expression::Field { scan, identity } => {
                let path = format!("outputs/{position}/expression");
                (
                    if plan.aggregate {
                        "group-field"
                    } else {
                        "field"
                    },
                    path.clone(),
                    json!([{"path":path,"scan":scan,"field":identity}]),
                )
            }
            app::Expression::Sum { argument, .. } if live_sum => (
                "sum",
                format!("outputs/{position}"),
                json!([{"path":format!("outputs/{position}/argument"),"scan":argument.scan,"field":argument.identity}]),
            ),
            app::Expression::Count { .. } => ("count", format!("outputs/{position}"), json!([])),
            app::Expression::CountDistinct { argument, .. } => (
                "count-distinct",
                format!("outputs/{position}"),
                json!([{"path":format!("outputs/{position}/argument"),"scan":argument.scan,"field":argument.identity}]),
            ),
            _ => return Err(fail()),
        };
        cells.push(json!({"position":position,"kind":kind,"sourcePath":path,"sources":sources}));
    }
    let group_sources=plan.groups.iter().enumerate().map(|(index,field)|json!({"path":format!("groups/{index}"),"scan":field.scan,"field":field.identity})).collect::<Vec<_>>();
    let private = Some(PrivateLineage {
        version: if live_sum {
            "weft.databricks.current-scalar/0.2-candidate"
        } else {
            "weft.databricks.current-scalar/0.1-candidate"
        },
        columns: json!([{"position":columns.len(),"name":"__weft_contributors","purpose":"source-row-contributor-bag","encoding":"json-text:weft.stage-contributor-bag/0.1-candidate","scans":scan_ids,"maxKeyUtf8Bytes":256,"multiplicity":"bag"}]),
        cells: json!(cells),
        groups: Some(json!(group_sources)),
    });
    let sql = format!(
        "WITH {} SELECT {} FROM {}{}{}{}{}{}",
        ctes.join(", "),
        output_sql.join(", "),
        alias(&plan.source.occurrence)?,
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
        relations: serde_json::to_value(&binding.scans).map_err(|_| fail())?,
        sql,
        parameters: build.parameters,
        columns,
        private,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use weft_core::{application_lineage, current_query_designation, current_query_operands};
    fn request(sum: bool) -> Value {
        serde_json::from_str(if sum {include_str!("../../../tests/current-query-designation/fixtures/live-original-integer-sum.request.json")} else {include_str!("../../../tests/current-query-designation/fixtures/mixed-original-uses.request.json")}).unwrap()
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
        let scans=std::iter::once(&p.source).chain(p.joins.iter().map(|j|&j.right)).map(|s|json!({"scan":s.occurrence,"target":s.record,"relation":{"catalog":"client_dev","schema":"private_stage","table":"current_values","uuid":"00000000-0000-4000-8000-000000000000","version":"0","schemaSha256":"b".repeat(64),"eligibleColumn":"eligible","rowKeyColumn":"__native_row_key"}})).collect::<Vec<_>>();
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
        json!({"version":if d.supports_live_original_integer_sum(){"weft.databricks.current-values/0.2-candidate"}else{"weft.databricks.current-values/0.1-candidate"},"sourceRequestSha256":sha256(original.source_request_json().as_bytes()),"designationSha256":sha256(d.json().as_bytes()),"sourceBindingSha256":d.binding_scope().sha256,"scans":scans,"values":values})
    }
    fn run(r: &Value, sum: bool, edit: impl FnOnce(&mut Value)) -> Result<Value> {
        let p = application_lineage::prepare(&r.to_string())?;
        let d = current_query_designation::derive(&p)?;
        let g = current_query_operands::derive(&d)?;
        let mut b = binding(&g);
        edit(&mut b);
        (if sum {
            lower_original_sum(&g, &b.to_string())
        } else {
            lower(&g, &b.to_string())
        })
        .map(|s| s.json())
    }
    // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn actual_context_emits_dialect_specific_masked_read_raw_predicate_and_private_bag() {
        let out = run(&request(false), false, |_| {}).unwrap();
        let s = out["sql"].as_str().unwrap();
        for part in [
            "`client_dev`.`private_stage`.`current_values` VERSION AS OF 0",
            "`eligible` IS TRUE",
            "`wft_s0`.`salary_public` COLLATE UTF8_BINARY",
            "CAST(:wft_1 AS BIGINT)",
            "system.builtin.to_json",
            "system.builtin.named_struct",
            "AS `__weft_contributors`",
            "DESC",
        ] {
            assert!(s.contains(part), "{part}: {s}");
        }
        for forbidden in ["pg_catalog", "MATERIALIZED", "SELECT *", "$1", "jsonb"] {
            assert!(!s.contains(forbidden));
        }
        assert_eq!(out["parameters"][0]["value"], "100");
        assert_eq!(out["parameters"][0]["kind"], "source-value");
        let label = out["parameters"].as_array().unwrap().last().unwrap();
        assert_eq!(label["kind"], "internal-lineage-label");
        assert_eq!(label["value"], "s0");
        assert_eq!(
            out["resultColumns"][0]["effectiveDomain"]["element"],
            "salaryMaskLabel"
        );
        assert_eq!(out["authority"], false);
        assert_eq!(out["securityReady"], false);
        assert_eq!(out["nativeAdmission"], "required");
        assert_eq!(
            out["publicCellSources"][0]["sourcePath"],
            "outputs/0/expression"
        );
        assert_eq!(out["relations"][0]["relation"]["version"], "0");
    }
    // @covers US-009-AC1 @covers US-009-AC2
    #[test]
    fn version_profiles_never_downgrade_even_for_empty_count() {
        for (sum, sql) in [
            (
                false,
                "SELECT COUNT(*) AS n FROM Resource r WHERE FALSE LIMIT 0",
            ),
            (
                true,
                "SELECT COUNT(*) AS n FROM Resource r WHERE FALSE LIMIT 0",
            ),
        ] {
            let mut r = request(sum);
            r["sql"] = json!(sql);
            assert!(run(&r, sum, |_| {}).is_ok());
            assert!(run(&r, !sum, |_| {}).is_err());
            for version in [
                "weft.postgresql.current-values/0.1-candidate",
                "weft.postgresql.current-values/0.4-candidate",
                "weft.databricks.current-values/9.0-candidate",
            ] {
                assert!(run(&r, sum, |b| b["version"] = json!(version)).is_err());
            }
        }
    }
    // @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn closed_binding_pins_presence_identities_and_modes_refuse() {
        let r = request(false);
        for key in [
            "sourceRequestSha256",
            "designationSha256",
            "sourceBindingSha256",
        ] {
            assert!(run(&r, false, |b| b[key] = json!("0".repeat(64))).is_err());
        }
        for key in [
            "sourceRequestSha256",
            "designationSha256",
            "sourceBindingSha256",
            "scans",
            "values",
        ] {
            assert!(run(&r, false, |b| {
                b.as_object_mut().unwrap().remove(key);
            })
            .is_err());
        }
        for mode in ["prohibited", "unresolved"] {
            assert!(run(&r, false, |b| b["values"][0]["mode"] = json!(mode)).is_err());
        }
        assert!(run(&r, false, |b| b["authority"] = json!(true)).is_err());
        assert!(run(&r, false, |b| b["values"][0]["field"]["extra"] =
            json!(true))
        .is_err());
        assert!(run(&r, false, |b| b["values"][0]["originalAction"] =
            json!("read"))
        .is_err());
        assert!(run(&r, false, |b| {
            b["values"].as_array_mut().unwrap().pop();
        })
        .is_err());
        assert!(run(&r, false, |b| {
            let v = b["values"][0].clone();
            b["values"].as_array_mut().unwrap().push(v);
        })
        .is_err());
        assert!(run(&r, false, |b| {
            let v = b["scans"][0].clone();
            b["scans"].as_array_mut().unwrap().push(v);
        })
        .is_err());
        assert!(run(&r, false, |b| {
            b["values"][0]["mode"] = json!("disclosed");
            b["values"][0]["originalAction"] = Value::Null;
            b["values"][0]["domain"] = json!(identity("salaryMaskLabel"));
        })
        .is_err());
    }
    // @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn identifiers_quote_without_interpolation_and_version_numbers_are_exact() {
        let r = request(false);
        let out = run(&r, false, |b| {
            b["scans"][0]["relation"]["table"] = json!("x`; DROP TABLE y;--")
        })
        .unwrap();
        assert!(out["sql"]
            .as_str()
            .unwrap()
            .contains("`x``; DROP TABLE y;--`"));
        for v in [
            json!(0),
            json!("-1"),
            json!("01"),
            json!("1; SELECT 2"),
            json!("9223372036854775808"),
            json!(""),
            json!("0.0"),
        ] {
            assert!(run(&r, false, |b| b["scans"][0]["relation"]["version"] = v).is_err());
        }
        for key in [
            "catalog",
            "schema",
            "table",
            "eligibleColumn",
            "rowKeyColumn",
        ] {
            for v in ["".to_owned(), "x".repeat(256), "bad\0name".into()] {
                assert!(
                    run(&r, false, |b| b["scans"][0]["relation"][key] = json!(v)).is_err(),
                    "{key}"
                );
            }
        }
        for key in ["uuid", "schemaSha256"] {
            assert!(run(&r, false, |b| b["scans"][0]["relation"][key] =
                json!("invalid"))
            .is_err());
        }
    }
    // @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn complete_group_and_join_bags_preserve_typed_scan_order() {
        let mut r = request(false);
        r["sql"]=json!("SELECT a.token, COUNT(DISTINCT b.token) AS n FROM Resource a JOIN Resource b ON a.token=b.token GROUP BY a.token ORDER BY a.token");
        let out = run(&r, false, |_| {}).unwrap();
        let s = out["sql"].as_str().unwrap();
        assert!(s.contains("JOIN `wft_s1`"));
        assert!(s.contains("system.builtin.collect_list(system.builtin.array("));
        assert!(!s.contains("collect_list(DISTINCT"));
        assert_eq!(out["privateResultColumns"][0]["scans"], json!(["s0", "s1"]));
        assert_eq!(out["publicCellSources"][1]["kind"], "count-distinct");
        assert_eq!(out["privateGroupSources"][0]["path"], "groups/0");
        assert!(run(&r, false, |b| b["values"][1]["comparisonScope"] =
            json!("c".repeat(64)))
        .is_err());
    }
    // @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn grouped_projection_cannot_silently_fold_a_raw_group_into_a_masked_dto() {
        let mut r = request(false);
        r["sql"] = json!("SELECT r.salary,COUNT(*) AS n FROM Resource r GROUP BY r.salary");
        assert!(run(&r, false, |_| {}).is_err());
        r["sql"] = json!("SELECT COUNT(*) AS n FROM Resource r GROUP BY r.salary");
        assert!(run(&r, false, |_| {}).is_ok());
    }
    // @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn empty_queries_do_not_bypass_admission_and_all_counts_keep_lineage() {
        let mut r = request(false);
        r["sql"] = json!("SELECT COUNT(DISTINCT r.token) AS n FROM Resource r WHERE FALSE LIMIT 0");
        assert!(run(&r, false, |b| b["values"][0]["mode"] = json!("prohibited")).is_err());
        r["sql"] = json!("SELECT COUNT(*) AS n FROM Resource r WHERE FALSE LIMIT 0");
        let out = run(&r, false, |_| {}).unwrap();
        let s = out["sql"].as_str().unwrap();
        assert!(s.contains("SELECT `__native_row_key`"));
        assert!(s.ends_with("LIMIT 0"));
        assert_eq!(out["publicCellSources"][0]["sources"], json!([]));
        assert_eq!(out["resultColumns"][0]["logicalType"]["facets"], json!({}));
    }
    // @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn original_sum_uses_guarded_decimal_accumulation_and_contextual_nullity() {
        for grouped in [false, true] {
            let mut r = request(true);
            if grouped {
                r["sql"] = json!("SELECT SUM(r.salary) AS total FROM Resource r GROUP BY r.salary");
            }
            let out = run(&r, true, |_| {}).unwrap();
            let s = out["sql"].as_str().unwrap();
            assert!(s.contains("system.builtin.try_sum(CAST(`wft_s0`.`salary` AS DECIMAL(38,0)))"));
            assert!(s.contains("system.builtin.raise_error('WFT-CAPABILITY')"));
            assert!(!s.contains("salary_public"));
            assert_eq!(out["resultColumns"][0]["logicalType"]["nullable"], !grouped);
            assert_eq!(out["publicCellSources"][0]["kind"], "sum");
            assert_eq!(
                out["version"],
                "weft.databricks.current-scalar/0.2-candidate"
            );
        }
        for mode in ["disclosed", "prohibited", "unresolved"] {
            assert!(run(&request(true), true, |b| b["values"][0]["mode"] =
                json!(mode))
            .is_err());
        }
    }
    // @covers US-009-AC1 @covers US-009-AC2
    #[test]
    fn duplicate_json_keys_and_explicit_null_omission_refuse() {
        let r = request(false);
        let p = application_lineage::prepare(&r.to_string()).unwrap();
        let d = current_query_designation::derive(&p).unwrap();
        let g = current_query_operands::derive(&d).unwrap();
        let b = binding(&g);
        let raw = b.to_string();
        let duplicate = format!(
            "{{\"version\":\"{}\",{}",
            b["version"].as_str().unwrap(),
            &raw[1..]
        );
        assert!(lower(&g, &duplicate).is_err());
        for key in ["mode", "originalAction", "exposure"] {
            assert!(run(&r, false, |b| {
                b["values"][0].as_object_mut().unwrap().remove(key);
            })
            .is_err());
        }
    }
}
