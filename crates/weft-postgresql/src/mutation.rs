// SPDX-License-Identifier: MIT OR Apache-2.0
//! Original-admission PostgreSQL selection lowering; no connection, guard or authorization issuer.
use crate::native_comparator_definition::{Operation, Strategy};
use crate::{
    binding::Admission,
    original_backend::OriginalBackend,
    property_definition::HomeAdmission,
    qualified,
    row_join_definition::{Column, RecordKind},
    Identifier,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    ir::{Family, Identity, ModelPin},
    model::Catalog,
    mutation::{Artifact, MResult, MutationDiagnostic},
    mutation_backend::*,
};
/// Original inventory interpretation supplied by trusted host registration.
/// Host must qualify exact native int8 columns/source custody before executing output.
pub struct NativeObjectColumns {
    pub inventory: crate::leaf_codec_definition::OriginalArtifact,
    pub relation_identity: String,
    pub identity: Column,
    pub version: Column,
    pub evidence: Vec<String>,
}
pub struct OriginalMutationBackend {
    original: OriginalBackend,
    model: Artifact,
    binding: Artifact,
    pin: ModelPin,
    catalog: Catalog,
    columns: NativeObjectColumns,
}
fn fail(code: &str, message: &str) -> MutationDiagnostic {
    MutationDiagnostic {
        code: format!("WFT-MUTATION-{code}"),
        phase: if code == "LIMIT" { "limit" } else { "action" }.into(),
        path: String::new(),
        message: message.into(),
    }
}
fn original<T>(result: weft_core::error::Result<T>) -> MResult<T> {
    result.map_err(|_| {
        fail(
            "ACTION",
            "Original PostgreSQL mapping/codec admission refused",
        )
    })
}
fn literal(text: &str) -> MResult<String> {
    if text.contains('\0') {
        return Err(fail(
            "ACTION",
            "Native PostgreSQL text cannot represent NUL",
        ));
    }
    Ok(format!("'{}'", text.replace('\'', "''")))
}
impl OriginalMutationBackend {
    pub fn new(
        backend: OriginalBackend,
        catalog: Catalog,
        binding: Artifact,
        columns: NativeObjectColumns,
    ) -> MResult<Self> {
        if catalog.inputs().len() != 1 || columns.evidence.is_empty() {
            return Err(fail(
                "ACTION",
                "One original model and native selection evidence required",
            ));
        }
        let input = &catalog.inputs()[0];
        if input.pin.umf_version != "0.10.0" {
            return Err(fail("ACTION", "Original core0.10 input required"));
        }
        let model = Artifact {
            media_type: "application/json".into(),
            utf8: input.document_json.clone(),
            sha256: input.pin.sha256.clone(),
        };
        model.read()?;
        if binding.media_type != "application/json"
            || binding.sha256 != backend.binding_sha256
            || binding.sha256 != weft_core::json::sha256(binding.utf8.as_bytes())
        {
            return Err(fail("ACTION", "Original native binding pin differs"));
        }
        let value: Value = original_json(&binding)?;
        let admitted = original(Admission::parse(
            &binding.utf8,
            value["bindingProfileId"]
                .as_str()
                .ok_or_else(|| fail("ACTION", "Binding profile missing"))?,
        ))?;
        if admitted.artifacts.get("/basis/layoutInventory") != Some(&columns.inventory.bytes)
            || value["basis"]["layoutInventory"]["identity"] != columns.inventory.identity
            || columns.identity.relation_identity != columns.relation_identity
            || columns.version.relation_identity != columns.relation_identity
            || columns.identity.name == columns.version.name
        {
            return Err(fail(
                "ACTION",
                "Native identity/version inventory custody differs",
            ));
        }
        original(Identifier::new(&columns.identity.name))?;
        original(Identifier::new(&columns.version.name))?;
        let inventory: Value = weft_core::json::checked_json(
            std::str::from_utf8(&columns.inventory.bytes)
                .map_err(|_| fail("ACTION", "Native inventory UTF8 refused"))?,
        )
        .map_err(|_| fail("ACTION", "Native inventory JSON refused"))?;
        if inventory["interfaceVersion"] != "truss-mutation-read-columns/0.1.0"
            || inventory["relationIdentity"] != columns.relation_identity
            || inventory["relationName"] != "object"
        {
            return Err(fail("ACTION", "Original mutation column profile refused"));
        }
        for selected in [&columns.identity, &columns.version] {
            let matches: Vec<_> = inventory["columns"]
                .as_object()
                .ok_or_else(|| fail("ACTION", "Native inventory columns absent"))?
                .values()
                .filter(|c| {
                    c["name"] == selected.name
                        && c["relationIdentity"] == selected.relation_identity
                        && c["nativeType"] == "pg_catalog.int8"
                })
                .collect();
            if matches.len() != 1 {
                return Err(fail(
                    "ACTION",
                    "Native id/version must select exact int8 source columns",
                ));
            }
        }
        let pin = input.pin.clone();
        Ok(Self {
            original: backend,
            model,
            binding,
            pin,
            catalog,
            columns,
        })
    }
    fn build(
        &self,
        selection: &Artifact,
        model: &Artifact,
        binding: &Artifact,
        max_items: u32,
    ) -> MResult<SelectionLowering> {
        if model != &self.model || binding != &self.binding {
            return Err(fail(
                "ACTION",
                "Registered original model/binding bytes differ",
            ));
        }
        if !(1..=10000).contains(&max_items) {
            return Err(fail("LIMIT", "Selection maxItems exceeded"));
        }
        weft_core::mutation_backend::inspect_selection(selection, model)?;
        let program = selection.read()?;
        let document = model.read()?;
        if program["protocol"] != "weft.update-selection/1" {
            return Err(fail("ACTION", "Exact selection profile required"));
        }
        let identity = Identity {
            document_id: self.pin.document_id.clone(),
            revision: self.pin.revision.clone(),
            module: program["record"]["module"]
                .as_str()
                .ok_or_else(|| fail("ACTION", "Record module missing"))?
                .into(),
            element: program["record"]["element"]
                .as_str()
                .ok_or_else(|| fail("ACTION", "Record identity missing"))?
                .into(),
        };
        let record = original(self.catalog.record_by_identity(&identity))?;
        let key = original(
            self.catalog.authored_key(
                &record,
                program["key"]["key"]
                    .as_str()
                    .ok_or_else(|| fail("ACTION", "Original Key missing"))?,
            ),
        )?;
        if program["key"]["module"] != identity.module
            || program["key"]["element"] != identity.element
        {
            return Err(fail("ACTION", "Selection Record/Key differs"));
        }
        let read = self
            .original
            .records
            .get(&serde_json::to_string(&identity).unwrap())
            .ok_or_else(|| fail("ACTION", "Original Record admission absent"))?;
        let b = original_json(binding)?;
        let admitted = original(Admission::parse(
            &binding.utf8,
            b["bindingProfileId"].as_str().unwrap_or(""),
        ))?;
        original(read.verify_key_mapping(&admitted, &self.catalog, &key))?;
        let (mapping, type_id) = read.mutation_mapping();
        if mapping.record_kind != RecordKind::Object {
            return Err(fail(
                "ACTION",
                "Mutation read only admits original object homes",
            ));
        }
        // Fixed Truss object roles are explicit host selections, never inferred from a business name.
        if read.mutation_relation_identity() != self.columns.relation_identity {
            return Err(fail(
                "ACTION",
                "Identity/version columns belong to another Record source",
            ));
        }
        let namespace = original(Identifier::new(
            b["basis"]["namespace"]
                .as_str()
                .ok_or_else(|| fail("ACTION", "Original namespace absent"))?,
        ))?;
        let alias = original(Identifier::new("weft_mutation"))?;
        let source = format!(
            "{} AS {}",
            qualified(&namespace, &mapping.relation),
            alias.sql()
        );
        let scope = format!(
            "{}.{} = {}::pg_catalog.int4",
            alias.sql(),
            mapping.discriminator_column.sql(),
            type_id
        );
        let id = format!(
            "{}.{}",
            alias.sql(),
            original(Identifier::new(&self.columns.identity.name))?.sql()
        );
        let version = format!(
            "{}.{}",
            alias.sql(),
            original(Identifier::new(&self.columns.version.name))?.sql()
        );
        let mut refs = Vec::new();
        let mut operations: BTreeMap<String, BTreeSet<Operation>> = BTreeMap::new();
        let mut work = vec![(&program["predicate"], 1)];
        let mut nodes = 0;
        while let Some((node, depth)) = work.pop() {
            if depth > 24 {
                return Err(fail("LIMIT", "Selection depth exceeded"));
            }
            if let Some(c) = node.get("compare") {
                nodes += 1;
                if nodes > 256 {
                    return Err(fail("LIMIT", "Comparison bound exceeded"));
                }
                let operation = match c["op"].as_str() {
                    Some("eq" | "ne") => Operation::Equality,
                    Some("lt" | "le" | "gt" | "ge") => Operation::Ordering,
                    _ => return Err(fail("ACTION", "Unknown comparison operation")),
                };
                operations
                    .entry(weft_core::mutation::canonical(&c["field"]))
                    .or_default()
                    .insert(operation);
                if !refs.contains(&c["field"]) {
                    refs.push(c["field"].clone());
                }
            } else {
                let cs = node
                    .get("and")
                    .or_else(|| node.get("or"))
                    .and_then(Value::as_array)
                    .filter(|cs| (2..=64).contains(&cs.len()))
                    .ok_or_else(|| fail("ACTION", "Typed predicate node required"))?;
                work.extend(cs.iter().rev().map(|c| (c, depth + 1)));
            }
        }
        for field in &key.fields {
            let r = json!({"module":field.module,"element":field.element});
            if !refs.contains(&r) {
                refs.push(r);
            }
        }
        if refs.len() >= 64 {
            return Err(fail("LIMIT", "Native preflight count exceeded"));
        }
        let mut fields = BTreeMap::new();
        let mut preflight = Vec::new();
        preflight.push(SelectionPreflight{sql:format!("SELECT count(*)::pg_catalog.text AS violations FROM {source} WHERE ({scope}) AND ({id} IS NULL OR {id}<=0 OR {version} IS NULL OR {version}<=0)"),parameters:vec![]});
        for reference in refs {
            let field = Identity {
                document_id: identity.document_id.clone(),
                revision: identity.revision.clone(),
                module: reference["module"]
                    .as_str()
                    .ok_or_else(|| fail("ACTION", "Field module missing"))?
                    .into(),
                element: reference["element"]
                    .as_str()
                    .ok_or_else(|| fail("ACTION", "Field identity missing"))?
                    .into(),
            };
            let property = self
                .original
                .properties
                .get(&crate::comparator_requirements::registration_key(
                    &identity, &field,
                ))
                .ok_or_else(|| fail("ACTION", "Original property admission absent"))?;
            original(read.verify_property(property))?;
            let (_, descriptors) =
                original(self.catalog.member_descriptor_by_identity(&record, &field))?;
            let descriptor = descriptors
                .iter()
                .find(|d| d.identity == field)
                .ok_or_else(|| fail("ACTION", "Original Field descriptor missing"))?;
            let weft_core::application_model::Shape::Scalar { logical_type } = &descriptor.shape
            else {
                return Err(fail("ACTION", "Compound mutation read unsupported"));
            };
            if logical_type.nullable || descriptor.availability.as_deref() != Some("required") {
                return Err(fail("ACTION", "Required scalar source required"));
            }
            let definition = document["modules"]
                .as_array()
                .and_then(|ms| ms.iter().find(|m| m["id"] == field.module))
                .and_then(|m| m["elements"].as_array())
                .and_then(|es| es.iter().find(|e| e["id"] == field.element))
                .ok_or_else(|| fail("ACTION", "Original Field declaration absent"))?;
            if definition.get("allowedValues").is_some()
                || definition["extensions"]
                    .as_object()
                    .is_some_and(|e| !e.is_empty())
                || logical_type.family == Family::Integer
                    && logical_type
                        .facets
                        .as_object()
                        .is_none_or(|f| f.len() != 1 || !f.contains_key("integerWidth"))
                || logical_type.family != Family::Integer
                    && logical_type
                        .facets
                        .as_object()
                        .is_some_and(|f| !f.is_empty())
            {
                return Err(fail(
                    "ACTION",
                    "Native mutation domain refinements require separate qualification",
                ));
            }
            let HomeAdmission::Props { member, .. } = &property.home else {
                return Err(fail(
                    "ACTION",
                    "Native row mutation selection needs separate qualification",
                ));
            };
            // Reuse the original scalar codec and source-home admission. Metadata member/type constants are admitted and quoted.
            let mut slots = crate::Parameters::default();
            let location = original(property.home.props_location(&alias, &mut slots))?;
            let replace = |text: &str| -> MResult<String> {
                Ok(text.replace(
                    "$1::pg_catalog.text",
                    &format!("{}::pg_catalog.text", literal(member)?),
                ))
            };
            let storage = original(property.value.props_scalar_storage(&location))?
                .ok_or_else(|| fail("ACTION", "Original scalar codec absent"))?;
            let carrier = replace(&storage.carrier)?;
            let physical = replace(&storage.storage_integrity)?;
            let comparator =
                self.original
                    .comparators
                    .get(&crate::comparator_requirements::registration_key(
                        &identity, &field,
                    ));
            if let Some(required) = operations.get(&weft_core::mutation::canonical(&reference)) {
                let admitted = comparator
                    .ok_or_else(|| fail("ACTION", "Original predicate comparator absent"))?;
                original(admitted.require_type(logical_type))?;
                for operation in required {
                    original(admitted.require(*operation))?;
                }
                let declaration: Value = weft_core::json::checked_json(&admitted.original_json)
                    .map_err(|_| fail("ACTION", "Original comparator JSON refused"))?;
                let expected = match logical_type.family {
                    Family::Boolean => (Strategy::Boolean, "native-boolean", "pg_catalog.bool"),
                    Family::String => (Strategy::UnicodeText, "unicode-text-C", "pg_catalog.text"),
                    Family::Integer => (
                        admitted.strategy.clone(),
                        if logical_type.facets["integerWidth"]["signed"] == true {
                            "signed-integer"
                        } else {
                            "unsigned-integer"
                        },
                        admitted.native_type.as_str(),
                    ),
                    _ => return Err(fail("ACTION", "Comparator family unsupported")),
                };
                if admitted.strategy != expected.0
                    || declaration["strategy"]["kind"] != expected.1
                    || declaration["strategy"]["nativeType"] != expected.2
                    || admitted.native_type != expected.2
                {
                    return Err(fail(
                        "ACTION",
                        "Original closed comparator strategy differs",
                    ));
                }
            }
            let logical = match logical_type.family {
                Family::Boolean => format!("({carrier})::pg_catalog.bool"),
                Family::String => format!("({carrier})::pg_catalog.text COLLATE pg_catalog.\"C\""),
                Family::Integer => format!("({carrier})::pg_catalog.numeric"),
                _ => {
                    return Err(fail(
                        "ACTION",
                        "Key/native scalar family requires separate mutation qualification",
                    ))
                }
            };
            let mut domain = physical;
            if logical_type.family == Family::Integer {
                let comparator = self
                    .original
                    .comparators
                    .get(&crate::comparator_requirements::registration_key(
                        &identity, &field,
                    ))
                    .ok_or_else(|| fail("ACTION", "Original integer comparator absent"))?;
                original(comparator.require_type(logical_type))?;
                if comparator.native_type != "pg_catalog.numeric" {
                    return Err(fail(
                        "ACTION",
                        "Integer operator domain needs separate native qualification",
                    ));
                }

                let lexical=format!("({carrier}) ~ '^-?(0|[1-9][0-9]*)([.][0-9]+)?([eE][+-]?[0-9]+)?$' AND pg_catalog.pg_input_is_valid(({carrier}),'pg_catalog.numeric')");
                let range = original(comparator.numeric_domain_sql(&carrier))?;
                domain = format!("({domain}) AND CASE WHEN {lexical} THEN {range} ELSE FALSE END");
            }
            // Every referenced Key/predicate value is checked over the complete owner population before WHERE.
            preflight.push(SelectionPreflight{sql:format!("SELECT count(*)::pg_catalog.text AS violations FROM {source} WHERE ({scope}) AND ({domain}) IS DISTINCT FROM TRUE"),parameters:vec![]});
            fields.insert(
                weft_core::mutation::canonical(&reference),
                Field {
                    reference,
                    expression: logical,
                    carrier,
                    family: logical_type.family.clone(),
                    definition: definition.clone(),
                },
            );
        }
        let mut parameters = Vec::new();
        let predicate = render(&program["predicate"], &fields, &mut parameters, 1)?;
        let mut projection = vec![
            format!("{id}::pg_catalog.text AS identity"),
            format!("{version}::pg_catalog.text AS version"),
        ];
        let mut columns = vec![
            SelectionColumn {
                name: "identity".into(),
                kind: "identity".into(),
                field: None,
            },
            SelectionColumn {
                name: "version".into(),
                kind: "version".into(),
                field: None,
            },
        ];
        for (index, field) in key.fields.iter().enumerate() {
            let reference = json!({"module":field.module,"element":field.element});
            let read = fields
                .get(&weft_core::mutation::canonical(&reference))
                .unwrap();
            let name = format!("key{index}");
            projection.push(format!("({})::pg_catalog.text AS {name}", read.carrier));
            columns.push(SelectionColumn {
                name,
                kind: "key".into(),
                field: Some(reference),
            });
        }
        Ok(SelectionLowering::Lowered{sql:format!("SELECT {} FROM {source} WHERE ({scope}) AND ({predicate}) ORDER BY {id} ASC LIMIT {}",projection.join(", "),max_items+1),parameters,columns,preflight,ordering:SelectionOrdering{kind:"native-integer-identity".into(),direction:"asc".into()},bounds:SelectionBounds{max_items,fetch_limit:max_items+1},obligations:vec!["Original admitted authorized coherent snapshot and source correspondence before all preflight/selection".into(),"Host must abort on any nonzero preflight and refuse maxItems+1 overflow without trimming".into(),"Native column int8/nonreuse/version, CoreLiteral decoding and guard issuance remain separately qualified".into()]})
    }
}
fn original_json(binding: &Artifact) -> MResult<Value> {
    if binding.utf8.len() > 4194304 {
        return Err(fail("LIMIT", "Binding bytes exceeded"));
    }
    weft_core::json::checked_json(&binding.utf8)
        .map_err(|_| fail("ACTION", "Original binding JSON refused"))
}
struct Field {
    reference: Value,
    expression: String,
    carrier: String,
    family: Family,
    definition: Value,
}
fn render(
    node: &Value,
    fields: &BTreeMap<String, Field>,
    parameters: &mut Vec<SelectionParameter>,
    depth: usize,
) -> MResult<String> {
    if depth > 24 {
        return Err(fail("LIMIT", "Predicate depth exceeded"));
    }
    if let Some(c) = node.get("compare") {
        let field = fields
            .get(&weft_core::mutation::canonical(&c["field"]))
            .ok_or_else(|| fail("ACTION", "Predicate Field admission absent"))?;
        let op = match c["op"].as_str() {
            Some("eq") => "=",
            Some("ne") => "<>",
            Some("lt") if field.family != Family::Boolean => "<",
            Some("le") if field.family != Family::Boolean => "<=",
            Some("gt") if field.family != Family::Boolean => ">",
            Some("ge") if field.family != Family::Boolean => ">=",
            _ => return Err(fail("ACTION", "Predicate operator unsupported")),
        };
        let value = &c["value"];
        original(weft_core::security_literals::check_literal(
            &field.definition,
            value,
        ))?;
        if field.family == Family::String
            && value["string"].as_str().is_some_and(|s| s.contains('\0'))
        {
            return Err(fail(
                "ACTION",
                "Native string parameter cannot represent NUL",
            ));
        }
        if field.family == Family::Integer {
            let token = value["integerToken"]
                .as_str()
                .ok_or_else(|| fail("ACTION", "Integer token absent"))?;
            // An explicit native representation subset, independent of exact mathematical-domain admission.
            // Preserve the original token; refuse representations outside the qualified PostgreSQL cast range.
            let exponent = token.split(['e', 'E']).nth(1);
            if token.len() > 256
                || exponent.is_some_and(|e| {
                    e.parse::<i32>()
                        .map_or(true, |e| !(-100..=100).contains(&e))
                })
            {
                return Err(fail(
                    "ACTION",
                    "Integer parameter representation needs separate native qualification",
                ));
            }
        }
        if parameters.len() >= 1024 {
            return Err(fail("LIMIT", "Parameter count exceeded"));
        }
        let position = parameters.len() + 1;
        parameters.push(SelectionParameter {
            name: format!("p{position}"),
            field: field.reference.clone(),
            value: value.clone(),
        });
        let cast = match field.family {
            Family::Boolean => "pg_catalog.bool",
            Family::String => "pg_catalog.text COLLATE pg_catalog.\"C\"",
            Family::Integer => "pg_catalog.numeric",
            _ => return Err(fail("ACTION", "Parameter family unsupported")),
        };
        return Ok(format!("({} {op} ${position}::{cast})", field.expression));
    }
    let (and, children) = if let Some(cs) = node.get("and") {
        (true, cs)
    } else {
        (false, &node["or"])
    };
    let cs = children
        .as_array()
        .filter(|cs| (2..=64).contains(&cs.len()))
        .ok_or_else(|| fail("ACTION", "Typed predicate required"))?;
    let parts = cs
        .iter()
        .map(|c| render(c, fields, parameters, depth + 1))
        .collect::<MResult<Vec<_>>>()?;
    Ok(format!(
        "({})",
        parts.join(if and { " AND " } else { " OR " })
    ))
}
impl SelectionBackend for OriginalMutationBackend {
    fn describe(&self) -> MResult<SelectionCapabilities> {
        Ok(SelectionCapabilities {
            profile: SelectionProfile {
                id: "weft.update-selection".into(),
                version: "1".into(),
            },
            claims: ["selection.interpretation", "selection.completeRead"]
                .iter()
                .map(|id| SelectionClaim {
                    id: (*id).into(),
                    status: SelectionStatus::Supported,
                    evidence: self.columns.evidence.clone(),
                })
                .collect(),
        })
    }
    fn assess_selection(
        &self,
        selection: &Artifact,
        model: &Artifact,
        binding: &Artifact,
    ) -> MResult<SelectionAssessment> {
        match self.build(selection, model, binding, 1) {
            Ok(_) => Ok(SelectionAssessment {
                status: SelectionStatus::Supported,
                diagnostics: vec![],
            }),
            Err(diagnostic) => Ok(SelectionAssessment {
                status: SelectionStatus::Unsupported,
                diagnostics: vec![diagnostic],
            }),
        }
    }
    fn lower_selection(
        &self,
        selection: &Artifact,
        model: &Artifact,
        binding: &Artifact,
        max_items: u32,
    ) -> MResult<SelectionLowering> {
        match self.build(selection, model, binding, max_items) {
            Ok(value) => Ok(value),
            Err(diagnostic) => Ok(SelectionLowering::Blocked {
                diagnostics: vec![diagnostic],
            }),
        }
    }
}
