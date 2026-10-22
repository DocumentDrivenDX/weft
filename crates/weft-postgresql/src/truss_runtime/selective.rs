// SPDX-License-Identifier: MIT OR Apache-2.0
//! ABI1.2 source choices from the original typed plan; no SQL recognition.
use super::*;

pub(super) fn query_abi_pin()->serde_json::Value {
    json!({"identity":"truss-postgresql-logical-query","version":"1.2",
        "sha256":weft_core::json::sha256(include_bytes!("../../../../tests/truss-selective/query-abi1.2.json"))})
}

fn operand(value:&app::Value)->Option<&str> {
    match value {app::Value::Literal{value,..}|app::Value::Parameter{value,..}=>Some(value),_=>None}
}

impl Build<'_> {
    pub(super) fn native(&self,name:&str)->String {
        if self.selective_sources {format!("person_{name}")} else {name.to_owned()}
    }

    pub(super) fn source(&mut self,scan:&app::Scan,continuation:Option<usize>,cursor_delivery:bool)->Result<String> {
        let native_type=self.mapping.record(&scan.record)?.type_id.clone();
        let alias=Identifier::new(&scan.occurrence)?.sql();
        let binding=self.binding_slot.clone();
        if self.selective_sources {
            // Every other filter remains outside this complete bounded candidate
            // source. Exceeding its budget refuses, never cuts a sparse page.
            for predicate in &self.plan.filters {
                if let app::Predicate::Equal{left,right}=predicate {
                    if left.scan!=scan.occurrence {continue;}
                    let Some(value)=operand(right) else {continue;};
                    let cell=match left.logical_type.family {
                        Family::String=>json!({"kind":"string","text":value}),
                        Family::Boolean=>json!({"kind":"boolean","value":match value {"true"=>true,"false"=>false,_=>return Err(fail("Original typed boolean operand required"))}}),
                        _=>continue,
                    };
                    let field=self.mapping.field(&scan.record,&left.identity)?;
                    let selected=json!({field.property_id.clone():cell}).to_string();
                    let slot=self.parameters.push(LogicalType{family:Family::String,facets:json!({}),nullable:false},selected,
                        json!({"kind":"backend-binding","identity":left.identity,"path":["indexedSelection"]}))?;
                    return Ok(format!("\"truss\".\"person_query_selected_objects\"({binding}::pg_catalog.jsonb,{native_type}::pg_catalog.int4,{slot}::pg_catalog.jsonb) AS {alias}"));
                }
            }
            if !cursor_delivery && scan.occurrence==self.plan.source.occurrence && self.plan.joins.is_empty()
                && self.plan.filters.len()==usize::from(continuation.is_some()) {
                if let Some(key)=&self.plan.page_key {
                    if key.fields.len()==1 && key.types.len()==1 && key.types[0].family==Family::String
                        && key.types[0].facets==json!({}) && !key.types[0].nullable {
                        let limit=self.plan.limit.map(u32::from).unwrap_or(self.mapping.binding.maximum_result_rows+1)
                            .min(self.mapping.binding.maximum_result_rows+1);
                        if (1..=1000).contains(&limit) {
                            let after=if let Some(index)=continuation {
                                let Some(app::Predicate::LexicographicGreater{columns,values})=self.plan.filters.get(index) else {return Err(fail("Original generated continuation required"));};
                                if columns.len()!=1 || values.len()!=1 || columns[0].scan!=scan.occurrence || columns[0].identity!=key.fields[0] {return Err(fail("Original complete page key continuation required"));}
                                let token=operand(&values[0]).ok_or_else(||fail("Original page continuation value required"))?;
                                self.parameters.push(key.types[0].clone(),token.to_owned(),json!({"kind":"backend-binding","identity":key.fields[0],"path":["pageContinuation"]}))?
                            } else {"NULL".into()};
                            return Ok(format!("\"truss\".\"person_query_seek_objects\"({binding}::pg_catalog.jsonb,{native_type}::pg_catalog.int4,{},({after})::pg_catalog.text,{limit}::pg_catalog.int4) AS {alias}",literal(&key.id)?));
                        }
                    }
                }
            }
        }
        // Preserve the old source's complete bag and source-bound-before-filter
        // contract for every path outside the new finite optimization subset.
        if let Some(key)=&self.plan.page_key {
            Ok(format!("\"truss\".\"{}\"({binding}::pg_catalog.jsonb,{native_type}::pg_catalog.int4,{}) AS {alias}",self.native("query_page_objects"),literal(&key.id)?))
        } else {
            Ok(format!("\"truss\".\"{}\"({binding}::pg_catalog.jsonb,{native_type}::pg_catalog.int4) AS {alias}",self.native("query_objects")))
        }
    }
}
