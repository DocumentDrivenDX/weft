---
ddx:
  id: STP-001
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

# STP-001: resolve-logical-query

## Story Reference

[[US-001-resolve-logical-query]], [[TD-001-resolve-logical-query]], [[SD-001]], TP-001.

## Scope and Objective

Prove this P0 journey under its selected dialect/model/backend/host versions.
Every covering test must cite its AC as `@covers US-001-ACm`. All tests below are
planned; no compiler implementation or executed coverage exists at bootstrap.

## Acceptance Criteria Test Mapping

| AC | Planned test | Observable assertion | Layer | State |
| --- | --- | --- | --- | --- |
| US-001-AC1 | `join_group_sum` | Identity-qualified plan; duplicate join rows contribute to SUM. | Rust + host/contract | passed in qualified compiler scope |
| US-001-AC2 | `module_name_resolution` | Distinct identities; no fallback to first match. | Rust + host/contract | passed in qualified compiler scope |
| US-001-AC3 | `dialect_refusals` | Unsupported nodes never pass from parser to lowering. | Rust + host/contract | passed in qualified compiler scope |
| US-001-AC4 | `relational_exactness` | Exact independent result bags and nullable empty SUM. | Rust + host/contract | passed in qualified compiler scope |

## Executable Proof and Data

Planned implementation tests: `tests/resolve-logical-query/`; shared language-neutral fixtures
in `docs/helix/03-test/fixtures/`. The existing `bun run specs:check` validates
spec/fixture integrity only; it does not execute this story or certify coverage.
Add runnable Rust/host/native commands with pinned versions before implementation.

## Edge Cases and Build Handoff

100% of applicable positive/refusal/boundary cases must pass. Do not count a
skipped platform test as a pass; missing native/host access blocks qualification.
Expected results are authored independently of emitted SQL/compiler code.
Require tests-before-code, retained logs/input hashes, and zero phantom claims.

## Current acceptance determination

The [30-criterion matrix](../../04-build/evidence/B-007-acceptance-matrix.json) records this story's passing P0 assertions, covering evidence and exact limits. Both current-source workspace compositions pass; qualified native compiler and actual Python/browser correspondence are separately retained. Historical component observations remain scoped to their recorded revisions. Distribution and production execution are separately qualified; the final CI/review/merge gates remain open.
