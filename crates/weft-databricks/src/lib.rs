//! Pure Ashlar/Databricks backend components. Hosts own native execution.
pub mod binding;
pub mod candidate;
pub mod mathematical_integer;

/// Engine-pinned candidate registration for the B-007 native support review.
pub mod native_profile;

/// Native-semantics-qualified registration; explicit host obligations remain mandatory.
pub mod qualified_profile;
