fn main() -> Result<(), Box<dyn std::error::Error>> {
 let mut registry=weft_core::backend03::Registry::default();
 registry.register(weft_databricks::paths_keys_having::CountStarHaving).map_err(|diagnostic| format!("{diagnostic:?}"))?;
 let manifest=registry.manifest(weft_databricks::paths_keys::ID).ok_or("registered backend missing")?;
 println!("{}",serde_json::to_string(manifest)?);Ok(())
}
