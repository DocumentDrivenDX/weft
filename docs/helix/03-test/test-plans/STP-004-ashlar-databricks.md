---
ddx:
  id: STP-004
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

# STP-004: ashlar-databricks

## Story Reference

[[US-004-ashlar-databricks]], [[TD-004-ashlar-databricks]], [[SD-002]], TP-001.

## Scope and Objective

Prove this P0 journey under its selected dialect/model/backend/host versions.
Every covering test must cite its AC as `@covers US-004-ACm`. All tests below are
planned; no compiler implementation or executed coverage exists at bootstrap.

## Acceptance Criteria Test Mapping

| AC | Planned test | Observable assertion | Layer | State |
| --- | --- | --- | --- | --- |
| US-004-AC1 | `ashlar_native_corpus` | Exact native corpus comparison; no smaller-engine substitute. | Native/backend | planned |
| US-004-AC2 | `ashlar_layout_boundaries` | Explicit accepted layout; no inferred JSONB or edge deduplication. | Native/backend | planned |
| US-004-AC3 | `ashlar_numeric_collation` | Native settings, domain boundaries and error outcomes agree. | Native/backend | planned |
| US-004-AC4 | `ashlar_publication_obligations` | No mixed-publication/service-principal fallback. | Native/backend | planned |

## Executable Proof and Data

Planned implementation tests: `tests/ashlar-databricks/`; shared language-neutral fixtures
in `docs/helix/03-test/fixtures/`. The existing `bun run specs:check` validates
spec/fixture integrity only; it does not execute this story or certify coverage.
Add runnable Rust/host/native commands with pinned versions before implementation.

## Edge Cases and Build Handoff

100% of applicable positive/refusal/boundary cases must pass. Do not count a
skipped platform test as a pass; missing native/host access blocks qualification.
Expected results are authored independently of emitted SQL/compiler code.
Require tests-before-code, retained logs/input hashes, and zero phantom claims.

## Execution checkpoint, 2026-10-08

B-006 is in progress. [Native preparation and execution evidence](../../04-build/evidence/B-006-native-preparation.md)
records explicit binding admission and registered 0.1/0.2 required-scalar lowering,
fifteen Rust tests, native scalar/typed-home results, 112 entity/count/keyset cases,
key/domain refusals and aggregate controls. AC1–AC3 have partial native component
evidence; key integrity covers part of AC4. Full story acceptance remains open:
compound values and actual host authorization/publication/pin refusals
remain unfinished. Optional scalar values have 36 native cases and the fresh
Python/Chromium embeddings match all 276 saved full artifacts. Authored
relationship access has 52 native cases (28 results, 24 refusals), covering
canonical/serving edges, props/typed keys, inverse direction, parallel edges and
whole-input endpoint/key/multiplicity guards. Synthetic admin
fixtures and candidate capabilities do not establish production compatibility.
