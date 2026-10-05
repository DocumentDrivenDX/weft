---
ddx:
  id: weft.vision
  type: product-vision
  activity: discover
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.research
      kind: informed_by
---

# Weft product vision

## Mission Statement

Give developers one SQL dialect over UMF models, with pluggable backends that
preserve logical meaning and explain every execution limit.

## Positioning

For engineers embedding queries into Python and TypeScript applications over
Truss operational data and Ashlar warehouse data, Weft is a universal logical SQL
dialect powered by UMF. It replaces separately handwritten PostgreSQL/Databricks
queries with one model-aware language and inspectable backend compilation.

## Vision

A query names logical records and fields, survives a storage-layout change, and
can target a newly registered backend without learning its physical conventions.
Users know which meaning is portable and which capability a target cannot honor.

**North Star:** Every advertised dialect capability has passing model-aware,
version-qualified conformance evidence on each backend claiming it.

## User Experience

A developer selects a UMF revision and physical binding, writes a Customer/Orders
join with grouped totals, and chooses a backend. They inspect SQL, parameters,
result types and obligations. Switching to Ashlar changes physical lowering;
logical names and bag multiplicity keep their defined meaning. A missing target
capability produces an actionable refusal before execution.

## Target Market

| Attribute | Description |
| --- | --- |
| Who | Python/TypeScript engineers building UMF-backed applications with Truss or Ashlar |
| Pain | Query logic repeatedly embeds storage names and target coercion assumptions |
| Current Solution | Separate handwritten PostgreSQL and Databricks SQL |
| Why They Switch | Model-driven reuse plus evidence-backed portability and Python/browser embedding |

## Key Value Propositions

| Value | Benefit |
| --- | --- |
| One logical dialect | Authors keep queries when physical bindings change |
| Pluggable backend contract | Backend authors add targets without forking query semantics |
| Visible semantic limits | Consumers can reject unsafe execution before data changes meaning |

## Success Definition

| Metric | Target / measurement |
| --- | --- |
| Shared query outcomes | 100% exact agreement on the declared cross-backend corpus at each release |
| Silent semantic changes | Zero undisclosed changes in negative/boundary fixtures |
| Embedding parity | Identical canonical reports from Rust, Python and browser WASM |
| Extensibility | At least one independent additional backend without frontend changes |

## Why Now

The owner has selected Truss/PostgreSQL and Ashlar/Databricks as the first targets
and clarified Weft's independent dialect role on 2026-10-05. UMF now exposes core
records, fields and authored metadata; duplicating another query frontend in
each storage project would spread model interpretation across both consumers.
The [foundation review](research.md) identifies reusable parsing and binding tools.
