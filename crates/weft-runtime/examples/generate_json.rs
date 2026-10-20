use std::io::Read;
fn main() {
    let mut raw=String::new();std::io::stdin().read_to_string(&mut raw).unwrap();
    let args:Vec<String>=std::env::args().collect();
    let result=match args.get(1).map(String::as_str) {
        Some("application")=>weft_runtime::prepare_application_lineage_json(&raw),
        Some("designation")=>weft_runtime::prepare_current_query_designations_json(&raw),
        Some("security")=>Ok(weft_runtime::prepare_security_mapping_json(&raw)),
        _=>panic!("usage: generate_json application|designation|security"),
    };
    match result {Ok(value)=>print!("{value}"),Err(error)=>{eprintln!("{}: {}",error.code,error.message);std::process::exit(1)}}
}
