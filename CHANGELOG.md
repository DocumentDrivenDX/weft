# Changelog

Older releases: see [GitHub Releases](https://github.com/DocumentDrivenDX/weft/releases).

## Unreleased

- New crate `weft-model` 0.1.0 (own semver; `crates/weft-model/README.md`): UMF
  0.10 document preparation, name and identity resolution, logical types,
  descriptors, exact literals, exact instants and diagnostics, moved out of
  `weft-core` so other compilers can depend on them. `weft-core` re-exports them
  under their old module paths (`model`, `application_model`, `ir`, `error`,
  `json`, `current_string_domain`, `syntax::Name`), so existing code compiles
  unchanged.
- `Catalog` admission flags are an explicit `Admission` profile
  (`Catalog::prepare_with`, `Catalog::admit`, `Catalog::admission`); prepared
  documents are read through `Catalog::documents`.
- `LogicalType` is generic over its family set. weft's dialect profiles keep the
  four-family `Family` and cannot represent anything wider, so weft-sql 0.1
  through 0.3 accept and refuse exactly what they did. The opt-in `ScalarFamily`
  adds UMF `timestamp` (exact UTC instants) and `float` (finite binary64 tokens),
  available through `Catalog::field_as`, `Catalog::entity_descriptor_as` and the
  other `*_as` methods; `Admission::nullable_fields` types `absent-allowed`
  Fields as nullable.
- CoreLiteral admission against UMF Field domains (`security_literals`, and the
  field-domain normalizer from `security_ontology`) moves to
  `weft_model::core_literal`; `weft_core::security_literals` re-exports it.
  Diagnostic codes are unchanged.
- New crate `weft-expr` 0.1.0 (own semver; `crates/weft-expr/README.md`): typed
  `Field`/`Value` operands, comparison operators and their typing rule, exact
  arithmetic domains and expressions, and the `weft.update-selection/1`
  predicate format of CONTRACT-009. `weft-core` re-exports the operand,
  comparison and arithmetic types under their old paths. The mutation frontend
  builds selections with it, and `inspect_selection` and the PostgreSQL mutation
  lowering decode them with one shared walker instead of two hand-written ones.
  Wire shapes and refusal messages are unchanged.
- New crate `weft-backend` 0.1.0 (own semver; `crates/weft-backend/README.md`):
  the CONTRACT-002 backend boundary. Manifest types and validation (accepted
  interface versions and language pairs come from the frontend), the `Frontend`
  and `Backend<F>` traits, a `TargetArtifact` emit output (`sql::Emission` for
  SQL backends, `NoArtifact` for backends that emit nothing), the checked compile
  pipeline and `Registry<F, A>`. `weft_core::backend` becomes weft's frontend
  (`WeftPlans`) and keeps its `Backend` trait, `Context`, `Registry` and type
  paths, so backend implementations compile unchanged; check order, refusal
  codes and the `Compilation` wire shape are unchanged.
- New crate `weft-postgresql-emit` 0.1.0 (own semver;
  `crates/weft-postgresql-emit/README.md`): PostgreSQL identifier quoting,
  parameter slots, string literals, typed casts with the `C` collation and exact
  result decoders, moved out of `weft-postgresql` (which re-exports
  `Identifier`, `qualified`, `Parameters` and `CatalogDomain`). Duplicated cast,
  literal and decoder helpers in the Truss runtime, candidate and mutation
  lowerings now call it; emitted SQL is unchanged. The Truss layout stays in
  `weft-postgresql`.
- The SQL carrier has result decoders for the opt-in families: `utc-instant`
  (`YYYY-MM-DDTHH:MM:SS.ffffffZ`) for `timestamp` and `ieee754-binary64-hex` (the
  binary64 bit pattern) for `float`. `Emission`, `Column` and `Representation`
  take a family-set parameter that defaults to `Family`, so weft's emissions
  cannot carry these columns and weft's emission checks refuse the new decoders.
  `weft_postgresql_emit::decodable` projects a PostgreSQL expression to its
  decoder's text, and `decoder_for` now returns a decoder for every family.
  Experimental: only unit tests exercise them.
- `bun scripts/module-boundaries-update.ts` regenerates the dependency and
  public-declaration inventory in `docs/helix/02-design/module-boundaries.json`.
  `scripts/check-release-version.py --shared-crates` (run in CI) checks that each
  shared crate's README states its Cargo version.

## 0.4.58 - 2026-11-11

- Generated read algebra v6 to v8 and ordinary pages (PR #59).
- The query-codec-instructions schema reference uses the target's URN `$id`, so
  `bun run specs:check` passes again.
- Release process simplified: one `release.yml` (tag-driven; tests, builds Linux
  and macOS arm64 wheels, an sdist and WebAssembly tarballs, smoke-installs each
  wheel, then publishes to PyPI with trusted publishing and creates the GitHub
  Release with SHA256SUMS and a build provenance attestation).
- CI tests every pull request: Rust workspace, installed wheel, schema checks.
- Repository cleanup: committed run output, receipts and candidate binaries were
  removed from the tree (they remain in git history).
- UMF is consumed by release tag instead of a commit pin, and moves to UMF 0.10.11
  (`umf-v0.10.11`, the release that ports the UMF core to Rust). The vendored
  `umf-temporal` schemas were refreshed from that tag.
- The source distribution now builds a wheel (flat license files, schema inputs
  shipped in the sdist); a metadata check guards it in pull requests.
- The `weft-sql` distribution name is retired. `umf-weft` is the only name.
- The 408 MB source tarball is no longer attached to releases; the sdist and git
  tag are the source distribution.

## 0.4.57

- Published to PyPI as `umf-weft` with wheels for Linux x86_64, Linux aarch64 and
  macOS arm64. See the GitHub Release for details.

## 0.4.56

- See the GitHub Release.

## 0.4.55

- The Python distribution was renamed from `weft-sql` to `umf-weft`. The import
  stays `weft`. Compiler behavior is unchanged from 0.4.54.

## 0.4.54

- Unity current aggregate compiler work integrated; typed-expression compiler
  (G02) retained.

## 0.4.53

- Closed compiler profile for typed same-row backfill expressions in UMF table
  operation 0.2 (G02).
