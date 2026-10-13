//! Versioned byte equality; projection-only temporal semantics are unchanged.
use crate::{application_ir as ir,application_model::{ExactScalarType,Shape},application_scope::Scope,application_syntax as ast,error::{Diagnostic,Result},syntax::{Column,LiteralKind}};
fn fail(code:&str,message:&str)->Diagnostic {Diagnostic::new(code,"resolve",message)}
impl Scope<'_> {
    fn binary_field(&mut self,column:&Column)->Result<Option<ir::ExactField>> {
        let (record,scan)=self.record_for_column(column)?;
        let (member,graph)=self.catalog.member_descriptor(record,&column.field)?;
        let descriptor=graph.iter().find(|d|d.identity==member.identity).ok_or_else(||fail("WFT-TYPE","Original binary descriptor required"))?;
        if !matches!(&descriptor.shape,Shape::ExactScalar{scalar_type:ExactScalarType::Binary}) {return Ok(None);}
        if descriptor.availability.as_deref()!=Some("required") {return Err(fail("WFT-PROFILE","Binary equality initially requires a singular required field"));}
        let field=ir::ExactField{scan,identity:member.identity,scalar_type:ExactScalarType::Binary,span:column.span.clone()};
        self.descriptors(graph);Ok(Some(field))
    }
    pub(crate) fn binary_equal(&mut self,left:&Column,right:&ast::Value,on:bool)->Result<Option<ir::Predicate>> {
        if !self.binary_equality {return Ok(None);}
        let Some(left)=self.binary_field(left)? else {return Ok(None)};
        if on && !matches!(right,ast::Value::Field(_)) {return Err(fail("WFT-TYPE","Inner join requires field equality"));}
        let right=match right {
            ast::Value::Field(column)=>ir::ExactValue::Field{field:self.binary_field(column)?.ok_or_else(||fail("WFT-TYPE","Binary equality requires a binary field"))?},
            ast::Value::Literal(literal)=>{
                if !matches!(literal.kind,LiteralKind::String) {return Err(fail("WFT-TYPE","Binary literal requires canonical base64 text"));}
                ir::ExactValue::Literal{value:literal.value.clone(),scalar_type:ExactScalarType::Binary,span:literal.span.clone()}
            },
            ast::Value::Parameter(name)=>{
                if name.quoted {return Err(fail("WFT-PARAMETER","Exact source parameters must be unquoted ASCII names"));}
                let normalized=name.value.to_ascii_lowercase();
                let parameter=self.exact_parameters.get(&normalized).ok_or_else(||fail("WFT-PARAMETER","Typed binary parameter required"))?;
                if parameter.scalar_type!=ExactScalarType::Binary {return Err(fail("WFT-TYPE","Binary parameter family required"));}
                self.used.insert(normalized);self.caps.insert("parameter.named".into());
                ir::ExactValue::Parameter{name:name.value.clone(),value:parameter.value.clone(),scalar_type:ExactScalarType::Binary,span:name.span.clone()}
            },
        };
        self.caps.insert("compare.binary.equal".into());self.caps.insert("equal".into());
        Ok(Some(ir::Predicate::ExactEqual{left,right}))
    }
}
