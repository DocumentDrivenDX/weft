//! Local Delta4 whole-candidate action SQL. The engine owns one optimistic
//! transaction containing data, original verification markers and request token.
use serde_json::{json,Value};
use std::collections::BTreeMap;
use weft_core::raw_action::{self,ActionEntity,ActionField,AdmittedAction,RuleExpression,RuleNode,ActionCondition};
use weft_core::raw_read::{Binding,Carrier};
type Result<T>=std::result::Result<T,&'static str>;
fn ident(s:&str)->Result<String>{if s.is_empty()||s.len()>255||s.contains('\0'){return Err("bounded Spark identifier required")};Ok(format!("`{}`",s.replace('`',"``")))}
fn field_id(f:&ActionField)->String{weft_core::mutation::canonical(&f.reference)}
struct Statement{slots:Vec<Value>}
impl Statement{
 fn slot(&mut self,source:Value,c:&Carrier)->String{self.slots.push(json!({"source":source,"carrier":c}));if *c==Carrier::Integer { format!("CAST(:p{} AS BIGINT)",self.slots.len()) } else { format!(":p{}",self.slots.len()) }}
 fn finish(self,sql:String)->Value{json!({"sql":sql,"parameters":self.slots})}
}
struct Lower<'a>{action:&'a AdmittedAction,binding:&'a Binding}
impl Lower<'_>{
 fn column(&self,f:&ActionField)->Result<String>{let t=self.binding.tables.iter().find(|t|t.logical==f.table).ok_or("action table absent")?;let c=t.columns.iter().find(|c|c.logical==f.logical).ok_or("action Field absent")?;ident(&c.native)}
 fn predicate(&self,e:&ActionEntity,s:&mut Statement)->Result<String>{e.key.fields.iter().zip(&e.components).map(|(f,v)|Ok(format!("{} <=> {}",self.column(f)?,s.slot(v.clone(),&f.carrier)))).collect::<Result<Vec<_>>>().map(|v|v.join(" AND "))}
 fn image(&self,name:&str,e:&ActionEntity,fields:&[ActionField])->Result<Value>{let mut s=Statement{slots:vec![]};let p=self.predicate(e,&mut s)?;let columns=fields.iter().enumerate().map(|(i,f)|Ok(format!("{} AS f{i}",self.column(f)?))).collect::<Result<Vec<_>>>()?.join(", ");Ok(json!({"name":name,"entity":e,"columns":fields,"statement":s.finish(format!("SELECT {columns} FROM `__umf_delta_stage` WHERE {p}"))}))}
 fn rule(&self,e:&RuleExpression,s:&mut Statement)->Result<(String,String)>{
  let(value,error)=match &e.node{
   RuleNode::Literal{value}=>{if value.is_null(){("NULL".into(),"FALSE".into())}else{let c=match e.family.as_str(){"boolean"=>Carrier::Boolean,"integer"=>Carrier::Integer,"string"=>Carrier::String,_=>return Err("Delta rule literal carrier unsupported")};(s.slot(json!({"literal":value}),&c),"FALSE".into())}},
   RuleNode::Input{id}=>{let p=self.action.action()["parameters"].as_array().unwrap().iter().find(|p|p["id"]==*id).ok_or("rule input absent")?;let f=self.action.fields().get(&weft_core::mutation::canonical(&json!([p["field"]["module"],p["field"]["element"]]))).ok_or("rule input Field absent")?;(s.slot(json!({"input":id}),&f.carrier),"FALSE".into())},
   RuleNode::State{phase,frame,field}=>{let exists=s.slot(json!({"snapshotExists":phase,"image":format!("frame:{frame}")}),&Carrier::Boolean);(s.slot(json!({"snapshot":phase,"image":format!("frame:{frame}"),"field":field.reference}),&field.carrier),format!("NOT ({exists})"))},
   RuleNode::Exists{phase,frame}=>(s.slot(json!({"snapshotExists":phase,"image":format!("frame:{frame}")}),&Carrier::Boolean),"FALSE".into()),
   RuleNode::Operation{operator,args}=>{let(a,ae)=self.rule(&args[0],s)?;if operator=="present"{("TRUE".into(),ae)}else if operator=="not"{(format!("NOT ({a})"),format!("({ae} OR {a} IS NULL)"))}else{let(b,be)=self.rule(&args[1],s)?;match operator.as_str(){
    "and"=>(format!("CASE WHEN {a} IS TRUE THEN {b} ELSE FALSE END"),format!("({ae} OR {a} IS NULL OR ({a} IS TRUE AND ({be} OR {b} IS NULL)))")),
    "or"=>(format!("CASE WHEN {a} IS TRUE THEN TRUE ELSE {b} END"),format!("({ae} OR {a} IS NULL OR ({a} IS FALSE AND ({be} OR {b} IS NULL)))")),
    "eq"=>(format!("({a} <=> {b})"),format!("({ae} OR {be})")),
    op=>{let native=match op{"lt"=>"<","le"=>"<=","gt"=>">","ge"=>">=","add"=>"+","sub"=>"-",_=>return Err("Delta rule operator unsupported")};(format!("({a} {native} {b})"),format!("({ae} OR {be} OR {a} IS NULL OR {b} IS NULL)"))}
   }}}
  };if value.len().saturating_add(error.len())>8192{return Err("Delta lowered rule budget")};Ok((value,error))
 }
 fn condition(&self,c:&ActionCondition)->Result<Value>{let mut s=Statement{slots:vec![]};let(v,e)=self.rule(&c.expression,&mut s)?;Ok(json!({"id":c.id,"code":c.code,"retryable":c.retryable,"source":c.source,"statement":s.finish(format!("SELECT ({e} OR {v} IS NULL) AS evaluation_error, ({v} IS TRUE) AS passed"))}))}
}
fn compile(source:&str)->Result<Value>{
 let admitted=raw_action::admit_json(source)?;let r=admitted.request();if r.target!="spark-delta4"||r.binding.tables.len()!=1||!r.binding.relationships.is_empty(){return Err("one original Delta table action required")}
 for f in admitted.fields().values(){match &f.carrier{Carrier::String|Carrier::Boolean=>{},Carrier::Integer=>{let w=&f.declaration["facets"]["integerWidth"];let bits=w["bits"].as_u64().ok_or("bounded integer width required")?;if bits==0||bits>64||w["signed"]!=true{return Err("signed native BIGINT domain required")}},_=>return Err("Delta action carrier not qualified")}}
 let table=&r.binding.tables[0];if table.columns.iter().any(|c|!matches!(c.carrier,Carrier::String|Carrier::Boolean|Carrier::Integer)){return Err("complete Delta table carrier profile unsupported")}
 let lower=Lower{action:&admitted,binding:&r.binding};let mut images=BTreeMap::<String,(ActionEntity,BTreeMap<String,ActionField>)>::new();
 for(name,e)in admitted.entities(){images.insert(format!("entity:{name}"),(e.clone(),e.key.fields.iter().map(|f|(field_id(f),f.clone())).collect()));}
 for(name,frame)in admitted.frames(){images.insert(format!("frame:{name}"),(frame.entity.clone(),frame.entity.key.fields.iter().chain(frame.fields.iter()).map(|f|(field_id(f),f.clone())).collect()));}
 for(i,effect)in admitted.effects().iter().enumerate(){images.insert(format!("effect:{i}"),(effect.entity.clone(),effect.entity.key.fields.iter().chain(effect.values.iter().map(|(f,_)|f)).map(|f|(field_id(f),f.clone())).collect()));}
 if images.values().any(|(e,_)|e.key.table!=table.logical){return Err("cross-table Delta atomic action unsupported")}
 let pre=images.iter().map(|(name,(e,f))|lower.image(name,e,&f.values().cloned().collect::<Vec<_>>())).collect::<Result<Vec<_>>>()?;
 let mut effects=vec![];
 for(i,effect)in admitted.effects().iter().enumerate(){let mut s=Statement{slots:vec![]};let sql=match effect.kind.as_str(){
  "set"=>{if effect.values.is_empty()||effect.values.iter().any(|(f,_)|effect.entity.key.fields.iter().any(|k|k.reference==f.reference)){return Err("nonempty non-Key Delta SET required")};let mut columns=vec![];for c in &table.columns{let name=ident(&c.native)?;let expression=if let Some((_,v))=effect.values.iter().find(|(f,_)|f.logical==c.logical){let p=lower.predicate(&effect.entity,&mut s)?;let value=s.slot(v.clone(),&c.carrier);format!("CASE WHEN {p} THEN {value} ELSE {name} END AS {name}")}else{name};columns.push(expression)}format!("SELECT {} FROM `__umf_delta_stage`",columns.join(", "))},
  "delete"=>{let p=lower.predicate(&effect.entity,&mut s)?;format!("SELECT * FROM `__umf_delta_stage` WHERE NOT ({p})")},
  "create"=>{if effect.values.len()!=table.columns.len(){return Err("Delta CREATE requires every scalar member")};let mut values=vec![];for c in &table.columns{let(_,value)=effect.values.iter().find(|(f,_)|f.logical==c.logical).ok_or("complete Delta CREATE Field inventory required")?;values.push(format!("{} AS {}",s.slot(value.clone(),&c.carrier),ident(&c.native)?))};format!("SELECT * FROM `__umf_delta_stage` UNION ALL SELECT {}",values.join(", "))},_=>return Err("Delta effect unsupported")};
  let guard=if effect.kind=="set"{let mut g=Statement{slots:vec![]};let p=lower.predicate(&effect.entity,&mut g)?;let diff=effect.values.iter().map(|(f,v)|Ok(format!("NOT ({} <=> {})",lower.column(f)?,g.slot(v.clone(),&f.carrier)))).collect::<Result<Vec<_>>>()?.join(" OR ");g.finish(format!("SELECT ({diff}) AS changed FROM `__umf_delta_stage` WHERE {p}"))}else{Value::Null};
  effects.push(json!({"guard":guard,"id":effect.id,"kind":effect.kind,"entity":effect.entity,"image":format!("effect:{i}"),"values":effect.values,"statement":s.finish(sql)}));
 }
 let key_columns=table.key.iter().map(|name|table.columns.iter().find(|c|&c.logical==name).ok_or("original complete native Key absent").and_then(|c|ident(&c.native))).collect::<Result<Vec<_>>>()?;
 if key_columns.is_empty(){return Err("original native Key required")};let names=key_columns.join(", ");let nulls=key_columns.iter().map(|c|format!("{c} IS NULL")).collect::<Vec<_>>().join(" OR ");
 let key_check=json!({"sql":format!("SELECT COUNT(*) AS violations FROM (SELECT {names} FROM `__umf_delta_stage` GROUP BY {names} HAVING COUNT(*) > 1 OR {nulls}) invalid_keys"),"parameters":[]});
 let mut out=json!({"version":"weft.native-action/0.1","profile":"umf.actions.native-delta/0.1","status":"compiled","target":r.target,"source":r.source,"binding":r.binding,"security":r.security,"roleMappings":r.role_mappings,"action":admitted.action(),"entities":admitted.entities(),"fields":admitted.fields(),"frames":admitted.frames(),"keyCheck":key_check,"preImages":pre,"postImages":pre,"preconditions":admitted.preconditions().iter().map(|c|lower.condition(c)).collect::<Result<Vec<_>>>()?,"postconditions":admitted.postconditions().iter().map(|c|lower.condition(c)).collect::<Result<Vec<_>>>()?,"effects":effects,"obligations":{"transaction":"native-Delta4-optimistic-Serializable-whole-table","snapshotView":"__umf_delta_stage","nativeAdmission":"original-host-qualified-unchanged-closure-fence","principal":"native-PG-issued-human-actor-local-Spark-owner-executor","receipt":"original-native-table-epoch-and-commit-version","markers":"same-native-commit-with-data-and-SetTransaction","noRetry":true,"validate":"same-checks-without-native-commit","rowBound":10000}});
 out["structuralDigest"]=json!({"owner":"weft-databricks-native-action","version":"0.1","sha256":weft_core::json::sha256(weft_core::mutation::canonical(&out).as_bytes())});Ok(out)
}
pub fn compile_json(source:&str)->String{match compile(source){Ok(v)=>v,Err(message)=>json!({"version":"weft.native-action/0.1","status":"blocked","diagnostics":[{"code":"WFT-NATIVE-DELTA","message":message}]})}.to_string()}

#[cfg(test)]mod tests{
 use super::*;
 fn request()->Value{let mut r:Value=serde_json::from_str(include_str!("../../../tests/native-actions/action-request.json")).unwrap();r["target"]=json!("spark-delta4");r}
 #[test]fn delta_program_retains_original_action_and_same_commit_duties(){let r=request();let out:Value=serde_json::from_str(&compile_json(&r.to_string())).unwrap();assert_eq!(out["status"],"compiled","{out}");assert_eq!(out["profile"],"umf.actions.native-delta/0.1");assert_eq!(out["source"],r["source"]);assert!(out["effects"][0]["statement"]["sql"].as_str().unwrap().contains("CASE WHEN"));assert!(!out.to_string().contains("RETURNING"));assert_eq!(out["obligations"]["markers"],"same-native-commit-with-data-and-SetTransaction");assert_eq!(out["structuralDigest"]["owner"],"weft-databricks-native-action");}
 #[test]fn postgres_profile_never_lowers_as_delta(){let mut r=request();r["target"]=json!("postgresql18");let out:Value=serde_json::from_str(&compile_json(&r.to_string())).unwrap();assert_eq!(out["status"],"blocked");}
}
