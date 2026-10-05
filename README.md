# Weft

A universal UMF-powered SQL dialect with pluggable storage backends.

**Input:** Weft SQL + supplied UMF modules. **Output:** SQL for the selected backend, with parameters, result types, decoding instructions and diagnostics.

Initial backends: **Ashlar on Databricks** and **Truss on PostgreSQL**. Weft is a separate compiler project that consumes UMF; storage layouts remain owned by Ashlar and Truss.

This repository contains **HELIX specifications, 636 conformance scenarios and a bounded Rust/Python/browser embedding spike**. It has no released compiler or production backend support. [B-001](spikes/b001/README.md) passes 597 shared-core scenarios on its recorded platform; Rust is selected for the compiler foundation. Weft SQL 0.1 intentionally defines a small exact subset; unsupported meaning must produce an explicit refusal.

Start at [the specification index](docs/helix/README.md), [PRD](docs/helix/01-frame/prd.md), [language contract](docs/helix/02-design/contracts/CONTRACT-001-weft-sql-and-logical-plan.md) and [test plan](docs/helix/03-test/TP-001-compiler-conformance.md).

## Check the bootstrap

```sh
bun install --frozen-lockfile
bun run specs:check
```

This checks document traceability, public JSON schemas and fixture integrity. It does not execute a compiler or database. See the [build plan](docs/helix/04-build/implementation-plan.md) for the implementation gates.
