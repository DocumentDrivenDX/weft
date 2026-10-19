// SPDX-License-Identifier: MIT OR Apache-2.0
//! Versioned byte equality; projection-only temporal semantics are unchanged.
use crate::{application_ir as ir,application_model::{ExactScalarType,Shape},application_scope::Scope,application_syntax as ast,error::{Diagnostic,Result},syntax::{Column,LiteralKind}};
fn fail(code:&str,message:&str)->Diagnostic {Diagnostic::new(code,"resolve",message)}
fn equality_capability(kind:&ExactScalarType,optional:bool)->&'static str {
    match (kind,optional) {
        (ExactScalarType::Binary,false)=>"compare.binary.equal",
        (ExactScalarType::Binary,true)=>"compare.binary.optionalEqual",
        (ExactScalarType::Timestamp,false)=>"compare.timestamp.instantEqual",
        (ExactScalarType::Timestamp,true)=>"compare.timestamp.optionalInstantEqual",
        (ExactScalarType::Date,false)=>"compare.date.equal",
        (ExactScalarType::Date,true)=>"compare.date.optionalEqual",
    }
}
impl Scope<'_> {
    fn exact_comparison_field(&mut self,column:&Column)->Result<Option<ir::ExactField>> {
        let (record,scan)=self.record_for_column(column)?;
        let (member,graph)=self.catalog.member_descriptor(record,&column.field)?;
        let descriptor=graph.iter().find(|d|d.identity==member.identity).ok_or_else(||fail("WFT-TYPE","Original binary descriptor required"))?;
        let scalar_type=match &descriptor.shape {
            Shape::ExactScalar{scalar_type:ExactScalarType::Binary} if self.binary_equality=>ExactScalarType::Binary,
            Shape::ExactScalar{scalar_type:ExactScalarType::Timestamp} if self.timestamp_equality=>ExactScalarType::Timestamp,
            Shape::ExactScalar{scalar_type:ExactScalarType::Date} if self.catalog.exact_dates=>ExactScalarType::Date,
            _=>return Ok(None),
        };
        match descriptor.availability.as_deref() {
            Some("required") => {},
            Some("absent-allowed") => {self.caps.insert(equality_capability(&scalar_type,true).into());},
            _ => return Err(fail("WFT-PROFILE","Binary equality requires a singular required or absent-allowed field")),
        }
        let field=ir::ExactField{scan,identity:member.identity,scalar_type,span:column.span.clone()};
        self.descriptors(graph);Ok(Some(field))
    }
    pub(crate) fn exact_equal(&mut self,left:&Column,right:&ast::Value,on:bool)->Result<Option<ir::Predicate>> {
        if !self.binary_equality {return Ok(None);}
        let Some(left)=self.exact_comparison_field(left)? else {return Ok(None)};
        if on && !matches!(right,ast::Value::Field(_)) {return Err(fail("WFT-TYPE","Inner join requires field equality"));}
        let scalar_type=left.scalar_type.clone();
        let right=match right {
            ast::Value::Field(column)=>{
                let field=self.exact_comparison_field(column)?.ok_or_else(||fail("WFT-TYPE","Exact equality requires an admitted field"))?;
                if field.scalar_type!=scalar_type {return Err(fail("WFT-TYPE","Exact equality operand families must match"));}
                ir::ExactValue::Field{field}
            },
            ast::Value::Literal(literal)=>{
                if !matches!(literal.kind,LiteralKind::String) {return Err(fail("WFT-TYPE","Binary literal requires canonical base64 text"));}
                ir::ExactValue::Literal{value:literal.value.clone(),scalar_type:scalar_type.clone(),span:literal.span.clone()}
            },
            ast::Value::Parameter(name)=>{
                if name.quoted {return Err(fail("WFT-PARAMETER","Exact source parameters must be unquoted ASCII names"));}
                let normalized=name.value.to_ascii_lowercase();
                let parameter=self.exact_parameters.get(&normalized).ok_or_else(||fail("WFT-PARAMETER","Typed binary parameter required"))?;
                if parameter.scalar_type!=scalar_type {return Err(fail("WFT-TYPE","Binary parameter family required"));}
                self.used.insert(normalized);self.caps.insert("parameter.named".into());
                ir::ExactValue::Parameter{name:name.value.clone(),value:parameter.value.clone(),scalar_type:scalar_type.clone(),span:name.span.clone()}
            },
        };
        match &right {
            ir::ExactValue::Literal{value,..}|ir::ExactValue::Parameter{value,..}=>{
                match scalar_type {ExactScalarType::Timestamp=>{crate::exact_instant::key(value)?;},ExactScalarType::Binary=>crate::exact::validate_binary_token(value)?,ExactScalarType::Date=>crate::exact::validate_date_token(value)?}
            },_=>{},
        }
        self.caps.insert(equality_capability(&scalar_type,false).into());self.caps.insert("equal".into());
        Ok(Some(ir::Predicate::ExactEqual{left,right}))
    }
}
