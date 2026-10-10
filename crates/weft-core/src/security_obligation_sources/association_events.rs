//! Original association declaration custody; no physical join or authority inference.
use super::{fail, Budget};
use crate::{error::Result, security_ontology::SecurityRef,
    security_scan_obligations::{ActionObligations, ScanObligations}};
use serde_json::Value;
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug)]
pub(crate) struct Payload<'a> {
    pub(crate) scan: &'a ScanObligations,
    pub(crate) action: &'a ActionObligations,
    pub(crate) reference: &'a SecurityRef,
    pub(crate) carrier: &'a Value,
    /// Complete original ontology declaration, including ordered endpoints and fields.
    pub(crate) declaration: &'a Value,
}
pub(crate) type Events<'a> = BTreeMap<String, Payload<'a>>;
pub(super) fn retain<'a>(events:&mut Events<'a>,tokens:&[&str],payload:Payload<'a>,budget:&mut Budget)->Result<()> {
    let id=budget.encode(tokens)?;budget.charge(id.len())?;
    if let Some(old)=events.get(&id) {
        if !std::ptr::eq(old.scan,payload.scan) || !std::ptr::eq(old.action,payload.action)
            || !std::ptr::eq(old.reference,payload.reference) || !std::ptr::eq(old.carrier,payload.carrier)
            || !std::ptr::eq(old.declaration,payload.declaration) { return Err(fail()); }
        return Ok(());
    }
    if events.len()>=4096 {return Err(fail());}
    events.insert(id,payload);Ok(())
}
#[cfg(test)]
pub(crate) fn test_retain<'a>(events:&mut Events<'a>,tokens:&[&str],payload:Payload<'a>,work:usize,text:usize)->Result<(usize,usize)> {
    let mut budget=Budget{work,text};retain(events,tokens,payload,&mut budget)?;Ok((budget.work,budget.text))
}
