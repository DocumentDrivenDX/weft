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
    local: Option<crate::truss_browser_binding::BrowserMapping>,
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
            || !matches!(b["releaseId"].as_str(), Some("truss-catalog-runtime/0.65-candidate" | "truss-catalog-runtime/0.66-candidate" | "truss-catalog-runtime/0.67-candidate" | "truss-catalog-runtime/0.68-candidate"))
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
            local: None,
        })
    }
    /// Separately admitted browser correspondence, never native release admission.
    pub fn new_browser(model:Artifact,binding:Artifact,pin:ModelPin)->MResult<Self>{
        let document=model.read()?;
        if document["umf"]!="0.8.0"||document["id"]!=pin.document_id||pin.sha256!=model.sha256{return Err(fail("Original browser model pin required"));}
        let modules=document["modules"].as_array().ok_or_else(||fail("Original modules required"))?.iter()
            .map(|m|m["id"].as_str().map(str::to_owned).ok_or_else(||fail("Module identity required"))).collect::<MResult<Vec<_>>>()?;
        let catalog=owner(Catalog::prepare_security(vec![ModuleInput{document_json:model.utf8.clone(),pin:pin.clone(),selected_module_ids:modules}]))?;
        let local=crate::truss_browser_binding::BrowserMapping::admit(&binding,&model,&pin,&catalog)?;
        Ok(Self{model,binding,catalog,evidence:vec!["Browser local original catalog correspondence only; host-issued owner/read grant and current store cut required".into()],local:Some(local)})
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
        navigation:None,
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
        let mapping = if self.local.is_none(){Some(owner(crate::truss_runtime::admit_mutation_mapping(&context))?)}else{None};
        let tid = if let Some(local)=&self.local{local.type_id(&identity)?}else{owner(crate::truss_runtime::mutation_type(mapping.as_ref().unwrap(), &identity))?};
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
        let mut browser_validations=Vec::new();
        let mut preflight=vec![SelectionPreflight{sql:format!("SELECT CASE WHEN pg_catalog.count(*)>{} THEN '1' ELSE '0' END::pg_catalog.text AS violations FROM {source} WHERE {scope}",b["maximumScanRows"].as_u64().ok_or_else(||fail("Scan bound required"))?),parameters:vec![]},SelectionPreflight{sql:format!("SELECT pg_catalog.count(*)::pg_catalog.text AS violations FROM {source} WHERE {scope} AND (\"weft_mutation\".id IS NULL OR \"weft_mutation\".id<=0 OR \"weft_mutation\".ver IS NULL OR \"weft_mutation\".ver<=0)"),parameters:vec![]}];
        for member in &entity.members {
            let reference =
                json!({"module":member.identity.module,"element":member.identity.element});
            let key = weft_core::mutation::canonical(&reference);
            if self.local.is_none() && !needed.contains(&key) {
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
            let (expression, carrier, valid) = if let Some(local)=&self.local{
                local.scalar(&identity,&member.identity,logical_type)?
            }else{owner(crate::truss_runtime::mutation_scalar(mapping.as_ref().unwrap(),&identity,&member.identity,logical_type))?};
            if self.local.is_some(){browser_validations.push(format!("({valid}) IS NOT TRUE"));}
            else{preflight.push(SelectionPreflight{sql:format!("SELECT pg_catalog.count(*)::pg_catalog.text AS violations FROM {source} WHERE {scope} AND ({valid}) IS NOT TRUE"),parameters:vec![]});}
            fields.insert(key, (expression, carrier, logical_type.clone()));
        }
        if !browser_validations.is_empty(){preflight.push(SelectionPreflight{sql:format!("SELECT pg_catalog.count(*)::pg_catalog.text AS violations FROM {source} WHERE {scope} AND ({})",browser_validations.join(" OR ")),parameters:vec![]});}
        if (self.local.is_none() && fields.len() != needed.len()) || needed.iter().any(|key|!fields.contains_key(key)) {
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
        Ok(SelectionLowering::Lowered{sql:format!("SELECT {} FROM {source} WHERE ({scope}) AND ({predicate}) ORDER BY \"weft_mutation\".id ASC LIMIT {}",projection.join(", "),max+1),parameters,columns,preflight,ordering:SelectionOrdering{kind:"native-integer-identity".into(),direction:"asc".into()},bounds:SelectionBounds{max_items:max,fetch_limit:max+1},obligations:vec![if self.local.is_some(){"Browser PGlite0.5.10/PostgreSQL18.6/layout1.0; original catalog mapping and complete Record population preflight, no native release claim".into()}else if self.binding.read()?["releaseId"] == "truss-catalog-runtime/0.68-candidate" { "Current PostgreSQL18.6/layout1.0/tagged runtime0.68 exact original binding; rootless selection only, no owned/provider/security capability inferred".into() } else if self.binding.read()?["releaseId"] == "truss-catalog-runtime/0.67-candidate" { "Current PostgreSQL18.6/layout1.0/tagged runtime0.67 original binding; same original document bytes retain accepted native revision separately from action compilation revision".into() } else if self.binding.read()?["releaseId"] == "truss-catalog-runtime/0.66-candidate" { "Current PostgreSQL18.6/layout1.0/tagged runtime0.66 original binding; same original document bytes retain accepted native revision separately from action compilation revision".into() } else { "Current PostgreSQL18.6/layout1.0/tagged runtime0.65 original binding; same original document bytes retain accepted native revision separately from action compilation revision".into() },if self.local.is_some(){"Host must hold original issued local owner and independent read grant over the same current store/incarnation/catalog/model cut; run every complete-population preflight before selection and refuse violations/overflow without trimming".into()}else{"Host must admit original authenticated authorized coherent source/caller cut, run every complete-population preflight before selection, reject any violations and maxItems+1 overflow without trimming".into()},"No connection, raw caller SQL, RR facade, action execution, authorization or publication is issued by this compiler".into()]})
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

#[cfg(test)]
mod native066_tests {
    use super::*;
    #[test]
    fn original_products_native066_selection_lowers_without_repinning_sources() {
        let fixture:Value=serde_json::from_str(include_str!("../../../tests/mutation/fixtures/truss066-products.json")).unwrap();
        let model:Artifact=serde_json::from_value(fixture["model"].clone()).unwrap();
        let binding:Artifact=serde_json::from_value(fixture["binding"].clone()).unwrap();
        let selection:Artifact=serde_json::from_value(fixture["selection"].clone()).unwrap();
        let pin:ModelPin=serde_json::from_value(fixture["modelPin"].clone()).unwrap();
        let historical:Artifact=serde_json::from_value(fixture["historicalNative65Binding"].clone()).unwrap();
        let old=CurrentTrussMutationBackend::new(model.clone(),historical.clone(),pin.clone(),vec!["Original historical65 compiler fixture only".into()]).unwrap();
        let old_result=serde_json::to_value(old.lower_selection(&selection,&model,&historical,2).unwrap()).unwrap();
        assert_eq!(old_result["status"],"lowered", "{old_result}");
        assert_eq!(old_result["obligations"][0],"Current PostgreSQL18.6/layout1.0/tagged runtime0.65 original binding; same original document bytes retain accepted native revision separately from action compilation revision");
        let backend=CurrentTrussMutationBackend::new(model.clone(),binding.clone(),pin.clone(),vec!["Original Products native66 public catalog export; compiler-only evidence".into()]).unwrap();
        let result=serde_json::to_value(backend.lower_selection(&selection,&model,&binding,2).unwrap()).unwrap();
        assert_eq!(result["status"],"lowered", "{result}");
        assert_eq!(result["bounds"],json!({"maxItems":2,"fetchLimit":3}));
        assert!(result["sql"].as_str().unwrap().contains("\"truss\".\"object\""));
        assert_eq!(result["parameters"][0]["value"],json!({"string":"bar"}));
        assert!(result["obligations"][0].as_str().unwrap().contains("runtime0.66"));
        // A runtime label cannot bypass native66's original manifest hash.
        let mut changed=binding.read().unwrap();changed["releaseSha256"]=json!("0".repeat(64));
        let utf8=changed.to_string();let bad=Artifact{media_type:"application/json".into(),sha256:weft_core::json::sha256(utf8.as_bytes()),utf8};
        let backend=CurrentTrussMutationBackend::new(model.clone(),bad.clone(),pin,vec!["Changed manifest refusal".into()]).unwrap();
        let refused=serde_json::to_value(backend.lower_selection(&selection,&model,&bad,2).unwrap()).unwrap();
        assert_eq!(refused["status"],"blocked");
        // A recognized0.68 registration also cannot upgrade a wrong original
        // manifest hash into qualified native selection lowering.
        let mut changed=bad.read().unwrap();changed["releaseId"]=json!("truss-catalog-runtime/0.68-candidate");
        let utf8=changed.to_string();let native068=Artifact{media_type:"application/json".into(),sha256:weft_core::json::sha256(utf8.as_bytes()),utf8};
        let pin:ModelPin=serde_json::from_value(fixture["modelPin"].clone()).unwrap();
        let backend=CurrentTrussMutationBackend::new(model.clone(),native068.clone(),pin,vec!["Recognized original68 source; wrong native manifest hash".into()]).unwrap();
        let refused=serde_json::to_value(backend.lower_selection(&selection,&model,&native068,2).unwrap()).unwrap();
        assert_eq!(refused["status"],"blocked");
    }
}

#[cfg(test)]
mod browser_tests {
 use super::*;
 #[test]
 fn browser_mapping_lowers_original_selection_with_complete_record_preflight(){
  let fixture:Value=serde_json::from_str(include_str!("../../../tests/mutation/fixtures/local-products-input.json")).unwrap();
  let mut registry=weft_core::mutation::MutationRegistry::from_json(&fixture["configuration"].to_string()).unwrap();
  let response:Value=serde_json::from_str(&registry.compile_json(&fixture["request"].to_string())).unwrap();
  let plan:Value=serde_json::from_str(response["plan"]["utf8"].as_str().unwrap()).unwrap();
  let model:Artifact=serde_json::from_value(fixture["request"]["model"].clone()).unwrap();
  let pin:ModelPin=serde_json::from_value(fixture["request"]["modelPin"].clone()).unwrap();
  let identity=|element:&str|json!({"documentId":pin.document_id,"revision":pin.revision,"module":"sales","element":element});
  let value=json!({"interfaceVersion":"truss-browser-binding/0.1","pgliteVersion":"0.5.10","postgresqlVersion":"18.6",
   "store":{"profile":"truss-browser-layout/0.1","origin":"http://localhost","namespace":"truss-browser:products","storeId":"12345678-1234-4234-8234-123456789abc","incarnation":"22345678-1234-4234-8234-123456789abc","layoutSha256":"c203d09418fb4eddda829d791dda11c20341227c2d1d82e86d4c24afda5e7294"},
   "catalogRevision":"1","document":pin,"maximumScanRows":1000,
   "record":{"identity":identity("products"),"typeId":"1","fields":[{"identity":identity("product-id"),"propertyId":"1"},{"identity":identity("product-name"),"propertyId":"2"}],"key":{"keyId":"pk","propertyIds":["1"]}}});
  let artifact=|v:&Value|{let utf8=v.to_string();Artifact{media_type:"application/json".into(),sha256:weft_core::json::sha256(utf8.as_bytes()),utf8}};
  let binding=artifact(&value);
  let backend=CurrentTrussMutationBackend::new_browser(model.clone(),binding.clone(),pin.clone()).unwrap();
  let selection:Artifact=serde_json::from_value(plan["selection"]["artifact"].clone()).unwrap();
  let handle=registry.install_selection_backend(Box::new(backend)).unwrap();
  let result=serde_json::to_value(registry.lower_selection(handle,&selection,&model,&binding,2).unwrap()).unwrap();
  assert_eq!(result["status"],"lowered","{result}");
  assert_eq!(result["preflight"].as_array().unwrap().len(),4);
  assert!(result["preflight"][2]["sql"].as_str().unwrap().contains("props->'2'"));
  // The public owner prepared this synthetic64-member model; use the full outer registry.
  let wide_fixture:Value=serde_json::from_str(include_str!("../../../tests/mutation/fixtures/local-products-64-input.json")).unwrap();
  let mut outer=weft_core::mutation::MutationRegistry::from_json(&wide_fixture["configuration"].to_string()).unwrap();
  let wide_model:Artifact=serde_json::from_value(wide_fixture["request"]["model"].clone()).unwrap();
  let wide_pin:ModelPin=serde_json::from_value(wide_fixture["request"]["modelPin"].clone()).unwrap();
  let response:Value=serde_json::from_str(&outer.compile_json(&wide_fixture["request"].to_string())).unwrap();
  let wide_plan:Value=serde_json::from_str(response["plan"]["utf8"].as_str().unwrap()).unwrap();
  let wide_selection:Artifact=serde_json::from_value(wide_plan["selection"]["artifact"].clone()).unwrap();
  let mut wide_binding=value.clone();wide_binding["document"]=json!(wide_pin);
  for n in 3..=64{wide_binding["record"]["fields"].as_array_mut().unwrap().push(json!({"identity":identity(&format!("extra-{n}")),"propertyId":n.to_string()}));}
  let wide_binding=artifact(&wide_binding);
  let backend=CurrentTrussMutationBackend::new_browser(wide_model.clone(),wide_binding.clone(),wide_pin).unwrap();
  let handle=outer.install_selection_backend(Box::new(backend)).unwrap();
  let result=serde_json::to_value(outer.lower_selection(handle,&wide_selection,&wide_model,&wide_binding,2).unwrap()).unwrap();
  assert_eq!(result["status"],"lowered","{result}");assert_eq!(result["preflight"].as_array().unwrap().len(),4);
  assert!(result["preflight"][2]["sql"].as_str().unwrap().contains("props->'64'"));
  let mut wrong=value;wrong["record"]["key"]["propertyIds"]=json!(["2"]);
  assert!(CurrentTrussMutationBackend::new_browser(model,artifact(&wrong),pin).is_err());
 }
}
