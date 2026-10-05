# B-002 frontend evidence — 2026-10-05

**Outcome:** the bounded 0.1 model/parser/typed-IR slice passes its gate. The next slice is B-002A for the owner-requested PR #2 application reads; B-003 follows that extension. This is frontend evidence, not production backend, public compile-envelope or full release conformance.

## Implementation and profile

Rust 1.90.0; serde 1.0.228 / serde_json 1.0.145 arbitrary precision; jsonschema 0.56.0 compile-time macros with file/HTTP resolution disabled. The pinned UMF experimental 0.7.0 schema dependency and provenance live under `spec/upstream/`. The frontend uses its own bounded lexer/parser with UTF-8 byte spans and the exact 0.1 grammar; sqlparser remains a B-001 experiment dependency. No broad parser acceptance becomes a support claim.

Types are finalized in `crates/weft-core/src/ir.rs` against CONTRACT-001's schema. Model preparation verifies owning-document byte hashes, IDs/core versions, duplicate module/element identities, selected modules and pinned structural constraints. Selected record member references resolve locally through their owning document. Whole source text and unknown content remain retained. Selected unsupported references/facets/cardinality/availability refuse. This is a semantic participation subset, not a claim of complete Rust UMF validation.

## Verification

`sh scripts/run-b002.sh` passed with the same isolated Rust tooling and macOS arm64 / CPython 3.12.14 / Node 24.19.0 / Bun 1.4.2 / Playwright 1.62.1 / Chromium 153.0.8010.12 profile used for the prior spike.

- Four Rust integration tests pass. The corpus test checks all **636** independent expected resolution/refusal outcomes; additional tests check UTF-8 quoted names/spans, self-join occurrence IDs, ON visibility, GROUP BY without aggregate, duplicate identities, missing local dependencies, invalid envelopes and a retained 40-digit unknown number.
- The independent Python bag interpreter checks **13** expected relational result bags with Python integers and Decimal precision 100. It preserves join duplicates, exact SUM, string distinctions and empty global/grouped aggregate behavior. It emits no SQL and reuses no Rust resolver/numeric evaluator.
- **303** successful typed plans validate against the public IR JSON schema. The remaining cases are expected refusals, not skipped execution.
- A test-only WASM probe produces byte-identical frontend reports for all **636** cases in real Chromium. Network APIs are disabled during evaluation, Node globals are absent and imports are restricted to wasm-bindgen throw/reference-table helpers. No entropy, file, database or network host interface is imported.
- Specification integrity checks pass: 43 governed artifacts, five schemas, 30 planned public acceptance criteria and the unchanged 636-case 0.1 corpus.

The [run log](b002/run.log), [browser summary](b002/browser-summary.json) and [source hashes](b002/source-sha256.json) retain exact evidence. Generated response files/binaries stay under ignored `target/`. Browser linear-memory observations are not total/peak-memory or performance claims.

## Dependency correction

The initial runtime schema validator pulled in ahash runtime entropy, which failed the first WASM check and would violate the core boundary. The final validator compiles the fixed schema at build time. A minimal [licensed ahash 0.8.12 manifest patch](../../../../vendor/README.md) selects its existing no-rng feature rather than runtime-rng; hashing source is unchanged. Unused direct WASM getrandom dependencies select the unsupported backend. Final native tests and the actual browser import/call probe pass. Upstream vendored cfg/dead-code warnings remain visible; they are not compiler test failures. Downstream Rust-source packaging must retain this workspace patch or prove an equivalent no-IO dependency configuration before qualification.

## Scope update from PR #2

The owner's request promoted six discovery outcomes into FR-13 through FR-18, FEAT-005, US-007, STP-007 and draft CONTRACT-004: whole entities with precise descriptors, stable keyset paging, COUNT, related/inverse reads, typed parameters and recognizable subsets. They are required implementation work, not declared supported by the current frontend. B-002A will finalize the separate 0.2 grammar/IR/transport fixtures while preserving 0.1 behavior. The current frontend explicitly refuses other dialect versions.

All 30 public story criteria remain planned for complete host/native qualification. This evidence establishes only their frontend components. Native target/layout/publication and Python public artifact qualification remain later gates.
