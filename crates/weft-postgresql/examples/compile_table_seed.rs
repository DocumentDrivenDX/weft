// SPDX-License-Identifier: MIT OR Apache-2.0
use std::io::{self,Read};
fn main(){let mut input=String::new();io::stdin().read_to_string(&mut input).unwrap();println!("{}",weft_postgresql::table_seed::compile_json(&input));}
