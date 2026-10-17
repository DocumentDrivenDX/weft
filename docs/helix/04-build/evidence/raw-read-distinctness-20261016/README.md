# Raw PostgreSQL null-safe distinctness — 2026-10-16

WR-A13 compiler increment: the PostgreSQL raw-read AST lowerer admits
`IS DISTINCT FROM` and `IS NOT DISTINCT FROM` only when both operands pass the
existing scalar/group guard and have exactly equal declared carriers. A NULL
literal in either operand lowers to IS NULL / IS NOT NULL after validating
the other operand; two NULL literals lower to TRUE / FALSE. Nested NULL is
recognized. The general scalar profile is unchanged. It emits
native PostgreSQL operators, retaining parameter casts, source SQL, bindings,
obligations and covering structural digest. Existing expressions are unchanged.

Pure Rust raw-read regressions: 7 passed, 0 failed with Rust 1.90.0. Tests cover
both operators, nullable/non-null decimal/string and nullable integer parameters,
exact literals, nullable column comparisons, carrier mismatch, NULL literals on either side and both sides, unsupported
functions and ungrouped HAVING refusal (also against a NULL literal). Parameters are declarations; this
packet does not execute runtime null values or claim native truth-table proof.
NULL literal support is specific to these null-safe predicates; other scalar
operations retain their prior admission rules. No temporal, Spark/Delta, engine-host or performance
qualification follows. Native E2E and Python/WASM replay remain required.

Command:
```
CARGO_HOME=/private/tmp/umf-praxis-joins/cargo-home CARGO_TARGET_DIR=/private/tmp/sdk-weft-complete-target cargo test -p weft-postgresql raw_read::raw_read_tests --lib
```
The retained log includes pre-existing warnings. An initial test-harness attempt
incorrectly added a runtime `value` member to the declaration-only parameter ABI;
it failed admission with `invalid read request`. Removing that invalid member
corrected the fixture; no admission rule was weakened.

Final execution: the exact new native compiler wheel and SDK44c44d61 execute22 real PostgreSQL18.6 compiler/read/catalog/Key tests with no skips; owned server stopped. Generated catalog predicates compare nullable joined names with None, equal and unequal strings, and a NULL literal under caller-role RLS. Actual Chromium151.0.7922.34 compares51 complete Python/WASM programs, including four new null-safe cases. All11 Cargo modules and explicit source boundaries pass. Initial WASM invocation requested an unavailable crate feature; initial TypeScript boundary invocation omitted required WEFT_CARGO. Both failed before their intended checks; corrected commands and logs are retained. No temporal/managed/engine-wide qualification follows.
