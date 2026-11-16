// SPDX-License-Identifier: MIT OR Apache-2.0
//! Authored source names. Quoted names match exactly; unquoted names are written
//! lower-case and match the ASCII-lower-cased authored name.
use crate::types::Span;

#[derive(Debug, Clone)]
pub struct Name {
    pub value: String,
    pub quoted: bool,
    pub span: Span,
}
impl Name {
    pub fn matches(&self, value: &str) -> bool {
        if self.quoted {
            self.value == value
        } else {
            self.value == value.to_ascii_lowercase()
        }
    }
    pub fn key(&self) -> &str {
        &self.value
    }
    /// A quoted name for identity-driven lookups that have no source span.
    pub fn quoted(value: &str) -> Self {
        Self { value: value.into(), quoted: true, span: Span { start: 0, end: 0 } }
    }
}
