// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::syntax::Name;
use crate::{
    error::{Diagnostic, Result},
    ir::{Family, Identity, LogicalType, ModelPin},
    json::{checked_json, sha256},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap};

type ModuleElementIndex = HashMap<String, (usize, HashMap<String, usize>)>;
type RecordLocation = (usize, usize);
#[derive(Debug, Default, Clone)]
struct RecordNameIndex {
    quoted_names: HashMap<String, Vec<RecordLocation>>,
    unquoted_names: HashMap<String, Vec<RecordLocation>>,
    quoted_unnamed_ids: HashMap<String, Vec<RecordLocation>>,
    unquoted_unnamed_ids: HashMap<String, Vec<RecordLocation>>,
    scoped_quoted_names: HashMap<(String, String), Vec<RecordLocation>>,
    scoped_unquoted_names: HashMap<(String, String), Vec<RecordLocation>>,
    scoped_quoted_unnamed_ids: HashMap<(String, String), Vec<RecordLocation>>,
    scoped_unquoted_unnamed_ids: HashMap<(String, String), Vec<RecordLocation>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModuleInput {
    pub document_json: String,
    pub pin: ModelPin,
    pub selected_module_ids: Vec<String>,
}
#[derive(Debug, Clone)]
/// A prepared snapshot retains exact source and parsed meaning together.
/// Direct mutation is inaccessible:
/// ```compile_fail
/// fn tamper(catalog: &mut weft_core::model::Catalog) {
///     catalog.inputs[0].document_json.clear();
/// }
/// ```
/// The public accessor also cannot mutate retained source:
/// ```compile_fail
/// fn tamper(catalog: &mut weft_core::model::Catalog) {
///     catalog.inputs()[0].document_json.clear();
/// }
/// ```

pub struct Catalog {
    inputs: Vec<ModuleInput>,
    pub(crate) exact_numbers: bool,
    pub(crate) field_projection_decimal38: bool,
    pub(crate) sdk_scalar_presence: bool,
    security_inputs: bool,
    pub(crate) identity_name_fallback: bool,
    raw_read_opaque_extensions: bool,
    native_operation_domains: bool,
    logical_defaults020: bool,
    pub(crate) exact_scalars: bool,
    pub(crate) exact_dates: bool,
    pub(crate) bounded_current_strings: bool,
    pub(crate) documents: Vec<Value>,
    element_locations: Vec<ModuleElementIndex>,
    record_names: Vec<RecordNameIndex>,
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
/// UMF 0.10 selected semantics are admitted only where this compiler can
/// preserve their value domain. External semantic type references are retained
/// in source but remain outside the raw scalar profile.
pub(crate) fn selected010(pin: &ModelPin, element: &Value) -> Result<()> {
    selected010_inner(pin, element, false, false, false, None)
}
pub(crate) fn selected010_raw_read(pin: &ModelPin, element: &Value, document: &Value) -> Result<()> {
    selected010_inner(pin, element, true, false, false, Some(document))
}
pub(crate) fn selected010_table_operation(pin: &ModelPin, element: &Value, document: &Value) -> Result<()> {
    selected010_inner(pin, element, true, true, false, Some(document))
}
pub(crate) fn selected010_table_operation020(pin: &ModelPin, element: &Value, document: &Value) -> Result<()> {
    selected010_inner(pin, element, true, true, true, Some(document))
}
fn selected010_inner(pin: &ModelPin, element: &Value, allow_raw_read_metadata: bool, native_domains: bool, logical_defaults020: bool, document: Option<&Value>) -> Result<()> {
    if pin.umf_version != "0.10.0" { return Ok(()); }
    const KNOWN: &[&str] = &["id", "name", "title", "description", "kind", "scalarType",
        "nullability", "cardinality", "itemType", "recordType", "facets", "members",
        "keys", "references", "extensions", "semanticTypes"];
    let extensions_supported = element.get("extensions").is_none_or(|value| {
        value.as_object().is_some_and(|map| {
            if !allow_raw_read_metadata { return map.is_empty(); }
            map.iter().all(|(name, payload)| {
                payload.is_object() && match name.as_str() {
                    "umf.ddd" => element["kind"] == "record",
                    "hohfeld.view" => matches!(element["kind"].as_str(), Some("record" | "field")),
                    "arras.view" => matches!(element["kind"].as_str(), Some("record" | "field"))
                        && document.is_some_and(|doc| doc["vocabularies"]["arras.view"]["version"] == "0.1.0"),
                    _ => false,
                }
            })
        })
    });
    let default020_is_closed = element.get("default").is_none_or(|default| {
        logical_defaults020
            && element["kind"] == "field"
            && default.as_object().is_some_and(|members| {
                members.len() == 2
                    && members.contains_key("on")
                    && members.contains_key("value")
                    && default["on"] == "missing"
                    && !default["value"].is_null()
            })
    });
    if element.as_object().is_none_or(|m|m.keys().any(|k|!KNOWN.contains(&k.as_str())&&!(native_domains&&element["kind"]=="field"&&k=="allowedValues")&&!(logical_defaults020&&element["kind"]=="field"&&k=="default"&&default020_is_closed)))
        || element.get("semanticTypes").is_some()
        || !extensions_supported
    { return Err(fail("WFT-TYPE", "Selected UMF 0.10 element has semantics outside the raw scalar profile")); }
    if element["kind"] == "record" && element.get("references").and_then(Value::as_array).is_some_and(|r|!r.is_empty()) {
        return Err(fail("WFT-TYPE", "Selected UMF 0.10 Record reference roles are unsupported"));
    }
    for member in element.get("members").and_then(Value::as_array).into_iter().flatten() {
        if member.as_object().is_none_or(|m|m.keys().any(|k|!matches!(k.as_str(),"module"|"element"))) {
            return Err(fail("WFT-TYPE", "Selected UMF 0.10 reference has unsupported qualifiers"));
        }
    }
    Ok(())
}
#[jsonschema::validator(path = "../../spec/upstream/umf-0.10.0-bundled.schema.json")]
struct Envelope10;
impl Catalog {
    /// Immutable evidence of the trusted SDK scalar presence preparation path.
    pub fn sdk_scalar_presence(&self)->bool {self.sdk_scalar_presence}
    /// New host-selected profile only; authored names always take precedence.
    pub(crate) fn query_name<'a>(&self, element: &'a Value) -> Option<&'a str> {
        element.get("name").and_then(Value::as_str).or_else(|| {
            self.identity_name_fallback.then(|| element["id"].as_str()).flatten()
        })
    }

    pub(crate) fn validate_selected010(&self, pin: &ModelPin, element: &Value) -> Result<()> {
        if self.native_operation_domains || self.raw_read_opaque_extensions {
            let document = self.inputs.iter().zip(&self.documents)
                .find(|(input, _)| input.pin == *pin)
                .map(|(_, document)| document)
                .ok_or_else(|| fail("WFT-PIN", "Selected element has no exact owning document"))?;
            if self.native_operation_domains && self.logical_defaults020 {
                selected010_table_operation020(pin, element, document)
            } else if self.native_operation_domains {
                selected010_table_operation(pin, element, document)
            }
            else { selected010_raw_read(pin, element, document) }
        }
        else { selected010(pin, element) }
    }

    /// Read-only original inputs. Clone and prepare a new Catalog to change them.
    pub fn inputs(&self) -> &[ModuleInput] {
        &self.inputs
    }
    /// Explicit host-selected admission of facetless mathematical numeric fields.
    /// Source bytes/pins remain original; default preparation is unchanged.
    pub fn prepare_exact_numbers(inputs: Vec<ModuleInput>) -> Result<Self> {
        let mut catalog=Self::prepare(inputs)?;catalog.exact_numbers=true;Ok(catalog)
    }
    pub fn prepare(inputs: Vec<ModuleInput>) -> Result<Self> {
        Self::prepare_version(inputs, None, false)
    }
    pub(crate) fn prepare_truss_runtime(inputs: Vec<ModuleInput>, exact_numbers: bool) -> Result<Self> {
        let mut catalog = Self::prepare_version(inputs, None, true)?;
        catalog.exact_numbers = exact_numbers;
        catalog.raw_read_opaque_extensions = true;
        Ok(catalog)
    }
    /// Source custody only; security activation remains independently gated.
    pub fn prepare_security(inputs: Vec<ModuleInput>) -> Result<Self> {
        Self::prepare_version(inputs, Some("0.10.0"), false)
    }
    pub(crate) fn prepare_raw_read010(inputs: Vec<ModuleInput>) -> Result<Self> {
        let mut catalog = Self::prepare_version(inputs, Some("0.10.0"), false)?;
        catalog.identity_name_fallback = true;
        catalog.raw_read_opaque_extensions = true;
        Ok(catalog)
    }
    /// Finite Unity aggregate admission retains the raw-read checks while
    /// permitting source facets that its bounded aggregate owners validate.
    pub(crate) fn prepare_raw_read010_finite_current_aggregate(inputs: Vec<ModuleInput>) -> Result<Self> {
        let mut catalog = Self::prepare_raw_read010(inputs)?;
        catalog.exact_numbers = true;
        catalog.field_projection_decimal38 = true;
        Ok(catalog)
    }
    pub(crate) fn prepare_table_operation010(inputs: Vec<ModuleInput>) -> Result<Self> {
        let mut catalog=Self::prepare_raw_read010(inputs)?;
        catalog.native_operation_domains=true;
        Ok(catalog)
    }
    pub(crate) fn prepare_table_operation020(inputs: Vec<ModuleInput>) -> Result<Self> {
        let mut catalog=Self::prepare_table_operation010(inputs)?;
        catalog.logical_defaults020=true;
        Ok(catalog)
    }
    pub(crate) fn prepare_security_bounded_strings(inputs: Vec<ModuleInput>) -> Result<Self> {
        let mut catalog = Self::prepare_security(inputs)?;
        catalog.bounded_current_strings = true;
        Ok(catalog)
    }
    fn prepare_version(inputs: Vec<ModuleInput>, required_version: Option<&str>, truss_runtime: bool) -> Result<Self> {
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
            let doc = (if truss_runtime {
                crate::json::checked_truss_runtime_catalog_json(&input.document_json)
            } else {
                checked_json(&input.document_json)
            })
                .map_err(|c| fail(c, "Owning document JSON refused"))?;
            if doc["id"] != input.pin.document_id {
                return Err(fail("WFT-PIN", "Owning document identity mismatch"));
            }
            if input.pin.umf_version != "0.10.0"
                || doc["umf"] != "0.10.0"
                || required_version.is_some_and(|version| input.pin.umf_version != version)
            {
                return Err(fail("WFT-MODEL-VERSION", "Unsupported UMF profile"));
            }
            if input.pin.revision.is_empty() || !ids.insert(input.pin.document_id.clone()) {
                return Err(fail(
                    "WFT-MODEL",
                    "Empty revision or repeated owning document",
                ));
            }
            if !Envelope10::is_valid(&doc) {
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
        // Index coordinates only. Original immutable documents and their order
        // remain authoritative; no Field meaning or authorization is cached.
        let element_locations = documents.iter().map(|document| {
            document["modules"].as_array().unwrap().iter().enumerate().map(|(mi, module)| {
                let elements = module["elements"].as_array().unwrap().iter().enumerate()
                    .map(|(ei, element)| (element["id"].as_str().unwrap().to_owned(), ei)).collect();
                (module["id"].as_str().unwrap().to_owned(), (mi, elements))
            }).collect()
        }).collect();
        let record_names = documents.iter().map(|document| {
            let mut index = RecordNameIndex::default();
            for (module_index, module) in document["modules"].as_array().unwrap().iter().enumerate() {
                for (element_index, element) in module["elements"].as_array().unwrap().iter().enumerate() {
                    if element["kind"] != "record" { continue; }
                    let namespace = module["namespace"].as_str();
                    if let Some(name) = element["name"].as_str() {
                        index.quoted_names.entry(name.to_owned()).or_default().push((module_index, element_index));
                        index.unquoted_names.entry(name.to_ascii_lowercase()).or_default().push((module_index, element_index));
                        if let Some(namespace) = namespace {
                            let folded_namespace = namespace.to_ascii_lowercase();
                            index.scoped_quoted_names.entry((namespace.to_owned(), name.to_owned())).or_default().push((module_index, element_index));
                            index.scoped_unquoted_names.entry((namespace.to_owned(), name.to_ascii_lowercase())).or_default().push((module_index, element_index));
                            if folded_namespace != namespace {
                                index.scoped_quoted_names.entry((folded_namespace.clone(), name.to_owned())).or_default().push((module_index, element_index));
                                index.scoped_unquoted_names.entry((folded_namespace, name.to_ascii_lowercase())).or_default().push((module_index, element_index));
                            }
                        }
                    } else if let Some(id) = element["id"].as_str() {
                        index.quoted_unnamed_ids.entry(id.to_owned()).or_default().push((module_index, element_index));
                        index.unquoted_unnamed_ids.entry(id.to_ascii_lowercase()).or_default().push((module_index, element_index));
                        if let Some(namespace) = namespace {
                            let folded_namespace = namespace.to_ascii_lowercase();
                            index.scoped_quoted_unnamed_ids.entry((namespace.to_owned(), id.to_owned())).or_default().push((module_index, element_index));
                            index.scoped_unquoted_unnamed_ids.entry((namespace.to_owned(), id.to_ascii_lowercase())).or_default().push((module_index, element_index));
                            if folded_namespace != namespace {
                                index.scoped_quoted_unnamed_ids.entry((folded_namespace.clone(), id.to_owned())).or_default().push((module_index, element_index));
                                index.scoped_unquoted_unnamed_ids.entry((folded_namespace, id.to_ascii_lowercase())).or_default().push((module_index, element_index));
                            }
                        }
                    }
                }
            }
            index
        }).collect();
        Ok(Self { inputs, documents, element_locations, record_names, exact_numbers:false, field_projection_decimal38:false, sdk_scalar_presence:false, exact_scalars:false, exact_dates:false, bounded_current_strings:false, security_inputs:required_version == Some("0.10.0"), identity_name_fallback:false, raw_read_opaque_extensions:false, native_operation_domains:false, logical_defaults020:false })
    }
    pub(crate) fn local_element_by_ids(&self, document: usize, module: &str, element: &str) -> Option<&Value> {
        let (module_index, elements) = self.element_locations.get(document)?.get(module)?;
        let element_index = elements.get(element)?;
        self.documents.get(document)?.get("modules")?.get(*module_index)?.get("elements")?.get(*element_index)
    }
    pub fn pins(&self) -> Vec<ModelPin> {
        self.inputs.iter().map(|i| i.pin.clone()).collect()
    }
    /// Resolve a selected Record by complete original identity, never its name.
    pub fn record_by_identity(&self, identity: &Identity) -> Result<Record> {
        let mut found = Vec::new();
        for (document, input) in self.inputs.iter().enumerate() {
            if input.pin.document_id != identity.document_id
                || input.pin.revision != identity.revision
            {
                continue;
            }
            if !input.selected_module_ids.contains(&identity.module) { continue; }
            if let Some(element) = self.local_element_by_ids(document, &identity.module, &identity.element) {
                if element["kind"] == "record" {
                    self.validate_selected010(&input.pin, element)?;
                    found.push(Record { identity: identity.clone(), pin: input.pin.clone(), value: element.clone(), document });
                }
            }
        }
        if found.len() != 1 {
            return Err(fail(
                "WFT-NAME-MISSING",
                "Original Record identity is missing or ambiguous",
            ));
        }
        Ok(found.pop().unwrap())
    }
    pub fn record(&self, namespace: Option<&Name>, name: &Name) -> Result<Record> {
        let mut found = Vec::new();
        for (d, input) in self.inputs.iter().enumerate() {
            let index = &self.record_names[d];
            let named = if let Some(namespace) = namespace {
                let key = (namespace.value.clone(), name.value.clone());
                if name.quoted {
                    index.scoped_quoted_names.get(&key)
                } else {
                    index.scoped_unquoted_names.get(&key)
                }
            } else {
                if name.quoted {
                    index.quoted_names.get(&name.value)
                } else {
                    index.unquoted_names.get(&name.value)
                }
            }.map(Vec::as_slice).unwrap_or(&[]);
            let unnamed = if self.identity_name_fallback {
                if let Some(namespace) = namespace {
                    let key = (namespace.value.clone(), name.value.clone());
                    if name.quoted {
                        index.scoped_quoted_unnamed_ids.get(&key)
                    } else {
                        index.scoped_unquoted_unnamed_ids.get(&key)
                    }
                } else {
                    if name.quoted {
                        index.quoted_unnamed_ids.get(&name.value)
                    } else {
                        index.unquoted_unnamed_ids.get(&name.value)
                    }
                }.map(Vec::as_slice).unwrap_or(&[])
            } else { &[] };
            let (mut named_index, mut unnamed_index) = (0, 0);
            while named_index < named.len() || unnamed_index < unnamed.len() {
                let (module_index, element_index) = if named_index == named.len()
                    || (unnamed_index < unnamed.len() && unnamed[unnamed_index] < named[named_index])
                {
                    let location = unnamed[unnamed_index];
                    unnamed_index += 1;
                    location
                } else {
                    let location = named[named_index];
                    named_index += 1;
                    location
                };
                let module = &self.documents[d]["modules"][module_index];
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
                let e = &module["elements"][element_index];
                if e["kind"] == "record" {
                    self.validate_selected010(&input.pin, e)?;
                    if self.raw_read_opaque_extensions && input.pin.umf_version == "0.10.0" {
                            for member in e["members"].as_array().ok_or_else(|| fail("WFT-TYPE", "Selected UMF 0.10 Record has no members"))? {
                                let validate_reference = |reference: &Value| -> Result<()> {
                                    let module_id = reference["module"].as_str().ok_or_else(|| fail("WFT-TYPE", "Selected UMF 0.10 reference module absent"))?;
                                    let element_id = reference["element"].as_str().ok_or_else(|| fail("WFT-TYPE", "Selected UMF 0.10 reference element absent"))?;
                                    let (module_index, element_indexes) = self.element_locations[d].get(module_id).ok_or_else(|| fail("WFT-TYPE", "Selected UMF 0.10 reference module absent"))?;
                                    let element_index = element_indexes.get(element_id).ok_or_else(|| fail("WFT-TYPE", "Selected UMF 0.10 reference element absent"))?;
                                    let field = &self.documents[d]["modules"][*module_index]["elements"][*element_index];
                                    self.validate_selected010(&input.pin, field)?;
                                    if let Some(item) = field.get("itemType") {
                                        let item_module = item["module"].as_str().ok_or_else(|| fail("WFT-TYPE", "Selected UMF 0.10 item module absent"))?;
                                        let item_element = item["element"].as_str().ok_or_else(|| fail("WFT-TYPE", "Selected UMF 0.10 item element absent"))?;
                                        let (item_module_index, item_indexes) = self.element_locations[d].get(item_module).ok_or_else(|| fail("WFT-TYPE", "Selected UMF 0.10 item module absent"))?;
                                        let item_index = item_indexes.get(item_element).ok_or_else(|| fail("WFT-TYPE", "Selected UMF 0.10 item element absent"))?;
                                        self.validate_selected010(&input.pin, &self.documents[d]["modules"][*item_module_index]["elements"][*item_index])?;
                                    }
                                    Ok(())
                                };
                                validate_reference(member)?;
                            }
                    }
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
        if found.len() > 1 {
            return Err(Diagnostic::new(
                "WFT-NAME-AMBIGUOUS",
                "resolve",
                "Source name matches multiple records",
            ).unavailable_source()
            .at(&name.span));
        }
        found.pop().ok_or_else(|| {
            Diagnostic::new(
                "WFT-NAME-MISSING",
                "resolve",
                "No queryable record matches source",
            ).unavailable_source()
            .at(&name.span)
        })
    }
    pub fn field(&self, record: &Record, name: &Name) -> Result<(Identity, LogicalType, String)> {
        self.field_mode(record,name,self.security_inputs)
    }
    pub(crate) fn field03(&self, record:&Record,name:&Name)->Result<(Identity,LogicalType,String)> {
        self.field_mode(record,name,true)
    }
    fn field_mode(&self,record:&Record,name:&Name,optional03:bool)->Result<(Identity,LogicalType,String)> {
        let members = record.value["members"]
            .as_array()
            .ok_or_else(|| fail("WFT-TYPE", "Selected record has no explicit members"))?;
        let mut refs = BTreeSet::new();
        let mut found = Vec::new();
        for reference in members {
            let mid = reference["module"].as_str().unwrap();
            let eid = reference["element"].as_str().unwrap();
            if !refs.insert((mid, eid)) {
                return Err(fail("WFT-MODEL", "Duplicate record member reference"));
            }
            if !self.element_locations[record.document].contains_key(mid) {
                return Err(fail("WFT-MODEL", "Referenced local module missing"));
            }
            let element = self.local_element_by_ids(record.document, mid, eid)
                .ok_or_else(|| fail("WFT-MODEL", "Referenced local element missing"))?;
            if name.matches(self.query_name(element).unwrap_or("")) {
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
            ).column(true)
            .at(&name.span));
        }
        let (identity, field) = found.pop().ok_or_else(|| {
            Diagnostic::new(
                "WFT-NAME-MISSING",
                "resolve",
                "Property not found among record members",
            ).column(false)
            .at(&name.span)
        })?;
        self.validate_selected010(&record.pin, field)?;
        let ideal;
        let selected=if optional03 && field["nullability"]=="absent-allowed" {
            // Derive the ideal scalar domain, leaving original selected source bytes untouched.
            ideal={let mut v=field.clone();v["nullability"]=Value::String("required".into());v};&ideal
        } else {field};
        let logical_type = scalar_type_projected(selected, name, self.exact_numbers, self.exact_numbers, self.bounded_current_strings, self.field_projection_decimal38)?;
        Ok((
            identity,
            logical_type,
            self.query_name(field).expect("matched query name").into(),
        ))
    }
}

#[cfg(test)]
mod selected_module020_tests {
    use super::*;
    use serde_json::json;

    fn pin() -> ModelPin {
        ModelPin {
            document_id: "demo".into(),
            revision: "1".into(),
            umf_version: "0.10.0".into(),
            sha256: "0".repeat(64),
        }
    }

    fn field(default: Value) -> Value {
        json!({
            "kind":"field", "scalarType":"integer", "cardinality":"one",
            "nullability":"required", "extensions":{}, "facets":{},
            "default":default
        })
    }

    #[test]
    fn module020_is_the_only_selected_path_that_admits_closed_non_null_defaults() {
        let pin = pin();
        let document = json!({});
        let accepted = field(json!({"on":"missing","value":{"integerToken":"5"}}));
        assert!(selected010_table_operation020(&pin, &accepted, &document).is_ok());
        assert!(selected010_table_operation(&pin, &accepted, &document).is_err());
        assert!(selected010(&pin, &accepted).is_err());

        for invalid in [
            json!({"on":"missing","value":null}),
            json!({"on":"null","value":{"integerToken":"5"}}),
            json!({"on":"missing","value":{"integerToken":"5"},"extra":true}),
        ] {
            let field = field(invalid);
            assert!(selected010_table_operation020(&pin, &field, &document).is_err());
        }
    }
}

pub(crate) fn scalar_type(field: &Value, name: &Name, mathematical_integer: bool, mathematical_decimal: bool, bounded_strings: bool) -> Result<LogicalType> {
    scalar_type_projected(field,name,mathematical_integer,mathematical_decimal,bounded_strings,false)
}
pub(crate) fn scalar_type_projected(field: &Value, name: &Name, mathematical_integer: bool, mathematical_decimal: bool, bounded_strings: bool, field_decimal38:bool) -> Result<LogicalType> {
    let err = || {
        Diagnostic::new("WFT-TYPE", "type", "Selected field meaning is unsupported").at(&name.span)
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
        Family::String if bounded_strings => &["length"],
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
    if family == Family::String && bounded_strings {
        crate::current_string_domain::bounds(&facets).map_err(|_| err())?;
    }
    if family == Family::Integer && !(mathematical_integer && facets == json!({})) {
        let width = &facets["integerWidth"];
        let bits = width["bits"].as_u64().ok_or_else(err)?;
        if !(1..=64).contains(&bits)
            || !width["signed"].is_boolean()
            || width.as_object().is_none_or(|m| m.len() != 2)
        {
            return Err(err());
        }
    }
    if family == Family::Decimal && !(mathematical_decimal && facets==json!({})) {
        let p = facets["precision"].as_u64().ok_or_else(err)?;
        let s = facets["scale"].as_u64().ok_or_else(err)?;
        if !(1..=if field_decimal38 {38}else{28}).contains(&p) || s > p {
            return Err(err());
        }
    }
    Ok(LogicalType {
        family,
        facets,
        nullable: false,
    })
}

#[cfg(test)]
mod core010_admission_tests {
    use super::*;

    fn current_input() -> ModuleInput {
        let document_json = include_str!("../../../tests/raw-postgresql/table-lifecycle-model.json").to_owned();
        let doc: Value = serde_json::from_str(&document_json).unwrap();
        ModuleInput {
            pin: ModelPin {
                document_id: doc["id"].as_str().unwrap().to_owned(),
                revision: "current-fixture".into(),
                umf_version: "0.10.0".into(),
                sha256: sha256(document_json.as_bytes()),
            },
            document_json,
            selected_module_ids: vec!["demo".into()],
        }
    }

    #[test]
    fn current_010_is_admitted_and_older_core_versions_are_rejected() {
        let input = current_input();
        let catalog = Catalog::prepare(vec![input.clone()]).expect("UMF 0.10 is supported");
        assert_eq!(catalog.pins()[0].umf_version, "0.10.0");
        Catalog::prepare_security(vec![input]).expect("security preparation uses the same current core");

        for version in ["0.7.0", "0.8.0"] {
            let mut document: Value = serde_json::from_str(&current_input().document_json).unwrap();
            document["umf"] = json!(version);
            let document_json = document.to_string();
            let mut input = current_input();
            input.document_json = document_json.clone();
            input.pin.umf_version = version.into();
            input.pin.sha256 = sha256(document_json.as_bytes());
            let error = Catalog::prepare(vec![input.clone()]).unwrap_err();
            assert_eq!(error.code, "WFT-MODEL-VERSION", "version {version}");
            assert_eq!(Catalog::prepare_security(vec![input]).unwrap_err().code, "WFT-MODEL-VERSION");
        }
    }

    #[test]
    fn arras_view_requires_the_exact_owning_document_declaration() {
        let mut input_a = current_input();
        let mut document_a: Value = serde_json::from_str(&input_a.document_json).unwrap();
        document_a["vocabularies"]["arras.view"] = json!({"version":"0.1.0"});
        let field_a = document_a["modules"][0]["elements"].as_array_mut().unwrap()
            .iter_mut().find(|element| element["kind"] == "field").unwrap();
        field_a["extensions"]["arras.view"] = json!({"opaque":"preserved"});
        input_a.document_json = document_a.to_string();
        input_a.pin.sha256 = sha256(input_a.document_json.as_bytes());

        let mut input_b = input_a.clone();
        let mut document_b: Value = serde_json::from_str(&input_b.document_json).unwrap();
        document_b["id"] = json!("second-document");
        document_b["vocabularies"].as_object_mut().unwrap().remove("arras.view");
        input_b.pin.document_id = "second-document".into();
        let field_b = document_b["modules"][0]["elements"].as_array_mut().unwrap()
            .iter_mut().find(|element| element["kind"] == "field").unwrap();
        field_b["extensions"]["arras.view"] = json!({"opaque":"belongs to B"});
        input_b.document_json = document_b.to_string();
        input_b.pin.sha256 = sha256(input_b.document_json.as_bytes());

        let catalog = Catalog::prepare_raw_read010(vec![input_a.clone(), input_b.clone()]).unwrap();
        let field_a = &catalog.documents[0]["modules"][0]["elements"].as_array().unwrap()
            .iter().find(|element| element["kind"] == "field").unwrap();
        assert!(catalog.validate_selected010(&input_a.pin, field_a).is_ok());
        let field_b = &catalog.documents[1]["modules"][0]["elements"].as_array().unwrap()
            .iter().find(|element| element["kind"] == "field").unwrap();
        assert_eq!(catalog.validate_selected010(&input_b.pin, field_b).unwrap_err().code, "WFT-TYPE");

        // The ordinary selected profile has no document metadata context.
        let ordinary = Catalog::prepare(vec![input_a.clone()]).unwrap();
        assert_eq!(ordinary.validate_selected010(&input_a.pin, field_a).unwrap_err().code, "WFT-TYPE");
        let mut foreign_pin = input_a.pin.clone();
        foreign_pin.revision = "foreign".into();
        assert_eq!(catalog.validate_selected010(&foreign_pin, field_a).unwrap_err().code, "WFT-PIN");
    }

    #[test]
    fn indexed_record_lookup_preserves_name_matching_and_ambiguity() {
        let mut input = current_input();
        let mut document: Value = serde_json::from_str(&input.document_json).unwrap();
        document["modules"][0]["elements"].as_array_mut().unwrap()
            .iter_mut().find(|element| element["id"] == "Note").unwrap()["name"] = json!("Display");
        input.document_json = document.to_string();
        input.pin.sha256 = sha256(input.document_json.as_bytes());
        let single = Catalog::prepare_raw_read010(vec![input.clone()]).unwrap();
        let name = |value: &str, quoted| Name {
            value: value.into(), quoted, span: crate::ir::Span { start: 0, end: 0 },
        };
        assert_eq!(single.record(None, &name("display", false)).unwrap().identity.element, "Note");
        assert!(single.record(None, &name("DISPLAY", false)).is_err());
        assert_eq!(single.record(None, &name("Display", true)).unwrap().identity.element, "Note");
        assert!(single.record(None, &name("display", true)).is_err());

        let elements = document["modules"][0]["elements"].as_array_mut().unwrap();
        let mut duplicate = elements.iter().find(|element| element["id"] == "Note").unwrap().clone();
        duplicate["id"] = json!("Note2");
        duplicate["name"] = json!("DISPLAY");
        elements.push(duplicate);
        input.document_json = document.to_string();
        input.pin.sha256 = sha256(input.document_json.as_bytes());
        let catalog = Catalog::prepare_raw_read010(vec![input]).unwrap();

        assert_eq!(catalog.record(None, &name("display", false)).unwrap_err().code, "WFT-NAME-AMBIGUOUS");
        assert_eq!(catalog.record(None, &name("Display", true)).unwrap().identity.element, "Note");
        assert_eq!(catalog.record(None, &name("DISPLAY", true)).unwrap().identity.element, "Note2");
        // A declared display Name takes precedence over the identity-ID fallback.
        assert!(catalog.record(None, &name("Note", true)).is_err());
    }

    #[test]
    fn current_selected_content_still_refuses_unrecognized_extension_meaning() {
        let mut input = current_input();
        let mut document: Value = serde_json::from_str(&input.document_json).unwrap();
        let field = document["modules"][0]["elements"].as_array_mut().unwrap()
            .iter_mut().find(|element| element["id"] == "owner").unwrap();
        field["extensions"]["example.unknown"] = json!({"payload": 1});
        input.document_json = document.to_string();
        input.pin.sha256 = sha256(input.document_json.as_bytes());
        let mut catalog = Catalog::prepare(vec![input]).unwrap();
        catalog.identity_name_fallback = true;
        let name = Name { value: "owner".into(), quoted: true, span: crate::ir::Span { start: 0, end: 0 } };
        let record_name = Name { value: "Note".into(), quoted: true, span: crate::ir::Span { start: 0, end: 0 } };
        let record = catalog.record(None, &record_name).unwrap();
        assert_eq!(catalog.field(&record, &name).unwrap_err().code, "WFT-TYPE");
    }

    #[test]
    fn exact_numeric_mode_remains_available_for_current_core_sources() {
        let mut document: Value = serde_json::from_str(current_input().document_json.as_str()).unwrap();
        let elements = document["modules"][0]["elements"].as_array_mut().unwrap();
        for element in elements.iter_mut() {
            if element["name"].is_null() {
                element["name"] = element["id"].clone();
            }
        }
        let record = elements.iter_mut().find(|element| element["id"] == "Note").unwrap();
        record["members"].as_array_mut().unwrap().extend([
            json!({"module":"demo","element":"amount"}),
            json!({"module":"demo","element":"units"}),
        ]);
        elements.extend([
            json!({"id":"amount","name":"amount","kind":"field","scalarType":"decimal","nullability":"required","cardinality":"one","extensions":{}}),
            json!({"id":"units","name":"units","kind":"field","scalarType":"integer","nullability":"required","cardinality":"one","extensions":{}}),
        ]);
        let document_json = document.to_string();
        let input = ModuleInput {
            pin: ModelPin {
                document_id: document["id"].as_str().unwrap().to_owned(),
                revision: "numeric-current".into(),
                umf_version: "0.10.0".into(),
                sha256: sha256(document_json.as_bytes()),
            },
            document_json,
            selected_module_ids: vec!["demo".into()],
        };
        let identity = Identity {
            document_id: input.pin.document_id.clone(),
            revision: input.pin.revision.clone(),
            module: "demo".into(),
            element: "Note".into(),
        };
        let default = Catalog::prepare(vec![input.clone()]).unwrap();
        assert!(default.entity_descriptor(&default.record_by_identity(&identity).unwrap()).is_err());
        let exact = Catalog::prepare_exact_numbers(vec![input.clone()]).unwrap();
        let graph = exact.entity_descriptor(&exact.record_by_identity(&identity).unwrap()).unwrap().graph;
        assert_eq!(exact.inputs(), &[input]);
        for id in ["amount", "units"] {
            let descriptor = graph.iter().find(|d| d.identity.element == id).unwrap();
            let crate::application_model::Shape::Scalar { logical_type } = &descriptor.shape else { panic!() };
            assert_eq!(logical_type.facets, json!({}));
            let value = if id == "amount" { "9007199254740993.1234567890123456789000" } else { "900719925474099312345678901234567890" };
            crate::exact::literal(&crate::syntax::Literal { value: value.into(), kind: crate::syntax::LiteralKind::Number, span: crate::ir::Span { start: 0, end: 0 } }, logical_type).unwrap();
        }
    }

    #[test]
    fn derived_commerce_path_successor_passes_the_current_core_schema() {
        let document_json = include_str!("../../../tests/fixtures/umf010/commerce-paths.json").to_owned();
        let document: Value = serde_json::from_str(&document_json).unwrap();
        let catalog = Catalog::prepare(vec![ModuleInput {
            pin: ModelPin {
                document_id: document["id"].as_str().unwrap().to_owned(),
                revision: "commerce-paths-010".into(),
                umf_version: "0.10.0".into(),
                sha256: sha256(document_json.as_bytes()),
            },
            document_json,
            selected_module_ids: vec!["domain".into()],
        }]).expect("the adapted test source validates under current UMF core");
        assert_eq!(catalog.pins()[0].umf_version, "0.10.0");
    }
}
