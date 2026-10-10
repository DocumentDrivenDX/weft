//! Borrowed original query semantics; not template applicability or authority.
use super::{fail, Budget};
use crate::{application_ir::Plan, error::Result, security_ontology::SecurityRef,
    security_query_uses::Projection, security_requirements::{SecurityOperatorRequirement, SecurityOutputRequirement},
    security_scan_obligations::ScanObligations};
use serde_json::Value;
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FieldUse { Projection, QueryField }
#[derive(Clone, Copy, Debug)]
pub(crate) enum Payload<'a> {
    Field { kind:FieldUse, scan:&'a ScanObligations, reference:&'a SecurityRef,
        carrier:&'a Value, classification:&'a Value, projection:Option<&'a Projection>, plan:&'a Plan },
    Operator { index:usize, requirement:&'a SecurityOperatorRequirement<'a>,
        carrier:&'a Value, classification:&'a Value, plan:&'a Plan },
    Output { requirement:&'a SecurityOutputRequirement<'a>, plan:&'a Plan },
}
pub(crate) type Events<'a> = BTreeMap<String, Payload<'a>>;
fn same(a:Payload<'_>,b:Payload<'_>)->bool {
    match (a,b) {
        (Payload::Field{kind:a,scan:s,reference:r,carrier:c,classification:d,projection:p,plan:q},
         Payload::Field{kind:b,scan:t,reference:u,carrier:e,classification:f,projection:v,plan:w}) =>
            a==b && std::ptr::eq(s,t) && std::ptr::eq(r,u) && std::ptr::eq(c,e)
            && std::ptr::eq(d,f) && std::ptr::eq(q,w) && match(p,v){(None,None)=>true,(Some(x),Some(y))=>std::ptr::eq(x,y),_=>false},
        (Payload::Operator{index:i,requirement:r,carrier:c,classification:d,plan:p},
         Payload::Operator{index:j,requirement:s,carrier:e,classification:f,plan:q}) =>
            i==j && std::ptr::eq(r,s) && std::ptr::eq(c,e) && std::ptr::eq(d,f) && std::ptr::eq(p,q),
        (Payload::Output{requirement:r,plan:p},Payload::Output{requirement:s,plan:q}) =>
            std::ptr::eq(r,s) && std::ptr::eq(p,q),
        _=>false,
    }
}
pub(super) fn retain<'a>(events:&mut Events<'a>,tokens:&[&str],payload:Payload<'a>,budget:&mut Budget)->Result<()> {
    let id=budget.encode(tokens)?;budget.charge(id.len())?;
    if let Some(old)=events.get(&id){if !same(*old,payload){return Err(fail());}return Ok(());}
    if events.len()>=4096{return Err(fail());}events.insert(id,payload);Ok(())
}

#[cfg(test)]
pub(crate) fn test_retain<'a>(events:&mut Events<'a>,tokens:&[&str],payload:Payload<'a>,work:usize,text:usize)->Result<(usize,usize)> {
    let mut budget=Budget{work,text};retain(events,tokens,payload,&mut budget)?;Ok((budget.work,budget.text))
}
