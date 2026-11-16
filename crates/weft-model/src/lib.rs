// SPDX-License-Identifier: MIT OR Apache-2.0
//! UMF model loading, identity resolution, logical typing and exact values.
//!
//! This crate is the shared base of weft's compiler and of other compilers that
//! type programs against UMF models. It reads pinned UMF 0.10 documents, resolves
//! authored names to original identities, derives logical scalar types under an
//! explicit [`catalog::Admission`] profile, and checks exact literal tokens. It does
//! no SQL parsing, planning, backend lowering or IO. See `README.md` for the
//! versioned public surface.
pub mod catalog;
pub mod descriptor;
pub mod diagnostic;
pub mod exact;
pub mod exact_instant;
pub mod json;
pub mod name;
pub mod string_domain;
pub mod types;

/// This crate's contract version (`README.md`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
