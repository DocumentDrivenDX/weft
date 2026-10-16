use crate::syntax::Name;
use crate::{
    error::{Diagnostic, Result},
    ir::{Family, Identity, LogicalType, ModelPin},
    json::{checked_json, sha256},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

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
    security_inputs: bool,
    pub(crate) identity_name_fallback: bool,
    pub(crate) exact_scalars: bool,
    pub(crate) bounded_current_strings: bool,
    pub(crate) documents: Vec<Value>,
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
/// Newly admitted owning versions must not turn additional selected meaning into
/// ignored annotations. Older owning-version behavior remains unchanged.
pub(crate) fn selected08(pin: &ModelPin, element: &Value) -> Result<()> {
    if pin.umf_version != "0.8.0" { return Ok(()); }
    const KNOWN: &[&str] = &["id", "name", "title", "description", "kind",
        "scalarType", "nullability", "cardinality", "itemType", "recordType",
        "facets", "members", "keys", "references", "extensions"];
    if element.as_object().is_none_or(|m| m.keys().any(|k| !KNOWN.contains(&k.as_str())))
        || element.get("extensions").is_some_and(|e| e.as_object().is_none_or(|m| !m.is_empty()))
    { return Err(fail("WFT-TYPE", "Selected core 0.8 element contains unestablished semantics")); }
    if element["kind"] == "record" && element.get("references").and_then(Value::as_array).is_some_and(|r| !r.is_empty()) {
        return Err(fail("WFT-TYPE", "Selected core 0.8 Record reference roles are unestablished"));
    }
    for member in element.get("members").and_then(Value::as_array).into_iter().flatten() {
        reference08(pin, member, &["module", "element"])?;
    }
    if let Some(item) = element.get("itemType") { reference08(pin, item, &["module", "element"])?; }
    for reference in element.get("references").and_then(Value::as_array).into_iter().flatten() {
        reference08(pin, reference, &["module", "element", "role"])?;
    }
    Ok(())
}
pub(crate) fn reference08(pin: &ModelPin, reference: &Value, known: &[&str]) -> Result<()> {
    if pin.umf_version == "0.8.0" && reference.as_object().is_none_or(|m| m.keys().any(|k| !known.contains(&k.as_str()))) {
        return Err(fail("WFT-TYPE", "Selected core 0.8 reference contains unestablished qualifiers"));
    }
    Ok(())
}
#[jsonschema::validator(path = "../../spec/upstream/umf-0.7.0.schema.json")]
struct Envelope;
#[jsonschema::validator(path = "../../spec/upstream/umf-0.8.0.schema.json")]
struct Envelope08;
impl Catalog {
    /// New host-selected profile only; authored names always take precedence.
    pub(crate) fn query_name<'a>(&self, element: &'a Value) -> Option<&'a str> {
        element.get("name").and_then(Value::as_str).or_else(|| {
            self.identity_name_fallback.then(|| element["id"].as_str()).flatten()
        })
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
        Self::prepare_version(inputs, None)
    }
    /// Source custody only; security activation remains independently gated.
    pub fn prepare_security(inputs: Vec<ModuleInput>) -> Result<Self> {
        Self::prepare_version(inputs, Some("0.8.0"))
    }
    pub(crate) fn prepare_security_bounded_strings(inputs: Vec<ModuleInput>) -> Result<Self> {
        let mut catalog = Self::prepare_security(inputs)?;
        catalog.bounded_current_strings = true;
        Ok(catalog)
    }
    fn prepare_version(inputs: Vec<ModuleInput>, required_version: Option<&str>) -> Result<Self> {
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
            if !matches!(input.pin.umf_version.as_str(), "0.7.0" | "0.8.0")
                || doc["umf"] != input.pin.umf_version
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
            if !(if input.pin.umf_version == "0.8.0" {
                Envelope08::is_valid(&doc)
            } else { Envelope::is_valid(&doc) }) {
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
        Ok(Self { inputs, documents, exact_numbers:false, exact_scalars:false, bounded_current_strings:false, security_inputs:required_version==Some("0.8.0"), identity_name_fallback:false })
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
            for module in self.documents[document]["modules"].as_array().unwrap() {
                if module["id"] != identity.module
                    || !input.selected_module_ids.contains(&identity.module)
                {
                    continue;
                }
                for element in module["elements"].as_array().unwrap() {
                    if element["id"] == identity.element && element["kind"] == "record" {
                        selected08(&input.pin, element)?;
                        found.push(Record {
                            identity: identity.clone(),
                            pin: input.pin.clone(),
                            value: element.clone(),
                            document,
                        });
                    }
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
                    if e["kind"] == "record" && name.matches(self.query_name(e).unwrap_or("")) {
                        selected08(&input.pin, e)?;
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
        self.field_mode(record,name,self.security_inputs)
    }
    pub(crate) fn field03(&self, record:&Record,name:&Name)->Result<(Identity,LogicalType,String)> {
        self.field_mode(record,name,true)
    }
    fn field_mode(&self,record:&Record,name:&Name,optional03:bool)->Result<(Identity,LogicalType,String)> {
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
        selected08(&record.pin, field)?;
        let ideal;
        let selected=if optional03 && field["nullability"]=="absent-allowed" {
            // Derive the ideal scalar domain, leaving original selected source bytes untouched.
            ideal={let mut v=field.clone();v["nullability"]=Value::String("required".into());v};&ideal
        } else {field};
        let logical_type = scalar_type(selected, name, self.exact_numbers || record.pin.umf_version == "0.8.0", self.exact_numbers, self.bounded_current_strings)?;
        Ok((
            identity,
            logical_type,
            self.query_name(field).expect("matched query name").into(),
        ))
    }
}

pub(crate) fn scalar_type(field: &Value, name: &Name, mathematical_integer: bool, mathematical_decimal: bool, bounded_strings: bool) -> Result<LogicalType> {
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
        if !(1..=28).contains(&p) || s > p {
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
mod numeric_preparation_tests {
    use super::*;
    #[test]
    fn explicit_identity_names_preserve_precedence_ambiguity_and_original_bytes() {
        let mut doc = json!({"umf":"0.7.0","id":"owner","vocabularies":{},"extensions":{},"modules":[{"id":"m","namespace":"m","extensions":{},"elements":[{"id":"row-id","kind":"record","members":[{"module":"m","element":"field.id"},{"module":"m","element":"other-id"}],"extensions":{}},{"id":"field.id","kind":"field","scalarType":"string","nullability":"required","cardinality":"one","extensions":{}},{"id":"other-id","name":"Authored","kind":"field","scalarType":"string","nullability":"required","cardinality":"one","extensions":{}}]}]});
        let input = |v:&Value| {let raw=v.to_string();ModuleInput{document_json:raw.clone(),pin:ModelPin{document_id:"owner".into(),revision:"r1".into(),sha256:sha256(raw.as_bytes()),umf_version:"0.7.0".into()},selected_module_ids:vec!["m".into()]}};
        let name = |v:&str| Name{value:v.into(),quoted:true,span:crate::ir::Span{start:0,end:0}};
        let original=input(&doc);let mut c=Catalog::prepare(vec![original.clone()]).unwrap();
        assert!(c.record(None,&name("row-id")).is_err());c.identity_name_fallback=true;
        let record=c.record(None,&name("row-id")).unwrap();assert_eq!(record.identity.element,"row-id");
        assert_eq!(c.field(&record,&name("field.id")).unwrap().0.element,"field.id");
        assert_eq!(c.field(&record,&name("Authored")).unwrap().0.element,"other-id");
        assert!(c.field(&record,&name("other-id")).is_err());
        assert_eq!(c.record_members(&record).unwrap()[0].name,"field.id");assert_eq!(c.inputs(),&[original]);
        doc["modules"][0]["elements"][2]["name"]=json!("field.id");
        let mut c=Catalog::prepare(vec![input(&doc)]).unwrap();c.identity_name_fallback=true;let r=c.record(None,&name("row-id")).unwrap();assert!(c.field(&r,&name("field.id")).is_err());
        doc["modules"][0]["elements"][0]["name"]=json!("AuthoredRow");let mut c=Catalog::prepare(vec![input(&doc)]).unwrap();c.identity_name_fallback=true;assert!(c.record(None,&name("row-id")).is_err());assert!(c.record(None,&name("AuthoredRow")).is_ok());
    }
    #[test]
    fn exact_numeric_mode_is_explicit_and_preserves_original_source() {
        let source = r#" {"umf":"0.7.0","id":"numeric-owner","vocabularies":{},"extensions":{},"modules":[{"id":"m","namespace":"m","extensions":{},"elements":[{"id":"row","name":"Row","kind":"record","members":[{"module":"m","element":"amount"},{"module":"m","element":"units"}],"extensions":{}},{"id":"amount","name":"amount","kind":"field","scalarType":"decimal","nullability":"required","cardinality":"one","extensions":{}},{"id":"units","name":"units","kind":"field","scalarType":"integer","nullability":"required","cardinality":"one","extensions":{}}]}]} "#;
        let input=ModuleInput{document_json:source.into(),pin:ModelPin{document_id:"numeric-owner".into(),revision:"r1".into(),sha256:sha256(source.as_bytes()),umf_version:"0.7.0".into()},selected_module_ids:vec!["m".into()]};
        let owner=Identity{document_id:"numeric-owner".into(),revision:"r1".into(),module:"m".into(),element:"row".into()};
        let default=Catalog::prepare(vec![input.clone()]).unwrap();
        assert!(default.entity_descriptor(&default.record_by_identity(&owner).unwrap()).is_err());
        let exact=Catalog::prepare_exact_numbers(vec![input.clone()]).unwrap();
        let graph=exact.entity_descriptor(&exact.record_by_identity(&owner).unwrap()).unwrap().graph;
        assert_eq!(exact.inputs(),&[input]);
        for id in ["amount","units"] {
            let descriptor=graph.iter().find(|d|d.identity.element==id).unwrap();
            let crate::application_model::Shape::Scalar{logical_type}=&descriptor.shape else{panic!()};
            assert_eq!(logical_type.facets,json!({}));
            let value=if id=="amount"{"9007199254740993.1234567890123456789000"}else{"900719925474099312345678901234567890"};
            crate::exact::literal(&crate::syntax::Literal{value:value.into(),kind:crate::syntax::LiteralKind::Number,span:crate::ir::Span{start:0,end:0}},logical_type).unwrap();
        }
    }
}
