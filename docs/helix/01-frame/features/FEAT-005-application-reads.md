---
ddx:
  id: FEAT-005
  type: feature-specification
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.prd
      kind: informed_by
---

# FEAT-005: Application reads

**Priority:** P0. **Covered PRD subsystem:** Application reads. **Covered PRD requirements:** FR-13 through FR-18.

## Overview

A host can use Weft for complete, bounded, deterministic entity reads and related summaries. This capability is required by the owner's steering after PR #2, preserving the existing 0.1 contract through a versioned extension.

## Ideal Future State

The same explicitly bounded read has typed member/presence/relationship descriptors on both storage backends and can be recognized by a host from normative subsets.

## Requirements

Whole entities include required, optional, many-valued and structured declared members. Key order and comparison preserve exact numeric/string ordering. Counting preserves bags. Related filtering is existential without row multiplication; related-key projection states its bound and truncation. Parameter values are typed data. Unsupported representation, order, presence or relationship meaning refuses explicitly.

## User Stories

US-007 owns six observable outcomes; STP-007 allocates each to a planned test. CONTRACT-004 governs versioned source/IR/result semantics.

## Dependencies and Constraints

UMF key/relationship/member semantics and accepted storage bindings must agree. Hosts enforce prepared execution and publication/policy contexts. These are separate from frontend language acceptance.

## Out of Scope

Arbitrary nested SQL, unrestricted graph traversal, host privilege fallback, OFFSET paging and implicit loss of absent/null/list semantics.

## Success Metrics

All six US-007 criteria have passing independent frontend, host and native adapter evidence for the released profiles. Existing 0.1 fixtures remain unchanged.
