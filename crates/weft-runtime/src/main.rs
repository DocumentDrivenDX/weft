use std::io::{self, Read};
fn main() {
    let mut request = String::new();
    io::stdin().read_to_string(&mut request).unwrap();
    println!("{}", weft_runtime::compile_json(&request));
}
