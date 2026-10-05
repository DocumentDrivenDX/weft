---
ddx:
  id: US-002
  type: user-stories
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-002
      kind: informed_by
---

# US-002: Register a backend against the typed plan interface

**Feature:** FEAT-002. **Feature Requirements:** BACK-01, BACK-03.
**PRD Requirements:** FR-4, FR-7. **Priority:** P0. **Status:** Draft.

## Story

As an backend author, I want to register a backend against the typed plan interface so that I can add storage support without forking Weft SQL.

## Context and Walkthrough

I supply Weft SQL and pinned UMF modules, select a registered backend, and
inspect the complete artifact or refusal. Backend configuration comes from
owned physical binding metadata. I only execute through a host that can fulfill
reported obligations; compilation does not certify database state or policy.

## Acceptance Criteria

- **US-002-AC1:** Given a third synthetic backend, when the host registers it, then the same frontend compiles a plan through its capabilities without source changes.
- **US-002-AC2:** Given an absent/incompatible capability, when compilation requests it, then the query blocks before emission.
- **US-002-AC3:** Given unverified backend semantics, when candidate mode is omitted, then compilation blocks; explicit candidate mode returns visible qualification and obligations.
- **US-002-AC4:** Given hostile mapping data or plugin failure, when lowering runs, then no code is loaded from data and no partial artifact escapes.

## Edge Cases and Test Scenarios

- `third_backend_registration`: No frontend backend-ID switch; explicit registry selection.
- `capability_negotiation`: No fallback; missing manifest/interface versions identify refusal.
- `candidate_qualification`: Candidate cannot masquerade as verified conformance.
- `plugin_trust_boundary`: Injection/version/failure guards; trusted registration only.

## Dependencies

CONTRACT-001–003; FEAT-002; corresponding TD-002 / STP-002.
Backend integration requires the owning layout/profile contracts, not inferred
spike table names. Planned tests do not constitute delivered coverage.

## Out of Scope

Database execution inside the compiler, undisclosed fallback/loss and unsupported
broader SQL. User-visible surfaces are governed by Contracts, not invented here.
