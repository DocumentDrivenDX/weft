// SPDX-License-Identifier: MIT OR Apache-2.0
//! Source-only candidate. This prepares original ordering IR; no backend is enabled.
use crate::{
    application_ir::Plan,
    application_model::{AuthoredKey, Shape},
    application_resolve::{self, Parameters},
    application_syntax as ast,
    error::{Diagnostic, Result},
    ir::{Family, Identity, LogicalType, Span},
    model::Catalog,
    syntax::{Column, Literal, LiteralKind, Name},
};
use serde::{Deserialize, Serialize};

fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-ORDERED-PAGE", "resolve", message)
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectedKey {
    pub module: String,
    pub record: String,
    pub key: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "state", rename_all = "camelCase", deny_unknown_fields)]
pub enum CursorValue {
    Present { carrier: Family, value: String },
    Null {},
    Missing {},
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Cursor {
    pub key: Vec<CursorValue>,
    pub sort_values: Vec<CursorValue>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SortSlot {
    identity: Identity,
    logical_type: LogicalType,
    descending: bool,
    nulls_first: bool,
    nullable: bool,
    missing: bool,
    value_output: usize,
    state_output: usize,
}
impl SortSlot {
    pub fn identity(&self) -> &Identity {
        &self.identity
    }
    pub fn logical_type(&self) -> &LogicalType {
        &self.logical_type
    }
    pub fn descending(&self) -> bool {
        self.descending
    }
    pub fn nulls_first(&self) -> bool {
        self.nulls_first
    }
    pub fn nullable(&self) -> bool {
        self.nullable
    }
    pub fn missing(&self) -> bool {
        self.missing
    }
    pub fn hidden_outputs(&self) -> (usize, usize) {
        (self.value_output, self.state_output)
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AfterBranch {
    /// Prefix equality treats missing and null as the same ordering class.
    equal_slots: Vec<usize>,
    strict_slot: usize,
}
impl AfterBranch {
    pub fn equal_slots(&self) -> &[usize] {
        &self.equal_slots
    }
    pub fn strict_slot(&self) -> usize {
        self.strict_slot
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderedPage {
    version: &'static str,
    source_sql: String,
    base: Plan,
    selected_key: AuthoredKey,
    sort_slots: Vec<SortSlot>,
    key_slots: Vec<usize>,
    page_size: u16,
    // Original full typed values. Native lowering/admission is a separate profile.
    after: Option<Cursor>,
    after_branches: Vec<AfterBranch>,
}
impl OrderedPage {
    pub fn version(&self)->&str{self.version}
    pub(crate) fn base_mut(&mut self)->&mut Plan{&mut self.base}
    pub fn source_sql(&self) -> &str {
        &self.source_sql
    }
    pub fn base(&self) -> &Plan {
        &self.base
    }
    pub fn selected_key(&self) -> &AuthoredKey {
        &self.selected_key
    }
    pub fn sort_slots(&self) -> &[SortSlot] {
        &self.sort_slots
    }
    pub fn key_slots(&self) -> &[usize] {
        &self.key_slots
    }
    pub fn page_size(&self) -> u16 {
        self.page_size
    }
    pub fn after(&self) -> Option<&Cursor> {
        self.after.as_ref()
    }
    /// Original lexicographic disjunction. Native owners must lower every branch
    /// using its slot direction/null rank and exact scalar comparator.
    pub fn after_branches(&self) -> &[AfterBranch] {
        &self.after_branches
    }
    /// Direction and null rank remain separate. This is an IR property, not SQL.
    pub fn null_rank(&self, index: usize, value: &CursorValue) -> Result<u8> {
        let slot = self
            .sort_slots
            .get(index)
            .ok_or_else(|| fail("Sort slot outside original inventory"))?;
        Ok(
            if matches!(value, CursorValue::Null {} | CursorValue::Missing {}) {
                u8::from(!slot.nulls_first)
            } else {
                u8::from(slot.nulls_first)
            },
        )
    }
}
fn slot(
    catalog: &Catalog,
    record: &crate::model::Record,
    column: &Column,
    alias: &Name,
    descending: bool,
    nulls_first: bool,
) -> Result<SortSlot> {
    if !column.unqualified && !column.alias.matches(&alias.value) {
        return Err(fail("Ordered Field belongs to another source occurrence"));
    }
    let (member, graph) = catalog.member_descriptor(record, &column.field)?;
    let descriptor = graph
        .iter()
        .find(|d| d.identity == member.identity)
        .ok_or_else(|| fail("Original ordered Field descriptor absent"))?;
    let Shape::Scalar { logical_type } = &descriptor.shape else {
        return Err(fail(
            "Ordered Field needs a separately admitted non-temporal scalar domain",
        ));
    };
    let missing = descriptor.availability.as_deref() == Some("absent-allowed");
    if !matches!(
        descriptor.availability.as_deref(),
        Some("required" | "absent-allowed")
    ) {
        return Err(fail("Original Field availability absent"));
    }
    Ok(SortSlot {
        identity: member.identity,
        logical_type: logical_type.clone(),
        descending,
        nulls_first,
        nullable: logical_type.nullable || missing,
        missing,
        value_output: 0,
        state_output: 0,
    })
}
fn validate_value(value: &CursorValue, slot: &SortSlot) -> Result<()> {
    match value {
        CursorValue::Null {} if slot.nullable => Ok(()),
        CursorValue::Missing {} if slot.missing => Ok(()),
        CursorValue::Present { carrier, value } if carrier == &slot.logical_type.family => {
            let kind = match carrier {
                Family::String => LiteralKind::String,
                Family::Boolean => {
                    if !matches!(value.as_str(), "true" | "false") {
                        return Err(fail("Boolean cursor token differs from original carrier"));
                    }
                    LiteralKind::Boolean
                }
                Family::Integer => {
                    let n = value.strip_prefix('-').unwrap_or(value);
                    if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
                        return Err(fail("Integer cursor token differs from original carrier"));
                    }
                    LiteralKind::Number
                }
                Family::Decimal => {
                    crate::exact::validate_mathematical_decimal(value)?;
                    LiteralKind::Number
                }
            };
            crate::exact::literal(
                &Literal {
                    value: value.clone(),
                    kind,
                    span: Span { start: 0, end: 0 },
                },
                &slot.logical_type,
            )?;
            Ok(())
        }
        _ => Err(fail(
            "Cursor state or carrier differs from original sort slot",
        )),
    }
}
/// Prepare only source-owned candidate IR. It cannot be passed to old emitters.
/// Host/native uniqueness, binary comparison and hidden disclosure remain unadmitted.
pub fn prepare(
    catalog: &Catalog,
    sql: &str,
    parameters: Parameters,
    selected: &SelectedKey,
    page_size: u16,
    after: Option<Cursor>,
) -> Result<OrderedPage> {
    prepare_inner(catalog,sql,parameters,selected,page_size,after,false)
}
/// Original navigation source ordering. Its explicit source limit remains a
/// source selection bound, never a page/default delivery cap.
pub(crate) fn prepare_source(catalog:&Catalog,sql:&str,parameters:Parameters,selected:&SelectedKey)->Result<(Plan,Vec<crate::application_ir::ReadSortSlot>)>{
    let query=ast::parse_security(sql)?.0;
    if !query.joins.is_empty()&&(!query.order.is_empty()||query.limit.is_some()){return Err(Diagnostic::new("WFT-ORDERED-SOURCE","resolve","Ordered or limited joined source requires separately admitted source identity ordering proof"));}
    let ordered=prepare_inner(catalog,sql,parameters,selected,1,None,true)?;
    let mut base=ordered.base;
    base.required_capabilities.retain(|c|!matches!(c.as_str(),"page.ordered"|"cursor.stateTuple"));
    base.order=ordered.sort_slots.iter().map(|s|crate::application_ir::Field{scan:base.source.occurrence.clone(),identity:s.identity.clone(),logical_type:s.logical_type.clone(),span:Span{start:0,end:0}}).collect();
    Ok((base,ordered.sort_slots.iter().map(crate::application_ir::ReadSortSlot::from_page).collect()))
}
pub(crate) fn prepare_read(catalog:&Catalog,sql:&str,parameters:Parameters,selected:&SelectedKey)->Result<Plan>{
 let query=ast::parse_security(sql)?.0;
 if !query.joins.is_empty(){return Err(Diagnostic::new("WFT-ORDERED-READ","resolve","Ordered joined result requires an original unique result tuple Key profile"));}
 let ordered=prepare_inner(catalog,sql,parameters,selected,1,None,true)?;
 let mut base=ordered.base;
 base.required_capabilities.retain(|c|!matches!(c.as_str(),"page.ordered"|"cursor.stateTuple"));
 base.order=ordered.sort_slots.iter().map(|s|crate::application_ir::Field{scan:base.source.occurrence.clone(),identity:s.identity.clone(),logical_type:s.logical_type.clone(),span:Span{start:0,end:0}}).collect();
 base.ir_version="weft-ir/engine-ordered-read/0.1".into();
 base.read_order=Some(crate::application_ir::ReadOrder::new(ordered.selected_key,ordered.sort_slots.iter().map(crate::application_ir::ReadSortSlot::from_page).collect()));
 Ok(base)
}
fn prepare_inner(catalog:&Catalog,sql:&str,parameters:Parameters,selected:&SelectedKey,page_size:u16,after:Option<Cursor>,source_mode:bool)->Result<OrderedPage>{
    if !(1..=999).contains(&page_size) {
        return Err(fail("Original ordered page size must be1..999"));
    }
    let (query, directions) = ast::parse_security(sql)?;
    if (!source_mode && query.limit.is_some())
        || (!source_mode && !query.joins.is_empty())
        || (!source_mode && query.outputs.iter().any(|o|matches!(o.output,ast::Output::Entity(_))))
        || !query.groups.is_empty()
        || query.outputs.iter().any(|o| {
            matches!(
                o.output,
                ast::Output::Count
                    | ast::Output::CountDistinct(_)
                    | ast::Output::Sum(_)
            )
        })
    {
        return Err(fail(
            "Ordered page candidate needs an explicit single entity source projection without caller LIMIT or aggregate",
        ));
    }
    if query.order.len() != directions.len() || query.order.len() != query.order_nulls.len() {
        return Err(fail("Complete original order facets required"));
    }
    let record = catalog.record(query.source.namespace.as_ref(), &query.source.name)?;
    if selected.module != record.identity.module
        || selected.record != record.identity.element
        || selected.key.is_empty()
    {
        return Err(fail(
            "Selected Key identity belongs to another original Record",
        ));
    }
    let key = catalog.authored_key(&record, &selected.key)?;
    if key.fields.is_empty()
        || key.fields.len() != key.types.len()
        || key.types.iter().any(|t| t.nullable)
    {
        return Err(fail("Complete required original selected Key required"));
    }
    let mut slots = Vec::new();
    for ((column, descending), nulls) in query.order.iter().zip(&directions).zip(&query.order_nulls)
    {
        slots.push(slot(
            catalog,
            &record,
            column,
            &query.source.alias,
            *descending,
            nulls.unwrap_or(false),
        )?);
    }
    let suffix = slots.len() >= key.fields.len()
        && slots[slots.len() - key.fields.len()..]
            .iter()
            .zip(&key.fields)
            .all(|(s, id)| s.identity == *id && !s.descending && !s.nullable);
    if !suffix {
        for id in &key.fields {
            let (member, _) = catalog.member_descriptor_by_identity(&record, id)?;
            let column = Column {
                unqualified: false,
                alias: query.source.alias.clone(),
                field: Name {
                    value: member.name,
                    quoted: true,
                    span: query.source.name.span.clone(),
                },
                span: query.source.name.span.clone(),
            };
            slots.push(slot(
                catalog,
                &record,
                &column,
                &query.source.alias,
                false,
                false,
            )?);
        }
    }
    let key_slots = (slots.len() - key.fields.len()..slots.len()).collect::<Vec<_>>();
    // Validate the ordinary original source/parameters independently. Ordered AST
    // is retained above, and is never replaced by this base plan's empty ordering.
    let mut base_query = query.clone();
    base_query.order.clear();
    base_query.order_nulls.clear();
    let mut base = application_resolve::resolve(catalog, base_query, parameters, None)?;
    // Hidden ordering values remain original selected Field dependencies. Retain
    // descriptors even when the public projection omits them.
    for slot in &slots {
        let (_,graph)=catalog.member_descriptor_by_identity(&record,&slot.identity)?;
        for descriptor in graph {if !base.type_graph.iter().any(|d|d.identity==descriptor.identity){base.type_graph.push(descriptor);}}
    }
    for slot in &slots {let family=match slot.logical_type.family{Family::String=>"string",Family::Boolean=>"boolean",Family::Integer=>"integer",Family::Decimal=>"decimal"};base.required_capabilities.push(format!("type.{family}"));if slot.missing{base.required_capabilities.push("value.presence".into());}}
    base.page_key=Some(key.clone());
    for capability in ["page.ordered","cursor.stateTuple","key.uniqueStable","order.asc"] {if !base.required_capabilities.iter().any(|c|c==capability){base.required_capabilities.push(capability.into());}}
    if slots.iter().any(|s|s.descending){base.required_capabilities.push("order.desc".into());}
    base.required_capabilities.sort();base.required_capabilities.dedup();
    for (i, s) in slots.iter_mut().enumerate() {
        s.value_output = base.outputs.len() + i * 2;
        s.state_output = s.value_output + 1;
    }
    if let Some(cursor) = &after {
        if cursor.sort_values.len() != slots.len() || cursor.key.len() != key.fields.len() {
            return Err(fail(
                "Cursor must retain the complete original effective sort and selected Key inventory",
            ));
        }
        for (value, s) in cursor.sort_values.iter().zip(&slots) {
            validate_value(value, s)?;
        }
        for (value, index) in cursor.key.iter().zip(&key_slots) {
            if !matches!(value, CursorValue::Present { .. }) {
                return Err(fail("Selected Key cursor component must be present"));
            }
            validate_value(value, &slots[*index])?;
            if value != &cursor.sort_values[*index] {
                return Err(fail("Selected Key and original sort tuple disagree"));
            }
        }
    }
    let after_branches = if after.is_some() {
        (0..slots.len())
            .map(|strict_slot| AfterBranch {
                equal_slots: (0..strict_slot).collect(),
                strict_slot,
            })
            .collect()
    } else {
        vec![]
    };
    Ok(OrderedPage {
        version: "weft-ir/engine-ordered-page/0.1",
        source_sql: sql.into(),
        base,
        selected_key: key,
        sort_slots: slots,
        key_slots,
        page_size,
        after,
        after_branches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    fn catalog() -> Catalog {
        let request: Value = serde_json::from_str(include_str!(
            "../../../tests/engine-catalog/native-ashlar-related-request.json"
        ))
        .unwrap();
        let mut modules: Vec<crate::model::ModuleInput> =
            serde_json::from_value(request["modules"].clone()).unwrap();
        let mut doc: Value = serde_json::from_str(&modules[0].document_json).unwrap();
        let module = &mut doc["modules"][0];
        let elements = module["elements"].as_array_mut().unwrap();
        elements.iter_mut().find(|e| e["id"] == "status").unwrap()["nullability"] =
            json!("absent-allowed");
        let mut code = elements.iter().find(|e| e["id"] == "id").unwrap().clone();
        code["id"] = json!("code");
        code["name"] = json!("code");
        elements.push(code);
        let record = elements.iter_mut().find(|e| e["id"] == "order").unwrap();
        record["members"]
            .as_array_mut()
            .unwrap()
            .push(json!({"module":"sales","element":"code"}));
        record["keys"].as_array_mut().unwrap().push(json!({"id":"alternate","name":"alternate","fields":[{"module":"sales","element":"code"}],"primary":false}));
        let raw = doc.to_string();
        let hash = crate::json::sha256(raw.as_bytes());
        modules[0].document_json = raw;
        modules[0].pin.revision = hash.clone();
        modules[0].pin.sha256 = hash;
        Catalog::prepare(modules).unwrap()
    }
    fn selected() -> SelectedKey {
        SelectedKey {
            module: "sales".into(),
            record: "order".into(),
            key: "alternate".into(),
        }
    }
    const SQL: &str = "SELECT r.id AS id FROM sales.\"order\" r ORDER BY r.status DESC NULLS FIRST,r.id ASC NULLS LAST";
    fn present(value: &str) -> CursorValue {
        CursorValue::Present {
            carrier: Family::String,
            value: value.into(),
        }
    }
    fn cursor() -> Cursor {
        Cursor {
            key: vec![present("code-a")],
            sort_values: vec![CursorValue::Missing {}, present("id-a"), present("code-a")],
        }
    }
    #[test]
    fn original_alternate_key_and_full_mixed_tuple_are_preserved_without_native_enrollment() {
        let c = catalog();
        let p = prepare(&c, SQL, Parameters::new(), &selected(), 2, Some(cursor())).unwrap();
        assert_eq!(p.source_sql(), SQL);
        assert_eq!(p.selected_key().id, "alternate");
        assert_eq!(p.sort_slots().len(), 3);
        assert_eq!(p.key_slots(), [2]);
        assert_eq!(p.after_branches().len(), 3);
        assert_eq!(p.after_branches()[2].equal_slots(), [0, 1]);
        assert_eq!(p.after_branches()[2].strict_slot(), 2);
        assert_eq!(p.sort_slots()[0].identity().element, "status");
        assert!(p.sort_slots()[0].descending());
        assert!(p.sort_slots()[0].nulls_first());
        assert!(p.sort_slots()[0].nullable() && p.sort_slots()[0].missing());
        assert_eq!(p.sort_slots()[2].identity().element, "code");
        assert!(!p.sort_slots()[2].descending());
        assert!(!p.sort_slots()[2].nullable());
        assert_eq!(p.sort_slots()[0].hidden_outputs(), (1, 2));
        assert_eq!(p.sort_slots()[2].hidden_outputs(), (5, 6));
        assert_eq!(
            p.null_rank(0, &CursorValue::Missing {}).unwrap(),
            p.null_rank(0, &CursorValue::Null {}).unwrap()
        );
        assert_eq!(p.null_rank(0, &CursorValue::Null {}).unwrap(), 0);
        assert_eq!(p.null_rank(2, &present("code-a")).unwrap(), 0);
        assert!(p.base().order.is_empty());
        assert!(p.base().page_key.is_none());
        assert_eq!(p.base().ir_version, "weft-ir/0.2.0");
        assert_eq!(p.after().unwrap().sort_values, cursor().sort_values);
        let no_suffix = prepare(
            &c,
            "SELECT r.id FROM sales.\"order\" r ORDER BY r.code ASC",
            Parameters::new(),
            &selected(),
            2,
            None,
        )
        .unwrap();
        assert_eq!(no_suffix.sort_slots().len(), 1);
        let duplicate = prepare(
            &c,
            "SELECT r.id FROM sales.\"order\" r ORDER BY r.code DESC",
            Parameters::new(),
            &selected(),
            2,
            None,
        )
        .unwrap();
        assert_eq!(duplicate.sort_slots().len(), 2);
        assert!(duplicate.sort_slots()[0].descending());
        assert!(!duplicate.sort_slots()[1].descending());
    }
    #[test]
    fn incomplete_or_foreign_cursor_and_state_domains_refuse() {
        let c = catalog();
        let mut cases = vec![];
        let mut v = cursor();
        v.sort_values.pop();
        cases.push(v);
        let mut v = cursor();
        v.key.clear();
        cases.push(v);
        let mut v = cursor();
        v.key[0] = present("other");
        cases.push(v);
        let mut v = cursor();
        v.sort_values[1] = CursorValue::Null {};
        cases.push(v);
        let mut v = cursor();
        v.sort_values[2] = CursorValue::Missing {};
        cases.push(v);
        let mut v = cursor();
        v.sort_values[0] = CursorValue::Present {
            carrier: Family::Integer,
            value: "1".into(),
        };
        cases.push(v);
        for v in cases {
            assert!(prepare(&c, SQL, Parameters::new(), &selected(), 2, Some(v)).is_err());
        }
        let mut v = cursor();
        v.sort_values[0] = CursorValue::Null {};
        assert!(prepare(&c, SQL, Parameters::new(), &selected(), 2, Some(v)).is_ok());
        for key in [
            SelectedKey {
                module: "foreign".into(),
                ..selected()
            },
            SelectedKey {
                record: "customer".into(),
                ..selected()
            },
            SelectedKey {
                key: "missing".into(),
                ..selected()
            },
        ] {
            assert!(prepare(&c, SQL, Parameters::new(), &key, 2, None).is_err());
        }
        for text in [
            r#"{"state":"null","value":"forged"}"#,
            r#"{"state":"present","carrier":"string","value":"a","extra":true}"#,
            r#"{"state":"other"}"#,
        ] {
            assert!(serde_json::from_str::<CursorValue>(text).is_err());
        }
    }
    #[test]
    fn original_syntax_parameter_and_historical_profile_boundaries_remain_closed() {
        let c = catalog();
        for sql in [
            "SELECT r.id FROM sales.\"order\" r ORDER BY x.id",
            "SELECT COUNT(*) FROM sales.\"order\" r",
            "SELECT r.id FROM sales.\"order\" r LIMIT 2",
            "SELECT r.id FROM sales.\"order\" r JOIN sales.customer c ON r.id=c.customerId",
            "SELECT r.id FROM sales.\"order\" r GROUP BY r.id",
        ] {
            assert!(
                prepare(&c, sql, Parameters::new(), &selected(), 2, None).is_err(),
                "{sql}"
            );
        }
        for size in [0, 1000] {
            assert!(prepare(&c, SQL, Parameters::new(), &selected(), size, None).is_err());
        }
        let mut parameters = Parameters::new();
        parameters.insert(
            "unused".into(),
            application_resolve::Parameter {
                family: Family::String,
                value: "a".into(),
            },
        );
        assert!(prepare(&c, SQL, parameters, &selected(), 2, None).is_err());
        assert!(ast::parse(SQL).is_err()); // old application parser still refuses DESC.
        let old = ast::parse("SELECT r.id FROM sales.\"order\" r").unwrap();
        let a = application_resolve::resolve(&c, old.clone(), Parameters::new(), None).unwrap();
        let _ = prepare(&c, SQL, Parameters::new(), &selected(), 2, None).unwrap();
        let b = application_resolve::resolve(&c, old, Parameters::new(), None).unwrap();
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(b).unwrap()
        );
    }
}
