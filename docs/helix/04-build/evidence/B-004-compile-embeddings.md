# B-004 compile and embedding evidence — 2026-10-05

**Outcome:** the public Rust compiler envelope and native Python/browser transports
pass the complete `scripts/run-b004.sh` gate. B-004 completes its component scope.
Production backends, other wheel platforms and story/release conformance remain
B-005 through B-007.

## Surface and qualification

The same pure Rust Compiler handles strict 0.1/0.2 requests, supplied UMF bytes,
exact binding hashes, resolution, trusted registered backends and atomic compiled
artifacts or blocked diagnostics. Artifacts retain typed logical plans, model pins,
backend/target context, result decoding and operation declarations/assessments.
Draft 0.1 response schema is finalized for backend interface 0.2; this is not a
migration claim for a previously released API. The 0.2 response adds typed
scalar/value/related-key carriers. All selected fields are checked by versioned
schemas, with no network schema resolution.

Python distribution `weft-sql` imports `weft`; PyO3 0.27.1 and Maturin 1.9.6 build
an abi3 wheel tested on CPython 3.12.14, macOS arm64. No other platform or Python
version execution is claimed. Browser WASM uses wasm-bindgen 0.2.105 with explicit
host initialization and an unpublished TypeScript scalar transport wrapper.
Rust 1.90.0, Bun 1.4.2, Node 24.19.0 and Playwright 1.62.1 drive the gate.

Default builds register no production adapters. The explicit `test-third` feature
links supported and candidate fixture backends for conformance tests; request or
UMF content cannot load executable implementations. Fixture SQL execution on
SQLite 3.53.1 does not qualify Truss/PostgreSQL or Ashlar/Databricks.

## Execution

- All 38 core tests pass, including three public-envelope tests.
- All original 636, application-read 620 and third-backend 26 regression cases
  pass their existing gates and browser parity checks.
- Public corpus preserves all 636 original and 620 application input cases,
  plus 17 supported/candidate backend cases: 1,273 authored outcomes.
  Frontend-valid requests targeting unregistered production adapters explicitly
  refuse `WFT-BACKEND-MISSING`. One deliberate request-schema negative now refuses
  `WFT-INPUT` at the public envelope. Fault-injection probe flags are excluded.
- Every public response validates against its corresponding response schema.
- Seven compiled fixture queries execute against independently authored rows,
  retaining duplicates, Unicode and injection-like text.
- Native Python produces byte-identical output for all 1,273 cases with PATH empty
  and subprocess calls disabled; invalid argument types and lone surrogates refuse.
- Chromium 153.0.8010.12 produces byte-identical output for all 1,273 cases.
  No Node globals or network calls are available during compilation. WASM imports
  only generated error and externref interop; memory grows from 1,769,472 to
  1,966,080 bytes. Module size is 9,093,187 bytes.
- Browser wrapper rejects unpaired UTF-16 surrogates before encoding. A real
  WebAssembly unreachable fixture verifies the fixed fatal host diagnostic and
  prevents re-entry after a trap. Recovery needs a fresh module context/realm.
- Formatting and spec checks pass: 43 artifacts, 10 schemas, 30 still-planned
  acceptance criteria. The full gate log is `/private/tmp/weft-b004-full.log`;
  generated parity/oracle reports remain reproducible under `target/b004`.

## Snapshot hashes

| File | SHA-256 |
| --- | --- |
| `crates/weft-core/src/compile.rs` | `237226350f93fd16e5c9f8a162e71be22fb958bbeceecd7d8159ad67abca949b` |
| `crates/weft-runtime/src/lib.rs` | `19367eedd037598c8edb4a2bb8e1d416fa43b9923b4b81dc7e5864cfbfdacb9a` |
| `crates/weft-python/src/lib.rs` | `844bcae0d33cbf34aaee1301777b3d844e9bed66354aea7e387acb95cb32b246` |
| `crates/weft-wasm/src/lib.rs` | `b4f59dca522785722bc6fddef3044aa41a5a7ca7cf5848ce74f974efc36e6ee4` |
| `packages/weft-browser/src/index.ts` | `ff575c5e5866cb33cc3ec949bf25f3d94a3414a486a8507173902ac852044a11` |
| `tests/compile/fixtures/cases.json` | `871d078977497b2534cc3fbefb07592b27b6c1572610d554b317750bbe58296f` |
| `target/b004/test-wheels/weft_sql-0.1.0-cp39-abi3-macosx_11_0_arm64.whl` | `e1db8da87da69992a9aa16f83ff65380b5ad9e8b2fa8aa33d4d25c7e96cc0cd5` |
| `target/b004/web/weft_wasm_bg.wasm` | `a18d3911b0c22ef7f629349246453137465028d7232dff2a0fa2f52e4ceae7e4` |
