//! Exact scalar literal/refinement checks for the shared security subset. No native authorization.
use crate::error::{Diagnostic,Result};
use num_bigint::{BigInt,Sign};
use serde_json::{json,Value};
use std::collections::BTreeSet;
const MAX_TEXT:usize=4_000_000;
fn fail()->Diagnostic{Diagnostic::new("WFT-SECURITY-LITERAL","model","Security literal or refinement refused")}
#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub enum ScalarLiteral{Boolean(bool),String(String),Binary(String),Number(BigInt)}
fn coefficient(token:&str,scale:u64,precision:Option<u64>)->Result<BigInt>{
 if token.len()>MAX_TEXT{return Err(fail());}
 let (negative,body)=if let Some(v)=token.strip_prefix('-'){(true,v)}else{(false,token)};
 let at=body.find(['e','E']);let (mantissa,exponent)=if let Some(at)=at{(&body[..at],&body[at+1..])}else{(body,"0")};
 let exponent_digits=exponent.strip_prefix(['+','-']).unwrap_or(exponent);
 if exponent_digits.is_empty()||!exponent_digits.bytes().all(|b|b.is_ascii_digit()){return Err(fail());}
 let (whole,fraction)=if let Some((whole,fraction))=mantissa.split_once('.'){
  if fraction.is_empty()||!fraction.bytes().all(|b|b.is_ascii_digit()){return Err(fail());}(whole,fraction)
 }else{(mantissa,"")};
 if whole.is_empty()||!whole.bytes().all(|b|b.is_ascii_digit())||(whole.len()>1&&whole.starts_with('0')){return Err(fail());}
 let joined=format!("{whole}{fraction}");let mut digits=joined.trim_start_matches('0').to_string();
 if digits.is_empty(){return Ok(BigInt::from(0));}
 if exponent.len()>32{return Err(fail());}let exponent: i128=exponent.parse().map_err(|_|fail())?;
 let shift=exponent-fraction.len() as i128+scale as i128;
 if shift<0{let cut=-shift;if cut>digits.len() as i128{return Err(fail());}let at=digits.len()-cut as usize;if !digits[at..].bytes().all(|b|b==b'0'){return Err(fail());}digits.truncate(at);}
 else{let bound=precision.unwrap_or(MAX_TEXT as u64).min(MAX_TEXT as u64);if digits.len() as i128+shift>bound as i128{return Err(fail());}digits.extend(std::iter::repeat_n('0',shift as usize));}
 if precision.is_some_and(|p|digits.len() as u64>p){return Err(fail());}
 if digits.is_empty(){return Ok(BigInt::from(0));}
 if negative{digits.insert(0,'-');}BigInt::parse_bytes(digits.as_bytes(),10).ok_or_else(fail)
}
pub(crate) fn unsigned(v:&Value)->Result<u64>{
 if !v.is_number(){return Err(fail());}let text=v.to_string();let n=coefficient(&text,0,None)?;
 if n.sign()==Sign::Minus||text.starts_with('-')&&n==BigInt::from(0){return Err(fail());}
 let (_,words)=n.to_u64_digits();if words.len()>1{return Err(fail());}let value=words.first().copied().unwrap_or(0);if value>9_007_199_254_740_991{return Err(fail());}Ok(value)
}
pub(crate) fn normalized_facets(field:&Value)->Result<Value>{
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
pub fn check_literal(field:&Value,value:&Value)->Result<()>{crate::security_ontology::domain(field)?;scalar(field,value,true).map(|_|())}
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
 crate::security_ontology::domain(field)?;
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

pub(crate) fn normalized_literal(field:&Value,value:&Value)->Result<Option<ScalarLiteral>>{validate_field(field)?;scalar(field,value,true)}
