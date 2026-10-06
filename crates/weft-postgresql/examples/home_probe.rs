//! Native location witness; not a registered Truss compiler profile.
use serde_json::json;
use weft_core::ir::{Family, LogicalType};
use weft_postgresql::{access::ObjectProperty, Identifier, Parameters};
fn main() {
    let namespace = Identifier::new("pg_temp").unwrap();
    let alias = Identifier::new("owner").unwrap();
    let logical_type = LogicalType {
        family: Family::Integer,
        facets: json!({"integerWidth":{"bits":64,"signed":false}}),
        nullable: false,
    };
    let field = ObjectProperty {
        namespace: &namespace,
        owner_alias: &alias,
        type_id: "-1",
        property_id: "0",
        logical_type: &logical_type,
    };
    let mut output = vec![];
    for row in [false, true] {
        let mut parameters = Parameters::default();
        let access = if row {
            field.row(&mut parameters, 0).unwrap()
        } else {
            field.props(&mut parameters).unwrap()
        };
        let from = format!("pg_temp.object AS \"owner\" {}", access.joins.join(" "));
        output.push(json!({"home":if row{"row"}else{"props"},"sql":format!("SELECT ({})::text AS exact_value FROM {} ORDER BY {}",access.value,from,access.value),"integritySql":format!("SELECT count(*)::text FROM {} WHERE ({}) IS DISTINCT FROM TRUE",from,access.integrity),"parameters":parameters.into_slots()}));
    }
    println!("{}", json!(output));
}
