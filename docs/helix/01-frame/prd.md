---
ddx:
  id: weft.prd
  type: prd
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.vision
      kind: informed_by
---

# Weft product requirements

## Summary

Weft is a universal UMF-powered SQL dialect with pluggable backends. It owns
logical query meaning, resolves names/types against supplied pinned UMF modules, and produces
an inspectable compiled artifact through a chosen backend. Initial backends are
Ashlar on Databricks and Truss on PostgreSQL. Native Python and browser/TypeScript
embedding use one compiler implementation.

Universal means one extensible language over supported UMF meanings and backend
capabilities. It does not mean every query executes on every engine or that
native semantics are interchangeable. The public flow is Weft SQL + UMF modules -> selected backend SQL.
Initial grammar limits define v0.1 delivery,
not a permanent product ceiling. No implementation or support is claimed here.

## Problem and Goals

Storage-specific queries couple applications to table/property layouts and
implicit coercions. The same displayed SQL can hide different equality, decimal,
presence or multiplicity behavior. Engineers need model-aware compilation with
explicit target capabilities and exact host boundaries.

Goals: define one dialect; resolve UMF identity without inference; allow backend
registration independently; preserve exact values; make limits inspectable.

| Metric | v0.1 acceptance target | Measurement |
| --- | --- | --- |
| Declared query semantics | 100% expected-result agreement | Shared native corpus on both selected profiles |
| Unsafe unsupported queries | 100% fail before producing executable output | Authored refusal corpus |
| Numeric transport | Zero binary floating-point coercions | Rust/Python/browser boundary fixtures |
| Plugin independence | Third test backend, zero frontend changes | Registration/conformance test |

Non-goals for v0.1: query execution engine, database drivers, storage/schema
bootstrap, catalog management, mutation/history/feed operations, authentication,
policy decision engine, cost optimizer and cross-backend federation. Longer-term
language additions require separately governed semantics and delivery selection.

## Users and Scope

Primary: application developer embedding logical queries in Python/TypeScript.
Secondary: backend author exposing storage under Weft's defined language.
Operators execute compiled artifacts through backend-specific hosts.

## Requirements

All FR-1–FR-12 below are P0 for the first release. Scope is an explicitly supplied bundle of pinned UMF modules, resolved backend
mappings and one backend per compilation. v0.1 starts
with required single scalar fields, inner equijoins, filters and exact SUM.
No broader SQL/version/backend support follows from the project name.

## Functional Requirements

### Subsystem: Logical dialect and model resolution

- **FR-1:** Define a versioned source SQL dialect independently of target SQL.
- **FR-2:** Resolve logical record/field names to exact UMF identities in an
  integrity-verified model revision; ambiguous, missing or unknown selected meaning must block.
- **FR-3:** Define typed relational operations, bag multiplicity, comparison,
  aggregation and output semantics before choosing physical access paths.

### Subsystem: Backend compilation

- **FR-4:** Register independently supplied backends behind one versioned
  interface; model/binding content must never load executable code.
- **FR-5:** Provide a Truss/PostgreSQL backend using Truss-owned physical binding
  semantics and an explicit supported storage-layout revision.
- **FR-6:** Provide an Ashlar/Databricks backend using Ashlar-owned physical
  binding semantics and an explicit supported layout/publication revision.
- **FR-7:** Negotiate target capabilities and execution obligations; incompatible
  or unproven required semantics must refuse or produce an explicitly selected candidate.

### Subsystem: Artifacts and embedding

- **FR-8:** Return target SQL, ordered typed parameters, result descriptors,
  resolved logical plan, pins, provenance, obligations and diagnostics as one artifact.
- **FR-9:** Preserve exact integer/decimal values at every language boundary and
  identify numeric overflow, unsupported presence and comparison semantics explicitly.
- **FR-10:** Supply an in-process native Python API without a JavaScript runtime
  and a browser/TypeScript WASM API over the same compiler core.

### Subsystem: Conformance and evolution

- **FR-11:** Publish executable language-neutral positive/refusal/boundary
  fixtures with independent expected outcomes and version-qualified backend evidence.
- **FR-12:** Bound untrusted inputs and retain unknown UMF extension content;
  evolve dialect, model participation, logical IR and backend interfaces independently.

## Acceptance Test Sketches

| Requirements | Scenario | Expected outcome |
| --- | --- | --- |
| FR-1–3 | Sales query; colliding display names; absent member; digest mismatch | Typed identity plan or diagnostic; no guessed reference |
| FR-4–7 | Register third backend; run same logical query on Truss and Ashlar | Frontend unchanged; matching results or explicit capability refusal |
| FR-8–10 | Large integer, fixed decimal, empty SUM through Python/browser | Same artifact and exact values; no Node/Bun process |
| FR-11–12 | Unknown extension, hostile manifest, over-limit query | Retained source; bounded refusal; evidence distinct from declarations |

## Technical Context

Rust core/PyO3/WASM is selected for the foundation by ADR-001 after B-001;
dependency/toolchain versions are pinned for that bounded experiment. UMF 0.7.0 is the
initial model profile. Target engine versions and production binding contracts
are mandatory release gates, not assumptions derived from sibling spike evidence.

## Constraints, Assumptions, Dependencies

UMF owns metamodel semantics. Truss and Ashlar own physical layouts and storage
behavior. The host owns preparation, parameter binding, effective authorization,
consistent publication and execution. Compilation cannot certify stored data.
Trusted code registration is explicit and offline; neither model nor manifest
content authorizes fetching or dynamic code execution.

## Risks

Rust may reinterpret UMF differently: require shared validation fixtures.
Databricks may lack an exact selected operation: negotiate refusal, never silently
substitute a function. Backend layouts are unfinished: gate integration on their
owned contracts rather than invent names here. Shipping wheels/WASM has platform
costs: prove both early.

## Open Questions

Q1: What production binding revisions and engine profiles can each owner supply?
Q2: B-001 records passing pinned versions on one platform. Which additional Python/OS/browser versions will be qualified for release?
Q3: Which Python OS/architecture/ABI and browser support matrix ships first?
Q4: What numeric domains/backend aggregate bounds receive initial conformance?
Q5: What license/package names and maintenance commitments should be adopted?

These decisions block their respective delivery gates; this draft defines the
compiler boundary without pretending they are resolved.

## Success Criteria and Review Checklist

Every FR maps to a story and planned observable tests; exact contracts are in
02-design. No release is complete until both native backends and both embedding
paths pass their selected profiles. Draft specifications require review; all
compiler/backend support claims remain unverified until evidence exists.

## Application-read requirements incorporated from PR #2

The owner requested that [the merged discovery input](../00-discover/application-read-requirements-input.md) be accounted for in design and implementation. These are now required outcomes, not optional roadmap suggestions. Preserve Weft SQL 0.1 compatibility and introduce a versioned 0.2 application-read profile with independent fixtures. FEAT-005 and US-007 own this cohesive subsystem; CONTRACT-004 defines its boundary.

| ID | Priority | Required outcome |
|---|---|---|
| FR-13 | P0 | Whole-entity projection has deterministic declared-member order and explicit absent/null/value, list and structured-value descriptors |
| FR-14 | P0 | Bounded deterministic reads support complete-key ascending order, LIMIT and exact ordered key comparisons/keyset continuation |
| FR-15 | P0 | COUNT(*) with GROUP BY returns exact integers; empty global count is zero and empty grouped count has no rows |
| FR-16 | P0 | Relationship filtering by target key, bounded related-key projection with truncation, and inverse traversal are explicit and preserve multiplicity |
| FR-17 | P0 | Source parameters are typed against UMF fields and bound as data; literal grammar and exactness remain normative |
| FR-18 | P0 | Named normative application-read subsets, positive/refusal fixtures and construct diagnostics let hosts recognize supported queries without widening semantics |

Backend requirements include Truss key lookups/property filters/ordered scans and Ashlar gold mappings with host publication positions. Production layout, ordering, presence and relationship guarantees require selected binding evidence. User-requested language behavior must not be simulated by SQL text forwarding or uncertified fixture profiles.
