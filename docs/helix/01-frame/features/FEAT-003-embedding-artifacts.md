---
ddx:
  id: FEAT-003
  type: feature-specification
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.prd
      kind: informed_by
---

# FEAT-003: Artifacts and embedding

**Feature ID:** FEAT-003. **Priority:** P0. **Owner:** Weft owner.
**Covered PRD Subsystem(s):** Artifacts and embedding.
**Covered PRD Requirements:** FR-8, FR-9, FR-10.
**Cross-Subsystem Rationale:** None; one cohesive capability.

## Overview

This capability lets consumers rely on artifacts and embedding as part of Weft's
universal UMF-powered SQL dialect. The PRD governs product scope; exact surface
and version rules belong in CONTRACT-001–003.

## Ideal Future State

A developer can inspect the logical meaning, choose a backend and explain the
result or refusal without depending on hidden storage/coercion assumptions.

## Problem Statement

Without this boundary, target-specific behavior or host conversions can silently
change a query's identity, exact values or observed results.

## Requirements

- **HOST-01:** Consumers receive complete compilation artifacts and exact parameter/result metadata.
- **HOST-02:** Python must call the compiler in-process without JavaScript; browsers must use the same core.
- **HOST-03:** No host boundary may silently convert exact numbers or alter diagnostics.

## User Stories

US-005 govern the observable journeys; their acceptance criteria
own the story-level outcomes and are mapped to planned tests in STP artifacts.

## Edge Cases and Error Handling

Missing meaning, inconsistent pins, unsupported capability, exactness mismatch
and resource exhaustion must remain identifiable. Compilation cannot replace a
refusal with a different target, scope, policy or comparison silently.

## Success Metrics

100% agreement on the declared story corpus; zero hidden substitutions.

## Dependencies and Constraints

UMF core 0.7.0, queryable names derived from supplied modules and pinned physical binding; backend
contracts and actual execution/embedding evidence are separate gates.

## Out of Scope

Unspecified broader SQL, runtime execution engines and production policy/storage
management. The v0.1 limit does not constrain future governed language versions.

## Review Checklist

Draft behavior is traceable; approval and executable compiler evidence remain
required before release. No test coverage or native support is claimed by presence.
