//! Pure Ashlar/Databricks backend components. Hosts own native execution.
pub mod binding;
pub mod candidate;
pub mod mathematical_integer;

/// Engine-pinned candidate registration for the B-007 native support review.
pub mod native_profile;

/// Native-semantics-qualified registration; explicit host obligations remain mandatory.
pub mod qualified_profile;

/// Explicit finite coefficient lowering for exact 0.3 row arithmetic.
pub mod arithmetic;

/// Explicit required String set/count composition; preserves prior arithmetic profile.
pub mod count_distinct;

pub mod count_having;

/// Explicit scalar String outer-join profile.
pub mod left_join;

/// Explicit Backend03 candidate; native acceptance remains a host obligation.
pub mod paths;

/// Separate required-root bounded key candidate; no implicit upgrade.
pub mod paths_keys;

pub mod paths_keys_having;
