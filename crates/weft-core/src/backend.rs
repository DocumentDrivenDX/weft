// SPDX-License-Identifier: MIT OR Apache-2.0
//! Trusted, explicitly registered backend declarations. No content-driven loading.
use crate::{error::Result, json::checked_json};
pub use weft_backend::manifest::{
    Capability, LanguageProfile, Manifest, Obligation, ObligationOwner, Status, TargetProfile,
};
pub(crate) use weft_backend::manifest::{restore_manifest_opaque, take_manifest_opaque};
use weft_backend::{
    boundary::Frontend,
    manifest::{failure, ManifestRules},
    sql::SqlChecks,
};
#[cfg(test)]
use weft_backend::boundary::merge_obligations;
pub fn validate_manifest_json(raw: &str) -> Result<Manifest> {
    weft_backend::manifest::validate_manifest_json::<WeftPlans>(raw)
}
pub fn validate_manifest(m: &Manifest) -> Result<()> {
    weft_backend::manifest::validate_manifest::<WeftPlans>(m)
}
/// Backend-owned mapping and target types remain statically checked inside this trait.
/// Only an explicitly registered host object implements it; manifests never load code.
pub trait Backend: Send + Sync + 'static {
    type Mapping: Send + Sync;
    type TargetPlan: Send + Sync;
    fn describe(&self) -> Result<Manifest>;
    fn validate_binding(&self, context: &Context<'_>) -> Result<Validated<Self::Mapping>>;
    fn assess(&self, context: &Context<'_>, mapping: &Self::Mapping) -> Result<Vec<Assessment>>;
    fn lower(&self, context: &Context<'_>, mapping: &Self::Mapping) -> Result<Self::TargetPlan>;
    fn emit(&self, context: &Context<'_>, plan: &Self::TargetPlan) -> Result<Emission>;
}
#[derive(Debug, Clone, Copy)]
pub enum Plan<'a> {
    V01(&'a crate::ir::LogicalPlan),
    V02(&'a crate::application_ir::Plan),
    V03(&'a crate::arithmetic_plan::Plan),
    #[cfg(feature="engine-ordered-page-candidate")]
    Ordered(&'a crate::application_ordered_page::OrderedPage),
}
impl Plan<'_> {
    pub fn language(&self) -> LanguageProfile {
        #[cfg(feature="engine-ordered-page-candidate")]
        if let Self::Ordered(p)=self {return LanguageProfile{dialect_profile:"weft-sql/0.2.0".into(),ir_version:p.version().into()};}
        if let Self::V02(p) = self {
            return LanguageProfile { dialect_profile: "weft-sql/0.2.0".into(), ir_version: p.ir_version.clone() };
        }
        let suffix = match self {
            Self::V01(_) => "0.1.0",
            Self::V02(_) => "0.2.0",
            Self::V03(_) => "0.3.0",
            #[cfg(feature="engine-ordered-page-candidate")]
            Self::Ordered(_)=>unreachable!(),
        };
        LanguageProfile {
            dialect_profile: format!("weft-sql/{suffix}"),
            ir_version: format!("weft-ir/{suffix}"),
        }
    }
    pub fn capabilities(&self) -> &[String] {
        match self {
            Self::V01(p) => &p.required_capabilities,
            Self::V02(p) => &p.required_capabilities,
            Self::V03(p) => &p.required_capabilities,
            #[cfg(feature="engine-ordered-page-candidate")]
            Self::Ordered(p)=>&p.base().required_capabilities,
        }
    }
    pub fn pins(&self) -> &[crate::ir::ModelPin] {
        match self {
            Self::V01(p) => &p.module_pins,
            Self::V02(p) => &p.module_pins,
            Self::V03(p) => &p.module_pins,
            #[cfg(feature="engine-ordered-page-candidate")]
            Self::Ordered(p)=>&p.base().module_pins,
        }
    }
}
pub use weft_backend::boundary::{Assessment, BindingInput, Selection, Target, Validated};
/// The binding context a weft backend receives.
pub type Context<'a> = weft_backend::boundary::Context<'a, WeftPlans>;
pub use weft_backend::sql::{
    Column, Emission, ExactParameterKind, OuterJoin, ParameterSlot, ParameterType, Representation, ScalarCarrier,
    ScalarDecoder,
};
pub use weft_backend::boundary::Qualification;
/// A weft compilation: a [`weft_backend::boundary::Compilation`] of SQL [`Emission`].
pub type Compilation = weft_backend::boundary::Compilation<Emission>;

/// weft's frontend for the shared backend pipeline: weft-sql 0.1-0.3 plans and
/// engine pages, weft-backend/0.2.x manifests.
pub struct WeftPlans;
impl ManifestRules for WeftPlans {
    fn admits_interface(interface: &str) -> bool {
        matches!(interface, "weft-backend/0.2.0" | "weft-backend/0.2.1" | "weft-backend/0.2.2" | "weft-backend/0.2.3" | "weft-backend/0.2.4" | "weft-backend/0.2.5" | "weft-backend/0.2.6" | "weft-backend/0.2.7" | "weft-backend/0.2.8" | "weft-backend/0.2.9")
    }
    fn admits_language(interface: &str, p: &LanguageProfile) -> bool {
        fn profile_key(p: &LanguageProfile) -> (&str, &str) {
            (&p.dialect_profile, &p.ir_version)
        }
            matches!(
            profile_key(p),
            ("weft-sql/0.1.0", "weft-ir/0.1.0") | ("weft-sql/0.2.0", "weft-ir/0.2.0") | ("weft-sql/0.3.0", "weft-ir/0.3.0") | ("weft-sql/0.2.0", "weft-ir/engine-navigation/0.1")
        ) || (interface == "weft-backend/0.2.8" && matches!(profile_key(p),("weft-sql/0.2.0","weft-ir/engine-ordered-page/0.1")|("weft-sql/0.2.0","weft-ir/engine-navigation/0.2")|("weft-sql/0.2.0","weft-ir/engine-ordered-read/0.1"))) || (interface == "weft-backend/0.2.1" && profile_key(p) == ("weft-sql/0.2.0", "weft-ir/0.2.1"))
            || (interface == "weft-backend/0.2.9" && profile_key(p) == ("weft-sql/0.2.0", "weft-ir/0.2.8"))
            || (interface == "weft-backend/0.2.7" && profile_key(p) == ("weft-sql/0.2.0", "weft-ir/0.2.7"))
            || (interface == "weft-backend/0.2.6" && profile_key(p) == ("weft-sql/0.2.0", "weft-ir/0.2.6"))
            || (interface == "weft-backend/0.2.5" && profile_key(p) == ("weft-sql/0.2.0", "weft-ir/0.2.5"))
            || (interface == "weft-backend/0.2.4" && profile_key(p) == ("weft-sql/0.2.0", "weft-ir/0.2.4"))
            || (interface == "weft-backend/0.2.3" && profile_key(p) == ("weft-sql/0.2.0", "weft-ir/0.2.3"))
            || (interface == "weft-backend/0.2.2" && profile_key(p) == ("weft-sql/0.2.0", "weft-ir/0.2.2"))
    }
}
impl Frontend for WeftPlans {
    type Program<'a> = Plan<'a>;
    fn reborrow<'s, 'l: 's>(plan: Plan<'l>) -> Plan<'s> {
        plan
    }
    fn language(plan: Plan<'_>) -> LanguageProfile {
        plan.language()
    }
    fn ir_version(plan: Plan<'_>) -> String {
        match plan {
            Plan::V01(p) => p.ir_version.clone(),
            Plan::V02(p) => p.ir_version.clone(),
            Plan::V03(p) => p.ir_version.clone(),
            #[cfg(feature="engine-ordered-page-candidate")]
            Plan::Ordered(p)=>p.version().into(),
        }
    }
    fn capabilities(plan: Plan<'_>) -> Vec<String> {
        plan.capabilities().to_vec()
    }
    fn pins(plan: Plan<'_>) -> Vec<crate::ir::ModelPin> {
        plan.pins().to_vec()
    }
    fn selection(plan: Plan<'_>) -> Selection {
        selected(plan)
    }
    fn read_binding(target: &Target, actual_ir: &str, json: &str) -> std::result::Result<serde_json::Value, ()> {
        (if target.backend_id == "truss.postgresql.runtime"
            && ((target.backend_version == "0.4.41" && target.profile_id == "truss-runtime-pg18.6/0.5" && actual_ir == "weft-ir/0.2.7")
             || (matches!(target.backend_version.as_str(),"0.4.43"|"0.4.44") && matches!(target.profile_id.as_str(),"truss-runtime-pg18.6/0.6"|"truss-runtime-pg18.6-export/0.2") && actual_ir == "weft-ir/0.2.8"))
        {
            crate::json::checked_truss_runtime_catalog_json(json)
        } else {
            checked_json(json)
        }).map_err(|_| ())
    }
    fn preflight(manifest: &Manifest, plan: Plan<'_>, target: &Target, language: &LanguageProfile) -> Result<()> {
        let language = language.clone();
        // Unbounded source integers are a new explicit opt-in. Refuse before
        // unchanged backend binding code can assume an authored width.
        if plan.capabilities().iter().any(|c| c == "type.integer.unbounded")
            && !manifest.capabilities.iter().any(|c| c.id == "type.integer.unbounded"
                && (c.status == Status::Supported || (c.status == Status::Candidate && target.allow_candidate))
                && c.target_profiles.contains(&target.profile_id) && c.language_profiles.contains(&language)) {
            return Err(failure("WFT-CAPABILITY", "Selected backend profile does not admit mathematical integer source domains"));
        }
        if plan.capabilities().iter().any(|c| c == "type.decimal.unbounded")
            && !manifest.capabilities.iter().any(|c| c.id == "type.decimal.unbounded"
                && (c.status == Status::Supported || (c.status == Status::Candidate && target.allow_candidate))
                && c.target_profiles.contains(&target.profile_id) && c.language_profiles.contains(&language)) {
            return Err(failure("WFT-CAPABILITY", "Selected backend profile does not admit mathematical decimal source domains"));
        }
        // New 0.3 comparisons must be admitted before any backend binding callback.
        // Retain the ordering of older operations; do not retroactively preflight them.
        if matches!(plan, Plan::V03(_)) {
            let distinct_count=plan.capabilities().iter().any(|id|id=="aggregate.countDistinct");
            for id in plan.capabilities().iter().filter(|id| ["join.left", "value.outerJoinPresence", "compare.less", "compare.lessEqual", "compare.greaterEqual", "compare.notEqual", "compare.scalarJoin", "project.positionedOutputs", "project.distinct", "predicate.nativeNull", "compare.nullAwareStringEqual", "value.nativeNull", "predicate.stringIn", "aggregate.countDistinct", "aggregate.countDistinct.optional", "aggregate.havingCountDistinctGreater"].contains(&id.as_str()) || (distinct_count && ["aggregate","group"].contains(&id.as_str()))) {
                if !manifest.capabilities.iter().any(|c| &c.id == id
                    && (c.status == Status::Supported || (c.status == Status::Candidate && target.allow_candidate))
                    && c.target_profiles.contains(&target.profile_id) && c.language_profiles.contains(&language)) {
                    return Err(failure("WFT-CAPABILITY", "Selected backend profile does not admit the new comparison operator"));
                }
            }
        }
        if plan.capabilities().iter().any(|id| id == "compare.binary.optionalEqual")
            && !manifest.capabilities.iter().any(|c| c.id == "compare.binary.optionalEqual"
                && (c.status == Status::Supported || (c.status == Status::Candidate && target.allow_candidate))
                && c.target_profiles.contains(&target.profile_id) && c.language_profiles.contains(&language)) {
            return Err(failure("WFT-CAPABILITY", "Selected backend profile does not admit optional binary equality"));
        }
        for id in plan.capabilities().iter().filter(|id| matches!(id.as_str(),"compare.timestamp.instantEqual"|"compare.timestamp.optionalInstantEqual")) {
            if !manifest.capabilities.iter().any(|c|&c.id==id
                && (c.status==Status::Supported || (c.status==Status::Candidate&&target.allow_candidate))
                && c.target_profiles.contains(&target.profile_id)&&c.language_profiles.contains(&language)) {
                return Err(failure("WFT-CAPABILITY","Selected backend does not admit exact instant equality"));
            }
        }
        if plan.capabilities().iter().any(|id| id == "type.decimal.scale2")
            && !manifest.capabilities.iter().any(|c| c.id == "type.decimal.scale2"
                && (c.status == Status::Supported || (c.status == Status::Candidate && target.allow_candidate))
                && c.target_profiles.contains(&target.profile_id) && c.language_profiles.contains(&language)) {
            return Err(failure("WFT-CAPABILITY", "Selected backend profile does not admit declared decimal18,2"));
        }
        Ok(())
    }
    fn admit_catalog(catalog: &crate::model::Catalog) -> Result<()> {
        if catalog.admission().bounded_current_strings {
            return Err(failure("WFT-VERSION", "Bounded-string catalogs require the dedicated current-query candidate compiler"));
        }
        Ok(())
    }
}
impl SqlChecks for WeftPlans {
    fn check_emission(plan: Plan<'_>, emission: &Emission, selection: &Selection, assessments: &[Assessment]) -> Result<()> {
        crate::backend_emission::validate(plan, emission, selection, assessments)
    }
}
/// Bridges a weft [`Backend`] to the shared pipeline.
struct Weft<B>(B);
impl<B: Backend> weft_backend::boundary::Backend<WeftPlans> for Weft<B> {
    type Mapping = B::Mapping;
    type TargetPlan = B::TargetPlan;
    type Artifact = Emission;
    fn describe(&self) -> Result<Manifest> { self.0.describe() }
    fn validate_binding(&self, context: &Context<'_>) -> Result<Validated<B::Mapping>> { self.0.validate_binding(context) }
    fn assess(&self, context: &Context<'_>, mapping: &B::Mapping) -> Result<Vec<Assessment>> { self.0.assess(context, mapping) }
    fn lower(&self, context: &Context<'_>, mapping: &B::Mapping) -> Result<B::TargetPlan> { self.0.lower(context, mapping) }
    fn emit(&self, context: &Context<'_>, plan: &B::TargetPlan) -> Result<Emission> { self.0.emit(context, plan) }
}
/// Registered weft backends by ID; the checked pipeline is `weft_backend`'s.
#[derive(Default)]
pub struct Registry(weft_backend::boundary::Registry<WeftPlans, Emission>);
impl Registry {
    pub fn register<B: Backend>(&mut self, backend: B) -> Result<()> {
        self.0.register(Weft(backend))
    }
    pub fn manifest(&self, id: &str) -> Option<&Manifest> {
        self.0.manifest(id)
    }
    pub fn compile(
        &self,
        catalog: &crate::model::Catalog,
        plan: Plan<'_>,
        target: &Target,
        binding: &BindingInput,
    ) -> Result<Compilation> {
        self.0.compile(catalog, plan, target, binding)
    }
}
/// Exact original application selection, including admitted source navigation.
pub fn selection(plan: Plan<'_>)->Selection{selected(plan)}
fn selected(plan: Plan<'_>) -> Selection {
    use crate::{
        application_ir::{Expression as AE, Predicate as AP, Value as AV},
        ir::{Expression as E, Node},
    };
    fn expression(e: &E, s: &mut Selection) {
        match e {
            E::Field { identity, .. } => s.fields.push(identity.clone()),
            E::Literal { .. } => {}
            E::Equal { left, right, .. } | E::And { left, right, .. } => {
                expression(left, s);
                expression(right, s);
            }
            E::Sum { argument, .. } => expression(argument, s),
        }
    }
    fn node(n: &Node, s: &mut Selection) {
        match n {
            Node::Scan { record, .. } => s.records.push(record.clone()),
            Node::InnerJoin { left, right, on } => {
                node(left, s);
                node(right, s);
                expression(on, s);
            }
            Node::Filter { input, predicate } => {
                node(input, s);
                expression(predicate, s);
            }
            Node::Aggregate {
                input,
                groups,
                aggregates,
            } => {
                node(input, s);
                for e in groups.iter().chain(aggregates) {
                    expression(e, s);
                }
            }
            Node::Project { input, outputs } => {
                node(input, s);
                for o in outputs {
                    expression(&o.expression, s);
                }
            }
        }
    }
    fn rel(r: &crate::application_model::RelationshipRead, s: &mut Selection) {
        s.relationships.push(r.identity.clone());
        s.records.extend([r.from.clone(), r.to.clone()]);
        s.fields.extend(r.source_key.fields.clone());
        s.fields.extend(r.target_key.fields.clone());
    }
    fn value(v: &AV, s: &mut Selection) {
        if let AV::Field { field } = v {
            s.fields.push(field.identity.clone());
        }
    }
    fn predicate(p: &AP, s: &mut Selection) {
        match p {
            AP::Boolean{..}=>{},
            AP::ExactEqual { left, right } => {
                s.fields.push(left.identity.clone());
                if let crate::application_ir::ExactValue::Field { field } = right {
                    s.fields.push(field.identity.clone());
                }
            }
            AP::Equal { left, right } => {
                s.fields.push(left.identity.clone());
                value(right, s);
            }
            AP::LexicographicGreater { columns, values } => {
                s.fields.extend(columns.iter().map(|f| f.identity.clone()));
                for v in values {
                    value(v, s);
                }
            }
            AP::HasRelated {
                relationship, key, ..
            } => {
                rel(relationship, s);
                for v in key {
                    value(v, s);
                }
            }
        }
    }
    let mut s = Selection::default();
    match plan {
        #[cfg(feature="engine-ordered-page-candidate")]
        Plan::Ordered(p)=>{let mut base=selected(Plan::V02(p.base()));base.fields.extend(p.sort_slots().iter().map(|s|s.identity().clone()));return base;},
        Plan::V01(p) => node(&p.root, &mut s),
        Plan::V03(p) => {
            fn arithmetic(e:&crate::arithmetic_resolve::Expression,s:&mut Selection) {
                use crate::arithmetic_resolve::Kind;
                match &e.kind {
                    Kind::Field{field}=>s.fields.push(field.identity.clone()),
                    Kind::Negate{operand}=>arithmetic(operand,s),
                    Kind::Binary{left,right,..}=>{arithmetic(left,s);arithmetic(right,s)},
                    Kind::Literal{..}|Kind::Parameter{..}=>{}
                }
            }
            fn pred(p:&crate::arithmetic_plan::Predicate,s:&mut Selection) {
                match p {
                    crate::arithmetic_plan::Predicate::Legacy{predicate:p}=>predicate(p,s),
                    crate::arithmetic_plan::Predicate::ArithmeticCompare{left,right,..}|crate::arithmetic_plan::Predicate::ArithmeticCompareExtended{left,right,..}=>{arithmetic(left,s);arithmetic(right,s)},
                    crate::arithmetic_plan::Predicate::StringIn{field,values}=>{s.fields.push(field.identity.clone());for v in values{value(v,s)}},
                    crate::arithmetic_plan::Predicate::NullTest{field,..}=>s.fields.push(field.identity.clone()),
                    crate::arithmetic_plan::Predicate::NullableStringEqual{left,right}=>{s.fields.push(left.identity.clone());s.fields.push(right.identity.clone())},
                    crate::arithmetic_plan::Predicate::ScalarCompare{left,right,..}=>{s.fields.push(left.identity.clone());value(right,s)}
                }
            }
            s.records.push(p.source.record.clone());
            for j in &p.joins {s.records.push(j.right.record.clone());for p in &j.on {pred(p,&mut s)}}
            for p in &p.filters {pred(p,&mut s)}
            s.fields.extend(p.groups.iter().chain(&p.order).map(|f|f.identity.clone()));
            if let Some(key) = &p.page_key {
                s.fields.extend(key.fields.clone());
            }
            for output in &p.outputs {
                use crate::arithmetic_plan::Expression;
                match &output.expression {
                    Expression::Field{identity,..}=>s.fields.push(identity.clone()),
                    Expression::Sum{argument,..}|Expression::CountDistinct{argument,..}=>s.fields.push(argument.identity.clone()),
                    Expression::RelatedKeys{relationship,..}=>rel(relationship,&mut s),
                    Expression::Arithmetic{expression}=>arithmetic(expression,&mut s),
                    Expression::Count{..}=>{}
                }
            }
            s.types.extend(p.type_graph.iter().map(|d|d.identity.clone()));
        }
        Plan::V02(p) => {
            if let Some(navigation)=&p.navigation{
                let source=selected(Plan::V02(navigation.source()));s.records.extend(source.records);s.fields.extend(source.fields);s.types.extend(source.types);s.relationships.extend(source.relationships);rel(navigation.relationship(),&mut s);
            }
            s.records.push(p.source.record.clone());
            for j in &p.joins {
                s.records.push(j.right.record.clone());
                for pred in &j.on {
                    predicate(pred, &mut s);
                }
            }
            for pred in &p.filters {
                predicate(pred, &mut s);
            }
            s.fields
                .extend(p.groups.iter().chain(&p.order).map(|f| f.identity.clone()));
            if let Some(key) = &p.page_key {
                s.fields.extend(key.fields.clone());
            }
            for output in &p.outputs {
                match &output.expression {
                    AE::Field { identity, .. } => s.fields.push(identity.clone()),
                    AE::CountDistinct{argument,..}|AE::Sum { argument, .. } => s.fields.push(argument.identity.clone()),
                    AE::CompleteRelatedKeys { relationship, .. } | AE::RelatedKeys { relationship, .. } => rel(relationship, &mut s),
                    AE::Count { .. } => {},
                    AE::NativeMetadata { record, .. } => s.types.push(record.clone()),
                }
            }
            s.types
                .extend(p.type_graph.iter().map(|d| d.identity.clone()));
        }
    }
    fn dedup(v: &mut Vec<crate::ir::Identity>) {
        v.sort_by(|a, b| {
            (&a.document_id, &a.revision, &a.module, &a.element).cmp(&(
                &b.document_id,
                &b.revision,
                &b.module,
                &b.element,
            ))
        });
        v.dedup();
    }
    dedup(&mut s.records);
    dedup(&mut s.fields);
    dedup(&mut s.types);
    s.relationships.sort_by(|a, b| {
        (&a.document_id, &a.revision, &a.module, &a.relationship).cmp(&(
            &b.document_id,
            &b.revision,
            &b.module,
            &b.relationship,
        ))
    });
    s.relationships.dedup();
    s
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    #[test]
    fn arithmetic_selection_retains_original_operands_and_versions() {
        let cases: Value = serde_json::from_str(include_str!("../../../tests/application/fixtures/cases.json")).unwrap();
        let req = &cases.as_array().unwrap().iter().find(|c| c["id"] == "join-count").unwrap()["request"];
        let catalog = crate::model::Catalog::prepare(serde_json::from_value(req["modules"].clone()).unwrap()).unwrap();
        let plan = crate::arithmetic_application_resolve::resolve(&catalog, crate::arithmetic_query::parse("SELECT c.id+1 AS next,o.total*12.5000 AS scaled FROM Customer c JOIN Orders o ON o.customer_id=c.id WHERE o.total*2>1").unwrap(), Default::default(), None).unwrap();
        let selected = selected(Plan::V03(&plan));
        assert_eq!(Plan::V03(&plan).language().ir_version, "weft-ir/0.3.0");
        assert_eq!(selected.records.len(), 2);
        assert_eq!(selected.fields.len(), 3);
        let mut with_key = plan.clone();
        let mut key_field = with_key.source.record.clone();
        key_field.element = "page-key-not-in-query".into();
        with_key.page_key = Some(crate::application_model::AuthoredKey {
            id:"original-page-key".into(), fields:vec![key_field.clone()],
            types:vec![crate::ir::LogicalType { family:crate::ir::Family::Integer, facets:json!({}), nullable:false }],
        });
        let keyed_selection = self::selected(Plan::V03(&with_key));
        assert_eq!(keyed_selection.fields.len(), 4);
        assert!(keyed_selection.fields.contains(&key_field));

        for output in &plan.outputs {
            let crate::arithmetic_plan::Expression::Arithmetic { expression } = &output.expression else { panic!("expected arithmetic"); };
            let crate::arithmetic_resolve::Kind::Binary { left, .. } = &expression.kind else { panic!("expected binary"); };
            let crate::arithmetic_resolve::Kind::Field { field } = &left.kind else { panic!("expected field"); };
            assert!(selected.fields.contains(&field.identity));
        }
    }
    #[test]
    fn obligations_are_retained_sorted_and_conflicts_refuse() {
        let obligation = |id: &str, parameters: Value| Obligation {
            id: id.into(),
            parameters,
            owner: ObligationOwner::Host,
            failure_code: "WFT-OBLIGATION".into(),
        };
        let mut merged = vec![];
        let a = obligation("a", json!({"snapshot":"pinned"}));
        let b = obligation("b", json!({"transport":"exact-text"}));
        merge_obligations(&mut merged, &[b.clone(), a.clone(), a.clone()]).unwrap();
        assert_eq!(merged, vec![a.clone(), b]);
        assert!(merge_obligations(
            &mut merged,
            &[obligation("a", json!({"snapshot":"different"}))]
        )
        .is_err());
        let mut malformed = a;
        malformed.failure_code = "WFT-".into();
        assert!(merge_obligations(&mut vec![], &[malformed]).is_err());
    }
}

#[cfg(test)]
mod lineage_candidate_refusal_tests{
 use super::*;
 use serde_json::Value;
 struct Closed;
 impl Backend for Closed{
  type Mapping=();type TargetPlan=();
  fn describe(&self)->Result<Manifest>{serde_json::from_value(serde_json::json!({"backendId":"test.closed","backendVersion":"1","interfaceVersion":"weft-backend/0.2.0","languageProfiles":[{"dialectProfile":"weft-sql/0.2.0","irVersion":"weft-ir/0.2.0"}],"bindingProfile":"test.closed/1","targetProfiles":[{"id":"fixture","engine":"synthetic","engineVersion":"1","sessionSettings":{},"storageLayoutRevision":"1","publicationRevision":"1"}],"capabilities":[{"id":"scan","targetProfiles":["fixture"],"languageProfiles":[{"dialectProfile":"weft-sql/0.2.0","irVersion":"weft-ir/0.2.0"}],"logicalDomain":{"fixture":true},"resultDomain":{"fixture":true},"constraints":[],"obligations":[],"status":"candidate","evidence":[]}],"evidence":[]})).map_err(|_|failure("WFT-TEST","Fixture declaration refused"))}
  fn validate_binding(&self,_:&Context<'_>)->Result<Validated<()>>{panic!("new lineage IR must refuse before binding callback")}
  fn assess(&self,_:&Context<'_>,_:&())->Result<Vec<Assessment>>{panic!("new lineage IR must refuse before assess")}
  fn lower(&self,_:&Context<'_>,_:&())->Result<()>{panic!("new lineage IR must refuse before lower")}
  fn emit(&self,_:&Context<'_>,_:&())->Result<Emission>{panic!("new lineage IR must refuse before emit")}
 }
 #[test]
 fn old_registration_refuses_new_ir_before_any_backend_callback(){
  let r:Value=serde_json::from_str(include_str!("../../../tests/application-lineage/fixtures/token-equality.request.json")).unwrap();let mut registry=Registry::default();registry.register(Closed).unwrap();
  for sql in ["SELECT COUNT(DISTINCT r.token) FROM Resource r","SELECT COUNT(*) FROM Resource r WHERE FALSE","SELECT COUNT(*) FROM Resource r LIMIT 0","SELECT r.token FROM Resource r"]{let mut input=r.clone();input["interfaceVersion"]=serde_json::json!("weft-application-lineage/0.2-candidate");input["sql"]=serde_json::json!(sql);let prepared=crate::application_lineage::prepare(&input.to_string()).unwrap();assert_eq!(prepared.plan().ir_version,"weft-ir/application-lineage/0.2-candidate");let error=registry.compile(prepared.catalog(),Plan::V02(prepared.plan()),&Target{backend_id:"test.closed".into(),backend_version:"1".into(),profile_id:"fixture".into(),allow_candidate:true},&BindingInput{profile:"never-reached".into(),json:"{}".into(),sha256:"0".repeat(64)}).err().unwrap();assert_eq!(error.code,"WFT-BACKEND-VERSION");}
 }
}
