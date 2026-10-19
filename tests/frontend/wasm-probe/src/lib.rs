// SPDX-License-Identifier: MIT OR Apache-2.0
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub fn resolve_json(input: &str) -> String {
    weft_core::frontend_json(input)
}
