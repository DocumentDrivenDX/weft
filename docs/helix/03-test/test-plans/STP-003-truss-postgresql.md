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
Every covering test must cite its AC as `@covers US-003-ACm`. All tests below are
planned; no compiler implementation or executed coverage exists at bootstrap.

## Acceptance Criteria Test Mapping

| AC | Planned test | Observable assertion | Layer | State |
| --- | --- | --- | --- | --- |
| US-003-AC1 | `truss_native_corpus` | Native prepared SQL and independent exact result agreement. | Native/backend | planned |
| US-003-AC2 | `truss_mapping_boundaries` | Stable property identities; storage ID is not logical key. | Native/backend | planned |
| US-003-AC3 | `truss_semantic_refusals` | Selected unknown/native unsupported meanings remain explicit. | Native/backend | planned |
| US-003-AC4 | `truss_host_obligations` | No default JSON/float decoding; context obligations observable. | Native/backend | planned |

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
