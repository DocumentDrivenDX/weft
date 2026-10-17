//! Compiler-owned asynchronous UC candidate search and current single-key hydration.
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use sqlparser::ast::*;
use weft_core::{raw_read,search};
type Result<T>=std::result::Result<T,&'static str>;
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct VectorBinding{field:Value,native:String,dimensions:u16,#[serde(default,skip_serializing_if="Option::is_none")]native_type:Option<String>}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct ProjectionBinding{field:Value,native:String,native_type:String}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct BindingDocument{source_id:String,document_json:String,document_sha256:String}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct NativeColumns{column_set:search::Source,binding:BindingDocument}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct Managed{target:Value,catalog:String,schema:String,table_id:String,index_type:String,endpoint_name:String,index_name:String,vector_binding:VectorBinding,#[serde(default,skip_serializing_if="Vec::is_empty")]projection_bindings:Vec<ProjectionBinding>,native_columns:NativeColumns}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct Request{version:String,source:search::Source,selection:raw_read::Request,query_parameter:String,top_k:u16,projection:Vec<search::Projection>,allow_partial:bool,managed:Managed,fusion:Option<Value>,keyword_source:Option<search::Source>,text_query_parameter:Option<String>}
fn ident(s:&str)->Result<String>{if s.is_empty()||s.len()>255||s.contains('\0'){return Err("bounded native identifier required")};Ok(format!("\"{}\"",s.replace('"',"\"\"")))}
fn plain(s:&str)->bool{!s.is_empty()&&s.len()<=255&&s.bytes().all(|c|c.is_ascii_alphanumeric()||c==b'_')}
fn predicate_fields(e:&Expr,out:&mut Vec<String>)->Result<()>{match e{
 Expr::CompoundIdentifier(p) if p.len()==2=>out.push(if p[1].quote_style.is_some(){p[1].value.clone()}else{p[1].value.to_ascii_lowercase()}),
 Expr::Value(_)=>{},Expr::Nested(e)|Expr::IsNull(e)|Expr::IsNotNull(e)|Expr::UnaryOp{expr:e,..}=>predicate_fields(e,out)?,
 Expr::BinaryOp{left,right,..}=>{predicate_fields(left,out)?;predicate_fields(right,out)?},
 Expr::Between{expr,low,high,..}=>{predicate_fields(expr,out)?;predicate_fields(low,out)?;predicate_fields(high,out)?},
 Expr::InList{expr,list,..}=>{predicate_fields(expr,out)?;for v in list{predicate_fields(v,out)?}},
 _=>return Err("predicate field inventory unsupported")};Ok(())}
fn field_key(v:&Value)->String{weft_core::mutation::canonical(v)}
fn exact_keys(v:&Value,required:&[&str],optional:&[&str])->bool{
 v.as_object().is_some_and(|m|required.iter().all(|k|m.contains_key(*k))&&m.keys().all(|k|required.contains(&k.as_str())||optional.contains(&k.as_str())))
}
fn native_type(raw:&str)->Result<String>{
 if raw.len()>100||!raw.is_ascii(){return Err("bounded native type spelling required")}
 let chars=raw.chars().collect::<Vec<_>>();
 for (i,c) in chars.iter().enumerate(){if c.is_ascii_whitespace(){
   let before=chars[..i].iter().rev().find(|c|!c.is_ascii_whitespace());let after=chars[i+1..].iter().find(|c|!c.is_ascii_whitespace());
   if before.is_some_and(|c|c.is_ascii_alphanumeric()||*c=='_')&&after.is_some_and(|c|c.is_ascii_alphanumeric()||*c=='_'){return Err("native type tokens cannot be joined across whitespace")}
 }}
 Ok(chars.into_iter().filter(|c|!c.is_ascii_whitespace()).collect::<String>().to_ascii_lowercase())
}
fn native_columns(r:&Request,doc:&Value,table:&raw_read::Table,record:&Value)->Result<std::collections::BTreeMap<String,(String,String,bool)>>{
 let m=&r.managed;let custody=&m.native_columns;let source=&custody.column_set;
 if doc["vocabularies"]["umf.access"]["version"]!="0.1.0"||source.document_id!=doc["id"]||source.core_version!="0.8.0"||source.document_sha256!=r.selection.binding.model_sha256{return Err("ColumnSet original model source differs")}
 let mut sets=vec![];
 for (mi,module) in doc["modules"].as_array().unwrap().iter().enumerate(){
   for (ci,set) in module["extensions"]["umf.access"]["columns"].as_array().into_iter().flatten().enumerate(){
     if source.pointer==format!("/modules/{mi}/extensions/umf.access/columns/{ci}")&&source.module==module["id"]{sets.push(set)}
   }
 }
 if sets.len()!=1{return Err("unique source-pinned original ColumnSet required")}
 let set=sets[0];let b=&custody.binding;
 if !exact_keys(set,&["binding","target","columns"],&[])||!exact_keys(&set["binding"],&["sourceId","module"],&[])||b.source_id.is_empty(){return Err("closed original ColumnSet required")}
 let binding=weft_core::json::checked_json(&b.document_json).map_err(|_|"original binding JSON refused")?;
 if weft_core::json::sha256(b.document_json.as_bytes())!=b.document_sha256||binding["umf"]!="0.8.0"||binding["vocabularies"]["umf.binding"]["version"]!="0.2.0"||binding["id"].as_str().is_none_or(str::is_empty)||set["binding"]!=json!({"sourceId":b.source_id,"module":record["module"]})||set["target"]!=m.target{return Err("ColumnSet binding/source/target differs")}
 let payload=&binding["extensions"]["umf.binding"];
 if !exact_keys(payload,&["profile","logical","target","elements","fields","relationships","indexes"],&[])||!exact_keys(&payload["logical"],&["documentId","coreVersion"],&[])||!exact_keys(&payload["target"],&["system","version"],&["subset"]){return Err("closed original binding profile required")}
 if payload["profile"]!="umf-binding-2"||payload["logical"]!=json!({"documentId":doc["id"],"coreVersion":"0.8.0"})||payload["target"]["system"]!=m.target["platform"]||payload["target"]["version"]!=m.target["majorVersion"]||payload["target"]["subset"]!="typed-access-columns/0.1"{return Err("original binding logical/native target differs")}
 let elements=payload["elements"].as_array().ok_or("binding element inventory absent")?.iter().filter(|v|v["module"]==record["module"]&&v["element"]==record["element"]).collect::<Vec<_>>();
 if elements.len()!=1||!exact_keys(elements[0],&["module","element","table"],&[])||elements[0]["table"]!=table.native{return Err("original native Record table differs")}
 let record_body=doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==record["module"]).and_then(|m|m["elements"].as_array().unwrap().iter().find(|e|e["id"]==record["element"])).ok_or("original ColumnSet Record absent")?;
 let mut columns=std::collections::BTreeMap::new();let mut physical=std::collections::BTreeSet::new();
 for column in set["columns"].as_array().ok_or("ColumnSet columns absent")? {
   if !exact_keys(column,&["field","type","notNull","generation"],&[])||!exact_keys(&column["field"],&["module","element"],&[]){return Err("closed original NativeColumn required")}
   let field=&column["field"];if !record_body["members"].as_array().unwrap().contains(field){continue}
   let fields=payload["fields"].as_array().ok_or("binding field inventory absent")?.iter().filter(|v|v["module"]==field["module"]&&v["element"]==field["element"]).collect::<Vec<_>>();
   if fields.len()!=1||!exact_keys(fields[0],&["module","element","storage","column"],&[])||fields[0]["storage"]!="column"{return Err("original physical Field column absent/ambiguous")}
   let native=fields[0]["column"].as_str().ok_or("original native column name absent")?;ident(native)?;
   let ty=native_type(column["type"].as_str().ok_or("original native column type absent")?)?;
   let not_null=column["notNull"].as_bool().ok_or("original ColumnSet notNull required")?;
   if !(column["generation"].is_null()||matches!(column["generation"].as_str(),Some("identity"|"stored-expression"|"default")))||column.get("generation").is_none(){return Err("original native generation unsupported")}
   if !physical.insert(native.to_string())||columns.insert(field_key(field),(native.to_string(),ty,not_null)).is_some(){return Err("ambiguous original ColumnSet column inventory")}
 }
 Ok(columns)
}
fn compile(raw:&str)->Result<Value>{
 let r:Request=serde_json::from_value(weft_core::json::checked_json(raw).map_err(|_|"bounded UC search JSON required")?).map_err(|_|"exact UC search request required")?;
 let m=&r.managed;
 if r.version!="weft.unity-search/0.1"||!r.allow_partial||r.selection.target!="unity-catalog-current"||r.selection.keyset||r.selection.snapshots.is_some()||r.selection.binding.schema.as_deref()!=Some(&m.schema)||m.index_type!="DIRECT_ACCESS"||m.target["platform"]!="unity-catalog"||m.target["majorVersion"].as_str().is_none_or(str::is_empty)||m.target.as_object().is_none_or(|v|v.len()!=2)||![&m.catalog,&m.schema].iter().all(|s|plain(s))||m.endpoint_name.is_empty()||m.endpoint_name.len()>255||!m.endpoint_name.bytes().all(|c|c.is_ascii_alphanumeric()||matches!(c,b'_'|b'-'))||m.index_name.split('.').count()!=3||!m.index_name.split('.').all(plain)||m.table_id.len()!=36||!m.table_id.bytes().all(|c|c.is_ascii_hexdigit()||c==b'-') {return Err("explicit bounded managed current profile required")}
 if m.vector_binding.native_type.as_deref().is_some_and(|v|v!="array<float>"){return Err("managed Vector native type must be ARRAY<FLOAT>")}
 if r.keyword_source.is_some()!=r.text_query_parameter.is_some(){return Err("HYBRID requires both original keyword source and text parameter")}
 if r.keyword_source.is_some(){if r.fusion!=Some(json!({"kind":"target-defined"}))||r.top_k>200{return Err("native HYBRID requires explicit target-defined fusion and topK<=200")}}else if r.fusion.is_some(){return Err("ANN cannot carry fusion")}
 let native_vector=search::VectorBinding{field:m.vector_binding.field.clone(),native:m.vector_binding.native.clone(),dimensions:m.vector_binding.dimensions,extension_schema:String::new(),extension_version:String::new()};
 let original=search::SearchRequest{version:"weft.search/0.1".into(),source:serde_json::from_value(serde_json::to_value(&r.source).unwrap()).unwrap(),selection:serde_json::from_value(serde_json::to_value(&r.selection).unwrap()).unwrap(),query_parameter:r.query_parameter.clone(),top_k:r.top_k,projection:serde_json::from_value(serde_json::to_value(&r.projection).unwrap()).unwrap(),vector_binding:Some(native_vector),allow_partial:true};
 let admitted=search::admit_unity_json(&serde_json::to_string(&original).unwrap(),&m.target)?;
 if admitted.index()["kind"]!="vector"||admitted.binding()["native"]!=m.index_name{return Err("original managed vector native mapping differs")}

 let q=admitted.selection().query();
 if q.with.is_some()||q.limit_clause.is_some()||q.order_by.is_some(){return Err("search cannot truncate/reorder original predicate")}
 let SetExpr::Select(s)=q.body.as_ref()else{return Err("single source selection required")};
 if !s.optimizer_hints.is_empty()||s.select_modifiers.is_some(){return Err("selected query modifier outside managed profile")}
 if s.from.len()!=1||!s.from[0].joins.is_empty(){return Err("single source current hydration required")}
 let TableFactor::Table{name,alias:Some(alias),..}=&s.from[0].relation else{return Err("owned aliased source required")};
 if !alias.columns.is_empty(){return Err("column alias inventories unsupported")}
 let alias_name=if alias.name.quote_style.is_some(){alias.name.value.clone()}else{alias.name.value.to_ascii_lowercase()};let alias=alias_name.as_str();
 let path=name.0.iter().map(|p|match p{ObjectNamePart::Identifier(i)=>Ok(if i.quote_style.is_some(){i.value.clone()}else{i.value.to_ascii_lowercase()}),_=>Err("static source name required")}).collect::<Result<Vec<_>>>()?;
 let table=r.selection.binding.tables.iter().find(|t|t.logical==path).ok_or("original bound source absent")?;
 let logical_columns=table.columns.iter().map(|c|&c.logical).collect::<std::collections::BTreeSet<_>>();
 let native_column_names=table.columns.iter().map(|c|&c.native).collect::<std::collections::BTreeSet<_>>();
 if logical_columns.len()!=table.columns.len()||native_column_names.len()!=table.columns.len(){return Err("ambiguous current scalar column inventory")}
 if table.key.len()!=1{return Err("initial native search requires single selected Key")}
 if table.columns.iter().any(|c|c.native==m.vector_binding.native){return Err("vector native column collides with scalar binding")}
 let key=table.columns.iter().find(|c|c.logical==table.key[0]).ok_or("selected Key column absent")?;
 if key.carrier!=raw_read::Carrier::String||key.nullable{return Err("native search Key must be required String")}
 let doc:Value=serde_json::from_str(&r.selection.binding.model_json).unwrap();
 let module=doc["modules"].as_array().unwrap().iter().find(|v|v["id"]==admitted.index()["record"]["module"]).ok_or("original Record module absent")?;
 if path!=vec![module["namespace"].as_str().ok_or("Record namespace absent")?.to_string(),admitted.record()["name"].as_str().ok_or("Record name absent")?.to_string()]{return Err("selected Record/table owner differs")}
 let original_columns=native_columns(&r,&doc,table,&admitted.index()["record"])?;
 let (vector_native,vector_type,_)=original_columns.get(&field_key(&m.vector_binding.field)).ok_or("original vector ColumnSet absent")?;
 if vector_native!=&m.vector_binding.native||vector_type!="array<float>"{return Err("original vector binding must be ARRAY<FLOAT>")}
 let views=doc["modules"].as_array().unwrap().iter().flat_map(|m|m["extensions"]["umf.access"]["views"].as_array().into_iter().flatten()).filter(|v|v["record"]==admitted.index()["record"]).collect::<Vec<_>>();
 if views.len()!=1{return Err("unique original current Record view required")}
 let row_nullable=|reference:&Value|->Result<bool>{
   let matches=views[0]["fields"].as_array().ok_or("original current view fields absent")?.iter().filter(|f|f["field"]==*reference).collect::<Vec<_>>();
   if matches.len()!=1{return Err("unique original current Field view required")}
   let nullable=matches[0]["row"]["null"].as_bool().ok_or("original current row null policy required")?;
   let owner=doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==reference["module"]).ok_or("row view module absent")?;
   let f=owner["elements"].as_array().unwrap().iter().find(|f|f["id"]==reference["element"]).ok_or("row view Field absent")?;
   if matches[0]["row"]["missing"]!=false||!matches!(f["nullability"].as_str(),Some("required"|"absent-allowed"))||(nullable&&f["nullability"]!="absent-allowed"){return Err("current row view contradicts core/relational presence")}
   let (_,_,not_null)=original_columns.get(&field_key(reference)).ok_or("current Field absent from original ColumnSet")?;
   if *not_null==nullable{return Err("original ColumnSet notNull contradicts row view")}
   for module in doc["modules"].as_array().unwrap(){for set in module["extensions"]["umf.access"]["columns"].as_array().into_iter().flatten(){for column in set["columns"].as_array().into_iter().flatten(){if column["field"]==*reference&&column["notNull"]!=!nullable{return Err("another authored ColumnSet contradicts original row view")}}}}
   Ok(nullable)
 };
 for c in &table.columns {
   let members=admitted.record()["members"].as_array().unwrap();
   let matches=members.iter().filter(|member|doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==member["module"]).and_then(|m|m["elements"].as_array().unwrap().iter().find(|f|f["id"]==member["element"])).is_some_and(|f|f["name"]==c.logical)).collect::<Vec<_>>();
   if matches.len()!=1||c.nullable!=row_nullable(matches[0])?{return Err("current scalar nullability differs from original row view")}
 }
 // Additional current-only physical shapes are source-bound, never inferred.
 let mut shaped=std::collections::BTreeMap::new();let mut physical=table.columns.iter().map(|c|c.native.clone()).collect::<std::collections::BTreeSet<_>>();
 physical.insert(m.vector_binding.native.clone());
 for binding in &m.projection_bindings {
   if shaped.contains_key(&field_key(&binding.field))||binding.field==m.vector_binding.field{return Err("duplicate current projection shape")}
   if !admitted.record()["members"].as_array().unwrap().contains(&binding.field){return Err("projection shape outside original Record")}
   let owner=doc["modules"].as_array().unwrap().iter().find(|v|v["id"]==binding.field["module"]).ok_or("projection shape module absent")?;
   let f=owner["elements"].as_array().unwrap().iter().find(|v|v["id"]==binding.field["element"]).ok_or("projection shape Field absent")?;
   if f["kind"]!="field"||f["cardinality"]!="one"||f["extensions"].as_object().is_none_or(|v|!v.is_empty())||f.get("allowedValues").is_some(){return Err("current projection constraints not qualified")}
   let facets=f.get("facets").cloned().unwrap_or(json!({}));
   let mut meta=json!({"carrier":f["scalarType"],"nullable":row_nullable(&binding.field)?,"nativeType":binding.native_type});
   match f["scalarType"].as_str(){
     Some("integer")=>{let w=&facets["integerWidth"];let ty=match w["bits"].as_u64(){Some(8)=>"tinyint",Some(16)=>"smallint",Some(32)=>"int",Some(64)=>"bigint",_=>return Err("exact signed native integer width required")};
       if w["signed"]!=true||w.as_object().is_none_or(|v|v.len()!=2)||facets.as_object().is_none_or(|v|v.len()!=1)||binding.native_type!=ty{return Err("integer native shape differs")};meta["integerWidth"]=w.clone();},
     Some("decimal")=>{let precision=facets["precision"].as_u64().filter(|v|(1..=38).contains(v)).ok_or("native Decimal precision required")?;let scale=facets["scale"].as_u64().filter(|v|*v<=precision).ok_or("native Decimal scale required")?;
       if facets.as_object().is_none_or(|v|v.len()!=2)||binding.native_type!=format!("decimal({precision},{scale})"){return Err("Decimal native shape differs")};meta["precision"]=json!(precision);meta["scale"]=json!(scale);},
     Some("binary")=>{if facets.as_object().is_none_or(|v|!v.is_empty())||binding.native_type!="binary"{return Err("Binary native shape differs")}},
     _=>return Err("current projection shape carrier unsupported")
   }
   let logical=f["name"].as_str().ok_or("projection shape name absent")?;
   if let Some(c)=table.columns.iter().find(|c|c.logical==logical){
     if c.native!=binding.native||serde_json::to_value(&c.carrier).unwrap()!=meta["carrier"]||c.nullable!=meta["nullable"]{return Err("current projection shape differs from scalar binding")}
   }else if !physical.insert(binding.native.clone()){return Err("current projection native column collision")}
   ident(&binding.native)?;
   shaped.insert(field_key(&binding.field),(binding.native.clone(),logical.to_string(),meta));
 }
 for c in &table.columns {if matches!(c.carrier,raw_read::Carrier::Integer|raw_read::Carrier::Decimal)&&!shaped.values().any(|(native,logical,_)|native==&c.native&&logical==&c.logical){return Err("all numeric bound columns require original physical shapes")}}
 let _original=crate::raw_read::lower_current_shaped(admitted.selection(),&m.catalog)?;
 let mut used_fields=vec![];if let Some(e)=&s.selection{predicate_fields(e,&mut used_fields)?}
 let mut predicate_refs=vec![];
 for field in used_fields{let f=module["elements"].as_array().unwrap().iter().find(|v|v["kind"]=="field"&&v["name"]==field).ok_or("predicate Field original source absent")?;let reference=json!({"module":module["id"],"element":f["id"]});predicate_refs.push(reference.clone());if !admitted.index()["filterable"].as_array().unwrap().contains(&reference){return Err("predicate Field not declared filterable")}}
 let mut keyword=Value::Null;
 if let(Some(source),Some(text))=(&r.keyword_source,&r.text_query_parameter){
   if text==&r.query_parameter||text.is_empty()||text.len()>64||r.selection.parameters.iter().any(|p|&p.name==text){return Err("distinct bounded HYBRID parameter required")}
   let mut k:Value=serde_json::to_value(&original).unwrap();k["source"]=serde_json::to_value(source).unwrap();k["queryParameter"]=json!(text);
   let a=search::admit_unity_json(&k.to_string(),&m.target)?;
   if a.index()["kind"]!="keyword"||a.index()["record"]!=admitted.index()["record"]||a.binding()["native"]!=m.index_name{return Err("HYBRID original source/native owner differs")}
   // Native HYBRID searches every indexed STRING column: require exact authored inventory.
   let fields=a.index()["keyword"]["source"].as_array().ok_or("HYBRID source inventory required")?;
   let strings=table.columns.iter().filter(|c|c.carrier==raw_read::Carrier::String).map(|c|{
     module["elements"].as_array().unwrap().iter().find(|f|f["name"]==c.logical&&f["kind"]=="field").map(|f|json!({"module":module["id"],"element":f["id"]})).ok_or("HYBRID source Field mapping absent")
   }).collect::<Result<Vec<_>>>()?;
   if fields.len()!=strings.len()||strings.iter().any(|f|!fields.contains(f)){return Err("HYBRID must author full native STRING source inventory")}
   if predicate_refs.iter().any(|f|!a.index()["filterable"].as_array().unwrap().contains(f)){return Err("HYBRID predicate Field not declared filterable")}
   keyword=a.index().clone();
 }
 // Rebuild only the closed original logical AST; the runtime receives final native SQL.
 let mut native_projection=vec![];
 let mut current_columns=serde_json::to_value(&table.columns).unwrap().as_array().unwrap().clone();
 for (native,logical,metadata) in shaped.values(){
   let mut column=metadata.clone();column["native"]=json!(native);column["logical"]=json!(logical);
   if let Some(existing)=current_columns.iter_mut().find(|c|c["logical"]==*logical){*existing=column}else{current_columns.push(column)}
 }
 for column in &mut current_columns {
   let logical=column["logical"].as_str().unwrap();
   let refs=admitted.record()["members"].as_array().unwrap().iter().filter(|reference|doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==reference["module"]).and_then(|m|m["elements"].as_array().unwrap().iter().find(|f|f["id"]==reference["element"])).is_some_and(|f|f["name"]==logical)).collect::<Vec<_>>();
   if refs.len()!=1{return Err("current column logical owner ambiguous")}
   let (native,ty,_)=original_columns.get(&field_key(refs[0])).ok_or("current column original declaration absent")?;
   let expected=if let Some(ty)=column.get("nativeType").and_then(Value::as_str){ty}else{match column["carrier"].as_str(){Some("string")=>"string",Some("boolean")=>"boolean",_=>return Err("explicit native shape required")}};
   if column["native"]!=*native||expected!=ty{return Err("current physical column/type differs from original binding/ColumnSet")}
   if column["carrier"]=="string"{column["collation"]=json!("UTF8_BINARY")}
 }
 for p in &r.projection{
   let owner=doc["modules"].as_array().unwrap().iter().find(|v|v["id"]==p.field["module"]).ok_or("projection source module absent")?;
   let field=owner["elements"].as_array().unwrap().iter().find(|v|v["id"]==p.field["element"]).ok_or("projection source Field absent")?;
   if p.field==m.vector_binding.field {
     let ty=m.vector_binding.native_type.as_deref().filter(|v|*v=="array<float>").ok_or("projected Vector requires exact native array type")?;
     let nullable=row_nullable(&p.field)?;
     let (native,original_type,_)=original_columns.get(&field_key(&p.field)).ok_or("original Vector ColumnSet absent")?;
     if native!=&m.vector_binding.native||original_type!=ty{return Err("Vector physical mapping differs from original ColumnSet")}

     let output=json!({"name":p.name,"carrier":"vector","nullable":nullable,"dimensions":m.vector_binding.dimensions,"nativeType":ty});
     native_projection.push((m.vector_binding.native.clone(),output));
     let column=json!({"logical":field["name"],"native":m.vector_binding.native,"carrier":"vector","nullable":nullable,"dimensions":m.vector_binding.dimensions,"nativeType":ty});
     if !current_columns.contains(&column){current_columns.push(column)}
   }else if let Some((native,_,metadata))=shaped.get(&field_key(&p.field)){
     let mut output=metadata.clone();output["name"]=json!(p.name);native_projection.push((native.clone(),output));
   }else{
     let col=table.columns.iter().find(|c|Some(c.logical.as_str())==field["name"].as_str()).ok_or("projected nonscalar not qualified for current hydration")?;
     native_projection.push((col.native.clone(),json!({"name":p.name,"carrier":col.carrier,"nullable":col.nullable})));
   }
 }
 native_projection.push((key.native.clone(),json!({"name":"__umf_key_0","carrier":"string","nullable":false})));
 let predicate=s.selection.as_ref().map(|e|format!("({e}) AND ")).unwrap_or_default();
 let logical=path.iter().map(|v|ident(v)).collect::<Result<Vec<_>>>()?.join(".");
 let mut hydration:raw_read::Request=serde_json::from_value(serde_json::to_value(&r.selection).unwrap()).unwrap();
 hydration.sql=format!("SELECT {} FROM {logical} AS {} WHERE {predicate}{}.{} = :__umf_candidate_key",format!("{}.{} AS \"__umf_key_0\"",ident(alias)?,ident(&key.logical)?),ident(alias)?,ident(alias)?,ident(&key.logical)?);
 if hydration.parameters.iter().any(|p|p.name=="__umf_candidate_key"){return Err("reserved candidate parameter collision")}
 hydration.parameters.push(raw_read::Parameter{name:"__umf_candidate_key".into(),carrier:raw_read::Carrier::String,nullable:false});hydration.row_cap=2;
 let h=raw_read::admit_json(&serde_json::to_string(&hydration).unwrap())?;
 let lowered=crate::raw_read::lower_current_projected(&h,&m.catalog,&native_projection)?;
 let bindings=lowered["parameterOrder"].as_array().unwrap().iter().enumerate().map(|(i,p)|{
   let name=p.as_str().unwrap();let def=hydration.parameters.iter().find(|p|p.name==name).unwrap();
   json!({"name":format!("p{}",i+1),"source":if name=="__umf_candidate_key"{json!({"candidateKey":0})}else{json!({"parameter":name})},"carrier":def.carrier,"nullable":def.nullable})
 }).collect::<Vec<_>>();
 let mut query_parameters=vec![json!({"name":r.query_parameter,"source":{"parameter":r.query_parameter},"destination":"query_vector","carrier":"vector"})];
 if let Some(text)=&r.text_query_parameter{query_parameters.push(json!({"name":text,"source":{"parameter":text},"destination":"query_text","carrier":"string"}))}
 let mut outputs=lowered["outputs"].as_array().unwrap().clone();let hidden=outputs.pop().unwrap();
 let mut out=json!({"version":"weft.unity-search/0.1","status":"compiled","profile":"unity-catalog-current/0.1","request":r,"source":admitted.request().source,"index":admitted.index(),"keywordIndex":keyword,"nativeIndex":m.index_name,"query":{"method":"POST","path":format!("/api/2.0/vector-search/indexes/{}/query",m.index_name),"body":{"query_type":if keyword.is_null(){"ANN"}else{"HYBRID"},"num_results":r.top_k,"columns":[key.native]},"parameters":query_parameters},"hydration":{"statement":{"sql":lowered["sql"],"parameters":bindings},"outputs":outputs,"hiddenKey":[hidden]},"obligations":{"allowPartial":true,"atMostKReads":r.top_k,"preserveNativeOrdinal":true,"refill":false,"nativeAdmissionEachOperation":true,"currentTableId":m.table_id,"principalRequired":true,"scoreKind":"implementation-defined","metric":"l2","temporalQualification":"held","nativeIndex":{"name":m.index_name,"endpointName":m.endpoint_name,"indexType":"DIRECT_ACCESS","primaryKey":key.native,"vectorColumn":m.vector_binding.native,"dimensions":m.vector_binding.dimensions,"stringColumns":table.columns.iter().filter(|c|c.carrier==raw_read::Carrier::String).map(|c|c.native.clone()).collect::<std::collections::BTreeSet<_>>()},"currentTable":{"catalog":m.catalog,"schema":m.schema,"table":table.native,"id":m.table_id,"columns":current_columns}}});
 out["structuralDigest"]=json!({"owner":"weft-databricks-search","version":"0.1","sha256":weft_core::json::sha256(weft_core::mutation::canonical(&out).as_bytes())});Ok(out)
}
pub fn compile_json(raw:&str)->String{compile(raw).unwrap_or_else(|reason|json!({"version":"weft.unity-search/0.1","status":"blocked","diagnostics":[{"code":"WFT-UNITY-SEARCH","message":reason}]})).to_string()}
#[cfg(test)]
mod tests{
 use super::*;
 fn request()->Value{
  let mut r:Value=serde_json::from_str(include_str!("../../../tests/native-search/keyword-request.json")).unwrap();
  let mut doc:Value=serde_json::from_str(r["selection"]["binding"]["modelJson"].as_str().unwrap()).unwrap();
  let access=&mut doc["modules"][0]["extensions"]["umf.access"];
  for i in access["search"].as_array_mut().unwrap(){i["exactness"]=json!("approximate");if i["kind"]=="vector"{i["vector"]["metric"]=json!("l2")}else{i["keyword"]["analyzer"]=json!("implementation-defined");i["keyword"]["source"]=json!([{"module":"search","element":"id"},{"module":"search","element":"body"}])}}
  for b in access["searchBindings"].as_array_mut().unwrap(){b["target"]=json!({"platform":"unity-catalog","majorVersion":"2026-10-16"});b["native"]=json!("sdk_qualification.sdk_search.docs_index");b["freshness"]=json!("asynchronous");b["filtering"]=json!("partial")}
  let model=doc.to_string();let digest=weft_core::json::sha256(model.as_bytes());
  r["selection"]["binding"]["modelJson"]=json!(model);r["selection"]["binding"]["modelSha256"]=json!(digest);r["source"]["documentSha256"]=json!(digest);
  r.as_object_mut().unwrap().remove("vectorBinding");
  r["source"]["pointer"]=json!("/modules/0/extensions/umf.access/search/1");r["version"]=json!("weft.unity-search/0.1");r["allowPartial"]=json!(true);r["selection"]["target"]=json!("unity-catalog-current");r["selection"]["binding"]["schema"]=json!("sdk_search");r["projection"].as_array_mut().unwrap().pop();
  r["managed"]=json!({"target":{"platform":"unity-catalog","majorVersion":"2026-10-16"},"catalog":"sdk_qualification","schema":"sdk_search","tableId":"00000000-0000-0000-0000-000000000001","indexType":"DIRECT_ACCESS","endpointName":"praxis-attachments","indexName":"sdk_qualification.sdk_search.docs_index","vectorBinding":{"field":{"module":"search","element":"embedding"},"native":"embedding","dimensions":3}});native_fixture(&mut r);r
 }
 fn native_fixture(r:&mut Value){
   let mut doc:Value=serde_json::from_str(r["selection"]["binding"]["modelJson"].as_str().unwrap()).unwrap();
   let members=doc["modules"][0]["elements"].as_array().unwrap().iter().find(|e|e["id"]=="Doc").unwrap()["members"].as_array().unwrap().clone();
   let old_view=doc["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"].as_array().unwrap().clone();
   let mut ordered_view=members.iter().map(|reference|old_view.iter().find(|v|v["field"]==*reference).unwrap().clone()).collect::<Vec<_>>();
   ordered_view.extend(old_view.into_iter().filter(|v|!members.contains(&v["field"])));
   doc["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"]=json!(ordered_view);
   let mut columns=vec![];let mut fields=vec![];
   for reference in members {
     let f=doc["modules"][0]["elements"].as_array().unwrap().iter().find(|f|f["id"]==reference["element"]).unwrap();
     let (native,ty)=if reference==r["managed"]["vectorBinding"]["field"]{(r["managed"]["vectorBinding"]["native"].clone(),json!("ARRAY<FLOAT>"))}else if let Some(shape)=r["managed"]["projectionBindings"].as_array().into_iter().flatten().find(|b|b["field"]==reference){(shape["native"].clone(),shape["nativeType"].clone())}else{let c=r["selection"]["binding"]["tables"][0]["columns"].as_array().unwrap().iter().find(|c|c["logical"]==f["name"]).unwrap();(c["native"].clone(),json!(if c["carrier"]=="string"{"STRING"}else{"BOOLEAN"}))};
     let view=doc["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"].as_array().unwrap().iter().find(|v|v["field"]==reference).unwrap();
     columns.push(json!({"field":reference,"type":ty,"notNull":!view["row"]["null"].as_bool().unwrap(),"generation":null}));
     fields.push(json!({"module":reference["module"],"element":reference["element"],"storage":"column","column":native}));
   }
   doc["modules"][0]["extensions"]["umf.access"]["columns"]=json!([{"binding":{"sourceId":"uc-search-binding","module":"search"},"target":r["managed"]["target"],"columns":columns}]);
   let binding=json!({"umf":"0.8.0","id":"uc-search-binding","vocabularies":{"umf.binding":{"version":"0.2.0"}},"modules":[],"extensions":{"umf.binding":{"profile":"umf-binding-2","logical":{"documentId":doc["id"],"coreVersion":"0.8.0"},"target":{"system":"unity-catalog","version":r["managed"]["target"]["majorVersion"],"subset":"typed-access-columns/0.1"},"elements":[{"module":"search","element":"Doc","table":"docs"}],"fields":fields,"relationships":[],"indexes":[]}}}).to_string();
   r["managed"]["nativeColumns"]=json!({"columnSet":{"documentId":doc["id"],"coreVersion":"0.8.0","documentSha256":"pending","module":"search","pointer":"/modules/0/extensions/umf.access/columns/0"},"binding":{"sourceId":"uc-search-binding","documentJson":binding,"documentSha256":weft_core::json::sha256(binding.as_bytes())}});
   repin(r,doc);
 }
 #[test]fn ann_current_hydration_and_native_hybrid_are_owned(){
  let mut r=request();let p=compile(&r.to_string()).unwrap();assert_eq!(p["query"]["body"]["query_type"],"ANN");assert!(p["hydration"]["statement"]["sql"].as_str().unwrap().contains("`sdk_qualification`.`sdk_search`.`docs`"));assert!(!p["hydration"]["statement"]["sql"].as_str().unwrap().contains("VERSION AS OF"));assert_eq!(p["hydration"]["statement"]["parameters"][0]["source"],json!({"candidateKey":0}));
  r["keywordSource"]=r["source"].clone();r["keywordSource"]["pointer"]=json!("/modules/0/extensions/umf.access/search/0");r["textQueryParameter"]=json!("text");r["fusion"]=json!({"kind":"target-defined"});let p=compile(&r.to_string()).unwrap();assert_eq!(p["query"]["body"]["query_type"],"HYBRID");assert_eq!(p["query"]["parameters"].as_array().unwrap().len(),2);
 }
 #[test]fn original_vector_projection_has_exact_native_carrier(){
   for ty in ["array<float>"] {
     let mut r=request();r["managed"]["vectorBinding"]["nativeType"]=json!(ty);
     r["projection"].as_array_mut().unwrap().push(json!({"field":{"module":"search","element":"embedding"},"name":"embedding"}));
     let p=compile(&r.to_string()).unwrap();
     assert_eq!(p["hydration"]["outputs"][2],json!({"name":"embedding","carrier":"vector","nullable":false,"dimensions":3,"nativeType":ty}));
     assert!(p["hydration"]["statement"]["sql"].as_str().unwrap().contains("`r`.`embedding` AS `embedding`"));
     assert_eq!(p["obligations"]["currentTable"]["columns"][2]["nativeType"],ty);
     r["managed"]["vectorBinding"].as_object_mut().unwrap().remove("nativeType");assert!(compile(&r.to_string()).is_err());
     r["managed"]["vectorBinding"]["nativeType"]=json!("array<double>");assert!(compile(&r.to_string()).is_err());
   }
 }
 fn shaped_request()->Value{
   let mut r=request();let mut doc:Value=serde_json::from_str(r["selection"]["binding"]["modelJson"].as_str().unwrap()).unwrap();
   for (id,carrier,facets,ty) in [("count","integer",json!({"integerWidth":{"bits":64,"signed":true}}),"bigint"),("amount","decimal",json!({"precision":28,"scale":6}),"decimal(28,6)"),("payload","binary",json!({}),"binary")]{
     let elements=doc["modules"][0]["elements"].as_array_mut().unwrap();
     elements.push(json!({"id":id,"name":id,"kind":"field","scalarType":carrier,"cardinality":"one","nullability":"required","extensions":{},"facets":facets}));
     elements.iter_mut().find(|e|e["id"]=="Doc").unwrap()["members"].as_array_mut().unwrap().push(json!({"module":"search","element":id}));
     doc["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"].as_array_mut().unwrap().push(json!({"field":{"module":"search","element":id},"row":{"missing":false,"null":false},"create":{"missing":false,"null":false},"mutable":true,"generated":false}));
     if carrier!="binary"{r["selection"]["binding"]["tables"][0]["columns"].as_array_mut().unwrap().push(json!({"logical":id,"native":id,"carrier":carrier,"nullable":false}));}
     if r["managed"]["projectionBindings"].is_null(){r["managed"]["projectionBindings"]=json!([])}
     r["managed"]["projectionBindings"].as_array_mut().unwrap().push(json!({"field":{"module":"search","element":id},"native":id,"nativeType":ty}));
     r["projection"].as_array_mut().unwrap().push(json!({"field":{"module":"search","element":id},"name":id}));
   }
   repin(&mut r,doc);native_fixture(&mut r);r
 }
 #[test]fn numeric_and_binary_projection_shapes_are_exact(){
   let original=compile(include_str!("../../../tests/native-search/unity-shaped-request.json")).unwrap();
   assert!(original["hydration"]["outputs"].as_array().unwrap().iter().any(|o|o["precision"]==38&&o["scale"]==10));
   let r=shaped_request();let p=compile(&r.to_string()).unwrap();
   assert_eq!(p["hydration"]["outputs"][2],json!({"name":"count","carrier":"integer","nullable":false,"nativeType":"bigint","integerWidth":{"bits":64,"signed":true}}));
   assert_eq!(p["hydration"]["outputs"][3]["precision"],28);assert_eq!(p["hydration"]["outputs"][3]["scale"],6);
   assert_eq!(p["hydration"]["outputs"][4]["carrier"],"binary");
   let sql=p["hydration"]["statement"]["sql"].as_str().unwrap();assert!(sql.contains("`r`.`payload` AS `payload`"));assert!(!sql.contains("CAST"));
   for (bits,ty) in [(8,"tinyint"),(16,"smallint"),(32,"int"),(64,"bigint")] {
     let mut q=shaped_request();let mut doc:Value=serde_json::from_str(q["selection"]["binding"]["modelJson"].as_str().unwrap()).unwrap();
     doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|f|f["id"]=="count").unwrap()["facets"]["integerWidth"]["bits"]=json!(bits);
     q["managed"]["projectionBindings"][0]["nativeType"]=json!(ty);repin(&mut q,doc);native_fixture(&mut q);assert!(compile(&q.to_string()).is_ok());
   }
   for (bits,signed) in [(7,true),(128,true),(64,false)] {
     let mut q=shaped_request();let mut doc:Value=serde_json::from_str(q["selection"]["binding"]["modelJson"].as_str().unwrap()).unwrap();
     doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|f|f["id"]=="count").unwrap()["facets"]["integerWidth"]=json!({"bits":bits,"signed":signed});repin(&mut q,doc);assert!(compile(&q.to_string()).is_err());
   }
   for mode in 0..4 {let mut bad=shaped_request();match mode{0=>bad["managed"]["projectionBindings"][0]["nativeType"]=json!("int"),1=>bad["managed"]["projectionBindings"][1]["nativeType"]=json!("decimal(28,5)"),2=>bad["managed"]["projectionBindings"][2]["nativeType"]=json!("string"),_=>{bad["managed"]["projectionBindings"].as_array_mut().unwrap().remove(0);}};assert!(compile(&bad.to_string()).is_err(),"shape mode {mode}")}
 }
 #[test]fn current_nullability_preserves_original_view_and_column_sets(){
   let mut double=request();double["managed"]["vectorBinding"]["nativeType"]=json!("array<double>");assert!(compile(&double.to_string()).is_err());
   let mut r=request();r["selection"]["binding"]["tables"][0]["columns"][1]["nullable"]=json!(true);assert!(compile(&r.to_string()).is_err());
   let mut doc:Value=serde_json::from_str(r["selection"]["binding"]["modelJson"].as_str().unwrap()).unwrap();
   doc["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|f|f["id"]=="body").unwrap()["nullability"]=json!("absent-allowed");
   r["selection"]["binding"]["tables"][0]["columns"][1]["nullable"]=json!(false);repin(&mut r,doc.clone());
   assert_eq!(compile(&r.to_string()).unwrap()["hydration"]["outputs"][1]["nullable"],false);
   doc["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"][1]["row"]["null"]=json!(true);
   r["selection"]["binding"]["tables"][0]["columns"][1]["nullable"]=json!(true);repin(&mut r,doc.clone());native_fixture(&mut r);
   assert_eq!(compile(&r.to_string()).unwrap()["hydration"]["outputs"][1]["nullable"],true);
   doc["modules"][0]["extensions"]["umf.access"]["views"][0]["fields"][1]["row"]["missing"]=json!(true);repin(&mut r,doc);assert!(compile(&r.to_string()).is_err());
 }
 #[test]fn original_column_custody_refuses_mismatches(){
   for mode in 0..8 {let mut r=request();match mode{
     0=>r["managed"]["nativeColumns"]["binding"]["documentSha256"]=json!("0".repeat(64)),
     1=>r["managed"]["nativeColumns"]["columnSet"]["pointer"]=json!("/modules/0/extensions/umf.access/columns/9"),
     2=>r["managed"]["nativeColumns"]["binding"]["sourceId"]=json!("different-alias"),
     3=>{r["managed"].as_object_mut().unwrap().remove("nativeColumns");},
     _=>{let mut doc:Value=serde_json::from_str(r["selection"]["binding"]["modelJson"].as_str().unwrap()).unwrap();match mode{
       4=>doc["modules"][0]["extensions"]["umf.access"]["columns"][0]["columns"][0]["notNull"]=json!(false),
       5=>doc["modules"][0]["extensions"]["umf.access"]["columns"][0]["columns"][0]["type"]=json!("BINARY"),
       6=>doc["modules"][0]["extensions"]["umf.access"]["columns"][0]["columns"][2]["type"]=json!("ARRAY<DOUBLE>"),
       _=>{let mut other=doc["modules"][0]["extensions"]["umf.access"]["columns"][0].clone();other["columns"][0]["notNull"]=json!(false);doc["modules"][0]["extensions"]["umf.access"]["columns"].as_array_mut().unwrap().push(other);}
     };repin(&mut r,doc);}
   };assert!(compile(&r.to_string()).is_err(),"custody mode {mode}")}
 }
 #[test]fn unsupported_native_meaning_is_closed(){for mode in 0..6{let mut r=request();match mode{0=>r["allowPartial"]=json!(false),1=>r["managed"]["tableId"]=json!("unknown"),2=>r["managed"]["indexName"]=json!("different.index.name"),3=>r["selection"]["sql"]=json!("SELECT r.id AS id FROM search.Doc AS r LIMIT 1"),4=>r["managed"]["vectorBinding"]["dimensions"]=json!(4),_=>r["textQueryParameter"]=json!("text")};assert!(compile(&r.to_string()).is_err(),"mode {mode}")}}
 fn repin(r:&mut Value,doc:Value){let model=doc.to_string();let digest=weft_core::json::sha256(model.as_bytes());r["selection"]["binding"]["modelJson"]=json!(model);r["selection"]["binding"]["modelSha256"]=json!(digest);r["source"]["documentSha256"]=json!(digest);if !r["keywordSource"].is_null(){r["keywordSource"]["documentSha256"]=json!(digest)}if !r["managed"]["nativeColumns"].is_null(){r["managed"]["nativeColumns"]["columnSet"]["documentSha256"]=json!(digest)}}
 #[test]fn predicate_and_hybrid_inventory_never_silently_broaden(){
  let mut r=request();r["selection"]["sql"]=json!("SELECT r.id AS id FROM search.\"Doc\" AS r WHERE r.body = :body");r["selection"]["parameters"]=json!([{"name":"body","carrier":"string","nullable":false}]);
  let p=compile(&r.to_string()).unwrap();let stmt=&p["hydration"]["statement"];assert_eq!(stmt["parameters"][0]["source"],json!({"parameter":"body"}));assert_eq!(stmt["parameters"][1]["source"],json!({"candidateKey":0}));assert!(stmt["sql"].as_str().unwrap().contains("`r`.`body` = :p1"));
  let mut doc:Value=serde_json::from_str(r["selection"]["binding"]["modelJson"].as_str().unwrap()).unwrap();doc["modules"][0]["extensions"]["umf.access"]["search"][1]["filterable"]=json!([]);repin(&mut r,doc);assert_eq!(compile(&r.to_string()).unwrap_err(),"predicate Field not declared filterable");
  let mut r=request();r["keywordSource"]=r["source"].clone();r["keywordSource"]["pointer"]=json!("/modules/0/extensions/umf.access/search/0");r["textQueryParameter"]=json!("text");r["fusion"]=json!({"kind":"target-defined"});
  let mut doc:Value=serde_json::from_str(r["selection"]["binding"]["modelJson"].as_str().unwrap()).unwrap();doc["modules"][0]["extensions"]["umf.access"]["search"][0]["keyword"]["source"]=json!([{"module":"search","element":"body"}]);repin(&mut r,doc);assert_eq!(compile(&r.to_string()).unwrap_err(),"HYBRID must author full native STRING source inventory");
 }

}
