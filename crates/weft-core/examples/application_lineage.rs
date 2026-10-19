// SPDX-License-Identifier: MIT OR Apache-2.0
//! Policy-independent application lineage; no authorization or native execution.
use std::io::{Read, Write};
fn main() {
    let mut input = Vec::new();
    let result = std::io::stdin()
        .take(4_000_001)
        .read_to_end(&mut input)
        .map_err(|_| "WFT-APPLICATION-LINEAGE".to_owned())
        .and_then(|_| std::str::from_utf8(&input).map_err(|_| "WFT-APPLICATION-LINEAGE".to_owned()))
        .and_then(|raw| {
            weft_core::application_lineage::prepare(raw)
                .map(|p| p.json().to_owned())
                .map_err(|d| format!("{}: {}", d.code, d.message))
        });
    match result {
        Ok(output) => {
            if std::io::stdout().write_all(output.as_bytes()).is_err() {
                std::process::exit(1);
            }
        }
        Err(d) => {
            let _ = writeln!(std::io::stderr(), "{d}");
            std::process::exit(1);
        }
    }
}
