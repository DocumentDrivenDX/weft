//! Current Truss tagged-value selection. Pure compiler; host owns cuts and execution.
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use weft_core::{
    application_ir as app,
    application_model::Shape,
    backend::{BindingInput, Context, Plan, Selection, TargetProfile},
    ir::{Family, Identity, LogicalType, ModelPin},
    model::{Catalog, ModuleInput},
    mutation::{Artifact, MResult, MutationDiagnostic},
    mutation_backend::*,
};
fn fail(message: &str) -> MutationDiagnostic {
    MutationDiagnostic {
        code: "WFT-MUTATION-TRUSS-CURRENT".into(),
        phase: "action".into(),
        path: String::new(),
        message: message.into(),
    }
}
fn owner<T>(v: weft_core::error::Result<T>) -> MResult<T> {
    v.map_err(|e| fail(&e.message))
}
pub struct CurrentTrussMutationBackend {
    model: Artifact,
    binding: Artifact,
    catalog: Catalog,
    evidence: Vec<String>,
}
impl CurrentTrussMutationBackend {
    pub fn new(
        model: Artifact,
        binding: Artifact,
        model_pin: ModelPin,
        evidence: Vec<String>,
    ) -> MResult<Self> {
        let document = model.read()?;
        let b = binding.read()?;
        if evidence.is_empty()
            || evidence.len() > 32
            || evidence.iter().any(|s| s.is_empty() || s.len() > 4096)
            || document["umf"] != "0.8.0"
            || (b["releaseId"] != "truss-catalog-runtime/0.65-candidate" && b["releaseId"] != "truss-catalog-runtime/0.66-candidate")
            || b["interfaceVersion"] != "truss-runtime-binding/0.4"
        {
            return Err(fail("Original current model/binding/evidence required"));
        }
        let docs = b["documents"]
            .as_array()
            .filter(|d| d.len() == 1)
            .ok_or_else(|| fail("One original accepted document required"))?;
        let d = &docs[0];
        if d["documentId"] != document["id"] || d["sha256"] != model.sha256 {
            return Err(fail(
                "Accepted document bytes differ from original mutation model",
            ));
        }
        let pin = ModelPin {
            document_id: d["documentId"]
                .as_str()
                .ok_or_else(|| fail("Document identity required"))?
                .into(),
            revision: d["revision"]
                .as_str()
                .ok_or_else(|| fail("Accepted revision required"))?
                .into(),
            umf_version: "0.8.0".into(),
            sha256: model.sha256.clone(),
        };
        if model_pin != pin {
            return Err(fail(
                "Original action model revision must equal accepted native revision",
            ));
        }
        let modules = document["modules"]
            .as_array()
            .ok_or_else(|| fail("Original modules required"))?
            .iter()
            .map(|m| {
                m["id"]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| fail("Module identity required"))
            })
            .collect::<MResult<Vec<_>>>()?;
        let catalog = owner(Catalog::prepare_security(vec![ModuleInput {
            document_json: model.utf8.clone(),
            pin,
            selected_module_ids: modules,
        }]))?;
        Ok(Self {
            model,
            binding,
            catalog,
            evidence,
        })
    }
    fn build(
        &self,
        selection: &Artifact,
        model: &Artifact,
        binding: &Artifact,
        max: u32,
    ) -> MResult<SelectionLowering> {
        if !(1..=10000).contains(&max)
            || model.sha256 != self.model.sha256
            || model.utf8 != self.model.utf8
            || binding.sha256 != self.binding.sha256
            || binding.utf8 != self.binding.utf8
        {
            return Err(fail("Original registered source bytes/maxItems required"));
        }
        let keys = inspect_selection(selection, model)?;
        let program = selection.read()?;
        let b = binding.read()?;
        let pin = &self.catalog.inputs()[0].pin;
        let identity = Identity {
            document_id: pin.document_id.clone(),
            revision: pin.revision.clone(),
            module: program["record"]["module"]
                .as_str()
                .ok_or_else(|| fail("Record module required"))?
                .into(),
            element: program["record"]["element"]
                .as_str()
                .ok_or_else(|| fail("Record identity required"))?
                .into(),
        };
        let record = owner(self.catalog.record_by_identity(&identity))?;
        let entity = owner(self.catalog.entity_descriptor(&record))?;
        let coverage = Selection {
            records: vec![identity.clone()],
            fields: entity.members.iter().map(|m| m.identity.clone()).collect(),
            types: entity.graph.iter().map(|d| d.identity.clone()).collect(),
            relationships: vec![],
        };
        let plan = app::Plan {
            ir_version: "weft-ir/0.2.0".into(),
            module_pins: self.catalog.pins(),
            read_profile: None,
            required_capabilities: vec![],
            type_graph: entity.graph.clone(),
            source: app::Scan {
                occurrence: "weft_mutation".into(),
                record: identity.clone(),
                pin: pin.clone(),
            },
            page_key: None,
            joins: vec![],
            filters: vec![],
            groups: vec![],
            aggregate: false,
            outputs: vec![],
            order: vec![],
            limit: None,
        };
        let target = TargetProfile {
            id: crate::truss_runtime::PAGE_TARGET.into(),
            engine: "postgresql".into(),
            engine_version: "18.6".into(),
            session_settings: json!({"server_encoding":"UTF8","search_path":"pg_catalog,pg_temp","read_context":"live"}),
            storage_layout_revision: "truss-runtime-keyset-query/0.4".into(),
            publication_revision: "caller-transaction/0.1".into(),
        };
        let input = BindingInput {
            profile: "truss-runtime-binding/0.4".into(),
            json: binding.utf8.clone(),
            sha256: binding.sha256.clone(),
        };
        let context = Context {
            catalog: &self.catalog,
            plan: Plan::V02(&plan),
            target: &target,
            binding: &input,
            binding_value: &b,
            selection: &coverage,
        };
        let mapping = owner(crate::truss_runtime::admit_mutation_mapping(&context))?;
        let tid = owner(crate::truss_runtime::mutation_type(&mapping, &identity))?;
        let source = "\"truss\".\"object\" AS \"weft_mutation\"";
        let scope = format!("\"weft_mutation\".type_id={tid}::pg_catalog.int4");
        let mut needed: BTreeSet<String> =
            keys.iter().map(weft_core::mutation::canonical).collect();
        let mut stack = vec![&program["predicate"]];
        while let Some(node) = stack.pop() {
            if let Some(compare) = node.get("compare") {
                needed.insert(weft_core::mutation::canonical(&compare["field"]));
            } else {
                for child in node
                    .get("and")
                    .or_else(|| node.get("or"))
                    .and_then(Value::as_array)
                    .ok_or_else(|| fail("Original predicate required"))?
                {
                    stack.push(child);
                }
            }
        }
        let mut fields = BTreeMap::new();
        let mut preflight=vec![SelectionPreflight{sql:format!("SELECT CASE WHEN pg_catalog.count(*)>{} THEN '1' ELSE '0' END::pg_catalog.text AS violations FROM {source} WHERE {scope}",b["maximumScanRows"].as_u64().ok_or_else(||fail("Scan bound required"))?),parameters:vec![]},SelectionPreflight{sql:format!("SELECT pg_catalog.count(*)::pg_catalog.text AS violations FROM {source} WHERE {scope} AND (\"weft_mutation\".id IS NULL OR \"weft_mutation\".id<=0 OR \"weft_mutation\".ver IS NULL OR \"weft_mutation\".ver<=0)"),parameters:vec![]}];
        for member in &entity.members {
            let reference =
                json!({"module":member.identity.module,"element":member.identity.element});
            let key = weft_core::mutation::canonical(&reference);
            if !needed.contains(&key) {
                continue;
            }
            let descriptor = entity
                .graph
                .iter()
                .find(|d| d.identity == member.identity)
                .ok_or_else(|| fail("Original field descriptor missing"))?;
            let Shape::Scalar { logical_type } = &descriptor.shape else {
                return Err(fail("Current selection admits scalar fields only"));
            };
            if descriptor.availability.as_deref() != Some("required") {
                return Err(fail("Current selection requires original required fields"));
            }
            let (expression, carrier, valid) = owner(crate::truss_runtime::mutation_scalar(
                &mapping,
                &identity,
                &member.identity,
                logical_type,
            ))?;
            preflight.push(SelectionPreflight{sql:format!("SELECT pg_catalog.count(*)::pg_catalog.text AS violations FROM {source} WHERE {scope} AND ({valid}) IS NOT TRUE"),parameters:vec![]});
            fields.insert(key, (expression, carrier, logical_type.clone()));
        }
        if fields.len() != needed.len() {
            return Err(fail("Complete original selection field mapping required"));
        }
        fn render(
            node: &Value,
            fields: &BTreeMap<String, (String, String, LogicalType)>,
            params: &mut Vec<SelectionParameter>,
        ) -> MResult<String> {
            if let Some(c) = node.get("compare") {
                let (expression, _, logical) = fields
                    .get(&weft_core::mutation::canonical(&c["field"]))
                    .ok_or_else(|| fail("Predicate field mapping absent"))?;
                let op = match c["op"].as_str() {
                    Some("eq") => "=",
                    Some("ne") => "<>",
                    Some("lt") => "<",
                    Some("le") => "<=",
                    Some("gt") => ">",
                    Some("ge") => ">=",
                    _ => return Err(fail("Predicate operator refused")),
                };
                if c["value"]["string"]
                    .as_str()
                    .is_some_and(|s| s.contains('\0'))
                {
                    return Err(fail("PostgreSQL text parameter contains unsupported NUL"));
                }
                let index = params.len() + 1;
                params.push(SelectionParameter {
                    name: format!("p{index}"),
                    field: c["field"].clone(),
                    value: c["value"].clone(),
                });
                let cast = match logical.family {
                    Family::String => "pg_catalog.text COLLATE pg_catalog.\"C\"",
                    Family::Boolean => "pg_catalog.bool",
                    _ => return Err(fail("Current predicate scalar domain refused")),
                };
                Ok(format!("({expression}) {op} (${index}::{cast})"))
            } else {
                let (children, op) = if let Some(x) = node.get("and") {
                    (x, " AND ")
                } else {
                    (&node["or"], " OR ")
                };
                Ok(format!(
                    "({})",
                    children
                        .as_array()
                        .ok_or_else(|| fail("Original predicate children required"))?
                        .iter()
                        .map(|n| render(n, fields, params))
                        .collect::<MResult<Vec<_>>>()?
                        .join(op)
                ))
            }
        }
        let key_expressions = keys
            .iter()
            .map(|k| {
                fields
                    .get(&weft_core::mutation::canonical(k))
                    .map(|f| f.0.clone())
                    .ok_or_else(|| fail("Key comparator mapping missing"))
            })
            .collect::<MResult<Vec<_>>>()?;
        preflight.push(SelectionPreflight{sql:format!("SELECT pg_catalog.count(*)::pg_catalog.text AS violations FROM (SELECT 1 FROM {source} WHERE {scope} GROUP BY {} HAVING pg_catalog.count(*)>1) duplicate_keys",key_expressions.join(",")),parameters:vec![]});
        let mut parameters = vec![];
        let predicate = render(&program["predicate"], &fields, &mut parameters)?;
        let mut projection = vec![
            "\"weft_mutation\".id::pg_catalog.text AS identity".to_string(),
            "\"weft_mutation\".ver::pg_catalog.text AS version".into(),
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
        for (index, field) in keys.iter().enumerate() {
            let (_, carrier, _) = fields
                .get(&weft_core::mutation::canonical(field))
                .ok_or_else(|| fail("Original key mapping required"))?;
            let name = format!("key{index}");
            projection.push(format!("({carrier})::pg_catalog.text AS {name}"));
            columns.push(SelectionColumn {
                name,
                kind: "key".into(),
                field: Some(field.clone()),
            });
        }
        Ok(SelectionLowering::Lowered{sql:format!("SELECT {} FROM {source} WHERE ({scope}) AND ({predicate}) ORDER BY \"weft_mutation\".id ASC LIMIT {}",projection.join(", "),max+1),parameters,columns,preflight,ordering:SelectionOrdering{kind:"native-integer-identity".into(),direction:"asc".into()},bounds:SelectionBounds{max_items:max,fetch_limit:max+1},obligations:vec!["Current PostgreSQL18.6/layout1.0/tagged runtime0.65/0.66 original binding; same original document bytes retain accepted native revision separately from action compilation revision".into(),"Host must admit original authenticated authorized coherent source/caller cut, run every complete-population preflight before selection, reject any violations and maxItems+1 overflow without trimming".into(),"No connection, raw caller SQL, RR facade, action execution, authorization or publication is issued by this compiler".into()]})
    }
}
impl SelectionBackend for CurrentTrussMutationBackend {
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
                    evidence: self.evidence.clone(),
                })
                .collect(),
        })
    }
    fn assess_selection(
        &self,
        s: &Artifact,
        m: &Artifact,
        b: &Artifact,
    ) -> MResult<SelectionAssessment> {
        Ok(match self.build(s, m, b, 1) {
            Ok(_) => SelectionAssessment {
                status: SelectionStatus::Supported,
                diagnostics: vec![],
            },
            Err(d) => SelectionAssessment {
                status: SelectionStatus::Unsupported,
                diagnostics: vec![d],
            },
        })
    }
    fn lower_selection(
        &self,
        s: &Artifact,
        m: &Artifact,
        b: &Artifact,
        max: u32,
    ) -> MResult<SelectionLowering> {
        Ok(match self.build(s, m, b, max) {
            Ok(v) => v,
            Err(d) => SelectionLowering::Blocked {
                diagnostics: vec![d],
            },
        })
    }
}
