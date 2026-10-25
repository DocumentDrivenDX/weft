// SPDX-License-Identifier: MIT OR Apache-2.0
//! Source-bound finite Unity Catalog current scalar-predicate compiler (request 0.2).
//! This module is additive to unity_current_read/0.1 and performs no host I/O.
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use weft_core::raw_read;

type Result<T> = std::result::Result<T, &'static str>;
const CORE_VERSION: &str = "0.10.0";
const REQUEST_VERSION: &str = "weft.unity-current-read-request/0.2";
const RESPONSE_VERSION: &str = "weft.unity-current-selection/0.3";
const COMPILER_VERSION: &str = "weft-databricks-unity-current-read/0.2";
const LIMIT_SQL: usize = 65_536;

pub(super) fn exact(v: &Value, required: &[&str], optional: &[&str]) -> Result<()> {
    let o = v.as_object().ok_or("closed object required")?;
    if required.iter().any(|k| !o.contains_key(*k))
        || o.keys().any(|k| !required.contains(&k.as_str()) && !optional.contains(&k.as_str()))
    { return Err("unknown or missing member"); }
    Ok(())
}
pub(super) fn logical_ident(s: &str) -> Result<String> {
    if s.is_empty() || s.len() > 255 || s.contains('\0') { return Err("bounded source identifier required"); }
    Ok(format!("\"{}\"", s.replace('"', "\"\"")))
}
fn key(module: &str, element: &str) -> String { format!("{module}\0{element}") }
pub(super) fn checked_ref(v: &Value) -> Result<(String, String)> {
    exact(v, &["module", "element"], &[])?;
    let m = v["module"].as_str().ok_or("Field module required")?;
    let e = v["element"].as_str().ok_or("Field element required")?;
    if m.is_empty() || e.is_empty() || m.len() > 255 || e.len() > 255 { return Err("bounded Field reference required"); }
    Ok((m.to_owned(), e.to_owned()))
}
fn quote_literal(s: &str) -> Result<String> {
    if s.contains('\0') || s.len() > 4096 { return Err("bounded non-NUL STRING literal required"); }
    // The shared Weft SQL parser uses PostgreSQL string rules with
    // backslash-escapes disabled: doubled quotes decode to the authored value,
    // while authored backslashes remain ordinary UTF-8 data. Lower recognizes
    // this private typed marker and emits only an ASCII UTF-8 hex decode.
    Ok(format!("__weft_utf8_value('{}')", s.replace('\'', "''")))
}
pub(super) fn valid_parameter_name(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64
        && (s.as_bytes()[0].is_ascii_alphabetic() || s.as_bytes()[0] == b'_')
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

pub(super) struct Source {
    pub(super) table_module: String,
    pub(super) record_id: String,
    pub(super) record_name: String,
    fields: BTreeMap<String, SourceField>, // Record membership identity -> source facts
    sum_fields: BTreeMap<String, SumField>, // Only the additive SUM owner reads this map.
}
pub(super) struct SumField {
    pub(super) name: String,
    pub(super) carrier: String,
    pub(super) nullable: bool,
    pub(super) facets: Value,
}
struct SourceField {
    name: String,
    carrier: String,
    nullable: Option<bool>,
    admitted: bool,
    create_missing: bool,
    create_null: bool,
    generated: bool,
}
impl Source {
    pub(super) fn field(&self, r: &Value) -> Result<(String, String, bool)> {
        let (m,e)=checked_ref(r)?;
        let field=self.fields.get(&key(&m,&e)).ok_or("Field absent from selected Record")?;
        if !field.admitted { return Err("selected Field outside finite STRING/BOOLEAN profile"); }
        Ok((field.name.clone(),field.carrier.clone(),field.nullable.ok_or("Field nullability outside profile")?))
    }
    pub(super) fn sql_field(&self, r: &Value) -> Result<(String, String, bool)> {
        let (name,carrier,nullable)=self.field(r)?;
        Ok((format!("r.{}",logical_ident(&name)?),carrier,nullable))
    }
}

pub(super) fn source_profile(model: &Value, table_ref: &Value) -> Result<Source> {
    let (module_id,record_id)=checked_ref(table_ref)?;
    let modules=model["modules"].as_array().ok_or("source modules required")?;
    let modules=modules.iter().filter(|m|m["id"]==module_id).collect::<Vec<_>>();
    if modules.len()!=1 { return Err("source module absent or ambiguous"); }
    let module=modules[0];
    let elements=module["elements"].as_array().ok_or("source elements required")?;
    let records=elements.iter().filter(|e|e["id"]==record_id && e["kind"]=="record").collect::<Vec<_>>();
    if records.len()!=1 { return Err("selected Record absent or ambiguous"); }
    let record=records[0];
    if record["extensions"].as_object().is_none_or(|e|!e.is_empty()) { return Err("Record extensions unsupported"); }
    let record_name=record["name"].as_str().filter(|s|!s.is_empty()).ok_or("Record name required")?.to_owned();
    let mut fields=BTreeMap::new();
    let members=record["members"].as_array().ok_or("Record members required")?;
    if members.is_empty() { return Err("selected Record must have members"); }
    for member in members {
        exact(member,&["module","element"],&[])?;
        let m=member["module"].as_str().ok_or("member module required")?;
        let e=member["element"].as_str().ok_or("member element required")?;
        let field_modules=model["modules"].as_array().unwrap().iter().filter(|candidate|candidate["id"]==m).collect::<Vec<_>>();
        if field_modules.len()!=1 { return Err("Field module absent or ambiguous"); }
        let field_elements=field_modules[0]["elements"].as_array().ok_or("Field module elements required")?;
        let candidates=field_elements.iter().filter(|x|x["id"]==e && x["kind"]=="field").collect::<Vec<_>>();
        if candidates.len()!=1 { return Err("Field absent or ambiguous"); }
        let field=candidates[0];
        let name=field["name"].as_str().filter(|s|!s.is_empty()).ok_or("Field name required")?;
        let carrier=field["scalarType"].as_str().ok_or("Field scalar type required")?;
        let nullable=match field["nullability"].as_str(){Some("required")=>Some(false),Some("absent-allowed")=>Some(true),_=>None};
        let admitted=matches!(carrier,"string"|"boolean") && field["cardinality"]=="one" && nullable.is_some() && field["extensions"].as_object().is_some_and(|x|x.is_empty()) && field.get("facets").is_none_or(|x|x==&json!({})) && field.get("semanticTypes").is_none();
        if fields.insert(key(m,e),SourceField{name:name.to_owned(),carrier:carrier.to_owned(),nullable,admitted,create_missing:false,create_null:false,generated:false}).is_some(){return Err("duplicate Field identity");}
    }
    Ok(Source{table_module:module_id,record_id,record_name,fields,sum_fields:BTreeMap::new()})
}

/// SUM-only predicate facts. Unlike the historical predicate profile, this
/// constructor retains every original Record member but admits only reached
/// STRING/BOOLEAN predicate leaves; unselected vectors/binary/extensions stay
/// present in the checked source and are never projected into these facts.
pub(super) fn source_profile_for_sum(model: &Value, table_ref: &Value) -> Result<Source> {
    let (module_id,record_id)=checked_ref(table_ref)?;
    let modules=model["modules"].as_array().ok_or("source modules required")?;
    let owners=modules.iter().filter(|m|m["id"]==module_id).collect::<Vec<_>>();
    if owners.len()!=1{return Err("source module absent or ambiguous")}
    let module=owners[0];
    let elements=module["elements"].as_array().ok_or("source elements required")?;
    let records=elements.iter().filter(|e|e["id"]==record_id&&e["kind"]=="record").collect::<Vec<_>>();
    if records.len()!=1{return Err("selected Record absent or ambiguous")}
    let record=records[0];
    if record["extensions"].as_object().is_none_or(|e|!e.is_empty()){return Err("selected Record extensions unsupported")}
    let record_name=record["name"].as_str().filter(|s|!s.is_empty()).unwrap_or(record_id.as_str()).to_owned();
    let members=record["members"].as_array().filter(|m|!m.is_empty()&&m.len()<=256).ok_or("bounded original Record members required")?;
    let record_ref=json!({"module":module_id,"element":record_id});
    let access_views=model["modules"].as_array().unwrap().iter().flat_map(|m|m["extensions"]["umf.access"]["views"].as_array().into_iter().flatten())
        .filter(|v|v["record"]==record_ref).collect::<Vec<_>>();
    if access_views.len()!=1{return Err("unique original Access RecordView required")}
    let access_fields=access_views[0]["fields"].as_array().ok_or("original Access FieldViews required")?;
    let mut fields=BTreeMap::new();
    let mut sum_fields=BTreeMap::new();
    for member in members {
        exact(member,&["module","element"],&[])?;
        let m=member["module"].as_str().filter(|s|!s.is_empty()).ok_or("member module required")?;
        let e=member["element"].as_str().filter(|s|!s.is_empty()).ok_or("member element required")?;
        let field_modules=modules.iter().filter(|candidate|candidate["id"]==m).collect::<Vec<_>>();
        if field_modules.len()!=1{return Err("Field module absent or ambiguous")}
        let field_elements=field_modules[0]["elements"].as_array().ok_or("Field module elements required")?;
        let rows=field_elements.iter().filter(|x|x["id"]==e&&x["kind"]=="field").collect::<Vec<_>>();
        if rows.len()!=1{return Err("Record member Field absent or ambiguous")}
        let field=rows[0];
        let name=field["name"].as_str().filter(|s|!s.is_empty()).unwrap_or(e);
        let carrier=field["scalarType"].as_str().unwrap_or("unadmitted");
        let core_nullable=match field["nullability"].as_str(){Some("required")=>Some(false),Some("absent-allowed")=>Some(true),_=>None};
        let field_ref=json!({"module":m,"element":e});
        let views=access_fields.iter().filter(|v|v["field"]==field_ref).collect::<Vec<_>>();
        if views.len()!=1{return Err("unique original Access FieldView required")}
        let row_nullable=views[0]["row"]["null"].as_bool().ok_or("original row null policy required")?;
        if views[0]["row"]["missing"]!=false || core_nullable.is_none() || (row_nullable && core_nullable!=Some(true)) { return Err("Access row policy contradicts core field presence"); }
        let create_missing=views[0]["create"]["missing"].as_bool().ok_or("create missing policy required")?;
        let create_null=views[0]["create"]["null"].as_bool().ok_or("create null policy required")?;
        let generated=views[0]["generated"].as_bool().ok_or("FieldView generated marker required")?;
        if create_null && !row_nullable { return Err("create-null policy contradicts non-null row policy"); }
        let admitted=matches!(carrier,"string"|"boolean")&&field["cardinality"]=="one"
            &&field["extensions"].as_object().is_some_and(|x|x.is_empty())
            &&field.get("facets").is_none_or(|x|x==&json!({}))&&field.get("semanticTypes").is_none();
        if fields.insert(key(m,e),SourceField{name:name.to_owned(),carrier:carrier.to_owned(),nullable:Some(row_nullable),admitted,create_missing,create_null,generated}).is_some(){return Err("duplicate Field identity")}
        if matches!(carrier, "integer" | "decimal")
            && field["cardinality"] == "one"
            && field["extensions"].as_object().is_some_and(|x| x.is_empty())
            && field.get("semanticTypes").is_none()
        {
            let facets = field.get("facets").cloned().unwrap_or_else(|| json!({}));
            sum_fields.insert(key(m,e), SumField {
                name: name.to_owned(), carrier: carrier.to_owned(),
                nullable: row_nullable, facets,
            });
        }
    }
    Ok(Source{table_module:module_id,record_id,record_name,fields,sum_fields})
}

pub(super) fn sum_field<'a>(source: &'a Source, field_ref: &Value) -> Result<&'a SumField> {
    let (module, element) = checked_ref(field_ref)?;
    source.sum_fields.get(&key(&module, &element)).ok_or("SUM Field outside exact integer/decimal source profile")
}

pub(super) fn validate_predicate(source:&Source, node:&Value, depth:usize, count:&mut usize, params:&BTreeMap<String,(String,bool,Option<u16>)>, used:&mut Vec<String>, bound:&mut BTreeSet<String>) -> Result<String> {
    *count+=1; if depth>4 || *count>32 { return Err("predicate depth/node budget exceeded"); }
    let kind=node["kind"].as_str().ok_or("predicate kind required")?;
    match kind {
        "comparison"=>{
            exact(node,&["kind","op","fieldRef","operand"],&[])?;
            exact(&node["operand"],&["parameter"],&[])?;
            let op=node["op"].as_str().ok_or("comparison operator required")?;
            if !matches!(op,"eq"|"ne"|"lt"|"le"|"gt"|"ge"){return Err("comparison operator outside profile");}
            let (field,carrier,_nullable)=source.sql_field(&node["fieldRef"])?;
            let (name,p)= (node["operand"]["parameter"].as_str().ok_or("parameter name required")?, node["operand"]["parameter"].as_str().unwrap());
            let (p_carrier,_,_)=params.get(name).ok_or("parameter descriptor absent")?;
            if p_carrier!=&carrier || (carrier=="boolean" && !matches!(op,"eq"|"ne")){return Err("comparison carrier/operator mismatch");}
            bound.insert(key(&checked_ref(&node["fieldRef"])?.0,&checked_ref(&node["fieldRef"])?.1)); used.push(p.to_owned());
            let sqlop=match op{"eq"=>"=","ne"=>"<>","lt"=>"<","le"=>"<=","gt"=>">","ge"=>">=",_=>unreachable!()};
            Ok(format!("({field} {sqlop} :{name})"))
        }
        "is-null"|"is-not-null"=>{
            exact(node,&["kind","fieldRef"],&[])?;
            let (field,_,_)=source.sql_field(&node["fieldRef"])?;
            let (m,e)=checked_ref(&node["fieldRef"])?; bound.insert(key(&m,&e));
            Ok(format!("({field} IS {}NULL)",if kind=="is-not-null"{"NOT "}else{""}))
        }
        "and"|"or"=>{
            exact(node,&["kind","terms"],&[])?;
            let terms=node["terms"].as_array().filter(|a|a.len()>=2).ok_or("AND/OR requires at least two terms")?;
            let op=if kind=="and"{"AND"}else{"OR"};
            let mut parts=Vec::new(); for term in terms {parts.push(validate_predicate(source,term,depth+1,count,params,used,bound)?);}
            Ok(format!("({})",parts.join(&format!(" {op} "))))
        }
        "not"=>{
            exact(node,&["kind","term"],&[])?;
            Ok(format!("NOT ({})",validate_predicate(source,&node["term"],depth+1,count,params,used,bound)?))
        }
        "in"|"not-in"=>{
            exact(node,&["kind","fieldRef","values"],&[])?;
            let (field,carrier,_)=source.sql_field(&node["fieldRef"])?;
            let (m,e)=checked_ref(&node["fieldRef"])?; bound.insert(key(&m,&e));
            let values=node["values"].as_array().filter(|a|a.len()<=16).ok_or("membership value limit")?;
            if values.is_empty(){return Ok(if kind=="in"{format!("in_empty({field})")}else{format!("NOT (in_empty({field}))")});}
            let mut literals=Vec::new();
            for value in values { exact(value,&["carrier","value"],&[])?; let c=value["carrier"].as_str().ok_or("literal carrier required")?; if c!=carrier{return Err("membership carrier mismatch");}
                let literal=if value["value"].is_null(){"NULL".to_owned()}else if c=="string"{quote_literal(value["value"].as_str().ok_or("STRING literal required")?)?}else{match value["value"].as_bool(){Some(true)=>"TRUE".into(),Some(false)=>"FALSE".into(),None=>return Err("BOOLEAN literal required")}}; literals.push(literal);
            }
            Ok(format!("({field} {}IN ({}))",if kind=="not-in"{"NOT "}else{""},literals.join(", ")))
        }
        _=>Err("predicate form outside reviewed profile"),
    }
}

fn compile(raw:&str)->Result<Value>{
    let value=weft_core::json::checked_json(raw).map_err(|_|"invalid or duplicate JSON")?;
    exact(&value,&["version","source","expectedTarget","read","parameters"],&[])?;
    exact(&value["source"],&["version","utf8","sha256"],&[])?;
    exact(&value["expectedTarget"],&["kind","warehouseId","catalog","schema","table","tableUuid","runtimeVersion","columns"],&[])?;
    exact(&value["read"],&["kind","tableRef","projection","predicate","rowCap","frameCapBytes","deadlineMs"],&[])?;
    if value["version"]!=REQUEST_VERSION || value["source"]["version"]!=CORE_VERSION || value["source"]["sha256"].as_str().is_none_or(|d|d.len()!=64||!d.bytes().all(|b|b.is_ascii_hexdigit())) || value["source"]["utf8"].as_str().is_none_or(|s|weft_core::json::sha256(s.as_bytes())!=value["source"]["sha256"]) {return Err("source/request version or digest differs");}
    let source_doc=weft_core::json::checked_json(value["source"]["utf8"].as_str().unwrap()).map_err(|_|"invalid source JSON")?;
    if source_doc["umf"]!=CORE_VERSION || source_doc["modules"].as_array().is_none(){return Err("UMF core document required");}
    let target=&value["expectedTarget"];
    if target["kind"]!="unity-catalog" {return Err("Unity Catalog target required");}
    for name in ["warehouseId","catalog","schema","table","tableUuid","runtimeVersion"] {let s=target[name].as_str().ok_or("target identity string required")?; if s.is_empty()||s.len()>255||s.contains('\0'){return Err("bounded target identity required");}}
    let read=&value["read"];
    if read["kind"]!="finite-current" || read["rowCap"].as_u64().is_none_or(|n|n==0||n>1000) || read["frameCapBytes"].as_u64().is_none_or(|n|n==0||n>1_048_576) || read["deadlineMs"].as_u64().is_none_or(|n|n==0||n>30_000) {return Err("finite-current bounds required");}
    let table_ref=&read["tableRef"];
    let source=source_profile(&source_doc,table_ref)?;
    let projection=read["projection"].as_array().filter(|a|!a.is_empty()&&a.len()<=8).ok_or("projection bound required")?;
    let mut projections=Vec::new(); let mut bound=BTreeSet::new(); let mut projected=BTreeSet::new();
    for rf in projection {let (n,c,nullable)=source.field(rf)?;let (m,e)=checked_ref(rf)?;let k=key(&m,&e);if !projected.insert(k.clone()){return Err("duplicate projection Field");}bound.insert(k.clone());projections.push((rf.clone(),n,c,nullable,k));}
    let descriptors=value["parameters"].as_array().filter(|a|a.len()<=8).ok_or("parameter bound required")?;
    let mut params=BTreeMap::new(); let mut response_params=Vec::new();
    for p in descriptors {let carrier=p["carrier"].as_str().ok_or("parameter carrier required")?; if !matches!(carrier,"string"|"boolean"){return Err("parameter carrier outside profile");} if carrier=="string"{exact(p,&["name","carrier","nullable","maxUtf8Bytes"],&[])?;}else{exact(p,&["name","carrier","nullable"],&[])?;}
        let n=p["name"].as_str().filter(|s|valid_parameter_name(s)).ok_or("bounded parameter name required")?;
        let nullable=p["nullable"].as_bool().ok_or("parameter nullable boolean required")?;
        let max=if carrier=="string" {Some(u16::try_from(p["maxUtf8Bytes"].as_u64().filter(|v|*v>0&&*v<=4096).ok_or("STRING parameter byte bound required")?).map_err(|_|"parameter bound")?)} else {None};
        if params.insert(n.to_owned(),(carrier.to_owned(),nullable,max)).is_some(){return Err("duplicate parameter name");}
        response_params.push(p.clone());
    }
    let mut used=Vec::new(); let mut count=0;
    let predicate_sql=validate_predicate(&source,&read["predicate"],1,&mut count,&params,&mut used,&mut bound)?;
    if bound.len()>9 || used.len()==0 && !params.is_empty() || params.keys().any(|name|!used.iter().any(|u|u==name)){return Err("predicate field/parameter coverage differs");}
    let target_columns=target["columns"].as_array().ok_or("target columns required")?;
    if target_columns.len()!=bound.len(){return Err("target columns must exactly bind selected fields");}
    let mut columns=BTreeMap::new();
    let mut native_names=BTreeSet::new();
    for column in target_columns {exact(column,&["fieldRef","nativeName","nativeType","nullable"],&[])?;let (m,e)=checked_ref(&column["fieldRef"])?;let k=key(&m,&e);let (logical,carrier,nullable)=source.field(&column["fieldRef"])?;
        let native=if carrier=="string"{"STRING"}else{"BOOLEAN"};let native_name=column["nativeName"].as_str().filter(|s|!s.is_empty()&&s.len()<=255&&!s.contains('\0')).ok_or("bounded native column name required")?;if column["nativeType"]!=native || column["nullable"]!=nullable || !native_names.insert(native_name.to_owned()) || columns.insert(k.clone(),(native_name.to_owned(),carrier,nullable,logical)).is_some(){return Err("target native ColumnSet differs from source Field");}
        if !bound.contains(&k){return Err("unselected target Field refused");}
    }
    let relation=format!("{}.{}",logical_ident(&source.table_module)?,logical_ident(&source.record_name)?);
    let mut selects=Vec::new();let mut raw_columns=Vec::new();let mut outputs=Vec::new();
    for (ordinal,(rf,n,c,nullable,k)) in projections.iter().enumerate(){let(native,_,_,_)=columns.get(k).ok_or("projection native binding absent")?;selects.push(format!("r.{} AS __weft_out_{ordinal}",logical_ident(n)?));outputs.push(json!({"fieldRef":rf,"nativeName":native,"carrier":c,"nullable":nullable}));}
    for (k,(native,c,nullable,logical)) in &columns {raw_columns.push(json!({"logical":logical,"native":native,"carrier":c,"nullable":nullable}));let _=k;}
    let mut raw_parameters=Vec::new();for p in descriptors{raw_parameters.push(json!({"name":p["name"],"carrier":p["carrier"],"nullable":p["nullable"]}));}
    let sql=format!("SELECT {} FROM {relation} AS r WHERE {predicate_sql}",selects.join(", "));
    if sql.len()>LIMIT_SQL{return Err("lowered SQL byte limit exceeded");}
    let internal=json!({"version":"weft.raw-read/0.1","target":"unity-catalog-current","sql":sql,"parameters":raw_parameters,"rowCap":read["rowCap"],"keyset":false,"binding":{"version":"umf.raw-read-binding/0.1","schema":target["schema"],"tables":[{"logical":[source.table_module,source.record_name],"native":target["table"],"columns":raw_columns,"key":[]}],"relationships":[],"modelSha256":value["source"]["sha256"],"modelJson":value["source"]["utf8"]}});
    let admitted=raw_read::admit_finite_current_predicate_json(&internal.to_string()).map_err(|_|"derived predicate request failed shared source admission")?;
    let lowered=super::raw_read::lower_current_finite_predicate_shaped(&admitted,target["catalog"].as_str().unwrap()).map_err(|_|"predicate lowering refused")?;
    if lowered["status"]!="compiled"
        || lowered["parameterOrder"]!=json!(used)
        || lowered["outputs"].as_array().is_none_or(|a|a.len()!=projections.len() || a.iter().enumerate().any(|(i,o)| o["name"]!=format!("__weft_out_{i}") || o["carrier"].as_str()!=Some(projections[i].2.as_str()) || o["nullable"].as_bool()!=Some(projections[i].3)))
        || lowered["referencedTables"].as_array().is_none_or(|a|a.len()!=1 || a[0]["native"]!=target["table"] || a[0]["logical"]!=json!([source.table_module,source.record_name]))
        || lowered["snapshotTables"]!=json!([]) || lowered["nativeChecks"]!=json!([])
        || lowered["cursorParameters"]!=json!([]) || lowered["cursorOutputs"]!=json!([]) || lowered["cursorOrdering"]!=json!([])
    {return Err("shared predicate lowering shape differs");}
    let final_sql=lowered["sql"].as_str().filter(|s|!s.is_empty()).ok_or("final lowered SQL missing")?;
    if final_sql.len()>LIMIT_SQL{return Err("final lowered SQL byte limit exceeded");}
    let program=json!({"version":RESPONSE_VERSION,"status":"compiled","sourceSha256":value["source"]["sha256"],"expectedTargetDigest":weft_core::json::sha256(weft_core::mutation::canonical(target).as_bytes()),"readDigest":weft_core::json::sha256(weft_core::mutation::canonical(read).as_bytes()),"compilerVersion":COMPILER_VERSION,"sql":final_sql,"sqlSha256":weft_core::json::sha256(final_sql.as_bytes()),"parameterOrder":lowered["parameterOrder"],"parameters":response_params,"outputs":outputs,"rowCap":read["rowCap"],"frameCapBytes":read["frameCapBytes"],"deadlineMs":read["deadlineMs"],"expectedTarget":target,"obligations":{"transaction":"current-read","principal":"native-person","tableIdentity":"native-current-uuid","nativeAdmissionEachOperation":true,"continuation":false,"temporalQualification":"held","predicateProfile":"bounded-current-scalar/0.1","binaryStringCollation":true}});
    let digest=weft_core::json::sha256(weft_core::mutation::canonical(&program).as_bytes());
    let mut output=program;output["structuralDigest"]=json!({"owner":"weft-databricks-unity-current-read","version":"0.3","sha256":digest});Ok(output)
}

pub(crate) fn compile_json(raw:&str)->String{match compile(raw){Ok(v)=>v.to_string(),Err(message)=>json!({"version":RESPONSE_VERSION,"status":"blocked","diagnostics":[{"code":"WFT-UNITY-CURRENT-PREDICATE","message":message}]}).to_string()}}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(predicate: Value, parameters: Value) -> Value {
        let model=json!({"umf":"0.10.0","id":"predicate-fixture","vocabularies":{},"modules":[{"id":"sales","namespace":"sales","elements":[
            {"id":"Customer","name":"Customer","kind":"record","members":[{"module":"sales","element":"id"},{"module":"sales","element":"note"},{"module":"sales","element":"active"},{"module":"sales","element":"optional"}],"extensions":{}},
            {"id":"id","name":"id","kind":"field","scalarType":"string","cardinality":"one","nullability":"required","extensions":{}},
            {"id":"note","name":"note","kind":"field","scalarType":"string","cardinality":"one","nullability":"absent-allowed","extensions":{}},
            {"id":"active","name":"active","kind":"field","scalarType":"boolean","cardinality":"one","nullability":"required","extensions":{}},
            {"id":"optional","name":"optional","kind":"field","scalarType":"boolean","cardinality":"one","nullability":"absent-allowed","extensions":{}}
        ],"extensions":{}}]});
        let source=model.to_string();
        json!({"version":REQUEST_VERSION,"source":{"version":CORE_VERSION,"utf8":source,"sha256":weft_core::json::sha256(source.as_bytes())},
            "expectedTarget":{"kind":"unity-catalog","warehouseId":"w","catalog":"main","schema":"sales","table":"customers","tableUuid":"uuid-1","runtimeVersion":"16.4","columns":[
                {"fieldRef":{"module":"sales","element":"id"},"nativeName":"customer_id","nativeType":"STRING","nullable":false},
                {"fieldRef":{"module":"sales","element":"note"},"nativeName":"note","nativeType":"STRING","nullable":true},
                {"fieldRef":{"module":"sales","element":"active"},"nativeName":"active","nativeType":"BOOLEAN","nullable":false},
                {"fieldRef":{"module":"sales","element":"optional"},"nativeName":"optional","nativeType":"BOOLEAN","nullable":true}]},
            "read":{"kind":"finite-current","tableRef":{"module":"sales","element":"Customer"},"projection":[{"module":"sales","element":"id"},{"module":"sales","element":"note"},{"module":"sales","element":"active"},{"module":"sales","element":"optional"}],"predicate":predicate,"rowCap":20,"frameCapBytes":65536,"deadlineMs":5000},"parameters":parameters})
    }
    fn response(request: Value) -> Value {
        serde_json::from_str(&super::super::unity_current_read::compile_json(&request.to_string())).unwrap()
    }
    fn reseal_source(request: &mut Value, model: &Value) {
        let source=model.to_string();
        request["source"]["utf8"]=json!(source);
        request["source"]["sha256"]=json!(weft_core::json::sha256(source.as_bytes()));
    }

    #[test]
    fn parser_logical_quoting_stays_separate_from_physical_databricks_quoting() {
        let mut req = request(
            eq_parameter("id", "p"),
            json!([{"name":"p","carrier":"string","nullable":false,"maxUtf8Bytes":64}]),
        );
        let mut model: Value = serde_json::from_str(req["source"]["utf8"].as_str().unwrap()).unwrap();
        model["modules"][0]["elements"].as_array_mut().unwrap()
            .iter_mut().find(|field| field["id"] == "Customer").unwrap()["name"] = json!(r#"Customer "quoted`name"#);
        model["modules"][0]["elements"].as_array_mut().unwrap()
            .iter_mut().find(|field| field["id"] == "id").unwrap()["name"] = json!(r#"customer "id`value"#);
        req["expectedTarget"]["columns"][0]["nativeName"] = json!("physical`customer");
        reseal_source(&mut req, &model);

        assert_eq!(logical_ident(r#"customer "id`value"#).unwrap(), r#""customer ""id`value""#);
        assert_eq!(crate::raw_read::ident("physical`customer").unwrap(), "`physical``customer`");
        let out = response(req);
        assert_eq!(out["status"], "compiled", "{out}");
        assert!(out["sql"].as_str().unwrap().contains("`r`.`physical``customer`"));
    }
    fn eq_parameter(field: &str, parameter: &str) -> Value { json!({"kind":"comparison","op":"eq","fieldRef":{"module":"sales","element":field},"operand":{"parameter":parameter}}) }
    #[test]
    fn public_v02_route_lowers_string_boolean_nullable_and_nested_predicates() {
        let pred=json!({"kind":"and","terms":[eq_parameter("id","p"),{"kind":"not","term":{"kind":"is-null","fieldRef":{"module":"sales","element":"optional"}}}]});
        let req=request(pred,json!([{"name":"p","carrier":"string","nullable":true,"maxUtf8Bytes":128}]));
        let out=response(req);assert_eq!(out["status"],"compiled");assert_eq!(out["version"],RESPONSE_VERSION);assert_eq!(out["compilerVersion"],COMPILER_VERSION);
        assert_eq!(out["parameters"][0]["nullable"],true);assert_eq!(out["outputs"][1]["nullable"],true);assert!(out["sql"].as_str().unwrap().contains("COLLATE UTF8_BINARY"));assert_eq!(out["obligations"]["binaryStringCollation"],true);
    }
    #[test]
    fn all_string_order_operators_and_boolean_equality_use_the_selected_profile() {
        for op in ["eq","ne","lt","le","gt","ge"] {
            let mut predicate=eq_parameter("id","p");predicate["op"]=json!(op);
            let out=response(request(predicate,json!([{"name":"p","carrier":"string","nullable":false,"maxUtf8Bytes":16}])));
            assert_eq!(out["status"],"compiled","operator {op}: {out}");
            assert!(out["sql"].as_str().unwrap().contains("COLLATE UTF8_BINARY"));
        }
        let pred=json!({"kind":"comparison","op":"eq","fieldRef":{"module":"sales","element":"active"},"operand":{"parameter":"flag"}});
        let out=response(request(pred,json!([{"name":"flag","carrier":"boolean","nullable":false}])));assert_eq!(out["status"],"compiled");assert!(!out["sql"].as_str().unwrap().contains("COLLATE UTF8_BINARY"));
    }
    #[test]
    fn parameter_occurrence_order_and_metadata_are_exact() {
        let pred=json!({"kind":"and","terms":[eq_parameter("id","second"),eq_parameter("note","first")]});
        let req=request(pred,json!([{"name":"first","carrier":"string","nullable":false,"maxUtf8Bytes":10},{"name":"second","carrier":"string","nullable":true,"maxUtf8Bytes":64}]));
        let out=response(req);assert_eq!(out["status"],"compiled");assert_eq!(out["parameterOrder"],json!(["second","first"]));
        assert_eq!(out["parameters"][0]["name"],"first");assert_eq!(out["parameters"][1]["maxUtf8Bytes"],64);
    }
    #[test]
    fn unselected_other_carrier_member_does_not_expand_selected_profile() {
        let mut req=request(eq_parameter("id","p"),json!([{"name":"p","carrier":"string","nullable":false,"maxUtf8Bytes":32}]));
        let mut model:Value=serde_json::from_str(req["source"]["utf8"].as_str().unwrap()).unwrap();
        model["modules"][0]["elements"][0]["members"].as_array_mut().unwrap().push(json!({"module":"sales","element":"age"}));
        model["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":"age","name":"age","kind":"field","scalarType":"integer","cardinality":"one","nullability":"required","extensions":{}}));
        reseal_source(&mut req,&model);
        assert_eq!(response(req)["status"],"compiled");
    }
    #[test]
    fn public_v02_boolean_and_null_membership_use_three_valued_sql() {
        let pred=json!({"kind":"or","terms":[{"kind":"comparison","op":"ne","fieldRef":{"module":"sales","element":"active"},"operand":{"parameter":"enabled"}},{"kind":"not-in","fieldRef":{"module":"sales","element":"optional"},"values":[{"carrier":"boolean","value":true},{"carrier":"boolean","value":null}]}]});
        let req=request(pred,json!([{"name":"enabled","carrier":"boolean","nullable":false}]));let out=response(req);assert_eq!(out["status"],"compiled");let sql=out["sql"].as_str().unwrap();assert!(sql.contains("CAST(NULL AS BOOLEAN)"));assert!(sql.contains("NOT ("));assert_eq!(out["parameterOrder"],json!(["enabled"]));
    }
    #[test]
    fn membership_null_and_empty_truth_table_is_wr_s8() {
        let all_null = json!([{"carrier":"string","value":null}]);
        let in_sql=response(request(json!({"kind":"in","fieldRef":{"module":"sales","element":"id"},"values":all_null}),json!([])));
        let not_in_sql=response(request(json!({"kind":"not-in","fieldRef":{"module":"sales","element":"id"},"values":[{"carrier":"string","value":null}]}),json!([])));
        assert_eq!(in_sql["status"],"compiled");assert_eq!(not_in_sql["status"],"compiled");
        let in_text=in_sql["sql"].as_str().unwrap();let not_in_text=not_in_sql["sql"].as_str().unwrap();
        assert!(in_text.contains("WHEN `r`.`customer_id` IS NULL THEN CAST(NULL AS BOOLEAN)"));
        assert!(in_text.contains("ELSE FALSE END"));
        assert!(not_in_text.contains("NOT (CASE WHEN `r`.`customer_id` IS NULL THEN CAST(NULL AS BOOLEAN)"));
        let empty_in=response(request(json!({"kind":"in","fieldRef":{"module":"sales","element":"note"},"values":[]}),json!([])));
        let empty_not_in=response(request(json!({"kind":"not-in","fieldRef":{"module":"sales","element":"note"},"values":[]}),json!([])));
        assert_eq!(empty_in["status"],"compiled");assert_eq!(empty_not_in["status"],"compiled");
        assert!(empty_in["sql"].as_str().unwrap().contains("FALSE"));
        assert!(empty_not_in["sql"].as_str().unwrap().contains("TRUE"));
    }
    #[test]
    fn membership_literals_are_data_and_empty_membership_is_constant() {
        let pred=json!({"kind":"in","fieldRef":{"module":"sales","element":"id"},"values":[{"carrier":"string","value":"x'\\雪 OR 1=1"},{"carrier":"string","value":null},{"carrier":"string","value":"x'\\雪 OR 1=1"}]});
        let out=response(request(pred,json!([])));assert_eq!(out["status"],"compiled");let sql=out["sql"].as_str().unwrap();assert!(sql.contains("decode(unhex('78275CE99BAA204F5220313D31'), 'UTF-8')"));assert!(sql.contains("CAST(NULL AS BOOLEAN)"));assert_eq!(out["parameterOrder"],json!([]));
        let empty=response(request(json!({"kind":"in","fieldRef":{"module":"sales","element":"id"},"values":[]}),json!([])));assert_eq!(empty["status"],"compiled");assert!(empty["sql"].as_str().unwrap().contains("FALSE"));
        assert_eq!(quote_literal("a'\\雪").unwrap(),"__weft_utf8_value('a''\\雪')");
    }
    #[test]
    fn backslash_sequences_remain_distinct_string_data() {
        for (value, expected_hex) in [("\\n", "5C6E"), ("\n", "0A"), ("\\b", "5C62"), ("\\q", "5C71"), ("雪", "E99BAA"), ("", "")] {
            let pred=json!({"kind":"in","fieldRef":{"module":"sales","element":"id"},"values":[{"carrier":"string","value":value}]});
            let out=response(request(pred,json!([])));assert_eq!(out["status"],"compiled","{value:?}: {out}");
            let sql=out["sql"].as_str().unwrap();
            assert!(sql.contains(&format!("decode(unhex('{expected_hex}'), 'UTF-8')")), "{value:?}: {sql}");
        }
    }
    #[test]
    fn membership_expansion_enforces_final_sql_byte_limit() {
        let values=(0..15).map(|_|json!({"carrier":"string","value":"a".repeat(4000)})).collect::<Vec<_>>();
        let pred=json!({"kind":"in","fieldRef":{"module":"sales","element":"id"},"values":values});
        let out=response(request(pred,json!([])));
        assert_eq!(out["status"],"blocked");
        assert_eq!(out["diagnostics"][0]["message"],"final lowered SQL byte limit exceeded");
    }
    #[test]
    fn reserved_and_malformed_predicates_refuse_without_losing_request_identity() {
        let cases=[
            json!({"kind":"comparison","op":"gt","fieldRef":{"module":"sales","element":"active"},"operand":{"parameter":"p"}}),
            json!({"kind":"comparison","op":"eq","fieldRef":{"module":"sales","element":"id"},"operand":{"parameter":"p"},"sql":"x"}),
            json!({"kind":"in","fieldRef":{"module":"sales","element":"id"},"values":[{"carrier":"boolean","value":true}]}),
            json!({"kind":"page","cursor":"opaque"}),
        ];
        for pred in cases {let out=response(request(pred,json!([{"name":"p","carrier":"string","nullable":false,"maxUtf8Bytes":64}])));assert_eq!(out["version"],RESPONSE_VERSION);assert_eq!(out["status"],"blocked");assert!(out["sql"].is_null());}
        let mut bad=request(eq_parameter("id","p"),json!([{"name":"p","carrier":"string","nullable":false,"maxUtf8Bytes":64}]));
        bad["expectedTarget"]["columns"][0]["nullable"]=json!(true);assert_eq!(response(bad)["status"],"blocked");
        let mut duplicate_native=request(eq_parameter("id","p"),json!([{"name":"p","carrier":"string","nullable":false,"maxUtf8Bytes":64}]));
        duplicate_native["expectedTarget"]["columns"][1]["nativeName"]=json!(duplicate_native["expectedTarget"]["columns"][0]["nativeName"]);
        assert_eq!(response(duplicate_native)["status"],"blocked");
        let mut bad=request(eq_parameter("id","p"),json!([{"name":"p","carrier":"string","nullable":false,"maxUtf8Bytes":64}]));
        bad["source"]["utf8"]=json!(bad["source"]["utf8"].as_str().unwrap().replace("predicate-fixture","tampered"));assert_eq!(response(bad)["status"],"blocked");
        let mut deep=json!({"kind":"not","term":{"kind":"not","term":{"kind":"not","term":{"kind":"not","term":{"kind":"is-null","fieldRef":{"module":"sales","element":"id"}}}}}});
        assert_eq!(response(request(deep.take(),json!([])))["status"],"blocked");
    }
    #[test]
    fn public_dispatch_keeps_v01_compiler_profile() {
        let model=json!({"umf":"0.10.0","id":"old","vocabularies":{},"modules":[{"id":"sales","namespace":"sales","elements":[
            {"id":"Customer","name":"Customer","kind":"record","members":[{"module":"sales","element":"id"},{"module":"sales","element":"note"}],"extensions":{}},
            {"id":"id","name":"id","kind":"field","scalarType":"string","cardinality":"one","nullability":"required","extensions":{}},
            {"id":"note","name":"note","kind":"field","scalarType":"string","cardinality":"one","nullability":"required","extensions":{}}
        ],"extensions":{}}]});let source=model.to_string();
        let old=json!({"version":"weft.unity-current-read-request/0.1","source":{"version":CORE_VERSION,"utf8":source,"sha256":weft_core::json::sha256(source.as_bytes())},"expectedTarget":{"kind":"unity-catalog","warehouseId":"w","catalog":"main","schema":"sales","table":"customers","tableUuid":"uuid","runtimeVersion":"16.4","columns":[{"fieldRef":{"module":"sales","element":"id"},"nativeName":"id","nativeType":"STRING","nullable":false},{"fieldRef":{"module":"sales","element":"note"},"nativeName":"note","nativeType":"STRING","nullable":false}]},"read":{"kind":"finite-current","tableRef":{"module":"sales","element":"Customer"},"projection":[{"module":"sales","element":"note"}],"predicate":{"kind":"eq","fieldRef":{"module":"sales","element":"id"},"parameter":"p1"},"rowCap":10,"frameCapBytes":65536,"deadlineMs":5000},"parameters":[{"name":"p1","carrier":"string","nullable":false,"maxUtf8Bytes":4096}]});
        let out:Value=serde_json::from_str(&super::super::unity_current_read::compile_json(&old.to_string())).unwrap();
        assert_eq!(out["version"],"weft.unity-current-selection/0.2");assert_eq!(out["compilerVersion"],"weft-databricks-unity-current-read/0.1");
    }
}
