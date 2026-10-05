---
ddx:
  id: FEAT-002
  type: feature-specification
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.prd
      kind: informed_by
---

# FEAT-002: Backend compilation

**Feature ID:** FEAT-002. **Priority:** P0. **Owner:** Weft owner.
**Covered PRD Subsystem(s):** Backend compilation.
**Covered PRD Requirements:** FR-4, FR-5, FR-6, FR-7.
**Cross-Subsystem Rationale:** None; one cohesive capability.

## Overview

This capability lets consumers rely on backend compilation as part of Weft's
universal UMF-powered SQL dialect. The PRD governs product scope; exact surface
and version rules belong in CONTRACT-001–003.

## Ideal Future State

A developer can inspect the logical meaning, choose a backend and explain the
result or refusal without depending on hidden storage/coercion assumptions.

## Problem Statement

Without this boundary, target-specific behavior or host conversions can silently
change a query's identity, exact values or observed results.

## Requirements

- **BACK-01:** Hosts register a backend without modifying source-dialect semantics.
- **BACK-02:** The same logical query must lower through separately supplied Truss and Ashlar bindings.
- **BACK-03:** Unsupported target meaning must produce an actionable refusal or explicitly selected candidate.

## User Stories

US-002, US-003, US-004 govern the observable journeys; their acceptance criteria
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
