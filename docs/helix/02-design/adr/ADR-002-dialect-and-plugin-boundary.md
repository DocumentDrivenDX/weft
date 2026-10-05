---
ddx:
  id: ADR-002
  type: adr
  activity: design
  status: accepted
  authoring:
    home: repo
  links:
    - id: weft.architecture
      kind: informed_by
---

# ADR-002: One logical dialect, independently registered backends

**Status:** Accepted for the compiler foundation. **Date:** 2026-10-05. **Decider:** Project owner.

## Context

The owner defines Weft as a universal UMF-powered SQL dialect with pluggable
backends, initially Ashlar/Databricks and Truss/PostgreSQL. A generic switch that
rewrites native SQL or a common subset defined by today's two backends would
make target behavior govern the language.

## Decision

The frontend resolves Weft SQL into identity-qualified typed relational IR.
Backend packages implement the same capability/lowering boundary. Dialect
versions define meaning; manifests declare which meanings a target can honor.
Adding a backend may not edit the frontend or silently change query meaning.

Separate backend identity from SQL dialect identity: two layouts on PostgreSQL
may require different plugins; one backend may eventually support several target
versions. The first delivery accepts one backend per query. Federation and target
fallback are separate future language/execution decisions.

## Alternatives and Consequences

Direct SQL-to-SQL transpilation cannot establish UMF identity or physical access.
Backend-specific source syntax prevents one logical language. A plugin boundary
adds conformance/version work, but preserves source semantics and layout ownership.
Trusted registration belongs to hosts; bindings/manifests cannot load code.

## Validation

A third test backend must compile a typed plan without frontend edits. Both
initial backends must execute the same expected-result corpus, including refusal
and exactness boundaries, against pinned layouts/engines before support is claimed.
No convenience fallback may replace an unsupported operation.


## B-003 decision evidence

The pure Rust associated-type backend trait and explicit registry support typed
0.1/0.2 plans without frontend backend-ID switches. The third fixture plugin has
native schema/version/pin/capability/coverage/emission/refusal tests, independent
SQLite execution of its simple string-projection SQL and real Chromium byte
parity. [B-003 evidence](../../04-build/evidence/B-003-backend-interface.md) records
exact versions, inputs and qualification limits. This accepts the structural
boundary, not production engine/layout compatibility or package distribution.

Rust plugins are linked or explicitly registered by trusted host/build code;
Python and browser wrappers consume that same compiler core. Content cannot load
code. Dynamic native ABI loading and Python/JavaScript callback plugins are not
part of this version. Weft owns logical compilation and backend lower/emit;
Truss/Ashlar owners supply their versioned mappings, physical mechanics and host
execution/native evidence. Backend distribution and public packaging ownership
remain an integration/release decision.
