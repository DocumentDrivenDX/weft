---
ddx:
  id: weft.research
  type: research-plan
  activity: discover
  status: draft
  authoring:
    home: repo
---

# Weft research and foundation review

## Research Questions

Can one SQL dialect target distinct physical layouts without changing logical
meaning? Can its compiler embed in Python and browsers from one implementation?
Which existing tools provide reusable mechanisms rather than a competing model?

## Findings

| Source, checked 2026-10-05 | Finding | Decision implication |
| --- | --- | --- |
| [Apache Calcite adapters](https://calcite.apache.org/docs/adapter.html), living docs | Separates schemas, logical relational operators, physical conventions and dialect capabilities | Use a resolved logical plan and explicit backend contract; a full JVM optimizer is outside the first embedding proof |
| [sqlparser-rs 0.63.0](https://docs.rs/sqlparser/0.63.0/sqlparser/), Rust parser docs | Parses SQL into syntax trees and records source spans | Candidate parser dependency; parser acceptance alone cannot define Weft's type/meaning support |
| [PyO3 packaging](https://pyo3.rs/main/building-and-distribution.html), living docs | Native Python extensions and wheel packaging through maturin | Prove direct Python import and calls, with no JavaScript runtime |
| [wasm-bindgen guide](https://wasm-bindgen.github.io/wasm-bindgen/), living docs | Rust/JavaScript WebAssembly interoperability | Prove the same compiler in a real browser; wrappers own boundary conversion only |
| [PostgreSQL 17 aggregates](https://www.postgresql.org/docs/17/functions-aggregate.html) | Numeric SUM and empty-group behavior differ by argument domain | Define logical aggregate types; backend capability cannot follow from function spelling |
| [Databricks SUM](https://docs.databricks.com/aws/en/sql/language-manual/functions/sum), living docs | Decimal aggregation is bounded and overflow behavior is configuration-sensitive | Require explicit domain/session obligations and differential boundary cases |
| [Databricks collations](https://docs.databricks.com/aws/en/sql/language-manual/sql-ref-collation), Runtime 16.1+ | Binary UTF-8 comparison is a selectable versioned behavior | Qualify string equality by target profile and actual tests |

## Project Evidence and Gaps

[UMF](https://github.com/DocumentDrivenDX/umf) supplies versioned core records,
fields, facets and relationships with retained native semantics. It is the
metamodel authority, not a SQL language or a Rust reader dependency.
[Truss](https://github.com/DocumentDrivenDX/truss) ADR-002 adopts generic catalog
storage: typed objects, property-id JSONB maps, typed edges and separate retained
data. Its production backend contract/version is still missing.
[Ashlar](https://github.com/DocumentDrivenDX/ashlar) FEAT-002 selects gold Delta
but leaves the shared/per-type/hybrid physical layout open. No final tables may
be inferred. These are design inputs, not proof of Weft integration.

The local TypeScript UMF experiment is useful discovery input only. It neither
supplies Weft implementation nor proves Python embedding, backend extensibility
or native Databricks equivalence. Its fixture layouts must not become production
contracts by copying them here.

## Foundation and Next Evidence

Propose Rust core + PyO3/maturin + browser WebAssembly (WASM); versions and wheel
matrix must be pinned in the first spike. Retain an independent expected-result
corpus. A third synthetic backend must register without changing the frontend.
No benchmark or market superiority is claimed; the observed need is the owner's
shared UMF query dialect across Truss and Ashlar.
