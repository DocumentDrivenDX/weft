// SPDX-License-Identifier: MIT OR Apache-2.0
//! The explicitly selected finite temporal frontend. Ordinary resolvers retain
//! their historical semantics. UMF owns access admission and token meaning.
use crate::{
    error::Diagnostic,
    ir::{Family, Identity, LogicalType},
    model::{Catalog, ModuleInput},
    syntax::{Column, LiteralKind, Name, Parser},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
mod inspection;
/// Ordered upstream diagnostics remain intact at the engine boundary.
#[derive(Clone, Debug)]
pub struct Failure {
    pub diagnostics: Vec<Value>,
}
impl From<Diagnostic> for Failure {
    fn from(d: Diagnostic) -> Self {
        Self {
            diagnostics: vec![serde_json::to_value(d).unwrap()],
        }
    }
}
pub type Result<T> = std::result::Result<T, Failure>;
pub const PROFILE: &str = "umf.weft-binding-temporal/0.2";
pub fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-TEMPORAL", "resolve", message)
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeyRef {
    pub module: String,
    pub record: String,
    pub key: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub interface_version: String,
    pub dialect: String,
    pub modules: Vec<ModuleInput>,
    pub sql: String,
    pub parameters: BTreeMap<String, crate::application_resolve::Parameter>,
    pub exact_parameters: BTreeMap<String, Value>,
    pub temporal_profile: String,
    pub temporal_parameters: BTreeMap<String, TemporalParameter>,
    pub options: Options,
    pub target: Target,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Options {
    pub allow_candidate: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Target {
    pub backend_id: String,
    pub backend_version: String,
    pub target_profile: String,
    pub binding_json: String,
    pub binding_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalParameter {
    pub kind: String,
    pub value: String,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub identity: Identity,
    pub kind: String,
    pub missing: bool,
    pub nullable: bool,
    pub descriptor: Value,
    pub logical_type: Option<LogicalType>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Operand {
    pub kind: String,
    pub value: Option<String>,
    pub field: Option<usize>,
    pub origin: Value,
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Predicate {
    Compare {
        left: usize,
        operator: String,
        right: Operand,
    },
    And {
        args: Vec<Predicate>,
    },
    Or {
        args: Vec<Predicate>,
    },
    Not {
        arg: Box<Predicate>,
    },
    Boolean {
        value: bool,
    },
}
#[derive(Clone, Debug, Serialize)]
pub struct Sort {
    pub field: usize,
    pub direction: String,
    pub nulls: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct Output {
    pub field: usize,
    pub name: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub record: Identity,
    pub fields: Vec<Field>,
    pub outputs: Vec<Output>,
    pub sort: Vec<Sort>,
    pub predicate: Option<Predicate>,
    pub limit: Option<u16>,
    pub selected_key: KeyRef,
    pub key_fields: Vec<usize>,
    pub model_pins: Vec<crate::ir::ModelPin>,
}
pub fn canonical(kind: &str, token: &str) -> Result<String> {
    if token.len() > 32 {
        return Err((fail("CONTRACT-075 token exceeds32 ASCII characters")).into());
    }
    umf_core::access_temporal::parse_access_temporal(
        &umf_core::Value::string_utf8(kind),
        &umf_core::Value::string_utf8(token),
    )
    .map(|v| v.canonical_text)
    .map_err(|_| fail("CONTRACT-075 strict temporal token refused").into())
}
fn parameter_name(name: &str) -> bool {
    name.as_bytes()
        .first()
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
fn ordinary(field: &Field, text: &str, kind: LiteralKind) -> Result<()> {
    let logical = field
        .logical_type
        .as_ref()
        .ok_or_else(|| fail("Ordinary operand requires ordinary Field"))?;
    if logical.family == Family::Boolean && !matches!(text, "true" | "false") {
        return Err((fail("Exact boolean token required")).into());
    }
    crate::exact::literal(
        &crate::syntax::Literal {
            value: text.into(),
            kind,
            span: crate::ir::Span { start: 0, end: 0 },
        },
        logical,
    )?;
    Ok(())
}
fn field_index(
    column: &Column,
    alias: &Name,
    members: &[crate::application_model::Member],
) -> Result<usize> {
    if !column.unqualified && !column.alias.matches(&alias.value) {
        return Err((fail("Original source occurrence required")).into());
    }
    let found = members
        .iter()
        .enumerate()
        .filter(|(_, m)| column.field.matches(&m.name))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if found.len() != 1 {
        return Err((Diagnostic::new(
            if found.is_empty() {
                "WFT-COLUMN-MISSING"
            } else {
                "WFT-COLUMN-AMBIGUOUS"
            },
            "resolve",
            "Unique original Field required",
        )
        .at(&column.span))
        .into());
    }
    Ok(found[0])
}
struct Resolve<'a> {
    alias: &'a Name,
    members: &'a [crate::application_model::Member],
    fields: &'a [Field],
    request: &'a Request,
    used: BTreeSet<String>,
}
impl Resolve<'_> {
    fn operand(&mut self, p: &mut Parser, left: usize) -> Result<Operand> {
        let field = &self.fields[left];
        if p.peek_symbol(':') {
            p.symbol(':')?;
            let name = p.name()?;
            let key = name.value.to_ascii_lowercase();
            self.used.insert(key.clone());
            let ordinary = self
                .request
                .parameters
                .iter()
                .find(|(n, _)| n.to_ascii_lowercase() == key);
            let temporal = self
                .request
                .temporal_parameters
                .iter()
                .find(|(n, _)| n.to_ascii_lowercase() == key);
            let (value, origin) = if is_temporal(&field.kind) {
                let (original, v) = temporal
                    .ok_or_else(|| fail("Explicit same-kind temporal parameter required"))?;
                if v.kind != field.kind {
                    return Err((fail("Temporal operand kind differs")).into());
                }
                canonical(&v.kind, &v.value)?;
                (
                    v.value.clone(),
                    json!({"kind":"parameter","name":original,"temporalKind":v.kind}),
                )
            } else {
                let (original, v) = ordinary
                    .ok_or_else(|| fail("Explicit same-family ordinary parameter required"))?;
                if field
                    .logical_type
                    .as_ref()
                    .is_none_or(|t| t.family != v.family)
                {
                    return Err((fail("Parameter family differs")).into());
                }
                ordinary_validate(field, &v.value)?;
                (v.value.clone(), json!({"kind":"parameter","name":original}))
            };
            return Ok(Operand {
                kind: field.kind.clone(),
                value: Some(value),
                field: None,
                origin,
            });
        }
        if p.peek_identifier() && !p.peek_word("true") && !p.peek_word("false") {
            let c = p.column03()?;
            let i = field_index(&c, self.alias, self.members)?;
            if self.fields[i].kind != field.kind
                || self.fields[i].logical_type != field.logical_type
            {
                return Err((fail("Comparison operands differ in kind or domain")).into());
            }
            return Ok(Operand {
                kind: field.kind.clone(),
                value: None,
                field: Some(i),
                origin: json!({"kind":"field","identity":self.fields[i].identity}),
            });
        }
        let lit = p.literal()?;
        if is_temporal(&field.kind) {
            if !matches!(lit.kind, LiteralKind::String) {
                return Err((fail("Temporal literals must be explicit text tokens")).into());
            }
            canonical(&field.kind, &lit.value)?;
        } else {
            ordinary(field, &lit.value, lit.kind)?;
        }
        Ok(Operand {
            kind: field.kind.clone(),
            value: Some(lit.value),
            field: None,
            origin: json!({"kind":"literal","span":lit.span}),
        })
    }
    fn expression(&mut self, p: &mut Parser, depth: usize) -> Result<Predicate> {
        if depth > 64 {
            return Err((Diagnostic::new(
                "WFT-LIMIT",
                "parse",
                "Temporal predicate depth exceeded",
            ))
            .into());
        }
        let mut args = vec![self.conjunction(p, depth + 1)?];
        while p.peek_word("or") {
            p.word("or")?;
            args.push(self.conjunction(p, depth + 1)?)
        }
        Ok(if args.len() == 1 {
            args.remove(0)
        } else {
            Predicate::Or { args }
        })
    }
    fn conjunction(&mut self, p: &mut Parser, depth: usize) -> Result<Predicate> {
        let mut args = vec![self.atom(p, depth + 1)?];
        while p.peek_word("and") {
            p.word("and")?;
            args.push(self.atom(p, depth + 1)?)
        }
        Ok(if args.len() == 1 {
            args.remove(0)
        } else {
            Predicate::And { args }
        })
    }
    fn atom(&mut self, p: &mut Parser, depth: usize) -> Result<Predicate> {
        if depth > 64 {
            return Err((Diagnostic::new(
                "WFT-LIMIT",
                "parse",
                "Temporal predicate depth exceeded",
            ))
            .into());
        }
        if p.peek_word("not") {
            p.word("not")?;
            return Ok(Predicate::Not {
                arg: Box::new(self.atom(p, depth + 1)?),
            });
        }
        if p.peek_symbol('(') {
            p.symbol('(')?;
            let v = self.expression(p, depth + 1)?;
            p.symbol(')')?;
            return Ok(v);
        }
        if p.peek_word("true") || p.peek_word("false") {
            let value = p.peek_word("true");
            p.word(if value { "true" } else { "false" })?;
            return Ok(Predicate::Boolean { value });
        }
        let column = p.column03()?;
        let left = field_index(&column, self.alias, self.members)?;
        let operator = if p.peek_symbol('=') {
            p.symbol('=')?;
            "="
        } else if p.peek_symbol('<') {
            p.symbol('<')?;
            if p.peek_symbol('=') {
                p.symbol('=')?;
                "<="
            } else if p.peek_symbol('>') {
                p.symbol('>')?;
                "<>"
            } else {
                "<"
            }
        } else {
            p.symbol('>')?;
            if p.peek_symbol('=') {
                p.symbol('=')?;
                ">="
            } else {
                ">"
            }
        };
        let right = self.operand(p, left)?;
        Ok(Predicate::Compare {
            left,
            operator: operator.into(),
            right,
        })
    }
}
fn ordinary_validate(field: &Field, text: &str) -> Result<()> {
    ordinary(
        field,
        text,
        match field.kind.as_str() {
            "string" => LiteralKind::String,
            "boolean" => LiteralKind::Boolean,
            _ => LiteralKind::Number,
        },
    )
}
pub fn is_temporal(kind: &str) -> bool {
    matches!(kind, "date" | "instant" | "local")
}
#[jsonschema::validator(
    path = "../../docs/helix/02-design/contracts/compile-request-v0.2.10.schema.json"
)]
struct TemporalRequest;
pub fn prepare(value: &Value, page: Option<(&KeyRef, u16)>) -> Result<(Request, Plan, Catalog)> {
    if !TemporalRequest::is_valid(value) {
        return Err((Diagnostic::new(
            "WFT-INPUT",
            "input",
            "Request violates closed compile0.2.10 envelope",
        ))
        .into());
    }
    let request: Request = serde_json::from_value(value.clone()).map_err(|_| {
        Diagnostic::new(
            "WFT-INPUT",
            "input",
            "Closed compile0.2.10 temporal request required",
        )
    })?;
    if request.interface_version != "weft-compile/0.2.10"
        || request.dialect != "weft-sql/0.2.0"
        || request.temporal_profile != PROFILE
        || !request.exact_parameters.is_empty()
        || !request.options.allow_candidate
    {
        return Err((fail("Explicit candidate temporal request tuple required")).into());
    }
    if request.sql.len() > 65536
        || request.target.binding_json.len() > 4_194_304
        || request.parameters.len() + request.temporal_parameters.len() > 1024
    {
        return Err(
            (Diagnostic::new("WFT-LIMIT", "input", "Temporal request bounds exceeded")).into(),
        );
    }
    let mut names = BTreeSet::new();
    for name in request
        .parameters
        .keys()
        .chain(request.temporal_parameters.keys())
    {
        if !parameter_name(name) || !names.insert(name.to_ascii_lowercase()) {
            return Err((fail("Closed disjoint normalized parameter inventory required")).into());
        }
    }
    for t in request.temporal_parameters.values() {
        canonical(&t.kind, &t.value)?;
    }
    let mut p = Parser::new(&request.sql)?;
    p.word("select")?;
    let mut projected = vec![];
    loop {
        let c = p.column03()?;
        let alias = if p.peek_word("as") {
            p.word("as")?;
            Some(p.name()?)
        } else {
            None
        };
        projected.push((c, alias));
        if !p.peek_symbol(',') {
            break;
        }
        p.symbol(',')?;
    }
    p.word("from")?;
    let source = p.source()?;
    let mut catalog = Catalog::prepare_truss_runtime(request.modules.clone(), true)?;
    catalog.identity_name_fallback = true;
    catalog.exact_scalars = true;
    catalog.exact_dates = true;
    let record = catalog.record(source.namespace.as_ref(), &source.name)?;
    let members = catalog.record_members(&record)?;
    let inspected = inspection::inspect(
        &request.modules[record.document].document_json,
        &record.identity,
    )?;
    let resolved = inspected["records"]
        .as_array()
        .filter(|v| v.len() == 1)
        .ok_or_else(|| fail("One original inspected Record required"))?;
    let rows = resolved[0]["fields"]
        .as_array()
        .ok_or_else(|| fail("Inspected Field inventory missing"))?;
    if rows.len() != members.len() {
        return Err((fail("Complete inspected Record member inventory required")).into());
    }
    let mut fields = vec![];
    for member in &members {
        let rows = rows
            .iter()
            .filter(|r| {
                r["ref"]["module"] == member.identity.module
                    && r["ref"]["element"] == member.identity.element
            })
            .collect::<Vec<_>>();
        if rows.len() != 1 {
            return Err((fail("Exact source FieldView identity required")).into());
        }
        let row = rows[0];
        if row["cardinality"] != "one" {
            return Err((fail("Temporal route refuses containers")).into());
        }
        let kind = match row["scalarType"].as_str() {
            Some("timestamp") => row["temporal"]
                .as_str()
                .ok_or_else(|| fail("Authored timestamp meaning required"))?,
            Some(k) => k,
            None => return Err((fail("Scalar member required")).into()),
        };
        let (_, graph) = catalog.member_descriptor_by_identity(&record, &member.identity)?;
        let descriptor = graph
            .iter()
            .find(|d| d.identity == member.identity)
            .ok_or_else(|| fail("Original descriptor required"))?;
        let logical_type = if is_temporal(kind) {
            if row["facets"] != json!({})
                || row["properties"].as_object().is_some_and(|m| {
                    m.keys()
                        .any(|k| !["title", "aliases", "examples"].contains(&k.as_str()))
                })
            {
                return Err((fail("Refined temporal Fields refuse")).into());
            }
            None
        } else {
            match &descriptor.shape {
                crate::application_model::Shape::Scalar { logical_type } => {
                    Some(logical_type.clone())
                }
                _ => return Err((fail("Only ordinary scalar siblings admitted")).into()),
            }
        };
        fields.push(Field {
            identity: member.identity.clone(),
            kind: kind.into(),
            missing: row["row"]["missing"]
                .as_bool()
                .ok_or_else(|| fail("Exact row missing presence required"))?,
            nullable: row["row"]["null"]
                .as_bool()
                .ok_or_else(|| fail("Exact row null presence required"))?,
            descriptor: serde_json::to_value(descriptor).unwrap(),
            logical_type,
        });
    }
    let inspected_keys = resolved[0]["keys"]
        .as_array()
        .ok_or_else(|| fail("Inspected selected Key required"))?;
    let selected = inspected_keys
        .first()
        .ok_or_else(|| fail("Selected authored Key required"))?;
    let selected_key = KeyRef {
        module: record.identity.module.clone(),
        record: record.identity.element.clone(),
        key: selected["id"]
            .as_str()
            .ok_or_else(|| fail("Selected Key id required"))?
            .into(),
    };
    // UMF orders the explicitly selected Key first; every declared temporal Key
    // already produces an inspection diagnostic. Never infer a primary Key.
    let key = catalog.authored_key(&record, &selected_key.key)?;
    let key_fields = key
        .fields
        .iter()
        .map(|id| {
            fields
                .iter()
                .position(|f| f.identity == *id)
                .ok_or_else(|| fail("Original Key member required").into())
        })
        .collect::<Result<Vec<_>>>()?;
    if let Some((selected, size)) = page {
        if *selected != selected_key || !(1..=999).contains(&size) {
            return Err((fail("Exact authored selected Key and page bound required")).into());
        }
        if key_fields
            .iter()
            .any(|i| fields[*i].kind != "string" || fields[*i].missing || fields[*i].nullable)
        {
            return Err((fail("Complete required string selected Key required")).into());
        }
    }
    let mut outputs = vec![];
    let mut output_names = BTreeSet::new();
    for (column, name) in projected {
        let field = field_index(&column, &source.alias, &members)?;
        let name = name
            .map(|v| v.value)
            .unwrap_or_else(|| members[field].name.clone());
        if !output_names.insert(name.clone()) {
            return Err((fail("Unique output names required")).into());
        }
        outputs.push(Output { field, name });
    }
    let mut resolve = Resolve {
        alias: &source.alias,
        members: &members,
        fields: &fields,
        request: &request,
        used: BTreeSet::new(),
    };
    let predicate = if p.peek_word("where") {
        p.word("where")?;
        Some(resolve.expression(&mut p, 0)?)
    } else {
        None
    };
    if names != resolve.used {
        return Err((fail("Parameters must match the complete SQL placeholder inventory")).into());
    }
    let mut sort = vec![];
    if p.peek_word("order") {
        p.word("order")?;
        p.word("by")?;
        loop {
            let column = p.column03()?;
            let field = field_index(&column, &source.alias, &members)?;
            if !matches!(
                fields[field].kind.as_str(),
                "string" | "date" | "instant" | "local"
            ) {
                return Err(
                    (fail("Temporal ordering admits temporal and string Fields only")).into(),
                );
            }
            let direction = if p.peek_word("desc") {
                p.word("desc")?;
                "desc"
            } else {
                if p.peek_word("asc") {
                    p.word("asc")?;
                }
                "asc"
            };
            let nulls = if p.peek_word("nulls") {
                p.word("nulls")?;
                if p.peek_word("first") {
                    p.word("first")?;
                    "first"
                } else {
                    p.word("last")?;
                    "last"
                }
            } else {
                "last"
            };
            sort.push(Sort {
                field,
                direction: direction.into(),
                nulls: nulls.into(),
            });
            if !p.peek_symbol(',') {
                break;
            }
            p.symbol(',')?;
        }
    }
    let limit = if p.peek_word("limit") {
        p.word("limit")?;
        let n = p.literal()?;
        if !matches!(n.kind, LiteralKind::Number) {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "parse",
                "Application bound must be an integer literal",
            )
            .at(&n.span)
            .into());
        }
        Some(
            n.value
                .parse::<u16>()
                .ok()
                .filter(|n| (1..=1000).contains(n))
                .ok_or_else(|| fail("Finite integer LIMIT1..1000 required"))?,
        )
    } else {
        None
    };
    p.finish_application()?;
    if page.is_some() {
        if limit.is_some() {
            return Err((fail("Page compiler owns the page limit")).into());
        }
        let suffix = sort.len() >= key_fields.len()
            && sort[sort.len() - key_fields.len()..]
                .iter()
                .zip(&key_fields)
                .all(|(s, f)| s.field == *f && s.direction == "asc" && s.nulls == "last");
        if !suffix {
            for f in &key_fields {
                sort.push(Sort {
                    field: *f,
                    direction: "asc".into(),
                    nulls: "last".into(),
                })
            }
        }
    }
    let plan = Plan {
        record: record.identity,
        fields,
        outputs,
        sort,
        predicate,
        limit,
        selected_key,
        key_fields,
        model_pins: request.modules.iter().map(|m| m.pin.clone()).collect(),
    };
    Ok((request, plan, catalog))
}
