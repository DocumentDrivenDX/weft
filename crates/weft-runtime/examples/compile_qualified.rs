//! Explicit qualification harness; neither models nor bindings select code.
use std::io::{self, BufRead};
use weft_core::{
    backend::{Backend, Registry},
    compile::Compiler,
};
fn main() {
    if std::env::args().nth(1).as_deref() == Some("--manifests") {
        println!(
            "{}",
            serde_json::json!([
                weft_postgresql::qualified_profile::Qualified
                    .describe()
                    .unwrap(),
                weft_databricks::qualified_profile::Qualified
                    .describe()
                    .unwrap()
            ])
        );
        return;
    }
    let mut registry = Registry::default();
    registry
        .register(weft_postgresql::qualified_profile::Qualified)
        .unwrap();
    registry
        .register(weft_databricks::qualified_profile::Qualified)
        .unwrap();
    let compiler = Compiler { registry };
    for line in io::stdin().lock().lines() {
        println!("{}", compiler.compile_json(&line.unwrap()));
    }
}
