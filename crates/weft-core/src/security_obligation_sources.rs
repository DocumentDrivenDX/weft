//! Owner-derived declaration lineage only. No native evidence or enforcement admission.
mod association_events;
pub(crate) use association_events::Payload as AssociationPayload;
#[cfg(test)]
pub(crate) use association_events::test_retain as test_association_retain;
mod key_events;
mod query_events;
pub(crate) use query_events::{Payload as QueryPayload, FieldUse};
#[cfg(test)]
pub(crate) use query_events::test_retain as test_query_retain;
pub(crate) use key_events::Payload as KeyPayload;
use crate::{
    error::{Diagnostic, Result},
    security_backend::SecurityBackendContext,
    security_ontology::SecurityRef,
    security_requirements::SecurityOperatorMode,
};
use std::collections::{BTreeMap, BTreeSet};
use crate::security_semantic_coverage::{CoverageScope, OwnerCoverage};
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-LOWERING-UNSUPPORTED",
        "capability",
        "Obligation source correspondence refused",
    )
}
struct Budget {
    work: usize,
    text: usize,
}
impl Budget {
    fn charge(&mut self, bytes: usize) -> Result<()> {
        if self.work == 0 || self.text < bytes {
            return Err(fail());
        }
        self.work -= 1;
        self.text -= bytes;
        Ok(())
    }
    fn encode(&mut self, tokens: &[&str]) -> Result<String> {
        // Exact JSON string-array size, charged before the serializer allocates.
        let mut size = 2usize + tokens.len().saturating_sub(1);
        for token in tokens {
            self.charge(token.len())?;
            if token.contains('\0') {
                return Err(fail());
            }
            size = size.checked_add(2).ok_or_else(fail)?;
            for byte in token.bytes() {
                size = size
                    .checked_add(match byte {
                        b'"' | b'\\' | 8 | 9 | 10 | 12 | 13 => 2,
                        0..=31 => 6,
                        _ => 1,
                    })
                    .ok_or_else(fail)?;
            }
        }
        if size > 4096 {
            return Err(fail());
        }
        self.charge(size)?;
        let id = serde_json::to_string(tokens).map_err(|_| fail())?;
        if id.len() != size {
            return Err(fail());
        }
        Ok(id)
    }
    fn issue(&mut self, tokens: &[&str], out: &mut BTreeSet<String>) -> Result<()> {
        out.insert(self.encode(tokens)?);
        if out.len() > 4096 { return Err(fail()); }
        Ok(())
    }
}
/// Owner event categories are lineage, not enforcement obligation kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum OwnerEventKind {
 PrimaryAction, Policy, Ontology, Query, Model, Module, Scan, Projection,
 QueryField, Action, Rule, Key, KeyField, Field, Context, Association, Operator, Output,
}
/// Actual field carrier and qualified references; no normalized/cloned domain.
#[derive(Clone, Copy, Debug)]
pub(crate) enum FieldChannel<'a> {
    Stored { target: &'a SecurityRef },
    Context,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct FieldEvent<'a> {
    pub(crate) reference: &'a SecurityRef,
    pub(crate) carrier: &'a serde_json::Value,
    pub(crate) channel: FieldChannel<'a>,
    /// Exact ontology field declaration (protection/query-use), absent for context.
    pub(crate) classification: Option<&'a serde_json::Value>,
}
type FieldEvents<'a> = BTreeMap<String, FieldEvent<'a>>;
type Events = BTreeMap<String, OwnerEventKind>;
// References point only into the retained coverage relation; no capability product.
type Demands<'a> = BTreeMap<String, BTreeSet<&'a CoverageScope>>;
#[allow(dead_code)]
pub(crate) struct OwnerSourceDemands<'a, 'm, 'c, 's> {
    coverage: &'a OwnerCoverage<'m, 'c, 's>,
    demands: Demands<'a>,
    events: Events,
    rule_events: BTreeMap<String, &'a crate::security_ir::Rule>,
    field_events: FieldEvents<'a>,
    key_events: key_events::Events<'a>,
    query_events: query_events::Events<'a>,
    association_events: association_events::Events<'a>,
}
#[allow(dead_code)]
impl<'a, 'm, 'c, 's> OwnerSourceDemands<'a, 'm, 'c, 's> {
    pub(crate) fn coverage(&self) -> &'a OwnerCoverage<'m, 'c, 's> { self.coverage }
    pub(crate) fn rule_events(&self) -> &BTreeMap<String, &'a crate::security_ir::Rule> { &self.rule_events }
    pub(crate) fn field_events(&self) -> &FieldEvents<'a> { &self.field_events }
    pub(crate) fn key_events(&self) -> &key_events::Events<'a> { &self.key_events }
    pub(crate) fn association_events(&self) -> &association_events::Events<'a> { &self.association_events }
    pub(crate) fn query_events(&self) -> &query_events::Events<'a> { &self.query_events }
    pub(crate) fn events(&self) -> &Events { &self.events }
    pub(crate) fn demands(&self) -> &Demands<'a> { &self.demands }
    pub(crate) fn candidates(&self, scope: &CoverageScope) -> Option<&BTreeSet<String>> {
        self.coverage.assignments().get(scope)
    }
}
#[allow(dead_code)]
pub(crate) fn issue_demands<'a, 'm, 'c, 's>(coverage: &'a OwnerCoverage<'m, 'c, 's>) -> Result<OwnerSourceDemands<'a, 'm, 'c, 's>> {
    let (_, demands, events, rule_events, field_events, key_events, query_events, association_events) = traverse(coverage.context(), Some(coverage.assignments()),
        &mut Budget { work: 1_000_000, text: 16_000_000 })?;
    Ok(OwnerSourceDemands { coverage, demands, events, rule_events, field_events, key_events, query_events, association_events })
}
struct SourceIssuer<'a, 'b> {
    ledger: &'b mut Budget,
    assignments: Option<&'a BTreeMap<CoverageScope, BTreeSet<String>>>,
    current: BTreeSet<&'a CoverageScope>,
    demands: Demands<'a>,
    events: Events,
    rule_events: BTreeMap<String, &'a crate::security_ir::Rule>,
    field_events: FieldEvents<'a>,
    key_events: key_events::Events<'a>,
    query_events: query_events::Events<'a>,
    association_events: association_events::Events<'a>,
    edges: usize,
}
impl<'a> SourceIssuer<'a, '_> {
    fn charge(&mut self, bytes: usize) -> Result<()> { self.ledger.charge(bytes) }
    fn clear(&mut self) { self.current.clear(); }
    fn action(&mut self, scan: &str, action: &str) -> Result<()> {
        let Some(assignments) = self.assignments else { return Ok(()); };
        for scope in assignments.keys() {
            self.ledger.charge(0)?;
            if matches!(scope, CoverageScope::ScanAction { scan: s, action: a } if s == scan && a == action) {
                self.current.insert(scope); return Ok(());
            }
        }
        Err(fail())
    }
    fn application(&mut self) -> Result<()> {
        let Some(assignments) = self.assignments else { return Ok(()); };
        self.ledger.charge(0)?;
        let (scope, _) = assignments.get_key_value(&CoverageScope::Application).ok_or_else(fail)?;
        self.current.insert(scope); Ok(())
    }
    fn global(&mut self) -> Result<()> {
        self.clear();
        if let Some(assignments) = self.assignments {
            for scope in assignments.keys() { self.ledger.charge(0)?; self.current.insert(scope); }
        }
        Ok(())
    }
    fn primary(&mut self, scan: &str, action: &str) -> Result<()> {
        self.clear(); self.action(scan, action)?; self.application()
    }
    fn retain_rule(&mut self,tokens:&[&str],rule:&'a crate::security_ir::Rule)->Result<()> {
        if self.assignments.is_none() { return Ok(()); }
        let id=self.ledger.encode(tokens)?;self.charge(id.len())?;
        if let Some(old)=self.rule_events.insert(id,rule) { if !std::ptr::eq(old,rule) {return Err(fail());} }
        if self.rule_events.len()>4096 {return Err(fail());} Ok(())
    }
    fn field_carrier(&mut self, ctx: &'a SecurityBackendContext<'_>, reference: &SecurityRef) -> Result<&'a serde_json::Value> {
        for document in &ctx.catalog().documents {
            self.charge(0)?;
            if document["id"].as_str() != Some(reference.document_id.as_str()) { continue; }
            for module in document["modules"].as_array().ok_or_else(fail)? {
                self.charge(0)?;
                if module["id"].as_str() != Some(reference.module_id.as_str()) { continue; }
                for element in module["elements"].as_array().ok_or_else(fail)? {
                    self.charge(0)?;
                    if element["id"].as_str() == Some(reference.element_id.as_str()) { return Ok(element); }
                }
            }
        }
        Err(fail())
    }
    fn field_classification(&mut self, ctx: &'a SecurityBackendContext<'_>, target: &SecurityRef, field: &SecurityRef) -> Result<&'a serde_json::Value> {
        fn matches(value: &serde_json::Value, reference: &SecurityRef) -> bool {
            value["documentId"].as_str() == Some(reference.document_id.as_str())
                && value["moduleId"].as_str() == Some(reference.module_id.as_str())
                && value["elementId"].as_str() == Some(reference.element_id.as_str())
        }
        let ontology = ctx.logical_plan().source().ontology();
        for group in ["entities", "associations"] {
            for owner in ontology[group].as_array().ok_or_else(fail)? {
                self.charge(0)?;
                if !matches(&owner["type"], target) { continue; }
                for declaration in owner["fields"].as_array().ok_or_else(fail)? {
                    self.charge(0)?;
                    if matches(&declaration["ref"], field) { return Ok(declaration); }
                }
            }
        }
        Err(fail())
    }
    fn retain_field(&mut self, tokens: &[&str], field: FieldEvent<'a>) -> Result<()> {
        if self.assignments.is_none() { return Ok(()); }
        let id = self.ledger.encode(tokens)?;
        self.charge(id.len())?;
        if let Some(old) = self.field_events.get(&id) {
            let same_channel = match (old.channel, field.channel) {
                (FieldChannel::Context, FieldChannel::Context) => true,
                (FieldChannel::Stored { target: a }, FieldChannel::Stored { target: b }) => std::ptr::eq(a, b),
                _ => false,
            };
            if !same_channel || !std::ptr::eq(old.reference, field.reference)
                || !std::ptr::eq(old.carrier, field.carrier)
                || match (old.classification, field.classification) {
                    (None, None) => false,
                    (Some(a), Some(b)) => !std::ptr::eq(a, b),
                    _ => true,
                } { return Err(fail()); }
            return Ok(());
        }
        if self.field_events.len() >= 4096 { return Err(fail()); }
        self.field_events.insert(id, field);
        Ok(())
    }
    fn event(&mut self, kind: OwnerEventKind, tokens: &[&str], out: &mut BTreeSet<String>) -> Result<()> {
        self.issue(tokens, out)?;
        if self.assignments.is_some() {
            let id = self.ledger.encode(tokens)?;
            self.ledger.charge(id.len())?;
            if let Some(old) = self.events.insert(id, kind) {
                if old != kind { return Err(fail()); }
            }
            if self.events.len() > 4096 { return Err(fail()); }
        }
        Ok(())
    }
    fn issue(&mut self, tokens: &[&str], out: &mut BTreeSet<String>) -> Result<()> {
        if self.assignments.is_none() { return self.ledger.issue(tokens, out); }
        if self.current.is_empty() { return Err(fail()); }
        let id = self.ledger.encode(tokens)?;
        // The source inventory and demand-map key are distinct retained copies.
        self.ledger.charge(id.len())?;
        out.insert(id.clone());
        if out.len() > 4096 { return Err(fail()); }
        let entry = self.demands.entry(id).or_default();
        for scope in &self.current {
            self.ledger.charge(0)?;
            if !entry.contains(scope) {
                if self.edges >= 65_536 { return Err(fail()); }
                entry.insert(*scope); self.edges += 1;
            }
        }
        Ok(())
    }
}
fn reference<'a>(r: &'a SecurityRef) -> [&'a str; 3] {
    [&r.document_id, &r.module_id, &r.element_id]
}
#[allow(dead_code)]
pub(crate) fn derive(ctx: &SecurityBackendContext<'_>) -> Result<BTreeSet<String>> {
    derive_budget(
        ctx,
        &mut Budget {
            work: 1_000_000,
            text: 16_000_000,
        },
    )
}
fn derive_budget(ctx: &SecurityBackendContext<'_>, b: &mut Budget) -> Result<BTreeSet<String>> {
    traverse(ctx, None, b).map(|(sources, _, _, _, _, _, _, _)| sources)
}
fn traverse<'a>(ctx: &'a SecurityBackendContext<'_>, assignments: Option<&'a BTreeMap<CoverageScope, BTreeSet<String>>>, ledger: &mut Budget) -> Result<(BTreeSet<String>, Demands<'a>, Events, BTreeMap<String, &'a crate::security_ir::Rule>, FieldEvents<'a>, key_events::Events<'a>, query_events::Events<'a>, association_events::Events<'a>)> {
    let mut b = SourceIssuer { ledger, assignments, current: BTreeSet::new(), demands: BTreeMap::new(), events: BTreeMap::new(), rule_events: BTreeMap::new(), field_events: BTreeMap::new(), key_events: BTreeMap::new(), query_events: BTreeMap::new(), association_events: BTreeMap::new(), edges: 0 };
    let mut out = BTreeSet::new();
    let req = ctx.requirements();
    for scan in req.scans() { b.action(scan.inventory().scan(), req.primary_action())?; }
    b.application()?;
    b.event(OwnerEventKind::PrimaryAction,&["primary-action", req.primary_action()], &mut out)?;
    b.global()?;
    for (kind, event, raw) in [
        ("policy", OwnerEventKind::Policy, ctx.logical_plan().source().policy_json()),
        ("ontology", OwnerEventKind::Ontology, ctx.logical_plan().source().ontology_json()),
        ("query", OwnerEventKind::Query, ctx.query().sql()),
    ] {
        b.charge(raw.len())?;
        let hash = crate::json::sha256(raw.as_bytes());
        b.event(event, &[kind, &hash], &mut out)?;
    }
    for input in &ctx.catalog().inputs {
        let p = &input.pin;
        b.event(OwnerEventKind::Model,
            &[
                "model",
                &p.document_id,
                &p.revision,
                &p.umf_version,
                &p.sha256,
            ],
            &mut out,
        )?;
        for module in &input.selected_module_ids {
            b.event(OwnerEventKind::Module,&["module", &p.document_id, module], &mut out)?;
        }
    }
    for scan in req.scans() {
        let s = scan.inventory();
        let name = s.scan();
        let target = reference(s.target());
        b.clear();
        for action in scan.actions() { b.action(name, action.inventory().action())?; }
        b.application()?;
        b.event(OwnerEventKind::Scan,&["scan", name, target[0], target[1], target[2]], &mut out)?;
        for (kind, event, fields) in [
            ("projection", OwnerEventKind::Projection, s.projection_fields()),
            ("query-field", OwnerEventKind::QueryField, s.query_fields()),
        ] {
            for field in fields {
                b.primary(name, req.primary_action())?;
                if assignments.is_some() && kind == "query-field" {
                    for operator in req.operators() {
                        b.charge(0)?;
                        if operator.usage().scan == name && &operator.usage().field == field {
                            if let SecurityOperatorMode::OriginalAuthorized(action) = operator.mode() { b.action(name, action)?; }
                        }
                    }
                }
                let f = reference(field);
                b.event(event, &[kind, name, f[0], f[1], f[2]], &mut out)?;
                if assignments.is_some() {
                    let carrier=b.field_carrier(ctx,field)?;
                    let classification=b.field_classification(ctx,s.target(),field)?;
                    let (usage,projection)=match event {
                        OwnerEventKind::Projection=>{
                            let mut original=None;
                            for projection in ctx.query().projections(){b.charge(0)?;if projection.scan==name && projection.target==*s.target() && projection.field==*field {if original.is_some(){return Err(fail());}original=Some(projection);}}
                            (FieldUse::Projection,Some(original.ok_or_else(fail)?))
                        },
                        OwnerEventKind::QueryField=>(FieldUse::QueryField,None),
                        _=>return Err(fail()),
                    };
                    query_events::retain(&mut b.query_events,&[kind,name,f[0],f[1],f[2]],
                        QueryPayload::Field{kind:usage,scan:s,reference:field,carrier,classification,projection,plan:ctx.query().application_plan()},b.ledger)?;
                }
            }
        }
        for action in scan.actions() {
            let a = action.inventory();
            let an = a.action();
            b.clear(); b.action(name, an)?;
            b.event(OwnerEventKind::Action,&["action", name, an], &mut out)?;
            for rule in action.rules() {
                b.event(OwnerEventKind::Rule,&["rule", name, an, &rule.id], &mut out)?;
                b.retain_rule(&["rule",name,an,&rule.id],rule)?;
            }
            for (target, (key, fields)) in a.keys() {
                let t = reference(target);
                b.event(OwnerEventKind::Key,&["key", name, an, t[0], t[1], t[2], key], &mut out)?;
                let key_payload = if assignments.is_some() {
                    let record = b.field_carrier(ctx, target)?;
                    let payload = key_events::resolve(target, key, fields, record, b.ledger)?;
                    key_events::retain(&mut b.key_events, &["key", name, an, t[0], t[1], t[2], key], KeyPayload::Key(payload), b.ledger)?;
                    Some(payload)
                } else { None };
                for (i, field) in fields.iter().enumerate() {
                    let f = reference(field);
                    let position = (i + 1).to_string();
                    b.event(OwnerEventKind::KeyField,
                        &[
                            "key-field",
                            name,
                            an,
                            t[0],
                            t[1],
                            t[2],
                            key,
                            &position,
                            f[0],
                            f[1],
                            f[2],
                        ],
                        &mut out,
                    )?;
                    if let Some(key) = key_payload {
                        let carrier = b.field_carrier(ctx, field)?;
                        let declaration = &key.definition["fields"][i];
                        key_events::retain(&mut b.key_events, &["key-field", name, an, t[0], t[1], t[2], key.id, &position, f[0], f[1], f[2]],
                            KeyPayload::Member { key, index: i, reference: field, carrier, declaration }, b.ledger)?;
                    }
                }
            }
            for (target, fields) in a.fields() {
                let t = reference(target);
                for field in fields {
                    let f = reference(field);
                    b.event(OwnerEventKind::Field,
                        &["field", name, an, t[0], t[1], t[2], f[0], f[1], f[2]],
                        &mut out,
                    )?;
                    if assignments.is_some() {
                        let carrier = b.field_carrier(ctx, field)?;
                        let classification = b.field_classification(ctx, target, field)?;
                        b.retain_field(&["field", name, an, t[0], t[1], t[2], f[0], f[1], f[2]],
                            FieldEvent { reference: field, carrier, channel: FieldChannel::Stored { target }, classification: Some(classification) })?;
                    }
                }
            }
            for field in a.context() {
                let f = reference(field);
                b.event(OwnerEventKind::Context,&["context", name, an, f[0], f[1], f[2]], &mut out)?;
                if assignments.is_some() {
                    let carrier = b.field_carrier(ctx, field)?;
                    b.retain_field(&["context", name, an, f[0], f[1], f[2]],
                        FieldEvent { reference: field, carrier, channel: FieldChannel::Context, classification: None })?;
                }
            }
            for association in a.associations() {
                let r = reference(association);
                b.event(OwnerEventKind::Association,&["association", name, an, r[0], r[1], r[2]], &mut out)?;
                if assignments.is_some() {
                    let carrier=b.field_carrier(ctx,association)?;
                    let mut declaration=None;
                    for candidate in ctx.logical_plan().source().ontology()["associations"].as_array().ok_or_else(fail)? {
                        b.charge(0)?;
                        let t=&candidate["type"];
                        if t["documentId"].as_str()==Some(r[0]) && t["moduleId"].as_str()==Some(r[1]) && t["elementId"].as_str()==Some(r[2]) {
                            if declaration.is_some(){return Err(fail());} declaration=Some(candidate);
                        }
                    }
                    association_events::retain(&mut b.association_events,&["association",name,an,r[0],r[1],r[2]],
                        AssociationPayload{scan:s,action:a,reference:association,carrier,declaration:declaration.ok_or_else(fail)?},b.ledger)?;
                }
            }
        }
    }
    for (i, operator) in req.operators().iter().enumerate() {
        let u = operator.usage();
        b.primary(&u.scan, req.primary_action())?;
        if let SecurityOperatorMode::OriginalAuthorized(action) = operator.mode() { b.action(&u.scan, action)?; }
        let t = reference(&u.target);
        let f = reference(&u.field);
        let position = (i + 1).to_string();
        let mut tokens = vec![
            "operator",
            &position,
            &u.scan,
            t[0],
            t[1],
            t[2],
            f[0],
            f[1],
            f[2],
            u.operator.name(),
        ];
        match operator.mode() {
            SecurityOperatorMode::Disclosed => tokens.push("disclosed"),
            SecurityOperatorMode::OriginalAuthorized(action) => {
                tokens.extend(["original-authorized", action])
            }
        }
        b.event(OwnerEventKind::Operator, &tokens, &mut out)?;
        if assignments.is_some() {
            let carrier=b.field_carrier(ctx,&u.field)?;
            let classification=b.field_classification(ctx,&u.target,&u.field)?;
            query_events::retain(&mut b.query_events,&tokens,QueryPayload::Operator{index:i,requirement:operator,carrier,classification,plan:ctx.query().application_plan()},b.ledger)?;
        }
    }
    for output in req.outputs() {
        if assignments.is_some() {
            let crate::application_ir::Expression::Field { scan, .. } = &output.output().expression else { return Err(fail()); };
            b.primary(scan, req.primary_action())?;
        }
        let position = output.position().to_string();
        b.event(OwnerEventKind::Output,&["output", &position, &output.output().name], &mut out)?;
        if assignments.is_some() {
            query_events::retain(&mut b.query_events,&["output",&position,&output.output().name],QueryPayload::Output{requirement:output,plan:ctx.query().application_plan()},b.ledger)?;
        }
    }
    if assignments.is_some() && (b.demands.len() != out.len() || !b.demands.keys().eq(out.iter())) { return Err(fail()); }
    if assignments.is_some() && !b.events.keys().eq(out.iter()) { return Err(fail()); }
    if assignments.is_some() {
        let required = b.events.iter().filter(|(_, kind)| matches!(kind, OwnerEventKind::Field | OwnerEventKind::Context)).map(|(id, _)| id);
        if !required.eq(b.field_events.keys()) { return Err(fail()); }
        let required = b.events.iter().filter(|(_, kind)| matches!(kind, OwnerEventKind::Key | OwnerEventKind::KeyField)).map(|(id, _)| id);
        if !required.eq(b.key_events.keys()) { return Err(fail()); }
        let required=b.events.iter().filter(|(_,kind)|matches!(kind,OwnerEventKind::Projection|OwnerEventKind::QueryField|OwnerEventKind::Operator|OwnerEventKind::Output)).map(|(id,_)|id);
        if !required.eq(b.query_events.keys()){return Err(fail());}
        let required=b.events.iter().filter(|(_,kind)|**kind==OwnerEventKind::Association).map(|(id,_)|id);
        if !required.eq(b.association_events.keys()){return Err(fail());}
    }
    Ok((out, b.demands, b.events, b.rule_events, b.field_events, b.key_events, b.query_events, b.association_events))
}
#[cfg(test)]
pub(crate) fn test_budget(ctx: &SecurityBackendContext<'_>, work: usize, text: usize) -> Result<BTreeSet<String>> {
    derive_budget(ctx, &mut Budget { work, text })
}
#[cfg(test)]
pub(crate) fn test_demands<'a>(ctx: &'a SecurityBackendContext<'a>, assignments: &'a BTreeMap<CoverageScope, BTreeSet<String>>, work: usize, text: usize) -> Result<(Demands<'a>, usize, usize)> {
    let mut ledger = Budget { work, text };
    let (_, demands, _, _, _, _, _, _) = traverse(ctx, Some(assignments), &mut ledger)?;
    Ok((demands, ledger.work, ledger.text))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tuple_encoding_preserves_escaping_boundaries_and_exact_utf8_limits() {
        let mut b = Budget {
            work: 1000,
            text: 100000,
        };
        let mut out = BTreeSet::new();
        for tokens in [["a/b", "c"], ["a", "b/c"], ["a\"b", "c\\d"], ["é", "\n"]] {
            b.issue(&tokens, &mut out).unwrap();
            let id = serde_json::to_string(&tokens).unwrap();
            assert!(out.contains(&id));
            assert_eq!(serde_json::from_str::<Vec<String>>(&id).unwrap(), tokens);
        }
        assert_eq!(out.len(), 4);
        for (work,text,admitted) in [(2,6,true),(1,6,false),(2,5,false)] {
            let mut ledger=Budget{work,text};let mut issued=BTreeSet::new();
            assert_eq!(ledger.issue(&["x"],&mut issued).is_ok(),admitted);
            assert_eq!(issued.is_empty(),!admitted);
        }

        assert!(b.issue(&["nul\0"], &mut out).is_err());
        let exact = "é".repeat(2046); // 4092 UTF8 bytes + four JSON framing bytes.
        assert!(b.issue(&[&exact], &mut out).is_ok());
        assert!(b.issue(&[&(exact + "é")], &mut out).is_err());
    }
    #[test]
    fn demand_retention_merges_edges_and_refuses_exact_bound_overrun() {
        let scopes = BTreeMap::from([(CoverageScope::Application, BTreeSet::from(["whole".into()]))]);
        let mut ledger = Budget { work: 100, text: 1000 };
        let mut issuer = SourceIssuer { ledger: &mut ledger, assignments: Some(&scopes), current: BTreeSet::new(), demands: BTreeMap::new(), events: BTreeMap::new(), rule_events: BTreeMap::new(), field_events: BTreeMap::new(), key_events: BTreeMap::new(), query_events: BTreeMap::new(), association_events: BTreeMap::new(), edges: 65_535 };
        issuer.application().unwrap();
        let mut sources = BTreeSet::new();
        issuer.issue(&["x"], &mut sources).unwrap();
        assert_eq!(issuer.edges, 65_536);
        issuer.issue(&["x"], &mut sources).unwrap(); // Duplicate edge is merged.
        assert_eq!(issuer.edges, 65_536);
        assert!(issuer.issue(&["y"], &mut sources).is_err());
        // Independently populate the entire finite edge boundary, not just its counter.
        let population: BTreeMap<_, _> = (0..256).map(|i| (CoverageScope::ScanAction { scan: format!("s{i}"), action: "read".into() }, BTreeSet::from(["whole".into()]))).collect();
        let mut ledger = Budget { work: 1_000_000, text: 16_000_000 };
        let mut population_issuer = SourceIssuer { ledger: &mut ledger, assignments: Some(&population), current: BTreeSet::new(), demands: BTreeMap::new(), events: BTreeMap::new(), rule_events: BTreeMap::new(), field_events: BTreeMap::new(), key_events: BTreeMap::new(), query_events: BTreeMap::new(), association_events: BTreeMap::new(), edges: 0 };
        population_issuer.global().unwrap();
        let mut population_sources = BTreeSet::new();
        for i in 0..256 { population_issuer.issue(&["population", &i.to_string()], &mut population_sources).unwrap(); }
        assert_eq!(population_issuer.demands.len(), 256);
        assert_eq!(population_issuer.demands.values().map(BTreeSet::len).sum::<usize>(), 65_536);
        assert_eq!(population_issuer.edges, 65_536);
        assert!(population_issuer.issue(&["population", "overflow"], &mut population_sources).is_err());
        // Only the private traversal sees intermediates; issuance returns no prefix on Err.
        for (work, text, ok) in [(5, 11, true), (4, 11, false), (5, 10, false)] {
            let mut ledger = Budget { work, text };
            let mut issuer = SourceIssuer { ledger: &mut ledger, assignments: Some(&scopes), current: BTreeSet::new(), demands: BTreeMap::new(), events: BTreeMap::new(), rule_events: BTreeMap::new(), field_events: BTreeMap::new(), key_events: BTreeMap::new(), query_events: BTreeMap::new(), association_events: BTreeMap::new(), edges: 0 };
            issuer.application().unwrap();
            assert_eq!(issuer.issue(&["x"], &mut BTreeSet::new()).is_ok(), ok);
        }
    }

    #[test]
    fn typed_events_reject_category_aliasing_and_charge_the_retained_copy() {
        let scopes=BTreeMap::from([(CoverageScope::Application,BTreeSet::from(["whole".into()]))]);
        // Independently calculate one scope lookup, source issue and event copy.
        // ["x"] encodes to5 bytes: source11 + event11 =22 retained/processed bytes.
        for (work,text,ok) in [(8,22,true),(7,22,false),(8,21,false)] {
            let mut ledger=Budget{work,text};
            let mut issuer=SourceIssuer{ledger:&mut ledger,assignments:Some(&scopes),current:BTreeSet::new(),demands:BTreeMap::new(),events:BTreeMap::new(),rule_events:BTreeMap::new(),field_events:BTreeMap::new(),key_events:BTreeMap::new(), query_events: BTreeMap::new(), association_events: BTreeMap::new(),edges:0};
            issuer.application().unwrap();
            assert_eq!(issuer.event(OwnerEventKind::Output,&["x"],&mut BTreeSet::new()).is_ok(),ok);
        }
        let mut ledger=Budget{work:100,text:1000};
        let mut issuer=SourceIssuer{ledger:&mut ledger,assignments:Some(&scopes),current:BTreeSet::new(),demands:BTreeMap::new(),events:BTreeMap::new(),rule_events:BTreeMap::new(),field_events:BTreeMap::new(),key_events:BTreeMap::new(), query_events: BTreeMap::new(), association_events: BTreeMap::new(),edges:0};
        issuer.application().unwrap();let mut out=BTreeSet::new();
        issuer.event(OwnerEventKind::Output,&["x"],&mut out).unwrap();
        issuer.event(OwnerEventKind::Output,&["x"],&mut out).unwrap();
        assert_eq!(issuer.events.len(),1);
        assert!(issuer.event(OwnerEventKind::Operator,&["x"],&mut out).is_err());
    }

    #[test]
    fn rule_retention_requires_exact_owner_pointer_and_independent_copy_budget() {
        let rule=crate::security_ir::Rule{id:"r".into(),effect:crate::security_ir::Effect::Permit,actions:vec!["read".into()],target:crate::security_ontology::SecurityRef{document_id:"d".into(),module_id:"m".into(),element_id:"t".into()},condition:crate::security_ir::Expression::Literal(false),disclosure:vec![]};
        let equal=rule.clone();assert_eq!(rule,equal);assert!(!std::ptr::eq(&rule,&equal));
        let scopes=BTreeMap::from([(CoverageScope::Application,BTreeSet::new())]);
        // Four token bytes(16), JSON encoding(29), retained ID(29):6 visits/74 bytes.
        for (work,text,ok) in [(6,74,true),(5,74,false),(6,73,false)] {
         let mut ledger=Budget{work,text};let mut issuer=SourceIssuer{ledger:&mut ledger,assignments:Some(&scopes),current:BTreeSet::new(),demands:BTreeMap::new(),events:BTreeMap::new(),rule_events:BTreeMap::new(),field_events:BTreeMap::new(),key_events:BTreeMap::new(), query_events: BTreeMap::new(), association_events: BTreeMap::new(),edges:0};
         assert_eq!(issuer.retain_rule(&["rule","s0","read","reader"],&rule).is_ok(),ok);
        }
        let mut ledger=Budget{work:100,text:1000};let mut issuer=SourceIssuer{ledger:&mut ledger,assignments:Some(&scopes),current:BTreeSet::new(),demands:BTreeMap::new(),events:BTreeMap::new(),rule_events:BTreeMap::new(),field_events:BTreeMap::new(),key_events:BTreeMap::new(), query_events: BTreeMap::new(), association_events: BTreeMap::new(),edges:0};
        issuer.retain_rule(&["rule","s0","read","reader"],&rule).unwrap();issuer.retain_rule(&["rule","s0","read","reader"],&rule).unwrap();
        assert_eq!(issuer.rule_events.len(),1);assert!(issuer.retain_rule(&["rule","s0","read","reader"],&equal).is_err());
    }

    #[test]
    fn field_retention_refuses_equal_foreign_carriers_references_channels_and_classifications() {
        let reference = SecurityRef { document_id: "d".into(), module_id: "m".into(), element_id: "f".into() };
        let foreign_reference = reference.clone();
        let target = SecurityRef { element_id: "t".into(), ..reference.clone() };
        let foreign_target = target.clone();
        let carrier = serde_json::json!({"id":"f","facets":{"integerWidth":{"bits":32,"signed":false}},"allowedValues":[1,2]});
        let foreign_carrier = carrier.clone();
        let classification = serde_json::json!({"protection":"protected","queryUse":{"predicate":"original-authorized"}});
        let foreign_classification = classification.clone();
        let original = FieldEvent { reference: &reference, carrier: &carrier, channel: FieldChannel::Stored { target: &target }, classification: Some(&classification) };
        let scopes = BTreeMap::from([(CoverageScope::Application, BTreeSet::new())]);
        // One token byte + encoded five-byte key + retained five-byte key: 3 visits/11 bytes.
        for (work, text, ok) in [(3,11,true),(2,11,false),(3,10,false)] {
            let mut ledger = Budget { work, text };
            let mut issuer = SourceIssuer { ledger: &mut ledger, assignments: Some(&scopes), current: BTreeSet::new(), demands: BTreeMap::new(), events: BTreeMap::new(), rule_events: BTreeMap::new(), field_events: BTreeMap::new(), key_events: BTreeMap::new(), query_events: BTreeMap::new(), association_events: BTreeMap::new(), edges: 0 };
            assert_eq!(issuer.retain_field(&["x"], original).is_ok(), ok);
        }
        let mut ledger = Budget { work: 1000, text: 10000 };
        let mut issuer = SourceIssuer { ledger: &mut ledger, assignments: Some(&scopes), current: BTreeSet::new(), demands: BTreeMap::new(), events: BTreeMap::new(), rule_events: BTreeMap::new(), field_events: BTreeMap::new(), key_events: BTreeMap::new(), query_events: BTreeMap::new(), association_events: BTreeMap::new(), edges: 0 };
        issuer.retain_field(&["x"], original).unwrap();
        issuer.retain_field(&["x"], original).unwrap();
        for substituted in [
            FieldEvent { reference: &foreign_reference, ..original },
            FieldEvent { carrier: &foreign_carrier, ..original },
            FieldEvent { channel: FieldChannel::Stored { target: &foreign_target }, ..original },
            FieldEvent { channel: FieldChannel::Context, ..original }, // Channel-only substitution.
            FieldEvent { channel: FieldChannel::Context, classification: None, ..original },
            FieldEvent { classification: Some(&foreign_classification), ..original },
            FieldEvent { classification: None, ..original },
        ] { assert!(issuer.retain_field(&["x"], substituted).is_err()); }
        assert_eq!(issuer.field_events.len(), 1);
        assert!(std::ptr::eq(issuer.field_events["[\"x\"]"].carrier, &carrier));
        // Separate occurrences can borrow the same field without collapsing.
        issuer.retain_field(&["y"], original).unwrap();
        assert_eq!(issuer.field_events.len(), 2);
        let mut ledger = Budget { work: 100_000, text: 1_000_000 };
        let mut issuer = SourceIssuer { ledger: &mut ledger, assignments: Some(&scopes), current: BTreeSet::new(), demands: BTreeMap::new(), events: BTreeMap::new(), rule_events: BTreeMap::new(), field_events: BTreeMap::new(), key_events: BTreeMap::new(), query_events: BTreeMap::new(), association_events: BTreeMap::new(), edges: 0 };
        for i in 0..4096 { issuer.retain_field(&["population", &i.to_string()], original).unwrap(); }
        assert_eq!(issuer.field_events.len(), 4096);
        assert!(issuer.retain_field(&["population", "overflow"], original).is_err());
        assert_eq!(issuer.field_events.len(), 4096);
    }

}
