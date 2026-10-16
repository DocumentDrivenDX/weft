use std::io::{self,Read};
fn main(){let mut s=String::new();io::stdin().read_to_string(&mut s).unwrap();println!("{}",weft_postgresql::raw_read::compile_json(&s));}
