// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded native set-based UPDATE program, separate from manifest mutation.
use serde_json::{Value, json};
use sqlparser::ast::*;
use weft_core::{raw_action::ActionField, raw_bulk, raw_read::Carrier};
type Result<T> = std::result::Result<T, &'static str>;
fn ident(s: &str) -> Result<String> {
    if s.is_empty() || s.len() > 63 || s.contains('\0') {
        return Err("native bulk identifier");
    };
    Ok(format!("\"{}\"", s.replace('"', "\"\"")))
}
fn name(n: &Ident) -> String {
    if n.quote_style.is_some() {
        n.value.clone()
    } else {
        n.value.to_ascii_lowercase()
    }
}
fn native(c: &Carrier) -> &'static str {
    match c {
        Carrier::String => "text",
        Carrier::Integer | Carrier::Decimal => "numeric",
        Carrier::Boolean => "bool",
        Carrier::Uuid => "uuid",
        Carrier::Bits128 => "bytea",
    }
}
fn compile(raw: &str) -> Result<Value> {
    let admitted = raw_bulk::admit_json(raw)?;
    let r = admitted.request();
    let action = admitted.action();
    let effect = &action.effects()[0];
    let template: Value = serde_json::from_str(&crate::raw_action::compile_json(
        &serde_json::to_string(&r.action).unwrap(),
    ))
    .unwrap();
    if template["status"] != "compiled" {
        return Err("native action lowering blocked");
    }
    let read = crate::raw_read::lower(admitted.selection())?;
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
    let native_table = match &r.action.binding.schema {
        Some(schema) => format!("{}.{}", ident(schema)?, ident(&table.native)?),
        None => ident(&table.native)?,
    };
    let mut parameters = vec![];
    let mut assignments = vec![];
    let mut changed = vec![];
    for (field, source) in &effect.values {
        let source = if let Some(id) = source["input"].as_str() {
            json!({"bulkInput":r.action_inputs.get(id).ok_or("bulk input mapping absent")?})
        } else {
            source.clone()
        };
        parameters.push(json!({"source":source,"carrier":field.carrier,"array":false}));
        let slot = format!(
            "${}::pg_catalog.{}",
            parameters.len(),
            native(&field.carrier)
        );
        let col = column(field)?;
        assignments.push(format!("{col} = {slot}"));
        changed.push(format!("{col} IS DISTINCT FROM {slot}"));
    }
    let mut arrays = vec![];
    for (index, field) in effect.entity.key.fields.iter().enumerate() {
        parameters.push(json!({"source":{"bulkKeys":index},"carrier":field.carrier,"array":true}));
        arrays.push(format!(
            "${}::pg_catalog.{}[]",
            parameters.len(),
            native(&field.carrier)
        ))
    }
    let keys = effect
        .entity
        .key
        .fields
        .iter()
        .map(column)
        .collect::<Result<Vec<_>>>()?;
    let aliases = (0..keys.len())
        .map(|i| ident(&format!("k{i}")))
        .collect::<Result<Vec<_>>>()?;
    let predicate = keys
        .iter()
        .zip(&aliases)
        .map(|(col, alias)| format!("{col} IS NOT DISTINCT FROM __umf_keys.{alias}"))
        .collect::<Vec<_>>()
        .join(" AND ");
    let sql = format!(
        "UPDATE {native_table} SET {} WHERE EXISTS (SELECT 1 FROM ROWS FROM ({}) AS __umf_keys({}) WHERE {predicate}) AND ({}) RETURNING {}",
        assignments.join(", "),
        arrays
            .iter()
            .map(|a| format!("pg_catalog.unnest({a})"))
            .collect::<Vec<_>>()
            .join(", "),
        aliases.join(", "),
        changed.join(" OR "),
        keys.join(", ")
    );
    let slots = read["parameterOrder"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| {
            let p = r
                .selection
                .parameters
                .iter()
                .find(|p| p.name == *name.as_str().unwrap())
                .unwrap();
            let mut slot = json!({"source":{"bulkInput":p.name},"carrier":p.carrier});
            if p.container.is_some() {
                slot["nullable"] = json!(p.nullable);
                slot["container"] = serde_json::to_value(&p.container).unwrap();
                slot["elementNullable"] = json!(p.element_nullable);
                slot["maxItems"] = json!(p.max_items);
            }
            slot
        })
        .collect::<Vec<_>>();
    let selection = json!({"sql":read["sql"],"parameters":slots});
    let mut locked = selection.clone();
    locked["sql"] = json!(format!("{} FOR UPDATE", read["sql"].as_str().unwrap()));
    let mut program = json!({"version":"weft.native-bulk/0.1","profile":"umf.actions.native-bulk/0.1","status":"compiled","target":r.action.target,"source":r.action.source,"binding":r.action.binding,"security":r.action.security,"action":action.action(),"maxItems":r.max_items,"parameters":r.parameters,"actionInputs":r.action_inputs,"selectionFields":fields,"entity":effect.entity,"values":effect.values,"preview":selection,"selection":locked,"update":{"sql":sql,"parameters":parameters},"admission":template["admission"],"lockSql":template["lockSql"],"obligations":template["obligations"],"actionTemplate":template,"readTemplate":read});
    let read_exact=read["obligations"]["nativeProjection"]["verificationProfile"]=="source-exact/0.1";
    let action_exact=template["obligations"]["nativeProjection"]["verificationProfile"]=="source-exact/0.1";
    if read_exact!=action_exact{return Err("bulk action/selection source verification profiles differ")}
    if read_exact {
        // The bulk host owns the same original search-path context as action SQL;
        // no read-host appSchema parameter is invented or bound by the SDK.
        let selected=admitted.selection();let table=selected.request().binding.tables.iter().find(|t|t.logical==effect.entity.key.table).ok_or("bulk original selection table absent")?;
        let columns=crate::raw_read::native_projection::columns(selected,table)?;
        let mut selected_admissions=columns.values().map(|c|json!({"nativeAdmission":crate::raw_read::native_projection::action_admission(c,table,&r.selection.binding.schema)})).collect::<Vec<_>>();
        selected_admissions.push(json!({"nativeAdmission":crate::raw_read::native_projection::action_table_admission(table,&r.selection.binding.schema,&columns.values().map(|c|c.native.clone()).collect::<Vec<_>>())}));
        let mut packet=template["obligations"]["nativeProjection"].clone();let values=packet["columns"].as_array_mut().unwrap();for value in selected_admissions {if !values.contains(&value){values.push(value);}}program["obligations"]["nativeProjection"]=packet;
    }
    if let Some(term)=read["obligations"].get("termFilter") {
        let table=r.selection.binding.tables.iter().find(|t|t.logical==effect.entity.key.table).ok_or("bulk term source table absent")?;
        let mut indexes=vec![];
        for original in term["indexes"].as_array().ok_or("bulk original term inventory absent")? {
            let column=table.columns.iter().find(|c|original["nativeColumn"]==c.native).ok_or("bulk original term column absent")?;
            indexes.push(crate::raw_read::action_term_index_metadata(admitted.selection(),table,column)?);
        }
        let mut composed=term.clone();composed["indexes"]=json!(indexes);
        program["obligations"]["termFilter"]=composed;
    }
    crate::native_catalog_fence::attach(&r.action.binding,&mut program["obligations"])?;
    crate::native_catalog_fence::seal_version(&mut program)?;
    let digest = weft_core::json::sha256(weft_core::mutation::canonical(&program).as_bytes());
    program["structuralDigest"] =
        json!({"owner":"weft-postgresql-native-bulk","version":"0.1","sha256":digest});
    Ok(program)
}
pub fn compile_json(raw: &str) -> String {
    match compile(raw){Ok(v)=>v.to_string(),Err(reason)=>json!({"version":"weft.native-bulk/0.1","status":"blocked","diagnostics":[{"code":reason}]}).to_string()}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> Value {
        let action: Value = serde_json::from_str(include_str!(
            "../../../tests/native-actions/action-request.json"
        ))
        .unwrap();
        json!({"version":"weft.native-bulk/0.1","selection":{"version":"weft.raw-read/0.1","target":"postgresql18","sql":"SELECT o.id AS id, o.status AS status FROM sales.order AS o ORDER BY o.id ASC","binding":action["binding"],"parameters":[],"rowCap":4},"action":action,"maxItems":3,"actionInputs":{},"parameters":[]})
    }
    fn result(request: &Value) -> Value {
        serde_json::from_str(&compile_json(&request.to_string())).unwrap()
    }
    #[test]
    fn term_selection_admissions_survive_mutation_composition(){
        let r:Value=serde_json::from_str(include_str!("../../../tests/native-actions/source-exact-terms-bulk-request.json")).unwrap();
        for neutral in [false,true] {
            let mut r=r.clone();for c in ["action","selection"]{r[c]["binding"]["nativeCatalogFence"]=json!({"version":"umf.pg-ts-fence/0.1","schema":"native_fence","view":"ts_catalog_lock","owner":"fence_owner","ownerProfile":"non-login-maintainer/0.1"});}if neutral {r["selection"]["binding"].as_object_mut().unwrap().remove("schema");r["action"]["binding"].as_object_mut().unwrap().remove("schema");}
            let p=result(&r);assert_eq!(p["status"],"compiled","{p}");
            if let Ok(dir)=std::env::var("WEFT_COMPOSED_TERM_OUTPUT") {std::fs::write(format!("{dir}/bulk-{}.json",if neutral{"neutral"}else{"qualified"}),p.to_string()).unwrap();}
            if !neutral {let old:Value=serde_json::from_str(include_str!("../../../tests/native-actions/source-exact-terms-read-r20.json")).unwrap();assert_eq!(p["readTemplate"]["sql"],old["sql"]);assert_eq!(p["readTemplate"]["outputs"],old["outputs"]);assert_eq!(p["readTemplate"]["obligations"]["termFilter"],old["obligations"]["termFilter"],"existing raw analyzer SQL remains unchanged; fence/version are new");}
            let top=&p["obligations"]["termFilter"];let original=&p["readTemplate"]["obligations"]["termFilter"];
            assert_eq!(top["version"],original["version"]);assert_eq!(top["nativeCatalogBeforeAfter"],true);
            assert_eq!(top["indexes"].as_array().unwrap().len(),1);
            let mut a=top["indexes"][0].clone();let mut b=original["indexes"][0].clone();a.as_object_mut().unwrap().remove("nativeAdmission");b.as_object_mut().unwrap().remove("nativeAdmission");assert_eq!(a,b);
            let admission=&top["indexes"][0]["nativeAdmission"];
            assert!(admission["sql"].as_str().unwrap().contains("pg_ts_config_map"));
            assert!(admission["parameters"].as_array().unwrap().iter().all(|p|p["source"].get("literal").is_some()));
            if neutral {assert!(admission["sql"].as_str().unwrap().contains("to_regclass(pg_catalog.quote_ident("));}
        }
    }
    #[test]
    fn source_exact_bulk_unions_both_original_admission_packets(){
        let mut r=request();let action:Value=serde_json::from_str(include_str!("../../../tests/native-actions/source-exact-action-request.json")).unwrap();
        r["action"]=action.clone();r["selection"]["binding"]=action["binding"].clone();r["selection"]["verificationProfile"]=json!("source-exact/0.1");
        let p:Value=serde_json::from_str(&compile_json(&r.to_string())).unwrap();assert_eq!(p["status"],"compiled","{p}");
        assert_eq!(p["obligations"]["nativeProjection"]["verificationProfile"],"source-exact/0.1");
        assert_eq!(p["obligations"]["nativeProjection"]["columns"].as_array().unwrap().len(),4);
        assert_eq!(p["obligations"]["nativeProjection"],p["actionTemplate"]["obligations"]["nativeProjection"]);
        for c in p["obligations"]["nativeProjection"]["columns"].as_array().unwrap(){assert!(c["nativeAdmission"]["sql"].as_str().unwrap().contains("pg_catalog.to_regclass(pg_catalog.quote_ident("));assert!(c["nativeAdmission"]["parameters"].as_array().unwrap().iter().all(|p|p["source"].get("literal").is_some()));}
        r["action"].as_object_mut().unwrap().remove("verificationProfile");assert_eq!(serde_json::from_str::<Value>(&compile_json(&r.to_string())).unwrap()["status"],"blocked");
    }
    #[test]
    fn bounded_set_program_retains_original_sources() {
        let request = request();
        let out = result(&request);
        assert_eq!(out["status"], "compiled", "{out}");
        assert_eq!(out, result(&request));
        assert_eq!(out["source"], request["action"]["source"]);
        assert_eq!(out["binding"], request["action"]["binding"]);
        assert_eq!(out["security"], request["action"]["security"]);
        assert_eq!(out["maxItems"], 3);
        assert!(
            out["selection"]["sql"]
                .as_str()
                .unwrap()
                .ends_with("LIMIT 4 FOR UPDATE")
        );
        assert!(
            !out["preview"]["sql"]
                .as_str()
                .unwrap()
                .contains("FOR UPDATE")
        );
        assert!(
            out["update"]["sql"]
                .as_str()
                .unwrap()
                .starts_with("UPDATE ")
        );
        assert!(out["update"]["sql"].as_str().unwrap().contains("ROWS FROM"));
        assert!(out.get("manifest").is_none());
        assert_eq!(
            out["structuralDigest"]["owner"],
            "weft-postgresql-native-bulk"
        );
    }
    #[test]
    fn sentinel_order_and_source_boundaries_refuse() {
        for change in 0..5 {
            let mut r = request();
            match change {
                0 => r["maxItems"] = json!(0),
                1 => r["selection"]["rowCap"] = json!(3),
                2 => {
                    r["selection"]["sql"] = json!(
                        "SELECT o.id AS id, o.status AS status FROM sales.order AS o ORDER BY o.status ASC"
                    )
                }
                3 => {
                    r["selection"]["sql"] = json!(
                        "SELECT o.id AS id, o.status AS status FROM sales.order AS o ORDER BY o.id ASC LIMIT 1"
                    )
                }
                _ => r["action"]["source"]["documentSha256"] = json!("wrong"),
            };
            let out = result(&r);
            assert_eq!(out["status"], "blocked", "{out}");
            assert!(out.get("update").is_none());
        }
    }
}

#[cfg(test)]
mod list_parameter_tests {
    use super::*;
    fn request() -> Value {
        let action: Value = serde_json::from_str(include_str!(
            "../../../tests/native-actions/action-request.json"
        ))
        .unwrap();
        json!({"version":"weft.native-bulk/0.1","selection":{"version":"weft.raw-read/0.1","target":"postgresql18","sql":"SELECT o.id AS id, o.status AS status FROM sales.order AS o ORDER BY o.id ASC","binding":action["binding"],"parameters":[],"rowCap":4},"action":action,"maxItems":3,"actionInputs":{},"parameters":[]})
    }
    fn compile(r: &Value) -> Value {
        serde_json::from_str(&compile_json(&r.to_string())).unwrap()
    }
    fn list_request() -> Value {
        let mut r = request();
        let p = json!({"name":"statuses","carrier":"string","nullable":false,"container":"list","elementNullable":true,"maxItems":8});
        r["parameters"] = json!([p]);
        r["selection"]["parameters"] = r["parameters"].clone();
        r["selection"]["sql"] = json!(
            "SELECT o.id AS id,o.status AS status FROM sales.order AS o WHERE o.status IN (:statuses) ORDER BY o.id ASC"
        );
        r
    }
    #[test]
    fn selection_list_metadata_and_dbapi_transport_are_original() {
        let scalar = compile(&request());
        assert_eq!(scalar["status"], "compiled", "{scalar}");
        assert_eq!(
            scalar["structuralDigest"]["sha256"],
            "96464cf62dc5c5995eac318bb3cc6b7ca84118a6367a13268fb5faa083c362a5"
        );
        let r = list_request();
        let p = compile(&r);
        assert_eq!(p["status"], "compiled", "{p}");
        let slot = &p["selection"]["parameters"][0];
        assert_eq!(slot["source"], json!({"bulkInput":"statuses"}));
        assert_eq!(slot["container"], "list");
        assert_eq!(slot["nullable"], false);
        assert_eq!(slot["elementNullable"], true);
        assert_eq!(slot["maxItems"], 8);
        assert!(
            p["selection"]["sql"]
                .as_str()
                .unwrap()
                .contains("::pg_catalog.text[]")
        );
        assert_eq!(p["parameters"], r["parameters"]);
        for (field, value) in [
            ("container", Value::Null),
            ("elementNullable", json!(false)),
            ("maxItems", json!(7)),
            ("nullable", json!(true)),
        ] {
            let mut invalid = r.clone();
            invalid["parameters"][0][field] = value;
            assert_eq!(compile(&invalid)["status"], "blocked", "{field}");
        }
    }
    #[test]
    fn action_value_parameters_remain_scalar_even_when_selection_lists_are_supported() {
        let mut r = request();
        let mut doc: Value =
            serde_json::from_str(r["action"]["binding"]["modelJson"].as_str().unwrap()).unwrap();
        let action = &mut doc["modules"][0]["extensions"]["umf.actions"]["actions"][0];
        action["parameters"].as_array_mut().unwrap().push(json!({"id":"next","kind":"value","field":{"module":"sales","element":"status"},"required":true}));
        action["binding"]["effects"][0]["values"][0]["value"] = json!({"parameter":"next"});
        let raw = doc.to_string();
        let hash = weft_core::json::sha256(raw.as_bytes());
        r["action"]["binding"]["modelJson"] = json!(raw);
        r["action"]["binding"]["modelSha256"] = json!(hash);
        r["action"]["source"]["documentSha256"] = json!(hash);
        let mut ontology: Value = serde_json::from_str(
            r["action"]["security"]["ontology"]["utf8"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        ontology["documents"][0]["revision"] = json!(hash);
        let ontology = ontology.to_string();
        r["action"]["security"]["ontology"]["sha256"] =
            json!(weft_core::json::sha256(ontology.as_bytes()));
        r["action"]["security"]["ontology"]["utf8"] = json!(ontology);
        r["selection"]["binding"] = r["action"]["binding"].clone();
        r["actionInputs"] = json!({"next":"new_status"});
        r["parameters"] = json!([{"name":"new_status","carrier":"string","nullable":false}]);
        let p = compile(&r);
        assert_eq!(p["status"], "compiled", "{p}");
        for (field, value) in [
            ("container", json!("list")),
            ("elementNullable", json!(true)),
            ("maxItems", json!(8)),
        ] {
            let mut invalid = r.clone();
            invalid["parameters"][0][field] = value;
            assert_eq!(compile(&invalid)["status"], "blocked", "{field}");
        }
    }
}
