# Declared table operation compiler candidate

Unpublished Weft0.4.48 successor on `codex/praxis-lifecycle-048`. The separately
coordinated0.4.47 release repair is published and integrated at12051af8.
Rebasing preserves every compiler/runtime source byte; only release documentation
and current-core capacity fixtures differ from the verified pre-rebase tree. Original UMF6d623fc52 supplies
the pinned `umf.table-operation/0.1` declaration schema; full core0.10 source
bytes and native mappings remain preserved. All fixtures are public synthetic
examples, with no private Praxis names, source, SQL or data copied.

The dedicated compiler admits module-owned named operation invocations with
closed, typed inputs. It lowers declared read/count/page, insert/update/delete,
upsert ignore/update and atomic single-operation batch shapes through shared Rust
APIs exposed to native Python and WASM. All input and literal data is bound.
Key, immutable/generated Field, replay, exact native capacity and cursor-tuple
checks precede emission. Existing raw-read/action/search command guards remain.
Programs retain original request/source, derived declaration, digests and explicit
unfulfilled native authority obligations. `check_table_operation_json` recompiles
the original source; self-rehashed programs do not acquire trust.

The `pg18-effect-count/0.1` write guard counts direct target effects inside a
materialized data-modifying CTE. Excess rows cause native SQLSTATE22012 and
statement rollback; writes are never clipped. Native foreign-key actions retain
their authored PostgreSQL semantics and are outside this target-row count. Batch
shape cannot choose batch identity. Count returns exact integer text, numerics
stay exact, prefix data is literal, and nullable equality/membership semantics
are explicit in CONTRACT-004. Cursor metadata requires current source/principal
binding; it is not an authentication claim for a caller-provided tuple.

Reproduce from the repository root using Rust1.90.0 and Python3.11:

```sh
cargo test -p weft-postgresql --test table_operation --test table_lifecycle --offline
cargo build -p weft-postgresql --example compile_table_operation --offline
python -I tests/raw-postgresql/table-operation-bindings.py target/debug/examples/compile_table_operation /absolute/bindings.json
python -I tests/raw-postgresql/table-operation-native.py --dsn OWNED_PG18_DSN --report /absolute/native.json
cargo check --workspace --all-targets --offline
cargo check -p weft-python --no-default-features --offline
cargo check -p weft-wasm --target wasm32-unknown-unknown --features raw-postgresql-reads --offline
bun scripts/module-boundaries.ts
bun scripts/check-specs.ts
```

The native oracle additionally requires the pinned installed `umf_access` and
`umf_codegen` candidates, Psycopg3.2.13 and Pydantic2.13.5. The dedicated CI job
builds those dependencies from exact UMF6d623fc52 source, builds the Weft wheel
and runs against disposable PostgreSQL18.6. New hosted outcomes remain pending.
The retained [receipt](receipt.json), [binding comparison](bindings.json) and
[native oracle](native.json) qualify only their actual versions and subsets.
The native oracle installs the original source through UMF's approved migration
API, then deliberately exercises compiler SQL inside caller transactions as
ordinary roles. It is a test oracle, not a production executor or authorization
provider. No global connection/pool or retry is introduced by the compiler.

Production module invocation/admission, source-ledger current generation and
privilege closure, authenticated cursors, durable data outcomes and generated
native methods remain open. Date/time/binary codecs, necessary schema evolution
and seeds, full private consumer conversion and available Lakebase acceptance
still gate the full Praxis goal. The installed public SyncSession/AsyncSession
composition passes50 native assertions, releasing37/37 sync and8/8 async leases,
including caller savepoints and repeated cancellation during a confirmed row-lock
wait. Psycopg2/SQLAlchemy composition, real pools and production execution remain
unqualified. The final compiler passes10 operation plus10 custody tests,27
installed CLI/Python comparisons and10 self-rehashed controls. `nativeExecutionQualified`
remains false. Real Chromium153.0.8010.12 now agrees with installed native compilation on all27
cases, checks compiled programs and rejects one modified program and two invalid
Unicode transports. The [browser receipt](browser.json) pins actual WASM/JS
bytes built with wasm-bindgen0.2.105; no host network IO occurs during compilation.
This qualifies the shared browser compiler only. No tag or release asset was
published for this candidate.

Browser reproduction additionally uses Rust's wasm32 target, wasm-bindgen-cli
0.2.105 and Playwright1.62.1 with Chromium. Pass a fourth output argument to the
Python parity script for its exact request/native-result corpus, build
`weft-wasm` with `raw-postgresql-reads`, and generate web bindings with the
matching CLI. Run `table-operation-browser.mjs` with corpus, binding-directory
and report paths. `WEFT_PLAYWRIGHT_MODULE` selects the installed pinned module.
The CI workflow records these concrete commands and retains the browser report.

## Read-frame guard successor — 2026-10-19

The [read-frame receipt](read-frames/receipt.json) pins the successor source,
local wheel, focused/native/browser checks and licensing. The backend now
materializes the bounded read, checks uncompressed visible octets including
lookahead against8MiB and raises SQLSTATE22012 on excess. Hidden native sort
projections preserve order when those Fields are omitted from the public result;
scalar text coercion never substitutes for native ordering. The new result
profile is `pg18-read-frame/0.1`. Output cardinality/exact carriers and final JSON
wire size remain separate SDK responsibilities.

Twenty-one focused Rust tests,27 installed CLI/Python comparisons,10 altered
program controls,50 PostgreSQL18.6 assertions and actual Chromium153 parity for
all27 cases pass. Browser checks also refuse one tampered program and two invalid
Unicode transports. Workspace/all-target,11-package module ownership, governed
specification and MIT OR Apache-2.0 wheel audits pass. These reports supersede
the earlier compiler bytes only for this dedicated operation compiler increment;
legacy184-test evidence remains predecessor evidence. UMF independently verifies
its per-call native reader, scoped cursors and compressed-frame refusal. No
compiler result authenticates a principal or admits durable writes, and no0.4.48
release has been published.

## Atomic mutation-returning frames — 2026-10-19

The [mutation-frame receipt](mutation-frames/receipt.json) pins the next compiler
bytes and local wheel. `pg18-write-frame/0.1` measures uncompressed RETURNING
octets and the effect marker inside the existing materialized effect-count
guard. Oversize causes SQLSTATE22012 and native statement rollback, with no
partial frame or write clipping. The host additionally bounds the cumulative
batch wire frame and owns atomic authenticated outcome storage.

Twenty-two focused tests,27 native/Python and Chromium comparisons,10 altered
program controls,50 native PG18.6 assertions,11-package boundary checks,
workspace/all-target/specification and dual-license checks pass. This extends
the dedicated operation compiler only. Native admission, effect-only durable
recovery and sync/async driver behavior have separate UMF SDK qualification.
Generated lifecycle clients, remaining lifecycle/codec/consumer acceptance and
publication still gate the complete Praxis release.

## Finite temporal operation increment

The [temporal evidence](temporal/receipt.json) extends the same dedicated
compiler with exact finite dates and microsecond UTC instants. Temporal source
selection requires explicit native lifecycle mappings; generic raw APIs remain
closed to those carriers. Original lexemes are retained while driver values
normalize without floats, locale, timezone databases or rounding. PostgreSQL
projections use fixed numeric formatting and explicit UTC. Native Python/CLI and
actual Chromium WASM share the Rust implementation and checked artifacts.

The matching UMF component owns finite date/instant/numeric CHECK constraints,
locale-independent expression-tree witnesses, generated Python carriers and
native driver/pool execution. Its synthetic fixture covers source-declared
current timestamps, predicates, alternate ambient timezone/date format, exact
rows, replay/batch and both transaction owners. Neither component completes the
remaining seeds/evolution, private Praxis/managed runtime or publication gates.

The [scalar-domain/clock increment](scalar-domains/receipt.json) pins the exact
UMF operation-schema producer and adds typed transaction-time values plus
source-field enumeration admission in the dedicated operation profile. Generic
raw guards remain. Original allowed values and range/width/length qualifiers are
checked without floats; the matching UMF candidate emits native CHECKs and typed
client validation. Native compiler, Python and actual Chromium WASM agree on the
closed corpus. The separate UMF150-assertion native fixture proves generated
execution and native domain enforcement; compiler acceptance alone grants no
native authority. Full seeds/evolution, consumer and publication gates remain.
