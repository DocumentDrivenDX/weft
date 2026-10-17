//! Explicit native text-search deployment fence. Metadata never grants authority.
use serde_json::{json,Value};
use std::collections::BTreeMap;
use weft_core::raw_read::Binding;
type Result<T> = std::result::Result<T,&'static str>;
const VERSION:&str="umf.pg-ts-fence/0.1";
const CATALOGS:[&str;7]=["pg_namespace","pg_proc","pg_ts_config","pg_ts_config_map","pg_ts_dict","pg_ts_parser","pg_ts_template"];
const VIEW_QUERY:&str=r#"SELECT cfg.oid AS config_oid,m.mapcfg,d.oid AS dict_oid,t.oid AS template_oid,p.oid AS parser_oid,n.oid AS namespace_oid,proc.oid AS procedure_oid FROM pg_catalog.pg_ts_config cfg LEFT JOIN pg_catalog.pg_ts_config_map m ON m.mapcfg=cfg.oid LEFT JOIN pg_catalog.pg_ts_dict d ON d.oid=m.mapdict LEFT JOIN pg_catalog.pg_ts_template t ON t.oid=d.dicttemplate LEFT JOIN pg_catalog.pg_ts_parser p ON p.oid=cfg.cfgparser LEFT JOIN pg_catalog.pg_namespace n ON n.oid=cfg.cfgnamespace LEFT JOIN pg_catalog.pg_proc proc ON proc.oid=p.prsstart"#;
const VIEW_DEFINITION:&str=r#" SELECT cfg.oid AS config_oid,
    m.mapcfg,
    d.oid AS dict_oid,
    t.oid AS template_oid,
    p.oid AS parser_oid,
    n.oid AS namespace_oid,
    proc.oid AS procedure_oid
   FROM ((((((pg_ts_config cfg
     LEFT JOIN pg_ts_config_map m ON ((m.mapcfg = cfg.oid)))
     LEFT JOIN pg_ts_dict d ON ((d.oid = m.mapdict)))
     LEFT JOIN pg_ts_template t ON ((t.oid = d.dicttemplate)))
     LEFT JOIN pg_ts_parser p ON ((p.oid = cfg.cfgparser)))
     LEFT JOIN pg_namespace n ON ((n.oid = cfg.cfgnamespace)))
     LEFT JOIN pg_proc proc ON ((proc.oid = (p.prsstart)::oid)));"#;
fn ident(value:&str)->String {format!("\"{}\"",value.replace('"',"\"\""))}
fn admission(schema:&str,view:&str,owner:&str)->Value {
 let columns=json!([ [1,"config_oid",26,-1,0,false],[2,"mapcfg",26,-1,0,false],[3,"dict_oid",26,-1,0,false],[4,"template_oid",26,-1,0,false],[5,"parser_oid",26,-1,0,false],[6,"namespace_oid",26,-1,0,false],[7,"procedure_oid",26,-1,0,false] ]);
 let catalogs=CATALOGS.iter().map(|name|format!("('pg_catalog.{name}'::pg_catalog.regclass)")).collect::<Vec<_>>().join(",");
 let render=|dbapi:bool|{
  let p=|n:usize|if dbapi {format!("%(__umf_catalog_fence_{n})s::pg_catalog.text")}else{format!("${n}::pg_catalog.text")};
  let schema=p(1);let view=p(2);let owner=p(3);let definition=p(4);let columns=p(5);
  format!(r#"WITH expected(relation) AS (VALUES {catalogs})
SELECT (pg_catalog.current_setting('server_version_num')='180006' AND (pg_catalog.current_schemas(true))[1]='pg_catalog'
 AND v.relkind='v' AND NOT v.relispartition AND v.reloptions IS NULL AND n.nspowner=o.oid
 AND NOT o.rolsuper AND NOT o.rolbypassrls AND NOT o.rolcanlogin AND NOT o.rolcreatedb AND NOT o.rolcreaterole AND NOT o.rolreplication
 AND NOT me.rolsuper AND NOT me.rolbypassrls AND NOT pg_catalog.pg_has_role(me.oid,o.oid,'MEMBER')
 AND pg_catalog.has_schema_privilege(me.oid,n.oid,'USAGE') AND NOT pg_catalog.has_schema_privilege(me.oid,n.oid,'CREATE')
 AND pg_catalog.has_table_privilege(me.oid,v.oid,'MAINTAIN')
 AND pg_catalog.pg_get_viewdef(v.oid,false)={definition}
 AND (SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_array(a.attnum,a.attname,a.atttypid::pg_catalog.int8,a.atttypmod,a.attcollation::pg_catalog.int8,a.attnotnull) ORDER BY a.attnum) FROM pg_catalog.pg_attribute a WHERE a.attrelid=v.oid AND a.attnum>0 AND NOT a.attisdropped)={columns}::pg_catalog.jsonb
 AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_attribute a WHERE a.attrelid=v.oid AND a.attnum>0 AND (a.attisdropped OR a.atthasdef OR a.attidentity<>'' OR a.attgenerated<>''))
 AND (SELECT pg_catalog.count(*) FROM pg_catalog.pg_rewrite r WHERE r.ev_class=v.oid)=1
 AND EXISTS(SELECT 1 FROM pg_catalog.pg_rewrite r WHERE r.ev_class=v.oid AND r.rulename='_RETURN' AND r.ev_type='1' AND r.is_instead
  AND (SELECT pg_catalog.count(*) FROM pg_catalog.pg_depend d WHERE d.classid='pg_catalog.pg_rewrite'::pg_catalog.regclass AND d.objid=r.oid)=1
  AND EXISTS(SELECT 1 FROM pg_catalog.pg_depend d WHERE d.classid='pg_catalog.pg_rewrite'::pg_catalog.regclass AND d.objid=r.oid AND d.objsubid=0 AND d.refclassid='pg_catalog.pg_class'::pg_catalog.regclass AND d.refobjid=v.oid AND d.refobjsubid=0 AND d.deptype='i'))
 AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_inherits i WHERE i.inhrelid=v.oid OR i.inhparent=v.oid OR i.inhrelid IN (SELECT relation FROM expected) OR i.inhparent IN (SELECT relation FROM expected))
 AND NOT EXISTS(SELECT 1 FROM expected e JOIN pg_catalog.pg_class c ON c.oid=e.relation WHERE c.relkind<>'r' OR c.relispartition OR NOT pg_catalog.has_table_privilege(o.oid,e.relation,'MAINTAIN'))
 AND NOT EXISTS(SELECT 1 FROM (SELECT relation FROM expected UNION ALL SELECT v.oid) e WHERE NOT EXISTS(SELECT 1 FROM pg_catalog.pg_lock_status() AS l WHERE l.locktype='relation' AND l.pid=pg_catalog.pg_backend_pid() AND l.database=(SELECT oid FROM pg_catalog.pg_database WHERE datname=pg_catalog.current_database()) AND l.relation=e.relation AND l.mode='ShareLock' AND l.granted))
 ) AS correspondence
FROM pg_catalog.pg_class v JOIN pg_catalog.pg_namespace n ON n.oid=v.relnamespace JOIN pg_catalog.pg_roles o ON o.oid=v.relowner JOIN pg_catalog.pg_roles me ON me.rolname=current_user
WHERE n.nspname={schema} AND v.relname={view} AND o.rolname={owner}"#)
 };
 let parameters=[schema,view,owner,VIEW_DEFINITION,&columns.to_string()].iter().enumerate().map(|(i,value)|json!({"name":format!("__umf_catalog_fence_{}",i+1),"carrier":"string","source":{"literal":{"string":value}}})).collect::<Vec<_>>();
 json!({"sql":render(false),"dbapiSql":render(true),"dbapiParameterOrder":parameters.iter().map(|p|p["name"].clone()).collect::<Vec<_>>(),"parameters":parameters})
}
pub(crate) fn attach(binding:&Binding,obligations:&mut Value)->Result<()> {
 let mut origins:BTreeMap<String,Value>=BTreeMap::new();
 for (kind,entries) in [("termFilter","indexes"),("keywordConfiguration","configurations"),("textSearchEffects","indexes")] {
  if let Some(packet)=obligations.get(kind){for item in packet[entries].as_array().ok_or("native catalog fence original inventory absent")?{
   let configuration=item["configuration"].as_str().ok_or("native catalog fence configuration absent")?;
   if !matches!(configuration,"pg_catalog.simple"|"pg_catalog.english")||!item["source"].is_object()||!item["index"].is_object(){return Err("native catalog fence unsupported original configuration")}
   let origin=json!({"source":item["source"],"index":item["index"],"configuration":configuration});origins.insert(weft_core::mutation::canonical(&origin),origin);
  }}
 }
 if origins.is_empty(){return Ok(())}
 let fence=binding.native_catalog_fence.as_ref().ok_or("TEXT_SEARCH_FENCE: explicit original deployment required")?;
 let qualified=format!("{}.{}",ident(&fence.schema),ident(&fence.view));
 obligations["nativeCatalogFence"]=json!({"version":VERSION,"deployment":fence,"origins":origins.into_values().collect::<Vec<_>>(),"lockSql":format!("LOCK TABLE {qualified} IN SHARE MODE NOWAIT"),"lockPosition":"before-first-snapshot","nativeCatalogBeforeAfter":true,"nativeAdmission":admission(&fence.schema,&fence.view,&fence.owner),"deploymentProgram":{"version":"weft.pg-ts-fence-deployment/0.1","createViewSql":format!("CREATE VIEW {qualified} AS {VIEW_QUERY}"),"alterOwnerSql":format!("ALTER VIEW {qualified} OWNER TO {}",ident(&fence.owner)),"ownerPrerequisites":{"role":fence.owner,"superuser":false,"bypassRls":false,"login":false,"createRole":false,"createDb":false,"replication":false,"schemaOwner":true,"catalogMaintain":CATALOGS.iter().map(|n|format!("pg_catalog.{n}")).collect::<Vec<_>>()},"readerPrerequisites":{"schemaUsage":true,"viewMaintain":true,"ownerMembership":false}}});Ok(())
}

pub(crate) fn seal_version(program:&mut Value)->Result<()> {
 if program["obligations"].get("nativeCatalogFence").is_some()||program["obligations"].get("mutationIndexEffects").is_some(){
  program["version"]=json!(match program["version"].as_str().ok_or("fenced artifact version absent")?{
   "weft.raw-read/0.1"|"weft.raw-read/0.2"=>"weft.raw-read/0.2",
   "weft.search-input/0.1"|"weft.search-input/0.2"=>"weft.search-input/0.2",
   "weft.search/0.1"|"weft.search/0.2"=>"weft.search/0.2",
   "weft.native-action/0.1"|"weft.native-action/0.2"=>"weft.native-action/0.2",
   "weft.native-bulk/0.1"|"weft.native-bulk/0.2"=>"weft.native-bulk/0.2",
   _=>return Err("unsupported fenced artifact family"),
  });
 }
 Ok(())
}

pub(crate) fn mutation_index_admission(table:&weft_core::raw_read::Table,schema:&Option<String>,allowed:&[String],keys:&[Vec<String>])->Value {
 let render=|dbapi:bool|{
  let p=|n:usize|if dbapi{format!("%(__umf_index_effects_{n})s::pg_catalog.text")}else{format!("${n}::pg_catalog.text")};
  let scope=match schema{Some(_)=>format!("n.nspname={} AND t.relname={}",p(1),p(2)),None=>format!("t.oid=pg_catalog.to_regclass(pg_catalog.quote_ident({}))",p(2))};
  format!("SELECT (t.relkind='r' AND NOT t.relispartition AND EXISTS(SELECT 1 FROM pg_catalog.pg_lock_status() l WHERE l.locktype='relation' AND l.pid=pg_catalog.pg_backend_pid() AND l.database=(SELECT oid FROM pg_catalog.pg_database WHERE datname=pg_catalog.current_database()) AND l.relation=t.oid AND l.mode='ShareRowExclusiveLock' AND l.granted) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_index i JOIN pg_catalog.pg_class x ON x.oid=i.indexrelid JOIN pg_catalog.pg_am am ON am.oid=x.relam WHERE i.indrelid=t.oid AND NOT (x.relname IN (SELECT pg_catalog.jsonb_array_elements_text({}::pg_catalog.jsonb)) OR (i.indisunique AND i.indisvalid AND i.indisready AND i.indislive AND i.indimmediate AND i.indpred IS NULL AND i.indexprs IS NULL AND i.indnatts=i.indnkeyatts AND (SELECT pg_catalog.jsonb_agg(a.attname ORDER BY k.ordinality) FROM pg_catalog.unnest(i.indkey::pg_catalog.int2[]) WITH ORDINALITY AS k(attnum,ordinality) JOIN pg_catalog.pg_attribute a ON a.attrelid=t.oid AND a.attnum=k.attnum WHERE k.ordinality<=i.indnkeyatts) IN (SELECT pg_catalog.jsonb_array_elements({}::pg_catalog.jsonb)) AND NOT EXISTS(SELECT 1 FROM pg_catalog.unnest(i.indkey::pg_catalog.int2[]) WITH ORDINALITY AS k(attnum,ordinality) LEFT JOIN pg_catalog.pg_attribute a ON a.attrelid=t.oid AND a.attnum=k.attnum WHERE k.ordinality<=i.indnkeyatts AND (a.attnum IS NULL OR NOT a.attnotnull OR a.attisdropped OR i.indcollation[(k.ordinality-1)::pg_catalog.int4]<>a.attcollation OR (a.attcollation<>0 AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_collation co WHERE co.oid=a.attcollation AND co.collisdeterministic)))) AND am.amname='btree' AND am.amhandler='pg_catalog.bthandler'::pg_catalog.regproc AND NOT EXISTS(SELECT 1 FROM pg_catalog.unnest(i.indclass::pg_catalog.oid[]) AS k(oid) LEFT JOIN pg_catalog.pg_opclass opc ON opc.oid=k.oid LEFT JOIN pg_catalog.pg_namespace ns ON ns.oid=opc.opcnamespace WHERE ns.nspname IS DISTINCT FROM 'pg_catalog' OR opc.opcname NOT IN ('text_ops','bool_ops','int2_ops','int4_ops','int8_ops','numeric_ops','bytea_ops')))))) AS correspondence FROM pg_catalog.pg_class t JOIN pg_catalog.pg_namespace n ON n.oid=t.relnamespace WHERE {scope}",p(3),p(4))
 };
 let vals=[schema.clone().unwrap_or_default(),table.native.clone(),json!(allowed).to_string(),json!(keys).to_string()];
 let parameters=vals.iter().enumerate().filter(|(i,_)|schema.is_some()||*i!=0).map(|(i,v)|json!({"name":format!("__umf_index_effects_{}",i+1),"carrier":"string","source":{"literal":{"string":v}}})).collect::<Vec<_>>();
 // Positional protocol slots must remain contiguous when the neutral schema input is absent.
 let sql=if schema.is_none(){render(false).replace("$2::","$1::").replace("$3::","$2::").replace("$4::","$3::")}else{render(false)};
 json!({"sql":sql,"dbapiSql":render(true),"parameters":parameters,"dbapiParameterOrder":parameters.iter().map(|p|p["name"].clone()).collect::<Vec<_>>()})
}

#[cfg(test)]
mod tests {
 use super::*;
 fn descriptor()->Value{json!({"version":VERSION,"schema":"native_fence","view":"ts_catalog_lock","owner":"fence_owner","ownerProfile":"non-login-maintainer/0.1"})}
 #[test]
 fn original_fenced_search_and_missing_deployment_refusal(){
  let mut r:Value=serde_json::from_str(include_str!("../../../tests/native-search/source-exact-terms-keyword-request.json")).unwrap();
  let blocked:Value=serde_json::from_str(&crate::search::compile_json(&r.to_string())).unwrap();
  assert_eq!(blocked["status"],"blocked");
  r["selection"]["binding"]["nativeCatalogFence"]=descriptor();
  let p:Value=serde_json::from_str(&crate::search::compile_json(&r.to_string())).unwrap();
  assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["version"],"weft.search/0.2");
  let f=&p["obligations"]["nativeCatalogFence"];
  assert_eq!(f["lockSql"],"LOCK TABLE \"native_fence\".\"ts_catalog_lock\" IN SHARE MODE NOWAIT");
  assert_eq!(f["deployment"],descriptor());assert!(!f["origins"].as_array().unwrap().is_empty());
  let a=f["nativeAdmission"]["sql"].as_str().unwrap();assert!(a.contains("pg_catalog.pg_lock_status()"));assert!(a.contains("d.deptype='i'"));assert!(a.contains("NOT o.rolcanlogin"));
  if let Ok(dir)=std::env::var("WEFT_FENCE_PACKET_DIR") {
   std::fs::create_dir_all(&dir).unwrap();
   std::fs::write(format!("{dir}/ranked-request.json"),r.to_string()).unwrap();std::fs::write(format!("{dir}/ranked.json"),p.to_string()).unwrap();
   let probes:Value=serde_json::from_str(&crate::native_privilege_probes::compile_json(&json!({"version":"weft.native-privilege-probes/0.1","request":r,"program":p,"probeTable":"__weft_privilege_fence"}).to_string())).unwrap();
   assert_eq!(probes["status"],"compiled","{probes}");assert!(probes["locks"].as_array().unwrap().iter().any(|l|l["role"]=="catalog-fence-lock"));
   std::fs::write(format!("{dir}/ranked-probes.json"),probes.to_string()).unwrap();
  }

 }
 #[test]
 fn source_exact_mutation_index_inventory_is_mandatory_projection(){
  let r:Value=serde_json::from_str(include_str!("../../../tests/native-actions/source-exact-action-request.json")).unwrap();
  let p:Value=serde_json::from_str(&crate::raw_action::compile_json(&r.to_string())).unwrap();assert_eq!(p["status"],"compiled","{p}");
  assert_eq!(p["version"],"weft.native-action/0.2");
  let m=&p["obligations"]["mutationIndexEffects"];assert_eq!(m["lockPosition"],"before-first-snapshot");assert_eq!(m["lockSql"][0],"LOCK TABLE \"orders\" IN SHARE ROW EXCLUSIVE MODE NOWAIT");assert_eq!(m["version"],"umf.pg-index-effects/0.1");assert!(!m["tables"].as_array().unwrap().is_empty());
  let probes:Value=serde_json::from_str(&crate::native_privilege_probes::compile_json(&json!({"version":"weft.native-privilege-probes/0.1","request":r,"program":p,"probeTable":"__weft_privilege_scalar_action"}).to_string())).unwrap();assert_eq!(probes["status"],"compiled","{probes}");
  assert!(probes["statements"].as_array().unwrap().iter().any(|s|s["path"]=="/obligations/mutationIndexEffects/tables/0/nativeAdmission/sql"));
  assert!(probes["locks"].as_array().unwrap().iter().any(|s|s["path"]=="/obligations/mutationIndexEffects/lockSql/0"));
  if let Ok(dir)=std::env::var("WEFT_FENCE_PACKET_DIR"){std::fs::create_dir_all(&dir).unwrap();std::fs::write(format!("{dir}/scalar-action.json"),p.to_string()).unwrap();std::fs::write(format!("{dir}/scalar-action-probes.json"),probes.to_string()).unwrap();}
  for t in m["tables"].as_array().unwrap(){assert!(p["obligations"]["nativeProjection"]["columns"].as_array().unwrap().contains(&json!({"nativeAdmission":t["nativeAdmission"]})));let sql=t["nativeAdmission"]["sql"].as_str().unwrap();assert!(sql.contains("i.indpred IS NULL"));assert!(sql.contains("i.indexprs IS NULL"));assert!(sql.contains("opc.opcname NOT IN"));}
 }
 #[test]
 fn original_effect_only_and_composed_mutation_fence(){
  let mut r:Value=serde_json::from_str(include_str!("../../../tests/native-actions/source-exact-terms-bulk-request.json")).unwrap();
  for context in ["action","selection"]{r[context]["binding"]["nativeCatalogFence"]=descriptor();}
  let mut binding:weft_core::raw_read::Binding=serde_json::from_value(r["action"]["binding"].clone()).unwrap();
  let mut doc:Value=serde_json::from_str(&binding.model_json).unwrap();
  doc["modules"][0]["extensions"]["umf.access"]["search"][0]["keyword"]["source"].as_array_mut().unwrap().push(json!({"module":"sales","element":"id"}));
  binding.model_json=doc.to_string();binding.model_sha256=weft_core::json::sha256(binding.model_json.as_bytes());
  let indexes=weft_core::search::admit_keyword_effects(&binding,"postgresql18",&["sales".to_owned(),"order".to_owned()]).unwrap();
  let effect=crate::raw_read::action_keyword_index_metadata(&indexes[0],&binding.tables[0],&binding.schema).unwrap();
  assert_eq!(effect["nativeColumns"].as_array().unwrap().len(),2);assert!(effect["nativeAdmission"]["parameters"].as_array().unwrap().iter().any(|p|p["source"]["literal"]["string"].as_str().is_some_and(|s|s.contains(" || ' '::text"))));
  let action:Value=serde_json::from_str(&crate::raw_action::compile_json(&r["action"].to_string())).unwrap();
  assert_eq!(action["status"],"compiled","{action}");assert_eq!(action["version"],"weft.native-action/0.2");
  assert!(action["obligations"].get("termFilter").is_none());assert_eq!(action["obligations"]["textSearchEffects"]["indexes"].as_array().unwrap().len(),1);
  let p:Value=serde_json::from_str(&crate::raw_bulk::compile_json(&r.to_string())).unwrap();
  assert_eq!(p["status"],"compiled","{p}");assert_eq!(p["version"],"weft.native-bulk/0.2");
  assert_eq!(p["obligations"]["nativeCatalogFence"]["origins"].as_array().unwrap().len(),1);
  let probes:Value=serde_json::from_str(&crate::native_privilege_probes::compile_json(&json!({"version":"weft.native-privilege-probes/0.1","request":r,"program":p,"probeTable":"__weft_privilege_fenced_bulk"}).to_string())).unwrap();assert_eq!(probes["status"],"compiled","{probes}");
  assert!(probes["statements"].as_array().unwrap().iter().any(|s|s["path"]=="/obligations/mutationIndexEffects/tables/0/nativeAdmission/sql"));
  assert!(probes["locks"].as_array().unwrap().iter().any(|s|s["path"]=="/obligations/mutationIndexEffects/lockSql/0"));
  if let Ok(dir)=std::env::var("WEFT_FENCE_PACKET_DIR"){
   std::fs::write(format!("{dir}/bulk-probes.json"),probes.to_string()).unwrap();
   std::fs::write(format!("{dir}/effect-action.json"),action.to_string()).unwrap();std::fs::write(format!("{dir}/bulk-request.json"),r.to_string()).unwrap();std::fs::write(format!("{dir}/bulk.json"),p.to_string()).unwrap();
  }
 }

}
