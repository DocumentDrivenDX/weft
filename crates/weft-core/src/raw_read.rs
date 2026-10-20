// SPDX-License-Identifier: MIT OR Apache-2.0
//! Owned, pinned frontend admission for the experimental raw-read profile.
//! Backends borrow this artifact; JSON, UMF identity and SQL parsing stay in core.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use sqlparser::{
    ast::{Query, Statement},
    dialect::PostgreSqlDialect,
    parser::Parser,
};
use std::collections::BTreeMap;
type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Carrier {
    Integer,
    Decimal,
    String,
    Boolean,
    Uuid,
    Bits128,
    Date,
    Timestamp,
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
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub module: Option<String>,
    pub source: Vec<String>,
    pub target: Vec<String>,
    pub pairs: Vec<[String; 2]>,
    pub to_one: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all="kebab-case")]
pub enum SchemaBinding { Qualified, SearchPath }
/// Explicit native deployment facts; never a grant or authentication credential.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeCatalogFence {
    pub version:String,
    pub schema:String,
    pub view:String,
    pub owner:String,
    pub owner_profile:String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Binding {
    pub version: String,
    pub schema: Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub schema_binding: Option<SchemaBinding>,
    pub tables: Vec<Table>,
    pub relationships: Vec<Relationship>,
    pub model_sha256: String,
    pub model_json: String,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub native_columns: Option<Json>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub native_catalog_fence: Option<NativeCatalogFence>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub native_lifecycle: Option<crate::table_lifecycle::NativeLifecycle>,
}
pub(crate) fn validate_catalog_fence(binding:&Binding,target:&str)->Result<()> {
    if let Some(fence)=&binding.native_catalog_fence {
        if !matches!(target,"postgresql18"|"lakebase18")||fence.version!="umf.pg-ts-fence/0.1"||fence.owner_profile!="non-login-maintainer/0.1" {return Err("unsupported native catalog fence profile")}
        for name in [&fence.schema,&fence.view,&fence.owner] {if name.is_empty()||name.len()>63||name.contains('\0'){return Err("native catalog fence identifier outside profile")}}
    }
    Ok(())
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all="lowercase")]
pub enum ParameterContainer { List }
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    pub carrier: Carrier,
    pub nullable: bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub container: Option<ParameterContainer>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub element_nullable: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_items: Option<u16>,
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
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub verification_profile: Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub snapshots: Option<BTreeMap<String,u64>>,
    #[serde(default)]
    pub keyset: bool,
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    pub stream_page_size: bool,
    #[serde(default, skip_serializing_if="snapshot_false")]
    pub snapshot: bool,
}
fn snapshot_false(value:&bool)->bool{!*value}
pub(crate) fn validate_model(binding: &Binding) -> Result<()> {
    let model=prepare_model(binding)?;
    if model.lifecycle.is_some(){return Err("native table action profile is not yet admitted")}
    Ok(())
}

/// Resolve a logical Record through the same original binding admission used by reads.
/// This exposes source facts only and grants no native execution authority.
pub fn binding_record(binding:&Binding,logical_table:&[String])->Result<crate::model::Record>{
 prepare_model(binding)?.records.get(logical_table).cloned().ok_or("table outside admitted binding")
}

#[derive(Debug)]
pub(crate) struct AdmittedModel {
    pub(crate) document: Json,
    pub(crate) records: BTreeMap<Vec<String>, crate::model::Record>,
    pub(crate) lifecycle:Option<crate::table_lifecycle::AdmittedLifecycle>,
}

fn prepare_model(binding: &Binding) -> Result<AdmittedModel> {
    prepare_operation_model(binding,None)
}
pub(crate) fn prepare_operation_model(binding: &Binding, operation:Option<&str>) -> Result<AdmittedModel> {
    if binding.tables.iter().flat_map(|t| &t.columns).any(|c| matches!(c.carrier, Carrier::Date | Carrier::Timestamp))
        && (operation.is_none() || binding.native_lifecycle.is_none())
    { return Err("temporal carriers require declared native lifecycle operations"); }
    let doc = crate::json::checked_json(&binding.model_json).map_err(|_| "model JSON refused")?;
    if crate::json::sha256(binding.model_json.as_bytes()) != binding.model_sha256 {
        return Err("model source pin mismatch");
    }
    let modules = doc["modules"].as_array().ok_or("model modules")?;
    let version = doc["umf"].as_str().ok_or("model UMF version")?;
    let input = crate::model::ModuleInput {
        document_json: binding.model_json.clone(),
        pin: crate::ir::ModelPin {
            document_id: doc["id"].as_str().ok_or("model ID")?.into(),
            revision: binding.model_sha256.clone(),
            umf_version: version.into(),
            sha256: binding.model_sha256.clone(),
        },
        selected_module_ids: modules
            .iter()
            .map(|m| m["id"].as_str().unwrap_or("").to_owned())
            .collect(),
    };
    let catalog = match version {
        "0.8.0" => crate::model::Catalog::prepare_security(vec![input]),
        "0.10.0" => if operation.is_some() { crate::model::Catalog::prepare_table_operation010(vec![input]) } else { crate::model::Catalog::prepare_raw_read010(vec![input]) },
        _ => return Err("raw-read supports original UMF core 0.8.0 and 0.10.0 sources"),
    }.map_err(|_| "UMF core envelope/profile refused")?;
    let n = |value: &str| crate::syntax::Name {
        value: value.into(),
        quoted: true,
        span: crate::ir::Span { start: 0, end: 0 },
    };
    let mut records = BTreeMap::new();
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
                if catalog.query_name(field).is_some_and(|name| name == c.logical) {
                    matches.push((field, member));
                }
            }
            if matches.len() != 1 {
                return Err("column absent/ambiguous in UMF");
            };
            let (field, member) = matches[0];
            (if operation.is_some() { crate::model::selected010_table_operation(&record.pin, field) } else { crate::model::selected010_raw_read(&record.pin, field) }).map_err(|_| "selected UMF 0.10 field has unsupported meaning")?;
            if version != "0.10.0" && field.get("extensions").and_then(Json::as_object).is_some_and(|m| !m.is_empty()) {
                return Err("column meaning outside scalar read profile");
            }
            let family = match c.carrier {
                Carrier::Integer => "integer",
                Carrier::Decimal => "decimal",
                Carrier::String => "string",
                Carrier::Boolean => "boolean",
                Carrier::Uuid => "uuid",
                Carrier::Bits128 => "bits128",
                Carrier::Date => "date",
                Carrier::Timestamp => "timestamp",
            };
            if matches!(c.carrier, Carrier::Uuid | Carrier::Bits128)
                && (version != "0.10.0"
                    || field.get("facets").is_some_and(|v|v != &json!({}))
                    || field.get("semanticTypes").is_some())
            { return Err("UMF 0.10 UUID/bits128 native refinements or semantic references are unsupported"); }
            if field["kind"] != "field"
                || field["cardinality"] != "one"
                || field["scalarType"] != family
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
        if records.insert(table.logical.clone(), record).is_some(){return Err("ambiguous duplicate logical table binding")}
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
            .filter(|m|rel.module.as_ref().is_none_or(|name|m["id"]==name.as_str()))
            .flat_map(|m| m["relationships"].as_array().into_iter().flatten())
            .filter(|r| r["id"] == rel.id)
            .collect();
        if declared.len() != 1 {
            return Err("relationship identity absent/ambiguous in UMF");
        };
        let d = declared[0];
        if d.as_object().is_none_or(|m|m.keys().any(|k|!matches!(k.as_str(),"id"|"name"|"title"|"description"|"source"|"target"|"sourceMultiplicity"|"targetMultiplicity"|"targetLifecycle"|"directed"|"inverse"|"associationRecord"|"extensions")))
            || !matches!(d["targetLifecycle"].as_str(),Some("owned"|"independent"))
            || d["source"].as_array().is_none_or(|items|items.iter().any(|r|r.as_object().is_none_or(|m|m.keys().any(|k|!matches!(k.as_str(),"module"|"element")))))
            || d["target"].as_array().is_none_or(|items|items.iter().any(|r|r.as_object().is_none_or(|m|m.keys().any(|k|!matches!(k.as_str(),"module"|"element"|"key")))))
            || ["sourceMultiplicity","targetMultiplicity"].iter().any(|key|d[*key].as_object().is_none_or(|m|m.keys().any(|k|!matches!(k.as_str(),"min"|"max"))))
        {return Err("selected original raw relationship contains unsupported meaning")}
        let target_key=target.value["keys"].as_array().and_then(|keys|keys.iter().find(|k|k["id"]==d["target"][0]["key"])).ok_or("relationship target Key absent")?;
        let target_fields=target_key["fields"].as_array().ok_or("relationship target Key fields absent")?.iter().map(|reference| {
            let module=modules.iter().find(|m|m["id"]==reference["module"]).ok_or("relationship target Field module absent")?;
            let field=module["elements"].as_array().unwrap().iter().find(|f|f["id"]==reference["element"]).ok_or("relationship target Field absent")?;
            let logical=catalog.query_name(field).ok_or("relationship target Field name absent")?;
            if !binding.tables.iter().find(|t|t.logical==rel.target).unwrap().columns.iter().any(|c|c.logical==logical){return Err("relationship target Key field unbound")}
            Ok(logical.to_string())
        }).collect::<Result<Vec<_>>>()?;
        for pair in &rel.pairs {
            let source_table=binding.tables.iter().find(|t|t.logical==rel.source).unwrap();let target_table=binding.tables.iter().find(|t|t.logical==rel.target).unwrap();
            let source_column=source_table.columns.iter().find(|c|c.logical==pair[0]).ok_or("relationship source Field unbound")?;
            let target_column=target_table.columns.iter().find(|c|c.logical==pair[1]).ok_or("relationship target Field unbound")?;
            if source_column.carrier!=target_column.carrier{return Err("relationship native logical carrier mismatch")}
        }
        if rel.pairs.iter().map(|p|p[1].clone()).collect::<Vec<_>>()!=target_fields
            || d.get("extensions").is_some_and(|e|e.as_object().is_none_or(|m|!m.is_empty()))
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
    let lifecycle=crate::table_lifecycle::admit(binding,&doc,&records,operation)?;
    Ok(AdmittedModel { document: doc, records, lifecycle })
}

#[derive(Debug)]
pub struct AdmittedRead {
    request: Request,
    query: Query,
    model: AdmittedModel,
}
impl AdmittedRead {
    pub fn native_lifecycle(&self)->Option<&crate::table_lifecycle::AdmittedLifecycle>{self.model.lifecycle.as_ref()}
    pub fn request(&self) -> &Request {
        &self.request
    }
    pub fn query(&self) -> &Query {
        &self.query
    }
    /// The exact parsed source whose bytes and core semantics passed admission.
    pub fn original_document(&self) -> &Json {
        &self.model.document
    }
    /// Resolve a logical binding path through the admitted Catalog, never by ID guessing.
    pub fn original_record(&self, logical_table: &[String]) -> Result<&crate::model::Record> {
        self.model.records.get(logical_table).ok_or("table outside admitted binding")
    }
    /// Resolve even an unbound projection Field through original Record membership.
    pub fn original_field_ref(&self, logical_table: &[String], logical_column: &str) -> Result<Json> {
        let record = self.original_record(logical_table)?;
        let modules = self.model.document["modules"].as_array().ok_or("model modules")?;
        let members = record.value["members"].as_array().ok_or("record members")?;
        let mut found = None;
        for member in members {
            let module = modules.iter().find(|m| m["id"] == member["module"])
                .ok_or("missing member module")?;
            let field = module["elements"].as_array().ok_or("module elements")?.iter()
                .find(|f| f["id"] == member["element"]).ok_or("missing member")?;
            let query_name = field["name"].as_str().or_else(|| {
                (self.model.document["umf"] == "0.10.0").then(|| field["id"].as_str()).flatten()
            });
            if field["kind"] == "field" && query_name == Some(logical_column) {
                if found.is_some() { return Err("column absent/ambiguous in UMF"); }
                found = Some(member.clone());
            }
        }
        found.ok_or("column absent/ambiguous in UMF")
    }
}
pub fn admit_json(raw: &str) -> Result<AdmittedRead> {
    let v = crate::json::checked_json(raw).map_err(|_| "invalid or duplicate JSON")?;
    let request: Request = serde_json::from_value(v).map_err(|_| "invalid read request")?;
    if request.binding.tables.iter().flat_map(|t|&t.columns).any(|c|matches!(c.carrier,Carrier::Uuid|Carrier::Bits128))
        && !matches!(request.target.as_str(),"postgresql18"|"lakebase18")
    { return Err("UMF 0.10 UUID/bits128 raw carriers are currently PostgreSQL-only"); }
    validate_catalog_fence(&request.binding,&request.target)?;
    if let Some(profile)=&request.verification_profile {
        if !matches!(profile.as_str(),"source-exact/0.1"|"table-lifecycle/0.1") || !matches!(request.target.as_str(),"postgresql18"|"lakebase18") {return Err("unsupported source verification profile")}
        if profile=="source-exact/0.1"&&request.binding.native_columns.is_none(){return Err("source-exact verification requires original native ColumnSet facts")}
    }
    if request.snapshot && (!request.keyset || !matches!(request.target.as_str(), "spark-delta4" | "unity-catalog")){return Err("snapshot pages require an explicit Delta snapshot target")}
    if request.version != "weft.raw-read/0.1"
        || request.sql.len() > 65536
        || request.parameters.len() > 64
        || request.binding.tables.len() > 256
    {
        return Err("unsupported raw-read version or input budget");
    }
    for parameter in &request.parameters {
        if parameter.container.is_some() {
            if !matches!(request.target.as_str(),"postgresql18"|"lakebase18"|"spark-delta4") || parameter.nullable || parameter.element_nullable.is_none() || !parameter.max_items.is_some_and(|n|n>0&&n<=1000){return Err("list parameter requires an exact bounded nonnullable membership profile")}
        } else if parameter.element_nullable.is_some()||parameter.max_items.is_some(){return Err("scalar parameter cannot carry list facets")}
    }
    let model = prepare_model(&request.binding)?;
    if model.lifecycle.is_some()!=(request.verification_profile.as_deref()==Some("table-lifecycle/0.1")){return Err("native table lifecycle requires its explicit verification profile")}
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
        model,
    })
}

#[cfg(test)]
mod source_identity_tests {
    use super::*;

    #[test]
    fn original_umf010_uuid_and_bits128_source_is_retained_and_pinned() {
        let mut request: Json = serde_json::from_str(include_str!("../../../tests/raw-read/source-identity.json")).unwrap();
        let mut doc: Json = serde_json::from_str(request["binding"]["modelJson"].as_str().unwrap()).unwrap();
        doc["umf"] = json!("0.10.0");
        let elements = doc["modules"][0]["elements"].as_array_mut().unwrap();
        for field in elements {
            match field["id"].as_str() {
                Some("Order_id") => field["scalarType"] = json!("uuid"),
                Some("Order_status") => field["scalarType"] = json!("bits128"),
                _ => {}
            }
        }
        let source = serde_json::to_string(&doc).unwrap();
        let digest = crate::json::sha256(source.as_bytes());
        request["binding"]["modelSha256"] = json!(digest);
        request["binding"]["modelJson"] = json!(source);
        request["binding"]["tables"].as_array_mut().unwrap().truncate(1);
        request["binding"]["tables"][0]["columns"][0]["carrier"] = json!("uuid");
        request["binding"]["tables"][0]["columns"][2]["carrier"] = json!("bits128");
        request["binding"]["relationships"] = json!([]);
        request["target"] = json!("postgresql18");
        request["sql"] = json!("SELECT o.id AS id, o.status AS status FROM commerce.Order AS o WHERE o.id = :id AND o.status = :opaque");
        request["parameters"] = json!([
            {"name":"id","carrier":"uuid","nullable":false},
            {"name":"opaque","carrier":"bits128","nullable":false}
        ]);
        request["keyset"] = json!(false);
        request.as_object_mut().unwrap().remove("snapshots");

        let admitted = admit_json(&request.to_string()).expect("UMF 0.10 exact UUID/128 source admitted");
        assert_eq!(admitted.request().binding.model_sha256, digest);
        assert_eq!(crate::json::sha256(admitted.request().binding.model_json.as_bytes()), digest);
        assert_eq!(admitted.original_document(), &doc);
        let table = vec!["commerce".into(), "Order".into()];
        let record = admitted.original_record(&table).unwrap();
        assert_eq!(record.pin.umf_version, "0.10.0");
        assert_eq!(admitted.original_field_ref(&table, "id").unwrap(), json!({"module":"commerce","element":"Order_id"}));
        assert_eq!(admitted.original_field_ref(&table, "status").unwrap(), json!({"module":"commerce","element":"Order_status"}));
    }

    #[test]
    fn resolved_source_identity_is_not_logical_name_guessing() {
        let mut request: Json = serde_json::from_str(include_str!("../../../tests/raw-read/source-identity.json")).unwrap();
        let mut doc: Json = serde_json::from_str(request["binding"]["modelJson"].as_str().unwrap()).unwrap();
        fn remap(value: &mut Json) {
            match value {
                Json::String(s) => {
                    *s = match s.as_str() {
                        "commerce" => "module-identity".into(),
                        "Order" => "record-order-identity".into(),
                        "Customer" => "record-customer-identity".into(),
                        _ => s.clone(),
                    };
                }
                Json::Array(a) => a.iter_mut().for_each(remap),
                Json::Object(o) => o.values_mut().for_each(remap),
                _ => {}
            }
        }
        remap(&mut doc);
        doc["modules"][0]["namespace"] = json!("commerce");
        for element in doc["modules"][0]["elements"].as_array_mut().unwrap() {
            if element["id"] == "record-order-identity" { element["name"] = json!("Order"); }
            if element["id"] == "record-customer-identity" { element["name"] = json!("Customer"); }
        }
        // Status is a real original member omitted from the flat scalar binding.
        request["binding"]["tables"][0]["columns"].as_array_mut().unwrap()
            .retain(|c| c["logical"] != "status");
        let source = serde_json::to_string(&doc).unwrap();
        request["binding"]["modelSha256"] = json!(crate::json::sha256(source.as_bytes()));
        request["binding"]["modelJson"] = json!(source);
        let admitted = admit_json(&request.to_string()).unwrap();
        let table = vec!["commerce".into(), "Order".into()];
        let record = admitted.original_record(&table).unwrap();
        assert_eq!(record.identity.module, "module-identity");
        assert_eq!(record.identity.element, "record-order-identity");
        assert_eq!(admitted.original_document(), &doc);
        assert_eq!(admitted.original_field_ref(&table, "status").unwrap(),
            json!({"module":"module-identity", "element":"Order_status"}));
        assert!(admitted.original_record(&["module-identity".into(), "record-order-identity".into()]).is_err());
        assert!(admitted.original_field_ref(&table, "missing").is_err());
        assert!(admitted.original_field_ref(&table, "Customer_name").is_err());
        // Callers receive owned references, not mutation access to the retained source.
        let mut reference = admitted.original_field_ref(&table, "status").unwrap();
        reference["element"] = json!("tampered");
        assert_eq!(admitted.original_field_ref(&table, "status").unwrap()["element"], "Order_status");
    }
}
