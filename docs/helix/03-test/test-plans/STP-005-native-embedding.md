---
ddx:
  id: STP-005
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

# STP-005: native-embedding

## Story Reference

[[US-005-native-embedding]], [[TD-005-native-embedding]], [[SD-003]], TP-001.

## Scope and Objective

Prove this P0 journey under its selected dialect/model/backend/host versions.
Every covering test must cite its AC as `@covers US-005-ACm`. All tests below are
planned; no compiler implementation or executed coverage exists at bootstrap.

## Acceptance Criteria Test Mapping

| AC | Planned test | Observable assertion | Layer | State |
| --- | --- | --- | --- | --- |
| US-005-AC1 | `python_inprocess` | Direct extension import/call; no Node/Bun/server required. | Rust + host/contract | planned |
| US-005-AC2 | `wasm_browser_parity` | Actual browser instance; identical canonical reports. | Rust + host/contract | planned |
| US-005-AC3 | `artifact_exact_transport` | No binary64 conversion; parameter/result metadata complete. | Rust + host/contract | planned |
| US-005-AC4 | `wrapper_failure_parity` | Thin wrappers never recreate resolver/lowering semantics. | Rust + host/contract | planned |

## Executable Proof and Data

Planned implementation tests: `tests/native-embedding/`; shared language-neutral fixtures
in `docs/helix/03-test/fixtures/`. The existing `bun run specs:check` validates
spec/fixture integrity only; it does not execute this story or certify coverage.
Add runnable Rust/host/native commands with pinned versions before implementation.

## Edge Cases and Build Handoff

100% of applicable positive/refusal/boundary cases must pass. Do not count a
skipped platform test as a pass; missing native/host access blocks qualification.
Expected results are authored independently of emitted SQL/compiler code.
Require tests-before-code, retained logs/input hashes, and zero phantom claims.
