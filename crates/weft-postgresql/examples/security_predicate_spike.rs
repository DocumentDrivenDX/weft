// SPDX-License-Identifier: MIT OR Apache-2.0
//! Separate diagnostic transport; ordinary security compilation remains refused.
use std::io::{Read, Write};
use weft_core::error::{Diagnostic, Result};
fn refused() -> Diagnostic {
    Diagnostic::new("WFT-SECURITY-PREDICATE-SPIKE", "capability", "Bounded closed diagnostic input required")
}
// Fixture declarations only; original owner checking admits exact query correspondence.
fn inspect_column(input:&serde_json::Value,output:&serde_json::Value,probe:&serde_json::Value,position:usize)->Result<weft_core::security_lowering::SecurityResultColumn> {
 use weft_core::security_lowering::*;
 let object=probe.as_object().ok_or_else(refused)?;
 if object.len()!=3||object.keys().any(|k|!matches!(k.as_str(),"target"|"field"|"outputName")){return Err(refused());}
 fn identity(input:&serde_json::Value,r:&serde_json::Value)->Result<SecurityIdentity>{
  let document=r["documentId"].as_str().ok_or_else(refused)?;
  let pins=input["modules"].as_array().ok_or_else(refused)?;
  let selected=pins.iter().filter(|m|m["pin"]["documentId"]==document).collect::<Vec<_>>();if selected.len()!=1{return Err(refused());}
  Ok(SecurityIdentity{document_id:document.into(),revision:selected[0]["pin"]["revision"].as_str().ok_or_else(refused)?.into(),module:r["moduleId"].as_str().ok_or_else(refused)?.into(),element:r["elementId"].as_str().ok_or_else(refused)?.into()})
 }
 let profile=weft_core::json::checked_json(input["queryProfileJson"].as_str().ok_or_else(refused)?).map_err(|_|refused())?;
 let groups=output["eligibility"].as_array().ok_or_else(refused)?.iter().filter(|g|g["target"]==probe["target"]&&g["action"]==profile["action"]).collect::<Vec<_>>();if groups.len()!=1{return Err(refused());}
 let fields=groups[0]["disclosure"].as_array().ok_or_else(refused)?.iter().filter(|f|f["field"]==probe["field"]).collect::<Vec<_>>();if fields.len()!=1{return Err(refused());}
 let field=identity(input,&probe["field"])?;let target=identity(input,&probe["target"])?;
 let policy=weft_core::json::checked_json(input["policyJson"].as_str().ok_or_else(refused)?).map_err(|_|refused())?;
 let ontology=weft_core::json::checked_json(input["ontologyJson"].as_str().ok_or_else(refused)?).map_err(|_|refused())?;
 let classification=ontology["entities"].as_array().ok_or_else(refused)?.iter().chain(ontology["associations"].as_array().ok_or_else(refused)?).find(|e|e["type"]==probe["target"]).ok_or_else(refused)?["fields"].as_array().ok_or_else(refused)?.iter().find(|f|f["ref"]==probe["field"]).ok_or_else(refused)?;
 let mut original=classification["protection"]=="unprotected";let mut withheld=false;
 for rule in policy["rules"].as_array().ok_or_else(refused)? {if rule["effect"]!="permit"||!rule["target"].as_array().ok_or_else(refused)?.contains(&probe["target"])||!rule["actions"].as_array().ok_or_else(refused)?.contains(&profile["action"]){continue;}
  if let Some(disclosures)=rule["disclosure"].as_array(){for d in disclosures {if d["field"]==probe["field"]{original|=d["disposition"]["kind"]=="original";withheld|=d["disposition"]["kind"]=="withheld";}}}
 }
 let mut outcomes=Vec::new();if original{outcomes.push(SecurityResultOutcome::Original{id:"original".into(),domain:SecurityResultDomain::Model{field:field.clone()}});
  let doc=weft_core::json::checked_json(input["modules"].as_array().ok_or_else(refused)?.iter().find(|m|m["pin"]["documentId"]==field.document_id).ok_or_else(refused)?["documentJson"].as_str().ok_or_else(refused)?).map_err(|_|refused())?;
  let source=doc["modules"].as_array().ok_or_else(refused)?.iter().find(|m|m["id"]==field.module).ok_or_else(refused)?["elements"].as_array().ok_or_else(refused)?.iter().find(|e|e["id"]==field.element).ok_or_else(refused)?;
  if source["nullability"]=="absent-allowed"{outcomes.push(SecurityResultOutcome::Absent{id:"absent".into()});}
 }
 if withheld{outcomes.push(SecurityResultOutcome::Withheld{id:"withheld".into()});}
 let mut classes:std::collections::BTreeMap<String,(SecurityTransform,SecurityResultDomain,Vec<SecurityDispositionSource>)>=std::collections::BTreeMap::new();
 for t in fields[0]["transformations"].as_array().ok_or_else(refused)? {
  let d=&t["disposition"]["transformed"];let output_field=identity(input,&d["outputField"])?;
  let entry=classes.entry(t["outcomeId"].as_str().ok_or_else(refused)?.into()).or_insert((SecurityTransform::Constant{version:d["version"].as_str().ok_or_else(refused)?.into(),output_field:output_field.clone(),literal:d["literal"].clone()},SecurityResultDomain::Model{field:output_field},vec![]));
  entry.2.push(SecurityDispositionSource{rule_id:t["ruleId"].as_str().ok_or_else(refused)?.into(),target:target.clone(),field:field.clone()});
 }
 for (id,(transform,domain,disposition_sources)) in classes{outcomes.push(SecurityResultOutcome::Transformed{id,transform,domain,disposition_sources});}
 Ok(SecurityResultColumn{position,output_name:probe["outputName"].as_str().ok_or_else(refused)?.into(),source_fields:vec![field],outcomes})
}
fn inspect_probe(input:&serde_json::Value,output:&serde_json::Value,probe:&serde_json::Value)->Result<()> {
 use weft_core::security_lowering::*;
 let object=probe.as_object().ok_or_else(refused)?;
 if object.contains_key("factCut")!=object.contains_key("scopedFacts")||!object.contains_key("rows"){return Err(refused());}
 let paired=usize::from(object.contains_key("factCut"))*2;
 let columns=if let Some(specs)=object.get("columns") {
  if object.len()!=2+paired||object.keys().any(|k|!matches!(k.as_str(),"columns"|"rows"|"factCut"|"scopedFacts")){return Err(refused());}
  let specs=specs.as_array().ok_or_else(refused)?;
  if specs.is_empty()||specs.len()>256{return Err(refused());}
  // Conservative source expansion reservation before repeated declaration copies.
  let bytes=input.to_string().len().checked_add(output.to_string().len()).ok_or_else(refused)?;
  if bytes.checked_mul(specs.len()).ok_or_else(refused)?>16_000_000{return Err(refused());}
  specs.iter().enumerate().map(|(i,spec)|inspect_column(input,output,spec,i+1)).collect::<Result<Vec<_>>>()?
 } else {
  if object.len()!=4+paired||object.keys().any(|k|!matches!(k.as_str(),"target"|"field"|"outputName"|"rows"|"factCut"|"scopedFacts")){return Err(refused());}
  let spec=serde_json::json!({"target":probe["target"],"field":probe["field"],"outputName":probe["outputName"]});
  vec![inspect_column(input,output,&spec,1)?]
 };
 let contract=SecurityResultContract{version:"weft.security.result-contract/0.1.0".into(),encoding:"weft.security.cells/0.1.0".into(),columns};
 let contract_json=serde_json::to_string(&contract).map_err(|_|refused())?;let hash=weft_core::json::sha256(contract_json.as_bytes());
 let batch=serde_json::json!({"version":"weft.security.cells/0.1.0","resultContractSha256":hash,"rows":probe["rows"]});
 if let (Some(cut),Some(rows))=(object.get("factCut"),object.get("scopedFacts")) {
  weft_core::security_mapping_transport::inspect_security_mapping_result_facts(&input.to_string(),&contract,&contract_json,&hash,&batch.to_string(),&cut.to_string(),&rows.to_string())
 } else {
  weft_core::security_mapping_transport::inspect_security_mapping_result_cells(&input.to_string(),&contract,&contract_json,&hash,&batch.to_string())
 }
}
fn run() -> Result<()> {
    let mut raw = String::new();
    std::io::stdin().take(16_000_001).read_to_string(&mut raw).map_err(|_| refused())?;
    if raw.len() > 16_000_000 { return Err(refused()); }
    let value = weft_core::json::checked_json(&raw).map_err(|_| refused())?;
    let object = value.as_object().ok_or_else(refused)?;
    // Source-only semantic inspection does not need a physical assessment/render.
    if object.len()==2&&object.contains_key("request")&&object.contains_key("subjectKeyProbe") {
        weft_core::security_mapping_transport::inspect_security_mapping_subject_key(&value["request"].to_string(),&value["subjectKeyProbe"].to_string())?;
        let output=serde_json::json!({"securityReady":false,"subjectKeyInspectionPassed":true});
        let encoded=serde_json::to_vec(&output).map_err(|_|refused())?;
        std::io::stdout().lock().write_all(&encoded).and_then(|_|std::io::stdout().lock().write_all(b"\n")).map_err(|_|refused())?;
        return Ok(());
    }
    if object.keys().any(|k| !matches!(k.as_str(), "request" | "assessment" | "cut" | "evaluationRequest" | "resultProbe" | "subjectKeyProbe"))
        || !object.contains_key("request") || !object.contains_key("assessment")
        || object.contains_key("cut") != object.contains_key("evaluationRequest") { return Err(refused()); }
    let input = &value["request"];
    let catalog = weft_core::model::Catalog::prepare_security(
        serde_json::from_value(input["modules"].clone()).map_err(|_| refused())?)?;
    let source = weft_core::security_source::SecuritySourcePacket::read(
        input["policyJson"].as_str().ok_or_else(refused)?,
        input["ontologyJson"].as_str().ok_or_else(refused)?, &catalog)?;
    let plan = weft_core::security_ir::SecurityLogicalPlan::read(source, &catalog)?;
    let mut output = weft_postgresql::security_predicate_spike::render(&plan, &catalog, &value["assessment"])?;
    if object.contains_key("cut") {
        let results = weft_core::security_evaluation::simulate_json(&plan, &catalog,
            &value["cut"].to_string(), &value["evaluationRequest"].to_string())?;
        output["simulationDisclosures"] = serde_json::to_value(results.iter().map(|r| &r.disclosure).collect::<Vec<_>>()).map_err(|_| refused())?;
        output["simulationDecisions"] = serde_json::json!(results.iter().map(|r| format!("{:?}", r.decision).to_lowercase()).collect::<Vec<_>>());
    }
    if let Some(probe)=object.get("resultProbe") { inspect_probe(input,&output,probe)?;output["resultCellInspectionPassed"]=serde_json::json!(true);if probe.get("factCut").is_some(){output["conditionalFactSelectionPassed"]=serde_json::json!(true);} }
    if let Some(selection)=object.get("subjectKeyProbe") {
        weft_core::security_mapping_transport::inspect_security_mapping_subject_key(&input.to_string(),&selection.to_string())?;
        output["subjectKeyInspectionPassed"]=serde_json::json!(true);
    }
    let encoded = serde_json::to_vec(&output).map_err(|_| refused())?;
    if encoded.len() > 16_000_000 { return Err(refused()); }
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&encoded).and_then(|_| stdout.write_all(b"\n")).map_err(|_| refused())
}
fn main() {
    if let Err(error) = run() {
        if let Ok(encoded) = serde_json::to_string(&error) { eprintln!("{encoded}"); }
        std::process::exit(1);
    }
}
