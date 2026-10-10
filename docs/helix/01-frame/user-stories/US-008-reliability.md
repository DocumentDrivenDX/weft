---
ddx:
  id: US-008
  type: user-stories
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-004
      kind: informed_by
---

# US-008: reliability

**Feature:** FEAT-004. **PRD Requirements:** FR-10, FR-11, FR-12. **Priority:** P0.

## Story

As a compiler maintainer, I want to qualify a revision reproducibly so that consumers can assess its bounded support and failures.

## Context and Walkthrough

Select explicit source/toolchain inputs, execute the qualification pipeline, inspect safe evidence and verify the pushed revision. Missing execution keeps support unknown.

## Acceptance Criteria

- **US-008-AC1:** Given retained historical receipts and a changed revision, when a maintainer qualifies support, then complete input custody and fresh binary/artifact evidence distinguish historical observations from current qualification.
- **US-008-AC2:** Given hostile or malformed input, when CLI or library hosts admit it, then bounded reads and immutable prepared-source custody prevent unbounded allocation, stale parsed meaning and partial artifacts.
- **US-008-AC3:** Given changed package dependencies or private access, when boundaries are checked, then actual forbidden edges/cycles refuse and ownership/construction/integration responsibilities are inspectable.
- **US-008-AC4:** Given a configured toolchain and mutation experiment, when the harness runs, then validated configuration and fail-closed detection distinguish actual killed mutations from missing tests, unapplied edits and optimized Python.
- **US-008-AC5:** Given runner success, failure, timeout or diagnostic outage, when evidence is captured and retrieved, then bounded safe events identify outcomes/loss without exposing raw source, credentials or subprocess output.
- **US-008-AC6:** Given governed requirements and security foundations, when allocation/formal assurance is checked, then omitted requirements/criteria fail and precise bounded properties, witnesses and correspondence remain distinct from deferred native acceptance.
- **US-008-AC7:** Given the final pushed compiler revision, when CI qualifies the ordinary subset, then fresh Python and Chromium artifacts match the full reference artifact corpus and every required job completes successfully at that revision.

## Edge Cases and Test Scenarios

- `current_qualification`: Given retained historical receipts and a changed revision, when a maintainer qualifies support, then complete input custody and fresh binary/artifact evidence distinguish historical observations from current qualification.
- `bounded_host_inputs`: Given hostile or malformed input, when CLI or library hosts admit it, then bounded reads and immutable prepared-source custody prevent unbounded allocation, stale parsed meaning and partial artifacts.
- `module_boundaries`: Given changed package dependencies or private access, when boundaries are checked, then actual forbidden edges/cycles refuse and ownership/construction/integration responsibilities are inspectable.
- `configured_mutations`: Given a configured toolchain and mutation experiment, when the harness runs, then validated configuration and fail-closed detection distinguish actual killed mutations from missing tests, unapplied edits and optimized Python.
- `safe_diagnostics`: Given runner success, failure, timeout or diagnostic outage, when evidence is captured and retrieved, then bounded safe events identify outcomes/loss without exposing raw source, credentials or subprocess output.
- `exhaustive_formal_allocation`: Given governed requirements and security foundations, when allocation/formal assurance is checked, then omitted requirements/criteria fail and precise bounded properties, witnesses and correspondence remain distinct from deferred native acceptance.
- `fresh_host_ci`: Given the final pushed compiler revision, when CI qualifies the ordinary subset, then fresh Python and Chromium artifacts match the full reference artifact corpus and every required job completes successfully at that revision.

## Dependencies

CONTRACT-001–005, architecture, corresponding technical design and story test plan.

## Out of Scope

Complete security/native product delivery, package release and production deployment.
