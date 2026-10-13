# Explicit Truss native metadata IR foundation — 2026-10-13

The application IR now has a NativeMetadata expression carrying its scan, actual Record identity, explicit metadata enum and logical type. It does not invent authored UMF Field identities. Record dependency selection and scalar emission checks recognize it. Existing candidate/original/security paths explicitly refuse unsupported metadata. Databricks refuses Truss metadata too.

No resolver constructs the expression, and no schema/profile admits or public entry point exposes it. This is unactivated implementation source, not a complete projection or new support claim. The new profile must still pin metadata type/nullability/collision semantics and versioned schema admission; Truss lowering and whole-entity integration remain pending.

Cargo checks pass for the Python bridge with truss-postgresql-pg18 and for Databricks. Module boundaries pass11packages. Astra Ultra reviewed the unactivated foundation. No native whole-entity pass is claimed.

Resolver increment: the explicit complete-entity profile now constructs the five typed metadata outputs for alias.*, rejects reserved-name/member collisions and requires a nonaggregated single-source entity projection. It selects IR0.2.2; older profile/version combinations are unchanged. Python and Databricks cargo checks and11-package module boundaries pass. Astra Ultra reviewed the source. Public compiler dispatch still refuses0.2.2, and complete relationship expansion, compile.rs IR-version preservation, lowering, Python decoding and release activation remain pending.
