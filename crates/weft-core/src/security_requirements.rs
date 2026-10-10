//! Owner-issued immutable requirements. Declaration correspondence is not authority.
use crate::{application_ir, error::{Diagnostic, Result}, security_ir::{Rule, SecurityLogicalPlan}, security_query_profile::SecurityProfiledQuery, security_query_uses::QueryUse, security_scan_obligations::{ActionObligations, ScanObligations}};
use std::collections::BTreeSet;
fn fail() -> Diagnostic { Diagnostic::new("WFT-SECURITY-LOWERING-UNSUPPORTED", "capability", "Owner security requirement derivation refused") }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityOperatorMode<'a> { Disclosed, OriginalAuthorized(&'a str) }
#[derive(Debug)]
pub struct SecurityOperatorRequirement<'a> { usage: &'a QueryUse, mode: SecurityOperatorMode<'a> }
impl<'a> SecurityOperatorRequirement<'a> {
    pub fn usage(&self) -> &QueryUse { self.usage }
    pub fn mode(&self) -> SecurityOperatorMode<'a> { self.mode }
}
#[derive(Debug)]
pub struct SecurityActionRequirement<'a> { inventory: &'a ActionObligations, rules: Vec<&'a Rule> }
impl SecurityActionRequirement<'_> {
    pub fn inventory(&self) -> &ActionObligations { self.inventory }
    pub fn rules(&self) -> &[&Rule] { &self.rules }
}
#[derive(Debug)]
pub struct SecurityScanRequirement<'a> { inventory: &'a ScanObligations, actions: Vec<SecurityActionRequirement<'a>> }
impl SecurityScanRequirement<'_> {
    pub fn inventory(&self) -> &ScanObligations { self.inventory }
    pub fn actions(&self) -> &[SecurityActionRequirement<'_>] { &self.actions }
}
#[derive(Debug)]
pub struct SecurityOutputRequirement<'a> { position: usize, output: &'a application_ir::Output }
impl SecurityOutputRequirement<'_> {
    pub fn position(&self) -> usize { self.position }
    /// The actual expression retains revision-qualified lineage and computed type.
    /// COUNT/SUM are retained requirements, not admitted result contracts.
    pub fn output(&self) -> &application_ir::Output { self.output }
}
/// Only the compiler can construct this inventory. External declarations cannot
/// replace it, including by deserialization or assignment to public fields.
/// ```compile_fail
/// use weft_core::security_requirements::SecurityRequirements;
/// fn replace(r: &mut SecurityRequirements<'_>) { r.primary_action = "other"; }
/// ```
#[derive(Debug)]
pub struct SecurityRequirements<'a> {
    primary_action: &'a str,
    scans: Vec<SecurityScanRequirement<'a>>,
    operators: Vec<SecurityOperatorRequirement<'a>>,
    outputs: Vec<SecurityOutputRequirement<'a>>,
}
impl<'a> SecurityRequirements<'a> {
    pub fn primary_action(&self) -> &str { self.primary_action }
    pub fn scans(&self) -> &[SecurityScanRequirement<'a>] { &self.scans }
    pub fn operators(&self) -> &[SecurityOperatorRequirement<'a>] { &self.operators }
    pub fn outputs(&self) -> &[SecurityOutputRequirement<'a>] { &self.outputs }
    // Context construction first rechecks exact sources and actual query identity.
    pub(crate) fn derive(profile: &'a SecurityProfiledQuery<'a>, plan: &'a SecurityLogicalPlan) -> Result<Self> {
        Self::derive_with_budget(profile, plan, Budget { work: 1_000_000, text: 16_000_000 })
    }
    fn derive_with_budget(profile: &'a SecurityProfiledQuery<'a>, plan: &'a SecurityLogicalPlan, mut budget: Budget) -> Result<Self> {
        let query = profile.query();
        let primary_action = profile.primary_action();
        budget.charge(primary_action.len())?;
        if profile.obligations().len() != query.scans().len() { return Err(fail()); }
        let mut scans = Vec::new();
        for scan in profile.obligations() {
            budget.charge(scan.scan().len())?;
            if query.scans().get(scan.scan()) != Some(scan.target()) { return Err(fail()); }
            let mut actions = Vec::new();
            for inventory in scan.actions() {
                budget.charge(inventory.action().len())?;
                let mut rules = Vec::new();
                for rule in plan.rules() {
                    budget.charge(0)?; // Charge every scan/action/rule comparison.
                    if &rule.target == scan.target() && rule.actions.iter().any(|a| a == inventory.action()) {
                        budget.charge(rule.id.len())?;
                        rules.push(rule); // Borrow complete conditions/disclosures; never clone trees.
                    }
                }
                let actual: BTreeSet<_> = rules.iter().map(|r| r.id.as_str()).collect();
                let expected: BTreeSet<_> = inventory.rule_ids().iter().map(String::as_str).collect();
                if actual != expected || actual.len() != rules.len() { return Err(fail()); }
                actions.push(SecurityActionRequirement { inventory, rules });
            }
            if !actions.iter().any(|a| a.inventory.action() == primary_action) { return Err(fail()); }
            scans.push(SecurityScanRequirement { inventory: scan, actions });
        }
        let mut operators = Vec::new();
        for (usage, original) in profile.uses() {
            budget.charge(usage.scan.len())?;
            let mode = match original { Some(action) => { budget.charge(action.len())?; SecurityOperatorMode::OriginalAuthorized(action) }, None => SecurityOperatorMode::Disclosed };
            operators.push(SecurityOperatorRequirement { usage, mode });
        }
        let mut outputs = Vec::new();
        for (index, output) in query.application_plan().outputs.iter().enumerate() {
            budget.charge(output.name.len())?;
            outputs.push(SecurityOutputRequirement { position: index + 1, output });
        }
        Ok(Self { primary_action, scans, operators, outputs })
    }
}
struct Budget { work: usize, text: usize }
impl Budget {
    fn charge(&mut self, bytes: usize) -> Result<()> {
        if self.work == 0 || bytes > self.text { return Err(fail()); }
        self.work -= 1; self.text -= bytes; Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn budget_refuses_before_allocation_or_underflow() {
        let mut b = Budget { work: 1, text: 2 }; assert!(b.charge(3).is_err());
        assert_eq!((b.work,b.text),(1,2)); assert!(b.charge(2).is_ok());
        assert!(b.charge(0).is_err()); assert_eq!((b.work,b.text),(0,0));
    }
}

#[cfg(test)]
mod derivation_tests {
 use super::*;
 use crate::{json::sha256, model::Catalog, security_source::SecuritySourcePacket, security_query_profile::SecurityQueryProfile, security_query_uses::SecurityResolvedQuery};
 use serde_json::{json,Value};
 #[test]
 fn actual_admitted_count_cannot_issue_requirements_after_work_or_text_exhaustion() {
  let fixture:Value=serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();
  let source=&fixture["resolution"]["documents"][0];let mut doc=source["document"].clone();
  for element in doc["modules"][0]["elements"].as_array_mut().unwrap(){element["name"]=element["id"].clone();}
  let text=doc.to_string();let inputs=serde_json::from_value(json!([{"documentJson":text,"pin":{"documentId":doc["id"],"revision":source["revision"],"umfVersion":"0.8.0","sha256":sha256(text.as_bytes())},"selectedModuleIds":["m"]}])).unwrap();
  let catalog=Catalog::prepare_security(inputs).unwrap();
  let policy=fixture["policy"].to_string();let ontology=fixture["resolution"]["ontology"].to_string();
  let packet=SecuritySourcePacket::read(&policy,&ontology,&catalog).unwrap();let plan=SecurityLogicalPlan::read(packet,&catalog).unwrap();
  let raw=json!({"version":"weft.security.query-profile/0.1.0","id":"budget-profile","revision":"p1","action":"read","modelPins":catalog.pins(),"policySha256":sha256(policy.as_bytes()),"ontologySha256":sha256(ontology.as_bytes()),"binding":{"backendId":"fixture","backendVersion":"unqualified","targetProfile":"fixture-only","sha256":sha256(b"{}")},"targets":[{"documentId":"domain","moduleId":"m","elementId":"Resource"}],"bindings":[]}).to_string();
  let profile=SecurityQueryProfile::read(&raw,&plan,&catalog,"{}","fixture","unqualified","fixture-only").unwrap();
  let query=SecurityResolvedQuery::resolve("SELECT COUNT(*) FROM Resource r",&catalog,&plan,Default::default(),None).unwrap();
  let admitted=profile.admit_resolved_query(&query,&plan,&catalog,"{}","fixture","unqualified","fixture-only").unwrap();
  assert_eq!(SecurityRequirements::derive(&admitted,&plan).unwrap().scans().len(),1);
  for budget in [Budget{work:0,text:16_000_000},Budget{work:1_000_000,text:0},Budget{work:3,text:16_000_000}] {
   assert_eq!(SecurityRequirements::derive_with_budget(&admitted,&plan,budget).unwrap_err().code,"WFT-SECURITY-LOWERING-UNSUPPORTED");
  }
 }
}
