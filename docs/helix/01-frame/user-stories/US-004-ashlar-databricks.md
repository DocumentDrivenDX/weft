---
ddx:
  id: US-004
  type: user-stories
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-002
      kind: informed_by
---

# US-004: Compile the logical corpus for ashlar on databricks

**Feature:** FEAT-002. **Feature Requirements:** BACK-02.
**PRD Requirements:** FR-6. **Priority:** P0. **Status:** Draft.

## Story

As an analytics developer, I want to compile the logical corpus for Ashlar on Databricks so that I can query the warehouse with the same Weft language.

## Context and Walkthrough

I supply Weft SQL and pinned UMF modules, select a registered backend, and
inspect the complete artifact or refusal. Backend configuration comes from
owned physical binding metadata. I only execute through a host that can fulfill
reported obligations; compilation does not certify database state or policy.

## Acceptance Criteria

- **US-004-AC1:** Given an accepted Ashlar gold binding and pinned target profile, when the same query executes, then rows/types/multiplicity match independent expected values.
- **US-004-AC2:** Given a mapping with isolated records and parallel matches, when compiled, then the chosen layout preserves record identity and bags without invented table conventions.
- **US-004-AC3:** Given decimal overflow or different collation behavior, when the profile cannot honor language meaning, then it refuses or states an exact-or-error obligation without rounding/wrapping/null substitution.
- **US-004-AC4:** Given missing publication/delegation guarantees, when host execution begins, then required obligations prevent a claimed valid read under an inconsistent or broader context.

## Edge Cases and Test Scenarios

- `ashlar_native_corpus`: Exact native corpus comparison; no smaller-engine substitute.
- `ashlar_layout_boundaries`: Explicit accepted layout; no inferred JSONB or edge deduplication.
- `ashlar_numeric_collation`: Native settings, domain boundaries and error outcomes agree.
- `ashlar_publication_obligations`: No mixed-publication/service-principal fallback.

## Dependencies

CONTRACT-001–003; FEAT-002; corresponding TD-004 / STP-004.
Backend integration requires the owning layout/profile contracts, not inferred
spike table names. Planned tests do not constitute delivered coverage.

## Out of Scope

Database execution inside the compiler, undisclosed fallback/loss and unsupported
broader SQL. User-visible surfaces are governed by Contracts, not invented here.
