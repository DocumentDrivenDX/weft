# Declared native table read custody candidate

Unpublished successor0.4.47 candidate over origin/main e2d83d8a. Original
core0.10.0/umf.tables0.1.0 schema is pinned to UMF e48c0ba96674e7b20cf3001e3d90c368b7ced5ad
in `spec/upstream/umf-tables-0.1.0.source.json`. All fixtures are synthetic;
no private Praxis source, SQL, names or data are copied.

The explicit `table-lifecycle/0.1` compiler profile emits `weft.table-read/0.1`
with original source/mapping custody and unresolved native host obligations.
Selected policy meaning cannot disappear into an older raw-read profile.
Compilation deliberately does not authorize database execution.

Verification commands (run from the repository root with pinned Rust1.90.0):

```sh
cargo test -p weft-postgresql --test table_lifecycle --offline
cargo test -p weft-postgresql --lib --offline
cargo check -p weft-python --features raw-postgresql-reads --offline
cargo check -p weft-wasm --target wasm32-unknown-unknown --features raw-postgresql-reads --offline
bun scripts/module-boundaries.ts
bun scripts/check-specs.ts
```

Ten focused integration tests cover original bytes, explicit profile/program
version, keyset and count custody, policy-source pin changes, complete inventory,
primary Keys, FK pairs and target Key identity, opaque command/extension refusal,
forged contexts/mappings, and retention of unselected module content. The prior52d0926d full PostgreSQL library regression passed184tests; the decimal-domain regression took over
one minute and was allowed to finish. Native Python and WASM target compilation
pass; WASM compilation is not browser execution evidence. Module ownership checks
pass for11packages. Source license audit passes with dual-license headers and
package metadata.

The installed native Python/CLI comparison is reproducible using the candidate
wheel, the `compile_raw_read` example and
`tests/raw-postgresql/table-lifecycle-bindings-check.py`. Its [machine receipt](receipt.json) records eighteen passing installed-Python/CLI
comparisons in [bindings.json](bindings.json), including four accepted and fourteen
refused requests. The local wheel hash and exact source hashes are retained. No connection or data execution is used.

Remaining gates: typed command declarations and mutation lowering, date/time
native codecs, generated module operations, native ledger/current-role and
privilege admission for data access, cursor generation/principal binding,
seed/evolution migration support, actual browser parity, Praxis/Lakebase
acceptance and immutable publication. Existing UMF client-generation refusal
remains. This component does not establish full Praxis support or a release.


A final negative control found that52d0926d structurally accepted an unknown
selected payload member because the original UMF schema deliberately preserves
unknown members. The successor derives a private closed interpretation schema
from those exact pinned schema bytes; source and upstream schema remain unchanged.
Eight selected member locations now refuse in both native interfaces, while a
focused Core test preserves arbitrary declared CoreLiteral map keys. The final
wheel/CLI comparison, focused integration tests, WASM target check and licensing
were rerun. No native execution authority was available in the precursor either.
