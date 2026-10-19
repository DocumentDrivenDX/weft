// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded transport of a logical mapping input; never executes native SQL.
use std::io::{self, Read, Write};
const LIMIT:usize=weft_core::security_mapping_transport::MAX_REQUEST_BYTES;
fn read_request(input:impl Read)->Result<String,&'static str>{
    let mut bytes=Vec::new();
    input.take(LIMIT as u64+1).read_to_end(&mut bytes).map_err(|_|"WEFT_CLI_INPUT_IO")?;
    if bytes.len()>LIMIT{return Err("WEFT_CLI_INPUT_LIMIT");}
    String::from_utf8(bytes).map_err(|_|"WEFT_CLI_UTF8")
}
fn main(){
    let result=read_request(io::stdin().lock()).and_then(|request|{
        let response=weft_runtime::prepare_security_mapping_json(&request);
        let mut out=io::stdout().lock();
        writeln!(out,"{response}").and_then(|_|out.flush()).map_err(|_|"WEFT_CLI_OUTPUT_IO")
    });
    if let Err(code)=result{let _=writeln!(io::stderr().lock(),"{code}");std::process::exit(2);}
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn original_utf8_bytes_and_exact_byte_ceiling(){
        let raw=format!("{}é"," ".repeat(LIMIT-2));
        assert_eq!(read_request(raw.as_bytes()).unwrap(),raw);
        assert_eq!(read_request(format!("{raw} ").as_bytes()),Err("WEFT_CLI_INPUT_LIMIT"));
        assert_eq!(read_request([255u8].as_slice()),Err("WEFT_CLI_UTF8"));
    }
}
