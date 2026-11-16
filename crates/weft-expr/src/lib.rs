// SPDX-License-Identifier: MIT OR Apache-2.0
//! Typed expression and predicate core shared by weft and other compilers.
//!
//! Field references and literal/parameter operands over a [`weft_model`] family
//! set, comparison operators and their typing rule, exact arithmetic domains,
//! and the `weft.update-selection/1` predicate format (CONTRACT-009). Parsing
//! source text and lowering to a backend stay with each compiler. See
//! `README.md` for the versioned public surface.
pub mod arithmetic;
pub mod compare;
pub mod operand;
pub mod selection;

/// This crate's contract version (`README.md`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
