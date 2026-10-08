//! Host-owned JSON-lines harness for exact engine-profile qualification.
//! Build with both initial adapter features; this does not change public composition.
use std::io::{self, BufRead};
use weft_core::{backend::{Backend, Registry}, compile::Compiler};
fn main() {
    if std::env::args().nth(1).as_deref() == Some("--manifests") {
        println!("{}", serde_json::json!([
            weft_postgresql::native_profile::NativeReview.describe().unwrap(),
            weft_databricks::native_profile::NativeReview.describe().unwrap()
        ]));
        return;
    }
    let mut registry = Registry::default();
    registry.register(weft_postgresql::native_profile::NativeReview).unwrap();
    registry.register(weft_databricks::native_profile::NativeReview).unwrap();
    let compiler = Compiler { registry };
    for line in io::stdin().lock().lines() {
        println!("{}", compiler.compile_json(&line.unwrap()));
    }
}
