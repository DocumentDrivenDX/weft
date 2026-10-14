---
ddx:
  id: STP-010
  type: story-test-plan
  activity: test
  status: draft
  authoring:
    home: repo
  links:
    - id: US-010
      kind: informed_by
    - id: TD-010
      kind: informed_by
    - id: SD-001
      kind: informed_by
    - id: CONTRACT-009
      kind: informed_by
    - id: CONTRACT-001
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# STP-010: Compile bulk UPDATE to registered UMF actions

## Story Reference

[US-010](../../01-frame/user-stories/US-010-compile-bulk-update.md), [TD-010](../../02-design/technical-designs/TD-010-compile-bulk-update.md), SD-001.

## Scope and Objective

Prove US-010's original portable or native outcomes under its contracted profile. All rows are planned; no existing historical pass is allocated to new criteria.

## Acceptance Criteria Test Mapping

| Criterion | Planned test | Asserted behavior | Canonical citation | Layer | Planned file |
| --- | --- | --- | --- | --- | --- |
| US-010-AC1 | `update_plan` | output contains the exact UMF action/model pins, false assignment, typed lt selection and complete source provenance without target discovery or job submission. | `@covers US-010-AC1` | compiler-conformance | `tests/mutation/compiler.rs` |
| US-010-AC2 | `name_and_input_resolution` | resolution/type/action correspondence refuses before any plan is emitted. | `@covers US-010-AC2` | compiler-conformance | `tests/mutation/compiler.rs` |
| US-010-AC3 | `bounded_syntax` | version-qualified refusal preserves the original source/model and does not fall back to read compilation. | `@covers US-010-AC3` | compiler-conformance | `tests/mutation/compiler.rs` |
| US-010-AC4 | `predicate_exactness` | operator/precedence/dependency order agrees with an independently implemented evaluator; invalid stored-value behavior remains a backend obligation. | `@covers US-010-AC4` | compiler-conformance | `tests/mutation/compiler.rs` |
| US-010-AC5 | `compiler_limits` | deterministic bounded refusal produces no partial compiled plan; existing read fixtures remain unchanged. | `@covers US-010-AC5` | compiler-conformance | `tests/mutation/compiler.rs` |
| US-010-AC6 | `selection_lowering` | exact Key/identity/version decoding, numeric identity order and maxItems+1 overflow agree with the oracle; missing/null/malformed predicate values including masked nonmatching/AND-OR rows refuse in the admitted authorized view before filtering. | `@covers US-010-AC6` | compiler-conformance | `tests/mutation/lowering.rs` |

## Executable Proof

Primary planned command: `cargo test -p weft-core --test mutation_compile`. Create the target before feature implementation; every test must cite its allocated AC and be capable of failing. Record results under `docs/helix/04-build/evidence/bulk-update/`, with actual source/profile/runtime versions, original owner hashes, negative controls and explicit residuals. Actual Chromium is required for the browser rows; pure/static checks cannot substitute for native or browser outcomes.

## Data and Setup

Original fixture owner/path: UMF `docs/helix/03-test/fixtures/bulk-update/cases.json` and `model.json`, owned by US-902; all consumers pin the originals. Shared language-neutral fixture: three matching Customer identities2/10/11, active true, integer last_seen below100, fourth identity12 above100; batchSize2 yields [2,1]; registered set-active recipe plus original model/action/source/selection pins. Negative controls mutate one pinned input/guard/fence/permission at a time. Runtime rows require isolated owned databases/roles and current source/installer closure; synthetic issuers cannot qualify protected ordinary-person admission.

## Edge Cases and Failure Modes

Empty/no-op, exact bound/overflow, Unicode/large-integer, duplicate/unknown fields, tampering, missing authority, drift and uncertainty must keep their distinct contract outcomes. Native concurrency controls use actual two-connection barriers and independent effect/outcome queries.

## Build Handoff

UMF US-902 / CONTRACT-902 and shared original core/actions corpus; registry supplied explicitly. Create red tests first; implement the contracted boundary; then independently qualify effects/refusals and source correspondence. Done requires every allocated AC's executed cited test and expected outcome, current upstream pins and complete applicable dependency evidence. This plan allocates tests; it does not report executed coverage.

## Native selection component evidence (2026-10-13)

[Current original-admission lowering evidence](../../04-build/evidence/bulk-update/native-selection.md) retains eight focused Rust tests, thirty PostgreSQL17.11 selection/preflight cases and fresh32-case Python/Chromium compiler parity. Native target is owned by weft-postgresql and requires conformance-original for the original fixture adapter. Protected physical admission/guard/materialization/effects and complete family/operator qualification remain open; this receipt does not check off AC6.
