//! Test-only generated SQL witness. Not a Truss mapping or execution API.
use serde_json::json;
use weft_core::ir::{Family, LogicalType};
use weft_postgresql::{CatalogDomain, Identifier, Parameters};
fn main() {
    let mut parameters = Parameters::default();
    let type_id = parameters
        .catalog(CatalogDomain::Int, "-2147483648", json!({"catalog":"type"}))
        .unwrap();
    let key_num = parameters
        .catalog(CatalogDomain::SmallInt, "-32768", json!({"catalog":"key"}))
        .unwrap();
    let exact = parameters
        .push(
            LogicalType {
                family: Family::Integer,
                facets: json!({"integerWidth":{"bits":64,"signed":false}}),
                nullable: false,
            },
            "18446744073709551615".into(),
            json!({"literal":"unsigned-boundary"}),
        )
        .unwrap();
    let text = parameters
        .push(
            LogicalType {
                family: Family::String,
                facets: json!({}),
                nullable: false,
            },
            "a'; DROP SCHEMA public; --\\ 😀 ".into(),
            json!({"literal":"text"}),
        )
        .unwrap();
    let alias = Identifier::new("odd.\"alias;--").unwrap();
    let sql=format!("SELECT {type_id}::int::text AS {}, {key_num}::smallint::text AS key_num, {exact}::numeric::text AS exact_value, {text}::text AS text_value",alias.sql());
    println!(
        "{}",
        json!({"sql":sql,"parameters":parameters.into_slots()})
    );
}
