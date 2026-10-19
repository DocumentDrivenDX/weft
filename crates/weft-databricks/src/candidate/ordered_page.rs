// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native Spark lowering of original Core ordering slots, never emitted-SQL surgery.
use super::*;
use weft_core::application_ordered_page::{CursorValue,OrderedPage};
pub(super) struct Clauses {pub order:Vec<String>,pub after:Option<String>}
pub(super) fn append(lower:&mut Lower<'_>,plan:&app::Plan,page:&OrderedPage,outputs:&mut Vec<String>,columns:&mut Vec<Column>)->Result<Clauses>{
 let mut order=vec![];let mut expressions=vec![];let mut nullish=vec![];
 for (i,slot) in page.sort_slots().iter().enumerate(){
  let f=app::Field{scan:plan.source.occurrence.clone(),identity:slot.identity().clone(),logical_type:slot.logical_type().clone(),span:Span{start:0,end:0}};
  let sql=lower.expression(&legacy(&f))?;
  let key=(f.scan.clone(),serde_json::to_string(&f.identity).unwrap());let present=lower.presence.get(&key).cloned().unwrap_or_else(||"TRUE".into());
  let absent=format!("NOT ({present})");let nil=format!("({absent} OR {sql} IS NULL)");
  order.push(format!("{sql} {} NULLS {}",if slot.descending(){"DESC"}else{"ASC"},if slot.nulls_first(){"FIRST"}else{"LAST"}));
  let scalar=if slot.logical_type().family==Family::Boolean{sql.clone()}else{format!("CAST({sql} AS STRING)")};
  let envelope=format!("CASE WHEN {absent} THEN to_json(named_struct('state','absent')) WHEN {sql} IS NULL THEN to_json(named_struct('state','null')) ELSE to_json(named_struct('state','value','value',{scalar})) END");
  let state=format!("CASE WHEN {absent} THEN 'missing' WHEN {sql} IS NULL THEN 'null' ELSE 'present' END");
  let (value_position,state_position)=slot.hidden_outputs();
  for (position,name,sql,representation) in [(value_position,format!("__weft_sort_value_{i}"),envelope,Representation::Value{descriptor:slot.identity().clone(),native_null:true,outer_join:None}),(state_position,format!("__weft_sort_state_{i}"),state,representation(&string_type()))] {
   if position!=columns.len(){return Err(fail("WFT-EMIT","Original ordered output positions differ"));}
   outputs.push(format!("{sql} AS {}",binding::quote(&name)));columns.push(Column{position:position+1,output_name:name,carrier_name:None,representation,source_identities:vec![slot.identity().clone()],nullable:false});
  }
  expressions.push(sql);nullish.push(nil);
 }
 let after=if let Some(cursor)=page.after(){
  let mut equal=vec![];let mut greater=vec![];
  for (i,(slot,value)) in page.sort_slots().iter().zip(&cursor.sort_values).enumerate(){
   let nil=&nullish[i];let sql=&expressions[i];
   match value {
    CursorValue::Null{}|CursorValue::Missing{}=>{equal.push(nil.clone());greater.push(if slot.nulls_first(){format!("NOT {nil}")}else{"FALSE".into()});},
    CursorValue::Present{value,..}=>{
     let mut ty=slot.logical_type().clone();ty.nullable=false;
     let parameter=lower.slot(ty.clone(),value.clone(),json!({"kind":"orderedPageCursor","slot":i,"field":slot.identity(),"state":"present"}))?;
     let native=typed(&parameter,&ty);
     equal.push(format!("(NOT {nil} AND {sql} = {native})"));
     let strict=if ty.family==Family::Boolean {
      // Boolean comparison grammar stays equality-only. Native ordering is
      // false/true and is qualified independently by this exact native profile.
      if (!slot.descending()&&value=="false")||(slot.descending()&&value=="true"){format!("{sql} = {}",if slot.descending(){"FALSE"}else{"TRUE"})}else{"FALSE".into()}
     }else{format!("{sql} {} {native}",if slot.descending(){"<"}else{">"})};
     greater.push(format!("((NOT {nil} AND {strict}) OR {})",if slot.nulls_first(){"FALSE".to_string()}else{nil.clone()}));
    }
   }
  }
  let branches=page.after_branches().iter().map(|b|{let mut terms=b.equal_slots().iter().map(|i|equal[*i].clone()).collect::<Vec<_>>();terms.push(greater[b.strict_slot()].clone());format!("({})",terms.join(" AND "))}).collect::<Vec<_>>();
  Some(format!("({})",branches.join(" OR ")))
 }else{None};
 Ok(Clauses{order,after})
}
