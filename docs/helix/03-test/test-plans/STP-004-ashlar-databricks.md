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
Every covering test must cite its AC as `@covers US-004-ACm`. The table preserves
the bootstrap planning allocations; executed candidate evidence is audited in
the checkpoint below.

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

B-006 passes its owner-authorized candidate compiler gate. The
[acceptance audit](../../04-build/evidence/B-006-acceptance.md) maps all four criteria
to actual native/compiler/host observations with exact profile boundaries. Native
evidence includes 36 optional, 52 relationship, 133 compound and 48 compound
entity/keyset cases, six name/resource controls, 30 actual buffered host cases
and 49 independent phase/transport controls. Final Ashlar Rust tests pass 18;
fresh Python/Chromium/CLI builds match 463 artifacts with the documented
presence-declaration correction. Bootstrap allocations above remain planned
records; the audit is the executed component evidence. B-007 release qualification
remains open. Synthetic authority attestations and the observed warehouse do not
certify production policy or broader engine/platform compatibility.
