// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::ir::Span;
use serde::Serialize;
#[derive(Debug, Clone, Copy)]
pub(crate) enum ResolutionReason {
    ColumnMissing,
    ColumnAmbiguous,
    SourceUnavailable,
}
impl ResolutionReason {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::ColumnMissing => "WFT-COLUMN-MISSING",
            Self::ColumnAmbiguous => "WFT-COLUMN-AMBIGUOUS",
            Self::SourceUnavailable => "WFT-SOURCE-UNAVAILABLE",
        }
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_span: Option<Span>,
    #[serde(skip)]
    pub(crate) resolution_reason: Option<ResolutionReason>,
}
impl Diagnostic {
    pub fn new(code: &str, phase: &str, message: &str) -> Self {
        Self {
            code: code.into(),
            severity: "error".into(),
            message: message.into(),
            phase: phase.into(),
            source_span: None,
            resolution_reason: None,
        }
    }
    pub fn at(mut self, span: &Span) -> Self {
        self.source_span = Some(span.clone());
        self
    }
    pub(crate) fn column(mut self, ambiguous: bool) -> Self {
        self.resolution_reason = Some(if ambiguous { ResolutionReason::ColumnAmbiguous } else { ResolutionReason::ColumnMissing });
        self
    }
    pub(crate) fn unavailable_source(mut self) -> Self {
        self.resolution_reason = Some(ResolutionReason::SourceUnavailable);
        self
    }
}
pub type Result<T> = std::result::Result<T, Diagnostic>;
