//! Original guarded candidate correspondence, not execution/release permission.
use std::io::{Read, Write};
use weft_core::error::{Diagnostic, Result};
fn run() -> Result<String> {
    let refused = || {
        Diagnostic::new(
            "WFT-SECURITY-GUARDED-INPUT",
            "input",
            "Bounded original guarded input required",
        )
    };
    let mut bytes = Vec::new();
    std::io::stdin()
        .take(32000001)
        .read_to_end(&mut bytes)
        .map_err(|_| refused())?;
    if bytes.len() > 32000000 {
        return Err(refused());
    }
    let raw = std::str::from_utf8(&bytes).map_err(|_| refused())?;
    weft_postgresql::security_guarded_transport::inspect_guarded_graph_result(raw)
}
fn main() {
    match run() {
        Ok(output) => {
            if writeln!(std::io::stdout(), "{output}").is_err() {
                std::process::exit(1)
            }
        }
        Err(d) => {
            let _ = writeln!(std::io::stderr(), "{}", serde_json::to_string(&d).unwrap());
            std::process::exit(1)
        }
    }
}
