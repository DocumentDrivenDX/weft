//! Finite descriptor graph and recursive runtime-value token encoder.
use super::*;
use weft_core::application_model::{Descriptor,Shape};

pub(crate) fn pack(graph:&[Descriptor])->Result<Value> {
    let index=|id:&Identity|graph.iter().position(|d|&d.identity==id).ok_or_else(||fail("WFT-BINDING","Recursive descriptor dependency is missing"));
    let mut packed=Vec::new();
    for d in graph {
        let mut entry=json!({"optional":d.availability.as_deref()==Some("absent-allowed")});
        match &d.shape {
            Shape::Scalar{..}=>entry["kind"]=json!("scalar"),
            Shape::Sequence{item}|Shape::Map{item}=>{entry["kind"]=json!(if matches!(d.shape,Shape::Sequence{..}) {"sequence"} else {"map"});entry["item"]=json!(index(item)?);},
            Shape::Structured{..}|Shape::Record{..}=>{
                let members=match &d.shape {
                    Shape::Record{members}=>members,
                    Shape::Structured{record}=>match &graph[index(record)?].shape {Shape::Record{members}=>members,_=>return Err(fail("WFT-BINDING","Structured value needs a record descriptor"))},
                    _=>unreachable!(),
                };
                let mut names=BTreeSet::new();
                if members.iter().any(|m|m.name.contains('\0')||!names.insert(&m.name)) {return Err(fail("WFT-CAPABILITY","JSON record encoding needs unique non-NUL authored member names"));}
                entry["kind"]=json!("record");
                entry["members"]=json!(members.iter().map(|m|Ok(json!({"name":m.name,"id":index(&m.identity)?}))).collect::<Result<Vec<_>>>()?);
            }
        }
        packed.push(entry);
    }
    Ok(json!(packed))
}
pub(super) struct Encoded {pub value:String,pub join:String,pub check:Value}
fn json_string(value:&str)->String {
    let array=format!("to_json(array({value}))");
    format!("substring({array}, 2, length({array})-2)")
}
pub(super) fn encode(lower:&mut Lower<'_>,graph:&[Descriptor],root:&Identity,table:&str,owner:&str,revision:&str,path:&str)->Result<Encoded> {
    let packed=pack(graph)?;
    let slot=lower.slot(string_type(),packed.to_string(),json!({"kind":"compoundCodecGraph","root":root,"encoding":"ashlar-weft-json-value/0.1-candidate"}))?;
    let root_index=graph.iter().position(|d|&d.identity==root).unwrap();
    let mut number=lower.ctes.len();
    let prefix=loop {let name=format!("__weft_codec_{number}");number+=1;if !lower.scans.contains_key(&name){break name;}};
    let g=binding::quote(&format!("{prefix}_graph"));let walk=binding::quote(&format!("{prefix}_walk"));let encoded=binding::quote(&format!("{prefix}_value"));
    let graph_cte=format!("{g} AS (SELECT pos AS id, col AS d FROM posexplode(from_json({slot}, 'ARRAY<STRUCT<kind:STRING,optional:BOOLEAN,item:INT,members:ARRAY<STRUCT<name:STRING,id:INT>>>>')))");
    let quoted_name=json_string("m.name");
    let map_key=json_string("m.key");
    let record_child=format!("transform(g.d.members, (m, i) -> named_struct('id',m.id,'v',element_at(try_cast(w.v AS MAP<STRING COLLATE UTF8_BINARY,VARIANT>),m.name COLLATE UTF8_BINARY),'slot',i,'prefix',concat(CASE WHEN i>0 THEN ',' ELSE '' END,{quoted_name},':'),'allow_absent',TRUE))");
    let sequence_child="transform(try_cast(w.v AS ARRAY<VARIANT>), (v, i) -> named_struct('id',g.d.item,'v',v,'slot',i,'prefix',CASE WHEN i>0 THEN ',' ELSE '' END,'allow_absent',FALSE))";
    let map_child=format!("transform(array_sort(map_entries(try_cast(w.v AS MAP<STRING COLLATE UTF8_BINARY,VARIANT>)), (a,b) -> CASE WHEN (a.key COLLATE UTF8_BINARY) < (b.key COLLATE UTF8_BINARY) THEN -1 WHEN (a.key COLLATE UTF8_BINARY) > (b.key COLLATE UTF8_BINARY) THEN 1 ELSE 0 END), (m,i) -> named_struct('id',g.d.item,'v',m.value,'slot',i,'prefix',concat(CASE WHEN i>0 THEN ',' ELSE '' END,{map_key},':'),'allow_absent',FALSE))");
    let children=format!("CASE WHEN g.d.kind='record' AND schema_of_variant(w.v) RLIKE '^OBJECT' THEN {record_child} WHEN g.d.kind='sequence' AND schema_of_variant(w.v) RLIKE '^ARRAY' THEN {sequence_child} WHEN g.d.kind='map' AND schema_of_variant(w.v) RLIKE '^OBJECT' THEN {map_child} ELSE CAST(array() AS ARRAY<STRUCT<id:INT,v:VARIANT,slot:INT,prefix:STRING,allow_absent:BOOLEAN>>) END");
    let walk_cte=format!("{walk}(owner_id,id,v,path,prefix,depth,tagged,allow_absent,base_valid) MAX RECURSION LEVEL 130 AS (SELECT r.id,{root_index},variant_get(parse_json(r.props_json),{path}),CAST(array() AS ARRAY<INT>),'',0,TRUE,TRUE,(schema_of_variant(parse_json(r.props_json)) RLIKE '^OBJECT' AND ({revision})) FROM {table} r WHERE {owner} UNION ALL SELECT w.owner_id,children.col.id,children.col.v,concat(w.path,array(children.col.slot)),children.col.prefix,w.depth+1,cg.d.optional,children.col.allow_absent,w.base_valid FROM {walk} w JOIN {g} g ON w.id=g.id, LATERAL explode({children}) children JOIN {g} cg ON cg.id=children.col.id WHERE w.depth<128 AND w.v IS NOT NULL AND NOT is_variant_null(w.v))");
    let scalar="CAST(w.v AS STRING)";let schema="schema_of_variant(w.v)";
    let mut valid_arms=Vec::new();let mut render_arms=Vec::new();
    for (id,d) in graph.iter().enumerate() {
        if let Shape::Scalar{logical_type:ty}=&d.shape {
            let valid=match ty.family {
                Family::String=>format!("{schema}='STRING' AND instr({scalar},char(0))=0"),
                Family::Boolean=>format!("{schema}='BOOLEAN'"),
                Family::Integer=>format!("({schema}='BIGINT' OR {schema} RLIKE '^DECIMAL\\\\([0-9]+,0\\\\)$') AND {}",numeric_guard(scalar,ty)),
                Family::Decimal=>format!("({schema}='BIGINT' OR ({schema} RLIKE '^DECIMAL\\\\([0-9]+,[0-9]+\\\\)$' AND coalesce(try_cast(regexp_extract({schema}, ',([0-9]+)\\\\)$',1) AS INT),0)<={})) AND {}",ty.facets["scale"],numeric_guard(scalar,ty)),
            };
            valid_arms.push(format!("WHEN {id} THEN ({valid})"));
            let render=if ty.family==Family::Boolean {scalar.into()} else {json_string(&format!("CAST({} AS STRING)",typed(scalar,ty)))};
            render_arms.push(format!("WHEN {id} THEN {render}"));
        }
    }
    let member_names="transform(g.d.members, m -> m.name COLLATE UTF8_BINARY)";
    let object_keys="map_keys(try_cast(w.v AS MAP<STRING COLLATE UTF8_BINARY,VARIANT>))";
    let valid=format!("w.base_valid AND w.depth<128 AND CASE WHEN w.v IS NULL THEN g.d.optional AND w.allow_absent ELSE CASE g.d.kind WHEN 'scalar' THEN CASE g.id {} ELSE FALSE END WHEN 'sequence' THEN {schema} RLIKE '^ARRAY' WHEN 'map' THEN {schema} RLIKE '^OBJECT' AND forall({object_keys}, k -> instr(k,char(0))=0) WHEN 'record' THEN {schema} RLIKE '^OBJECT' AND forall({object_keys}, k -> array_contains({member_names},k)) ELSE FALSE END END",valid_arms.join(" "));
    let scalar_render=format!("CASE g.id {} ELSE 'null' END",render_arms.join(" "));
    let start=format!("concat(w.prefix,CASE WHEN w.v IS NULL THEN '{{\"state\":\"absent\"}}' ELSE concat(CASE WHEN w.tagged THEN '{{\"state\":\"value\",\"value\":' ELSE '' END, CASE g.d.kind WHEN 'scalar' THEN CASE WHEN ({valid}) THEN {scalar_render} ELSE 'null' END WHEN 'sequence' THEN '[' ELSE '{{' END, CASE WHEN g.d.kind='scalar' AND w.tagged THEN '}}' ELSE '' END) END)");
    let close="concat(CASE WHEN g.d.kind='sequence' THEN ']' ELSE '}' END, CASE WHEN w.tagged THEN '}' ELSE '' END)";
    let token_array=format!("CASE WHEN w.v IS NOT NULL AND g.d.kind<>'scalar' THEN array(named_struct('path',w.path,'token',{start}),named_struct('path',concat(w.path,array(2147483647)),'token',{close})) ELSE array(named_struct('path',w.path,'token',{start})) END");
    let count=count_sql();
    let nodes=format!("SELECT w.*, ({valid}) AS valid, {token_array} AS tokens FROM {walk} w JOIN {g} g ON w.id=g.id");
    let value_cte=format!("{encoded} AS (SELECT owner_id, bool_and(coalesce(valid,FALSE)) AND ({count})<=100000 AS valid, concat_ws('',transform(array_sort(flatten(collect_list(tokens))), x -> x.token)) AS value FROM ({nodes}) nodes GROUP BY owner_id)");
    let codec_ctes=vec![graph_cte,walk_cte,value_cte];
    let check=json!({"field":root,"sql":format!("WITH RECURSIVE {} SELECT CAST({count} AS STRING) AS violations FROM {table} r LEFT JOIN {encoded} ON {encoded}.owner_id=r.id WHERE {owner} AND CASE WHEN {encoded}.valid THEN FALSE ELSE TRUE END",codec_ctes.join(", ")),"failureCode":"WFT-OBLIGATION","limits":{"depthExclusive":128,"nodes":100000},"encoding":"ashlar-weft-json-value/0.1-candidate"});
    lower.ctes.extend(codec_ctes);
    // Duplicate physical IDs cannot be merged by the per-owner codec aggregation.
    lower.compound_checks.push(json!({"field":root,"sql":format!("SELECT CAST({count} AS STRING) AS violations FROM (SELECT r.id FROM {table} r WHERE {owner} GROUP BY r.id HAVING {count}>1) duplicates"),"failureCode":"WFT-BINDING"}));
    Ok(Encoded{value:format!("CASE WHEN {encoded}.valid THEN {encoded}.value ELSE raise_error('WFT-OBLIGATION') END"),join:format!("LEFT JOIN {encoded} ON {encoded}.owner_id=r.id"),check})
}
