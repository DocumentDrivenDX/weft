//! Distribution producer probe; source metadata is not executable self-description.
fn main() {
    let mut registry = weft_core::backend::Registry::default();
    registry
        .register(weft_databricks::candidate::Candidate)
        .expect("explicit candidate registration must be valid and unique");
    let manifest = registry
        .manifest("ashlar.databricks")
        .expect("the explicitly registered candidate must exist");
    println!("{}", serde_json::to_string(manifest).expect("public manifest serialization"));
}
