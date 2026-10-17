//! Exact source-bound search admission. Backend mechanics stay out of core.
use crate::{
    json::{checked_json, sha256},
    raw_read::{self, AdmittedRead, Request},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Source {
    pub document_id: String,
    pub core_version: String,
    pub document_sha256: String,
    pub module: String,
    pub pointer: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Projection {
    pub field: Value,
    pub name: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VectorBinding {
    pub field: Value,
    pub native: String,
    pub dimensions: u16,
    pub extension_schema: String,
    pub extension_version: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SearchRequest {
    pub version: String,
    pub source: Source,
    pub selection: Request,
    pub query_parameter: String,
    pub top_k: u16,
    pub projection: Vec<Projection>,
    pub vector_binding: Option<VectorBinding>,
    pub allow_partial: bool,
}
pub struct AdmittedSearch {
    request: SearchRequest,
    selection: AdmittedRead,
    index: Value,
    binding: Value,
    record: Value,
}
impl AdmittedSearch {
    pub fn request(&self) -> &SearchRequest {
        &self.request
    }
    pub fn selection(&self) -> &AdmittedRead {
        &self.selection
    }
    pub fn index(&self) -> &Value {
        &self.index
    }
    pub fn binding(&self) -> &Value {
        &self.binding
    }
    pub fn record(&self) -> &Value {
        &self.record
    }
}
fn exact(v: &Value, required: &[&str], optional: &[&str]) -> Result<(), &'static str> {
    let o = v.as_object().ok_or("search object required")?;
    if required.iter().any(|k| !o.contains_key(*k))
        || o.keys()
            .any(|k| !required.contains(&k.as_str()) && !optional.contains(&k.as_str()))
    {
        return Err("unknown selected search meaning");
    };
    Ok(())
}
fn reference(v: &Value) -> Result<(), &'static str> {
    exact(v, &["module", "element"], &[])?;
    if ["module", "element"]
        .iter()
        .any(|k| v[*k].as_str().is_none_or(str::is_empty))
    {
        return Err("exact search Field reference required");
    };
    Ok(())
}
fn locate<'a>(doc: &'a Value, r: &Value) -> Result<&'a Value, &'static str> {
    reference(r)?;
    doc["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == r["module"])
        .and_then(|m| {
            m["elements"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["id"] == r["element"])
        })
        .ok_or("search reference absent")
}
pub fn admit_json(raw: &str) -> Result<AdmittedSearch, &'static str> { admit_profile(raw,false,None) }
pub fn admit_unity_json(raw:&str,target:&Value)->Result<AdmittedSearch,&'static str>{admit_profile(raw,false,Some(target))}
fn admit_profile(raw: &str, terms:bool, unity:Option<&Value>) -> Result<AdmittedSearch, &'static str> {
    let r: SearchRequest =
        serde_json::from_value(checked_json(raw).map_err(|_| "bounded search JSON required")?)
            .map_err(|_| "exact search request required")?;
    if r.version != "weft.search/0.1"
        || !(1..=1000).contains(&r.top_k)
        || (r.selection.keyset && !terms)
        || r.query_parameter.is_empty()
        || r.query_parameter.len() > 64
        || r.selection
            .parameters
            .iter()
            .any(|p| p.name == r.query_parameter)
        || r.projection.is_empty()
        || r.projection.len() > 256
    {
        return Err("search profile or budget refused");
    }
    let selection = raw_read::admit_json(&serde_json::to_string(&r.selection).unwrap())?;
    let doc = checked_json(&r.selection.binding.model_json)
        .map_err(|_| "original search source refused")?;
    if r.source.document_id != doc["id"]
        || r.source.core_version != "0.8.0"
        || r.source.document_sha256 != r.selection.binding.model_sha256
        || r.source.document_sha256 != sha256(r.selection.binding.model_json.as_bytes())
        || doc["vocabularies"]["umf.access"]["version"] != "0.1.0"
    {
        return Err("original search source pin differs");
    }
    let mut selected = None;
    for (mi, module) in doc["modules"].as_array().unwrap().iter().enumerate() {
        if module["id"] != r.source.module {
            continue;
        }
        let payload = &module["extensions"]["umf.access"];
        let indices = payload["search"]
            .as_array()
            .ok_or("original search declarations absent")?;
        for (ii, index) in indices.iter().enumerate() {
            if r.source.pointer == format!("/modules/{mi}/extensions/umf.access/search/{ii}") {
                selected = Some((index, payload));
            }
        }
    }
    let (index, payload) = selected.ok_or("exact search source pointer absent")?;
    exact(
        index,
        &["id", "record", "kind", "exactness", "filterable"],
        &["vector", "keyword"],
    )?;
    if index["id"]
        .as_str()
        .is_none_or(|s| s.is_empty() || s.len() > 256)
        || !matches!(index["kind"].as_str(), Some("keyword" | "vector"))
        || index["exactness"] != if unity.is_some(){"approximate"}else{"exact"}
    {
        return Err("SEARCH_UNSUPPORTED: initial exact native profile required");
    }
    let record = locate(&doc, &index["record"])?;
    if record["extensions"]
        .as_object()
        .is_none_or(|o| !o.is_empty())
    {
        return Err("selected search Record extension unsupported");
    };
    if record["kind"] != "record" {
        return Err("search owner must be Record");
    }
    let members = record["members"]
        .as_array()
        .ok_or("search Record inventory absent")?;
    let filterable = index["filterable"]
        .as_array()
        .ok_or("filterable Field inventory required")?;
    if filterable.len() > 256
        || filterable
            .iter()
            .map(crate::mutation::canonical)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != filterable.len()
    {
        return Err("filterable inventory budget/duplicate");
    };
    for field in filterable {
        if !members.contains(field) {
            return Err("search filter Field outside Record");
        };
        locate(&doc, field)?;
    }
    let platform = match r.selection.target.as_str() {
        "postgresql18" => "postgresql",
        "lakebase18" => "lakebase",
        "unity-catalog-current" if unity.is_some()=>"unity-catalog",
        _ => return Err("SEARCH_UNSUPPORTED: native target not qualified"),
    };
    let bindings = payload["searchBindings"]
        .as_array()
        .ok_or("search native bindings absent")?;
    let found = bindings
        .iter()
        .filter(|b| {
            b["index"] == index["id"]
                && b["target"] == unity.cloned().unwrap_or_else(||json!({"platform":platform,"majorVersion":"18"}))
        })
        .collect::<Vec<_>>();
    if found.len() != 1 {
        return Err("SEARCH_UNSUPPORTED: unique native binding required");
    };
    let binding = found[0];
    exact(
        binding,
        &[
            "index",
            "target",
            "native",
            "freshness",
            "maxK",
            "filtering",
        ],
        &[],
    )?;
    if binding["freshness"] != if unity.is_some(){"asynchronous"}else{"transactional"}
        || binding["native"].as_str().is_none_or(str::is_empty)
        || binding["maxK"]
            .as_u64()
            .is_none_or(|k| k < u64::from(r.top_k))
        || !matches!(binding["filtering"].as_str(), Some("complete" | "partial"))
        || binding["filtering"] == "partial" && !r.allow_partial
    {
        return Err("SEARCH_FILTER: native binding/freshness/limit refused");
    }
    let mut names = std::collections::BTreeSet::new();
    for p in &r.projection {
        if !members.contains(&p.field)
            || p.name.is_empty()
            || p.name.starts_with("__umf_")
            || !names.insert(&p.name)
        {
            return Err("search projection inventory differs");
        };
        locate(&doc, &p.field)?;
    }
    if index["kind"] == "keyword" {
        if index.get("vector").is_some() {
            return Err("keyword/vector meaning differs");
        };
        let keyword = &index["keyword"];
        exact(keyword, &["source", "ranking", "analyzer", "syntax"], &[])?;
        exact(&keyword["ranking"], &["family"], &[])?;
        if keyword["ranking"]["family"] != "implementation-defined" || keyword["syntax"] != "plain"
        {
            return Err("SEARCH_UNSUPPORTED: PostgreSQL keyword ranking requires implementation-defined plain semantics");
        }
        if unity.is_some() && keyword["analyzer"] != "implementation-defined" {return Err("UC native analyzer must be explicitly implementation-defined")};
        if keyword["analyzer"] != "implementation-defined" {
            exact(&keyword["analyzer"], &["language", "stemming"], &[])?;
            if keyword["analyzer"] != json!({"language":"english","stemming":true})
                && keyword["analyzer"] != json!({"language":"simple","stemming":false})
            {
                return Err("SEARCH_UNSUPPORTED: analyzer configuration unavailable");
            }
        }
        let sources = keyword["source"]
            .as_array()
            .ok_or("keyword source inventory required")?;
        if sources.is_empty() || sources.len() > 64 {
            return Err("keyword source budget");
        };
        for field in sources {
            let f = locate(&doc, field)?;
            if !members.contains(field)
                || f["scalarType"] != "string"
                || f["cardinality"] != "one"
                || f["extensions"].as_object().is_none_or(|o| !o.is_empty())
            {
                return Err("keyword source must be owned scalar text");
            }
        }
    }
    if index["kind"] == "vector" || r.vector_binding.is_some() {
        if index["kind"] == "vector" && index.get("keyword").is_some() {
            return Err("vector/keyword meaning differs");
        };
        let vector = if index["kind"] == "vector" {
            &index["vector"]
        } else {
            let native = r.vector_binding.as_ref().unwrap();
            &payload["search"]
                .as_array()
                .unwrap()
                .iter()
                .find(|entry| {
                    entry["kind"] == "vector"
                        && entry["record"] == index["record"]
                        && entry["vector"]["field"] == native.field
                })
                .ok_or("projected vector requires its original authored index")?["vector"]
        };
        exact(
            vector,
            &["field", "model", "dimensions", "metric", "computedBy"],
            &["source"],
        )?;
        exact(&vector["model"], &["id", "version"], &[])?;
        if !matches!(vector["metric"].as_str(), Some("cosine" | "dot" | "l2"))
            || !matches!(vector["computedBy"].as_str(), Some("host" | "target"))
            || vector["model"]["id"].as_str().is_none_or(str::is_empty)
            || vector["model"]["version"]
                .as_str()
                .is_none_or(str::is_empty)
        {
            return Err("vector semantic profile unsupported");
        }
        let field = locate(&doc, &vector["field"])?;
        let leaf = locate(&doc, &field["itemType"])?;
        let native = r
            .vector_binding
            .as_ref()
            .ok_or("explicit native vector column binding required")?;
        if !members.contains(&vector["field"])
            || field["cardinality"] != "array"
            || leaf["scalarType"] != "float"
            || leaf["cardinality"] != "one"
            || leaf["nullability"] != "required"
            || field["facets"]["collectionSize"]
                != json!({"min":native.dimensions,"max":native.dimensions})
            || vector["dimensions"] != native.dimensions
            || native.field != vector["field"]
            || native.dimensions == 0
            || native.dimensions > 16000
            || (unity.is_none() && !matches!(native.extension_version.as_str(), "0.8.6" | "0.8.7"))
            || (unity.is_some() && (vector["metric"]!="l2" || vector["computedBy"]!="host"))
        {
            return Err("SEARCH_UNSUPPORTED: exact pinned pgvector0.8.6 or 0.8.7 shape required");
        }
        for node in [field, leaf] {
            if node["extensions"].as_object().is_none_or(|m| !m.is_empty())
                || node.get("allowedValues").is_some()
            {
                return Err("selected vector Field constraints unsupported");
            }
        }
        if leaf
            .get("facets")
            .is_some_and(|f| f.as_object().is_none_or(|f| !f.is_empty()))
        {
            return Err("selected vector component constraints unsupported");
        }
    }
    Ok(AdmittedSearch {
        index: index.clone(),
        binding: binding.clone(),
        record: record.clone(),
        request: r,
        selection,
    })
}

/// Resolve one explicitly authored single-field keyword index for an unranked
/// raw predicate. This returns metadata only; native correspondence stays a host
/// obligation, and ordinary ranked search keeps its original admission profile.
pub fn admit_terms(read:&raw_read::AdmittedRead,table:&[String],column:&str)->Result<AdmittedSearch,&'static str> {
    if table.len()!=2 {return Err("TERM_INDEX: logical owner required")};
    let request=read.request();
    let doc=checked_json(&request.binding.model_json).map_err(|_|"TERM_INDEX: original source refused")?;
    let modules=doc["modules"].as_array().ok_or("TERM_INDEX: original modules required")?;
    let mut owners=vec![];
    for module in modules {
        if module["namespace"]!=table[0] {continue};
        for record in module["elements"].as_array().ok_or("TERM_INDEX: element inventory required")? {
            if record["kind"]=="record" && record["name"]==table[1] {owners.push((module,record));}
        }
    }
    if owners.len()!=1 {return Err("TERM_INDEX: unique original Record required")};
    let (module,record)=owners[0];
    let record_ref=json!({"module":module["id"],"element":record["id"]});
    let mut fields=vec![];
    for reference in record["members"].as_array().ok_or("TERM_INDEX: original members required")? {
        let field=locate(&doc,reference)?;
        if field["name"]==column {fields.push(reference.clone());}
    }
    if fields.len()!=1 {return Err("TERM_INDEX: unique original Field required")};
    let field_ref=&fields[0];
    let mut found=vec![];
    for (mi,module) in modules.iter().enumerate() {
        let Some(indices)=module["extensions"]["umf.access"]["search"].as_array() else {continue};
        for (ii,index) in indices.iter().enumerate() {
            if index["kind"]=="keyword" && index["record"]==record_ref && index["keyword"]["source"]==json!([field_ref]) {
                found.push((mi,ii,module,index));
            }
        }
    }
    if found.len()!=1 {return Err("TERM_INDEX: one declared single-field keyword index required")};
    let (mi,ii,module,index)=found[0];
    if index["keyword"]["analyzer"]=="implementation-defined" {return Err("TERM_ANALYZER: explicit supported analyzer required")};
    let source=json!({"documentId":doc["id"],"coreVersion":"0.8.0","documentSha256":request.binding.model_sha256,"module":module["id"],"pointer":format!("/modules/{mi}/extensions/umf.access/search/{ii}")});
    let probe=json!({"version":"weft.search/0.1","source":source,"selection":request,"queryParameter":"__umf_term_query","topK":1,"projection":[{"field":field_ref,"name":"term_field"}],"vectorBinding":null,"allowPartial":false});
    admit_profile(&probe.to_string(),true,None)
}

/// Admit each explicitly authored keyword index of a mutation's original Record.
/// The constructed logical Key scan is metadata context only and is never executed.
pub fn admit_keyword_effects(binding:&raw_read::Binding,target:&str,table:&[String])->Result<Vec<AdmittedSearch>,&'static str>{
    raw_read::validate_catalog_fence(binding,target)?;
    let record=raw_read::binding_record(binding,table)?;
    let record_ref=json!({"module":record.identity.module,"element":record.identity.element});
    let original=checked_json(&binding.model_json).map_err(|_|"keyword effects original source refused")?;
    let native=binding.tables.iter().find(|t|t.logical==table).ok_or("keyword effects table unbound")?;
    let quote=|name:&str|format!("\"{}\"",name.replace('"',"\"\""));
    let columns=native.key.iter().map(|k|format!("r.{} AS {}",quote(k),quote(k))).collect::<Vec<_>>().join(", ");
    let logical=table.iter().map(|n|quote(n)).collect::<Vec<_>>().join(".");
    let selection=json!({"version":"weft.raw-read/0.1","target":target,"sql":format!("SELECT {columns} FROM {logical} AS r"),"binding":binding,"parameters":[],"rowCap":1});
    let mut out=vec![];
    for (mi,module) in original["modules"].as_array().ok_or("keyword effects modules absent")?.iter().enumerate(){
        let Some(indexes)=module["extensions"]["umf.access"]["search"].as_array() else{continue};
        for (ii,index) in indexes.iter().enumerate(){
            if index["kind"]!="keyword"||index["record"]!=record_ref{continue}
            let first=index["keyword"]["source"].as_array().and_then(|a|a.first()).ok_or("keyword effects source fields absent")?;
            let source=json!({"documentId":original["id"],"coreVersion":"0.8.0","documentSha256":binding.model_sha256,"module":module["id"],"pointer":format!("/modules/{mi}/extensions/umf.access/search/{ii}")});
            let request=json!({"version":"weft.search/0.1","source":source,"selection":selection,"queryParameter":"__umf_effect_query","topK":1,"projection":[{"field":first,"name":"effect_field"}],"vectorBinding":null,"allowPartial":true});
            out.push(admit_profile(&request.to_string(),false,None)?);
        }
    }
    Ok(out)
}
