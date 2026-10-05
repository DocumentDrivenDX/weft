---
ddx:
  id: STP-007
  type: story-test-plan
  activity: test
  status: draft
  authoring:
    home: repo
  links:
    - id: CONTRACT-004
      kind: informed_by
---

# STP-007: Application reads

## Story Reference

US-007, FEAT-005, CONTRACT-004 and TP-001.

## Acceptance Criteria Test Mapping

| Criterion | Planned test | Assertion | State |
|---|---|---|---|
| US-007-AC1 | `entity_members` | Complete declared-member order and exact absent/null/value/list/structured descriptors. | planned |
| US-007-AC2 | `keyset_pages` | Stable ascending complete-key order, exact resume comparison and enforced bound; no silent cut. | planned |
| US-007-AC3 | `exact_count` | COUNT(*) preserves duplicates; empty global count 0, grouped empty output empty. | planned |
| US-007-AC4 | `related_reads` | Existential key filtering, bounded related keys with exact truncation and inverse resolution. | planned |
| US-007-AC5 | `typed_parameters` | Marker values match resolved UMF types and remain bound data; literal quoting/exactness is normative. | planned |
| US-007-AC6 | `recognizable_profiles` | Named subsets have independent positive/refusal fixtures and identify excluded constructs without changing valid syntax. | planned |

## Layers and Gate

Parser/model/IR and independent bag/presence/key-order oracle first; same requests through Python/browser; native PostgreSQL and Databricks bindings last. Include compound keys, exact numeric extremes, Unicode/trailing-space ordering, duplicate edges, inverse names, missing/explicit-null/empty-list distinctions, cyclic structure descriptors, COUNT over empty/duplicate inputs, injection strings and wrong parameter types. Unavailable production binding or native test access blocks qualification, never counts as pass.
