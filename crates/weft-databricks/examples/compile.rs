//! Host CLI for component/native fixtures; the compiler library has no IO.
use std::io::{self, Read};
fn main() {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw).unwrap();
    let mut registry = weft_core::backend::Registry::default();
    registry
        .register(weft_databricks::candidate::Candidate)
        .unwrap();
    println!(
        "{}",
        weft_core::compile::Compiler { registry }.compile_json(&raw)
    );
}
