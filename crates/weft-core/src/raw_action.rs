//! Source-bound scalar graph-write admission. No native SQL or execution authority.
use crate::{
    ir::ModelPin,
    json::{checked_json, sha256},
    model::{Catalog, ModuleInput},
    mutation::Artifact,
    raw_read::{Binding, Carrier},
    security_ontology::SecurityOntologyClosure,
    security_source::SecuritySourcePacket,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
pub type ActionResult<T> = Result<T, &'static str>;
#[jsonschema::validator(path = "../../spec/upstream/sdk-actions/actions.schema.json")]
struct ActionsSchema;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct InvocationProfile {
    pub id: String,
    pub version: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ActionSource {
    pub document_id: String,
    pub core_version: String,
    pub module: String,
    pub action: String,
    pub document_sha256: String,
    pub action_pointer: String,
    pub invocation: InvocationProfile,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SecuritySources {
    pub policy: Artifact,
    pub ontology: Artifact,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ActionRequest {
    pub version: String,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub verification_profile: Option<String>,
    pub target: String,
    pub binding: Binding,
    pub source: ActionSource,
    pub role_mappings: BTreeMap<String, String>,
    pub session_principal: String,
    pub security: SecuritySources,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionField {
    pub reference: Value,
    pub table: Vec<String>,
    pub logical: String,
    pub carrier: Carrier,
    pub nullable: bool,
    pub declaration: Value,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionKey {
    pub reference: Value,
    pub table: Vec<String>,
    pub fields: Vec<ActionField>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionEntity {
    pub key: ActionKey,
    pub components: Vec<Value>,
    pub required_existing: bool,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionFrame {
    pub source: Value,
    pub entity: ActionEntity,
    pub fields: Vec<ActionField>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionEffect {
    pub id: String,
    pub kind: String,
    pub entity: ActionEntity,
    pub values: Vec<(ActionField, Value)>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RuleNode {
    Literal {
        value: Value,
    },
    Input {
        id: String,
    },
    State {
        phase: String,
        frame: String,
        field: ActionField,
    },
    Exists {
        phase: String,
        frame: String,
    },
    Operation {
        operator: String,
        args: Vec<RuleExpression>,
    },
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleExpression {
    pub family: String,
    pub nullable: bool,
    pub node: RuleNode,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionCondition {
    pub id: String,
    pub code: String,
    pub retryable: bool,
    pub source: Value,
    pub expression: RuleExpression,
}
#[derive(Debug)]
pub struct AdmittedAction {
    request: ActionRequest,
    action: Value,
    fields: BTreeMap<String, ActionField>,
    entities: BTreeMap<String, ActionEntity>,
    frames: BTreeMap<String, ActionFrame>,
    effects: Vec<ActionEffect>,
    preconditions: Vec<ActionCondition>,
    postconditions: Vec<ActionCondition>,
}
impl AdmittedAction {
    pub fn request(&self) -> &ActionRequest {
        &self.request
    }
    pub fn action(&self) -> &Value {
        &self.action
    }
    pub fn fields(&self) -> &BTreeMap<String, ActionField> {
        &self.fields
    }
    pub fn entities(&self) -> &BTreeMap<String, ActionEntity> {
        &self.entities
    }
    pub fn frames(&self) -> &BTreeMap<String, ActionFrame> {
        &self.frames
    }
    pub fn effects(&self) -> &Vec<ActionEffect> {
        &self.effects
    }
    pub fn preconditions(&self) -> &Vec<ActionCondition> {
        &self.preconditions
    }
    pub fn postconditions(&self) -> &Vec<ActionCondition> {
        &self.postconditions
    }
}
fn object(v: &Value, required: &[&str], optional: &[&str]) -> ActionResult<()> {
    let o = v.as_object().ok_or("exact action object required")?;
    if required.iter().any(|k| !o.contains_key(*k))
        || o.keys()
            .any(|k| !required.contains(&k.as_str()) && !optional.contains(&k.as_str()))
    {
        return Err("unknown or missing selected action meaning");
    }
    Ok(())
}
fn array(v: &Value) -> ActionResult<&Vec<Value>> {
    v.as_array().ok_or("action array required")
}
fn text(v: &Value) -> ActionResult<&str> {
    v.as_str()
        .filter(|s| !s.is_empty() && s.len() <= 256)
        .ok_or("bounded action identity required")
}
fn ref_id(r: &Value) -> ActionResult<String> {
    object(r, &["module", "element"], &[])?;
    text(&r["module"])?;
    text(&r["element"])?;
    Ok(crate::mutation::canonical(&json!([
        r["module"],
        r["element"]
    ])))
}
fn locate<'a>(doc: &'a Value, r: &Value) -> ActionResult<(&'a Value, &'a Value)> {
    let m = array(&doc["modules"])?
        .iter()
        .find(|m| m["id"] == r["module"])
        .ok_or("action module reference absent")?;
    let e = array(&m["elements"])?
        .iter()
        .find(|e| e["id"] == r["element"])
        .ok_or("action element reference absent")?;
    Ok((m, e))
}
fn field(doc: &Value, binding: &Binding, reference: &Value) -> ActionResult<ActionField> {
    ref_id(reference)?;
    let (_, f) = locate(doc, reference)?;
    if crate::mutation::canonical(f).len() > 4096 {
        return Err("native action Field metadata budget");
    }
    let carrier = match f["scalarType"].as_str() {
        Some("string") => Carrier::String,
        Some("boolean") => Carrier::Boolean,
        Some("integer") => Carrier::Integer,
        Some("decimal") => Carrier::Decimal,
        _ => return Err("native scalar action carrier unsupported"),
    };
    crate::security_literals::validate_field(f)
        .map_err(|_| "native action Field constraints unsupported")?;
    if f["cardinality"] != "one"
        || f["extensions"].as_object().is_none_or(|o| !o.is_empty())
        || !matches!(
            f["nullability"].as_str(),
            Some("required" | "absent-allowed")
        )
    {
        return Err("native action Field meaning unsupported");
    }
    let mut found = vec![];
    for table in &binding.tables {
        let m = array(&doc["modules"])?
            .iter()
            .find(|m| m["namespace"] == table.logical[0])
            .ok_or("bound module absent")?;
        let record = array(&m["elements"])?
            .iter()
            .find(|e| e["name"] == table.logical[1] && e["kind"] == "record")
            .ok_or("bound Record absent")?;
        if array(&record["members"])?.contains(reference) {
            for column in &table.columns {
                if f["name"] == column.logical {
                    found.push((table, column));
                }
            }
        }
    }
    if found.len() != 1 {
        return Err("action Field requires one exact bound owner");
    }
    let (table, column) = found[0];
    if column.carrier != carrier || column.nullable != (f["nullability"] == "absent-allowed") {
        return Err("action Field/native domain mismatch");
    }
    Ok(ActionField {
        reference: reference.clone(),
        table: table.logical.clone(),
        logical: column.logical.clone(),
        carrier,
        nullable: column.nullable,
        declaration: f.clone(),
    })
}
fn key(doc: &Value, binding: &Binding, reference: &Value) -> ActionResult<ActionKey> {
    object(reference, &["module", "element", "key"], &[])?;
    let (m, r) = locate(doc, reference)?;
    if r["kind"] != "record" {
        return Err("action Key needs Record");
    }
    object(
        r,
        &["id", "kind", "members", "keys", "extensions"],
        &["name", "description", "title", "aliases"],
    )?;
    if r["extensions"].as_object().is_none_or(|o| !o.is_empty()) {
        return Err("selected Record extension unsupported");
    }
    let keys: Vec<_> = array(&r["keys"])?
        .iter()
        .filter(|k| k["id"] == reference["key"])
        .collect();
    if keys.len() != 1 {
        return Err("exact action Key absent");
    };
    let fields: Vec<_> = array(&keys[0]["fields"])?
        .iter()
        .map(|f| field(doc, binding, f))
        .collect::<ActionResult<_>>()?;
    let logical = vec![text(&m["namespace"])?.into(), text(&r["name"])?.into()];
    let table = binding
        .tables
        .iter()
        .find(|t| t.logical == logical)
        .ok_or("action Record binding absent")?;
    if array(&r["members"])?.len() > 256 {
        return Err("native action Record width budget");
    }
    if table.columns.len() != array(&r["members"])?.len() {
        return Err("native actions require complete scalar Record inventory");
    }
    for member in array(&r["members"])? {
        field(doc, binding, member)?;
    }
    if fields.is_empty()
        || fields.iter().any(|f| f.table != logical || f.nullable)
        || fields.iter().map(|f| f.logical.clone()).collect::<Vec<_>>() != table.key
    {
        return Err("action Key/binding identity differs");
    }
    Ok(ActionKey {
        reference: reference.clone(),
        table: logical,
        fields,
    })
}
fn literal(v: &Value, f: &ActionField) -> ActionResult<()> {
    crate::security_literals::check_literal(&f.declaration, v)
        .map_err(|_| "action literal violates exact Field")
}
fn value(
    v: &Value,
    f: &ActionField,
    parameters: &BTreeMap<String, Value>,
    fields: &BTreeMap<String, ActionField>,
) -> ActionResult<Value> {
    if v.get("parameter").is_some() {
        object(v, &["parameter"], &[])?;
        let id = text(&v["parameter"])?;
        let p = parameters
            .get(id)
            .ok_or("action assignment parameter absent")?;
        if p["kind"] != "value"
            || p["required"] != true
            || ref_id(&p["field"])? != ref_id(&f.reference)?
        {
            return Err("action assignment requires exact Field parameter");
        };
        Ok(json!({"input":id}))
    } else {
        object(v, &["literal"], &[])?;
        literal(&v["literal"], f)?;
        let _ = fields;
        Ok(v.clone())
    }
}

fn family(field: &ActionField) -> ActionResult<&'static str> {
    match field.carrier {
        Carrier::Boolean => Ok("boolean"),
        Carrier::String => Ok("string"),
        Carrier::Integer => {
            let width = &field.declaration["facets"]["integerWidth"];
            if width["bits"].as_u64().is_none_or(|n| n > 64 || n == 0) {
                return Err("action rule integer domain must be bounded to 64 bits");
            };
            Ok("integer")
        }
        _ => Err("rules/1 supports boolean/string/integer operands"),
    }
}
fn condition_rules(
    phase: &str,
    action: &Value,
    parameters: &BTreeMap<String, Value>,
    fields: &BTreeMap<String, ActionField>,
    frames: &BTreeMap<String, ActionFrame>,
) -> ActionResult<Vec<ActionCondition>> {
    let mut failures = BTreeMap::new();
    for failure in array(&action["failures"])? {
        object(failure, &["code", "message", "retryable"], &[])?;
        if failures
            .insert(
                text(&failure["code"])?,
                failure["retryable"]
                    .as_bool()
                    .ok_or("failure retryable required")?,
            )
            .is_some()
        {
            return Err("duplicate action failure");
        }
    }
    let mut out = vec![];
    let mut identities = BTreeSet::new();
    let mut total_nodes = 0usize;
    for condition in array(
        &action[if phase == "pre" {
            "preconditions"
        } else {
            "postconditions"
        }],
    )? {
        object(condition, &["id", "rule", "failure"], &[])?;
        object(&condition["failure"], &["code", "message"], &[])?;
        let id = text(&condition["id"])?;
        if !identities.insert(id) {
            return Err("duplicate condition");
        };
        let code = text(&condition["failure"]["code"])?;
        let retryable = *failures
            .get(code)
            .ok_or("condition failure code must be declared")?;
        let rule = &condition["rule"];
        object(
            rule,
            &["language", "version", "expression", "references"],
            &[],
        )?;
        if rule["language"] != "umf.actions.rules" || rule["version"] != "1" {
            return Err("native condition profile unsupported");
        }
        let refs = array(&rule["references"])?;
        for reference in refs {
            let o = reference
                .as_object()
                .ok_or("condition reference required")?;
            if o.len() != 1 {
                return Err("condition reference shape");
            };
            if let Some(id) = reference.get("parameter") {
                if !parameters.contains_key(text(id)?) {
                    return Err("condition parameter absent");
                }
            } else if let Some(record) = reference.get("record") {
                ref_id(record)?;
                if !frames.values().any(|f| f.source["record"] == *record) {
                    return Err("condition Record absent");
                }
            } else {
                return Err("native condition dependency unsupported");
            }
        }
        struct Context<'a> {
            phase: &'a str,
            refs: &'a Vec<Value>,
            parameters: &'a BTreeMap<String, Value>,
            fields: &'a BTreeMap<String, ActionField>,
            frames: &'a BTreeMap<String, ActionFrame>,
            reads: &'a Vec<Value>,
            nodes: usize,
        }
        impl Context<'_> {
            fn dependency(&self, reference: Value) -> ActionResult<()> {
                if !self.refs.contains(&reference) {
                    return Err("condition dependency undeclared");
                };
                Ok(())
            }
            fn frame(&self, phase: &str, id: &str) -> ActionResult<&ActionFrame> {
                if !matches!(phase, "pre" | "post") || self.phase == "pre" && phase == "post" {
                    return Err("condition state phase unsupported");
                };
                let f = self.frames.get(id).ok_or("condition frame absent")?;
                if !self.reads.contains(&f.source) {
                    return Err("condition requires declared read frame");
                };
                self.dependency(json!({"record":f.source["record"]}))?;
                Ok(f)
            }
            fn parse(&mut self, v: &Value, depth: usize) -> ActionResult<RuleExpression> {
                self.nodes += 1;
                if depth > 32 || self.nodes > 512 {
                    return Err("condition expression budget");
                }
                let result = if v.get("literal").is_some() {
                    object(v, &["literal"], &[])?;
                    let literal = &v["literal"];
                    let kind = if literal.is_null() {
                        "null"
                    } else {
                        let o = literal
                            .as_object()
                            .ok_or("condition literal object required")?;
                        if o.len() != 1 {
                            return Err("condition literal shape");
                        };
                        if literal.get("string").is_some() {
                            "string"
                        } else if literal.get("boolean").is_some() {
                            "boolean"
                        } else if literal.get("integerToken").is_some() {
                            "integer"
                        } else {
                            return Err("rules/1 literal family unsupported");
                        }
                    };
                    if kind != "null" {
                        let field = json!({"kind":"field","scalarType":kind,"cardinality":"one","nullability":"required","extensions":{},"facets":if kind=="integer"{json!({"integerWidth":{"bits":64,"signed":true}})}else{json!({})}});
                        crate::security_literals::check_literal(&field, literal)
                            .map_err(|_| "condition literal domain refused")?;
                    }
                    RuleExpression {
                        family: kind.into(),
                        nullable: kind == "null",
                        node: RuleNode::Literal {
                            value: literal.clone(),
                        },
                    }
                } else if let Some(id) = v.get("input") {
                    object(v, &["input"], &[])?;
                    let id = text(id)?;
                    self.dependency(json!({"parameter":id}))?;
                    let p = self.parameters.get(id).ok_or("rule input absent")?;
                    if p["kind"] != "value" {
                        return Err("rule scalar input required");
                    };
                    let f = self
                        .fields
                        .get(&ref_id(&p["field"])?)
                        .ok_or("rule input Field absent")?;
                    RuleExpression {
                        family: family(f)?.into(),
                        nullable: f.nullable,
                        node: RuleNode::Input { id: id.into() },
                    }
                } else if v.get("state").is_some() {
                    object(v, &["state", "frame", "field"], &[])?;
                    let phase = text(&v["state"])?;
                    let id = text(&v["frame"])?;
                    let frame = self.frame(phase, id)?;
                    let field = frame
                        .fields
                        .iter()
                        .find(|f| f.reference == v["field"])
                        .ok_or("condition field outside read frame")?;
                    RuleExpression {
                        family: family(field)?.into(),
                        nullable: field.nullable,
                        node: RuleNode::State {
                            phase: phase.into(),
                            frame: id.into(),
                            field: field.clone(),
                        },
                    }
                } else if let Some(exists) = v.get("exists") {
                    object(v, &["exists"], &[])?;
                    object(exists, &["state", "frame"], &[])?;
                    let phase = text(&exists["state"])?;
                    let id = text(&exists["frame"])?;
                    self.frame(phase, id)?;
                    RuleExpression {
                        family: "boolean".into(),
                        nullable: false,
                        node: RuleNode::Exists {
                            phase: phase.into(),
                            frame: id.into(),
                        },
                    }
                } else {
                    object(v, &["op", "args"], &[])?;
                    let op = text(&v["op"])?;
                    let args = array(&v["args"])?;
                    let unary = matches!(op, "present" | "not");
                    if args.len() != if unary { 1 } else { 2 } {
                        return Err("condition operator arity");
                    };
                    let args = args
                        .iter()
                        .map(|a| self.parse(a, depth + 1))
                        .collect::<ActionResult<Vec<_>>>()?;
                    let output = match op {
                        "eq" => {
                            if args[0].family != args[1].family
                                && !(args[0].family == "null" && args[1].nullable
                                    || args[1].family == "null" && args[0].nullable)
                            {
                                return Err("condition equality family differs");
                            };
                            "boolean"
                        }
                        "lt" | "le" | "gt" | "ge" | "add" | "sub" => {
                            if args.iter().any(|a| a.family != "integer") {
                                return Err("integer rule operand required");
                            };
                            if matches!(op, "add" | "sub") {
                                "integer"
                            } else {
                                "boolean"
                            }
                        }
                        "and" | "or" | "not" => {
                            if args.iter().any(|a| a.family != "boolean") {
                                return Err("boolean rule operand required");
                            };
                            "boolean"
                        }
                        "present" => "boolean",
                        _ => return Err("condition operator unsupported"),
                    };
                    RuleExpression {
                        family: output.into(),
                        nullable: false,
                        node: RuleNode::Operation {
                            operator: op.into(),
                            args,
                        },
                    }
                };
                Ok(result)
            }
        }
        let mut context = Context {
            phase,
            refs,
            parameters,
            fields,
            frames,
            reads: array(&action["reads"])?,
            nodes: 0,
        };
        let ast = checked_json(
            rule["expression"]
                .as_str()
                .ok_or("rule expression required")?,
        )
        .map_err(|_| "rule expression JSON refused")?;
        let expression = context.parse(&ast, 1)?;
        total_nodes += context.nodes;
        if total_nodes > 1024 {
            return Err("native action aggregate rule node budget");
        };
        if expression.family != "boolean" {
            return Err("boolean condition required");
        }
        out.push(ActionCondition {
            id: id.into(),
            code: code.into(),
            retryable,
            source: condition.clone(),
            expression,
        });
    }
    Ok(out)
}
pub fn admit_json(raw: &str) -> ActionResult<AdmittedAction> {
    let parsed = checked_json(raw).map_err(|_| "bounded action request JSON refused")?;
    let request: ActionRequest =
        serde_json::from_value(parsed).map_err(|_| "exact native-action request required")?;
    crate::raw_read::validate_catalog_fence(&request.binding,&request.target)?;
    if let Some(profile)=&request.verification_profile {
        if profile!="source-exact/0.1"||!matches!(request.target.as_str(),"postgresql18"|"lakebase18")||request.binding.native_columns.is_none(){return Err("source-exact action verification requires original PG native facts")}
        match &request.binding.schema_binding {Some(crate::raw_read::SchemaBinding::Qualified) if request.binding.schema.is_none()=>return Err("source-exact qualified action requires explicit schema"),Some(crate::raw_read::SchemaBinding::SearchPath) if request.binding.schema.is_some()=>return Err("source-exact search-path action forbids schema override"),_=>{}}
    }
    if request.version != "weft.native-action/0.1"
        || !matches!(request.target.as_str(), "postgresql18" | "lakebase18" | "spark-delta4" | "databricks-managed-delta")
        || request.session_principal != "human"
    {
        return Err("NATIVE_PROFILE_UNSUPPORTED");
    }
    crate::raw_read::validate_model(&request.binding)?;
    let doc =
        checked_json(&request.binding.model_json).map_err(|_| "original model JSON refused")?;
    if request.source.document_sha256 != request.binding.model_sha256
        || request.source.document_id != doc["id"]
        || request.source.core_version != "0.8.0"
        || request.source.invocation.id != "umf.actions.tx"
        || request.source.invocation.version != "1"
    {
        return Err("exact original action source/invocation pin required");
    }
    if doc["vocabularies"]["umf.actions"]["version"] != "0.1.0" {
        return Err("actions vocabulary version unsupported");
    }
    let (mi, module) = array(&doc["modules"])?
        .iter()
        .enumerate()
        .find(|(_, m)| m["id"] == request.source.module)
        .ok_or("action module absent")?;
    let payload = &module["extensions"]["umf.actions"];
    object(payload, &["actions"], &[])?;
    if !ActionsSchema::is_valid(payload) {
        return Err("action source schema refused");
    }
    let (ai, action) = array(&payload["actions"])?
        .iter()
        .enumerate()
        .find(|(_, a)| a["id"] == request.source.action)
        .ok_or("selected action absent")?;
    if request.source.action_pointer != format!("/modules/{mi}/extensions/umf.actions/actions/{ai}")
    {
        return Err("original action pointer differs");
    }
    for key in [
        "parameters",
        "reads",
        "writes",
        "failures",
        "preconditions",
        "postconditions",
    ] {
        if array(&action[key])?.len() > 64 {
            return Err("native action inventory budget");
        }
    }
    let mut expression_bytes = 0;
    for condition in array(&action["preconditions"])?
        .iter()
        .chain(array(&action["postconditions"])?.iter())
    {
        let length = condition["rule"]["expression"]
            .as_str()
            .ok_or("rule expression required")?
            .len();
        expression_bytes += length;
        if length > 8192 || expression_bytes > 65536 {
            return Err("native action rule budget");
        }
    }
    object(
        action,
        &[
            "id",
            "name",
            "description",
            "parameters",
            "outputs",
            "preconditions",
            "postconditions",
            "failures",
            "reads",
            "writes",
            "binding",
            "authorization",
            "attribution",
            "atomicity",
            "idempotency",
            "result",
        ],
        &[],
    )?;
    if action["atomicity"] != "single-store"
        || action["idempotency"] != "caller-key-optional"
        || action["result"]
            != json!({"identities":"created-and-changed","version":"store-opaque","receipt":"read-at-least"})
        || !array(&action["outputs"])?.is_empty()
    {
        return Err("native action obligations unsupported");
    }
    object(&action["binding"], &["kind", "profile", "effects"], &[])?;
    if array(&action["binding"]["effects"])?.len() > 64 {
        return Err("native action effect budget");
    }
    if action["binding"]["kind"] != "recipe"
        || action["binding"]["profile"] != json!({"id":"graph-write","version":"1"})
    {
        return Err("NATIVE_PROFILE_UNSUPPORTED");
    }
    object(&action["authorization"], &["kind", "profile", "roles"], &[])?;
    if action["authorization"]["kind"] != "roles"
        || action["authorization"]["profile"] != json!({"id":"umf.actions.roles","version":"1"})
    {
        return Err("NATIVE_PROFILE_UNSUPPORTED");
    }
    for role in array(&action["authorization"]["roles"])? {
        if !request.role_mappings.contains_key(text(role)?) {
            return Err("ROLE_MAPPING_MISSING");
        }
    }
    if action["attribution"] != json!({"subject":"person-required"})
        && action["attribution"]
            != json!({"subject":"actor-profile","profile":{"id":"umf.actions.native-session","version":"0.1"}})
    {
        return Err("NATIVE_PROFILE_UNSUPPORTED");
    }
    let catalog = Catalog::prepare_security(vec![ModuleInput {
        document_json: request.binding.model_json.clone(),
        pin: ModelPin {
            document_id: request.source.document_id.clone(),
            revision: request.binding.model_sha256.clone(),
            umf_version: "0.8.0".into(),
            sha256: sha256(request.binding.model_json.as_bytes()),
        },
        selected_module_ids: array(&doc["modules"])?
            .iter()
            .map(|m| m["id"].as_str().unwrap_or("").to_owned())
            .collect(),
    }])
    .map_err(|_| "native action model admission refused")?;
    request
        .security
        .policy
        .read()
        .map_err(|_| "original security policy integrity refused")?;
    request
        .security
        .ontology
        .read()
        .map_err(|_| "original security ontology integrity refused")?;
    let security = SecuritySourcePacket::read(
        &request.security.policy.utf8,
        &request.security.ontology.utf8,
        &catalog,
    )
    .map_err(|_| "security source admission refused")?;
    SecurityOntologyClosure::read(&security, &catalog)
        .map_err(|_| "security ontology closure refused")?;
    if security.policy().get("native").is_some() || security.ontology().get("native").is_some() {
        return Err("native security qualifiers require their own lowering");
    }
    if !array(&security.policy()["rules"])?.is_empty()
        || !array(&security.ontology()["associations"])?.is_empty()
        || !array(&security.ontology()["context"])?.is_empty()
    {
        return Err("selected security policy requires a qualified action lowering");
    }
    for e in array(&security.ontology()["entities"])? {
        for f in array(&e["fields"])? {
            if f["protection"] != "unprotected" {
                return Err("protected action Fields require a qualified lowering");
            };
            object(f, &["ref", "protection"], &[])?;
        }
    }
    let mut field_work = 0usize;
    let mut metadata_work = 0usize;
    for frame in array(&action["reads"])?
        .iter()
        .chain(array(&action["writes"])?.iter())
    {
        for reference in array(&frame["fields"])? {
            let (_, field) = locate(&doc, reference)?;
            field_work += 1;
            metadata_work = metadata_work.saturating_add(crate::mutation::canonical(field).len());
            if field_work > 1024 || metadata_work > 262144 {
                return Err("native action retained Field expansion budget");
            }
        }
    }
    let mut parameters = BTreeMap::new();
    let mut fields = BTreeMap::new();
    let mut entities = BTreeMap::new();
    for p in array(&action["parameters"])? {
        let id = text(&p["id"])?;
        if parameters.insert(id.into(), p.clone()).is_some() {
            return Err("duplicate action parameter");
        }
        match p["kind"].as_str() {
            Some("entity") => {
                object(p, &["id", "kind", "target", "required"], &[])?;
                if p["required"] != true {
                    return Err("optional native entity parameter unsupported");
                }
                let k = key(&doc, &request.binding, &p["target"])?;
                let components = (0..k.fields.len())
                    .map(|i| json!({"entity":id,"component":i}))
                    .collect();
                entities.insert(
                    crate::mutation::canonical(&json!(["parameter", id])),
                    ActionEntity {
                        key: k,
                        components,
                        required_existing: true,
                    },
                );
            }
            Some("value") => {
                object(p, &["id", "kind", "field", "required"], &[])?;
                if p["required"] != true {
                    return Err("optional native recipe parameters unsupported");
                };
                let f = field(&doc, &request.binding, &p["field"])?;
                fields.insert(ref_id(&p["field"])?, f);
            }
            _ => return Err("unknown action parameter kind"),
        }
    }
    let mut frames = BTreeMap::new();
    for f in array(&action["reads"])?
        .iter()
        .chain(array(&action["writes"])?.iter())
    {
        object(
            f,
            &[
                "id",
                "record",
                "selector",
                "maxEntities",
                "fields",
                "relationships",
                "create",
                "delete",
            ],
            &[],
        )?;
        if f["maxEntities"] != 1 || !array(&f["relationships"])?.is_empty() {
            return Err("native frame profile requires one bounded scalar entity");
        }
        object(
            &f["selector"],
            &["language", "version", "expression", "references"],
            &[],
        )?;
        if f["selector"]["language"] != "umf.actions.keys" || f["selector"]["version"] != "1" {
            return Err("native frame selector unsupported");
        }
        let selector = checked_json(
            f["selector"]["expression"]
                .as_str()
                .ok_or("selector expression required")?,
        )
        .map_err(|_| "selector JSON refused")?;
        let refs = array(&f["selector"]["references"])?;
        for reference in refs {
            object(reference, &["parameter"], &[])?;
            if !parameters.contains_key(text(&reference["parameter"])?) {
                return Err("selector dependency absent");
            }
        }
        let need = |id: &str| -> ActionResult<()> {
            if !refs.iter().any(|r| r["parameter"] == id) {
                return Err("selector dependency undeclared");
            }
            Ok(())
        };
        if let Some(id) = selector.get("entity") {
            need(text(id)?)?;
        } else {
            for component in array(&selector["components"])? {
                if let Some(id) = component.get("input") {
                    need(text(id)?)?;
                }
            }
        }
        let entity = if let Some(p) = selector.get("entity") {
            object(&selector, &["entity"], &[])?;
            let entity = entities
                .get(&crate::mutation::canonical(&json!(["parameter", text(p)?])))
                .ok_or("frame entity parameter absent")?
                .clone();
            if entity.key.reference["module"] != f["record"]["module"]
                || entity.key.reference["element"] != f["record"]["element"]
            {
                return Err("frame entity owner differs");
            };
            entity
        } else {
            object(&selector, &["key", "components"], &[])?;
            let mut reference = f["record"].clone();
            reference["key"] = selector["key"].clone();
            let k = key(&doc, &request.binding, &reference)?;
            let c = array(&selector["components"])?;
            if c.len() != k.fields.len() {
                return Err("frame Key arity differs");
            };
            let components = c
                .iter()
                .zip(&k.fields)
                .map(|(v, field)| {
                    if v.get("input").is_some() {
                        object(v, &["input"], &[])?;
                        let p = parameters
                            .get(text(&v["input"])?)
                            .ok_or("frame input absent")?;
                        if p["field"] != field.reference || p["required"] != true {
                            return Err("frame input Field differs");
                        };
                        Ok(v.clone())
                    } else {
                        object(v, &["literal"], &[])?;
                        literal(&v["literal"], field)?;
                        Ok(v.clone())
                    }
                })
                .collect::<ActionResult<_>>()?;
            ActionEntity {
                key: k,
                components,
                required_existing: false,
            }
        };
        let mut seen_fields = BTreeSet::new();
        for reference in array(&f["fields"])? {
            if !seen_fields.insert(ref_id(reference)?) {
                return Err("duplicate frame Field");
            }
        }
        let fs = array(&f["fields"])?
            .iter()
            .map(|r| {
                let field = field(&doc, &request.binding, r)?;
                if field.table != entity.key.table {
                    return Err("frame Field owner differs");
                };
                Ok(field)
            })
            .collect::<ActionResult<Vec<_>>>()?;
        for field in &fs {
            fields.insert(ref_id(&field.reference)?, field.clone());
        }
        if frames
            .insert(
                text(&f["id"])?.into(),
                ActionFrame {
                    source: f.clone(),
                    entity,
                    fields: fs,
                },
            )
            .is_some()
        {
            return Err("duplicate frame identity");
        }
    }
    let mut effects = BTreeSet::new();
    let mut effect_program = vec![];
    for effect in array(&action["binding"]["effects"])? {
        let id = text(&effect["id"])?;
        if !effects.insert(id) {
            return Err("duplicate action effect");
        }
        let (entity, values) = match effect["kind"].as_str() {
            Some("create") => {
                object(effect, &["id", "kind", "record", "key", "values"], &[])?;
                let mut r = effect["record"].clone();
                r["key"] = effect["key"].clone();
                let k = key(&doc, &request.binding, &r)?;
                let assignments = array(&effect["values"])?;
                let mut components = vec![];
                for keyfield in &k.fields {
                    let a = assignments
                        .iter()
                        .find(|a| a["field"] == keyfield.reference)
                        .ok_or("create Key value absent")?;
                    components.push(value(&a["value"], keyfield, &parameters, &fields)?)
                }
                let entity = ActionEntity {
                    key: k,
                    components,
                    required_existing: false,
                };
                entities.insert(
                    crate::mutation::canonical(&json!(["created", id])),
                    entity.clone(),
                );
                (entity, Some(assignments))
            }
            Some("set" | "delete") => {
                object(
                    effect,
                    if effect["kind"] == "set" {
                        &["id", "kind", "entity", "values"]
                    } else {
                        &["id", "kind", "entity"]
                    },
                    &[],
                )?;
                let e = &effect["entity"];
                let name = if let Some(p) = e.get("parameter") {
                    object(e, &["parameter"], &[])?;
                    crate::mutation::canonical(&json!(["parameter", text(p)?]))
                } else {
                    object(e, &["created"], &[])?;
                    crate::mutation::canonical(&json!(["created", text(&e["created"])?]))
                };
                (
                    entities
                        .get(&name)
                        .ok_or("effect entity absent or not yet created")?
                        .clone(),
                    if effect["kind"] == "set" {
                        Some(array(&effect["values"])?)
                    } else {
                        None
                    },
                )
            }
            _ => return Err("native relationship effect requires qualified binding lowering"),
        };
        let write_frames: Vec<_> = frames
            .values()
            .filter(|f| {
                array(&action["writes"]).unwrap().contains(&f.source)
                    && f.entity.key.reference == entity.key.reference
                    && f.entity.components == entity.components
            })
            .collect();
        if write_frames.is_empty()
            || effect["kind"] == "create"
                && !write_frames.iter().any(|f| f.source["create"] == true)
            || effect["kind"] == "delete"
                && !write_frames.iter().any(|f| f.source["delete"] == true)
        {
            return Err("effect escapes declared frame");
        }
        let mut assignments = vec![];
        if let Some(values) = values {
            let mut seen = BTreeSet::new();
            for a in values {
                object(a, &["field", "value"], &[])?;
                let f = field(&doc, &request.binding, &a["field"])?;
                if f.table != entity.key.table
                    || !seen.insert(ref_id(&f.reference)?)
                    || !write_frames
                        .iter()
                        .any(|w| w.fields.iter().any(|wf| wf.reference == f.reference))
                {
                    return Err("assignment escapes declared frame");
                };
                let source = value(&a["value"], &f, &parameters, &fields)?;
                assignments.push((f.clone(), source));
                fields.insert(ref_id(&f.reference)?, f);
            }
        }
        effect_program.push(ActionEffect {
            id: id.into(),
            kind: text(&effect["kind"])?.into(),
            entity,
            values: assignments,
        });
    }
    let preconditions = condition_rules("pre", action, &parameters, &fields, &frames)?;
    let postconditions = condition_rules("post", action, &parameters, &fields, &frames)?;
    Ok(AdmittedAction {
        action: action.clone(),
        request,
        fields,
        entities,
        frames,
        effects: effect_program,
        preconditions,
        postconditions,
    })
}
