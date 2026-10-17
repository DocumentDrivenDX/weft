//! Distinct native atomic-bulk admission; never a Truss manifest or tx/1 plan.
use crate::{
    json::checked_json,
    raw_action::{self, ActionRequest, AdmittedAction},
    raw_read::{self, AdmittedRead, Request},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RequestBulk {
    pub version: String,
    pub action: ActionRequest,
    pub selection: Request,
    pub max_items: u16,
    pub action_inputs: BTreeMap<String, String>,
    pub parameters: Vec<crate::raw_read::Parameter>,
}
pub struct AdmittedBulk {
    request: RequestBulk,
    action: AdmittedAction,
    selection: AdmittedRead,
}
impl AdmittedBulk {
    pub fn request(&self) -> &RequestBulk {
        &self.request
    }
    pub fn action(&self) -> &AdmittedAction {
        &self.action
    }
    pub fn selection(&self) -> &AdmittedRead {
        &self.selection
    }
}
pub fn admit_json(raw: &str) -> Result<AdmittedBulk, &'static str> {
    let r: RequestBulk =
        serde_json::from_value(checked_json(raw).map_err(|_| "bounded native bulk JSON required")?)
            .map_err(|_| "exact native bulk request required")?;
    if r.version != "weft.native-bulk/0.1"
        || !(1..=999).contains(&r.max_items)
        || r.selection.row_cap != r.max_items + 1
        || r.selection.keyset
    {
        return Err("native bulk bound/profile refused");
    }
    if r.action.target != r.selection.target
        || crate::mutation::canonical(&serde_json::to_value(&r.action.binding).unwrap())
            != crate::mutation::canonical(&serde_json::to_value(&r.selection.binding).unwrap())
    {
        return Err("native bulk selection/action source binding differs");
    }
    let action = raw_action::admit_json(&serde_json::to_string(&r.action).unwrap())?;
    let selection = raw_read::admit_json(&serde_json::to_string(&r.selection).unwrap())?;
    if action.effects().len() != 1
        || action.effects()[0].kind != "set"
        || !action.preconditions().is_empty()
        || !action.postconditions().is_empty()
    {
        return Err("native bulk profile requires one scalar SET without row rules");
    }
    let parameters = action.action()["parameters"].as_array().unwrap();
    if parameters.iter().filter(|p| p["kind"] == "entity").count() != 1 {
        return Err("native bulk requires one entity parameter");
    }
    let scalar_ids: std::collections::BTreeSet<_> = parameters
        .iter()
        .filter(|p| p["kind"] == "value")
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    if scalar_ids != r.action_inputs.keys().map(String::as_str).collect() {
        return Err("native bulk scalar input inventory differs");
    }
    if r.parameters.len() > 64
        || r.parameters
            .iter()
            .map(|p| p.name.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != r.parameters.len()
    {
        return Err("native bulk parameter inventory");
    }
    for p in &r.selection.parameters {
        if !r
            .parameters
            .iter()
            .any(|v| v == p)
        {
            return Err("native bulk filter parameter differs");
        }
    }
    if r.parameters.iter().any(|p| {
        !r.action_inputs.values().any(|name| name == &p.name)
            && !r.selection.parameters.iter().any(|v| v.name == p.name)
    }) {
        return Err("native bulk unused parameter");
    }
    // Scalar parameter domains remain the exact original action Field domains.
    for (id, name) in &r.action_inputs {
        let p = parameters.iter().find(|p| p["id"] == *id).unwrap();
        let f = action
            .fields()
            .values()
            .find(|f| f.reference == p["field"])
            .ok_or("bulk scalar Field absent")?;
        let bound = r
            .parameters
            .iter()
            .find(|p| p.name == *name)
            .ok_or("bulk action parameter absent")?;
        if bound.container.is_some()
            || bound.element_nullable.is_some()
            || bound.max_items.is_some()
            || bound.carrier != f.carrier
            || bound.nullable != f.nullable
        {
            return Err("bulk scalar carrier differs");
        }
    }
    Ok(AdmittedBulk {
        request: r,
        action,
        selection,
    })
}
