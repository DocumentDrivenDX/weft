# B-003 backend boundary evidence — 2026-10-05

**Outcome:** the versioned Rust library backend boundary passes its component,
independent fixture-SQL and real-browser gates. B-003 completes the registry,
binding/capability pipeline and structural ADR-002 decision. Public compile
wrappers and production/native target qualification remain B-004 through B-007.

## Exact implemented surface

[CONTRACT-002](../../02-design/contracts/CONTRACT-002-backend-interface.md) finalizes
`weft-backend/0.2.0` for explicit typed language/IR pairs 0.1 and 0.2. The old
manifest schema and historical evidence remain unchanged. A trusted host registers
Rust `Backend` implementations with associated Mapping/TargetPlan types. A generic
adapter snapshots the manifest and runs validate-binding, assess, lower and emit;
there is no frontend backend-ID switch or content-driven executable loading.

Manifest guards cover exact language/IR pairs, distinct IDs, selected target
profiles, domains, declared evidence, obligation shapes and unknown members.
Binding JSON is bounded to four MiB, duplicate checked and pinned by exact byte
SHA-256. Plans must match the supplied catalog pins. Every selected record, field,
type graph and relationship identity needs declared binding/codec coverage.
Physical mapping correctness remains the backend's responsibility; common
coverage is not a storage or existing-data attestation.

Assessments cover every required operation once, including binding-derived
capabilities. Default compilation blocks candidates; opt-in retains candidate
status. Candidate declarations cannot be upgraded to supported. Unsupported,
missing and profile-incompatible operations refuse without fallback. Each emitted
qualification retains its declaration's domains/constraints/obligations/evidence
and per-query assessment. Evidence IDs are trusted qualification references;
registration does not fetch or independently certify referenced records.

Emission guards preserve logical result count/order/names, types, selected source
identities, presence/type/relationship descriptors and exact numeric text decoders.
Native-null representations require an explicitly assessed binding-derived
`value.nativeNull` capability. Parameter slots retain exact strings and checked
lexical/domains; positions are contiguous and origins typed objects. SQL is bounded
to one MiB with no silent clipping. Malformed/conflicting obligations, backend
errors and native unwind panics block atomically. WASM trap/abort normalization
belongs to the future public wrapper/host gate; it is not proved by unwind tests.

## Third backend and independent execution

A test-only plugin lives outside the compiler frontend in
`tests/register-backend/fixture.rs`, with its own mapping and target-plan Rust
types. Its explicit synthetic profile supports one required string projection
and ASCII physical identifiers. It emits quoted SQL over supplied physical homes
and retains logical output labels, including Unicode and embedded quotes.

The independent CPython/SQLite host executes seven positive SQL cases against
nine authored rows each: duplicate values, Unicode composed/decomposed text,
trailing spaces, empty strings, injection-looking data/labels and alternate
column/table homes. Nineteen independently specified refusals cover capability,
registration/version/profile, mapping/pin, injection and emission faults. SQLite
3.53.1 is an execution witness for this fixture subset, not a production SQLite,
Truss or Ashlar backend qualification. Synthetic evidence declaration IDs do not
become native target support claims.

## Verification

`sh scripts/run-b003.sh` passes with Rust 1.90.0 on macOS arm64, CPython 3.12.14,
SQLite 3.53.1, Node 24.19.0, Bun 1.4.2, Playwright 1.62.1 and Chromium 153.0.8010.12.
The [run log](b003/run.log), [source hashes](b003/source-sha256.json),
[SQLite summary](b003/oracle-summary.json) and [browser summary](b003/browser-summary.json)
record exact evidence.

- All 35 core unit/integration tests pass, including 12 backend boundary tests and
  the 23 prior frontend tests. Coverage includes both typed plan versions,
  candidate refusal/retention, missing operations/coverage, wrong versions/pins,
  selected unknown/injected mapping members, lower errors, unwind normalization,
  result metadata/slot faults, exact numeric parameter/result checks, native-null
  descriptor gating and obligation retention/conflicts.
- The unchanged 636 original and 620 application decisions/oracles/schemas pass.
  Application schema checks retain 304 successful plans; original checks retain
  303 plans. Independent frontend oracles retain 13 + 18 result scenarios.
- All 26 third-backend reports match native and real Chromium byte for byte;
  seven have independently executed SQL and schema-valid manifest snapshots.
  Original and application Chromium gates also pass: 1,282 total request reports.
- Browser compilation uses asynchronous WebAssembly.compile on the normal main
  thread. The initial synchronous harness hit Chromium's debug-binary size limit;
  no special flag or relaxed browser setting is used by the passing gate.
  Network APIs are disabled during calls, Node globals are absent and imports
  remain restricted to wasm-bindgen reference-table/throw helpers. Memory/size
  observations are not performance or peak-memory claims.
- Specification checks pass for 43 governed artifacts, nine schemas, 30 planned
  public criteria and the original 636-case corpus.

## Traceability and limits

US-002-AC1–AC4 have component evidence through registry/pipeline, fixture-SQL and
browser tests. Full host/native story qualification remains planned. ADR-002
accepts the structural boundary only. There is no released dynamic plugin ABI,
Python/JavaScript callback API, production layout selection or database access
inside the compiler. Linked/trusted host/build registration is the supported
library mechanism for this foundation.

Truss owns its physical/catalog mapping and native execution mechanics; Ashlar
owns its gold layout/publication mapping. Weft owns logical compilation and the
registered lower/emit boundary. Accepted production mappings, document-qualified
identity access, exact key-component ordering, decoding/publication obligations
and target-native evidence remain integration gates. No Truss-owned artifact or
second UMF compiler is authored by this slice.
