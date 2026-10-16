//! Bounded Delta4 set-based whole-candidate plan, never a Truss manifest.
use serde_json::{json, Value};
use sqlparser::ast::*;
use weft_core::{raw_action::ActionField, raw_bulk};
type Result<T> = std::result::Result<T, &'static str>;
fn ident(s: &str) -> Result<String> {
    if s.is_empty() || s.len() > 255 || s.contains('\0') {
        return Err("native bulk identifier");
    };
    Ok(format!("`{}`", s.replace('`', "``")))
}
fn name(n: &Ident) -> String {
    if n.quote_style.is_some() {
        n.value.clone()
    } else {
        n.value.to_ascii_lowercase()
    }
}
fn compile(raw: &str) -> Result<Value> {
    let admitted = raw_bulk::admit_json(raw)?;
    let r = admitted.request();
    if r.action.target!="spark-delta4"{return Err("original Delta4 bulk target required")}
    let action = admitted.action();
    let effect = &action.effects()[0];
    let template: Value = serde_json::from_str(&crate::raw_action::compile_json(
        &serde_json::to_string(&r.action).unwrap(),
    ))
    .unwrap();
    if template["status"] != "compiled" {
        return Err("native action lowering blocked");
    }
    let read = crate::raw_read::lower_transaction(admitted.selection())?;
    let q = admitted.selection().query();
    if q.limit_clause.is_some() || q.with.is_some() {
        return Err("bulk requires original complete bounded selection");
    }
    let SetExpr::Select(s) = q.body.as_ref() else {
        return Err("bulk select required");
    };
    if s.from.len() != 1
        || !s.from[0].joins.is_empty()
        || s.having.is_some()
        || s.distinct.is_some()
    {
        return Err("bulk selection must be one scalar base table");
    }
    let TableFactor::Table {
        alias: Some(alias), ..
    } = &s.from[0].relation
    else {
        return Err("bulk bound table alias required");
    };
    let alias = name(&alias.name);
    let tables = read["referencedTables"]
        .as_array()
        .ok_or("bulk native provenance absent")?;
    if tables.len() != 1 || tables[0]["logical"] != json!(effect.entity.key.table) {
        return Err("bulk selection action owner differs");
    }
    let mut fields = Vec::<ActionField>::new();
    for projection in &s.projection {
        let SelectItem::ExprWithAlias {
            expr: Expr::CompoundIdentifier(path),
            alias: output,
        } = projection
        else {
            return Err("bulk projection needs explicit source Field aliases");
        };
        if path.len() != 2 || name(&path[0]) != alias || name(&path[1]) != name(output) {
            return Err("bulk source projection differs");
        }
        let field = effect
            .entity
            .key
            .fields
            .iter()
            .chain(effect.values.iter().map(|(f, _)| f))
            .find(|f| f.logical == name(&path[1]))
            .ok_or("bulk projection outside exact read/write frame")?;
        if fields.iter().any(|f| f.reference == field.reference) {
            return Err("duplicate bulk projection");
        };
        fields.push(field.clone());
    }
    for f in effect
        .entity
        .key
        .fields
        .iter()
        .chain(effect.values.iter().map(|(f, _)| f))
    {
        if !fields.iter().any(|p| p.reference == f.reference) {
            return Err("bulk projection must retain complete Key and changed Fields");
        }
    }
    let GroupByExpr::Expressions(group, modifiers) = &s.group_by else {
        return Err("bulk grouping unsupported");
    };
    if !group.is_empty() || !modifiers.is_empty() {
        return Err("bulk grouping unsupported");
    }
    let Some(order) = &q.order_by else {
        return Err("bulk canonical Key order required");
    };
    let OrderByKind::Expressions(order) = &order.kind else {
        return Err("bulk canonical Key order required");
    };
    if order.len() != effect.entity.key.fields.len() {
        return Err("bulk complete Key order required");
    }
    for (item, field) in order.iter().zip(&effect.entity.key.fields) {
        let Expr::CompoundIdentifier(path) = &item.expr else {
            return Err("bulk direct Key order required");
        };
        if path.len() != 2
            || name(&path[0]) != alias
            || name(&path[1]) != field.logical
            || matches!(item.options.sort, Some(OrderBySort::Desc))
            || item.options.nulls_first == Some(true)
        {
            return Err("bulk canonical ascending Key order required");
        }
    }
    let table = r
        .action
        .binding
        .tables
        .iter()
        .find(|t| t.logical == effect.entity.key.table)
        .unwrap();
    let column = |field: &ActionField| -> Result<String> {
        ident(
            &table
                .columns
                .iter()
                .find(|c| c.logical == field.logical)
                .ok_or("bulk column absent")?
                .native,
        )
    };
    let mut parameters=read["parameterOrder"].as_array().unwrap().iter().map(|name|{let p=r.selection.parameters.iter().find(|p|p.name==name.as_str().unwrap()).unwrap();json!({"source":{"bulkInput":p.name},"carrier":p.carrier})}).collect::<Vec<_>>();
    let selection=json!({"sql":read["sql"],"parameters":parameters});
    let mut assignments=Vec::new();
    let key_join=effect.entity.key.fields.iter().map(|f|Ok(format!("selected.{} <=> rows.{}",ident(&f.logical)?,column(f)?))).collect::<Result<Vec<_>>>()?.join(" AND ");
    for c in &table.columns{
        let native=ident(&c.native)?;
        if let Some((field,source))=effect.values.iter().find(|(f,_)|f.logical==c.logical){
            let source=if let Some(input)=source["input"].as_str(){json!({"bulkInput":r.action_inputs.get(input).ok_or("bulk input mapping absent")?})}else{source.clone()};
            parameters.push(json!({"source":source,"carrier":field.carrier}));let slot=format!(":p{}",parameters.len());
            assignments.push(format!("CASE WHEN EXISTS(SELECT 1 FROM __umf_selected selected WHERE {key_join}) THEN {slot} ELSE rows.{native} END AS {native}"));
        }else{assignments.push(format!("rows.{native}"))}
    }
    let sql=format!("WITH __umf_selected AS ({}) SELECT {} FROM `__umf_delta_stage` rows",read["sql"].as_str().unwrap(),assignments.join(", "));
    let mut verify_parameters=vec![];let mut violations=vec![];
    for c in &table.columns {
        let native=ident(&c.native)?;
        let expected=if let Some((field,source))=effect.values.iter().find(|(f,_)|f.logical==c.logical){
            let source=if let Some(input)=source["input"].as_str(){json!({"bulkInput":r.action_inputs.get(input).ok_or("bulk input mapping absent")?})}else{source.clone()};
            verify_parameters.push(json!({"source":source,"carrier":field.carrier}));
            let selected=effect.entity.key.fields.iter().map(|f|Ok(format!("s.{} <=> before.{}",ident(&f.logical)?,column(f)?))).collect::<Result<Vec<_>>>()?.join(" AND ");
            format!("CASE WHEN EXISTS (SELECT 1 FROM `__umf_delta_selected` s WHERE {selected}) THEN :p{} ELSE before.{native} END",verify_parameters.len())
        }else{format!("before.{native}")};
        violations.push(format!("NOT (rows.{native} <=> {expected})"));
    }
    let key_match=effect.entity.key.fields.iter().map(|f|Ok(format!("before.{} <=> rows.{}",column(f)?,column(f)?))).collect::<Result<Vec<_>>>()?.join(" AND ");
    let first=column(&effect.entity.key.fields[0])?;
    let verify_sql=format!("SELECT COUNT(*) AS violations FROM `__umf_delta_before` before FULL OUTER JOIN `__umf_delta_stage` rows ON {key_match} WHERE before.{first} IS NULL OR rows.{first} IS NULL OR {}",violations.join(" OR "));
    let mut out=json!({"version":"weft.native-bulk/0.1","profile":"umf.actions.native-delta-bulk/0.1","status":"compiled","target":"spark-delta4","source":r.action.source,"binding":r.action.binding,"security":r.action.security,"action":action.action(),"roleMappings":r.action.role_mappings,"maxItems":r.max_items,"parameters":r.parameters,"actionInputs":r.action_inputs,"selectionFields":fields,"entity":effect.entity,"values":effect.values,"preview":selection,"selection":selection,"update":{"sql":sql,"parameters":parameters},"verification":{"sql":verify_sql,"parameters":verify_parameters},"keyCheck":template["keyCheck"],"obligations":template["obligations"],"actionTemplate":template,"readTemplate":read,"originalRequest":r});
    out["structuralDigest"]=json!({"owner":"weft-databricks-native-bulk","version":"0.1","sha256":weft_core::json::sha256(weft_core::mutation::canonical(&out).as_bytes())});
    Ok(out)
}
pub fn compile_json(raw:&str)->String{match compile(raw){Ok(v)=>v,Err(reason)=>json!({"version":"weft.native-bulk/0.1","status":"blocked","diagnostics":[{"code":"WFT-DELTA-BULK","message":reason}]})}.to_string()}
