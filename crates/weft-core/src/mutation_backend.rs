//! Trusted selection registration/dispatch. Declarations never attest native execution.
use crate::mutation::{fail, validate_ref, Artifact, MResult, MutationDiagnostic};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SelectionStatus {
    Supported,
    Unsupported,
    Unknown,
}
#[derive(Clone, Debug, Serialize)]
pub struct SelectionProfile {
    pub id: String,
    pub version: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct SelectionClaim {
    pub id: String,
    pub status: SelectionStatus,
    pub evidence: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct SelectionCapabilities {
    pub profile: SelectionProfile,
    pub claims: Vec<SelectionClaim>,
}
#[derive(Clone, Debug, Serialize)]
pub struct SelectionAssessment {
    pub status: SelectionStatus,
    pub diagnostics: Vec<MutationDiagnostic>,
}
#[derive(Clone, Debug, Serialize)]
pub struct SelectionParameter {
    pub name: String,
    pub field: Value,
    pub value: Value,
}
#[derive(Clone, Debug, Serialize)]
pub struct SelectionColumn {
    pub name: String,
    pub kind: String,
    pub field: Option<Value>,
}
#[derive(Clone, Debug, Serialize)]
pub struct SelectionPreflight {
    pub sql: String,
    pub parameters: Vec<SelectionParameter>,
}
#[derive(Clone, Debug, Serialize)]
pub struct SelectionOrdering {
    pub kind: String,
    pub direction: String,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionBounds {
    pub max_items: u32,
    pub fetch_limit: u32,
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum SelectionLowering {
    Lowered {
        sql: String,
        parameters: Vec<SelectionParameter>,
        columns: Vec<SelectionColumn>,
        preflight: Vec<SelectionPreflight>,
        ordering: SelectionOrdering,
        bounds: SelectionBounds,
        obligations: Vec<String>,
    },
    Blocked {
        diagnostics: Vec<MutationDiagnostic>,
    },
}
/// Implementations are executable trusted Rust registrations. Models contain no plugin URLs/code.
/// Each backend owns original mapping/native admission and complete authorized-population preflight.
pub trait SelectionBackend: Send + Sync {
    fn describe(&self) -> MResult<SelectionCapabilities>;
    fn assess_selection(
        &self,
        selection: &Artifact,
        model: &Artifact,
        binding: &Artifact,
    ) -> MResult<SelectionAssessment>;
    fn lower_selection(
        &self,
        selection: &Artifact,
        model: &Artifact,
        binding: &Artifact,
        max_items: u32,
    ) -> MResult<SelectionLowering>;
}
struct Registration {
    backend: Box<dyn SelectionBackend>,
    capabilities: SelectionCapabilities,
}
#[derive(Default)]
pub(crate) struct SelectionRegistry {
    registrations: Vec<Registration>,
}
fn refused(message: &str) -> MutationDiagnostic {
    fail("ACTION", "action", message)
}
impl SelectionRegistry {
    pub(crate) fn install(&mut self, backend: Box<dyn SelectionBackend>) -> MResult<usize> {
        if self.registrations.len() >= 64 {
            return Err(fail(
                "LIMIT",
                "limit",
                "Selection registration bound exceeded",
            ));
        }
        let capabilities = callback(backend.describe())?;
        if capabilities.profile.id != "weft.update-selection"
            || capabilities.profile.version != "1"
            || capabilities.claims.len() != 2
        {
            return Err(refused(
                "Exact selection profile and complete claims required",
            ));
        }
        let mut ids = BTreeSet::new();
        for claim in &capabilities.claims {
            if claim.evidence.len() > 64 {
                return Err(fail("LIMIT", "limit", "Selection evidence count exceeded"));
            }
            for evidence in &claim.evidence {
                metadata(evidence, 4096, false)?;
            }
            if !matches!(
                claim.id.as_str(),
                "selection.interpretation" | "selection.completeRead"
            ) || !ids.insert(claim.id.clone())
                || claim.status == SelectionStatus::Supported && claim.evidence.is_empty()
            {
                return Err(refused("Selection claim identity/evidence refused"));
            }
        }
        let handle = self.registrations.len();
        self.registrations.push(Registration {
            backend,
            capabilities,
        });
        Ok(handle)
    }
    pub(crate) fn capabilities(&self, handle: usize) -> MResult<SelectionCapabilities> {
        Ok(self
            .registrations
            .get(handle)
            .ok_or_else(|| refused("Host selection registration missing"))?
            .capabilities
            .clone())
    }
    pub(crate) fn lower(
        &self,
        handle: usize,
        selection: &Artifact,
        model: &Artifact,
        binding: &Artifact,
        max_items: u32,
    ) -> MResult<SelectionLowering> {
        if !(1..=10000).contains(&max_items) {
            return Err(fail(
                "LIMIT",
                "limit",
                "Selection maxItems outside plan bound",
            ));
        }
        let registration = self
            .registrations
            .get(handle)
            .ok_or_else(|| refused("Host selection registration missing"))?;
        // Native binding semantics belong to the backend; common transport still checks original bytes.
        let program = selection.read()?;
        let document = model.read()?;
        if binding.utf8.len() > 4194304 {
            return Err(fail("LIMIT", "limit", "Binding byte bound exceeded"));
        }
        if binding.media_type != "application/json"
            || binding.sha256 != crate::json::sha256(binding.utf8.as_bytes())
        {
            return Err(refused(
                "Original bounded backend binding integrity refused",
            ));
        }
        crate::json::checked_json_bounded(&binding.utf8, 4194304, 65, 100000).map_err(|code| {
            if code == "WFT-LIMIT" {
                fail("LIMIT", "limit", "Binding JSON bound exceeded")
            } else {
                refused("Binding JSON malformed")
            }
        })?;
        let key_fields = validate_selection(&program, &document)?;
        if registration
            .capabilities
            .claims
            .iter()
            .any(|c| c.status != SelectionStatus::Supported)
        {
            return Ok(SelectionLowering::Blocked {
                diagnostics: vec![refused(
                    "Registered selection capability unsupported or unknown",
                )],
            });
        }
        let assessment = callback(
            registration
                .backend
                .assess_selection(selection, model, binding),
        )?;
        diagnostics(&assessment.diagnostics)?;
        if assessment.status == SelectionStatus::Supported && !assessment.diagnostics.is_empty() {
            return Err(refused(
                "Supported selection assessment must have no diagnostics",
            ));
        }
        if assessment.status != SelectionStatus::Supported {
            return Ok(SelectionLowering::Blocked {
                diagnostics: if assessment.diagnostics.is_empty() {
                    vec![refused(
                        "Selection mapping assessment unsupported or unknown",
                    )]
                } else {
                    assessment.diagnostics
                },
            });
        }
        let lowered = callback(
            registration
                .backend
                .lower_selection(selection, model, binding, max_items),
        )?;

        match &lowered {
            SelectionLowering::Blocked { diagnostics: ds } => {
                diagnostics(ds)?;
                if ds.is_empty() {
                    return Err(refused("Blocked lowering requires a diagnostic"));
                }
            }
            SelectionLowering::Lowered {
                sql,
                parameters,
                columns,
                preflight,
                ordering,
                bounds,
                obligations,
            } => {
                sql_text(sql)?;
                if columns.len() > 66 || preflight.len() > 64 || obligations.len() > 128 {
                    return Err(fail(
                        "LIMIT",
                        "limit",
                        "Selection collection bound exceeded",
                    ));
                }
                for obligation in obligations {
                    metadata(obligation, 4096, false)?;
                }
                if ordering.kind != "native-integer-identity"
                    || ordering.direction != "asc"
                    || bounds.max_items != max_items
                    || bounds.fetch_limit != max_items + 1
                    || columns.len() != key_fields.len() + 2
                    || preflight.is_empty()
                    || obligations.is_empty()
                {
                    return Err(refused(
                        "Incomplete selection lowering, preflight, order or overflow contract",
                    ));
                }
                let mut names = BTreeSet::new();
                let mut identity = 0;
                let mut version = 0;
                let mut keys = Vec::new();
                for column in columns {
                    metadata(&column.name, 256, false)?;
                    if !names.insert(&column.name) {
                        return Err(refused("Repeated or invalid result column"));
                    }
                    match column.kind.as_str() {
                        "identity" if column.field.is_none() => identity += 1,
                        "version" if column.field.is_none() => version += 1,
                        "key" if column.field.is_some() => {
                            validate_ref(column.field.as_ref().unwrap(), false)?;
                            keys.push(column.field.clone().unwrap())
                        }
                        _ => return Err(refused("Invalid selection result descriptor")),
                    }
                }
                if identity != 1 || version != 1 || keys != key_fields {
                    return Err(refused(
                        "Complete identity/version/declared Key order required",
                    ));
                }
                check_parameters(parameters, &document)?;
                for step in preflight {
                    sql_text(&step.sql)?;
                    check_parameters(&step.parameters, &document)?;
                }
            }
        }
        serialized_bound(&lowered, 16777216)?;
        Ok(lowered)
    }
}
fn callback<T>(result: MResult<T>) -> MResult<T> {
    match result {
        Ok(value) => Ok(value),
        Err(diagnostic) => {
            diagnostics(std::slice::from_ref(&diagnostic))?;
            Err(diagnostic)
        }
    }
}
fn sql_text(value: &str) -> MResult<()> {
    if value.len() > 65536 {
        return Err(fail("LIMIT", "limit", "Selection SQL byte bound exceeded"));
    }
    if value.is_empty() {
        return Err(refused("Nonempty SQL required"));
    }
    Ok(())
}
fn metadata(value: &str, maximum: usize, empty: bool) -> MResult<()> {
    if value.encode_utf16().count() > maximum {
        return Err(fail("LIMIT", "limit", "Selection metadata bound exceeded"));
    }
    if !empty && value.is_empty() {
        return Err(refused("Nonempty metadata required"));
    }
    Ok(())
}
// Count serialized output with a bounded writer; do not allocate an oversized response buffer.
fn serialized_bound<T: Serialize>(value: &T, maximum: usize) -> MResult<()> {
    struct Counter {
        used: usize,
        maximum: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.maximum - self.used {
                return Err(std::io::Error::other("selection output bound"));
            }
            self.used += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter { used: 0, maximum }, value)
        .map_err(|_| fail("LIMIT", "limit", "Selection output byte bound exceeded"))
}
fn literal_bound(value: &Value) -> MResult<()> {
    // Exact selected scalar parameters cannot smuggle recursive trees into serialization.
    let object = value
        .as_object()
        .filter(|o| o.len() == 1)
        .ok_or_else(|| refused("Exact scalar parameter carrier required"))?;
    if object
        .values()
        .any(|v| !matches!(v, Value::Bool(_) | Value::String(_) | Value::Number(_)))
    {
        return Err(refused("Scalar parameter payload required"));
    }
    serialized_bound(value, 1048576)
}

fn diagnostics(ds: &[MutationDiagnostic]) -> MResult<()> {
    if ds.len() > 128 {
        return Err(fail(
            "LIMIT",
            "limit",
            "Selection diagnostic count exceeded",
        ));
    }
    for d in ds {
        if !matches!(
            d.phase.as_str(),
            "input" | "parse" | "resolve" | "type" | "action" | "limit"
        ) {
            return Err(refused("Uncontracted backend diagnostic phase"));
        }
        metadata(&d.code, 256, false)?;
        metadata(&d.path, 4096, true)?;
        metadata(&d.message, 4096, false)?;
    }
    Ok(())
}
fn exact(value: &Value, keys: &[&str]) -> MResult<()> {
    let o = value
        .as_object()
        .ok_or_else(|| refused("Selection object required"))?;
    if o.len() != keys.len() || keys.iter().any(|k| !o.contains_key(*k)) {
        return Err(refused("Unknown selection member"));
    }
    Ok(())
}
fn element<'a>(document: &'a Value, reference: &Value) -> MResult<&'a Value> {
    validate_ref(reference, false)?;
    document["modules"]
        .as_array()
        .and_then(|ms| ms.iter().find(|m| m["id"] == reference["module"]))
        .and_then(|m| m["elements"].as_array())
        .and_then(|es| es.iter().find(|e| e["id"] == reference["element"]))
        .ok_or_else(|| refused("Selection Field/Record missing from original model"))
}
fn scalar<'a>(document: &'a Value, reference: &Value) -> MResult<&'a Value> {
    let f = element(document, reference)?;
    if f["kind"] != "field"
        || f["cardinality"] != "one"
        || f["nullability"] != "required"
        || !matches!(
            f["scalarType"].as_str(),
            Some("boolean" | "integer" | "string")
        )
    {
        return Err(refused("Required scalar selection Field required"));
    }
    crate::security_literals::validate_field(f)
        .map_err(|_| refused("Original Field domain refused"))?;
    Ok(f)
}
fn check_parameters(parameters: &[SelectionParameter], document: &Value) -> MResult<()> {
    if parameters.len() > 1024 {
        return Err(fail("LIMIT", "limit", "Selection parameter bound exceeded"));
    }
    let mut names = BTreeSet::new();
    for p in parameters {
        metadata(&p.name, 256, false)?;
        if !names.insert(&p.name) {
            return Err(refused("Repeated or invalid parameter name"));
        }
        literal_bound(&p.value)?;
        let field = element(document, &p.field)?;
        if field["kind"] != "field" {
            return Err(refused("Parameter must name an original Field"));
        }
        crate::security_literals::check_literal(field, &p.value)
            .map_err(|_| refused("Selection parameter violates original Field domain"))?;
    }
    Ok(())
}
/// Pure selection transport/domain inspection. It issues no owner preparation or native admission.
pub fn inspect_selection(selection: &Artifact, model: &Artifact) -> MResult<Vec<Value>> {
    validate_selection(&selection.read()?, &model.read()?)
}
fn validate_selection(program: &Value, document: &Value) -> MResult<Vec<Value>> {
    exact(program, &["protocol", "record", "key", "predicate"])?;
    if program["protocol"] != "weft.update-selection/1" || document["umf"] != "0.8.0" {
        return Err(refused("Exact selection/core profile required"));
    }
    validate_ref(&program["key"], true)?;
    let record = element(document, &program["record"])?;
    if record["kind"] != "record"
        || program["key"]["module"] != program["record"]["module"]
        || program["key"]["element"] != program["record"]["element"]
    {
        return Err(refused("Selection Record/Key differs"));
    }
    let key = record["keys"]
        .as_array()
        .and_then(|ks| ks.iter().find(|k| k["id"] == program["key"]["key"]))
        .ok_or_else(|| refused("Original selection Key missing"))?;
    let fields = key["fields"]
        .as_array()
        .filter(|fs| !fs.is_empty())
        .ok_or_else(|| refused("Original Key component bounds refused"))?;
    if fields.len() > 64 {
        return Err(fail("LIMIT", "limit", "Key component count exceeded"));
    }
    let members = record["members"]
        .as_array()
        .ok_or_else(|| refused("Original Record members required"))?;
    let mut seen = BTreeSet::new();
    for field in fields {
        if !members.contains(field) || !seen.insert(crate::mutation::canonical(field)) {
            return Err(refused("Key Fields must be unique Record members"));
        }
        if element(document, field)?["kind"] != "field" {
            return Err(refused("Original Key component is not a Field"));
        }
    }
    let mut work = vec![(&program["predicate"], 1)];
    let mut count = 0;
    while let Some((node, depth)) = work.pop() {
        if depth > 24 {
            return Err(fail("LIMIT", "limit", "Selection predicate depth exceeded"));
        }
        let o = node
            .as_object()
            .filter(|o| o.len() == 1)
            .ok_or_else(|| refused("Exact predicate node required"))?;
        if let Some(compare) = o.get("compare") {
            exact(compare, &["field", "op", "value"])?;
            if !members.contains(&compare["field"]) {
                return Err(refused("Predicate Field outside Record"));
            }
            let f = scalar(document, &compare["field"])?;
            let op = compare["op"].as_str().unwrap_or("");
            if !matches!(op, "eq" | "ne" | "lt" | "le" | "gt" | "ge")
                || f["scalarType"] == "boolean" && !matches!(op, "eq" | "ne")
            {
                return Err(refused("Predicate operator/domain refused"));
            }
            crate::security_literals::check_literal(f, &compare["value"])
                .map_err(|_| refused("Predicate literal violates original Field"))?;
            count += 1;
            if count > 256 {
                return Err(fail(
                    "LIMIT",
                    "limit",
                    "Selection comparison bound exceeded",
                ));
            }
        } else {
            let children = o
                .get("and")
                .or_else(|| o.get("or"))
                .and_then(Value::as_array)
                .filter(|cs| cs.len() >= 2)
                .ok_or_else(|| refused("Unknown predicate or operand bounds"))?;
            if children.len() > 64 {
                return Err(fail("LIMIT", "limit", "Selection operand count exceeded"));
            }
            work.extend(children.iter().rev().map(|child| (child, depth + 1)));
        }
    }
    Ok(fields.clone())
}
