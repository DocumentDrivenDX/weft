---
ddx:
  id: FEAT-006
  type: feature-specification
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.prd
      kind: informed_by
---

# FEAT-006: Security compilation

**Priority:** P0. **Owner:** Weft owner. **Status:** Draft.
**Covered PRD Subsystem(s):** Shared security compiler integration requirements.
**Covered PRD Requirements:** FR-19, FR-20, FR-21, FR-22.
**Cross-Subsystem Rationale:** None; one security-compilation capability.

## Overview

Compile protected logical reads under explicitly supplied UMF security meaning.
CONTRACT-007 owns exact transport, admission and refusal semantics.

## Ideal Future State

A caller can inspect what protected query meaning was accepted, what native
obligations remain and why any unsupported meaning refused before emission.

## Problem Statement

An ordinary compiled query cannot establish policy or current authority. Partial
security admission could hide missing evidence and produce unsafe executable SQL.
The present foundation refuses public activation and must retain this boundary.

## Functional Areas

| Area | Caller job | Responsibility |
| --- | --- | --- |
| Admission | Supply pinned policy/model meaning | Preserve identity and refuse unknown selected meaning |
| Protected lowering | Inspect protected operations | Preserve policy composition and disclosure distinctions |
| Host handoff | Execute with authenticated evidence | Declare mandatory custody/current-authority/release obligations |

## Requirements

- **SEC-01:** The transport shall preserve prior version contracts and exact pinned source meaning.
- **SEC-02:** Admission shall preserve complete policy/ontology semantics or refuse.
- **SEC-03:** Lowering shall retain protected operation and disclosure semantics.
- **SEC-04:** Artifacts shall expose mandatory native host obligations; compiler results cannot assert authentication or current authority.

Reliability target: every selected unsupported security meaning refuses atomically,
with zero partial executable artifacts. Each support claim requires qualified
versions, domains and independently obtained native evidence.

## User Stories

[US-009: Compile a protected read](../user-stories/US-009-security-compilation.md).

## Edge Cases and Error Handling

Stale pins, incomplete populations, unknown selected vocabulary, absent native
custody or unsupported lowering remain identifiable refusals. Simulation facts
cannot become authenticated native authority.

## Success Metrics

100% agreement on an independently authored security corpus before activation;
zero executable outputs for unsupported selected security semantics.

## Constraints and Assumptions

UMF governs source meaning; hosts own authentication, current authority and native
facts. Public activation remains closed during reliability remediation.

## Dependencies

FR-19–22, CONTRACT-007, ordinary logical dialect and backend/host contracts.

## Out of Scope

Authentication services, policy administration and database provisioning. This
feature remains open; B-009 does not deliver complete protected query lowering.
