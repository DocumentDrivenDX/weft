//! Bounded relational comparison over an actual CQP and declared captured cells.
//! This low-level backend building block returns no permit or protocol receipt.
//! The selected backend must invoke its actual lowerer before using these inputs;
//! native authentication, complete factual custody and guarded release are external.
use crate::{
    application_ir as app,
    application_lineage::OrderDirection,
    application_model::Shape,
    current_query_operands::{CurrentQueryOperands, Input, Kind},
    error::{Diagnostic, Result},
    ir::{Family, Identity, LogicalType},
    json::{checked_json_bounded, sha256},
};
use num_bigint::BigInt;
use serde::Deserialize;
use serde_json::json;
use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};
const BYTES: usize = 16_777_216;
const ROWS: usize = 10_000;
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-CURRENT-QUERY-REFINEMENT",
        "capability",
        "Conditional current-query refinement refused",
    )
}
/// Actual validated backend selection, retained per occurrence, never per Field.
/// These public Rust values are adapter data, not a source/completeness credential.
#[derive(Clone)]
pub struct Selection {
    pub path: String,
    pub scan: String,
    pub field: Identity,
    pub domain: Identity,
    pub column: String,
    pub logical_type: LogicalType,
    pub comparison_scope: String,
    pub mode: Option<String>,
    pub original_action: Option<String>,
    pub exposure: Option<String>,
}
#[derive(Clone, Copy)]
pub enum Encoding {
    Rows,
    Contributors,
}
/// Metadata from the actual selected lowerer's privately constructed output.
/// This is a low-level conformance input; only backend check_* entrypoints own
/// CONTRACT-010 report emission after actual lowering and exact artifact capture.
pub struct Lowered<'a> {
    pub binding_sha256: &'a str,
    pub artifact_sha256: &'a str,
    pub selections: &'a [Selection],
    pub output_types: &'a [LogicalType],
    pub encoding: Encoding,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Frame {
    version: String,
    source_request_sha256: String,
    designation_sha256: String,
    materialization_binding_sha256: String,
    compiled_artifact_sha256: String,
    scans: Vec<Scan>,
    result_rows: Vec<Vec<Option<String>>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Scan {
    scan: String,
    expected_row_count: String,
    rows: Vec<Row>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    key: String,
    cells: BTreeMap<String, Cell>,
}
#[derive(Deserialize)]
#[serde(tag = "family", rename_all = "lowercase", deny_unknown_fields)]
enum Cell {
    String { value: String },
    Integer { value: String },
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Scalar {
    String(String),
    Integer(i64),
}
impl Scalar {
    fn text(&self) -> Cow<'_, str> {
        match self {
            Self::String(v) => Cow::Borrowed(v),
            Self::Integer(v) => Cow::Owned(v.to_string()),
        }
    }
}
struct Captured {
    key: String,
    cells: BTreeMap<String, Scalar>,
}
type Tuple = Vec<usize>;
type Bag = BTreeMap<Vec<String>, usize>;
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Signature {
    public: Vec<Option<String>>,
    bag: Vec<(Vec<String>, usize)>,
}
struct Expected<'a> {
    signature: Signature,
    order: Vec<&'a Scalar>,
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn supported(t: &LogicalType) -> bool {
    !t.nullable
        && match t.family {
            Family::String => t.facets == json!({}),
            Family::Integer => t.facets == json!({"integerWidth":{"bits":64,"signed":true}}),
            _ => false,
        }
}
fn scalar(value: &str, t: &LogicalType, canonical: bool) -> Result<Scalar> {
    if !supported(t) {
        return Err(fail());
    }
    match t.family {
        Family::String => Ok(Scalar::String(value.into())),
        Family::Integer => {
            let n = value.parse::<i64>().map_err(|_| fail())?;
            if canonical && n.to_string() != value {
                return Err(fail());
            }
            Ok(Scalar::Integer(n))
        }
        _ => Err(fail()),
    }
}
struct Eval<'a> {
    context: &'a CurrentQueryOperands<'a>,
    selected: BTreeMap<(String, String), &'a Selection>,
    scans: Vec<String>,
    positions: BTreeMap<String, usize>,
    rows: Vec<Vec<Captured>>,
    keysets: Vec<BTreeSet<String>>,
}
impl Eval<'_> {
    fn selection(&self, path: &str, scan: &str, id: &Identity) -> Result<&Selection> {
        let s = self
            .selected
            .get(&(path.into(), scan.into()))
            .ok_or_else(fail)?;
        if &s.field != id {
            return Err(fail());
        }
        Ok(s)
    }
    fn field(&self, path: &str, field: &app::Field, t: &Tuple) -> Result<&Scalar> {
        self.field_id(path, &field.scan, &field.identity, t)
    }
    fn field_id(&self, path: &str, scan: &str, id: &Identity, t: &Tuple) -> Result<&Scalar> {
        let s = self.selection(path, scan, id)?;
        let index = *self.positions.get(scan).ok_or_else(fail)?;
        self.rows
            .get(index)
            .and_then(|r| r.get(*t.get(index)?))
            .and_then(|r| r.cells.get(&s.column))
            .ok_or_else(fail)
    }
    fn predicate(&self, path: &str, p: &app::Predicate, t: &Tuple) -> Result<bool> {
        match p {
            app::Predicate::Boolean { value } => Ok(*value),
            app::Predicate::Equal { left, right } => {
                let left = self.field(&format!("{path}/left"), left, t)?;
                match right {
                    app::Value::Field { field } => {
                        Ok(left == self.field(&format!("{path}/right"), field, t)?)
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
                    } => match (left, &logical_type.family) {
                        (Scalar::String(v), Family::String) => Ok(v == value),
                        (Scalar::Integer(v), Family::Integer) => {
                            Ok(*v == value.parse::<i64>().map_err(|_| fail())?)
                        }
                        _ => Err(fail()),
                    },
                }
            }
            _ => Err(fail()),
        }
    }
    fn bag(&self, tuples: &[Tuple]) -> Result<BTreeMap<Vec<&str>, usize>> {
        let mut bag = BTreeMap::new();
        for t in tuples {
            if t.len() != self.scans.len() {
                return Err(fail());
            }
            let keys = t
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    self.rows[i]
                        .get(*n)
                        .map(|r| r.key.as_str())
                        .ok_or_else(fail)
                })
                .collect::<Result<Vec<_>>>()?;
            *bag.entry(keys).or_default() += 1;
        }
        Ok(bag)
    }
    fn derive(&self) -> Result<Vec<Expected<'_>>> {
        let plan = self.context.designation().preparation().plan();
        let mut tuples: Vec<Tuple> = (0..self.rows[0].len()).map(|n| vec![n]).collect();
        for (i, join) in plan.joins.iter().enumerate() {
            if join.on.is_empty()
                || tuples
                    .len()
                    .checked_mul(self.rows[i + 1].len())
                    .ok_or_else(fail)?
                    > 1_000_000
            {
                return Err(fail());
            }
            let mut next = Vec::new();
            for left in &tuples {
                for n in 0..self.rows[i + 1].len() {
                    let mut t = left.clone();
                    t.push(n);
                    let mut yes = true;
                    for (k, p) in join.on.iter().enumerate() {
                        yes &= self.predicate(&format!("joins/{i}/on/{k}"), p, &t)?;
                    }
                    if yes {
                        if next.len() >= ROWS {
                            return Err(fail());
                        }
                        next.push(t)
                    }
                }
            }
            tuples = next;
        }
        let mut filtered = Vec::new();
        for t in tuples {
            let mut yes = true;
            for (i, p) in plan.filters.iter().enumerate() {
                yes &= self.predicate(&format!("filters/{i}"), p, &t)?;
            }
            if yes {
                filtered.push(t)
            }
        }
        let groups: Vec<Vec<Tuple>> = if plan.aggregate {
            if plan.groups.is_empty() {
                vec![filtered]
            } else {
                let mut groups: BTreeMap<Vec<&Scalar>, Vec<Tuple>> = BTreeMap::new();
                for t in filtered {
                    let key = plan
                        .groups
                        .iter()
                        .enumerate()
                        .map(|(i, f)| self.field(&format!("groups/{i}"), f, &t))
                        .collect::<Result<Vec<_>>>()?;
                    groups.entry(key).or_default().push(t);
                }
                groups.into_values().collect()
            }
        } else {
            if !plan.groups.is_empty() {
                return Err(fail());
            }
            filtered.into_iter().map(|t| vec![t]).collect()
        };
        if groups.len() > ROWS
            || groups
                .len()
                .checked_mul(
                    plan.outputs
                        .len()
                        .checked_add(plan.order.len())
                        .ok_or_else(fail)?,
                )
                .ok_or_else(fail)?
                > 1_000_000
        {
            return Err(fail());
        }
        // Standard semantic result-payload bound, not allocator/work tracing.
        // Count text/key occurrences before copying any large borrowed values.
        let mut payload_left = BYTES;
        let mut expected = Vec::new();
        for group in groups {
            let mut public = Vec::new();
            for (i, o) in plan.outputs.iter().enumerate() {
                let path = format!("outputs/{i}");
                let value = match &o.expression {
                    app::Expression::Field { scan, identity } => Some(
                        self.field_id(
                            &format!("{path}/expression"),
                            scan,
                            identity,
                            group.first().ok_or_else(fail)?,
                        )?
                        .text(),
                    ),
                    app::Expression::Count { .. } => Some(Cow::Owned(group.len().to_string())),
                    app::Expression::CountDistinct { argument, .. } => {
                        let set = group
                            .iter()
                            .map(|t| self.field(&format!("{path}/argument"), argument, t))
                            .collect::<Result<BTreeSet<_>>>()?;
                        Some(Cow::Owned(set.len().to_string()))
                    }
                    app::Expression::Sum { argument, .. } => {
                        if group.is_empty() {
                            None
                        } else {
                            let mut sum = BigInt::from(0);
                            for t in &group {
                                match self.field(&format!("{path}/argument"), argument, t)? {
                                    Scalar::Integer(n) => sum += BigInt::from(*n),
                                    _ => return Err(fail()),
                                }
                            }
                            Some(Cow::Owned(sum.to_string()))
                        }
                    }
                    _ => return Err(fail()),
                };
                public.push(value);
            }
            let order = plan
                .order
                .iter()
                .enumerate()
                .map(|(i, f)| self.field(&format!("order/{i}"), f, group.first().ok_or_else(fail)?))
                .collect::<Result<Vec<_>>>()?;
            let bag = self.bag(&group)?;
            let mut bytes = public.iter().try_fold(0usize, |n, v| {
                n.checked_add(v.as_ref().map_or(0, |s| s.len()))
                    .ok_or_else(fail)
            })?;
            for (keys, count) in &bag {
                let key_bytes = keys
                    .iter()
                    .try_fold(0usize, |n, k| n.checked_add(k.len()).ok_or_else(fail))?;
                bytes = bytes
                    .checked_add(key_bytes.checked_mul(*count).ok_or_else(fail)?)
                    .ok_or_else(fail)?;
            }
            payload_left = payload_left.checked_sub(bytes).ok_or_else(fail)?;
            expected.push(Expected {
                signature: Signature {
                    public: public.into_iter().map(|v| v.map(Cow::into_owned)).collect(),
                    bag: bag
                        .into_iter()
                        .map(|(keys, n)| (keys.into_iter().map(str::to_owned).collect(), n))
                        .collect(),
                },
                order,
            });
        }
        Ok(expected)
    }
}
/// Compare only conditional relational meaning. Never establishes native input
/// provenance/completeness or emits an importable CONTRACT-010 success report.
pub fn compare<'a>(
    context: &'a CurrentQueryOperands<'a>,
    lowered: Lowered<'a>,
    raw: &str,
) -> Result<()> {
    compare_inner(context, lowered, raw).map_err(|_| fail())
}
fn compare_inner<'a>(
    context: &'a CurrentQueryOperands<'a>,
    data: Lowered<'a>,
    raw: &str,
) -> Result<()> {
    let original = context.designation().preparation();
    let plan = original.plan();
    if matches!(data.encoding, Encoding::Rows) && plan.aggregate {
        return Err(fail());
    }
    if plan.outputs.is_empty()
        || plan.outputs.len() > 1024
        || data.output_types.len() != plan.outputs.len()
        || data.selections.len() > 4096
    {
        return Err(fail());
    }
    let frame: Frame = serde_json::from_value(
        checked_json_bounded(raw, BYTES, 32, 1_000_000).map_err(|_| fail())?,
    )
    .map_err(|_| fail())?;
    if frame.version != "weft.current-query.refinement-input/0.1-candidate"
        || frame.source_request_sha256 != sha256(original.source_request_json().as_bytes())
        || frame.designation_sha256 != sha256(context.designation().json().as_bytes())
        || frame.materialization_binding_sha256 != data.binding_sha256
        || frame.compiled_artifact_sha256 != data.artifact_sha256
        || ![
            &frame.source_request_sha256,
            &frame.designation_sha256,
            &frame.materialization_binding_sha256,
            &frame.compiled_artifact_sha256,
        ]
        .into_iter()
        .all(|s| digest(s))
    {
        return Err(fail());
    }
    let scans: Vec<_> = std::iter::once(&plan.source)
        .chain(plan.joins.iter().map(|j| &j.right))
        .map(|s| s.occurrence.clone())
        .collect();
    if scans.is_empty()
        || scans.len() > 16
        || frame.scans.len() != scans.len()
        || frame.result_rows.len() > ROWS
    {
        return Err(fail());
    }
    let positions: BTreeMap<_, _> = scans
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i))
        .collect();
    if positions.len() != scans.len() {
        return Err(fail());
    }
    let mut expected = BTreeMap::new();
    for op in context.operations() {
        if !matches!(
            op.kind(),
            Kind::Boolean(_)
                | Kind::Equal
                | Kind::Group
                | Kind::Order(_)
                | Kind::Count
                | Kind::CountDistinct
                | Kind::Sum
        ) {
            return Err(fail());
        }
        for operand in op.operands() {
            if let Input::Field { scan, identity, .. } = operand.input() {
                if expected
                    .insert(
                        (operand.path().to_owned(), (*scan).to_owned()),
                        (*identity, false),
                    )
                    .is_some()
                {
                    return Err(fail());
                }
            }
        }
    }
    for (i, o) in plan.outputs.iter().enumerate() {
        if let app::Expression::Field { scan, identity } = &o.expression {
            if expected
                .insert(
                    (format!("outputs/{i}/expression"), scan.clone()),
                    (identity, true),
                )
                .is_some()
            {
                return Err(fail());
            }
        }
    }
    let mut selected = BTreeMap::new();
    let mut columns: Vec<BTreeMap<String, LogicalType>> = vec![BTreeMap::new(); scans.len()];
    for s in data.selections {
        let key = (s.path.clone(), s.scan.clone());
        let (field, projection) = expected.get(&key).ok_or_else(fail)?;
        if *field != &s.field
            || !digest(&s.comparison_scope)
            || s.column.is_empty()
            || s.column.contains('\0')
        {
            return Err(fail());
        }
        for id in [&s.field, &s.domain] {
            let desc = original
                .catalog()
                .scalar_descriptor_by_identity(id)
                .map_err(|_| fail())?;
            let Shape::Scalar { logical_type } = desc.shape else {
                return Err(fail());
            };
            if !supported(&logical_type) || id == &s.domain && logical_type != s.logical_type {
                return Err(fail());
            }
        }
        if *projection {
            if s.mode.is_some()
                || s.original_action.is_some()
                || !matches!(s.exposure.as_deref(), Some("original" | "transformed"))
                || s.exposure.as_deref() == Some("original") && s.field != s.domain
            {
                return Err(fail());
            }
        } else {
            if s.exposure.is_some() {
                return Err(fail());
            }
            match s.mode.as_deref() {
                Some("disclosed") if s.original_action.is_none() => {}
                Some("original-authorized")
                    if s.field == s.domain
                        && s.original_action.is_some()
                        && s.original_action.as_deref()
                            == context.designation().original_action(&s.path, &s.scan)? => {}
                _ => return Err(fail()),
            }
        }
        let i = *positions.get(&s.scan).ok_or_else(fail)?;
        if columns[i]
            .insert(s.column.clone(), s.logical_type.clone())
            .is_some_and(|old| old != s.logical_type)
            || selected.insert(key, s).is_some()
        {
            return Err(fail());
        }
    }
    if selected.len() != expected.len() {
        return Err(fail());
    }
    // Resolve every operand even when its population or final result is empty.
    for op in context.operations() {
        let mut types = Vec::new();
        let mut scopes = Vec::new();
        for operand in op.operands() {
            match operand.input() {
                Input::Field { scan, .. } => {
                    let s = selected
                        .get(&(operand.path().into(), (*scan).into()))
                        .ok_or_else(fail)?;
                    types.push(s.logical_type.clone());
                    scopes.push(Some(s.comparison_scope.clone()));
                }
                Input::Literal {
                    value,
                    logical_type,
                }
                | Input::Parameter {
                    value,
                    logical_type,
                    ..
                } => {
                    scalar(value, logical_type, false)?;
                    types.push((*logical_type).clone());
                    scopes.push(None);
                }
            }
        }
        if op.kind() == Kind::Equal
            && (types.len() != 2
                || types[0] != types[1]
                || scopes[0].is_some() && scopes[1].is_some() && scopes[0] != scopes[1])
        {
            return Err(fail());
        }
        if op.kind() == Kind::Sum {
            let operand = op.operands().first().ok_or_else(fail)?;
            let Input::Field { scan, .. } = operand.input() else {
                return Err(fail());
            };
            let s = selected
                .get(&(operand.path().into(), (*scan).into()))
                .ok_or_else(fail)?;
            if !context.designation().supports_live_original_integer_sum()
                || s.mode.as_deref() != Some("original-authorized")
                || s.field != s.domain
                || s.logical_type.family != Family::Integer
            {
                return Err(fail());
            }
        }
    }
    for (i, o) in plan.outputs.iter().enumerate() {
        let t = match &o.expression {
            app::Expression::Field { scan, identity } => {
                let s = selected
                    .get(&(format!("outputs/{i}/expression"), scan.clone()))
                    .ok_or_else(fail)?;
                if plan.aggregate
                    && !plan.groups.iter().enumerate().any(|(g, f)| {
                        f.scan == *scan
                            && f.identity == *identity
                            && same_selection(
                                s,
                                selected
                                    .get(&(format!("groups/{g}"), scan.clone()))
                                    .copied(),
                            )
                    })
                {
                    return Err(fail());
                }
                s.logical_type.clone()
            }
            app::Expression::Count { logical_type }
            | app::Expression::CountDistinct { logical_type, .. } => {
                if !plan.aggregate
                    || logical_type.family != Family::Integer
                    || logical_type.nullable
                    || logical_type.facets != json!({})
                {
                    return Err(fail());
                }
                logical_type.clone()
            }
            app::Expression::Sum { logical_type, .. } => {
                if !plan.aggregate
                    || logical_type.family != Family::Integer
                    || logical_type.facets != json!({})
                    || logical_type.nullable != plan.groups.is_empty()
                {
                    return Err(fail());
                }
                logical_type.clone()
            }
            _ => return Err(fail()),
        };
        if t != data.output_types[i] {
            return Err(fail());
        }
    }
    for (i, f) in plan.order.iter().enumerate() {
        let s = selected
            .get(&(format!("order/{i}"), f.scan.clone()))
            .ok_or_else(fail)?;
        if plan.aggregate
            && !plan.groups.iter().enumerate().any(|(g, key)| {
                key.scan == f.scan
                    && key.identity == f.identity
                    && same_selection(
                        s,
                        selected
                            .get(&(format!("groups/{g}"), f.scan.clone()))
                            .copied(),
                    )
            })
        {
            return Err(fail());
        }
    }
    let mut total = 0usize;
    let mut rows = Vec::new();
    let mut keysets = Vec::new();
    for (i, scan) in frame.scans.into_iter().enumerate() {
        let count = scan
            .expected_row_count
            .parse::<usize>()
            .map_err(|_| fail())?;
        if scan.scan != scans[i]
            || count.to_string() != scan.expected_row_count
            || count != scan.rows.len()
            || count > ROWS
        {
            return Err(fail());
        }
        total = total.checked_add(count).ok_or_else(fail)?;
        if total > ROWS {
            return Err(fail());
        }
        let mut keys = BTreeSet::new();
        let mut captured = Vec::new();
        for row in scan.rows {
            if row.key.is_empty()
                || row.key.len() > 256
                || !keys.insert(row.key.clone())
                || row.cells.keys().ne(columns[i].keys())
            {
                return Err(fail());
            }
            let mut cells = BTreeMap::new();
            for (column, cell) in row.cells {
                let t = &columns[i][&column];
                let (family, value) = match cell {
                    Cell::String { value } => (Family::String, value),
                    Cell::Integer { value } => (Family::Integer, value),
                };
                if t.family != family {
                    return Err(fail());
                }
                let captured = if family == Family::String {
                    Scalar::String(value)
                } else {
                    scalar(&value, t, true)?
                };
                cells.insert(column, captured);
            }
            captured.push(Captured {
                key: row.key,
                cells,
            });
        }
        rows.push(captured);
        keysets.push(keys);
    }
    let eval = Eval {
        context,
        selected,
        scans,
        positions,
        rows,
        keysets,
    };
    let expected = eval.derive()?;
    let wanted = plan
        .limit
        .map_or(expected.len(), |n| usize::from(n).min(expected.len()));
    if frame.result_rows.len() != wanted {
        return Err(fail());
    }
    let mut pool: BTreeMap<Signature, Vec<usize>> = BTreeMap::new();
    for (i, row) in expected.iter().enumerate() {
        let ids = pool.entry(row.signature.clone()).or_default();
        if ids.first().is_some_and(|n| expected[*n].order != row.order) {
            return Err(fail());
        }
        ids.push(i);
    }
    let mut observed = Vec::new();
    let mut contributions = 0usize;
    for row in frame.result_rows {
        if row.len() != plan.outputs.len() + 1 {
            return Err(fail());
        }
        let carrier = row.last().and_then(|s| s.as_deref()).ok_or_else(fail)?;
        let bag = decode_bag(carrier, data.encoding, &eval)?;
        contributions = contributions
            .checked_add(bag.values().sum::<usize>())
            .ok_or_else(fail)?;
        if contributions > ROWS {
            return Err(fail());
        }
        let signature = Signature {
            public: row[..plan.outputs.len()].to_vec(),
            bag: bag.into_iter().collect(),
        };
        observed.push(
            pool.get_mut(&signature)
                .and_then(Vec::pop)
                .ok_or_else(fail)?,
        );
    }
    let directions = original.order_directions();
    if directions.len() != plan.order.len() {
        return Err(fail());
    }
    if !directions.is_empty() && wanted > 0 {
        let cmp = |a: usize, b: usize| order(&expected[a].order, &expected[b].order, directions);
        if observed
            .windows(2)
            .any(|w| cmp(w[0], w[1]) == Ordering::Greater)
        {
            return Err(fail());
        }
        let mut sorted: Vec<_> = (0..expected.len()).collect();
        sorted.sort_by(|a, b| cmp(*a, *b));
        let boundary = sorted[wanted - 1];
        let included: BTreeSet<_> = observed.iter().copied().collect();
        if observed
            .iter()
            .any(|n| cmp(*n, boundary) == Ordering::Greater)
            || (0..expected.len())
                .any(|n| cmp(n, boundary) == Ordering::Less && !included.contains(&n))
        {
            return Err(fail());
        }
    }
    Ok(())
}
fn same_selection(a: &Selection, b: Option<&Selection>) -> bool {
    b.is_some_and(|b| {
        a.column == b.column
            && a.domain == b.domain
            && a.comparison_scope == b.comparison_scope
            && a.logical_type == b.logical_type
    })
}
fn order(a: &[&Scalar], b: &[&Scalar], directions: &[OrderDirection]) -> Ordering {
    for ((a, b), direction) in a.iter().zip(b).zip(directions) {
        let c = if std::ptr::eq(*a, *b) {
            Ordering::Equal
        } else {
            a.cmp(b)
        };
        let c = if *direction == OrderDirection::Desc {
            c.reverse()
        } else {
            c
        };
        if c != Ordering::Equal {
            return c;
        }
    }
    Ordering::Equal
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateCell {
    scan: String,
    key: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateRows {
    version: String,
    rows: Vec<PrivateCell>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateBag {
    version: String,
    tuples: Vec<Vec<PrivateCell>>,
}
fn decode_bag(raw: &str, encoding: Encoding, eval: &Eval<'_>) -> Result<Bag> {
    let value = checked_json_bounded(raw, BYTES, 32, 1_000_000).map_err(|_| fail())?;
    let tuples = match encoding {
        Encoding::Rows => {
            let p: PrivateRows = serde_json::from_value(value).map_err(|_| fail())?;
            if p.version != "weft.stage-row-tuple/0.1-candidate" {
                return Err(fail());
            }
            vec![p.rows]
        }
        Encoding::Contributors => {
            let p: PrivateBag = serde_json::from_value(value).map_err(|_| fail())?;
            if p.version != "weft.stage-contributor-bag/0.1-candidate" {
                return Err(fail());
            }
            p.tuples
        }
    };
    if tuples.len() > ROWS {
        return Err(fail());
    }
    let mut bag = Bag::new();
    for tuple in tuples {
        if tuple.len() != eval.scans.len() {
            return Err(fail());
        }
        let mut keys = Vec::new();
        for (i, cell) in tuple.into_iter().enumerate() {
            if cell.scan != eval.scans[i]
                || cell.key.is_empty()
                || cell.key.len() > 256
                || !eval.keysets[i].contains(&cell.key)
            {
                return Err(fail());
            }
            keys.push(cell.key)
        }
        *bag.entry(keys).or_default() += 1;
    }
    Ok(bag)
}

#[cfg(test)]
mod tests;
