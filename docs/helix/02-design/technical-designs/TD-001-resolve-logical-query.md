---
ddx:
  id: TD-001
  type: technical-design
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: ADR-001
      kind: informed_by
    - id: CONTRACT-001
      kind: informed_by
    - id: CONTRACT-002
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# TD-001: resolve-logical-query

**User Story:** [[US-001-resolve-logical-query]]. **Feature:** FEAT-001.
**Parent Solution:** [[SD-001]].

## Technical Approach

Inherit SD-001's separation of source meaning and physical lowering. Realize
US-001-AC1–AC4 through the shared pure Rust core and its registered backend/host
boundaries. Contracts 001–003 own exact interfaces and failure semantics.

## Component Changes

Target `crates/weft-core/`, backend integration packages and thin
`crates/weft-python/` / `crates/weft-wasm/` wrappers according to story scope.
Use one resolved plan, explicit capability evidence and exact value carriers.
Before implementation, pin dependency versions and finalize the IR/trait schema
listed in CONTRACT-001/002. Do not expand this story to invent database layouts.

## Interfaces and Security

Apply CONTRACT-003 input/pin/limit guards, CONTRACT-001 meaning and CONTRACT-002
plugin registration. Core has no I/O; plugins are trusted host code. No credentials,
policy decisions or untrusted SQL fragments enter the model-driven frontend.

## Testing

STP-001 names planned failing tests and assertions for every AC. Native profiles,
Python/browser behavior and independent expected values must receive evidence
at their actual layer; mocks/generated SQL never replace native semantic checks.

## Migration, Rollback and Implementation Sequence

No database migrations. Add fixtures/refusal tests first, implement the smallest
story components, execute the mapped evidence, then review conformance. A rejected
new profile/version rolls back by unregistering it; retained models are untouched.
Keep source-language/IR/backend versions separate and refuse stale caches.
