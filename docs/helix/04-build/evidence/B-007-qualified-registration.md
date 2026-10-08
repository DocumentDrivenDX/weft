# Supported native-semantics registrations

Separate `Qualified` registrations now declare `0.1.0-qualified` with profiles
`pg17.9-qualified-fixtures` and `dbsql2026.39-qualified`. Their evidence IDs pin
the complete qualification JSON records by SHA-256. The records preserve exact
engine/settings, native domains/storage homes, evidence references and exclusions.
Frozen branch-review inputs retain the qualification checkpoint; later status edits
cannot silently rewrite that evidence. Supported is a compiler-semantic disposition
under explicit host obligations, not database execution or authorization.

The actual direct-registration harness passes 2,181 native-tested inputs with
`allowCandidate:false`: 2,180 conformance-verified outputs and one identical
unsupported UInt64-column refusal. Every meaning-bearing output preserves the
retained native-tested SQL/guards/parameters/decoders/plan/pins. Profile metadata
and declarations/assessments explicitly change to the qualified identities. All
27 required capability IDs are observed per backend; six stale/newer/profile
controls refuse without SQL.

The public runtime's qualified-feature composition passes two unfiltered tests,
including both adapters with no candidate opt-in and exact-version refusal.
Python/WASM Cargo features forward the same registration choices. Candidate and
qualified features for one backend conflict at compile time, so no version
silently wins. The historical candidates remain available.

`B-007-qualified-registration/` retains artifacts, manifests, source/build/custody
records and runtime evidence. Its failed initial runtime compile used a nonexistent
test fixture path; it is retained as an authoring failure, not a backend defect.
Both backend feature-conflict checks refuse compilation as designed. Two scoped
historical NativeReview library regressions pass (PostgreSQL 76 other unit tests
filtered, Databricks none filtered). An incremental-cache build failure is retained;
the successful rerun disables incremental compilation. Final native Python/Chromium
qualified builds, current
workspace checks and the 30-criterion audit remain open before PR #9 can merge.
