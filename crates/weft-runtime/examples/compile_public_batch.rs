// SPDX-License-Identifier: MIT OR Apache-2.0
//! JSON-lines host for the explicitly selected public runtime composition.
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        println!("{}", weft_runtime::compile_json(&line.unwrap()));
    }
}
