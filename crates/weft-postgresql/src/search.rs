//! Exact same-table keyword/pgvector top-k lowering. No embedding service or I/O.
use serde_json::{json, Value as Json};
use sqlparser::ast::*;
use weft_core::{raw_read::Carrier, search};
type Result<T> = std::result::Result<T, &'static str>;
fn ident(s: &str) -> Result<String> {
    if s.is_empty() || s.len() > 63 || s.contains('\0') {
        return Err("search identifier outside native profile");
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
fn field<'a>(doc: &'a Json, reference: &Json) -> Result<&'a Json> {
    doc["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == reference["module"])
        .and_then(|m| {
            m["elements"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["id"] == reference["element"])
        })
        .ok_or("search Field absent")
}
fn compile(raw: &str) -> Result<Json> {
    let admitted = search::admit_json(raw)?;
    let r = admitted.request();
    let index = admitted.index();
    let binding = &r.selection.binding;
    let read = crate::raw_read::lower_search_input(admitted.selection())?;
    let q = admitted.selection().query();
    if q.with.is_some() || q.limit_clause.is_some() || q.order_by.is_some() {
        return Err("search input cannot truncate or reorder matching rows");
    }
    let SetExpr::Select(s) = q.body.as_ref() else {
        return Err("search scalar selection required");
    };
    if s.from.len() != 1
        || !s.from[0].joins.is_empty()
        || s.distinct.is_some()
        || s.having.is_some()
    {
        return Err("search base-table selection required");
    }
    let GroupByExpr::Expressions(group, modifiers) = &s.group_by else {
        return Err("search grouping unavailable");
    };
    if !group.is_empty() || !modifiers.is_empty() {
        return Err("search grouping unavailable");
    }
    let TableFactor::Table {
        name: logical,
        alias: Some(alias),
        ..
    } = &s.from[0].relation
    else {
        return Err("search source alias required");
    };
    let alias = name(&alias.name);
    let path = logical
        .0
        .iter()
        .map(|part| match part {
            ObjectNamePart::Identifier(i) => Ok(name(i)),
            _ => Err("search source path"),
        })
        .collect::<Result<Vec<_>>>()?;
    let table = binding
        .tables
        .iter()
        .find(|t| t.logical == path)
        .ok_or("search table binding absent")?;
    let doc: Json = serde_json::from_str(&binding.model_json).unwrap();
    let module = doc["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == index["record"]["module"])
        .ok_or("search module absent")?;
    if path
        != vec![
            module["namespace"].as_str().unwrap().to_owned(),
            admitted.record()["name"]
                .as_str()
                .ok_or("search Record logical name required")?
                .to_owned(),
        ]
    {
        return Err("search selected owner/binding differs");
    }
    let members = admitted.record()["members"].as_array().unwrap();
    let column_ref = |logical: &str| -> Result<&Json> {
        members
            .iter()
            .find(|reference| field(&doc, reference).is_ok_and(|f| f["name"] == logical))
            .ok_or("search source Field binding differs")
    };
    if s.projection.len() != table.key.len() {
        return Err("search input must project only the complete ordered Key");
    }
    for (p, key) in s.projection.iter().zip(&table.key) {
        let SelectItem::ExprWithAlias {
            expr: Expr::CompoundIdentifier(parts),
            alias: output,
        } = p
        else {
            return Err("search input Key alias required");
        };
        if parts.len() != 2
            || name(&parts[0]) != alias
            || name(&parts[1]) != *key
            || name(output) != *key
        {
            return Err("search Key projection differs");
        }
    }
    fn filter_fields(e: &Expr, out: &mut Vec<Vec<Ident>>) -> Result<()> {
        match e {
            Expr::CompoundIdentifier(parts) => out.push(parts.clone()),
            Expr::Nested(e)
            | Expr::IsNull(e)
            | Expr::IsNotNull(e)
            | Expr::UnaryOp { expr: e, .. } => filter_fields(e, out)?,
            Expr::BinaryOp { left, right, .. } => {
                filter_fields(left, out)?;
                filter_fields(right, out)?
            }
            Expr::Between {
                expr, low, high, ..
            } => {
                filter_fields(expr, out)?;
                filter_fields(low, out)?;
                filter_fields(high, out)?
            }
            Expr::InList { expr, list, .. } => {
                filter_fields(expr, out)?;
                for e in list {
                    filter_fields(e, out)?;
                }
            }
            Expr::Value(_) => {}
            _ => return Err("search filter expression unavailable"),
        };
        Ok(())
    }
    let mut predicates = vec![];
    if let Some(e) = &s.selection {
        filter_fields(e, &mut predicates)?;
    }
    for parts in predicates {
        if parts.len() != 2
            || name(&parts[0]) != alias
            || !index["filterable"]
                .as_array()
                .unwrap()
                .contains(column_ref(&name(&parts[1]))?)
        {
            return Err("SEARCH_FILTER: Field is not filterable");
        }
    }
    let native_table = match &binding.schema {
        Some(schema) => format!("{}.{}", ident(schema)?, ident(&table.native)?),
        None => ident(&table.native)?,
    };
    let native_column = |reference: &Json| -> Result<(String, String, bool)> {
        let f = field(&doc, reference)?;
        if let Some(vector) = &r.vector_binding {
            if vector.field == *reference {
                return Ok((
                    ident(&vector.native)?,
                    "vector".into(),
                    f["nullability"] != "required",
                ));
            }
        }
        let c = table
            .columns
            .iter()
            .find(|c| f["name"] == c.logical)
            .ok_or("search projected Field/native correspondence absent")?;
        Ok((
            ident(&c.native)?,
            match c.carrier {
                Carrier::String => "string",
                Carrier::Boolean => "boolean",
                Carrier::Integer => "integer",
                Carrier::Decimal => "decimal",
            }
            .into(),
            c.nullable,
        ))
    };
    let mut required = table
        .key
        .iter()
        .map(|key| column_ref(key).cloned())
        .collect::<Result<Vec<_>>>()?;
    for p in &r.projection {
        if !required.contains(&p.field) {
            required.push(p.field.clone())
        }
    }
    if index["kind"] == "keyword" {
        for reference in index["keyword"]["source"].as_array().unwrap() {
            if !required.contains(reference) {
                required.push(reference.clone())
            }
        }
    } else {
        let reference = &index["vector"]["field"];
        if !required.contains(reference) {
            required.push(reference.clone())
        }
    }
    let selected = required
        .iter()
        .enumerate()
        .map(|(i, reference)| {
            Ok(format!(
                "t.{} AS {}",
                native_column(reference)?.0,
                ident(&format!("f{i}"))?
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .join(", ");
    let cell = |reference: &Json| -> Result<String> {
        Ok(format!(
            "c.{}",
            ident(&format!(
                "f{}",
                required
                    .iter()
                    .position(|r| r == reference)
                    .ok_or("search cell outside source")?
            ))?
        ))
    };
    let join = table
        .key
        .iter()
        .map(|key| {
            let c = table.columns.iter().find(|c| &c.logical == key).unwrap();
            Ok(format!(
                "t.{} IS NOT DISTINCT FROM m.{}",
                ident(&c.native)?,
                ident(key)?
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .join(" AND ");
    let mut parameters = read["parameterOrder"]
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
            json!({"source":{"input":p.name},"carrier":p.carrier})
        })
        .collect::<Vec<_>>();
    parameters.push(json!({"source":{"input":r.query_parameter},"carrier":if index["kind"]=="vector"{"vector"}else{"string"}}));
    let slot = parameters.len();
    let (score, ordering, condition, kind, mechanism) = if index["kind"] == "keyword" {
        let config = if index["keyword"]["analyzer"]["language"] == "english" {
            "pg_catalog.english"
        } else {
            "pg_catalog.simple"
        };
        let texts = index["keyword"]["source"]
            .as_array()
            .unwrap()
            .iter()
            .map(|reference| {
                Ok(format!(
                    "COALESCE({}, ''::pg_catalog.text)",
                    cell(reference)?
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let text = texts
            .into_iter()
            .reduce(|a, b| format!("({a} || ' '::pg_catalog.text || {b})"))
            .unwrap();
        let vector = format!("pg_catalog.to_tsvector('{config}'::pg_catalog.regconfig, {text})");
        let query=format!("pg_catalog.plainto_tsquery('{config}'::pg_catalog.regconfig, ${slot}::pg_catalog.text)");
        (
            format!("pg_catalog.ts_rank_cd({vector}, {query})::pg_catalog.float8"),
            "DESC",
            Some(format!("{vector} @@ {query}")),
            "implementation-defined",
            json!({"kind":"postgresql-fts","configuration":config,"ranking":"ts_rank_cd","syntax":"plain"}),
        )
    } else {
        let vector = r.vector_binding.as_ref().unwrap();
        let schema = ident(&vector.extension_schema)?;
        let left = cell(&vector.field)?;
        let right = format!("${slot}::{schema}.vector({})", vector.dimensions);
        let metric = index["vector"]["metric"].as_str().unwrap();
        let operator = match metric {
            "cosine" => "<=>",
            "dot" => "<#>",
            "l2" => "<->",
            _ => return Err("vector metric unsupported"),
        };
        let distance = format!("({left} OPERATOR({schema}.{operator}) {right})");
        let (score, direction, kind) = if metric == "dot" {
            (format!("-({distance})"), "DESC", "dot-similarity")
        } else {
            (
                distance,
                "ASC",
                if metric == "cosine" {
                    "cosine-distance"
                } else {
                    "l2-distance"
                },
            )
        };
        // A zero-norm cosine vector has no defined distance. Fail rather than omit it.
        (
            score,
            direction,
            None,
            kind,
            json!({"kind":"pgvector","version":vector.extension_version,"schema":vector.extension_schema,"field":vector.field,"nativeColumn":vector.native,"dimensions":vector.dimensions,"metric":metric,"exactInput":"materialized-complete-matching-set"}),
        )
    };
    let mut outputs = vec![];
    let mut projection = vec![];
    for p in &r.projection {
        let (_, carrier, nullable) = native_column(&p.field)?;
        let value = cell(&p.field)?;
        projection.push(format!(
            "{} AS {}",
            if carrier == "vector" {
                format!("{value}::pg_catalog.text")
            } else {
                value
            },
            ident(&p.name)?
        ));
        outputs.push(json!({"name":p.name,"field":p.field,"carrier":carrier,"nullable":nullable,"decoder":if carrier=="vector"{json!({"profile":format!("pgvector-text-float32/{}",r.vector_binding.as_ref().unwrap().extension_version),"dimensions":r.vector_binding.as_ref().unwrap().dimensions})}else{Json::Null}}));
    }
    let mut key_outputs = vec![];
    for (i, key) in table.key.iter().enumerate() {
        let reference = column_ref(key)?;
        let (_, carrier, _) = native_column(reference)?;
        let name = format!("__umf_key_{i}");
        projection.push(format!("{} AS {}", cell(reference)?, ident(&name)?));
        key_outputs.push(json!({"name":name,"field":reference,"carrier":carrier,"nullable":false}));
    }
    let tie = table
        .key
        .iter()
        .map(|key| {
            let reference = column_ref(key)?;
            let value = cell(reference)?;
            let column = table.columns.iter().find(|c| &c.logical == key).unwrap();
            Ok(if column.carrier == Carrier::String {
                format!("{value} COLLATE pg_catalog.\"C\" ASC")
            } else {
                format!("{value} ASC")
            })
        })
        .collect::<Result<Vec<_>>>()?
        .join(", ");
    projection.push(format!("{score} AS __umf_score"));
    let prefix=format!("WITH __umf_match AS MATERIALIZED ({}), __umf_candidates AS MATERIALIZED (SELECT {selected} FROM {native_table} AS t JOIN __umf_match AS m ON {join})",read["sql"].as_str().unwrap());
    let sql=format!("{prefix} SELECT {} FROM __umf_candidates AS c{} ORDER BY __umf_score {ordering}, {tie} LIMIT {}",projection.join(", "),condition.map(|c|format!(" WHERE {c}")).unwrap_or_default(),r.top_k);
    let preflights = if index["kind"] == "vector" && index["vector"]["metric"] == "cosine" {
        let vector = r.vector_binding.as_ref().unwrap();
        vec![
            json!({"sql":format!("{prefix} SELECT EXISTS(SELECT 1 FROM __umf_candidates AS c WHERE {}.vector_norm({}) = 0) AS invalid_cosine_vector",ident(&vector.extension_schema)?,cell(&vector.field)?),"parameters":parameters[..parameters.len()-1]}),
        ]
    } else {
        vec![]
    };
    let mut admission_parameters = vec![];
    let mut slot = |source: Json| {
        admission_parameters.push(json!({"source":source,"carrier":"string"}));
        format!("${}::pg_catalog.text", admission_parameters.len())
    };
    let app = slot(
        binding
            .schema
            .as_ref()
            .map(|schema| json!({"literal":{"string":schema}}))
            .unwrap_or(json!({"appSchema":true})),
    );
    let native_index = slot(json!({"literal":{"string":admitted.binding()["native"]}}));
    let table_name = slot(json!({"literal":{"string":table.native}}));
    let common=format!("i.indisvalid AND i.indisready AND i.indislive AND i.indnkeyatts=1 AND i.indnatts=1 AND i.indpred IS NULL");
    let extra = if index["kind"] == "keyword" {
        let config = if index["keyword"]["analyzer"]["language"] == "english" {
            "english"
        } else {
            "simple"
        };
        let config = slot(json!({"literal":{"string":config}}));
        let mut arguments = vec![config];
        let mut pieces = vec![];
        for reference in index["keyword"]["source"].as_array().unwrap() {
            let native = native_column(reference)?.0;
            let _ = native;
            let f = field(&doc, reference)?;
            let native = &table
                .columns
                .iter()
                .find(|c| f["name"] == c.logical)
                .unwrap()
                .native;
            arguments.push(slot(json!({"literal":{"string":native}})));
            arguments.push(slot(json!({"literal":{"string":""}})));
            pieces.push("COALESCE(%I, %L::text)".to_owned());
        }
        let text = pieces
            .into_iter()
            .reduce(|a, b| format!("(({a} || ' '::text) || {b})"))
            .unwrap();
        let format =
            slot(json!({"literal":{"string":format!("to_tsvector(%L::regconfig, {text})")}}));
        format!("am.amname='gin' AND opc.opcname='tsvector_ops' AND opns.nspname='pg_catalog' AND i.indkey[0]=0 AND i.indexprs IS NOT NULL AND pg_catalog.pg_get_indexdef(i.indexrelid,1,true)=pg_catalog.format({format}, {})",arguments.join(", "))
    } else {
        let vector = r.vector_binding.as_ref().unwrap();
        let column = slot(json!({"literal":{"string":vector.native}}));
        let schema = slot(json!({"literal":{"string":vector.extension_schema}}));
        let version = slot(json!({"literal":{"string":vector.extension_version}}));
        let opclass = match index["vector"]["metric"].as_str().unwrap() {
            "cosine" => "vector_cosine_ops",
            "dot" => "vector_ip_ops",
            _ => "vector_l2_ops",
        };
        let opclass = slot(json!({"literal":{"string":opclass}}));
        format!("am.amname IN ('hnsw','ivfflat') AND opc.opcname={opclass} AND opns.nspname={schema} AND i.indexprs IS NULL AND EXISTS(SELECT 1 FROM pg_catalog.pg_attribute AS a JOIN pg_catalog.pg_type AS ty ON ty.oid=a.atttypid JOIN pg_catalog.pg_namespace AS tn ON tn.oid=ty.typnamespace JOIN pg_catalog.pg_depend AS dep ON dep.classid='pg_catalog.pg_type'::pg_catalog.regclass AND dep.objid=ty.oid AND dep.refclassid='pg_catalog.pg_extension'::pg_catalog.regclass AND dep.deptype='e' JOIN pg_catalog.pg_extension AS ext ON ext.oid=dep.refobjid WHERE a.attrelid=t.oid AND a.attnum=i.indkey[0] AND a.attname={column} AND NOT a.attisdropped AND ty.typname='vector' AND tn.nspname={schema} AND a.atttypmod={} AND ext.extname='vector' AND ext.extversion={version})",vector.dimensions)
    };
    let extra = if index["kind"] == "keyword" {
        if let Some(vector) = &r.vector_binding {
            let column = slot(json!({"literal":{"string":vector.native}}));
            let schema = slot(json!({"literal":{"string":vector.extension_schema}}));
            let version = slot(json!({"literal":{"string":vector.extension_version}}));
            format!("({extra}) AND EXISTS(SELECT 1 FROM pg_catalog.pg_attribute AS a JOIN pg_catalog.pg_type AS ty ON ty.oid=a.atttypid JOIN pg_catalog.pg_namespace AS tn ON tn.oid=ty.typnamespace JOIN pg_catalog.pg_depend AS dep ON dep.classid='pg_catalog.pg_type'::pg_catalog.regclass AND dep.objid=ty.oid AND dep.refclassid='pg_catalog.pg_extension'::pg_catalog.regclass AND dep.deptype='e' JOIN pg_catalog.pg_extension AS ext ON ext.oid=dep.refobjid WHERE a.attrelid=t.oid AND a.attname={column} AND NOT a.attisdropped AND ty.typname='vector' AND tn.nspname={schema} AND a.atttypmod={} AND ext.extname='vector' AND ext.extversion={version})",vector.dimensions)
        } else {
            extra
        }
    } else {
        extra
    };
    let admission = json!({"sql":format!("SELECT ({common} AND ({extra})) AS correspondence FROM pg_catalog.pg_index AS i JOIN pg_catalog.pg_class AS idx ON idx.oid=i.indexrelid JOIN pg_catalog.pg_namespace AS ins ON ins.oid=idx.relnamespace JOIN pg_catalog.pg_class AS t ON t.oid=i.indrelid JOIN pg_catalog.pg_namespace AS ns ON ns.oid=t.relnamespace JOIN pg_catalog.pg_am AS am ON am.oid=idx.relam JOIN pg_catalog.pg_opclass AS opc ON opc.oid=i.indclass[0] JOIN pg_catalog.pg_namespace AS opns ON opns.oid=opc.opcnamespace WHERE ins.nspname={app} AND ns.nspname={app} AND idx.relname={native_index} AND t.relname={table_name}"),"parameters":admission_parameters});
    let mut result = json!({"version":"weft.search/0.1","status":"compiled","profile":"umf.access.search/0.1","request":r,"target":r.selection.target,"source":r.source,"binding":binding,"index":index,"nativeIndex":admitted.binding(),"mechanism":mechanism,"scoreKind":kind,"queryParameter":r.query_parameter,"topK":r.top_k,"approximate":false,"partial":admitted.binding()["filtering"]=="partial","nativeAdmission":admission,"preflights":preflights,"statement":{"sql":sql,"parameters":parameters},"outputs":outputs,"keyOutputs":key_outputs,"selectionTemplate":read,"obligations":{"transaction":"repeatable-read-read-only","sameStatementAuthorization":true,"nativeIndexCorrespondence":"host-observed-original-index-and-column-types","filtering":"before-ranking-complete-matching-set","exactTieBreak":"complete-selected-Key-ascending","timeout":"one-call-deadline","noEmbeddingService":true}});
    let digest = weft_core::json::sha256(weft_core::mutation::canonical(&result).as_bytes());
    result["structuralDigest"] =
        json!({"owner":"weft-postgresql-search","version":"0.1","sha256":digest});
    Ok(result)
}
pub fn compile_json(raw: &str) -> String {
    match compile(raw) {
        Ok(v) => v.to_string(),
        Err(reason) => {
            json!({"version":"weft.search/0.1","status":"blocked","diagnostics":[{"code":reason}]})
                .to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(name: &str) -> Json {
        serde_json::from_str(match name {
            "vector" => include_str!("../../../tests/native-search/vector-request.json"),
            _ => include_str!("../../../tests/native-search/keyword-request.json"),
        })
        .unwrap()
    }
    fn output(r: Json) -> Json {
        serde_json::from_str(&compile_json(&r.to_string())).unwrap()
    }
    #[test]
    fn exact_search_owns_filter_snapshot_and_native_admission() {
        for name in ["vector", "keyword"] {
            let r = request(name);
            let a = output(r.clone());
            assert_eq!(a, output(r));
            assert_eq!(a["status"], "compiled");
            assert_eq!(a["selectionTemplate"]["version"], "weft.search-input/0.1");
            assert!(a["selectionTemplate"]["rowCap"].is_null());
            assert!(a["statement"]["sql"]
                .as_str()
                .unwrap()
                .contains("MATERIALIZED"));
            assert!(!a["nativeAdmission"]["parameters"]
                .as_array()
                .unwrap()
                .is_empty());
            assert_eq!(a["approximate"], false);
        }
    }
    #[test]
    fn vector_native_version_is_exact_and_retained() {
        for version in ["0.8.6", "0.8.7"] {
            let mut r = request("vector");
            r["vectorBinding"]["extensionVersion"] = json!(version);
            let a = output(r);
            assert_eq!(a["status"], "compiled", "{a}");
            assert_eq!(a["request"]["vectorBinding"]["extensionVersion"], version);
            assert!(a["nativeAdmission"]["parameters"].as_array().unwrap().iter()
                .any(|p| p["source"]["literal"]["string"] == version));
        }
    }
    #[test]
    fn unsupported_selected_meaning_never_publishes_sql() {
        for kind in 0..5 {
            let mut r = request("vector");
            match kind {
                0 => r["source"]["documentSha256"] = json!("0".repeat(64)),
                1 => r["topK"] = json!(1001),
                2 => r["vectorBinding"]["extensionVersion"] = json!("unknown"),
                3 => r["selection"]["target"] = json!("spark-delta"),
                _ => r["unknown"] = json!(true),
            }
            let a = output(r);
            assert_eq!(a["status"], "blocked");
            assert!(a.get("statement").is_none());
        }
    }
}
