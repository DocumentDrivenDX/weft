---
ddx:
  id: US-003
  type: user-stories
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-002
      kind: informed_by
---

# US-003: Compile the logical corpus for truss on postgresql

**Feature:** FEAT-002. **Feature Requirements:** BACK-02.
**PRD Requirements:** FR-5. **Priority:** P0. **Status:** Draft.

## Story

As an application developer, I want to compile the logical corpus for Truss on PostgreSQL so that I can query generic storage without writing storage SQL.

## Context and Walkthrough

I supply Weft SQL and pinned UMF modules, select a registered backend, and
inspect the complete artifact or refusal. Backend configuration comes from
owned physical binding metadata. I only execute through a host that can fulfill
reported obligations; compilation does not certify database state or policy.

## Acceptance Criteria

- **US-003-AC1:** Given a pinned Truss storage binding and engine profile, when the same logical query executes, then rows/types/multiplicity match independent expected values.
- **US-003-AC2:** Given catalog property IDs and multiple object types, when scans/joins compile, then exact field mappings and type filters prevent unrelated rows from entering results.
- **US-003-AC3:** Given unknown homes, absent/null distinctions or unsupported stored meanings, when selected, then compilation refuses rather than collapse/coerce them.
- **US-003-AC4:** Given exact large values and driver boundaries, when host execution decodes results, then numeric text survives and preparation/policy/revision obligations are enforced or execution is refused.

## Edge Cases and Test Scenarios

- `truss_native_corpus`: Native prepared SQL and independent exact result agreement.
- `truss_mapping_boundaries`: Stable property identities; storage ID is not logical key.
- `truss_semantic_refusals`: Selected unknown/native unsupported meanings remain explicit.
- `truss_host_obligations`: No default JSON/float decoding; context obligations observable.

## Dependencies

CONTRACT-001–003; FEAT-002; corresponding TD-003 / STP-003.
Backend integration requires the owning layout/profile contracts, not inferred
spike table names. Planned tests do not constitute delivered coverage.

## Out of Scope

Database execution inside the compiler, undisclosed fallback/loss and unsupported
broader SQL. User-visible surfaces are governed by Contracts, not invented here.
