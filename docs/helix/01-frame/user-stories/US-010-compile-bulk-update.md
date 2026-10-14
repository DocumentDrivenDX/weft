---
ddx:
  id: US-010
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

# US-010: Compile bulk UPDATE to registered UMF actions

**Feature:** FEAT-001. **Feature Requirements:** PAP-WFT-01–03. **PRD Requirements:** FR-23, FR-24, FR-25. **Priority:** First bulk UPDATE slice. **Status:** Draft.

## Story

As a application developer, I want to compile one logical UPDATE into an inert portable action plan, so that I can queue the same logical mutation without knowing Truss physical tables.

## Context

This is a bounded part of the owner-selected portable-action proof. Existing action/core meaning is authoritative; compilation, selection and native execution remain separate. Governing contracts are CONTRACT-009, CONTRACT-001, CONTRACT-003; upstream owner dependencies are pinned explicitly at build/admission.

## Walkthrough

1. Select the original model/action/plan and exact profiles.
2. Request this story's operation through its owned public boundary.
3. Inspect the retained result or source-qualified refusal.
4. Verify the appropriate portable or native evidence before claiming support.

## Acceptance Criteria

- **US-010-AC1:** Given the registered set-active action and UPDATE Customer SET active=FALSE WHERE last_seen<:cutoff, when compiled, then output contains the exact UMF action/model pins, false assignment, typed lt selection and complete source provenance without target discovery or job submission.
- **US-010-AC2:** Given ambiguous names, missing/unused markers, marker reuse across different Fields or different-field action inputs, when compiled, then resolution/type/action correspondence refuses before any plan is emitted.
- **US-010-AC3:** Given missing WHERE, joins, arithmetic, nullable/null SET, key changes, handlers or unknown selected semantics, when compiled, then version-qualified refusal preserves the original source/model and does not fall back to read compilation.
- **US-010-AC4:** Given boolean/integer/string predicates with AND/OR parentheses and exact large-integer or Unicode values, when lowered to selection AST, then operator/precedence/dependency order agrees with an independently implemented evaluator; invalid stored-value behavior remains a backend obligation.
- **US-010-AC5:** Given declared SQL/model/node/depth/assignment bounds, when exceeded, then deterministic bounded refusal produces no partial compiled plan; existing read fixtures remain unchanged.

- **US-010-AC6:** Given original backend mappings and owner-prepared action input, when selection lowers and executes on independent native fixtures, then exact Key/identity/version decoding, numeric identity order and maxItems+1 overflow agree with the oracle; missing/null/malformed predicate values including masked nonmatching/AND-OR rows refuse in the admitted authorized view before filtering.

## Edge Cases

Relevant unknown meaning and mismatched pins refuse; missing native evidence cannot be replaced by shape validity. Uncertain commits require original reconciliation. Empty/no-op work never fabricates business changes.

## Test Scenarios

| Scenario | Criterion | State/action | Expected result |
| --- | --- | --- | --- |
| update_plan | US-010-AC1 | Owner fixture cases keyed by this test name; exact named negative mutation and expected outcome in the shared corpus | output contains the exact UMF action/model pins, false assignment, typed lt selection and complete source provenance without target discovery or job submission. |
| name_and_input_resolution | US-010-AC2 | Owner fixture cases keyed by this test name; exact named negative mutation and expected outcome in the shared corpus | resolution/type/action correspondence refuses before any plan is emitted. |
| bounded_syntax | US-010-AC3 | Owner fixture cases keyed by this test name; exact named negative mutation and expected outcome in the shared corpus | version-qualified refusal preserves the original source/model and does not fall back to read compilation. |
| predicate_exactness | US-010-AC4 | Owner fixture cases keyed by this test name; exact named negative mutation and expected outcome in the shared corpus | operator/precedence/dependency order agrees with an independently implemented evaluator; invalid stored-value behavior remains a backend obligation. |
| compiler_limits | US-010-AC5 | Owner fixture cases keyed by this test name; exact named negative mutation and expected outcome in the shared corpus | deterministic bounded refusal produces no partial compiled plan; existing read fixtures remain unchanged. |

## Dependencies

UMF US-902 / CONTRACT-902 and shared original core/actions corpus; registry supplied explicitly. Parent solution design SD-001; contracts CONTRACT-009, CONTRACT-001, CONTRACT-003.

## Out of Scope

Live predicates, nullable/null mutation semantics, computed assignments, joins, key edits, whole-job atomicity, external business effects, compensation and a second native backend. Historical acceptance totals do not include this story.
