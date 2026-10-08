//! Native test transport with explicit candidate registration; not host execution.
use std::io::{self, Read};
fn main() {
    let mut registry = weft_core::backend::Registry::default();
    registry
        .register(weft_postgresql::candidate::Candidate)
        .unwrap();
    let compiler = weft_core::compile::Compiler { registry };
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw).unwrap();
    println!("{}", compiler.compile_json(&raw));
}
