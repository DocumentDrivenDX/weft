# B-007 conformance commands

Run from the repository root. Python receipt checks require Python 3.10 or later and use the standard library:

```sh
python3 tests/qualify-and-evolve/check-retained-evidence.py
```

This replays eight saved-evidence/control components and verifies hashes referenced
by the 30-criterion matrix and candidate support inventory. It writes its own
summary under `docs/helix/04-build/evidence/B-007-retained-evidence-replay/`.
It does not run engines or prove release qualification.

Rust 1.90.0, locked dependencies and Proptest 1.11.0 execute live compiler checks:

```sh
cargo test -p weft-core --test qualification-properties --locked --offline -- --nocapture
cargo test -p weft-core --test qualification-resources --locked --offline -- --nocapture
cargo test -p weft-core --test compile-envelope --locked --offline
cargo test --workspace --all-features --locked --offline
```

Use a dedicated Cargo target directory per source checkout/build configuration.
A filtered command that executes zero tests is not evidence. Property generators
pin seeds/counts; shrinking failures persist in `property-regressions.txt`.
Mutant failures are retained separately and are not ordinary compiler defects.
The current source-mutation runner is local-toolchain-specific; its absolute
Cargo/toolchain paths must be deliberately configured before another host uses it.

The frontend oracle and relational generator accept `WEFT_FRONTEND_BINARY`.
The former accepts `WEFT_ORACLE_OUTPUT`, the latter `WEFT_RELATIONAL_OUTPUT`.
`tests/compile/reports.py` accepts `WEFT_COMPILE_BINARY` and `WEFT_COMPILE_OUTPUT`;
its fixture SQLite execution is not a substitute for either native target.
Python and browser commands are documented with their actual build features and
artifact hashes in the B-007 preparation/evidence records.

Saved receipt reconciliation proves SQL/parameter/result consistency and checks
recorded authored expectations where named. It cannot create producer provenance,
new native execution, a production policy service or qualified warehouse release.
The release gate still requires the exact support/acceptance audit, critical
branch review and owner decisions recorded in B-007-release-readiness.md.

The decimal-domain native harness compiles every valid precision/scale pair and
executes read-only emitted-SQL owner substitutions on the existing fixture
warehouse. Set `WEFT_ASHLAR_COMPILER` to a freshly built/frozen Databricks example
and `WEFT_ASHLAR_EVIDENCE_OUTPUT` to a fresh output directory; run
`tests/ashlar-databricks/decimal-domains-native.py` with the existing SDK host.
`WEFT_DECIMAL_PREPARE_ONLY=1` compiles without database calls. Frozen producer
source, build/source hashes and exact compressed archives are retained in
B-007-decimal-domains-native. `reconcile-decimal-domains.py` and
`decimal-reconcile-controls.py` replay independently without native access;
both are included in the standard retained-evidence check.
