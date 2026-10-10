//! Borrowed selected Key semantics. No native key enforcement or authority.
use super::{fail, Budget};
use crate::{error::Result, security_ontology::SecurityRef};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Key<'a> {
    pub(crate) target: &'a SecurityRef,
    pub(crate) id: &'a str,
    pub(crate) members: &'a [SecurityRef],
    pub(crate) definition: &'a Value,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Payload<'a> {
    Key(Key<'a>),
    Member { key: Key<'a>, index: usize, reference: &'a SecurityRef,
        carrier: &'a Value, declaration: &'a Value },
}
pub(crate) type Events<'a> = BTreeMap<String, Payload<'a>>;

pub(super) fn resolve<'a>(target: &'a SecurityRef, id: &'a str,
    members: &'a [SecurityRef], record: &'a Value, budget: &mut Budget) -> Result<Key<'a>> {
    if record["kind"] != "record" || members.is_empty() { return Err(fail()); }
    let mut selected = None;
    for definition in record["keys"].as_array().ok_or_else(fail)? {
        budget.charge(0)?;
        if definition["id"].as_str() != Some(id) { continue; }
        if selected.is_some() { return Err(fail()); }
        let native = definition["fields"].as_array().ok_or_else(fail)?;
        if native.len() != members.len() { return Err(fail()); }
        for (field, declaration) in members.iter().zip(native) {
            budget.charge(0)?;
            if field.document_id != target.document_id
                || declaration["module"].as_str() != Some(field.module_id.as_str())
                || declaration["element"].as_str() != Some(field.element_id.as_str()) { return Err(fail()); }
        }
        selected = Some(definition);
    }
    Ok(Key { target, id, members, definition: selected.ok_or_else(fail)? })
}
fn same_key(a: Key<'_>, b: Key<'_>) -> bool {
    std::ptr::eq(a.target, b.target) && std::ptr::eq(a.id, b.id)
        && std::ptr::eq(a.members, b.members) && std::ptr::eq(a.definition, b.definition)
}
fn same(a: Payload<'_>, b: Payload<'_>) -> bool {
    match (a, b) {
        (Payload::Key(a), Payload::Key(b)) => same_key(a, b),
        (Payload::Member { key:a, index:i, reference:r, carrier:c, declaration:d },
         Payload::Member { key:b, index:j, reference:s, carrier:e, declaration:f }) =>
            same_key(a,b) && i == j && std::ptr::eq(r,s) && std::ptr::eq(c,e) && std::ptr::eq(d,f),
        _ => false,
    }
}
pub(super) fn retain<'a>(events: &mut Events<'a>, tokens: &[&str], payload: Payload<'a>, budget: &mut Budget) -> Result<()> {
    let id = budget.encode(tokens)?;
    budget.charge(id.len())?;
    if let Some(old) = events.get(&id) {
        if !same(*old, payload) { return Err(fail()); }
        return Ok(());
    }
    if events.len() >= 4096 { return Err(fail()); }
    events.insert(id, payload);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn reference(id: &str) -> SecurityRef { SecurityRef { document_id:"d".into(), module_id:"m".into(), element_id:id.into() } }
    #[test]
    fn selected_definition_keeps_primary_and_refuses_missing_duplicate_reordered_or_foreign_members() {
        let target=reference("t"); let fields=vec![reference("a"),reference("b")];
        let record=json!({"kind":"record","keys":[{"id":"other","primary":true,"fields":[{"module":"m","element":"b"}]},{"id":"chosen","primary":false,"fields":[{"module":"m","element":"a"},{"module":"m","element":"b"}]}]});
        let mut b=Budget { work:100,text:1000 };
        let key=resolve(&target,"chosen",&fields,&record,&mut b).unwrap();
        assert!(std::ptr::eq(key.definition,&record["keys"][1])); assert_eq!(key.definition["primary"],false);
        assert!(std::ptr::eq(key.members,fields.as_slice()));
        assert!(resolve(&target,"missing",&fields,&record,&mut b).is_err());
        let reversed=vec![fields[1].clone(),fields[0].clone()];
        assert!(resolve(&target,"chosen",&reversed,&record,&mut b).is_err());
        let mut foreign=fields.clone();foreign[0].document_id="other".into();
        assert!(resolve(&target,"chosen",&foreign,&record,&mut b).is_err());
        assert!(resolve(&target,"chosen",&fields[..1],&record,&mut b).is_err());
        let duplicate=json!({"kind":"record","keys":[record["keys"][1],record["keys"][1]]});
        assert!(resolve(&target,"chosen",&fields,&duplicate,&mut b).is_err());
        // Two candidate visits plus two ordered-member comparisons; no copies.
        for (work,ok) in [(4,true),(3,false)] { assert_eq!(resolve(&target,"chosen",&fields,&record,&mut Budget{work,text:0}).is_ok(),ok); }
    }
    #[test]
    fn retained_payload_refuses_each_equal_foreign_owner_and_exact_budget_overrun() {
        let target=reference("t");let other_target=target.clone();let id=String::from("k");let other_id=id.clone();
        let fields=vec![reference("a"),reference("b")];let other_fields=fields.clone();
        let definition=json!({"id":"k","fields":[{"module":"m","element":"a"},{"module":"m","element":"b"}]});let other_definition=definition.clone();
        let carrier=json!({"kind":"field","scalarType":"integer"});let other_carrier=carrier.clone();
        let key=Key{target:&target,id:&id,members:&fields,definition:&definition};
        let payload=Payload::Member{key,index:0,reference:&fields[0],carrier:&carrier,declaration:&definition["fields"][0]};
        for (work,text,ok) in [(3,11,true),(2,11,false),(3,10,false)] {
            assert_eq!(retain(&mut Events::new(),&["x"],payload,&mut Budget{work,text}).is_ok(),ok);
        }
        let mut events=Events::new();let mut b=Budget{work:100_000,text:1_000_000};
        retain(&mut events,&["x"],payload,&mut b).unwrap();retain(&mut events,&["x"],payload,&mut b).unwrap();
        for substitute in [
            Payload::Member{key:Key{target:&other_target,..key},index:0,reference:&fields[0],carrier:&carrier,declaration:&definition["fields"][0]},
            Payload::Member{key:Key{id:&other_id,..key},index:0,reference:&fields[0],carrier:&carrier,declaration:&definition["fields"][0]},
            Payload::Member{key:Key{members:&other_fields,..key},index:0,reference:&fields[0],carrier:&carrier,declaration:&definition["fields"][0]},
            Payload::Member{key:Key{definition:&other_definition,..key},index:0,reference:&fields[0],carrier:&carrier,declaration:&definition["fields"][0]},
            Payload::Member{key,index:1,reference:&fields[0],carrier:&carrier,declaration:&definition["fields"][0]},
            Payload::Member{key,index:0,reference:&other_fields[0],carrier:&carrier,declaration:&definition["fields"][0]},
            Payload::Member{key,index:0,reference:&fields[0],carrier:&other_carrier,declaration:&definition["fields"][0]},
            Payload::Member{key,index:0,reference:&fields[0],carrier:&carrier,declaration:&other_definition["fields"][0]},
            Payload::Key(key),
        ] { assert!(retain(&mut events,&["x"],substitute,&mut b).is_err()); }
        assert_eq!(events.len(),1);
        for i in 0..4095 { retain(&mut events,&["occurrence",&i.to_string()],Payload::Key(key),&mut b).unwrap(); }
        assert_eq!(events.len(),4096);
        assert!(retain(&mut events,&["overflow"],Payload::Key(key),&mut b).is_err());assert_eq!(events.len(),4096);
    }
}
