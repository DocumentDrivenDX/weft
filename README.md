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
