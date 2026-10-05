---
ddx:
  id: US-001
  type: user-stories
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-001
      kind: informed_by
---

# US-001: Compile weft sql against supplied umf modules

**Feature:** FEAT-001. **Feature Requirements:** LANG-01–03.
**PRD Requirements:** FR-1, FR-2, FR-3. **Priority:** P0. **Status:** Draft.

## Story

As an application developer, I want to compile Weft SQL against supplied UMF modules so that I can reuse model-aware queries across targets.

## Context and Walkthrough

I supply Weft SQL and pinned UMF modules, select a registered backend, and
inspect the complete artifact or refusal. Backend configuration comes from
owned physical binding metadata. I only execute through a host that can fulfill
reported obligations; compilation does not certify database state or policy.

## Acceptance Criteria

- **US-001-AC1:** Given supplied sales modules, when I compile the example join/grouped SUM, then the typed logical plan identifies exact records/fields and preserves bags.
- **US-001-AC2:** Given two same-named records in separate modules, when I use an unqualified source, then resolution refuses ambiguity; an explicit namespace selects the intended record.
- **US-001-AC3:** Given unsupported syntax or selected field meaning, when compilation runs, then it returns an actionable refusal without executable SQL.
- **US-001-AC4:** Given exact fields, when equality/group/SUM runs, then numeric values, string distinctions and empty aggregate outcomes match the language contract.

## Edge Cases and Test Scenarios

- `join_group_sum`: Identity-qualified plan; duplicate join rows contribute to SUM.
- `module_name_resolution`: Distinct identities; no fallback to first match.
- `dialect_refusals`: Unsupported nodes never pass from parser to lowering.
- `relational_exactness`: Exact independent result bags and nullable empty SUM.

## Dependencies

CONTRACT-001–003; FEAT-001; corresponding TD-001 / STP-001.
Backend integration requires the owning layout/profile contracts, not inferred
spike table names. Planned tests do not constitute delivered coverage.

## Out of Scope

Database execution inside the compiler, undisclosed fallback/loss and unsupported
broader SQL. User-visible surfaces are governed by Contracts, not invented here.
