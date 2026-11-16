// SPDX-License-Identifier: MIT OR Apache-2.0
//! Re-export of `weft_model::core_literal` (CoreLiteral admission against UMF Field domains).
pub use weft_model::core_literal::{check_literal, validate_field, PreparedLiteral, ScalarLiteral};
pub(crate) use weft_model::core_literal::{coefficient, normalized_facets, normalized_literal, unsigned, MAX_TEXT};
