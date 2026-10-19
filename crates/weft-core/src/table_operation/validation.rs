// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
struct Check<'a> {
    admitted: &'a AdmittedOperation,
    parameters: BTreeMap<String, &'a OperationField>,
    used: BTreeSet<String>,
    nodes: usize,
}
impl Check<'_> {
    fn value(&mut self, value: &Value, field: &Value) -> Result<()> {
        let target = self.admitted.field(field)?;
        if value["kind"] == "literal" {
            return literal(target, &value["value"]);
        }
        let id = value["parameter"]
            .as_str()
            .ok_or("operation parameter absent")?;
        let parameter = self
            .parameters
            .get(id)
            .ok_or("undeclared operation parameter")?;
        if domain(parameter) != domain(target) {
            return Err("operation parameter domain or native capacity differs");
        }
        self.used.insert(id.to_owned());
        Ok(())
    }
    fn predicate(&mut self, node: &Value, depth: usize) -> Result<()> {
        self.nodes += 1;
        if depth > 16 || self.nodes > 4096 {
            return Err("operation predicate capacity exceeded");
        }
        let op = node["op"].as_str().ok_or("operation predicate absent")?;
        match op {
            "all" => Ok(()),
            "and" | "or" => {
                for child in node["args"]
                    .as_array()
                    .ok_or("operation boolean operands absent")?
                {
                    self.predicate(child, depth + 1)?
                }
                Ok(())
            }
            "not" => self.predicate(&node["arg"], depth + 1),
            _ => {
                let field = self.admitted.field(&node["field"])?;
                if op == "isNull" {
                    return Ok(());
                }
                if op == "startsWith" && field.carrier != Carrier::String {
                    return Err("operation prefix requires string");
                }
                if ["lt", "le", "gt", "ge"].contains(&op)
                    && !matches!(
                        field.carrier,
                        Carrier::Integer | Carrier::Decimal | Carrier::String
                    )
                {
                    return Err("operation ordering carrier unsupported");
                }
                if op == "in" {
                    for value in node["values"]
                        .as_array()
                        .ok_or("operation membership values absent")?
                    {
                        self.value(value, &node["field"])?
                    }
                    Ok(())
                } else {
                    self.value(&node["value"], &node["field"])
                }
            }
        }
    }
}
fn unique_refs(admitted: &AdmittedOperation, refs: &Value) -> Result<Vec<String>> {
    let mut seen = BTreeSet::new();
    let mut names = vec![];
    for reference in refs.as_array().ok_or("operation Field list absent")? {
        let id = reference_id(reference)?;
        if !seen.insert(id) {
            return Err("duplicate operation Field");
        }
        names.push(admitted.field(reference)?.native.clone());
    }
    Ok(names)
}
pub(super) fn validate(
    admitted: AdmittedOperation,
    record: &Value,
    table: &Value,
) -> Result<AdmittedOperation> {
    let mut check = Check {
        admitted: &admitted,
        parameters: BTreeMap::new(),
        used: BTreeSet::new(),
        nodes: 0,
    };
    for parameter in admitted.declaration["parameters"].as_array().unwrap() {
        let id = parameter["id"].as_str().unwrap();
        let field = admitted
            .parameter_fields
            .get(&reference_id(&parameter["field"])?)
            .ok_or("parameter domain outside owning module")?;
        if check.parameters.insert(id.to_owned(), field).is_some() {
            return Err("duplicate operation input");
        }
    }
    let body = &admitted.declaration["body"];
    let kind = body["kind"].as_str().unwrap();
    let read = kind == "read";
    let max = admitted.max_rows;
    let mut cursor_fields = vec![];
    if read {
        if admitted.declaration["replay"] != "none"
            || admitted.declaration.get("batch").is_some()
            || admitted.request.idempotency_key.is_some()
        {
            return Err("read cannot carry write replay or batch grants");
        }
        let fields = unique_refs(&admitted, &body["fields"])?;
        check.predicate(&body["where"], 0)?;
        let result = body["result"].as_str().unwrap();
        let mut ordering = BTreeSet::new();
        for order in body["orderBy"].as_array().unwrap() {
            let field = admitted.field(&order["field"])?;
            if !ordering.insert(field.native.clone()) {
                return Err("duplicate operation ordering");
            }
        }
        if result == "count" {
            if !fields.is_empty() || !ordering.is_empty() || max != 1 {
                return Err("count result shape differs");
            }
        } else if fields.is_empty() || matches!(result, "one" | "optional") && max != 1 {
            return Err("operation row result shape differs");
        }
        if result == "page" {
            let key = record["keys"]
                .as_array()
                .unwrap()
                .iter()
                .find(|k| k["id"] == body["cursorKey"])
                .ok_or("original page Key absent")?;
            let key = key["fields"].as_array().unwrap();
            let order = body["orderBy"].as_array().unwrap();
            if key.len() != order.len()
                || key.iter().zip(order).any(|(f, o)| {
                    f != &o["field"]
                        || o["direction"] != "asc"
                        || !body["fields"].as_array().unwrap().contains(f)
                })
            {
                return Err("operation cursor Key/order/projection differs");
            }
            for reference in key {
                let field = admitted.field(reference)?;
                if field.nullable
                    || !matches!(
                        field.carrier,
                        Carrier::Integer | Carrier::Decimal | Carrier::String | Carrier::Uuid
                    )
                {
                    return Err("operation cursor carrier/availability unsupported");
                }
                cursor_fields.push(field);
            }
        } else if body.get("cursorKey").is_some() {
            return Err("nonpage operation has cursor Key");
        }
    } else {
        if admitted.declaration["consistency"] != "transaction"
            || admitted.declaration["replay"] == "none"
        {
            return Err("write needs transaction and declared replay");
        }
        if admitted.declaration["replay"] == "caller-key-required"
            && admitted.request.idempotency_key.is_none()
        {
            return Err("write replay key required");
        }
        if admitted
            .request
            .idempotency_key
            .as_ref()
            .is_some_and(|s| s.is_empty() || s.len() > 256 || s.contains('\0'))
        {
            return Err("write replay key outside bounds");
        }
        unique_refs(&admitted, &body["returning"])?;
        if kind == "update" || kind == "delete" {
            check.predicate(&body["where"], 0)?
        }
        let mutable = |reference: &Value| -> bool {
            table["columns"].as_array().unwrap().iter().any(|c| {
                c["field"] == *reference && c["immutable"] != true && c.get("generated").is_none()
            }) && !record["keys"]
                .as_array()
                .unwrap()
                .iter()
                .any(|k| k["fields"].as_array().unwrap().contains(reference))
        };
        if kind != "delete" {
            let mut assigned = BTreeSet::new();
            for assignment in body["values"].as_array().unwrap() {
                let reference = &assignment["field"];
                admitted.field(reference)?;
                if !assigned.insert(reference_id(reference)?) {
                    return Err("duplicate assigned operation Field");
                }
                let column = table["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|c| c["field"] == *reference)
                    .unwrap();
                if column.get("generated").is_some() || kind == "update" && !mutable(reference) {
                    return Err("operation assigns generated/immutable/Key Field");
                }
                check.value(&assignment["value"], reference)?;
            }
            if kind == "insert" || kind == "upsert" {
                for column in table["columns"].as_array().unwrap() {
                    if !admitted.field(&column["field"])?.nullable
                        && column.get("generated").is_none()
                        && column.get("default").is_none()
                        && !assigned.contains(&reference_id(&column["field"])?)
                    {
                        return Err("operation omits required create value");
                    }
                }
            }
        }
        if kind == "upsert" {
            let key = record["keys"]
                .as_array()
                .unwrap()
                .iter()
                .find(|k| k["id"] == body["conflictKey"])
                .ok_or("original conflict Key absent")?;
            for reference in key["fields"].as_array().unwrap() {
                if admitted.field(reference)?.nullable
                    || !body["values"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|a| a["field"] == *reference)
                {
                    return Err("conflict Key input/availability differs");
                }
            }
            let updates = unique_refs(&admitted, &body["updateFields"])?;
            if (body["onConflict"] == "ignore") != updates.is_empty() {
                return Err("conflict whitelist differs");
            }
            for reference in body["updateFields"].as_array().unwrap() {
                if !mutable(reference)
                    || !body["values"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|a| a["field"] == *reference)
                {
                    return Err("conflict update is not a mutable create Field");
                }
            }
        }
    }
    if check.parameters.keys().any(|id| !check.used.contains(id)) {
        return Err("unused declared operation input");
    }
    let validate_inputs = |inputs: &BTreeMap<String, Value>| -> Result<()> {
        if inputs.len() != check.parameters.len() {
            return Err("operation input inventory differs");
        }
        for (id, field) in &check.parameters {
            literal(
                field,
                inputs.get(id).ok_or("required operation input absent")?,
            )?
        }
        Ok(())
    };
    match &admitted.request.call {
        OperationCall::Single { inputs, after, .. } => {
            validate_inputs(inputs)?;
            if let Some(after) = after {
                if cursor_fields.is_empty() || after.len() != cursor_fields.len() {
                    return Err("operation cursor tuple differs");
                }
                for (field, value) in cursor_fields.iter().zip(after) {
                    literal(field, value)?
                }
            }
        }
        OperationCall::Batch { items } => {
            let limit =
                crate::security_literals::unsigned(&admitted.declaration["batch"]["maxItems"])
                    .map_err(|_| "operation batch not declared")?;
            if read || items.is_empty() || items.len() as u64 > limit {
                return Err("operation batch size/kind differs");
            }
            for inputs in items {
                validate_inputs(inputs)?
            }
        }
    }
    drop(check);
    Ok(admitted)
}
