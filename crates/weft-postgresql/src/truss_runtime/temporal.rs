// SPDX-License-Identifier: MIT OR Apache-2.0
//! ABI1.3 full buffered source. No selected/seek optimization is reachable here.
use super::*;
use serde_json::Value;
use weft_core::application_temporal::{self as front, KeyRef, Plan, Predicate};
type Result<T> = front::Result<T>;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct After {
    version: String,
    selected_key: KeyRef,
    sort_values: Vec<State>,
    key_values: Vec<String>,
}
#[derive(Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "camelCase", deny_unknown_fields)]
enum State {
    Present { value: String },
    Null {},
    Missing {},
}
struct Emit<'a> {
    plan: &'a Plan,
    record: &'a RecordBinding,
    slots: Vec<Value>,
}
impl Emit<'_> {
    fn slot(&mut self, logical_type: Value, value: String, origin: Value) -> Result<String> {
        if self.slots.len() >= 1024 || value.len() > 4_194_304 || value.contains('\0') {
            return Err((Diagnostic::new(
                "WFT-LIMIT",
                "emit",
                "Bounded PostgreSQL parameter required",
            ))
            .into());
        }
        let position = self.slots.len() + 1;
        self.slots.push(
            json!({"position":position,"logicalType":logical_type,"value":value,"origin":origin}),
        );
        Ok(format!("${position}"))
    }
    fn text_slot(&mut self, value: String, origin: Value) -> Result<String> {
        self.slot(
            json!({"family":"string","facets":{},"nullable":false}),
            value,
            origin,
        )
    }
    fn cell(&self, i: usize) -> Result<String> {
        let field = &self.plan.fields[i];
        let matches = self
            .record
            .fields
            .iter()
            .filter(|f| f.identity == field.identity)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err((fail("Exact native Field mapping required")).into());
        }
        Ok(format!(
            "\"__weft_temporal_source\".props->{}",
            literal(&matches[0].property_id)?
        ))
    }
    fn comparable(&self, i: usize) -> Result<String> {
        let f = &self.plan.fields[i];
        let cell = self.cell(i)?;
        let raw = if front::is_temporal(&f.kind) {
            format!(
                "\"truss\".\"query_temporal_canonical_text\"({},({cell})->>'text')",
                literal(&f.kind)?
            )
        } else {
            match f.kind.as_str() {
                "string" => format!("({cell})->>'text'"),
                "boolean" => format!("(({cell})->>'value')::pg_catalog.bool"),
                "integer" | "decimal" => format!("(({cell})->>'text')::pg_catalog.numeric"),
                _ => return Err((unsupported("Unsupported temporal sibling")).into()),
            }
        };
        let value = format!(
            "CASE WHEN {cell} IS NULL OR ({cell})->>'kind'='null' THEN NULL ELSE {raw} END"
        );
        Ok(if front::is_temporal(&f.kind) || f.kind == "string" {
            format!("({value}) COLLATE \"C\"")
        } else {
            value
        })
    }
    fn operand(&mut self, left: usize, right: &front::Operand) -> Result<String> {
        if let Some(i) = right.field {
            return self.comparable(i);
        }
        let f = &self.plan.fields[left];
        let value = right
            .value
            .as_ref()
            .ok_or_else(|| fail("Original operand token required"))?;
        let typ = if front::is_temporal(&f.kind) {
            json!({"kind":"accessTemporal","temporalKind":f.kind,"profile":front::PROFILE})
        } else {
            json!(f.logical_type)
        };
        let slot = self.slot(typ, value.clone(), right.origin.clone())?;
        Ok(if front::is_temporal(&f.kind) {
            format!("\"truss\".\"query_temporal_canonical_text\"({},{slot}::pg_catalog.text) COLLATE \"C\"",literal(&f.kind)?)
        } else {
            match f.kind.as_str() {
                "string" => format!("({slot}::pg_catalog.text) COLLATE \"C\""),
                "boolean" => format!("{slot}::pg_catalog.bool"),
                _ => format!("{slot}::pg_catalog.numeric"),
            }
        })
    }
    fn predicate(&mut self, p: &Predicate) -> Result<String> {
        Ok(match p {
            Predicate::Compare {
                left,
                operator,
                right,
            } => {
                let l = self.comparable(*left)?;
                let r = self.operand(*left, right)?;
                format!("({l} {operator} {r})")
            }
            Predicate::And { args } | Predicate::Or { args } => {
                let op = if matches!(p, Predicate::And { .. }) {
                    " AND "
                } else {
                    " OR "
                };
                format!(
                    "({})",
                    args.iter()
                        .map(|a| self.predicate(a))
                        .collect::<Result<Vec<_>>>()?
                        .join(op)
                )
            }
            Predicate::Not { arg } => format!("(NOT {})", self.predicate(arg)?),
            Predicate::Boolean { value } => if *value { "TRUE" } else { "FALSE" }.into(),
        })
    }
    fn projection(&self, i: usize) -> Result<String> {
        let f = &self.plan.fields[i];
        let cell = self.cell(i)?;
        // Exactly the existing exact-scalar/property presence envelope, including
        // original token spelling. Canonicalization is comparison-only.
        if front::is_temporal(&f.kind) || f.missing || f.nullable {
            let v = if f.kind == "boolean" {
                format!("({cell})->'value'")
            } else {
                format!("pg_catalog.to_jsonb(({cell})->>'text')")
            };
            if !f.missing && !f.nullable {
                return Ok(format!("{v}::pg_catalog.text"));
            }
            Ok(format!("(CASE WHEN {cell} IS NULL THEN pg_catalog.jsonb_build_object('state','absent') WHEN ({cell})->>'kind'='null' THEN pg_catalog.jsonb_build_object('state','null') ELSE pg_catalog.jsonb_build_object('state','value','value',{v}) END)::pg_catalog.text"))
        } else {
            Ok(if f.kind == "boolean" {
                format!("(({cell})->>'value')::pg_catalog.text")
            } else {
                format!("({cell})->>'text'")
            })
        }
    }
    fn continuation(&mut self, after: &After) -> Result<String> {
        if after.version != "weft.engine-temporal-after/0.2"
            || after.selected_key != self.plan.selected_key
            || after.sort_values.len() != self.plan.sort.len()
            || after.key_values.len() != self.plan.key_fields.len()
        {
            return Err((fail("Complete original continuation Key and tuple required")).into());
        }
        let start = self.plan.sort.len() - self.plan.key_fields.len();
        let mut repeated = std::collections::BTreeMap::new();
        let mut prefixes = vec![];
        let mut branches = vec![];
        for (i, (sort, state)) in self.plan.sort.iter().zip(&after.sort_values).enumerate() {
            if let Some(previous) = repeated.insert(sort.field, state) {
                if previous != state {
                    return Err(fail(
                        "Repeated sort source position has inconsistent continuation state/value",
                    )
                    .into());
                }
            }
            let f = &self.plan.fields[sort.field];
            let left = self.comparable(sort.field)?;
            let (equal, strict) = match state {
                State::Null {} | State::Missing {} => {
                    if matches!(state, State::Null {}) && !f.nullable
                        || matches!(state, State::Missing {}) && !f.missing
                    {
                        return Err(
                            (fail("Continuation state outside original row presence")).into()
                        );
                    }
                    (
                        format!("{left} IS NULL"),
                        if sort.nulls == "first" {
                            format!("{left} IS NOT NULL")
                        } else {
                            "FALSE".into()
                        },
                    )
                }
                State::Present { value } => {
                    if value.len() > 1_048_576 || value.contains('\0') {
                        return Err((fail("Bounded exact continuation text required")).into());
                    }
                    if front::is_temporal(&f.kind) && front::canonical(&f.kind, value)? != *value {
                        return Err((fail("Canonical temporal continuation required")).into());
                    }
                    if i >= start
                        && (sort.field != self.plan.key_fields[i - start]
                            || after.key_values[i - start] != *value)
                    {
                        return Err(
                            (fail("Complete selected Key suffix must agree with tuple")).into()
                        );
                    }
                    let right = self.text_slot(
                        value.clone(),
                        json!({"kind":"page-continuation","component":i,"identity":f.identity}),
                    )?;
                    let r = format!("({right}::pg_catalog.text) COLLATE \"C\"");
                    let op = if sort.direction == "desc" { "<" } else { ">" };
                    (
                        format!("{left} IS NOT DISTINCT FROM {r}"),
                        format!(
                            "({left} {op} {r}{})",
                            if sort.nulls == "last" {
                                format!(" OR {left} IS NULL")
                            } else {
                                String::new()
                            }
                        ),
                    )
                }
            };
            if i >= start && !matches!(state, State::Present { .. }) {
                return Err((fail("Selected Key suffix values must be present")).into());
            }
            branches.push(format!(
                "({}{strict})",
                if prefixes.is_empty() {
                    String::new()
                } else {
                    format!("{} AND ", prefixes.join(" AND "))
                }
            ));
            prefixes.push(equal);
        }
        Ok(format!("({})", branches.join(" OR ")))
    }
}
fn digest_token(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
}
fn validate<'a>(
    r: &front::Request,
    plan: &Plan,
    b: &'a Binding,
    raw: &Value,
    catalog: &weft_core::model::Catalog,
) -> Result<&'a RecordBinding> {
    if r.target.backend_id != BACKEND
        || r.target.backend_version != "0.4.47"
        || r.target.target_profile != "truss-runtime-pg18.6/0.7-temporal-candidate"
        || weft_core::json::sha256(r.target.binding_json.as_bytes()) != r.target.binding_sha256
    {
        return Err((fail("Exact temporal backend and original binding pin required")).into());
    }
    if b.interface_version != "truss-runtime-binding/0.5"
        || b.schema != "truss"
        || b.installation_id.is_empty()
        || b.release_id != "truss-catalog-runtime/0.76-temporal-candidate"
        || !digest_token(&b.release_sha256)
        || b.catalog_revision < 1
        || !(1..=10000).contains(&b.maximum_scan_rows)
        || !(1..=10000).contains(&b.maximum_result_rows)
        || !(1..=16777216).contains(&b.maximum_bytes)
        || b.records.is_empty()
        || b.records.len() > 1024
    {
        return Err(
            (fail("Exact fresh temporal capture and positive resource caps required")).into(),
        );
    }
    let abi = &raw["queryAbi"];
    if abi.as_object().is_none_or(|m| m.len() != 3)
        || abi["identity"] != "truss-postgresql-logical-query"
        || abi["version"] != "1.3"
        || !abi["sha256"].as_str().is_some_and(digest_token)
    {
        return Err((fail("Explicit native temporal ABI1.3 descriptor required")).into());
    }
    if !raw.get("relationships").is_some_and(Value::is_array)
        || raw["records"]
            .as_array()
            .is_none_or(|v| v.iter().any(|r| r.get("keys").is_none()))
    {
        return Err((fail("Complete captured version0.5 mapping required")).into());
    }
    if b.documents.len() != r.modules.len() {
        return Err((fail("Exact original model pin inventory required")).into());
    }
    let mut documents = BTreeSet::new();
    for p in &b.documents {
        if !documents.insert((&p.document_id, &p.revision))
            || !r.modules.iter().any(|m| {
                m.pin.document_id == p.document_id
                    && m.pin.revision == p.revision
                    && m.pin.sha256 == p.sha256
            })
        {
            return Err((fail("Original captured document pins differ")).into());
        }
    }
    // Identity's ordinary wire type is intentionally extensible. This closed
    // temporal capture cannot silently discard a selected identity qualifier.
    let exact_identity = |v: &Value| {
        v.as_object().is_some_and(|m| {
            m.len() == 4
                && ["documentId", "revision", "module", "element"]
                    .iter()
                    .all(|k| {
                        m.get(*k)
                            .is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty()))
                    })
        })
    };
    for rec in raw["records"]
        .as_array()
        .ok_or_else(|| fail("Original record inventory required"))?
    {
        if !exact_identity(&rec["identity"])
            || rec["fields"]
                .as_array()
                .is_none_or(|fields| fields.iter().any(|f| !exact_identity(&f["identity"])))
        {
            return Err(fail("Closed complete native source identities required").into());
        }
    }
    let mut records = BTreeSet::new();
    let mut native_types = BTreeSet::new();
    let mut native_fields = BTreeSet::new();
    for rec in &b.records {
        native(&rec.type_id)?;
        if !records.insert(key(&rec.identity)) || !native_types.insert(&rec.type_id) {
            return Err((fail("Unique native source mapping required")).into());
        }
        let source = catalog.record_by_identity(&rec.identity)?;
        let members = catalog.record_members(&source)?;
        if members.len() != rec.fields.len()
            || members.iter().any(|m| {
                rec.fields
                    .iter()
                    .filter(|f| f.identity == m.identity)
                    .count()
                    != 1
            })
        {
            return Err(
                fail("Captured Record must match its complete original member mapping").into(),
            );
        }
        if rec.identity == plan.record {
            for field in &plan.fields {
                if field.logical_type.is_some() {
                    let (_, graph) =
                        catalog.member_descriptor_by_identity(&source, &field.identity)?;
                    let descriptor = graph
                        .iter()
                        .find(|d| d.identity == field.identity)
                        .ok_or_else(|| fail("Original scalar sibling descriptor required"))?;
                    // Reuse the ordinary backend's domain admission. This profile
                    // adds finite temporal meaning, not new scalar facet support.
                    scalar_type(descriptor, true, true, true, true)?;
                }
            }
        }
        let authored = source.value["keys"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let mut native_keys = BTreeSet::new();
        if rec.keys.len() != authored.len() {
            return Err(fail("Complete captured authored Key inventory required").into());
        }
        for mapped in &rec.keys {
            if !native_keys.insert(&mapped.key_id) {
                return Err(fail("Unique captured Key mapping required").into());
            }
            let original = authored
                .iter()
                .find(|k| k["id"] == mapped.key_id)
                .ok_or_else(|| fail("Captured Key is not authored"))?;
            let mut properties = vec![];
            for field in original["fields"]
                .as_array()
                .ok_or_else(|| fail("Original Key Fields required"))?
            {
                let mapped = rec
                    .fields
                    .iter()
                    .find(|f| {
                        json!(f.identity.module) == field["module"]
                            && json!(f.identity.element) == field["element"]
                    })
                    .ok_or_else(|| fail("Original Key member mapping missing"))?;
                properties.push(mapped.property_id.clone());
            }
            if properties != mapped.property_ids {
                return Err(fail("Native Key property order differs from original source").into());
            }
        }
        for f in &rec.fields {
            native(&f.property_id)?;
            if !native_fields.insert(&f.property_id) {
                return Err((fail("Unique native property mapping required")).into());
            }
        }
    }
    let matched = b
        .records
        .iter()
        .filter(|r| r.identity == plan.record)
        .collect::<Vec<_>>();
    if matched.len() != 1 {
        return Err((fail("Unique original native Record mapping required")).into());
    }
    let record = matched[0];
    if record.fields.len() != plan.fields.len() {
        return Err((fail("Complete original Field mapping required")).into());
    }
    for f in &plan.fields {
        if record
            .fields
            .iter()
            .filter(|m| m.identity == f.identity)
            .count()
            != 1
        {
            return Err((fail("Exact full source identity correspondence required")).into());
        }
    }
    Ok(record)
}
pub fn compile_value(
    request: &Value,
    selected: Option<(&KeyRef, u16)>,
    after: &Value,
) -> Result<Value> {
    let (r, plan, catalog) = front::prepare(request, selected)?;
    let raw = weft_core::json::checked_truss_runtime_catalog_json(&r.target.binding_json).map_err(
        |code| Diagnostic::new(code, "binding", "Original temporal binding JSON refused"),
    )?;
    let binding: Binding = serde_json::from_value(raw.clone())
        .map_err(|_| fail("Closed original temporal binding required"))?;
    let record = validate(&r, &plan, &binding, &raw, &catalog)?;
    let mut emit = Emit {
        plan: &plan,
        record,
        slots: vec![],
    };
    let binding_slot = emit.text_slot(
        r.target.binding_json.clone(),
        json!({"kind":"runtime-binding"}),
    )?;
    let requirements = json!({"version":"truss.temporal-source-guard/0.2","record":plan.record,"fields":plan.fields.iter().filter(|f|front::is_temporal(&f.kind)).map(|f|json!({"field":f.identity,"kind":f.kind,"presence":{"missing":f.missing,"null":f.nullable}})).collect::<Vec<_>>()});
    let requirements_slot = emit.text_slot(
        weft_core::mutation::canonical(&requirements),
        json!({"kind":"temporal-source-requirements","record":plan.record}),
    )?;
    let key_sql = if selected.is_some() {
        let properties = plan
            .key_fields
            .iter()
            .map(|i| {
                record
                    .fields
                    .iter()
                    .find(|m| m.identity == plan.fields[*i].identity)
                    .map(|f| f.property_id.clone())
                    .ok_or_else(|| fail("Original selected Key property required").into())
            })
            .collect::<Result<Vec<_>>>()?;
        let keys = record
            .keys
            .iter()
            .filter(|k| k.key_id == plan.selected_key.key && k.property_ids == properties)
            .collect::<Vec<_>>();
        if keys.len() != 1 {
            return Err((fail("Captured native selected key_id must match authored Key")).into());
        }
        format!(",{}", literal(&keys[0].key_id)?)
    } else {
        String::new()
    };
    let function = if selected.is_some() {
        "person_query_temporal_page_objects"
    } else {
        "person_query_temporal_objects"
    };
    let from=format!("\"truss\".\"{function}\"({binding_slot}::pg_catalog.jsonb,{}::pg_catalog.int4{key_sql},{requirements_slot}::pg_catalog.jsonb)",record.type_id);
    // MATERIALIZED also prevents outer predicate/continuation pushdown. The native
    // STABLE source must buffer/validate its entire selected bag before returning.
    let mut filters = vec![];
    if let Some(p) = &plan.predicate {
        filters.push(emit.predicate(p)?)
    }
    let parsed_after: Option<After> = if after.is_null() {
        None
    } else {
        Some(
            serde_json::from_value(after.clone())
                .map_err(|_| fail("Closed whole temporal continuation required"))?,
        )
    };
    if let Some(after) = &parsed_after {
        if selected.is_none() {
            return Err((fail("Read route rejects continuation")).into());
        }
        filters.push(emit.continuation(after)?)
    }
    let mut outputs = plan.outputs.clone();
    let visible = outputs.len();
    let mut key_positions = vec![];
    let mut sort_slots = vec![];
    if selected.is_some() {
        for sort in &plan.sort {
            let pos = outputs
                .iter()
                .position(|o| o.field == sort.field)
                .unwrap_or_else(|| {
                    let mut name = format!("__weft_temporal_hidden_{}", outputs.len() + 1);
                    while outputs.iter().any(|o| o.name == name) {
                        name.push('_')
                    }
                    outputs.push(front::Output {
                        field: sort.field,
                        name,
                    });
                    outputs.len() - 1
                });
            sort_slots.push(json!({"field":plan.fields[sort.field].identity,"direction":sort.direction,"nulls":sort.nulls,"kind":plan.fields[sort.field].kind,"outputPosition":pos+1}));
        }
        for key in &plan.key_fields {
            key_positions.push(
                outputs
                    .iter()
                    .position(|o| o.field == *key)
                    .ok_or_else(|| fail("Hidden Key output absent"))?
                    + 1,
            )
        }
    }
    let mut columns = vec![];
    let mut descriptors = vec![];
    let mut projections = vec![];
    for (i, output) in outputs.iter().enumerate() {
        let f = &plan.fields[output.field];
        let representation = if front::is_temporal(&f.kind) || f.missing || f.nullable {
            json!({"kind":"value","descriptor":f.identity,"nativeNull":true})
        } else {
            json!({"kind":"scalar","logicalType":f.logical_type,"carrier":"text","decoder":decoder(&f.logical_type.as_ref().unwrap().family)})
        };
        projections.push(format!(
            "{} AS {}",
            emit.projection(output.field)?,
            Identifier::new(&output.name)?.sql()
        ));
        columns.push(json!({"position":i+1,"outputName":output.name,"representation":representation,"nullable":f.nullable||f.missing,"sourceIdentities":[f.identity]}));
        let mut descriptor = json!({"name":output.name,"carrier":f.kind,"nullable":f.nullable,"missing":f.missing,"source":f.identity,"representation":representation,"modelPins":plan.model_pins});
        if front::is_temporal(&f.kind) {
            descriptor["decoder"] = json!({"profile":"truss-temporal/0.2","kind":f.kind,"temporalProfile":front::PROFILE})
        }
        descriptors.push(descriptor);
    }
    let order = plan
        .sort
        .iter()
        .map(|s| {
            emit.comparable(s.field).map(|c| {
                format!(
                    "{c} {} NULLS {}",
                    s.direction.to_ascii_uppercase(),
                    s.nulls.to_ascii_uppercase()
                )
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut sql=format!("WITH \"__weft_temporal_source\" AS MATERIALIZED (SELECT id,props FROM {from}) SELECT {} FROM \"__weft_temporal_source\"",projections.join(","));
    if !filters.is_empty() {
        sql.push_str(&format!(" WHERE {}", filters.join(" AND ")))
    }
    if !order.is_empty() {
        sql.push_str(&format!(" ORDER BY {}", order.join(",")))
    }
    let limit = selected
        .map(|(_, size)| u32::from(size) + 1)
        .or(plan.limit.map(u32::from));
    if let Some(limit) = limit {
        if limit > binding.maximum_result_rows {
            return Err((fail("Requested result bound exceeds captured cap")).into());
        }
        sql.push_str(&format!(" LIMIT {limit}"))
    }
    if sql.len() > 1_048_576 {
        return Err(
            Diagnostic::new("WFT-LIMIT", "emit", "Emitted temporal SQL exceeds one MiB").into(),
        );
    }
    let source_function = if selected.is_some() {
        "truss.person_query_temporal_page_objects(jsonb,int,text,jsonb)"
    } else {
        "truss.person_query_temporal_objects(jsonb,int,jsonb)"
    };
    let checks = json!({"version":"weft.truss-temporal-domain-checks/0.2","mode":"native-validated-source","queryAbi":raw["queryAbi"],"maxScanRows":binding.maximum_scan_rows,"maxBytes":binding.maximum_bytes,"sources":[{"record":plan.record,"bindingParameter":1,"requirementsParameter":2,"sourceFunction":source_function,"selectedKey":selected.map(|(k,_)|k)}]});
    let page=selected.map(|(key,_)|json!({"version":"weft.engine-temporal-page/0.2","selectedKey":key,"visibleOutputPositions":(1..=visible).collect::<Vec<_>>(),"keyOutputPositions":key_positions,"sort":sort_slots}));
    let artifact = json!({"interfaceVersion":"weft-compile/0.2.10","status":"compiled","diagnostics":[],"dialect":"weft-sql/0.2.0","compilerVersion":concat!("weft/",env!("CARGO_PKG_VERSION")),"modelPins":plan.model_pins,"bindingSha256":r.target.binding_sha256,"backend":{"backendId":BACKEND,"backendVersion":"0.4.47","targetProfile":r.target.target_profile,"interfaceVersion":"weft-backend/0.2.10"},"targetContext":{"queryAbi":raw["queryAbi"],"releaseId":binding.release_id,"releaseSha256":binding.release_sha256,"installationId":binding.installation_id,"catalogRevision":binding.catalog_revision},"logicalPlan":plan,"sql":sql,"parameters":emit.slots,"columns":columns,"sourceDomainChecks":checks,"obligations":[{"id":"truss.runtime.temporal-source/0.2","owner":"host","failureCode":"WFT-OBLIGATION","parameters":{"source":"complete-buffered-statement-snapshot","authority":"fresh-capture-and-final-recheck","maximumResultRows":binding.maximum_result_rows}}],"qualification":{"status":"candidate","evidence":[],"assumptions":[],"operations":[]}});
    Ok(json!({"artifact":artifact,"outputs":descriptors,"page":page}))
}
