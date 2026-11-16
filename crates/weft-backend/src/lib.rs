// SPDX-License-Identifier: MIT OR Apache-2.0
//! The backend boundary shared by weft and other compilers (CONTRACT-002).
//!
//! - [`manifest`]: backend manifests, capabilities, obligations and their
//!   validation. Which interface versions and language/IR pairs are valid is the
//!   frontend's choice ([`boundary::Frontend`]).
//! - [`boundary`]: the `Frontend` and `Backend` traits, the binding context,
//!   assessment, the checked compile pipeline and the `Registry`. A backend's
//!   `emit` returns a target artifact of its own type.
//! - [`sql`]: one artifact type, the SQL emission carrier (statement text,
//!   typed parameter slots, result columns and decoders).
//!
//! Manifests never load code; only an explicitly registered host object is a
//! backend. See `README.md` for the versioned surface.
pub mod boundary;
pub mod manifest;
pub mod sql;

/// The weft release version this crate was built at (see `README.md`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
