//! Owned, pinned frontend admission for the experimental raw-read profile.
//! Backends borrow this artifact; JSON, UMF identity and SQL parsing stay in core.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use sqlparser::{
    ast::{Query, Statement},
    dialect::PostgreSqlDialect,
    parser::Parser,
};
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Carrier {
    Integer,
    Decimal,
    String,
    Boolean,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Column {
    pub logical: String,
    pub native: String,
    pub carrier: Carrier,
    pub nullable: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Table {
    pub logical: Vec<String>,
    pub native: String,
    pub columns: Vec<Column>,
    pub key: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Relationship {
    pub id: String,
    pub source: Vec<String>,
    pub target: Vec<String>,
    pub pairs: Vec<[String; 2]>,
    pub to_one: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Binding {
    pub version: String,
    pub schema: Option<String>,
    pub tables: Vec<Table>,
    pub relationships: Vec<Relationship>,
    pub model_sha256: String,
    pub model_json: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    pub carrier: Carrier,
    pub nullable: bool,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Request {
    pub version: String,
    pub target: String,
    pub sql: String,
    pub binding: Binding,
    pub parameters: Vec<Parameter>,
    pub row_cap: u16,
    #[serde(default)]
    pub keyset: bool,
}
fn validate_model(binding: &Binding) -> Result<()> {
    let doc = crate::json::checked_json(&binding.model_json).map_err(|_| "model JSON refused")?;
    if crate::json::sha256(binding.model_json.as_bytes()) != binding.model_sha256 {
        return Err("model source pin mismatch");
    }
    let modules = doc["modules"].as_array().ok_or("model modules")?;
    let input = crate::model::ModuleInput {
        document_json: binding.model_json.clone(),
        pin: crate::ir::ModelPin {
            document_id: doc["id"].as_str().ok_or("model ID")?.into(),
            revision: binding.model_sha256.clone(),
            umf_version: "0.8.0".into(),
            sha256: binding.model_sha256.clone(),
        },
        selected_module_ids: modules
            .iter()
            .map(|m| m["id"].as_str().unwrap_or("").to_owned())
            .collect(),
    };
    let catalog = crate::model::Catalog::prepare_security(vec![input])
        .map_err(|_| "UMF core 0.8 model refused")?;
    let n = |value: &str| crate::syntax::Name {
        value: value.into(),
        quoted: true,
        span: crate::ir::Span { start: 0, end: 0 },
    };
    let mut records = BTreeMap::new();
    let mut key_ids: BTreeMap<Vec<String>, BTreeSet<String>> = BTreeMap::new();
    for table in &binding.tables {
        if table.logical.len() != 2 {
            return Err("logical table path");
        }
        let record = catalog
            .record(Some(&n(&table.logical[0])), &n(&table.logical[1]))
            .map_err(|_| "table absent/ambiguous in UMF")?;
        let members = record.value["members"].as_array().ok_or("record members")?;
        let mut field_ids = BTreeMap::new();
        for c in &table.columns {
            let mut matches = vec![];
            for member in members {
                let Some(module) = modules.iter().find(|m| m["id"] == member["module"]) else {
                    return Err("missing member module");
                };
                let Some(field) = module["elements"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|f| f["id"] == member["element"])
                else {
                    return Err("missing member");
                };
                if field["name"] == c.logical {
                    matches.push((field, member));
                }
            }
            if matches.len() != 1 {
                return Err("column absent/ambiguous in UMF");
            };
            let (field, member) = matches[0];
            let family = match c.carrier {
                Carrier::Integer => "integer",
                Carrier::Decimal => "decimal",
                Carrier::String => "string",
                Carrier::Boolean => "boolean",
            };
            if field["kind"] != "field"
                || field["cardinality"] != "one"
                || field["scalarType"] != family
                || field["extensions"]
                    .as_object()
                    .is_some_and(|m| !m.is_empty())
            {
                return Err("column meaning outside scalar read profile");
            }
            field_ids.insert(c.logical.clone(), member.clone());
        }
        let key: Vec<Json> = table
            .key
            .iter()
            .map(|k| field_ids.get(k).cloned().ok_or("key field absent"))
            .collect::<Result<_>>()?;
        if !record.value["keys"]
            .as_array()
            .is_some_and(|keys| keys.iter().any(|k| k["fields"] == json!(key)))
        {
            return Err("binding key not declared in UMF");
        }
        key_ids.insert(
            table.logical.clone(),
            record.value["keys"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|k| k["fields"] == json!(key))
                .map(|k| k["id"].as_str().unwrap().to_owned())
                .collect(),
        );
        records.insert(table.logical.clone(), record);
    }
    for rel in &binding.relationships {
        let source = records
            .get(&rel.source)
            .ok_or("relationship source binding")?;
        let target = records
            .get(&rel.target)
            .ok_or("relationship target binding")?;
        let declared: Vec<&Json> = modules
            .iter()
            .flat_map(|m| m["relationships"].as_array().into_iter().flatten())
            .filter(|r| r["id"] == rel.id)
            .collect();
        if declared.len() != 1 {
            return Err("relationship identity absent/ambiguous in UMF");
        };
        let d = declared[0];
        if !key_ids[&rel.target].contains(d["target"][0]["key"].as_str().unwrap_or(""))
            || d["extensions"].as_object().is_some_and(|m| !m.is_empty())
            || d["directed"] != true
            || d["targetMultiplicity"]["max"] != 1
            || !rel.to_one
            || d.get("associationRecord").is_some()
            || d["source"].as_array().is_none_or(|a| a.len() != 1)
            || d["target"].as_array().is_none_or(|a| a.len() != 1)
            || d["source"][0]["module"] != source.identity.module
            || d["source"][0]["element"] != source.identity.element
            || d["target"][0]["module"] != target.identity.module
            || d["target"][0]["element"] != target.identity.element
        {
            return Err("relationship endpoints/multiplicity differ from UMF");
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct AdmittedRead {
    request: Request,
    query: Query,
}
impl AdmittedRead {
    pub fn request(&self) -> &Request {
        &self.request
    }
    pub fn query(&self) -> &Query {
        &self.query
    }
}
pub fn admit_json(raw: &str) -> Result<AdmittedRead> {
    let v = crate::json::checked_json(raw).map_err(|_| "invalid or duplicate JSON")?;
    let request: Request = serde_json::from_value(v).map_err(|_| "invalid read request")?;
    if request.version != "weft.raw-read/0.1"
        || request.sql.len() > 65536
        || request.parameters.len() > 64
        || request.binding.tables.len() > 256
    {
        return Err("unsupported raw-read version or input budget");
    }
    validate_model(&request.binding)?;
    let mut statements =
        Parser::parse_sql(&PostgreSqlDialect {}, &request.sql).map_err(|_| "SQL parse refused")?;
    if statements.len() != 1 {
        return Err("exactly one SELECT required");
    }
    let Statement::Query(query) = statements.remove(0) else {
        return Err("exactly one SELECT required");
    };
    Ok(AdmittedRead {
        request,
        query: *query,
    })
}
