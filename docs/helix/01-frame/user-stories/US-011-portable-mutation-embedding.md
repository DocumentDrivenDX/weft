---
ddx:
  id: US-011
  type: user-stories
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-001
      kind: informed_by
    - id: weft.prd
      kind: informed_by
---

# US-011: Share mutation plans through Python and browser bindings

**Feature:** FEAT-001. **Feature Requirements:** PAP-WFT-04; supporting PAP-WFT-01–03. **PRD Requirements:** FR-26. **Priority:** First bulk UPDATE slice. **Status:** Draft.

## Story

As a backend author, I want to consume one mutation plan through native Python, browser WASM and independent executor registrations, so that compiler portability has observable exact semantic and transport evidence.

## Context

This is a bounded part of the owner-selected portable-action proof. Existing action/core meaning is authoritative; compilation, selection and native execution remain separate. Governing contracts are CONTRACT-009, CONTRACT-002, CONTRACT-003; upstream owner dependencies are pinned explicitly at build/admission.

## Walkthrough

1. Select the original model/action/plan and exact profiles.
2. Request this story's operation through its owned public boundary.
3. Inspect the retained result or source-qualified refusal.
4. Verify the appropriate portable or native evidence before claiming support.

## Acceptance Criteria

- **US-011-AC1:** Given the mutation corpus, when the installed native Python bridge compiles it, then the original Rust implementation is invoked in-process with no Node/Bun/subprocess compiler semantics.
- **US-011-AC2:** Given the same source/model/action bytes, when Rust, Python and actual Chromium WASM compile, then exact plan/source artifacts and ordered diagnostics agree, including large integer and non-ASCII cases.
- **US-011-AC3:** Given independently registered test executors, when they consume the unchanged selection/plan corpus, then no frontend backend-ID switch is needed and each either agrees with the independent expected effects or emits a specific capability refusal.
- **US-011-AC4:** Given missing native selection/guard/bulk evidence, when reporting capabilities, then compiler parity and synthetic executor agreement do not become Truss/Ashlar native support or authorization claims.
- **US-011-AC5:** Given existing read artifacts and unsupported mutation/profile versions, when bindings run, then reads retain original interfaces/results and mutations refuse unsupported profiles without automatic migration.

## Edge Cases

Relevant unknown meaning and mismatched pins refuse; missing native evidence cannot be replaced by shape validity. Uncertain commits require original reconciliation. Empty/no-op work never fabricates business changes.

## Test Scenarios

| Scenario | Criterion | State/action | Expected result |
| --- | --- | --- | --- |
| python_no_sidecar | US-011-AC1 | Owner fixture cases keyed by this test name; exact named negative mutation and expected outcome in the shared corpus | the original Rust implementation is invoked in-process with no Node/Bun/subprocess compiler semantics. |
| browser_artifact_parity | US-011-AC2 | Owner fixture cases keyed by this test name; exact named negative mutation and expected outcome in the shared corpus | exact plan/source artifacts and ordered diagnostics agree, including large integer and non-ASCII cases. |
| independent_executor | US-011-AC3 | Owner fixture cases keyed by this test name; exact named negative mutation and expected outcome in the shared corpus | no frontend backend-ID switch is needed and each either agrees with the independent expected effects or emits a specific capability refusal. |
| no_execution_claim | US-011-AC4 | Owner fixture cases keyed by this test name; exact named negative mutation and expected outcome in the shared corpus | compiler parity and synthetic executor agreement do not become Truss/Ashlar native support or authorization claims. |
| read_compatibility | US-011-AC5 | Owner fixture cases keyed by this test name; exact named negative mutation and expected outcome in the shared corpus | reads retain original interfaces/results and mutations refuse unsupported profiles without automatic migration. |

## Dependencies

US-010; UMF US-902/903; real Truss execution is owned by Truss US-048. Parent solution design SD-001; contracts CONTRACT-009, CONTRACT-002, CONTRACT-003.

## Out of Scope

Live predicates, nullable/null mutation semantics, computed assignments, joins, key edits, whole-job atomicity, external business effects, compensation and a second native backend. Historical acceptance totals do not include this story.
