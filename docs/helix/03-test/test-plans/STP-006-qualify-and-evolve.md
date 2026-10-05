---
ddx:
  id: STP-006
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

# STP-006: qualify-and-evolve

## Story Reference

[[US-006-qualify-and-evolve]], [[TD-006-qualify-and-evolve]], [[SD-004]], TP-001.

## Scope and Objective

Prove this P0 journey under its selected dialect/model/backend/host versions.
Every covering test must cite its AC as `@covers US-006-ACm`. All tests below are
planned; no compiler implementation or executed coverage exists at bootstrap.

## Acceptance Criteria Test Mapping

| AC | Planned test | Observable assertion | Layer | State |
| --- | --- | --- | --- | --- |
| US-006-AC1 | `support_evidence_audit` | Missing/failed/skipped evidence cannot count as supported. | Rust + host/contract | planned |
| US-006-AC2 | `pin_and_version_guards` | SHA-256 bytes and identity guards; no implicit migrations. | Rust + host/contract | planned |
| US-006-AC3 | `unknown_content_retention` | Original bytes/content recoverable; no opaque-to-known inference. | Rust + host/contract | planned |
| US-006-AC4 | `resource_and_fuzz_guards` | Duplicate keys, bounds, nesting, malformed text and plugin failures. | Rust + host/contract | planned |

## Executable Proof and Data

Planned implementation tests: `tests/qualify-and-evolve/`; shared language-neutral fixtures
in `docs/helix/03-test/fixtures/`. The existing `bun run specs:check` validates
spec/fixture integrity only; it does not execute this story or certify coverage.
Add runnable Rust/host/native commands with pinned versions before implementation.

## Edge Cases and Build Handoff

100% of applicable positive/refusal/boundary cases must pass. Do not count a
skipped platform test as a pass; missing native/host access blocks qualification.
Expected results are authored independently of emitted SQL/compiler code.
Require tests-before-code, retained logs/input hashes, and zero phantom claims.
