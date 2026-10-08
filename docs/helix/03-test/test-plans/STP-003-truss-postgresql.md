---
ddx:
  id: STP-003
  type: story-test-plan
  activity: test
  status: draft
  authoring:
    home: repo
  links:
    - id: CONTRACT-001
      kind: informed_by
    - id: CONTRACT-002
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# STP-003: truss-postgresql

## Story Reference

[[US-003-truss-postgresql]], [[TD-003-truss-postgresql]], [[SD-002]], TP-001.

## Scope and Objective

Prove this P0 journey under its selected dialect/model/backend/host versions.
Every covering test must cite its AC as `@covers US-003-ACm`. At bootstrap these tests were planned. Candidate implementation and versioned
execution evidence now exist; the matrix below distinguishes partial evidence
from complete story acceptance.

## Acceptance Criteria Test Mapping

| AC | Planned test | Observable assertion | Layer | State |
| --- | --- | --- | --- | --- |
| US-003-AC1 | `truss_native_corpus` | Native prepared SQL and independent exact result agreement. | Native/backend | partial candidate evidence |
| US-003-AC2 | `truss_mapping_boundaries` | Stable property identities; storage ID is not logical key. | Native/backend | partial candidate evidence |
| US-003-AC3 | `truss_semantic_refusals` | Selected unknown/native unsupported meanings remain explicit. | Native/backend | partial candidate evidence |
| US-003-AC4 | `truss_host_obligations` | No default JSON/float decoding; context obligations observable. | Native/backend | partial candidate evidence |

## Executable Proof and Data

Planned implementation tests: `tests/truss-postgresql/`; shared language-neutral fixtures
in `docs/helix/03-test/fixtures/`. The existing `bun run specs:check` validates
spec/fixture integrity only; it does not execute this story or certify coverage.
Add runnable Rust/host/native commands with pinned versions before implementation.

## Edge Cases and Build Handoff

100% of applicable positive/refusal/boundary cases must pass. Do not count a
skipped platform test as a pass; missing native/host access blocks qualification.
Expected results are authored independently of emitted SQL/compiler code.
Require tests-before-code, retained logs/input hashes, and zero phantom claims.

## B-005 native probe preparation

The independent read-only PostgreSQL probe and its authored exact-output checker
now exist in `tests/truss-postgresql/`. They deliberately assert numeric cursor
order differs from canonical-text order, preserve trailing spaces/UTF8 C ordering,
and distinguish absent JSON, explicit null and empty arrays. They now pass nine engine observations on PostgreSQL 17.9; this does not close any AC. Their engine-only results must precede the
full generated-SQL corpus and role/catalog/decoder execution checks.

Owner integration gates are documented in TD-003. Draft flat layout 0.2 cannot be
confused with the earlier partitioned profile. Record approved binding/layout
hashes and native server/session versions before backend support claims.

## Candidate evidence status (2026-10-07)

The [B-005 execution record](../../04-build/evidence/B-005-native-preparation.md)
records original UMF Record/property/value/presence/comparator/relationship
admission, typed native row and JSONB homes, recursive results, exact numeric
operations and directed relationship queries. The latest composite native matrix
contains 624 PostgreSQL 17.9 executions across uint64, decimal/uint64 and
UTF8-C-string/uint64 key variants. It includes 208 complete-tuple cursor cases,
within-key mixed homes, owner-wide corrupt-data prerequisites and independent
result expectations. Its string controls positively verify adversarial ICU
equivalence before requiring explicit C-collation distinctions. These are
synthetic fixtures against pinned draft realization descriptions.

This is partial evidence for US-003-AC1–AC3, not blanket acceptance. AC4 retains
its own native host transport/policy/revision evidence requirements; the compiler
reports obligations and cannot certify host enforcement or database state.
Optional Unicode scalar roots and seven scalar/recursive whole-entity cuts now
have synthetic native SQL and fresh Python/browser evidence. The test-only host
orchestration suite adds 588 callback cases for preparation, pin/visibility and
publication refusal. Actual psycopg execution adds 252 native cases for these original entity paths,
including publication refusal after injected authority/pin/visibility changes.
Broader scalar/query domains, multiple recursive roots and full host/story
qualification remain open. Native null is explicitly unsupported by the pinned
profile and requires a separately admitted profile before any positive claim. Truss runtime implementation/adoption is not a
compiler dependency. Production qualification remains separate.
