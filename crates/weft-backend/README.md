# weft-backend 0.1.0

The backend boundary of CONTRACT-002, shared by weft's compiler (`weft-core`)
and other compilers (Fireweed's `fireweed-compiler`): backend manifests and
their validation, the frontend and backend traits, the checked
validate-binding / assess / lower / emit pipeline, refusal diagnostics, the
registry, and the SQL emission carrier as one artifact type.

It depends on `weft-model` (catalog, identities, logical types, diagnostics,
bounded JSON). It contains no dialect plan, no physical mapping and no target
SQL text.

## Version

Own semver, independent of the workspace and the `umf-weft` wheel;
`weft_backend::VERSION` reports it. While the major version is 0, a minor bump
may break the public surface below; a patch bump may not. Manifest and
compilation wire shapes, refusal codes and the order of pipeline checks are
part of the surface.

## Public surface

| Module | Items | Meaning |
| --- | --- | --- |
| `manifest` | `Manifest`, `LanguageProfile`, `TargetProfile`, `Capability`, `Status`, `Obligation`, `ObligationOwner`, `ManifestRules`, `validate_manifest`, `validate_manifest_json`, `OpaqueManifest`, `take_manifest_opaque`, `restore_manifest_opaque`, `failure`, `nonempty`, `unique_strings` | Manifest types and structural validation. Accepted `interfaceVersion`s and dialect/IR pairs come from the caller's `ManifestRules`. |
| `boundary` | `Frontend`, `Backend<F>`, `TargetArtifact<F>`, `NoArtifact`, `Context<'a, F>`, `Validated`, `Assessment`, `Selection`, `BindingInput`, `Target`, `Qualification`, `Compilation<A>`, `Registry<F, A>`, `merge_obligations`, `coverage_contains` | The checked pipeline. |
| `sql` | `Emission`, `ParameterSlot`, `ParameterType`, `ExactParameterKind`, `Column`, `Representation`, `ScalarCarrier`, `ScalarDecoder`, `OuterJoin`, `NativeMetadata`, `SqlChecks` | The SQL artifact: statement text, typed parameter slots, result columns with carriers and exact decoders. Target-neutral (PostgreSQL and Databricks both emit it). |

## The pipeline

A `Frontend` names its typed program (`Program<'a>`) and supplies the language
pair, IR version, required operations, model pins and selected identities of a
program, plus three optional hooks: `read_binding`, `preflight` (refusals before
any backend callback) and `admit_catalog`.

A `Backend<F>` declares a manifest and implements `validate_binding`, `assess`,
`lower` and `emit`. `emit` returns the backend's `Artifact`, any type that
implements `TargetArtifact<F>`: weft's backends return `sql::Emission`; a backend
that emits nothing (an in-memory reference interpreter) returns `NoArtifact`.

`Registry<F, A>::compile` checks, in this order: backend version, program IR and
operation identities, language pair, target profile, binding profile, model pins,
binding size and digest, binding JSON, the frontend's preflight; then calls
`validate_binding`, checks identity coverage and binding-derived capabilities,
calls `assess` and checks every required operation against its declaration,
status, candidate opt-in and evidence; then `lower`, `emit`, the artifact's own
`check`, and merges obligations. Backend panics become `WFT-BACKEND-FAILURE`.

## What stays in weft-core

`weft_core::backend` is weft's frontend (`WeftPlans`): the `Plan` enum over
weft-sql 0.1–0.3 plans, the `weft-backend/0.2.x` manifest rules, the Truss
runtime binding reader, capability preflights, and the emission checks in
`backend_emission`. Its `Backend` trait, `Context`, `Registry` and `Compilation`
keep their paths and shapes, so every backend implementation compiles unchanged.
The `backend03` interface (0.4 plans), `security_backend` and the mutation
`SelectionBackend` are separate weft boundaries and stay in weft-core.
