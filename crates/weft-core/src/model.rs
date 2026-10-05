use crate::syntax::Name;
use crate::{
    error::{Diagnostic, Result},
    ir::{Family, Identity, LogicalType, ModelPin},
    json::{checked_json, sha256},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModuleInput {
    pub document_json: String,
    pub pin: ModelPin,
    pub selected_module_ids: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct Catalog {
    pub inputs: Vec<ModuleInput>,
    documents: Vec<Value>,
}
#[derive(Debug, Clone)]
pub struct Record {
    pub identity: Identity,
    pub pin: ModelPin,
    pub value: Value,
    pub document: usize,
}
fn fail(code: &str, message: &str) -> Diagnostic {
    Diagnostic::new(code, "model", message)
}
#[jsonschema::validator(path = "../../spec/upstream/umf-0.7.0.schema.json")]
struct Envelope;
impl Catalog {
    pub fn prepare(inputs: Vec<ModuleInput>) -> Result<Self> {
        if inputs.is_empty() || inputs.len() > 32 {
            return Err(fail(
                "WFT-LIMIT",
                "Module bundle size is outside the supported bound",
            ));
        }
        let mut documents = Vec::new();
        let mut ids = BTreeSet::new();
        let mut selections = 0;
        for input in &inputs {
            if input.document_json.len() > 4 * 1024 * 1024 {
                return Err(fail("WFT-LIMIT", "Owning document exceeds byte limit"));
            }
            if input.pin.sha256 != sha256(input.document_json.as_bytes()) {
                return Err(fail("WFT-PIN", "Owning document digest mismatch"));
            }
            let doc = checked_json(&input.document_json)
                .map_err(|c| fail(c, "Owning document JSON refused"))?;
            if doc["id"] != input.pin.document_id {
                return Err(fail("WFT-PIN", "Owning document identity mismatch"));
            }
            if doc["umf"] != "0.7.0" || input.pin.umf_version != "0.7.0" {
                return Err(fail("WFT-MODEL-VERSION", "Unsupported UMF profile"));
            }
            if input.pin.revision.is_empty() || !ids.insert(input.pin.document_id.clone()) {
                return Err(fail(
                    "WFT-MODEL",
                    "Empty revision or repeated owning document",
                ));
            }
            if !Envelope::is_valid(&doc) {
                return Err(fail(
                    "WFT-MODEL",
                    "Document violates the pinned UMF envelope",
                ));
            }
            let modules = doc["modules"].as_array().expect("schema checked modules");
            let mut mids = BTreeSet::new();
            for module in modules {
                if !mids.insert(module["id"].as_str().unwrap()) {
                    return Err(fail("WFT-MODEL", "Duplicate module identity"));
                }
                let mut eids = BTreeSet::new();
                for element in module["elements"].as_array().unwrap() {
                    if !eids.insert(element["id"].as_str().unwrap()) {
                        return Err(fail("WFT-MODEL", "Duplicate element identity"));
                    }
                }
            }
            let mut selected = BTreeSet::new();
            if input.selected_module_ids.is_empty() {
                return Err(fail("WFT-MODEL", "No module selected"));
            }
            for id in &input.selected_module_ids {
                if !selected.insert(id) || !mids.contains(id.as_str()) {
                    return Err(fail("WFT-MODEL", "Missing or repeated selected module"));
                }
            }
            selections += selected.len();
            if selections > 256 {
                return Err(fail("WFT-LIMIT", "Selected module count exceeds bound"));
            }
            documents.push(doc);
        }
        Ok(Self { inputs, documents })
    }
    pub fn pins(&self) -> Vec<ModelPin> {
        self.inputs.iter().map(|i| i.pin.clone()).collect()
    }
    pub fn record(&self, namespace: Option<&Name>, name: &Name) -> Result<Record> {
        let mut found = Vec::new();
        for (d, input) in self.inputs.iter().enumerate() {
            for module in self.documents[d]["modules"].as_array().unwrap() {
                if !input
                    .selected_module_ids
                    .iter()
                    .any(|id| module["id"] == id.as_str())
                {
                    continue;
                }
                if namespace.is_some_and(|n| !n.matches(module["namespace"].as_str().unwrap())) {
                    continue;
                }
                for e in module["elements"].as_array().unwrap() {
                    if e["kind"] == "record" && name.matches(e["name"].as_str().unwrap_or("")) {
                        found.push(Record {
                            identity: Identity {
                                document_id: input.pin.document_id.clone(),
                                revision: input.pin.revision.clone(),
                                module: module["id"].as_str().unwrap().into(),
                                element: e["id"].as_str().unwrap().into(),
                            },
                            pin: input.pin.clone(),
                            value: e.clone(),
                            document: d,
                        });
                    }
                }
            }
        }
        if found.len() > 1 {
            return Err(Diagnostic::new(
                "WFT-NAME-AMBIGUOUS",
                "resolve",
                "Source name matches multiple records",
            )
            .at(&name.span));
        }
        found.pop().ok_or_else(|| {
            Diagnostic::new(
                "WFT-NAME-MISSING",
                "resolve",
                "No queryable record matches source",
            )
            .at(&name.span)
        })
    }
    pub fn field(&self, record: &Record, name: &Name) -> Result<(Identity, LogicalType, String)> {
        let members = record.value["members"]
            .as_array()
            .ok_or_else(|| fail("WFT-TYPE", "Selected record has no explicit members"))?;
        let modules = self.documents[record.document]["modules"]
            .as_array()
            .unwrap();
        let mut refs = BTreeSet::new();
        let mut found = Vec::new();
        for reference in members {
            let mid = reference["module"].as_str().unwrap();
            let eid = reference["element"].as_str().unwrap();
            if !refs.insert((mid, eid)) {
                return Err(fail("WFT-MODEL", "Duplicate record member reference"));
            }
            let module = modules
                .iter()
                .find(|m| m["id"] == mid)
                .ok_or_else(|| fail("WFT-MODEL", "Referenced local module missing"))?;
            let element = module["elements"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["id"] == eid)
                .ok_or_else(|| fail("WFT-MODEL", "Referenced local element missing"))?;
            if name.matches(element["name"].as_str().unwrap_or("")) {
                let mut identity = record.identity.clone();
                identity.module = mid.into();
                identity.element = eid.into();
                found.push((identity, element));
            }
        }
        if found.len() > 1 {
            return Err(Diagnostic::new(
                "WFT-NAME-AMBIGUOUS",
                "resolve",
                "Property name matches multiple members",
            )
            .at(&name.span));
        }
        let (identity, field) = found.pop().ok_or_else(|| {
            Diagnostic::new(
                "WFT-NAME-MISSING",
                "resolve",
                "Property not found among record members",
            )
            .at(&name.span)
        })?;
        let err = || {
            Diagnostic::new("WFT-TYPE", "type", "Selected field meaning is unsupported")
                .at(&name.span)
        };
        if field["kind"] != "field"
            || field["nullability"] != "required"
            || field["cardinality"] != "one"
            || field.get("itemType").is_some()
            || field.get("recordType").is_some()
            || field["references"]
                .as_array()
                .is_some_and(|r| !r.is_empty())
        {
            return Err(err());
        }
        let family = match field["scalarType"].as_str() {
            Some("boolean") => Family::Boolean,
            Some("string") => Family::String,
            Some("integer") => Family::Integer,
            Some("decimal") => Family::Decimal,
            _ => return Err(err()),
        };
        let facets = field.get("facets").cloned().unwrap_or(json!({}));
        let allowed: &[&str] = match family {
            Family::Integer => &["integerWidth"],
            Family::Decimal => &["precision", "scale"],
            _ => &[],
        };
        if facets
            .as_object()
            .unwrap()
            .keys()
            .any(|k| !allowed.contains(&k.as_str()))
        {
            return Err(err());
        }
        if family == Family::Integer {
            let width = &facets["integerWidth"];
            let bits = width["bits"].as_u64().ok_or_else(err)?;
            if !(1..=64).contains(&bits)
                || !width["signed"].is_boolean()
                || width.as_object().is_none_or(|m| m.len() != 2)
            {
                return Err(err());
            }
        }
        if family == Family::Decimal {
            let p = facets["precision"].as_u64().ok_or_else(err)?;
            let s = facets["scale"].as_u64().ok_or_else(err)?;
            if !(1..=28).contains(&p) || s > p {
                return Err(err());
            }
        }
        Ok((
            identity,
            LogicalType {
                family,
                facets,
                nullable: false,
            },
            field["name"].as_str().unwrap().into(),
        ))
    }
}
