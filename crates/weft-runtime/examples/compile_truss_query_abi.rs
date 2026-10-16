//! JSON-lines bridge to the actual explicit Truss runtime compiler.
use std::io::{self,BufRead};
fn main(){for line in io::stdin().lock().lines(){println!("{}",weft_runtime::compile_truss_runtime_json(&line.unwrap()));}}
