//! Application relational stages over the same candidate storage access.
use super::*;
use weft_core::{
    application_ir as app,
    application_model::Shape,
    ir::{LogicalType, Span},
};
fn legacy(f: &app::Field) -> Expression {
    Expression::Field {
        scan: f.scan.clone(),
        identity: f.identity.clone(),
        logical_type: f.logical_type.clone(),
        span: f.span.clone(),
    }
}
fn boolean() -> LogicalType {
    LogicalType {
        family: Family::Boolean,
        facets: json!({}),
        nullable: false,
    }
}
fn scan(s: &app::Scan) -> Node {
    Node::Scan {
        occurrence: s.occurrence.clone(),
        record: s.record.clone(),
        pin: s.pin.clone(),
    }
}
fn on(p: &app::Predicate) -> Result<Expression> {
    match p {
        app::Predicate::Equal {
            left,
            right: app::Value::Field { field },
        } => Ok(Expression::Equal {
            left: Box::new(legacy(left)),
            right: Box::new(legacy(field)),
            logical_type: boolean(),
            span: left.span.clone(),
        }),
        _ => Err(capability("Join ON requires typed field equality")),
    }
}
fn source(plan: &app::Plan) -> Result<Node> {
    let mut node = scan(&plan.source);
    for join in &plan.joins {
        let mut predicates = join.on.iter().map(on);
        let mut expression = predicates
            .next()
            .ok_or_else(|| capability("Join needs a predicate"))??;
        for predicate in predicates {
            expression = Expression::And {
                left: Box::new(expression),
                right: Box::new(predicate?),
                logical_type: boolean(),
                span: Span { start: 0, end: 0 },
            };
        }
        node = Node::InnerJoin {
            left: Box::new(node),
            right: Box::new(scan(&join.right)),
            on: expression,
        };
    }
    Ok(node)
}
fn descriptor<'a>(
    plan: &'a app::Plan,
    id: &Identity,
) -> Result<&'a weft_core::application_model::Descriptor> {
    plan.type_graph
        .iter()
        .find(|d| &d.identity == id)
        .ok_or_else(|| fail("Selected descriptor is missing"))
}
fn decoder(family: &Family) -> ScalarDecoder {
    match family {
        Family::String => ScalarDecoder::Text,
        Family::Boolean => ScalarDecoder::Boolean,
        Family::Integer => ScalarDecoder::ExactInteger,
        Family::Decimal => ScalarDecoder::ExactDecimal,
    }
}
pub(super) fn lower(
    context: &Context<'_>,
    mapping: &Admission,
    plan: &app::Plan,
) -> Result<TargetPlan> {
    for predicate in &plan.filters {
        if let app::Predicate::HasRelated { relationship, .. } = predicate {
            relationships::validate(context, mapping, relationship)?;
        }
    }
    for output in &plan.outputs {
        if let app::Expression::RelatedKeys { relationship, .. } = &output.expression {
            relationships::validate(context, mapping, relationship)?;
        }
    }
    let node = source(plan)?;
    let mut build = Build {
        mapping,
        namespace: Identifier::new(mapping.value["basis"]["namespace"].as_str().unwrap())?,
        parameters: Parameters::default(),
        scans: BTreeMap::new(),
        fields: BTreeMap::new(),
        accesses: BTreeMap::new(),
        next_field: 0,
    };
    build.scans(&node)?;
    build.fields_node(&node)?;
    for f in plan.groups.iter().chain(&plan.order) {
        build.field(&legacy(f))?;
    }
    for predicate in &plan.filters {
        build.application_fields(predicate)?;
    }
    for output in &plan.outputs {
        match &output.expression {
            app::Expression::Field { scan, identity } => {
                let d = descriptor(plan, identity)?;
                if !matches!(d.shape, Shape::Scalar { .. })
                    && build.recursive_props(plan, scan, identity)?
                {
                    if d.availability.as_deref() == Some("absent-allowed") {
                        build.allow_absence(scan, identity)?;
                    }
                    continue;
                }
                if let Shape::Structured { record } = &d.shape {
                    build.structured(plan, scan, identity, record)?;
                    if d.availability.as_deref() == Some("absent-allowed") {
                        build.allow_absence(scan, identity)?;
                    }
                    continue;
                }
                if let Shape::Sequence { item } | Shape::Map { item } = &d.shape {
                    let item = descriptor(plan, item)?;
                    let Shape::Scalar { logical_type } = &item.shape else {
                        return Err(capability(
                            "Recursive sequence items require recursive value lowering",
                        ));
                    };
                    if item.availability.as_deref() != Some("required") {
                        return Err(capability(
                            "Sequence item availability requires an admitted item codec",
                        ));
                    }
                    build.collection(
                        scan,
                        identity,
                        logical_type,
                        if matches!(d.shape, Shape::Map { .. }) {
                            crate::collection::Kind::Map
                        } else {
                            crate::collection::Kind::Sequence
                        },
                    )?;
                    if d.availability.as_deref() == Some("absent-allowed") {
                        build.allow_absence(scan, identity)?;
                    }
                    continue;
                }
                let Shape::Scalar { logical_type } = &d.shape else {
                    return Err(capability(
                        "Recursive application value lowering is not yet implemented",
                    ));
                };
                let f = Expression::Field {
                    scan: scan.clone(),
                    identity: identity.clone(),
                    logical_type: logical_type.clone(),
                    span: Span { start: 0, end: 0 },
                };
                build.field(&f)?;
                if d.availability.as_deref() == Some("absent-allowed") {
                    build.allow_absence(scan, identity)?;
                }
            }
            app::Expression::Sum { argument, .. } => build.field(&legacy(argument))?,
            app::Expression::Count { .. } => {}
            app::Expression::RelatedKeys { .. } => {}
        }
    }
    let mut projection = vec![];
    let mut columns = vec![];
    for (i, output) in plan.outputs.iter().enumerate() {
        let (value, representation, nullable, identities) = match &output.expression {
            app::Expression::Field { scan, identity } => {
                let d = descriptor(plan, identity)?;
                if let Shape::Sequence { .. } | Shape::Map { .. } | Shape::Structured { .. } =
                    &d.shape
                {
                    let access = &build.accesses[&(scan.clone(), json!(identity).to_string())];
                    (
                        if d.availability.as_deref() == Some("absent-allowed") {
                            format!("(CASE WHEN NOT {} THEN pg_catalog.jsonb_build_object('state','absent') ELSE pg_catalog.jsonb_build_object('state','value','value',{}) END)::text",access.present,access.value)
                        } else {
                            format!("({})::text", access.value)
                        },
                        Representation::Value {
                            descriptor: identity.clone(),
                            native_null: false,
                        },
                        false,
                        vec![identity.clone()],
                    )
                } else {
                    let Shape::Scalar { logical_type } = &d.shape else {
                        unreachable!()
                    };
                    let access = &build.accesses[&(scan.clone(), json!(identity).to_string())];
                    if d.availability.as_deref() == Some("required") {
                        (
                            format!("({})::text", access.value),
                            Representation::Scalar {
                                logical_type: logical_type.clone(),
                                carrier: ScalarCarrier::Text,
                                decoder: decoder(&logical_type.family),
                            },
                            false,
                            vec![identity.clone()],
                        )
                    } else {
                        let value = match logical_type.family {
                            Family::Integer | Family::Decimal => {
                                format!("pg_catalog.to_jsonb(({})::text)", access.value)
                            }
                            _ => format!("pg_catalog.to_jsonb({})", access.value),
                        };
                        (format!("(CASE WHEN NOT {} THEN pg_catalog.jsonb_build_object('state','absent') ELSE pg_catalog.jsonb_build_object('state','value','value',{value}) END)::text",access.present),Representation::Value{descriptor:identity.clone(),native_null:false},false,vec![identity.clone()])
                    }
                }
            }
            app::Expression::Count { logical_type } => (
                "count(*)::text".into(),
                Representation::Scalar {
                    logical_type: logical_type.clone(),
                    carrier: ScalarCarrier::Text,
                    decoder: ScalarDecoder::ExactInteger,
                },
                false,
                vec![plan.source.record.clone()],
            ),
            app::Expression::Sum {
                argument,
                logical_type,
            } => (
                format!("sum({})::text", build.expression(&legacy(argument))?),
                Representation::Scalar {
                    logical_type: logical_type.clone(),
                    carrier: ScalarCarrier::Text,
                    decoder: decoder(&logical_type.family),
                },
                logical_type.nullable,
                vec![argument.identity.clone()],
            ),
            app::Expression::RelatedKeys {
                scan,
                relationship,
                bound,
            } => (
                build.related_keys(scan, relationship, *bound)?,
                Representation::RelatedKeys {
                    relationship: relationship.identity.clone(),
                    key: relationship.target_key.clone(),
                    bound: *bound,
                },
                false,
                relationship.target_key.fields.clone(),
            ),
        };
        projection.push(format!(
            "{value} AS {}",
            Identifier::new(&output.name)?.sql()
        ));
        columns.push(Column {
            position: i + 1,
            carrier_name: None, output_name: output.name.clone(),
            representation,
            nullable,
            source_identities: identities,
        });
    }
    let (from, _, _) = build.from(&node)?;
    let mut sql = format!("SELECT {} FROM {from}", projection.join(", "));
    let filters = plan
        .filters
        .iter()
        .map(|p| build.application_predicate(p))
        .collect::<Result<Vec<_>>>()?;
    if !filters.is_empty() {
        sql.push_str(&format!(" WHERE {}", filters.join(" AND ")));
    }
    if !plan.groups.is_empty() {
        let groups = plan
            .groups
            .iter()
            .map(|f| build.expression(&legacy(f)))
            .collect::<Result<Vec<_>>>()?;
        sql.push_str(&format!(" GROUP BY {}", groups.join(", ")));
    }
    if !plan.order.is_empty() {
        let order = plan
            .order
            .iter()
            .map(|f| build.expression(&legacy(f)).map(|s| format!("{s} ASC")))
            .collect::<Result<Vec<_>>>()?;
        sql.push_str(&format!(" ORDER BY {}", order.join(", ")));
    }
    if let Some(limit) = plan.limit {
        let slot = build.parameters.push(
            LogicalType {
                family: Family::Integer,
                facets: json!({"integerWidth":{"bits":16,"signed":false}}),
                nullable: false,
            },
            limit.to_string(),
            json!({"applicationLimit":limit}),
        )?;
        sql.push_str(&format!(" LIMIT {slot}::int"));
    }
    let integrity:Vec<_>=build.scans.values().filter(|s|!s.guards.is_empty()).map(|s|json!({"sql":format!("SELECT count(*)::text FROM {} {} WHERE ({}) IS DISTINCT FROM TRUE",s.source,s.joins.join(" "),s.guards.join(" AND ")),"requiredViolations":"0"})).collect();
    let mut required = obligations();
    required.push(Obligation{id:"truss.candidate.scalarIntegrity".into(),parameters:json!({"checks":integrity,"context":"same snapshot before casts/user filters","domainQualification":"separate exact facets/codec obligation"}),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into()});
    let mut edge_checks = vec![];
    let mut seen = std::collections::BTreeSet::new();
    let related = plan
        .filters
        .iter()
        .filter_map(|p| match p {
            app::Predicate::HasRelated { relationship, .. } => Some(relationship),
            _ => None,
        })
        .chain(plan.outputs.iter().filter_map(|o| match &o.expression {
            app::Expression::RelatedKeys { relationship, .. } => Some(relationship),
            _ => None,
        }));
    for relationship in related {
        if !seen.insert(json!(relationship.identity).to_string()) {
            continue;
        }
        let physical = relationships::validate(context, mapping, relationship)?;
        let rid = build.parameters.catalog(
            crate::CatalogDomain::Int,
            physical["relationshipId"].as_str().unwrap(),
            json!({"integrityRelationship":relationship.identity}),
        )?;
        let source = build.parameters.catalog(
            crate::CatalogDomain::Int,
            physical["sourceTypeId"].as_str().unwrap(),
            json!({"integrityEndpoint":"source"}),
        )?;
        let target = build.parameters.catalog(
            crate::CatalogDomain::Int,
            physical["targetTypeId"].as_str().unwrap(),
            json!({"integrityEndpoint":"target"}),
        )?;
        let edges = qualified(&build.namespace, &Identifier::new("edge")?);
        let objects = qualified(&build.namespace, &Identifier::new("object")?);
        edge_checks.push(json!({"sql":format!("SELECT count(*)::text FROM {edges} e LEFT JOIN {objects} s ON s.id=e.source_id AND s.type_id=e.source_type LEFT JOIN {objects} t ON t.id=e.target_id AND t.type_id=e.target_type WHERE e.rel_type_id={rid}::int AND (s.id IS NULL OR t.id IS NULL OR e.source_type IS DISTINCT FROM {source}::int OR e.target_type IS DISTINCT FROM {target}::int)"),"requiredViolations":"0"}));
    }
    if !edge_checks.is_empty() {
        required.push(Obligation{id:"truss.candidate.relationshipIntegrity".into(),parameters:json!({"checks":edge_checks,"context":"same admitted authorized view before publication; endpoint visibility must be complete"}),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into()});
    }
    if plan.page_key.is_some() {
        required.push(Obligation{id:"truss.candidate.authoredPageKey".into(),parameters:json!({"key":plan.page_key,"snapshot":"caller-held for stable pages","comparison":"typed component order; no canonical key text"}),owner:ObligationOwner::Host,failure_code:"WFT-OBLIGATION".into()});
    }
    Ok(TargetPlan {
        emission: Emission {
            sql,
            parameters: build.parameters.into_slots(),
            columns,
            obligations: required,
        },
    })
}
impl Build<'_> {
    fn recursive_props(
        &mut self,
        plan: &app::Plan,
        scan: &str,
        identity: &Identity,
    ) -> Result<bool> {
        let key = (scan.to_owned(), json!(identity).to_string());
        if self.accesses.contains_key(&key) {
            return Ok(true);
        }
        let owner = self
            .scans
            .get(scan)
            .ok_or_else(|| fail("Recursive property owner is missing"))?;
        let (index, property) = self.mapping.value["properties"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .find(|(_, p)| p["ownerTypeId"] == owner.type_id && p["logical"] == json!(identity))
            .ok_or_else(|| fail("Recursive property mapping is missing"))?;
        if !matches!(
            self.mapping.property_home(index)?,
            PropertyHome::Props { .. }
        ) {
            if !matches!(self.mapping.property_home(index)?,PropertyHome::Row { access:mode } if mode=="complete-value-tree")
            {
                return Ok(false);
            }
            while ["weft_state", "weft_node", "weft_scalar"]
                .iter()
                .any(|p| self.scans.contains_key(&format!("{p}_{}", self.next_field)))
            {
                self.next_field += 1;
            }
            let alias = Identifier::new(scan)?;
            let source = crate::row_codec::Property {
                namespace: &self.namespace,
                owner: &alias,
                type_id: &owner.type_id,
                property_id: property["propertyId"].as_str().unwrap(),
            };
            let access = source.access(
                &plan.type_graph,
                identity,
                &mut self.parameters,
                self.next_field,
            )?;
            self.next_field += 1;
            let owner = self.scans.get_mut(scan).unwrap();
            owner.joins.extend(access.joins.clone());
            owner.guards.push(access.integrity.clone());
            self.accesses.insert(key, access);
            return Ok(true);
        }
        let slot = self.parameters.push(
            LogicalType {
                family: Family::String,
                facets: json!({}),
                nullable: false,
            },
            property["propertyId"].as_str().unwrap().into(),
            json!({"recursiveProperty":identity}),
        )?;
        let root = format!("{}.props", Identifier::new(scan)?.sql());
        let leaf = format!("({root} -> {slot}::text)");
        let encoded =
            crate::json_codec::encode(&plan.type_graph, identity, &leaf, &mut self.parameters)?;
        let access=crate::access::ScalarAccess{value:encoded.value,present:format!("CASE WHEN pg_catalog.jsonb_typeof({root})='object' THEN {root} ? {slot}::text ELSE NULL END"),native_null:format!("pg_catalog.jsonb_typeof({leaf})='null'"),integrity:format!("({root} IS NOT NULL AND pg_catalog.jsonb_typeof({root})='object' AND {})",encoded.integrity),joins:vec![]};
        self.scans
            .get_mut(scan)
            .unwrap()
            .guards
            .push(access.integrity.clone());
        self.accesses.insert(key, access);
        Ok(true)
    }
    fn structured(
        &mut self,
        plan: &app::Plan,
        scan: &str,
        identity: &Identity,
        record: &Identity,
    ) -> Result<()> {
        let key = (scan.to_owned(), json!(identity).to_string());
        if self.accesses.contains_key(&key) {
            return Ok(());
        }
        let Shape::Record { members } = &descriptor(plan, record)?.shape else {
            return Err(fail("Structured record descriptor is missing"));
        };
        let members = members
            .iter()
            .map(|member| {
                let d = descriptor(plan, &member.identity)?;
                let Shape::Scalar { logical_type } = &d.shape else {
                    return Err(capability(
                        "Recursive structured members require recursive value lowering",
                    ));
                };
                Ok(crate::structured::Member {
                    name: member.name.clone(),
                    identity: member.identity.clone(),
                    logical_type: logical_type.clone(),
                    optional: d.availability.as_deref() == Some("absent-allowed"),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        while ["weft_state", "weft_node", "weft_scalar"]
            .iter()
            .any(|p| self.scans.contains_key(&format!("{p}_{}", self.next_field)))
        {
            self.next_field += 1;
        }
        let owner = self
            .scans
            .get(scan)
            .ok_or_else(|| fail("Structured owner scan is missing"))?;
        let (index, property) = self.mapping.value["properties"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .find(|(_, p)| p["ownerTypeId"] == owner.type_id && p["logical"] == json!(identity))
            .ok_or_else(|| fail("Structured property mapping is missing"))?;
        let alias = Identifier::new(scan)?;
        let access = crate::structured::StructuredProperty {
            namespace: &self.namespace,
            owner: &alias,
            type_id: &owner.type_id,
            property_id: property["propertyId"].as_str().unwrap(),
            members: &members,
        };
        let lowered = match self.mapping.property_home(index)? {
            PropertyHome::Props { .. } => access.props(&mut self.parameters)?,
            PropertyHome::Row { access: mode } if mode == "complete-value-tree" => {
                access.row(&mut self.parameters, self.next_field)?
            }
            _ => {
                return Err(capability(
                    "Structured row storage requires complete-value-tree access",
                ))
            }
        };
        self.next_field += 1;
        let owner = self.scans.get_mut(scan).unwrap();
        owner.joins.extend(lowered.joins.clone());
        owner.guards.push(lowered.integrity.clone());
        self.accesses.insert(key, lowered);
        Ok(())
    }
    fn allow_absence(&mut self, scan: &str, identity: &Identity) -> Result<()> {
        let key = (scan.to_owned(), json!(identity).to_string());
        let access = self
            .accesses
            .get(&key)
            .ok_or_else(|| fail("Optional property access is missing"))?;
        let owner = self
            .scans
            .get_mut(scan)
            .ok_or_else(|| fail("Optional property owner is missing"))?;
        if let Some(index) = owner.guards.iter().position(|g| g == &access.integrity) {
            owner.guards[index] = format!("((NOT {}) OR {})", access.present, access.integrity);
        }
        Ok(())
    }
    fn collection(
        &mut self,
        scan: &str,
        identity: &Identity,
        item: &LogicalType,
        kind: crate::collection::Kind,
    ) -> Result<()> {
        let key = (scan.to_owned(), json!(identity).to_string());
        if self.accesses.contains_key(&key) {
            return Ok(());
        }
        while ["weft_state", "weft_node", "weft_scalar"]
            .iter()
            .any(|p| self.scans.contains_key(&format!("{p}_{}", self.next_field)))
        {
            self.next_field += 1;
        }
        let owner = self
            .scans
            .get(scan)
            .ok_or_else(|| fail("Sequence owner scan is missing"))?;
        let (index, property) = self.mapping.value["properties"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .find(|(_, p)| p["ownerTypeId"] == owner.type_id && p["logical"] == json!(identity))
            .ok_or_else(|| fail("Sequence property mapping is missing"))?;
        let alias = Identifier::new(scan)?;
        let access = crate::collection::CollectionProperty {
            namespace: &self.namespace,
            owner: &alias,
            type_id: &owner.type_id,
            property_id: property["propertyId"].as_str().unwrap(),
            item,
            kind,
        };
        let lowered = match self.mapping.property_home(index)? {
            PropertyHome::Props { .. } => access.props(&mut self.parameters)?,
            PropertyHome::Row { access: mode } if mode == "complete-value-tree" => {
                access.row(&mut self.parameters, self.next_field)?
            }
            _ => {
                return Err(capability(
                    "Sequence row storage requires complete-value-tree access",
                ))
            }
        };
        self.next_field += 1;
        let owner = self.scans.get_mut(scan).unwrap();
        owner.joins.extend(lowered.joins.clone());
        owner.guards.push(lowered.integrity.clone());
        self.accesses.insert(key, lowered);
        Ok(())
    }
    fn fresh_alias(&mut self, prefix: &str) -> String {
        loop {
            let name = format!("{prefix}_{}", self.next_field);
            self.next_field += 1;
            if !self.scans.contains_key(&name) {
                return name;
            }
        }
    }

    fn related_keys(
        &mut self,
        scan: &str,
        relationship: &weft_core::application_model::RelationshipRead,
        bound: u16,
    ) -> Result<String> {
        let physical = self.mapping.value["relationships"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["logical"] == json!(relationship.identity))
            .ok_or_else(|| fail("Related projection mapping is missing"))?
            .clone();
        let alias = loop {
            let name = format!("weft_related_{}", self.next_field);
            self.next_field += 1;
            if !self.scans.contains_key(&name) {
                break name;
            }
        };
        let (from_side, to_side) = if relationship.inverse {
            ("target", "source")
        } else {
            ("source", "target")
        };
        let type_id = physical[format!("{to_side}TypeId")]
            .as_str()
            .unwrap()
            .to_owned();
        let slot = self.parameters.catalog(
            crate::CatalogDomain::Int,
            &type_id,
            json!({"relationshipEndpoint":relationship.identity}),
        )?;
        let quoted = Identifier::new(&alias)?.sql();
        self.scans.insert(
            alias.clone(),
            Scan {
                type_id,
                source: format!(
                    "(SELECT * FROM {} WHERE type_id={slot}::int) AS {quoted}",
                    qualified(&self.namespace, &Identifier::new("object")?)
                ),
                joins: vec![],
                guards: vec![],
            },
        );
        let mut ordered = vec![];
        let mut values = vec![];
        for (identity, ty) in relationship
            .target_key
            .fields
            .iter()
            .zip(&relationship.target_key.types)
        {
            let expression = Expression::Field {
                scan: alias.clone(),
                identity: identity.clone(),
                logical_type: ty.clone(),
                span: Span { start: 0, end: 0 },
            };
            self.field(&expression)?;
            let value = self.expression(&expression)?;
            values.push(match ty.family {
                Family::Integer | Family::Decimal => format!("({value})::text"),
                _ => value.clone(),
            });
            ordered.push(value);
        }
        let rid = self.parameters.catalog(
            crate::CatalogDomain::Int,
            physical["relationshipId"].as_str().unwrap(),
            json!({"relationship":relationship.identity}),
        )?;
        let source_type = self.parameters.catalog(
            crate::CatalogDomain::Int,
            physical[format!("{from_side}TypeId")].as_str().unwrap(),
            json!({"relationshipSource":relationship.identity}),
        )?;
        let ty = LogicalType {
            family: Family::Integer,
            facets: json!({"integerWidth":{"bits":16,"signed":false}}),
            nullable: false,
        };
        let bound_slot =
            self.parameters
                .push(ty.clone(), bound.to_string(), json!({"relatedBound":bound}))?;
        let probe_slot = self.parameters.push(
            ty,
            (u32::from(bound) + 1).to_string(),
            json!({"relatedProbe":u32::from(bound)+1}),
        )?;
        let target_source = self.scans[&alias].source.clone();
        let target_joins = self.scans[&alias].joins.join(" ");
        let edge = Identifier::new(&self.fresh_alias("weft_edge"))?.sql();
        let owner = Identifier::new(scan)?.sql();
        let from = format!("{} AS {edge} JOIN {} ON {quoted}.id={edge}.{to_side}_id AND {quoted}.type_id={edge}.{to_side}_type {}",qualified(&self.namespace,&Identifier::new("edge")?),target_source,target_joins);
        let predicate = format!("{edge}.rel_type_id={rid}::int AND {edge}.{from_side}_id={owner}.id AND {edge}.{from_side}_type={source_type}::int");
        let order = ordered.join(", ");
        let inner = format!("SELECT pg_catalog.jsonb_build_array({}) AS k, row_number() OVER (ORDER BY {order}) AS ord FROM {from} WHERE {predicate} ORDER BY {order} LIMIT {probe_slot}::int",values.join(", "));
        Ok(format!("(SELECT pg_catalog.jsonb_build_object('items',COALESCE(pg_catalog.jsonb_agg(k ORDER BY ord) FILTER (WHERE ord<={bound_slot}::int),'[]'::jsonb),'truncated',count(*)>{bound_slot}::int)::text FROM ({inner}) AS weft_bounded)"))
    }
    fn application_fields(&mut self, p: &app::Predicate) -> Result<()> {
        match p {
            app::Predicate::Equal { left, right } => {
                self.field(&legacy(left))?;
                if let app::Value::Field { field } = right {
                    self.field(&legacy(field))?;
                }
            }
            app::Predicate::LexicographicGreater { columns, values } => {
                for f in columns {
                    self.field(&legacy(f))?;
                }
                for v in values {
                    if let app::Value::Field { field } = v {
                        self.field(&legacy(field))?;
                    }
                }
            }
            app::Predicate::HasRelated { key, .. } => {
                for v in key {
                    if let app::Value::Field { field } = v {
                        self.field(&legacy(field))?;
                    }
                }
            }
        };
        Ok(())
    }
    fn application_value(&mut self, value: &app::Value) -> Result<String> {
        let (value, ty, origin) = match value {
            app::Value::Field { field } => return self.expression(&legacy(field)),
            app::Value::Literal {
                value,
                logical_type,
                span,
            } => (value, logical_type, json!({"literalSpan":span})),
            app::Value::Parameter {
                name,
                value,
                logical_type,
                span,
            } => (value, logical_type, json!({"parameter":name,"span":span})),
        };
        let slot = self.parameters.push(ty.clone(), value.clone(), origin)?;
        let cast = match ty.family {
            Family::String => "text",
            Family::Boolean => "bool",
            Family::Integer | Family::Decimal => "numeric",
        };
        let value = format!("{slot}::pg_catalog.{cast}");
        Ok(if ty.family == Family::String {
            format!("{value} COLLATE pg_catalog.\"C\"")
        } else {
            value
        })
    }
    fn application_predicate(&mut self, p: &app::Predicate) -> Result<String> {
        Ok(match p {
            app::Predicate::Equal { left, right } => format!(
                "({} = {})",
                self.expression(&legacy(left))?,
                self.application_value(right)?
            ),
            app::Predicate::LexicographicGreater { columns, values } => {
                let fields = columns
                    .iter()
                    .map(|f| self.expression(&legacy(f)))
                    .collect::<Result<Vec<_>>>()?;
                let values = values
                    .iter()
                    .map(|v| self.application_value(v))
                    .collect::<Result<Vec<_>>>()?;
                format!("(ROW({}) > ROW({}))", fields.join(", "), values.join(", "))
            }
            app::Predicate::HasRelated {
                scan,
                relationship,
                key,
            } => {
                let physical = self.mapping.value["relationships"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["logical"] == json!(relationship.identity))
                    .unwrap()
                    .clone();
                let alias = loop {
                    let name = format!("weft_related_{}", self.next_field);
                    self.next_field += 1;
                    if !self.scans.contains_key(&name) {
                        break name;
                    }
                };
                let (from_side, to_side) = if relationship.inverse {
                    ("target", "source")
                } else {
                    ("source", "target")
                };
                let type_id = physical[format!("{to_side}TypeId")]
                    .as_str()
                    .unwrap()
                    .to_owned();
                let slot = self.parameters.catalog(
                    crate::CatalogDomain::Int,
                    &type_id,
                    json!({"relationshipEndpoint":relationship.identity}),
                )?;
                let quoted = Identifier::new(&alias)?.sql();
                self.scans.insert(
                    alias.clone(),
                    Scan {
                        type_id,
                        source: format!(
                            "(SELECT * FROM {} WHERE type_id={slot}::int) AS {quoted}",
                            qualified(&self.namespace, &Identifier::new("object")?)
                        ),
                        joins: vec![],
                        guards: vec![],
                    },
                );
                let mut matches = vec![];
                for ((identity, ty), value) in relationship
                    .target_key
                    .fields
                    .iter()
                    .zip(&relationship.target_key.types)
                    .zip(key)
                {
                    let expression = Expression::Field {
                        scan: alias.clone(),
                        identity: identity.clone(),
                        logical_type: ty.clone(),
                        span: Span { start: 0, end: 0 },
                    };
                    self.field(&expression)?;
                    matches.push(format!(
                        "{} = {}",
                        self.expression(&expression)?,
                        self.application_value(value)?
                    ));
                }
                let target_source = self.scans[&alias].source.clone();
                let target_joins = self.scans[&alias].joins.join(" ");
                let rel_slot = self.parameters.catalog(
                    crate::CatalogDomain::Int,
                    physical["relationshipId"].as_str().unwrap(),
                    json!({"relationship":relationship.identity}),
                )?;
                let source_type = self.parameters.catalog(
                    crate::CatalogDomain::Int,
                    physical[format!("{from_side}TypeId")].as_str().unwrap(),
                    json!({"relationshipSource":relationship.identity}),
                )?;
                let edge = Identifier::new(&self.fresh_alias("weft_edge"))?.sql();
                let owner = Identifier::new(scan)?.sql();
                format!("EXISTS (SELECT 1 FROM {} AS {edge} JOIN {} ON {quoted}.id={edge}.{to_side}_id AND {quoted}.type_id={edge}.{to_side}_type {} WHERE {edge}.rel_type_id={rel_slot}::int AND {edge}.{from_side}_id={owner}.id AND {edge}.{from_side}_type={source_type}::int AND {})", qualified(&self.namespace,&Identifier::new("edge")?), target_source,target_joins,matches.join(" AND "))
            }
        })
    }
}
