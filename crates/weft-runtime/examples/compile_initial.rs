//! Host-owned CLI for the explicitly selected build-time adapters.
//! Build with the desired runtime features; no backend is chosen by this host.
use std::io::{self, Read};
fn main() {
    let mut request = String::new();
    io::stdin().read_to_string(&mut request).unwrap();
    println!("{}", weft_runtime::compile_json(&request));
}
