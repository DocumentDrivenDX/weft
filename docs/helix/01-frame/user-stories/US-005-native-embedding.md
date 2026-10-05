---
ddx:
  id: US-005
  type: user-stories
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-003
      kind: informed_by
---

# US-005: Call one compiler in my runtime and inspect its complete result

**Feature:** FEAT-003. **Feature Requirements:** HOST-01–03.
**PRD Requirements:** FR-8, FR-9, FR-10. **Priority:** P0. **Status:** Draft.

## Story

As an Python or TypeScript developer, I want to call one compiler in my runtime and inspect its complete result so that I can embed logical queries without duplicating semantics.

## Context and Walkthrough

I supply Weft SQL and pinned UMF modules, select a registered backend, and
inspect the complete artifact or refusal. Backend configuration comes from
owned physical binding metadata. I only execute through a host that can fulfill
reported obligations; compilation does not certify database state or policy.

## Acceptance Criteria

- **US-005-AC1:** Given the same request, when native Python invokes the compiler in-process, then it returns the same report without launching JavaScript or using a service.
- **US-005-AC2:** Given the same request, when real-browser WASM invokes the compiler, then it matches Rust/Python results without host globals or external requests.
- **US-005-AC3:** Given wide integer/decimal values and empty SUM, when artifacts cross hosts, then exact text, result types, nullability and decoders remain intact.
- **US-005-AC4:** Given a refusal or host obligation, when wrappers return it, then diagnostics, pins, spans and qualification remain unchanged.

## Edge Cases and Test Scenarios

- `python_inprocess`: Direct extension import/call; no Node/Bun/server required.
- `wasm_browser_parity`: Actual browser instance; identical canonical reports.
- `artifact_exact_transport`: No binary64 conversion; parameter/result metadata complete.
- `wrapper_failure_parity`: Thin wrappers never recreate resolver/lowering semantics.

## Dependencies

CONTRACT-001–003; FEAT-003; corresponding TD-005 / STP-005.
Backend integration requires the owning layout/profile contracts, not inferred
spike table names. Planned tests do not constitute delivered coverage.

## Out of Scope

Database execution inside the compiler, undisclosed fallback/loss and unsupported
broader SQL. User-visible surfaces are governed by Contracts, not invented here.
