//! Original declaration custody only. Parameter interpretation and admission are separate.
use crate::{
    backend::{Obligation, Status},
    error::{Diagnostic, Result},
    security_backend::RegisteredSecurityDeclaration,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-LOWERING-UNSUPPORTED",
        "capability",
        "Obligation declaration custody refused",
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
    fn value(&mut self, value: &Value, depth: usize) -> Result<()> {
        self.charge(0)?;
        if depth > 64 {
            return Err(fail());
        }
        match value {
            Value::String(s) => self.charge(s.len())?,
            Value::Number(n) => self.charge(n.as_str().len())?,
            Value::Array(a) => {
                for v in a {
                    self.value(v, depth + 1)?;
                }
            }
            Value::Object(o) => {
                for (k, v) in o {
                    self.charge(k.len())?;
                    self.value(v, depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn obligation(&mut self, o: &Obligation) -> Result<()> {
        self.charge(o.id.len())?;
        self.charge(o.failure_code.len())?;
        self.value(&o.parameters, 0)
    }
}
/// Every member borrows the same immutable registry entry; no constructor is public.
#[allow(dead_code)]
pub(crate) struct ObligationCustody<'a> {
    declaration: RegisteredSecurityDeclaration<'a>,
    entries: BTreeMap<&'a str, Entry<'a>>,
    selected: BTreeSet<&'a str>,
}
/// Private paired subjects; selected-wide requirements never invent semantic scope.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ScopedSubject<'a> {
    Semantic { source: &'a str, scope: crate::security_semantic_coverage::CoverageScope },
    SelectedCapability { profile_requirement_id: &'a str },
}
#[allow(dead_code)]
pub(crate) struct ScopedDeclaration<'a> {
    pub(crate) original: &'a Obligation,
    pub(crate) origins: &'a [&'a str],
    pub(crate) subjects: BTreeSet<ScopedSubject<'a>>,
    pub(crate) site: &'a str,
    pub(crate) prerequisites: BTreeSet<&'a str>,
    pub(crate) cases: BTreeSet<&'a str>,
}
struct Entry<'a> {
    original: &'a Obligation,
    capabilities: Vec<&'a str>,
}
#[allow(dead_code)]
impl<'a> ObligationCustody<'a> {
    pub(crate) fn collect(
        declaration: RegisteredSecurityDeclaration<'a>,
        ids: &[String],
        allow_candidate: bool,
    ) -> Result<Self> {
        Self::collect_budget(
            declaration,
            ids,
            allow_candidate,
            &mut Budget {
                work: 1_000_000,
                text: 16_000_000,
            },
        )
    }
    fn collect_budget(
        declaration: RegisteredSecurityDeclaration<'a>,
        ids: &[String],
        allow: bool,
        b: &mut Budget,
    ) -> Result<Self> {
        b.charge(declaration.manifest_json().len())?;
        let manifest = declaration.manifest();
        if ids.is_empty() || ids.len() > 4096 || manifest.capabilities.len() > 4096 {
            return Err(fail());
        }
        let mut capabilities = BTreeMap::new();
        for c in &manifest.capabilities {
            b.charge(c.id.len())?;
            if capabilities.insert(c.id.as_str(), c).is_some() {
                return Err(fail());
            }
        }
        let mut selected: BTreeSet<&'a str> = BTreeSet::new();
        let mut entries: BTreeMap<&str, Entry<'a>> = BTreeMap::new();
        for id in ids {
            b.charge(id.len())?;
            let c = capabilities.get(id.as_str()).ok_or_else(fail)?;
            if !selected.insert(c.id.as_str()) {
                return Err(fail());
            }
            if c.status == Status::Unsupported || (c.status == Status::Candidate && !allow) {
                return Err(fail());
            }
            let mut applicable = false;
            for t in &c.target_profiles {
                b.charge(t.len())?;
                if t == &declaration.target().id {
                    applicable = true;
                }
            }
            if !applicable || c.obligations.len() > 4096 {
                return Err(fail());
            }
            let mut local = BTreeSet::new();
            for o in &c.obligations {
                b.obligation(o)?;
                if !local.insert(o.id.as_str()) {
                    return Err(fail());
                }
                if let Some(entry) = entries.get_mut(o.id.as_str()) {
                    // Comparison traversal is charged again, independently of source indexing.
                    b.obligation(entry.original)?;
                    b.obligation(o)?;
                    if entry.original != o {
                        return Err(fail());
                    }
                    b.charge(c.id.len())?;
                    entry.capabilities.push(&c.id);
                } else {
                    b.charge(c.id.len())?;
                    entries.insert(
                        &o.id,
                        Entry {
                            original: o,
                            capabilities: vec![&c.id],
                        },
                    );
                }
            }
        }
        Ok(Self {
            declaration,
            entries,
            selected,
        })
    }
    /// Versioned declaration decoding only; no independent requirements or evidence.
    pub(crate) fn project_scoped(&self) -> Result<Vec<ScopedDeclaration<'_>>> {
        self.scoped_budget(&mut Budget { work: 1_000_000, text: 16_000_000 })
    }
    fn scoped_budget(&self, b: &mut Budget) -> Result<Vec<ScopedDeclaration<'_>>> {
        use crate::backend::ObligationOwner;
        use crate::security_semantic_coverage::CoverageScope;
        fn object(v: &Value, keys: &[&str]) -> Result<()> {
            let o = v.as_object().ok_or_else(fail)?;
            if o.len() != keys.len() || keys.iter().any(|k| !o.contains_key(*k)) { return Err(fail()); }
            Ok(())
        }
        fn id<'v>(v: &'v Value, b: &mut Budget) -> Result<&'v str> {
            let s = v.as_str().ok_or_else(fail)?;
            b.charge(s.len())?;
            if s.is_empty() || s.len() > 4096 || s.contains('\0') { return Err(fail()); }
            Ok(s)
        }
        fn ids<'v>(v: &'v Value, nonempty: bool, b: &mut Budget) -> Result<BTreeSet<&'v str>> {
            let a = v.as_array().ok_or_else(fail)?;
            if a.len() > 4096 || (nonempty && a.is_empty()) { return Err(fail()); }
            let mut out = BTreeSet::new();
            for v in a { if !out.insert(id(v,b)?) { return Err(fail()); } }
            Ok(out)
        }
        if self.entries.is_empty() || self.entries.len() > 4096 { return Err(fail()); }
        for selected in &self.selected {
            b.charge(selected.len())?;
            if selected.is_empty() || selected.len()>4096 || selected.contains('\0') { return Err(fail()); }
        }
        let mut result = Vec::new();
        let mut atoms = 0usize;
        for (key, e) in &self.entries {
            b.obligation(e.original)?;
            if key.is_empty() || key.len()>4096 || key.contains('\0') { return Err(fail()); }
            let p = &e.original.parameters;
            object(p, &["version","subjects","enforcementSite","prerequisites","evidenceCaseIds"])?;
            if p["version"].as_str()!=Some("weft.security.admission-obligation/0.2.0") { return Err(fail()); }
            let site = id(&p["enforcementSite"], b)?;
            match (&e.original.owner,site) {
                (ObligationOwner::Host,"host") | (ObligationOwner::Backend,"backend"|"native") => {},
                _ => return Err(fail()),
            }
            let a = p["subjects"].as_array().ok_or_else(fail)?;
            if a.is_empty() || a.len()>4096 { return Err(fail()); }
            let mut subjects = BTreeSet::new();
            let mut kind = None;
            for v in a {
                b.charge(0)?;
                let subject = match v["kind"].as_str() {
                    Some("semantic") => {
                        object(v,&["kind","source","scope"])?;
                        let q=&v["scope"];
                        let scope=match q["kind"].as_str() {
                            Some("application") => { object(q,&["kind"])?; CoverageScope::Application },
                            Some("scan-action") => {
                                object(q,&["kind","scan","action"])?;
                                CoverageScope::ScanAction { scan:id(&q["scan"],b)?.to_owned(), action:id(&q["action"],b)?.to_owned() }
                            },
                            _ => return Err(fail()),
                        };
                        ScopedSubject::Semantic { source:id(&v["source"],b)?,scope }
                    },
                    Some("selected-capability") => {
                        object(v,&["kind","profileRequirementId"])?;
                        ScopedSubject::SelectedCapability { profile_requirement_id:id(&v["profileRequirementId"],b)? }
                    },
                    _ => return Err(fail()),
                };
                let semantic = matches!(subject,ScopedSubject::Semantic { .. });
                if kind.is_some_and(|k|k!=semantic) || !subjects.insert(subject) { return Err(fail()); }
                kind=Some(semantic);
            }
            let prerequisites=ids(&p["prerequisites"],false,b)?;
            let cases=ids(&p["evidenceCaseIds"],true,b)?;
            for origin in &e.capabilities {
                b.charge(origin.len())?;
                for _ in &subjects { for case in &cases {
                    b.charge(case.len())?;
                    atoms=atoms.checked_add(1).ok_or_else(fail)?;
                    if atoms>4096 { return Err(fail()); }
                } }
            }
            result.push(ScopedDeclaration { original:e.original,origins:&e.capabilities,subjects,site,prerequisites,cases });
        }
        // Complete original-ID DAG, including disconnected components.
        let indices:BTreeMap<_,_>=result.iter().enumerate().map(|(i,d)|(d.original.id.as_str(),i)).collect();
        let mut edges=vec![Vec::new();result.len()];
        let mut incoming=vec![0usize;result.len()];
        for (i,d) in result.iter().enumerate() { for prerequisite in &d.prerequisites {
            b.charge(prerequisite.len())?;
            let j=*indices.get(prerequisite).ok_or_else(fail)?;
            edges[j].push(i); incoming[i]+=1;
        } }
        let mut ready:Vec<_>=incoming.iter().enumerate().filter_map(|(i,n)|(*n==0).then_some(i)).collect();
        let mut visited=0;
        while let Some(i)=ready.pop() { b.charge(0)?; visited+=1; for &j in &edges[i] {
            b.charge(0)?;incoming[j]-=1;if incoming[j]==0 {ready.push(j);}
        } }
        if visited!=result.len() { return Err(fail()); }
        Ok(result)
    }
    /// Closed declaration projection only; neither source/evidence existence nor enforcement.
    pub(crate) fn project_admission(
        &self,
    ) -> Result<Vec<crate::security_lowering::SecurityAdmissionObligation>> {
        self.project_budget(&mut Budget {
            work: 1_000_000,
            text: 16_000_000,
        })
    }
    /// Exact source-ID correspondence only, not source meaning or native case qualification.
    pub(crate) fn project_for_context(
        &self,
        ctx: &crate::security_backend::SecurityBackendContext<'_>,
    ) -> Result<Vec<crate::security_lowering::SecurityAdmissionObligation>> {
        let manifest = self.declaration.manifest();
        if manifest.backend_id != ctx.backend_id()
            || manifest.backend_version != ctx.backend_version()
            || self.declaration.target().id != ctx.target_profile()
        {
            return Err(fail());
        }
        let result = self.project_admission()?;
        let required = crate::security_obligation_sources::derive(ctx)?;
        let mut budget = Budget {
            work: 1_000_000,
            text: 16_000_000,
        };
        let mut declared = BTreeSet::new();
        for o in &result {
            for source in &o.semantic_sources {
                budget.charge(source.len())?;
                declared.insert(source.as_str());
            }
        }
        for source in &required {
            budget.charge(source.len())?;
        }
        if declared != required.iter().map(String::as_str).collect() {
            return Err(fail());
        }
        Ok(result)
    }
    fn project_budget(
        &self,
        b: &mut Budget,
    ) -> Result<Vec<crate::security_lowering::SecurityAdmissionObligation>> {
        use crate::backend::ObligationOwner;
        use crate::security_lowering::{SecurityAdmissionObligation, SecurityEnforcementSite};
        fn strings(v: &Value, nonempty: bool, b: &mut Budget) -> Result<Vec<String>> {
            let a = v.as_array().ok_or_else(fail)?;
            if a.len() > 4096 || (nonempty && a.is_empty()) {
                return Err(fail());
            }
            let mut seen = BTreeSet::new();
            let mut result = Vec::with_capacity(a.len());
            for value in a {
                let s = value.as_str().ok_or_else(fail)?;
                b.charge(s.len())?;
                if s.is_empty() || s.len() > 4096 || s.contains('\0') || !seen.insert(s) {
                    return Err(fail());
                }
                result.push(s.to_owned());
            }
            Ok(result)
        }
        if self.entries.is_empty() || self.entries.len() > 4096 {
            return Err(fail());
        }
        let mut result = Vec::with_capacity(self.entries.len());
        let mut indices = BTreeMap::new();
        for (id, entry) in &self.entries {
            let o = entry.original;
            if id.len() > 4096 {
                return Err(fail());
            }
            b.obligation(o)?;
            let p = o.parameters.as_object().ok_or_else(fail)?;
            let keys = [
                "version",
                "semanticSources",
                "enforcementSite",
                "prerequisites",
                "evidenceCaseIds",
            ];
            if p.len() != keys.len()
                || keys.iter().any(|key| !p.contains_key(*key))
                || p["version"].as_str() != Some("weft.security.admission-obligation/0.1.0")
            {
                return Err(fail());
            }
            let site = match (&o.owner, p["enforcementSite"].as_str()) {
                (ObligationOwner::Host, Some("host")) => SecurityEnforcementSite::Host,
                (ObligationOwner::Backend, Some("backend")) => SecurityEnforcementSite::Backend,
                (ObligationOwner::Backend, Some("native")) => SecurityEnforcementSite::Native,
                _ => return Err(fail()),
            };
            let semantic_sources = strings(&p["semanticSources"], true, b)?;
            let prerequisites = strings(&p["prerequisites"], false, b)?;
            let evidence_case_ids = strings(&p["evidenceCaseIds"], true, b)?;
            b.charge(id.len())?;
            b.charge(o.failure_code.len())?;
            indices.insert(*id, result.len());
            result.push(SecurityAdmissionObligation {
                id: (*id).to_owned(),
                semantic_sources,
                enforcement_site: site,
                prerequisites,
                failure_code: o.failure_code.clone(),
                evidence_case_ids,
            });
        }
        // Kahn's algorithm covers every component, including disconnected cycles.
        let mut dependants = vec![Vec::new(); result.len()];
        let mut remaining = vec![0usize; result.len()];
        for (i, o) in result.iter().enumerate() {
            for prerequisite in &o.prerequisites {
                b.charge(prerequisite.len())?;
                let j = *indices.get(prerequisite.as_str()).ok_or_else(fail)?;
                dependants[j].push(i);
                remaining[i] += 1;
            }
        }
        let mut ready: Vec<usize> = remaining
            .iter()
            .enumerate()
            .filter_map(|(i, n)| (*n == 0).then_some(i))
            .collect();
        let mut visited = 0;
        while let Some(i) = ready.pop() {
            b.charge(0)?;
            visited += 1;
            for &j in &dependants[i] {
                b.charge(0)?;
                remaining[j] -= 1;
                if remaining[j] == 0 {
                    ready.push(j);
                }
            }
        }
        if visited != result.len() {
            return Err(fail());
        }
        Ok(result)
    }
    /// This equality must use an independently trusted host selection, not a packet's echo.
    pub(crate) fn same_registration(&self, host: &RegisteredSecurityDeclaration<'_>) -> Result<()> {
        let mut b = Budget {
            work: 1_000_000,
            text: 16_000_000,
        };
        b.charge(self.declaration.manifest_json().len())?;
        b.charge(host.manifest_json().len())?;
        b.charge(self.declaration.target().id.len())?;
        b.charge(host.target().id.len())?;
        if self.declaration.manifest_json() != host.manifest_json()
            || self.declaration.target().id != host.target().id
        {
            return Err(fail());
        }
        Ok(())
    }
    /// Includes selected capabilities with no obligation occurrences.
    pub(crate) fn selected_capabilities(&self) -> &BTreeSet<&'a str> {
        &self.selected
    }
    pub(crate) fn original(&self, id: &str) -> Result<&'a Obligation> {
        self.entries.get(id).map(|e| e.original).ok_or_else(fail)
    }
    pub(crate) fn capability_sources(&self, id: &str) -> Result<&[&'a str]> {
        self.entries
            .get(id)
            .map(|e| e.capabilities.as_slice())
            .ok_or_else(fail)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::security_backend::{SecurityBackend, SecurityBackendContext, SecurityRegistry};
    use crate::security_lowering::SecurityLowering;
    use serde_json::json;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    struct Backend {
        before: String,
        after: String,
        drift: Arc<AtomicBool>,
    }
    impl SecurityBackend for Backend {
        fn manifest_json(&self) -> &str {
            if self.drift.load(Ordering::SeqCst) {
                &self.after
            } else {
                &self.before
            }
        }
        fn lower(&self, _: &SecurityBackendContext<'_>) -> Result<SecurityLowering> {
            panic!("custody must never invoke lower")
        }
    }
    fn obligation(id: &str) -> Value {
        json!({"id":id,"parameters":{"opaque":{"future":[null,true,"preserved",{"integerToken":"9007199254740993"}]}},"owner":"host","failureCode":"WFT-CUSTODY"})
    }
    pub(super) fn manifest() -> Value {
        json!({"interfaceVersion":"weft-security-backend/0.1.0","backendId":"fixture","backendVersion":"v1","sourceProfiles":[{"dialect":"weft-sql/0.2.0","applicationIr":"weft-ir/0.2.0","policy":"0.1.0","ontology":"0.1.0","securityIr":"weft.security.logical-ir/0.1.0"}],"bindingProfile":"fixture-uninterpreted","targetProfiles":[{"id":"one","engine":"fixture","engineVersion":"v1","sessionSettings":{},"storageLayoutRevision":"v1","publicationRevision":"v1"},{"id":"two","engine":"fixture","engineVersion":"v1","sessionSettings":{},"storageLayoutRevision":"v1","publicationRevision":"v1"}],"capabilities":[{"id":"a","targetProfiles":["one"],"languageProfiles":[{"dialectProfile":"weft-sql/0.2.0","irVersion":"weft-ir/0.2.0"}],"logicalDomain":{"fixture":true},"resultDomain":{"fixture":true},"constraints":[],"obligations":[obligation("shared"),obligation("a-only")],"status":"supported","evidence":["fixture-only"]}],"evidence":["fixture-only"]})
    }
    fn registry(raw: String) -> (SecurityRegistry, Arc<AtomicBool>) {
        let drift = Arc::new(AtomicBool::new(false));
        let mut registry = SecurityRegistry::default();
        registry
            .register(Backend {
                before: raw,
                after: "{}".into(),
                drift: drift.clone(),
            })
            .unwrap();
        (registry, drift)
    }
    fn selected(r: &SecurityRegistry) -> RegisteredSecurityDeclaration<'_> {
        r.declaration_for("fixture", "v1", "one").unwrap()
    }
    fn ids(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn original_bytes_and_opaque_parameters_survive_callback_drift() {
        let mut m = manifest();
        m["capabilities"][0]["obligations"][0]["parameters"]["number"] = json!(1);
        let raw = format!(
            " \n{}\n",
            m.to_string().replace("\"number\":1", "\"number\":1.00e0")
        );
        let (r, drift) = registry(raw.clone());
        drift.store(true, Ordering::SeqCst);
        let d = selected(&r);
        assert_eq!(d.manifest_json(), raw);
        let inventory = ObligationCustody::collect(d, &ids(&["a"]), false).unwrap();
        assert_eq!(
            inventory.original("shared").unwrap().parameters["number"]
                .as_number()
                .unwrap()
                .as_str(),
            "1.00e0"
        );
        assert_eq!(
            inventory.original("shared").unwrap().parameters["opaque"],
            obligation("shared")["parameters"]["opaque"]
        );
        assert_eq!(inventory.capability_sources("shared").unwrap(), ["a"]);
        assert!(inventory.original("missing").is_err());
    }
    #[test]
    fn complete_inventory_and_repeated_provenance_are_retained() {
        let mut m = manifest();
        let mut c = m["capabilities"][0].clone();
        c["id"] = json!("b");
        c["obligations"] = json!([obligation("shared"), obligation("b-only")]);
        m["capabilities"].as_array_mut().unwrap().push(c);
        let (r, _) = registry(m.to_string());
        let inventory = ObligationCustody::collect(selected(&r), &ids(&["a", "b"]), false).unwrap();
        assert_eq!(inventory.entries.len(), 3);
        assert_eq!(inventory.capability_sources("shared").unwrap(), ["a", "b"]);
        assert!(inventory.original("a-only").is_ok());
        assert!(inventory.original("b-only").is_ok());
    }
    #[test]
    fn exact_selection_retains_empty_capabilities_from_immutable_registration() {
        let mut m = manifest();
        let mut c = m["capabilities"][0].clone();
        c["id"] = json!("b");
        c["obligations"] = json!([]);
        m["capabilities"].as_array_mut().unwrap().push(c);
        let (r, drift) = registry(m.to_string());
        let only_a = ObligationCustody::collect(selected(&r), &ids(&["a"]), false).unwrap();
        let mut caller_selection = ids(&["a", "b"]);
        let both = ObligationCustody::collect(selected(&r), &caller_selection, false).unwrap();
        caller_selection.clear();
        drop(caller_selection);
        drift.store(true, Ordering::SeqCst);
        assert_eq!(only_a.entries.len(), both.entries.len());
        assert_eq!(only_a.capability_sources("shared").unwrap(), both.capability_sources("shared").unwrap());
        assert_eq!(only_a.selected_capabilities(), &BTreeSet::from(["a"]));
        assert_eq!(both.selected_capabilities(), &BTreeSet::from(["a", "b"]));
        let b = &selected(&r).manifest().capabilities[1].id;
        assert!(std::ptr::eq(*both.selected_capabilities().get("b").unwrap(), b.as_str()));
        assert!(ObligationCustody::collect(selected(&r), &ids(&["b", "b"]), false).is_err());
    }
    fn scoped_parameters() -> Value {
        json!({"version":"weft.security.admission-obligation/0.2.0",
            "subjects":[{"kind":"semantic","source":"source-a","scope":{"kind":"scan-action","scan":"s0","action":"read"}},
                        {"kind":"semantic","source":"source-a","scope":{"kind":"application"}}],
            "enforcementSite":"host","prerequisites":[],"evidenceCaseIds":["case-a","case-b"]})
    }
    fn scoped_manifest() -> Value {
        let mut m=manifest();
        for o in m["capabilities"][0]["obligations"].as_array_mut().unwrap() { o["parameters"]=scoped_parameters(); }
        m
    }
    #[test]
    fn scoped_decoder_retains_paired_subjects_originals_and_all_origins() {
        let mut m=scoped_manifest();
        let mut c=m["capabilities"][0].clone();c["id"]=json!("b");
        m["capabilities"].as_array_mut().unwrap().push(c);
        let (r,_)=registry(m.to_string());
        let custody=ObligationCustody::collect(selected(&r),&ids(&["a","b"]),false).unwrap();
        let decoded=custody.project_scoped().unwrap();
        assert_eq!(decoded.len(),2);
        assert_eq!(decoded[0].subjects,BTreeSet::from([
            ScopedSubject::Semantic {source:"source-a",scope:crate::security_semantic_coverage::CoverageScope::ScanAction {scan:"s0".into(),action:"read".into()}},
            ScopedSubject::Semantic {source:"source-a",scope:crate::security_semantic_coverage::CoverageScope::Application},
        ]));
        assert_eq!(decoded[0].origins,["a","b"]);
        assert!(std::ptr::eq(decoded[0].original,custody.original(&decoded[0].original.id).unwrap()));
        assert_eq!(decoded[0].cases,BTreeSet::from(["case-a","case-b"]));
        assert!(custody.project_admission().is_err());
        let mut measured=Budget {work:1_000_000,text:16_000_000};
        custody.scoped_budget(&mut measured).unwrap();
        let work=1_000_000-measured.work;let text=16_000_000-measured.text;
        assert!(custody.scoped_budget(&mut Budget {work,text}).is_ok());
        assert!(custody.scoped_budget(&mut Budget {work:work-1,text}).is_err());
        assert!(custody.scoped_budget(&mut Budget {work,text:text-1}).is_err());
        let mut legacy=manifest();
        for o in legacy["capabilities"][0]["obligations"].as_array_mut().unwrap() {
            o["parameters"]=json!({"version":"weft.security.admission-obligation/0.1.0","semanticSources":["legacy-source"],"enforcementSite":"host","prerequisites":[],"evidenceCaseIds":["legacy-case"]});
        }
        let (legacy_registry,_)=registry(legacy.to_string());
        let legacy_custody=ObligationCustody::collect(selected(&legacy_registry),&ids(&["a"]),false).unwrap();
        assert!(legacy_custody.project_admission().is_ok());
        assert!(legacy_custody.project_scoped().is_err());
        let mut m=scoped_manifest();
        for o in m["capabilities"][0]["obligations"].as_array_mut().unwrap() {
            o["parameters"]["subjects"]=json!([{"kind":"selected-capability","profileRequirementId":"deployment-a"}]);
        }
        let (r,_)=registry(m.to_string());
        let custody=ObligationCustody::collect(selected(&r),&ids(&["a"]),false).unwrap();
        assert!(matches!(custody.project_scoped().unwrap()[0].subjects.first(),Some(ScopedSubject::SelectedCapability {profile_requirement_id:"deployment-a"})));
    }
    #[test]
    fn scoped_decoder_refuses_ambiguous_subjects_and_legacy_parameters() {
        let mut mutants=Vec::new();
        let base=scoped_parameters();
        let mut v=base.clone();v["subjects"]=json!([]);mutants.push(v);
        let mut v=base.clone();v["subjects"][1]=v["subjects"][0].clone();mutants.push(v);
        let mut v=base.clone();v["subjects"][1]=json!({"kind":"selected-capability","profileRequirementId":"deployment"});mutants.push(v);
        let mut v=base.clone();v["subjects"][0]["scope"]["extra"]=json!(true);mutants.push(v);
        let mut v=base.clone();v["subjects"][0]["source"]=json!("x\0y");mutants.push(v);
        let mut v=base.clone();v["evidenceCaseIds"]=json!(["case-a","case-a"]);mutants.push(v);
        let mut v=base.clone();v["enforcementSite"]=json!("native");mutants.push(v);
        let mut v=base.clone();v["version"]=json!("weft.security.admission-obligation/0.1.0");mutants.push(v);
        for (characters,empty) in [(2048,false),(2049,false),(2048,true),(2049,true)] {
            let mut m=scoped_manifest();
            let name="é".repeat(characters);
            let selections=if empty {
                let mut c=m["capabilities"][0].clone();c["id"]=json!(name);c["obligations"]=json!([]);
                m["capabilities"].as_array_mut().unwrap().push(c);
                vec!["a".to_owned(),name]
            } else {m["capabilities"][0]["id"]=json!(name);vec![name]};
            let (r,_)=registry(m.to_string());
            let custody=ObligationCustody::collect(selected(&r),&selections,false).unwrap();
            assert_eq!(custody.project_scoped().is_ok(),characters==2048);
        }
        for parameters in mutants {
            let mut m=scoped_manifest();m["capabilities"][0]["obligations"][0]["parameters"]=parameters;
            let (r,_)=registry(m.to_string());
            assert!(ObligationCustody::collect(selected(&r),&ids(&["a"]),false).unwrap().project_scoped().is_err());
        }
    }
    #[test]
    fn scoped_decoder_checks_disconnected_dag_and_repeated_origin_expansion_bound() {
        for prerequisites in [json!(["missing"]),json!(["shared"])] {
            let mut m=scoped_manifest();m["capabilities"][0]["obligations"][0]["parameters"]["prerequisites"]=prerequisites;
            let (r,_)=registry(m.to_string());
            assert!(ObligationCustody::collect(selected(&r),&ids(&["a"]),false).unwrap().project_scoped().is_err());
        }
        let mut m=scoped_manifest();
        m["capabilities"][0]["obligations"][1]["parameters"]["prerequisites"]=json!(["shared"]);
        let cases:Vec<_>=(0..1024).map(|i|format!("case-{i}")).collect();
        for o in m["capabilities"][0]["obligations"].as_array_mut().unwrap() {o["parameters"]["evidenceCaseIds"]=json!(cases);}
        let mut c=m["capabilities"][0].clone();c["id"]=json!("b");
        m["capabilities"].as_array_mut().unwrap().push(c);
        let (r,_)=registry(m.to_string());
        assert!(ObligationCustody::collect(selected(&r),&ids(&["a"]),false).unwrap().project_scoped().is_ok());
        assert!(ObligationCustody::collect(selected(&r),&ids(&["a","b"]),false).unwrap().project_scoped().is_err());
        let mut boundary=scoped_manifest();
        boundary["capabilities"][0]["obligations"][0]["parameters"]["evidenceCaseIds"]=json!((0..2048).map(|i|format!("case-{i}")).collect::<Vec<_>>());
        boundary["capabilities"][0]["obligations"][1]["parameters"]["subjects"]=json!([{"kind":"selected-capability","profileRequirementId":"deployment"}]);
        boundary["capabilities"][0]["obligations"][1]["parameters"]["evidenceCaseIds"]=json!(["case-extra"]);
        let (r,_)=registry(boundary.to_string());
        assert!(ObligationCustody::collect(selected(&r),&ids(&["a"]),false).unwrap().project_scoped().is_err());
        boundary["capabilities"][0]["obligations"].as_array_mut().unwrap().pop();
        let (r,_)=registry(boundary.to_string());
        assert!(ObligationCustody::collect(selected(&r),&ids(&["a"]),false).unwrap().project_scoped().is_ok());
    }
    #[test]
    fn repeated_id_parameter_owner_and_failure_code_conflicts_refuse() {
        for field in ["parameters", "owner", "failureCode"] {
            let mut m = manifest();
            let mut c = m["capabilities"][0].clone();
            c["id"] = json!("b");
            c["obligations"][0][field] = match field {
                "parameters" => json!({"changed":true}),
                "owner" => json!("backend"),
                _ => json!("WFT-OTHER"),
            };
            m["capabilities"].as_array_mut().unwrap().push(c);
            let (r, _) = registry(m.to_string());
            assert!(ObligationCustody::collect(selected(&r), &ids(&["a", "b"]), false).is_err());
        }
    }
    #[test]
    fn incomplete_duplicate_inapplicable_and_status_selection_refuse() {
        let (r, _) = registry(manifest().to_string());
        for names in [&[][..], &["a", "a"][..], &["missing"][..]] {
            assert!(ObligationCustody::collect(selected(&r), &ids(names), false).is_err());
        }
        for (key, value, allow, expected) in [
            ("targetProfiles", json!(["two"]), true, false),
            ("status", json!("unsupported"), true, false),
            ("status", json!("candidate"), false, false),
            ("status", json!("candidate"), true, true),
        ] {
            let mut m = manifest();
            m["capabilities"][0][key] = value;
            let (r, _) = registry(m.to_string());
            assert_eq!(
                ObligationCustody::collect(selected(&r), &ids(&["a"]), allow).is_ok(),
                expected
            );
        }
    }
    #[test]
    fn host_reconstruction_requires_exact_bytes_and_target() {
        let raw = manifest().to_string();
        let (compiler, _) = registry(raw.clone());
        let custody = ObligationCustody::collect(selected(&compiler), &ids(&["a"]), false).unwrap();
        let (host, _) = registry(raw.clone());
        assert!(custody.same_registration(&selected(&host)).is_ok());
        assert!(
            custody
                .same_registration(&host.declaration_for("fixture", "v1", "two").unwrap())
                .is_err()
        );
        let (different_whitespace, _) = registry(format!(" {raw}"));
        assert!(
            custody
                .same_registration(&selected(&different_whitespace))
                .is_err()
        );
        let mut m = manifest();
        m["capabilities"][0]["obligations"][0]["parameters"]["substitution"] = json!(true);
        let (different_parameters, _) = registry(m.to_string());
        assert!(
            custody
                .same_registration(&selected(&different_parameters))
                .is_err()
        );
    }
    #[test]
    fn shared_obligation_comparisons_have_independent_budget_cost() {
        let mut m = manifest();
        let mut c = m["capabilities"][0].clone();
        c["id"] = json!("b");
        c["obligations"] = json!([obligation("shared")]);
        m["capabilities"].as_array_mut().unwrap().push(c);
        let (r, _) = registry(m.to_string());
        let names = ids(&["a", "b"]);
        let mut full = Budget {
            work: 1_000_000,
            text: 16_000_000,
        };
        ObligationCustody::collect_budget(selected(&r), &names, false, &mut full).unwrap();
        m["capabilities"][1]["obligations"] = json!([]);
        let (empty, _) = registry(m.to_string());
        let mut baseline = Budget {
            work: 1_000_000,
            text: 16_000_000,
        };
        ObligationCustody::collect_budget(selected(&empty), &names, false, &mut baseline).unwrap();
        // This fixed opaque fixture has 8 value nodes, 3 names, 2 string payloads,
        // plus ID/failure-code visits: 15 visits and 66 UTF8 bytes per traversal.
        // Shared entry adds 3 traversals and one capability-origin retention visit.
        // Raw manifest lengths differ, so exclude that separately charged payload.
        assert_eq!(baseline.work - full.work, 46);
        let raw_delta = selected(&r).manifest_json().len() - selected(&empty).manifest_json().len();
        assert_eq!(baseline.text - full.text - raw_delta, 199);
        let work = 1_000_000 - full.work;
        let text = 16_000_000 - full.text;
        assert!(
            ObligationCustody::collect_budget(
                selected(&r),
                &names,
                false,
                &mut Budget { work, text }
            )
            .is_ok()
        );
        assert!(
            ObligationCustody::collect_budget(
                selected(&r),
                &names,
                false,
                &mut Budget {
                    work: work - 30,
                    text
                }
            )
            .is_err()
        );
        assert!(
            ObligationCustody::collect_budget(
                selected(&r),
                &names,
                false,
                &mut Budget {
                    work,
                    text: text - 132
                }
            )
            .is_err()
        );
    }
    #[test]
    fn exact_inventory_budget_and_parameter_depth_are_atomic() {
        let (r, _) = registry(manifest().to_string());
        let names = ids(&["a"]);
        let mut b = Budget {
            work: 1_000_000,
            text: 16_000_000,
        };
        ObligationCustody::collect_budget(selected(&r), &names, false, &mut b).unwrap();
        let work = 1_000_000 - b.work;
        let text = 16_000_000 - b.text;
        assert!(
            ObligationCustody::collect_budget(
                selected(&r),
                &names,
                false,
                &mut Budget { work, text }
            )
            .is_ok()
        );
        for (work, text) in [(work - 1, text), (work, text - 1), (0, text)] {
            assert!(
                ObligationCustody::collect_budget(
                    selected(&r),
                    &names,
                    false,
                    &mut Budget { work, text }
                )
                .is_err()
            );
        }
        let mut value = json!(null);
        for _ in 0..66 {
            value = json!([value]);
        }
        assert!(
            Budget {
                work: 1000,
                text: 1000
            }
            .value(&value, 0)
            .is_err()
        );
    }
    fn projection_manifest() -> Value {
        let mut m = manifest();
        for o in m["capabilities"][0]["obligations"].as_array_mut().unwrap() {
            o["parameters"] = json!({"version":"weft.security.admission-obligation/0.1.0",
                "semanticSources":["scan:resource/action:read"], "enforcementSite":"host",
                "prerequisites":[], "evidenceCaseIds":["independent-native-case"]});
        }
        m["capabilities"][0]["obligations"][1]["parameters"]["prerequisites"] = json!(["shared"]);
        m
    }
    fn project_fixture(
        m: Value,
    ) -> Result<Vec<crate::security_lowering::SecurityAdmissionObligation>> {
        let (r, _) = registry(m.to_string());
        ObligationCustody::collect(selected(&r), &["a".into()], false)?.project_admission()
    }
    #[test]
    fn closed_projection_preserves_complete_declared_inventory() {
        let m = projection_manifest();
        let projected = project_fixture(m.clone()).unwrap();
        assert_eq!(projected.len(), 2);
        for source in m["capabilities"][0]["obligations"].as_array().unwrap() {
            let row = projected
                .iter()
                .find(|r| r.id == source["id"].as_str().unwrap())
                .unwrap();
            let value = serde_json::to_value(row).unwrap();
            assert_eq!(value["failureCode"], source["failureCode"]);
            for field in [
                "semanticSources",
                "enforcementSite",
                "prerequisites",
                "evidenceCaseIds",
            ] {
                assert_eq!(value[field], source["parameters"][field]);
            }
        }
        // Opaque originals remain preservable, but cannot be projected with guessed semantics.
        assert!(project_fixture(manifest()).is_err());
    }
    #[test]
    fn unknown_projection_meaning_owner_and_carriers_refuse() {
        let base = projection_manifest();
        let controls = [
            ("version", json!("future")),
            (
                "version",
                json!({"weft.security.admission-obligation/0.1.0":null}),
            ),
            ("enforcementSite", json!({"host":null})),
            ("enforcementSite", json!("native")),
            ("semanticSources", json!([])),
            ("semanticSources", json!(["same", "same"])),
            ("semanticSources", json!([null])),
            ("evidenceCaseIds", json!([])),
            ("prerequisites", json!(["shared", "shared"])),
        ];
        for (field, value) in controls {
            let mut m = base.clone();
            m["capabilities"][0]["obligations"][0]["parameters"][field] = value;
            assert!(project_fixture(m).is_err(), "{field}");
        }
        let mut extra = base.clone();
        extra["capabilities"][0]["obligations"][0]["parameters"]["future"] = json!(true);
        assert!(project_fixture(extra).is_err());
        let mut missing = base.clone();
        missing["capabilities"][0]["obligations"][0]["parameters"]
            .as_object_mut()
            .unwrap()
            .remove("evidenceCaseIds");
        assert!(project_fixture(missing).is_err());
        for site in ["backend", "native"] {
            let mut m = base.clone();
            m["capabilities"][0]["obligations"][0]["owner"] = json!("backend");
            m["capabilities"][0]["obligations"][0]["parameters"]["enforcementSite"] = json!(site);
            assert!(project_fixture(m).is_ok());
        }
    }
    #[test]
    fn prerequisite_closure_and_disconnected_cycles_refuse_atomically() {
        let base = projection_manifest();
        for (a, b) in [
            (vec!["missing"], vec!["shared"]),
            (vec!["shared"], vec![]),
            (vec!["a-only"], vec!["shared"]),
        ] {
            let mut m = base.clone();
            m["capabilities"][0]["obligations"][0]["parameters"]["prerequisites"] = json!(a);
            m["capabilities"][0]["obligations"][1]["parameters"]["prerequisites"] = json!(b);
            assert!(project_fixture(m).is_err());
        }
        let mut disconnected = base.clone();
        let mut cycle = disconnected["capabilities"][0]["obligations"][0].clone();
        cycle["id"] = json!("disconnected");
        cycle["parameters"]["prerequisites"] = json!(["disconnected"]);
        disconnected["capabilities"][0]["obligations"]
            .as_array_mut()
            .unwrap()
            .push(cycle);
        assert!(project_fixture(disconnected).is_err());
        // The admitted shared-prerequisite counterpart does not form a cycle.
        assert!(project_fixture(base).is_ok());
    }

    #[test]
    fn prerequisites_resolve_only_in_complete_selected_capability_inventory() {
        let mut m = projection_manifest();
        let mut second = m["capabilities"][0].clone();
        second["id"] = json!("b");
        let mut prerequisite = second["obligations"][0].clone();
        prerequisite["id"] = json!("b-only");
        second["obligations"] = json!([prerequisite]);
        m["capabilities"][0]["obligations"][0]["parameters"]["prerequisites"] = json!(["b-only"]);
        m["capabilities"].as_array_mut().unwrap().push(second);
        let (r, _) = registry(m.to_string());
        let both =
            ObligationCustody::collect(selected(&r), &["a".into(), "b".into()], false).unwrap();
        assert_eq!(both.project_admission().unwrap().len(), 3);
        let first = ObligationCustody::collect(selected(&r), &["a".into()], false).unwrap();
        assert!(first.project_admission().is_err());
        assert_eq!(both.capability_sources("b-only").unwrap(), ["b"]);
    }
    #[test]
    fn diamond_dag_shared_dependencies_and_isolated_cycle_are_distinct() {
        let mut m = projection_manifest();
        let template = m["capabilities"][0]["obligations"][0].clone();
        let mut obligations = Vec::new();
        for (id, dependencies) in [
            ("root", vec![]),
            ("left", vec!["root"]),
            ("right", vec!["root"]),
            ("top", vec!["left", "right"]),
        ] {
            let mut o = template.clone();
            o["id"] = json!(id);
            o["parameters"]["prerequisites"] = json!(dependencies);
            obligations.push(o);
        }
        m["capabilities"][0]["obligations"] = json!(obligations);
        assert_eq!(project_fixture(m.clone()).unwrap().len(), 4);
        for (id, dependency) in [("isolated-a", "isolated-b"), ("isolated-b", "isolated-a")] {
            let mut o = template.clone();
            o["id"] = json!(id);
            o["parameters"]["prerequisites"] = json!([dependency]);
            m["capabilities"][0]["obligations"]
                .as_array_mut()
                .unwrap()
                .push(o);
        }
        assert!(project_fixture(m).is_err());
    }
    #[test]
    fn projection_work_and_utf8_limits_refuse_before_returning_inventory() {
        let (r, _) = registry(projection_manifest().to_string());
        let custody = ObligationCustody::collect(selected(&r), &["a".into()], false).unwrap();
        let mut measured = Budget {
            work: 1_000_000,
            text: 16_000_000,
        };
        custody.project_budget(&mut measured).unwrap();
        let work = 1_000_000 - measured.work;
        let text = 16_000_000 - measured.text;
        assert!(custody.project_budget(&mut Budget { work, text }).is_ok());
        assert!(
            custody
                .project_budget(&mut Budget {
                    work: work - 1,
                    text
                })
                .is_err()
        );
        assert!(
            custody
                .project_budget(&mut Budget {
                    work,
                    text: text - 1
                })
                .is_err()
        );
        let mut long = projection_manifest();
        long["capabilities"][0]["obligations"][0]["parameters"]["semanticSources"] =
            json!(["é".repeat(2049)]);
        assert!(project_fixture(long).is_err());
        for (length, expected) in [(2048, true), (2049, false)] {
            let mut m = projection_manifest();
            m["capabilities"][0]["obligations"][0]["id"] = json!("é".repeat(length));
            m["capabilities"][0]["obligations"][1]["parameters"]["prerequisites"] = json!([]);
            assert_eq!(
                project_fixture(m).is_ok(),
                expected,
                "original ID UTF8 bytes"
            );
        }
    }
}
#[cfg(test)]
mod faithful_parameter_tests {
    use super::*;
    use crate::security_backend::validate_security_manifest_json;
    use serde_json::json;
    #[test]
    fn malformed_manifest_roots_refuse_without_panicking() {
        for raw in ["[]", "true", "1", "\"x\"", "null"] {
            assert!(crate::backend::validate_manifest_json(raw).is_err());
            assert!(validate_security_manifest_json(raw).is_err());
        }
    }
    #[test]
    fn typed_status_and_owner_do_not_accept_object_enum_carriers() {
        for owner in [false, true] {
            let mut security = super::tests::manifest();
            if owner {
                security["capabilities"][0]["obligations"][0]["owner"] = json!({"host":null});
            } else {
                security["capabilities"][0]["status"] = json!({"supported":null});
            }
            assert!(validate_security_manifest_json(&security.to_string()).is_err());
            security.as_object_mut().unwrap().remove("sourceProfiles");
            security["interfaceVersion"] = json!("weft-backend/0.2.0");
            security["languageProfiles"] =
                json!([{"dialectProfile":"weft-sql/0.2.0","irVersion":"weft-ir/0.2.0"}]);
            assert!(crate::backend::validate_manifest_json(&security.to_string()).is_err());
        }
    }
    #[test]
    fn raw_manifest_reserved_looking_keys_are_ordinary_opaque_members() {
        for key in [
            "$serde_json::private::Number",
            "$serde_json::private::RawValue",
        ] {
            for payload in [
                json!("1"),
                json!("not JSON"),
                json!(false),
                Value::Null,
                json!([1, true]),
            ] {
                for first in [true, false] {
                    for nested in [true, false] {
                        let member = format!("{}:{}", serde_json::to_string(key).unwrap(), payload);
                        let object = if first {
                            format!("{{{member},\"sibling\":true}}")
                        } else {
                            format!("{{\"sibling\":true,{member}}}")
                        };
                        let fragment = if nested {
                            format!("[{{\"nested\":{object}}}]")
                        } else {
                            object
                        };
                        let mut expected_map = serde_json::Map::new();
                        expected_map.insert(key.into(), payload.clone());
                        expected_map.insert("sibling".into(), json!(true));
                        let object = Value::Object(expected_map);
                        let expected = if nested {
                            json!([{"nested":object}])
                        } else {
                            object
                        };
                        let mut base = super::tests::manifest();
                        base["targetProfiles"][0]["sessionSettings"] = json!({"marker":"OPAQUE"});
                        base["capabilities"][0]["logicalDomain"] = json!({"marker":"OPAQUE"});
                        base["capabilities"][0]["resultDomain"] = json!({"marker":"OPAQUE"});
                        base["capabilities"][0]["obligations"][0]["parameters"] =
                            json!({"marker":"OPAQUE"});
                        let raw = base.to_string().replace("\"OPAQUE\"", &fragment);
                        let mut ordinary = base.clone();
                        ordinary.as_object_mut().unwrap().remove("sourceProfiles");
                        ordinary["interfaceVersion"] = json!("weft-backend/0.2.0");
                        ordinary["languageProfiles"] = json!([{"dialectProfile":"weft-sql/0.2.0","irVersion":"weft-ir/0.2.0"}]);
                        let ordinary_raw = ordinary.to_string().replace("\"OPAQUE\"", &fragment);
                        let ordinary_parsed =
                            crate::backend::validate_manifest_json(&ordinary_raw).unwrap();
                        for value in [
                            &ordinary_parsed.target_profiles[0].session_settings,
                            &ordinary_parsed.capabilities[0].logical_domain,
                            &ordinary_parsed.capabilities[0].result_domain,
                            &ordinary_parsed.capabilities[0].obligations[0].parameters,
                        ] {
                            assert_eq!(
                                value["marker"], expected,
                                "ordinary/{key}/{first}/{nested}"
                            );
                        }
                        let parsed = validate_security_manifest_json(&raw).unwrap();
                        for value in [
                            &parsed.target_profiles[0].session_settings,
                            &parsed.capabilities[0].logical_domain,
                            &parsed.capabilities[0].result_domain,
                            &parsed.capabilities[0].obligations[0].parameters,
                        ] {
                            assert_eq!(value["marker"], expected, "{key}/{first}/{nested}");
                        }
                    }
                }
            }
        }
    }
}
