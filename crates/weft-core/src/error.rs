use crate::ir::Span;
use serde::Serialize;
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_span: Option<Span>,
}
impl Diagnostic {
    pub fn new(code: &str, phase: &str, message: &str) -> Self {
        Self {
            code: code.into(),
            severity: "error".into(),
            message: message.into(),
            phase: phase.into(),
            source_span: None,
        }
    }
    pub fn at(mut self, span: &Span) -> Self {
        self.source_span = Some(span.clone());
        self
    }
}
pub type Result<T> = std::result::Result<T, Diagnostic>;
