//! Source-owned Backend03 metadata; no executable, index or native qualification.
fn main() {
    let mut registry = weft_core::backend03::Registry::default();
    registry
        .register(weft_databricks::paths_keys::PathsKeys)
        .expect("explicit path registration must be valid and unique");
    let manifest = registry
        .manifest(weft_databricks::paths_keys::ID)
        .expect("the explicitly registered path backend must exist");
    println!(
        "{}",
        serde_json::to_string(manifest).expect("public manifest serialization")
    );
}
