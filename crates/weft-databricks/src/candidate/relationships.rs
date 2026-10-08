//! Authored directed edges; physical IDs never become logical keys.
use super::*;
use weft_core::{application_ir as app, application_model::RelationshipRead, ir::Span};

struct Access {
    descriptor: RelationshipRead,
    source: String,
    target: String,
    edge: String,
    physical: crate::binding::Relationship,
}
pub(super) struct Traversals {
    accesses: BTreeMap<String, Access>,
    next: usize,
}
fn key(r: &RelationshipRead) -> String {
    json!(r).to_string()
}
fn field(scan: &str, identity: &Identity, ty: &LogicalType) -> Expression {
    Expression::Field {
        scan: scan.into(),
        identity: identity.clone(),
        logical_type: ty.clone(),
        span: Span { start: 0, end: 0 },
    }
}
fn fresh(lower: &Lower<'_>, next: &mut usize, suffix: &str) -> String {
    loop {
        let name = format!("__weft_rel_{}_{}", *next, suffix);
        *next += 1;
        if !lower.scans.contains_key(&name) {
            return name;
        }
    }
}
impl Traversals {
    pub(super) fn collect(lower: &mut Lower<'_>, plan: &app::Plan) -> Result<Self> {
        let mut result = Self {
            accesses: BTreeMap::new(),
            next: 0,
        };
        let mut refs = Vec::new();
        for p in plan
            .filters
            .iter()
            .chain(plan.joins.iter().flat_map(|j| &j.on))
        {
            if let app::Predicate::HasRelated {
                scan, relationship, ..
            } = p
            {
                refs.push((scan, relationship));
            }
        }
        for o in &plan.outputs {
            if let app::Expression::RelatedKeys {
                scan, relationship, ..
            } = &o.expression
            {
                refs.push((scan, relationship));
            }
        }
        for (scan, r) in refs {
            lower.physical_ids.insert(scan.clone());
            if result.accesses.contains_key(&key(r)) {
                continue;
            }
            let physical = lower
                .binding
                .relationships
                .iter()
                .find(|p| p.logical == json!(r.identity))
                .cloned()
                .ok_or_else(|| fail("WFT-BINDING", "Missing authored edge mapping"))?;
            let source = fresh(lower, &mut result.next, "source");
            lower.scans.insert(source.clone(), physical.source.clone());
            lower.physical_ids.insert(source.clone());
            let target = fresh(lower, &mut result.next, "target");
            lower.scans.insert(target.clone(), physical.target.clone());
            lower.physical_ids.insert(target.clone());
            let keys = if r.inverse {
                [(&source, &r.target_key), (&target, &r.source_key)]
            } else {
                [(&source, &r.source_key), (&target, &r.target_key)]
            };
            for (alias, k) in keys {
                for (id, ty) in k.fields.iter().zip(&k.types) {
                    collect_expression(&field(alias, id, ty), &mut lower.fields);
                }
            }
            let edge = fresh(lower, &mut result.next, "edge");
            result.accesses.insert(
                key(r),
                Access {
                    descriptor: r.clone(),
                    source,
                    target,
                    edge,
                    physical,
                },
            );
        }
        Ok(result)
    }
    pub(super) fn prepare(&self, lower: &mut Lower<'_>) -> Result<Vec<Value>> {
        let mut checks = Vec::new();
        for access in self.accesses.values() {
            let p = &access.physical;
            let source_record = lower
                .binding
                .records
                .iter()
                .find(|r| r.logical == p.source)
                .unwrap()
                .clone();
            let target_record = lower
                .binding
                .records
                .iter()
                .find(|r| r.logical == p.target)
                .unwrap()
                .clone();
            let scope = lower.slot(
                string_type(),
                p.source_system.clone(),
                json!({"kind":"relationshipSourceScope","relationship":p.logical}),
            )?;
            let int = LogicalType {
                family: Family::Integer,
                facets: json!({"integerWidth":{"bits":64,"signed":true}}),
                nullable: false,
            };
            let relation = lower.slot(
                int.clone(),
                p.type_id.clone(),
                json!({"kind":"relationshipType","relationship":p.logical}),
            )?;
            let source_type = lower.slot(
                int.clone(),
                source_record.type_id,
                json!({"kind":"relationshipEndpointType","side":"source"}),
            )?;
            let target_type = lower.slot(
                int,
                target_record.type_id,
                json!({"kind":"relationshipEndpointType","side":"target"}),
            )?;
            let table = lower.binding.publication.tables[p.table].sql();
            let owner=format!("(e.source_system COLLATE UTF8_BINARY) = ({scope} COLLATE UTF8_BINARY) AND e.rel_type_id = CAST({relation} AS BIGINT)");
            let edge = binding::quote(&access.edge);
            let source = binding::quote(&access.source);
            let target = binding::quote(&access.target);
            lower.ctes.push(format!(
                "{edge} AS (SELECT e.* FROM {table} e WHERE {owner})"
            ));
            let native = if p.kind == RecordKind::Edge {
                let rev = lower.slot(
                    string_type(),
                    p.schema_revision.clone(),
                    json!({"kind":"relationshipSchemaRevision","relationship":p.logical}),
                )?;
                format!("e.source_type = CAST({source_type} AS BIGINT) AND e.target_type = CAST({target_type} AS BIGINT) AND (e.schema_revision COLLATE UTF8_BINARY) = ({rev} COLLATE UTF8_BINARY)")
            } else {
                "(e.src COLLATE UTF8_BINARY) = (s.__node_key COLLATE UTF8_BINARY) AND (e.dst COLLATE UTF8_BINARY) = (t.__node_key COLLATE UTF8_BINARY)".into()
            };
            let count = count_sql();
            checks.push(json!({"relationship":p.logical,"sql":format!("WITH {} SELECT CAST({count} AS STRING) AS violations FROM {edge} e LEFT JOIN {source} s ON e.source_id=s.__id LEFT JOIN {target} t ON e.target_id=t.__id WHERE CASE WHEN s.__id IS NOT NULL AND t.__id IS NOT NULL AND ({native}) THEN FALSE ELSE TRUE END",lower.ctes.join(", ")),"failureCode":"WFT-BINDING"}));
            checks.push(json!({"relationship":p.logical,"sql":format!("WITH {} SELECT CAST({count} AS STRING) AS violations FROM (SELECT id FROM {edge} GROUP BY id HAVING {count}>1) duplicates",lower.ctes.join(", ")),"failureCode":"WFT-BINDING"}));
            let r = &access.descriptor;
            let endpoint_keys = if r.inverse {
                [
                    (&access.source, &r.target_key),
                    (&access.target, &r.source_key),
                ]
            } else {
                [
                    (&access.source, &r.source_key),
                    (&access.target, &r.target_key),
                ]
            };
            for (alias, k) in endpoint_keys {
                let keys = k
                    .fields
                    .iter()
                    .zip(&k.types)
                    .map(|(id, ty)| lower.expression(&field(alias, id, ty)))
                    .collect::<Result<Vec<_>>>()?
                    .join(", ");
                let quoted = binding::quote(alias);
                for columns in [keys, "__id".into()] {
                    checks.push(json!({"relationship":p.logical,"sql":format!("WITH {} SELECT CAST({count} AS STRING) AS violations FROM (SELECT {columns} FROM {quoted} GROUP BY {columns} HAVING {count}>1) duplicates",lower.ctes.join(", ")),"failureCode":"WFT-BINDING"}));
                }
            }
            // Authored multiplicities remain in their forward orientation, even for inverse reads.
            for (alias, column, m) in [
                (&source, "source_id", &r.target_multiplicity),
                (&target, "target_id", &r.source_multiplicity),
            ] {
                let min = m["min"].as_u64().unwrap();
                let mut violations = format!("n < {min} OR n > 9223372036854775807");
                if let Some(max) = m["max"].as_u64() {
                    violations.push_str(&format!(" OR n > {max}"));
                }
                checks.push(json!({"relationship":p.logical,"sql":format!("WITH {} SELECT CAST({count} AS STRING) AS violations FROM {alias} v LEFT JOIN (SELECT {column}, {count} AS n FROM {edge} GROUP BY {column}) degree ON v.__id=degree.{column} WHERE {}",lower.ctes.join(", "),violations.replace("n ","coalesce(n, CAST(0 AS DECIMAL(38,0))) ")),"failureCode":"WFT-BINDING"}));
            }
        }
        Ok(checks)
    }
    fn endpoint<'a>(
        &'a self,
        r: &RelationshipRead,
    ) -> Result<(&'a Access, &'a str, &'static str, &'static str)> {
        let a = self
            .accesses
            .get(&key(r))
            .ok_or_else(|| fail("WFT-EMIT", "Unprepared relationship"))?;
        Ok(if r.inverse {
            (a, &a.source, "target_id", "source_id")
        } else {
            (a, &a.target, "source_id", "target_id")
        })
    }
    pub(super) fn exists(
        &self,
        lower: &mut Lower<'_>,
        scan: &str,
        r: &RelationshipRead,
        values: &[app::Value],
    ) -> Result<String> {
        let (a, target, from_id, to_id) = self.endpoint(r)?;
        let mut predicates = vec![format!("e.{from_id} = {}.__id", binding::quote(scan))];
        for ((id, ty), v) in r
            .target_key
            .fields
            .iter()
            .zip(&r.target_key.types)
            .zip(values)
        {
            let lhs = lower.expression(&field(target, id, ty))?;
            let rhs = super::application::value(lower, v)?;
            predicates.push(format!("{lhs} = {rhs}"));
        }
        Ok(format!(
            "EXISTS (SELECT 1 FROM {} e JOIN {} ON e.{to_id} = {}.__id WHERE {})",
            binding::quote(&a.edge),
            binding::quote(target),
            binding::quote(target),
            predicates.join(" AND ")
        ))
    }
    pub(super) fn related(
        &mut self,
        lower: &mut Lower<'_>,
        scan: &str,
        r: &RelationshipRead,
        bound: u16,
    ) -> Result<(String, String)> {
        let (a, target, from_id, to_id) = self.endpoint(r)?;
        let edge = binding::quote(&a.edge);
        let target = target.to_owned();
        let values = r
            .target_key
            .fields
            .iter()
            .zip(&r.target_key.types)
            .map(|(id, ty)| lower.expression(&field(&target, id, ty)))
            .collect::<Result<Vec<_>>>()?;
        let ordered = values
            .iter()
            .map(|v| format!("{v} ASC"))
            .collect::<Vec<_>>()
            .join(", ");
        let tuple = values
            .iter()
            .map(|v| format!("CAST({v} AS STRING)"))
            .collect::<Vec<_>>()
            .join(", ");
        let ranked = fresh(lower, &mut self.next, "ranked");
        let aggregate = fresh(lower, &mut self.next, "bounded");
        let qrank = binding::quote(&ranked);
        let qaggregate = binding::quote(&aggregate);
        let qtarget = binding::quote(&target);
        lower.ctes.push(format!("{qrank} AS (SELECT e.{from_id} AS owner_id, array({tuple}) AS k, row_number() OVER (PARTITION BY e.{from_id} ORDER BY {ordered}, e.id ASC) AS ord FROM {edge} e JOIN {qtarget} ON e.{to_id}={qtarget}.__id)"));
        lower.ctes.push(format!("{qaggregate} AS (SELECT owner_id, to_json(named_struct('items', transform(filter(sort_array(collect_list(named_struct('ord',ord,'k',k))), x -> x.ord <= {bound}), x -> x.k), 'truncated', {} > {bound})) AS value FROM {qrank} WHERE ord <= {} GROUP BY owner_id)",count_sql(),u32::from(bound)+1));
        let join = format!(
            "LEFT JOIN {qaggregate} ON {qaggregate}.owner_id = {}.__id",
            binding::quote(scan)
        );
        Ok((
            format!("coalesce({qaggregate}.value, '{{\"items\":[],\"truncated\":false}}')"),
            join,
        ))
    }
}
