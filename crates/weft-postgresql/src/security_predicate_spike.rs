//! Diagnostic native predicate lowering from the original admitted IR.
//! Not a registered security backend, result contract, or execution permit.
use std::collections::BTreeMap;
use serde_json::{Value,json};
use weft_core::{error::{Diagnostic,Result},model::Catalog,security_ir::{SecurityLogicalPlan,Expression,Term,Binding,Effect,Disposition},security_ontology::SecurityRef};
fn reference(v:&Value)->Result<SecurityRef>{serde_json::from_value(v.clone()).map_err(|_|fail())}
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-PREDICATE-SPIKE","capability","Native predicate correspondence unavailable")}
#[derive(Clone)]
struct Cell{known:String,sql:String,family:String}
struct Tuple{cells:Vec<Cell>,identity:Option<(SecurityRef,String)>}
struct Render<'a>{inventory:&'a Value,ontology:&'a Value,documents:Vec<Value>,parameters:Vec<Value>,parameter_bytes:usize,work:usize}
impl Render<'_>{
 fn step(&mut self)->Result<()>{if self.work==0{return Err(fail());}self.work-=1;Ok(())}
 fn one<'a>(&self,rows:Vec<&'a Value>)->Result<&'a Value>{if rows.len()!=1{return Err(fail());}Ok(rows[0])}
 fn type_id(&self,r:&SecurityRef)->Result<i64>{let row=self.one(self.inventory["types"].as_array().ok_or_else(fail)?.iter().filter(|v|v["retired_rev"].is_null()&&v["document_id"]==r.document_id&&v["module"]==r.module_id&&v["element"]==r.element_id&&v["kind"]=="record").collect())?;let id=row["type_id"].as_i64().ok_or_else(fail)?;if !(1..=i32::MAX as i64).contains(&id){return Err(fail());}Ok(id)}
 fn field(&self,r:&SecurityRef)->Result<&Value>{let doc=self.one(self.documents.iter().filter(|d|d["id"]==r.document_id).collect())?;let module=self.one(doc["modules"].as_array().ok_or_else(fail)?.iter().filter(|m|m["id"]==r.module_id).collect())?;self.one(module["elements"].as_array().ok_or_else(fail)?.iter().filter(|f|f["id"]==r.element_id&&f["kind"]=="field").collect())}
 fn cell(&self,target:&SecurityRef,r:&SecurityRef,alias:&str)->Result<Cell>{
  let tid=self.type_id(target)?;let row=self.one(self.inventory["properties"].as_array().ok_or_else(fail)?.iter().filter(|p|p["retired_rev"].is_null()&&p["type_id"]==tid&&p["definition_document_id"]==r.document_id&&p["declaration_module"]==r.module_id&&p["element"]==r.element_id&&p["home"]=="json").collect())?;
  let id=row["prop_id"].as_i64().ok_or_else(fail)?;if !(1..=i32::MAX as i64).contains(&id){return Err(fail());}
  let field=self.field(r)?;let family=field["scalarType"].as_str().ok_or_else(fail)?;if !matches!(family,"string"|"boolean"){return Err(fail());}
  let raw=format!("{alias}.props->'{id}'");let nullable=field["nullability"]=="absent-allowed";
  let known=format!("truss.valid_scalar_cell({raw},'{family}',{nullable}) IS TRUE");
  let sql=if family=="string"{format!("(CASE WHEN {raw}->>'kind'='null' THEN NULL ELSE {raw}->>'text' END) COLLATE pg_catalog.\"C\"")}else{format!("CASE WHEN {raw}->>'kind'='null' THEN NULL ELSE {raw}->'value' END")};
  Ok(Cell{known,sql,family:family.into()})
 }
 fn key(&self,target:&SecurityRef,key:&str)->Result<Vec<SecurityRef>>{
  let tid=self.type_id(target)?;let row=self.one(self.inventory["keys"].as_array().ok_or_else(fail)?.iter().filter(|k|k["type_id"]==tid&&k["key_id"]==key&&k["retired_rev"].is_null()).collect())?;
  let mut result=Vec::new();for id in row["prop_ids"].as_array().ok_or_else(fail)?{let p=self.one(self.inventory["properties"].as_array().ok_or_else(fail)?.iter().filter(|p|p["type_id"]==tid&&p["prop_id"]==*id&&p["retired_rev"].is_null()).collect())?;result.push(SecurityRef{document_id:p["definition_document_id"].as_str().ok_or_else(fail)?.into(),module_id:p["declaration_module"].as_str().ok_or_else(fail)?.into(),element_id:p["element"].as_str().ok_or_else(fail)?.into()});}if result.is_empty()||result.len()>64{return Err(fail());}Ok(result)
 }
 fn binding(&self,b:&Binding,resource:&SecurityRef,vars:&BTreeMap<usize,SecurityRef>)->Result<(SecurityRef,String)>{Ok(match b{Binding::Subject=>(reference(&self.ontology["subject"])? ,"s".into()),Binding::Resource=>(resource.clone(),"r".into()),Binding::Variable(slot)=>(vars.get(slot).ok_or_else(fail)?.clone(),format!("v{slot}"))})}
 fn term(&mut self,t:&Term,resource:&SecurityRef,vars:&BTreeMap<usize,SecurityRef>)->Result<Tuple>{self.step()?;match t{
  Term::Field{binding,field,..}=>{let (owner,alias)=self.binding(binding,resource,vars)?;Ok(Tuple{cells:vec![self.cell(&owner,field,&alias)?],identity:None})},
  Term::Constant{domain,literal,..}=>{let family=domain["scalarType"].as_str().ok_or_else(fail)?;if !matches!(family,"string"|"boolean")||self.parameters.len()>=1024{return Err(fail());}let original=if literal.is_null(){literal}else{&literal[family]};let bytes=match original{Value::String(text)=>text.bytes().try_fold(2usize,|n,b|n.checked_add(if b<32{6}else if b==b'"'||b==b'\\'{2}else{1})).ok_or_else(fail)?,Value::Bool(_)=>5,Value::Null=>4,_=>return Err(fail())};self.parameter_bytes=self.parameter_bytes.checked_add(bytes).ok_or_else(fail)?;if self.parameter_bytes>1_048_576{return Err(fail());}let value=original.clone();if !(value.is_null()||family=="string"&&value.is_string()||family=="boolean"&&value.is_boolean()){return Err(fail());}self.parameters.push(value);let slot=self.parameters.len();let sql=if family=="string"{format!("${slot}::pg_catalog.text COLLATE pg_catalog.\"C\"")}else{format!("pg_catalog.to_jsonb(${slot}::pg_catalog.bool)")};Ok(Tuple{cells:vec![Cell{known:"TRUE".into(),sql,family:family.into()}],identity:None})},
  Term::Identity{binding,target,key_id}=>{let (owner,alias)=self.binding(binding,resource,vars)?;if &owner!=target{return Err(fail());}let refs=self.key(target,key_id)?;Ok(Tuple{cells:refs.iter().map(|r|self.cell(target,r,&alias)).collect::<Result<_>>()?,identity:Some((target.clone(),key_id.clone()))})},
  Term::Endpoint{binding,association,role,target,key_id}=>{let a=association.record().ok_or_else(fail)?;let (owner,alias)=self.binding(binding,resource,vars)?;if &owner!=a{return Err(fail());}let declaration=self.one(self.ontology["associations"].as_array().ok_or_else(fail)?.iter().filter(|v|reference(&v["type"]).ok().as_ref()==Some(a)).collect())?;let endpoint=self.one(declaration["endpoints"].as_array().ok_or_else(fail)?.iter().filter(|e|e["role"]==*role).collect())?;if reference(&endpoint["target"])?!=*target{return Err(fail());}let refs=endpoint["fields"].as_array().ok_or_else(fail)?.iter().map(reference).collect::<Result<Vec<_>>>()?;if refs.len()!=self.key(target,key_id)?.len(){return Err(fail());}Ok(Tuple{cells:refs.iter().map(|r|self.cell(a,r,&alias)).collect::<Result<_>>()?,identity:Some((target.clone(),key_id.clone()))})},
  Term::Context{..}=>Err(fail())}}
 fn expression(&mut self,e:&Expression,resource:&SecurityRef,vars:&BTreeMap<usize,SecurityRef>,depth:usize)->Result<String>{self.step()?;if depth>32{return Err(fail());}let sql=match e{
  Expression::Literal(v)=>if *v{"TRUE".into()}else{"FALSE".into()},
  Expression::Equal(a,b)=>{let a=self.term(a,resource,vars)?;let b=self.term(b,resource,vars)?;if a.cells.len()!=b.cells.len(){return Err(fail());}let known=a.cells.iter().chain(&b.cells).map(|c|format!("({})",c.known)).collect::<Vec<_>>().join(" AND ");let mut parts=Vec::new();for (a,b) in a.cells.iter().zip(&b.cells){if a.family!=b.family{return Err(fail());}parts.push(format!("({}) IS NOT DISTINCT FROM ({})",a.sql,b.sql));}let equality=if a.identity!=b.identity{"FALSE".into()}else{parts.join(" AND ")};format!("CASE WHEN {known} THEN ({equality}) ELSE NULL::pg_catalog.bool END")},
  Expression::And(args)|Expression::Or(args)=>{if args.is_empty()||args.len()>64{return Err(fail());}let separator=if matches!(e,Expression::And(_)){" AND "}else{" OR "};args.iter().map(|a|self.expression(a,resource,vars,depth+1).map(|s|format!("({s})"))).collect::<Result<Vec<_>>>()?.join(separator)},
  Expression::Not(a)=>format!("NOT ({})",self.expression(a,resource,vars,depth+1)?),
  Expression::Exists{slot,association,condition}=>{if vars.contains_key(slot){return Err(fail());}let a=association.record().ok_or_else(fail)?;let tid=self.type_id(a)?;let mut nested=vars.clone();nested.insert(*slot,a.clone());let condition=self.expression(condition,resource,&nested,depth+1)?;format!("(SELECT CASE WHEN pg_catalog.bool_or(t.truth IS TRUE) THEN TRUE WHEN pg_catalog.bool_or(t.truth IS NULL) THEN NULL::pg_catalog.bool ELSE FALSE END FROM (SELECT ({condition}) AS truth FROM truss.object AS v{slot} WHERE v{slot}.type_id={tid}) AS t)")}
 };if sql.len()>1_048_576{return Err(fail());}Ok(sql)}
}
/// Requires actual owner plan/catalog objects and a truthful native assessment.
/// Subjects are explicit fixture rows, not authenticated principals. Complete
/// field-domain-valid facts and valid declared witness keys are prerequisites;
/// the SQL expression does not itself establish native fact admission.
pub fn render(plan:&SecurityLogicalPlan,catalog:&Catalog,assessment:&Value)->Result<Value>{
 plan.require_catalog(catalog)?;if assessment["modelPins"]!=serde_json::to_value(catalog.pins()).map_err(|_|fail())?{return Err(fail());}
 let documents=catalog.inputs().iter().map(|i|weft_core::json::checked_json(&i.document_json).map_err(|_|fail())).collect::<Result<Vec<_>>>()?;
 let mut renderer=Render{inventory:&assessment["inventory"],ontology:plan.source().ontology(),documents,parameters:vec![],parameter_bytes:0,work:8192};let mut rules=Vec::new();let mut total=0;
 for (index,rule) in plan.rules().iter().enumerate(){let expression=renderer.expression(&rule.condition,&rule.target,&BTreeMap::new(),0)?;total+=expression.len();if total>4_000_000{return Err(fail());}rules.push(json!({"id":rule.id,"target":rule.target,"effect":rule.effect,"actions":rule.actions,"expression":expression,"truthColumn":format!("q{index}")}));}
 let mut group_bytes=0usize;
 let mut groups:BTreeMap<(SecurityRef,String),Vec<(usize,Effect)>>=BTreeMap::new();
 for (index,rule) in plan.rules().iter().enumerate(){for action in &rule.actions{renderer.step()?;group_bytes=group_bytes.checked_add(rule.target.document_id.len()+rule.target.module_id.len()+rule.target.element_id.len()+action.len()).ok_or_else(fail)?;if group_bytes>1_048_576{return Err(fail());}groups.entry((rule.target.clone(),action.clone())).or_default().push((index,rule.effect.clone()));}}
 let mut eligibility=Vec::new();
 for ((target,action),selected) in groups {
  let unknown=selected.iter().map(|(i,_)|format!("q{i} IS NULL")).collect::<Vec<_>>().join(" OR ");
  let permits=selected.iter().filter(|(_,e)|*e==Effect::Permit).map(|(i,_)|format!("q{i} IS TRUE")).collect::<Vec<_>>();
  let obligations=selected.iter().filter(|(_,e)|*e!=Effect::Permit).map(|(i,e)|format!("q{i} IS {}",if *e==Effect::Require{"TRUE"}else{"FALSE"})).collect::<Vec<_>>();
  let permit=if permits.is_empty(){"FALSE".into()}else{permits.join(" OR ")};
  let obligation=if obligations.is_empty(){"TRUE".into()}else{obligations.join(" AND ")};
  let decision=format!("CASE WHEN ({unknown}) THEN 'indeterminate' WHEN ({permit}) AND ({obligation}) THEN 'permit' ELSE 'deny' END");
  total+=decision.len();if total>4_000_000{return Err(fail());}
  let bindings=plan.source().ontology()["entities"].as_array().ok_or_else(fail)?;
  let associations=plan.source().ontology()["associations"].as_array().ok_or_else(fail)?;
  let binding=renderer.one(bindings.iter().chain(associations).filter(|e|e["type"]["documentId"].as_str()==Some(target.document_id.as_str())&&e["type"]["moduleId"].as_str()==Some(target.module_id.as_str())&&e["type"]["elementId"].as_str()==Some(target.element_id.as_str())).collect())?;
  let mut disclosure=Vec::new();
  for field in binding["fields"].as_array().ok_or_else(fail)? {
   renderer.step()?;for key in ["documentId","moduleId","elementId"] {group_bytes=group_bytes.checked_add(field["ref"][key].as_str().ok_or_else(fail)?.len()).ok_or_else(fail)?;}if group_bytes>1_048_576{return Err(fail());}let field_ref=reference(&field["ref"])?;let mut declared=Vec::new();let mut withheld=Vec::new();let mut transforms=Vec::new();
   for (index,effect) in &selected {renderer.step()?;if *effect!=Effect::Permit{continue;}for (declared_field,disposition) in &plan.rules()[*index].disclosure {
    renderer.step()?;if declared_field!=&field_ref{continue;}declared.push(format!("q{index} IS TRUE"));
    match disposition {Disposition::Withheld=>withheld.push(format!("q{index} IS TRUE")),Disposition::Original=>{},Disposition::Transformed{..}=>transforms.push((*index,disposition))}
   }}
   let declared=if declared.is_empty(){"FALSE".into()}else{declared.join(" OR ")};let withheld=if withheld.is_empty(){"FALSE".into()}else{withheld.join(" OR ")};
   let mut transform_metadata=Vec::new();let mut choices=Vec::new();let mut conflicts=Vec::new();
   for (slot,(index,disposition)) in transforms.iter().enumerate() {
    renderer.step()?;let encoded=serde_json::to_vec(disposition).map_err(|_|fail())?;
    group_bytes=group_bytes.checked_add(encoded.len()).ok_or_else(fail)?;if group_bytes>1_048_576{return Err(fail());}
    transform_metadata.push(json!({"slot":slot,"disposition":disposition}));choices.push(format!("WHEN q{index} IS TRUE THEN {slot}"));
    for (other,_) in transforms.iter().take(slot) {
     renderer.step()?;
     let truths=selected.iter().map(|(i,e)|(plan.rules()[*i].id.clone(),if *e==Effect::Require||(*e==Effect::Permit&&(*i==*index||*i==*other)){weft_core::security_composition::Truth::True}else{weft_core::security_composition::Truth::False})).collect();
     let composed=weft_core::security_composition::compose(plan,catalog,&target,&action,std::slice::from_ref(&field_ref),&truths)?;
     if composed.decision==weft_core::security_composition::Decision::Conflict{conflicts.push(format!("(q{index} IS TRUE AND q{other} IS TRUE)"));}
    }
   }
   let conflict=if conflicts.is_empty(){"FALSE".into()}else{conflicts.join(" OR ")};
   let transformed=if transforms.is_empty(){"FALSE".into()}else{transforms.iter().map(|(i,_)|format!("q{i} IS TRUE")).collect::<Vec<_>>().join(" OR ")};
   let transform_choice=if choices.is_empty(){"NULL::pg_catalog.int4".into()}else{format!("CASE {} ELSE NULL::pg_catalog.int4 END",choices.join(" "))};
   let fallback=if field["protection"]=="protected"{"indeterminate"}else{"original"};
   let field_state=format!("CASE WHEN ({withheld}) THEN 'withheld' WHEN ({conflict}) THEN 'conflict' WHEN ({transformed}) THEN 'transformed' WHEN ({declared}) THEN 'original' ELSE '{fallback}' END");
   let state=format!("CASE WHEN ({decision}) <> 'permit' THEN 'not-applicable' ELSE ({field_state}) END");
   if state.len()>1_048_576{return Err(fail());}
   total+=state.len();if total>4_000_000{return Err(fail());}
   let transform_choice=format!("CASE WHEN ({state})='transformed' THEN ({transform_choice}) ELSE NULL::pg_catalog.int4 END");
   if transform_choice.len()>1_048_576{return Err(fail());}
   total+=transform_choice.len();if total>4_000_000{return Err(fail());}
   disclosure.push(json!({"field":field_ref,"stateExpression":state,"transformationChoiceExpression":transform_choice,"transformations":transform_metadata}));
  }
  eligibility.push(json!({"target":target,"action":action,"decisionExpression":decision,"disclosure":disclosure}));
 }
 let output=json!({"version":"weft.truss.predicate-spike/0.4","securityReady":false,"scope":"conditional_rule_truths_on_originally_validated_string_boolean_record_facts_only","rules":rules,"missingEligibilityGroup":"unsupported_refuse","indeterminateField":"projection_must_refuse","conflictingField":"projection_must_refuse","disclosureScope":"row_permit_gated_original_withheld_transformed_conflict_states_no_value_projection","eligibilityScope":"selected_rule_groups_empty_disclosure_only_no_missing_group_authorization","eligibility":eligibility,"parameters":renderer.parameters,"prerequisites":["original_fact_domain_and_declared_key_validation","complete_fact_population","truthful_current_native_mapping","pg_catalog_pg_temp_search_path_and_authentic_native_helpers"]});
 if serde_json::to_vec(&output).map_err(|_|fail())?.len()>16_000_000{return Err(fail());}Ok(output)
}
