// SPDX-License-Identifier: MIT OR Apache-2.0
//! UMF CoreLiteral admission against a selected Field's value domain: exact
//! scalar literals, facets, ranges, allowed values, examples and defaults.
//! Diagnostic codes `WFT-SECURITY-LITERAL` and `WFT-SECURITY-ONTOLOGY` are the
//! historical codes of weft's security subset, where this module started; they
//! are kept so existing reports do not change. No native authorization.
use crate::diagnostic::{Diagnostic,Result};
use num_bigint::{BigInt,Sign};
use serde_json::{json,Value};
use std::collections::BTreeSet;
pub const MAX_TEXT:usize=4_000_000;
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-LITERAL","model","Security literal or refinement refused")}
fn ontology_fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-ONTOLOGY","model","Security ontology closure refused")}
fn ontology_members(v:&Value,known:&[&str])->Result<()>{let object=v.as_object().ok_or_else(ontology_fail)?;if object.keys().any(|k|!known.contains(&k.as_str())){return Err(ontology_fail());}Ok(())}
#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub enum ScalarLiteral{Boolean(bool),String(String),Binary(String),Number(BigInt)}
/// A validated scalar Field with its authored allowed-value domain normalized once.
/// Reusing this avoids reparsing an allowed-value list for every invocation value.
#[derive(Debug,Clone)]
pub struct PreparedLiteral {
 base: Value,
 allowed: Option<BTreeSet<ScalarLiteral>>,
}
impl PreparedLiteral {
 pub fn new(field:&Value)->Result<Self>{
  validate_field(field)?;
  Self::from_validated(field)
 }
 pub fn from_validated(field:&Value)->Result<Self>{
  let base=json!({
   "kind":field["kind"],
   "cardinality":field["cardinality"],
   "nullability":field["nullability"],
   "scalarType":field["scalarType"],
   "facets":normalized_facets(field)?,
  });
  let authored=field.get("allowedValues").cloned();
  let allowed=if let Some(values)=authored {
   let values=values.as_array().ok_or_else(fail)?;
   let mut identities=BTreeSet::new();
   for value in values {
    let identity=scalar(&base,value,true)?.ok_or_else(fail)?;
    if !identities.insert(identity){return Err(fail());}
   }
   Some(identities)
  } else {None};
  Ok(Self{base,allowed})
 }
 pub fn check(&self,value:&Value)->Result<()>{
  if let Some(identity)=scalar(&self.base,value,true)? {
   if self.allowed.as_ref().is_some_and(|allowed|!allowed.contains(&identity)){return Err(fail());}
  }
  Ok(())
 }

 /// Export the already validated, normalized operands needed by the closed
 /// generated query codec VM. This does not emit source text or attest to a
 /// trusted build; it serializes only the constraints this owner has already
 /// admitted and compiled.
 pub fn generated_query_refinements(&self)->Result<Value>{
  let facets=&self.base["facets"];
  let range=if let Some(range)=facets.get("range"){
   let min=range.get("min").map(|value|numeric(&self.base,value).map(|number|number.to_str_radix(10))).transpose()?;
   let max=range.get("max").map(|value|numeric(&self.base,value).map(|number|number.to_str_radix(10))).transpose()?;
   json!({
    "minCoefficient":min,
    "minInclusive":range.get("minInclusive").and_then(Value::as_bool).unwrap_or(true),
    "maxCoefficient":max,
    "maxInclusive":range.get("maxInclusive").and_then(Value::as_bool).unwrap_or(true),
   })
  }else{Value::Null};
  let allowed=self.allowed.as_ref().map(|values|values.iter().map(|value|match value{
   ScalarLiteral::Boolean(value)=>json!({"kind":"boolean","value":value}),
   ScalarLiteral::String(value)=>json!({"kind":"string","value":value}),
   ScalarLiteral::Binary(value)=>json!({"kind":"binaryHex","value":value}),
   ScalarLiteral::Number(value)=>json!({"kind":"numberCoefficient","value":value.to_str_radix(10)}),
  }).collect::<Vec<_>>());
  Ok(json!({
   "length":facets.get("length").cloned().unwrap_or(Value::Null),
   "integerWidth":facets.get("integerWidth").cloned().unwrap_or(Value::Null),
   "range":range,
   "allowedIdentities":allowed,
  }))
 }
}
/// Exact decimal coefficient ([`crate::exact::coefficient`]) with this module's diagnostic.
pub fn coefficient(token:&str,scale:u64,precision:Option<u64>)->Result<BigInt>{
 crate::exact::coefficient(token,scale,precision).ok_or_else(fail)
}
pub fn unsigned(v:&Value)->Result<u64>{
 if !v.is_number(){return Err(fail());}let text=v.to_string();let n=coefficient(&text,0,None)?;
 if n.sign()==Sign::Minus||text.starts_with('-')&&n==BigInt::from(0){return Err(fail());}
 let (_,words)=n.to_u64_digits();if words.len()>1{return Err(fail());}let value=words.first().copied().unwrap_or(0);if value>9_007_199_254_740_991{return Err(fail());}Ok(value)
}
pub fn normalized_facets(field:&Value)->Result<Value>{
 let mut facets=field.get("facets").cloned().unwrap_or(json!({}));
 for key in ["precision","scale"]{if let Some(value)=facets.get(key){let n=unsigned(value)?;facets[key]=json!(n);}}
 for (group,keys) in [("length",&["min","max"][..]),("integerWidth",&["bits"][..])]{for key in keys{if let Some(value)=facets.get(group).and_then(|g|g.get(*key)){let n=unsigned(value)?;facets[group][*key]=json!(n);}}}
 Ok(facets)
}
fn numeric(field:&Value,value:&Value)->Result<BigInt>{
 let facets=normalized_facets(field)?;let kind=field["scalarType"].as_str().ok_or_else(fail)?;
 let wrapper=if kind=="integer"{"integerToken"}else if kind=="decimal"{"decimalToken"}else{return Err(fail());};
 let scale=if kind=="decimal"{unsigned(&facets["scale"])?}else{0};let precision=if kind=="decimal"{Some(unsigned(&facets["precision"])?)}else{None};
 coefficient(value[wrapper].as_str().ok_or_else(fail)?,scale,precision)
}
fn power_of_two(n:&BigInt)->bool{n.magnitude().to_u64_digits().iter().map(|n|n.count_ones() as u64).sum::<u64>()==1}
fn fits_width(n:&BigInt,width:&Value)->Result<()>{
 let bits=unsigned(&width["bits"])?;if bits==0{return Err(fail());}let magnitude=n.magnitude().bits();let negative=n.sign()==Sign::Minus;
 let signed=width["signed"].as_bool().ok_or_else(fail)?;
 if if signed{if negative{magnitude<bits||magnitude==bits&&power_of_two(n)}else{magnitude<bits}}else{!negative&&magnitude<=bits}{Ok(())}else{Err(fail())}
}
fn scalar(field:&Value,value:&Value,refinements:bool)->Result<Option<ScalarLiteral>>{
 if field["kind"]!="field"||field["cardinality"]!="one"{return Err(fail());}
 if value.is_null(){return if field["nullability"]=="absent-allowed"{Ok(None)}else{Err(fail())};}
 if value.as_object().is_none_or(|m|m.len()!=1){return Err(fail());}
 let facets=normalized_facets(field)?;let kind=field["scalarType"].as_str().ok_or_else(fail)?;
 let (identity,length)=match kind{
  "boolean"=>(ScalarLiteral::Boolean(value["boolean"].as_bool().ok_or_else(fail)?),None),
  "string"=>{let text=value["string"].as_str().ok_or_else(fail)?;(ScalarLiteral::String(text.into()),Some(text.chars().count() as u64))},
  "binary"=>{let text=value["binaryHex"].as_str().ok_or_else(fail)?;if text.len()%2!=0||!text.bytes().all(|b|b.is_ascii_hexdigit()){return Err(fail());}(ScalarLiteral::Binary(text.to_ascii_lowercase()),Some(text.len() as u64/2))},
  "integer"|"decimal"=>{let n=numeric(field,value)?;if let Some(width)=facets.get("integerWidth"){fits_width(&n,width)?;}
   if refinements{if let Some(range)=facets.get("range"){for (end,lower) in [("min",true),("max",false)]{if let Some(bound)=range.get(end){let b=numeric(field,bound)?;if if lower{n<b||n==b&&range["minInclusive"]==false}else{n>b||n==b&&range["maxInclusive"]==false}{return Err(fail());}}}}}
   (ScalarLiteral::Number(n),None)
  },_=>return Err(fail())
 };
 if let Some(bounds)=facets.get("length"){
  let length=length.ok_or_else(fail)?;let expected=if kind=="string"{"unicode-scalar"}else{"byte"};if bounds["unit"]!=expected{return Err(fail());}
  if bounds.get("min").is_some_and(|v|unsigned(v).is_err()||length<unsigned(v).unwrap())||bounds.get("max").is_some_and(|v|unsigned(v).is_err()||length>unsigned(v).unwrap()){return Err(fail());}
 }
 if refinements{if let Some(allowed)=field["allowedValues"].as_array(){let mut matched=false;for candidate in allowed{if scalar(field,candidate,false)?.as_ref()==Some(&identity){matched=true;break;}}if !matched{return Err(fail());}}}
 Ok(Some(identity))
}
pub fn check_literal(field:&Value,value:&Value)->Result<()>{field_domain(field)?;scalar(field,value,true).map(|_|())}
fn at_extreme(field:&Value,n:&BigInt,upper:bool)->Result<bool>{
 let f=normalized_facets(field)?;if field["scalarType"]=="integer"{
  let Some(w)=f.get("integerWidth") else{return Ok(false);};let bits=unsigned(&w["bits"])?;let signed=w["signed"].as_bool().ok_or_else(fail)?;
  if !upper{return Ok(if signed{n.sign()==Sign::Minus&&n.magnitude().bits()==bits&&power_of_two(n)}else{*n==BigInt::from(0)});}
  let expected=if signed{bits-1}else{bits};let ones=n.magnitude().to_u64_digits().iter().map(|w|w.count_ones() as u64).sum::<u64>();
  return Ok(n.sign()!=Sign::Minus&&n.magnitude().bits()==expected&&ones==expected);
 }
 let precision=unsigned(&f["precision"])?;let text=n.to_str_radix(10);let digits=text.trim_start_matches('-');
 Ok((if upper{n.sign()!=Sign::Minus}else{n.sign()==Sign::Minus})&&digits.len() as u64==precision&&digits.bytes().all(|b|b==b'9'))
}
pub fn validate_field(field:&Value)->Result<()>{
 field_domain(field)?;
 let facets=normalized_facets(field)?;
 if let Some(length)=facets.get("length"){if length.get("min").is_some()&&length.get("max").is_some()&&unsigned(&length["min"])? > unsigned(&length["max"])?{return Err(fail());}}
 if let Some(range)=facets.get("range"){
  let mut base=field.clone();base["facets"].as_object_mut().ok_or_else(fail)?.remove("range");base.as_object_mut().unwrap().remove("allowedValues");
  for (end,flag) in [("min","minInclusive"),("max","maxInclusive")]{if let Some(bound)=range.get(end){if bound.is_null(){return Err(fail());}scalar(&base,bound,false)?;}else if range.get(flag).is_some(){return Err(fail());}}
  if range.get("min").is_some()&&range["minInclusive"]==false&&at_extreme(field,&numeric(field,&range["min"])?,true)?{return Err(fail());}
  if range.get("max").is_some()&&range["maxInclusive"]==false&&at_extreme(field,&numeric(field,&range["max"])?,false)?{return Err(fail());}
  if range.get("min").is_some()&&range.get("max").is_some(){let min=numeric(field,&range["min"])?;let max=numeric(field,&range["max"])?;let low=min+BigInt::from(if range["minInclusive"]==false{1}else{0});let high=max-BigInt::from(if range["maxInclusive"]==false{1}else{0});if low>high{return Err(fail());}}
 }
 if let Some(allowed)=field["allowedValues"].as_array(){let mut base=field.clone();base.as_object_mut().unwrap().remove("allowedValues");let mut unique=BTreeSet::new();for value in allowed{let identity=scalar(&base,value,true)?.ok_or_else(fail)?;if !unique.insert(identity){return Err(fail());}}}
 if let Some(examples)=field["examples"].as_array(){for value in examples{scalar(field,value,false)?;}}
 if let Some(default)=field.get("default"){if default.as_object().is_none_or(|m|m.keys().any(|k|k!="on"&&k!="value"))||!matches!(default["on"].as_str(),Some("missing"|"null"|"missing-or-null")){return Err(fail());}check_literal(field,&default["value"])?;}
 Ok(())
}

pub fn normalized_literal(field:&Value,value:&Value)->Result<Option<ScalarLiteral>>{validate_field(field)?;scalar(field,value,true)}

/// Normalize a selected scalar Field's UMF value domain (scalar type, cardinality,
/// nullability, facets, allowed values), or refuse it.
pub fn field_domain(field:&Value)->Result<Value>{
 ontology_members(field,&["id","kind","name","description","extensions","scalarType","nullability","cardinality","facets","references","title","aliases","examples","allowedValues","default"])?;
 if field["kind"]!="field"||field["cardinality"]!="one"||!matches!(field["scalarType"].as_str(),Some("boolean"|"string"|"integer"|"decimal"|"binary"))||field["references"].as_array().is_some_and(|r|!r.is_empty()){return Err(ontology_fail());}
 let facets=normalized_facets(field)?;
 ontology_members(&facets,&["length","precision","scale","integerWidth","range","collectionSize"])?;
 for (group,keys) in [("length",&["min","max","unit"][..]),("integerWidth",&["bits","signed"][..]),("range",&["min","max","minInclusive","maxInclusive"][..]),("collectionSize",&["min","max"][..])]{if let Some(value)=facets.get(group){ontology_members(value,keys)?;}}
 if !matches!(field["nullability"].as_str(),Some("required"|"absent-allowed"|"unspecified")){return Err(ontology_fail());}
 let kind=field["scalarType"].as_str().unwrap();
 if facets.get("collectionSize").is_some()||((facets.get("precision").is_some()||facets.get("scale").is_some())&&kind!="decimal")||(facets.get("integerWidth").is_some()&&kind!="integer")||(facets.get("range").is_some()&&!matches!(kind,"integer"|"decimal")){return Err(ontology_fail());}
 if let Some(length)=facets.get("length"){
  if length["unit"]!=if kind=="string"{"unicode-scalar"}else if kind=="binary"{"byte"}else{return Err(ontology_fail());}{return Err(ontology_fail());}
 }
 if let Some(width)=facets.get("integerWidth"){
  let bits=width["bits"].as_u64().ok_or_else(ontology_fail)?;if bits==0||bits>9_007_199_254_740_991||!width["signed"].is_boolean(){return Err(ontology_fail());}
 }
 if field["scalarType"]=="decimal"{
  let precision=facets["precision"].as_u64().ok_or_else(ontology_fail)?;let scale=facets["scale"].as_u64().ok_or_else(ontology_fail)?;
  if precision==0||precision>9_007_199_254_740_991||scale>precision{return Err(ontology_fail());}
 }
 Ok(json!({"scalarType":field["scalarType"],"cardinality":field["cardinality"],"nullability":field["nullability"],"facets":facets,"allowedValues":field.get("allowedValues").unwrap_or(&Value::Null)}))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn field(scalar: &str, facets: Value) -> Value {
        json!({"id":"f","kind":"field","scalarType":scalar,"cardinality":"one","nullability":"required","facets":facets})
    }
    #[test]
    fn literals_are_checked_against_the_field_domain() {
        let small = field("integer", json!({"integerWidth":{"bits":8,"signed":true},"range":{"min":{"integerToken":"-5"},"max":{"integerToken":"5"},"maxInclusive":false}}));
        validate_field(&small).unwrap();
        check_literal(&small, &json!({"integerToken":"-5"})).unwrap();
        for bad in [json!({"integerToken":"5"}), json!({"integerToken":"1.5"}), json!({"string":"1"}), Value::Null] {
            assert_eq!(check_literal(&small, &bad).unwrap_err().code, "WFT-SECURITY-LITERAL", "{bad}");
        }
        let money = field("decimal", json!({"precision":4,"scale":2}));
        check_literal(&money, &json!({"decimalToken":"99.99"})).unwrap();
        assert!(check_literal(&money, &json!({"decimalToken":"100.00"})).is_err());
        let mut tagged = field("string", json!({"length":{"max":3,"unit":"unicode-scalar"}}));
        tagged["allowedValues"] = json!([{"string":"a"},{"string":"bc"}]);
        let prepared = PreparedLiteral::new(&tagged).unwrap();
        prepared.check(&json!({"string":"bc"})).unwrap();
        assert!(prepared.check(&json!({"string":"x"})).is_err());
        assert_eq!(field_domain(&field("float", json!({}))).unwrap_err().code, "WFT-SECURITY-ONTOLOGY");
    }
}
