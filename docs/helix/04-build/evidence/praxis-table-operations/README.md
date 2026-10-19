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
remains false. No tag or release asset was published for this candidate.
