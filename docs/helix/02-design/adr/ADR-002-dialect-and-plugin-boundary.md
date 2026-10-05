---
ddx:
  id: ADR-002
  type: adr
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.architecture
      kind: informed_by
---

# ADR-002: One logical dialect, independently registered backends

**Status:** Proposed. **Date:** 2026-10-05. **Decider:** Project owner.

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
