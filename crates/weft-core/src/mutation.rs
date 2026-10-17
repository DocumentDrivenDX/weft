//! Portable SQL UPDATE compiler. Host-installed owner preparation is not execution authority.
use crate::mutation_selection::Resolver;
use crate::mutation_syntax::{self};
use crate::{
    ir::ModelPin,
    json::{checked_json_bounded, sha256},
    model::{Catalog, ModuleInput},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
const BYTES: usize = 1048576;
#[derive(Debug, Clone, Serialize)]
pub struct MutationDiagnostic {
    pub code: String,
    pub phase: String,
    pub path: String,
    pub message: String,
}
pub type MResult<T> = Result<T, MutationDiagnostic>;
pub(crate) fn fail(code: &str, phase: &str, message: &str) -> MutationDiagnostic {
    MutationDiagnostic {
        code: format!("WFT-MUTATION-{code}"),
        phase: phase.into(),
        path: String::new(),
        message: message.into(),
    }
}
fn exact(value: &Value, keys: &[&str]) -> MResult<()> {
    let object = value
        .as_object()
        .ok_or_else(|| fail("INPUT", "input", "Exact object required"))?;
    if object.len() != keys.len() || keys.iter().any(|k| !object.contains_key(*k)) {
        return Err(fail("INPUT", "input", "Unknown or missing member"));
    }
    Ok(())
}
fn parse(text: &str) -> MResult<Value> {
    checked_json_bounded(text, BYTES, 65, 100000).map_err(|c| {
        fail(
            if c == "WFT-LIMIT" { "LIMIT" } else { "INPUT" },
            "input",
            "Bounded original JSON refused",
        )
    })
}
fn text<'a>(value: &'a Value) -> MResult<&'a str> {
    value
        .as_str()
        .filter(|s| !s.is_empty() && s.encode_utf16().count() <= 256)
        .ok_or_else(|| fail("INPUT", "input", "Bounded identity required"))
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Artifact {
    pub media_type: String,
    pub utf8: String,
    pub sha256: String,
}
impl Artifact {
    pub fn read(&self) -> MResult<Value> {
        if self.media_type != "application/json"
            || self.utf8.len() > BYTES
            || self.sha256 != sha256(self.utf8.as_bytes())
        {
            return Err(fail(
                "INPUT",
                "input",
                "Original artifact integrity mismatch",
            ));
        }
        parse(&self.utf8)
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActionRegistration {
    pub pin: Value,
    pub snapshot: Artifact,
    pub preparation: Artifact,
}
#[jsonschema::validator(path="../../spec/upstream/actions-plan/plan-model.schema.json",resources={"urn:umf:actions:plan:1"=>{path="../../spec/upstream/actions-plan/plan.schema.json"},"urn:umf:core:0.8.0"=>{path="../../spec/upstream/actions-plan/core.schema.json"}})]
struct PreparationSchema;
#[derive(Default)]
pub struct MutationRegistry {
    actions: Vec<(ActionRegistration, Value)>,
    selection_backends: crate::mutation_backend::SelectionRegistry,
}
impl MutationRegistry {
    pub fn new(actions: Vec<ActionRegistration>) -> MResult<Self> {
        if actions.len() > 64 {
            return Err(fail("LIMIT", "limit", "Registration bound exceeded"));
        }
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for registration in actions {
            let model = registration.snapshot.read()?;
            let prepared = registration.preparation.read()?;
            if !PreparationSchema::is_valid(&prepared) {
                return Err(fail(
                    "ACTION",
                    "action",
                    "Original owner preparation violates schema",
                ));
            }
            exact(
                &prepared,
                &[
                    "protocol",
                    "model",
                    "action",
                    "entityParameter",
                    "target",
                    "assignments",
                    "frames",
                    "producer",
                    "schema",
                    "obligations",
                ],
            )?;
            validate_model_pin(&prepared["model"])?;
            validate_action_pin(&registration.pin)?;
            if prepared["action"] != registration.pin
                || prepared["model"]["sha256"] != registration.snapshot.sha256
                || prepared["model"]["documentId"] != model["id"]
                || model["umf"] != "0.8.0"
            {
                return Err(fail(
                    "ACTION",
                    "action",
                    "Original preparation/model/action pins differ",
                ));
            }
            if prepared["schema"]["sha256"]
                != sha256(include_bytes!(
                    "../../../spec/upstream/actions-plan/plan-model.schema.json"
                ))
            {
                return Err(fail(
                    "ACTION",
                    "action",
                    "Original preparation schema pin differs",
                ));
            }
            exact(&prepared["producer"], &["id", "version", "build"])?;
            let owner: Value = serde_json::from_str(include_str!(
                "../../../spec/upstream/actions-plan/owner-pins.json"
            ))
            .expect("pinned build resource");
            let local = registration.pin["invocation"] == json!({"id":"umf.actions.local-tx","version":"1"});
            let local_owner: Value = serde_json::from_str(include_str!(
                "../../../spec/upstream/actions-plan-local/owner-pins.json"
            )).expect("pinned local owner resource");
            let expected_owner = if local { &local_owner } else { &owner };
            let current_owner: Value = serde_json::from_str(include_str!(
                "../../../spec/upstream/actions-plan-current/owner-pins.json"
            )).expect("checked current original producer resource");
            if prepared["producer"] != expected_owner["producer"]
                && prepared["producer"] != current_owner["producer"] {
                return Err(fail(
                    "ACTION",
                    "action",
                    "Owner preparation build is not pinned by this compiler",
                ));
            }
            validate_local_bundle(&model, &registration.pin, local)?;
            exact(&prepared["schema"], &["id", "sha256"])?;
            text(&prepared["entityParameter"])?;
            validate_ref(&prepared["target"], true)?;
            let bindings = prepared["assignments"]
                .as_array()
                .ok_or_else(|| fail("ACTION", "action", "Bindings missing"))?;
            let mut fields = BTreeSet::new();
            let mut inputs = BTreeSet::new();
            for binding in bindings {
                exact(binding, &["input", "field"])?;
                text(&binding["input"])?;
                validate_ref(&binding["field"], false)?;
                if !inputs.insert(canonical(&binding["input"]))
                    || !fields.insert(canonical(&binding["field"]))
                {
                    return Err(fail("ACTION", "action", "Repeated original binding"));
                }
            }
            let frames = prepared["frames"].as_array().unwrap();
            if frames.iter().any(|f| text(f).is_err())
                || frames.iter().map(canonical).collect::<BTreeSet<_>>().len() != frames.len()
            {
                return Err(fail("ACTION", "action", "Invalid original frames"));
            }
            let obligations = prepared["obligations"].as_array().unwrap();
            if obligations
                .iter()
                .any(|o| exact(o, &["id", "status"]).is_err() || o["status"] != "checked")
                || obligations
                    .iter()
                    .map(|o| canonical(&o["id"]))
                    .collect::<BTreeSet<_>>()
                    .len()
                    != obligations.len()
            {
                return Err(fail(
                    "ACTION",
                    "action",
                    "Invalid original checked obligations",
                ));
            }
            if !seen.insert(canonical(&registration.pin)) {
                return Err(fail("ACTION", "action", "Repeated registration"));
            }
            out.push((registration, prepared));
        }
        Ok(Self { actions: out, selection_backends: Default::default() })
    }
    pub fn from_json(configuration: &str) -> MResult<Self> {
        let value = parse(configuration)?;
        exact(&value, &["actions", "selectionBackends"])?;
        if value["selectionBackends"] != json!([]) {
            return Err(fail(
                "INPUT",
                "input",
                "Selection backend code must be host-installed",
            ));
        }
        let actions = serde_json::from_value(value["actions"].clone())
            .map_err(|_| fail("INPUT", "input", "Original action registration required"))?;
        Self::new(actions)
    }
    /// Host code supplies executable backends; JSON configuration cannot install them.
    pub fn install_selection_backend(&mut self,backend:Box<dyn crate::mutation_backend::SelectionBackend>)->MResult<usize>{self.selection_backends.install(backend)}
    pub fn selection_capabilities(&self,handle:usize)->MResult<crate::mutation_backend::SelectionCapabilities>{self.selection_backends.capabilities(handle)}
    pub fn lower_selection(&self,handle:usize,selection:&Artifact,model:&Artifact,binding:&Artifact,max_items:u32)->MResult<crate::mutation_backend::SelectionLowering>{
        // Only original owner-inspected model bytes installed by this host are admitted.
        let selected = selection.read()?;
        if !self.actions.iter().any(|(registration,prepared)|registration.snapshot==*model && prepared["target"]==selected["key"]){return Err(fail("ACTION","action","Owner-prepared model not installed"));}
        self.selection_backends.lower(handle,selection,model,binding,max_items)
    }
    pub fn compile_json(&self, request: &str) -> String {
        let result = self.compile(request);
        canonical(&match result {
            Ok(value) => value,
            Err(diagnostic) => {
                json!({"interfaceVersion":"weft-mutation-compile/0.1.0","status":"blocked","diagnostics":[diagnostic]})
            }
        })
    }
    fn compile(&self, source: &str) -> MResult<Value> {
        let request = parse(source)?;
        exact(
            &request,
            &[
                "interfaceVersion",
                "dialect",
                "sql",
                "parameters",
                "model",
                "modelPin",
                "action",
                "target",
                "selectionProfile",
                "batchSize",
                "limits",
            ],
        )?;
        if request["interfaceVersion"] != "weft-mutation-compile/0.1.0"
            || request["dialect"] != "weft-mutation/0.1.0"
            || request["selectionProfile"] != json!({"id":"weft.update-selection","version":"1"})
        {
            return Err(fail(
                "INPUT",
                "input",
                "Exact mutation interface/profile required",
            ));
        }
        validate_model_pin(&request["modelPin"])?;
        validate_action_pin(&request["action"])?;
        validate_ref(&request["target"], true)?;
        let artifact: Artifact = serde_json::from_value(request["model"].clone())
            .map_err(|_| fail("INPUT", "input", "Original model artifact required"))?;
        let document = artifact.read()?;
        if request["modelPin"]["sha256"] != artifact.sha256
            || request["action"]["documentSha256"] != artifact.sha256
            || request["modelPin"]["documentId"] != document["id"]
        {
            return Err(fail("INPUT", "input", "Original source pins differ"));
        }
        let (registration, prepared) = self
            .actions
            .iter()
            .find(|(r, _)| r.pin == request["action"])
            .ok_or_else(|| fail("ACTION", "action", "Exact host-installed action missing"))?;
        if registration.snapshot != artifact
            || prepared["model"] != request["modelPin"]
            || prepared["target"] != request["target"]
        {
            return Err(fail(
                "ACTION",
                "action",
                "Original installed preparation/source differs",
            ));
        }
        let batch = unsigned(&request["batchSize"])?;
        exact(&request["limits"], &["maxItems", "maxManifestBytes"])?;
        let max_items = unsigned(&request["limits"]["maxItems"])?;
        let max_bytes = unsigned(&request["limits"]["maxManifestBytes"])?;
        if !(1..=100).contains(&batch)
            || !(1..=10000).contains(&max_items)
            || !(1..=16777216).contains(&max_bytes)
        {
            return Err(fail("LIMIT", "limit", "Plan policy bound exceeded"));
        }
        let sql = request["sql"]
            .as_str()
            .ok_or_else(|| fail("INPUT", "input", "SQL text required"))?;
        let update = mutation_syntax::parse(sql)?;
        let parameters = request["parameters"]
            .as_object()
            .ok_or_else(|| fail("INPUT", "input", "Exact parameter map required"))?;
        if parameters.len() > 64 {
            return Err(fail("LIMIT", "limit", "Parameter bound exceeded"));
        }
        let modules = document["modules"]
            .as_array()
            .ok_or_else(|| fail("INPUT", "input", "Original modules required"))?
            .iter()
            .map(|m| m["id"].as_str().unwrap_or("").to_string())
            .collect();
        let pin: ModelPin = serde_json::from_value(request["modelPin"].clone())
            .map_err(|_| fail("INPUT", "input", "Model pin required"))?;
        let catalog = Catalog::prepare_security(vec![ModuleInput {
            document_json: artifact.utf8.clone(),
            pin,
            selected_module_ids: modules,
        }])
        .map_err(|_| fail("INPUT", "input", "Original core document refused"))?;
        let record = catalog
            .record(update.namespace.as_ref(), &update.record)
            .map_err(|_| fail("RESOLVE", "resolve", "Record name missing or ambiguous"))?;
        if request["target"]["module"] != record.identity.module
            || request["target"]["element"] != record.identity.element
        {
            return Err(fail(
                "ACTION",
                "action",
                "SQL Record differs from original action target",
            ));
        }
        let mut resolver = Resolver {
            catalog: &catalog,
            record: &record,
            parameters,
            used: BTreeMap::new(),
            dependencies: Vec::new(),
        };
        let mut assignments = Vec::new();
        let mut provenance = Vec::new();
        let mut assigned = BTreeSet::new();
        for (ordinal, (name, input)) in update.assignments.iter().enumerate() {
            let (reference, field) = resolver.field(name)?;
            if !assigned.insert(canonical(&reference)) {
                return Err(fail("ACTION", "action", "Repeated SET Field"));
            }
            let binding = prepared["assignments"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["field"] == reference)
                .ok_or_else(|| {
                    fail(
                        "ACTION",
                        "action",
                        "SET Field outside installed action bindings",
                    )
                })?;
            let value = resolver.value(input, &reference, &field)?;
            assignments.push(json!({"input":binding["input"],"field":reference,"value":value}));
            provenance.push(json!({"ordinal":ordinal,"field":reference,"input":binding["input"]}));
        }
        if assignments.len() != prepared["assignments"].as_array().unwrap().len() {
            return Err(fail(
                "ACTION",
                "action",
                "Complete original action binding required",
            ));
        }
        let predicate = resolver.predicate(&update.predicate, 1)?;
        if resolver.used.len() != parameters.len() {
            return Err(fail("TYPE", "type", "Unused parameter refused"));
        }
        let selection = generated(
            json!({"protocol":"weft.update-selection/1","record":{"module":record.identity.module,"element":record.identity.element},"key":request["target"],"predicate":predicate}),
        )?;
        let source_envelope =
            generated(json!({"dialect":request["dialect"],"sql":sql,"parameters":parameters}))?;
        let parameter_artifact = generated(request["parameters"].clone())?;
        let plan = generated(
            json!({"protocol":"umf.actions.plan/1","model":request["modelPin"],"action":request["action"],"target":request["target"],"source":source_envelope,"selection":{"profile":request["selectionProfile"],"artifact":selection,"dependencies":resolver.dependencies},"inputs":{"entityParameter":prepared["entityParameter"],"assignments":assignments},"execution":{"selection":"frozen","atomicity":"batch","conflict":"stop","partition":"manifest-order","batchSize":batch},"limits":{"maxItems":max_items,"maxManifestBytes":max_bytes}}),
        )?;
        Ok(
            json!({"interfaceVersion":"weft-mutation-compile/0.1.0","status":"compiled","plan":plan,"provenance":{"sourceSha256":source_envelope.sha256,"parameters":parameter_artifact,"entityParameter":prepared["entityParameter"],"action":request["action"],"assignments":provenance},"diagnostics":[]}),
        )
    }
}
fn unsigned(value: &Value) -> MResult<u64> {
    crate::security_literals::unsigned(value).map_err(|_| {
        fail(
            "INPUT",
            "input",
            "Safe unsigned structural integer required",
        )
    })
}
fn validate_model_pin(pin: &Value) -> MResult<()> {
    exact(pin, &["documentId", "revision", "umfVersion", "sha256"])?;
    text(&pin["documentId"])?;
    text(&pin["revision"])?;
    if pin["umfVersion"] != "0.8.0" || !digest(&pin["sha256"]) {
        return Err(fail("INPUT", "input", "Original model pin refused"));
    }
    Ok(())
}
fn validate_action_pin(pin: &Value) -> MResult<()> {
    exact(pin, &["identity", "documentSha256", "invocation"])?;
    exact(&pin["identity"], &["module", "action"])?;
    text(&pin["identity"]["module"])?;
    text(&pin["identity"]["action"])?;
    if !digest(&pin["documentSha256"])
        || (pin["invocation"] != json!({"id":"umf.actions.tx","version":"1"})
            && pin["invocation"] != json!({"id":"umf.actions.local-tx","version":"1"}))
    {
        return Err(fail("INPUT", "input", "Original action pin refused"));
    }
    Ok(())
}
// Declaration correspondence only. No capability or execution authority is issued here.
fn validate_local_bundle(model: &Value, pin: &Value, local: bool) -> MResult<()> {
    let action = model["modules"].as_array().into_iter().flatten()
        .filter(|m| m["id"] == pin["identity"]["module"])
        .flat_map(|m| m["extensions"]["umf.actions"]["actions"].as_array().into_iter().flatten())
        .filter(|a| a["id"] == pin["identity"]["action"]).collect::<Vec<_>>();
    if action.len() != 1 { return Err(fail("ACTION","action","Unique original local action required")); }
    let a = action[0];
    if !local {
        if a["attribution"]["profile"]["id"] == "umf.actions.local-owner"
            || a["authorization"]["profile"]["id"] == "umf.actions.local-store-owner" {
            return Err(fail("ACTION","action","Local source declarations cannot select native invocation"));
        }
        return Ok(());
    }
    let reads = a["reads"].as_array().ok_or_else(|| fail("ACTION","action","Original read frames required"))?;
    let writes = a["writes"].as_array().ok_or_else(|| fail("ACTION","action","Original write frames required"))?;
    let resources: Vec<Value> = reads.iter().chain(writes).map(|f| f["id"].clone()).collect();
    if a["attribution"] != json!({"subject":"actor-profile","profile":{"id":"umf.actions.local-owner","version":"1"}})
        || a["authorization"] != json!({"kind":"policy","profile":{"id":"umf.actions.local-store-owner","version":"1"},"resources":resources}) {
        return Err(fail("ACTION","action","Complete original local actor/policy bundle required"));
    }
    Ok(())
}
pub(crate) fn validate_ref(reference: &Value, key: bool) -> MResult<()> {
    exact(
        reference,
        if key {
            &["module", "element", "key"]
        } else {
            &["module", "element"]
        },
    )?;
    for value in reference.as_object().unwrap().values() {
        text(value)?;
    }
    Ok(())
}
fn digest(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}
pub fn canonical(value: &Value) -> String {
    match value {
        Value::Object(object) => {
            let mut entries: Vec<_> = object.iter().collect();
            entries.sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));
            format!(
                "{{{}}}",
                entries
                    .iter()
                    .map(|(k, v)| format!("{}:{}", serde_json::to_string(k).unwrap(), canonical(v)))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(canonical).collect::<Vec<_>>().join(",")
        ),
        _ => serde_json::to_string(value).unwrap(),
    }
}
fn generated(value: Value) -> MResult<Artifact> {
    let utf8 = canonical(&value);
    parse(&utf8)?;
    Ok(Artifact {
        media_type: "application/json".into(),
        sha256: sha256(utf8.as_bytes()),
        utf8,
    })
}

#[cfg(test)]
mod current_preparation_tests {
    use super::*;
    #[test]
    fn current_preparations_compile_and_preserve_compile_boundary_custody() {
        for (raw,request) in [
            (include_str!("../../../spec/upstream/actions-plan-current/standard-registration.json"),include_str!("../../../spec/upstream/actions-plan-current/standard-request.json")),
            (include_str!("../../../spec/upstream/actions-plan-current/local-registration.json"),include_str!("../../../spec/upstream/actions-plan-current/local-request.json"))] {
            let registration:ActionRegistration=serde_json::from_str(raw).unwrap();
            let registry=MutationRegistry::new(vec![registration.clone()]).unwrap();
            let request:Value=serde_json::from_str(request).unwrap();
            let response:Value=serde_json::from_str(&registry.compile_json(&request.to_string())).unwrap();assert_eq!(response["status"],"compiled","{response}");
            let mut altered=registration.clone();let mut prepared=altered.preparation.read().unwrap();prepared["target"]["element"]=json!("foreign-original-target");altered.preparation=generated(prepared).unwrap();
            let registry=MutationRegistry::new(vec![altered]).unwrap();
            let response:Value=serde_json::from_str(&registry.compile_json(&request.to_string())).unwrap();assert_eq!(response["status"],"blocked","{response}");assert!(response.get("plan").is_none());
            for mode in 0..3 {let mut altered=request.clone();match mode {0=>altered["model"]["sha256"]=json!("0".repeat(64)),1=>altered["modelPin"]["revision"]=json!("foreign"),_=>altered["action"]["identity"]["action"]=json!("foreign")};
                let response:Value=serde_json::from_str(&MutationRegistry::new(vec![registration.clone()]).unwrap().compile_json(&altered.to_string())).unwrap();assert_eq!(response["status"],"blocked","{mode}: {response}");assert!(response.get("plan").is_none());}
        }
    }
    #[test]
    fn checked_original_current_standard_and_local_preparations() {
        for raw in [include_str!("../../../spec/upstream/actions-plan-current/standard-registration.json"), include_str!("../../../spec/upstream/actions-plan-current/local-registration.json")] {
            let registration:ActionRegistration=serde_json::from_str(raw).unwrap();
            assert!(MutationRegistry::new(vec![registration.clone()]).is_ok());
            for build in ["sha256:ea8280583fe9d7757fbf8df21d1753eaa787857371b9366858e28c9a113b98a1", "sha256:0000000000000000000000000000000000000000000000000000000000000000"] {
                let mut altered=registration.clone();let mut prepared=altered.preparation.read().unwrap();
                prepared["producer"]["build"]=json!(build);altered.preparation=generated(prepared).unwrap();
                assert!(MutationRegistry::new(vec![altered]).is_err());
            }
            let mut altered=registration.clone();let mut prepared=altered.preparation.read().unwrap();
            prepared["action"]["identity"]["action"]=json!("foreign-original-action");altered.preparation=generated(prepared).unwrap();
            assert!(MutationRegistry::new(vec![altered]).is_err());
        }
    }
}
