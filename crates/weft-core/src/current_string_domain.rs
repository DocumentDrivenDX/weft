//! Exact bounded Unicode scalar domains for the explicitly selected new candidate.
//! Pure type/value checks confer no permission or native correspondence.
use crate::{error::{Diagnostic, Result}, ir::{Family, LogicalType}};
use serde_json::{json, Value};

const MAX_BOUND: u64 = 9_007_199_254_740_991;
fn fail() -> Diagnostic {
    Diagnostic::new("WFT-CURRENT-STRING-DOMAIN", "capability", "Current string domain refused")
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Length { min: u64, max: u64 }
impl Length {
    pub fn min(&self) -> u64 { self.min }
    pub fn max(&self) -> u64 { self.max }
}
// Interpret the JSON number exactly, without f64 or exponent-sized allocation.
fn bound(value: &Value) -> Result<u64> {
    let number = value.as_number().ok_or_else(fail)?;
    if let Some(n) = number.as_u64() { return if n <= MAX_BOUND { Ok(n) } else { Err(fail()) }; }
    let token = number.to_string();
    let (mantissa, exponent) = token.split_once(['e','E']).unwrap_or((&token,"0"));
    let negative = mantissa.starts_with('-');
    let mantissa = mantissa.strip_prefix('-').unwrap_or(mantissa);
    let fraction = mantissa.split_once('.').map_or(0, |(_,f)| f.len());
    let digits: Vec<u8> = mantissa.bytes().filter(|b| *b != b'.').collect();
    let Some(first) = digits.iter().position(|b| *b != b'0') else { return Ok(0); };
    if negative { return Err(fail()); }
    let digits = &digits[first..];
    let shift = exponent.parse::<i64>().map_err(|_| fail())?
        .checked_sub(i64::try_from(fraction).map_err(|_|fail())?).ok_or_else(fail)?;
    let (significant, zeros) = if shift < 0 {
        let removed = usize::try_from(shift.checked_neg().ok_or_else(fail)?).map_err(|_|fail())?;
        if removed >= digits.len() || digits[digits.len()-removed..].iter().any(|b| *b != b'0') { return Err(fail()); }
        (&digits[..digits.len()-removed], 0)
    } else {
        (digits, usize::try_from(shift).map_err(|_|fail())?)
    };
    if significant.len().checked_add(zeros).ok_or_else(fail)? > 16 { return Err(fail()); }
    let mut n = 0u64;
    for digit in significant { n = n.checked_mul(10).and_then(|v| v.checked_add((digit-b'0') as u64)).ok_or_else(fail)?; }
    for _ in 0..zeros { n = n.checked_mul(10).ok_or_else(fail)?; }
    if n > MAX_BOUND { return Err(fail()); }
    Ok(n)
}
/// Empty facets mean the existing unconstrained String, not a guessed bound.
pub fn bounds(facets: &Value) -> Result<Option<Length>> {
    let object = facets.as_object().ok_or_else(fail)?;
    if object.is_empty() { return Ok(None); }
    if object.len() != 1 { return Err(fail()); }
    let length = object.get("length").and_then(Value::as_object).ok_or_else(fail)?;
    if length.len() != 3 || length.get("unit").and_then(Value::as_str) != Some("unicode-scalar") {
        return Err(fail());
    }
    let min = bound(length.get("min").ok_or_else(fail)?)?;
    let max = bound(length.get("max").ok_or_else(fail)?)?;
    if min > max || max > MAX_BOUND { return Err(fail()); }
    Ok(Some(Length { min, max }))
}
/// The original profiles always pass false; no old profile is widened by this helper.
pub fn supported(t: &LogicalType, bounded: bool) -> bool {
    !t.nullable && match t.family {
        Family::String => t.facets == json!({}) || (bounded && bounds(&t.facets).is_ok()),
        Family::Integer => t.facets == json!({"integerWidth":{"bits":64,"signed":true}}),
        _ => false,
    }
}
pub fn validate(t: &LogicalType, value: &str, bounded: bool) -> Result<()> {
    if t.family != Family::String || !supported(t, bounded) { return Err(fail()); }
    validate_facets(&t.facets, value)
}
/// Literal admission checks the actual non-null value even for a nullable Field.
pub fn validate_facets(facets: &Value, value: &str) -> Result<()> {
    if let Some(length) = bounds(facets)? {
        // Work depends on the actual framed value, never on a declared maximum.
        let count = value.chars().count() as u64;
        if count < length.min || count > length.max { return Err(fail()); }
    }
    Ok(())
}
/// Semantically identical bounded domains can retain different exact JSON spellings.
/// Identity and comparison-scope checks remain separate mandatory obligations.
pub fn same_domain(left: &LogicalType, right: &LogicalType, bounded: bool) -> bool {
    supported(left, bounded) && supported(right, bounded)
        && (left == right || (bounded && left.family == Family::String
            && right.family == Family::String && bounds(&left.facets).ok() == bounds(&right.facets).ok()))
}
/// A literal keeps its own original domain/value. The other operand's bounds
/// constrain that operand, not this value. Field/field pairing is a separate rule.
pub fn literal_compatible(original: &LogicalType, effective: &LogicalType, value: &str, bounded: bool) -> bool {
    original.family == Family::String && effective.family == Family::String
        && supported(effective, bounded) && validate(original, value, bounded).is_ok()
        && (original == effective || bounded)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn t(min:u64,max:u64)->LogicalType {
        LogicalType {family:Family::String,nullable:false,facets:json!({"length":{"min":min,"max":max,"unit":"unicode-scalar"}})}
    }
    #[test]
    fn exact_domains_and_safe_bounds_refuse_other_meanings() {
        assert_eq!(bounds(&json!({})).unwrap(),None);
        assert_eq!(bounds(&t(0,MAX_BOUND).facets).unwrap().unwrap().max(),MAX_BOUND);
        assert_eq!(bounds(&t(2,3).facets).unwrap().unwrap().min(),2);
        for facets in [json!(null),json!([]),json!({"length":null}),json!({"other":{}}),
            json!({"length":{"min":0,"max":1,"unit":"unicode-scalar"},"extra":true}),
            json!({"length":{"min":0,"max":1,"unit":"unicode-scalar","extra":0}}),
            json!({"length":{"max":1,"unit":"unicode-scalar"}}),
            json!({"length":{"min":0,"unit":"unicode-scalar"}}),
            json!({"length":{"min":0,"max":1}}),
            json!({"length":{"min":-1,"max":1,"unit":"unicode-scalar"}}),
            json!({"length":{"min":true,"max":1,"unit":"unicode-scalar"}}),
            json!({"length":{"min":0.1,"max":1,"unit":"unicode-scalar"}}),
            json!({"length":{"min":0,"max":1.5,"unit":"unicode-scalar"}}),
            t(2,1).facets,t(0,MAX_BOUND+1).facets] {
            assert!(bounds(&facets).is_err(),"{facets}");
        }
        for unit in ["byte","utf16","unicode-code-unit","grapheme","Unicode-scalar",""] {
            let mut f=t(0,3).facets;f["length"]["unit"]=json!(unit);assert!(bounds(&f).is_err());
        }
    }
    #[test]
    fn mathematical_json_integer_bounds_keep_original_representation() {
        for (text, expected) in [("0.0",0),("-0",0),("-0.0e999999999999999999999999",0),("1.0",1),("1e0",1),("1.5e1",15),("9007199254740991.0000000",MAX_BOUND),("9.007199254740991e15",MAX_BOUND)] {
            let value:Value=serde_json::from_str(text).unwrap();let original=value.clone();
            assert_eq!(bound(&value).unwrap(),expected);assert_eq!(value,original);
        }
        for text in ["0.1","1e-1","-1.0","9007199254740992.0","9007199254740991.0000001","1e999999999999999999999999","1e-999999999999999999999999"] {
            assert!(bound(&serde_json::from_str::<Value>(text).unwrap()).is_err(),"{text}");
        }
    }
    #[test]
    fn independent_scalar_count_vectors_detect_bytes_units_and_normalization() {
        // Expected lengths are authored, not obtained from the implementation.
        for (value,count) in [("",0),("A",1),("😀",1),("é",1),("e\u{301}",2),("😀e\u{301}",3),("\r\n",2),("\0",1)] {
            assert!(validate(&t(count,count),value,true).is_ok());
            if count>0 {assert!(validate(&t(0,count-1),value,true).is_err());}
            assert!(validate(&t(count+1,MAX_BOUND),value,true).is_err());
        }
        assert_eq!("😀".len(),4);assert_eq!("😀".encode_utf16().count(),2);
        assert!(validate(&t(1,1),"😀",true).is_ok());
        assert!(validate(&t(1,1),"e\u{301}",true).is_err());
        assert_ne!("é","e\u{301}");
        assert!(serde_json::from_str::<String>(r#""\ud800""#).is_err());
    }
    #[test]
    fn literal_provenance_and_distinct_effective_bounds_are_retained() {
        let original=t(1,3);let effective=t(10,64);let before=original.facets.clone();
        assert!(literal_compatible(&original,&effective,"é",true));
        assert!(!literal_compatible(&original,&effective,"four",true));
        assert!(!literal_compatible(&original,&effective,"",true));
        assert_eq!(original.facets,before);
        let equivalent=LogicalType{family:Family::String,nullable:false,facets:serde_json::from_str(r#"{"length":{"min":1.0,"max":3e0,"unit":"unicode-scalar"}}"#).unwrap()};
        assert!(same_domain(&original,&equivalent,true));
        assert!(!same_domain(&original,&effective,true));
        assert!(!same_domain(&original,&equivalent,false));
        let mut other=effective.clone();other.family=Family::Integer;
        assert!(!literal_compatible(&original,&other,"1",true));
        other=effective.clone();other.nullable=true;
        assert!(!literal_compatible(&original,&other,"é",true));
    }
    #[test]
    fn old_profiles_and_nullable_domains_remain_closed() {
        assert!(!supported(&t(0,3),false));assert!(validate(&t(0,3),"A",false).is_err());
        assert!(!literal_compatible(&t(0,3),&t(0,3),"A",false));
        let mut nullable=t(0,3);nullable.nullable=true;assert!(!supported(&nullable,true));
        let unbounded=LogicalType{family:Family::String,nullable:false,facets:json!({})};
        assert!(supported(&unbounded,false));assert!(validate(&unbounded,"😀",false).is_ok());
        assert!(literal_compatible(&unbounded,&unbounded,"😀",false));
    }
}
