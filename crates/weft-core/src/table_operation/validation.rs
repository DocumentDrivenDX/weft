// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
struct Check<'a> {
    admitted: &'a AdmittedOperation,
    parameters: BTreeMap<String, &'a OperationField>,
    used: BTreeSet<String>,
    nodes: usize,
    source_scope: bool,
    related_scope: Option<String>,
    assignment: bool,
}
impl<'a> Check<'a> {
    fn field(&self, reference: &Value) -> Result<&'a OperationField> {
        if let Some(table)=&self.related_scope { self.admitted.related_field(table,reference) } else if self.source_scope { self.admitted.source_field(reference) } else { self.admitted.field(reference) }
    }
    fn value(&mut self, value: &Value, field: &Value) -> Result<()> {
        let target = self.field(field)?;
        if value["kind"] == "currentPrincipal" {
            return if target.carrier==Carrier::String && matches!(target.native_type["kind"].as_str(),Some("text"|"varchar")) { Ok(()) } else { Err("current principal needs native text storage") };
        }
        if value["kind"] == "sourceField" {
            if !self.assignment || self.source_scope { return Err("source Field is only available in insert-from assignments"); }
            let origin=self.admitted.source_field(&value["field"])?;
            if origin.carrier != target.carrier || origin.native_type != target.native_type || origin.nullable && !target.nullable { return Err("source native capacity or availability differs"); }
            if let Some(values)=origin.declaration["allowedValues"].as_array().filter(|v|!v.is_empty()) {
                for value in values { if !value.is_null() || origin.nullable { literal(target,value)?; } }
            } else {
                if target.declaration.get("allowedValues").is_some() { return Err("source does not prove target membership"); }
                let source=crate::security_literals::normalized_facets(&origin.declaration).map_err(|_|"source facets unsupported")?;
                let destination=crate::security_literals::normalized_facets(&target.declaration).map_err(|_|"target facets unsupported")?;
                for (key,facet) in destination.as_object().ok_or("target facets absent")? { if source.get(key) != Some(facet) { return Err("source does not prove target refinement"); } }
            }
            return Ok(());
        }
        if value["kind"] == "currentTimestamp" {
            return if target.carrier==Carrier::Timestamp && target.native_type["kind"]=="timestamptz" { Ok(()) } else { Err("transaction timestamp requires explicit instant storage") };
        }
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
            "related" => {
                let id=node["table"].as_str().ok_or("related table identity absent")?;
                self.admitted.related_table(id)?;
                let mut seen=BTreeSet::new();
                for pair in node["matches"].as_array().ok_or("related matches absent")? {
                    if !seen.insert(pair.to_string()) { return Err("duplicate related match"); }
                    let source=self.field(&pair["source"])?;let target=self.admitted.related_field(id,&pair["target"])?;
                    if source.carrier!=target.carrier || source.native_type!=target.native_type { return Err("related native equality capacity differs"); }
                }
                let previous=self.related_scope.replace(id.to_owned());
                let result=self.predicate(&node["where"],depth+1);self.related_scope=previous;result
            }

            _ => {
                let field = self.field(&node["field"])?;
                if op == "isNull" {
                    return Ok(());
                }
                if op == "startsWith" && field.carrier != Carrier::String {
                    return Err("operation prefix requires string");
                }
                if ["lt", "le", "gt", "ge"].contains(&op)
                    && !matches!(
                        field.carrier,
                        Carrier::Integer | Carrier::Decimal | Carrier::String | Carrier::Date | Carrier::Timestamp
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
pub(super) fn validate_static(
    admitted: AdmittedOperation,
    record: &Value,
    table: &Value,
) -> Result<AdmittedOperation> {
    let mut check = Check {
        admitted: &admitted,
        parameters: BTreeMap::new(),
        used: BTreeSet::new(),
        nodes: 0,
        source_scope: false,
        related_scope: None,
        assignment: false,
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
            if order.len() < key.len()
                || key.iter().zip(&order[order.len().saturating_sub(key.len())..]).any(|(f, o)| f != &o["field"])
                || order.iter().any(|o| !body["fields"].as_array().unwrap().contains(&o["field"]))
            {
                return Err("operation cursor Key suffix/order/projection differs");
            }
            for item in order {
                let field = admitted.field(&item["field"])?;
                if field.nullable
                    || !matches!(
                        field.carrier,
                        Carrier::Integer | Carrier::Decimal | Carrier::String | Carrier::Date | Carrier::Timestamp | Carrier::Uuid
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
        unique_refs(&admitted, &body["returning"])?;
        if kind == "insertFrom" {
            if admitted.source_table().is_none() { return Err("original source table absent"); }
            check.source_scope=true;
            check.predicate(&body["source"]["where"],0)?;
            check.source_scope=false;
        }
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
                check.assignment=true;
                check.value(&assignment["value"], reference)?;
                check.assignment=false;
            }
            if kind == "insert" || kind == "insertFrom" || kind == "upsert" {
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
    drop(check);
    Ok(admitted)
}

pub(super) fn validate_call(admitted: &AdmittedOperation) -> Result<()> {
    let body = &admitted.declaration["body"];
    let read = body["kind"] == "read";
    if !read {
        if admitted.declaration["replay"] == "caller-key-required" && admitted.idempotency_key().is_none() {
            return Err("write replay key required");
        }
        if admitted.idempotency_key().is_some_and(|s| s.is_empty() || s.len() > 256 || s.contains('\0')) {
            return Err("write replay key outside bounds");
        }
    } else if admitted.idempotency_key().is_some() {
        return Err("read cannot carry write replay key");
    }
    let validate_inputs = |inputs: &BTreeMap<String, Value>| -> Result<()> {
        if inputs.len() != admitted.declaration["parameters"].as_array().ok_or("operation parameter inventory absent")?.len() {
            return Err("operation input inventory differs");
        }
        for parameter in admitted.declaration["parameters"].as_array().unwrap() {
            let id = parameter["id"].as_str().ok_or("operation parameter absent")?;
            let field = admitted.parameter_fields.get(&reference_id(&parameter["field"])?).ok_or("parameter domain outside owning module")?;
            literal(field, inputs.get(id).ok_or("required operation input absent")?)?;
        }
        Ok(())
    };
    match admitted.call() {
        OperationCall::Single { inputs, after, .. } => {
            validate_inputs(inputs)?;
            if let Some(after) = after {
                if admitted.cursor_fields.is_empty() || after.len() != admitted.cursor_fields.len() { return Err("operation cursor tuple differs"); }
                for (field, value) in admitted.cursor_fields.iter().zip(after) { literal(field, value)?; }
            }
        }
        OperationCall::Batch { items } => {
            let limit = crate::security_literals::unsigned(&admitted.declaration["batch"]["maxItems"]).map_err(|_| "operation batch not declared")?;
            if read || items.is_empty() || items.len() as u64 > limit { return Err("operation batch size/kind differs"); }
            for inputs in items { validate_inputs(inputs)?; }
        }
    }
    Ok(())
}
