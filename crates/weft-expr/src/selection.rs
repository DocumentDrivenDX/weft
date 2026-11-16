// SPDX-License-Identifier: MIT OR Apache-2.0
//! The `weft.update-selection/1` predicate format (CONTRACT-009).
//!
//! A predicate is exactly `{compare:{field:FieldRef,op,value:CoreLiteral}}`,
//! `{and:[Predicate; 2..=64]}` or `{or:[Predicate; 2..=64]}`, at most
//! [`MAX_DEPTH`] deep with at most [`MAX_COMPARISONS`] comparisons. FieldRef and
//! CoreLiteral stay JSON here: whether a FieldRef names a Field of the selected
//! Record and whether a literal is in that Field's domain is the caller's check,
//! made through the visitor passed to [`walk`].
use crate::compare::CompareOp;
use serde_json::{json, Value};

pub const PROTOCOL: &str = "weft.update-selection/1";
pub const MAX_DEPTH: usize = 24;
pub const MIN_OPERANDS: usize = 2;
pub const MAX_OPERANDS: usize = 64;
pub const MAX_COMPARISONS: usize = 256;

/// One comparison: a local FieldRef (`{module, element}`), an operator and a UMF
/// CoreLiteral.
#[derive(Debug, Clone, PartialEq)]
pub struct Compare {
    pub field: Value,
    pub op: CompareOp,
    pub value: Value,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    Compare(Compare),
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
}
/// Why a predicate is structurally outside the format. Callers attach their
/// own diagnostic codes and messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// Nesting deeper than [`MAX_DEPTH`].
    Depth,
    /// A node that is not a one-member object.
    Node,
    /// A `compare` member that is not an object.
    CompareObject,
    /// A `compare` object without exactly `field`, `op` and `value`.
    CompareMembers,
    /// An operator outside `eq ne lt le gt ge`.
    Operator,
    /// Neither `compare`, nor `and`/`or` with at least [`MIN_OPERANDS`] operands.
    Operands,
    /// More than [`MAX_OPERANDS`] operands.
    OperandCount,
    /// More than [`MAX_COMPARISONS`] comparisons.
    Comparisons,
}

/// Decode a predicate, calling `visit` on each `compare` object (members checked,
/// operator not yet parsed) in source pre-order. A `visit` error stops the walk;
/// structural refusals go through `refuse`. The order of checks is fixed so that
/// every caller reports the same first refusal.
pub fn walk<E>(
    root: &Value,
    mut visit: impl FnMut(&Value) -> Result<(), E>,
    refuse: impl Fn(Refusal) -> E,
) -> Result<Predicate, E> {
    let mut comparisons = 0;
    walk_node(root, 1, &mut comparisons, &mut visit, &refuse)
}
fn walk_node<E>(
    node: &Value,
    depth: usize,
    comparisons: &mut usize,
    visit: &mut impl FnMut(&Value) -> Result<(), E>,
    refuse: &impl Fn(Refusal) -> E,
) -> Result<Predicate, E> {
    if depth > MAX_DEPTH {
        return Err(refuse(Refusal::Depth));
    }
    let object = node.as_object().filter(|o| o.len() == 1).ok_or_else(|| refuse(Refusal::Node))?;
    if let Some(compare) = object.get("compare") {
        let members = compare.as_object().ok_or_else(|| refuse(Refusal::CompareObject))?;
        if members.len() != 3 || ["field", "op", "value"].iter().any(|k| !members.contains_key(*k)) {
            return Err(refuse(Refusal::CompareMembers));
        }
        visit(compare)?;
        let op = compare["op"].as_str().and_then(CompareOp::parse).ok_or_else(|| refuse(Refusal::Operator))?;
        *comparisons += 1;
        if *comparisons > MAX_COMPARISONS {
            return Err(refuse(Refusal::Comparisons));
        }
        return Ok(Predicate::Compare(Compare { field: compare["field"].clone(), op, value: compare["value"].clone() }));
    }
    let (and, children) = match (object.get("and"), object.get("or")) {
        (Some(children), _) => (true, children),
        (None, Some(children)) => (false, children),
        _ => return Err(refuse(Refusal::Operands)),
    };
    let children = children.as_array().filter(|c| c.len() >= MIN_OPERANDS).ok_or_else(|| refuse(Refusal::Operands))?;
    if children.len() > MAX_OPERANDS {
        return Err(refuse(Refusal::OperandCount));
    }
    let children = children
        .iter()
        .map(|child| walk_node(child, depth + 1, comparisons, visit, refuse))
        .collect::<Result<Vec<_>, E>>()?;
    Ok(if and { Predicate::And(children) } else { Predicate::Or(children) })
}

impl Predicate {
    /// The format's JSON. `walk(&p.to_json(), ..)` returns `p`.
    pub fn to_json(&self) -> Value {
        match self {
            Self::Compare(c) => json!({"compare": {"field": c.field, "op": c.op.name(), "value": c.value}}),
            Self::And(children) => json!({"and": children.iter().map(Self::to_json).collect::<Vec<_>>()}),
            Self::Or(children) => json!({"or": children.iter().map(Self::to_json).collect::<Vec<_>>()}),
        }
    }
    /// Comparisons in source pre-order.
    pub fn comparisons(&self) -> Vec<&Compare> {
        let mut out = Vec::new();
        fn visit<'a>(p: &'a Predicate, out: &mut Vec<&'a Compare>) {
            match p {
                Predicate::Compare(c) => out.push(c),
                Predicate::And(children) | Predicate::Or(children) => children.iter().for_each(|c| visit(c, out)),
            }
        }
        visit(self, &mut out);
        out
    }
    /// Every compared FieldRef once, in first source-use order (CONTRACT-009
    /// plan `dependencies`).
    pub fn dependencies(&self) -> Vec<Value> {
        let mut out: Vec<Value> = Vec::new();
        for c in self.comparisons() {
            if !out.contains(&c.field) {
                out.push(c.field.clone());
            }
        }
        out
    }
    /// Append `child` to a conjunction (`and == true`) or disjunction under
    /// construction, splicing a child of the same connective into it.
    pub fn splice(operands: &mut Vec<Predicate>, and: bool, child: Predicate) {
        match child {
            Self::And(children) if and => operands.extend(children),
            Self::Or(children) if !and => operands.extend(children),
            other => operands.push(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cmp(element: &str, op: &str) -> Value {
        json!({"compare":{"field":{"module":"m","element":element},"op":op,"value":{"integerToken":"1"}}})
    }
    fn decode(v: &Value) -> Result<Predicate, Refusal> {
        walk(v, |_| Ok(()), |r| r)
    }
    #[test]
    fn round_trip_and_dependencies() {
        let source = json!({"and":[cmp("a","eq"), {"or":[cmp("b","lt"), cmp("a","ge")]}]});
        let p = decode(&source).unwrap();
        assert_eq!(p.to_json(), source);
        assert_eq!(p.dependencies(), vec![json!({"module":"m","element":"a"}), json!({"module":"m","element":"b"})]);
        assert_eq!(p.comparisons().iter().map(|c| c.op).collect::<Vec<_>>(), [CompareOp::Eq, CompareOp::Lt, CompareOp::Ge]);
    }
    #[test]
    fn structural_refusals() {
        assert_eq!(decode(&json!({"and":[cmp("a","eq")]})), Err(Refusal::Operands));
        assert_eq!(decode(&json!({"not":cmp("a","eq")})), Err(Refusal::Operands));
        assert_eq!(decode(&json!({"and":[cmp("a","eq")],"or":[]})), Err(Refusal::Node));
        assert_eq!(decode(&json!({"compare":{"field":{},"op":"eq"}})), Err(Refusal::CompareMembers));
        assert_eq!(decode(&json!({"compare":[]})), Err(Refusal::CompareObject));
        assert_eq!(decode(&cmp("a", "like")), Err(Refusal::Operator));
        let wide: Vec<Value> = (0..65).map(|_| cmp("a", "eq")).collect();
        assert_eq!(decode(&json!({"or": wide})), Err(Refusal::OperandCount));
        let mut deep = cmp("a", "eq");
        for _ in 0..MAX_DEPTH { deep = json!({"and":[deep.clone(), cmp("b","eq")]}); }
        assert_eq!(decode(&deep), Err(Refusal::Depth));
        let many: Vec<Value> = (0..64).map(|_| json!({"and":[cmp("a","eq"), cmp("a","eq"), cmp("a","eq"), cmp("a","eq"), cmp("a","eq")]})).collect();
        assert_eq!(decode(&json!({"or": many})), Err(Refusal::Comparisons));
    }
    #[test]
    fn visitor_errors_come_first_in_pre_order() {
        let source = json!({"and":[cmp("bad","eq"), cmp("a","like")]});
        assert_eq!(walk(&source, |c| if c["field"]["element"] == "bad" { Err("domain") } else { Ok(()) }, |_| "structure"), Err("domain"));
    }
    #[test]
    fn splice_flattens_same_connective() {
        let mut operands = Vec::new();
        Predicate::splice(&mut operands, true, Predicate::And(vec![decode(&cmp("a","eq")).unwrap(), decode(&cmp("b","eq")).unwrap()]));
        Predicate::splice(&mut operands, true, Predicate::Or(vec![decode(&cmp("c","eq")).unwrap(), decode(&cmp("d","eq")).unwrap()]));
        assert_eq!(operands.len(), 3);
    }
}
