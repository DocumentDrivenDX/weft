use serde_json::json;
use weft_core::security_literals::{check_literal,validate_field};
fn field(kind:&str)->serde_json::Value{json!({"id":"f","kind":"field","scalarType":kind,"cardinality":"one","nullability":"required"})}
#[test]
fn exact_numeric_tokens_preserve_large_values_and_reject_rounding(){
 let i=field("integer");for token in ["9007199254740993","-9007199254740993","1.00e3","-0","0e999999999999999999999999999999999999"]{assert!(check_literal(&i,&json!({"integerToken":token})).is_ok(),"{token}");}
 for token in ["1.5","01","1e-1","1e4000001","1e+","1.","1E2E3"]{assert!(check_literal(&i,&json!({"integerToken":token})).is_err(),"{token}");}
 let mut d=field("decimal");d["facets"]=json!({"precision":4,"scale":2});for token in ["1.2300","1.23e0","-99.99"]{assert!(check_literal(&d,&json!({"decimalToken":token})).is_ok());}for token in ["1.234","100","0.001"]{assert!(check_literal(&d,&json!({"decimalToken":token})).is_err());}
}
#[test]
fn signed_unsigned_and_exclusive_extremes_are_exact(){
 let mut f=field("integer");f["facets"]=json!({"integerWidth":{"bits":8,"signed":true}});for n in ["-128","127"]{assert!(check_literal(&f,&json!({"integerToken":n})).is_ok());}for n in ["-129","128"]{assert!(check_literal(&f,&json!({"integerToken":n})).is_err());}
 f["facets"]["range"]=json!({"min":{"integerToken":"127"},"minInclusive":false});assert!(validate_field(&f).is_err());
 f["facets"]=json!({"integerWidth":{"bits":8,"signed":false}});assert!(check_literal(&f,&json!({"integerToken":"255"})).is_ok());assert!(check_literal(&f,&json!({"integerToken":"-1"})).is_err());
 f["facets"]["range"]=json!({"max":{"integerToken":"0"},"maxInclusive":false});assert!(validate_field(&f).is_err());
}
#[test]
fn length_binary_enums_defaults_and_examples_have_separate_meaning(){
 let mut s=field("string");s["facets"]=json!({"length":{"min":1,"max":1,"unit":"unicode-scalar"}});assert!(check_literal(&s,&json!({"string":"😀"})).is_ok());assert!(check_literal(&s,&json!({"string":"é"})).is_err());
 let mut b=field("binary");b["allowedValues"]=json!([{"binaryHex":"aAFF"}]);assert!(check_literal(&b,&json!({"binaryHex":"AAff"})).is_ok());assert!(check_literal(&b,&json!({"binaryHex":"aG"})).is_err());
 let mut i=field("integer");i["allowedValues"]=json!([{"integerToken":"1"},{"integerToken":"1.0e0"}]);assert!(validate_field(&i).is_err());i["allowedValues"]=json!([{"integerToken":"1"}]);i["examples"]=json!([{"integerToken":"2"}]);assert!(validate_field(&i).is_ok());i["default"]=json!({"on":"missing","value":{"integerToken":"2"}});assert!(validate_field(&i).is_err());
}
