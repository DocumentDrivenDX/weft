// SPDX-License-Identifier: MIT OR Apache-2.0
//! Build-issued, single-root fragments for compiler-free composed reads.
use serde::Deserialize;
use serde_json::{json, Value};
use sqlparser::{
    ast::{SetExpr, Statement},
    dialect::PostgreSqlDialect,
    parser::Parser,
};
use std::collections::BTreeMap;
use weft_core::raw_read::{Binding, Parameter};
type Result<T> = std::result::Result<T, &'static str>;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Request {
    version: String,
    target: String,
    binding: Binding,
    module: String,
}

fn field_key(reference: &Value) -> Result<String> {
    let component = |value: &str| value.replace('~', "~0").replace('/', "~1");
    Ok(format!(
        "{}/{}",
        component(reference["module"].as_str().ok_or("field module absent")?),
        component(
            reference["element"]
                .as_str()
                .ok_or("field element absent")?
        )
    ))
}
fn identity_component(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}
fn parse_compiled(sql: &str) -> Result<(Vec<String>, String, Vec<String>)> {
    let mut statements =
        Parser::parse_sql(&PostgreSqlDialect {}, sql).map_err(|_| "issuer SQL parse refused")?;
    if statements.len() != 1 {
        return Err("issuer SQL statement count differs");
    }
    let Statement::Query(query) = statements.remove(0) else {
        return Err("issuer SQL is not SELECT");
    };
    let SetExpr::Select(select) = query.body.as_ref() else {
        return Err("issuer SQL root differs");
    };
    let projections = select.projection.iter().map(ToString::to_string).collect();
    let from = select.from.first().ok_or("issuer FROM absent")?.to_string();
    let order = query
        .order_by
        .as_ref()
        .and_then(|o| match &o.kind {
            sqlparser::ast::OrderByKind::Expressions(items) => {
                Some(items.iter().map(ToString::to_string).collect())
            }
            _ => None,
        })
        .unwrap_or_default();
    Ok((projections, from, order))
}
fn compile_sql(
    target: &str,
    binding: &Binding,
    sql: String,
    parameters: Vec<Parameter>,
) -> Result<Value> {
    let admitted = weft_core::raw_read::admit_generated_read(
        target.to_owned(),
        binding.clone(),
        sql,
        parameters,
        1000,
    )?;
    super::lower(&admitted)
}
fn dbapi_sql_with_order(sql: &str, parameter_order: &[&str]) -> Result<(String, Vec<String>)> {
    let mut transport = json!({"sql":sql,"parameterOrder":parameter_order});
    crate::dbapi::attach(&mut transport)?;
    let sql = transport["dbapiSql"]
        .as_str()
        .map(str::to_owned)
        .ok_or("generated DB-API SQL absent")?;
    let order = transport["dbapiParameterOrder"]
        .as_array()
        .filter(|items| items.iter().all(Value::is_string))
        .ok_or("generated DB-API parameter order absent")?
        .iter()
        .map(|item| item.as_str().unwrap().to_owned())
        .collect();
    Ok((sql, order))
}
fn dbapi_sql(sql: &str, parameter_order: &[&str]) -> Result<String> {
    dbapi_sql_with_order(sql, parameter_order).map(|(sql, _)| sql)
}
fn get_many_key_cast(carrier: &weft_core::raw_read::Carrier) -> Result<&'static str> {
    match carrier {
        weft_core::raw_read::Carrier::Integer => Ok("numeric"),
        weft_core::raw_read::Carrier::Decimal => Ok("numeric"),
        weft_core::raw_read::Carrier::String => Ok("text"),
        weft_core::raw_read::Carrier::Boolean => Ok("bool"),
        weft_core::raw_read::Carrier::Uuid => Ok("uuid"),
        weft_core::raw_read::Carrier::Date => Ok("date"),
        weft_core::raw_read::Carrier::Timestamp => Ok("timestamptz"),
        weft_core::raw_read::Carrier::Bits128 => {
            Err("generated read v4 Bits128 Key equality is outside the issued native profile")
        }
        weft_core::raw_read::Carrier::LocalTimestamp => Err(
            "generated read v4 LocalTimestamp Key equality is outside the issued native profile",
        ),
    }
}
fn native_type_carrier_oid(native_type: &Value) -> Result<(&'static str, u64)> {
    let kind = native_type["kind"]
        .as_str()
        .ok_or("generated native type kind absent")?;
    match kind {
        "boolean" => Ok(("boolean", 16)),
        "int2" => Ok(("integer", 21)),
        "int4" => Ok(("integer", 23)),
        "int8" => Ok(("integer", 20)),
        "float4" => Ok(("float", 700)),
        "float8" => Ok(("float", 701)),
        "numeric" => Ok(("decimal", 1700)),
        "text" => Ok(("string", 25)),
        "varchar" => Ok(("string", 1043)),
        "char" => Ok(("string", 1042)),
        "uuid" => Ok(("uuid", 2950)),
        "date" => Ok(("date", 1082)),
        "timestamp" => Ok(("timestamp", 1114)),
        "timestamptz" => Ok(("timestamp", 1184)),
        "bytea" => Ok(("bits128", 17)),
        _ => Err("generated read v5 native output type is outside the exact OID profile"),
    }
}
fn ordinary_page_field_output(field_id: &str, public_field: &Value, expected_ref: &Value, bound: &weft_core::raw_read::Column, mapping: &Value) -> Result<Value> {
    let source_ref = &public_field["sourceField"];
    if source_ref != expected_ref { return Err("generated Page source Field identity differs"); }
    if mapping["field"] != *expected_ref || mapping["name"] != bound.native { return Err("generated Page Field source binding differs"); }
    let field_native_type = mapping.get("nativeType").ok_or("generated Page Field native type absent")?.clone();
    let (carrier, oid) = native_type_carrier_oid(&field_native_type)?;
    if public_field["output"]["carrier"] != carrier || bound.nullable != (public_field["output"]["nullable"] == true) { return Err("generated Page Field carrier/nullability differs"); }
    let mut public_output = public_field["output"].clone();
    public_output["nativeType"] = field_native_type;
    public_output["oid"] = json!(oid);
    Ok(json!({"fieldId":field_id,"sourceField":expected_ref,"nativeColumn":bound.native,"output":public_output}))
}
fn ordinary_page_key_type_supported(native_type: &Value, bound: &weft_core::raw_read::Column) -> bool {
    native_type_carrier_oid(native_type).is_ok_and(|(carrier, _)| carrier == "string" && native_type["kind"] != "char")
        && bound.carrier == weft_core::raw_read::Carrier::String && !bound.nullable
}
fn mark_forward_connection_unsupported(root: &mut Value) -> Result<()> {
    root.as_object_mut()
        .ok_or("generated read v5 root object absent")?
        .insert(
            "forwardConnectionSupport".into(),
            json!({"profile":"weft.generated-forward-connection/1","status":"unsupported","reason":"ROOT_OUTSIDE_PROFILE"}),
        );
    Ok(())
}
fn path_sql(path: &[String]) -> Result<String> {
    if path.len() != 2 {
        return Err("generated root logical path differs");
    }
    let parts = path
        .iter()
        .map(|s| {
            if s.is_empty() || s.contains('\0') {
                return Err("generated logical root name outside SQL identifier syntax");
            }
            Ok(format!("\"{}\"", s.replace('"', "\"\"")))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(parts.join("."))
}
fn opaque(prefix: &str, value: &Value) -> String {
    format!(
        "{prefix}_{}",
        weft_core::json::sha256(weft_core::mutation::canonical(value).as_bytes())
    )
}
fn build(request: Request, binding_digest: String) -> Result<Value> {
    if request.version != "weft.generated-read-algebra-request/1"
        || !matches!(request.target.as_str(), "postgresql18" | "lakebase18")
        || request.module.is_empty()
        || request.module.len() > 256
    {
        return Err("generated read algebra request profile differs");
    }
    if request
        .binding
        .native_lifecycle
        .as_ref()
        .is_none_or(|l| l.module != request.module)
    {
        return Err("generated read algebra requires exact native lifecycle module binding");
    }
    let binding = request.binding;
    let model_sha = binding.model_sha256.clone();
    let source_sql = "SELECT 1".to_owned();
    let admitted = weft_core::raw_read::admit_generated_read(
        request.target.clone(),
        binding.clone(),
        source_sql,
        vec![],
        1000,
    )?;
    let document = admitted.original_document();
    let modules = document["modules"]
        .as_array()
        .ok_or("generated read source modules absent")?;
    let module = modules
        .iter()
        .find(|m| m["id"] == request.module)
        .ok_or("generated read source module absent")?;
    let _elements = module["elements"]
        .as_array()
        .ok_or("generated read source elements absent")?;
    let mut roots = BTreeMap::new();
    for table in &binding.tables {
        let record = admitted.original_record(&table.logical)?;
        if record.identity.module != request.module {
            continue;
        }
        let record_id = record.identity.element.clone();
        let relation = path_sql(&table.logical)?;
        let mut members = Vec::new();
        for column in &table.columns {
            let reference = admitted.original_field_ref(&table.logical, &column.logical)?;
            if reference["module"] != request.module {
                return Err("generated read profile requires same-module native fields");
            }
            let field_id = field_key(&reference)?;
            let codec =
                weft_core::table_query::generated_codec_for_member(document, &reference).ok();
            members.push((reference, field_id, column.clone(), codec));
        }
        if members.is_empty() {
            return Err("generated root has no bound scalar fields");
        }
        let source_members = record
            .value["members"]
            .as_array()
            .ok_or("generated Record member order absent")?;
        let mut member_positions = BTreeMap::new();
        for (position, reference) in source_members.iter().enumerate() {
            if member_positions.insert(field_key(reference)?, position).is_some() {
                return Err("generated Record member order is ambiguous");
            }
        }
        let mut ordered_members = members
            .into_iter()
            .map(|member| {
                let position = member_positions
                    .get(&member.1)
                    .copied()
                    .ok_or("generated bound Field is absent from Record member order")?;
                Ok((position, member))
            })
            .collect::<Result<Vec<_>>>()?;
        ordered_members.sort_by_key(|(position, _)| *position);
        let members = ordered_members
            .into_iter()
            .map(|(_, member)| member)
            .collect::<Vec<_>>();
        let projection_sql = members
            .iter()
            .enumerate()
            .map(|(i, (_, _, c, _))| {
                format!(
                    "\"r\".\"{}\" AS \"__weft_field_{i}\"",
                    c.logical.replace('"', "\"\"")
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("SELECT {projection_sql} FROM {relation} AS \"r\"");
        let compiled = compile_sql(&request.target, &binding, sql, vec![])?;
        if compiled["status"] != "compiled" {
            return Err("generated root field admission blocked");
        }
        let (select_sql, from_sql, _) = parse_compiled(
            compiled["sql"]
                .as_str()
                .ok_or("generated root SQL absent")?,
        )?;
        if select_sql.len() != members.len() {
            return Err("generated root projection count differs");
        }
        let select_sql = select_sql
            .iter()
            .map(|sql| dbapi_sql(sql, &[]))
            .collect::<Result<Vec<_>>>()?;
        let from_sql = dbapi_sql(&from_sql, &[])?;
        let mut fields = BTreeMap::new();
        let mut default_selection = Vec::new();
        let mut predicates = BTreeMap::new();
        let mut orders = BTreeMap::new();
        let mut by_logical = BTreeMap::new();
        for (index, (reference, field_id, column, codec)) in members.iter().enumerate() {
            if fields.insert(field_id.clone(), json!({"sourceField":{"module":reference["module"],"element":reference["element"]},"selectSql":select_sql[index],"output":compiled["outputs"][index]})).is_some() {
                return Err("generated field identity is ambiguous");
            }
            default_selection.push(field_id.clone());
            by_logical.insert(column.logical.clone(), field_id.clone());
            for direction in ["asc", "desc"] {
                let ordering = format!("SELECT \"r\".\"{}\" AS \"__weft_order\" FROM {relation} AS \"r\" ORDER BY \"r\".\"{}\" {} NULLS LAST", column.logical.replace('"', "\"\""), column.logical.replace('"', "\"\""), direction.to_ascii_uppercase());
                let order_compiled = compile_sql(&request.target, &binding, ordering, vec![])?;
                if order_compiled["status"] != "compiled" {
                    continue;
                }
                let (_, _, fragments) = parse_compiled(
                    order_compiled["sql"]
                        .as_str()
                        .ok_or("generated order SQL absent")?,
                )?;
                if fragments.len() != 1 {
                    return Err("generated order fragment count differs");
                }
                let order_sql = dbapi_sql(&fragments[0], &[])?;
                let id = opaque("o", &json!([field_id, direction, "last"]));
                if orders.insert(id, json!({"fieldId":field_id,"direction":direction,"nulls":"last","sql":order_sql})).is_some() {
                    return Err("generated order identity collision");
                }
            }
            if let Some(codec) = codec {
                for operator in ["eq", "ne", "lt", "le", "gt", "ge"] {
                    let token = match operator {
                        "eq" => "=",
                        "ne" => "<>",
                        "lt" => "<",
                        "le" => "<=",
                        "gt" => ">",
                        "ge" => ">=",
                        _ => unreachable!(),
                    };
                    let parameter = Parameter {
                        name: "__weft_generated_value".into(),
                        carrier: column.carrier.clone(),
                        nullable: column.nullable,
                        container: None,
                        element_nullable: None,
                        max_items: None,
                    };
                    let predicate_sql = format!("SELECT \"r\".\"{}\" AS \"__weft_predicate\" FROM {relation} AS \"r\" WHERE \"r\".\"{}\" {token} :__weft_generated_value", column.logical.replace('"', "\"\""), column.logical.replace('"', "\"\""));
                    let predicate_compiled =
                        compile_sql(&request.target, &binding, predicate_sql, vec![parameter])?;
                    if predicate_compiled["status"] != "compiled" {
                        continue;
                    }
                    let mut statements = Parser::parse_sql(
                        &PostgreSqlDialect {},
                        predicate_compiled["sql"]
                            .as_str()
                            .ok_or("generated predicate SQL absent")?,
                    )
                    .map_err(|_| "generated predicate SQL parse refused")?;
                    let Statement::Query(query) = statements.remove(0) else {
                        return Err("generated predicate query differs");
                    };
                    let SetExpr::Select(select) = query.body.as_ref() else {
                        return Err("generated predicate SELECT differs");
                    };
                    let predicate = select
                        .selection
                        .as_ref()
                        .ok_or("generated predicate WHERE absent")?
                        .to_string();
                    let predicate = dbapi_sql(&predicate, &["__weft_generated_value"])?;
                    let id = opaque("p", &json!([field_id, operator]));
                    if predicates.insert(id, json!({"fieldId":field_id,"operator":operator,"sql":predicate,"arity":1,"codec":codec})).is_some() {
                        return Err("generated predicate identity collision");
                    }
                }
            }
            for operator in ["is-null", "is-not-null"] {
                let suffix = if operator == "is-null" {
                    "IS NULL"
                } else {
                    "IS NOT NULL"
                };
                let null_sql = format!("SELECT \"r\".\"{}\" AS \"__weft_null\" FROM {relation} AS \"r\" WHERE \"r\".\"{}\" {suffix}", column.logical.replace('"', "\"\""), column.logical.replace('"', "\"\""));
                let null_compiled = compile_sql(&request.target, &binding, null_sql, vec![])?;
                if null_compiled["status"] != "compiled" {
                    continue;
                }
                let mut statements = Parser::parse_sql(
                    &PostgreSqlDialect {},
                    null_compiled["sql"].as_str().ok_or("null SQL absent")?,
                )
                .map_err(|_| "null SQL parse refused")?;
                let Statement::Query(query) = statements.remove(0) else {
                    return Err("null query differs");
                };
                let SetExpr::Select(select) = query.body.as_ref() else {
                    return Err("null SELECT differs");
                };
                let fragment = select
                    .selection
                    .as_ref()
                    .ok_or("null predicate absent")?
                    .to_string();
                let fragment = dbapi_sql(&fragment, &[])?;
                let id = opaque("p", &json!([field_id, operator]));
                if predicates
                    .insert(
                        id,
                        json!({"fieldId":field_id,"operator":operator,"sql":fragment,"arity":0}),
                    )
                    .is_some()
                {
                    return Err("generated predicate identity collision");
                }
            }
        }
        let key_fields = table
            .key
            .iter()
            .map(|logical| {
                by_logical
                    .get(logical)
                    .ok_or("generated root Key field lacks bound scalar Field")
            })
            .collect::<Result<Vec<_>>>()?;
        let key_tie_order = key_fields
            .iter()
            .map(|field_id| opaque("o", &json!([field_id, "asc", "last"])))
            .collect::<Vec<_>>();
        let unique_key_fields = key_fields.iter().collect::<std::collections::BTreeSet<_>>();
        if key_tie_order.is_empty()
            || key_tie_order.len() != table.key.len()
            || unique_key_fields.len() != key_fields.len()
            || key_tie_order.iter().any(|id| !orders.contains_key(id))
        {
            return Err("generated root stable Key ordering unavailable");
        }
        let root_key = format!(
            "{}/{}",
            identity_component(&request.module),
            identity_component(&record_id)
        );
        let obligations = compiled["obligations"].clone();
        let native_read_guard = json!({"profile":"pg18-read-frame/0.1","maxNativeBytes":8388608,"sqlstate":"22012","includesLookahead":true,"partialFrames":false});
        if roots.insert(root_key,json!({"module":request.module,"record":record_id,"fromSql":format!("FROM {from_sql}"),"defaultSelection":default_selection,"fields":fields,"predicates":predicates,"orders":orders,"keyTieOrder":key_tie_order,"maxRows":1000,"obligations":obligations,"nativeReadGuard":native_read_guard})).is_some() {
            return Err("generated Record root identity collision");
        }
    }
    if roots.is_empty() {
        return Err("selected module has no admitted bound Record roots");
    }
    Ok(
        json!({"version":"weft.generated-read-algebra/1","readiness":"weft.generated-read-executable/1","source":{"modelSha256":model_sha,"module":request.module},"target":{"id":request.target,"modelSha256":binding.model_sha256,"bindingDigest":binding_digest},"roots":roots}),
    )
}

fn scalar_terminal(
    target: &str,
    binding: &Binding,
    relation: &str,
    expression: &str,
    alias: &str,
    expected_select: &str,
    expected_from: &str,
    expected_carrier: &str,
    native_type: Value,
    oid: u64,
) -> Result<Value> {
    let sql = format!(
        "SELECT {expression} AS \"{alias}\" FROM {relation} AS \"r\""
    );
    let compiled = compile_sql(target, binding, sql, vec![])?;
    if compiled["status"] != "compiled" {
        return Err("generated scalar terminal admission blocked");
    }
    let (projections, from, _) = parse_compiled(
        compiled["sql"]
            .as_str()
            .ok_or("generated scalar terminal SQL absent")?,
    )?;
    if projections.len() != 1 {
        return Err("generated scalar terminal projection count differs");
    }
    let select_sql = dbapi_sql(&projections[0], &[])?;
    let from_sql = format!("FROM {}", dbapi_sql(&from, &[])?);
    if select_sql != expected_select || from_sql != expected_from {
        return Err("generated scalar terminal SQL differs from issued root");
    }
    let mut output = compiled["outputs"]
        .as_array()
        .filter(|outputs| outputs.len() == 1)
        .and_then(|outputs| outputs.first())
        .cloned()
        .ok_or("generated scalar terminal output absent")?;
    if output["name"] != alias
        || output["carrier"] != expected_carrier
        || output["nullable"] != false
    {
        return Err("generated scalar terminal output profile differs");
    }
    let output_object = output
        .as_object_mut()
        .ok_or("generated scalar output descriptor differs")?;
    output_object.insert("nativeType".into(), native_type);
    output_object.insert("oid".into(), json!(oid));
    Ok(json!({
        "selectSql":select_sql,
        "output":output,
    }))
}

fn build_v2(request: Request, binding_digest: String) -> Result<Value> {
    if request.version != "weft.generated-read-algebra-request/2" {
        return Err("generated read algebra v2 request profile differs");
    }
    let mut internal = request.clone();
    internal.version = "weft.generated-read-algebra-request/1".into();
    let mut output = build(internal, binding_digest)?;
    let admitted = weft_core::raw_read::admit_generated_read(
        request.target.clone(),
        request.binding.clone(),
        "SELECT 1".to_owned(),
        vec![],
        1000,
    )?;
    let document = admitted.original_document();
    let roots = output["roots"]
        .as_object_mut()
        .ok_or("generated read v2 roots absent")?;
    for table in &request.binding.tables {
        let record = admitted.original_record(&table.logical)?;
        if record.identity.module != request.module {
            continue;
        }
        let root_id = format!(
            "{}/{}",
            identity_component(&request.module),
            identity_component(&record.identity.element)
        );
        let root = roots
            .get_mut(&root_id)
            .ok_or("generated read v2 root absent")?;
        let relation = path_sql(&table.logical)?;
        let root_from = root["fromSql"]
            .as_str()
            .ok_or("generated read v2 root FROM absent")?
            .to_owned();
        let mut primary_key = Vec::new();
        let mut key_ids = std::collections::BTreeSet::new();
        for logical in &table.key {
            let native_column = table
                .columns
                .iter()
                .find(|column| column.logical == *logical)
                .ok_or("generated read v2 Key column is unmapped")?;
            if native_column.nullable {
                return Err("generated read v2 Key column is natively nullable");
            }
            let reference = admitted.original_field_ref(&table.logical, logical)?;
            let source_field = document["modules"]
                .as_array()
                .and_then(|modules| {
                    modules.iter().find(|module| module["id"] == reference["module"])
                })
                .and_then(|module| module["elements"].as_array())
                .and_then(|elements| {
                    elements.iter().find(|field| field["id"] == reference["element"])
                })
                .ok_or("generated read v2 Key Field declaration absent")?;
            if source_field["nullability"] != "required" {
                return Err("generated read v2 Key Field is not source-required");
            }
            let field_id = field_key(&reference)?;
            if !key_ids.insert(field_id.clone())
                || root["fields"].get(&field_id).is_none()
            {
                return Err("generated read v2 primary Key Field is ambiguous or unmapped");
            }
            let predicate_id = root["predicates"]
                .as_object()
                .and_then(|predicates| {
                    predicates.iter().find_map(|(id, predicate)| {
                        (predicate["fieldId"] == field_id
                            && predicate["operator"] == "eq"
                            && predicate["arity"] == 1
                            && predicate["codec"]["profile"]
                                == "weft.query-codec-instructions/1"
                            && predicate["codec"]["steps"][0]["nullable"] == false)
                            .then(|| id.clone())
                    })
                })
                .ok_or("generated read v2 Key equality codec unavailable")?;
            primary_key.push(json!({"fieldId":field_id,"predicateId":predicate_id}));
        }
        if primary_key.is_empty() || primary_key.len() > 16 {
            return Err("generated read v2 primary Key outside profile bounds");
        }
        let binding_key_fields = table
            .key
            .iter()
            .map(|logical| admitted.original_field_ref(&table.logical, logical))
            .collect::<Result<Vec<_>>>()?;
        let original_primary_keys = record.value["keys"]
            .as_array()
            .ok_or("generated read v2 original Record Keys absent")?
            .iter()
            .filter(|key| key["primary"] == true && key["fields"] == json!(binding_key_fields))
            .collect::<Vec<_>>();
        if original_primary_keys.len() != 1 {
            return Err("generated read v2 binding key does not identify the unique original primary Key");
        }
        let primary_key_id = original_primary_keys[0]["id"]
            .as_str()
            .filter(|id| !id.is_empty())
            .ok_or("generated read v2 original primary Key ID absent")?;
        let count = scalar_terminal(
            &request.target,
            &request.binding,
            &relation,
            "COUNT(*)",
            "__weft_count",
            "pg_catalog.count(*) AS \"__weft_count\"",
            &root_from,
            "integer",
            json!({"kind":"int8"}),
            20,
        )?;
        let exists = scalar_terminal(
            &request.target,
            &request.binding,
            &relation,
            "TRUE",
            "__weft_exists",
            "true AS \"__weft_exists\"",
            &root_from,
            "boolean",
            json!({"kind":"boolean"}),
            16,
        )?;
        root.as_object_mut()
            .ok_or("generated read v2 root object absent")?
            .insert("primaryKey".into(), json!(primary_key));
        root.as_object_mut()
            .ok_or("generated read v2 root object absent")?
            .insert("primaryKeyId".into(), json!(primary_key_id));
        root.as_object_mut()
            .ok_or("generated read v2 root object absent")?
            .insert(
                "terminals".into(),
                json!({
                    "first":{"limit":1,"cardinality":"zero-or-one"},
                    "one":{"limit":2,"cardinality":"exactly-one"},
                    "count":{"selectSql":count["selectSql"],"output":count["output"],"cardinality":"one-integer"},
                    "exists":{"selectSql":exists["selectSql"],"output":exists["output"],"limit":1,"cardinality":"row-presence"},
                    "get":{"limit":2,"cardinality":"exactly-one"}
                }),
            );
    }
    output["version"] = json!("weft.generated-read-algebra/2");
    output["readiness"] = json!("weft.generated-read-executable/2");
    Ok(output)
}

pub fn prepare_generated_read_algebra_v2_json(raw: &str) -> Result<String> {
    if raw.len() > 8_388_608 {
        return Err("generated read algebra request capacity exceeded");
    }
    let value =
        weft_core::json::checked_json(raw).map_err(|_| "generated read algebra JSON invalid")?;
    let binding = value
        .get("binding")
        .ok_or("generated read algebra binding absent")?;
    let binding_digest =
        weft_core::json::sha256(weft_core::mutation::canonical(binding).as_bytes());
    let request: Request =
        serde_json::from_value(value).map_err(|_| "generated read algebra shape differs")?;
    let result = build_v2(request, binding_digest)?;
    let output = weft_core::mutation::canonical(&result);
    if output.len() > 8_388_608 {
        return Err("generated read algebra artifact capacity exceeded");
    }
    Ok(output)
}

fn build_v3(mut request: Request, binding_digest: String) -> Result<Value> {
    if request.version != "weft.generated-read-algebra-request/3" {
        return Err("generated read algebra v3 request profile differs");
    }
    request.version = "weft.generated-read-algebra-request/2".into();
    let mut output = build_v2(request, binding_digest)?;
    let template = crate::query_wire::composed_native_frame_template();
    if template.matches("{{SOURCE_SQL}}").count() != 1
        || template.matches("{{RETURN_COLUMNS}}").count() != 1
    {
        return Err("generated read v3 frame template slot count differs");
    }
    let template_digest = weft_core::json::sha256(template.as_bytes());
    let roots = output["roots"]
        .as_object_mut()
        .ok_or("generated read v3 roots absent")?;
    for root in roots.values_mut() {
        let default_selection = root["defaultSelection"]
            .as_array()
            .ok_or("generated read v3 default selection absent")?
            .iter()
            .map(|field_id| {
                field_id
                    .as_str()
                    .map(str::to_owned)
                    .ok_or("generated read v3 field identity absent")
            })
            .collect::<Result<Vec<_>>>()?;
        let fields = root["fields"]
            .as_object_mut()
            .ok_or("generated read v3 fields absent")?;
        for (index, field_id) in default_selection.iter().enumerate() {
            let field = fields
                .get_mut(field_id)
                .ok_or("generated read v3 selected field absent")?;
            field["frameSelectSql"] = json!(format!("f.\"__weft_field_{index}\""));
        }
        for terminal in ["count", "exists"] {
            let terminal = root["terminals"]
                .get_mut(terminal)
                .ok_or("generated read v3 scalar terminal absent")?;
            let name = terminal["output"]["name"]
                .as_str()
                .ok_or("generated read v3 scalar output name absent")?;
            let alias = match name {
                "__weft_count" => "f.\"__weft_count\"",
                "__weft_exists" => "f.\"__weft_exists\"",
                _ => return Err("generated read v3 scalar output alias differs"),
            };
            terminal["frameSelectSql"] = json!(alias);
        }
        root["frameOrdinalSql"] = json!("ROW_NUMBER() OVER (ORDER BY {{ORDER_SQL}}) AS \"__weft_ordinal\"");
        root["frameScalarOrdinalSql"] = json!("1::pg_catalog.int8 AS \"__weft_ordinal\"");
        root["nativeReadGuard"] = json!({
            "profile":"pg18-composed-native-frame/1",
            "byteMetric":"pg18-row-json-utf8/1",
            "maxNativeBytes":8388608,
            "sqlstate":"22012",
            "partialFrames":false,
            "template":{
                "version":"weft.pg18-composed-frame-template/1",
                "sql":template,
                "sha256":template_digest
            }
        });
    }
    output["version"] = json!("weft.generated-read-algebra/3");
    output["readiness"] = json!("weft.generated-read-executable/3");
    Ok(output)
}

pub fn prepare_generated_read_algebra_v3_json(raw: &str) -> Result<String> {
    if raw.len() > 8_388_608 {
        return Err("generated read algebra request capacity exceeded");
    }
    let value = weft_core::json::checked_json(raw)
        .map_err(|_| "generated read algebra JSON invalid")?;
    let binding = value
        .get("binding")
        .ok_or("generated read algebra binding absent")?;
    let binding_digest =
        weft_core::json::sha256(weft_core::mutation::canonical(binding).as_bytes());
    let request: Request =
        serde_json::from_value(value).map_err(|_| "generated read algebra shape differs")?;
    let result = build_v3(request, binding_digest)?;
    let output = weft_core::mutation::canonical(&result);
    if output.len() > 8_388_608 {
        return Err("generated read algebra artifact capacity exceeded");
    }
    Ok(output)
}

fn build_v4(mut request: Request, binding_digest: String) -> Result<Value> {
    if request.version != "weft.generated-read-algebra-request/4" {
        return Err("generated read algebra v4 request profile differs");
    }
    request.version = "weft.generated-read-algebra-request/3".into();
    let mut output = build_v3(request.clone(), binding_digest)?;
    let admitted = weft_core::raw_read::admit_generated_read(
        request.target.clone(),
        request.binding.clone(),
        "SELECT 1".to_owned(),
        vec![],
        1000,
    )?;
    let roots = output
        .get_mut("roots")
        .and_then(Value::as_object_mut)
        .ok_or("generated read v4 roots absent")?;
    for table in &request.binding.tables {
        let record = admitted.original_record(&table.logical)?;
        if record.identity.module != request.module {
            continue;
        }
        let root_key = format!(
            "{}/{}",
            identity_component(&request.module),
            identity_component(&record.identity.element)
        );
        let root = roots
            .get_mut(&root_key)
            .ok_or("generated read v4 root absent")?;
        let primary_key = root["primaryKey"]
            .as_array()
            .ok_or("generated read v4 primary Key absent")?;
        if primary_key.is_empty() || primary_key.len() > 16 {
            return Err("generated read v4 primary Key outside profile bounds");
        }
        let primary_key_id = root["primaryKeyId"]
            .as_str()
            .filter(|value| !value.is_empty())
            .ok_or("generated read v4 primary Key ID absent")?
            .to_owned();
        let root_from = root["fromSql"]
            .as_str()
            .ok_or("generated read v4 source relation absent")?;
        let relation = root_from
            .strip_prefix("FROM ")
            .filter(|value| value.ends_with(" AS \"r\""))
            .ok_or("generated read v4 source relation alias differs")?;

        let mut key_fields = Vec::with_capacity(primary_key.len());
        let mut tuple_slots = vec!["%s::pg_catalog.int8".to_owned()];
        let mut key_columns = Vec::with_capacity(primary_key.len());
        let mut join_terms = Vec::with_capacity(primary_key.len());
        for (index, key_component) in primary_key.iter().enumerate() {
            let field_id = key_component["fieldId"]
                .as_str()
                .ok_or("generated read v4 Key Field ID absent")?;
            let predicate_id = key_component["predicateId"]
                .as_str()
                .ok_or("generated read v4 Key predicate ID absent")?;
            let predicate = root["predicates"]
                .get(predicate_id)
                .ok_or("generated read v4 Key predicate absent")?;
            if predicate["fieldId"] != field_id
                || predicate["operator"] != "eq"
                || predicate["arity"] != 1
            {
                return Err("generated read v4 Key predicate meaning differs");
            }
            let codec = predicate
                .get("codec")
                .filter(|codec| codec["profile"] == "weft.query-codec-instructions/1")
                .cloned()
                .ok_or("generated read v4 Key codec absent")?;
            let source_field = root["fields"]
                .get(field_id)
                .and_then(|field| field.get("sourceField"))
                .ok_or("generated read v4 Key source Field absent")?;
            let mut key_column = None;
            for column in &table.columns {
                let reference = admitted.original_field_ref(&table.logical, &column.logical)?;
                if field_key(&reference)? == field_id {
                    key_column = Some(column);
                    break;
                }
            }
            let column = key_column.ok_or("generated read v4 Key column mapping absent")?;
            if column.nullable {
                return Err("generated read v4 Key column is nullable");
            }
            let source_element = source_field["element"]
                .as_str()
                .ok_or("generated read v4 Key source element absent")?;
            let expected_reference =
                admitted.original_field_ref(&table.logical, &column.logical)?;
            let expected_element = expected_reference["element"]
                .as_str()
                .ok_or("generated read v4 mapped Field element absent")?;
            if source_field["module"] != expected_reference["module"]
                || source_element != expected_element
            {
                return Err("generated read v4 Key Field mapping differs");
            }
            let cast = get_many_key_cast(&column.carrier)?;
            tuple_slots.push(format!("%s::pg_catalog.{cast}"));
            key_columns.push(format!("\"__weft_key_{index}\""));
            let physical = format!("\"r\".\"{}\"", column.native.replace('"', "\"\""));
            let logical_key = format!("k.\"__weft_key_{index}\"");
            let (left, right) = if column.carrier == weft_core::raw_read::Carrier::String {
                (
                    format!("{physical} COLLATE pg_catalog.\"C\""),
                    format!("{logical_key} COLLATE pg_catalog.\"C\""),
                )
            } else {
                (physical, logical_key)
            };
            join_terms.push(dbapi_sql(&format!("{left} = {right}"), &[])?);
            key_fields.push(json!({"fieldId":field_id,"codec":codec}));
        }
        let key_tuple_sql = format!("({})", tuple_slots.join(", "));
        let key_rows_columns = std::iter::once("\"__weft_ordinal\"".to_owned())
            .chain(key_columns)
            .collect::<Vec<_>>()
            .join(", ");
        let present_column = table
            .key
            .first()
            .and_then(|logical| {
                table
                    .columns
                    .iter()
                    .find(|column| column.logical == *logical)
            })
            .ok_or("generated read v4 presence Key column absent")?;
        let present_sql = dbapi_sql(
            &format!(
                "(\"r\".\"{}\" IS NOT NULL)",
                present_column.native.replace('"', "\"\"")
            ),
            &[],
        )?;
        let template = format!(
            "WITH \"__weft_key_rows\" ({key_rows_columns}) AS (VALUES {{{{KEY_ROWS}}}}) SELECT k.\"__weft_ordinal\" AS \"__weft_ordinal\", {present_sql} AS \"__weft_present\"{{{{SELECT_COLUMNS}}}} FROM \"__weft_key_rows\" AS k LEFT JOIN {relation} ON {} ORDER BY k.\"__weft_ordinal\"",
            join_terms.join(" AND ")
        );
        if std::iter::once(relation)
            .chain(std::iter::once(present_sql.as_str()))
            .chain(join_terms.iter().map(String::as_str))
            .any(|fragment| fragment.contains("{{KEY_ROWS}}") || fragment.contains("{{SELECT_COLUMNS}}"))
        {
            return Err("generated read v4 physical identifier collides with a template slot");
        }
        if template.matches("{{KEY_ROWS}}").count() != 1
            || template.matches("{{SELECT_COLUMNS}}").count() != 1
        {
            return Err("generated read v4 source template slot count differs");
        }
        let template_sha256 = weft_core::json::sha256(template.as_bytes());
        let output_object = root
            .as_object_mut()
            .ok_or("generated read v4 root object absent")?;
        output_object.insert(
            "getMany".into(),
            json!({
                "profile":"weft.generated-key-batch/1",
                "maxKeys":100,
                "keyId":primary_key_id,
                "keyFields":key_fields,
                "keyTupleSql":key_tuple_sql,
                "sourceSqlTemplate":template,
                "sourceTemplateSha256":template_sha256,
                "ordinalOutput":{"name":"__weft_ordinal","carrier":"integer","nullable":false,"nativeType":{"kind":"int8"},"oid":20},
                "presentOutput":{"name":"__weft_present","carrier":"boolean","nullable":false,"nativeType":{"kind":"boolean"},"oid":16}
            }),
        );
    }
    output["version"] = json!("weft.generated-read-algebra/4");
    output["readiness"] = json!("weft.generated-read-executable/4");
    Ok(output)
}

pub fn prepare_generated_read_algebra_v4_json(raw: &str) -> Result<String> {
    if raw.len() > 8_388_608 {
        return Err("generated read algebra request capacity exceeded");
    }
    let value =
        weft_core::json::checked_json(raw).map_err(|_| "generated read algebra JSON invalid")?;
    let binding = value
        .get("binding")
        .ok_or("generated read algebra binding absent")?;
    let binding_digest =
        weft_core::json::sha256(weft_core::mutation::canonical(binding).as_bytes());
    let request: Request =
        serde_json::from_value(value).map_err(|_| "generated read algebra shape differs")?;
    let result = build_v4(request, binding_digest)?;
    let output = weft_core::mutation::canonical(&result);
    if output.len() > 8_388_608 {
        return Err("generated read algebra artifact capacity exceeded");
    }
    Ok(output)
}

fn build_v5(request: Request, binding_digest: String, require_eligible: bool) -> Result<Value> {
    build_v5_internal(request, binding_digest, require_eligible, false)
}

fn build_v5_internal(mut request: Request, binding_digest: String, require_eligible: bool, resolve_authored_names: bool) -> Result<Value> {
    if request.version != "weft.generated-read-algebra-request/5" {
        return Err("generated read algebra v5 request profile differs");
    }
    request.version = "weft.generated-read-algebra-request/4".into();
    let mut output = build_v4(request.clone(), binding_digest)?;
    let admitted = weft_core::raw_read::admit_generated_read(
        request.target.clone(),
        request.binding.clone(),
        "SELECT 1".to_owned(),
        vec![],
        1000,
    )?;
    let roots = output
        .get_mut("roots")
        .and_then(Value::as_object_mut)
        .ok_or("generated read v5 roots absent")?;
    let mut eligible_roots = 0usize;
    for table in &request.binding.tables {
        let record = admitted.original_record(&table.logical)?;
        if record.identity.module != request.module {
            continue;
        }
        let root_key = format!(
            "{}/{}",
            identity_component(&request.module),
            identity_component(&record.identity.element)
        );
        let root = roots
            .get_mut(&root_key)
            .ok_or("generated read v5 root absent")?;
        let Some(primary_key) = root["primaryKey"]
            .as_array()
            .filter(|items| items.len() == 1)
            .cloned()
        else {
            mark_forward_connection_unsupported(root)?;
            continue;
        };
        let key_id = root["primaryKeyId"]
            .as_str()
            .filter(|value| !value.is_empty())
            .ok_or("generated forward connection requires the primary Key")?
            .to_owned();
        let key_field_id = primary_key[0]["fieldId"]
            .as_str()
            .ok_or("generated forward connection Key Field absent")?
            .to_owned();
        let key_predicate_id = primary_key[0]["predicateId"]
            .as_str()
            .ok_or("generated forward connection Key predicate absent")?;
        let key_codec = root["predicates"][key_predicate_id]
            .get("codec")
            .filter(|codec| codec["profile"] == "weft.query-codec-instructions/1")
            .cloned()
            .ok_or("generated forward connection Key codec absent")?;
        let key_field = root["fields"]
            .get(&key_field_id)
            .ok_or("generated forward connection Key field absent")?;
        let mut key_output = key_field
            .get("output")
            .cloned()
            .ok_or("generated forward connection Key output absent")?;
        let table_column = table
            .key
            .first()
            .and_then(|logical| table.columns.iter().find(|column| column.logical == *logical))
            .ok_or("generated forward connection Key column absent")?;
        let source_module = admitted.original_document()["modules"]
            .as_array()
            .and_then(|modules| modules.iter().find(|module| module["id"] == request.module))
            .ok_or("generated forward connection source module absent")?;
        let source_tables = source_module["extensions"]["umf.tables"]["tables"]
            .as_array()
            .ok_or("generated forward connection source tables absent")?;
        let expected_record = json!({"module":request.module,"element":record.identity.element});
        let matching_tables = source_tables
            .iter()
            .filter(|source_table| source_table["record"] == expected_record)
            .collect::<Vec<_>>();
        if matching_tables.len() != 1 {
            return Err("generated forward connection source table mapping is ambiguous");
        }
        let source_table = matching_tables[0];
        let expected_field = admitted.original_field_ref(&table.logical, &table_column.logical)?;
        let matching_columns = source_table["columns"]
            .as_array()
            .ok_or("generated forward connection source columns absent")?
            .iter()
            .filter(|column| column["field"] == expected_field)
            .collect::<Vec<_>>();
        if matching_columns.len() != 1 || matching_columns[0]["name"] != table_column.native {
            return Err("generated forward connection source Key mapping differs");
        }
        let native_type = matching_columns[0]
            .get("nativeType")
            .cloned()
            .ok_or("generated forward connection Key native type absent")?;
        let Ok((native_carrier, expected_oid)) = native_type_carrier_oid(&native_type) else {
            mark_forward_connection_unsupported(root)?;
            continue;
        };
        if native_carrier != "string" || native_type["kind"] == "char" {
            mark_forward_connection_unsupported(root)?;
            continue;
        }
        if key_output["carrier"] != "string" || key_output["nullable"] != false
        {
            mark_forward_connection_unsupported(root)?;
            continue;
        }
        if table_column.carrier != weft_core::raw_read::Carrier::String || table_column.nullable {
            mark_forward_connection_unsupported(root)?;
            continue;
        }
        key_output["name"] = json!("__weft_connection_key");
        key_output["nativeType"] = native_type;
        key_output["oid"] = json!(expected_oid);
        let mut field_outputs = Vec::new();
        let root_fields = root["fields"]
            .as_object()
            .ok_or("generated forward connection fields absent")?;
        let reserved_slots = ["{{FILTER}}", "{{SELECT_COLUMNS}}", "{{SOURCE_SQL}}", "{{RETURN_COLUMNS}}"];
        for field in root_fields.values() {
            let fragment = field["selectSql"]
                .as_str()
                .ok_or("generated forward connection field SQL absent")?;
            if reserved_slots.iter().any(|slot| fragment.contains(slot)) {
                return Err("generated forward connection field collides with a template slot");
            }
        }
        for predicate in root["predicates"]
            .as_object()
            .ok_or("generated forward connection predicates absent")?
            .values()
        {
            let fragment = predicate["sql"]
                .as_str()
                .ok_or("generated forward connection predicate SQL absent")?;
            if reserved_slots.iter().any(|slot| fragment.contains(slot)) {
                return Err("generated forward connection predicate collides with a template slot");
            }
        }
        let mut unsupported_field_type = false;
        for (field_id, field) in root_fields {
            let source_field = field
                .get("sourceField")
                .ok_or("generated forward connection source Field absent")?;
            let source_field_id = source_field["element"]
                .as_str()
                .filter(|_| source_field["module"] == request.module)
                .ok_or("generated forward connection cross-module Field unsupported")?;
            let (mapped_column, source_field_ref) = if resolve_authored_names {
                let matches = table.columns.iter().filter_map(|column| {
                    admitted.original_field_ref(&table.logical, &column.logical).ok()
                        .filter(|reference| field_key(reference).ok().as_deref() == Some(field_id))
                        .map(|reference| (column, reference))
                }).collect::<Vec<_>>();
                if matches.len() != 1 { return Err("generated forward connection Field column absent"); }
                matches.into_iter().next().unwrap()
            } else {
                let column = table.columns.iter().find(|column| column.logical == source_field_id)
                    .ok_or("generated forward connection Field column absent")?;
                (column, admitted.original_field_ref(&table.logical, source_field_id)?)
            };
            let native_columns = source_table["columns"]
                .as_array()
                .ok_or("generated forward connection source columns absent")?;
            let matching_native_columns = native_columns
                .iter()
                .filter(|column| column["field"] == source_field_ref)
                .collect::<Vec<_>>();
            if matching_native_columns.len() != 1
                || matching_native_columns[0]["name"] != mapped_column.native
            {
                return Err("generated forward connection Field mapping differs");
            }
            let field_native_type = matching_native_columns[0]
                .get("nativeType")
                .cloned()
                .ok_or("generated forward connection Field native type absent")?;
            let Ok((field_carrier, field_oid)) = native_type_carrier_oid(&field_native_type) else {
                unsupported_field_type = true;
                break;
            };
            let mut output = field
                .get("output")
                .cloned()
                .ok_or("generated forward connection Field output absent")?;
            if output["carrier"] != field_carrier
                || output["nullable"] != mapped_column.nullable
            {
                return Err("generated forward connection Field type or nullability differs");
            }
            let output_object = output
                .as_object_mut()
                .ok_or("generated forward connection Field output shape differs")?;
            output_object.insert("nativeType".into(), field_native_type);
            output_object.insert("oid".into(), json!(field_oid));
            field_outputs.push(json!({"fieldId":field_id,"output":output}));
        }
        if unsupported_field_type {
            mark_forward_connection_unsupported(root)?;
            continue;
        }
        let root_from = root["fromSql"]
            .as_str()
            .ok_or("generated forward connection source relation absent")?;
        let from_fragment = root_from
            .strip_prefix("FROM ")
            .filter(|value| value.ends_with(" AS \"r\""))
            .ok_or("generated forward connection source relation differs")?;
        let from_fragment = format!("FROM {}", from_fragment.replace("%%", "%"));
        let physical_key = format!("\"r\".\"{}\"", table_column.native.replace('"', "\"\""));
        let key_projection =
            format!("{physical_key} COLLATE pg_catalog.\"C\" AS \"__weft_connection_key\"");
        let guard = root["nativeReadGuard"]["template"]["sql"]
            .as_str()
            .filter(|_| {
                root["nativeReadGuard"]["profile"] == "pg18-composed-native-frame/1"
                    && root["nativeReadGuard"]["maxNativeBytes"] == 8_388_608
            })
            .ok_or("generated forward connection native byte guard absent")?;
        if guard.matches("{{SOURCE_SQL}}").count() != 1
            || guard.matches("{{RETURN_COLUMNS}}").count() != 1
        {
            return Err("generated forward connection native byte guard template differs");
        }
        let forward_source = format!(
            "SELECT ROW_NUMBER() OVER (ORDER BY {physical_key} COLLATE pg_catalog.\"C\" ASC) AS \"__weft_ordinal\", {key_projection}{{{{SELECT_COLUMNS}}}} {from_fragment} WHERE ({{{{FILTER}}}}) AND ($1::pg_catalog.text IS NULL OR {physical_key} COLLATE pg_catalog.\"C\" > $2::pg_catalog.text COLLATE pg_catalog.\"C\") ORDER BY {physical_key} COLLATE pg_catalog.\"C\" ASC LIMIT $3::pg_catalog.int8"
        );
        let forward_source_template = guard
            .replace("{{SOURCE_SQL}}", &forward_source)
            .replace("{{RETURN_COLUMNS}}", "f.*");
        let prior_source = format!(
            "SELECT 1::pg_catalog.int8 AS \"__weft_ordinal\", EXISTS (SELECT 1 {from_fragment} WHERE ({{{{FILTER}}}}) AND ($1::pg_catalog.text IS NOT NULL AND {physical_key} COLLATE pg_catalog.\"C\" <= $2::pg_catalog.text COLLATE pg_catalog.\"C\") LIMIT 1) AS \"__weft_previous\""
        );
        let prior_source_template = guard
            .replace("{{SOURCE_SQL}}", &prior_source)
            .replace("{{RETURN_COLUMNS}}", "f.\"__weft_previous\"");
        let (forward_template, forward_dbapi_order) =
            dbapi_sql_with_order(&forward_source_template, &["after", "after", "limit"])?;
        let (prior_template, prior_dbapi_order) =
            dbapi_sql_with_order(&prior_source_template, &["after", "after"])?;
        for (label, template, slots) in [
            ("forward", forward_template.as_str(), 1usize),
            ("previous", prior_template.as_str(), 1usize),
        ] {
            if template.matches("{{FILTER}}").count() != slots
                || (label == "forward" && template.matches("{{SELECT_COLUMNS}}").count() != 1)
                || (label == "previous" && template.contains("{{SELECT_COLUMNS}}"))
            {
                return Err("generated forward connection template slots differ");
            }
        }
        if forward_template.contains("{{AFTER}}") || prior_template.contains("{{AFTER}}") {
            return Err("generated forward connection contains an unknown template slot");
        }
        let forward_sha256 = weft_core::json::sha256(forward_template.as_bytes());
        let previous_sha256 = weft_core::json::sha256(prior_template.as_bytes());
        root.as_object_mut()
            .ok_or("generated read v5 root object absent")?
            .insert(
                "forwardConnection".into(),
                json!({
                    "profile":"weft.generated-forward-connection/1",
                    "maxFirst":100,
                    "lookaheadRows":1,
                    "zeroFirstProbeRows":1,
                    "keyId":key_id,
                    "nativeCollation":"pg_catalog.\"C\"",
                    "nativeCollationPrecondition":"umf.generated-table-collation-c/1",
                    "key":{"fieldId":key_field_id,"codec":key_codec,"output":key_output,"direction":"asc","nulls":"last","collation":"pg_catalog.\"C\""},
                    "fieldOutputs":field_outputs,
                    "forward":{"sourceSqlTemplate":forward_source_template,"sourceTemplateSha256":weft_core::json::sha256(forward_source_template.as_bytes()),"sqlTemplate":forward_template,"templateSha256":forward_sha256,"filterParameterGroup":"filterValues","parameterOrder":["filterValues","after","after","limit"],"dbapiParameterOrder":forward_dbapi_order,"keyOutput":key_output},
                    "previous":{"sourceSqlTemplate":prior_source_template,"sourceTemplateSha256":weft_core::json::sha256(prior_source_template.as_bytes()),"sqlTemplate":prior_template,"templateSha256":previous_sha256,"filterParameterGroup":"filterValues","parameterOrder":["filterValues","after","after"],"dbapiParameterOrder":prior_dbapi_order,"output":{"name":"__weft_previous","carrier":"boolean","nullable":false,"nativeType":{"kind":"boolean"},"oid":16}}
                }),
            );
        root.as_object_mut()
            .ok_or("generated read v5 root object absent")?
            .insert(
                "forwardConnectionSupport".into(),
                json!({"profile":"weft.generated-forward-connection/1","status":"eligible"}),
            );
        eligible_roots += 1;
    }
    if eligible_roots == 0 && require_eligible {
        return Err("generated read v5 has no eligible forward connection roots");
    }
    output["version"] = json!("weft.generated-read-algebra/5");
    output["readiness"] = json!("weft.generated-read-executable/5");
    output["executorAbi"] = json!({"major":1,"minor":5});
    Ok(output)
}

pub fn prepare_generated_read_algebra_v5_json(raw: &str) -> Result<String> {
    if raw.len() > 8_388_608 {
        return Err("generated read algebra request capacity exceeded");
    }
    let value =
        weft_core::json::checked_json(raw).map_err(|_| "generated read algebra JSON invalid")?;
    let binding = value
        .get("binding")
        .ok_or("generated read algebra binding absent")?;
    let binding_digest =
        weft_core::json::sha256(weft_core::mutation::canonical(binding).as_bytes());
    let request: Request =
        serde_json::from_value(value).map_err(|_| "generated read algebra shape differs")?;
    let result = build_v5(request, binding_digest, true)?;
    let output = weft_core::mutation::canonical(&result);
    if output.len() > 8_388_608 {
        return Err("generated read algebra artifact capacity exceeded");
    }
    Ok(output)
}

// Ordinary typed Page issuance is deliberately a separate family from the
// Relay connection added in v5. The descriptor carries only compiler-owned
// fragments; runtimes bind the requested size and continuation value.
fn build_v6(request: Request, binding_digest: String, require_eligible: bool) -> Result<Value> {
    build_v6_internal(request, binding_digest, require_eligible, false)
}

fn build_v6_internal(mut request: Request, binding_digest: String, require_eligible: bool, resolve_authored_names: bool) -> Result<Value> {
    if request.version != "weft.generated-read-algebra-request/6" {
        return Err("generated read algebra v6 request profile differs");
    }
    request.version = "weft.generated-read-algebra-request/4".into();
    let mut output = build_v4(request.clone(), binding_digest.clone())?;
    let admitted = weft_core::raw_read::admit_generated_read(
        request.target.clone(), request.binding.clone(), "SELECT 1".to_owned(), vec![], 1000,
    )?;
    let source_context = output["source"].clone();
    let target_context = output["target"].clone();
    let roots = output["roots"].as_object_mut().ok_or("generated read v6 roots absent")?;
    let mut eligible = 0usize;
    for table in &request.binding.tables {
        let record = admitted.original_record(&table.logical)?;
        if record.identity.module != request.module { continue; }
        let root_key = format!("{}/{}", identity_component(&request.module), identity_component(&record.identity.element));
        let Some(root) = roots.get_mut(&root_key) else { continue };
        let unsupported = |reason: &str| json!({"profile":"weft.generated-ordinary-page/1","status":"unsupported","reason":reason});
        root["ordinaryPage"] = unsupported("root-not-yet-qualified");
        let Some(pk) = root["primaryKey"].as_array().filter(|items| items.len() == 1) else {
            root["ordinaryPage"] = unsupported("single-required-string-primary-key-required");
            continue;
        };
        let key_id = root["primaryKeyId"].as_str().filter(|value| !value.is_empty()).ok_or("generated ordinary page primary Key ID absent")?.to_owned();
        let Some(pk_field_id) = pk[0]["fieldId"].as_str() else {
            root["ordinaryPage"] = unsupported("primary-key-field-identity-absent");
            continue;
        };
        let Some(pk_predicate_id) = pk[0]["predicateId"].as_str() else {
            root["ordinaryPage"] = unsupported("primary-key-codec-absent");
            continue;
        };
        let Some(column) = table.key.first().and_then(|logical| table.columns.iter().find(|column| column.logical == *logical)) else {
            root["ordinaryPage"] = unsupported("primary-key-binding-absent");
            continue;
        };
        let predicate = root["predicates"].get(pk_predicate_id).ok_or("generated ordinary page Key predicate absent")?;
        if predicate["fieldId"] != pk_field_id || predicate["operator"] != "eq" || predicate["arity"] != 1 {
            root["ordinaryPage"] = unsupported("primary-key-predicate-meaning-differs"); continue;
        }
        let codec = predicate.get("codec").filter(|codec| codec["profile"] == "weft.query-codec-instructions/1").cloned().ok_or("generated ordinary page Key codec absent")?;
        let field = root["fields"].get(pk_field_id).ok_or("generated ordinary page Key Field absent")?;
        let expected_field_ref = admitted.original_field_ref(&table.logical, &column.logical)?;
        if field["sourceField"] != json!({"module":expected_field_ref["module"],"element":expected_field_ref["element"]}) {
            return Err("generated ordinary page Key source Field differs");
        }
        let authored_primary = record.value["keys"].as_array().and_then(|keys| keys.iter().find(|key| key["id"] == key_id && key["primary"] == true)).ok_or("generated ordinary page authored primary Key absent")?;
        let authored_fields = authored_primary["fields"].as_array().filter(|fields| fields.len() == 1).ok_or("generated ordinary page authored primary Key arity differs")?;
        if authored_fields[0] != expected_field_ref { return Err("generated ordinary page authored Key Field differs"); }
        let source_codec = weft_core::table_query::generated_codec_for_member(admitted.original_document(), &expected_field_ref).map_err(|_| "generated ordinary page source codec unsupported")?;
        if source_codec != codec { return Err("generated ordinary page predicate codec differs from source Field"); }
        let mut key_output = field["output"].clone();
        if table.key.len() != 1 || table.key[0] != column.logical || column.carrier != weft_core::raw_read::Carrier::String || column.nullable || key_output["carrier"] != "string" || key_output["nullable"] != false {
            root["ordinaryPage"] = unsupported("required-string-primary-key-required"); continue;
        }
        let source_module = admitted.original_document()["modules"].as_array().and_then(|items| items.iter().find(|item| item["id"] == request.module)).ok_or("generated ordinary page source module absent")?;
        let source_tables = source_module["extensions"]["umf.tables"]["tables"].as_array().ok_or("generated ordinary page source tables absent")?;
        let source_record = json!({"module":request.module,"element":record.identity.element});
        let matching_tables = source_tables.iter().filter(|item| item["record"] == source_record).collect::<Vec<_>>();
        if matching_tables.len() != 1 { return Err("generated ordinary page source table ambiguous"); }
        let source_field = if resolve_authored_names { expected_field_ref.clone() } else { json!({"module":request.module,"element":column.logical}) };
        let matching_columns = matching_tables[0]["columns"].as_array().ok_or("generated ordinary page source columns absent")?.iter().filter(|item| item["field"] == source_field).collect::<Vec<_>>();
        if matching_columns.len() != 1 { return Err("generated ordinary page Key source mapping differs"); }
        if matching_columns[0]["name"] != column.native { return Err("generated ordinary page Key native column differs"); }
        let native_type = matching_columns[0].get("nativeType").ok_or("generated ordinary page Key native type absent")?.clone();
        let Ok((native_carrier, expected_oid)) = native_type_carrier_oid(&native_type) else { root["ordinaryPage"] = unsupported("unsupported-primary-key-native-type"); continue; };
        if native_carrier != "string" || !ordinary_page_key_type_supported(&native_type, column) { root["ordinaryPage"] = unsupported("required-string-primary-key-required"); continue; }
        key_output["name"] = json!("__weft_page_key");
        key_output["nativeType"] = native_type;
        key_output["oid"] = json!(expected_oid);
        // Page selection exposes public carrier metadata joined to the exact
        // source Field, authored table column and request binding.
        let mut field_outputs = Vec::new();
        for (field_id, public_field) in root["fields"].as_object().ok_or("generated ordinary page fields absent")? {
            let source_ref = &public_field["sourceField"];
            let (column, exact_ref) = if resolve_authored_names {
                let matches = table.columns.iter().filter_map(|item| admitted.original_field_ref(&table.logical, &item.logical).ok().filter(|reference| reference == source_ref).map(|reference| (item, reference))).collect::<Vec<_>>();
                if matches.len() != 1 { return Err("generated Page Field binding ambiguous"); }
                matches.into_iter().next().unwrap()
            } else {
                let logical = source_ref["element"].as_str().ok_or("generated Page Field identity absent")?;
                let exact_ref = admitted.original_field_ref(&table.logical, logical)?;
                let columns = table.columns.iter().filter(|item| item.logical == logical).collect::<Vec<_>>();
                if columns.len() != 1 { return Err("generated Page Field binding ambiguous"); }
                (columns[0], exact_ref)
            };
            let mappings = matching_tables[0]["columns"].as_array().ok_or("generated Page source columns absent")?.iter().filter(|item| item["field"] == exact_ref).collect::<Vec<_>>();
            if mappings.len() != 1 { return Err("generated Page Field source mapping ambiguous"); }
            field_outputs.push(ordinary_page_field_output(field_id, public_field, &exact_ref, column, mappings[0])?);
        }
        let from = root["fromSql"].as_str().ok_or("generated ordinary page FROM absent")?;
        let relation = from.strip_prefix("FROM ").filter(|v| v.ends_with(" AS \"r\"")).ok_or("generated ordinary page relation shape differs")?;
        let relation = relation.replace("%%", "%");
        let physical_key = format!("\"r\".\"{}\"", column.native.replace('"', "\"\""));
        let guard = root["nativeReadGuard"]["template"]["sql"].as_str().filter(|_| root["nativeReadGuard"]["profile"] == "pg18-composed-native-frame/1" && root["nativeReadGuard"]["maxNativeBytes"] == 8_388_608).ok_or("generated ordinary page byte guard absent")?;
        if guard.matches("{{SOURCE_SQL}}").count()!=1 || guard.matches("{{RETURN_COLUMNS}}").count()!=1 { return Err("generated ordinary page guard template differs"); }
        let reserved_slots = ["{{SOURCE_SQL}}", "{{RETURN_COLUMNS}}", "{{FILTER}}", "{{PUBLIC_PROJECTION}}"];
        if reserved_slots.iter().any(|slot| relation.contains(slot) || physical_key.contains(slot)) { return Err("generated ordinary page physical identifier collides with template slot"); }
        for field in root["fields"].as_object().ok_or("generated ordinary page fields absent")?.values() {
            let fragment = field["selectSql"].as_str().ok_or("generated ordinary page field SQL absent")?;
            if reserved_slots.iter().any(|slot| fragment.contains(slot)) { return Err("generated ordinary page field fragment collides with template slot"); }
        }
        let predicates = root["predicates"].as_object().ok_or("generated ordinary page predicates absent")?;
        let mut predicate_inputs = Vec::new();
        for (predicate_id, predicate) in predicates {
            let fragment = predicate["sql"].as_str().ok_or("generated ordinary page predicate SQL absent")?;
            if reserved_slots.iter().any(|slot| fragment.contains(slot)) { return Err("generated ordinary page predicate fragment collides with template slot"); }
            if predicate["arity"] == 1 {
                let input_codec = predicate.get("codec").filter(|codec| codec["profile"] == "weft.query-codec-instructions/1").ok_or("generated ordinary page predicate input codec absent")?;
                predicate_inputs.push(json!({"id":predicate_id,"fieldId":predicate["fieldId"],"operator":predicate["operator"],"arity":1,"codec":input_codec}));
            }
        }
        let native_schema = request.binding.schema.as_deref().filter(|schema| !schema.is_empty()).ok_or("generated ordinary page requires qualified schema binding")?;
        let native_table = table.native.as_str();
        let native_column = column.native.as_str();
        if [native_schema,native_table,native_column].iter().any(|part| reserved_slots.iter().any(|slot| part.contains(slot))) { return Err("generated ordinary page native identifier collides with template slot"); }
        let source_sql = format!("SELECT ROW_NUMBER() OVER (ORDER BY {physical_key} COLLATE pg_catalog.\"C\" ASC) AS \"__weft_ordinal\", {physical_key} COLLATE pg_catalog.\"C\" AS \"__weft_page_key\"{{{{PUBLIC_PROJECTION}}}} FROM {relation} WHERE ({{{{FILTER}}}}) AND ($1::pg_catalog.text IS NULL OR {physical_key} COLLATE pg_catalog.\"C\" > $2::pg_catalog.text COLLATE pg_catalog.\"C\") ORDER BY {physical_key} COLLATE pg_catalog.\"C\" ASC LIMIT ($3::pg_catalog.int8 + 1)");
        let guarded = guard.replace("{{SOURCE_SQL}}", &source_sql).replace("{{RETURN_COLUMNS}}", "f.*");
        if source_sql.matches("{{PUBLIC_PROJECTION}}").count()!=1 || source_sql.matches("{{FILTER}}").count()!=1 || guarded.matches("{{SOURCE_SQL}}").count()!=0 || guarded.matches("{{RETURN_COLUMNS}}").count()!=0 || guarded.matches("{{FILTER}}").count()!=1 || guarded.matches("{{PUBLIC_PROJECTION}}").count()!=1 { return Err("generated ordinary page SQL slot count differs"); }
        let (sql, order) = dbapi_sql_with_order(&guarded, &["after","after","size"])?;
        let precondition_source = "SELECT cn.nspname, co.collname, co.collisdeterministic FROM pg_catalog.pg_attribute AS a JOIN pg_catalog.pg_class AS c ON c.oid = a.attrelid JOIN pg_catalog.pg_namespace AS n ON n.oid = c.relnamespace JOIN pg_catalog.pg_collation AS co ON co.oid = a.attcollation JOIN pg_catalog.pg_namespace AS cn ON cn.oid = co.collnamespace WHERE n.nspname = $1::pg_catalog.text AND c.relname = $2::pg_catalog.text AND a.attname = $3::pg_catalog.text AND a.attnum > 0 AND NOT a.attisdropped";
        let (precondition_sql, precondition_order) = dbapi_sql_with_order(precondition_source, &["schema","table","column"])?;
        let page_template_sha = weft_core::json::sha256(sql.as_bytes());
        let mut descriptor = json!({
            "profile":"weft.generated-ordinary-page/1","status":"eligible","maxSize":1000,"snapshot":{"supported":false,"trueOutcome":"UNSUPPORTED"},
            "key":{"id":key_id,"fieldId":pk_field_id,"codec":codec,"output":key_output,"direction":"asc","nulls":"last","collation":"pg_catalog.\"C\""},
            "size":{"slot":"size","carrier":"int64","minimum":1,"maximum":1000,"identity":true},
            "nativeCollationPrecondition":{"profile":"weft.generated-table-collation-c/1","required":true,"sourceSqlTemplate":precondition_source,"sqlTemplate":precondition_sql,"parameterOrder":["schema","table","column"],"dbapiParameterOrder":precondition_order,"parameterSources":["binding.schema","binding.table.native","binding.keyColumn.native"],"expected":{"schema":"pg_catalog","name":"C","deterministic":true}},
            "page":{"sourceSqlTemplate":source_sql,"sqlTemplate":sql,"templateSha256":page_template_sha,"filterParameterGroup":"filterValues","projectionSlot":{"name":"PUBLIC_PROJECTION","emptyValue":"","nonemptyPrefix":", "},"parameterOrder":["filterValues","after","after","size"],"dbapiParameterOrder":order,"lookaheadRows":1,"serverByteGuard":true,"ordinalOutput":{"name":"__weft_ordinal","carrier":"integer","nullable":false,"nativeType":{"kind":"int8"},"oid":20},"hiddenKeyOutput":key_output,"fieldOutputs":field_outputs},
            "continuation":{"profile":"weft.generated-ordinary-page-continuation-identity/1","ownerCursor":null,"snapshotVersions":[],"exactFullPageYieldsContinuation":true,
                "fixed":{"sourceModelSha256":source_context["modelSha256"],"targetId":target_context["id"],"targetModelSha256":target_context["modelSha256"],"bindingDigest":target_context["bindingDigest"],"module":request.module,"record":record.identity.element,"keyId":key_id,"keyFieldId":pk_field_id,"issuedPageTemplateSha256":page_template_sha},
                "requestInputs":{"selectedFieldIds":{"allowed":root["fields"].as_object().ok_or("generated ordinary page fields absent")?.keys().cloned().collect::<Vec<_>>(),"ordered":true,"identity":true},"filterProgram":{"profile":"weft.generated-page-filter-program/1","identity":"canonical-exact-tree","predicateIdSet":"predicateIds.allowed","operators":["predicate","and","or","not"],"valueOrder":"depth-first-left-to-right","emptyExpression":null,"emptySql":"TRUE"},"predicateInputs":predicate_inputs,"predicateIds":{"allowed":root["predicates"].as_object().ok_or("generated ordinary page predicates absent")?.keys().cloned().collect::<Vec<_>>(),"ordered":true,"identity":true},"normalizedPredicateValues":{"occurrenceOrder":"depth-first-left-to-right-arity-one-predicate-leaves","exactCodecNormalization":true,"identity":true},"size":{"minimum":1,"maximum":1000,"exactInteger":true,"identity":true}}}
        });
        root["ordinaryPage"] = descriptor;
        // Bind the complete root algebra (including inherited select/predicate
        // SQL and all output metadata) without creating a recursive digest.
        let mut digest_root = root.clone();
        digest_root["ordinaryPage"]["continuation"]["fixed"].as_object_mut()
            .ok_or("generated Page continuation identity absent")?.remove("issuedProgramDigest");
        let issued_digest = weft_core::json::sha256(weft_core::mutation::canonical(&digest_root).as_bytes());
        root["ordinaryPage"]["continuation"]["fixed"]["issuedProgramDigest"] = json!(issued_digest);
        eligible += 1;
    }
    if eligible == 0 && require_eligible { return Err("generated read v6 has no eligible ordinary page roots"); }
    output["version"] = json!("weft.generated-read-algebra/6");
    output["readiness"] = json!("weft.generated-read-executable/6");
    output["executorAbi"] = json!({"major":1,"minor":6});
    Ok(output)
}

// Combined opt-in artifact: same independently issued /5 and /6 semantics,
// with only the Page root digest rebound over the complete combined root.
fn build_v7(request: Request, binding_digest: String) -> Result<Value> {
    build_v7_internal(request, binding_digest, false)
}

fn build_v7_query_names(request: Request, binding_digest: String) -> Result<Value> {
    build_v7_internal(request, binding_digest, true)
}

fn build_v7_internal(request: Request, binding_digest: String, resolve_authored_names: bool) -> Result<Value> {
    if request.version != "weft.generated-read-algebra-request/7" {
        return Err("generated read algebra v7 request profile differs");
    }
    let mut v5_request = request.clone();
    v5_request.version = "weft.generated-read-algebra-request/5".into();
    let mut v6_request = request.clone();
    v6_request.version = "weft.generated-read-algebra-request/6".into();
    let v5 = if resolve_authored_names { build_v5_internal(v5_request, binding_digest.clone(), false, true).map_err(|_| "generated read v8 inherited forward branch failed")? } else { build_v5_internal(v5_request, binding_digest.clone(), false, false)? };
    let v6 = if resolve_authored_names { build_v6_internal(v6_request, binding_digest, false, true).map_err(|_| "generated read v8 inherited Page branch failed")? } else { build_v6_internal(v6_request, binding_digest, false, false)? };
    if v5["source"] != v6["source"] || v5["target"] != v6["target"] {
        return Err("generated read v7 source or target identity differs");
    }
    let mut output = v6;
    let roots = output["roots"].as_object_mut().ok_or("generated read v7 roots absent")?;
    let v5_roots = v5["roots"].as_object().ok_or("generated read v7 connection roots absent")?;
    if roots.len() != v5_roots.len() { return Err("generated read v7 root sets differ"); }
    for (root_id, root) in roots.iter_mut() {
        let connection_root = v5_roots.get(root_id).ok_or("generated read v7 root identity differs")?;
        if root["module"] != connection_root["module"] || root["record"] != connection_root["record"]
            || root["fields"] != connection_root["fields"]
            || root["predicates"] != connection_root["predicates"]
            || root["primaryKeyId"] != connection_root["primaryKeyId"]
            || root["primaryKey"] != connection_root["primaryKey"]
        { return Err("generated read v7 shared source, Record, Key or Field identity differs"); }
        let page_eligible = root["ordinaryPage"]["status"] == "eligible";
        let forward_eligible = connection_root["forwardConnectionSupport"]["status"] == "eligible";
        let forward = connection_root.get("forwardConnection");
        if forward_eligible != forward.is_some() { return Err("generated read v7 connection branch differs"); }
        if page_eligible && forward_eligible {
            let page_key = &root["ordinaryPage"]["key"];
            let forward_key = &forward.unwrap()["key"];
            if page_key["id"] != connection_root["primaryKeyId"]
                || page_key["id"] != root["primaryKeyId"]
                || page_key["fieldId"] != forward_key["fieldId"]
                || page_key["codec"] != forward_key["codec"]
                || page_key["direction"] != forward_key["direction"]
                || page_key["nulls"] != forward_key["nulls"]
                || page_key["collation"] != forward_key["collation"]
                || page_key["output"]["nativeType"] != forward_key["output"]["nativeType"]
                || page_key["output"]["oid"] != forward_key["output"]["oid"]
                || page_key["output"]["carrier"] != forward_key["output"]["carrier"]
                || page_key["output"]["nullable"] != forward_key["output"]["nullable"]
            { return Err("generated read v7 shared Key codec, identity or native mapping differs"); }
            let page_fields = root["ordinaryPage"]["page"]["fieldOutputs"].as_array().ok_or("generated Page field outputs absent")?;
            let forward_fields = forward.unwrap()["fieldOutputs"].as_array().ok_or("generated connection field outputs absent")?;
            if page_fields.len() != forward_fields.len() { return Err("generated read v7 selectable Field sets differ"); }
            for page_field in page_fields {
                let field_id = page_field["fieldId"].as_str().ok_or("generated Page Field identity absent")?;
                let matches = forward_fields.iter().filter(|field| field["fieldId"] == field_id).collect::<Vec<_>>();
                if matches.len() != 1
                    || page_field["sourceField"] != root["fields"][field_id]["sourceField"]
                    || page_field["output"] != matches[0]["output"]
                    || page_field["output"]["nativeType"].is_null()
                    || page_field["output"]["oid"].is_null()
                    || page_field["nativeColumn"].as_str().is_none()
                { return Err("generated read v7 shared Field source or native output mapping differs"); }
            }
        }
        if let Some(forward) = forward { root["forwardConnection"] = forward.clone(); }
        root["forwardConnectionSupport"] = connection_root["forwardConnectionSupport"].clone();
        if root["ordinaryPage"]["status"] == "eligible" {
            let mut digest_root = root.clone();
            digest_root["ordinaryPage"]["continuation"]["fixed"].as_object_mut()
                .ok_or("generated Page continuation identity absent")?.remove("issuedProgramDigest");
            let digest = weft_core::json::sha256(weft_core::mutation::canonical(&digest_root).as_bytes());
            root["ordinaryPage"]["continuation"]["fixed"]["issuedProgramDigest"] = json!(digest);
        }
    }
    output["version"] = json!("weft.generated-read-algebra/7");
    output["readiness"] = json!("weft.generated-read-executable/7");
    output["executorAbi"] = json!({"major":1,"minor":7});
    Ok(output)
}

pub fn prepare_generated_read_algebra_v8_json(raw: &str) -> Result<String> {
    generated_join_v8::prepare_generated_read_algebra_v8_json(raw)
}

pub fn prepare_generated_read_algebra_v7_json(raw: &str) -> Result<String> {
    if raw.len() > 8_388_608 { return Err("generated read algebra request capacity exceeded"); }
    let value = weft_core::json::checked_json(raw).map_err(|_| "generated read algebra JSON invalid")?;
    let binding = value.get("binding").ok_or("generated read algebra binding absent")?;
    let digest = weft_core::json::sha256(weft_core::mutation::canonical(binding).as_bytes());
    let request: Request = serde_json::from_value(value).map_err(|_| "generated read algebra shape differs")?;
    let result = build_v7(request, digest)?;
    let output = weft_core::mutation::canonical(&result);
    if output.len() > 8_388_608 { return Err("generated read algebra artifact capacity exceeded"); }
    Ok(output)
}

pub fn prepare_generated_read_algebra_v6_json(raw: &str) -> Result<String> {
    if raw.len() > 8_388_608 { return Err("generated read algebra request capacity exceeded"); }
    let value = weft_core::json::checked_json(raw).map_err(|_| "generated read algebra JSON invalid")?;
    let binding = value.get("binding").ok_or("generated read algebra binding absent")?;
    let digest = weft_core::json::sha256(weft_core::mutation::canonical(binding).as_bytes());
    let request: Request = serde_json::from_value(value).map_err(|_| "generated read algebra shape differs")?;
    let result = build_v6(request, digest, true)?;
    let output = weft_core::mutation::canonical(&result);
    if output.len() > 8_388_608 { return Err("generated read algebra artifact capacity exceeded"); }
    Ok(output)
}

pub fn prepare_generated_read_algebra_json(raw: &str) -> Result<String> {
    if raw.len() > 8_388_608 {
        return Err("generated read algebra request capacity exceeded");
    }
    let value =
        weft_core::json::checked_json(raw).map_err(|_| "generated read algebra JSON invalid")?;
    let binding = value
        .get("binding")
        .ok_or("generated read algebra binding absent")?;
    let binding_digest =
        weft_core::json::sha256(weft_core::mutation::canonical(binding).as_bytes());
    let request: Request =
        serde_json::from_value(value).map_err(|_| "generated read algebra shape differs")?;
    let result = build(request, binding_digest)?;
    let output = weft_core::mutation::canonical(&result);
    if output.len() > 8_388_608 {
        return Err("generated read algebra artifact capacity exceeded");
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_with_access(version: &str, selected_key: &str, add_alternate: bool) -> Request {
        let mut template: Value = serde_json::from_str(include_str!(
            "../../../tests/raw-postgresql/table-lifecycle-read-template.json"
        ))
        .unwrap();
        let mut binding: Binding = serde_json::from_value(template["binding"].take()).unwrap();
        let mut document: Value = serde_json::from_str(include_str!(
            "../../../tests/raw-postgresql/table-operation-model.json"
        ))
        .unwrap();
        document["vocabularies"]["umf.access"] = json!({"version":version});
        let module = document["modules"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|module| module["id"] == "demo")
            .unwrap();
        let record = module["elements"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|element| element["id"] == "Note")
            .unwrap();
        if add_alternate {
            record["keys"].as_array_mut().unwrap().push(json!({
                "id":"alternate",
                "name":"alternate",
                "primary":false,
                "fields":[{"module":"demo","element":"owner"}]
            }));
        }
        let members = record["members"].as_array().unwrap().clone();
        let fields = members
            .iter()
            .map(|member| {
                json!({
                    "field":member,
                    "row":{"missing":false,"null":false},
                    "create":{"missing":false,"null":false},
                    "mutable":false,
                    "generated":false
                })
            })
            .collect::<Vec<_>>();
        let record_ref = json!({"module":"demo","element":"Note"});
        module["extensions"]["umf.access"] = json!({
            "views":[{"record":record_ref,"fields":fields,
                "selectedKey":{"module":"demo","element":"Note","key":selected_key}}],
            "columns":[],"search":[],"searchBindings":[]
        });
        let model = serde_json::to_string(&document).unwrap();
        binding.model_json = model.clone();
        binding.model_sha256 = weft_core::json::sha256(model.as_bytes());
        Request {
            version: "weft.generated-read-algebra-request/1".into(),
            target: "postgresql18".into(),
            binding,
            module: "demo".into(),
        }
    }

    fn issue(request: Request) -> Result<Value> {
        let binding_value = serde_json::to_value(&request.binding).unwrap();
        let binding_digest =
            weft_core::json::sha256(weft_core::mutation::canonical(&binding_value).as_bytes());
        build(request, binding_digest)
    }

    fn issue_v5(mut request: Request) -> Result<Value> {
        request.version = "weft.generated-read-algebra-request/5".into();
        let raw = weft_core::mutation::canonical(&json!({
            "version":request.version,"target":request.target,"binding":request.binding,"module":request.module
        }));
        let result = prepare_generated_read_algebra_v5_json(&raw)?;
        serde_json::from_str(&result).map_err(|_| "generated read v5 output JSON invalid")
    }

    fn issue_v6(mut request: Request) -> Result<Value> {
        request.version = "weft.generated-read-algebra-request/6".into();
        let raw = weft_core::mutation::canonical(&json!({
            "version":request.version,"target":request.target,"binding":request.binding,"module":request.module
        }));
        let result = prepare_generated_read_algebra_v6_json(&raw)?;
        serde_json::from_str(&result).map_err(|_| "generated read v6 output JSON invalid")
    }

    fn issue_v7(mut request: Request) -> Result<Value> {
        request.version = "weft.generated-read-algebra-request/7".into();
        let raw = weft_core::mutation::canonical(&json!({
            "version":request.version,"target":request.target,"binding":request.binding,"module":request.module
        }));
        let result = prepare_generated_read_algebra_v7_json(&raw)?;
        serde_json::from_str(&result).map_err(|_| "generated read v7 output JSON invalid")
    }

    #[test]
    fn v7_retains_an_inherited_read_when_both_extensions_refuse() {
        let mut request = request_with_mixed_connection_roots();
        let mut model: Value = serde_json::from_str(&request.binding.model_json).unwrap();
        let note = model["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|element| element["id"] == "Note").unwrap();
        note["keys"][0]["fields"] = json!([{"module":"demo","element":"id"},{"module":"demo","element":"owner"}]);
        request.binding.tables.iter_mut().find(|table| table.logical == vec!["demo".to_owned(), "Note".to_owned()]).unwrap().key = vec!["id".into(),"owner".into()];
        request.binding.model_json = weft_core::mutation::canonical(&model);
        request.binding.model_sha256 = weft_core::json::sha256(request.binding.model_json.as_bytes());
        let v7 = issue_v7(request.clone()).unwrap();
        for root in v7["roots"].as_object().unwrap().values() {
            assert_eq!(root["ordinaryPage"]["status"], "unsupported");
            assert_eq!(root["forwardConnectionSupport"]["status"], "unsupported");
            assert!(root["fields"].is_object(), "inherited /4 read algebra remains available");
            assert!(root.get("forwardConnection").is_none());
        }
        assert!(issue_v5(request.clone()).is_err(), "standalone /5 retains its global eligibility gate");
        assert!(issue_v6(request).is_err(), "standalone /6 retains its global eligibility gate");
    }

    #[test]
    fn v7_coissues_v5_and_v6_without_changing_their_payloads() {
        let request = request_with_access("0.1.0", "primary", false);
        let v5 = issue_v5(request.clone()).unwrap();
        let v6 = issue_v6(request.clone()).unwrap();
        let v7 = issue_v7(request).unwrap();
        assert_eq!(v7["version"], "weft.generated-read-algebra/7");
        assert_eq!(v7["executorAbi"], json!({"major":1,"minor":7}));
        let root = &v7["roots"]["demo/Note"];
        assert_eq!(root["forwardConnection"], v5["roots"]["demo/Note"]["forwardConnection"]);
        assert_eq!(root["forwardConnectionSupport"], v5["roots"]["demo/Note"]["forwardConnectionSupport"]);
        assert_eq!(root["ordinaryPage"]["page"], v6["roots"]["demo/Note"]["ordinaryPage"]["page"]);
        assert_eq!(root["ordinaryPage"]["key"], v6["roots"]["demo/Note"]["ordinaryPage"]["key"]);
        let mut preimage = root.clone();
        preimage["ordinaryPage"]["continuation"]["fixed"].as_object_mut().unwrap().remove("issuedProgramDigest");
        let expected = weft_core::json::sha256(weft_core::mutation::canonical(&preimage).as_bytes());
        assert_eq!(root["ordinaryPage"]["continuation"]["fixed"]["issuedProgramDigest"], expected);
        let mut mutated = preimage.clone();
        mutated["forwardConnection"]["forward"]["sqlTemplate"] = json!("mutated");
        assert_ne!(weft_core::json::sha256(weft_core::mutation::canonical(&mutated).as_bytes()), expected);
    }

    fn request_with_native_outputs(include_char: bool, include_bytea: bool) -> Request {
        let mut request = request_with_access("0.1.0", "primary", false);
        let mut document: Value = serde_json::from_str(&request.binding.model_json).unwrap();
        let module = document["modules"].as_array_mut().unwrap().iter_mut().find(|m| m["id"] == "demo").unwrap();
        let mut fields = Vec::new();
        if include_char { fields.push(("short_code", "string", "char", "string")); }
        if include_bytea { fields.push(("opaque", "bits128", "bytea", "bits128")); }
        for (field_id, scalar_type, native_type, carrier) in fields {
            let field_ref = json!({"module":"demo","element":field_id});
            module["elements"].as_array_mut().unwrap().push(json!({"id":field_id,"kind":"field","scalarType":scalar_type,"cardinality":"one","nullability":"required","extensions":{}}));
            module["elements"].as_array_mut().unwrap().iter_mut().find(|e| e["id"] == "Note").unwrap()["members"].as_array_mut().unwrap().push(field_ref.clone());
            let view_fields = &mut module["extensions"]["umf.access"]["views"][0]["fields"];
            view_fields.as_array_mut().unwrap().push(json!({"field":field_ref,"row":{"missing":false,"null":false},"create":{"missing":false,"null":false},"mutable":false,"generated":false}));
            let native_type_value = if native_type == "char" { json!({"kind":"char","length":8}) } else { json!({"kind":native_type}) };
            module["extensions"]["umf.tables"]["tables"][0]["columns"].as_array_mut().unwrap().push(json!({"field":field_ref,"name":field_id,"nativeType":native_type_value}));
            request.binding.tables[0].columns.push(weft_core::raw_read::Column { logical:field_id.into(), native:field_id.into(), carrier:serde_json::from_value(json!(carrier)).unwrap(), nullable:false });
        }
        request.binding.model_json = weft_core::mutation::canonical(&document);
        request.binding.model_sha256 = weft_core::json::sha256(request.binding.model_json.as_bytes());
        request
    }

    #[test]
    fn v6_emits_bytea_public_metadata_and_closes_char_helper_but_upstream_refuses_authored_char() {
        let bytea_request = request_with_native_outputs(false, true);
        let artifact = issue_v6(bytea_request).unwrap();
        let outputs = artifact["roots"]["demo/Note"]["ordinaryPage"]["page"]["fieldOutputs"].as_array().unwrap();
        let bytea_output = outputs.iter().find(|v| v["fieldId"] == "demo/opaque").unwrap();
        assert_eq!(bytea_output["output"]["nativeType"]["kind"], "bytea");
        assert_eq!(bytea_output["output"]["oid"], 17);
        assert_eq!(bytea_output["output"]["carrier"], "bits128");
        if std::env::var_os("WEFT_TEST_CAPTURE_GENERATED_READ_V6_NATIVE_TYPES").is_some() {
            eprintln!("WEFT_GENERATED_READ_V6_BYTEA_JSON={}", weft_core::mutation::canonical(&artifact));
        }

        // This helper vector exercises the exact compiler-owned join/output
        // branch for char. The lifecycle admission profile rejects authored
        // char storage before public Page issuance (text/varchar only).
        let char_ref = json!({"module":"demo","element":"short_code"});
        let char_public = json!({"sourceField":char_ref,"output":{"name":"field","carrier":"string","nullable":false}});
        let char_column = weft_core::raw_read::Column { logical:"short_code".into(), native:"short_code".into(), carrier:weft_core::raw_read::Carrier::String, nullable:false };
        let char_mapping = json!({"field":char_ref,"name":"short_code","nativeType":{"kind":"char","length":8}});
        let char_output = ordinary_page_field_output("demo/short_code", &char_public, &char_ref, &char_column, &char_mapping).unwrap();
        assert_eq!(char_output["output"]["nativeType"]["kind"], "char");
        assert_eq!(char_output["output"]["oid"], 1042);
        assert_eq!(char_output["output"]["carrier"], "string");
        assert!(!ordinary_page_key_type_supported(&char_mapping["nativeType"], &char_column));
        if std::env::var_os("WEFT_TEST_CAPTURE_GENERATED_READ_V6_NATIVE_TYPES").is_some() {
            eprintln!("WEFT_GENERATED_READ_V6_CHAR_HELPER_JSON={}", weft_core::mutation::canonical(&char_output));
        }

        let char_request = request_with_native_outputs(true, false);
        assert_eq!(issue_v6(char_request).unwrap_err(), "native column type/mapping/availability differs");
    }

    #[test]
    fn issues_closed_ordinary_page_and_keeps_v5_independent() {
        let request = request_with_access("0.1.0", "primary", false);
        let v5_before = issue_v5(request.clone()).unwrap();
        let frozen_v5: Value = serde_json::from_str(include_str!("../../../docs/helix/04-build/evidence/generated-connection-v5-20261104/issued-v5.json")).unwrap();
        assert_eq!(v5_before, frozen_v5);
        let v6 = issue_v6(request.clone()).unwrap();
        if std::env::var_os("WEFT_TEST_CAPTURE_GENERATED_READ_V6").is_some() {
            eprintln!("WEFT_GENERATED_READ_V6_JSON={}", weft_core::mutation::canonical(&v6));
        }
        let v5_after = issue_v5(request).unwrap();
        assert_eq!(v5_before, v5_after);
        assert_eq!(v6["version"], "weft.generated-read-algebra/6");
        assert_eq!(v6["executorAbi"], json!({"major":1,"minor":6}));
        let page = &v6["roots"]["demo/Note"]["ordinaryPage"];
        assert_eq!(page["profile"], "weft.generated-ordinary-page/1");
        assert_eq!(page["status"], "eligible");
        assert_eq!(page["size"]["identity"], true);
        assert_eq!(page["continuation"]["exactFullPageYieldsContinuation"], true);
        assert_eq!(page["continuation"]["ownerCursor"], Value::Null);
        assert_eq!(page["continuation"]["snapshotVersions"], json!([]));
        assert_eq!(page["continuation"]["requestInputs"]["filterProgram"]["profile"], "weft.generated-page-filter-program/1");
        assert_eq!(page["continuation"]["requestInputs"]["normalizedPredicateValues"]["identity"], true);
        let page_fields = page["page"]["fieldOutputs"].as_array().unwrap();
        assert!(!page_fields.is_empty());
        assert!(page_fields.iter().all(|field| field["output"]["nativeType"].is_object() && field["output"]["oid"].is_number()));
        let root = &v6["roots"]["demo/Note"];
        let issued_digest = page["continuation"]["fixed"]["issuedProgramDigest"].as_str().unwrap();
        let mut digest_root = root.clone();
        digest_root["ordinaryPage"]["continuation"]["fixed"].as_object_mut().unwrap().remove("issuedProgramDigest");
        assert_eq!(issued_digest, weft_core::json::sha256(weft_core::mutation::canonical(&digest_root).as_bytes()));
        digest_root["fields"].as_object_mut().unwrap().values_mut().next().unwrap()["selectSql"] = json!("SELECT 99");
        assert_ne!(issued_digest, weft_core::json::sha256(weft_core::mutation::canonical(&digest_root).as_bytes()));
        assert!(page["page"]["sqlTemplate"].as_str().unwrap().contains("__weft_page_key"));
        assert_eq!(page["page"]["serverByteGuard"], true);
        assert!(page["page"]["sqlTemplate"].as_str().unwrap().contains("ORDER BY f.\"__weft_ordinal\""));
        let guard = v6["roots"]["demo/Note"]["nativeReadGuard"]["template"]["sql"].as_str().unwrap();
        for projection in ["", ", \"r\".\"id\" AS \"__weft_field_0\""] {
            let source = page["page"]["sourceSqlTemplate"].as_str().unwrap().replace("{{PUBLIC_PROJECTION}}", projection).replace("{{FILTER}}", "TRUE");
            let guarded = guard.replace("{{SOURCE_SQL}}", &source).replace("{{RETURN_COLUMNS}}", "f.*");
            assert!(guarded.contains("ORDER BY f.\"__weft_ordinal\""));
            assert!(source.contains("AS \"__weft_page_key\""));
            assert!(!source.contains("__weft_page_key,  FROM"));
        }
        let predicate = v6["roots"]["demo/Note"]["predicates"].as_object().unwrap().values()
            .find(|p| p["operator"] == "eq" && p["arity"] == 1).unwrap()["sql"].as_str().unwrap();
        let predicate_pg = predicate.replace("%s", "$1");
        let nested_filter = format!("({}) AND (({}) OR ({}))", predicate_pg.replace("$1", "$1"), predicate_pg.replace("$1", "$2"), predicate_pg.replace("$1", "$3"));
        // Runtime filter composition assigns a depth-first value slot to each
        // repeated leaf; page continuation slots follow the filter group.
        let mut assembled_source = page["page"]["sourceSqlTemplate"].as_str().unwrap().replace("{{PUBLIC_PROJECTION}}", "");
        assembled_source = assembled_source.replace("$3::pg_catalog.int8", "$6::pg_catalog.int8").replace("$2::pg_catalog.text", "$5::pg_catalog.text").replace("$1::pg_catalog.text", "$4::pg_catalog.text");
        assembled_source = assembled_source.replace("{{FILTER}}", &nested_filter);
        let assembled_guard = guard.replace("{{SOURCE_SQL}}", &assembled_source).replace("{{RETURN_COLUMNS}}", "f.*");
        let (assembled_dbapi, assembled_order) = dbapi_sql_with_order(&assembled_guard, &["filterValues", "filterValues", "filterValues", "after", "after", "size"]).unwrap();
        assert_eq!(assembled_order, vec!["filterValues", "filterValues", "filterValues", "after", "after", "size"]);
        assert_eq!(assembled_dbapi.matches("%s").count(), 6);
        assert!(assembled_source.contains("owner"));
        let collation_source = page["nativeCollationPrecondition"]["sourceSqlTemplate"].as_str().unwrap();
        assert!(Parser::parse_sql(&PostgreSqlDialect {}, collation_source).is_ok());
        let mixed = issue_v6(request_with_mixed_connection_roots()).unwrap();
        if std::env::var_os("WEFT_TEST_CAPTURE_GENERATED_READ_V6").is_some() {
            eprintln!("WEFT_GENERATED_READ_V6_UNSUPPORTED_JSON={}", weft_core::mutation::canonical(&mixed));
        }
        assert_eq!(mixed["roots"]["demo/Metric"]["ordinaryPage"]["status"], "unsupported");
        assert_eq!(mixed["roots"]["demo/Metric"]["ordinaryPage"]["profile"], "weft.generated-ordinary-page/1");

        let mut quoted = request_with_access("0.1.0", "primary", false);
        quoted.binding.tables[0].native = "note%\"s".into();
        quoted.binding.tables[0].columns[0].native = "key%\"id".into();
        let mut model: Value = serde_json::from_str(&quoted.binding.model_json).unwrap();
        let native_table = &mut model["modules"][0]["extensions"]["umf.tables"]["tables"][0];
        native_table["name"] = json!("note%\"s");
        native_table["columns"][0]["name"] = json!("key%\"id");
        quoted.binding.model_json = weft_core::mutation::canonical(&model);
        quoted.binding.model_sha256 = weft_core::json::sha256(quoted.binding.model_json.as_bytes());
        let quoted_v6 = issue_v6(quoted.clone()).unwrap();
        if std::env::var_os("WEFT_TEST_CAPTURE_GENERATED_READ_V6").is_some() {
            eprintln!("WEFT_GENERATED_READ_V6_QUOTED_JSON={}", weft_core::mutation::canonical(&quoted_v6));
        }
        let quoted_page = quoted_v6["roots"]["demo/Note"]["ordinaryPage"].clone();
        let quoted_sql = quoted_page["page"]["sqlTemplate"].as_str().unwrap();
        assert!(quoted_sql.contains("note%%\"\"s"), "{quoted_sql}");
        assert!(quoted_sql.contains("key%%\"\"id"), "{quoted_sql}");
        let mut marker = quoted;
        marker.binding.tables[0].columns[0].native = "{{FILTER}}".into();
        let mut model: Value = serde_json::from_str(&marker.binding.model_json).unwrap();
        model["modules"][0]["extensions"]["umf.tables"]["tables"][0]["columns"][0]["name"] = json!("{{FILTER}}");
        marker.binding.model_json = weft_core::mutation::canonical(&model);
        marker.binding.model_sha256 = weft_core::json::sha256(marker.binding.model_json.as_bytes());
        assert!(issue_v6(marker).is_err());
    }

    fn request_with_mixed_connection_roots() -> Request {
        let mut request = request_with_access("0.1.0", "primary", false);
        let mut document: Value = serde_json::from_str(&request.binding.model_json).unwrap();
        let module = document["modules"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|module| module["id"] == "demo")
            .unwrap();
        let value_field = json!({"module":"demo","element":"value"});
        let metric_record = json!({"module":"demo","element":"Metric"});
        module["elements"].as_array_mut().unwrap().push(json!({
            "id":"value","kind":"field","scalarType":"integer","cardinality":"one","nullability":"required","extensions":{}
        }));
        module["elements"].as_array_mut().unwrap().push(json!({
            "id":"Metric","kind":"record","members":[value_field],"keys":[{"id":"primary","name":"primary","primary":true,"fields":[value_field]}],"extensions":{}
        }));
        let table = &mut module["extensions"]["umf.tables"]["tables"][0];
        let mut metric_table = table.clone();
        metric_table["id"] = json!("metrics");
        metric_table["name"] = json!("metrics");
        metric_table["record"] = metric_record.clone();
        metric_table["columns"] = json!([{"field":value_field,"name":"value","nativeType":{"kind":"int8"}}]);
        module["extensions"]["umf.tables"]["tables"]
            .as_array_mut()
            .unwrap()
            .push(metric_table);
        let access_fields = json!([{
            "field":value_field,"row":{"missing":false,"null":false},"create":{"missing":false,"null":false},"mutable":false,"generated":false
        }]);
        module["extensions"]["umf.access"]["views"]
            .as_array_mut()
            .unwrap()
            .push(json!({"record":metric_record,"fields":access_fields,"selectedKey":{"module":"demo","element":"Metric","key":"primary"}}));
        request.binding.tables.push(weft_core::raw_read::Table {
            logical: vec!["demo".into(), "Metric".into()],
            native: "metrics".into(),
            columns: vec![weft_core::raw_read::Column {
                logical: "value".into(),
                native: "value".into(),
                carrier: weft_core::raw_read::Carrier::Integer,
                nullable: false,
            }],
            key: vec!["value".into()],
        });
        request.binding.model_json = weft_core::mutation::canonical(&document);
        request.binding.model_sha256 = weft_core::json::sha256(request.binding.model_json.as_bytes());
        request.version = "weft.generated-read-algebra-request/5".into();
        request
    }

    #[test]
    fn admits_exact_same_source_access_view_and_refuses_alternate_key_view() {
        let valid = issue(request_with_access("0.1.0", "primary", false)).unwrap();
        assert_eq!(valid["readiness"], "weft.generated-read-executable/1");
        assert_eq!(valid["roots"]["demo/Note"]["defaultSelection"], json!(["demo/id", "demo/owner"]));
        assert_eq!(
            issue(request_with_access("0.1.0", "alternate", true)).unwrap_err(),
            "native table access view composition unsupported"
        );
    }

    #[test]
    fn refuses_unknown_access_vocabulary_version() {
        assert_eq!(
            issue(request_with_access("0.2.0", "primary", false)).unwrap_err(),
            "native table lifecycle selected vocabulary composition unsupported"
        );
    }

    #[test]
    fn issues_source_pinned_scalar_read_fragments_through_raw_lowerer() {
        let model = include_str!("../../../tests/raw-postgresql/table-operation-model.json");
        let mut template: Value = serde_json::from_str(include_str!(
            "../../../tests/raw-postgresql/table-lifecycle-read-template.json"
        ))
        .unwrap();
        let mut binding: Binding = serde_json::from_value(template["binding"].take()).unwrap();
        binding.tables[0].columns.reverse();
        binding.model_json = model.to_owned();
        binding.model_sha256 = weft_core::json::sha256(model.as_bytes());
        let request = Request {
            version: "weft.generated-read-algebra-request/1".into(),
            target: "postgresql18".into(),
            binding,
            module: "demo".into(),
        };
        let binding_value = serde_json::to_value(&request.binding).unwrap();
        let binding_digest =
            weft_core::json::sha256(weft_core::mutation::canonical(&binding_value).as_bytes());
        let result = build(request, binding_digest).unwrap();
        assert_eq!(result["readiness"], "weft.generated-read-executable/1");
        let root = &result["roots"]["demo/Note"];
        assert_eq!(root["defaultSelection"], json!(["demo/id", "demo/owner"]));
        assert_eq!(
            root["fields"]["demo/id"]["sourceField"],
            json!({"module":"demo","element":"id"})
        );
        assert_eq!(root["fields"]["demo/id"]["output"]["carrier"], "string");
        assert_eq!(
            root["fields"]["demo/id"]["selectSql"],
            "\"r\".\"id\" COLLATE pg_catalog.\"C\" AS \"__weft_field_0\""
        );
        assert!(root["fromSql"]
            .as_str()
            .unwrap()
            .contains("FROM \"plugin_demo\".\"notes\" AS \"r\""));
        assert_eq!(root["keyTieOrder"].as_array().unwrap().len(), 1);
        assert!(root["predicates"]
            .as_object()
            .unwrap()
            .values()
            .any(|value| value["operator"] == "eq"
                && value["codec"]["profile"] == "weft.query-codec-instructions/1"));
        assert_eq!(root["nativeReadGuard"]["profile"], "pg18-read-frame/0.1");
    }

    #[test]
    fn v2_issues_primary_key_and_scalar_terminal_native_descriptors() {
        let mut request = request_with_access("0.1.0", "primary", false);
        request.version = "weft.generated-read-algebra-request/2".into();
        let binding_value = serde_json::to_value(&request.binding).unwrap();
        let binding_digest =
            weft_core::json::sha256(weft_core::mutation::canonical(&binding_value).as_bytes());
        let result = build_v2(request, binding_digest).unwrap();
        assert_eq!(result["version"], "weft.generated-read-algebra/2");
        assert_eq!(result["readiness"], "weft.generated-read-executable/2");
        let root = &result["roots"]["demo/Note"];
        assert_eq!(root["primaryKey"].as_array().unwrap().len(), 1);
        assert_eq!(root["primaryKey"][0]["fieldId"], "demo/id");
        assert_eq!(root["primaryKeyId"], "primary");
        let count = &root["terminals"]["count"];
        assert_eq!(count["output"]["carrier"], "integer");
        assert_eq!(count["output"]["nullable"], false);
        assert_eq!(count["output"]["nativeType"]["kind"], "int8");
        assert_eq!(count["output"]["oid"], 20);
        assert!(count["selectSql"].as_str().unwrap().contains("count(*)"));
        let exists = &root["terminals"]["exists"];
        assert_eq!(exists["output"]["carrier"], "boolean");
        assert_eq!(exists["output"]["nullable"], false);
        assert_eq!(exists["output"]["nativeType"]["kind"], "boolean");
        assert_eq!(exists["output"]["oid"], 16);
        assert!(exists["selectSql"].as_str().unwrap().contains("true"));

        let mut alternate_binding = request_with_access("0.1.0", "primary", true);
        alternate_binding.version = "weft.generated-read-algebra-request/2".into();
        alternate_binding.binding.tables[0].key = vec!["owner".into()];
        let binding_value = serde_json::to_value(&alternate_binding.binding).unwrap();
        let digest = weft_core::json::sha256(
            weft_core::mutation::canonical(&binding_value).as_bytes(),
        );
        assert!(build_v2(alternate_binding, digest).is_err());
    }

    #[test]
    fn v3_issues_server_guard_template_and_raw_typed_frame_aliases() {
        let mut request = request_with_access("0.1.0", "primary", false);
        request.version = "weft.generated-read-algebra-request/3".into();
        let binding_value = serde_json::to_value(&request.binding).unwrap();
        let binding_digest =
            weft_core::json::sha256(weft_core::mutation::canonical(&binding_value).as_bytes());
        let result = build_v3(request, binding_digest).unwrap();
        assert_eq!(result["version"], "weft.generated-read-algebra/3");
        assert_eq!(result["readiness"], "weft.generated-read-executable/3");
        let root = &result["roots"]["demo/Note"];
        assert_eq!(root["nativeReadGuard"]["profile"], "pg18-composed-native-frame/1");
        assert_eq!(root["nativeReadGuard"]["byteMetric"], "pg18-row-json-utf8/1");
        assert_eq!(root["nativeReadGuard"]["sqlstate"], "22012");
        assert_eq!(root["nativeReadGuard"]["partialFrames"], false);
        let template = root["nativeReadGuard"]["template"]["sql"].as_str().unwrap();
        assert_eq!(template.matches("{{SOURCE_SQL}}").count(), 1);
        assert_eq!(template.matches("{{RETURN_COLUMNS}}").count(), 1);
        assert_eq!(root["frameOrdinalSql"], "ROW_NUMBER() OVER (ORDER BY {{ORDER_SQL}}) AS \"__weft_ordinal\"");
        assert_eq!(root["frameScalarOrdinalSql"], "1::pg_catalog.int8 AS \"__weft_ordinal\"");
        assert_eq!(root["fields"]["demo/id"]["frameSelectSql"], "f.\"__weft_field_0\"");
        assert_eq!(root["fields"]["demo/owner"]["frameSelectSql"], "f.\"__weft_field_1\"");
        assert_eq!(root["terminals"]["count"]["frameSelectSql"], "f.\"__weft_count\"");
        assert_eq!(root["terminals"]["exists"]["frameSelectSql"], "f.\"__weft_exists\"");
        let expected_digest = weft_core::json::sha256(template.as_bytes());
        assert_eq!(root["nativeReadGuard"]["template"]["sha256"], expected_digest);
    }

    #[test]
    fn v4_issues_ordered_key_batch_codec_tuple_and_rls_left_join_template() {
        let mut request = request_with_access("0.1.0", "primary", false);
        request.version = "weft.generated-read-algebra-request/4".into();
        let binding_value = serde_json::to_value(&request.binding).unwrap();
        let binding_digest =
            weft_core::json::sha256(weft_core::mutation::canonical(&binding_value).as_bytes());
        let result = build_v4(request, binding_digest).unwrap();
        assert_eq!(result["version"], "weft.generated-read-algebra/4");
        assert_eq!(result["readiness"], "weft.generated-read-executable/4");
        let root = &result["roots"]["demo/Note"];
        let batch = &root["getMany"];
        assert_eq!(batch["profile"], "weft.generated-key-batch/1");
        assert_eq!(batch["maxKeys"], 100);
        assert_eq!(batch["keyId"], "primary");
        assert_eq!(batch["keyFields"].as_array().unwrap().len(), 1);
        assert_eq!(batch["keyFields"][0]["fieldId"], "demo/id");
        assert_eq!(
            batch["keyFields"][0]["codec"]["profile"],
            "weft.query-codec-instructions/1"
        );
        assert_eq!(
            batch["keyTupleSql"],
            "(%s::pg_catalog.int8, %s::pg_catalog.text)"
        );
        let template = batch["sourceSqlTemplate"].as_str().unwrap();
        assert_eq!(template.matches("{{KEY_ROWS}}").count(), 1);
        assert_eq!(template.matches("{{SELECT_COLUMNS}}").count(), 1);
        assert!(template.contains("LEFT JOIN \"plugin_demo\".\"notes\" AS \"r\""));
        assert!(template.contains("k.\"__weft_ordinal\""));
        assert!(template.contains("(\"r\".\"id\" IS NOT NULL) AS \"__weft_present\""));
        assert!(template.contains(
            "\"r\".\"id\" COLLATE pg_catalog.\"C\" = k.\"__weft_key_0\" COLLATE pg_catalog.\"C\""
        ));
        assert_eq!(
            batch["sourceTemplateSha256"],
            weft_core::json::sha256(template.as_bytes())
        );
        assert_eq!(batch["ordinalOutput"]["oid"], 20);
        assert_eq!(batch["presentOutput"]["oid"], 16);

        let mut alternate = request_with_access("0.1.0", "primary", true);
        alternate.version = "weft.generated-read-algebra-request/4".into();
        alternate.binding.tables[0].key = vec!["owner".into()];
        let binding_value = serde_json::to_value(&alternate.binding).unwrap();
        let binding_digest =
            weft_core::json::sha256(weft_core::mutation::canonical(&binding_value).as_bytes());
        assert!(build_v4(alternate, binding_digest).is_err());
    }

    #[test]
    fn v5_issues_bounded_forward_connection_fragments_without_changing_v4() {
        let request = request_with_access("0.1.0", "primary", false);
        let result = issue_v5(request).unwrap();
        assert_eq!(result["version"], "weft.generated-read-algebra/5");
        assert_eq!(result["readiness"], "weft.generated-read-executable/5");
        assert_eq!(result["executorAbi"], json!({"major":1,"minor":5}));
        let root = &result["roots"]["demo/Note"];
        let connection = &root["forwardConnection"];
        assert_eq!(connection["profile"], "weft.generated-forward-connection/1");
        assert_eq!(connection["maxFirst"], 100);
        assert_eq!(connection["lookaheadRows"], 1);
        assert_eq!(connection["zeroFirstProbeRows"], 1);
        assert_eq!(connection["nativeCollation"], "pg_catalog.\"C\"");
        assert_eq!(connection["nativeCollationPrecondition"], "umf.generated-table-collation-c/1");
        assert_eq!(connection["keyId"], "primary");
        assert_eq!(connection["key"]["fieldId"], "demo/id");
        assert_eq!(connection["key"]["direction"], "asc");
        assert_eq!(connection["key"]["nulls"], "last");
        assert_eq!(connection["key"]["collation"], "pg_catalog.\"C\"");
        assert_eq!(connection["forward"]["keyOutput"]["oid"], 25);
        assert_eq!(connection["fieldOutputs"].as_array().unwrap().len(), 2);
        assert!(connection["fieldOutputs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|field| field["output"].get("oid").is_some()));
        assert_eq!(connection["previous"]["output"]["oid"], 16);
        let forward = connection["forward"]["sqlTemplate"].as_str().unwrap();
        let forward_source = connection["forward"]["sourceSqlTemplate"].as_str().unwrap();
        assert!(forward_source.contains("$1::pg_catalog.text"));
        assert_eq!(
            connection["forward"]["sourceTemplateSha256"],
            weft_core::json::sha256(forward_source.as_bytes())
        );
        assert_eq!(
            connection["forward"]["templateSha256"],
            weft_core::json::sha256(forward.as_bytes())
        );
        assert_eq!(connection["forward"]["parameterOrder"], json!(["filterValues","after","after","limit"]));
        assert_eq!(connection["forward"]["dbapiParameterOrder"], json!(["after","after","limit"]));
        assert_eq!(connection["forward"]["filterParameterGroup"], "filterValues");
        assert_eq!(forward.matches("{{FILTER}}").count(), 1);
        assert_eq!(forward.matches("{{SELECT_COLUMNS}}").count(), 1);
        assert!(forward.contains("<= 8388608"));
        assert!(forward.contains("LIMIT"), "{forward}");
        assert!(forward.contains("ORDER BY f.\"__weft_ordinal\""));
        let predicate = root["predicates"]
            .as_object()
            .unwrap()
            .values()
            .find(|predicate| predicate["operator"] == "eq")
            .unwrap()["sql"]
            .as_str()
            .unwrap();
        assert!(predicate.contains("%s"));
        assert_eq!(forward.replace("{{FILTER}}", predicate).matches("%s").count(), 4);
        let previous = connection["previous"]["sqlTemplate"].as_str().unwrap();
        let previous_source = connection["previous"]["sourceSqlTemplate"].as_str().unwrap();
        assert!(previous_source.contains("$1::pg_catalog.text IS NOT NULL"));
        assert_eq!(
            connection["previous"]["sourceTemplateSha256"],
            weft_core::json::sha256(previous_source.as_bytes())
        );
        assert_eq!(connection["previous"]["parameterOrder"], json!(["filterValues","after","after"]));
        assert_eq!(connection["previous"]["dbapiParameterOrder"], json!(["after","after"]));
        assert_eq!(connection["previous"]["filterParameterGroup"], "filterValues");
        assert_eq!(previous.matches("{{FILTER}}").count(), 1);
        assert!(previous.contains(" IS NOT NULL") && previous.contains(" <= "));
        assert_eq!(previous.replace("{{FILTER}}", predicate).matches("%s").count(), 3);
        assert!(previous.contains("<= 8388608"));
        assert_eq!(
            connection["forward"]["templateSha256"],
            weft_core::json::sha256(forward.as_bytes())
        );
        assert_eq!(
            connection["previous"]["templateSha256"],
            weft_core::json::sha256(previous.as_bytes())
        );
        assert_eq!(root["getMany"]["profile"], "weft.generated-key-batch/1");

        let mut integer_key = request_with_access("0.1.0", "primary", false);
        let mut document: Value = serde_json::from_str(&integer_key.binding.model_json).unwrap();
        let module = document["modules"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|module| module["id"] == "demo")
            .unwrap();
        module["elements"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|field| field["id"] == "id")
            .unwrap()["scalarType"] = json!("integer");
        module["extensions"]["umf.tables"]["tables"][0]["columns"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|column| column["field"]["element"] == "id")
            .unwrap()["nativeType"] = json!({"kind":"int8"});
        integer_key.binding.tables[0].columns[0].carrier = weft_core::raw_read::Carrier::Integer;
        integer_key.binding.model_json = weft_core::mutation::canonical(&document);
        integer_key.binding.model_sha256 =
            weft_core::json::sha256(integer_key.binding.model_json.as_bytes());
        assert_eq!(
            issue_v5(integer_key).unwrap_err(),
            "generated read v5 has no eligible forward connection roots"
        );
    }

    #[test]
    fn v5_keeps_ineligible_integer_roots_and_their_v4_read_algebra() {
        let request = request_with_mixed_connection_roots();
        let result = issue_v5(request).unwrap();
        let eligible = &result["roots"]["demo/Note"];
        let ineligible = &result["roots"]["demo/Metric"];
        assert_eq!(eligible["forwardConnectionSupport"]["status"], "eligible");
        assert_eq!(eligible["forwardConnection"]["profile"], "weft.generated-forward-connection/1");
        assert_eq!(ineligible["forwardConnectionSupport"]["status"], "unsupported");
        assert_eq!(ineligible["forwardConnectionSupport"]["reason"], "ROOT_OUTSIDE_PROFILE");
        assert!(ineligible.get("forwardConnection").is_none());
        assert_eq!(ineligible["getMany"]["profile"], "weft.generated-key-batch/1");
        assert_eq!(ineligible["primaryKey"][0]["fieldId"], "demo/value");
    }

    #[test]
    fn v5_escapes_percent_placeholders_in_physical_names_once() {
        let mut request = request_with_access("0.1.0", "primary", false);
        let mut document: Value = serde_json::from_str(&request.binding.model_json).unwrap();
        document["modules"][0]["extensions"]["umf.tables"]["tables"][0]["name"] =
            json!("n%s");
        request.binding.tables[0].native = "n%s".into();
        request.binding.model_json = weft_core::mutation::canonical(&document);
        request.binding.model_sha256 = weft_core::json::sha256(request.binding.model_json.as_bytes());
        let result = issue_v5(request).unwrap();
        let connection = &result["roots"]["demo/Note"]["forwardConnection"];
        for sql in [
            connection["forward"]["sqlTemplate"].as_str().unwrap(),
            connection["previous"]["sqlTemplate"].as_str().unwrap(),
        ] {
            assert!(sql.contains("\"n%%s\""), "{sql}");
            assert!(!sql.contains("%%%%s"), "{sql}");
        }
        assert_eq!(connection["forward"]["dbapiParameterOrder"], json!(["after","after","limit"]));
        assert_eq!(connection["previous"]["dbapiParameterOrder"], json!(["after","after"]));
    }

    #[test]
    fn v4_integer_key_uses_exact_numeric_parameter_cast_for_int8_column() {
        let mut request = request_with_access("0.1.0", "primary", false);
        let mut document: Value = serde_json::from_str(&request.binding.model_json).unwrap();
        let module = document["modules"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|module| module["id"] == "demo")
            .unwrap();
        module["elements"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|field| field["id"] == "id")
            .unwrap()["scalarType"] = json!("integer");
        module["extensions"]["umf.tables"]["tables"][0]["columns"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|column| column["field"]["element"] == "id")
            .unwrap()["nativeType"] = json!({"kind":"int8"});
        request.binding.tables[0].columns[0].carrier =
            weft_core::raw_read::Carrier::Integer;
        request.binding.model_json = weft_core::mutation::canonical(&document);
        request.binding.model_sha256 = weft_core::json::sha256(request.binding.model_json.as_bytes());
        request.version = "weft.generated-read-algebra-request/4".into();
        let binding_value = serde_json::to_value(&request.binding).unwrap();
        let binding_digest =
            weft_core::json::sha256(weft_core::mutation::canonical(&binding_value).as_bytes());
        let result = build_v4(request, binding_digest).unwrap();
        let batch = &result["roots"]["demo/Note"]["getMany"];
        assert_eq!(batch["keyTupleSql"], "(%s::pg_catalog.int8, %s::pg_catalog.numeric)");
        assert_eq!(batch["keyFields"][0]["codec"]["steps"][1]["carrier"], "integer");
        assert_eq!(batch["keyFields"][0]["fieldId"], "demo/id");
    }

    #[test]
    fn v4_refuses_physical_names_matching_template_slots() {
        for marker in ["{{KEY_ROWS}}", "{{SELECT_COLUMNS}}"] {
            let mut request = request_with_access("0.1.0", "primary", false);
            request.version = "weft.generated-read-algebra-request/4".into();
            let mut document: Value = serde_json::from_str(&request.binding.model_json).unwrap();
            document["modules"][0]["extensions"]["umf.tables"]["tables"][0]["name"] =
                json!(marker);
            request.binding.tables[0].native = marker.into();
            request.binding.model_json = weft_core::mutation::canonical(&document);
            request.binding.model_sha256 =
                weft_core::json::sha256(request.binding.model_json.as_bytes());
            let binding_value = serde_json::to_value(&request.binding).unwrap();
            let binding_digest =
                weft_core::json::sha256(weft_core::mutation::canonical(&binding_value).as_bytes());
            assert_eq!(
                build_v4(request, binding_digest).unwrap_err(),
                "generated read v4 physical identifier collides with a template slot"
            );
        }
    }

    #[test]
    fn v4_dbapi_escapes_percent_placeholder_text_in_physical_identifiers() {
        let mut request = request_with_access("0.1.0", "primary", false);
        request.version = "weft.generated-read-algebra-request/4".into();
        let mut document: Value = serde_json::from_str(&request.binding.model_json).unwrap();
        document["modules"][0]["extensions"]["umf.tables"]["tables"][0]["name"] =
            json!("notes%s");
        request.binding.tables[0].native = "notes%s".into();
        request.binding.model_json = weft_core::mutation::canonical(&document);
        request.binding.model_sha256 =
            weft_core::json::sha256(request.binding.model_json.as_bytes());
        let binding_value = serde_json::to_value(&request.binding).unwrap();
        let binding_digest =
            weft_core::json::sha256(weft_core::mutation::canonical(&binding_value).as_bytes());
        let result = build_v4(request, binding_digest).unwrap();
        let batch = &result["roots"]["demo/Note"]["getMany"];
        assert!(batch["sourceSqlTemplate"].as_str().unwrap().contains("\"notes%%s\""));
        assert_eq!(batch["keyTupleSql"], "(%s::pg_catalog.int8, %s::pg_catalog.text)");
        assert_eq!(batch["sourceSqlTemplate"].as_str().unwrap().matches("%s").count(), 1);
    }

    #[test]
    fn v4_refuses_unqualified_bits128_key_comparison() {
        assert_eq!(
            get_many_key_cast(&weft_core::raw_read::Carrier::Bits128).unwrap_err(),
            "generated read v4 Bits128 Key equality is outside the issued native profile"
        );
    }

    #[test]
    fn qualified_element_ids_use_json_pointer_component_escaping() {
        assert_eq!(
            field_key(&json!({"module":"a/b","element":"x~y"})).unwrap(),
            "a~1b/x~0y"
        );
    }

    #[test]
    fn logical_root_names_are_not_limited_as_physical_identifiers() {
        let long = "logical-name-that-is-not-a-postgresql-identifier-limit".repeat(2);
        let path = path_sql(&[long.clone(), "Record".into()]).unwrap();
        assert_eq!(path, format!("\"{long}\".\"Record\""));
    }

    #[test]
    fn dbapi_transport_escapes_percent_without_rewriting_quoted_dollar_number() {
        assert_eq!(
            dbapi_sql("\"r\".\"col%$1\"", &[]).unwrap(),
            "\"r\".\"col%%$1\""
        );
    }
}

#[path="raw_read_generated_join_v8.rs"]
mod generated_join_v8;
