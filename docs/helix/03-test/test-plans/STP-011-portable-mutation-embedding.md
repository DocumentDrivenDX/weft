---
ddx:
  id: STP-011
  type: story-test-plan
  activity: test
  status: draft
  authoring:
    home: repo
  links:
    - id: US-011
      kind: informed_by
    - id: TD-011
      kind: informed_by
    - id: SD-001
      kind: informed_by
    - id: CONTRACT-009
      kind: informed_by
    - id: CONTRACT-002
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# STP-011: Share mutation plans through Python and browser bindings

## Story Reference

[US-011](../../01-frame/user-stories/US-011-portable-mutation-embedding.md), [TD-011](../../02-design/technical-designs/TD-011-portable-mutation-embedding.md), SD-001.

## Scope and Objective

Prove US-011's original portable or native outcomes under its contracted profile. All rows are planned; no existing historical pass is allocated to new criteria.

## Acceptance Criteria Test Mapping

| Criterion | Planned test | Asserted behavior | Canonical citation | Layer | Planned file |
| --- | --- | --- | --- | --- | --- |
| US-011-AC1 | `python_no_sidecar` | the original Rust implementation is invoked in-process with no Node/Bun/subprocess compiler semantics. | `@covers US-011-AC1` | compiler-conformance | `tests/mutation/python-check.py` |
| US-011-AC2 | `browser_artifact_parity` | exact plan/source artifacts and ordered diagnostics agree, including large integer and non-ASCII cases. | `@covers US-011-AC2` | compiler-conformance | `tests/mutation/browser-check.ts` |
| US-011-AC3 | `independent_executor` | no frontend backend-ID switch is needed and each either agrees with the independent expected effects or emits a specific capability refusal. | `@covers US-011-AC3` | compiler-conformance | `tests/mutation/embedding.rs` |
| US-011-AC4 | `no_execution_claim` | compiler parity and synthetic executor agreement do not become Truss/Ashlar native support or authorization claims. | `@covers US-011-AC4` | compiler-conformance | `tests/mutation/embedding.rs` |
| US-011-AC5 | `read_compatibility` | reads retain original interfaces/results and mutations refuse unsupported profiles without automatic migration. | `@covers US-011-AC5` | compiler-conformance | `tests/mutation/embedding.rs` |

## Executable Proof

Primary planned command: `cargo test -p weft-core --test mutation_embedding`. Create the target before feature implementation; every test must cite its allocated AC and be capable of failing. Record results under `docs/helix/04-build/evidence/bulk-update/`, with actual source/profile/runtime versions, original owner hashes, negative controls and explicit residuals. Actual Chromium is required for the browser rows; pure/static checks cannot substitute for native or browser outcomes.

## Data and Setup

Original fixture owner/path: UMF `docs/helix/03-test/fixtures/bulk-update/cases.json` and `model.json`, owned by US-902; all consumers pin the originals. Shared language-neutral fixture: three matching Customer identities2/10/11, active true, integer last_seen below100, fourth identity12 above100; batchSize2 yields [2,1]; registered set-active recipe plus original model/action/source/selection pins. Negative controls mutate one pinned input/guard/fence/permission at a time. Runtime rows require isolated owned databases/roles and current source/installer closure; synthetic issuers cannot qualify protected ordinary-person admission.

## Edge Cases and Failure Modes

Empty/no-op, exact bound/overflow, Unicode/large-integer, duplicate/unknown fields, tampering, missing authority, drift and uncertainty must keep their distinct contract outcomes. Native concurrency controls use actual two-connection barriers and independent effect/outcome queries.

## Build Handoff

US-010; UMF US-902/903; real Truss execution is owned by Truss US-048. Create red tests first; implement the contracted boundary; then independently qualify effects/refusals and source correspondence. Done requires every allocated AC's executed cited test and expected outcome, current upstream pins and complete applicable dependency evidence. This plan allocates tests; it does not report executed coverage.

Installed Python gate: `python3 tests/mutation/python-check.py` against the installed original Rust wheel; browser gate: `bun tests/mutation/browser-check.ts` launching actual Chromium against the original WASM build. These are planned runners, following existing tests/compile/python-check.py and browser-check.mjs conventions. Both compare owner-authored expected outcomes, not merely each other.
