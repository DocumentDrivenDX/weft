// SPDX-License-Identifier: MIT OR Apache-2.0
//! Original authored FK relationship operations; no engine graph substitution.
use super::*;
pub(super) fn index_profile(index:&str,table:&str,nonnull:bool)->String{
 let required=if nonnull{"a.attnotnull AND "}else{""};
 format!("EXISTS(SELECT 1 FROM pg_catalog.pg_class ix JOIN pg_catalog.pg_am am ON am.oid=ix.relam WHERE ix.oid={index}.indexrelid AND am.amname='btree' AND NOT EXISTS(SELECT 1 FROM pg_catalog.unnest({index}.indkey::pg_catalog.int2[]) WITH ORDINALITY AS k(attnum,pos) JOIN pg_catalog.pg_attribute a ON a.attrelid={table}.oid AND a.attnum=k.attnum JOIN pg_catalog.pg_type ty ON ty.oid=a.atttypid JOIN pg_catalog.pg_namespace tn ON tn.oid=ty.typnamespace LEFT JOIN pg_catalog.pg_opclass op ON op.oid={index}.indclass[(k.pos-1)::pg_catalog.int4] LEFT JOIN pg_catalog.pg_namespace ons ON ons.oid=op.opcnamespace LEFT JOIN pg_catalog.pg_collation co ON co.oid=a.attcollation WHERE k.pos<={index}.indnkeyatts AND ({required}NOT a.attisdropped AND tn.nspname='pg_catalog' AND ons.nspname='pg_catalog' AND op.opcmethod=ix.relam AND op.opcname=CASE ty.typname WHEN 'int2' THEN 'int2_ops' WHEN 'int4' THEN 'int4_ops' WHEN 'int8' THEN 'int8_ops' WHEN 'numeric' THEN 'numeric_ops' WHEN 'text' THEN 'text_ops' WHEN 'varchar' THEN 'text_ops' WHEN 'bool' THEN 'bool_ops' END AND {index}.indcollation[(k.pos-1)::pg_catalog.int4]=a.attcollation AND (a.attcollation=0 OR co.collisdeterministic)) IS NOT TRUE))")
}
pub(super) fn admission(a:&AdmittedRead,rel:&weft_core::raw_read::Relationship)->Result<Json>{
 let binding=&a.request().binding;let source=binding.tables.iter().find(|t|t.logical==rel.source).ok_or("FK source absent")?;let target=binding.tables.iter().find(|t|t.logical==rel.target).ok_or("FK target absent")?;
 let mut originals=vec![];for module in a.original_document()["modules"].as_array().unwrap(){if rel.module.as_ref().is_some_and(|m|module["id"]!=*m){continue}for r in module["relationships"].as_array().into_iter().flatten(){if r["id"]==rel.id{originals.push((module,r));}}}
 if originals.len()!=1{return Err("FK original relationship ambiguous")};let(module,original)=originals[0];
 let native=|table:&Table,logical:&str|table.columns.iter().find(|c|c.logical==logical).map(|c|c.native.clone()).ok_or("FK column absent");
 let source_fields=rel.pairs.iter().map(|p|native(source,&p[0])).collect::<Result<Vec<_>>>()?;let target_fields=rel.pairs.iter().map(|p|native(target,&p[1])).collect::<Result<Vec<_>>>()?;
 let literal=|v:&str|json!({"source":{"literal":{"string":v}},"carrier":"string"});
 let mut parameters=vec![match &binding.schema{Some(s)=>literal(s),None=>json!({"source":{"appSchema":true},"carrier":"string"})},literal(&source.native),literal(&target.native),literal(&serde_json::to_string(&source_fields).unwrap()),literal(&serde_json::to_string(&target_fields).unwrap())];
 for(i,p)in parameters.iter_mut().enumerate(){p["name"]=json!(format!("__umf_fk_{}",i+1));}
 let minimum=original["sourceMultiplicity"]["min"].as_u64().ok_or("original source participation minimum absent")?;let maximum=if original["sourceMultiplicity"]["max"]=="*"{None}else{Some(original["sourceMultiplicity"]["max"].as_u64().ok_or("original source participation maximum absent")?)};
 let target_minimum=original["targetMultiplicity"]["min"].as_u64().ok_or("original target participation minimum absent")?;
 let degree=minimum>0||maximum.is_some_and(|n|n!=1);
 let render=|dbapi:bool|->Result<String>{let slot=|n:usize|if dbapi{format!("%(__umf_fk_{n})s::pg_catalog.text")}else{format!("${n}::pg_catalog.text")};
  let attributes=|relation:&str,keys:&str|format!("(SELECT pg_catalog.jsonb_agg(at.attname ORDER BY k.ord) FROM pg_catalog.unnest({keys}) WITH ORDINALITY AS k(attnum,ord) JOIN pg_catalog.pg_attribute at ON at.attrelid={relation} AND at.attnum=k.attnum WHERE NOT at.attisdropped)");
  let source_names=attributes("s.oid","c.conkey");let target_names=attributes("t.oid","c.confkey");
  let required=if target_minimum==1{" AND NOT EXISTS(SELECT 1 FROM pg_catalog.unnest(c.conkey) AS k(attnum) JOIN pg_catalog.pg_attribute at ON at.attrelid=s.oid AND at.attnum=k.attnum WHERE NOT at.attnotnull)"}else{""};
  let match_type=if rel.pairs.len()==1{"c.confmatchtype IN ('s','f')".to_owned()}else{"(c.confmatchtype='s' OR (c.confmatchtype='f' AND NOT EXISTS(SELECT 1 FROM pg_catalog.unnest(c.conkey) AS k(attnum) JOIN pg_catalog.pg_attribute at ON at.attrelid=s.oid AND at.attnum=k.attnum WHERE NOT at.attnotnull)))".to_owned()};
  let mut sql=format!("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_constraint c JOIN pg_catalog.pg_class s ON s.oid=c.conrelid JOIN pg_catalog.pg_namespace ns ON ns.oid=s.relnamespace JOIN pg_catalog.pg_class t ON t.oid=c.confrelid JOIN pg_catalog.pg_namespace nt ON nt.oid=t.relnamespace WHERE ns.nspname={} AND nt.nspname={} AND s.relname={} AND t.relname={} AND s.relkind='r' AND t.relkind='r' AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_inherits inh WHERE inh.inhparent IN (s.oid,t.oid) OR inh.inhrelid IN (s.oid,t.oid)) AND c.contype='f' AND c.convalidated AND c.conenforced AND NOT c.condeferrable AND NOT c.condeferred AND NOT c.conperiod AND {match_type} AND {source_names}={}::pg_catalog.jsonb AND {target_names}={}::pg_catalog.jsonb AND pg_catalog.cardinality(c.conpfeqop)={} AND NOT EXISTS(SELECT 1 FROM pg_catalog.unnest(c.conpfeqop) AS e(oid) JOIN pg_catalog.pg_operator o ON o.oid=e.oid JOIN pg_catalog.pg_namespace opns ON opns.oid=o.oprnamespace WHERE opns.nspname<>'pg_catalog' OR o.oprname<>'=') AND (SELECT pg_catalog.count(*) FROM pg_catalog.pg_trigger tr JOIN pg_catalog.pg_proc p ON p.oid=tr.tgfoid JOIN pg_catalog.pg_namespace pn ON pn.oid=p.pronamespace WHERE tr.tgconstraint=c.oid AND tr.tgisinternal AND tr.tgenabled IN ('O','A') AND pn.nspname='pg_catalog' AND pg_catalog.starts_with(p.proname::pg_catalog.text,'RI_FKey_'))=4 AND NOT EXISTS(SELECT 1 FROM pg_catalog.unnest(c.conkey) AS fkcol(attnum) JOIN pg_catalog.pg_attribute fa ON fa.attrelid=s.oid AND fa.attnum=fkcol.attnum LEFT JOIN pg_catalog.pg_collation fc ON fc.oid=fa.attcollation WHERE fa.attcollation<>0 AND (fc.oid IS NULL OR NOT fc.collisdeterministic)) AND EXISTS(SELECT 1 FROM pg_catalog.pg_index fi WHERE fi.indexrelid=c.conindid AND NOT EXISTS(SELECT 1 FROM pg_catalog.unnest(fi.indkey::pg_catalog.int2[]) WITH ORDINALITY AS fkcol(attnum,ord) JOIN pg_catalog.pg_attribute fa ON fa.attrelid=t.oid AND fa.attnum=fkcol.attnum LEFT JOIN pg_catalog.pg_collation fc ON fc.oid=fa.attcollation WHERE fkcol.ord<=fi.indnkeyatts AND (fi.indcollation[(fkcol.ord-1)::pg_catalog.int4]<>fa.attcollation OR (fa.attcollation<>0 AND (fc.oid IS NULL OR NOT fc.collisdeterministic))))) AND pg_catalog.current_setting('session_replication_role')='origin'{required}",slot(1),slot(1),slot(2),slot(3),slot(4),slot(5),rel.pairs.len());
  sql+=&format!(" AND EXISTS(SELECT 1 FROM pg_catalog.pg_index exact_index WHERE exact_index.indexrelid=c.conindid AND exact_index.indisunique AND exact_index.indisvalid AND exact_index.indisready AND exact_index.indislive AND exact_index.indimmediate AND exact_index.indexprs IS NULL AND exact_index.indpred IS NULL AND exact_index.indnkeyatts=pg_catalog.cardinality(c.confkey) AND {})",index_profile("exact_index","t",false));
  if maximum==Some(1){sql+=&format!(" AND EXISTS(SELECT 1 FROM pg_catalog.pg_index exact_index WHERE exact_index.indrelid=s.oid AND exact_index.indisunique AND exact_index.indisvalid AND exact_index.indisready AND exact_index.indislive AND exact_index.indimmediate AND exact_index.indexprs IS NULL AND exact_index.indpred IS NULL AND exact_index.indnkeyatts=pg_catalog.cardinality(c.conkey) AND (SELECT pg_catalog.array_agg(k.attnum ORDER BY k.ord) FROM pg_catalog.unnest(exact_index.indkey::pg_catalog.int2[]) WITH ORDINALITY AS k(attnum,ord) WHERE k.ord<=exact_index.indnkeyatts)=c.conkey AND {})",index_profile("exact_index","s",false));}
  if degree {
   let qualified=|table:&Table|->Result<String>{Ok(match &binding.schema{Some(s)=>format!("{}.{}",ident(s)?,ident(&table.native)?),None=>ident(&table.native)?})};
   let pairs=rel.pairs.iter().map(|p|Ok(format!("{} = {}",column_sql("fk_source",source,&p[0])?,column_sql("fk_target",target,&p[1])?))).collect::<Result<Vec<_>>>()?.join(" AND ");
   let identity=source.key.iter().map(|f|column_sql("fk_source",source,f)).collect::<Result<Vec<_>>>()?.join(", ");let count=format!("(SELECT pg_catalog.count(DISTINCT ROW({identity})) FROM {} AS fk_source WHERE {pairs})",qualified(source)?);
   let mut failures=vec![format!("{count}<{minimum}")];if let Some(max)=maximum{failures.push(format!("{count}>{max}"));}
   sql+=&format!(" AND NOT pg_catalog.row_security_active(s.oid) AND NOT pg_catalog.row_security_active(t.oid) AND pg_catalog.current_setting('transaction_isolation') IN ('repeatable read','serializable') AND pg_catalog.current_setting('transaction_read_only')='on' AND NOT EXISTS(SELECT 1 FROM {} AS fk_target WHERE {})",qualified(target)?,failures.join(" OR "));
  }
  sql.push(')');Ok(sql)
 };
 Ok(json!({"module":module["id"],"id":rel.id,"sourceTable":source.logical,"targetTable":target.logical,"logicalPairs":rel.pairs,"nativePairs":source_fields.iter().zip(&target_fields).map(|(s,t)|json!([s,t])).collect::<Vec<_>>(),"sourceMultiplicity":original["sourceMultiplicity"],"targetMultiplicity":original["targetMultiplicity"],"completeNativeSnapshotRequired":degree,"nativeAdmission":{"sql":render(false)?,"dbapiSql":render(true)?,"parameters":parameters,"dbapiParameterOrder":parameters.iter().map(|p|p["name"].clone()).collect::<Vec<_>>()}}))
}
fn column_sql(alias:&str,table:&Table,logical:&str)->Result<String>{let c=table.columns.iter().find(|c|c.logical==logical).ok_or("FK field mapping absent")?;Ok(format!("{}.{}{}",ident(alias)?,ident(&c.native)?,if c.carrier==Carrier::String{" COLLATE pg_catalog.\"C\""}else{""}))}
struct Relation<'a>{binding:&'a weft_core::raw_read::Relationship,owner:String,target_alias:String,target:&'a Table,pairs:Vec<[String;2]>,fields:Vec<String>,decoder:Json}
fn arguments(f:&Function)->Result<&Vec<FunctionArg>>{
 if f.uses_odbc_syntax||!matches!(f.parameters,FunctionArguments::None)||!f.within_group.is_empty()||f.filter.is_some()||f.null_treatment.is_some()||f.over.is_some(){return Err("relationship function modifiers refused")}
 let FunctionArguments::List(args)=&f.args else{return Err("relationship arguments required")};
 if args.duplicate_treatment.is_some()||!args.clauses.is_empty(){return Err("relationship argument modifiers refused")};Ok(&args.args)
}
fn expression(arg:&FunctionArg)->Result<&Expr>{match arg{FunctionArg::Unnamed(FunctionArgExpr::Expr(e))=>Ok(e),_=>Err("relationship unnamed expression required")}}
fn mode(f:&Function)->Result<(bool,Option<(&str,&str)>)>{
 let parts=f.name.0.iter().map(|p|match p{ObjectNamePart::Identifier(i)=>name(i),_=>String::new()}).collect::<Vec<_>>();
 let inverse=parts==["has_inverse_related"]||parts==["inverse_related_keys"];let args=arguments(f)?;
 if args.len()!=if inverse{4}else{2}{return Err("relationship function exact arity required")}
 if !inverse{return Ok((false,None))}
 fn text(arg:&FunctionArg)->Result<&str>{let Expr::Value(v)=expression(arg)? else{return Err("inverse relationship literal canonical identity required")};let Value::SingleQuotedString(s)=&v.value else{return Err("inverse relationship canonical identity string required")};if s.is_empty()||s.contains('\0'){return Err("inverse relationship canonical identity invalid")};Ok(s)}
 Ok((true,Some((text(&args[2])?,text(&args[3])?))))
}
fn resolve<'a>(l:&Lower<'a>,field:&Expr,forced_inverse:bool,identity:Option<(&str,&str)>)->Result<Relation<'a>>{
 let Expr::CompoundIdentifier(parts)=field else{return Err("relationship needs explicit owner alias")};if parts.len()!=2{return Err("relationship path")}
 let owner=name(&parts[0]);if l.internal_scans.contains(&owner){return Err("internal relationship alias is not an authored source")}
 let table=l.scans.get(&owner).ok_or("relationship owner alias absent")?.0;let logical=name(&parts[1]);let doc=l.admitted.original_document();let mut choices=vec![];
 for binding in &l.request.binding.relationships {for module in doc["modules"].as_array().unwrap(){
  if binding.module.as_ref().is_some_and(|m|module["id"]!=*m){continue}
  for original in module["relationships"].as_array().into_iter().flatten(){if original["id"]!=binding.id{continue}
   if identity.is_some_and(|(m,id)|module["id"]!=m||original["id"]!=id){continue}
   if forced_inverse {if original["name"]==logical&&binding.target==table.logical{choices.push((binding,module,original,true))}}else{
    if original["name"]==logical&&binding.source==table.logical{choices.push((binding,module,original,false))}
    if original["inverse"]==logical&&binding.target==table.logical{choices.push((binding,module,original,true))}
   }
  }
 }}
 if choices.len()!=1{return Err("relationship name absent or ambiguous in original owner")};let(binding,module,original,inverse)=choices[0];
 let target=l.request.binding.tables.iter().find(|t|t.logical==if inverse{binding.source.clone()}else{binding.target.clone()}).ok_or("relationship endpoint binding absent")?;
 let record=l.admitted.original_record(&target.logical)?;let keys=record.value["keys"].as_array().ok_or("relationship endpoint authored Keys absent")?;
 let key=if inverse{let primaries=keys.iter().filter(|k|k["primary"]==true).collect::<Vec<_>>();if primaries.len()==1{primaries[0]}else if primaries.is_empty()&&keys.len()==1{&keys[0]}else{return Err("inverse authored default Key ambiguous")}}else{let found=keys.iter().filter(|k|k["id"]==original["target"][0]["key"]).collect::<Vec<_>>();if found.len()!=1{return Err("relationship target Key absent")};found[0]};
 let mut fields=vec![];let mut components=vec![];
 for reference in key["fields"].as_array().ok_or("relationship Key Fields absent")?{
  let field=target.columns.iter().find(|c|l.admitted.original_field_ref(&target.logical,&c.logical).is_ok_and(|r|r==*reference)).ok_or("relationship Key component unbound")?;
  if field.nullable{return Err("relationship Key native nullable component")};fields.push(field.logical.clone());components.push(json!({"field":reference,"carrier":field.carrier,"nullable":false}));
 }
 if fields.is_empty()||fields.len()>16{return Err("relationship Key arity bound")}
 let mut target_alias="__umf_relationship".to_owned();while l.scans.contains_key(&target_alias){target_alias.push('_')}
 let pairs=binding.pairs.iter().map(|p|if inverse{[p[1].clone(),p[0].clone()]}else{p.clone()}).collect();
 Ok(Relation{binding,owner,target_alias,target,pairs,fields,decoder:json!({"profile":"pg-related-keys-json/0.1","relationship":{"module":module["id"],"id":binding.id,"direction":if inverse{"inverse"}else{"forward"}},"key":{"module":record.identity.module,"record":record.identity.element,"key":key["id"],"components":components}})})
}
fn operand<'a>(l:&mut Lower<'a>,field:&Expr,forced_inverse:bool,identity:Option<(&str,&str)>)->Result<Relation<'a>>{
 let mut r=resolve(l,field,forced_inverse,identity)?;
 // Reuse the registration selected before projection/native admission.
 if let Some((alias,_))=l.scans.iter().find(|(a,(t,_))|l.internal_scans.contains(*a)&&t.logical==r.target.logical){r.target_alias=alias.clone()}else{l.scans.insert(r.target_alias.clone(),(r.target,false));l.internal_scans.insert(r.target_alias.clone());}
 let original=admission(l.admitted,r.binding)?;l.relationship_admissions.insert(weft_core::mutation::canonical(&json!([original["module"],original["id"]])),original);
 if r.fields!=r.target.key{let key=relationship_key_metadata(r.target,&r.fields,&l.request.binding.schema)?;l.relationship_keys.insert(weft_core::mutation::canonical(&r.decoder["key"]),key);}
 Ok(r)
}
fn column(l:&Lower<'_>,alias:&str,table:&Table,logical:&str)->Result<String>{let c=table.columns.iter().find(|c|c.logical==logical).ok_or("relationship component mapping absent")?;l.used_columns.borrow_mut().insert((alias.to_owned(),logical.to_owned()));Ok(format!("{}.{}{}",ident(alias)?,ident(&c.native)?,if c.carrier==Carrier::String{" COLLATE pg_catalog.\"C\""}else{""}))}
fn from_where(l:&Lower<'_>,r:&Relation<'_>)->Result<(String,String)>{
 let owner=l.scans[&r.owner].0;let table=match &l.request.binding.schema{Some(s)=>format!("{}.{}",ident(s)?,ident(&r.target.native)?),None=>ident(&r.target.native)?};
 let pairs=r.pairs.iter().map(|p|Ok(format!("{} = {}",column(l,&r.owner,owner,&p[0])?,column(l,&r.target_alias,r.target,&p[1])?))).collect::<Result<Vec<_>>>()?;
 Ok((format!("{table} AS {}",ident(&r.target_alias)?),pairs.join(" AND ")))
}
pub(super) fn prepare(l:&mut Lower<'_>,e:&Expr)->Result<()>{
 match e{
  Expr::Function(f)=>{let n=f.name.0.iter().map(|p|match p{ObjectNamePart::Identifier(i)=>name(i),_=>String::new()}).collect::<Vec<_>>();if n==["has_related"]||n==["related_keys"]||n==["has_inverse_related"]||n==["inverse_related_keys"]{let args=arguments(f)?;let(inverse,identity)=mode(f)?;operand(l,expression(&args[0])?,inverse,identity)?;}if let FunctionArguments::List(args)=&f.args{for a in &args.args{if let FunctionArg::Unnamed(FunctionArgExpr::Expr(e))=a{prepare(l,e)?}}}},
  Expr::Nested(e)|Expr::UnaryOp{expr:e,..}|Expr::IsNull(e)|Expr::IsNotNull(e)=>prepare(l,e)?,
  Expr::BinaryOp{left,right,..}|Expr::IsDistinctFrom(left,right)|Expr::IsNotDistinctFrom(left,right)=>{prepare(l,left)?;prepare(l,right)?;},
  Expr::InList{expr,list,..}=>{prepare(l,expr)?;for e in list{prepare(l,e)?}},Expr::Between{expr,low,high,..}=>{prepare(l,expr)?;prepare(l,low)?;prepare(l,high)?},_=>{}
 };Ok(())
}
pub(super) fn has(l:&mut Lower<'_>,f:&Function)->Result<String>{
 let args=arguments(f)?;let(inverse,identity)=mode(f)?;let r=operand(l,expression(&args[0])?,inverse,identity)?;
 let Expr::Function(key)=expression(&args[1])? else{return Err("HAS_RELATED original KEY tuple required")};if key.name.to_string().to_ascii_lowercase()!="key"{return Err("HAS_RELATED KEY required")};let values=arguments(key)?;if values.len()!=r.fields.len(){return Err("HAS_RELATED complete original Key arity")}
 let(from,mut predicate)=from_where(l,&r)?;
 for (logical,arg)in r.fields.iter().zip(values){let c=r.target.columns.iter().find(|c|c.logical==*logical).unwrap();let v=l.value(expression(arg)?,false)?;if v.carrier!=c.carrier||v.nullable{return Err("HAS_RELATED Key component domain differs")};predicate+=&format!(" AND {} = {}",column(l,&r.target_alias,r.target,logical)?,v.sql);}
 Ok(format!("EXISTS(SELECT 1 FROM {from} WHERE {predicate})"))
}
pub(super) fn keys(l:&mut Lower<'_>,f:&Function)->Result<(String,Json)>{
 let args=arguments(f)?;let(inverse,identity)=mode(f)?;let r=operand(l,expression(&args[0])?,inverse,identity)?;
 let Expr::Value(v)=expression(&args[1])? else{return Err("RELATED_KEYS literal bound required")};let Value::Number(token,false)=&v.value else{return Err("RELATED_KEYS positive integer bound")};let n=token.parse::<u16>().ok().filter(|n|*n>0&&*n<=1000).ok_or("RELATED_KEYS bound outside 1..1000")?;if token!=&n.to_string(){return Err("RELATED_KEYS canonical bound")}
 let(from,predicate)=from_where(l,&r)?;let mut values=vec![];let mut ordering=vec![];
 for logical in &r.fields{let c=r.target.columns.iter().find(|c|c.logical==*logical).unwrap();let sql=column(l,&r.target_alias,r.target,logical)?;ordering.push(format!("{sql} ASC"));values.push(if c.carrier==Carrier::Boolean{sql}else{format!("({sql})::pg_catalog.text")});}
 let order=ordering.join(", ");let sql=format!("(SELECT pg_catalog.jsonb_build_object('items',COALESCE(pg_catalog.jsonb_agg(q.k ORDER BY q.ord) FILTER(WHERE q.ord<={n}),'[]'::pg_catalog.jsonb),'truncated',pg_catalog.count(*)>{n})::pg_catalog.text FROM (SELECT pg_catalog.jsonb_build_array({}) AS k,pg_catalog.row_number() OVER(ORDER BY {order}) AS ord FROM {from} WHERE {predicate} ORDER BY {order} LIMIT {}) AS q)",values.join(", "),u32::from(n)+1);
 let mut decoder=r.decoder;decoder["bound"]=json!(n);Ok((sql,decoder))
}
