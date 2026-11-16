// SPDX-License-Identifier: MIT OR Apache-2.0
pub mod access;
pub mod binding;
pub mod candidate;
pub mod collection;
#[cfg(feature = "conformance-original")]
pub mod conformance_original;
pub mod current_scalar;
mod dbapi;
mod json_codec;
pub mod mutation;
mod native_catalog_fence;
pub mod native_privilege_probes;
pub mod original_admission;
pub mod pg18;
pub mod presence_definition;
pub mod raw_read;
pub mod read_context_definition;
pub mod recursive_observation;
pub mod relationship_definition;
pub mod relationship_lowering;
mod row_codec;
pub mod row_tree_mapping;
pub mod row_value_traversal;
mod storage_composition;
pub mod structured;
mod tree;
pub mod truss_mutation;
pub mod truss_runtime;
pub mod value_definition;
// PostgreSQL emission primitives live in weft-postgresql-emit; storage mappings
// (the Truss layout) and engine support stay here.
pub use weft_postgresql_emit::{qualified, CatalogDomain, Identifier, Parameters};

pub mod leaf_codec_definition;

pub mod native_comparator_definition;

pub mod comparator_requirements;

pub mod row_join_definition;

pub mod property_definition;

pub mod numeric_correspondence;

pub mod registered_access;

pub mod expression;

pub mod relational;

pub mod record_definition;

pub mod result_definition;

pub mod select_definition;

pub mod row_custody;

pub mod value_traversal;

pub mod original_backend;

/// Engine-pinned candidate registration for the B-007 native support review.
pub mod native_profile;

/// Native-semantics-qualified registration; explicit host obligations remain mandatory.
pub mod qualified_profile;

pub mod security_predicate_spike;

/// Explicit reified graph correspondence, not protected execution admission.
pub mod security_graph_binding;

/// Owner-derived direct graph projection; native admission remains mandatory.
pub mod security_graph_query;

/// Original graph parameter source/ordinal correspondence; no native credential.
pub mod security_graph_parameters;

/// Separate guarded candidate handoff and original result correspondence; no native permit.
pub mod security_guarded_transport;

/// Original all-operator query-use guards; relational/native admission remains required.
pub mod security_query_use_guards;

/// Separate original relational query, policy and unvalued Subject ordinal channels.
pub mod security_relational_parameters;

/// Original relational graph SQL candidate; native operand/cut admission remains required.
pub mod security_relational_sql;

/// Separate guarded relational handoff; original correspondence only, no native permit.
pub mod security_relational_transport;

/// PostgreSQL18.6 value codec correspondence; no native authority or session admission.
pub mod security_native_codecs;

mod truss_browser_binding;

pub mod raw_action;

pub mod raw_bulk;

pub mod search;

#[cfg(feature = "current-presence-candidate")]
pub mod current_presence;

pub mod related_read;

pub mod table_operation;
pub mod table_seed;
pub mod table_query;
mod query_wire;
