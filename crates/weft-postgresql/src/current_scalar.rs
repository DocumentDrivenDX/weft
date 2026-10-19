// SPDX-License-Identifier: MIT OR Apache-2.0
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
    private: Option<PrivateLineage>,
}
struct PrivateLineage {
    version: &'static str,
    columns: Value,
    cells: Value,
    groups: Option<Value>,
}
struct Lineage {
    keys: BTreeMap<String, String>,
    contributor_bag: bool,
    live_original_integer_sum: bool,
    bounded_unicode_strings: bool,
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
        if let Some(private) = &self.private {
            value["version"] = json!(private.version);
            value["privateResultColumns"] = private.columns.clone();
            value["publicCellSources"] = private.cells.clone();
            if let Some(groups) = &private.groups {
                value["privateGroupSources"] = groups.clone();
            }
        }
        if self.context.designation().supports_bounded_unicode_strings() {
            value["capabilities"]=json!(["bounded-unicode-scalars"]);
            value["nativeRequirements"].as_array_mut().unwrap().push(json!("complete original/effective Unicode scalar length domain checks on every captured eligible cell before relational evaluation and every public cell before guarded release; SQL TEXT alone is insufficient"));
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
    bounded_unicode_strings: bool,
    selected: BTreeMap<(String, String), Selected<'a>>,
    used: BTreeSet<(String, String)>,
    parameters: Vec<Parameter>,
}
fn scalar_supported(t: &LogicalType, bounded: bool) -> bool {
    weft_core::current_string_domain::supported(t,bounded)
}
fn count_type(t: &LogicalType) -> Result<()> {
    if t.family != Family::Integer || t.nullable || t.facets != json!({}) {
        return Err(fail());
    }
    Ok(())
}
fn native_type(t: &LogicalType, bounded: bool) -> Result<&'static str> {
    if !scalar_supported(t,bounded) {
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
                let ty = native_type(logical_type,self.bounded_unicode_strings)?;
                if logical_type.family==Family::String { weft_core::current_string_domain::validate(logical_type,value,self.bounded_unicode_strings).map_err(|_|fail())?; }
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
                let compatible=if sb.is_some() {
                    weft_core::current_string_domain::same_domain(&ta,&tb,self.bounded_unicode_strings)
                } else if let app::Value::Literal{value,..}|app::Value::Parameter{value,..}=right {
                    ta==tb || weft_core::current_string_domain::literal_compatible(&tb,&ta,value,self.bounded_unicode_strings)
                } else {false};
                if !compatible || sb.is_some_and(|s| s != sa) {
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
    lineage: Option<Lineage>,
) -> Result<CurrentScalarSql<'a>> {
    let designation = context.designation();
    let bounded=lineage.as_ref().is_some_and(|l|l.bounded_unicode_strings);
    if designation.supports_bounded_unicode_strings()!=bounded { return Err(fail()); }
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
    let row_keys = lineage.as_ref().map(|value| &value.keys);
    let contributor_bag = lineage.as_ref().is_some_and(|value| value.contributor_bag);
    let live_sum = lineage
        .as_ref()
        .is_some_and(|value| value.live_original_integer_sum);
    if designation.supports_live_original_integer_sum() != live_sum {
        return Err(fail());
    }
    if row_keys.is_some() && plan.aggregate && !contributor_bag {
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
            if !scalar_supported(&logical_type,bounded) {
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
        let ty = native_type(&logical_type,bounded)?;
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
        bounded_unicode_strings:bounded,
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
                    format!("pg_catalog.sum({sql})"),
                    Some(argument.identity.clone()),
                    None,
                    logical_type.clone(),
                    "computed",
                )
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
            encoding: if let app::Expression::Sum { logical_type, .. } = &o.expression {
                if logical_type.nullable {
                    "native-numeric-integer-nullable-text"
                } else {
                    "native-numeric-integer-required-text"
                }
            } else if exposure == "computed" {
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
        if contributor_bag {
            let tuple = format!("pg_catalog.jsonb_build_array({})", row_sql.join(","));
            let tuples = if plan.aggregate {
                format!("COALESCE(pg_catalog.jsonb_agg({tuple}),'[]'::pg_catalog.jsonb)")
            } else {
                format!("pg_catalog.jsonb_build_array({tuple})")
            };
            output_sql.push(format!("pg_catalog.jsonb_build_object('version','weft.stage-contributor-bag/0.1-candidate','tuples',{tuples})::pg_catalog.text AS \"__weft_contributors\""));
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
                    app::Expression::Count { .. } => {
                        ("count", format!("outputs/{position}"), json!([]))
                    }
                    app::Expression::CountDistinct { argument, .. } => (
                        "count-distinct",
                        format!("outputs/{position}"),
                        json!([{"path":format!("outputs/{position}/argument"),"scan":argument.scan,"field":argument.identity}]),
                    ),
                    _ => return Err(fail()),
                };
                cells.push(
                    json!({"position":position,"kind":kind,"sourcePath":path,"sources":sources}),
                );
            }
            let groups=plan.groups.iter().enumerate().map(|(index,field)|json!({"path":format!("groups/{index}"),"scan":field.scan,"field":field.identity})).collect::<Vec<_>>();
            Some(PrivateLineage {
                version: if bounded { "weft.postgresql.current-scalar/0.6-candidate" } else if live_sum {
                    "weft.postgresql.current-scalar/0.4-candidate"
                } else {
                    "weft.postgresql.current-scalar/0.3-candidate"
                },
                columns: json!([{"position":columns.len(),"name":"__weft_contributors","purpose":"source-row-contributor-bag","encoding":"jsonb-text:weft.stage-contributor-bag/0.1-candidate","scans":scan_ids,"maxKeyUtf8Bytes":256,"multiplicity":"bag"}]),
                cells: json!(cells),
                groups: Some(json!(groups)),
            })
        } else {
            output_sql.push(format!("pg_catalog.jsonb_build_object('version','weft.stage-row-tuple/0.1-candidate','rows',pg_catalog.jsonb_build_array({}))::pg_catalog.text AS \"__weft_source_rows\"",row_sql.join(",")));
            let mut cells = Vec::new();
            for (position, output) in plan.outputs.iter().enumerate() {
                let app::Expression::Field { scan, identity } = &output.expression else {
                    return Err(fail());
                };
                cells.push(json!({"position":position,"sourcePath":format!("outputs/{position}/expression"),"scan":scan,"field":identity}));
            }
            Some(PrivateLineage {
                version: if bounded {"weft.postgresql.current-scalar/0.5-candidate"} else {"weft.postgresql.current-scalar/0.2-candidate"},
                columns: json!([{"position":columns.len(),"name":"__weft_source_rows","purpose":"source-row-contribution","encoding":"jsonb-text:weft.stage-row-tuple/0.1-candidate","scans":scan_ids,"maxKeyUtf8Bytes":256}]),
                cells: json!(cells),
                groups: None,
            })
        }
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
    #[serde(default,skip_serializing_if="Option::is_none")]
    capabilities: Option<Vec<String>>,
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
    lower_keyed(
        context,
        binding_json,
        "weft.postgresql.current-values/0.2-candidate",
        false,
        false,
    )
}
/// Complete joined contributor bags for supported direct/grouped COUNT operations.
/// Native source/mapping/domain admission and guarded decoding/release are mandatory.
pub fn lower_contributors<'a>(
    context: &'a CurrentQueryOperands<'a>,
    binding_json: &str,
) -> Result<CurrentScalarSql<'a>> {
    lower_keyed(
        context,
        binding_json,
        "weft.postgresql.current-values/0.3-candidate",
        true,
        false,
    )
}
/// Explicit live-original signed64 SUM capability with exact nullable integer output.
pub fn lower_original_sum<'a>(
    context: &'a CurrentQueryOperands<'a>,
    binding_json: &str,
) -> Result<CurrentScalarSql<'a>> {
    lower_keyed(
        context,
        binding_json,
        "weft.postgresql.current-values/0.4-candidate",
        true,
        true,
    )
}
/// Explicit bounded Unicode scalar candidate; no native authorization is conferred.
#[cfg(feature="bounded-current-string-candidate")]
pub fn lower_bounded_rows<'a>(context:&'a CurrentQueryOperands<'a>,binding_json:&str)->Result<CurrentScalarSql<'a>> {
    lower_keyed(context,binding_json,"weft.postgresql.current-values/0.5-candidate",false,false)
}
/// Explicit bounded Unicode scalar candidate with complete contributor bags.
#[cfg(feature="bounded-current-string-candidate")]
pub fn lower_bounded_contributors<'a>(context:&'a CurrentQueryOperands<'a>,binding_json:&str)->Result<CurrentScalarSql<'a>> {
    lower_keyed(context,binding_json,"weft.postgresql.current-values/0.5-candidate",true,false)
}
fn lower_keyed<'a>(
    context: &'a CurrentQueryOperands<'a>,
    binding_json: &str,
    version: &str,
    contributor_bag: bool,
    live_original_integer_sum: bool,
) -> Result<CurrentScalarSql<'a>> {
    if binding_json.len() > 4_000_000 {
        return Err(fail());
    }
    let raw = checked_json(binding_json).map_err(|_| fail())?;
    let binding: RowBinding = serde_json::from_value(raw.clone()).map_err(|_| fail())?;
    let bounded=version=="weft.postgresql.current-values/0.5-candidate";
    if context.designation().supports_bounded_unicode_strings()!=bounded
        || (if bounded { binding.capabilities.as_deref()!=Some(&["bounded-unicode-scalars".to_owned()][..]) } else { binding.capabilities.is_some() })
        || binding.version != version || serde_json::to_value(&binding).map_err(|_| fail())? != raw {
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
        Some(Lineage {
            keys,
            contributor_bag,
            live_original_integer_sum,
            bounded_unicode_strings:bounded,
        }),
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
        assert_eq!(
            out["privateResultColumns"][0]["position"],
            old["resultColumns"].as_array().unwrap().len()
        );
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
    fn row_versions_and_legacy_unknown_keys_refuse() {
        let r = request("mixed");
        assert!(rows(&r, |b| b["version"] =
            json!("weft.postgresql.current-values/0.1-candidate"))
        .is_err());
        assert!(run(&r, |b| b["scans"][0]["relation"]["rowKeyColumn"] =
            json!("__native_row_key"))
        .is_err());
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
    fn contributors(r: &Value, edit: impl FnOnce(&mut Value)) -> Result<Value> {
        let p = application_lineage::prepare(&r.to_string())?;
        let d = current_query_designation::derive(&p)?;
        let g = current_query_operands::derive(&d)?;
        let mut b = binding(&g);
        b["version"] = json!("weft.postgresql.current-values/0.3-candidate");
        for scan in b["scans"].as_array_mut().unwrap() {
            scan["relation"]["rowKeyColumn"] = json!("__native_row_key")
        }
        edit(&mut b);
        lower_contributors(&g, &b.to_string()).map(|sql| sql.json())
    }
    // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn contributor_bags_keep_count_and_distinct_input_rows_with_empty_aggregate_carrier() {
        let mut r = request("join");
        let out = contributors(&r, |_| {}).unwrap();
        let sql = out["sql"].as_str().unwrap();
        assert!(sql.contains("pg_catalog.count(DISTINCT"));
        assert!(sql.contains("pg_catalog.jsonb_agg(pg_catalog.jsonb_build_array("));
        assert!(!sql.contains("jsonb_agg(DISTINCT"));
        assert!(sql.contains("'[]'::pg_catalog.jsonb"));
        assert_eq!(
            out["version"],
            "weft.postgresql.current-scalar/0.3-candidate"
        );
        assert_eq!(out["privateResultColumns"][0]["scans"], json!(["s0", "s1"]));
        assert_eq!(out["privateResultColumns"][0]["multiplicity"], "bag");
        assert_eq!(out["publicCellSources"][0]["kind"], "count-distinct");
        assert_eq!(
            out["publicCellSources"][0]["sources"][0]["path"],
            "outputs/0/argument"
        );
        assert_eq!(out["resultColumns"][0]["logicalType"]["facets"], json!({}));
        r["sql"] = json!("SELECT COUNT(*) AS n FROM Resource a WHERE FALSE LIMIT 0");
        let out = contributors(&r, |_| {}).unwrap();
        assert_eq!(out["publicCellSources"][0]["kind"], "count");
        assert_eq!(out["publicCellSources"][0]["sources"], json!([]));
        assert_eq!(out["privateGroupSources"], json!([]));
        assert!(out["sql"].as_str().unwrap().ends_with(" LIMIT 0"));
    }
    // @covers US-009-AC1 @covers US-009-AC2
    #[test]
    fn grouped_field_count_and_private_group_paths_preserve_effective_domain_checks() {
        let mut r = request("mixed");
        r["sql"] = json!(
            "SELECT r.token,COUNT(*) AS n FROM Resource r GROUP BY r.token ORDER BY r.token DESC"
        );
        let out = contributors(&r, |_| {}).unwrap();
        assert_eq!(out["privateGroupSources"][0]["path"], "groups/0");
        assert_eq!(
            out["privateGroupSources"][0]["field"],
            json!(identity("token"))
        );
        assert_eq!(out["publicCellSources"][0]["kind"], "group-field");
        assert_eq!(out["publicCellSources"][1]["kind"], "count");
        r["sql"] = json!("SELECT r.salary,COUNT(*) AS n FROM Resource r GROUP BY r.salary");
        assert!(contributors(&r, |_| {}).is_err());
        r["sql"] = json!("SELECT r.salary FROM Resource r WHERE r.salary=100");
        let out = contributors(&r, |_| {}).unwrap();
        assert_eq!(out["publicCellSources"][0]["kind"], "field");
        assert_eq!(out["privateGroupSources"], json!([]));
        assert!(!out["sql"].as_str().unwrap().contains("jsonb_agg("));
    }
    // @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn contributor_versions_missing_keys_and_prohibited_uses_refuse_without_downgrade() {
        let r = request("mixed");
        assert!(contributors(&r, |b| b["version"] =
            json!("weft.postgresql.current-values/0.2-candidate"))
        .is_err());
        assert!(rows(&r, |b| b["version"] =
            json!("weft.postgresql.current-values/0.3-candidate"))
        .is_err());
        assert!(contributors(&r, |b| {
            b["scans"][0]["relation"]
                .as_object_mut()
                .unwrap()
                .remove("rowKeyColumn");
        })
        .is_err());
        assert!(contributors(&r, |b| b["contributorBag"] = json!(true)).is_err());
        let mut r = r;
        r["sql"] = json!("SELECT COUNT(DISTINCT r.token) AS n FROM Resource r WHERE FALSE LIMIT 0");
        assert!(contributors(&r, |b| b["values"][0]["mode"] = json!("prohibited")).is_err());
    }
    fn sum_request() -> Value {
        serde_json::from_str(include_str!("../../../tests/current-query-designation/fixtures/live-original-integer-sum.request.json")).unwrap()
    }
    fn sum_output(r: &Value, edit: impl FnOnce(&mut Value)) -> Result<Value> {
        let p = application_lineage::prepare(&r.to_string())?;
        let d = current_query_designation::derive(&p)?;
        let g = current_query_operands::derive(&d)?;
        let mut b = binding(&g);
        b["version"] = json!("weft.postgresql.current-values/0.4-candidate");
        for scan in b["scans"].as_array_mut().unwrap() {
            scan["relation"]["rowKeyColumn"] = json!("__native_row_key")
        }
        edit(&mut b);
        lower_original_sum(&g, &b.to_string()).map(|sql| sql.json())
    }
    // @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn original_integer_sum_uses_exact_numeric_accumulation_nullable_text_and_complete_bag() {
        let r = sum_request();
        let out = sum_output(&r, |_| {}).unwrap();
        let sql = out["sql"].as_str().unwrap();
        assert!(sql.contains("pg_catalog.sum(\"s0\".\"salary\")"));
        assert!(sql.contains("pg_catalog.jsonb_agg("));
        assert!(!sql.contains("salary_public"));
        assert_eq!(
            out["version"],
            "weft.postgresql.current-scalar/0.4-candidate"
        );
        assert_eq!(
            out["resultColumns"][0]["logicalType"],
            json!({"family":"integer","facets":{},"nullable":true})
        );
        assert_eq!(
            out["resultColumns"][0]["encoding"],
            "native-numeric-integer-nullable-text"
        );
        assert_eq!(out["publicCellSources"][0]["kind"], "sum");
        assert_eq!(
            out["publicCellSources"][0]["sources"][0]["path"],
            "outputs/0/argument"
        );
        let mut grouped = r.clone();
        grouped["sql"] = json!("SELECT SUM(r.salary) AS total FROM Resource r GROUP BY r.token");
        let out = sum_output(&grouped, |_| {}).unwrap();
        assert_eq!(out["resultColumns"][0]["logicalType"]["nullable"], false);
        assert_eq!(
            out["resultColumns"][0]["encoding"],
            "native-numeric-integer-required-text"
        );
        assert!(sum_output(&r, |b| {
            b["values"][0]["mode"] = json!("disclosed");
            b["values"][0]["originalAction"] = Value::Null;
        })
        .is_err());
        assert!(sum_output(&r, |b| b["values"][0]["originalAction"] = json!("read")).is_err());
    }
    // @covers US-009-AC2 @covers US-009-AC3
    #[test]
    fn sum_context_cannot_route_through_predecessors_even_for_count_or_empty_work() {
        let mut r = sum_request();
        r["sql"] = json!("SELECT COUNT(*) AS n FROM Resource r WHERE FALSE LIMIT 0");
        assert!(run(&r, |_| {}).is_err());
        assert!(rows(&r, |_| {}).is_err());
        assert!(contributors(&r, |_| {}).is_err());
        assert!(sum_output(&r, |_| {}).is_ok());
        assert!(sum_output(&r, |b| b["version"] =
            json!("weft.postgresql.current-values/0.3-candidate"))
        .is_err());
        assert!(sum_output(&request("mixed"), |_| {}).is_err());
        r["sql"] = json!("SELECT SUM(r.salary) AS total FROM Resource r WHERE FALSE LIMIT 0");
        assert!(sum_output(&r, |b| b["values"][0]["mode"] = json!("prohibited")).is_err());
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
    #[cfg(feature="bounded-current-string-candidate")]
    mod bounded {
        use super::*;
        fn source()->Value {serde_json::from_str(include_str!("../../../tests/current-query-designation/fixtures/bounded-unicode-strings.request.json")).unwrap()}
        fn materialization(g:&CurrentQueryOperands<'_>)->Value {
            let mut b=binding(g);b["version"]=json!("weft.postgresql.current-values/0.5-candidate");b["capabilities"]=json!(["bounded-unicode-scalars"]);
            for s in b["scans"].as_array_mut().unwrap(){s["relation"]["rowKeyColumn"]=json!("row_key");} b
        }
        fn frame(g:&CurrentQueryOperands<'_>,b:&Value,artifact:&Value,value:&str,contributor:bool)->Value {
            let carrier=if contributor {json!({"version":"weft.stage-contributor-bag/0.1-candidate","tuples":[[{"scan":"s0","key":"k"}]]})} else {json!({"version":"weft.stage-row-tuple/0.1-candidate","rows":[{"scan":"s0","key":"k"}]})};
            json!({"version":"weft.current-query.refinement-input/0.2-candidate","capabilities":["bounded-unicode-scalars"],"sourceRequestSha256":sha256(g.designation().preparation().source_request_json().as_bytes()),"designationSha256":sha256(g.designation().json().as_bytes()),"materializationBindingSha256":sha256(b.to_string().as_bytes()),"compiledArtifactSha256":sha256(artifact.to_string().as_bytes()),"scans":[{"scan":"s0","expectedRowCount":"1","rows":[{"key":"k","cells":{"token":{"family":"string","value":value}}}]}],"resultRows":[[value,carrier.to_string()]]})
        }
        // @covers US-009-AC3 @covers US-009-AC4
        #[test]
        fn bounded_rows_and_contributors_preserve_domains_and_close_old_entrypoints() {
            for contributor in [false,true] {
                let r=source();let p=application_lineage::prepare(&r.to_string()).unwrap();let d=current_query_designation::derive(&p).unwrap();let g=current_query_operands::derive(&d).unwrap();let b=materialization(&g);
                let lowered=if contributor {lower_bounded_contributors(&g,&b.to_string())} else {lower_bounded_rows(&g,&b.to_string())}.unwrap();
                let artifact=lowered.json();assert_eq!(artifact["version"],if contributor {"weft.postgresql.current-scalar/0.6-candidate"} else {"weft.postgresql.current-scalar/0.5-candidate"});
                assert_eq!(artifact["parameters"][0]["value"],"😀");assert_eq!(artifact["parameters"][0]["logicalType"]["facets"]["length"]["max"],64);
                let input=frame(&g,&b,&artifact,"😀",contributor);
                let result=if contributor {check_bounded_contributors(&g,&b.to_string(),&input.to_string())} else {check_bounded_rows(&g,&b.to_string(),&input.to_string())}.unwrap();assert_eq!(result["matched"],true);assert_eq!(result["authority"],false);
                assert!(lower_rows(&g,&b.to_string()).is_err());assert!(lower_contributors(&g,&b.to_string()).is_err());assert!(lower_original_sum(&g,&b.to_string()).is_err());
                for caps in [json!(null),json!([]),json!(["bounded-unicode-scalars","live-original-integer-sum"]),json!(["other"])] {let mut bad=b.clone();bad["capabilities"]=caps;assert!(lower_bounded_rows(&g,&bad.to_string()).is_err());}
            }
        }
        // @covers US-009-AC3
        #[test]
        fn bounded_captured_domains_validate_before_false_and_limit_zero() {
            let mut r=source();r["sql"]=json!("SELECT r.token FROM Resource r WHERE FALSE AND r.token=:needle LIMIT 0");
            let p=application_lineage::prepare(&r.to_string()).unwrap();let d=current_query_designation::derive(&p).unwrap();let g=current_query_operands::derive(&d).unwrap();let b=materialization(&g);let artifact=lower_bounded_rows(&g,&b.to_string()).unwrap().json();
            for value in ["😀".to_owned(),"e\u{301}".to_owned(),"😀".repeat(64)] {let mut input=frame(&g,&b,&artifact,&value,false);input["resultRows"]=json!([]);assert!(check_bounded_rows(&g,&b.to_string(),&input.to_string()).is_ok());}
            for value in ["".to_owned(),"😀".repeat(65)] {let mut input=frame(&g,&b,&artifact,&value,false);input["resultRows"]=json!([]);assert!(check_bounded_rows(&g,&b.to_string(),&input.to_string()).is_err());}
        }
        fn carrier(keys:&[usize])->String {
            json!({"version":"weft.stage-contributor-bag/0.1-candidate","tuples":keys.iter().map(|i|json!([{"scan":"s0","key":format!("k{i}")}])).collect::<Vec<_>>()} ).to_string()
        }
        fn captured(g:&CurrentQueryOperands<'_>,b:&Value,a:&Value,values:&[&str],public:Value)->Value {
            let mut input=frame(g,b,a,values[0],true);input["scans"][0]["expectedRowCount"]=json!(values.len().to_string());
            input["scans"][0]["rows"]=json!(values.iter().enumerate().map(|(i,value)|json!({"key":format!("k{i}"),"cells":{"token":{"family":"string","value":value}}})).collect::<Vec<_>>());input["resultRows"]=public;input
        }
        // @covers US-009-AC3
        #[test]
        fn bounded_unicode_order_is_scalar_order_and_counts_keep_contributor_bags() {
            for (sql,values,public) in [
                ("SELECT r.token FROM Resource r ORDER BY r.token DESC",vec!["\u{e000}","😀"],json!([["😀",carrier(&[1])],["\u{e000}",carrier(&[0])]])),
                ("SELECT r.token, COUNT(*) AS n FROM Resource r GROUP BY r.token ORDER BY r.token",vec!["😀","😀","e\u{301}"],json!([["e\u{301}","1",carrier(&[2])],["😀","2",carrier(&[0,1])]])),
                ("SELECT COUNT(DISTINCT r.token) AS n FROM Resource r",vec!["😀","😀","e\u{301}"],json!([["2",carrier(&[0,1,2])]])),
            ] {
                let mut r=source();r["sql"]=json!(sql);r["parameters"]=json!({});
                let p=application_lineage::prepare(&r.to_string()).unwrap();let d=current_query_designation::derive(&p).unwrap();let g=current_query_operands::derive(&d).unwrap();let b=materialization(&g);let a=lower_bounded_contributors(&g,&b.to_string()).unwrap().json();let input=captured(&g,&b,&a,&values,public);
                assert!(check_bounded_contributors(&g,&b.to_string(),&input.to_string()).is_ok(),"{sql}");
                let mut bad=input.clone();if bad["resultRows"].as_array().unwrap().len()>1 {bad["resultRows"].as_array_mut().unwrap().reverse();}else{bad["resultRows"][0][0]=json!("3");}
                assert!(check_bounded_contributors(&g,&b.to_string(),&bad.to_string()).is_err());
                let mut bad=input.clone();bad["scans"][0]["rows"][0]["cells"]["token"]["value"]=json!("");assert!(check_bounded_contributors(&g,&b.to_string(),&bad.to_string()).is_err());
            }
        }
        // @covers US-009-AC3
        #[test]
        fn bounded_public_cells_and_refinement_versions_refuse_bad_inputs() {
            let r=source();let p=application_lineage::prepare(&r.to_string()).unwrap();let d=current_query_designation::derive(&p).unwrap();let g=current_query_operands::derive(&d).unwrap();let b=materialization(&g);let artifact=lower_bounded_rows(&g,&b.to_string()).unwrap().json();let good=frame(&g,&b,&artifact,"😀",false);
            for value in [json!(null),json!(""),json!("😀".repeat(65))] {let mut bad=good.clone();bad["resultRows"][0][0]=value;assert!(check_bounded_rows(&g,&b.to_string(),&bad.to_string()).is_err());}
            let mut bad=good.clone();bad["version"]=json!("weft.current-query.refinement-input/0.1-candidate");assert!(check_bounded_rows(&g,&b.to_string(),&bad.to_string()).is_err());
            for caps in [json!([]),json!(["other"]),json!(["bounded-unicode-scalars","bounded-unicode-scalars"])] {let mut bad=good.clone();bad["capabilities"]=caps;assert!(check_bounded_rows(&g,&b.to_string(),&bad.to_string()).is_err());}
        }
    }
}

/// Optional conditional result comparison; actual lowering precedes all input evaluation.
#[cfg(feature="current-query-refinement-candidate")]
pub fn check_rows<'a>(context:&'a CurrentQueryOperands<'a>,binding_json:&str,input_json:&str)->Result<Value>{
    (||{
        let actual=lower_rows(context,binding_json)?;
        let binding:RowBinding=serde_json::from_str(binding_json).map_err(|_|fail())?;
        refine_actual(context,actual,binding.values,binding_json,input_json,weft_core::current_query_refinement::Encoding::Rows)
    })().map_err(|_|refinement_failure())
}
/// Optional conditional result comparison; actual lowering precedes all input evaluation.
#[cfg(feature="current-query-refinement-candidate")]
pub fn check_contributors<'a>(context:&'a CurrentQueryOperands<'a>,binding_json:&str,input_json:&str)->Result<Value>{
    (||{
        let actual=lower_contributors(context,binding_json)?;
        let binding:RowBinding=serde_json::from_str(binding_json).map_err(|_|fail())?;
        refine_actual(context,actual,binding.values,binding_json,input_json,weft_core::current_query_refinement::Encoding::Contributors)
    })().map_err(|_|refinement_failure())
}
/// Optional conditional result comparison; actual lowering precedes all input evaluation.
#[cfg(feature="current-query-refinement-candidate")]
pub fn check_original_sum<'a>(context:&'a CurrentQueryOperands<'a>,binding_json:&str,input_json:&str)->Result<Value>{
    (||{
        let actual=lower_original_sum(context,binding_json)?;
        let binding:RowBinding=serde_json::from_str(binding_json).map_err(|_|fail())?;
        refine_actual(context,actual,binding.values,binding_json,input_json,weft_core::current_query_refinement::Encoding::Contributors)
    })().map_err(|_|refinement_failure())
}

#[cfg(feature="bounded-current-string-candidate")]
pub fn check_bounded_rows<'a>(context:&'a CurrentQueryOperands<'a>,binding_json:&str,input_json:&str)->Result<Value>{
    (||{let actual=lower_bounded_rows(context,binding_json)?;let binding:RowBinding=serde_json::from_str(binding_json).map_err(|_|fail())?;
    refine_actual(context,actual,binding.values,binding_json,input_json,weft_core::current_query_refinement::Encoding::Rows)})().map_err(|_|refinement_failure())
}
#[cfg(feature="bounded-current-string-candidate")]
pub fn check_bounded_contributors<'a>(context:&'a CurrentQueryOperands<'a>,binding_json:&str,input_json:&str)->Result<Value>{
    (||{let actual=lower_bounded_contributors(context,binding_json)?;let binding:RowBinding=serde_json::from_str(binding_json).map_err(|_|fail())?;
    refine_actual(context,actual,binding.values,binding_json,input_json,weft_core::current_query_refinement::Encoding::Contributors)})().map_err(|_|refinement_failure())
}
#[cfg(feature="current-query-refinement-candidate")]
fn refinement_failure()->Diagnostic{Diagnostic::new("WFT-CURRENT-QUERY-REFINEMENT","capability","Conditional current-query refinement refused")}
#[cfg(feature="current-query-refinement-candidate")]
fn refine_actual<'a>(context:&'a CurrentQueryOperands<'a>,actual:CurrentScalarSql<'a>,values:Vec<BindingValue>,binding_json:&str,input_json:&str,encoding:weft_core::current_query_refinement::Encoding)->Result<Value>{
    use weft_core::current_query_refinement::{Selection,Lowered,compare};
    if !std::ptr::eq(context,actual.context){return Err(refinement_failure())}
    let artifact=actual.json_string()?;let artifact_sha=sha256(artifact.as_bytes());let binding_sha=sha256(binding_json.as_bytes());
    let selections=values.into_iter().map(|v|{
        let descriptor=context.designation().preparation().catalog().scalar_descriptor_by_identity(&v.domain)?;
        let Shape::Scalar{logical_type}=descriptor.shape else{return Err(refinement_failure())};
        Ok(Selection{path:v.path,scan:v.scan,field:v.field,domain:v.domain,column:v.column,logical_type,comparison_scope:v.comparison_scope,
            mode:v.mode.map(|m|match m{Mode::Disclosed=>"disclosed",Mode::OriginalAuthorized=>"original-authorized",Mode::Prohibited=>"prohibited",Mode::Unresolved=>"unresolved"}.into()),
            original_action:v.original_action,exposure:v.exposure.map(|m|match m{Exposure::Original=>"original",Exposure::Transformed=>"transformed"}.into())})
    }).collect::<Result<Vec<_>>>()?;
    let types=actual.columns().iter().map(|c|c.logical_type.clone()).collect::<Vec<_>>();
    let lowered=Lowered{binding_sha256:&binding_sha,artifact_sha256:&artifact_sha,selections:&selections,output_types:&types,encoding};
    let bounded=context.designation().supports_bounded_unicode_strings();
    if bounded {
        #[cfg(feature="bounded-current-string-candidate")]
        weft_core::current_query_refinement::compare_bounded(context,lowered,input_json)?;
        #[cfg(not(feature="bounded-current-string-candidate"))]
        return Err(refinement_failure());
    } else {compare(context,lowered,input_json)?;}
    let mut report=json!({"version":if bounded {"weft.current-query.refinement-result/0.2-candidate"} else {"weft.current-query.refinement-result/0.1-candidate"},"matched":true,"authority":false,"securityReady":false,"nativeAdmission":"required",
        "sourceRequestSha256":sha256(context.designation().preparation().source_request_json().as_bytes()),"designationSha256":sha256(context.designation().json().as_bytes()),
        "materializationBindingSha256":binding_sha,"compiledArtifactSha256":artifact_sha,"refinementInputSha256":sha256(input_json.as_bytes())});
    if bounded {report["capabilities"]=json!(["bounded-unicode-scalars"]);}
    Ok(report)
}
