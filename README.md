# Weft

A universal UMF-powered SQL dialect with pluggable storage backends.

**Input:** Weft SQL + supplied UMF modules. **Output:** SQL for the selected backend, with parameters, result types, decoding instructions and diagnostics.

Initial backends: **Ashlar on Databricks** and **Truss on PostgreSQL**. Weft is a separate compiler project that consumes UMF; storage layouts remain owned by Ashlar and Truss.

Weft implements a Rust compiler with native Python and browser WASM bindings. The versioned 0.1 scalar and 0.2 application-read dialects resolve pinned UMF modules, then lower through separately registered Truss/PostgreSQL and Ashlar/Databricks backends. Unsupported or unknown selected meaning produces an explicit refusal.

Native compiler qualification is scoped to PostgreSQL 17.9 synthetic Truss fixtures and Databricks SQL 2026.39 with the exact build/settings in the [support inventory](docs/helix/04-build/evidence/B-007-support-inventory.json). Python and actual Chromium share complete artifact parity. Database execution, authorization, integrity and buffered publication remain host obligations. All 30 P0 compiler acceptance criteria pass in the recorded scope. Packages have not been released and production storage compatibility is unqualified.

Start at [the specification index](docs/helix/README.md), [PRD](docs/helix/01-frame/prd.md), [language contract](docs/helix/02-design/contracts/CONTRACT-001-weft-sql-and-logical-plan.md) and [test plan](docs/helix/03-test/TP-001-compiler-conformance.md).

## Validate the specifications

```sh
bun install --frozen-lockfile
bun run specs:check
```

This checks document traceability, public JSON schemas and fixture integrity. It does not execute a compiler or database. See the [build plan](docs/helix/04-build/implementation-plan.md) for the implementation gates.

The [0.1 frontend](crates/weft-core/) resolves all 636 specification cases to typed logical plans or explicit refusals. [B-002 evidence](docs/helix/04-build/evidence/B-002-frontend.md) includes independent bag evaluation and browser parity. The [application-read contract](docs/helix/02-design/contracts/CONTRACT-004-application-reads.md) governs complete entities, exact keyset paging, counts, related reads and typed parameters. Executable compiler and evidence commands are in the [conformance guide](tests/qualify-and-evolve/README.md).

## Truss PG18 host bridge

Build the Python extension with `truss-postgresql-pg18` to expose
`weft.compile_truss_postgresql_json(request_json, composition_json)`. This normal
feature accepts host-supplied original storage declarations without the test
fixture registry. Use backend `truss.postgresql.original` version `0.2.0`, target
`truss-pg18.6-live/0.1` and composition version
`weft-truss-pg18-composition/0.1.0`. Candidate opt-in remains required.

Scalar filter/count compilation has native Python evidence. This does not
qualify execution against a live Truss database. The host must enforce the emitted
obligations, exact decoding, authority and `search_path=pg_catalog,pg_temp`;
Truss's public execution integration remains in progress. The target carries no
inherited PG17 native evidence.

The same feature also exports `compile_truss_runtime_json(request_json)` for the
actual Truss runtime mapping. It uses backend `truss.postgresql.runtime` version
`0.3.0`, binding `truss-runtime-binding/0.3` and target
`truss-runtime-pg18.6/0.3`. Truss constructs the binding from its accepted source
bytes and actual catalog IDs; consumers do not construct per-field codec graphs.
This separate adapter uses Weft's application-read frontend and typed IR, then
lowers to Truss's fixed, independently authorized query source. The current
installed consumer demonstrates string/boolean filters, exact integer/decimal
fields and parameters, counts, `alias.*`, optional/null values and read-only
execution on PG18.6. Decimal projections retain original tokens; the native
finite numeric domain is explicitly checked. Independent directed monomorphic
relationships support HAS_RELATED and bounded RELATED_KEYS, forward/inverse,
with authored required string keys. Edge bags and composite key ordering are
preserved. Multi-hop relationships, structured properties and page-key profiles
remain unsupported
in that consumer. All new declarations retain Candidate status.
