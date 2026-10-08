mod application;
mod relationships;
// Candidate registered Truss adapter. Qualification remains explicit and host-owned.
use crate::{
    access::ObjectProperty,
    binding::{Admission, PropertyHome},
    qualified, Identifier, Parameters,
};
use serde_json::json;
use std::collections::BTreeMap;
use weft_core::{
    backend::*,
    error::{Diagnostic, Result},
    ir::{Expression, Family, Identity, Node},
};
pub const PROFILE: &str = "truss-postgresql-candidate/0.1.0";
pub struct Candidate;
pub struct Mapping {
    admitted: Admission,
}
pub struct TargetPlan {
    emission: Emission,
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-BINDING", "binding", message)
}
fn capability(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-CAPABILITY", "lower", message)
}
fn original_element(
    c: &Context<'_>,
    identity: &Identity,
) -> Result<(serde_json::Value, serde_json::Value)> {
    let input = c
        .catalog
        .inputs
        .iter()
        .find(|input| {
            input.pin.document_id == identity.document_id && input.pin.revision == identity.revision
        })
        .ok_or_else(|| fail("Mapped identity does not belong to an original pinned module"))?;
    let document = weft_core::json::checked_json(&input.document_json)
        .map_err(|_| fail("Original model document could not be read"))?;
    let element = document["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == identity.module)
        .and_then(|m| m["elements"].as_array())
        .and_then(|elements| elements.iter().find(|e| e["id"] == identity.element))
        .ok_or_else(|| fail("Mapped identity is missing from its original document"))?
        .clone();
    Ok((document, element))
}
fn validate_key(
    mapping: &Admission,
    owner_identity: &Identity,
    key: &weft_core::application_model::AuthoredKey,
) -> Result<()> {
    let (entity_index, entity) = mapping.value["entities"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .find(|(_, entity)| entity["logical"] == json!(owner_identity))
        .ok_or_else(|| fail("Key owner mapping is missing"))?;
    let owner = &entity["typeId"];
    let (index, physical) = mapping.value["keys"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .find(|(_, physical)| &physical["ownerTypeId"] == owner && physical["keyId"] == key.id)
        .ok_or_else(|| fail("Authored key mapping is missing"))?;
    let expected = json!({"identity":"candidate-test-profile","version":"0.1.0","sha256":weft_core::json::sha256(b"{}")});
    for kind in ["comparison", "encoding"] {
        if physical[format!("{kind}Profile")] != expected
            || mapping
                .artifacts
                .get(&format!("/keys/{index}/{kind}Definition"))
                .is_none_or(|bytes| bytes != b"{}")
        {
            return Err(fail(
                "Selected key comparison or encoding meaning is not registered for this candidate",
            ));
        }
    }
    let ordered = key
        .fields
        .iter()
        .map(|field| {
            mapping.value["properties"]
                .as_array()
                .unwrap()
                .iter()
                .find(|property| {
                    &property["ownerTypeId"] == owner && property["logical"] == json!(field)
                })
                .map(|property| property["propertyId"].clone())
                .ok_or_else(|| fail("Key property mapping is missing"))
        })
        .collect::<Result<Vec<_>>>()?;
    if physical["orderedPropertyIds"] != json!(ordered) {
        return Err(fail("Key property order differs from authored key"));
    }
    let definition =
        mapping.decoded_json(&format!("/entities/{entity_index}/acceptedDefinition"))?;
    let authored = definition["keys"]
        .as_array()
        .and_then(|keys| keys.iter().find(|authored| authored["id"] == key.id))
        .ok_or_else(|| fail("Accepted entity lacks authored key"))?;
    if mapping.decoded_json(&format!("/keys/{index}/acceptedDefinition"))? != *authored {
        return Err(fail("Accepted key differs from its original entity key"));
    }
    Ok(())
}
fn obligations() -> Vec<Obligation> {
    vec![Obligation {
        id: "truss.candidate.context".into(),
        parameters: json!({"requirements":["same admitted catalog/layout/model/role view","exact native transport","exact numeric domain and finite SUM or runtime error","integrity checks before predicates/casts; no partial publication","complete authorized visibility before interpreting missing state or children","current authority and disclosure rechecked before publication"],"visibility":{"authorityOwner":"host","absenceRequiresCompleteStateView":true,"compoundRequiresCompleteChildView":true,"hiddenRowsAreNotAbsent":true},"execution":{"pinsRecheckedPerExecution":true,"sameAffineTransactionForIntegrityAndData":true,"separatePagesDoNotImplySnapshotContinuity":true,"unknownObligationMeaning":"refuse-before-sql"}}),
        owner: ObligationOwner::Host,
        failure_code: "WFT-OBLIGATION".into(),
    }]
}
impl Backend for Candidate {
    type Mapping = Mapping;
    type TargetPlan = TargetPlan;
    fn describe(&self) -> Result<Manifest> {
        let language = ["0.1.0", "0.2.0"]
            .iter()
            .map(|version| LanguageProfile {
                dialect_profile: format!("weft-sql/{version}"),
                ir_version: format!("weft-ir/{version}"),
            })
            .collect::<Vec<_>>();
        Ok(Manifest {
            backend_id: "truss.postgresql".into(),
            backend_version: "0.1.0-candidate".into(),
            interface_version: "weft-backend/0.2.0".into(),
            binding_profile: PROFILE.into(),
            language_profiles: language.clone(),
            target_profiles: vec![TargetProfile {
                id: "pg17.9-candidate".into(),
                engine: "postgresql".into(),
                engine_version: "17.9".into(),
                session_settings: json!({"server_encoding":"UTF8","comparison":"C","domain":"exact-or-error"}),
                storage_layout_revision: "truss-owner-candidate".into(),
                publication_revision: "host-admitted-view".into(),
            }],
            capabilities: [
                "scan",
                "project",
                "filter",
                "innerJoin",
                "equal",
                "and",
                "sum",
                "group",
                "type.string",
                "type.boolean",
                "type.integer",
                "type.decimal",
                "aggregate",
                "aggregate.count",
                "parameter.named",
                "compare.lexicographicGreater",
                "order.asc",
                "limit",
                "key.uniqueStable",
                "value.presence",
                "value.sequence",
                "value.map",
                "value.structured",
                "project.entity",
                "relationship.exists",
                "relationship.inverse",
                "relationship.boundedKeys",
            ]
            .iter()
            .map(|id| Capability {
                id: (*id).into(),
                target_profiles: vec!["pg17.9-candidate".into()],
                language_profiles: language.clone(),
                logical_domain: json!({"subset":"0.1 scalar relational plans"}),
                result_domain: json!({"transport":"exact text","qualification":"candidate"}),
                constraints: vec!["No production storage/domain support inferred".into()],
                obligations: obligations(),
                status: Status::Candidate,
                evidence: vec![],
            })
            .collect(),
            evidence: vec![],
        })
    }
    fn validate_binding(&self, c: &Context<'_>) -> Result<Validated<Mapping>> {
        let admitted = Admission::parse(&c.binding.json, PROFILE)?;
        let synthetic_profile = json!({"identity":"candidate-test-profile","version":"0.1.0","sha256":weft_core::json::sha256(b"{}")});
        if admitted.value["bindingProfile"] != synthetic_profile
            || !admitted.value["executionObligations"]
                .as_array()
                .unwrap()
                .is_empty()
        {
            return Err(fail("Binding profile or additional execution obligations are not registered for this candidate"));
        }
        for profile in [
            "readContextProfile",
            "layoutProfile",
            "identityProfile",
            "valueProfile",
            "keyProfile",
            "exporterProfile",
        ] {
            if admitted.value["basis"][profile] != synthetic_profile {
                return Err(fail(
                    "Execution basis profile is not registered for this candidate",
                ));
            }
        }
        for definition in [
            "readContextDefinition",
            "layoutInventory",
            "layoutSql",
            "acceptedCatalog",
        ] {
            if admitted.decoded_json(&format!("/basis/{definition}"))? != json!({}) {
                return Err(fail(
                    "Execution basis definition differs from the registered synthetic candidate",
                ));
            }
        }

        if admitted.decoded_json("/basis/modelBundle")?
            != serde_json::to_value(&c.catalog.inputs)
                .map_err(|_| fail("Model serialization failure"))?
        {
            return Err(fail(
                "Binding model bundle differs from original supplied modules",
            ));
        }
        Identifier::new(admitted.value["basis"]["namespace"].as_str().unwrap())?;
        for record in &c.selection.records {
            let index = admitted.value["entities"]
                .as_array()
                .unwrap()
                .iter()
                .position(|e| e["logical"] == json!(record))
                .ok_or_else(|| fail("Selected record mapping is missing"))?;
            let (document, element) = original_element(c, record)?;
            if admitted.decoded_json(&format!("/entities/{index}/source"))? != document
                || admitted.decoded_json(&format!("/entities/{index}/acceptedDefinition"))?
                    != element
            {
                return Err(fail(
                    "Selected record source or accepted definition differs from the pinned model",
                ));
            }
        }
        for field in &c.selection.fields {
            let indexes: Vec<_> = admitted.value["properties"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .filter(|(_, p)| p["logical"] == json!(field))
                .map(|(i, _)| i)
                .collect();
            if indexes.is_empty() {
                return Err(fail("Selected field mapping is missing"));
            }
            for i in indexes {
                // These are the registered synthetic codec pins for this candidate,
                // never a claim that an arbitrary owner profile has been adopted.
                let expected = json!({"identity":"candidate-test-profile","version":"0.1.0","sha256":weft_core::json::sha256(b"{}")});
                if admitted.value["properties"][i]["homeProfile"] != expected {
                    return Err(fail(
                        "Selected property home profile is not registered for this candidate",
                    ));
                }
                for kind in ["value", "presence"] {
                    if admitted.value["properties"][i][format!("{kind}Profile")] != expected
                        || admitted.decoded_json(&format!("/properties/{i}/{kind}Definition"))?
                            != json!({})
                    {
                        return Err(fail("Selected value or presence profile is not registered for this candidate"));
                    }
                }
                let (document, element) = original_element(c, field)?;
                if admitted.decoded_json(&format!("/properties/{i}/source"))? != document
                    || admitted.decoded_json(&format!("/properties/{i}/acceptedDefinition"))?
                        != element
                {
                    return Err(fail("Selected property source or accepted definition differs from the pinned model"));
                }
                if matches!(admitted.property_home(i)?, PropertyHome::Row { .. }) {
                    let home = admitted.decoded_json(&format!("/properties/{i}/homeDefinition"))?;
                    if home["joinProfile"] != expected
                        || home["joinDefinition"]["sha256"] != expected["sha256"]
                        || home["joinDefinition"]["bytesBase64"] != "e30="
                    {
                        return Err(fail(
                            "Selected native join profile is not registered for this candidate",
                        ));
                    }
                    if home["storedDomainObligation"] != "truss.candidate.context" {
                        return Err(fail(
                            "Selected row home names an unregistered stored-domain obligation",
                        ));
                    }
                }
            }
        }
        if let Plan::V02(plan) = c.plan {
            if let Some(key) = &plan.page_key {
                validate_key(&admitted, &plan.source.record, key)?;
            }
        }
        Ok(Validated {
            mapping: Mapping { admitted },
            additional_capabilities: vec![],
            coverage: c.selection.clone(),
            obligations: obligations(),
        })
    }
    fn assess(&self, c: &Context<'_>, _: &Mapping) -> Result<Vec<Assessment>> {
        Ok(c.plan
            .capabilities()
            .iter()
            .map(|id| Assessment {
                id: id.clone(),
                status: Status::Candidate,
                evidence: vec![],
                obligations: obligations(),
            })
            .collect())
    }
    fn lower(&self, c: &Context<'_>, m: &Mapping) -> Result<TargetPlan> {
        if let Plan::V02(plan) = c.plan {
            return application::lower(c, &m.admitted, plan);
        }
        let Plan::V01(plan) = c.plan else {
            return Err(capability(
                "Application-read lowering is not yet implemented",
            ));
        };
        let Node::Project { input, outputs } = &plan.root else {
            return Err(capability("Expected typed Project root"));
        };
        let namespace = Identifier::new(m.admitted.value["basis"]["namespace"].as_str().unwrap())?;
        let mut build = Build {
            mapping: &m.admitted,
            namespace,
            parameters: Parameters::default(),
            scans: BTreeMap::new(),
            fields: BTreeMap::new(),
            accesses: BTreeMap::new(),
            next_field: 0,
        };
        build.scans(input)?;
        build.fields_node(&plan.root)?;
        let mut projection = vec![];
        let mut columns = vec![];
        for (i, o) in outputs.iter().enumerate() {
            let expression = build.expression(&o.expression)?;
            projection.push(format!(
                "({expression})::text AS {}",
                Identifier::new(&o.name)?.sql()
            ));
            let ty = o.expression.logical_type().clone();
            let decoder = match ty.family {
                Family::String => ScalarDecoder::Text,
                Family::Boolean => ScalarDecoder::Boolean,
                Family::Integer => ScalarDecoder::ExactInteger,
                Family::Decimal => ScalarDecoder::ExactDecimal,
            };
            fn origins(e: &Expression) -> Vec<Identity> {
                match e {
                    Expression::Field { identity, .. } => vec![identity.clone()],
                    Expression::Sum { argument, .. } => origins(argument),
                    _ => vec![],
                }
            }
            columns.push(Column {
                position: i + 1,
                output_name: o.name.clone(),
                nullable: ty.nullable,
                representation: Representation::Scalar {
                    logical_type: ty,
                    carrier: ScalarCarrier::Text,
                    decoder,
                },
                source_identities: origins(&o.expression),
            });
        }
        let (from, filters, groups) = build.from(input)?;
        let mut sql = format!("SELECT {} FROM {from}", projection.join(", "));
        if !filters.is_empty() {
            sql.push_str(&format!(" WHERE {}", filters.join(" AND ")));
        }
        if !groups.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", groups.join(", ")));
        }
        let integrity:Vec<_>=build.scans.values().filter(|s|!s.guards.is_empty()).map(|s|json!({"sql":format!("SELECT count(*)::text FROM {} {} WHERE ({}) IS DISTINCT FROM TRUE",s.source,s.joins.join(" "),s.guards.join(" AND ")),"requiredViolations":"0"})).collect();
        let mut required = obligations();
        required.push(Obligation{id:"truss.candidate.scalarIntegrity".into(),parameters:json!({"checks":integrity,"context":"same snapshot before casts/user filters","domainQualification":"separate exact facets/codec obligation"}),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into()});
        Ok(TargetPlan {
            emission: Emission {
                sql,
                parameters: build.parameters.into_slots(),
                columns,
                obligations: required,
            },
        })
    }
    fn emit(&self, _: &Context<'_>, p: &TargetPlan) -> Result<Emission> {
        Ok(p.emission.clone())
    }
}
struct Scan {
    type_id: String,
    source: String,
    joins: Vec<String>,
    guards: Vec<String>,
}
struct Build<'a> {
    mapping: &'a Admission,
    namespace: Identifier,
    parameters: Parameters,
    scans: BTreeMap<String, Scan>,
    fields: BTreeMap<(String, String), String>,
    accesses: BTreeMap<(String, String), crate::access::ScalarAccess>,
    next_field: usize,
}
impl Build<'_> {
    fn scans(&mut self, node: &Node) -> Result<()> {
        match node {
            Node::Scan {
                occurrence, record, ..
            } => {
                let entity = self.mapping.value["entities"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| e["logical"] == json!(record))
                    .ok_or_else(|| fail("Record mapping missing"))?;
                let type_id = entity["typeId"].as_str().unwrap().to_owned();
                let slot = self.parameters.catalog(
                    crate::CatalogDomain::Int,
                    &type_id,
                    json!({"record":record}),
                )?;
                let source = format!(
                    "(SELECT * FROM {} WHERE type_id={slot}::int) AS {}",
                    qualified(&self.namespace, &Identifier::new("object")?),
                    Identifier::new(occurrence)?.sql()
                );
                self.scans.insert(
                    occurrence.clone(),
                    Scan {
                        type_id,
                        source,
                        joins: vec![],
                        guards: vec![],
                    },
                );
            }
            Node::InnerJoin { left, right, .. } => {
                self.scans(left)?;
                self.scans(right)?;
            }
            Node::Filter { input, .. }
            | Node::Aggregate { input, .. }
            | Node::Project { input, .. } => self.scans(input)?,
        };
        Ok(())
    }
    fn field(&mut self, e: &Expression) -> Result<()> {
        match e {
            Expression::Field {
                scan,
                identity,
                logical_type,
                ..
            } => {
                let key = (scan.clone(), json!(identity).to_string());
                if self.fields.contains_key(&key) {
                    return Ok(());
                }
                while ["weft_state", "weft_node", "weft_scalar"]
                    .iter()
                    .any(|prefix| {
                        self.scans
                            .contains_key(&format!("{prefix}_{}", self.next_field))
                    })
                {
                    self.next_field += 1;
                }
                let source = self
                    .scans
                    .get(scan)
                    .ok_or_else(|| fail("Unknown scan occurrence"))?;
                let choices: Vec<_> = self.mapping.value["properties"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| {
                        p["logical"] == json!(identity) && p["ownerTypeId"] == source.type_id
                    })
                    .collect();
                if choices.len() != 1 {
                    return Err(fail("Owner/field mapping is missing or ambiguous"));
                }
                let (index, property) = choices[0];
                let alias = Identifier::new(scan)?;
                let access = ObjectProperty {
                    namespace: &self.namespace,
                    owner_alias: &alias,
                    type_id: &source.type_id,
                    property_id: property["propertyId"].as_str().unwrap(),
                    logical_type,
                };
                let lowered = match self.mapping.property_home(index)? {
                    PropertyHome::Props { .. } => access.props(&mut self.parameters)?,
                    PropertyHome::Row { access: ref mode }
                        if mode == "scalar-root" || mode == "complete-value-tree" =>
                    {
                        let mut lowered = access.row(&mut self.parameters, self.next_field)?;
                        if mode == "complete-value-tree" {
                            let state =
                                Identifier::new(&format!("weft_state_{}", self.next_field))?.sql();
                            let root =
                                Identifier::new(&format!("weft_node_{}", self.next_field))?.sql();
                            lowered.integrity = format!(
                                "({} AND {})",
                                lowered.integrity,
                                crate::tree::integrity(&self.namespace, &state, &root)
                            );
                        }
                        lowered
                    }
                    _ => {
                        return Err(capability(
                            "Whole-value row access requires application decoder lowering",
                        ))
                    }
                };
                self.next_field += 1;
                let source = self.scans.get_mut(scan).unwrap();
                self.accesses.insert(key.clone(), lowered.clone());
                source.joins.extend(lowered.joins);
                source.guards.push(lowered.integrity);
                self.fields.insert(key, lowered.value);
            }
            Expression::Equal { left, right, .. } | Expression::And { left, right, .. } => {
                self.field(left)?;
                self.field(right)?;
            }
            Expression::Sum { argument, .. } => self.field(argument)?,
            Expression::Literal { .. } => {}
        };
        Ok(())
    }
    fn fields_node(&mut self, n: &Node) -> Result<()> {
        match n {
            Node::Scan { .. } => {}
            Node::InnerJoin { left, right, on } => {
                self.fields_node(left)?;
                self.fields_node(right)?;
                self.field(on)?;
            }
            Node::Filter { input, predicate } => {
                self.fields_node(input)?;
                self.field(predicate)?;
            }
            Node::Aggregate {
                input,
                groups,
                aggregates,
            } => {
                self.fields_node(input)?;
                for e in groups.iter().chain(aggregates) {
                    self.field(e)?;
                }
            }
            Node::Project { input, outputs } => {
                self.fields_node(input)?;
                for o in outputs {
                    self.field(&o.expression)?;
                }
            }
        };
        Ok(())
    }
    fn expression(&mut self, e: &Expression) -> Result<String> {
        Self::expression_with(&self.fields, &mut self.parameters, e)
    }
    fn expression_with(
        fields: &BTreeMap<(String, String), String>,
        parameters: &mut Parameters,
        e: &Expression,
    ) -> Result<String> {
        crate::expression::render(e, parameters, |e, operands, parameters| {
            Ok(match e {
                Expression::Field { scan, identity, .. } => fields
                    .get(&(scan.clone(), json!(identity).to_string()))
                    .ok_or_else(|| fail("Field access not prepared"))?
                    .clone(),
                Expression::Literal {
                    value,
                    logical_type,
                    span,
                } => {
                    let slot = parameters.push(
                        logical_type.clone(),
                        value.clone(),
                        json!({"literalSpan":span}),
                    )?;
                    let cast = match logical_type.family {
                        Family::String => "text",
                        Family::Boolean => "bool",
                        Family::Integer | Family::Decimal => "numeric",
                    };
                    let value = format!("{slot}::pg_catalog.{cast}");
                    if logical_type.family == Family::String {
                        format!("{value} COLLATE pg_catalog.\"C\"")
                    } else {
                        value
                    }
                }
                Expression::Equal { .. } => format!("({} = {})", operands[0], operands[1]),
                Expression::And { .. } => format!("({} AND {})", operands[0], operands[1]),
                Expression::Sum { .. } => format!("sum({})", operands[0]),
            })
        })
    }
    fn from(&mut self, n: &Node) -> Result<(String, Vec<String>, Vec<String>)> {
        let scans = &self.scans;
        let fields = &self.fields;
        let source = crate::relational::assemble(
            n,
            &mut self.parameters,
            |node, _| {
                let Node::Scan { occurrence, .. } = node else {
                    unreachable!("scan callback")
                };
                let source = scans
                    .get(occurrence)
                    .ok_or_else(|| fail("Original scan access is not prepared"))?;
                Ok(if source.joins.is_empty() {
                    source.source.clone()
                } else {
                    format!("({} {})", source.source, source.joins.join(" "))
                })
            },
            |expression, parameters| Self::expression_with(fields, parameters, expression),
        )?;
        Ok((source.sql, source.filters, source.groups))
    }
}
