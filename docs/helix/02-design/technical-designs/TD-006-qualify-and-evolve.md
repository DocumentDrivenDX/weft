---
ddx:
  id: TD-006
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

# TD-006: qualify-and-evolve

**User Story:** [[US-006-qualify-and-evolve]]. **Feature:** FEAT-004.
**Parent Solution:** [[SD-004]].

## Technical Approach

Inherit SD-004's separation of source meaning and physical lowering. Realize
US-006-AC1–AC4 through the shared pure Rust core and its registered backend/host
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

STP-006 names planned failing tests and assertions for every AC. Native profiles,
Python/browser behavior and independent expected values must receive evidence
at their actual layer; mocks/generated SQL never replace native semantic checks.

## Migration, Rollback and Implementation Sequence

No database migrations. Add fixtures/refusal tests first, implement the smallest
story components, execute the mapped evidence, then review conformance. A rejected
new profile/version rolls back by unregistering it; retained models are untouched.
Keep source-language/IR/backend versions separate and refuse stale caches.

## Engine-pinned registration review

B-007 adds explicit `NativeReview` registrations in each backend package, separate
from the retained B-005/B-006 `Candidate` identities. Both use backend version
`0.1.0-native-review`. Truss targets `pg17.9-native-review`, with PostgreSQL 17.9,
UTF8/C and repeatable-read context. Ashlar targets `dbsql2026.39-native-review`,
with the observed Databricks SQL 2026.39 u/r build hashes, ANSI mode and explicit
UTF8_BINARY comparison. These are pure trusted Rust registrations; a model or
binding cannot import them. The caller registers the chosen implementation.

Delegated binding admission, lowering and emission preserve the existing exact
homes, SQL, parameters and result representations. A new host-owned
`truss.nativeProfile` or `ashlar.nativeProfile` obligation requires verification
of the executing engine/settings before checks or user SQL and refuses drift.
It supplements the retained publication/context obligations; compiler admission
does not attest that a database is authorized, conforming or even connected.

These review registrations preserve candidate dispositions, require explicit
candidate opt-in and do not declare qualified evidence yet. Qualification must
join their exact profile, domains, unchanged native-tested SQL and host/compiler
evidence before a separately versioned supported registration is advertised.
Historical candidate manifests and receipts retain their original versions;
no Spark observation is reinterpreted as a warehouse release. Library-registration
checks must prove exact identity/settings, whole-operation refusal for stale
versions/profiles, no default candidate bypass, preserved typed/props lowering,
and retention of the new host obligation alongside existing obligations.
