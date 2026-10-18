//! PostgreSQL18 statement programs for admitted scalar graph-write actions.
//! No connection, authentication issuer, retry or runtime execution lives here.
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use weft_core::raw_action::{
    self, ActionCondition, ActionEntity, ActionField, AdmittedAction, RuleExpression, RuleNode,
};
use weft_core::raw_read::{Binding, Carrier};
type Result<T> = std::result::Result<T, &'static str>;
fn ident(s: &str) -> Result<String> {
    if s.is_empty() || s.len() > 63 || s.contains('\0') {
        return Err("native action identifier outside PostgreSQL profile");
    };
    Ok(format!("\"{}\"", s.replace('"', "\"\"")))
}
fn native_type(c: &Carrier) -> &'static str {
    match c {
        Carrier::String => "text",
        Carrier::Boolean => "bool",
        Carrier::Integer | Carrier::Decimal => "numeric",
        Carrier::Uuid => "uuid",
        Carrier::Bits128 => "bytea",
    }
}
fn field_id(f: &ActionField) -> String {
    weft_core::mutation::canonical(&f.reference)
}
struct Statement {
    slots: Vec<Value>,
}
impl Statement {
    fn slot(&mut self, source: Value, c: &Carrier) -> String {
        self.slots.push(json!({"source":source,"carrier":c}));
        let sql = format!("${}::pg_catalog.{}", self.slots.len(), native_type(c));
        if *c == Carrier::String {
            format!("({sql} COLLATE pg_catalog.\"C\")")
        } else {
            sql
        }
    }
    fn finish(self, sql: String) -> Value {
        json!({"sql":sql,"parameters":self.slots})
    }
}
struct Lower<'a> {
    action: &'a AdmittedAction,
    binding: &'a Binding,
}
impl Lower<'_> {
    fn table(&self, logical: &Vec<String>) -> Result<String> {
        let t = self
            .binding
            .tables
            .iter()
            .find(|t| &t.logical == logical)
            .ok_or("native action table absent")?;
        Ok(match &self.binding.schema {
            Some(s) => format!("{}.{}", ident(s)?, ident(&t.native)?),
            None => ident(&t.native)?,
        })
    }
    fn column(&self, f: &ActionField) -> Result<String> {
        let t = self
            .binding
            .tables
            .iter()
            .find(|t| t.logical == f.table)
            .ok_or("native action table absent")?;
        let c = t
            .columns
            .iter()
            .find(|c| c.logical == f.logical)
            .ok_or("native action column absent")?;
        ident(&c.native)
    }
    fn predicate(&self, entity: &ActionEntity, statement: &mut Statement) -> Result<String> {
        entity
            .key
            .fields
            .iter()
            .zip(&entity.components)
            .map(|(f, value)| {
                Ok(format!(
                    "{} IS NOT DISTINCT FROM {}",
                    self.column(f)?,
                    statement.slot(value.clone(), &f.carrier)
                ))
            })
            .collect::<Result<Vec<_>>>()
            .map(|parts| parts.join(" AND "))
    }
    fn image(
        &self,
        name: &str,
        entity: &ActionEntity,
        fields: &[ActionField],
        lock: bool,
    ) -> Result<Value> {
        let mut s = Statement { slots: vec![] };
        let predicate = self.predicate(entity, &mut s)?;
        let cols = fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                Ok(format!(
                    "{} AS {}",
                    self.column(f)?,
                    ident(&format!("f{i}"))?
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let sql = format!(
            "SELECT {} FROM {} WHERE {}{}",
            cols.join(", "),
            self.table(&entity.key.table)?,
            predicate,
            if lock { " FOR UPDATE" } else { "" }
        );
        Ok(json!({"name":name,"entity":entity,"columns":fields,"statement":s.finish(sql)}))
    }
    fn rule(&self, e: &RuleExpression, s: &mut Statement) -> Result<(String, String)> {
        let (value, error) = match &e.node {
            RuleNode::Literal { value } => {
                if value.is_null() {
                    ("NULL::pg_catalog.text".into(), "FALSE".into())
                } else {
                    let c = match e.family.as_str() {
                        "boolean" => Carrier::Boolean,
                        "integer" => Carrier::Integer,
                        "string" => Carrier::String,
                        _ => return Err("rule literal family"),
                    };
                    (s.slot(json!({"literal":value}), &c), "FALSE".into())
                }
            }
            RuleNode::Input { id } => {
                let p = self.action.action()["parameters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|p| p["id"] == *id)
                    .ok_or("rule input absent")?;
                let f = self
                    .action
                    .fields()
                    .get(&weft_core::mutation::canonical(&json!([
                        p["field"]["module"],
                        p["field"]["element"]
                    ])))
                    .ok_or("rule field absent")?;
                (s.slot(json!({"input":id}), &f.carrier), "FALSE".into())
            }
            RuleNode::State {
                phase,
                frame,
                field,
            } => {
                let exists = s.slot(
                    json!({"snapshotExists":phase,"image":format!("frame:{frame}")}),
                    &Carrier::Boolean,
                );
                (s.slot(json!({"snapshot":phase,"image":format!("frame:{frame}"),"field":field.reference}),&field.carrier),format!("NOT ({exists})"))
            }
            RuleNode::Exists { phase, frame } => (
                s.slot(
                    json!({"snapshotExists":phase,"image":format!("frame:{frame}")}),
                    &Carrier::Boolean,
                ),
                "FALSE".into(),
            ),
            RuleNode::Operation { operator, args } => {
                let (a, ae) = self.rule(&args[0], s)?;
                if operator == "present" {
                    ("TRUE".into(), ae)
                } else if operator == "not" {
                    (format!("NOT ({a})"), format!("({ae} OR {a} IS NULL)"))
                } else {
                    let (b, be) = self.rule(&args[1], s)?;
                    match operator.as_str() {
                        "and" => (
                            format!("CASE WHEN {a} IS TRUE THEN {b} ELSE FALSE END"),
                            format!(
                                "({ae} OR {a} IS NULL OR ({a} IS TRUE AND ({be} OR {b} IS NULL)))"
                            ),
                        ),
                        "or" => (
                            format!("CASE WHEN {a} IS TRUE THEN TRUE ELSE {b} END"),
                            format!(
                                "({ae} OR {a} IS NULL OR ({a} IS FALSE AND ({be} OR {b} IS NULL)))"
                            ),
                        ),
                        "eq" => {
                            let (a, b) = if args[0].family == "null" && args[1].family != "null" {
                                (
                                    format!(
                                        "NULL::pg_catalog.{}",
                                        if args[1].family == "boolean" {
                                            "bool"
                                        } else if args[1].family == "integer" {
                                            "numeric"
                                        } else {
                                            "text"
                                        }
                                    ),
                                    b,
                                )
                            } else if args[1].family == "null" && args[0].family != "null" {
                                (
                                    a,
                                    format!(
                                        "NULL::pg_catalog.{}",
                                        if args[0].family == "boolean" {
                                            "bool"
                                        } else if args[0].family == "integer" {
                                            "numeric"
                                        } else {
                                            "text"
                                        }
                                    ),
                                )
                            } else {
                                (a, b)
                            };
                            (
                                format!("({a} IS NOT DISTINCT FROM {b})"),
                                format!("({ae} OR {be})"),
                            )
                        }
                        op => {
                            let native = match op {
                                "lt" => "<",
                                "le" => "<=",
                                "gt" => ">",
                                "ge" => ">=",
                                "add" => "+",
                                "sub" => "-",
                                _ => return Err("rule operator unsupported"),
                            };
                            (
                                format!("({a} {native} {b})"),
                                format!("({ae} OR {be} OR {a} IS NULL OR {b} IS NULL)"),
                            )
                        }
                    }
                }
            }
        };
        if value.len().saturating_add(error.len()) > 8192 {
            return Err("native action lowered rule budget");
        };
        Ok((value, error))
    }
    fn condition(&self, condition: &ActionCondition) -> Result<Value> {
        let mut s = Statement { slots: vec![] };
        let (value, error) = self.rule(&condition.expression, &mut s)?;
        let sql = format!(
            "SELECT ({error} OR {value} IS NULL) AS evaluation_error, ({value} IS TRUE) AS passed"
        );
        Ok(
            json!({"id":condition.id,"code":condition.code,"retryable":condition.retryable,"source":condition.source,"statement":s.finish(sql)}),
        )
    }
}
fn compile(source: &str) -> Result<Value> {
    let admitted = raw_action::admit_json(source)?;
    let r = admitted.request();
    if !matches!(r.target.as_str(), "postgresql18" | "lakebase18") { return Err("PostgreSQL native action target required"); }
    let lower = Lower {
        action: &admitted,
        binding: &r.binding,
    };
    let mut s = Statement { slots: vec![] };
    let roles = r
        .role_mappings
        .iter()
        .filter(|(id, _)| {
            admitted.action()["authorization"]["roles"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == *id)
        })
        .map(|(_, native)| {
            let role = s.slot(json!({"nativeRole":native}), &Carrier::String);
            format!("pg_catalog.pg_has_role(session_user, {role}, 'MEMBER')")
        })
        .collect::<Vec<_>>();
    if roles.is_empty() {
        return Err("ROLE_MAPPING_MISSING");
    }
    let admission=s.finish(format!("SELECT ({}) AS allowed, session_user::pg_catalog.text AS authenticated, current_user::pg_catalog.text AS effective, pg_catalog.pg_current_xact_id()::pg_catalog.text AS xid",roles.join(" OR ")));
    let mut images = BTreeMap::<String, (ActionEntity, BTreeMap<String, ActionField>)>::new();
    for (name, entity) in admitted.entities() {
        let mut fields = BTreeMap::new();
        for f in &entity.key.fields {
            fields.insert(field_id(f), f.clone());
        }
        images.insert(format!("entity:{name}"), (entity.clone(), fields));
    }
    for (id, frame) in admitted.frames() {
        let mut fields = BTreeMap::new();
        for f in frame.entity.key.fields.iter().chain(frame.fields.iter()) {
            fields.insert(field_id(f), f.clone());
        }
        images.insert(format!("frame:{id}"), (frame.entity.clone(), fields));
    }
    for (i, effect) in admitted.effects().iter().enumerate() {
        let mut fields = BTreeMap::new();
        for f in effect
            .entity
            .key
            .fields
            .iter()
            .chain(effect.values.iter().map(|(f, _)| f))
        {
            fields.insert(field_id(f), f.clone());
        }
        images.insert(format!("effect:{i}"), (effect.entity.clone(), fields));
    }
    let mut pre_images = vec![];
    let mut post_images = vec![];
    for (name, (entity, fields)) in &images {
        let fields = fields.values().cloned().collect::<Vec<_>>();
        pre_images.push(lower.image(name, entity, &fields, true)?);
        post_images.push(lower.image(name, entity, &fields, false)?);
    }
    let mut effects = vec![];
    for (i, effect) in admitted.effects().iter().enumerate() {
        let mut s = Statement { slots: vec![] };
        let table = lower.table(&effect.entity.key.table)?;
        let returning = effect
            .entity
            .key
            .fields
            .iter()
            .map(|f| lower.column(f))
            .collect::<Result<Vec<_>>>()?
            .join(", ");
        let sql = match effect.kind.as_str() {
            "set" => {
                if effect.values.is_empty() {
                    return Err("empty scalar SET unsupported");
                };
                if effect.values.iter().any(|(f, _)| {
                    effect
                        .entity
                        .key
                        .fields
                        .iter()
                        .any(|k| k.reference == f.reference)
                }) {
                    return Err("native action Key mutation unsupported");
                };
                let set = effect
                    .values
                    .iter()
                    .map(|(f, value)| {
                        Ok(format!(
                            "{} = {}",
                            lower.column(f)?,
                            s.slot(value.clone(), &f.carrier)
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?
                    .join(", ");
                let predicate = lower.predicate(&effect.entity, &mut s)?;
                format!("UPDATE {table} SET {set} WHERE {predicate} RETURNING {returning}")
            }
            "delete" => {
                let predicate = lower.predicate(&effect.entity, &mut s)?;
                format!("DELETE FROM {table} WHERE {predicate} RETURNING {returning}")
            }
            "create" => {
                let bound = r
                    .binding
                    .tables
                    .iter()
                    .find(|t| t.logical == effect.entity.key.table)
                    .unwrap();
                if bound.columns.len() != effect.values.len() {
                    return Err("native CREATE requires every scalar member; no inferred defaults or missing/null collapse");
                };
                let columns = effect
                    .values
                    .iter()
                    .map(|(f, _)| lower.column(f))
                    .collect::<Result<Vec<_>>>()?
                    .join(", ");
                let values = effect
                    .values
                    .iter()
                    .map(|(f, v)| s.slot(v.clone(), &f.carrier))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("INSERT INTO {table} ({columns}) VALUES ({values}) RETURNING {returning}")
            }
            _ => return Err("unsupported native effect"),
        };
        let guard = if effect.kind == "set" {
            let mut g = Statement { slots: vec![] };
            let different = effect
                .values
                .iter()
                .map(|(f, v)| {
                    Ok(format!(
                        "{} IS DISTINCT FROM {}",
                        lower.column(f)?,
                        g.slot(v.clone(), &f.carrier)
                    ))
                })
                .collect::<Result<Vec<_>>>()?
                .join(" OR ");
            let predicate = lower.predicate(&effect.entity, &mut g)?;
            g.finish(format!(
                "SELECT ({different}) AS changed FROM {table} WHERE {predicate} FOR UPDATE"
            ))
        } else {
            Value::Null
        };
        effects.push(json!({"guard":guard,"id":effect.id,"kind":effect.kind,"entity":effect.entity,"image":format!("effect:{i}"),"values":effect.values,"statement":s.finish(sql)}));
    }
    let tables = images
        .values()
        .map(|(e, _)| lower.table(&e.key.table))
        .collect::<Result<BTreeSet<_>>>()?;
    let lock_sql = tables
        .into_iter()
        .map(|t| format!("LOCK TABLE {t} IN ROW EXCLUSIVE MODE"))
        .collect::<Vec<_>>();
    let preconditions = admitted
        .preconditions()
        .iter()
        .map(|c| lower.condition(c))
        .collect::<Result<Vec<_>>>()?;
    let postconditions = admitted
        .postconditions()
        .iter()
        .map(|c| lower.condition(c))
        .collect::<Result<Vec<_>>>()?;
    let mut result = json!({"version":"weft.native-action/0.1","profile":"umf.actions.native-sql/0.1","status":"compiled","target":r.target,"source":r.source,"binding":r.binding,"security":r.security,"action":admitted.action(),"entities":admitted.entities(),"fields":admitted.fields(),"frames":admitted.frames(),"admission":admission,"lockSql":lock_sql,"preImages":pre_images,"postImages":post_images,"preconditions":preconditions,"postconditions":postconditions,"effects":effects,"obligations":{"transaction":"serializable","nativeAdmission":"original-host-qualified-unchanged-closure-fence","principal":"qualified-human-session","sessionRole":"unchanged","receipt":"original-writer-incarnation-and-xid8","noRetry":true,"validate":"same-checks-always-rollback","dependencyProfile":"scalar-base-tables-no-user-triggers-rules-routines-fk-generated-defaults-or-checks"}});
    if r.verification_profile.as_deref()==Some("source-exact/0.1") {
        let doc=weft_core::json::checked_json(&r.binding.model_json).map_err(|_|"original action model JSON refused")?;
        let mut tables=std::collections::BTreeSet::new();
        for entity in admitted.entities().values(){tables.insert(entity.key.table.clone());}
        for field in admitted.fields().values(){tables.insert(field.table.clone());}
        let mut admissions=vec![];let mut index_tables=vec![];let mut keyword_effects=vec![];let mut index_locks=vec![];let mut locked_physical=std::collections::BTreeSet::new();
        for logical in tables {
            let table=r.binding.tables.iter().find(|t|t.logical==logical).ok_or("original action native table absent")?;
            let record=weft_core::raw_read::binding_record(&r.binding,&logical)?;
            let columns=crate::raw_read::native_projection::columns_for(&r.binding,&r.target,&doc,&record,table,true)?;
            for column in columns.values(){admissions.push(json!({"nativeAdmission":crate::raw_read::native_projection::action_admission(column,table,&r.binding.schema)}));}
            admissions.push(json!({"nativeAdmission":crate::raw_read::native_projection::action_table_admission(table,&r.binding.schema,&columns.values().map(|c|c.native.clone()).collect::<Vec<_>>())}));
            let mut allowed=vec![];
            for index in weft_core::search::admit_keyword_effects(&r.binding,&r.target,&logical)? {
                let metadata=crate::raw_read::action_keyword_index_metadata(&index,table,&r.binding.schema)?;
                allowed.push(metadata["nativeBinding"]["native"].as_str().ok_or("keyword mutation index name absent")?.to_owned());
                keyword_effects.push(metadata);
            }
            let mut native_keys=vec![];
            for key in record.value["keys"].as_array().ok_or("original Record Keys absent")? {
                let mut native_fields=vec![];
                for reference in key["fields"].as_array().ok_or("original Key fields absent")? {
                    let module=doc["modules"].as_array().unwrap().iter().find(|m|m["id"]==reference["module"]).ok_or("original Key Field module absent")?;
                    let field=module["elements"].as_array().unwrap().iter().find(|f|f["id"]==reference["element"]).ok_or("original Key Field absent")?;
                    let name=field["name"].as_str().ok_or("original Key Field name absent")?;
                    native_fields.push(columns.get(name).ok_or("original Key native column absent")?.native.clone());
                }
                if native_fields.is_empty(){return Err("original Key native inventory empty")}
                native_keys.push(native_fields);
            }
            if native_keys.is_empty(){return Err("original mutation Record Keys required")}
            let native_table=lower.table(&logical)?;
            if !locked_physical.insert(native_table.clone()){return Err("source-exact mutation duplicate physical Record tables unsupported")}
            index_locks.push(format!("LOCK TABLE {native_table} IN SHARE ROW EXCLUSIVE MODE NOWAIT"));
            let native_admission=crate::native_catalog_fence::mutation_index_admission(table,&r.binding.schema,&allowed,&native_keys);
            admissions.push(json!({"nativeAdmission":native_admission}));
            index_tables.push(json!({"logicalTable":logical,"nativeTable":table.native,"allowedIndexes":allowed,"originalKeys":native_keys,"nativeAdmission":native_admission}));
        }
        if admissions.is_empty(){return Err("source-exact action requires touched native table inventory")}
        result["obligations"]["nativeProjection"]=json!({"version":"umf.native-columns/0.1","verificationProfile":"source-exact/0.1","nativeCatalogBeforeAfter":true,"columns":admissions});
        result["obligations"]["mutationIndexEffects"]=json!({"version":"umf.pg-index-effects/0.1","nativeCatalogBeforeAfter":true,"lockPosition":"before-first-snapshot","lockSql":index_locks,"tables":index_tables});
        if !keyword_effects.is_empty(){result["obligations"]["textSearchEffects"]=json!({"version":"umf.pg-ts-effects/0.1","nativeCatalogBeforeAfter":true,"indexes":keyword_effects});}
    }
    crate::native_catalog_fence::attach(&r.binding,&mut result["obligations"])?;
    crate::native_catalog_fence::seal_version(&mut result)?;
    let digest = weft_core::json::sha256(weft_core::mutation::canonical(&result).as_bytes());
    result["structuralDigest"] =
        json!({"owner":"weft-postgresql-native-action","version":"0.1","sha256":digest});
    Ok(result)
}
pub fn compile_json(request: &str) -> String {
    match compile(request){Ok(value)=>value.to_string(),Err(code)=>json!({"version":"weft.native-action/0.1","status":"blocked","diagnostics":[{"code":code}]}).to_string()}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> Value {
        serde_json::from_str(include_str!(
            "../../../tests/native-actions/action-request.json"
        ))
        .unwrap()
    }
    fn compile_value(value: &Value) -> Value {
        serde_json::from_str(&compile_json(&value.to_string())).unwrap()
    }
    #[test]
    fn source_exact_action_retains_complete_original_native_facts(){
        let r:Value=serde_json::from_str(include_str!("../../../tests/native-actions/source-exact-action-request.json")).unwrap();
        let p=compile_value(&r);assert_eq!(p["status"],"compiled","{p}");
        assert_eq!(p["obligations"]["nativeProjection"]["verificationProfile"],"source-exact/0.1");
        assert_eq!(p["obligations"]["nativeProjection"]["columns"].as_array().unwrap().len(),4);
        assert!(p["obligations"]["nativeProjection"]["columns"][0]["nativeAdmission"]["sql"].as_str().unwrap().contains("co.oid=a.attcollation"));
        for c in p["obligations"]["nativeProjection"]["columns"].as_array().unwrap(){assert!(c["nativeAdmission"]["sql"].as_str().unwrap().contains("pg_catalog.to_regclass(pg_catalog.quote_ident("));assert!(c["nativeAdmission"]["parameters"].as_array().unwrap().iter().all(|p|p["source"].get("literal").is_some()));}
        let mut qualified=r.clone();qualified["binding"]["schema"]=json!("owned_schema");
        let q=compile_value(&qualified);assert_eq!(q["status"],"compiled","{q}");
        for c in q["obligations"]["nativeProjection"]["columns"].as_array().unwrap(){assert!(!c["nativeAdmission"]["sql"].as_str().unwrap().contains("to_regclass"));assert!(c["nativeAdmission"]["parameters"].as_array().unwrap().iter().any(|p|p["source"]["literal"]["string"]=="owned_schema"));}
        let mut b=r.clone();b["binding"].as_object_mut().unwrap().remove("nativeColumns");assert_eq!(compile_value(&b)["status"],"blocked");
        let mut b=r.clone();b["verificationProfile"]=json!("source-exact/0.2");assert_eq!(compile_value(&b)["status"],"blocked");
        let mut b=r.clone();b["binding"]["schemaBinding"]=json!("qualified");assert_eq!(compile_value(&b)["status"],"blocked");
        let mut b=qualified.clone();b["binding"]["schemaBinding"]=json!("search-path");assert_eq!(compile_value(&b)["status"],"blocked");
        assert!(compile_value(&request())["obligations"].get("nativeProjection").is_none());
    }
    #[test]
    fn original_source_pins_and_native_markers_are_deterministic() {
        let r = request();
        let result = compile_value(&r);
        assert_eq!(result["status"], "compiled", "{result}");
        assert_eq!(result, compile_value(&r));
        assert_eq!(result["profile"], "umf.actions.native-sql/0.1");
        assert_eq!(result["source"], r["source"]);
        assert_eq!(result["security"], r["security"]);
        assert_eq!(result["obligations"]["transaction"], "serializable");
        assert!(result["admission"]["sql"]
            .as_str()
            .unwrap()
            .contains("session_user"));
        assert!(result["effects"][0]["guard"]["sql"]
            .as_str()
            .unwrap()
            .contains("IS DISTINCT FROM"));
        assert!(result["effects"][0]["statement"]["sql"]
            .as_str()
            .unwrap()
            .contains("RETURNING"));
    }
    #[test]
    fn wrong_original_source_and_missing_role_mapping_refuse_before_program() {
        for field in ["documentSha256", "actionPointer", "documentId"] {
            let mut r = request();
            r["source"][field] = json!("wrong");
            let out = compile_value(&r);
            assert_eq!(out["status"], "blocked");
            assert!(out.get("effects").is_none());
        }
        let mut r = request();
        r["roleMappings"] = json!({});
        assert_eq!(compile_value(&r)["status"], "blocked");
        r = request();
        r["sessionPrincipal"] = json!("service");
        assert_eq!(compile_value(&r)["status"], "blocked");
    }
    #[test]
    fn unknown_selected_action_and_frame_escape_are_not_ignored() {
        for escape in [false, true] {
            let mut r = request();
            let mut d: Value =
                serde_json::from_str(r["binding"]["modelJson"].as_str().unwrap()).unwrap();
            let a = &mut d["modules"][0]["extensions"]["umf.actions"]["actions"][0];
            if escape {
                a["writes"][0]["fields"] = json!([]);
            } else {
                a["future"] = json!({"mustPreserve":true});
            }
            let text = d.to_string();
            let digest = weft_core::json::sha256(text.as_bytes());
            r["binding"]["modelJson"] = json!(text);
            r["binding"]["modelSha256"] = json!(digest);
            r["source"]["documentSha256"] = json!(digest);
            assert_eq!(compile_value(&r)["status"], "blocked");
        }
    }
}
